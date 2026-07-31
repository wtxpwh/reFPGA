use std::collections::HashMap;

use prjcombine_entity::{EntityId, EntityPartVec};
use prjcombine_interconnect::{
    db::{BelInfo, CellSlotId, ConnectorWire, TileClassId, TileWireCoord, WireKind, WireSlotIdExt},
    dir::{Dir, DirH, DirV},
};
use prjcombine_re_xilinx_rawdump::{Coord, TkWire};
use prjcombine_re_xilinx_rd2db_interconnect::{ExtrBelInfo, PipMode, XTileResult};
use prjcombine_versal::defs::{bslots, ccls, cslots, tcls, wires};

use crate::int::{IntMaker, XTileInfoExt};

pub const INTF_KINDS: &[(DirH, &str, TileClassId, &str, bool, bool)] = &[
    (
        DirH::W,
        "INTF_LOCF_BL_TILE",
        tcls::INTF_W,
        "INTF_W",
        false,
        false,
    ),
    (
        DirH::W,
        "INTF_LOCF_TL_TILE",
        tcls::INTF_W,
        "INTF_W",
        false,
        false,
    ),
    (
        DirH::E,
        "INTF_LOCF_BR_TILE",
        tcls::INTF_E,
        "INTF_E",
        false,
        false,
    ),
    (
        DirH::E,
        "INTF_LOCF_TR_TILE",
        tcls::INTF_E,
        "INTF_E",
        false,
        false,
    ),
    (
        DirH::W,
        "INTF_ROCF_BL_TILE",
        tcls::INTF_W,
        "INTF_W",
        false,
        false,
    ),
    (
        DirH::W,
        "INTF_ROCF_TL_TILE",
        tcls::INTF_W,
        "INTF_W",
        false,
        false,
    ),
    (
        DirH::E,
        "INTF_ROCF_BR_TILE",
        tcls::INTF_E,
        "INTF_E",
        false,
        false,
    ),
    (
        DirH::E,
        "INTF_ROCF_TR_TILE",
        tcls::INTF_E,
        "INTF_E",
        false,
        false,
    ),
    (
        DirH::W,
        "INTF_HB_LOCF_BL_TILE",
        tcls::INTF_W_HB,
        "INTF_W_HB",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HB_LOCF_TL_TILE",
        tcls::INTF_W_HB,
        "INTF_W_HB",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HB_LOCF_BR_TILE",
        tcls::INTF_E_HB,
        "INTF_E_HB",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HB_LOCF_TR_TILE",
        tcls::INTF_E_HB,
        "INTF_E_HB",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HB_ROCF_BL_TILE",
        tcls::INTF_W_HB,
        "INTF_W_HB",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HB_ROCF_TL_TILE",
        tcls::INTF_W_HB,
        "INTF_W_HB",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HB_ROCF_BR_TILE",
        tcls::INTF_E_HB,
        "INTF_E_HB",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HB_ROCF_TR_TILE",
        tcls::INTF_E_HB,
        "INTF_E_HB",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HDIO_LOCF_BL_TILE",
        tcls::INTF_W_HDIO,
        "INTF_W_HDIO",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HDIO_LOCF_TL_TILE",
        tcls::INTF_W_HDIO,
        "INTF_W_HDIO",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HDIO_LOCF_BR_TILE",
        tcls::INTF_E_HDIO,
        "INTF_E_HDIO",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HDIO_LOCF_TR_TILE",
        tcls::INTF_E_HDIO,
        "INTF_E_HDIO",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HDIO_ROCF_BL_TILE",
        tcls::INTF_W_HDIO,
        "INTF_W_HDIO",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_HDIO_ROCF_TL_TILE",
        tcls::INTF_W_HDIO,
        "INTF_W_HDIO",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HDIO_ROCF_BR_TILE",
        tcls::INTF_E_HDIO,
        "INTF_E_HDIO",
        false,
        true,
    ),
    (
        DirH::E,
        "INTF_HDIO_ROCF_TR_TILE",
        tcls::INTF_E_HDIO,
        "INTF_E_HDIO",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_CFRM_BL_TILE",
        tcls::INTF_W_PSS,
        "INTF_W_PSS",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_CFRM_TL_TILE",
        tcls::INTF_W_PSS,
        "INTF_W_PSS",
        false,
        true,
    ),
    (
        DirH::W,
        "INTF_PSS_BL_TILE",
        tcls::INTF_W_TERM_PSS,
        "INTF_W_TERM_PSS",
        true,
        true,
    ),
    (
        DirH::W,
        "INTF_PSS_TL_TILE",
        tcls::INTF_W_TERM_PSS,
        "INTF_W_TERM_PSS",
        true,
        true,
    ),
    (
        DirH::W,
        "INTF_GT_BL_TILE",
        tcls::INTF_W_TERM_GT,
        "INTF_W_TERM_GT",
        true,
        true,
    ),
    (
        DirH::W,
        "INTF_GT_TL_TILE",
        tcls::INTF_W_TERM_GT,
        "INTF_W_TERM_GT",
        true,
        true,
    ),
    (
        DirH::E,
        "INTF_GT_BR_TILE",
        tcls::INTF_E_TERM_GT,
        "INTF_E_TERM_GT",
        true,
        true,
    ),
    (
        DirH::E,
        "INTF_GT_TR_TILE",
        tcls::INTF_E_TERM_GT,
        "INTF_E_TERM_GT",
        true,
        true,
    ),
];

#[allow(clippy::type_complexity)]
pub const BLI_CLE_INTF_KINDS: &[(
    DirH,
    &str,
    [TileClassId; 4],
    [&str; 4],
    TileClassId,
    &str,
    bool,
)] = &[
    (
        DirH::E,
        "BLI_CLE_BOT_CORE",
        [
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_D_S,
        ],
        [
            "INTF_BLI_CLE_A_S",
            "INTF_BLI_CLE_B_S",
            "INTF_BLI_CLE_C_S",
            "INTF_BLI_CLE_D_S",
        ],
        tcls::BLI_CLE_S,
        "BLI_CLE_S",
        false,
    ),
    (
        DirH::E,
        "BLI_CLE_TOP_CORE",
        [
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_D_N,
        ],
        [
            "INTF_BLI_CLE_A_N",
            "INTF_BLI_CLE_B_N",
            "INTF_BLI_CLE_C_N",
            "INTF_BLI_CLE_D_N",
        ],
        tcls::BLI_CLE_N,
        "BLI_CLE_N",
        true,
    ),
    (
        DirH::W,
        "BLI_CLE_BOT_CORE_MY",
        [
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_ABC_S,
            tcls::INTF_BLI_CLE_D_S,
        ],
        [
            "INTF_BLI_CLE_A_S",
            "INTF_BLI_CLE_B_S",
            "INTF_BLI_CLE_C_S",
            "INTF_BLI_CLE_D_S",
        ],
        tcls::BLI_CLE_S,
        "BLI_CLE_S",
        false,
    ),
    (
        DirH::W,
        "BLI_CLE_TOP_CORE_MY",
        [
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_ABC_N,
            tcls::INTF_BLI_CLE_D_N,
        ],
        [
            "INTF_BLI_CLE_A_N",
            "INTF_BLI_CLE_B_N",
            "INTF_BLI_CLE_C_N",
            "INTF_BLI_CLE_D_N",
        ],
        tcls::BLI_CLE_N,
        "BLI_CLE_N",
        true,
    ),
];

impl<'rd> IntMaker<'rd> {
    pub fn fill_tiles_int(&mut self) {
        self.builder
            .int_type_id(tcls::INT, bslots::INT, "INT", "INT");
        let pips = self
            .builder
            .pips
            .get_mut(&(tcls::INT, bslots::INT))
            .unwrap();
        pips.pips = pips
            .pips
            .iter()
            .map(|(&(wt, wf), &mode)| {
                if wires::INODE.contains(wt.wire) || wires::SDQNODE.contains(wt.wire) {
                    let nwf = self.sng_fixup_map.get(&wf.tw).copied().unwrap_or(wf.tw);
                    ((wt, nwf.pos()), mode)
                } else {
                    ((wt, wf), mode)
                }
            })
            .collect();
        let naming = self.builder.ndb.get_tile_class_naming("INT");
        let naming = &mut self.builder.ndb.tile_class_namings[naming];
        for (&wf, &wt) in &self.sng_fixup_map {
            let name = naming.wires[&wf].clone();
            naming.wires.insert(wt, name);
        }
    }

    pub fn fill_tiles_cle_bc(&mut self) {
        for (tcid, naming, tkn) in [
            (tcls::CLE_BC, "CLE_BC", "CLE_BC_CORE"),
            (tcls::CLE_BC, "CLE_BC", "CLE_BC_CORE_MX"),
            (tcls::CLE_BC_SLL, "CLE_BC_SLL", "SLL"),
            (tcls::CLE_BC_SLL2, "CLE_BC_SLL2", "SLL2"),
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let td = &self.rd.tiles[&self.builder.delta(xy, 0, -1)];
                if self.rd.tile_kinds.key(td.kind) != tkn {
                    continue;
                }
                let tu = &self.rd.tiles[&self.builder.delta(xy, 0, 1)];
                if self.rd.tile_kinds.key(tu.kind) != tkn {
                    continue;
                }
                let int_xy_w = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let int_xy_e = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
                let mut bels = vec![];
                if tcid != tcls::CLE_BC {
                    for i in 0..6 {
                        let mut bel = self.builder.bel_virtual(bslots::LAGUNA[i]);
                        bel = bel
                            .extra_int_in("TX", &[format!("LAG_CASCOUT_TXI{i}")])
                            .extra_int_out("RX", &[format!("LAG_OUT{i}")])
                            .extra_wire("UBUMP", &[format!("UBUMP{i}")]);
                        bels.push(bel);
                    }
                }
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .ref_int_side(int_xy_w, DirH::E, 0)
                    .ref_int_side(int_xy_e, DirH::W, 1)
                    .extract_muxes(bslots::CLE_BC_INT)
                    .skip_muxes(&wires::OUT_LAG[..])
                    .bels(bels)
                    .extract();
                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                let naming = self.builder.ndb.get_tile_class_naming("CLE_BC");
                let int_naming = self.builder.ndb.get_tile_class_naming("INT");
                for (int_xy, int_subtile, cle_tile, side) in [
                    (
                        int_xy_e,
                        CellSlotId::from_idx(0),
                        CellSlotId::from_idx(1),
                        DirH::W,
                    ),
                    (
                        int_xy_w,
                        CellSlotId::from_idx(1),
                        CellSlotId::from_idx(0),
                        DirH::E,
                    ),
                ] {
                    let naming = &self.builder.ndb.tile_class_namings[naming];
                    let mut nodes = HashMap::new();
                    for w in wires::BNODE_CLE.into_iter().chain(wires::CNODE_CLE) {
                        if let Some(n) = naming.wires.get(&TileWireCoord {
                            cell: cle_tile,
                            wire: w,
                        }) {
                            let n = self.rd.wires.get(&n.name).unwrap();
                            if let &TkWire::Connected(idx) = tk.wires.get(&n).unwrap().1 {
                                nodes.insert(tile.conn_wires[idx], w);
                            }
                        }
                    }
                    let int_tile = &self.rd.tiles[&int_xy];
                    let int_tk = &self.rd.tile_kinds[int_tile.kind];
                    let int_naming = &self.builder.ndb.tile_class_namings[int_naming];
                    for w in wires::IMUX_BOUNCE {
                        if let Some(n) = int_naming.wires.get(&TileWireCoord {
                            cell: int_subtile,
                            wire: w,
                        }) {
                            let n = self.rd.wires.get(&n.name).unwrap();
                            if let &TkWire::Connected(idx) = int_tk.wires.get(&n).unwrap().1 {
                                nodes.insert(int_tile.conn_wires[idx], w);
                            }
                        }
                    }
                    let mut wires = EntityPartVec::new();
                    for w in wires::BNODE {
                        if self.builder.db[w].kind != WireKind::Branch(cslots::INTF) {
                            continue;
                        }
                        if let Some(n) = int_naming.wires.get(&TileWireCoord {
                            cell: int_subtile,
                            wire: w,
                        }) {
                            let n = self.rd.wires.get(&n.name).unwrap();
                            if let &TkWire::Connected(idx) = int_tk.wires.get(&n).unwrap().1
                                && let Some(&cw) = nodes.get(&int_tile.conn_wires[idx])
                            {
                                wires.insert(w, ConnectorWire::Reflect(cw));
                            }
                        }
                    }
                    self.builder.insert_connector(
                        match side {
                            DirH::W => ccls::CLE_W,
                            DirH::E => ccls::CLE_E,
                        },
                        wires.clone(),
                    );
                    self.builder.insert_connector(
                        match side {
                            DirH::W => ccls::CLE_BLI_W,
                            DirH::E => ccls::CLE_BLI_E,
                        },
                        wires,
                    )
                }
                break;
            }
        }
    }

    pub fn fill_tiles_intf(&mut self) {
        for &(side, tkn, tcid, naming, _, delay) in INTF_KINDS {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let td = &self.rd.tiles[&self.builder.delta(xy, 0, -1)];
                if self.rd.tile_kinds.key(td.kind) != tkn {
                    continue;
                }
                let tu = &self.rd.tiles[&self.builder.delta(xy, 0, 1)];
                if self.rd.tile_kinds.key(tu.kind) != tkn {
                    continue;
                }
                let int_xy = self.builder.walk_to_int(xy, (!side).into(), false).unwrap();
                let mut bels = vec![];
                for i in 0..4 {
                    let mut bel = self.builder.bel_xy(bslots::IRI[i], "IRI_QUAD", 0, i);
                    for i in 0..4 {
                        bel = bel
                            .pin_rename(format!("CE{i}"), format!("CE_{i}_"))
                            .pin_rename(format!("CE{i}_O"), format!("CE_O_{i}_"));
                    }
                    for i in 0..24 {
                        bel = bel
                            .pin_rename(format!("IMUX_IN{i}"), format!("IMUX_{i}_"))
                            .pin_rename(format!("IMUX_O{i}"), format!("IMUX_O_{i}_"));
                    }
                    bels.push(bel);
                }
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .ref_int_side(int_xy, side, 0)
                    .extract_muxes(bslots::INTF_INT)
                    .extract_intfs(bslots::INTF_TESTMUX, Some(bslots::INTF_TEST_INT), true)
                    .skip_muxes(&self.iri_wires)
                    .bels(bels);
                if delay {
                    xn = xn.extract_delay();
                }
                xn.extract();
                break;
            }
        }
    }

    fn get_bli_bels(&mut self, tkn: &str, bli_cell: usize) -> Vec<ExtrBelInfo<'rd>> {
        let mut bels = vec![];

        for i in 0..3 {
            let abc = ['A', 'B', 'C'][i];

            let slot = &format!("BLI_{abc}_GRP0");
            let pin = &format!("GRP0_{abc}_BLI_LOGIC_OUTS_CTRL_0_");
            self.builder.extra_name_pin_xy(
                tkn,
                slot,
                0,
                0,
                pin,
                [
                    wires::BLI_A_GRP0_OUT_CTRL,
                    wires::BLI_B_GRP0_OUT_CTRL,
                    wires::BLI_C_GRP0_OUT_CTRL,
                ][i]
                    .cell(bli_cell),
                true,
            );
            let mut bel = self
                .builder
                .bel_xy(bslots::BLI_GRP0[i], slot, 0, 0)
                .pin_rename(pin, "OUT_CTRL_I_0_")
                .pin_rename(format!("INT_LOGIC_OUTS_CTRL_{abc}_0_"), "OUT_CTRL_O_0_");
            for j in 0..16 {
                let pin = &format!("GRP0_{abc}_BLI_IMUX_{j}_");
                self.builder.extra_name_pin_xy(
                    tkn,
                    slot,
                    0,
                    0,
                    pin,
                    [
                        wires::BLI_A_GRP0_IMUX[j],
                        wires::BLI_B_GRP0_IMUX[j],
                        wires::BLI_C_GRP0_IMUX[j],
                    ][i]
                        .cell(bli_cell),
                    true,
                );
                bel = bel
                    .pin_rename(format!("INT_IMUX_{abc}_{j}_"), format!("IMUX_I_{j}_"))
                    .pin_rename(pin, format!("IMUX_O_{j}_"));
            }
            bels.push(bel);

            let slot = &format!("BLI_{abc}_GRP1");
            let mut bel = self.builder.bel_xy(bslots::BLI_GRP1[i], slot, 0, 0);
            for j in 0..32 {
                let pin = &format!("GRP1_{abc}_BLI_LOGIC_OUTS_{j}_");
                self.builder.extra_name_pin_xy(
                    tkn,
                    slot,
                    0,
                    0,
                    pin,
                    [
                        wires::BLI_A_GRP1_OUT[j],
                        wires::BLI_B_GRP1_OUT[j],
                        wires::BLI_C_GRP1_OUT[j],
                    ][i]
                        .cell(bli_cell),
                    true,
                );
                bel = bel
                    .pin_rename(format!("INT_LOGIC_OUTS_{abc}_{j}_"), format!("OUT_O_{j}_"))
                    .pin_rename(pin, format!("OUT_I_{j}_"));
            }
            for j in 0..2 {
                let jj = 1 + j;
                let pin = &format!("GRP1_{abc}_BLI_LOGIC_OUTS_CTRL_{j}_");
                self.builder.extra_name_pin_xy(
                    tkn,
                    slot,
                    0,
                    0,
                    pin,
                    [
                        wires::BLI_A_GRP1_OUT_CTRL[j],
                        wires::BLI_B_GRP1_OUT_CTRL[j],
                        wires::BLI_C_GRP1_OUT_CTRL[j],
                    ][i]
                        .cell(bli_cell),
                    true,
                );
                bel = bel
                    .pin_rename(
                        format!("INT_LOGIC_OUTS_CTRL_{abc}_{jj}_"),
                        format!("OUT_CTRL_O_{j}_"),
                    )
                    .pin_rename(pin, format!("OUT_CTRL_I_{j}_"));
            }
            bels.push(bel);

            let slot = &format!("BLI_{abc}_GRP2");
            let mut bel = self.builder.bel_xy(bslots::BLI_GRP2[i], slot, 0, 0);
            for j in 0..17 {
                let jj = 16 + j;
                let pin = &format!("GRP2_{abc}_BLI_IMUX_{j}_");
                self.builder.extra_name_pin_xy(
                    tkn,
                    slot,
                    0,
                    0,
                    pin,
                    [
                        wires::BLI_A_GRP2_IMUX[j],
                        wires::BLI_B_GRP2_IMUX[j],
                        wires::BLI_C_GRP2_IMUX[j],
                    ][i]
                        .cell(bli_cell),
                    true,
                );
                bel = bel
                    .pin_rename(format!("INT_IMUX_{abc}_{jj}_"), format!("IMUX_I_{j}_"))
                    .pin_rename(pin, format!("IMUX_O_{j}_"));
            }
            bels.push(bel);
        }
        for (grp, bslot, base, wo) in [
            (4, bslots::BLI_GRP4, 0, wires::BLI_D_GRP4_OUT.as_slice()),
            (5, bslots::BLI_GRP5, 4, wires::BLI_D_GRP5_OUT.as_slice()),
            (6, bslots::BLI_GRP6, 8, wires::BLI_D_GRP6_OUT.as_slice()),
            (7, bslots::BLI_GRP7, 12, wires::BLI_D_GRP7_OUT.as_slice()),
        ] {
            let slot = &format!("BLI_D_GRP{grp}");
            let mut bel = self.builder.bel_xy(bslot, slot, 0, 0).manual();
            for i in 0..wo.len() {
                let ii = base * 4 + i;
                let oi = base + i;
                let ipin = &format!("GRP{grp}_D_BLI_IMUX_{i}_");
                let opin = &format!("GRP{grp}_D_BLI_LOGIC_OUTS_{i}_");
                self.builder
                    .extra_name_pin_xy(tkn, slot, 0, 0, opin, wo[i].cell(bli_cell), true);
                bel = bel
                    .pin_rename(format!("INT_LOGIC_OUTS_D_{oi}_"), format!("OUT_O_{i}_"))
                    .pin_rename(opin, format!("OUT_I_{i}_"))
                    .pin_rename(format!("INT_IMUX_D_{ii}_"), format!("IMUX_I_{i}_"))
                    .pin_rename(ipin, format!("IMUX_O_{i}_"));
            }
            bels.push(bel);
        }
        bels.push(self.builder.bel_xy(bslots::BLI_TMR, "BLI_TMR", 0, 0));
        let mut bel = self.builder.bel_virtual(bslots::INT).manual().raw_tile(1);
        for abc in ['A', 'B', 'C'] {
            for i in 0..3 {
                let wn = &format!("BLI_GRP3_{abc}_BLI_CTRL{i}");
                bel = bel.extra_int_in(wn, &[wn]);
            }
        }
        for (grp, n) in [(4, 12), (5, 12), (6, 12), (7, 11)] {
            for i in 0..n {
                let wn = &format!("BLI_GRP{grp}_D_BLI_IMUX_BYP{i}");
                bel = bel.extra_int_in(wn, &[wn]);
            }
        }
        bels.push(bel);
        bels
    }

    fn walk_to_bli(&self, mut xy: Coord) -> Coord {
        loop {
            let tile = &self.rd.tiles[&xy];
            if self.rd.tile_kinds.key(tile.kind).starts_with("BLI") {
                return xy;
            }
            xy = self.builder.delta(xy, -1, 0);
        }
    }

    fn finalize_bli(
        &mut self,
        tcid: TileClassId,
        naming: &str,
        bli_cell: usize,
        mut xt: XTileResult,
    ) {
        let (bel, _) = xt.bels.pop().unwrap();

        let pips = self
            .builder
            .pips
            .entry((tcid, bslots::BEL_INT))
            .or_default();
        for (abc, w) in [
            ('A', wires::BLI_A_GRP3_IMUX_CTRL),
            ('B', wires::BLI_B_GRP3_IMUX_CTRL),
            ('C', wires::BLI_C_GRP3_IMUX_CTRL),
        ] {
            for i in 0..3 {
                let wo = w[i].cell(bli_cell);
                let wn = &format!("BLI_GRP3_{abc}_BLI_CTRL{i}");
                let wi = bel.pins[wn].wires.iter().copied().next().unwrap();
                pips.pips.insert((wo, wi.pos()), PipMode::PermaBuf);
            }
        }
        for (grp, w) in [
            (4, wires::BLI_D_GRP4_IMUX_BYP.as_slice()),
            (5, wires::BLI_D_GRP5_IMUX_BYP.as_slice()),
            (6, wires::BLI_D_GRP6_IMUX_BYP.as_slice()),
            (7, wires::BLI_D_GRP7_IMUX_BYP.as_slice()),
        ] {
            for i in 0..w.len() {
                let wo = w[i].cell(bli_cell);
                let wn = &format!("BLI_GRP{grp}_D_BLI_IMUX_BYP{i}");
                let wi = bel.pins[wn].wires.iter().copied().next().unwrap();
                pips.pips.insert((wo, wi.pos()), PipMode::PermaBuf);
            }
        }

        for ((bslot, wi), (mut bel, bn)) in [
            (bslots::BLI_GRP4, wires::BLI_D_GRP4_IMUX.as_slice()),
            (bslots::BLI_GRP5, wires::BLI_D_GRP5_IMUX.as_slice()),
            (bslots::BLI_GRP6, wires::BLI_D_GRP6_IMUX.as_slice()),
            (bslots::BLI_GRP7, wires::BLI_D_GRP7_IMUX.as_slice()),
        ]
        .into_iter()
        .zip(xt.bels)
        {
            let pips = self
                .builder
                .pips
                .entry((tcid, bslots::BEL_INT))
                .or_default();
            for (i, wi) in wi.iter().copied().enumerate() {
                let wi = wi.cell(bli_cell);
                let pn = &format!("IMUX_O_{i}_");
                let pin = bel.pins.get_mut(pn).unwrap();
                assert_eq!(pin.wires.len(), 1);
                let wo = pin.wires.pop_last().unwrap();
                pin.wires.insert(wi);
                pips.pips.insert((wo, wi.pos()), PipMode::PermaBuf);
            }
            self.builder
                .insert_tcls_bel(tcid, bslot, BelInfo::Legacy(bel));
            self.builder.insert_bel_naming(naming, bslot, bn);
        }
    }

    pub fn fill_tiles_bli_cle(&mut self) {
        let cle_bc = self.builder.ndb.get_tile_class_naming("CLE_BC");
        for &(side, tkn, tcid_intf, naming_intf, tcid_bli, naming_bli, is_top) in BLI_CLE_INTF_KINDS
        {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, (!side).into(), false).unwrap();
                let cle_xy = self
                    .builder
                    .delta(xy, if side == DirH::E { 1 } else { -1 }, 0);
                for idx in 0..4 {
                    let dy = if is_top { idx } else { 3 - idx };
                    let cur_int_xy = self.builder.delta(int_xy, 0, dy as i32);
                    let cur_cle_xy = self.builder.delta(cle_xy, 0, dy as i32);
                    let mut bels = vec![];
                    for j in 0..4 {
                        let mut bel =
                            self.builder
                                .bel_xy(bslots::IRI[j], "IRI_QUAD", 0, idx * 4 + j);
                        for i in 0..4 {
                            bel = bel
                                .pin_rename(format!("CE{i}"), format!("CE_{i}_"))
                                .pin_rename(format!("CE{i}_O"), format!("CE_O_{i}_"));
                        }
                        for i in 0..24 {
                            bel = bel
                                .pin_rename(format!("IMUX_IN{i}"), format!("IMUX_{i}_"))
                                .pin_rename(format!("IMUX_O{i}"), format!("IMUX_O_{i}_"));
                        }
                        bels.push(bel);
                    }
                    self.builder
                        .xtile_id(tcid_intf[idx], naming_intf[idx], xy)
                        .ref_int_side(cur_int_xy, side, 0)
                        .ref_xlat(
                            cur_cle_xy,
                            if side == DirH::E {
                                &[Some(0), None]
                            } else {
                                &[None, Some(0)]
                            },
                            cle_bc,
                        )
                        .extract_intfs(bslots::INTF_TESTMUX, Some(bslots::INTF_TEST_INT), true)
                        .skip_muxes(&self.iri_wires)
                        .bels(bels)
                        .extract();
                }

                let bli_xy = self.builder.delta(xy, 0, if is_top { 5 } else { -2 });
                let bli_xy = self.walk_to_bli(bli_xy);
                let bli_cell = if is_top { 3 } else { 0 };
                let bels = self.get_bli_bels(tkn, bli_cell);

                let mut xt = self
                    .builder
                    .xtile_id(tcid_bli, naming_bli, xy)
                    .raw_tile_single(bli_xy, bli_cell);
                for i in 0..4 {
                    let cur_int_xy = xt.builder.delta(int_xy, 0, i as i32);
                    let cur_cle_xy = xt.builder.delta(cle_xy, 0, i as i32);
                    xt = xt.ref_int_side(cur_int_xy, side, i).ref_xlat(
                        cur_cle_xy,
                        &if side == DirH::E {
                            [Some(i), None]
                        } else {
                            [None, Some(i)]
                        },
                        cle_bc,
                    );
                }
                let xt = xt.bels(bels).extract();
                self.finalize_bli(tcid_bli, naming_bli, bli_cell, xt);
            }
        }
    }

    pub fn fill_tiles_bli(&mut self) {
        for (h, v, tkn) in [
            (DirH::W, DirV::N, "BLI_DSP_LOCF_TL_TILE"),
            (DirH::E, DirV::N, "BLI_DSP_LOCF_TR_TILE"),
            (DirH::W, DirV::N, "BLI_DSP_ROCF_TL_TILE"),
            (DirH::E, DirV::N, "BLI_DSP_ROCF_TR_TILE"),
            (DirH::W, DirV::S, "BLI_DSP_ROCF_BL_TILE"),
            (DirH::E, DirV::S, "BLI_DSP_ROCF_BR_TILE"),
            (DirH::E, DirV::N, "BLI_BRAM_LOCF_TR_TILE"),
            (DirH::W, DirV::N, "BLI_BRAM_ROCF_TL_TILE"),
            (DirH::E, DirV::N, "BLI_BRAM_ROCF_TR_TILE"),
            (DirH::W, DirV::S, "BLI_BRAM_ROCF_BL_TILE"),
            (DirH::E, DirV::S, "BLI_BRAM_ROCF_BR_TILE"),
        ] {
            let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
                continue;
            };
            let intf = match h {
                DirH::E => "INTF_E",
                DirH::W => "INTF_W",
            };
            let intf = self.builder.ndb.get_tile_class_naming(intf);
            let intf_xy = if h == DirH::E {
                self.builder.delta(xy, -1, 0)
            } else {
                self.builder.delta(xy, 1, 0)
            };

            let (tcid, naming) = match v {
                DirV::S => (tcls::BLI_S, "BLI_S"),
                DirV::N => (tcls::BLI_N, "BLI_N"),
            };

            let bli_xy = self.builder.delta(xy, 0, if v == DirV::N { 5 } else { -2 });
            let bli_xy = self.walk_to_bli(bli_xy);
            let bli_cell = if v == DirV::N { 3 } else { 0 };
            let bels = self.get_bli_bels(tkn, bli_cell);

            let mut xt = self
                .builder
                .xtile_id(tcid, naming, xy)
                .raw_tile_single(bli_xy, bli_cell);
            for idx in 0..4 {
                let cur_intf_xy = xt.builder.delta(intf_xy, 0, idx as i32);
                xt = xt.ref_single(cur_intf_xy, idx, intf)
            }
            let xt = xt.bels(bels).extract();
            self.finalize_bli(tcid, naming, bli_cell, xt);
        }
    }

    pub fn fill_tiles_bli_mux(&mut self) {
        for (tcid, naming, tkn) in [
            (tcls::BLI_MUX, "BLI_MUX", "BLI_LS_CORE"),
            (tcls::BLI_MUX, "BLI_MUX", "BLI_LS_CORE_MY"),
            (tcls::BLI_MUX_NOC, "BLI_MUX_NOC", "BLI_LS_MUX_CORE"),
            (tcls::BLI_MUX_NOC, "BLI_MUX_NOC", "BLI_LS_MUX_CORE_MY"),
            (tcls::BLI_MUX, "BLI_MUX", "BLI_LS_CORE_MX"),
            (tcls::BLI_MUX, "BLI_MUX", "BLI_LS_CORE_R180"),
        ] {
            self.builder.int_type_id(tcid, bslots::BLI_MUX, tkn, naming);
        }
    }
}
