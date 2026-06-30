use prjcombine_entity::{EntityId, EntityVec};
use prjcombine_interconnect::{
    db::BelSlotId,
    dir::{Dir, DirV},
    grid::{ColId, DieId, DieIdExt, RowId, TileCoord},
};
use prjcombine_re_xilinx_naming_versal::DeviceNaming;
use prjcombine_re_xilinx_rawdump::{Coord, NodeOrWire, Part, Tile, TkSiteSlot};
use prjcombine_versal::{
    chip::{
        AiArray, AiColumn, AiKind, BramKind, Chip, CleKind, Column, ColumnKind, DisabledPart,
        FabricKind, GtRowKind, HRegId, HardColumn, HardRowKind, HbmRow, HsrNKind, HsrSKind,
        Interposer, InterposerKind, Me2HNoc, Me2Region, Me2Row, MeRegion, MeRow, MeVNocConn,
        NicBlock, NicRow, PsKind, RegId, VNocConn, X5io, X5ioClk, X5ioClkBli, X5ioDdrmc,
        X5ioDdrmcKind, X5ioDdrmcTestClk, X5ioHNoc, X5ioPllBli, X5ioRegion, X5ioRow, X5ioScanBli,
        XpioClkBli, XpioRegion, XpioRow, XpioTestBli,
    },
    defs::{bslots, tslots},
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use prjcombine_re_xilinx_rd2db_grid::{
    IntGrid, extract_int_slr, extract_int_slr_column, find_rows,
};

use crate::{
    bli::{BliConns, get_bli},
    clk,
};

fn adjust_column(col: ColId, side: Dir) -> ColId {
    match side {
        Dir::W => ColId::from_idx(col.to_idx() * 2),
        Dir::E => ColId::from_idx(col.to_idx() * 2 + 1),
        _ => unreachable!(),
    }
}

struct IntGridWrapper<'a> {
    int: IntGrid<'a>,
    bli: &'a BliConns,
}

impl<'a> IntGridWrapper<'a> {
    pub fn num_cols(&self) -> usize {
        self.int.cols.len() * 2
    }

    pub fn find_columns(&self, tts: &[&str]) -> Vec<i32> {
        self.int.find_columns(tts)
    }

    pub fn find_column(&self, tts: &[&str]) -> Option<i32> {
        self.int.find_column(tts)
    }

    pub fn find_tiles(&self, tts: &[&str]) -> Vec<(i32, i32)> {
        let mut res = Vec::from_iter(self.int.find_tiles(tts));
        res.sort_by_key(|&(x, y)| {
            (
                y * if self.int.mirror_y { -1 } else { 1 },
                x * if self.int.mirror_x { -1 } else { 1 },
            )
        });
        res
    }

    pub fn find_tiles_crd(&self, tts: &[&str]) -> Vec<Coord> {
        self.find_tiles(tts)
            .into_iter()
            .map(|(x, y)| Coord {
                x: x as u16,
                y: y as u16,
            })
            .collect()
    }

    pub fn find_rows(&self, tts: &[&str]) -> Vec<i32> {
        self.int.find_rows(tts)
    }

    pub fn lookup_column_inter(&self, col: i32) -> ColId {
        adjust_column(self.int.lookup_column_inter(col), Dir::W)
    }

    pub fn lookup_row(&self, row: i32) -> RowId {
        self.int.lookup_row(row)
    }

    pub fn delta(&self, xy: Coord, dx: i32, dy: i32) -> Coord {
        self.int.delta(xy, dx, dy)
    }

    fn xlat_bli_full(&self, crd: Coord, wire: &str) -> Option<(ColId, &'a str)> {
        let wi = self.int.rd.wires.get(wire)?;
        let &(bli_crd, w) = self.bli.get(&(crd, wi))?;
        let col = match self.int.try_lookup_column(bli_crd.x as i32) {
            Ok(c) => adjust_column(c, Dir::E),
            Err(c) => adjust_column(c - 1, Dir::E),
        };
        Some((col, &self.int.rd.wires[w]))
    }

    fn xlat_bli(&self, crd: Coord, wire: &str) -> Option<ColId> {
        let (bli_crd, _) = self.xlat_bli_full(crd, wire)?;
        Some(bli_crd)
    }

    fn xlat_hsr_clk(&self, crd: Coord, wire: &str) -> Option<ColId> {
        let nw = self.int.rd.lookup_wire(crd, wire)?;
        let NodeOrWire::Node(ni) = nw else {
            return None;
        };
        for (oxy, _ow) in self.int.rd.iter_node(ni) {
            let otile = &self.int.rd.tiles[&oxy];
            if otile.name.contains("VERT_TO_HSR") || otile.name.contains("REBUF_VERT") {
                let col = self.lookup_column_inter(oxy.x as i32);
                let col = if col.to_idx() == 0 { col } else { col - 1 };
                return Some(col);
            }
        }
        None
    }

    fn follow_wire(&self, xy: Coord, wire: &str) -> Option<Coord> {
        let ni = self.int.rd.lookup_wire(xy, wire)?;
        let NodeOrWire::Node(ni) = ni else {
            return None;
        };
        Some(self.int.rd.iter_node(ni).find(|&(cxy, _)| cxy != xy)?.0)
    }

    fn follow_wire_full(&self, xy: Coord, wire: &str) -> Option<(Coord, &str)> {
        let ni = self.int.rd.lookup_wire(xy, wire)?;
        let NodeOrWire::Node(ni) = ni else {
            return None;
        };
        self.int
            .rd
            .iter_node(ni)
            .find(|&(cxy, _)| cxy != xy)
            .map(|(xy, wn)| (xy, self.int.rd.wires[wn].as_str()))
    }

    fn get_simple_names(&self, tkns: &[&str]) -> Vec<(Coord, String, String)> {
        let mut res = vec![];
        for xy in self.find_tiles_crd(tkns) {
            let tile = &self.int.rd.tiles[&xy];
            let bname = tile.sites.values().next().unwrap();
            res.push((xy, tile.name.clone(), bname.clone()));
        }
        res
    }
}

fn extract_site<'a>(rd: &Part, tile: &'a Tile, sname: &str, x: u8, y: u8) -> Option<&'a str> {
    let tk = &rd.tile_kinds[tile.kind];
    let tks = TkSiteSlot::Xy(rd.slot_kinds.get(sname)?, x, y);
    let si = tk.sites.get(&tks)?.0;
    Some(tile.sites.get(si)?)
}

fn make_columns(
    die: DieId,
    int: &IntGridWrapper,
    disabled: &mut BTreeSet<DisabledPart>,
    naming: &mut DeviceNaming,
) -> (EntityVec<ColId, Column>, Vec<HardColumn>) {
    let mut res: EntityVec<ColId, Column> = (0..int.num_cols())
        .map(|_| Column {
            kind: ColumnKind::None,
            has_bli_s: false,
            has_bli_n: false,
        })
        .collect();

    for (tkn, kind_w, kind_e) in [
        (
            "CLE_W_CORE",
            ColumnKind::Cle(CleKind::Plain),
            ColumnKind::Cle(CleKind::Plain),
        ),
        (
            "CLE_W_VR_CORE",
            ColumnKind::Cle(CleKind::Plain),
            ColumnKind::Cle(CleKind::Plain),
        ),
        ("DSP_LOCF_B_TILE", ColumnKind::Dsp, ColumnKind::ContDsp),
        ("DSP_LOCF_T_TILE", ColumnKind::Dsp, ColumnKind::ContDsp),
        ("DSP_ROCF_B_TILE", ColumnKind::Dsp, ColumnKind::ContDsp),
        ("DSP_ROCF_T_TILE", ColumnKind::Dsp, ColumnKind::ContDsp),
        ("NOC_NSU512_TOP", ColumnKind::VNoc, ColumnKind::ContVNoc),
        (
            "NOC2_NSU512_VNOC_TILE",
            ColumnKind::VNoc2,
            ColumnKind::ContVNoc,
        ),
        (
            "NOC2_NSU512_VNOC4_TILE",
            ColumnKind::VNoc4,
            ColumnKind::ContVNoc,
        ),
    ] {
        for c in int.find_columns(&[tkn]) {
            let c = int.lookup_column_inter(c);
            res[c - 1].kind = kind_w;
            res[c].kind = kind_e;
        }
    }
    for (tkn, kind) in [
        ("BRAM_LOCF_TR_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_LOCF_BR_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_ROCF_TR_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_ROCF_BR_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("INTF_GT_TR_TILE", ColumnKind::Gt),
        ("INTF_GT_BR_TILE", ColumnKind::Gt),
    ] {
        for c in int.find_columns(&[tkn]) {
            let c = int.lookup_column_inter(c);
            res[c - 1].kind = kind;
        }
    }
    for (tkn, kind) in [
        ("BRAM_LOCF_TL_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_LOCF_BL_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_ROCF_TL_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("BRAM_ROCF_BL_TILE", ColumnKind::Bram(BramKind::Plain)),
        ("URAM_LOCF_TL_TILE", ColumnKind::Uram),
        ("URAM_LOCF_BL_TILE", ColumnKind::Uram),
        ("URAM_ROCF_TL_TILE", ColumnKind::Uram),
        ("URAM_ROCF_BL_TILE", ColumnKind::Uram),
        ("INTF_GT_TL_TILE", ColumnKind::Gt),
        ("INTF_GT_BL_TILE", ColumnKind::Gt),
    ] {
        for c in int.find_columns(&[tkn]) {
            let c = int.lookup_column_inter(c);
            res[c].kind = kind;
        }
    }
    for (tkn, kind) in [("SLL", CleKind::Sll), ("SLL2", CleKind::Sll2)] {
        for c in int.find_columns(&[tkn]) {
            let c = int.lookup_column_inter(c);
            assert_eq!(res[c].kind, ColumnKind::Cle(CleKind::Plain));
            assert_eq!(res[c - 1].kind, ColumnKind::Cle(CleKind::Plain));
            res[c].kind = ColumnKind::Cle(kind);
            res[c - 1].kind = ColumnKind::Cle(kind);
        }
    }
    for c in int.find_columns(&["RCLK_BRAM_CLKBUF_CORE", "RCLK_BRAM_CLKBUF_VR_CORE"]) {
        let c = int.lookup_column_inter(c);
        assert_eq!(res[c - 1].kind, ColumnKind::Bram(BramKind::Plain));
        res[c - 1].kind = ColumnKind::Bram(BramKind::SplitterPd);
    }
    for c in int.find_columns(&[
        "RCLK_BRAM_CLKBUF_NOPD_CORE",
        "RCLK_BRAM_CLKBUF_NOPD_VR_CORE",
    ]) {
        let c = int.lookup_column_inter(c);
        assert_eq!(res[c - 1].kind, ColumnKind::Bram(BramKind::Plain));
        res[c - 1].kind = ColumnKind::Bram(BramKind::Splitter);
    }
    for c in int.find_columns(&["RCLK_BRAM_CORE", "RCLK_BRAM_VR_CORE"]) {
        let c = int.lookup_column_inter(c);
        if res[c - 1].kind == ColumnKind::Bram(BramKind::Splitter) {
            res[c - 1].kind = ColumnKind::Bram(BramKind::OptSplitter);
        }
    }

    for c in int.find_columns(&[
        "BLI_CLE_TOP_CORE",
        "BLI_DSP_LOCF_TR_TILE",
        "BLI_DSP_ROCF_TR_TILE",
        "BLI_BRAM_LOCF_TR_TILE",
        "BLI_BRAM_ROCF_TR_TILE",
    ]) {
        let c = int.lookup_column_inter(c);
        res[c - 1].has_bli_n = true;
    }
    for c in int.find_columns(&[
        "BLI_CLE_TOP_CORE_MY",
        "BLI_DSP_LOCF_TL_TILE",
        "BLI_DSP_ROCF_TL_TILE",
        "BLI_BRAM_ROCF_TL_TILE",
        "BLI_URAM_LOCF_TL_TILE",
        "BLI_URAM_ROCF_TL_TILE",
    ]) {
        let c = int.lookup_column_inter(c);
        res[c].has_bli_n = true;
    }
    for c in int.find_columns(&[
        "BLI_CLE_BOT_CORE",
        "BLI_DSP_ROCF_BR_TILE",
        "BLI_BRAM_ROCF_BR_TILE",
    ]) {
        let c = int.lookup_column_inter(c);
        res[c - 1].has_bli_s = true;
    }
    for c in int.find_columns(&[
        "BLI_CLE_BOT_CORE_MY",
        "BLI_DSP_ROCF_BL_TILE",
        "BLI_BRAM_ROCF_BL_TILE",
        "BLI_URAM_ROCF_BL_TILE",
    ]) {
        let c = int.lookup_column_inter(c);
        res[c].has_bli_s = true;
    }

    let col_cfrm = int.lookup_column_inter(
        int.find_column(&["CFRM_PMC_TILE", "CFRM_PMC_VR_TILE"])
            .unwrap(),
    );
    res[col_cfrm].kind = ColumnKind::Cfrm;

    let mut hard_cells = BTreeMap::new();
    for (tt, kind) in [
        ("HDIO_TILE", HardRowKind::Hdio),
        ("HDIO_BOT_TILE", HardRowKind::Hdio),
        ("PCIEB_TOP_TILE", HardRowKind::Pcie4),
        ("PCIEB_BOT_TILE", HardRowKind::Pcie4),
        ("PCIEB5_TOP_TILE", HardRowKind::Pcie5),
        ("PCIEB5_BOT_TILE", HardRowKind::Pcie5),
        ("MRMAC_TOP_TILE", HardRowKind::Mrmac),
        ("MRMAC_BOT_TILE", HardRowKind::Mrmac),
        ("SDFECA_TOP_TILE", HardRowKind::Sdfec),
        ("DFE_CFC_BOT_TILE", HardRowKind::DfeCfc),
        ("DFE_CFC_TOP_TILE", HardRowKind::DfeCfc),
        ("CPM_EXT_TILE", HardRowKind::CpmExt),
    ] {
        for (x, y) in int.find_tiles(&[tt]) {
            let tile = &int.int.rd.tiles[&Coord {
                x: x as u16,
                y: y as u16,
            }];
            let col = int.lookup_column_inter(x) - 1;
            let reg = RegId::from_idx(int.lookup_row(y).to_idx() / 48);
            let row = RowId::from_idx(int.lookup_row(y).to_idx() / 48 * 48);
            if tile.sites.iter().next().is_none() {
                disabled.insert(DisabledPart::HardIpSite(die, col, reg));
            }
            hard_cells.insert((col, reg), kind);
            if tt.starts_with("HDIO") {
                for i in 0..11 {
                    let name = extract_site(int.int.rd, tile, "IOB", 0, i as u8).unwrap();
                    naming.bel_names.insert(
                        die.cell(col, row).bel(bslots::HDIOB[2 * i]),
                        vec![name.into()],
                    );
                    naming.bel_names.insert(
                        die.cell(col, row).bel(bslots::HDIOB[2 * i + 1]),
                        vec![name.into()],
                    );
                }
                if let Some(name) = extract_site(int.int.rd, tile, "DPLL", 0, 0) {
                    naming
                        .bel_names
                        .insert(die.cell(col, row).bel(bslots::DPLL_HDIO), vec![name.into()]);
                } else {
                    disabled.insert(DisabledPart::HdioDpll(die, col, reg));
                    let is_vc1902 = ["vc1902", "vc1802", "vm1802", "v65"]
                        .into_iter()
                        .any(|x| int.int.rd.part.contains(x));
                    if !is_vc1902 {
                        println!("MISSING DPLL FOR UNK PART {part}", part = int.int.rd.part);
                    }
                }
            }
        }
    }
    for (tt, kind) in [
        ("ILKN_TILE", HardRowKind::Ilkn),
        ("DCMAC_TILE", HardRowKind::Dcmac),
        ("HSC_TILE", HardRowKind::Hsc),
    ] {
        for (x, y) in int.find_tiles(&[tt]) {
            let tile = &int.int.rd.tiles[&Coord {
                x: x as u16,
                y: y as u16,
            }];
            let col = int.lookup_column_inter(x) - 1;
            let reg = RegId::from_idx(int.lookup_row(y).to_idx() / 48);
            if tile.sites.iter().next().is_none() {
                disabled.insert(DisabledPart::HardIpSite(die, col, reg));
            }
            hard_cells.insert((col, reg), kind);
        }
    }
    let mut cols_hard = Vec::new();
    let cols: BTreeSet<ColId> = hard_cells.keys().map(|&(c, _)| c).collect();
    for col in cols {
        res[col].kind = ColumnKind::Hard;
        res[col + 1].kind = ColumnKind::ContHard;
        let mut regs = EntityVec::new();
        for _ in 0..(int.int.rows.len() / 48) {
            regs.push(HardRowKind::None);
        }
        for (&(c, r), &kind) in hard_cells.iter() {
            if c == col {
                assert_eq!(regs[r], HardRowKind::None);
                regs[r] = kind;
            }
        }
        cols_hard.push(HardColumn { col, regs });
    }

    let col_e = res.last_id().unwrap();
    if !int.find_tiles(&["HNICX_TILE"]).is_empty() {
        res[col_e].kind = ColumnKind::HNicX;
    } else if !int.find_tiles(&["AMS_SAT_GT_BOT_TILE"]).is_empty() {
        res[col_e].kind = ColumnKind::Gt;
    } else if !int.find_tiles(&["RCLK_CIDB_CORE"]).is_empty() {
        res[col_e].kind = ColumnKind::TermSsit;
    } else {
        res[col_e].kind = ColumnKind::Term;
    };

    (res, cols_hard)
}

fn get_cols_vbrk(int: &IntGridWrapper) -> BTreeSet<ColId> {
    let mut res = BTreeSet::new();
    for c in int.find_columns(&["CBRK_LOCF_TOP_TILE", "CBRK_TOP_TILE"]) {
        res.insert(int.lookup_column_inter(c));
    }
    res
}

fn get_cols_cpipe(int: &IntGridWrapper) -> BTreeSet<ColId> {
    let mut res = BTreeSet::new();
    for c in int.find_columns(&["CPIPE_TOP_TILE"]) {
        res.insert(int.lookup_column_inter(c));
    }
    res
}

fn get_rows_gt_w(int: &IntGridWrapper) -> EntityVec<RegId, GtRowKind> {
    let mut res = EntityVec::new();
    for _ in 0..(int.int.rows.len() / 48) {
        res.push(GtRowKind::None);
    }
    for (tkn, kind) in [
        ("GTY_QUAD_SINGLE_MY", GtRowKind::Gty),
        ("GTYP_QUAD_SINGLE_MY", GtRowKind::Gtyp),
        ("GTM_QUAD_SINGLE_MY", GtRowKind::Gtm),
        ("GTMPW_QUAD_SINGLE_MY", GtRowKind::Gtm2),
        ("XRAM_CORE", GtRowKind::Xram),
    ] {
        for row in int.find_rows(&[tkn]) {
            let oob = if int.int.mirror_y {
                row < *int.int.rows.last().unwrap()
            } else {
                row > *int.int.rows.last().unwrap()
            };
            if !oob {
                let reg = RegId::from_idx(int.lookup_row(row).to_idx() / 48);
                res[reg] = kind;
            }
        }
    }
    res
}

fn get_rows_gt_e(int: &IntGridWrapper) -> EntityVec<RegId, GtRowKind> {
    let mut res = EntityVec::new();
    for _ in 0..(int.int.rows.len() / 48) {
        res.push(GtRowKind::None);
    }
    for (tkn, kind) in [
        ("GTY_QUAD_SINGLE", GtRowKind::Gty),
        ("GTYP_QUAD_SINGLE", GtRowKind::Gtyp),
        ("GTM_QUAD_SINGLE", GtRowKind::Gtm),
        ("GTMPW_QUAD_SINGLE", GtRowKind::Gtm2),
        ("VDU_CORE_MY", GtRowKind::Vdu),
        ("BFR_TILE_B_BOT_CORE", GtRowKind::BfrB),
        ("BFR_TILE_B_TOP_CORE", GtRowKind::BfrB),
        ("ISP2_CORE", GtRowKind::Isp2),
        ("VCU2_TILE", GtRowKind::Vcu2),
        ("RFADC_BOT_CORE", GtRowKind::RfAdc),
        ("RFDAC_BOT_CORE", GtRowKind::RfDac),
    ] {
        for row in int.find_rows(&[tkn]) {
            let reg = RegId::from_idx(int.lookup_row(row).to_idx() / 48);
            res[reg] = kind;
        }
    }
    res
}

pub fn get_fabric(rd: &Part) -> FabricKind {
    if rd.tile_kinds.contains_key("CLE_W_VR_CORE") {
        FabricKind::Ulvt
    } else if rd.slot_kinds.contains("GCLK_DELAY_SSIT2") {
        FabricKind::Ssit2
    } else if rd.slot_kinds.contains("GCLK_DELAY_SSIT") {
        FabricKind::Ssit
    } else if !rd.wires.contains("PSS_BASE_ATOM_0_MCLK") {
        FabricKind::Rev2
    } else {
        FabricKind::Orig
    }
}

fn get_hsr_xpio(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    edge: DirV,
) -> XpioRow {
    let mut pnoc_name = None;
    let row = match edge {
        DirV::S => RowId::from_idx(int.int.rows.len()),
        DirV::N => RowId::from_idx(int.int.rows.len() + 1),
    };
    if edge == DirV::S {
        for xy in int.find_tiles_crd(&["NOC_PNOC_MONO_CORE"]) {
            let tile = &int.int.rd.tiles[&xy];
            pnoc_name = Some(tile.name.clone());
            let tk = &int.int.rd.tile_kinds[tile.kind];
            for (sid, &slot, _) in &tk.sites {
                let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                    continue;
                };
                let sy = sy as usize;
                let (idx, bslot) = match int.int.rd.slot_kinds[slot].as_str() {
                    "NOC_NMU128" if sy < 4 => (0, bslots::HNOC_PS_NMU[sy]),
                    "NOC_NMU128" => (1, bslots::HNOC_PS_NMU[sy - 4]),
                    "NOC_NSU128" if sy < 2 => (0, bslots::HNOC_PS_NSU[sy]),
                    "NOC_NSU128" => (1, bslots::HNOC_PS_NSU[sy - 2]),
                    "NPI_NIR" => (0, bslots::HNOC_NPI_NIR),
                    _ => continue,
                };
                dev_naming.bel_names.insert(
                    die.cell(ColId::from_idx(idx * 9), row).bel(bslot),
                    vec![tile.sites[sid].clone()],
                );
            }
        }
    }
    let mut nibble_bli = vec![];
    for (i, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["XPIO_NIBBLE_SC", "XPIO_NIBBLE_SC_MY"],
            DirV::N => ["XPIO_NIBBLE_SC_MX", "XPIO_NIBBLE_SC_R180"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(i);
        let tcrd = die.cell(col, row).tile(tslots::HSR_IO);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "IOB" => bslots::XPIOB[sy * 2],
                "XPIOLOGIC" => bslots::XPIOLOGIC[sy * 2],
                "XPHY" => bslots::XPHY,
                "XPIO_VREF" => bslots::XPIO_VREF,
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        nibble_bli.push(int.xlat_bli(xy, "XPIO_LOGICPAIR_0_IDLYM_CNTVALUEIN_0_"));
    }

    let mut clk_rows = HashSet::new();
    let mut clk_test_xlat = HashMap::new();
    let mut clk_bli = vec![];
    let mut clk_route_col = vec![];
    let mut clk_distr_col = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["CLK_REBUF_BUFGS_HSR_CORE"],
            DirV::N => ["CLK_REBUF_BUFGS_HSR_CORE_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        clk_rows.insert(xy.y);
        clk_rows.insert(xy.y + 1);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "BUFG_HSR_TEST" => bslots::BUFG_HSR_TEST[sy],
                "BUFGCTRL" => bslots::BUFGCTRL[sy],
                "BUFGCE" => bslots::BUFGCE[sy],
                "BUFGCE_DIV" => bslots::BUFGCE_DIV[sy],
                "GCLK_TEST" => bslots::GCLK_TEST_HSR[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let bli = if let Some(col) = int.xlat_bli(xy, "CLK_BUFGCTRL_WITH_OPTINVS_64_IGNORE0") {
            XpioClkBli::Full(col)
        } else {
            XpioClkBli::Minimal(int.xlat_bli(xy, "CLK_BUFGCE_18_CE").unwrap())
        };
        clk_bli.push(bli);
        clk_route_col.push(int.xlat_hsr_clk(xy, "CLK_HROUTE_1_0"));
        clk_distr_col.push(int.xlat_hsr_clk(xy, "CLK_HDISTR_1_0"));
        if let Some(nw) = int.int.rd.lookup_wire(xy, "CLK_IN_TEST0") {
            clk_test_xlat.insert(nw, idx);
        }
    }

    let mut mmcm_bli = vec![];
    for (idx, (xy, tname, bname)) in int
        .get_simple_names(&match edge {
            DirV::S => ["CMT_MMCM"],
            DirV::N => ["CMT_MMCM_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_MMCM);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::MMCM), vec![bname]);
        dev_naming.tile_names.insert(tcrd, vec![tname]);
        mmcm_bli.push(int.xlat_bli(xy, "CMT_MMCM_TOP_0_PSCLK").unwrap());
    }

    let mut dpll_bli = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["CMT_DPLL"],
            DirV::N => ["CMT_DPLL_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_DPLL);
        if !clk_rows.contains(&xy.y) {
            continue;
        }
        let tile = &int.int.rd.tiles[&xy];
        let bname = tile.sites.values().next().unwrap();
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::DPLL_HSR), vec![bname.clone()]);
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        dpll_bli.push(int.xlat_bli(xy, "CMT_DPLL_TOP_0_PSCLK").unwrap());
    }

    let mut xpll_names = vec![];
    for xy in int.find_tiles_crd(&match edge {
        DirV::S => ["CMT_XPLL"],
        DirV::N => ["CMT_XPLL_MX"],
    }) {
        let tile = &int.int.rd.tiles[&xy];
        let bname = tile.sites.values().next().unwrap();
        xpll_names.push((tile.name.clone(), bname.clone()));
    }

    let mut xpll_bli = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["CLK_PLL_AND_PHY"],
            DirV::N => ["CLK_PLL_AND_PHY_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_XPLL);
        let tile = &int.int.rd.tiles[&xy];
        let (xtn0, xbn0) = xpll_names[idx * 2].clone();
        let (xtn1, xbn1) = xpll_names[idx * 2 + 1].clone();
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::XPLL[0]), vec![xbn0]);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::XPLL[1]), vec![xbn1]);
        dev_naming
            .tile_names
            .insert(tcrd, vec![tile.name.clone(), xtn0, xtn1]);
        xpll_bli.push(int.xlat_bli(xy, "PLL1_CLKFB_DESKEW0").unwrap());
    }

    let mut dci_bli = vec![];
    let mut scan_bli = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["XPIO_MISC_CORE"],
            DirV::N => ["XPIO_MISC_CORE_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_BANK);
        let tile = &int.int.rd.tiles[&xy];
        let bname = tile.sites.values().next().unwrap();
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::XPIO_DCI), vec![bname.clone()]);
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let (col, w) = int.xlat_bli_full(xy, "DCI_OK_TO_UPDATE").unwrap();
        dci_bli.push(match w {
            "BLI_GRP0_C_BLI_IMUX0" => XpioTestBli::PrimaryA(col),
            "BLI_GRP0_B_BLI_IMUX0" => XpioTestBli::PrimaryB(col),
            "BLI_GRP0_A_BLI_IMUX0" => XpioTestBli::PrimaryC(col),
            "BLI_GRP0_A_BLI_IMUX15" => XpioTestBli::Secondary(col),
            _ => panic!("ummm {w}"),
        });
        let (col, w) = int.xlat_bli_full(xy, "CLB2MISC_SCAN_EN_B").unwrap();
        scan_bli.push(match w {
            "BLI_GRP2_A_BLI_IMUX12" => XpioTestBli::PrimaryA(col),
            "BLI_GRP2_B_BLI_IMUX12" => XpioTestBli::PrimaryB(col),
            "BLI_GRP2_C_BLI_IMUX12" => XpioTestBli::PrimaryC(col),
            "BLI_GRP3_A_BLI_CTRL2" => XpioTestBli::Secondary(col),
            _ => panic!("ummm {w}"),
        });
    }

    let width = scan_bli.len();
    assert_eq!(nibble_bli.len(), width * 9);
    assert_eq!(clk_bli.len(), width);
    assert_eq!(mmcm_bli.len(), width);
    assert_eq!(dpll_bli.len(), width);
    assert_eq!(xpll_names.len(), width * 2);

    let mut regions = EntityVec::new();
    for i in 0..width {
        regions.push(XpioRegion {
            xpio_bli: core::array::from_fn(|j| nibble_bli[i * 9 + j]),
            clk_bli: clk_bli[i],
            mmcm_bli: mmcm_bli[i],
            dpll_bli: dpll_bli[i],
            xpll_bli: xpll_bli[i],
            dci_bli: dci_bli[i],
            scan_bli: scan_bli[i],
            ddrmc_bli: None,
            sysmon_bli: None,
            clk_route_col: clk_route_col[i],
            clk_distr_col: clk_distr_col[i],
            vnoc: [None; 2],
        });
    }

    for ((xy, tname, bname), (_, riu_tname, riu_bname)) in int
        .get_simple_names(&match edge {
            DirV::S => ["DDRMC_DMC_CORE"],
            DirV::N => ["DDRMC_DMC_CORE_MX"],
        })
        .into_iter()
        .zip(int.get_simple_names(&match edge {
            DirV::S => ["DDRMC_RIU_CORE"],
            DirV::N => ["DDRMC_RIU_CORE_MX"],
        }))
    {
        let nw = int
            .int
            .rd
            .lookup_wire(xy, "DDRMC_MAIN_0_TEST_CLK_OUT_0_")
            .unwrap();
        let idx = clk_test_xlat[&nw];
        let col = ColId::from_idx(idx * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_DDRMC);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::DDRMC), vec![bname, riu_bname]);
        dev_naming.tile_names.insert(tcrd, vec![tname, riu_tname]);
        let col0 = int
            .xlat_bli(xy, "DDRMC_MAIN_0_IF_UBLAZE_FABRIC_UB_MISC_OUT_0_")
            .unwrap();
        let col1 = int
            .xlat_bli(xy, "DDRMC_MAIN_0_IF_DMC_FABRIC_REF_ACK_1_0_")
            .unwrap();
        regions[HRegId::from_idx(idx)].ddrmc_bli = Some([col0, col1]);
    }

    let mut has_sysmon = regions.map(|_, _| true);
    for (reg, rd) in &regions {
        if rd.ddrmc_bli.is_some() {
            has_sysmon[reg - 1] = false;
            has_sysmon[reg] = false;
        }
    }
    let sysmon_regs = Vec::from_iter(
        has_sysmon
            .into_iter()
            .filter_map(|(reg, need)| if need { Some(reg) } else { None }),
    );

    for (smidx, xy) in int
        .find_tiles_crd(&match edge {
            DirV::S => ["HSM_AMS_SAT_LC"],
            DirV::N => ["HSM_AMS_SAT_LC_MX"],
        })
        .into_iter()
        .enumerate()
    {
        let reg = sysmon_regs[smidx];
        let col = ColId::from_idx(reg.to_idx() * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_SYSMON_SAT);
        let tile = &int.int.rd.tiles[&xy];
        if let Some(bname) = tile.sites.values().next() {
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslots::SYSMON_SAT_HSR), vec![bname.clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let col = int.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_FABRICCLK").unwrap();
        regions[reg].sysmon_bli = Some(col);
    }

    let mut ncrb_xlat = HashMap::new();
    if edge == DirV::N {
        for xy in int.find_tiles_crd(&["NOC_XPIO_NCRB_CORE_MX"]) {
            let ni = int.int.rd.lookup_wire(xy, "NOC_NCRB_TOP_0_IN_0").unwrap();
            let NodeOrWire::Node(ni) = ni else {
                unreachable!()
            };
            let vnoc_xy = int
                .int
                .rd
                .iter_node(ni)
                .find(|&(cxy, _)| cxy != xy)
                .unwrap()
                .0;
            let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
            let ni = int.int.rd.lookup_wire(xy, "NOC_NCRB_TOP_1_IN_0").unwrap();
            ncrb_xlat.insert(ni, VNocConn(col, 0));
            let ni = int.int.rd.lookup_wire(xy, "NOC_NCRB_TOP_1_IN_1").unwrap();
            ncrb_xlat.insert(ni, VNocConn(col, 1));
            let tcrd = die.cell(col, row - 2).tile(tslots::HNOC_NCRB);
            let tile = &int.int.rd.tiles[&xy];
            let tk = &int.int.rd.tile_kinds[tile.kind];
            for (sid, &slot, _) in &tk.sites {
                let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                    continue;
                };
                let sy = sy as usize;
                let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                    "NOC_NCRB" => bslots::HNOC_NCRB[sy],
                    _ => continue,
                };
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
            }
            dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        }
    }

    let mut nps_names = vec![];
    for (idx, xy) in int
        .find_tiles_crd(match edge {
            DirV::S => ["NOC_HNOC_S3_PS_CORE", "NOC_HNOC_S3_PL_CORE"].as_slice(),
            DirV::N => ["NOC_HNOC_S3_PL_CORE_MX"].as_slice(),
        })
        .into_iter()
        .enumerate()
    {
        let reg = HRegId::from_idx(idx / 2);
        let col = ColId::from_idx(idx / 2 * 9);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        let is_ps = int.int.rd.tile_kinds.key(tile.kind).contains("_PS_");
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS5555" => {
                    if is_ps {
                        bslots::HNOC_NPS[sy + 2 * (idx % 2)]
                    } else {
                        if sy == 1 {
                            bslots::HNOC_NPS_MID[idx % 2]
                        } else {
                            bslots::HNOC_NPS[sy / 2 + 2 * (idx % 2)]
                        }
                    }
                }
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        nps_names.push(&tile.name);
        if !is_ps {
            let ni = int
                .int
                .rd
                .lookup_wire(
                    xy,
                    ["NOC_NPS5555_TOP_1_OUT_0", "NOC_NPS5555_TOP_1_OUT_2"][idx % 2],
                )
                .unwrap();
            if edge == DirV::S {
                let NodeOrWire::Node(ni) = ni else {
                    unreachable!()
                };
                if let Some((vnoc_xy, wi)) = int.int.rd.iter_node(ni).find(|&(cxy, _)| cxy != xy) {
                    let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
                    let lane = match int.int.rd.wires[wi].as_str() {
                        "NOC_NPS_VNOC_ATOM_0_IN_2" => 0,
                        "NOC_NPS_VNOC_ATOM_0_IN_0" => 1,
                        x => panic!("um {x}"),
                    };
                    let conn = VNocConn(col, lane);
                    regions[reg].vnoc[idx % 2] = Some(conn);
                }
            } else {
                if let Some(&conn) = ncrb_xlat.get(&ni) {
                    regions[reg].vnoc[idx % 2] = Some(conn);
                }
            }
        }
    }

    assert_eq!(nps_names.len(), width * 2);
    for i in 0..width {
        let col = ColId::from_idx(i * 9);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        if edge == DirV::S && i < 3 {
            dev_naming.tile_names.insert(
                tcrd,
                vec![
                    nps_names[2 * i].clone(),
                    nps_names[2 * i + 1].clone(),
                    pnoc_name.clone().unwrap(),
                ],
            );
        } else {
            dev_naming.tile_names.insert(
                tcrd,
                vec![nps_names[2 * i].clone(), nps_names[2 * i + 1].clone()],
            );
        }
    }

    for (_xy, tname, bname) in int.get_simple_names(&["NOC2_HNOC_NIDB_CORE"]) {
        let col = ColId::from_idx(width * 9);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::HNOC_NIDB_H), vec![bname]);
        dev_naming.tile_names.insert(tcrd, vec![tname]);
    }

    for xy in int.find_tiles_crd(&["CLKE2_REBUF_HSR_EDGE_CORE"]) {
        let col = ColId::from_idx(width * 9);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        let tile = &int.int.rd.tiles[&xy];
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
    }

    XpioRow { regions }
}

fn name_noc2(
    rd: &Part,
    dev_naming: &mut DeviceNaming,
    tcrd: TileCoord,
    tiles: impl IntoIterator<Item = (Coord, BelSlotId)>,
) {
    let mut tnames = vec![];
    for (xy, bslot) in tiles {
        let tile = &rd.tiles[&xy];
        let bname = tile.sites.values().next().unwrap();
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslot), vec![bname.clone()]);
        tnames.push(tile.name.clone());
    }
    dev_naming.tile_names.insert(tcrd, tnames);
}

fn get_hsr_x5io_clk(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
) -> (Vec<X5ioClk>, HashMap<NodeOrWire, X5ioDdrmcTestClk>) {
    let row = RowId::from_idx(int.int.rows.len());

    let mut clk_rows = HashSet::new();
    let mut clk_bli = vec![];
    let mut clk_route_col = vec![];
    let mut clk_distr_col = vec![];
    let mut clk_test_xlat = HashMap::new();
    let mut clk_tiles = int.find_tiles_crd(&["CLK_REBUF_BUFGS_HSR_CORE"]);
    clk_tiles.sort_by_key(|xy| xy.x);
    for (idx, xy) in clk_tiles.into_iter().enumerate() {
        let col = ColId::from_idx(idx * 8);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        clk_rows.insert(xy.y);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "BUFG_HSR_TEST" => bslots::BUFG_HSR_TEST[sy],
                "BUFGCTRL" => bslots::BUFGCTRL[sy],
                "BUFGCE" => bslots::BUFGCE[sy],
                "BUFGCE_DIV" => bslots::BUFGCE_DIV[sy],
                "GCLK_TEST" => bslots::GCLK_TEST_HSR[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let bli = if let Some(col) = int.xlat_bli(xy, "CLK_BUFGCTRL_WITH_OPTINVS_64_IGNORE0") {
            X5ioClkBli::Full(col)
        } else {
            let (col, w) = int.xlat_bli_full(xy, "IF_BUFGS_FABRIC_CLK_INT").unwrap();
            match w {
                "BLI_GRP3_B_BLI_CTRL2" => X5ioClkBli::MinimalA(col),
                "BLI_GRP3_B_BLI_CTRL1" => X5ioClkBli::MinimalB(col),
                _ => unreachable!(),
            }
        };
        clk_bli.push(bli);
        clk_route_col.push(int.xlat_hsr_clk(xy, "CLK_HROUTE_1_0"));
        clk_distr_col.push(int.xlat_hsr_clk(xy, "CLK_HDISTR_1_0"));
        let reg = HRegId::from_idx(idx);
        if let Some(nw) = int.int.rd.lookup_wire(xy, "CLK_IN_TEST0") {
            clk_test_xlat.insert(nw, X5ioDdrmcTestClk::A(reg));
        }
        if let Some(nw) = int.int.rd.lookup_wire(xy, "CLK_IN_TEST2") {
            clk_test_xlat.insert(nw, X5ioDdrmcTestClk::B(reg));
        }
    }

    let mut mmcm_bli = vec![];
    for (idx, (xy, tname, bname)) in int.get_simple_names(&["CMT_MMCM"]).into_iter().enumerate() {
        let col = ColId::from_idx(idx * 8);
        let tcrd = die.cell(col, row).tile(tslots::HSR_MMCM);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::MMCM), vec![bname]);
        dev_naming.tile_names.insert(tcrd, vec![tname]);
        mmcm_bli.push(int.xlat_bli(xy, "CMT_MMCM_TOP_0_PSCLK").unwrap());
    }

    let mut dpll_bli = vec![];
    for (idx, (xy, tname, bname)) in int.get_simple_names(&["CMT_DPLL"]).into_iter().enumerate() {
        let col = ColId::from_idx(idx * 8);
        let tcrd = die.cell(col, row).tile(tslots::HSR_DPLL);
        if !clk_rows.contains(&xy.y) {
            continue;
        }
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::DPLL_HSR), vec![bname]);
        dev_naming.tile_names.insert(tcrd, vec![tname]);
        dpll_bli.push(int.xlat_bli(xy, "CMT_DPLL_TOP_0_PSCLK"));
    }

    let width = clk_bli.len();
    assert_eq!(mmcm_bli.len(), width);
    assert_eq!(dpll_bli.len(), width);

    let mut res = vec![];
    for i in 0..width {
        res.push(X5ioClk {
            clk_bli: clk_bli[i],
            mmcm_bli: mmcm_bli[i],
            dpll_bli: dpll_bli[i],
            clk_route_col: clk_route_col[i],
            clk_distr_col: clk_distr_col[i],
        });
    }
    (res, clk_test_xlat)
}

fn get_hsr_x5io_io(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    clk_test_xlat: HashMap<NodeOrWire, X5ioDdrmcTestClk>,
) -> Vec<X5io> {
    let row = RowId::from_idx(int.int.rows.len());
    let mut io = vec![];
    for (idx, cmu_xy) in int
        .find_tiles_crd(&["CMPHY_CMU_X32_CORE", "CMPHY_CMU_X32_CORE_MY"])
        .into_iter()
        .enumerate()
    {
        let col = if idx.is_multiple_of(2) {
            ColId::from_idx(idx * 4 + 3)
        } else {
            ColId::from_idx(idx * 4)
        };
        let cell = die.cell(col, row);
        let dfx_xy = int
            .follow_wire(cmu_xy, "X5PHIO_CMU_X32_INTF_1_MUX2APB_PCLK_CMU")
            .unwrap();
        let clk_xy = int.follow_wire(cmu_xy, "PHY2CLK_GCLK").unwrap();
        let io_xy = int
            .follow_wire(cmu_xy, "X5PHIO_CMU_X32_INTF_1_APB2CMU_CK")
            .unwrap();
        let pll_xy = int.follow_wire(cmu_xy, "X5PLL_INTF_0_APB2PLL_RST").unwrap();
        let xcv_xy = int.follow_wire(clk_xy, "CLK2XCV_PDQS_DIST_OCTAD0").unwrap();

        // X5IO_BANK
        dev_naming.tile_names.insert(
            cell.tile(tslots::HSR_BANK),
            vec![
                int.int.rd.tiles[&cmu_xy].name.clone(),
                int.int.rd.tiles[&io_xy].name.clone(),
            ],
        );
        dev_naming.bel_names.insert(
            cell.bel(bslots::X5PHIO_CMU),
            vec![
                int.int
                    .rd
                    .site_name_xy(cmu_xy, "X5PHIO_CMUIF", 0, 0)
                    .unwrap()
                    .to_string(),
                int.int
                    .rd
                    .site_name_xy(io_xy, "X5PHIO_CMU_X32", 0, 0)
                    .unwrap()
                    .to_string(),
            ],
        );
        let bank_bli = int.xlat_bli(cmu_xy, "X5PHIO_CMU_X32_INTF_1_CLB2APB_CK");

        // X5IO_SCAN
        dev_naming.tile_names.insert(
            cell.tile(tslots::HSR_SCAN),
            vec![int.int.rd.tiles[&dfx_xy].name.clone()],
        );
        dev_naming.bel_names.insert(
            cell.bel(bslots::X5IO_SCAN),
            vec![
                int.int
                    .rd
                    .site_name_xy(dfx_xy, "CMPHY_DFX_HB", 0, 0)
                    .unwrap()
                    .to_string(),
            ],
        );
        let scan_bli = match int
            .xlat_bli_full(dfx_xy, "CMPHY_DFX_HB_X32_0_SCAN_CLK_N_EXT")
            .unwrap()
        {
            (col, "BLI_GRP3_B_BLI_CTRL2") => X5ioScanBli::A(col),
            (col, "BLI_GRP3_A_BLI_CTRL2") => X5ioScanBli::B(col),
            (col, "BLI_GRP3_C_BLI_CTRL2") => X5ioScanBli::C(col),
            _ => unreachable!(),
        };

        // X5PLL
        dev_naming.tile_names.insert(
            cell.tile(tslots::HSR_XPLL),
            vec![
                int.int.rd.tiles[&cmu_xy].name.clone(),
                int.int.rd.tiles[&clk_xy].name.clone(),
                int.int.rd.tiles[&pll_xy].name.clone(),
                int.int.rd.tiles[&dfx_xy].name.clone(),
            ],
        );
        dev_naming.bel_names.insert(
            cell.bel(bslots::X5PLL),
            vec![
                int.int
                    .rd
                    .site_name_xy(pll_xy, "X5PLL", 0, 0)
                    .unwrap()
                    .to_string(),
                int.int
                    .rd
                    .site_name_xy(pll_xy, "X5PLL_S2P", 0, 0)
                    .unwrap()
                    .to_string(),
                int.int
                    .rd
                    .site_name_xy(cmu_xy, "X5PLL_INTF", 0, 0)
                    .unwrap()
                    .to_string(),
            ],
        );
        for i in 0..8 {
            dev_naming.bel_names.insert(
                cell.bel(bslots::X5PHIO_DCCIBUF[i]),
                vec![
                    int.int
                        .rd
                        .site_name_xy(clk_xy, "X5PHIO_DCCIBUF", 0, i)
                        .unwrap()
                        .to_string(),
                ],
            );
            dev_naming.bel_names.insert(
                cell.bel(bslots::X5PHIO_HARD_INV[i]),
                vec![
                    int.int
                        .rd
                        .site_name_xy(clk_xy, "X5PHIO_HARD_INV", 0, i)
                        .unwrap()
                        .to_string(),
                ],
            );
        }

        let pll_bli = if let Some((col_bli, wn)) = int.xlat_bli_full(pll_xy, "X5PLL_VPLL_TOP_1_RST")
        {
            match wn {
                "BLI_GRP3_A_BLI_CTRL1" => X5ioPllBli::Secondary(col_bli),
                "BLI_GRP4_D_BLI_IMUX_BYP11" => X5ioPllBli::PrimaryA(col_bli),
                "BLI_GRP4_D_BLI_IMUX_BYP8" => X5ioPllBli::PrimaryB(col_bli),
                _ => unreachable!(),
            }
        } else {
            X5ioPllBli::None
        };

        let mut octad_bli = [None; 4];
        for i in 0..4 {
            let col = if idx.is_multiple_of(2) {
                ColId::from_idx(idx * 4 + i)
            } else {
                ColId::from_idx(idx * 4 + 3 - i)
            };
            let tcrd = die.cell(col, row).tile(tslots::HSR_IO);
            let phy_xy = int
                .follow_wire(
                    xcv_xy,
                    &format!("X5PHIOBANK_XCVR_X2_{ii}_XCV2PHY_RD_DQ_0_", ii = i * 4),
                )
                .unwrap();
            let mut bnames = vec![
                int.int
                    .rd
                    .site_name_xy(phy_xy, "CMPHY_OCTAD", 0, 0)
                    .unwrap()
                    .to_string(),
            ];
            for j in 0..4 {
                bnames.push(
                    int.int
                        .rd
                        .site_name_xy(xcv_xy, "IOB", 0, i * 4 + j)
                        .unwrap()
                        .to_string(),
                );
            }
            let tnames = vec![
                int.int.rd.tiles[&phy_xy].name.clone(),
                int.int.rd.tiles[&xcv_xy].name.clone(),
            ];
            dev_naming.tile_names.insert(tcrd, tnames);
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslots::X5IO_OCTAD), bnames);
            octad_bli[i] = int.xlat_bli(phy_xy, "CMPHY_OCTAD_0_D0_0_");
        }

        let ddrmc = int.follow_wire(clk_xy, "CLK2PHY_DMC_CLK").map(|mc_xy| {
            let tkid = int.int.rd.tiles[&mc_xy].kind;
            let (kind, bk) = match int.int.rd.tile_kinds.key(tkid).as_str() {
                "DDRMC5_DMC_CORE" | "DDRMC5_DMC_CORE_MY" => (X5ioDdrmcKind::Ddrmc5, "DDRMC5"),
                "DDRMC5C_DMC_CORE" | "DDRMC5C_DMC_CORE_MY" => (X5ioDdrmcKind::Ddrmc5C, "DDRMC5C"),
                "DDRMC5E_DMC_CORE" | "DDRMC5E_DMC_CORE_MY" => (
                    if int
                        .int
                        .rd
                        .lookup_wire(mc_xy, "DDRMC5E_DMC_SITE_0_PLL2_CLK_PIN")
                        .is_some()
                    {
                        X5ioDdrmcKind::Ddrmc5E2
                    } else if int
                        .xlat_bli_full(mc_xy, "DDRMC5E_DMC_SITE_0_IF_UBLAZE_FABRIC_BLI2UB_CLK")
                        .unwrap()
                        .1
                        == "BLI_GRP4_D_BLI_IMUX_BYP4"
                    {
                        X5ioDdrmcKind::Ddrmc5EAlt
                    } else {
                        X5ioDdrmcKind::Ddrmc5E
                    },
                    "DDRMC5E",
                ),
                x => panic!("umm {x}"),
            };
            let bli = int
                .xlat_bli(
                    mc_xy,
                    &format!("{bk}_DMC_SITE_0_IF_DMC_FABRIC_BLI2UB_UART_RX"),
                )
                .unwrap();
            dev_naming.tile_names.insert(
                cell.tile(tslots::HSR_DDRMC),
                vec![int.int.rd.tiles[&mc_xy].name.clone()],
            );
            dev_naming.bel_names.insert(
                cell.bel(bslots::DDRMC5),
                vec![
                    int.int.rd.tiles[&mc_xy]
                        .sites
                        .values()
                        .next()
                        .unwrap()
                        .clone(),
                ],
            );
            let test_clk = clk_test_xlat[&int
                .int
                .rd
                .lookup_wire(mc_xy, &format!("{bk}_DMC_SITE_0_TEST_CLK_OUT_0_"))
                .unwrap()];
            X5ioDdrmc {
                kind,
                bli,
                test_clk,
            }
        });

        io.push(X5io {
            octad_bli,
            pll_bli,
            bank_bli,
            scan_bli,
            ddrmc,
        });
    }
    io
}

fn get_hsr_x5io_hnoc(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    io: &[X5io],
) -> Vec<X5ioHNoc> {
    let row = RowId::from_idx(int.int.rows.len());

    let npi_nir = int.find_tiles_crd(&["NPI2_NIR_TILE"])[0];
    let ps9 = int.find_tiles_crd(&["PSS_BASE_CORE"]).first().copied();
    let psxl = int.find_tiles_crd(&["PSXL_CORE"]).first().copied();
    let psxc = int.find_tiles_crd(&["PSXC_TILE"]).first().copied();

    let scans = Vec::from_iter(
        int.find_tiles_crd(&["NOC2_SCAN_TOP", "NOC2_SCAN_TOP_MY"])
            .into_iter()
            .filter(|&xy| xy.y < 9),
    );
    let mut scans_row = [const { vec![] }; 3];
    for xy in scans {
        scans_row[(xy.y - 6) as usize].push(xy);
    }

    let mut res = vec![];

    let mut chain;
    let scans;
    if let Some(ps9) = ps9 {
        let pss = if int.int.rd.wires.contains("PSS_BASE_L80_ATOM_0_PCIE_NOC_0") {
            "PSS_BASE_L80_ATOM_0"
        } else {
            "PSS_BASE_ATOM_0"
        };
        let nmu = [
            int.follow_wire(ps9, &format!("{pss}_PCIE_NOC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_PCIE_NOC_1")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_PMC_NOC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_RPU_NOC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_CCI_NOC_3")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_CCI_NOC_1")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_CCI_NOC_2")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_CCI_NOC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NCI_NOC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NCI_NOC_1")).unwrap(),
        ];
        let nsu = [
            int.follow_wire(ps9, &format!("{pss}_NOC_PCIE_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NOC_PMC_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NOC_NCI_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NOC_NCI_1")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NOC_CCI_0")).unwrap(),
            int.follow_wire(ps9, &format!("{pss}_NOC_CCI_1")).unwrap(),
        ];

        let nps = [
            int.follow_wire(nmu[3], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[2], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[0], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
        ];
        let cell = die.cell(ColId::from_idx(0), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (npi_nir, bslots::HNOC_NPI_NIR),
                (nmu[0], bslots::HNOC_PS_NMU[0]),
                (nmu[1], bslots::HNOC_PS_NMU[1]),
                (nmu[2], bslots::HNOC_PS_NMU[2]),
                (nmu[3], bslots::HNOC_PS_NMU[3]),
                (nsu[0], bslots::HNOC_PS_NSU[0]),
                (nsu[1], bslots::HNOC_PS_NSU[1]),
                (nps[0], bslots::HNOC_NPS[0]),
                (nps[1], bslots::HNOC_NPS[1]),
                (nps[2], bslots::HNOC_NPS[2]),
                (scans_row[1][0], bslots::HNOC_SCAN[0]),
            ],
        );

        let nps = [
            int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[1], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_3")
                .unwrap(),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
        ];
        let xbr = int
            .follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_1")
            .unwrap();
        let cell = die.cell(ColId::from_idx(4), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (nps[0], bslots::HNOC_NPS[0]),
                (nps[1], bslots::HNOC_NPS[1]),
                (nps[2], bslots::HNOC_NPS[2]),
                (nps[3], bslots::HNOC_NPS[3]),
                (nmu[4], bslots::HNOC_PS_NMU[0]),
                (nmu[5], bslots::HNOC_PS_NMU[1]),
                (xbr, bslots::HNOC_XBR4X2),
                (scans_row[0][0], bslots::HNOC_SCAN[0]),
            ],
        );

        let nps = [
            int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[1], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[3], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
        ];
        let nps2 = [
            int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[1], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
            int.follow_wire(nps[3], "NOC2_NPS5555_ATOM_0_OUT_2")
                .unwrap(),
        ];
        let xbr = int
            .follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_1")
            .unwrap();
        let cell = die.cell(ColId::from_idx(16), row);
        let scan_bli = int
            .xlat_bli(
                scans_row[2][0],
                "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
            )
            .unwrap();
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (nps[0], bslots::HNOC_NPS[0]),
                (nps[1], bslots::HNOC_NPS[1]),
                (nps[2], bslots::HNOC_NPS[2]),
                (nps[3], bslots::HNOC_NPS[3]),
                (nps2[0], bslots::HNOC_NPS[4]),
                (nps2[1], bslots::HNOC_NPS[5]),
                (nps2[2], bslots::HNOC_NPS[6]),
                (nps2[3], bslots::HNOC_NPS[7]),
                (nmu[6], bslots::HNOC_PS_NMU[0]),
                (nmu[7], bslots::HNOC_PS_NMU[1]),
                (nmu[8], bslots::HNOC_PS_NMU[2]),
                (nmu[9], bslots::HNOC_PS_NMU[3]),
                (nsu[2], bslots::HNOC_PS_NSU[0]),
                (nsu[3], bslots::HNOC_PS_NSU[1]),
                (nsu[4], bslots::HNOC_PS_NSU[2]),
                (nsu[5], bslots::HNOC_PS_NSU[3]),
                (xbr, bslots::HNOC_XBR4X2),
                (scans_row[2][0], bslots::HNOC_SCAN[0]),
                (scans_row[2][1], bslots::HNOC_SCAN[1]),
                (scans_row[2][2], bslots::HNOC_SCAN[2]),
            ],
        );

        chain = [
            int.follow_wire(nps2[0], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps2[1], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps2[2], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps2[3], "NOC2_NPS5555_ATOM_0_OUT_2"),
        ];
        scans = scans_row[1][1..].iter().copied();

        res.extend([
            X5ioHNoc::Ps9A,
            X5ioHNoc::Ps9B,
            X5ioHNoc::None,
            X5ioHNoc::None,
            X5ioHNoc::Ps9C { scan_bli },
        ]);
    } else {
        let pss = if psxl.is_some() {
            "PSXL_ATOM_0"
        } else {
            "PSXC_ATOM_0"
        };
        let psx = psxl.or(psxc).unwrap();
        let nmu = [
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_8")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_PMC_NOC_0")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_RPU_NOC_0")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_0")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_1")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_2")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_3")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_4")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_5")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_6")).unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_7")).unwrap(),
        ];
        let nps = int
            .follow_wire(nmu[0], "NOC2_NMU128_ATOM_0_TO_NOC")
            .unwrap();
        let nsu = [
            int.follow_wire(psx, &format!("{pss}_NOC_PMC_0")).unwrap(),
            int.follow_wire(nps, "NOC2_NPS5555_ATOM_0_OUT_0").unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_CCI_0"))
                .unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_CCI_1"))
                .unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_CCI_2"))
                .unwrap(),
            int.follow_wire(psx, &format!("{pss}_CMN_NOC_CCI_3"))
                .unwrap(),
        ];
        let cell = die.cell(ColId::from_idx(0), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (npi_nir, bslots::HNOC_NPI_NIR),
                (nmu[0], bslots::HNOC_PS_NMU[0]),
                (nmu[1], bslots::HNOC_PS_NMU[1]),
                (nmu[2], bslots::HNOC_PS_NMU[2]),
                (nsu[0], bslots::HNOC_PS_NSU[0]),
                (nsu[1], bslots::HNOC_PS_NSU[1]),
                (nps, bslots::HNOC_NPS[0]),
                (scans_row[1][0], bslots::HNOC_SCAN[0]),
            ],
        );

        let nps = [
            int.follow_wire(nmu[5], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[4], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[3], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[6], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
        ];
        let xbr = int
            .follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_1")
            .unwrap();
        let cell = die.cell(ColId::from_idx(4), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (nps[0], bslots::HNOC_NPS[0]),
                (nps[1], bslots::HNOC_NPS[1]),
                (nps[2], bslots::HNOC_NPS[2]),
                (nps[3], bslots::HNOC_NPS[3]),
                (nmu[3], bslots::HNOC_PS_NMU[0]),
                (nmu[4], bslots::HNOC_PS_NMU[1]),
                (nmu[5], bslots::HNOC_PS_NMU[2]),
                (nmu[6], bslots::HNOC_PS_NMU[3]),
                (xbr, bslots::HNOC_XBR4X2),
                (scans_row[0][0], bslots::HNOC_SCAN[0]),
            ],
        );

        let nps = [
            int.follow_wire(nmu[8], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[9], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[10], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
            int.follow_wire(nmu[7], "NOC2_NMU128_ATOM_0_TO_NOC")
                .unwrap(),
        ];
        let xbr = int
            .follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_1")
            .unwrap();
        let cell = die.cell(ColId::from_idx(16), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (nps[0], bslots::HNOC_NPS[0]),
                (nps[1], bslots::HNOC_NPS[1]),
                (nps[2], bslots::HNOC_NPS[2]),
                (nps[3], bslots::HNOC_NPS[3]),
                (nmu[10], bslots::HNOC_PS_NMU[0]),
                (nmu[9], bslots::HNOC_PS_NMU[1]),
                (nmu[8], bslots::HNOC_PS_NMU[2]),
                (nmu[7], bslots::HNOC_PS_NMU[3]),
                (xbr, bslots::HNOC_XBR4X2),
                (scans_row[0][1], bslots::HNOC_SCAN[0]),
            ],
        );

        let nps = [
            int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_0")
                .unwrap(),
            int.follow_wire(nps[1], "NOC2_NPS5555_ATOM_0_OUT_0")
                .unwrap(),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_0")
                .unwrap(),
            int.follow_wire(nps[3], "NOC2_NPS5555_ATOM_0_OUT_0")
                .unwrap(),
        ];
        let scan0 = scans_row[0][2];
        let scan1 = scans_row[0][3];
        let scan2 = scans_row[1][1];
        let scan3 = scans_row[1][2];
        let cell = die.cell(ColId::from_idx(20), row);
        let scan_bli = int
            .xlat_bli(scan0, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
            .unwrap();

        if psxl.is_some() {
            let nmu = [
                int.follow_wire(psx, &format!("{pss}_PSXL_NOC_PCIE_0"))
                    .unwrap(),
                int.follow_wire(psx, &format!("{pss}_PSXL_NOC_PCIE_1"))
                    .unwrap(),
                int.follow_wire(psx, &format!("{pss}_PSXL_NOC_PCIE_2"))
                    .unwrap(),
                int.follow_wire(psx, &format!("{pss}_PSXL_NOC_PCIE_3"))
                    .unwrap(),
            ];
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (nps[0], bslots::HNOC_NPS[0]),
                    (nps[1], bslots::HNOC_NPS[1]),
                    (nps[2], bslots::HNOC_NPS[2]),
                    (nps[3], bslots::HNOC_NPS[3]),
                    (nmu[0], bslots::HNOC_PS_NMU[0]),
                    (nmu[1], bslots::HNOC_PS_NMU[1]),
                    (nmu[2], bslots::HNOC_PS_NMU[2]),
                    (nmu[3], bslots::HNOC_PS_NMU[3]),
                    (nsu[2], bslots::HNOC_PS_NSU[0]),
                    (nsu[3], bslots::HNOC_PS_NSU[1]),
                    (nsu[4], bslots::HNOC_PS_NSU[2]),
                    (nsu[5], bslots::HNOC_PS_NSU[3]),
                    (scan0, bslots::HNOC_SCAN[0]),
                    (scan1, bslots::HNOC_SCAN[1]),
                    (scan2, bslots::HNOC_SCAN[2]),
                    (scan3, bslots::HNOC_SCAN[3]),
                ],
            );
        } else {
            let nmu = int.follow_wire(psx, &format!("{pss}_MMI_NOC_0")).unwrap();
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (nps[0], bslots::HNOC_NPS[0]),
                    (nps[1], bslots::HNOC_NPS[1]),
                    (nps[2], bslots::HNOC_NPS[2]),
                    (nps[3], bslots::HNOC_NPS[3]),
                    (nmu, bslots::HNOC_PS_NMU[0]),
                    (nsu[2], bslots::HNOC_PS_NSU[0]),
                    (nsu[3], bslots::HNOC_PS_NSU[1]),
                    (nsu[4], bslots::HNOC_PS_NSU[2]),
                    (nsu[5], bslots::HNOC_PS_NSU[3]),
                    (scan0, bslots::HNOC_SCAN[0]),
                    (scan1, bslots::HNOC_SCAN[1]),
                    (scan2, bslots::HNOC_SCAN[2]),
                    (scan3, bslots::HNOC_SCAN[3]),
                ],
            );
        }

        res.extend([
            X5ioHNoc::PsXA,
            X5ioHNoc::PsXB { flip: false },
            X5ioHNoc::None,
            X5ioHNoc::None,
            X5ioHNoc::PsXB { flip: true },
            X5ioHNoc::PsXC { scan_bli },
        ]);

        chain = [
            int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps[1], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_2"),
            int.follow_wire(nps[3], "NOC2_NPS5555_ATOM_0_OUT_2"),
        ];
        scans = scans_row[1][3..].iter().copied();
    }

    for scan in scans {
        let nps = chain.map(|xy| xy.unwrap());
        let (scan_bli, scan_wn) = int
            .xlat_bli_full(scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
            .unwrap();
        if nps[0] == nps[1] {
            let cell = die.cell(ColId::from_idx(res.len() * 4), row);
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (nps[0], bslots::HNOC_NPS[0]),
                    (nps[2], bslots::HNOC_NPS[1]),
                    (scan, bslots::HNOC_SCAN[0]),
                ],
            );
            res.push(X5ioHNoc::Cap { scan_bli });
        } else {
            let flip = match int
                .int
                .rd
                .tile_kinds
                .key(int.int.rd.tiles[&nps[0]].kind)
                .as_str()
            {
                "NOC2_NPS5555_TOP" => false,
                "NOC2_NPS5555_TOP_MY" => true,
                x => panic!("ummm {x}"),
            };
            let next_wire = if flip {
                "NOC2_NPS5555_ATOM_0_OUT_0"
            } else {
                "NOC2_NPS5555_ATOM_0_OUT_2"
            };
            if let Some(xbr) = int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_1")
                && int
                    .int
                    .rd
                    .tile_kinds
                    .key(int.int.rd.tiles[&xbr].kind)
                    .starts_with("NOC2_XBR4X2")
            {
                let nmu = [
                    int.follow_wire(nps[0], "NOC2_NPS5555_ATOM_0_OUT_3")
                        .unwrap(),
                    int.follow_wire(nps[2], "NOC2_NPS5555_ATOM_0_OUT_1")
                        .unwrap(),
                ];

                while io[res.len()].ddrmc.is_none() {
                    res.push(X5ioHNoc::None);
                }
                let cell = die.cell(ColId::from_idx(res.len() * 4), row);

                name_noc2(
                    int.int.rd,
                    dev_naming,
                    cell.tile(tslots::HNOC),
                    [
                        (nps[0], bslots::HNOC_NPS[0]),
                        (nps[1], bslots::HNOC_NPS[1]),
                        (nps[2], bslots::HNOC_NPS[2]),
                        (nps[3], bslots::HNOC_NPS[3]),
                        (nmu[0], bslots::HNOC_NMU512[0]),
                        (nmu[1], bslots::HNOC_NMU512[1]),
                        (xbr, bslots::HNOC_XBR4X2),
                        (scan, bslots::HNOC_SCAN[0]),
                    ],
                );
                let nmu_bli = nmu.map(|xy| {
                    let bli = [
                        "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_0_",
                        "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_128_",
                        "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_256_",
                    ]
                    .map(|w| int.xlat_bli(xy, w));
                    if bli[0].is_some() {
                        Some(bli.map(|col| col.unwrap()))
                    } else {
                        None
                    }
                });

                let vnoc = [
                    (nps[1], "NOC2_NPS5555_ATOM_0_OUT_3"),
                    (nps[3], "NOC2_NPS5555_ATOM_0_OUT_1"),
                ]
                .map(|(xy, w)| {
                    if let Some((vnoc_xy, vnoc_w)) = int.follow_wire_full(xy, w) {
                        let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
                        let port = match vnoc_w {
                            "NOC2_NPS5555_ATOM_0_IN_2" => 0,
                            "NOC2_NPS5555_ATOM_0_IN_0" => 1,
                            _ => panic!("ummm {vnoc_w}"),
                        };
                        Some(VNocConn(col, port))
                    } else {
                        None
                    }
                });

                res.push(X5ioHNoc::Ddrmc {
                    flip,
                    scan_bli,
                    nmu_bli,
                    vnoc,
                });

                chain = core::array::from_fn(|i| int.follow_wire(nps[i ^ 1], next_wire));
            } else {
                let cell = die.cell(ColId::from_idx(res.len() * 4), row);
                let alt = match scan_wn {
                    "BLI_GRP2_A_BLI_IMUX0" => false,
                    "BLI_GRP0_A_BLI_IMUX0" => true,
                    _ => panic!("umm {scan_wn}"),
                };

                name_noc2(
                    int.int.rd,
                    dev_naming,
                    cell.tile(tslots::HNOC),
                    [
                        (nps[0], bslots::HNOC_NPS[0]),
                        (nps[1], bslots::HNOC_NPS[1]),
                        (nps[2], bslots::HNOC_NPS[2]),
                        (nps[3], bslots::HNOC_NPS[3]),
                        (scan, bslots::HNOC_SCAN[0]),
                    ],
                );

                let vnoc =
                    if let Some(vnoc_xy) = int.follow_wire(nps[3], "NOC2_NPS5555_ATOM_0_OUT_1") {
                        let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
                        Some(col)
                    } else {
                        None
                    };

                res.push(X5ioHNoc::Cross {
                    alt,
                    flip,
                    scan_bli,
                    vnoc,
                });

                chain = nps.map(|xy| int.follow_wire(xy, next_wire));
            }
        }
    }

    res
}

fn get_hsr_x5io(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) -> X5ioRow {
    let row = RowId::from_idx(int.int.rows.len());

    let (clk, clk_test_xlat) = get_hsr_x5io_clk(int, die, dev_naming);
    let mut regions = EntityVec::from_iter(clk.into_iter().map(|clk| X5ioRegion {
        clk: Some(clk),
        hnoc: [X5ioHNoc::None; 2],
        io: [None; 2],
        sysmon_bli: None,
    }));

    let io = get_hsr_x5io_io(int, die, dev_naming, clk_test_xlat);
    let hnoc = get_hsr_x5io_hnoc(int, die, dev_naming, &io);

    assert_eq!(regions.len(), io.len().div_ceil(2));

    for (idx, io) in io.into_iter().enumerate() {
        regions[HRegId::from_idx(idx / 2)].io[idx % 2] = Some(io);
    }

    let sysmon_regs = Vec::from_iter(regions.iter().filter_map(|(reg, rd)| {
        if rd.io.into_iter().flatten().any(|io| io.ddrmc.is_some()) {
            None
        } else {
            Some(reg)
        }
    }));

    for (smidx, xy) in int
        .find_tiles_crd(&["HSM_AMS_SAT_LC"])
        .into_iter()
        .enumerate()
    {
        let reg = sysmon_regs[smidx];
        let col = ColId::from_idx(reg.to_idx() * 8);
        let tcrd = die.cell(col, row).tile(tslots::HSR_SYSMON_SAT);
        let tile = &int.int.rd.tiles[&xy];
        if let Some(bname) = tile.sites.values().next() {
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslots::SYSMON_SAT_HSR), vec![bname.clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let col = int.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_FABRICCLK").unwrap();
        regions[reg].sysmon_bli = Some(col);
    }

    for (idx, noc) in hnoc.into_iter().enumerate() {
        if idx == regions.len() * 2 {
            regions.push(X5ioRegion {
                clk: None,
                hnoc: [X5ioHNoc::None; 2],
                io: [None; 2],
                sysmon_bli: None,
            });
        }
        regions[HRegId::from_idx(idx / 2)].hnoc[idx % 2] = noc;
    }

    X5ioRow { regions }
}

fn get_noc_ssit_s(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    cols_vnoc: &[ColId],
) {
    let row = RowId::from_idx(0);
    let row_hsr = RowId::from_idx(int.int.rows.len());
    let mut pnoc_name = None;

    for xy in int.find_tiles_crd(&["NOC_PNOC_SSIT_CORE"]) {
        let tile = &int.int.rd.tiles[&xy];
        pnoc_name = Some(tile.name.clone());
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let (idx, bslot) = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NMU128" if sy < 4 => (0, bslots::HNOC_PS_NMU[sy]),
                "NOC_NMU128" => (1, bslots::HNOC_PS_NMU[sy - 4]),
                "NOC_NSU128" if sy < 2 => (0, bslots::HNOC_PS_NSU[sy]),
                "NOC_NSU128" => (1, bslots::HNOC_PS_NSU[sy - 2]),
                "NPI_NIR" => (0, bslots::HNOC_NPI_NIR),
                _ => continue,
            };
            dev_naming.bel_names.insert(
                die.cell(ColId::from_idx(idx * 2), row_hsr).bel(bslot),
                vec![tile.sites[sid].clone()],
            );
        }
    }
    let pnoc_name = pnoc_name.unwrap();

    for (i, xy) in int
        .find_tiles_crd(&["NOC_TNOC_SSIT_S2_CORE"])
        .into_iter()
        .enumerate()
    {
        let cell = die.cell(ColId::from_idx(i), row_hsr);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS7575" => bslots::HNOC_NPS[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(cell.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        dev_naming.tile_names.insert(
            cell.tile(tslots::HNOC),
            vec![tile.name.clone(), pnoc_name.clone()],
        );
    }

    // NCRB + bridge
    let mut ncrb_names = vec![];
    let mut bridge_names = vec![];
    for xy in int.find_tiles_crd(&["NOC_TNOC_SSIT_NCRB_CORE"]) {
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        let mut bels = HashMap::new();
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NCRB_SSIT" => bslots::HNOC_NCRB[sy],
                _ => continue,
            };
            bels.insert(bslot, vec![tile.sites[sid].clone()]);
        }
        ncrb_names.push((&tile.name, bels));
    }
    for xy in int.find_tiles_crd(&["NOC_TNOC_BRIDGE_BOT_CORE"]) {
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        let mut bels = HashMap::new();
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS7575" => bslots::HNOC_NPS[sy],
                "NOC_NIDB" => bslots::HNOC_NIDB_V[sy],
                "NOC_NPP_RPTR" => bslots::HNOC_NPP_RPTR[sy],
                _ => continue,
            };
            bels.insert(bslot, vec![tile.sites[sid].clone()]);
        }
        bridge_names.push((&tile.name, bels));
    }
    assert_eq!(ncrb_names.len(), cols_vnoc.len());
    assert_eq!(bridge_names.len(), cols_vnoc.len());
    for (i, &col) in cols_vnoc.iter().enumerate() {
        let cell = die.cell(col, row);
        let (ncrb_name, ref ncrb_bels) = ncrb_names[i];
        let (bridge_name, ref bridge_bels) = bridge_names[i];
        dev_naming
            .tile_names
            .insert(cell.tile(tslots::HNOC), vec![bridge_name.clone()]);
        dev_naming
            .tile_names
            .insert(cell.tile(tslots::HNOC_NCRB), vec![ncrb_name.clone()]);
        for (&bslot, names) in ncrb_bels {
            dev_naming.bel_names.insert(cell.bel(bslot), names.clone());
        }
        for (&bslot, names) in bridge_bels {
            dev_naming.bel_names.insert(cell.bel(bslot), names.clone());
        }
    }
}

fn get_noc_ssit_n(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    cols_vnoc: &[ColId],
) {
    let mut ncrb_names = vec![];
    let mut bridge_names = vec![];
    for xy in int.find_tiles_crd(&["NOC_TNOC_SSIT_NCRB_CORE_MY"]) {
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        let mut bels = HashMap::new();
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NCRB_SSIT" => bslots::HNOC_NCRB[sy],
                _ => continue,
            };
            bels.insert(bslot, vec![tile.sites[sid].clone()]);
        }
        ncrb_names.push((&tile.name, bels));
    }
    for xy in int.find_tiles_crd(&["NOC_TNOC_BRIDGE_TOP_CORE", "NOC2_TNOC_NIDB_CORE"]) {
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        let mut bels = HashMap::new();
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS7575" => bslots::HNOC_NPS[sy],
                "NOC_NIDB" | "NOC2_NIDBV" => bslots::HNOC_NIDB_V[sy],
                "NOC_NPP_RPTR" => bslots::HNOC_NPP_RPTR[sy],
                _ => continue,
            };
            bels.insert(bslot, vec![tile.sites[sid].clone()]);
        }
        bridge_names.push((&tile.name, bels));
    }
    assert_eq!(ncrb_names.len(), cols_vnoc.len());
    assert_eq!(bridge_names.len(), cols_vnoc.len());
    let row = int.int.rows.last_id().unwrap();
    for (i, &col) in cols_vnoc.iter().enumerate() {
        let cell = die.cell(col, row);
        let (ncrb_name, ref ncrb_bels) = ncrb_names[i];
        let (bridge_name, ref bridge_bels) = bridge_names[i];
        dev_naming
            .tile_names
            .insert(cell.tile(tslots::HNOC), vec![bridge_name.clone()]);
        dev_naming
            .tile_names
            .insert(cell.tile(tslots::HNOC_NCRB), vec![ncrb_name.clone()]);
        for (&bslot, names) in ncrb_bels {
            dev_naming.bel_names.insert(cell.bel(bslot), names.clone());
        }
        for (&bslot, names) in bridge_bels {
            dev_naming.bel_names.insert(cell.bel(bslot), names.clone());
        }
    }
}

fn get_hsr_me(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) -> MeRow {
    let row = RowId::from_idx(int.int.rows.len() + 1);

    let mut clk_col = vec![];
    let mut connect_hsm = vec![];
    let mut clk_from_me_xlat: HashMap<_, Vec<_>> = HashMap::new();
    for (idx, xy) in int
        .find_tiles_crd(&["CLK_REBUF_HSR_ME_TNOC_CORE_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4 + 1);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "BUFG_HSR_TEST" => bslots::BUFG_HSR_TEST[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        clk_col.push(int.xlat_hsr_clk(xy, "CLK_HDISTR_1_0"));
        let chsm = if let Some(ni) = int.int.rd.lookup_wire(xy, "HSM_REF_CLK0")
            && let NodeOrWire::Node(ni) = ni
        {
            int.int
                .rd
                .iter_node(ni)
                .any(|(crd, _)| int.int.rd.tiles[&crd].name.starts_with("PS"))
        } else {
            false
        };
        if let Some(ni) = int.int.rd.lookup_wire(xy, "CLK_FROM_ME0") {
            clk_from_me_xlat
                .entry(ni)
                .or_default()
                .push(HRegId::from_idx(idx));
        }
        connect_hsm.push(chsm);
    }

    let mut sysmon_bli = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&["HSM_AMS_SAT_ME_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4 + 1);
        let tcrd = die.cell(col, row).tile(tslots::HSR_SYSMON_SAT);
        let tile = &int.int.rd.tiles[&xy];
        if let Some(bname) = tile.sites.values().next() {
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslots::SYSMON_SAT_HSR), vec![bname.clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let bli = if let Some(bli0) = int.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_FABRICCLK")
            && let Some(bli1) = int.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_SCANCLKNEXT")
        {
            Some([bli0, bli1])
        } else {
            None
        };
        sysmon_bli.push(bli);
    }

    let mut ncrb_tcrd = vec![];
    let mut vnoc = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&["NOC_TNOC_S2_CORE_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4 + 1);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS5555" => bslots::HNOC_NPS[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        let ni = int
            .int
            .rd
            .lookup_wire(xy, "NOC_NPS5555_TOP_0_IN_1")
            .unwrap();
        let NodeOrWire::Node(ni) = ni else {
            unreachable!()
        };
        let oxy1 = int.int.rd.iter_node(ni).find(|&(cxy, _)| cxy != xy);
        let ni = int
            .int
            .rd
            .lookup_wire(xy, "NOC_NPS5555_TOP_0_IN_3")
            .unwrap();
        let NodeOrWire::Node(ni) = ni else {
            unreachable!()
        };
        let oxy3 = int.int.rd.iter_node(ni).find(|&(cxy, _)| cxy != xy);
        if let Some((oxy1, _)) = oxy1
            && let Some((oxy3, _)) = oxy3
        {
            for (i, oxy) in [oxy1, oxy3].into_iter().enumerate() {
                let otile = &int.int.rd.tiles[&oxy];
                let otk = &int.int.rd.tile_kinds[otile.kind];
                for (sid, &slot, _) in &otk.sites {
                    let TkSiteSlot::Xy(slot, _sx, _sy) = slot else {
                        continue;
                    };
                    let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                        "NOC_NMU128" => bslots::HNOC_ME_NMU[i],
                        "NOC_NSU128" => bslots::HNOC_ME_NSU[i],
                        _ => continue,
                    };
                    if let Some(bname) = otile.sites.get(sid) {
                        dev_naming
                            .bel_names
                            .insert(tcrd.bel(bslot), vec![bname.clone()]);
                    }
                }
            }
            vnoc.push(MeVNocConn::None);
            dev_naming.tile_names.insert(
                tcrd,
                vec![
                    tile.name.clone(),
                    int.int.rd.tiles[&oxy1].name.clone(),
                    int.int.rd.tiles[&oxy3].name.clone(),
                ],
            );
        } else if let Some((oxy1, wi)) = oxy1 {
            let col = int.lookup_column_inter(oxy1.x as i32) - 1;
            let lane = match int.int.rd.wires[wi].as_str() {
                "NOC_NPS_VNOC_ATOM_0_OUT_0" => 0,
                "NOC_NPS_VNOC_ATOM_0_OUT_2" => 1,
                "NOC2_NPS5555_ATOM_0_OUT_0" => 0,
                "NOC2_NPS5555_ATOM_0_OUT_2" => 1,
                _ => unreachable!(),
            };
            let conn0 = VNocConn(col, lane);
            let conn1 = VNocConn(col, lane ^ 1);
            vnoc.push(MeVNocConn::Port1([conn0, conn1]));
            dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
            ncrb_tcrd.push(tcrd.tile(tslots::HNOC_NCRB));
        } else {
            let (oxy3, wi) = oxy3.unwrap();
            let col = int.lookup_column_inter(oxy3.x as i32) - 1;
            let lane = match int.int.rd.wires[wi].as_str() {
                "NOC_NPS_VNOC_ATOM_0_OUT_0" => 0,
                "NOC_NPS_VNOC_ATOM_0_OUT_2" => 1,
                "NOC2_NPS5555_ATOM_0_OUT_0" => 0,
                "NOC2_NPS5555_ATOM_0_OUT_2" => 1,
                _ => unreachable!(),
            };
            let conn0 = VNocConn(col, lane);
            let conn1 = VNocConn(col, lane ^ 1);
            vnoc.push(MeVNocConn::Port3([conn0, conn1]));
            dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
            ncrb_tcrd.push(tcrd.tile(tslots::HNOC_NCRB));
        }
    }

    for (idx, xy) in int
        .find_tiles_crd(&["NOC_TNOC_NCRB_CORE_MX"])
        .into_iter()
        .enumerate()
    {
        let tcrd = ncrb_tcrd[idx];
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NCRB" => bslots::HNOC_NCRB[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
    }

    let width = clk_col.len();
    assert_eq!(sysmon_bli.len(), width);
    assert_eq!(vnoc.len(), width);

    let mut regions = EntityVec::new();
    for i in 0..width {
        regions.push(MeRegion {
            sysmon_bli: sysmon_bli[i],
            clk_col: clk_col[i],
            connect_hsm: connect_hsm[i],
            vnoc: vnoc[i],
            pll_test: false,
        });
    }

    let ai = if !int.find_tiles(&["AIE_INTF_A_CORE"]).is_empty() {
        let mut rows = BTreeSet::new();
        for xy in int.find_tiles(&["AIE_TILE", "AIE_TILE_MY"]) {
            rows.insert(xy.1);
        }
        let mut columns = EntityVec::new();
        let mut pll_bli = None;
        for xy in int.find_tiles_crd(&[
            "AIE_INTF_A_CORE",
            "AIE_INTF_B0_CORE",
            "AIE_INTF_B1_CORE",
            "AIE_INTF_B2_CORE",
            "AIE_INTF_B3_CORE",
            "AIE_INTF_C_CORE",
        ]) {
            let col: ColId = columns.next_id();
            if col.to_idx() == 0 {
                pll_bli = int.xlat_bli(xy, "AIE_PLL_AIE_PLL_TEST_OUT_0_");
                if let Some(ni) = int
                    .int
                    .rd
                    .lookup_wire(xy, "AIE_PLL_AIE_PLL_TEST_CLK_OUT_0_")
                    && let Some(regs) = clk_from_me_xlat.get(&ni)
                {
                    for &reg in regs {
                        regions[reg].pll_test = true;
                    }
                }
            }
            let axis_bli = [
                int.xlat_bli(xy, "AIE_PL_S_TDATA_0_0_"),
                int.xlat_bli(xy, "AIE_PL_S_TDATA_4_0_"),
            ];
            let test_bli = if let Some(bli0) = int.xlat_bli(xy, "PL_ME_SCAN_CLKDIV_RSTN_EXT")
                && let Some(bli1) = int.xlat_bli(xy, "ME_PL_SCAN_CHNL_OUT_EXT21")
            {
                Some([bli0, bli1])
            } else {
                None
            };
            let axi_clk_test = int
                .int
                .rd
                .lookup_wire(xy, "AIE_NOC_S_AXI_ACLK")
                .and_then(|ni| clk_from_me_xlat.get(&ni))
                .map(|regs| regs[0]);
            columns.push(AiColumn {
                axis_bli,
                test_bli,
                axi_clk_test,
            });
            let tile = &int.int.rd.tiles[&xy];
            let cell = die.cell(col, row + 1);
            if let Some(slot) = int.int.rd.tk_slot_xy(tile.kind, "AIE_PL", 0, 0)
                && tile.sites.contains_id(slot)
            {
                // OK
            } else if col.to_idx() == 0 {
                // also OK
            } else {
                dev_naming.skip_bels.insert(cell.bel(bslots::AIE_PL));
            }
            if let Some(slot) = int.int.rd.tk_slot_xy(tile.kind, "AIE_NOC", 0, 0)
                && !tile.sites.contains_id(slot)
            {
                dev_naming.skip_bels.insert(cell.bel(bslots::AIE_NOC));
            }
        }
        let pll_bli = pll_bli.unwrap();
        Some(AiArray {
            kind: AiKind::Ai,
            columns,
            rows: rows.len(),
            rows_mem: 0,
            pll_bli,
        })
    } else if !int.find_tiles(&["AIE_ML_INTF_A_CORE"]).is_empty() {
        let mut rows = BTreeSet::new();
        let mut rows_mem = BTreeSet::new();
        for xy in int.find_tiles(&["AIE_ML_TILE"]) {
            rows.insert(xy.1);
        }
        for xy in int.find_tiles(&["AIE_ML_MEM_TILE"]) {
            rows_mem.insert(xy.1);
        }
        let mut columns = EntityVec::new();
        let mut pll_bli = None;
        for xy in int.find_tiles_crd(&[
            "AIE_ML_INTF_A_CORE",
            "AIE_ML_SHIM_PL_TILE",
            "AIE_ML_SHIM_NOC_TILE",
        ]) {
            let col: ColId = columns.next_id();
            if col.to_idx() == 0 {
                pll_bli = int.xlat_bli(xy, "AIE_PLL_AIE_PLL_TEST_OUT_0_");
                if let Some(ni) = int
                    .int
                    .rd
                    .lookup_wire(xy, "AIE_PLL_AIE_PLL_TEST_CLK_OUT_0_")
                    && let Some(regs) = clk_from_me_xlat.get(&ni)
                {
                    for &reg in regs {
                        regions[reg].pll_test = true;
                    }
                }
            }
            let axis_bli = [
                int.xlat_bli(xy, "AIE_PL_S_TDATA_0_0_"),
                int.xlat_bli(xy, "AIE_PL_S_TDATA_4_0_"),
            ];
            let intf_xy = int.delta(xy, 0, -1);
            let test_bli = if let Some(bli0) = int.xlat_bli(intf_xy, "PL_ME_SCAN_CHNL_IN_EXT8")
                && let Some(bli1) = int.xlat_bli(intf_xy, "ME_PL_SCAN_CHNL_OUT_EXT23")
            {
                Some([bli0, bli1])
            } else {
                None
            };
            let axi_clk_test = int
                .int
                .rd
                .lookup_wire(xy, "AIE_NOC_S_AXI_ACLK")
                .and_then(|ni| clk_from_me_xlat.get(&ni))
                .map(|regs| regs[0]);
            columns.push(AiColumn {
                axis_bli,
                test_bli,
                axi_clk_test,
            });
        }
        let pll_bli = pll_bli.unwrap();
        Some(AiArray {
            kind: AiKind::AiMl,
            columns,
            rows: rows.len(),
            rows_mem: rows_mem.len(),
            pll_bli,
        })
    } else {
        None
    };

    MeRow { regions, ai }
}

fn get_hsr_me2(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) -> Me2Row {
    let row = RowId::from_idx(int.int.rows.len() + 1);

    let tiles = Vec::from_iter(
        int.find_tiles(&[
            "NOC2_NMU128_TOP_MX",
            "NOC2_NSU128_TOP_MX",
            "NOC2_NPS6X_TOP_MX",
            "NOC2_NCRB_TILE_MX",
            "NOC2_SCAN_TOP_MX",
        ])
        .into_iter()
        .filter(|&(_, y)| y > *int.int.rows.last().unwrap())
        .map(|(x, y)| Coord {
            x: x as u16,
            y: y as u16,
        }),
    );
    let y0 = tiles.iter().map(|&xy| xy.y).min().unwrap();
    let mut tiles_row = [const { vec![] }; 2];
    for xy in tiles {
        tiles_row[(xy.y - y0) as usize].push(xy);
    }
    let [trow0, trow1] = tiles_row;

    let scan_bli_w = {
        let cell = die.cell(ColId::from_idx(0), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (trow1[0], bslots::HNOC_ME_NMU[0]),
                (trow1[1], bslots::HNOC_ME_NMU[1]),
                (trow0[0], bslots::HNOC_SCAN[0]),
            ],
        );
        int.xlat_bli(trow0[0], "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
            .unwrap()
    };

    let mut scan_bli = vec![];
    let mut hnoc = vec![];
    let mut pos0 = 1;
    let mut pos1 = 2;
    while trow0.len() - pos0 > 3 {
        let cell = die.cell(ColId::from_idx(scan_bli.len() * 4 + 2), row);
        scan_bli.push(
            int.xlat_bli(
                trow0[pos0 + 3],
                "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
            )
            .unwrap(),
        );
        let p2_xy = int.follow_wire(trow0[pos0 + 1], "NOC2_NPS6X_ATOM_0_IN_2");
        let col_vnoc = if let Some(p2_xy) = p2_xy
            && p2_xy.y < y0
        {
            Some(int.lookup_column_inter(p2_xy.x as i32) - 1)
        } else {
            None
        };
        let kind = if int.int.rd.tiles[&trow0[pos0 + 2]]
            .name
            .starts_with("NOC2_NCRB")
        {
            let tcrd = cell.tile(tslots::HNOC);
            name_noc2(
                int.int.rd,
                dev_naming,
                tcrd,
                [
                    (trow1[pos1], bslots::HNOC_ME_NMU[0]),
                    (trow1[pos1 + 1], bslots::HNOC_ME_NMU[1]),
                    (trow1[pos1 + 2], bslots::HNOC_ME_NMU[2]),
                    (trow1[pos1 + 3], bslots::HNOC_ME_NMU[3]),
                    (trow1[pos1 + 4], bslots::HNOC_ME_NSU[0]),
                    (trow0[pos0], bslots::HNOC_NPS6X[0]),
                    (trow0[pos0 + 1], bslots::HNOC_NPS6X[1]),
                    (trow0[pos0 + 3], bslots::HNOC_SCAN[0]),
                ],
            );
            let ncrb_xy = trow0[pos0 + 2];
            let ncrb_tile = &int.int.rd.tiles[&ncrb_xy];
            dev_naming
                .tile_names
                .get_mut(&tcrd)
                .unwrap()
                .push(ncrb_tile.name.clone());
            let tk = &int.int.rd.tile_kinds[ncrb_tile.kind];
            for (sid, &slot, _) in &tk.sites {
                let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                    continue;
                };
                let sy = sy as usize;
                let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                    "NOC_NCRB" => bslots::HNOC_ME2_NCRB[sy],
                    _ => continue,
                };
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![ncrb_tile.sites[sid].clone()]);
            }
            pos0 += 4;
            pos1 += 5;
            Me2HNoc::VNocNcrb(col_vnoc.unwrap())
        } else if int.int.rd.tiles[&trow0[pos0]]
            .name
            .starts_with("NOC2_NPS6X")
        {
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (trow1[pos1], bslots::HNOC_ME_NMU[0]),
                    (trow1[pos1 + 1], bslots::HNOC_ME_NMU[1]),
                    (trow1[pos1 + 2], bslots::HNOC_ME_NMU[2]),
                    (trow1[pos1 + 3], bslots::HNOC_ME_NMU[3]),
                    (trow1[pos1 + 4], bslots::HNOC_ME_NSU[0]),
                    (trow0[pos0], bslots::HNOC_NPS6X[0]),
                    (trow0[pos0 + 1], bslots::HNOC_NPS6X[1]),
                    (trow0[pos0 + 2], bslots::HNOC_NPS6X[2]),
                    (trow0[pos0 + 3], bslots::HNOC_SCAN[0]),
                ],
            );
            pos0 += 4;
            pos1 += 5;
            Me2HNoc::VNoc4(col_vnoc.unwrap())
        } else {
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (trow1[pos1], bslots::HNOC_ME_NMU[0]),
                    (trow1[pos1 + 1], bslots::HNOC_ME_NMU[1]),
                    (trow1[pos1 + 2], bslots::HNOC_ME_NMU[2]),
                    (trow1[pos1 + 3], bslots::HNOC_ME_NMU[3]),
                    (trow0[pos0], bslots::HNOC_ME_NSU[0]),
                    (trow0[pos0 + 1], bslots::HNOC_NPS6X[0]),
                    (trow0[pos0 + 2], bslots::HNOC_NPS6X[1]),
                    (trow0[pos0 + 3], bslots::HNOC_SCAN[0]),
                ],
            );
            pos0 += 4;
            pos1 += 4;
            if let Some(col) = col_vnoc {
                Me2HNoc::VNoc(col)
            } else if p2_xy.is_some() {
                Me2HNoc::Cross
            } else {
                Me2HNoc::Plain
            }
        };
        hnoc.push(kind);
    }

    let scan_bli_e = {
        let cell = die.cell(ColId::from_idx(scan_bli.len() * 4 + 2), row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (trow1[pos1], bslots::HNOC_ME_NMU[0]),
                (trow1[pos1 + 2], bslots::HNOC_ME_NMU[1]),
                (trow1[pos1 + 1], bslots::HNOC_ME_NMU[2]),
                (trow0[pos0], bslots::HNOC_ME_NSU[0]),
                (trow0[pos0 + 1], bslots::HNOC_NPS6X[0]),
                (trow0[pos0 + 2], bslots::HNOC_SCAN[0]),
            ],
        );
        int.xlat_bli(
            trow0[pos0 + 2],
            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
        )
        .unwrap()
    };

    let mut clk_col = vec![];
    let mut connect_hsm = vec![];
    let mut clk_from_me_xlat = HashMap::new();
    for (idx, xy) in int
        .find_tiles_crd(&["CLK_REBUF_HSR_ME_TNOC_ULVT_CORE_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4 + 2);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "BUFG_HSR_TEST_ULVT" => bslots::BUFG_HSR_TEST[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        clk_col.push(int.xlat_hsr_clk(xy, "CLK_HDISTR_1_0"));
        let chsm = if let Some(ni) = int.int.rd.lookup_wire(xy, "HSM_REF_CLK0")
            && let NodeOrWire::Node(ni) = ni
        {
            int.int
                .rd
                .iter_node(ni)
                .any(|(crd, _)| int.int.rd.tiles[&crd].name.starts_with("PS"))
        } else {
            false
        };
        if let Some(ni) = int.int.rd.lookup_wire(xy, "CLK_FROM_ME0") {
            clk_from_me_xlat.insert(ni, HRegId::from_idx(idx));
        }
        connect_hsm.push(chsm);
    }

    let mut sysmon_bli = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&["HSM_AMS_SAT_ME_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4 + 2);
        let tcrd = die.cell(col, row).tile(tslots::HSR_SYSMON_SAT);
        let tile = &int.int.rd.tiles[&xy];
        if let Some(bname) = tile.sites.values().next() {
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslots::SYSMON_SAT_HSR), vec![bname.clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
        let bli0 = int.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_FABRICCLK").unwrap();
        let bli1 = int
            .xlat_bli(xy, "AMS_SATELLITE_ATOM_0_SCANCLKNEXT")
            .unwrap();
        sysmon_bli.push([bli0, bli1]);
    }

    let width = clk_col.len();
    assert_eq!(sysmon_bli.len(), width);
    assert_eq!(scan_bli.len(), width);

    let mut regions = EntityVec::new();
    for i in 0..width {
        regions.push(Me2Region {
            sysmon_bli: sysmon_bli[i],
            clk_col: clk_col[i],
            connect_hsm: connect_hsm[i],
            scan_bli: scan_bli[i],
            hnoc: hnoc[i],
        });
    }

    let mut rows = BTreeSet::new();
    let mut rows_mem = BTreeSet::new();
    for xy in int.find_tiles(&["AIE2P_S_TILE"]) {
        rows.insert(xy.1);
    }
    for xy in int.find_tiles(&["AIE2P_S_MEM_TILE"]) {
        rows_mem.insert(xy.1);
    }
    let mut columns = EntityVec::new();
    let mut pll_bli = None;
    for xy in int.find_tiles_crd(&["AIE2P_S_INTF_A_CORE", "AIE2P_S_INTF_C_CORE"]) {
        let col: ColId = columns.next_id();
        if col.to_idx() == 0 {
            pll_bli = int.xlat_bli(xy, "AIE_PLL_AIE_PLL_TEST_OUT_0_");
        }
        let axis_bli = [
            int.xlat_bli(xy, "AIE_PL_S_TDATA_0_0_"),
            int.xlat_bli(xy, "AIE_PL_S_TDATA_4_0_"),
        ];
        let test_bli = if let Some(bli0) = int.xlat_bli(xy, "IF_ME_DFX0_FABRIC_PL_ME_SCAN_EN_N")
            && let Some(bli1) = int.xlat_bli(xy, "IF_ME_DFX0_FABRIC_PL_ME_SCAN_CHNL_IN20")
        {
            Some([bli0, bli1])
        } else {
            None
        };
        let axi_clk_test = int
            .int
            .rd
            .lookup_wire(xy, "AIE_NOC_S_AXI_ACLK")
            .and_then(|ni| clk_from_me_xlat.get(&ni))
            .copied();
        columns.push(AiColumn {
            axis_bli,
            test_bli,
            axi_clk_test,
        });
    }
    let pll_bli = pll_bli.unwrap();
    let ai = AiArray {
        kind: AiKind::AiMl2,
        columns,
        rows: rows.len(),
        rows_mem: rows_mem.len(),
        pll_bli,
    };

    Me2Row {
        regions,
        ai,
        scan_bli_w,
        scan_bli_e,
    }
}

fn get_hsr_hbm(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) -> HbmRow {
    let row = RowId::from_idx(int.int.rows.len() + 1);

    for (idx, xy) in int
        .find_tiles_crd(&["CLKE2_REBUF_HBM_CORE_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4);
        let tcrd = die.cell(col, row).tile(tslots::HSR_CLK);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "BUFG_HSR_TEST" => bslots::BUFG_HSR_TEST[sy],
                "BUFG_PS" => bslots::BUFG_HBM[sy],
                _ => continue,
            };
            if let Some(bname) = tile.sites.get(sid) {
                dev_naming
                    .bel_names
                    .insert(tcrd.bel(bslot), vec![bname.clone()]);
            }
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
    }

    for xy in int.find_tiles_crd(&["NOC_HBM_NCRB_CORE_MX"]) {
        let ni = int.int.rd.lookup_wire(xy, "NOC_NCRB_TOP_1_IN_0").unwrap();
        let NodeOrWire::Node(ni) = ni else {
            unreachable!()
        };
        let vnoc_xy = int
            .int
            .rd
            .iter_node(ni)
            .find(|&(cxy, _)| cxy != xy)
            .unwrap()
            .0;
        let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
        let tcrd = die.cell(col, row - 2).tile(tslots::HNOC_NCRB);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NCRB" => bslots::HNOC_NCRB[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
    }

    let nmu_names = int.get_simple_names(&["NOC_NMU_HBM2E_TOP_MX"]);

    let mut intf_bli = EntityVec::new();
    for (idx, (xy, tname, bname)) in int
        .get_simple_names(&["NOC_HBM_BLI_SCAN_TOP_MX"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx);
        let tcrd = die.cell(col, row).tile(tslots::HSR_NMU);
        let names = vec![
            tname,
            nmu_names[idx * 2].1.clone(),
            nmu_names[idx * 2 + 1].1.clone(),
        ];
        dev_naming.tile_names.insert(tcrd, names);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::NOC_HBM_BLI_SCAN), vec![bname.clone()]);
        for j in 0..2 {
            dev_naming.bel_names.insert(
                tcrd.bel(bslots::NOC_NMU_HBM2E[j]),
                vec![nmu_names[2 * idx + j].2.clone()],
            );
        }

        let bli0 = int
            .xlat_bli(
                xy,
                "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN0_DFX_SCAN_CSSD_TRIG_N",
            )
            .unwrap();
        let bli1 = int
            .xlat_bli(
                xy,
                "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN1_DFX_SCAN_CSSD_TRIG_N",
            )
            .unwrap();
        let bli2 = int
            .xlat_bli(
                xy,
                "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN2_DFX_SCAN_CSSD_TRIG_N",
            )
            .unwrap();
        intf_bli.push([bli0, bli1, bli2]);
    }

    for (idx, xy) in int
        .find_tiles_crd(&["NOC_HBM_S4_CORE"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 8 + 2);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS5555" => bslots::HNOC_NPS[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        dev_naming.tile_names.insert(tcrd, vec![tile.name.clone()]);
    }

    let mut nps8_names = vec![];
    for (idx, xy) in int
        .find_tiles_crd(&["NOC_HBM_NPS8_CORE"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS4" => bslots::HNOC_NPS4[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        nps8_names.push(tile.name.clone());
    }
    for (idx, xy) in int
        .find_tiles_crd(&["NOC_HBM_S4_NPS6_CORE"])
        .into_iter()
        .enumerate()
    {
        let col = ColId::from_idx(idx * 4);
        let tcrd = die.cell(col, row).tile(tslots::HNOC);
        let tile = &int.int.rd.tiles[&xy];
        let tk = &int.int.rd.tile_kinds[tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NPS6" => bslots::HNOC_NPS6[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![tile.sites[sid].clone()]);
        }
        dev_naming
            .tile_names
            .insert(tcrd, vec![tile.name.clone(), nps8_names[idx].clone()]);
    }

    let hpll_names = int.get_simple_names(&["CMT_HPLL"]);
    let sysmon_names = int.get_simple_names(&["HSM_AMS_SAT_ME"]);
    let io_names = int.get_simple_names(&["HBM_IO_MS_TOP"]);
    let phy_names = int.get_simple_names(&["HBM_PHY_MS_TOP"]);
    assert_eq!(hpll_names.len(), 2);
    assert_eq!(sysmon_names.len(), 4);
    assert_eq!(io_names.len(), 2);
    assert_eq!(phy_names.len(), 2);
    let mut ms_bli = vec![];
    for (
        idx,
        (
            (
                ((xy_phy, tn_phy, bn_phy), (_, tn_io, bn_io)),
                &[(_, ref tn_sm0, ref bn_sm0), (_, ref tn_sm1, ref bn_sm1)],
            ),
            (_, tn_hpll, bn_hpll),
        ),
    ) in phy_names
        .into_iter()
        .zip(io_names)
        .zip(sysmon_names.as_chunks::<2>().0)
        .zip(hpll_names)
        .enumerate()
    {
        let col = ColId::from_idx(idx * 16);
        let tcrd = die.cell(col, row).tile(tslots::HSR_BANK);
        dev_naming.tile_names.insert(
            tcrd,
            vec![tn_phy, tn_io, tn_sm0.clone(), tn_sm1.clone(), tn_hpll],
        );
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::HBM_MS), vec![bn_phy, bn_io]);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::HPLL), vec![bn_hpll]);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::SYSMON_SAT_HBM[0]), vec![bn_sm0.clone()]);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::SYSMON_SAT_HBM[1]), vec![bn_sm1.clone()]);
        let bli = int
            .xlat_bli(xy_phy, "HBM_PHY_MS_CORE_0_PHY_MS_CORE_CLKOUTPHY_EN_MISC")
            .unwrap();
        ms_bli.push(bli);
    }

    let hbmmc_names = int.get_simple_names(&["HBMMC_CORE"]);
    let phy_names = int.get_simple_names(&["HBM_PHY_CHNL_TOP"]);
    let io_names = int.get_simple_names(&["HBM_IO_CHNL_TOP"]);
    assert_eq!(hbmmc_names.len(), 16);
    assert_eq!(io_names.len(), 16);
    assert_eq!(phy_names.len(), 16);
    for (idx, (((_, tn_phy, bn_phy), (_, tn_io, bn_io)), (_, tn_mc, bn_mc))) in phy_names
        .into_iter()
        .zip(io_names)
        .zip(hbmmc_names)
        .enumerate()
    {
        let col = ColId::from_idx(idx * 2);
        let tcrd = die.cell(col, row).tile(tslots::HSR_IO);
        dev_naming
            .tile_names
            .insert(tcrd, vec![tn_mc, tn_phy, tn_io]);
        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::HBM_CHNL), vec![bn_mc, bn_phy, bn_io]);
    }

    HbmRow {
        intf_bli,
        ms_bli: ms_bli.try_into().unwrap(),
    }
}

fn get_hsr_nic(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) -> NicRow {
    let tiles = Vec::from_iter(
        int.find_tiles(&[
            "NOC2_NMU256_TOP",
            "NOC2_NSU256_TOP",
            "NOC2_NPS5555_TOP",
            "NOC2_NPS7575_TOP",
            "NOC2_XBR2X4_TOP",
            "NOC2_NMU512_BLI_TILE",
            "NOC2_NSU512_BLI_TILE",
            "NOC2_NPP_RPTR_TILE",
            "NOC2_NCRB_TILE",
            "NOC2_SCAN_TOP",
        ])
        .into_iter()
        .filter(|&(_, y)| y > *int.int.rows.last().unwrap())
        .map(|(x, y)| Coord {
            x: x as u16,
            y: y as u16,
        }),
    );
    let y0 = tiles.iter().map(|&xy| xy.y).min().unwrap();
    let mut tiles_row = [const { vec![] }; 6];
    for xy in tiles {
        tiles_row[(xy.y - y0) as usize].push(xy);
    }
    let [
        tiles_cpm0,
        tiles_ncrb,
        tiles_cpm1,
        tiles_hnic0,
        tiles_hnic1,
        tiles_mid,
    ] = tiles_row;

    let row = RowId::from_idx(int.int.rows.len() + 1);

    let mut ncrb_scan_bli = vec![];
    let mut ncrb_tiles = vec![];

    for &[ncrb_xy, scan_xy] in tiles_ncrb.as_chunks::<2>().0 {
        let ni = int
            .int
            .rd
            .lookup_wire(ncrb_xy, "NOC_NCRB_TOP_1_IN_0")
            .unwrap();
        let NodeOrWire::Node(ni) = ni else {
            unreachable!()
        };
        let vnoc_xy = int
            .int
            .rd
            .iter_node(ni)
            .find(|&(cxy, _)| cxy != ncrb_xy)
            .unwrap()
            .0;
        let col = int.lookup_column_inter(vnoc_xy.x as i32) - 1;
        let tcrd = die.cell(col, row - 2).tile(tslots::HNOC_NCRB);
        let ncrb_tile = &int.int.rd.tiles[&ncrb_xy];
        let tk = &int.int.rd.tile_kinds[ncrb_tile.kind];
        for (sid, &slot, _) in &tk.sites {
            let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                continue;
            };
            let sy = sy as usize;
            let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                "NOC_NCRB" => bslots::HNOC_NCRB[sy],
                _ => continue,
            };
            dev_naming
                .bel_names
                .insert(tcrd.bel(bslot), vec![ncrb_tile.sites[sid].clone()]);
        }
        let scan_tile = &int.int.rd.tiles[&scan_xy];
        let bname = scan_tile.sites.values().next().unwrap();
        let scan_bli = int
            .xlat_bli(scan_xy, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
            .unwrap();
        ncrb_scan_bli.push(scan_bli);
        ncrb_tiles.push(ncrb_xy);

        dev_naming
            .bel_names
            .insert(tcrd.bel(bslots::HNOC_NCRB_SCAN), vec![bname.clone()]);
        dev_naming
            .tile_names
            .insert(tcrd, vec![ncrb_tile.name.clone(), scan_tile.name.clone()]);
    }

    let mut blocks = EntityVec::new();

    {
        let col = blocks.next_id();
        let cell = die.cell(col, row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (tiles_cpm0[0], bslots::HNOC_NIC_AXIS_NMU[0]),
                (tiles_cpm0[1], bslots::HNOC_NIC_AXIS_NMU[1]),
                (tiles_cpm0[2], bslots::HNOC_NIC_AXIS_NMU[2]),
                (tiles_cpm0[3], bslots::HNOC_NIC_AXIS_NMU[3]),
                (tiles_cpm0[4], bslots::HNOC_NIC_AXI_NMU),
                (tiles_cpm0[5], bslots::HNOC_NIC_XBR2X4),
                (tiles_cpm0[6], bslots::HNOC_NPS[0]),
                (tiles_cpm0[7], bslots::HNOC_NPS[1]),
                (tiles_cpm1[0], bslots::HNOC_NIC_AXIS_NSU[0]),
                (tiles_cpm1[1], bslots::HNOC_NIC_AXIS_NSU[1]),
                (tiles_cpm1[2], bslots::HNOC_NIC_AXIS_NSU[2]),
                (tiles_cpm1[3], bslots::HNOC_NIC_AXIS_NSU[3]),
                (tiles_cpm1[4], bslots::HNOC_NIC_AXI_NSU),
                (tiles_cpm1[5], bslots::HNOC_SCAN[0]),
                (tiles_cpm1[6], bslots::HNOC_NPS[2]),
                (tiles_cpm1[7], bslots::HNOC_NPS[3]),
            ],
        );
        blocks.push(NicBlock::Cpm5NCdx);
    }
    {
        let col = blocks.next_id();
        let cell = die.cell(col, row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (tiles_cpm0[8], bslots::HNOC_NIC_AXIS_NMU[0]),
                (tiles_cpm0[9], bslots::HNOC_NIC_AXIS_NMU[1]),
                (tiles_cpm0[10], bslots::HNOC_NIC_AXIS_NMU[2]),
                (tiles_cpm0[11], bslots::HNOC_NIC_AXIS_NMU[3]),
                (tiles_cpm0[12], bslots::HNOC_NPS[0]),
                (tiles_cpm0[13], bslots::HNOC_NPS[1]),
                (tiles_cpm0[14], bslots::HNOC_SCAN[0]),
                (tiles_cpm1[8], bslots::HNOC_NIC_AXIS_NSU[0]),
                (tiles_cpm1[9], bslots::HNOC_NIC_AXIS_NSU[1]),
                (tiles_cpm1[10], bslots::HNOC_NIC_AXIS_NSU[2]),
                (tiles_cpm1[11], bslots::HNOC_NIC_AXIS_NSU[3]),
                (tiles_cpm1[12], bslots::HNOC_NPS[2]),
                (tiles_cpm1[13], bslots::HNOC_NPS[3]),
                (tiles_cpm1[14], bslots::HNOC_SCAN[1]),
            ],
        );
        let scan_bli = int
            .xlat_bli(
                tiles_cpm0[14],
                "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
            )
            .unwrap();
        blocks.push(NicBlock::Cpm5NDpu { scan_bli });
    }

    let mut pos = 0;
    let mut vnoc_idx = 0;
    while pos < tiles_mid.len() {
        let col = blocks.next_id();
        let cell = die.cell(col, row);
        if int.int.rd.tiles[&tiles_mid[pos]]
            .name
            .starts_with("NOC2_NMU")
        {
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (tiles_mid[pos], bslots::HNOC_NMU512[0]),
                    (tiles_mid[pos + 1], bslots::HNOC_NSU512),
                    (tiles_mid[pos + 2], bslots::HNOC_NPS[0]),
                    (tiles_mid[pos + 3], bslots::HNOC_NPS[1]),
                    (tiles_mid[pos + 4], bslots::HNOC_NPP_RPTR[0]),
                    (tiles_mid[pos + 5], bslots::HNOC_NPP_RPTR[1]),
                    (tiles_mid[pos + 6], bslots::HNOC_SCAN[0]),
                ],
            );

            let nsu_bli = [
                "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_0_",
                "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_256_",
                "NOC2_NSU512_ATOM_0_IF_NOC_AXI_AWLEN_0_",
                "NOC2_NSU512_ATOM_0_IF_NOC_AXI_WDATA_320_",
            ]
            .map(|w| int.xlat_bli(tiles_mid[pos + 1], w).unwrap());
            let nmu_bli = [
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_0_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_128_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_256_",
            ]
            .map(|w| int.xlat_bli(tiles_mid[pos], w).unwrap());
            let scan_bli = int
                .xlat_bli(
                    tiles_mid[pos + 6],
                    "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                )
                .unwrap();
            blocks.push(NicBlock::Bli {
                nsu_bli,
                nmu_bli,
                scan_bli,
            });
            pos += 7;
        } else {
            name_noc2(
                int.int.rd,
                dev_naming,
                cell.tile(tslots::HNOC),
                [
                    (tiles_mid[pos], bslots::HNOC_NPS[0]),
                    (tiles_mid[pos + 1], bslots::HNOC_NPS[1]),
                    (tiles_mid[pos + 2], bslots::HNOC_NPS[2]),
                    (tiles_mid[pos + 3], bslots::HNOC_NPS[3]),
                    (tiles_mid[pos + 4], bslots::HNOC_SCAN[0]),
                ],
            );
            let scan_bli = int
                .xlat_bli(
                    tiles_mid[pos + 4],
                    "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                )
                .unwrap();
            let ncrb_xy = ncrb_tiles[vnoc_idx];
            let swap = int.int.rd.lookup_wire(ncrb_xy, "NOC_NCRB_TOP_0_IN_0")
                == int
                    .int
                    .rd
                    .lookup_wire(tiles_mid[pos + 3], "NOC2_NPS7575_ATOM_0_OUT_3");
            blocks.push(NicBlock::VNoc {
                scan_bli,
                ncrb_scan_bli: ncrb_scan_bli[vnoc_idx],
                swap,
            });
            vnoc_idx += 1;
            pos += 5;
        }
    }

    {
        let col = blocks.next_id();
        let cell = die.cell(col, row);
        name_noc2(
            int.int.rd,
            dev_naming,
            cell.tile(tslots::HNOC),
            [
                (tiles_hnic0[0], bslots::HNOC_NIC_AXIS_NMU[0]),
                (tiles_hnic0[1], bslots::HNOC_NIC_AXIS_NMU[1]),
                (tiles_hnic0[2], bslots::HNOC_NIC_AXIS_NMU[2]),
                (tiles_hnic0[3], bslots::HNOC_NIC_AXIS_NMU[3]),
                (tiles_hnic0[4], bslots::HNOC_NIC_AXI_NMU),
                (tiles_hnic0[5], bslots::HNOC_NIC_XBR2X4),
                (tiles_hnic0[6], bslots::HNOC_NPS[0]),
                (tiles_hnic0[7], bslots::HNOC_NPS[1]),
                (tiles_hnic1[0], bslots::HNOC_NIC_AXIS_NSU[0]),
                (tiles_hnic1[1], bslots::HNOC_NIC_AXIS_NSU[1]),
                (tiles_hnic1[2], bslots::HNOC_NIC_AXIS_NSU[2]),
                (tiles_hnic1[3], bslots::HNOC_NIC_AXIS_NSU[3]),
                (tiles_hnic1[4], bslots::HNOC_NIC_AXI_NSU),
                (tiles_hnic1[5], bslots::HNOC_SCAN[0]),
                (tiles_hnic1[6], bslots::HNOC_NPS[2]),
                (tiles_hnic1[7], bslots::HNOC_NPS[3]),
            ],
        );
        let scan_bli = int
            .xlat_bli(
                tiles_hnic1[5],
                "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
            )
            .unwrap();

        blocks.push(NicBlock::HNicX { scan_bli });
    }

    NicRow { blocks }
}

fn get_hsr_s(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    cols_vnoc: &[ColId],
) -> HsrSKind {
    if !int.find_tiles(&["BLI_LS_MUX_CORE"]).is_empty() {
        HsrSKind::X5io(get_hsr_x5io(int, die, dev_naming))
    } else if !int.find_tiles(&["NOC_PNOC_SSIT_CORE"]).is_empty() {
        get_noc_ssit_s(int, die, dev_naming, cols_vnoc);
        HsrSKind::Ssit
    } else {
        HsrSKind::Xpio(get_hsr_xpio(int, die, dev_naming, DirV::S))
    }
}

fn get_hsr_n(
    int: &IntGridWrapper,
    die: DieId,
    dev_naming: &mut DeviceNaming,
    cols_vnoc: &[ColId],
) -> HsrNKind {
    if !int.find_tiles(&["CMT_MMCM_MX"]).is_empty() {
        HsrNKind::Xpio(get_hsr_xpio(int, die, dev_naming, DirV::N))
    } else if !int.find_tiles(&["NOC_TNOC_ME_CORE_R180"]).is_empty() {
        HsrNKind::Me(get_hsr_me(int, die, dev_naming))
    } else if !int.find_tiles(&["CLKE2_REBUF_HBM_CORE_MX"]).is_empty() {
        HsrNKind::Hbm(get_hsr_hbm(int, die, dev_naming))
    } else if !int.find_tiles(&["NOC_TNOC_BRIDGE_TOP_CORE"]).is_empty() {
        get_noc_ssit_n(int, die, dev_naming, cols_vnoc);
        HsrNKind::Ssit
    } else if !int.find_tiles(&["NOC2_TNOC_NIDB_CORE"]).is_empty() {
        get_noc_ssit_n(int, die, dev_naming, cols_vnoc);
        HsrNKind::Ssit2
    } else if !int.find_tiles(&["NOC2_NMU256_TOP"]).is_empty() {
        HsrNKind::Nic(get_hsr_nic(int, die, dev_naming))
    } else if !int.find_tiles(&["NOC2_NMU128_TOP_MX"]).is_empty() {
        HsrNKind::Me2(get_hsr_me2(int, die, dev_naming))
    } else {
        HsrNKind::None
    }
}

fn get_vnoc_naming(int: &IntGridWrapper, die: DieId, dev_naming: &mut DeviceNaming) {
    for (x, y) in int.find_tiles(&["AMS_SAT_VNOC_TILE"]) {
        let col = int.lookup_column_inter(x) - 1;
        let row = RowId::from_idx(int.lookup_row(y + 1).to_idx() / 48 * 48);
        let tile = &int.int.rd.tiles[&Coord {
            x: x as u16,
            y: y as u16,
        }];
        if let Some(name) = extract_site(int.int.rd, tile, "SYSMON_SAT", 0, 0) {
            dev_naming.bel_names.insert(
                die.cell(col, row).bel(bslots::SYSMON_SAT_VNOC),
                vec![name.into()],
            );
        }
    }
    for (x, y) in int.find_tiles(&["NOC2_NSU512_VNOC_TILE"]) {
        let col = int.lookup_column_inter(x) - 1;
        let row = RowId::from_idx(int.lookup_row(y + 1).to_idx() / 48 * 48);
        let nsu_crd = Coord {
            x: x as u16,
            y: y as u16,
        };
        let mut nps_a_crd = int.delta(nsu_crd, 0, 4);
        if int.int.rd.tile_kinds.key(int.int.rd.tiles[&nps_a_crd].kind) == "NULL" {
            dev_naming.is_vnoc2_scan_offset = true;
            nps_a_crd = int.delta(nps_a_crd, -1, 0);
        }
        let nps_b_crd = int.delta(nps_a_crd, 0, 4);
        let nmu_crd = int.delta(nps_a_crd, 0, 7);
        let scan_crd = int.delta(nsu_crd, 1, 0);
        for (slot, crd, name, x, y) in [
            (bslots::VNOC_NSU512, nsu_crd, "NOC2_NSU512", 0, 0),
            (bslots::VNOC_NMU512, nmu_crd, "NOC2_NMU512", 0, 0),
            (bslots::VNOC_NPS_NSU, nps_a_crd, "NOC2_NPS5555", 0, 0),
            (bslots::VNOC_NPS_NMU, nps_b_crd, "NOC2_NPS5555", 0, 0),
            (bslots::VNOC_SCAN, scan_crd, "NOC2_SCAN", 0, 0),
        ] {
            dev_naming.bel_names.insert(
                die.cell(col, row).bel(slot),
                vec![
                    extract_site(int.int.rd, &int.int.rd.tiles[&crd], name, x, y)
                        .unwrap()
                        .into(),
                ],
            );
        }
    }
    for (x, y) in int.find_tiles(&["NOC2_NSU512_VNOC4_TILE"]) {
        let col = int.lookup_column_inter(x) - 1;
        let row = RowId::from_idx(int.lookup_row(y + 1).to_idx() / 48 * 48);
        let nsu_crd = Coord {
            x: x as u16,
            y: y as u16,
        };
        let mut nps_a_crd = int.delta(nsu_crd, 0, 4);
        if int.int.rd.tile_kinds.key(int.int.rd.tiles[&nps_a_crd].kind) == "NULL" {
            dev_naming.is_vnoc2_scan_offset = true;
            nps_a_crd = int.delta(nps_a_crd, -1, 0);
        }
        let nps_b_crd = int.delta(nps_a_crd, 0, 4);
        let nmu_crd = int.delta(nps_a_crd, 0, 7);
        let scan_crd = int.delta(nsu_crd, 1, 0);
        for (slot, crd, name, x, y) in [
            (bslots::VNOC_NSU512, nsu_crd, "NOC2_NSU512", 0, 0),
            (bslots::VNOC_NMU512, nmu_crd, "NOC2_NMU512", 0, 0),
            (bslots::VNOC_NPS6X_NSU, nps_a_crd, "NOC2_NPS6X", 0, 0),
            (bslots::VNOC_NPS6X_NMU, nps_b_crd, "NOC2_NPS6X", 0, 0),
            (bslots::VNOC_SCAN, scan_crd, "NOC2_SCAN", 0, 0),
        ] {
            dev_naming.bel_names.insert(
                die.cell(col, row).bel(slot),
                vec![
                    extract_site(int.int.rd, &int.int.rd.tiles[&crd], name, x, y)
                        .unwrap()
                        .into(),
                ],
            );
        }
    }
}

fn get_gt_naming(int: &IntGridWrapper, die: DieId, naming: &mut DeviceNaming) {
    for tkn in [
        "AMS_SAT_GT_BOT_TILE",
        "AMS_SAT_GT_TOP_TILE",
        "AMS_SAT_GT_BOT_TILE_MY",
        "AMS_SAT_GT_TOP_TILE_MY",
    ] {
        for (x, y) in int.find_tiles(&[tkn]) {
            let xy = Coord {
                x: x as u16,
                y: y as u16,
            };
            let mut col = int.lookup_column_inter(x);
            if col.to_idx() != 0 {
                col -= 1;
            }
            let row = RowId::from_idx(int.lookup_row(y + 1).to_idx() / 48 * 48);
            let tile = &int.int.rd.tiles[&xy];
            if let Some(name) = extract_site(int.int.rd, tile, "SYSMON_SAT", 0, 0) {
                naming.bel_names.insert(
                    die.cell(col, row).bel(bslots::SYSMON_SAT_GT),
                    vec![name.into()],
                );
            }
            let tile = &int.int.rd.tiles[&int.delta(xy, 0, -15)];
            if let Some(name) = extract_site(int.int.rd, tile, "DPLL", 0, 0) {
                naming
                    .bel_names
                    .insert(die.cell(col, row).bel(bslots::DPLL_GT), vec![name.into()]);
            }
        }
    }
}

fn get_clk_naming(int: &IntGridWrapper, die: DieId, naming: &mut DeviceNaming, col_cfrm: ColId) {
    for (tkns, tslot) in [
        (clk::CLK_VNOC_TILE_KINDS, tslots::CLK_INT),
        (clk::CLK_VNOC_BOT_TILE_KINDS, tslots::CLK_V_BUF),
        (clk::CLK_VNOC_REBUF_TILE_KINDS, tslots::CLK_V_BUF),
        (clk::CLK_GT_TILE_KINDS, tslots::CLK_INT),
        (clk::CLK_GT_BOT_TILE_KINDS, tslots::CLK_V_BUF),
        (clk::CLK_GT_REBUF_TILE_KINDS, tslots::CLK_V_BUF),
        (
            &[
                "RCLK_BRAM_CLKBUF_CORE",
                "RCLK_BRAM_CLKBUF_VR_CORE",
                "RCLK_BRAM_CLKBUF_NOPD_CORE",
                "RCLK_BRAM_CLKBUF_NOPD_VR_CORE",
                "RCLK_CIDB_CORE",
            ],
            tslots::CLK_INT,
        ),
    ] {
        for (x, y) in int.find_tiles(tkns) {
            let crd = Coord {
                x: x as u16,
                y: y as u16,
            };
            let anchor = int.delta(crd, 0, 1);
            let tile = &int.int.rd.tiles[&crd];
            let tkn = int.int.rd.tile_kinds.key(tile.kind);
            let col = int.lookup_column_inter(x);
            let col = if tkn == "CLKE2_GT_ULVT_TILE_MY" {
                col_cfrm
            } else if col.to_idx() == 0 {
                col
            } else {
                col - 1
            };
            let row = RowId::from_idx(int.lookup_row(anchor.y.into()).to_idx() / 48 * 48);
            let cell = die.cell(col, row);
            naming
                .tile_names
                .insert(cell.tile(tslot), vec![tile.name.clone()]);
            let tk = &int.int.rd.tile_kinds[tile.kind];
            for (sid, &slot, _) in &tk.sites {
                let TkSiteSlot::Xy(slot, _sx, sy) = slot else {
                    continue;
                };
                let sy = sy as usize;
                let bslot = match int.int.rd.slot_kinds[slot].as_str() {
                    "GCLK_TEST" | "GCLK_TEST_ULVT" => match tslot {
                        tslots::CLK_INT => bslots::GCLK_TEST[sy / 2 + sy % 2 * 24],
                        tslots::CLK_V_BUF => {
                            if tkn.contains("_GT_EDGE") {
                                match sy / 24 {
                                    0 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    1 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV1
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    2 => bslots::GCLK_TEST_RING_VDISTR[sy % 12 * 2 + sy / 12 % 2],
                                    3 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2_N
                                            [(sy % 6 * 2 + sy / 6 % 2) * 2]
                                    }
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_EDGE") {
                                match sy % 4 {
                                    0 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[sy / 2],
                                    1 => bslots::GCLK_TEST_RING_VDISTR[sy / 2],
                                    2 => bslots::GCLK_TEST_RING_VDISTR[sy / 2],
                                    3 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[sy / 2],
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_SSIT_HSR")
                                || tkn.contains("_SSIT_MB")
                                || tkn.contains("_SSIT_MT")
                            {
                                match sy / 12 {
                                    0 => bslots::GCLK_TEST_RING_VDISTR_LEV2[sy % 12 * 2],
                                    1 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[sy % 12 * 2 + 1],
                                    2 => bslots::GCLK_TEST_RING_VDISTR_LEV1[sy % 12 * 2],
                                    3 => bslots::GCLK_TEST_RING_VDISTR_LEV1_N[sy % 12 * 2 + 1],
                                    4 => bslots::GCLK_TEST_RING_VDISTR[sy % 12 * 2],
                                    5 => bslots::GCLK_TEST_RING_VDISTR_N[sy % 12 * 2 + 1],
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_GT_SSIT_TOP") || tkn.contains("_GT_SSIT_HBM") {
                                match sy {
                                    0..12 => bslots::GCLK_TEST_RING_VDISTR_LEV2[sy % 12 * 2],
                                    12..36 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV1
                                            [sy % 12 * 2 + (sy - 12) / 12]
                                    }
                                    36..60 => {
                                        bslots::GCLK_TEST_RING_VDISTR[sy % 12 * 2 + (sy - 36) / 12]
                                    }
                                    60..72 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[sy % 12 * 2],
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_GT_SSIT_BOT") {
                                match sy {
                                    0..12 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[sy % 12 * 2],
                                    12..36 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV1_N
                                            [sy % 12 * 2 + (sy - 12) / 12]
                                    }
                                    36..60 => {
                                        bslots::GCLK_TEST_RING_VDISTR_N
                                            [sy % 12 * 2 + (sy - 36) / 12]
                                    }
                                    60..72 => bslots::GCLK_TEST_RING_VDISTR_LEV2[sy % 12 * 2],
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_SSIT_TOP")
                                || tkn.contains("_SSIT_AIE")
                                || tkn.contains("_SSIT_HBM")
                            {
                                match sy / 24 {
                                    0 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    1 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV1
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    2 => bslots::GCLK_TEST_RING_VDISTR[sy % 12 * 2 + sy / 12 % 2],
                                    3 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2_N
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    _ => unreachable!(),
                                }
                            } else if tkn.contains("_SSIT_BOT") {
                                match sy / 24 {
                                    0 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2_N
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    1 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV1_N
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    2 => bslots::GCLK_TEST_RING_VDISTR_N[sy % 12 * 2 + sy / 12 % 2],
                                    3 => {
                                        bslots::GCLK_TEST_RING_VDISTR_LEV2
                                            [sy % 12 * 2 + sy / 12 % 2]
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                continue;
                            }
                        }
                        _ => unreachable!(),
                    },
                    "GCLK_TEST_RING" => {
                        if sy < 24 {
                            bslots::GCLK_TEST_RING_HDISTR[sy]
                        } else {
                            bslots::GCLK_TEST_RING_HROUTE[sy - 24]
                        }
                    }
                    "BUFG_FABRIC" | "BUFG_FABRIC_ULVT" => bslots::BUFG_FABRIC[sy],
                    "GCLK_DELAY" | "GCLK_DELAY_SSIT" | "GCLK_DELAY_SSIT2" => bslots::GCLK_DELAY[sy],
                    "GCLK_TAPS_DECODE_VNOC_PS" | "GCLK_TAPS_DECODE_VNOC" => {
                        bslots::GCLK_TAPS_DECODE_VNOC
                    }
                    "GCLK_PD" => match tslot {
                        tslots::CLK_INT => bslots::GCLK_PD_H[sy],
                        tslots::CLK_V_BUF => bslots::GCLK_PD_V[sy],
                        _ => unreachable!(),
                    },
                    "BUFG_PS" | "BUFG_PS_ULVT" => bslots::BUFG_PS[sy],
                    _ => continue,
                };
                naming
                    .bel_names
                    .insert(cell.bel(bslot), vec![tile.sites[sid].clone()]);
            }
        }
    }
}

fn get_vcu2_naming(int: &IntGridWrapper, die: DieId, naming: &mut DeviceNaming) {
    for (tkn, bslots, dys) in [
        (
            "VCU2_TILE",
            [
                bslots::VCU2_NMU[2],
                bslots::VCU2_NMU[3],
                bslots::VCU2_NMU[0],
                bslots::VCU2_NMU[1],
                bslots::VCU2_NSU,
                bslots::VCU2_NOC_SCAN,
                bslots::VCU2_NPS6X[2],
                bslots::VCU2_NPS6X[3],
                bslots::VCU2_NPS6X[0],
                bslots::VCU2_NPS6X[1],
            ]
            .as_slice(),
            [0, 3, 7, 11, 13].as_slice(),
        ),
        (
            "ISP2_CORE",
            [
                bslots::VCU2_NMU[0],
                bslots::VCU2_NMU[1],
                bslots::VCU2_NSU,
                bslots::VCU2_NOC_SCAN,
                bslots::VCU2_NPS6X[1],
                bslots::VCU2_NPS6X[0],
            ]
            .as_slice(),
            [0, 3, 7].as_slice(),
        ),
    ] {
        for (x, y) in int.find_tiles(&[tkn]) {
            let crd = Coord {
                x: x as u16,
                y: y as u16,
            };
            let mut anchor = crd;
            while !int
                .int
                .rd
                .tile_kinds
                .key(int.int.rd.tiles[&anchor].kind)
                .starts_with("NOC")
            {
                anchor = int.delta(anchor, 1, 0);
            }
            let mut col = int.lookup_column_inter(x);
            if col.to_idx() != 0 {
                col -= 1;
            }
            let row = RowId::from_idx(int.lookup_row(y).to_idx() / 48 * 48);
            let cell = die.cell(col, row);

            let mut tile_names = vec![int.int.rd.tiles[&crd].name.clone()];
            for (i, &bslot) in bslots.iter().enumerate() {
                let ncrd = int.delta(anchor, (i % 2) as i32, dys[i / 2]);
                let tile = &int.int.rd.tiles[&ncrd];
                tile_names.push(tile.name.clone());
                naming.bel_names.insert(
                    cell.bel(bslot),
                    vec![tile.sites.values().next().unwrap().clone()],
                );
            }
            naming.tile_names.insert(cell.tile(tslots::BEL), tile_names);
        }
    }
}

fn get_chip(
    die: DieId,
    int: &IntGridWrapper<'_>,
    disabled: &mut BTreeSet<DisabledPart>,
    dev_naming: &mut DeviceNaming,
    sll_columns: &mut EntityVec<DieId, Vec<ColId>>,
) -> Chip {
    let (columns, cols_hard) = make_columns(die, int, disabled, dev_naming);
    let ps = if !int.find_tiles(&["PSS_BASE_CORE"]).is_empty() {
        if !int.find_tiles(&["XRAM_CORE"]).is_empty() {
            PsKind::Ps9Xram
        } else if !int.find_tiles(&["CPM_CORE"]).is_empty() {
            PsKind::Ps9Cpm4
        } else if !int.find_tiles(&["CPM_G5_TILE"]).is_empty() {
            PsKind::Ps9Cpm5
        } else if !int.find_tiles(&["CPM6_TILE"]).is_empty() {
            PsKind::Ps9Cpm6
        } else {
            PsKind::Ps9
        }
    } else if !int.find_tiles(&["PSXL_CORE"]).is_empty() {
        PsKind::PsXCpm5N
    } else if !int.find_tiles(&["PSXC_TILE"]).is_empty() {
        PsKind::PsXcMmi
    } else {
        unreachable!()
    };
    assert_eq!(int.int.rows.len() % 48, 0);
    let regs_gt_w = get_rows_gt_w(int);
    let regs_gt_e = get_rows_gt_e(int);
    let fabric = get_fabric(int.int.rd);
    let cols_vnoc = Vec::from_iter(
        columns
            .iter()
            .filter_map(|(col, cd)| if cd.kind.is_vnoc() { Some(col) } else { None }),
    );
    let noc_s = get_hsr_s(int, die, dev_naming, &cols_vnoc);
    let noc_n = get_hsr_n(int, die, dev_naming, &cols_vnoc);
    let chip = Chip {
        columns,
        cols_vbrk: get_cols_vbrk(int),
        cols_cpipe: get_cols_cpipe(int),
        cols_hard,
        regs: int.int.rows.len() / 48,
        regs_gt_w,
        regs_gt_e,
        fabric,
        ps,
        hsr_n: noc_n,
        hsr_s: noc_s,
    };
    let col_cfrm = chip
        .columns
        .iter()
        .find(|(_, cd)| cd.kind == ColumnKind::Cfrm)
        .unwrap()
        .0;
    get_vnoc_naming(int, die, dev_naming);
    get_gt_naming(int, die, dev_naming);
    get_clk_naming(int, die, dev_naming, col_cfrm);
    get_vcu2_naming(int, die, dev_naming);
    let mut die_sll_columns = BTreeSet::new();
    for (x, y) in int.find_tiles(&["SLL", "SLL2"]) {
        let crd = Coord {
            x: x as u16,
            y: y as u16,
        };
        let Some(nw) = int.int.rd.lookup_wire(crd, "UBUMP2") else {
            continue;
        };
        let NodeOrWire::Node(node) = nw else {
            continue;
        };
        let node = &int.int.rd.nodes[node];
        let templ = &int.int.rd.templates[node.template];
        if templ.len() > 1 {
            let col = int.lookup_column_inter(x) - 1;
            die_sll_columns.insert(col);
        }
    }
    sll_columns.push(Vec::from_iter(die_sll_columns));
    chip
}

pub fn make_chips(
    rd: &Part,
) -> (
    EntityVec<DieId, Chip>,
    Interposer,
    BTreeSet<DisabledPart>,
    DeviceNaming,
) {
    let mut disabled = BTreeSet::new();
    let crd = rd.tiles_by_kind_name("INT").first().unwrap();
    let tile = &rd.tiles[crd];
    let mut ikind = if tile.name.contains("_S") {
        InterposerKind::MirrorSquare
    } else {
        InterposerKind::Column
    };
    let mut chips = EntityVec::new();
    let mut dev_naming = DeviceNaming {
        tile_names: Default::default(),
        bel_names: Default::default(),
        skip_bels: Default::default(),
        has_dsp_v2: false,
        has_term2: rd.tile_kinds.contains_key("RCLK_INTF_TERM2_RIGHT_CORE"),
        is_vnoc2_scan_offset: false,
    };
    let mut sll_columns = EntityVec::new();
    let bli = get_bli(rd);
    if ikind == InterposerKind::Column {
        let mut rows_slr_split: BTreeSet<_> = find_rows(rd, &["NOC_TNOC_BRIDGE_BOT_CORE"])
            .into_iter()
            .map(|r| r as u16)
            .collect();
        if rows_slr_split.is_empty() {
            ikind = InterposerKind::Single;
        }
        rows_slr_split.insert(0);
        rows_slr_split.insert(rd.height);
        let rows_slr_split: Vec<_> = rows_slr_split.iter().collect();
        for (dieid, w) in rows_slr_split.windows(2).enumerate() {
            let int = extract_int_slr_column(rd, &["INT"], &[], *w[0], *w[1]);
            let int = IntGridWrapper { int, bli: &bli };
            let die = DieId::from_idx(dieid);
            let chip = get_chip(die, &int, &mut disabled, &mut dev_naming, &mut sll_columns);
            chips.push(chip);
        }
    } else {
        for (dieid, int) in [
            extract_int_slr(
                rd,
                &["INT"],
                &[],
                0,
                rd.width / 2,
                0,
                rd.height / 2,
                false,
                false,
            ),
            extract_int_slr(
                rd,
                &["INT"],
                &[],
                0,
                rd.width / 2,
                rd.height / 2,
                rd.height,
                false,
                true,
            ),
            extract_int_slr(
                rd,
                &["INT"],
                &[],
                rd.width / 2,
                rd.width,
                rd.height / 2,
                rd.height,
                true,
                true,
            ),
            extract_int_slr(
                rd,
                &["INT"],
                &[],
                rd.width / 2,
                rd.width,
                0,
                rd.height / 2,
                true,
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let int = IntGridWrapper { int, bli: &bli };
            let die = DieId::from_idx(dieid);
            let chip = get_chip(die, &int, &mut disabled, &mut dev_naming, &mut sll_columns);
            chips.push(chip);
        }
    }
    if rd.part.contains("vc1502") {
        let s0 = DieId::from_idx(0);
        assert_eq!(chips[s0].regs, 7);
        let col_hard_r = &mut chips[s0].cols_hard[1];
        for (reg, kind) in [(0, HardRowKind::Mrmac), (6, HardRowKind::Hdio)] {
            let reg = RegId::from_idx(reg);
            assert_eq!(col_hard_r.regs[reg], HardRowKind::None);
            col_hard_r.regs[reg] = kind;
            disabled.insert(DisabledPart::HardIp(s0, col_hard_r.col, reg));
        }
        for reg in [0, 1, 6] {
            let reg = RegId::from_idx(reg);
            assert_eq!(chips[s0].regs_gt_e[reg], GtRowKind::None);
            chips[s0].regs_gt_e[reg] = GtRowKind::Gty;
            disabled.insert(DisabledPart::GtE(s0, reg));
        }
    }
    if rd.part.contains("vm1302") {
        let s0 = DieId::from_idx(0);
        assert_eq!(chips[s0].regs, 9);
        assert_eq!(chips[s0].columns.len(), 76);
        while chips[s0].columns.len() != 122 {
            chips[s0].columns.push(Column {
                kind: ColumnKind::None,
                has_bli_s: false,
                has_bli_n: false,
            });
        }
        for i in [
            73, 75, 77, 81, 83, 87, 89, 91, 95, 97, 99, 103, 105, 107, 111, 113, 117, 119,
        ] {
            let col = ColId::from_idx(i);
            chips[s0].columns[col].kind = ColumnKind::Cle(CleKind::Plain);
            chips[s0].columns[col + 1].kind = ColumnKind::Cle(CleKind::Plain);
            chips[s0].columns[col].has_bli_s = true;
            chips[s0].columns[col].has_bli_n = true;
            chips[s0].columns[col + 1].has_bli_s = true;
            chips[s0].columns[col + 1].has_bli_n = true;
        }
        for i in [79, 109] {
            let col = ColId::from_idx(i);
            chips[s0].columns[col].kind = ColumnKind::Dsp;
            chips[s0].columns[col + 1].kind = ColumnKind::ContDsp;
            chips[s0].columns[col].has_bli_s = true;
            chips[s0].columns[col].has_bli_n = true;
            chips[s0].columns[col + 1].has_bli_s = true;
            chips[s0].columns[col + 1].has_bli_n = true;
        }
        for i in [72, 86, 116] {
            let col = ColId::from_idx(i);
            chips[s0].columns[col].kind = ColumnKind::Bram(BramKind::Plain);
        }
        for i in [85, 101, 115] {
            let col = ColId::from_idx(i);
            chips[s0].columns[col].kind = ColumnKind::Bram(BramKind::Plain);
        }
        let col = ColId::from_idx(100);
        chips[s0].columns[col].kind = ColumnKind::Uram;
        chips[s0].columns[col].has_bli_n = true;
        chips[s0].columns[col - 1].has_bli_n = true;
        let col = ColId::from_idx(93);
        chips[s0].columns[col].kind = ColumnKind::VNoc;
        chips[s0].columns[col + 1].kind = ColumnKind::VNoc;
        let col = ColId::from_idx(121);
        chips[s0].columns[col].kind = ColumnKind::Term;
        for i in [74, 82, 92, 96, 106, 114, 118] {
            chips[s0].cols_vbrk.insert(ColId::from_idx(i));
        }
        for i in [86, 102] {
            chips[s0].cols_cpipe.insert(ColId::from_idx(i));
        }
        for i in 72..122 {
            disabled.insert(DisabledPart::Column(s0, ColId::from_idx(i)));
        }
    }
    if rd.part.contains("vp1002") {
        let s0 = DieId::from_idx(0);
        assert_eq!(chips[s0].regs, 11);
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(8)));
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(9)));
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(10)));
        let col_hard_l = &mut chips[s0].cols_hard[0];
        col_hard_l.regs[RegId::from_idx(8)] = HardRowKind::Dcmac;
        col_hard_l.regs[RegId::from_idx(10)] = HardRowKind::Mrmac;
        let col_hard_r = &mut chips[s0].cols_hard[1];
        col_hard_r.regs[RegId::from_idx(8)] = HardRowKind::Ilkn;
        col_hard_r.regs[RegId::from_idx(10)] = HardRowKind::Mrmac;
        chips[s0].regs_gt_w[RegId::from_idx(8)] = GtRowKind::Gtm;
        chips[s0].regs_gt_w[RegId::from_idx(9)] = GtRowKind::Gtm;
        chips[s0].regs_gt_w[RegId::from_idx(10)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(8)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(9)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(10)] = GtRowKind::Gtm;
    }
    if rd.part.contains("vp1102") {
        let s0 = DieId::from_idx(0);
        assert_eq!(chips[s0].regs, 14);
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(10)));
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(11)));
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(12)));
        disabled.insert(DisabledPart::Region(s0, RegId::from_idx(13)));
        let col_hard_l = &mut chips[s0].cols_hard[0];
        col_hard_l.regs[RegId::from_idx(10)] = HardRowKind::Dcmac;
        col_hard_l.regs[RegId::from_idx(12)] = HardRowKind::Dcmac;
        let col_hard_m = &mut chips[s0].cols_hard[1];
        col_hard_m.regs[RegId::from_idx(10)] = HardRowKind::Hsc;
        col_hard_m.regs[RegId::from_idx(12)] = HardRowKind::Hdio;
        col_hard_m.regs[RegId::from_idx(13)] = HardRowKind::Hdio;
        let col_hard_r = &mut chips[s0].cols_hard[2];
        col_hard_r.regs[RegId::from_idx(10)] = HardRowKind::Dcmac;
        col_hard_r.regs[RegId::from_idx(12)] = HardRowKind::Dcmac;
        chips[s0].regs_gt_w[RegId::from_idx(10)] = GtRowKind::Gtm;
        chips[s0].regs_gt_w[RegId::from_idx(11)] = GtRowKind::Gtm;
        chips[s0].regs_gt_w[RegId::from_idx(12)] = GtRowKind::Gtm;
        chips[s0].regs_gt_w[RegId::from_idx(13)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(10)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(11)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(12)] = GtRowKind::Gtm;
        chips[s0].regs_gt_e[RegId::from_idx(13)] = GtRowKind::Gtm;
    }
    dev_naming.has_dsp_v2 = rd.wires.contains("DSP_DSP58_4_CLK");
    let interposer = Interposer {
        kind: ikind,
        sll_columns,
    };
    (chips, interposer, disabled, dev_naming)
}
