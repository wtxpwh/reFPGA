#![allow(clippy::too_many_arguments)]

use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_entity::{EntityId, EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    db::{IntDb, TileClass, TileClassId},
    dir::{DirH, DirV},
    grid::{BelCoord, CellCoord, ColId, DieId, RowId, TileCoord},
};
use prjcombine_re_xilinx_naming::{db::NamingDb, grid::ExpandedGridNaming};
use prjcombine_versal::{
    chip::{
        BramKind, Chip, CleKind, ColumnKind, FabricKind, HardRowKind, HsrNKind, InterposerKind,
        PsKind,
    },
    defs::{bslots, cslots, tcls, tslots},
    expanded::ExpandedDevice,
};
use std::{
    cmp::max,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct DeviceNaming {
    pub tile_names: BTreeMap<TileCoord, Vec<String>>,
    pub bel_names: BTreeMap<BelCoord, Vec<String>>,
    pub skip_bels: BTreeSet<BelCoord>,
    pub has_dsp_v2: bool,
    pub has_term2: bool,
    pub is_vnoc2_scan_offset: bool,
}

impl DeviceNaming {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        for (tcrd, names) in &self.tile_names {
            let tn = tcrd.to_string(db);
            if names.len() == 1 {
                writeln!(o, "\ttile {tn} = {name};", name = names[0])?;
            } else {
                writeln!(o, "\ttile {tn} = {names};", names = names.iter().join(", "))?;
            }
        }
        for (bcrd, names) in &self.bel_names {
            let bn = bcrd.to_string(db);
            if names.len() == 1 {
                writeln!(o, "\tbel {bn} = {name};", name = names[0])?;
            } else {
                writeln!(o, "\tbel {bn} = {names};", names = names.iter().join(", "))?;
            }
        }
        for &bcrd in &self.skip_bels {
            let bn = bcrd.to_string(db);
            writeln!(o, "\tskip_bel {bn};")?;
        }
        if self.has_dsp_v2 {
            writeln!(o, "\thas_dsp_v2;")?;
        }
        if self.has_term2 {
            writeln!(o, "\thas_term2;")?;
        }
        if self.is_vnoc2_scan_offset {
            writeln!(o, "\tvnoc2_scan_offset;")?;
        }
        Ok(())
    }
}

pub const BUFDIV_LEAF_SWZ_A: [u32; 32] = [
    3, 2, 1, 0, 8, 9, 10, 11, 19, 18, 17, 16, 24, 25, 26, 27, 4, 5, 6, 7, 15, 14, 13, 12, 20, 21,
    22, 23, 31, 30, 29, 28,
];

pub const BUFDIV_LEAF_SWZ_B: [u32; 32] = [
    7, 6, 5, 4, 12, 13, 14, 15, 23, 22, 21, 20, 28, 29, 30, 31, 0, 1, 2, 3, 11, 10, 9, 8, 16, 17,
    18, 19, 27, 26, 25, 24,
];

pub const BUFDIV_LEAF_SWZ_AH: [u32; 32] = [
    35, 34, 33, 32, 40, 41, 42, 43, 51, 50, 49, 48, 56, 57, 58, 59, 36, 37, 38, 39, 47, 46, 45, 44,
    52, 53, 54, 55, 63, 62, 61, 60,
];

pub const BUFDIV_LEAF_SWZ_BH: [u32; 32] = [
    39, 38, 37, 36, 44, 45, 46, 47, 55, 54, 53, 52, 60, 61, 62, 63, 32, 33, 34, 35, 43, 42, 41, 40,
    48, 49, 50, 51, 59, 58, 57, 56,
];

#[derive(Clone, Debug)]
struct BelGrid {
    mirror_square: bool,
    xlut: EntityVec<DieId, EntityPartVec<ColId, i32>>,
    ylut: EntityVec<DieId, EntityPartVec<RowId, i32>>,
}

impl BelGrid {
    #[track_caller]
    fn name(&self, prefix: &str, die: DieId, col: ColId, row: RowId, dx: i32, dy: i32) -> String {
        self.name_mult(prefix, die, col, row, 1, dx, 1, dy)
    }

    #[track_caller]
    fn name_mult(
        &self,
        prefix: &str,
        die: DieId,
        col: ColId,
        row: RowId,
        mx: i32,
        dx: i32,
        my: i32,
        dy: i32,
    ) -> String {
        self.name_manual(
            prefix,
            die,
            (mx * self.xlut[die][col] + dx) as u32,
            (my * self.ylut[die][row] + dy) as u32,
        )
    }

    fn name_manual(&self, prefix: &str, die: DieId, x: u32, y: u32) -> String {
        if self.mirror_square {
            format!("{prefix}_S{die:#}X{x}Y{y}")
        } else {
            format!("{prefix}_X{x}Y{y}")
        }
    }
}

fn make_grid(
    edev: &ExpandedDevice,
    f: impl Fn(TileClassId, &TileClass) -> bool,
    n: (i32, i32),
) -> BelGrid {
    make_grid_complex(edev, f, |_, _, _| n)
}

fn make_grid_gt(
    edev: &ExpandedDevice,
    f: impl Fn(TileClassId, &TileClass) -> bool,
    n: (i32, i32),
) -> BelGrid {
    make_grid_complex_gen(edev, f, |_, _, _| n, true)
}

fn make_grid_complex(
    edev: &ExpandedDevice,
    f: impl Fn(TileClassId, &TileClass) -> bool,
    n: impl Fn(TileClassId, &TileClass, TileCoord) -> (i32, i32),
) -> BelGrid {
    make_grid_complex_gen(edev, f, n, false)
}

fn make_grid_complex_gen(
    edev: &ExpandedDevice,
    f: impl Fn(TileClassId, &TileClass) -> bool,
    n: impl Fn(TileClassId, &TileClass, TileCoord) -> (i32, i32),
    is_gt: bool,
) -> BelGrid {
    if edev.interposer.kind == InterposerKind::MirrorSquare {
        let mut res = BelGrid {
            mirror_square: true,
            xlut: EntityVec::new(),
            ylut: EntityVec::new(),
        };
        for (die, chip) in &edev.chips {
            let mut cols = BTreeMap::new();
            let mut rows = BTreeMap::new();
            for (tcid, tcls) in &edev.db.tile_classes {
                if f(tcid, tcls) {
                    for &tcrd in &edev.tile_index[tcid] {
                        if tcrd.die != die {
                            continue;
                        }
                        let (n_x, n_y) = n(tcid, tcls, tcrd);
                        let col = if is_gt {
                            if chip.col_side(tcrd.col) == DirH::W {
                                chip.col_w()
                            } else {
                                chip.col_e()
                            }
                        } else {
                            tcrd.col
                        };
                        let v_c = cols.entry(col).or_default();
                        *v_c = max(*v_c, n_x);
                        let v_r = rows.entry(tcrd.row).or_default();
                        *v_r = max(*v_r, n_y);
                    }
                }
            }
            let mut xlut = EntityPartVec::new();
            let mut ylut = EntityPartVec::new();
            let mut x = 0;
            for (col, num) in cols {
                xlut.insert(col, x);
                x += num;
            }
            if is_gt && let Some(&x) = xlut.get(chip.col_w()) {
                xlut.insert(edev.col_cfrm[die], x);
            }
            let mut y = 0;
            for (row, num) in rows {
                ylut.insert(row, y);
                y += num;
            }
            res.xlut.push(xlut);
            res.ylut.push(ylut);
        }
        res
    } else {
        let mut cols: EntityVec<_, _> = edev.chips.ids().map(|_| BTreeMap::new()).collect();
        let mut rows = BTreeMap::new();
        for (tcid, tcls) in &edev.db.tile_classes {
            if f(tcid, tcls) {
                for &tcrd in &edev.tile_index[tcid] {
                    let (n_x, n_y) = n(tcid, tcls, tcrd);
                    let chip = edev.chips[tcrd.die];
                    let col = if is_gt {
                        if chip.col_side(tcrd.col) == DirH::W {
                            chip.col_w()
                        } else {
                            chip.col_e()
                        }
                    } else {
                        tcrd.col
                    };
                    let v_c = cols[tcrd.die].entry(col).or_default();
                    *v_c = max(*v_c, n_x);
                    let v_r = rows.entry((tcrd.die, tcrd.row)).or_default();
                    *v_r = max(*v_r, n_y);
                }
            }
        }
        let mut xlut: EntityVec<_, _> = edev.chips.ids().map(|_| EntityPartVec::new()).collect();
        let mut x = 0;
        for die in edev.chips.ids() {
            let mut lx = 0;
            for (&col, &num) in &cols[die] {
                if col >= edev.col_cfrm[die] {
                    continue;
                }
                xlut[die].insert(col, lx);
                lx += num;
            }
            x = max(x, lx);
        }
        for (die, chip) in &edev.chips {
            let mut lx = x;
            for (&col, &num) in &cols[die] {
                if col < edev.col_cfrm[die] {
                    continue;
                }
                xlut[die].insert(col, lx);
                lx += num;
            }
            if is_gt && let Some(&x) = xlut[die].get(chip.col_w()) {
                xlut[die].insert(edev.col_cfrm[die], x);
            }
        }
        let mut y = 0;
        let mut ylut: EntityVec<_, _> = edev.chips.ids().map(|_| EntityPartVec::new()).collect();
        for ((die, row), num) in rows {
            ylut[die].insert(row, y);
            y += num;
        }
        BelGrid {
            mirror_square: false,
            xlut,
            ylut,
        }
    }
}

pub struct ExpandedNamedDevice<'a> {
    pub edev: &'a ExpandedDevice<'a>,
    pub ngrid: ExpandedGridNaming<'a>,
    pub dev_naming: &'a DeviceNaming,
}

pub fn name_device<'a>(
    edev: &'a ExpandedDevice<'a>,
    ndb: &'a NamingDb,
    dev_naming: &'a DeviceNaming,
) -> ExpandedNamedDevice<'a> {
    let mut int_grid = make_grid(edev, |tcid, _| tcid == tcls::INT, (1, 1));
    for (die, &chip) in &edev.chips {
        for col in chip.columns.ids() {
            if chip.col_side(col) == DirH::E {
                let x = int_grid.xlut[die][col - 1];
                int_grid.xlut[die].insert(col, x);
            }
        }
    }
    let mut int_grid_gt = int_grid.clone();
    let max_x = int_grid
        .xlut
        .values()
        .map(|xlut| xlut.values().copied().max().unwrap())
        .max()
        .unwrap();
    for (die, &chip) in &edev.chips {
        int_grid_gt.xlut[die][edev.col_cfrm[die]] = 0;
        int_grid_gt.xlut[die][chip.col_e()] = max_x;
    }
    let bufdiv_grid = make_grid_complex(
        edev,
        |tcid, _| matches!(tcid, tcls::RCLK_CLE | tcls::RCLK_INTF),
        |_, _, tcrd| {
            let chip = edev.chips[tcrd.die];
            let cd = &chip.columns[tcrd.col];
            match cd.kind {
                ColumnKind::Dsp => (1, 64),
                ColumnKind::ContHard => {
                    let hc = chip.get_col_hard(tcrd.col - 1).unwrap();
                    let reg = chip.row_to_reg(tcrd.row);
                    if reg.to_idx() != 0 && hc.regs[reg - 1].is_double() {
                        (1, 32)
                    } else {
                        (0, 0)
                    }
                }
                ColumnKind::Hard => {
                    let hc = chip.get_col_hard(tcrd.col).unwrap();
                    let reg = chip.row_to_reg(tcrd.row);
                    if reg.to_idx() != 0 && hc.regs[reg - 1].is_double() {
                        (1, 32)
                    } else {
                        (1, 64)
                    }
                }
                ColumnKind::ContDsp => (0, 0),
                _ => (1, 32),
            }
        },
    );
    let iri_grid = make_grid_complex(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::INTF_W
                    | tcls::INTF_E
                    | tcls::INTF_W_TERM_GT
                    | tcls::INTF_E_TERM_GT
                    | tcls::INTF_W_HDIO
                    | tcls::INTF_E_HDIO
                    | tcls::INTF_W_HB
                    | tcls::INTF_E_HB
                    | tcls::INTF_W_TERM_PSS
                    | tcls::INTF_W_PSS
                    | tcls::INTF_BLI_CLE_ABC_S
                    | tcls::INTF_BLI_CLE_ABC_N
                    | tcls::INTF_BLI_CLE_D_S
                    | tcls::INTF_BLI_CLE_D_N
            )
        },
        |tcid, _, tcrd| {
            if matches!(
                tcid,
                tcls::INTF_BLI_CLE_ABC_S
                    | tcls::INTF_BLI_CLE_ABC_N
                    | tcls::INTF_BLI_CLE_D_S
                    | tcls::INTF_BLI_CLE_D_N
            ) {
                if tcrd.row.to_idx().is_multiple_of(4) {
                    (1, 16)
                } else {
                    (0, 0)
                }
            } else {
                (1, 4)
            }
        },
    );
    let rclk_dfx_grid = make_grid_complex(
        edev,
        |tcid, _| matches!(tcid, tcls::RCLK_DFX),
        |_, _, tcrd| {
            let chip = edev.chips[tcrd.die];
            if chip.columns[tcrd.col].kind == ColumnKind::Bram(BramKind::OptSplitter) {
                (2, 1)
            } else {
                (1, 1)
            }
        },
    );
    let bli_grid = make_grid(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::BLI_S | tcls::BLI_N | tcls::BLI_CLE_S | tcls::BLI_CLE_N
            )
        },
        (1, 1),
    );
    let slice_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::CLE | tcls::CLE_VR),
        (2, 1),
    );
    let dsp_grid = make_grid(edev, |tcid, _| tcid == tcls::DSP, (1, 1));
    let bram_grid = make_grid(edev, |tcid, _| tcid == tcls::BRAM, (1, 1));
    let uram_grid = make_grid(edev, |tcid, _| matches!(tcid, tcls::URAM), (1, 1));
    let uram_delay_grid = make_grid_complex(
        edev,
        |tcid, _| tcid == tcls::URAM,
        |_, _, tcrd| {
            let reg = edev.row_to_reg(tcrd.row);
            if edev.is_reg_n(tcrd.die, reg) && tcrd.row.to_idx() % 48 == 44 {
                (1, 1)
            } else {
                (0, 0)
            }
        },
    );
    let pcie4_grid = make_grid(edev, |tcid, _| tcid == tcls::PCIE4, (1, 1));
    let pcie5_grid = make_grid(edev, |tcid, _| tcid == tcls::PCIE5, (1, 1));
    let mrmac_grid = make_grid(edev, |tcid, _| tcid == tcls::MRMAC, (1, 1));
    let sdfec_grid = make_grid(edev, |tcid, _| tcid == tcls::SDFEC, (1, 1));
    let dfe_cfc_bot_grid = make_grid_complex(
        edev,
        |tcid, _| tcid == tcls::DFE_CFC,
        |_, _, tcrd| {
            if edev.is_reg_n(tcrd.die, edev.row_to_reg(tcrd.row)) {
                (0, 0)
            } else {
                (1, 1)
            }
        },
    );
    let dfe_cfc_top_grid = make_grid_complex(
        edev,
        |tcid, _| tcid == tcls::DFE_CFC,
        |_, _, tcrd| {
            if edev.is_reg_n(tcrd.die, edev.row_to_reg(tcrd.row)) {
                (1, 1)
            } else {
                (0, 0)
            }
        },
    );
    let dcmac_grid = make_grid(edev, |tcid, _| tcid == tcls::DCMAC, (1, 1));
    let ilkn_grid = make_grid(edev, |tcid, _| tcid == tcls::ILKN, (1, 1));
    let hsc_grid = make_grid(edev, |tcid, _| tcid == tcls::HSC, (1, 1));
    let hdio_grid = make_grid(edev, |tcid, _| tcid == tcls::HDIO, (1, 1));
    let vnoc_grid = make_grid(edev, |tcid, _| tcid == tcls::VNOC, (1, 1));
    let misr_grid = make_grid(edev, |tcid, _| tcid == tcls::MISR, (1, 1));
    let clk_gt_grid = make_grid(
        edev,
        |_, tcls| tcls.bels.contains_id(bslots::GCLK_TAPS_DECODE_GT),
        (1, 1),
    );
    let gty_grid = make_grid_gt(edev, |_, tcls| tcls.bels.contains_id(bslots::GTY), (1, 1));
    let gtyp_grid = make_grid_gt(edev, |_, tcls| tcls.bels.contains_id(bslots::GTYP), (1, 1));
    let gtm_grid = make_grid_gt(edev, |_, tcls| tcls.bels.contains_id(bslots::GTM), (1, 1));
    let gtm2_grid = make_grid_gt(edev, |_, tcls| tcls.bels.contains_id(bslots::GTM2), (1, 1));
    let xpipe_grid = make_grid_gt(edev, |_, tcls| tcls.bels.contains_id(bslots::XPIPE), (1, 1));
    let hnicpipe_grid = make_grid_gt(
        edev,
        |_, tcls| tcls.bels.contains_id(bslots::HNICPIPE),
        (1, 1),
    );
    let rfadc_grid = make_grid(edev, |tcid, _| tcid == tcls::RFADC, (1, 2));
    let rfdac_grid = make_grid(edev, |tcid, _| tcid == tcls::RFDAC, (1, 2));
    let vdu_grid = make_grid(edev, |tcid, _| tcid == tcls::VDU, (1, 1));
    let bfr_b_grid = make_grid(edev, |tcid, _| tcid == tcls::BFR_B, (1, 1));
    let vcu2_grid = make_grid(edev, |tcid, _| tcid == tcls::VCU2, (1, 1));
    let isp2_grid = make_grid(edev, |tcid, _| tcid == tcls::ISP2, (1, 1));
    let ps_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::PS9 | tcls::PSXL | tcls::PSXC),
        (1, 1),
    );
    let xram_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::XRAM | tcls::XRAM_NC),
        (1, 1),
    );

    let aie_grid = make_grid(edev, |_, tcls| tcls.slot == tslots::AIE, (1, 1));
    let aie_core_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::AIE | tcls::AIE_ML | tcls::AIE2PS),
        (1, 1),
    );
    let aie_mem_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::AIE_ML_MEM | tcls::AIE2PS_MEM),
        (1, 1),
    );
    let aie_fifo_grid = make_grid(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::AIE
                    | tcls::AIE_ML
                    | tcls::AIE2PS
                    | tcls::AIE_SHIM
                    | tcls::AIE_ML_SHIM
                    | tcls::AIE_SHIM_PL
                    | tcls::AIE_SHIM_NOC_PL
                    | tcls::AIE_ML_SHIM_PL
                    | tcls::AIE_ML_SHIM_NOC_PL
                    | tcls::AIE2PS_SHIM_NOC_PL
            )
        },
        (1, 1),
    );
    let aie_pl_grid = make_grid(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::AIE_SHIM_PL
                    | tcls::AIE_SHIM_NOC_PL
                    | tcls::AIE_ML_SHIM_PL
                    | tcls::AIE_ML_SHIM_NOC_PL
                    | tcls::AIE2PS_SHIM_NOC_PL
            )
        },
        (1, 1),
    );
    let aie_noc_grid = make_grid(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::AIE_SHIM_NOC_PL | tcls::AIE_ML_SHIM_NOC_PL | tcls::AIE2PS_SHIM_NOC_PL
            )
        },
        (1, 1),
    );
    let aie_test_grid = make_grid(
        edev,
        |tcid, _| {
            matches!(
                tcid,
                tcls::AIE_INTF_TEST
                    | tcls::AIE_ML_INTF_TEST
                    | tcls::AIE2PS_INTF_TEST
                    | tcls::AIE2PS_INTF_TEST_NC
            )
        },
        (1, 1),
    );
    let aie_test_nc_grid = make_grid(
        edev,
        |tcid, _| matches!(tcid, tcls::AIE_INTF_TEST_NC | tcls::AIE_ML_INTF_TEST_NC),
        (1, 1),
    );

    let mut ngrid = ExpandedGridNaming::new(ndb, edev);

    for (tcrd, tile) in edev.tiles() {
        let chip = edev.chips[tcrd.die];
        let reg = edev.row_to_reg(tcrd.row);
        let CellCoord { die, col, row } = tcrd.cell;
        let kind = edev.db.tile_classes[tile.class].name.as_str();
        let ocf = if col < edev.col_cfrm[die] {
            "LOCF"
        } else {
            "ROCF"
        };

        match tile.class {
            tcls::INT => {
                ngrid.name_tile(tcrd, "INT", [int_grid.name("INT", die, col, row, 0, 0)]);
            }
            tcls::RCLK => {
                let lr = if col < edev.col_cfrm[die] { 'L' } else { 'R' };
                let vr = if chip.fabric == FabricKind::Ulvt {
                    "_VR"
                } else {
                    ""
                };
                ngrid.name_tile(
                    tcrd,
                    "RCLK",
                    [int_grid.name(
                        &format!("RCLK_INT_{lr}{vr}_FT"),
                        die,
                        col,
                        if reg.to_idx() % 2 == 1 { row - 1 } else { row },
                        0,
                        0,
                    )],
                );
            }
            tcls::CLE_BC | tcls::CLE_BC_SLL | tcls::CLE_BC_SLL2 => {
                let tk = match tile.class {
                    tcls::CLE_BC => "CLE_BC_CORE",
                    tcls::CLE_BC_SLL => "SLL",
                    tcls::CLE_BC_SLL2 => "SLL2",
                    _ => unreachable!(),
                };
                let bump_cur = col > edev.col_cfrm[die] && chip.cols_vbrk.contains(&(col + 1));
                let bump_prev = matches!(chip.columns[col - 1].kind, ColumnKind::Cle(_))
                    && col - 1 > edev.col_cfrm[die]
                    && chip.cols_vbrk.contains(&(col - 1));
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name(
                        &if bump_prev && !bump_cur {
                            format!("{tk}_1")
                        } else {
                            tk.to_string()
                        },
                        die,
                        if bump_cur { col + 1 } else { col },
                        row,
                        0,
                        0,
                    )],
                );
            }
            tcls::RCLK_CLE => {
                let ColumnKind::Cle(cle_kind) = chip.columns[col].kind else {
                    unreachable!()
                };
                let naming = &if cle_kind == CleKind::Plain {
                    if chip.fabric == FabricKind::Ulvt {
                        format!("{kind}_VR")
                    } else {
                        kind.to_string()
                    }
                } else {
                    format!("{kind}_LAG")
                };
                let ntile = ngrid.name_tile(
                    tcrd,
                    naming,
                    [int_grid.name(
                        if cle_kind == CleKind::Plain {
                            if chip.fabric == FabricKind::Ulvt {
                                "RCLK_CLE_VR_CORE"
                            } else {
                                "RCLK_CLE_CORE"
                            }
                        } else {
                            "RCLK_CLE_LAG_CORE"
                        },
                        die,
                        col - 1,
                        if reg.to_idx() % 2 == 1 { row - 1 } else { row },
                        0,
                        0,
                    )],
                );
                let swz = if cle_kind == CleKind::Plain && chip.fabric != FabricKind::Ulvt {
                    BUFDIV_LEAF_SWZ_A
                } else {
                    BUFDIV_LEAF_SWZ_B
                };
                for (i, dy) in swz.into_iter().enumerate() {
                    ntile.add_bel(
                        bslots::BUFDIV_LEAF[i],
                        bufdiv_grid.name(
                            if chip.fabric == FabricKind::Ulvt {
                                "BUFDIV_LEAF_ULVT"
                            } else {
                                "BUFDIV_LEAF"
                            },
                            die,
                            col,
                            row,
                            0,
                            dy as i32,
                        ),
                    );
                }
            }
            tcls::INTF_W
            | tcls::INTF_W_TERM_GT
            | tcls::INTF_W_HDIO
            | tcls::INTF_W_HB
            | tcls::INTF_W_PSS
            | tcls::INTF_W_TERM_PSS => {
                let bt = if chip.is_reg_n(reg) { 'T' } else { 'B' };
                let name = int_grid.name(
                    &match tile.class {
                        tcls::INTF_W => format!("INTF_{ocf}_{bt}L_TILE"),
                        tcls::INTF_W_TERM_GT => format!("INTF_GT_{bt}L_TILE"),
                        tcls::INTF_W_HDIO => format!("INTF_HDIO_{ocf}_{bt}L_TILE"),
                        tcls::INTF_W_HB => format!("INTF_HB_{ocf}_{bt}L_TILE"),
                        tcls::INTF_W_PSS => format!("INTF_CFRM_{bt}L_TILE"),
                        tcls::INTF_W_TERM_PSS => format!("INTF_PSS_{bt}L_TILE"),
                        _ => unreachable!(),
                    },
                    die,
                    col,
                    row,
                    0,
                    0,
                );
                let ntile = ngrid.name_tile(tcrd, kind, [name]);
                for i in 0..4 {
                    ntile.add_bel(
                        bslots::IRI[i],
                        iri_grid.name("IRI_QUAD", die, col, row, 0, i as i32),
                    );
                }
            }
            tcls::INTF_E | tcls::INTF_E_TERM_GT | tcls::INTF_E_HDIO | tcls::INTF_E_HB => {
                let bt = if chip.is_reg_n(reg) { 'T' } else { 'B' };
                let name = int_grid.name(
                    &match tile.class {
                        tcls::INTF_E => format!("INTF_{ocf}_{bt}R_TILE"),
                        tcls::INTF_E_TERM_GT => format!("INTF_GT_{bt}R_TILE"),
                        tcls::INTF_E_HDIO => format!("INTF_HDIO_{ocf}_{bt}R_TILE"),
                        tcls::INTF_E_HB => format!("INTF_HB_{ocf}_{bt}R_TILE"),
                        _ => unreachable!(),
                    },
                    die,
                    col,
                    row,
                    0,
                    0,
                );
                let ntile = ngrid.name_tile(tcrd, kind, [name]);
                for i in 0..4 {
                    ntile.add_bel(
                        bslots::IRI[i],
                        iri_grid.name("IRI_QUAD", die, col, row, 0, i as i32),
                    );
                }
            }
            tcls::INTF_BLI_CLE_ABC_S
            | tcls::INTF_BLI_CLE_ABC_N
            | tcls::INTF_BLI_CLE_D_S
            | tcls::INTF_BLI_CLE_D_N => {
                let side = if reg.to_idx() == 0 { DirV::S } else { DirV::N };
                let ridx = row.to_idx() % 4;
                let (dy, naming) = match (side, ridx) {
                    (DirV::S, 0) => (12, "INTF_BLI_CLE_D_S"),
                    (DirV::S, 1) => (8, "INTF_BLI_CLE_C_S"),
                    (DirV::S, 2) => (4, "INTF_BLI_CLE_B_S"),
                    (DirV::S, 3) => (0, "INTF_BLI_CLE_A_S"),
                    (DirV::N, 0) => (0, "INTF_BLI_CLE_A_N"),
                    (DirV::N, 1) => (4, "INTF_BLI_CLE_B_N"),
                    (DirV::N, 2) => (8, "INTF_BLI_CLE_C_N"),
                    (DirV::N, 3) => (12, "INTF_BLI_CLE_D_N"),
                    _ => unreachable!(),
                };
                let srow = row - ridx;
                let name = int_grid.name(
                    if side == DirV::S {
                        if matches!(chip.columns[col - 1].kind, ColumnKind::Cle(_))
                            && chip.columns[col - 1].has_bli_s
                            && chip.col_side(col) == DirH::E
                        {
                            "BLI_CLE_BOT_CORE_1"
                        } else {
                            "BLI_CLE_BOT_CORE"
                        }
                    } else {
                        if matches!(chip.columns[col - 1].kind, ColumnKind::Cle(_))
                            && chip.columns[col - 1].has_bli_n
                            && chip.col_side(col) == DirH::E
                        {
                            "BLI_CLE_TOP_CORE_1"
                        } else {
                            "BLI_CLE_TOP_CORE"
                        }
                    },
                    die,
                    col,
                    srow,
                    0,
                    0,
                );
                let ntile = ngrid.name_tile(tcrd, naming, [name]);
                for i in 0..4 {
                    ntile.add_bel(
                        bslots::IRI[i],
                        iri_grid.name("IRI_QUAD", die, col, srow, 0, dy + i as i32),
                    );
                }
            }
            tcls::BLI_CLE_S | tcls::BLI_CLE_N | tcls::BLI_S | tcls::BLI_N => {
                let v = if row.to_idx() == 0 { DirV::S } else { DirV::N };
                let srow = if v == DirV::S { row } else { row - 3 };
                let ntile = if matches!(tile.class, tcls::BLI_CLE_S | tcls::BLI_CLE_N) {
                    let name = int_grid.name(
                        if v == DirV::S {
                            if matches!(chip.columns[col - 1].kind, ColumnKind::Cle(_))
                                && chip.columns[col - 1].has_bli_s
                                && chip.col_side(col) == DirH::E
                            {
                                "BLI_CLE_BOT_CORE_1"
                            } else {
                                "BLI_CLE_BOT_CORE"
                            }
                        } else {
                            if matches!(chip.columns[col - 1].kind, ColumnKind::Cle(_))
                                && chip.columns[col - 1].has_bli_n
                                && chip.col_side(col) == DirH::E
                            {
                                "BLI_CLE_TOP_CORE_1"
                            } else {
                                "BLI_CLE_TOP_CORE"
                            }
                        },
                        die,
                        col,
                        srow,
                        0,
                        0,
                    );
                    ngrid.name_tile(tcrd, kind, [name])
                } else {
                    let k = match chip.columns[col].kind {
                        ColumnKind::Bram(_) => "BRAM",
                        ColumnKind::Uram => "URAM",
                        ColumnKind::Dsp | ColumnKind::ContDsp => "DSP",
                        _ => unreachable!(),
                    };
                    let tkn = format!(
                        "BLI_{k}_{ocf}_{bt}{lr}_TILE",
                        bt = match v {
                            DirV::S => 'B',
                            DirV::N => 'T',
                        },
                        lr = match chip.col_side(col) {
                            DirH::W => 'L',
                            DirH::E => 'R',
                        },
                    );
                    let name = int_grid.name(&tkn, die, col, srow, 0, 0);
                    ngrid.name_tile(tcrd, kind, [name])
                };
                for i in 0..3 {
                    let abc = ['A', 'B', 'C'][i];
                    ntile.add_bel(
                        bslots::BLI_GRP0[i],
                        bli_grid.name(&format!("BLI_{abc}_GRP0"), die, col, row, 0, 0),
                    );
                    ntile.add_bel(
                        bslots::BLI_GRP1[i],
                        bli_grid.name(&format!("BLI_{abc}_GRP1"), die, col, row, 0, 0),
                    );
                    ntile.add_bel(
                        bslots::BLI_GRP2[i],
                        bli_grid.name(&format!("BLI_{abc}_GRP2"), die, col, row, 0, 0),
                    );
                }
                ntile.add_bel(
                    bslots::BLI_GRP4,
                    bli_grid.name("BLI_D_GRP4", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::BLI_GRP5,
                    bli_grid.name("BLI_D_GRP5", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::BLI_GRP6,
                    bli_grid.name("BLI_D_GRP6", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::BLI_GRP7,
                    bli_grid.name("BLI_D_GRP7", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::BLI_TMR,
                    bli_grid.name("BLI_TMR", die, col, row, 0, 0),
                );
            }
            tcls::BLI_MUX | tcls::BLI_MUX_NOC => {
                let v = if row.to_idx() == 0 { DirV::S } else { DirV::N };
                let is_1 = if v == DirV::S {
                    chip.columns[col - 1].has_bli_s && chip.col_side(col) == DirH::E
                } else {
                    chip.columns[col - 1].has_bli_n && chip.col_side(col) == DirH::E
                };
                let tkn = if tile.class == tcls::BLI_MUX_NOC {
                    if is_1 {
                        "BLI_LS_MUX_CORE_1"
                    } else {
                        "BLI_LS_MUX_CORE"
                    }
                } else {
                    if is_1 { "BLI_LS_CORE_1" } else { "BLI_LS_CORE" }
                };
                let name = int_grid.name(tkn, die, col, row, 0, 0);
                ngrid.name_tile(tcrd, kind, [name]);
            }
            tcls::RCLK_INTF => {
                if chip.col_side(col) == DirH::W {
                    let srow = if reg.to_idx() % 2 == 1 { row - 1 } else { row };
                    let (subkind, name, swz, wide) = match chip.columns[col].kind {
                        ColumnKind::ContDsp => (
                            if chip.fabric == FabricKind::Ulvt {
                                "DSP_VR"
                            } else {
                                "DSP"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_DSP_VR_CORE"
                                } else {
                                    "RCLK_DSP_CORE"
                                },
                                die,
                                col - 1,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_BH
                            } else {
                                BUFDIV_LEAF_SWZ_AH
                            },
                            true,
                        ),
                        ColumnKind::Bram(BramKind::Plain) => (
                            if chip.fabric == FabricKind::Ulvt {
                                "BRAM_VR"
                            } else {
                                "BRAM"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                            false,
                        ),
                        ColumnKind::Uram => (
                            if chip.fabric == FabricKind::Ulvt {
                                "URAM_VR"
                            } else {
                                "URAM"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_URAM_VR_CORE"
                                } else {
                                    "RCLK_URAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                            false,
                        ),
                        ColumnKind::Gt => (
                            if chip.fabric == FabricKind::Ulvt {
                                "GT_VR"
                            } else {
                                "GT"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_INTF_TERM_LEFT_VR_CORE"
                                } else {
                                    "RCLK_INTF_TERM_LEFT_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                            false,
                        ),
                        ColumnKind::Cfrm => (
                            if chip.fabric == FabricKind::Ulvt {
                                "CFRM_VR"
                            } else {
                                "CFRM"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_INTF_OPT_VR_CORE"
                                } else {
                                    "RCLK_INTF_OPT_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                            false,
                        ),
                        ColumnKind::ContVNoc => (
                            if chip.fabric == FabricKind::Ulvt {
                                "VNOC_VR"
                            } else {
                                "VNOC"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_INTF_L_VR_CORE"
                                } else {
                                    "RCLK_INTF_L_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            BUFDIV_LEAF_SWZ_B,
                            false,
                        ),
                        ColumnKind::ContHard => {
                            let hc = chip.get_col_hard(col - 1).unwrap();
                            if reg.to_idx().is_multiple_of(2) {
                                if hc.regs[reg] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HDIO_VR"
                                        } else {
                                            "HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HDIO_CORE"
                                            },
                                            die,
                                            col - 1,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_BH
                                        } else {
                                            BUFDIV_LEAF_SWZ_AH
                                        },
                                        true,
                                    )
                                } else {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_VR"
                                        } else {
                                            "HB"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_VR_CORE"
                                            } else {
                                                "RCLK_HB_CORE"
                                            },
                                            die,
                                            col - 1,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_BH
                                        } else {
                                            BUFDIV_LEAF_SWZ_AH
                                        },
                                        true,
                                    )
                                }
                            } else {
                                if hc.regs[reg] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HDIO_VR"
                                        } else {
                                            "HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HDIO_CORE"
                                            },
                                            die,
                                            col - 1,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_BH
                                        } else {
                                            BUFDIV_LEAF_SWZ_AH
                                        },
                                        true,
                                    )
                                } else if hc.regs[reg - 1] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_HDIO_VR"
                                        } else {
                                            "HB_HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HB_HDIO_CORE"
                                            },
                                            die,
                                            col - 1,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        BUFDIV_LEAF_SWZ_BH,
                                        true,
                                    )
                                } else if hc.regs[reg - 1] == HardRowKind::DfeCfc {
                                    (
                                        "SDFEC",
                                        int_grid.name("RCLK_SDFEC_CORE", die, col - 1, srow, 0, 0),
                                        BUFDIV_LEAF_SWZ_BH,
                                        true,
                                    )
                                } else if reg.to_idx() != 0 && hc.regs[reg - 1].is_double() {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_FULL_VR"
                                        } else {
                                            "HB_FULL"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_FULL_R_VR_CORE"
                                            } else {
                                                "RCLK_HB_FULL_R_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        BUFDIV_LEAF_SWZ_B,
                                        false,
                                    )
                                } else {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_VR"
                                        } else {
                                            "HB"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_VR_CORE"
                                            } else {
                                                "RCLK_HB_CORE"
                                            },
                                            die,
                                            col - 1,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_BH
                                        } else {
                                            BUFDIV_LEAF_SWZ_AH
                                        },
                                        true,
                                    )
                                }
                            }
                        }
                        _ => unreachable!(),
                    };
                    let ntile = ngrid.name_tile(
                        tcrd,
                        &format!("{kind}_{side}_{subkind}", side = chip.col_side(col)),
                        [name],
                    );
                    for (i, dy) in swz.into_iter().enumerate() {
                        ntile.add_bel(
                            bslots::BUFDIV_LEAF[i],
                            if wide {
                                bufdiv_grid.name(
                                    if chip.fabric == FabricKind::Ulvt {
                                        "BUFDIV_LEAF_ULVT"
                                    } else {
                                        "BUFDIV_LEAF"
                                    },
                                    die,
                                    col - 1,
                                    row,
                                    0,
                                    dy as i32,
                                )
                            } else {
                                bufdiv_grid.name(
                                    if chip.fabric == FabricKind::Ulvt {
                                        "BUFDIV_LEAF_ULVT"
                                    } else {
                                        "BUFDIV_LEAF"
                                    },
                                    die,
                                    col,
                                    row,
                                    0,
                                    dy as i32,
                                )
                            },
                        );
                    }
                } else {
                    let srow = if reg.to_idx() % 2 == 1 { row - 1 } else { row };
                    let (subkind, name, swz) = match chip.columns[col].kind {
                        ColumnKind::Dsp => (
                            if chip.fabric == FabricKind::Ulvt {
                                "DSP_VR"
                            } else {
                                "DSP"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_DSP_VR_CORE"
                                } else {
                                    "RCLK_DSP_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                        ),
                        ColumnKind::Bram(BramKind::Plain) => (
                            if chip.fabric == FabricKind::Ulvt {
                                "BRAM_VR"
                            } else {
                                "BRAM"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                        ),
                        ColumnKind::Bram(BramKind::SplitterPd) => (
                            if chip.fabric == FabricKind::Ulvt {
                                "BRAM_CLKBUF_VR"
                            } else {
                                "BRAM_CLKBUF"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_CLKBUF_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CLKBUF_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                        ),
                        ColumnKind::Bram(BramKind::Splitter | BramKind::OptSplitter) => {
                            let is_splitter = row.to_idx() >= chip.ps.height()
                                || chip.columns[col].kind == ColumnKind::Bram(BramKind::Splitter);

                            if is_splitter {
                                (
                                    if chip.fabric == FabricKind::Ulvt {
                                        "BRAM_CLKBUF_NOPD_VR"
                                    } else {
                                        "BRAM_CLKBUF_NOPD"
                                    },
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_BRAM_CLKBUF_NOPD_VR_CORE"
                                        } else {
                                            "RCLK_BRAM_CLKBUF_NOPD_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    BUFDIV_LEAF_SWZ_B,
                                )
                            } else {
                                (
                                    if chip.fabric == FabricKind::Ulvt {
                                        "BRAM_VR"
                                    } else {
                                        "BRAM"
                                    },
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_BRAM_VR_CORE"
                                        } else {
                                            "RCLK_BRAM_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    if chip.fabric == FabricKind::Ulvt {
                                        BUFDIV_LEAF_SWZ_B
                                    } else {
                                        BUFDIV_LEAF_SWZ_A
                                    },
                                )
                            }
                        }
                        ColumnKind::Uram => (
                            if chip.fabric == FabricKind::Ulvt {
                                "URAM_VR"
                            } else {
                                "URAM"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_URAM_VR_CORE"
                                } else {
                                    "RCLK_URAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            if chip.fabric == FabricKind::Ulvt {
                                BUFDIV_LEAF_SWZ_B
                            } else {
                                BUFDIV_LEAF_SWZ_A
                            },
                        ),
                        ColumnKind::Gt
                        | ColumnKind::Term
                        | ColumnKind::TermSsit
                        | ColumnKind::HNicX => {
                            if reg.to_idx() == 1 && dev_naming.has_term2 {
                                (
                                    if chip.fabric == FabricKind::Ulvt {
                                        "GT_ALT_VR"
                                    } else {
                                        "GT_ALT"
                                    },
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_INTF_TERM2_RIGHT_VR_CORE"
                                        } else {
                                            "RCLK_INTF_TERM2_RIGHT_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    BUFDIV_LEAF_SWZ_B,
                                )
                            } else {
                                (
                                    if chip.fabric == FabricKind::Ulvt {
                                        "GT_VR"
                                    } else {
                                        "GT"
                                    },
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_INTF_TERM_RIGHT_VR_CORE"
                                        } else {
                                            "RCLK_INTF_TERM_RIGHT_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    if chip.fabric == FabricKind::Ulvt {
                                        BUFDIV_LEAF_SWZ_B
                                    } else {
                                        BUFDIV_LEAF_SWZ_A
                                    },
                                )
                            }
                        }
                        ColumnKind::VNoc | ColumnKind::VNoc2 | ColumnKind::VNoc4 => (
                            if chip.fabric == FabricKind::Ulvt {
                                "VNOC_VR"
                            } else {
                                "VNOC"
                            },
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_INTF_R_VR_CORE"
                                } else {
                                    "RCLK_INTF_R_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            BUFDIV_LEAF_SWZ_B,
                        ),
                        ColumnKind::Hard => {
                            let hc = chip.get_col_hard(col).unwrap();
                            if reg.to_idx().is_multiple_of(2) {
                                if hc.regs[reg] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HDIO_VR"
                                        } else {
                                            "HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HDIO_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_B
                                        } else {
                                            BUFDIV_LEAF_SWZ_A
                                        },
                                    )
                                } else {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_VR"
                                        } else {
                                            "HB"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_VR_CORE"
                                            } else {
                                                "RCLK_HB_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_B
                                        } else {
                                            BUFDIV_LEAF_SWZ_A
                                        },
                                    )
                                }
                            } else {
                                if hc.regs[reg] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HDIO_VR"
                                        } else {
                                            "HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HDIO_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_B
                                        } else {
                                            BUFDIV_LEAF_SWZ_A
                                        },
                                    )
                                } else if hc.regs[reg - 1] == HardRowKind::Hdio {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_HDIO_VR"
                                        } else {
                                            "HB_HDIO"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_HDIO_VR_CORE"
                                            } else {
                                                "RCLK_HB_HDIO_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        BUFDIV_LEAF_SWZ_B,
                                    )
                                } else if hc.regs[reg - 1] == HardRowKind::DfeCfc {
                                    (
                                        "SDFEC",
                                        int_grid.name("RCLK_SDFEC_CORE", die, col, srow, 0, 0),
                                        BUFDIV_LEAF_SWZ_B,
                                    )
                                } else if reg.to_idx() != 0 && hc.regs[reg - 1].is_double() {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_FULL_VR"
                                        } else {
                                            "HB_FULL"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_FULL_L_VR_CORE"
                                            } else {
                                                "RCLK_HB_FULL_L_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        BUFDIV_LEAF_SWZ_B,
                                    )
                                } else {
                                    (
                                        if chip.fabric == FabricKind::Ulvt {
                                            "HB_VR"
                                        } else {
                                            "HB"
                                        },
                                        int_grid.name(
                                            if chip.fabric == FabricKind::Ulvt {
                                                "RCLK_HB_VR_CORE"
                                            } else {
                                                "RCLK_HB_CORE"
                                            },
                                            die,
                                            col,
                                            srow,
                                            0,
                                            0,
                                        ),
                                        if chip.fabric == FabricKind::Ulvt {
                                            BUFDIV_LEAF_SWZ_B
                                        } else {
                                            BUFDIV_LEAF_SWZ_A
                                        },
                                    )
                                }
                            }
                        }
                        _ => unreachable!(),
                    };
                    let ntile = ngrid.name_tile(
                        tcrd,
                        &format!("{kind}_{side}_{subkind}", side = chip.col_side(col)),
                        [name],
                    );
                    for (i, dy) in swz.into_iter().enumerate() {
                        ntile.add_bel(
                            bslots::BUFDIV_LEAF[i],
                            bufdiv_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "BUFDIV_LEAF_ULVT"
                                } else {
                                    "BUFDIV_LEAF"
                                },
                                die,
                                col,
                                row,
                                0,
                                dy as i32,
                            ),
                        );
                    }
                }
            }
            tcls::RCLK_DFX => {
                let srow = if reg.to_idx() % 2 == 1 { row - 1 } else { row };
                let (subkind, name, dx) = if chip.col_side(col) == DirH::W {
                    match chip.columns[col].kind {
                        ColumnKind::ContDsp => (
                            "DSP",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_DSP_VR_CORE"
                                } else {
                                    "RCLK_DSP_CORE"
                                },
                                die,
                                col - 1,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        ColumnKind::Bram(BramKind::Plain) => (
                            "BRAM",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        ColumnKind::Uram => (
                            "URAM",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_URAM_VR_CORE"
                                } else {
                                    "RCLK_URAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        _ => unreachable!(),
                    }
                } else {
                    match chip.columns[col].kind {
                        ColumnKind::Bram(BramKind::Plain) => (
                            "BRAM",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        ColumnKind::Bram(BramKind::SplitterPd) => (
                            "BRAM_CLKBUF",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_BRAM_CLKBUF_VR_CORE"
                                } else {
                                    "RCLK_BRAM_CLKBUF_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        ColumnKind::Bram(BramKind::Splitter | BramKind::OptSplitter) => {
                            let is_opt =
                                chip.columns[col].kind == ColumnKind::Bram(BramKind::OptSplitter);
                            let is_splitter = row.to_idx() >= chip.ps.height() || !is_opt;
                            if is_splitter {
                                (
                                    "BRAM_CLKBUF_NOPD",
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_BRAM_CLKBUF_NOPD_VR_CORE"
                                        } else {
                                            "RCLK_BRAM_CLKBUF_NOPD_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    if is_opt { 1 } else { 0 },
                                )
                            } else {
                                (
                                    "BRAM",
                                    int_grid.name(
                                        if chip.fabric == FabricKind::Ulvt {
                                            "RCLK_BRAM_VR_CORE"
                                        } else {
                                            "RCLK_BRAM_CORE"
                                        },
                                        die,
                                        col,
                                        srow,
                                        0,
                                        0,
                                    ),
                                    0,
                                )
                            }
                        }
                        ColumnKind::Uram => (
                            "URAM",
                            int_grid.name(
                                if chip.fabric == FabricKind::Ulvt {
                                    "RCLK_URAM_VR_CORE"
                                } else {
                                    "RCLK_URAM_CORE"
                                },
                                die,
                                col,
                                srow,
                                0,
                                0,
                            ),
                            0,
                        ),
                        _ => unreachable!(),
                    }
                };
                let vr = if chip.fabric == FabricKind::Ulvt {
                    "_VR"
                } else {
                    ""
                };
                let ntile = ngrid.name_tile(tcrd, &format!("{kind}_{subkind}{vr}"), [name]);
                ntile.add_bel(
                    bslots::RCLK_DFX_TEST,
                    rclk_dfx_grid.name("RCLK", die, col, row, dx, 0),
                );
            }
            tcls::RCLK_HDIO | tcls::RCLK_HDIO_VR | tcls::RCLK_HB_HDIO | tcls::RCLK_HB_HDIO_VR => {
                let srow = if reg.to_idx() % 2 == 1 { row - 1 } else { row };
                let name = int_grid.name(&format!("{kind}_CORE"), die, col - 1, srow, 0, 0);
                ngrid.name_tile(tcrd, kind, [name]);
            }
            tcls::RCLK_SPLITTER
            | tcls::RCLK_SPLITTER_VR
            | tcls::RCLK_SPLITTER_PD
            | tcls::RCLK_SPLITTER_PD_VR => {
                let srow = if reg.to_idx() % 2 == 1 { row - 1 } else { row };
                let name = int_grid.name(
                    match tile.class {
                        tcls::RCLK_SPLITTER => "RCLK_BRAM_CLKBUF_NOPD_CORE",
                        tcls::RCLK_SPLITTER_VR => "RCLK_BRAM_CLKBUF_NOPD_VR_CORE",
                        tcls::RCLK_SPLITTER_PD => "RCLK_BRAM_CLKBUF_CORE",
                        tcls::RCLK_SPLITTER_PD_VR => "RCLK_BRAM_CLKBUF_VR_CORE",
                        _ => unreachable!(),
                    },
                    die,
                    col,
                    srow,
                    0,
                    0,
                );
                ngrid.name_tile(tcrd, kind, [name]);
            }
            tcls::RCLK_SPLITTER_SSIT => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLE | tcls::CLE_VR => {
                if chip.col_side(col) == DirH::W {
                    let tkn = if chip.fabric != FabricKind::Ulvt {
                        "CLE_E_CORE"
                    } else {
                        "CLE_E_VR_CORE"
                    };
                    let name = if col.to_idx() == 28 && chip.ps == PsKind::Ps9Cpm6 {
                        // ?!? fucking... I don't know, I give up
                        int_grid.name(&format!("{tkn}_1"), die, col - 1, row, 0, 0)
                    } else {
                        int_grid.name(tkn, die, col, row, 0, 0)
                    };
                    let ntile = ngrid.name_tile(tcrd, kind, [name]);
                    for i in 0..2 {
                        ntile.add_bel(
                            bslots::SLICE[i],
                            slice_grid.name("SLICE", die, col, row, i as i32, 0),
                        );
                    }
                } else {
                    let tkn = if chip.fabric != FabricKind::Ulvt {
                        "CLE_W_CORE"
                    } else {
                        "CLE_W_VR_CORE"
                    };
                    let ntile =
                        ngrid.name_tile(tcrd, kind, [int_grid.name(tkn, die, col, row, 0, 0)]);
                    for i in 0..2 {
                        ntile.add_bel(
                            bslots::SLICE[i],
                            slice_grid.name("SLICE", die, col, row, i as i32, 0),
                        );
                    }
                }
            }
            tcls::DSP => {
                let bt = if chip.is_reg_n(reg) { 'T' } else { 'B' };
                let name = int_grid.name(&format!("DSP_{ocf}_{bt}_TILE"), die, col, row, 0, 0);
                let ntile = ngrid.name_tile(
                    tcrd,
                    if dev_naming.has_dsp_v2 {
                        "DSP_V2"
                    } else {
                        "DSP_V1"
                    },
                    [name],
                );
                for i in 0..2 {
                    ntile.add_bel_multi(
                        bslots::DSP[i],
                        [
                            dsp_grid.name_mult("DSP", die, col, row, 2, i as i32, 1, 0),
                            dsp_grid.name("DSP58_CPLX", die, col, row, 0, 0),
                        ],
                    );
                }
            }
            tcls::BRAM => {
                let lr = if chip.col_side(col) == DirH::W {
                    'L'
                } else {
                    'R'
                };
                let bt = if chip.is_reg_n(reg) { 'T' } else { 'B' };
                let name = int_grid.name(&format!("BRAM_{ocf}_{bt}{lr}_TILE"), die, col, row, 0, 0);
                let ntile = ngrid.name_tile(tcrd, kind, [name]);
                ntile.add_bel_multi(
                    bslots::BRAM,
                    [
                        bram_grid.name("RAMB36", die, col, row, 0, 0),
                        bram_grid.name_mult("RAMB18", die, col, row, 1, 0, 2, 0),
                        bram_grid.name_mult("RAMB18", die, col, row, 1, 0, 2, 1),
                    ],
                );
            }
            tcls::URAM => {
                let is_delay = chip.is_reg_n(reg) && row.to_idx() % Chip::ROWS_PER_REG == 44;
                let bt = if chip.is_reg_n(reg) { 'T' } else { 'B' };
                let tkn = if is_delay { "URAM_DELAY" } else { "URAM" };
                let name = int_grid.name(&format!("{tkn}_{ocf}_{bt}L_TILE"), die, col, row, 0, 0);
                let ntile = ngrid.name_tile(tcrd, tkn, [name]);
                let mut names = vec![uram_grid.name("URAM288", die, col, row, 0, 0)];
                if is_delay {
                    names.push(uram_delay_grid.name("URAM_CAS_DLY", die, col, row, 0, 0));
                }
                ntile.add_bel_multi(bslots::URAM, names);
            }
            tcls::PCIE4 | tcls::PCIE5 | tcls::MRMAC | tcls::SDFEC | tcls::DFE_CFC => {
                let (slot, naming, tk, bk, bel_grid) = match tile.class {
                    tcls::PCIE4 => (bslots::PCIE4, "PCIE4", "PCIEB", "PCIE40", &pcie4_grid),
                    tcls::PCIE5 => (bslots::PCIE5, "PCIE5", "PCIEB5", "PCIE50", &pcie5_grid),
                    tcls::MRMAC => (bslots::MRMAC, "MRMAC", "MRMAC", "MRMAC", &mrmac_grid),
                    tcls::SDFEC => (bslots::SDFEC, "SDFEC", "SDFECA", "SDFEC_A", &sdfec_grid),
                    tcls::DFE_CFC => {
                        if edev.is_reg_n(die, reg) {
                            (
                                bslots::DFE_CFC,
                                "DFE_CFC_N",
                                "DFE_CFC",
                                "DFE_CFC_TOP",
                                &dfe_cfc_top_grid,
                            )
                        } else {
                            (
                                bslots::DFE_CFC,
                                "DFE_CFC_S",
                                "DFE_CFC",
                                "DFE_CFC_BOT",
                                &dfe_cfc_bot_grid,
                            )
                        }
                    }
                    _ => unreachable!(),
                };
                let bt = if chip.is_reg_n(reg) { "TOP" } else { "BOT" };
                let name = int_grid.name(&format!("{tk}_{bt}_TILE"), die, col, row, 0, 0);
                let ntile = ngrid.name_tile(tcrd, naming, [name]);
                ntile.add_bel(slot, bel_grid.name(bk, die, col, row, 0, 0));
            }
            tcls::DCMAC | tcls::ILKN | tcls::HSC => {
                let (slot, bk, bel_grid) = match tile.class {
                    tcls::DCMAC => (bslots::DCMAC, "DCMAC", &dcmac_grid),
                    tcls::ILKN => (bslots::ILKN, "ILKNF", &ilkn_grid),
                    tcls::HSC => (bslots::HSC, "HSC", &hsc_grid),
                    _ => unreachable!(),
                };
                let name = int_grid.name(&format!("{kind}_TILE"), die, col, row, 0, 0);
                let ntile = ngrid.name_tile(tcrd, kind, [name]);
                ntile.add_bel(slot, bel_grid.name(bk, die, col, row, 0, 0));
            }
            tcls::HDIO => {
                let name = int_grid.name(
                    if chip.is_reg_n(reg) {
                        "HDIO_TILE"
                    } else {
                        "HDIO_BOT_TILE"
                    },
                    die,
                    col,
                    row,
                    0,
                    0,
                );
                let ntile = ngrid.name_tile(tcrd, kind, [name]);
                for i in 0..11 {
                    let name = hdio_grid.name_mult("HDIOLOGIC", die, col, row, 1, 0, 11, i as i32);
                    ntile.add_bel(bslots::HDIOLOGIC[2 * i], name.clone());
                    ntile.add_bel(bslots::HDIOLOGIC[2 * i + 1], name.clone());
                    ntile.add_bel(bslots::HDIOLOGIC_SCAN[i], name);
                }
                for i in 0..4 {
                    ntile.add_bel(
                        bslots::BUFGCE_HDIO[i],
                        hdio_grid.name_mult("BUFGCE_HDIO", die, col, row, 1, 0, 4, i as i32),
                    );
                }
                ntile.add_bel(
                    bslots::HDIO_BIAS,
                    hdio_grid.name("HDIO_BIAS", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::RPI_HD_APB,
                    hdio_grid.name("RPI_HD_APB", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::HDLOGIC_APB,
                    hdio_grid.name("HDLOGIC_APB", die, col, row, 0, 0),
                );
            }
            tcls::VNOC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid.name("NOC_NSU512_TOP", die, col, row + 7, 0, 0),
                        int_grid.name("NOC_NPS_VNOC_TOP", die, col, row + 15, 0, 0),
                        int_grid.name("NOC_NPS_VNOC_TOP", die, col, row + 23, 0, 0),
                        int_grid.name("NOC_NMU512_TOP", die, col, row + 31, 0, 0),
                    ],
                );
                ntile.add_bel(
                    bslots::VNOC_NSU512,
                    vnoc_grid.name("NOC_NSU512", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::VNOC_NPS_NSU,
                    vnoc_grid.name_mult("NOC_NPS_VNOC", die, col, row, 1, 0, 2, 0),
                );
                ntile.add_bel(
                    bslots::VNOC_NPS_NMU,
                    vnoc_grid.name_mult("NOC_NPS_VNOC", die, col, row, 1, 0, 2, 1),
                );
                ntile.add_bel(
                    bslots::VNOC_NMU512,
                    vnoc_grid.name("NOC_NMU512", die, col, row, 0, 0),
                );
            }
            tcls::VNOC2 => {
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid.name("NOC2_NSU512_VNOC_TILE", die, col, row + 7, 0, 0),
                        int_grid.name("NOC2_NPS5555_TOP", die, col, row + 11, 0, 0),
                        int_grid.name("NOC2_NPS5555_TOP", die, col, row + 14, 0, 0),
                        int_grid.name("NOC2_NMU512_VNOC_TILE", die, col, row + 16, 0, 0),
                        int_grid.name(
                            "NOC2_SCAN_TOP",
                            die,
                            if dev_naming.is_vnoc2_scan_offset {
                                col + 1
                            } else {
                                col
                            },
                            row + 7,
                            0,
                            0,
                        ),
                    ],
                );
            }
            tcls::VNOC4 => {
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid.name("NOC2_NSU512_VNOC4_TILE", die, col, row + 7, 0, 0),
                        int_grid.name("NOC2_NPS6X_TOP", die, col, row + 11, 0, 0),
                        int_grid.name("NOC2_NPS6X_TOP", die, col, row + 14, 0, 0),
                        int_grid.name("NOC2_NMU512_VNOC4_TILE", die, col, row + 16, 0, 0),
                        int_grid.name("NOC2_SCAN_TOP", die, col, row + 7, 0, 0),
                    ],
                );
            }
            tcls::MISR => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name(
                        "MISR_TILE",
                        die,
                        col,
                        if reg.to_idx().is_multiple_of(2) {
                            row
                        } else {
                            row - 1
                        },
                        0,
                        0,
                    )],
                );
                ntile.add_bel(bslots::MISR, misr_grid.name("MISR", die, col, row, 0, 0));
            }
            tcls::RCLK_VNOC => {
                let naming = match chip.fabric {
                    FabricKind::Orig | FabricKind::Rev2 => "RCLK_VNOC",
                    FabricKind::Ssit => "RCLK_VNOC_SSIT",
                    FabricKind::Ssit2 => "RCLK_VNOC_SSIT2",
                    FabricKind::Ulvt => "RCLK_VNOC_ULVT",
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::RCLK_VNOC_PS => {
                let naming = match chip.fabric {
                    FabricKind::Orig | FabricKind::Rev2 => "RCLK_VNOC_PS",
                    FabricKind::Ssit => "RCLK_VNOC_PS_SSIT",
                    FabricKind::Ssit2 => "RCLK_VNOC_PS_SSIT2",
                    FabricKind::Ulvt => "RCLK_VNOC_PS_ULVT",
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::RCLK_GT | tcls::RCLK_GT_CPM6 => {
                let naming = if tile.class == tcls::RCLK_GT_CPM6 {
                    "RCLK_GT_CPM6_ULVT"
                } else if !edev[tcrd.cell].conns.contains_id(cslots::CLK_DRV) {
                    "RCLK_GT_NO_LOC_ULVT"
                } else {
                    match chip.fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "RCLK_GT",
                        FabricKind::Ssit => "RCLK_GT_SSIT",
                        FabricKind::Ssit2 => "RCLK_GT_SSIT2",
                        FabricKind::Ulvt => "RCLK_GT_ULVT",
                    }
                };
                let ntile = ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
                ntile.add_bel(
                    bslots::GCLK_TAPS_DECODE_GT,
                    clk_gt_grid.name("GCLK_TAPS_DECODE_GT", die, col, row, 0, 0),
                );
                for i in 0..24 {
                    ntile.add_bel(
                        bslots::BUFG_GT[i],
                        clk_gt_grid.name_mult(
                            if chip.fabric == FabricKind::Ulvt {
                                "BUFG_GT_ULVT"
                            } else {
                                "BUFG_GT"
                            },
                            die,
                            col,
                            row,
                            1,
                            0,
                            24,
                            i as i32,
                        ),
                    );
                }
                for i in 0..41 {
                    ntile.add_bel(
                        bslots::BUFG_GT_SYNC[i],
                        clk_gt_grid.name_mult("BUFG_GT_SYNC", die, col, row, 1, 0, 41, i as i32),
                    );
                }
            }
            tcls::CLK_V_BUF_VNOC => {
                let is_top = reg.to_idx() == chip.regs - 1;
                let naming = match (chip.fabric, is_top) {
                    (FabricKind::Orig | FabricKind::Rev2, false) => "CLK_V_BUF_VNOC",
                    (FabricKind::Ssit, false) => "CLK_V_BUF_VNOC_SSIT",
                    (FabricKind::Ssit2, false) => "CLK_V_BUF_VNOC_SSIT2",
                    (FabricKind::Ulvt, false) => "CLK_V_BUF_VNOC_ULVT",
                    (FabricKind::Orig | FabricKind::Rev2, true) => "CLK_V_BUF_VNOC_TOP",
                    (FabricKind::Ssit, true) => "CLK_V_BUF_VNOC_TOP_SSIT",
                    (FabricKind::Ssit2, true) => "CLK_V_BUF_VNOC_TOP_SSIT2",
                    (FabricKind::Ulvt, true) => "CLK_V_BUF_VNOC_TOP_ULVT",
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S
            | tcls::CLK_V_BUF_VNOC_SSIT_S
            | tcls::CLK_V_BUF_VNOC_SSIT_MID_S
            | tcls::CLK_V_BUF_VNOC_SSIT_MID_N
            | tcls::CLK_V_BUF_VNOC_SSIT2_EDGE
            | tcls::CLK_V_BUF_GT_SSIT_HNOC_S
            | tcls::CLK_V_BUF_GT_SSIT_S
            | tcls::CLK_V_BUF_GT_SSIT_MID_S
            | tcls::CLK_V_BUF_GT_SSIT_MID_N
            | tcls::CLK_V_BUF_GT_SSIT2_EDGE
            | tcls::HNOC_SSIT_PS9_A
            | tcls::HNOC_SSIT_PS9_BD
            | tcls::HNOC_SSIT_PS9_C
            | tcls::HNOC_SSIT
            | tcls::HNOC_SSIT2_V
            | tcls::HNOC_SSIT2_H => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::HNOC_NCRB => {
                let naming = if edev.has_bel(tcrd.bel(bslots::HNOC_NIDB_V[0])) {
                    "HNOC_NCRB_SSIT"
                } else {
                    "HNOC_NCRB"
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_V_BUF_VNOC_SSIT_N => {
                let naming = match chip.hsr_n {
                    HsrNKind::Ssit => "CLK_V_BUF_VNOC_SSIT_N",
                    HsrNKind::Me(_) => "CLK_V_BUF_VNOC_SSIT_N_AIE",
                    HsrNKind::Hbm(_) => "CLK_V_BUF_VNOC_SSIT_N_HBM",
                    _ => unreachable!(),
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_V_BUF_GT_SSIT_N => {
                let naming = match chip.hsr_n {
                    HsrNKind::Ssit => "CLK_V_BUF_GT_SSIT_N",
                    HsrNKind::Me(_) => "CLK_V_BUF_GT_SSIT_N_HBM",
                    HsrNKind::Hbm(_) => "CLK_V_BUF_GT_SSIT_N_HBM",
                    _ => unreachable!(),
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_V_BUF_GT => {
                let naming = match chip.fabric {
                    FabricKind::Orig | FabricKind::Rev2 => "CLK_V_BUF_GT",
                    FabricKind::Ssit => "CLK_V_BUF_GT_SSIT",
                    FabricKind::Ssit2 => "CLK_V_BUF_GT_SSIT2",
                    FabricKind::Ulvt => "CLK_V_BUF_GT_ULVT",
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_S_BUF_VNOC | tcls::CLK_S_BUF_GT => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::SYSMON_SAT_VNOC => {
                if !ndb.tile_class_namings.contains_key(kind) {
                    continue;
                }
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("AMS_SAT_VNOC_TILE", die, col, row + 39, 0, 0)],
                );
            }
            tcls::SYSMON_SAT_GT => {
                if !ndb.tile_class_namings.contains_key(kind) {
                    continue;
                }
                let bt = if chip.is_reg_n(reg) { "TOP" } else { "BOT" };
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid_gt.name(
                            &format!("AMS_SAT_GT_{bt}_TILE"),
                            die,
                            col,
                            row + 19,
                            0,
                            0,
                        ),
                    ],
                );
            }
            tcls::DPLL_GT => {
                ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid_gt.name("CMT_DPLL", die, col, row + 7, 0, 0)],
                );
            }
            tcls::GTY => {
                if chip.col_side(col) == DirH::W {
                    let ntile = ngrid.name_tile(
                        tcrd,
                        "GTY_XPIPE",
                        [
                            int_grid_gt.name("GTY_QUAD_SINGLE", die, col, row, 0, 0),
                            int_grid_gt.name("XPIPE_QUAD_SINGLE", die, col, row, 0, 0),
                        ],
                    );
                    ntile.add_bel_multi(
                        bslots::GTY,
                        [
                            gty_grid.name("GTY_QUAD", die, col, row, 0, 0),
                            gty_grid.name_mult("GTY_REFCLK", die, col, row, 1, 0, 2, 0),
                            gty_grid.name_mult("GTY_REFCLK", die, col, row, 1, 0, 2, 1),
                            xpipe_grid.name("XPIPE_QUAD", die, col, row, 0, 0),
                        ],
                    );
                } else {
                    let ntile = ngrid.name_tile(
                        tcrd,
                        "GTY",
                        [int_grid_gt.name("GTY_QUAD_SINGLE", die, col, row, 0, 0)],
                    );
                    ntile.add_bel_multi(
                        bslots::GTY,
                        [
                            gty_grid.name("GTY_QUAD", die, col, row, 0, 0),
                            gty_grid.name_mult("GTY_REFCLK", die, col, row, 1, 0, 2, 0),
                            gty_grid.name_mult("GTY_REFCLK", die, col, row, 1, 0, 2, 1),
                        ],
                    );
                }
            }
            tcls::XPIPE => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    if chip.is_reg_n(reg) {
                        "XPIPE_N"
                    } else {
                        "XPIPE_S"
                    },
                    [int_grid_gt.name("XPIPE_QUAD_SINGLE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::XPIPE,
                    xpipe_grid.name("XPIPE_QUAD", die, col, row, 0, 0),
                );
            }
            tcls::GTYP | tcls::GTYP_XRAM | tcls::GTYP_PS => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    "GTYP",
                    [int_grid_gt.name("GTYP_QUAD_SINGLE", die, col, row, 0, 0)],
                );
                ntile.add_bel_multi(
                    bslots::GTYP,
                    [
                        gtyp_grid.name("GTYP_QUAD", die, col, row, 0, 0),
                        gtyp_grid.name_mult("GTYP_REFCLK", die, col, row, 1, 0, 2, 0),
                        gtyp_grid.name_mult("GTYP_REFCLK", die, col, row, 1, 0, 2, 1),
                    ],
                );
            }
            tcls::GTM | tcls::GTM_HNIC => {
                let mut tnames = vec![int_grid_gt.name("GTM_QUAD_SINGLE", die, col, row, 0, 0)];
                if tile.class == tcls::GTM_HNIC {
                    tnames.push(int_grid_gt.name("HNICPIPE_QUAD_SINGLE", die, col, row, 0, 0));
                }
                let ntile = ngrid.name_tile(tcrd, kind, tnames);
                let mut names = vec![
                    gtm_grid.name("GTM_QUAD", die, col, row, 0, 0),
                    gtm_grid.name_mult("GTM_REFCLK", die, col, row, 1, 0, 2, 0),
                    gtm_grid.name_mult("GTM_REFCLK", die, col, row, 1, 0, 2, 1),
                ];
                if tile.class == tcls::GTM_HNIC {
                    let hn_name = hnicpipe_grid.name("HNICPIPE_QUAD", die, col, row, 0, 0);
                    ntile.add_bel(bslots::HNICPIPE, hn_name.clone());
                    names.push(hn_name);
                }
                ntile.add_bel_multi(bslots::GTM, names);
            }
            tcls::GTM2_S | tcls::GTM2_N | tcls::GTM2_PS => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    "GTM2",
                    [int_grid_gt.name("GTMPW_QUAD_SINGLE", die, col, row, 0, 0)],
                );
                ntile.add_bel_multi(
                    bslots::GTM2,
                    [
                        gtm2_grid.name("GTM2_QUAD", die, col, row, 0, 0),
                        gtm2_grid.name_mult("GTM2_REFCLK", die, col, row, 1, 0, 2, 0),
                        gtm2_grid.name_mult("GTM2_REFCLK", die, col, row, 1, 0, 2, 1),
                    ],
                );
            }
            tcls::RFADC | tcls::RFDAC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid_gt.name(&format!("{kind}_BOT_CORE"), die, col, row, 0, 0),
                        int_grid_gt.name(&format!("{kind}_TOP_CORE"), die, col, row, 0, 48),
                    ],
                );
                let (bslots, grid) = match tile.class {
                    tcls::RFADC => (bslots::RFADC, &rfadc_grid),
                    tcls::RFDAC => (bslots::RFDAC, &rfdac_grid),
                    _ => unreachable!(),
                };
                for i in 0..2 {
                    ntile.add_bel(bslots[i], grid.name(kind, die, col, row, 0, i as _));
                }
            }
            tcls::BFR_B => {
                let bt = if chip.is_reg_n(reg) { "TOP" } else { "BOT" };
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name(&format!("BFR_TILE_B_{bt}_CORE"), die, col, row, 0, 0)],
                );
                ntile.add_bel(bslots::BFR_B, bfr_b_grid.name("BFR_B", die, col, row, 0, 0));
            }
            tcls::VDU => {
                let ntile =
                    ngrid.name_tile(tcrd, kind, [int_grid.name("VDU_CORE", die, col, row, 0, 0)]);
                ntile.add_bel(bslots::VDU, vdu_grid.name("VDU", die, col, row, 0, 0));
            }
            tcls::VCU2 => {
                let ntile = ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
                ntile.add_bel(bslots::VCU2, vcu2_grid.name("VCU2", die, col, row, 0, 0));
            }
            tcls::ISP2 => {
                let ntile = ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
                ntile.add_bel(bslots::ISP2, isp2_grid.name("ISP2", die, col, row, 0, 0));
            }
            tcls::XRAM | tcls::XRAM_NC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    "XRAM",
                    [int_grid.name("XRAM_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(bslots::XRAM, xram_grid.name("XRAM", die, col, row, 0, 0));
            }
            tcls::PS9 => {
                let naming = if chip.ps == PsKind::Ps9Cpm6 {
                    "PS9_2VP"
                } else if chip.fabric != FabricKind::Orig {
                    "PS9_REV2"
                } else {
                    "PS9"
                };
                let ntile = ngrid.name_tile(
                    tcrd,
                    naming,
                    [int_grid.name("PSS_BASE_CORE", die, ColId::from_idx(0), row + 17, 0, 0)],
                );
                ntile.add_bel(bslots::PS9, ps_grid.name("PS9", die, col, row, 0, 0));
            }
            tcls::CPM4 => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [
                        int_grid.name("CPM_CORE", die, ColId::from_idx(0), row - 1, 0, 0),
                        int_grid.name("CPM_EXT_TILE", die, chip.cols_hard[0].col, row + 48, 0, 0),
                    ],
                );
                ntile.add_bel_multi(
                    bslots::CPM4,
                    ["CPM_MAIN_X0Y0".into(), "CPM_EXT_X0Y0".into()],
                );
            }
            tcls::CPM5 => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("CPM_G5_TILE", die, ColId::from_idx(0), row - 1, 0, 0)],
                );
                ntile.add_bel(bslots::CPM5, "CPM5_X0Y0".into());
            }
            tcls::CPM6 => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("CPM6_TILE", die, ColId::from_idx(0), row - 1, 0, 0)],
                );
                ntile.add_bel(bslots::CPM6, "CPM6_X0Y0".into());
            }
            tcls::PSXL => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("PSXL_CORE", die, ColId::from_idx(0), row, 0, 0)],
                );
                ntile.add_bel(bslots::PSXL, ps_grid.name("PSX", die, col, row, 0, 0));
            }
            tcls::CPM5N => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("CPM_G5N2X_TILE", die, ColId::from_idx(0), row - 1, 0, 0)],
                );
                ntile.add_bel(bslots::CPM5N, "CPM5N_X0Y0".into());
            }
            tcls::PSXC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("PSXC_TILE", die, ColId::from_idx(0), row, 0, 0)],
                );
                ntile.add_bel(bslots::PSXC, ps_grid.name("PS11", die, col, row, 0, 0));
            }
            tcls::MMI | tcls::MMI_NC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name(
                        "MMI_TILE",
                        die,
                        ColId::from_idx(0),
                        if tile.class == tcls::MMI_NC {
                            row
                        } else {
                            row - 1
                        },
                        0,
                        0,
                    )],
                );
                ntile.add_bel(
                    bslots::MMI,
                    if tile.class == tcls::MMI {
                        "MMI_X0Y0"
                    } else {
                        "MMI2_X0Y0"
                    }
                    .into(),
                );
            }
            tcls::HNICX => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [int_grid.name("HNICX_TILE", die, col, row, 0, 0)],
                );
                ntile.add_bel(bslots::HNICX, "HNICX_X0Y0".into());
            }

            tcls::SYSMON_SAT_HSR_XPIO | tcls::SYSMON_SAT_HSR_X5IO => {
                ngrid.name_tile(
                    tcrd,
                    "SYSMON_SAT_HSR_XPIO",
                    dev_naming.tile_names[&tcrd].clone(),
                );
            }
            tcls::SYSMON_SAT_HSR_ME | tcls::SYSMON_SAT_HSR_NC => {
                ngrid.name_tile(
                    tcrd,
                    "SYSMON_SAT_HSR_ME",
                    dev_naming.tile_names[&tcrd].clone(),
                );
            }

            tcls::CLK_XPIO_FULL
            | tcls::CLK_XPIO_MIN
            | tcls::CLK_X5IO_FULL
            | tcls::CLK_X5IO_MIN_A
            | tcls::CLK_X5IO_MIN_B => {
                ngrid.name_tile(tcrd, "CLK_XPIO", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_HSR_SSIT2_H => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::CLK_ME => {
                let naming = if matches!(chip.hsr_n, HsrNKind::Me2(_)) {
                    "CLK_ME_ME2"
                } else {
                    match chip.fabric {
                        FabricKind::Orig => "CLK_ME",
                        FabricKind::Ulvt => "CLK_ME_ULVT",
                        _ => "CLK_ME_REV2",
                    }
                };
                ngrid.name_tile(tcrd, naming, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::MMCM_XPIO | tcls::MMCM_X5IO => {
                ngrid.name_tile(tcrd, "MMCM", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::DPLL_XPIO | tcls::DPLL_X5IO | tcls::DPLL_X5IO_NC => {
                ngrid.name_tile(tcrd, "DPLL", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::XPIO | tcls::XPIO_NC => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::XPIO_DCI_A | tcls::XPIO_DCI_B | tcls::XPIO_DCI_C | tcls::XPIO_DCI_S => {
                ngrid.name_tile(tcrd, "XPIO_DCI", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::XPIO_SCAN_A | tcls::XPIO_SCAN_B | tcls::XPIO_SCAN_C | tcls::XPIO_SCAN_S => {
                let mut names = dev_naming.tile_names[&tcrd.tile(tslots::HSR_BANK)].clone();
                for i in 0..9 {
                    names.extend(
                        dev_naming.tile_names[&tcrd.delta(i, 0).tile(tslots::HSR_IO)].clone(),
                    );
                }
                ngrid.name_tile(tcrd, "XPIO_SCAN", names);
            }
            tcls::X5IO_BANK | tcls::X5IO_BANK_NC => {
                ngrid.name_tile(tcrd, "X5IO_BANK", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::X5PLL_A | tcls::X5PLL_B | tcls::X5PLL_S | tcls::X5PLL_NC => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::X5IO_SCAN_A | tcls::X5IO_SCAN_B | tcls::X5IO_SCAN_C => {
                ngrid.name_tile(tcrd, "X5IO_SCAN", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::X5IO | tcls::X5IO_NC => {
                let i = [0, 1, 2, 3, 3, 2, 1, 0][col.to_idx() % 8];
                ngrid.name_tile(
                    tcrd,
                    &format!("X5IO{i}"),
                    dev_naming.tile_names[&tcrd].clone(),
                );
            }
            tcls::XPLL
            | tcls::DDRMC
            | tcls::DDRMC5
            | tcls::DDRMC5C
            | tcls::DDRMC5E
            | tcls::DDRMC5E2
            | tcls::HNOC_XPIO
            | tcls::HNOC_XPIO_PS9_A
            | tcls::HNOC_XPIO_PS9_B
            | tcls::HNOC_XPIO_PS9_C
            | tcls::HNOC_X5IO_PS9_A
            | tcls::HNOC_X5IO_PS9_B
            | tcls::HNOC_X5IO_PS9_C
            | tcls::HNOC_X5IO_PSX_A
            | tcls::HNOC_X5IO_PSX_B
            | tcls::HNOC_X5IO_PSXL_C
            | tcls::HNOC_X5IO_PSXC_C
            | tcls::HNOC_X5IO_CAP
            | tcls::HNOC_X5IO_DDRMC
            | tcls::HNOC_ME
            | tcls::HNOC_ME_VNOC
            | tcls::HNOC_ME2_CAP_W
            | tcls::HNOC_ME2_CAP_E
            | tcls::HNOC_ME2
            | tcls::HNOC_ME2_NCRB
            | tcls::HNOC_ME2_VNOC4
            | tcls::HNOC_HBM
            | tcls::HNOC_HBM_VNOC
            | tcls::HNOC_NIC_CPM_CDX
            | tcls::HNOC_NIC_CPM_DPU
            | tcls::HNOC_NIC_VNOC
            | tcls::HNOC_NIC_BLI
            | tcls::HNOC_NIC_HNICX
            | tcls::HNOC2_NCRB
            | tcls::CLK_HBM
            | tcls::HBM_INTF
            | tcls::HBM_CHNL
            | tcls::HBM_MS => {
                ngrid.name_tile(tcrd, kind, dev_naming.tile_names[&tcrd].clone());
            }
            tcls::HNOC_X5IO_CROSS | tcls::HNOC_X5IO_CROSS_ALT => {
                ngrid.name_tile(
                    tcrd,
                    "HNOC_X5IO_CROSS",
                    dev_naming.tile_names[&tcrd].clone(),
                );
            }
            tcls::DDRMC5E_ALT => {
                ngrid.name_tile(tcrd, "DDRMC5E", dev_naming.tile_names[&tcrd].clone());
            }
            tcls::HSR_CLK_TEST_XPIO_REV1
            | tcls::HSR_CLK_TEST_XPIO_REV2
            | tcls::HSR_CLK_TEST_ME_REV1
            | tcls::HSR_CLK_TEST_ME_REV2
            | tcls::HSR_CLK_TEST_ME_SSIT
            | tcls::HSR_CLK_TEST_ME_ULVT
            | tcls::HSR_CLK_HBM_EVEN
            | tcls::HSR_CLK_HBM_ODD
            | tcls::HSR_CLK_HBM_MS_EVEN
            | tcls::HSR_CLK_HBM_MS_ODD
            | tcls::HSR_CLK_AIE_PLL
            | tcls::HSR_CLK_TEST_X5IO_A
            | tcls::HSR_CLK_TEST_X5IO_B
            | tcls::X5PLL_GASKET
            | tcls::X5PLL_GASKET_HALF => (),

            tcls::AIE => {
                let ntile =
                    ngrid.name_tile(tcrd, kind, [aie_grid.name("AIE_TILE", die, col, row, 0, 0)]);
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_TILECTRL", die, col, row, 0, 0),
                );
                for i in 0..2 {
                    ntile.add_bel(
                        bslots::AIE_FIFO[i],
                        aie_grid.name_mult("AIE_FIFO", die, col, row, 2, i as i32, 1, 0),
                    );
                }
                ntile.add_bel(
                    bslots::AIE_CORE,
                    aie_core_grid.name("AIE_CORE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ARRAYSWITCH,
                    aie_core_grid.name("AIE_ARRAYSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_MEMGRP,
                    aie_core_grid.name("AIE_MEMGRP", die, col, row, 0, 0),
                );
            }
            tcls::AIE_PLL => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_INTF_A_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(bslots::AIE_PLL, "AIE_PLL_X0Y0".into());
            }
            tcls::AIE_SHIM => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_INTF_A_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_TILECTRL", die, col, row, 0, 0),
                );
                for i in 0..2 {
                    ntile.add_bel(
                        bslots::AIE_FIFO[i],
                        aie_grid.name_mult("AIE_FIFO", die, col, row, 2, i as i32, 1, 0),
                    );
                }
                ntile.add_bel(
                    bslots::AIE_SHIMSWITCH,
                    aie_grid.name("AIE_SHIMSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_SHIMTRACE,
                    aie_grid.name("AIE_SHIMTRACE", die, col, row, 0, 0),
                );
            }
            tcls::AIE_SHIM_NOC_PL | tcls::AIE_SHIM_PL => {
                let HsrNKind::Me(ref me) = chip.hsr_n else {
                    unreachable!()
                };
                let prefix = if col.to_idx() < me.regions.len() * 4 + 1 {
                    [
                        "AIE_INTF_B0_CORE",
                        "AIE_INTF_B1_CORE",
                        "AIE_INTF_B2_CORE",
                        "AIE_INTF_B3_CORE",
                    ][(col.to_idx() - 1) % 4]
                } else {
                    "AIE_INTF_C_CORE"
                };
                let ntile =
                    ngrid.name_tile(tcrd, kind, [aie_grid.name(prefix, die, col, row, 0, 0)]);
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_TILECTRL", die, col, row, 0, 0),
                );
                for i in 0..2 {
                    ntile.add_bel(
                        bslots::AIE_FIFO[i],
                        aie_grid.name_mult("AIE_FIFO", die, col, row, 2, i as i32, 1, 0),
                    );
                }
                ntile.add_bel(
                    bslots::AIE_SHIMSWITCH,
                    aie_grid.name("AIE_SHIMSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_SHIMTRACE,
                    aie_grid.name("AIE_SHIMTRACE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_PL,
                    aie_pl_grid.name("AIE_PL", die, col, row, 0, 0),
                );
                if tile.class == tcls::AIE_SHIM_NOC_PL {
                    ntile.add_bel(
                        bslots::AIE_NOC,
                        aie_noc_grid.name("AIE_NOC", die, col, row, 0, 0),
                    );
                }
            }
            tcls::AIE_INTF_TEST => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_INTF_B0_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_INTF_TEST,
                    aie_test_grid.name("AIE_INTF_B_TEST", die, col, row, 0, 0),
                );
            }
            tcls::AIE_INTF_TEST_NC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_INTF_C_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_INTF_TEST,
                    aie_test_nc_grid.name("AIE_INTF_C_TEST", die, col, row, 0, 0),
                );
            }

            tcls::AIE_ML => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_TILE", die, col, row, 0, 1)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_ML_TILECTRL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_FIFO[0],
                    aie_fifo_grid.name("AIE_ML_FIFO", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_CORE,
                    aie_core_grid.name("AIE_ML_CORE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_ARRAYSWITCH,
                    aie_core_grid.name("AIE_ML_ARRAYSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_MEMGRP,
                    aie_core_grid.name("AIE_ML_MEMGRP", die, col, row, 0, 0),
                );
            }
            tcls::AIE_ML_MEM => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_MEM_TILE", die, col, row, 0, 1)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_ML_TILECTRL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_MEM_ARRAYSWITCH,
                    aie_mem_grid.name("AIE_ML_MEM_ARRAYSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_MEM_MEMGRP,
                    aie_mem_grid.name("AIE_ML_MEM_MEMGRP", die, col, row, 0, 0),
                );
            }
            tcls::AIE_ML_PLL => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_INTF_A_CORE", die, col, row, 0, 1)],
                );
                ntile.add_bel(bslots::AIE_PLL, "AIE_ML_PLL_X0Y0".into());
            }
            tcls::AIE_ML_SHIM => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_INTF_A_CORE", die, col, row, 0, 1)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_ML_TILECTRL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_FIFO[0],
                    aie_fifo_grid.name("AIE_ML_FIFO", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_SHIMSWITCH,
                    aie_grid.name("AIE_ML_SHIMSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_SHIMTRACE,
                    aie_grid.name("AIE_ML_SHIMTRACE", die, col, row, 0, 0),
                );
            }
            tcls::AIE_ML_SHIM_NOC_PL | tcls::AIE_ML_SHIM_PL => {
                let HsrNKind::Me(ref me) = chip.hsr_n else {
                    unreachable!()
                };
                let prefix = if col.to_idx() < me.regions.len() * 4 + 1 {
                    [
                        "AIE_ML_SHIM_PL_TILE",
                        "AIE_ML_SHIM_NOC_TILE",
                        "AIE_ML_SHIM_NOC_TILE",
                        "AIE_ML_SHIM_PL_TILE",
                    ][(col.to_idx() - 1) % 4]
                } else {
                    "AIE_ML_SHIM_PL_TILE"
                };
                let ntile =
                    ngrid.name_tile(tcrd, kind, [aie_grid.name(prefix, die, col, row, 0, 1)]);
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name("AIE_ML_TILECTRL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_FIFO[0],
                    aie_fifo_grid.name("AIE_ML_FIFO", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_SHIMSWITCH,
                    aie_grid.name("AIE_ML_SHIMSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_SHIMTRACE,
                    aie_grid.name("AIE_ML_SHIMTRACE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_PL,
                    aie_pl_grid.name("AIE_ML_PL", die, col, row, 0, 0),
                );
                if tile.class == tcls::AIE_ML_SHIM_NOC_PL {
                    ntile.add_bel(
                        bslots::AIE_NOC,
                        aie_noc_grid.name("AIE_ML_NOC", die, col, row, 0, 0),
                    );
                }
            }
            tcls::AIE_ML_INTF_TEST => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_INTF_B_TILE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_ML_INTF_TEST,
                    aie_test_grid.name("AIE_ML_INTF_B_TEST", die, col, row, 0, 0),
                );
            }
            tcls::AIE_ML_INTF_TEST_NC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE_ML_INTF_C_TILE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_ML_INTF_TEST,
                    aie_test_nc_grid.name("AIE_ML_INTF_C_TEST", die, col, row, 0, 0),
                );
            }

            tcls::AIE2PS => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE2P_S_TILE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name_mult("AIE2P_TILECTRL", die, col, row, 2, 0, 1, 0),
                );
                ntile.add_bel(
                    bslots::AIE_FIFO[0],
                    aie_fifo_grid.name_mult("AIE2P_FIFO", die, col, row, 2, 0, 1, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_CORE,
                    aie_core_grid.name("AIE2PS_CORE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_ARRAYSWITCH,
                    aie_core_grid.name("AIE2PS_ARRAYSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_MEMGRP,
                    aie_core_grid.name("AIE2PS_MEMGRP", die, col, row, 0, 0),
                );
            }
            tcls::AIE2PS_MEM => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE2P_S_MEM_TILE", die, col, row, 0, 0)],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name_mult("AIE2P_TILECTRL", die, col, row, 2, 0, 1, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_MEM_ARRAYSWITCH,
                    aie_mem_grid.name("AIE2PS_MEM_ARRAYSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_ML_MEM_MEMGRP,
                    aie_mem_grid.name("AIE2PS_MEM_MEMGRP", die, col, row, 0, 0),
                );
            }
            tcls::AIE2PS_SHIM_NOC_PL => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    if col.to_idx() == 0 {
                        "AIE2PS_SHIM_NOC_PL_A"
                    } else {
                        "AIE2PS_SHIM_NOC_PL"
                    },
                    [aie_grid.name(
                        if col.to_idx() == 0 {
                            "AIE2P_S_INTF_A_CORE"
                        } else {
                            "AIE2P_S_INTF_C_CORE"
                        },
                        die,
                        col,
                        row,
                        0,
                        0,
                    )],
                );
                ntile.add_bel(
                    bslots::AIE_TILECTRL,
                    aie_grid.name_mult("AIE2P_TILECTRL", die, col, row, 2, 1, 1, 0),
                );
                ntile.add_bel(
                    bslots::AIE_FIFO[0],
                    aie_fifo_grid.name_mult("AIE2P_FIFO", die, col, row, 2, 1, 1, 0),
                );
                ntile.add_bel(
                    bslots::AIE2PS_SHIM_CNTRL,
                    aie_grid.name("AIE2PS_SHIM_CNTRL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE2PS_NMU_ARBIT,
                    aie_grid.name("AIE2PS_NMU_ARBIT", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE2PS_SHIMSWITCH,
                    aie_grid.name("AIE2PS_SHIMSWITCH", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_SHIMTRACE,
                    aie_grid.name("AIE2PS_SHIMTRACE", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_PL,
                    aie_pl_grid.name("AIE2P_PL", die, col, row, 0, 0),
                );
                ntile.add_bel(
                    bslots::AIE_NOC,
                    aie_noc_grid.name("AIE2PS_NOC", die, col, row, 0, 0),
                );
            }
            tcls::AIE2PS_PLL => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    kind,
                    [aie_grid.name("AIE2P_S_INTF_A_CORE", die, col, row, 0, 0)],
                );
                ntile.add_bel(bslots::AIE_PLL, "AIE2PS_PLL_X0Y0".into());
            }
            tcls::AIE2PS_INTF_TEST | tcls::AIE2PS_INTF_TEST_NC => {
                let ntile = ngrid.name_tile(
                    tcrd,
                    "AIE2PS_INTF_TEST",
                    [aie_grid.name(
                        if col.to_idx() == 0 {
                            "AIE2P_S_INTF_A_CORE"
                        } else {
                            "AIE2P_S_INTF_C_CORE"
                        },
                        die,
                        col,
                        row,
                        0,
                        0,
                    )],
                );

                if col.to_idx() == 0 {
                    ntile.add_bel(
                        bslots::AIE2PS_INTF_TEST,
                        aie_grid.name("AIE2PS_INTF_A_TEST", die, col, row, 0, 0),
                    );
                } else {
                    ntile.add_bel(
                        bslots::AIE2PS_INTF_TEST,
                        aie_grid.name("AIE2PS_INTF_C_TEST", die, col, row, -1, 0),
                    );
                }
            }

            _ => panic!("how to {kind}"),
        }
    }

    for (&bcrd, names) in &dev_naming.bel_names {
        let tcrd = edev.bel_tile(bcrd);
        let Some(ntile) = ngrid.tiles.get_mut(&tcrd) else {
            continue;
        };
        ntile.add_bel_multi(bcrd.slot, names.clone());
    }

    ExpandedNamedDevice {
        edev,
        ngrid,
        dev_naming,
    }
}
