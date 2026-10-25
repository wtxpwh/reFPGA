use std::collections::{BTreeMap, btree_map};

use prjcombine_interconnect::{
    db::{CellSlotId, ConnectorWire, IntDb, TileWireCoord, WireSlotId, WireSlotIdExt},
    dir::{Dir, DirV},
};
use prjcombine_re_xilinx_rawdump::{Coord, Part};

use prjcombine_entity::{EntityId, EntityPartVec, id::EntityStaticRange};
use prjcombine_re_xilinx_naming::db::NamingDb;
use prjcombine_re_xilinx_naming_ultrascale::DeviceNaming;
use prjcombine_re_xilinx_rd2db_interconnect::{
    IntBuilder, PipMode,
    tile::{XTileInfo, XTileRef},
};
use prjcombine_types::enumerable::EnumMap;
use prjcombine_ultrascale::defs::{
    self, bslots, ccls, tcls, ultrascale::wires, wiredata::ultrascale as wiredata,
};

trait IntBuilderExt {
    fn wire_names_pair(&mut self, w: WireSlotId, raw_names: &[impl AsRef<str>; 2]) -> WireSlotId;
}

impl IntBuilderExt for IntBuilder<'_> {
    fn wire_names_pair(&mut self, w: WireSlotId, raw_names: &[impl AsRef<str>; 2]) -> WireSlotId {
        for (sub, name) in raw_names.iter().enumerate() {
            self.extra_name_cell(name.as_ref(), sub, w);
        }
        w
    }
}

trait XTileInfoExt {
    fn ref_int_side(self, xy: Coord, side: Dir, slot: usize) -> Self;
}

impl XTileInfoExt for XTileInfo<'_, '_> {
    fn ref_int_side(mut self, xy: Coord, side: Dir, slot: usize) -> Self {
        self.refs.push(XTileRef {
            xy,
            naming: None,
            tile_map: [(
                CellSlotId::from_idx(match side {
                    Dir::W => 0,
                    Dir::E => 1,
                    _ => unreachable!(),
                }),
                CellSlotId::from_idx(slot),
            )]
            .into_iter()
            .collect(),
        });
        self
    }
}

struct IntMaker<'a> {
    builder: IntBuilder<'a>,
    // how many mental illnesses do you think I could be diagnosed with just from this repo?
    sng_fixup_map: BTreeMap<TileWireCoord, TileWireCoord>,
    dev_naming: &'a DeviceNaming,
}

impl IntMaker<'_> {
    fn fill_wireset_ql<const N: usize>(
        &mut self,
        w: &[EntityStaticRange<WireSlotId, N>],
        dir: Dir,
        fts: Option<WireSlotId>,
        ftn: Option<WireSlotId>,
    ) {
        let l = w.len() - 1;
        let ftd = !dir;
        let ll = if matches!(dir, Dir::E | Dir::W) {
            l * 2
        } else {
            l
        };
        for i in 0..N {
            self.builder
                .wire_names(w[0][i], &[format!("{dir}{dir}{ll}_BEG{i}")]);
            for j in 1..l {
                let nn = (b'A' + (j as u8 - 1)) as char;
                let wname = format!("{dir}{dir}{ll}_{nn}_FT{ftd}{i}");
                self.builder.wire_names(w[j][i], &[wname]);
            }
            let wname = format!("{dir}{dir}{ll}_END{i}");
            self.builder.wire_names(w[l][i], &[wname]);
            if i == 0
                && let Some(fts) = fts
            {
                self.builder
                    .wire_names(fts, &[format!("{dir}{dir}{ll}_BLS_{i}_FTN")]);
            }
            if i == (N - 1)
                && let Some(ftn) = ftn
            {
                self.builder
                    .wire_names(ftn, &[format!("{dir}{dir}{ll}_BLN_{i}_FTS")]);
            }
        }
    }

    fn fill_wires_ql(&mut self) {
        self.fill_wireset_ql(
            &wiredata::LX4_E,
            Dir::E,
            Some(wires::LX4_E2_S0),
            Some(wires::LX4_E2_N15),
        );
        self.fill_wireset_ql(&wiredata::LX4_W, Dir::W, None, None);
        self.fill_wireset_ql(&wiredata::LX4_N, Dir::N, None, None);
        self.fill_wireset_ql(&wiredata::LX5_N, Dir::N, None, Some(wires::LX5_N5_N7));
        self.fill_wireset_ql(&wiredata::LX4_S, Dir::S, None, None);
        self.fill_wireset_ql(&wiredata::LX5_S, Dir::S, None, None);
        self.fill_wireset_ql(&wiredata::LX12_E, Dir::E, Some(wires::LX12_E6_S0), None);
        self.fill_wireset_ql(&wiredata::LX12_W, Dir::W, None, Some(wires::LX12_W6_N7));
        self.fill_wireset_ql(&wiredata::LX12_N, Dir::N, None, None);
        self.fill_wireset_ql(&wiredata::LX16_N, Dir::N, None, Some(wires::LX16_N16_N3));
        self.fill_wireset_ql(&wiredata::LX12_S, Dir::S, Some(wires::LX12_S12_S0), None);
        self.fill_wireset_ql(&wiredata::LX16_S, Dir::S, None, None);
    }

    fn fill_wires_sdnode(&mut self) {
        for i in 0..64 {
            let ii = i % 16;
            let iq = i / 16;
            let q = ["NE", "NW", "SE", "SW"][iq];
            match (iq, ii) {
                (1 | 3, 0) => {
                    self.builder.wire_names_pair(
                        wires::SDNODE[i],
                        &[format!("SDND{q}_W_{ii}_FTS"), format!("SDND{q}_E_{ii}_FTS")],
                    );
                    self.builder.wire_names_pair(
                        wires::SDNODE_S[i],
                        &[
                            format!("SDND{q}_W_BLS_{ii}_FTN"),
                            format!("SDND{q}_E_BLS_{ii}_FTN"),
                        ],
                    );
                }
                (1 | 3, 15) => {
                    self.builder.wire_names_pair(
                        wires::SDNODE[i],
                        &[format!("SDND{q}_W_{ii}_FTN"), format!("SDND{q}_E_{ii}_FTN")],
                    );
                    self.builder.wire_names_pair(
                        wires::SDNODE_N[i],
                        &[
                            format!("SDND{q}_W_BLN_{ii}_FTS"),
                            format!("SDND{q}_E_BLN_{ii}_FTS"),
                        ],
                    );
                }
                _ => {
                    let xlat = [0, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1, 2, 3, 4, 5, 6];
                    self.builder.wire_names_pair(
                        wires::SDNODE[i],
                        &[
                            format!(
                                "INT_NODE_SINGLE_DOUBLE_{n}_INT_OUT",
                                n = iq * 32 + 16 + xlat[ii]
                            ),
                            format!("INT_NODE_SINGLE_DOUBLE_{n}_INT_OUT", n = iq * 32 + xlat[ii]),
                        ],
                    );
                }
            }
        }
    }

    fn fill_wires_sng(&mut self) {
        for i in 0..8 {
            self.builder.wire_names_pair(
                wires::X1_E0[i],
                &[
                    if i == 0 {
                        format!("EE1_W_{i}_FTS")
                    } else {
                        format!("INT_INT_SINGLE_{n}_INT_OUT", n = i + 8)
                    },
                    format!("EE1_E_BEG{i}"),
                ],
            );
            self.builder
                .extra_name_cell(format!("EE1_E_END{i}"), 0, wires::X1_E1[i]);
            if i == 0 {
                self.builder.wire_names_pair(
                    wires::X1_E1_S0,
                    &[format!("EE1_E_BLS_{i}_FTN"), format!("EE1_W_BLS_{i}_FTN")],
                );
            }
            self.sng_fixup_map
                .insert(wires::X1_E0[i].cell(0), wires::X1_E1[i].cell(1));
        }
        for i in 0..8 {
            self.builder.wire_names_pair(
                wires::X1_W0[i],
                &[
                    format!("WW1_W_BEG{i}"),
                    format!("INT_INT_SINGLE_{n}_INT_OUT", n = i + 48),
                ],
            );
            self.builder
                .extra_name_cell(format!("WW1_W_END{i}"), 1, wires::X1_W1[i]);
            self.sng_fixup_map
                .insert(wires::X1_W0[i].cell(1), wires::X1_W1[i].cell(0));
        }
        for (dir, w0, w1) in [
            (Dir::N, wires::X1_N0, wires::X1_N1),
            (Dir::S, wires::X1_S0, wires::X1_S1),
        ] {
            for i in 0..8 {
                self.builder.wire_names_pair(
                    w0[i],
                    &[
                        format!("{dir}{dir}1_W_BEG{i}"),
                        format!("{dir}{dir}1_E_BEG{i}"),
                    ],
                );
                self.builder.wire_names_pair(
                    w1[i],
                    &[
                        format!("{dir}{dir}1_W_END{i}"),
                        format!("{dir}{dir}1_E_END{i}"),
                    ],
                );
                if i == 0 && dir == Dir::S {
                    self.builder.wire_names_pair(
                        wires::X1_S1_S0,
                        &[
                            format!("{dir}{dir}1_W_BLS_{i}_FTN"),
                            format!("{dir}{dir}1_E_BLS_{i}_FTN"),
                        ],
                    );
                }
            }
        }
    }

    fn fill_wires_dbl(&mut self) {
        for (dir, w0, w2) in [
            (Dir::E, wires::X2_E0, wires::X2_E2),
            (Dir::W, wires::X2_W0, wires::X2_W2),
        ] {
            for i in 0..8 {
                self.builder.wire_names_pair(
                    w0[i],
                    &[
                        format!("{dir}{dir}2_W_BEG{i}"),
                        format!("{dir}{dir}2_E_BEG{i}"),
                    ],
                );
                self.builder.wire_names_pair(
                    w2[i],
                    &[
                        format!("{dir}{dir}2_W_END{i}"),
                        format!("{dir}{dir}2_E_END{i}"),
                    ],
                );
                if i == 7 && dir == Dir::E {
                    self.builder.wire_names_pair(
                        wires::X2_E2_N7,
                        &[
                            format!("{dir}{dir}2_W_BLN_{i}_FTS"),
                            format!("{dir}{dir}2_E_BLN_{i}_FTS"),
                        ],
                    );
                }
            }
        }
        for (dir, w0, w1, w2) in [
            (Dir::N, wires::X2_N0, wires::X2_N1, wires::X2_N2),
            (Dir::S, wires::X2_S0, wires::X2_S1, wires::X2_S2),
        ] {
            let ftd = !dir;
            for i in 0..8 {
                self.builder.wire_names_pair(
                    w0[i],
                    &[
                        format!("{dir}{dir}2_W_BEG{i}"),
                        format!("{dir}{dir}2_E_BEG{i}"),
                    ],
                );
                self.builder.wire_names_pair(
                    w1[i],
                    &[
                        format!("{dir}{dir}2_W_A_FT{ftd}{i}"),
                        format!("{dir}{dir}2_E_A_FT{ftd}{i}"),
                    ],
                );
                self.builder.wire_names_pair(
                    w2[i],
                    &[
                        format!("{dir}{dir}2_W_END{i}"),
                        format!("{dir}{dir}2_E_END{i}"),
                    ],
                );
                if i == 7 && dir == Dir::N {
                    self.builder.wire_names_pair(
                        wires::X2_N2_N7,
                        &[
                            format!("{dir}{dir}2_W_BLN_{i}_FTS"),
                            format!("{dir}{dir}2_E_BLN_{i}_FTS"),
                        ],
                    );
                }
            }
        }
    }

    fn fill_wires_qlnode(&mut self) {
        for i in 0..64 {
            let ii = i % 16;
            let iq = i / 16;
            let q = ["NE", "NW", "SE", "SW"][iq];
            let xlat = [0, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1, 2, 3, 4, 5, 6];
            self.builder.wire_names_pair(
                wires::QLNODE[i],
                &[
                    format!(
                        "INT_NODE_QUAD_LONG_{n}_INT_OUT",
                        n = iq * 32 + 16 + xlat[ii]
                    ),
                    format!("INT_NODE_QUAD_LONG_{n}_INT_OUT", n = iq * 32 + xlat[ii]),
                ],
            );
            match (q, ii) {
                ("NW", 0) | ("SW", 0) | ("NW", 1) => {
                    self.builder.extra_name_cell(
                        format!("QLND{q}_W_{ii}_FTS"),
                        0,
                        wires::QLNODE[i],
                    );
                    self.builder.extra_name_cell(
                        format!("QLND{q}_E_{ii}_FTS"),
                        1,
                        wires::QLNODE[i],
                    );
                    self.builder.wire_names_pair(
                        wires::QLNODE_S[i],
                        &[
                            format!("QLND{q}_W_BLS_{ii}_FTN"),
                            format!("QLND{q}_E_BLS_{ii}_FTN"),
                        ],
                    );
                }
                ("NW", 15) | ("SW", 15) | ("SE", 15) => {
                    self.builder.extra_name_cell(
                        format!("QLND{q}_W_{ii}_FTN"),
                        0,
                        wires::QLNODE[i],
                    );
                    self.builder.extra_name_cell(
                        format!("QLND{q}_E_{ii}_FTN"),
                        1,
                        wires::QLNODE[i],
                    );
                    self.builder.wire_names_pair(
                        wires::QLNODE_N[i],
                        &[
                            format!("QLND{q}_W_BLN_{ii}_FTS"),
                            format!("QLND{q}_E_BLN_{ii}_FTS"),
                        ],
                    );
                }
                _ => (),
            }
        }
    }

    fn fill_wires_inode(&mut self) {
        for i in 0..64 {
            let iq = i / 32;
            let ii = i % 32;
            let q = ["1", "2"][iq];
            match ii {
                1 | 3 => {
                    self.builder.wire_names_pair(
                        wires::INODE[i],
                        &[
                            format!("INODE_{q}_W_{ii}_FTS"),
                            format!("INODE_{q}_E_{ii}_FTS"),
                        ],
                    );
                    self.builder.wire_names_pair(
                        wires::INODE_S[i],
                        &[
                            format!("INODE_{q}_W_BLS_{ii}_FTN"),
                            format!("INODE_{q}_E_BLS_{ii}_FTN"),
                        ],
                    );
                }
                28 | 30 => {
                    self.builder.wire_names_pair(
                        wires::INODE[i],
                        &[
                            format!("INODE_{q}_W_{ii}_FTN"),
                            format!("INODE_{q}_E_{ii}_FTN"),
                        ],
                    );
                    self.builder.wire_names_pair(
                        wires::INODE_N[i],
                        &[
                            format!("INODE_{q}_W_BLN_{ii}_FTS"),
                            format!("INODE_{q}_E_BLN_{ii}_FTS"),
                        ],
                    );
                }
                _ => {
                    let xlat = [
                        0, 11, 22, 25, 26, 27, 28, 29, 30, 31, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12,
                        13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24,
                    ];
                    self.builder.extra_name_tile_cell(
                        "INT",
                        format!("INT_NODE_IMUX_{n}_INT_OUT", n = iq * 64 + 32 + xlat[ii]),
                        0,
                        wires::INODE[i],
                    );
                    self.builder.extra_name_tile_cell(
                        "INT",
                        format!("INT_NODE_IMUX_{n}_INT_OUT", n = iq * 64 + xlat[ii]),
                        1,
                        wires::INODE[i],
                    );
                }
            }
        }
    }

    fn fill_wires_imux(&mut self) {
        for i in 0..10 {
            self.builder.wire_names_pair(
                wires::IMUX_CTRL[i],
                &[format!("CTRL_W_B{i}"), format!("CTRL_E_B{i}")],
            );
        }

        for i in 0..16 {
            match i {
                1 | 3 | 5 | 7 | 11 => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BOUNCE_W_{i}_FTS"), format!("BOUNCE_E_{i}_FTS")],
                    );
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP_S[i],
                        &[
                            format!("BOUNCE_W_BLS_{i}_FTN"),
                            format!("BOUNCE_E_BLS_{i}_FTN"),
                        ],
                    );
                }
                8 | 10 | 12 | 14 => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BOUNCE_W_{i}_FTN"), format!("BOUNCE_E_{i}_FTN")],
                    );
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP_N[i],
                        &[
                            format!("BOUNCE_W_BLN_{i}_FTS"),
                            format!("BOUNCE_E_BLN_{i}_FTS"),
                        ],
                    );
                }
                _ => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BYPASS_W{i}"), format!("BYPASS_E{i}")],
                    );
                }
            }
        }
        for i in 0..48 {
            self.builder.wire_names_pair(
                wires::IMUX_IMUX[i],
                &[format!("IMUX_W{i}"), format!("IMUX_E{i}")],
            );
            self.builder
                .mark_delay(wires::IMUX_IMUX[i], wires::IMUX_IMUX_DELAY[i]);
        }
    }

    fn fill_wires_rclk(&mut self) {
        #[allow(clippy::type_complexity)]
        const BUFCE_XLAT: [Option<((usize, usize), (usize, usize))>; 24] = [
            None,
            None,
            None,
            None,
            Some(((1, 8), (1, 4))),
            Some(((1, 13), (1, 6))),
            None,
            None,
            None,
            None,
            Some(((0, 9), (0, 7))),
            Some(((1, 11), (1, 2))),
            Some(((0, 0), (0, 1))),
            Some(((0, 4), (0, 5))),
            Some(((0, 6), (0, 8))),
            Some(((0, 12), (0, 13))),
            Some(((1, 3), (1, 10))),
            Some(((1, 15), (1, 14))),
            Some(((0, 2), (0, 3))),
            Some(((0, 10), (0, 11))),
            Some(((0, 14), (0, 15))),
            Some(((1, 1), (1, 0))),
            Some(((1, 5), (1, 9))),
            Some(((1, 7), (1, 12))),
        ];
        for i in 0..24 {
            if let Some(((wa, wb), (ea, eb))) = BUFCE_XLAT[i] {
                self.builder.wire_names_pair(
                    wires::IMUX_RCLK[i],
                    &[
                        format!("CLK_BUFCE_LEAF_X16_{wa}_CE_INT{wb}"),
                        format!("CLK_BUFCE_LEAF_X16_{ea}_CE_INT{eb}",),
                    ],
                );
            } else {
                self.builder.wire_names_pair(
                    wires::IMUX_RCLK[i],
                    &[
                        format!("INT_RCLK_TO_CLK_LEFT_{a}_{b}", a = i / 6, b = i % 6),
                        format!("INT_RCLK_TO_CLK_RIGHT_{a}_{b}", a = i / 6, b = i % 6),
                    ],
                );
            }
        }
        for i in 0..24 {
            for sub in 0..2 {
                for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
                    let ii = (i % 12) + (1 - sub) * 12 + (i / 12) * 24;
                    let name = format!("INT_NODE_IMUX_{ii}_INT_OUT");
                    self.builder
                        .extra_name_tile_cell(tkn, name, sub, wires::INODE_RCLK[i]);
                }
            }
        }
    }

    fn fill_wires(&mut self) {
        // common wires

        self.builder.wire_names(wires::TIE_0, &["GND_WIRE"]);
        self.builder.wire_names(wires::TIE_1, &["VCC_WIRE"]);

        for i in 0..16 {
            self.builder
                .wire_names(wires::GCLK[i], &[format!("GCLK_B_0_{i}")]);
            for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFCE_LEAF_X16_0_CLK_OUT{ii}", ii = i / 2 + (i % 2) * 8),
                    2,
                    wires::GCLK[i],
                );
            }
        }

        for i in 0..16 {
            for j in 0..2 {
                self.builder.wire_names(
                    wires::GNODE[i * 2 + j],
                    &[format!("INT_NODE_GLOBAL_{i}_OUT{j}")],
                );
            }
        }

        self.fill_wires_ql();

        // wires belonging to interconnect left/right half-tiles

        for i in 0..32 {
            self.builder
                .wire_names(wires::OUT[i], &[format!("LOGIC_OUTS_W{i}")]);
            self.builder
                .mark_test_mux_in(wires::OUT_BEL[i], wires::OUT[i]);
            self.builder
                .mark_test_mux_in_test(wires::OUT_TEST[i], wires::OUT[i]);
            self.builder
                .extra_name_cell(format!("LOGIC_OUTS_E{i}"), 1, wires::OUT[i]);
        }

        for i in 0..4 {
            self.builder
                .wire_names(wires::TEST[i], &[format!("BLOCK_OUTS{i}")]);
        }

        self.fill_wires_sdnode();
        self.fill_wires_sng();
        self.fill_wires_dbl();
        self.fill_wires_qlnode();
        self.fill_wires_inode();
        self.fill_wires_imux();
        self.fill_wires_rclk();
    }

    fn fill_tiles_int(&mut self) {
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
                if wires::INODE.contains(wt.wire)
                    || wires::SDNODE.contains(wt.wire)
                    || wires::QLNODE.contains(wt.wire)
                {
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

    fn extract_sn_term(&mut self, dir: DirV, int_xy: Coord) {
        let pass_rev = &self.builder.db[match dir {
            DirV::S => ccls::PASS_N,
            DirV::N => ccls::PASS_S,
        }];
        let naming =
            &self.builder.ndb.tile_class_namings[self.builder.ndb.get_tile_class_naming("INT")];
        let mut node2target = BTreeMap::new();
        for &ti in pass_rev.wires.values() {
            let ConnectorWire::Pass(wf) = ti else {
                unreachable!()
            };
            for cid in [0, 1] {
                let cell = CellSlotId::from_idx(cid);
                let Some(name) = naming.wires.get(&wf.cell(cid)) else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(int_xy, &name.name);
                let mut twf = (cell, wf);
                // sigh. no hope. no hope at all.
                if wf == wires::X1_E1[0] {
                    twf = (
                        CellSlotId::from_idx(cell.to_idx() ^ 1),
                        [wires::X1_E1_E0, wires::X1_E1_W0][cell.to_idx()],
                    );
                }
                if wf == wires::X2_E2[7] {
                    twf = (
                        CellSlotId::from_idx(cell.to_idx() ^ 1),
                        [wires::X2_E2_E7, wires::X2_E2_W7][cell.to_idx()],
                    );
                }
                assert!(node2target.insert(node, twf).is_none());
            }
        }
        for cid in [0, 1] {
            let pass = &self.builder.db[match dir {
                DirV::S => ccls::PASS_S,
                DirV::N => ccls::PASS_N,
            }];
            let naming =
                &self.builder.ndb.tile_class_namings[self.builder.ndb.get_tile_class_naming("INT")];
            let mut wires = EntityPartVec::new();
            for wt in pass.wires.ids() {
                let cell = CellSlotId::from_idx(cid);
                let Some(name) = naming.wires.get(&wt.cell(cid)) else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(int_xy, &name.name);
                if let Some(&(tf, wf)) = node2target.get(&node) {
                    assert_eq!(cell, tf);
                    wires.insert(wt, ConnectorWire::Reflect(wf));
                }
            }
            self.builder.insert_connector(
                match dir {
                    DirV::S => [ccls::TERM_S0, ccls::TERM_S1][cid],
                    DirV::N => [ccls::TERM_N0, ccls::TERM_N1][cid],
                },
                wires,
            );
        }
    }

    fn fill_terms(&mut self) {
        for &xy in self.builder.rd.tiles_by_kind_name("INT_TERM_B") {
            let int_xy = self.builder.walk_to_int(xy, Dir::N, true).unwrap();
            self.extract_sn_term(DirV::S, int_xy);
        }
        for &xy in self.builder.rd.tiles_by_kind_name("INT_TERM_T") {
            let int_xy = self.builder.walk_to_int(xy, Dir::S, true).unwrap();
            self.extract_sn_term(DirV::N, int_xy);
        }
    }

    fn fill_tiles_rclk_int(&mut self) {
        for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_FT0_{i}"),
                    wires::GCLK_HDISTR[i],
                );
            }
            for i in 0..16 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFCE_LEAF_X16_0_CLK_IN{i}"),
                    2,
                    wires::IMUX_BUFCE_LEAF[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFCE_LEAF_X16_1_CLK_IN{i}"),
                    0,
                    wires::IMUX_BUFCE_LEAF[i],
                );
            }
            for &xy in self.builder.rd.tiles_by_kind_name(tkn) {
                let mut bels = vec![];
                for i in 0..2 {
                    let slot = [bslots::BUFCE_LEAF_X16_S, bslots::BUFCE_LEAF_X16_N][i];
                    let mut bel = self.builder.bel_xy(slot, "BUFCE_LEAF_X16", 0, i);
                    for i in 0..16 {
                        bel = bel.pin_rename(format!("CE_INT{i}"), format!("CE{i}"));
                    }
                    bels.push(bel);
                }
                self.builder
                    .xtile_id(tcls::RCLK_INT, "RCLK_INT", xy)
                    .ref_int_side(xy.delta(0, 1), Dir::W, 0)
                    .ref_int_side(xy.delta(0, 1), Dir::E, 1)
                    .extract_muxes(bslots::RCLK_INT)
                    .optin_muxes(&wires::IMUX_BUFCE_LEAF[..])
                    .bels(bels)
                    .extract();
            }
        }
    }

    fn fill_tiles_intf(&mut self) {
        for (tcid, naming, dir, tkn, extract_delay) in [
            (tcls::INTF, "INTF_W", Dir::W, "INT_INTERFACE_L", false),
            (tcls::INTF, "INTF_E", Dir::E, "INT_INTERFACE_R", false),
            (
                tcls::INTF_DELAY,
                "INTF_W_IO",
                Dir::W,
                "INT_INT_INTERFACE_XIPHY_FT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_W_PCIE",
                Dir::W,
                "INT_INTERFACE_PCIE_L",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_PCIE",
                Dir::E,
                "INT_INTERFACE_PCIE_R",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_W_GT",
                Dir::W,
                "INT_INT_INTERFACE_GT_LEFT_FT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_GT",
                Dir::E,
                "INT_INTERFACE_GT_R",
                true,
            ),
        ] {
            for &xy in self.builder.rd.tiles_by_kind_name(tkn.as_ref()) {
                let int_xy = self.builder.walk_to_int(xy, !dir, false).unwrap();
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .extract_intfs(bslots::INTF_TESTMUX, Some(bslots::INTF_INT), true)
                    .ref_int_side(int_xy, dir, 0);
                if extract_delay {
                    xn = xn.extract_delay();
                }
                xn.extract();
            }
        }
    }

    fn fill_tiles_clb(&mut self) {
        for (tkn, tcid, naming, side) in [
            ("CLEL_L", tcls::CLEL, "CLEL", Dir::W),
            ("CLEL_R", tcls::CLEL, "CLEL", Dir::E),
            ("CLE_M", tcls::CLEM, "CLEM", Dir::W),
            ("CLE_M_R", tcls::CLEM, "CLEM", Dir::W),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = xy.delta(if side == Dir::W { 1 } else { -1 }, 0);
                let bel = self
                    .builder
                    .bel_xy(bslots::SLICE, "SLICE", 0, 0)
                    .pin_name_only("CIN", 1)
                    .pin_name_only("COUT", 0)
                    .pin_rename("CLK_B1", "CLK1")
                    .pin_rename("CLK_B2", "CLK2")
                    .pin_rename("SRST_B1", "SRST1")
                    .pin_rename("SRST_B2", "SRST2")
                    .pin_rename("CKEN_B1", "CKEN1")
                    .pin_rename("CKEN_B2", "CKEN2")
                    .pin_rename("CKEN_B3", "CKEN3")
                    .pin_rename("CKEN_B4", "CKEN4")
                    .pin_rename("LCLK_B", "LCLK")
                    .pin_rename("WCKEN_B", "WCKEN");
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .ref_int_side(int_xy, side, 0)
                    .bel(bel)
                    .extract();
            }
        }
    }

    fn fill_tiles_lag(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("LAGUNA_TILE")
            .iter()
            .next()
        {
            let mut bels = vec![];
            for i in 0..4 {
                let mut bel = self
                    .builder
                    .bel_xy(bslots::LAGUNA[i], "LAGUNA", i >> 1, i & 1)
                    .extra_wire("VCC", &["VCC_WIRE"]);
                for j in 0..6 {
                    bel = bel
                        .pin_rename("RX_CLK_B", "RX_CLK")
                        .pin_rename("TX_CLK_B", "TX_CLK")
                        .pin_name_only(&format!("RXQ{j}"), 0)
                        .pin_name_only(&format!("RXD{j}"), 0)
                        .pin_name_only(&format!("TXQ{j}"), 0)
                        .extra_int_out(format!("RXOUT{j}"), &[format!("RXD{ii}", ii = i * 6 + j)])
                        .extra_wire(
                            format!("TXOUT{j}"),
                            &[format!(
                                "LAG_MUX_ATOM_{ii}_TXOUT",
                                ii = match (i, j) {
                                    (0, 0) => 0,
                                    (0, 1) => 11,
                                    (0, 2) => 16,
                                    (0, 3) => 17,
                                    (0, 4) => 18,
                                    (0, 5) => 19,
                                    (1, 0) => 20,
                                    (1, 1) => 21,
                                    (1, 2) => 22,
                                    (1, 3) => 23,
                                    (1, 4) => 1,
                                    (1, 5) => 2,
                                    (2, 0) => 3,
                                    (2, 1) => 4,
                                    (2, 2) => 5,
                                    (2, 3) => 6,
                                    (2, 4) => 7,
                                    (2, 5) => 8,
                                    (3, 0) => 9,
                                    (3, 1) => 10,
                                    (3, 2) => 12,
                                    (3, 3) => 13,
                                    (3, 4) => 14,
                                    (3, 5) => 15,
                                    _ => unreachable!(),
                                }
                            )],
                        )
                        .extra_wire(format!("UBUMP{j}"), &[format!("UBUMP{ii}", ii = i * 6 + j)]);
                }
                if i == 0 {
                    bel = bel
                        .extra_wire("EXTRA_UBUMP", &["UBUMP_EXTRA"])
                        .extra_wire("EXTRA_RXD", &["LAG_IOBUF_ATOM_16_RXO"])
                        .extra_wire("EXTRA_TXOUT", &["VCC_WIRE0"]);
                }
                bels.push(bel);
            }
            self.builder
                .xtile_id(tcls::LAGUNA, "LAGUNA", xy)
                .ref_int_side(xy.delta(1, 0), Dir::W, 0)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_bram(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("BRAM").iter().next() {
            let intf = self.builder.ndb.get_tile_class_naming("INTF_W");
            let mut bel = self
                .builder
                .bel_xy(bslots::BRAM, "RAMB36", 0, 0)
                .pin_rename("CLKAL_X", "CLKAL")
                .pin_rename("CLKAU_X", "CLKAU")
                .pin_rename("CLKBL_X", "CLKBL")
                .pin_rename("CLKBU_X", "CLKBU")
                .pin_rename("ENAL_X", "ENAL")
                .pin_rename("ENAU_X", "ENAU")
                .pin_rename("ENBL_X", "ENBL")
                .pin_rename("ENBU_X", "ENBU")
                .pin_rename("REGCEAL_X", "REGCEAL")
                .pin_rename("REGCEAU_X", "REGCEAU")
                .pin_rename("REGCEBL_X", "REGCEBL")
                .pin_rename("REGCEBU_X", "REGCEBU")
                .pin_rename("REGCLKAL_X", "REGCLKAL")
                .pin_rename("REGCLKAU_X", "REGCLKAU")
                .pin_rename("REGCLKBL_X", "REGCLKBL")
                .pin_rename("REGCLKBU_X", "REGCLKBU")
                .pin_rename("RSTFIFO_X", "RSTFIFO")
                .pin_rename("RSTRAMAL_X", "RSTRAMAL")
                .pin_rename("RSTRAMAU_X", "RSTRAMAU")
                .pin_rename("RSTRAMBL_X", "RSTRAMBL")
                .pin_rename("RSTRAMBU_X", "RSTRAMBU")
                .pin_rename("RSTREGAL_X", "RSTREGAL")
                .pin_rename("RSTREGAU_X", "RSTREGAU")
                .pin_rename("RSTREGBL_X", "RSTREGBL")
                .pin_rename("RSTREGBU_X", "RSTREGBU")
                .pin_name_only("CASINSBITERR", 1)
                .pin_name_only("CASINDBITERR", 1)
                .pin_name_only("CASOUTSBITERR", 0)
                .pin_name_only("CASOUTDBITERR", 0)
                .pin_name_only("CASPRVEMPTY", 1)
                .pin_name_only("CASPRVRDEN", 1)
                .pin_name_only("CASNXTEMPTY", 1)
                .pin_name_only("CASNXTRDEN", 1)
                .pin_name_only("CASMBIST12OUT", 0)
                .pin_name_only("ENABLE_BIST", 1)
                .pin_name_only("START_RSR_NEXT", 0);
            for ab in ['A', 'B'] {
                for ul in ['U', 'L'] {
                    for i in 0..16 {
                        bel = bel.pin_name_only(&format!("CASDI{ab}{ul}{i}"), 1);
                        bel = bel.pin_name_only(&format!("CASDO{ab}{ul}{i}"), 1);
                    }
                    for i in 0..2 {
                        bel = bel.pin_name_only(&format!("CASDIP{ab}{ul}{i}"), 1);
                        bel = bel.pin_name_only(&format!("CASDOP{ab}{ul}{i}"), 1);
                    }
                }
            }
            let tile = &self.builder.rd.tiles[&xy];
            let tk = &self.builder.rd.tile_kinds[tile.kind];
            for i in 0..2 {
                bel = bel.sub_xy("RAMB18", 0, i);
                let tksite = tk
                    .sites
                    .get(&self.builder.rd.slot_xy("RAMB18", 0, i).unwrap())
                    .unwrap()
                    .1;
                for pin in tksite.pins.keys() {
                    bel = bel
                        .pin_rename(pin, format!("H{i}_{pin}"))
                        .pin_name_only(&format!("H{i}_{pin}"), 0);
                }
            }
            let mut xn = self.builder.xtile_id(tcls::BRAM, "BRAM", xy);
            for i in 0..5 {
                xn = xn
                    .ref_int_side(xy.delta(2, i as i32), Dir::W, i)
                    .ref_single(xy.delta(1, i as i32), i, intf);
            }
            xn.bel(bel).extract();
        }

        for tkn in [
            "RCLK_BRAM_L",
            "RCLK_BRAM_R",
            "RCLK_RCLK_BRAM_L_AUXCLMP_FT",
            "RCLK_RCLK_BRAM_L_BRAMCLMP_FT",
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let intf = self.builder.ndb.get_tile_class_naming("INTF_W");
                let mut bels = vec![];
                for (i, x, y) in [(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1)] {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HARD_SYNC[i], "HARD_SYNC", x, y)
                            .pin_rename("CLK_B", "CLK"),
                    );
                }
                self.builder
                    .xtile_id(tcls::HARD_SYNC, "HARD_SYNC", xy)
                    .ref_int_side(xy.delta(2, 1), Dir::W, 0)
                    .ref_single(xy.delta(1, 1), 0, intf)
                    .bels(bels)
                    .extract();
            }
        }
    }

    fn fill_tiles_dsp(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("DSP").iter().next() {
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E");
            let mut bels_dsp = vec![];
            for i in 0..2 {
                let buf_cnt = match i {
                    0 => 1,
                    1 => 0,
                    _ => unreachable!(),
                };
                let mut bel = self
                    .builder
                    .bel_xy(bslots::DSP[i], "DSP48E2", 0, i)
                    .pin_name_only("MULTSIGNIN", buf_cnt)
                    .pin_name_only("MULTSIGNOUT", 0)
                    .pin_name_only("CARRYCASCIN", buf_cnt)
                    .pin_name_only("CARRYCASCOUT", 0)
                    .pin_rename("CLK_B", "CLK")
                    .pin_rename("RSTA_B", "RSTA")
                    .pin_rename("RSTB_B", "RSTB")
                    .pin_rename("RSTC_B", "RSTC")
                    .pin_rename("RSTD_B", "RSTD")
                    .pin_rename("RSTM_B", "RSTM")
                    .pin_rename("RSTP_B", "RSTP")
                    .pin_rename("RSTINMODE_B", "RSTINMODE")
                    .pin_rename("RSTCTRL_B", "RSTCTRL")
                    .pin_rename("RSTALUMODE_B", "RSTALUMODE")
                    .pin_rename("RSTALLCARRYIN_B", "RSTALLCARRYIN")
                    .pin_rename("CEA1_B", "CEA1")
                    .pin_rename("CEA2_B", "CEA2")
                    .pin_rename("CEB1_B", "CEB1")
                    .pin_rename("CEB2_B", "CEB2")
                    .pin_rename("CEC_B", "CEC")
                    .pin_rename("CED_B", "CED")
                    .pin_rename("CEM_B", "CEM")
                    .pin_rename("CEP_B", "CEP")
                    .pin_rename("CEAD_B", "CEAD");
                for j in 0..30 {
                    bel = bel.pin_name_only(&format!("ACIN_B{j}"), buf_cnt);
                    bel = bel.pin_name_only(&format!("ACOUT_B{j}"), 0);
                }
                for j in 0..18 {
                    bel = bel.pin_name_only(&format!("BCIN_B{j}"), buf_cnt);
                    bel = bel.pin_name_only(&format!("BCOUT_B{j}"), 0);
                }
                for j in 0..48 {
                    bel = bel.pin_name_only(&format!("PCIN{j}"), buf_cnt);
                    bel = bel.pin_name_only(&format!("PCOUT{j}"), 0);
                }
                bels_dsp.push(bel);
            }
            let mut xn = self.builder.xtile_id(tcls::DSP, "DSP", xy);
            for i in 0..5 {
                xn = xn
                    .ref_int_side(xy.delta(-2, i as i32), Dir::E, i)
                    .ref_single(xy.delta(-1, i as i32), i, intf);
            }
            xn.bels(bels_dsp).extract();
        }
    }

    fn fill_tiles_hard(&mut self) {
        for (slot, tcid, naming, tkn, bk) in [
            (bslots::PCIE3, tcls::PCIE3, "PCIE3", "PCIE", "PCIE_3_1"),
            (
                bslots::CMAC,
                tcls::CMAC,
                "CMAC",
                "CMAC_CMAC_FT",
                "CMAC_SITE",
            ),
            (
                bslots::ILKN,
                tcls::ILKN,
                "ILKN",
                "ILMAC_ILMAC_FT",
                "ILKN_SITE",
            ),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_w_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let int_e_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W_PCIE");
                let mut bel = self.builder.bel_xy(slot, bk, 0, 0);
                if tcid == tcls::PCIE3 {
                    bel = bel
                        .pin_rename("CORE_CLK_B", "CORE_CLK")
                        .pin_rename("USER_CLK_B", "USER_CLK")
                        .pin_rename("DRP_CLK_B", "DRP_CLK")
                        .pin_rename("PIPE_CLK_B", "PIPE_CLK")
                        .pin_rename("MCAP_CLK_B", "MCAP_CLK")
                        .pin_rename(
                            "CORE_CLK_MI_COMPLETION_RAM_L_B",
                            "CORE_CLK_MI_COMPLETION_RAM_L",
                        )
                        .pin_rename(
                            "CORE_CLK_MI_COMPLETION_RAM_U_B",
                            "CORE_CLK_MI_COMPLETION_RAM_U",
                        )
                        .pin_rename("CORE_CLK_MI_REPLAY_RAM_B", "CORE_CLK_MI_REPLAY_RAM")
                        .pin_rename("CORE_CLK_MI_REQUEST_RAM_B", "CORE_CLK_MI_REQUEST_RAM")
                        .pin_name_only("MCAP_PERST0_B", 1)
                        .pin_name_only("MCAP_PERST1_B", 1);
                    for (prefix, num) in [
                        ("XIL_UNCONN_BOUT", 480),
                        ("XIL_UNCONN_BYP", 1920),
                        ("XIL_UNCONN_CLK_B", 951),
                        ("XIL_UNCONN_IN", 3189),
                        ("XIL_UNCONN_OUT", 861),
                    ] {
                        for i in 0..num {
                            bel = bel.pins_name_only(&[format!("{prefix}{i}")]);
                        }
                    }
                }
                if tcid == tcls::CMAC {
                    bel = bel
                        .pin_rename("DRP_CLK_B", "DRP_CLK")
                        .pin_rename("RX_CLK_B", "RX_CLK")
                        .pin_rename("TX_CLK_B", "TX_CLK")
                        .pin_rename("RX_SERDES_CLK_B0", "RX_SERDES_CLK0")
                        .pin_rename("RX_SERDES_CLK_B1", "RX_SERDES_CLK1")
                        .pin_rename("RX_SERDES_CLK_B2", "RX_SERDES_CLK2")
                        .pin_rename("RX_SERDES_CLK_B3", "RX_SERDES_CLK3")
                        .pin_rename("RX_SERDES_CLK_B4", "RX_SERDES_CLK4")
                        .pin_rename("RX_SERDES_CLK_B5", "RX_SERDES_CLK5")
                        .pin_rename("RX_SERDES_CLK_B6", "RX_SERDES_CLK6")
                        .pin_rename("RX_SERDES_CLK_B7", "RX_SERDES_CLK7")
                        .pin_rename("RX_SERDES_CLK_B8", "RX_SERDES_CLK8")
                        .pin_rename("RX_SERDES_CLK_B9", "RX_SERDES_CLK9");
                }
                if tcid == tcls::ILKN {
                    bel = bel
                        .pin_rename("DRP_CLK_B", "DRP_CLK")
                        .pin_rename("CORE_CLK_B", "CORE_CLK")
                        .pin_rename("LBUS_CLK_B", "LBUS_CLK")
                        .pin_rename("RX_SERDES_CLK_B0", "RX_SERDES_CLK0")
                        .pin_rename("RX_SERDES_CLK_B1", "RX_SERDES_CLK1")
                        .pin_rename("RX_SERDES_CLK_B2", "RX_SERDES_CLK2")
                        .pin_rename("RX_SERDES_CLK_B3", "RX_SERDES_CLK3")
                        .pin_rename("RX_SERDES_CLK_B4", "RX_SERDES_CLK4")
                        .pin_rename("RX_SERDES_CLK_B5", "RX_SERDES_CLK5")
                        .pin_rename("RX_SERDES_CLK_B6", "RX_SERDES_CLK6")
                        .pin_rename("RX_SERDES_CLK_B7", "RX_SERDES_CLK7")
                        .pin_rename("RX_SERDES_CLK_B8", "RX_SERDES_CLK8")
                        .pin_rename("RX_SERDES_CLK_B9", "RX_SERDES_CLK9")
                        .pin_rename("RX_SERDES_CLK_B10", "RX_SERDES_CLK10")
                        .pin_rename("RX_SERDES_CLK_B11", "RX_SERDES_CLK11")
                        .pin_rename("TX_SERDES_REFCLK_B", "TX_SERDES_REFCLK");
                }
                let mut xn = self.builder.xtile_id(tcid, naming, xy);
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_w_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_int_side(int_e_xy.delta(0, (i + i / 30) as i32), Dir::W, i + 60)
                        .ref_single(int_w_xy.delta(1, (i + i / 30) as i32), i, intf_w)
                        .ref_single(int_e_xy.delta(-1, (i + i / 30) as i32), i + 60, intf_e)
                }
                xn.bel(bel).extract();
            }
        }
    }

    fn fill_tiles_cfg(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("CFG_CFG").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
            let bels = [
                self.builder
                    .bel_xy(bslots::CFG, "CONFIG_SITE", 0, 0)
                    .pin_rename("ICAP_CLK_BOT_B", "ICAP_CLK_BOT")
                    .pin_rename("ICAP_CLK_TOP_B", "ICAP_CLK_TOP")
                    .pin_rename("USR_CCLK_O_B", "USR_CCLK_O")
                    .pin_rename("USR_DNA_CLK_B", "USR_DNA_CLK")
                    .pin_rename("USR_TCK_B", "USR_TCK"),
                self.builder
                    .bel_xy(bslots::ABUS_SWITCH_CFG, "ABUS_SWITCH", 0, 0),
            ];
            let mut xn = self.builder.xtile_id(tcls::CFG, "CFG", xy);
            for i in 0..60 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                    .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
            }
            xn.bels(bels).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("CFGIO_IOB")
            .iter()
            .next()
        {
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
            let bels = [
                self.builder.bel_xy(bslots::PMV, "PMV", 0, 0),
                self.builder.bel_xy(bslots::PMV2, "PMV2", 0, 0),
                self.builder.bel_xy(bslots::PMVIOB, "PMVIOB", 0, 0),
                self.builder.bel_xy(bslots::MTBF3, "MTBF3", 0, 0),
            ];
            let mut xn = self.builder.xtile_id(tcls::CFGIO, "CFGIO", xy);
            for i in 0..30 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                    .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
            }
            xn.bels(bels).extract();
        }

        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("AMS").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
            let mut bel = self
                .builder
                .bel_xy(bslots::SYSMON, "SYSMONE1", 0, 0)
                .pin_rename("CONVST_CLK_B", "CONVST_CLK")
                .pin_rename("RESET_USER_B", "RESET_USER")
                .pin_rename("DCLK_B", "DCLK")
                .pin_rename("TEST_ADC_CLK_B0", "TEST_ADC_CLK0")
                .pin_rename("TEST_ADC_CLK_B1", "TEST_ADC_CLK1")
                .pin_rename("TEST_ADC_CLK_B2", "TEST_ADC_CLK2")
                .pin_rename("TEST_ADC_CLK_B3", "TEST_ADC_CLK3")
                .pins_name_only(&["I2C_SCLK_TS", "I2C_SDA_TS"])
                .pin_name_only("I2C_SCLK_IN", 1)
                .pin_name_only("I2C_SDA_IN", 1);
            for i in 0..16 {
                bel = bel.pins_name_only(&[format!("VP_AUX{i}"), format!("VN_AUX{i}")]);
            }
            let mut xn = self.builder.xtile_id(tcls::AMS, "AMS", xy);
            for i in 0..30 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                    .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_cmt(&mut self) {
        let tkn = "XIPHY_L";
        for i in 0..24 {
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_VDISTR_BOT{i}"),
                30,
                wires::GCLK_VDISTR_S[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_VDISTR_TOP{i}"),
                30,
                wires::GCLK_VDISTR_N[i],
            );

            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_HROUTE_0_{i}"),
                30,
                wires::GCLK_HROUTE_FAR[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_HROUTE_1_{i}"),
                30,
                wires::GCLK_HROUTE[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_HDISTR_0_{i}"),
                30,
                wires::GCLK_HDISTR_FAR[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_HDISTR_1_{i}"),
                30,
                wires::GCLK_HDISTR[i],
            );

            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 1 + i * 8),
                30,
                wires::GCLK_HDISTR_FAR_MUX[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = i * 8),
                30,
                wires::GCLK_HDISTR_MUX[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 4 + i * 8),
                30,
                wires::OMUX_HDISTR_CMT[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 3 + i * 8),
                30,
                wires::GCLK_HROUTE_FAR_MUX[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 2 + i * 8),
                30,
                wires::GCLK_HROUTE_MUX[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 6 + i * 8),
                30,
                wires::GCLK_VDISTR_S_MUX[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 7 + i * 8),
                30,
                wires::GCLK_VDISTR_N_MUX[i],
            );

            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = i * 2 + 1),
                30,
                wires::IMUX_BUFG_CMT_HROUTE[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = i % 3 + i / 3 * 5),
                30,
                wires::IMUX_BUFG_CMT_PLL_CKINT[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_4TO1_{i}_CLK_OUT"),
                30,
                wires::IMUX_BUFG_CMT_TEST[i],
            );

            let dummy_base = [
                0, 3, 36, 53, 56, 59, 62, 65, 68, 71, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 39, 42,
                45, 48,
            ][i];
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_16_ENC_{i}_CLK_OUT"),
                30,
                wires::OMUX_BUFG_CMT[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("VCC_WIRE{dummy_base}"),
                30,
                wires::OMUX_BUFG_CMT_DUMMY0[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("VCC_WIRE{ii}", ii = dummy_base + 1),
                30,
                wires::OMUX_BUFG_CMT_DUMMY1[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("VCC_WIRE{ii}", ii = dummy_base + 2),
                30,
                wires::OMUX_BUFG_CMT_DUMMY2[i],
            );
        }
        for i in 0..6 {
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_XIPHY_{ii}_CLK_LEAF", ii = i + 6),
                29,
                wires::GCLK_XIPHY[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_XIPHY_{i}_CLK_LEAF"),
                30,
                wires::GCLK_XIPHY[i],
            );
        }
        for i in 0..4 {
            self.builder.extra_name_tile_cell(
                tkn,
                format!("IOB2CLK_CCIO{i}"),
                30,
                wires::CCIO_CMT[i],
            );
        }
        for byte in 0..4 {
            for i in [0, 6] {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "BITSLICE_RX_TX",
                    0,
                    byte * 13 + i,
                    "PHY2CLB_FIFO_WRCLK",
                    wires::OUT_PHY_CLK[i / 6].cell(byte * 15),
                    false,
                );
            }
        }
        let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let int_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
        let intf = self.builder.ndb.get_tile_class_naming("INTF_W_IO");
        let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
        let mut bels = vec![];
        for i in 0..24 {
            for (pin, wire) in [
                ("CLK_IN", wires::IMUX_BUFCE_ROW_CMT[i].cell(30)),
                ("CLK_OUT", wires::OUT_BUFCE_ROW_CMT[i].cell(30)),
                ("CLK_OUT_OPT_DLY", wires::OUT_BUFCE_ROW_CMT_DLY[i].cell(30)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "BUFCE_ROW", 0, i, pin, wire, false);
            }
            bels.push(
                self.builder
                    .bel_xy(bslots::BUFCE_ROW_CMT[i], "BUFCE_ROW", 0, i)
                    .pin_rename("CE_PRE_OPTINV", "CE"),
            );
        }
        for i in 0..24 {
            for (pin, wire) in [
                ("CLK_IN", wires::GCLK_HDISTR_TEST_CMT_IN[i].cell(30)),
                ("CLK_OUT", wires::GCLK_HDISTR_TEST_CMT[i].cell(30)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "GCLK_TEST_BUFE3", 0, i, pin, wire, false);
            }
            bels.push(
                self.builder
                    .bel_xy(bslots::GCLK_TEST_BUF_CMT[i], "GCLK_TEST_BUFE3", 0, i),
            );
        }
        for i in 0..24 {
            self.builder.extra_name_pin_xy(
                tkn,
                "BUFGCE",
                0,
                i,
                "CLK_IN",
                wires::IMUX_BUFG_CMT[i].cell(30),
                true,
            );
            self.builder.extra_name_pin_xy(
                tkn,
                "BUFGCE",
                0,
                i,
                "CLK_OUT",
                wires::OUT_BUFGCE_CMT[i].cell(30),
                false,
            );
            bels.push(
                self.builder
                    .bel_xy(bslots::BUFGCE[i], "BUFGCE", 0, i)
                    .pin_rename("CE_PRE_OPTINV", "CE"),
            );
        }
        for i in 0..8 {
            for (pin, wire) in [
                ("CLK_I0", wires::IMUX_BUFGCTRL_I0[i].cell(30)),
                ("CLK_I1", wires::IMUX_BUFGCTRL_I1[i].cell(30)),
                ("CLK_OUT", wires::OUT_BUFGCTRL[i].cell(30)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "BUFGCTRL", 0, i, pin, wire, false);
            }
            bels.push(
                self.builder
                    .bel_xy(bslots::BUFGCTRL[i], "BUFGCTRL", 0, i)
                    .pin_rename("CE0_PRE_OPTINV", "CE0")
                    .pin_rename("CE1_PRE_OPTINV", "CE1")
                    .pin_rename("IGNORE0_PRE_OPTINV", "IGNORE0")
                    .pin_rename("IGNORE1_PRE_OPTINV", "IGNORE1")
                    .pin_rename("SEL0_PRE_OPTINV", "SEL0")
                    .pin_rename("SEL1_PRE_OPTINV", "SEL1"),
            );
        }
        for i in 0..4 {
            self.builder.extra_name_pin_xy(
                tkn,
                "BUFGCE_DIV",
                0,
                i,
                "CLK_OUT",
                wires::OUT_BUFGCE_DIV[i].cell(30),
                false,
            );
            bels.push(
                self.builder
                    .bel_xy(bslots::BUFGCE_DIV[i], "BUFGCE_DIV", 0, i)
                    .pin_rename("RST_PRE_OPTINV", "RST")
                    .pin_rename("CE_PRE_OPTINV", "CE"),
            );
        }
        for i in 0..2 {
            let slot_name = "PLLE3_ADV";
            for (j, pin) in [
                "CLKOUT0", "CLKOUT0B", "CLKOUT1", "CLKOUT1B", "CLKFBOUT", "TMUXOUT",
            ]
            .into_iter()
            .enumerate()
            {
                self.builder.extra_name_pin_xy(
                    tkn,
                    slot_name,
                    0,
                    i,
                    pin,
                    [wires::OUT_PLL0, wires::OUT_PLL1][i][j].cell(30),
                    false,
                );
            }
            for (pin, wire) in [
                ("CLKIN", wires::IMUX_PLL_CLKIN[i].cell(30)),
                ("CLKFBIN", wires::IMUX_PLL_CLKFB[i].cell(30)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, slot_name, 0, i, pin, wire, false);
            }

            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = 56 + i * 2),
                30,
                wires::IMUX_PLL_CLKFB_HDISTR[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = 57 + i * 2),
                30,
                wires::IMUX_PLL_CLKFB_HDISTR_DLY[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 24 + i),
                30,
                wires::IMUX_PLL_CLKIN_MMCM[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = 60 + i * 3),
                30,
                wires::IMUX_PLL_CLKIN_HDISTR[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = 61 + i * 3),
                30,
                wires::IMUX_PLL_CLKIN_HROUTE[i],
            );
            self.builder.extra_name_tile_cell(
                tkn,
                format!("CLK_LEAF_MUX_{ii}_CLK_LEAF", ii = 62 + i * 3),
                30,
                wires::IMUX_PLL_CLKIN_HDISTR_DLY[i],
            );

            bels.push(
                self.builder
                    .bel_xy(bslots::PLL[i], "PLLE3_ADV", 0, i)
                    .pin_rename("DCLK_B", "DCLK")
                    .pin_rename("SCANCLK_B", "SCANCLK")
                    .pins_name_only(&["CLKOUTPHY_P"]),
            );
        }
        for (i, pin) in [
            "CLKOUT0",
            "CLKOUT0B",
            "CLKOUT1",
            "CLKOUT1B",
            "CLKOUT2",
            "CLKOUT2B",
            "CLKOUT3",
            "CLKOUT3B",
            "CLKOUT4",
            "CLKOUT5",
            "CLKOUT6",
            "CLKFBOUT",
            "CLKFBOUTB",
            "TMUXOUT",
        ]
        .into_iter()
        .enumerate()
        {
            self.builder.extra_name_pin_xy(
                tkn,
                "MMCME3_ADV",
                0,
                0,
                pin,
                wires::OUT_MMCM[i].cell(30),
                false,
            );
        }
        for (pin, wire) in [
            ("CLKIN1", wires::IMUX_MMCM_CLKIN1.cell(30)),
            ("CLKIN2", wires::IMUX_MMCM_CLKIN2.cell(30)),
            ("CLKFBIN", wires::IMUX_MMCM_CLKFB.cell(30)),
        ] {
            self.builder
                .extra_name_pin_xy(tkn, "MMCME3_ADV", 0, 0, pin, wire, false);
        }

        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_48_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKFB_HDISTR,
        );
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_49_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKFB_HDISTR_DLY,
        );
        self.builder
            .extra_name_tile_cell(tkn, "VCC_WIRE51", 30, wires::IMUX_MMCM_CLKFB_DUMMY0);
        self.builder
            .extra_name_tile_cell(tkn, "VCC_WIRE52", 30, wires::IMUX_MMCM_CLKFB_DUMMY1);
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_50_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN1_HDISTR,
        );
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_51_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN1_HROUTE,
        );
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_52_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN1_HDISTR_DLY,
        );
        self.builder
            .extra_name_tile_cell(tkn, "GND_WIRE0", 30, wires::IMUX_MMCM_CLKIN1_DUMMY0);
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_53_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN2_HDISTR,
        );
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_54_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN2_HROUTE,
        );
        self.builder.extra_name_tile_cell(
            tkn,
            "CLK_LEAF_MUX_55_CLK_LEAF",
            30,
            wires::IMUX_MMCM_CLKIN2_HDISTR_DLY,
        );
        self.builder
            .extra_name_tile_cell(tkn, "GND_WIRE1", 30, wires::IMUX_MMCM_CLKIN2_DUMMY0);
        bels.push(
            self.builder
                .bel_xy(bslots::MMCM, "MMCME3_ADV", 0, 0)
                .pin_rename("DCLK_B", "DCLK")
                .pin_rename("PSCLK_B", "PSCLK")
                .pin_rename("SCANCLK_B", "SCANCLK"),
        );
        bels.push(
            self.builder
                .bel_xy(bslots::ABUS_SWITCH_CMT, "ABUS_SWITCH", 0, 0),
        );
        let mut xn = self
            .builder
            .xtile_id(tcls::CMT, "CMT", xy)
            .switchbox(bslots::CMT_INT)
            .optin_muxes(&[
                wires::IMUX_MMCM_CLKIN1,
                wires::IMUX_MMCM_CLKIN1_HDISTR,
                wires::IMUX_MMCM_CLKIN1_HDISTR_DLY,
                wires::IMUX_MMCM_CLKIN1_HROUTE,
                wires::IMUX_MMCM_CLKIN2,
                wires::IMUX_MMCM_CLKIN2_HDISTR,
                wires::IMUX_MMCM_CLKIN2_HDISTR_DLY,
                wires::IMUX_MMCM_CLKIN2_HROUTE,
                wires::IMUX_MMCM_CLKFB,
                wires::IMUX_MMCM_CLKFB_HDISTR,
                wires::IMUX_MMCM_CLKFB_HDISTR_DLY,
            ])
            .optin_muxes(&wires::IMUX_PLL_CLKIN[..])
            .optin_muxes(&wires::IMUX_PLL_CLKIN_HDISTR[..])
            .optin_muxes(&wires::IMUX_PLL_CLKIN_HDISTR_DLY[..])
            .optin_muxes(&wires::IMUX_PLL_CLKIN_HROUTE[..])
            .optin_muxes(&wires::IMUX_PLL_CLKIN_MMCM[..])
            .optin_muxes(&wires::IMUX_PLL_CLKFB[..])
            .optin_muxes(&wires::IMUX_PLL_CLKFB_HDISTR[..])
            .optin_muxes(&wires::IMUX_PLL_CLKFB_HDISTR_DLY[..])
            .optin_muxes(&wires::IMUX_BUFG_CMT[..])
            .optin_muxes(&wires::IMUX_BUFG_CMT_HROUTE[..])
            .optin_muxes(&wires::IMUX_BUFG_CMT_PLL_CKINT[..])
            .optin_muxes(&wires::IMUX_BUFG_CMT_TEST[..])
            .optin_muxes(&wires::IMUX_BUFGCTRL_I0[..])
            .optin_muxes(&wires::IMUX_BUFGCTRL_I1[..])
            .optin_muxes(&wires::OMUX_BUFG_CMT[..])
            .optin_muxes(&wires::IMUX_BUFCE_ROW_CMT[..])
            .optin_muxes(&wires::OMUX_HDISTR_CMT[..])
            .optin_muxes(&wires::GCLK_HDISTR_TEST_CMT_IN[..])
            .optin_muxes(&wires::GCLK_HROUTE_MUX[..])
            .optin_muxes(&wires::GCLK_HROUTE[..])
            .optin_muxes(&wires::GCLK_HROUTE_FAR_MUX[..])
            .optin_muxes(&wires::GCLK_HROUTE_FAR[..])
            .optin_muxes(&wires::GCLK_VDISTR_S_MUX[..])
            .optin_muxes(&wires::GCLK_VDISTR_N_MUX[..])
            .optin_muxes(&wires::GCLK_VDISTR_S[..])
            .optin_muxes(&wires::GCLK_VDISTR_N[..])
            .optin_muxes(&wires::GCLK_HDISTR_MUX[..])
            .optin_muxes(&wires::GCLK_HDISTR[..])
            .optin_muxes(&wires::GCLK_HDISTR_FAR_MUX[..])
            .optin_muxes(&wires::GCLK_HDISTR_FAR[..])
            .optin_muxes(&wires::GCLK_XIPHY[..])
            .ref_xlat(int_xy.delta(0, 30), &[Some(30), None, None, None], rclk_int);
        for i in 0..60 {
            xn = xn
                .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                .ref_single(int_xy.delta(-1, (i + i / 30) as i32), i, intf)
        }
        xn.bels(bels).extract();
    }

    fn fill_tiles_xiphy(&mut self) {
        let Some(&xy) = self.builder.rd.tiles_by_kind_name("XIPHY_L").iter().next() else {
            return;
        };
        let int_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
        let intf = self.builder.ndb.get_tile_class_naming("INTF_W_IO");

        // yuuup, override the damn thing
        for i in 0..6 {
            self.builder.extra_name_tile(
                "XIPHY_L",
                format!("CLK_LEAF_MUX_XIPHY_{ii}_CLK_LEAF", ii = i + 6),
                wires::GCLK_XIPHY[i],
            );
            self.builder.extra_name_tile(
                "XIPHY_L",
                format!("CLK_LEAF_MUX_XIPHY_{i}_CLK_LEAF"),
                wires::GCLK_XIPHY[i],
            );
        }

        // XIPHY
        for byte in 0..4 {
            // absolute horrible hack.
            let extra_names_tile = self.builder.extra_names_tile.clone();
            let mut bels = vec![];
            for i in 0..13 {
                let fi = byte * 13 + i;
                for (pin, w) in [
                    ("RX_CLK", wires::IMUX_BITSLICE_CLK),
                    ("RX_CLK_C", wires::IMUX_BITSLICE_RX_CLK_C),
                    ("RX_CLK_C_B", wires::IMUX_BITSLICE_RX_CLK_C_B),
                    ("TX_OCLK", wires::IMUX_BITSLICE_TX_OCLK),
                ] {
                    self.builder.extra_name_pin_xy(
                        "XIPHY_L",
                        "BITSLICE_RX_TX",
                        0,
                        fi,
                        pin,
                        w[i].cell(0),
                        pin == "RX_CLK",
                    );
                }
                if matches!(i, 0 | 6) {
                    self.builder.extra_name_pin_xy(
                        "XIPHY_L",
                        "BITSLICE_RX_TX",
                        0,
                        fi,
                        "PHY2CLB_FIFO_WRCLK",
                        wires::OUT_PHY_CLK[i / 6].cell(0),
                        false,
                    );
                }

                let mut bel = self
                    .builder
                    .bel_xy(bslots::BITSLICE[i], "BITSLICE_RX_TX", 0, fi)
                    .manual()
                    .pin_rename("RX_CLK", "CLK")
                    .pin_rename("CLB2PHY_FIFO_RDEN", "FIFO_RDEN")
                    .pin_rename("PHY2CLB_FIFO_EMPTY", "FIFO_EMPTY")
                    .pin_rename("PHY2CLB_FIFO_WRCLK", "FIFO_WRCLK")
                    .pins_name_only(&[
                        "RX_CLKDIV",
                        "TX_CLK",
                        "TX_OCLKDIV",
                        "TX_DIV2_CLK",
                        "TX_DIV4_CLK",
                        "TX_DDR_CLK",
                        "TX_CTRL_CLK",
                        "TX_CTRL_CE",
                        "TX_CTRL_INC",
                        "TX_CTRL_LD",
                        "TX_TBYTE_IN",
                        "TX_WL_TRAIN",
                        "TX_MUX_360_N_SEL",
                        "TX_MUX_360_P_SEL",
                        "TX_MUX_720_P0_SEL",
                        "TX_MUX_720_P1_SEL",
                        "TX_MUX_720_P2_SEL",
                        "TX_MUX_720_P3_SEL",
                        "TX_VTC_READY",
                        "TX_TOGGLE_DIV2_SEL",
                        "TX_BS_RESET",
                        "TX_REGRST_B",
                        "TX_RST_B",
                        "TX_Q",
                        "RX_CLK_P",
                        "RX_CLK_N",
                        "RX_CTRL_CLK",
                        "RX_CTRL_CE",
                        "RX_CTRL_INC",
                        "RX_CTRL_LD",
                        "RX_RST_B",
                        "RX_DCC0",
                        "RX_DCC1",
                        "RX_DCC2",
                        "RX_DCC3",
                        "RX_VTC_READY",
                        "RX_RESET_B",
                        "RX_BS_RESET",
                        "RX_DQS_OUT",
                        "TX2RX_CASC_IN",
                        "TX2RX_CASC_OUT",
                        "RX2TX_CASC_RETURN_IN",
                        "CLB2PHY_FIFO_CLK",
                        "CTL2BS_FIFO_BYPASS",
                        "CTL2BS_RX_RECALIBRATE_EN",
                        "CTL2BS_TX_DDR_PHASE_SEL",
                        "CTL2BS_DYNAMIC_MODE_EN",
                        "BS2CTL_IDELAY_DELAY_FORMAT",
                        "BS2CTL_ODELAY_DELAY_FORMAT",
                        "BS2CTL_TX_DDR_PHASE_SEL",
                        "BS2CTL_RX_P0_DQ_OUT",
                        "BS2CTL_RX_N0_DQ_OUT",
                        "BS2CTL_RX_DDR_EN_DQS",
                    ])
                    .pin_name_only("RX_D", 1)
                    .extra_wire(
                        "DYN_DCI_OUT",
                        &[match fi {
                            0..=11 => format!("DYNAMIC_DCI_TS_BOT{fi}"),
                            12 => "DYNAMIC_DCI_TS_BOT_VR1".to_string(),
                            13..=24 => format!("DYNAMIC_DCI_TS_BOT{ii}", ii = fi - 1),
                            25 => "DYNAMIC_DCI_TS_BOT_VR2".to_string(),
                            26..=37 => format!("DYNAMIC_DCI_TS_TOP{ii}", ii = fi - 26),
                            38 => "DYNAMIC_DCI_TS_TOP_VR1".to_string(),
                            39..=50 => format!("DYNAMIC_DCI_TS_TOP{ii}", ii = fi - 27),
                            51 => "DYNAMIC_DCI_TS_TOP_VR2".to_string(),
                            _ => unreachable!(),
                        }],
                    )
                    .extra_int_in(
                        "DYN_DCI_OUT_INT",
                        &[match fi {
                            0..=11 => format!("CLB2PHY_DYNAMIC_DCI_TS_BOT{fi}"),
                            12 => "CLB2PHY_DYNAMIC_DCI_TS_BOT_VR1".to_string(),
                            13..=24 => format!("CLB2PHY_DYNAMIC_DCI_TS_BOT{ii}", ii = fi - 1),
                            25 => "CLB2PHY_DYNAMIC_DCI_TS_BOT_VR2".to_string(),
                            26..=37 => format!("CLB2PHY_DYNAMIC_DCI_TS_TOP{ii}", ii = fi - 26),
                            38 => "CLB2PHY_DYNAMIC_DCI_TS_TOP_VR1".to_string(),
                            39..=50 => format!("CLB2PHY_DYNAMIC_DCI_TS_TOP{ii}", ii = fi - 27),
                            51 => "CLB2PHY_DYNAMIC_DCI_TS_TOP_VR2".to_string(),
                            _ => unreachable!(),
                        }],
                    );
                if !matches!(i, 0 | 6) {
                    bel = bel.pins_name_only(&["FIFO_WRCLK"]);
                }
                for j in 0..18 {
                    bel = bel.pins_name_only(&[
                        format!("BS2CTL_IDELAY_FIXED_DLY_RATIO{j}"),
                        format!("BS2CTL_ODELAY_FIXED_DLY_RATIO{j}"),
                    ]);
                }
                for j in 0..9 {
                    bel = bel.pins_name_only(&[
                        format!("BS2CTL_RX_CNTVALUEOUT{j}"),
                        format!("BS2CTL_TX_CNTVALUEOUT{j}"),
                        format!("RX_CTRL_DLY{j}"),
                        format!("TX_CTRL_DLY{j}"),
                    ]);
                }
                bels.push(bel);
            }
            for i in 0..2 {
                let fi = byte * 2 + i;
                self.builder.extra_name_pin_xy(
                    "XIPHY_L",
                    "BITSLICE_TX",
                    0,
                    fi,
                    "CLK",
                    wires::IMUX_BITSLICE_T_CLK[i].cell(0),
                    false,
                );
                let mut bel = self
                    .builder
                    .bel_xy(bslots::BITSLICE_T[i], "BITSLICE_TX", 0, fi)
                    .manual()
                    .pins_name_only(&[
                        "DIV2_CLK",
                        "DIV4_CLK",
                        "DDR_CLK",
                        "CTRL_CLK",
                        "CTRL_CE",
                        "CTRL_INC",
                        "CTRL_LD",
                        "TX_MUX_360_N_SEL",
                        "TX_MUX_360_P_SEL",
                        "TX_MUX_720_P0_SEL",
                        "TX_MUX_720_P1_SEL",
                        "TX_MUX_720_P2_SEL",
                        "TX_MUX_720_P3_SEL",
                        "TOGGLE_DIV2_SEL",
                        "D0",
                        "D1",
                        "D2",
                        "D3",
                        "D4",
                        "D5",
                        "D6",
                        "D7",
                        "Q",
                        "RST_B",
                        "REGRST_B",
                        "BS_RESET",
                        "CDATAIN0",
                        "CDATAIN1",
                        "CDATAOUT",
                        "CTL2BS_TX_DDR_PHASE_SEL",
                        "CTL2BS_DYNAMIC_MODE_EN",
                        "BS2CTL_TX_DDR_PHASE_SEL",
                        "FORCE_OE_B",
                        "VTC_READY",
                    ]);
                for j in 0..9 {
                    bel = bel.pins_name_only(&[
                        format!("BS2CTL_CNTVALUEOUT{j}"),
                        format!("CTRL_DLY{j}"),
                    ]);
                }
                bels.push(bel);
            }
            for i in 0..2 {
                let fi = byte * 2 + i;
                let mut bel = self
                    .builder
                    .bel_xy(bslots::BITSLICE_CONTROL[i], "BITSLICE_CONTROL", 0, fi)
                    .manual()
                    .pins_name_only(&[
                        "PDQS_GT_IN",
                        "NDQS_GT_IN",
                        "PDQS_GT_OUT",
                        "NDQS_GT_OUT",
                        "FORCE_OE_B",
                        "PLL_CLK",
                        "PLL_CLK_EN",
                        "REFCLK_DFD",
                        "CLK_TO_EXT_SOUTH",
                        "CLK_TO_EXT_NORTH",
                        "CLB2PHY_CTRL_RST_B",
                        "LOCAL_DIV_CLK",
                        "BS_RESET_TRI",
                        "TRISTATE_ODELAY_CE_OUT",
                        "TRISTATE_ODELAY_INC_OUT",
                        "TRISTATE_ODELAY_LD_OUT",
                        "TRISTATE_VTC_READY",
                        "SCAN_INT",
                        "RIU2CLB_VALID",
                        "CLK_STOP",
                        "CLK_FROM_EXT",
                    ]);
                for j in 0..7 {
                    bel = bel.pins_name_only(&[
                        format!("RX_DCC{j:02}_0"),
                        format!("RX_DCC{j:02}_1"),
                        format!("RX_DCC{j:02}_2"),
                        format!("RX_DCC{j:02}_3"),
                        format!("RX_PDQ{j}_IN"),
                        format!("RX_NDQ{j}_IN"),
                        format!("IDELAY_CTRL_CLK{j}"),
                        format!("IDELAY_CE_OUT{j}"),
                        format!("IDELAY_INC_OUT{j}"),
                        format!("IDELAY_LD_OUT{j}"),
                        format!("FIXED_IDELAY{j:02}"),
                        format!("ODELAY_CE_OUT{j}"),
                        format!("ODELAY_INC_OUT{j}"),
                        format!("ODELAY_LD_OUT{j}"),
                        format!("FIXED_ODELAY{j:02}"),
                        format!("VTC_READY_IDELAY{j:02}"),
                        format!("VTC_READY_ODELAY{j:02}"),
                        format!("WL_TRAIN{j}"),
                        format!("DYN_DCI_OUT{j}"),
                        format!("DQS_IN{j}"),
                        format!("RX_BS_RESET{j}"),
                        format!("TX_BS_RESET{j}"),
                        format!("PDQS_OUT{j}"),
                        format!("NDQS_OUT{j}"),
                        format!("REFCLK_EN{j}"),
                        format!("IFIFO_BYPASS{j}"),
                        format!("BS2CTL_RIU_BS_DQS_EN{j}"),
                    ]);
                    for k in 0..9 {
                        bel = bel.pins_name_only(&[
                            format!("IDELAY{j:02}_IN{k}"),
                            format!("IDELAY{j:02}_OUT{k}"),
                            format!("ODELAY{j:02}_IN{k}"),
                            format!("ODELAY{j:02}_OUT{k}"),
                        ]);
                    }
                    for k in 0..18 {
                        bel = bel.pins_name_only(&[
                            format!("FIXDLYRATIO_IDELAY{j:02}_{k}"),
                            format!("FIXDLYRATIO_ODELAY{j:02}_{k}"),
                        ]);
                    }
                }
                for j in 0..8 {
                    bel = bel.pins_name_only(&[
                        format!("ODELAY_CTRL_CLK{j}"),
                        format!("DYNAMIC_MODE_EN{j}"),
                        format!("EN_DIV_DLY_OE{j}"),
                        format!("TOGGLE_DIV2_SEL{j}"),
                        format!("TX_DATA_PHASE{j}"),
                        format!("BS2CTL_RIU_TX_DATA_PHASE{j}"),
                        format!("DIV2_CLK_OUT{j}"),
                        format!("DIV_CLK_OUT{j}"),
                        format!("DDR_CLK_OUT{j}"),
                        format!("PH02_DIV2_360_{j}"),
                        format!("PH13_DIV2_360_{j}"),
                        format!("PH0_DIV_720_{j}"),
                        format!("PH1_DIV_720_{j}"),
                        format!("PH2_DIV_720_{j}"),
                        format!("PH3_DIV_720_{j}"),
                    ]);
                }
                for j in 0..9 {
                    bel = bel.pins_name_only(&[
                        format!("TRISTATE_ODELAY_IN{j}"),
                        format!("TRISTATE_ODELAY_OUT{j}"),
                    ]);
                }
                for j in 0..16 {
                    bel = bel.pins_name_only(&[format!("RIU2CLB_RD_DATA{j}")]);
                }
                bel = bel
                    .sub_xy("PLL_SELECT_SITE", 0, fi ^ 1)
                    .pin_rename("D0", "PLL_SELECT_D0")
                    .pin_rename("D1", "PLL_SELECT_D1")
                    .pin_rename("REFCLK_DFD", "PLL_SELECT_REFCLK_DFD")
                    .pin_rename("PLL_CLK_EN", "PLL_SELECT_PLL_CLK_EN")
                    .pin_rename("Z", "PLL_SELECT_Z")
                    .pins_name_only(&[
                        "PLL_SELECT_REFCLK_DFD",
                        "PLL_SELECT_Z",
                        "PLL_SELECT_PLL_CLK_EN",
                    ])
                    .pin_name_only("PLL_SELECT_D0", 1)
                    .pin_name_only("PLL_SELECT_D1", 1);

                bels.push(bel);
            }
            let mut bel = self
                .builder
                .bel_xy(bslots::XIPHY_BYTE, "XIPHY_FEEDTHROUGH", byte, 0)
                .manual()
                .pins_name_only(&[
                    "CLB2PHY_CTRL_RST_B_LOW_SMX",
                    "CLB2PHY_CTRL_RST_B_UPP_SMX",
                    "CLB2PHY_TRISTATE_ODELAY_RST_B_SMX0",
                    "CLB2PHY_TRISTATE_ODELAY_RST_B_SMX1",
                    "CLB2PHY_TXBIT_TRI_RST_B_SMX0",
                    "CLB2PHY_TXBIT_TRI_RST_B_SMX1",
                    "SCAN_INT_LOWER",
                    "SCAN_INT_UPPER",
                    "DIV_CLK_OUT_LOW",
                    "DIV_CLK_OUT_UPP",
                    "XIPHY_CLK_STOP_CTRL_LOW",
                    "XIPHY_CLK_STOP_CTRL_UPP",
                    "RCLK2PHY_CLKDR",
                    "RCLK2PHY_SHIFTDR",
                ]);
            for j in 0..13 {
                bel = bel.pins_name_only(&[
                    format!("CLB2PHY_TXBIT_RST_B_SMX{j}"),
                    format!("CLB2PHY_RXBIT_RST_B_SMX{j}"),
                    format!("CLB2PHY_FIFO_CLK_SMX{j}"),
                    format!("CLB2PHY_IDELAY_RST_B_SMX{j}"),
                    format!("CLB2PHY_ODELAY_RST_B_SMX{j}"),
                ]);
            }
            for j in 0..6 {
                bel = bel.pins_name_only(&[format!("CTL2BS_REFCLK_EN_LOW_SMX{j}")]);
            }
            for j in 0..7 {
                bel = bel.pins_name_only(&[
                    format!("CTL2BS_REFCLK_EN_LOW{j}"),
                    format!("CTL2BS_REFCLK_EN_UPP{j}"),
                    format!("CTL2BS_REFCLK_EN_UPP_SMX{j}"),
                ]);
            }
            bel = bel
                .sub_xy("RIU_OR", 0, byte)
                .pins_name_only(&["RIU_RD_VALID_LOW", "RIU_RD_VALID_UPP"]);
            for j in 0..16 {
                bel = bel.pins_name_only(&[
                    format!("RIU_RD_DATA_LOW{j}"),
                    format!("RIU_RD_DATA_UPP{j}"),
                ]);
            }
            bels.push(bel);

            let naming = &format!("XIPHY{byte}");
            let mut xn = self
                .builder
                .xtile_id(tcls::XIPHY, naming, xy)
                .switchbox(bslots::XIPHY_INT)
                .optin_muxes(&wires::IMUX_BITSLICE_CLK[..])
                .optin_muxes(&wires::IMUX_BITSLICE_RX_CLK_C[..])
                .optin_muxes(&wires::IMUX_BITSLICE_RX_CLK_C_B[..])
                .optin_muxes(&wires::IMUX_BITSLICE_TX_OCLK[..])
                .optin_muxes(&wires::IMUX_BITSLICE_T_CLK[..]);
            for i in 0..15 {
                let fi = byte * 15 + i;
                xn = xn
                    .ref_int_side(int_xy.delta(0, (fi + fi / 30) as i32), Dir::W, i)
                    .ref_single(int_xy.delta(-1, (fi + fi / 30) as i32), i, intf)
            }
            let mut xt = xn.bels(bels).extract();
            self.builder.extra_names_tile = extra_names_tile;

            let (mut bel_byte, mut naming_byte) = xt.bels.pop().unwrap();
            let mut bels_control = xt.bels.split_off(15);
            let mut bels_bitslice_t = xt.bels.split_off(13);
            let mut bels_bitslice = xt.bels;

            for i in 0..13 {
                for (pname, bpname) in [
                    ("FIFO_RDCLK", format!("CLB2PHY_FIFO_CLK{i}")),
                    ("RX_RST", format!("RXBIT_RST_B{i}")),
                    ("TX_RST", format!("TXBIT_RST_B{i}")),
                    ("RX_RST_DLY", format!("IDELAY_RST_B{i}")),
                    ("TX_RST_DLY", format!("ODELAY_RST_B{i}")),
                ] {
                    let pin = bel_byte.pins.remove(&bpname).unwrap();
                    let pn = naming_byte.pins[&bpname].clone();
                    bels_bitslice[i].0.pins.insert(pname.into(), pin);
                    bels_bitslice[i].1.pins.insert(pname.into(), pn);
                }
            }
            for i in 0..2 {
                for (pname, bpname) in [
                    ("RST", format!("TXBIT_TRI_RST_B{i}")),
                    ("RST_DLY", format!("TRISTATE_ODELAY_RST_B{i}")),
                ] {
                    let pin = bel_byte.pins.remove(&bpname).unwrap();
                    let pn = naming_byte.pins[&bpname].clone();
                    bels_bitslice_t[i].0.pins.insert(pname.into(), pin);
                    bels_bitslice_t[i].1.pins.insert(pname.into(), pn);
                }
            }

            for (tgt, pname, bpname) in [
                (0, "CLK", "CLB2PHY_CTRL_CLK_LOW"),
                (1, "CLK", "CLB2PHY_CTRL_CLK_UPP"),
                (0, "RST", "CTRL_RST_B_LOW"),
                (1, "RST", "CTRL_RST_B_UPP"),
            ] {
                let pin = bel_byte.pins.remove(bpname).unwrap();
                let pn = naming_byte.pins[bpname].clone();
                bels_control[tgt].0.pins.insert(pname.into(), pin);
                bels_control[tgt].1.pins.insert(pname.into(), pn);
            }

            {
                let pins = std::mem::take(&mut bel_byte.pins);

                for (pname, pin) in pins {
                    if pname.starts_with("CLB2PHY_DBG")
                        || pname.starts_with("PHY2CLB_DBG")
                        || pname.starts_with("RIU_")
                    {
                        bel_byte.pins.insert(pname, pin);
                    } else {
                        let new_pname = pname
                            .strip_prefix("CLB2PHY_")
                            .or_else(|| pname.strip_prefix("PHY2CLB_"))
                            .unwrap();
                        bel_byte.pins.insert(new_pname.into(), pin);
                        let pn = naming_byte.pins.remove(&pname).unwrap();
                        naming_byte.pins.insert(new_pname.into(), pn);
                    }
                }
            }

            for i in 0..2 {
                let (ref mut bel_c, ref mut bn_c) = bels_control[i];
                bel_c.pins.retain(|pname, pin| {
                    if let Some(stem) = pname.strip_prefix("CLB2RIU_") {
                        let new_pname = if stem == "NIBBLE_SEL" {
                            format!("RIU_NIBBLE_SEL{i}")
                        } else {
                            format!("RIU_{stem}")
                        };
                        let mut pn = bn_c.pins[pname].clone();
                        pn.name = pn.name_far.clone();
                        pn.pips.clear();
                        match bel_byte.pins.entry(new_pname) {
                            btree_map::Entry::Vacant(e) => {
                                naming_byte.pins.insert(e.key().clone(), pn);
                                e.insert(pin.clone());
                            }
                            btree_map::Entry::Occupied(e) => {
                                assert_eq!(naming_byte.pins[e.key()], pn);
                                assert_eq!(e.get(), pin);
                            }
                        }
                        false
                    } else {
                        true
                    }
                });

                // kill RIU_CLK pin (redundant with CLK)
                let pin = bel_c.pins.remove("RIU_CLK").unwrap();
                assert_eq!(pin, bel_c.pins["CLK"]);

                let pins = std::mem::take(&mut bel_c.pins);

                for (pname, pin) in pins {
                    if let Some(new_pname) = pname
                        .strip_prefix("CLB2PHY_")
                        .or(pname.strip_prefix("PHY2CLB_"))
                    {
                        bel_c.pins.insert(new_pname.into(), pin);
                        let pn = bn_c.pins.remove(&pname).unwrap();
                        bn_c.pins.insert(new_pname.into(), pn);
                    } else {
                        bel_c.pins.insert(pname, pin);
                    }
                }
            }

            for ((bel, bn), bslot) in bels_bitslice.into_iter().zip(bslots::BITSLICE) {
                self.builder.insert_tcls_bel(tcls::XIPHY, bslot, bel.into());
                self.builder.insert_bel_naming(naming, bslot, bn);
            }
            for ((bel, bn), bslot) in bels_bitslice_t.into_iter().zip(bslots::BITSLICE_T) {
                self.builder.insert_tcls_bel(tcls::XIPHY, bslot, bel.into());
                self.builder.insert_bel_naming(naming, bslot, bn);
            }
            for ((bel, bn), bslot) in bels_control.into_iter().zip(bslots::BITSLICE_CONTROL) {
                self.builder.insert_tcls_bel(tcls::XIPHY, bslot, bel.into());
                self.builder.insert_bel_naming(naming, bslot, bn);
            }
            self.builder
                .insert_tcls_bel(tcls::XIPHY, bslots::XIPHY_BYTE, bel_byte.into());
            self.builder
                .insert_bel_naming(naming, bslots::XIPHY_BYTE, naming_byte);
        }
    }

    fn fill_tiles_hpio(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("HPIO_L").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_W_IO");
            let mut bels = vec![];
            let mut is_nocfg = true;
            if let Some(wire) = self.builder.rd.wires.get("HPIO_IOB_3_TSDI_PIN") {
                let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                if tk.wires.contains_key(&wire) {
                    is_nocfg = false;
                }
            }
            for i in 0..26 {
                let mut bel = self
                    .builder
                    .bel_xy(bslots::HPIOB[i], "IOB", 0, i)
                    .pins_name_only(&[
                        "I",
                        "OUTB_B_IN",
                        "OUTB_B",
                        "TSTATE_IN",
                        "TSTATE_OUT",
                        "CTLE_IN",
                        "DOUT",
                        "IO",
                        "LVDS_TRUE",
                        "PAD_RES",
                        "O_B",
                        "TSTATEB",
                        "DYNAMIC_DCI_TS",
                        "VREF",
                    ])
                    .pin_name_only("SWITCH_OUT", usize::from(matches!(i, 4..=11 | 13..=20)))
                    .pin_name_only("OP", 1)
                    .pin_name_only("TSP", 1)
                    .pin_name_only("TSDI", 1);
                if is_nocfg || i == 25 {
                    bel = bel.pin_name_only("TSDI", 0);
                }
                if matches!(i, 12 | 25) {
                    bel = bel.pin_dummy("IO");
                }
                if let Some(idx) = match i {
                    0 => Some(0),
                    2 => Some(1),
                    21 => Some(2),
                    23 => Some(3),
                    _ => None,
                } {
                    bel = bel.extra_int_out_force(
                        "CLKPAD",
                        wires::OUT_IOB_CLKPAD[idx].cell(0),
                        [
                            "HPIO_IOB_3_I",
                            "HPIO_IOB_7_I",
                            "HPIO_IOB_43_I",
                            "HPIO_IOB_47_I",
                        ][idx],
                    );
                }
                bels.push(bel);
            }
            for i in 0..12 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HPIOB_DIFF_IN[i], "HPIOBDIFFINBUF", 0, i)
                        .pins_name_only(&[
                            "LVDS_TRUE",
                            "LVDS_COMP",
                            "PAD_RES_0",
                            "PAD_RES_1",
                            "VREF",
                            "CTLE_IN_1",
                        ]),
                );
            }
            for i in 0..12 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HPIOB_DIFF_OUT[i], "HPIOBDIFFOUTBUF", 0, i)
                        .pins_name_only(&["AOUT", "BOUT", "O_B", "TSTATEB"]),
                );
            }
            bels.push(
                self.builder
                    .bel_xy(bslots::HPIO_VREF, "HPIO_VREF_SITE", 0, 0)
                    .pins_name_only(&["VREF1", "VREF2"]),
            );
            let naming = if is_nocfg { "HPIO_NOCFG" } else { "HPIO" };
            let mut xn = self.builder.xtile_id(tcls::HPIO, naming, xy);
            for i in 0..30 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_xy.delta(-1, (i + i / 30) as i32), i, intf)
            }
            xn.bels(bels).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("RCLK_HPIO_L")
            .iter()
            .next()
        {
            let int_xy = self
                .builder
                .walk_to_int(xy.delta(0, -30), Dir::E, false)
                .unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_W_IO");
            let mut bels = vec![];
            for i in 0..5 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::ABUS_SWITCH_HPIO[i], "ABUS_SWITCH", i, 0),
                );
            }
            bels.push(
                self.builder
                    .bel_xy(bslots::HPIO_ZMATCH, "HPIO_ZMATCH_BLK_HCLK", 0, 0),
            );
            let mut xn = self.builder.xtile_id(tcls::RCLK_HPIO, "RCLK_HPIO", xy);
            for i in 0..60 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_xy.delta(-1, (i + i / 30) as i32), i, intf)
            }
            xn.bels(bels).extract();
            let pips = self
                .builder
                .pips
                .entry((tcls::RCLK_HPIO, bslots::RCLK_IOB_INT))
                .or_default();
            for i in 0..4 {
                pips.pips.insert(
                    (
                        wires::CCIO_CMT[i].cell(30),
                        wires::OUT_IOB_CLKPAD[i]
                            .cell(if i < 2 { 30 } else { 0 })
                            .pos(),
                    ),
                    PipMode::PermaBuf,
                );
            }
        }
    }

    fn fill_tiles_hrio(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("HRIO_L").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_W_IO");
            let mut bels = vec![];
            let mut is_nocfg = true;
            if let Some(wire) = self.builder.rd.wires.get("HRIO_IOB_5_TSDI_PIN") {
                let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                if tk.wires.contains_key(&wire) {
                    is_nocfg = false;
                }
            }
            for i in 0..26 {
                let mut bel = self
                    .builder
                    .bel_xy(bslots::HRIOB[i], "IOB", 0, i)
                    .pins_name_only(&[
                        "DOUT",
                        "OUTB_B_IN",
                        "OUTB_B",
                        "TSTATEIN",
                        "TSTATEOUT",
                        "IO",
                        "TMDS_IBUF_OUT",
                        "DRIVER_BOT_IBUF",
                        "O_B",
                        "TSTATEB",
                        "DYNAMIC_DCI_TS",
                    ])
                    .pin_name_only("SWITCH_OUT", usize::from(matches!(i, 4..=11 | 13..=20)))
                    .pin_name_only("OP", 1)
                    .pin_name_only("TSP", 1)
                    .pin_name_only("TSDI", 1);
                if is_nocfg || i == 25 {
                    bel = bel.pin_name_only("TSDI", 0);
                }
                if matches!(i, 12 | 25) {
                    bel = bel.pin_dummy("IO");
                }
                if let Some(idx) = match i {
                    0 => Some(0),
                    2 => Some(1),
                    21 => Some(2),
                    23 => Some(3),
                    _ => None,
                } {
                    bel = bel.extra_int_out_force(
                        "CLKPAD",
                        wires::OUT_IOB_CLKPAD[idx].cell(0),
                        [
                            "HRIO_IOB_5_DOUT",
                            "HRIO_IOB_9_DOUT",
                            "HRIO_IOB_45_DOUT",
                            "HRIO_IOB_49_DOUT",
                        ][idx],
                    );
                }
                bels.push(bel);
            }
            for i in 0..12 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HRIOB_DIFF_IN[i], "HRIODIFFINBUF", 0, i)
                        .pins_name_only(&[
                            "LVDS_IBUF_OUT",
                            "LVDS_IBUF_OUT_B",
                            "LVDS_IN_P",
                            "LVDS_IN_N",
                        ]),
                );
            }
            for i in 0..12 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HRIOB_DIFF_OUT[i], "HRIODIFFOUTBUF", 0, i)
                        .pins_name_only(&["AOUT", "BOUT", "O_B", "TSTATEB"]),
                );
            }
            let naming = if is_nocfg { "HRIO_NOCFG" } else { "HRIO" };
            let mut xn = self.builder.xtile_id(tcls::HRIO, naming, xy);
            for i in 0..30 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_xy.delta(-1, (i + i / 30) as i32), i, intf)
            }
            xn.bels(bels).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("RCLK_HRIO_L")
            .iter()
            .next()
        {
            let mut bels = vec![];
            for i in 0..8 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::ABUS_SWITCH_HRIO[i], "ABUS_SWITCH", i, 0),
                );
            }
            self.builder
                .xtile_id(tcls::RCLK_HRIO, "RCLK_HRIO", xy)
                .bels(bels)
                .extract();
            let pips = self
                .builder
                .pips
                .entry((tcls::RCLK_HRIO, bslots::RCLK_IOB_INT))
                .or_default();
            for i in 0..4 {
                pips.pips.insert(
                    (
                        wires::CCIO_CMT[i].cell(30),
                        wires::OUT_IOB_CLKPAD[i]
                            .cell(if i < 2 { 30 } else { 0 })
                            .pos(),
                    ),
                    PipMode::PermaBuf,
                );
            }
        }
    }

    fn fill_tiles_rclk(&mut self) {
        for (tcid, naming, tkn) in [
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "PCIE",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CMAC_CMAC_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "ILMAC_ILMAC_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CFG_CFG",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "RCLK_AMS_CFGIO",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_CLE,
                "RCLK_HROUTE_SPLITTER_CLE",
                "RCLK_CLEM_CLKBUF_L",
            ),
        ] {
            for i in 0..24 {
                if tcid == tcls::RCLK_HROUTE_SPLITTER_CLE {
                    self.builder.extra_name_tile(
                        tkn,
                        format!("CLK_HROUTE_L{i}"),
                        wires::GCLK_HROUTE_FAR[i],
                    );
                    self.builder.extra_name_tile(
                        tkn,
                        format!("CLK_HROUTE_R{i}"),
                        wires::GCLK_HROUTE[i],
                    );
                } else {
                    self.builder.extra_name_tile(
                        tkn,
                        format!("CLK_HROUTE_L{i}"),
                        wires::GCLK_HROUTE[i],
                    );
                    self.builder.extra_name_tile(
                        tkn,
                        format!("CLK_HROUTE_R{i}"),
                        wires::GCLK_HROUTE_FAR[i],
                    );
                }
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::RCLK_SPLITTER)
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .optin_muxes(&wires::GCLK_HROUTE_FAR[..])
                    .extract();
            }
        }

        {
            let tkn = "RCLK_DSP_CLKBUF_L";
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_L{i}"),
                    wires::GCLK_HDISTR[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_R{i}"),
                    wires::GCLK_HDISTR_FAR[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HROUTE_L{i}"),
                    wires::GCLK_HROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HROUTE_R{i}"),
                    wires::GCLK_HROUTE_FAR[i],
                );
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                self.builder
                    .xtile_id(tcls::RCLK_SPLITTER, "RCLK_SPLITTER", xy)
                    .switchbox(bslots::RCLK_SPLITTER)
                    .optin_muxes(&wires::GCLK_HDISTR[..])
                    .optin_muxes(&wires::GCLK_HDISTR_FAR[..])
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .optin_muxes(&wires::GCLK_HROUTE_FAR[..])
                    .extract();
            }
        }

        for tkn in [
            "RCLK_CLEL_L",
            "RCLK_CLEL_R",
            "RCLK_CLEL_R_L",
            "RCLK_CLEL_R_R",
            "RCLK_CLE_M_L",
            "RCLK_CLE_M_R",
            "RCLK_BRAM_L",
            "RCLK_BRAM_R",
            "RCLK_RCLK_BRAM_L_AUXCLMP_FT",
            "RCLK_RCLK_BRAM_L_BRAMCLMP_FT",
            "RCLK_DSP_L",
        ] {
            self.builder
                .extra_name_tile(tkn, "CLK_HROUTE_CORE_OPT", wires::GCLK_HROUTE_RCLK[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VDISTR_BOT", wires::GCLK_VDISTR_S[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VDISTR_TOP", wires::GCLK_VDISTR_N[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VROUTE_BOT", wires::GCLK_VROUTE_S[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VROUTE_TOP", wires::GCLK_VROUTE_N[0]);

            for i in 0..2 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HROUTE_CORE_OPT{i}"),
                    wires::GCLK_HROUTE_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_VDISTR_BOT{i}"),
                    wires::GCLK_VDISTR_S[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_VDISTR_TOP{i}"),
                    wires::GCLK_VDISTR_N[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_VROUTE_BOT{i}"),
                    wires::GCLK_VROUTE_S[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_VROUTE_TOP{i}"),
                    wires::GCLK_VROUTE_N[i],
                );

                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = i * 4),
                    wires::GCLK_VDISTR_S_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = i * 4 + 1),
                    wires::GCLK_VDISTR_N_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = i * 4 + 2),
                    wires::GCLK_VROUTE_S_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = i * 4 + 3),
                    wires::GCLK_VROUTE_N_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = i * 2 + 1),
                    wires::GCLK_HROUTE_MUX[i],
                );

                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFCE_ROW_{ii}_CLK_IN", ii = i * 2),
                    wires::IMUX_BUFCE_ROW_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFCE_ROW_{ii}_CLK_OUT", ii = i * 2),
                    wires::OUT_BUFCE_ROW_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_TEST_BUF_SITE_{ii}_CLK_OUT", ii = i * 2 + 1),
                    wires::GCLK_HDISTR_TEST_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_TEST_BUF_SITE_{ii}_CLK_IN", ii = i * 2 + 1),
                    wires::GCLK_HDISTR_RCLK[i],
                );
            }
        }
        for (tkn, side) in [
            ("RCLK_CLEL_L", Dir::W),
            ("RCLK_CLEL_R", Dir::W),
            ("RCLK_CLEL_R_L", Dir::E),
            ("RCLK_CLEL_R_R", Dir::E),
            ("RCLK_CLE_M_L", Dir::W),
            ("RCLK_CLE_M_R", Dir::W),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let is_alt = self.dev_naming.rclk_alt_pins[tkn];
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let int_xy = xy.delta(if side == Dir::W { 1 } else { -1 }, 0);
                let bels = vec![
                    self.builder
                        .bel_xy(defs::bslots::BUFCE_ROW_RCLK[0], "BUFCE_ROW", 0, 0)
                        .pin_rename("CE_PRE_OPTINV", "CE")
                        .pins_name_only(&["CLK_OUT_OPT_DLY"]),
                    self.builder.bel_xy(
                        defs::bslots::GCLK_TEST_BUF_RCLK[0],
                        "GCLK_TEST_BUFE3",
                        0,
                        0,
                    ),
                ];
                self.builder
                    .xtile_id(
                        tcls::RCLK_V_SINGLE_CLE,
                        if is_alt {
                            "RCLK_V_SINGLE_ALT"
                        } else {
                            "RCLK_V_SINGLE"
                        },
                        xy,
                    )
                    .switchbox(bslots::RCLK_V_INT)
                    .optin_muxes(&wires::IMUX_BUFCE_ROW_RCLK[..])
                    .optin_muxes(&wires::GCLK_HDISTR_RCLK[..])
                    .optin_muxes(&wires::GCLK_HROUTE_RCLK[..])
                    .optin_muxes(&wires::GCLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::GCLK_VDISTR_S[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N[..])
                    .optin_muxes(&wires::GCLK_VDISTR_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_MUX[..])
                    .ref_xlat(
                        int_xy,
                        &if side == Dir::W {
                            [Some(0), None, None, None]
                        } else {
                            [None, Some(0), None, None]
                        },
                        rclk_int,
                    )
                    .bels(bels)
                    .extract();
            }
        }

        for (tkn, tcid, side) in [
            ("RCLK_BRAM_L", tcls::RCLK_V_DOUBLE_BRAM, Dir::W),
            ("RCLK_BRAM_R", tcls::RCLK_V_DOUBLE_BRAM, Dir::W),
            (
                "RCLK_RCLK_BRAM_L_AUXCLMP_FT",
                tcls::RCLK_V_DOUBLE_BRAM,
                Dir::W,
            ),
            (
                "RCLK_RCLK_BRAM_L_BRAMCLMP_FT",
                tcls::RCLK_V_DOUBLE_BRAM,
                Dir::W,
            ),
            ("RCLK_DSP_L", tcls::RCLK_V_DOUBLE_DSP, Dir::E),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let is_alt = self.dev_naming.rclk_alt_pins[tkn];
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let int_xy = xy.delta(if side == Dir::W { 2 } else { -2 }, 0);
                let mut bels = vec![];
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFCE_ROW_RCLK[i], "BUFCE_ROW", i, 0)
                            .pin_rename("CE_PRE_OPTINV", "CE")
                            .pins_name_only(&["CLK_OUT_OPT_DLY"]),
                    );
                }
                for i in 0..2 {
                    bels.push(self.builder.bel_xy(
                        bslots::GCLK_TEST_BUF_RCLK[i],
                        "GCLK_TEST_BUFE3",
                        i,
                        0,
                    ));
                }
                self.builder
                    .xtile_id(
                        tcid,
                        if is_alt {
                            "RCLK_V_DOUBLE_ALT"
                        } else {
                            "RCLK_V_DOUBLE"
                        },
                        xy,
                    )
                    .switchbox(bslots::RCLK_V_INT)
                    .optin_muxes(&wires::IMUX_BUFCE_ROW_RCLK[..])
                    .optin_muxes(&wires::GCLK_HDISTR_RCLK[..])
                    .optin_muxes(&wires::GCLK_HROUTE_RCLK[..])
                    .optin_muxes(&wires::GCLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::GCLK_VDISTR_S[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N[..])
                    .optin_muxes(&wires::GCLK_VDISTR_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_MUX[..])
                    .ref_xlat(
                        int_xy,
                        &if side == Dir::W {
                            [Some(0), None, None, None]
                        } else {
                            [None, Some(0), None, None]
                        },
                        rclk_int,
                    )
                    .bels(bels)
                    .extract();
            }
        }
    }

    fn fill_tiles_gt(&mut self) {
        for (tkn, tcid, naming, side) in [
            ("GTH_QUAD_LEFT_FT", tcls::GTH, "GTH_L", Dir::W),
            ("GTY_QUAD_LEFT_FT", tcls::GTY, "GTY_L", Dir::W),
            ("GTH_R", tcls::GTH, "GTH_R", Dir::E),
        ] {
            if tcid == tcls::GTH {
                for (n, w) in [
                    "GTH_COMMON_BLH_2_REFCLK2HROW0",
                    "GTH_CHANNEL_BLH_4_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_3_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_4_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_3_RXRECCLK_INT",
                    "GTH_COMMON_BLH_2_REFCLK2HROW1",
                    "GTH_CHANNEL_BLH_0_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_1_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_0_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_1_RXRECCLK_INT",
                ]
                .into_iter()
                .zip(wires::OUT_GT_CLK)
                {
                    self.builder.extra_name_tile_cell(tkn, n, 30, w);
                }
            } else {
                for (n, w) in [
                    "GTY_COMMON_BLH_0_REFCLK2HROW0",
                    "GTY_CHANNEL_BLH_4_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_2_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_4_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_2_RXRECCLK_INT",
                    "GTY_COMMON_BLH_0_REFCLK2HROW1",
                    "GTY_CHANNEL_BLH_1_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_3_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_1_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_3_RXRECCLK_INT",
                ]
                .into_iter()
                .zip(wires::OUT_GT_CLK)
                {
                    self.builder.extra_name_tile_cell(tkn, n, 30, w);
                }
            }
            for i in 0..24 {
                if side == Dir::W {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HDISTR_FT1_{i}"),
                        30,
                        wires::GCLK_HDISTR[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HROUTE_FT1_{i}"),
                        30,
                        wires::GCLK_HROUTE[i],
                    );
                } else {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HDISTR_FT0_{i}"),
                        30,
                        wires::GCLK_HDISTR[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HROUTE_FT0_{i}"),
                        30,
                        wires::GCLK_HROUTE[i],
                    );
                }
                let idx = 9 + i;
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_WITH_OPTINVS_{idx}_CLK_IN"),
                    30,
                    wires::IMUX_BUFG_GT_CLK[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_WITH_OPTINVS_{idx}_RST_PRE_OPTINV"),
                    30,
                    wires::IMUX_BUFG_GT_RST[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_WITH_OPTINVS_{idx}_CE"),
                    30,
                    wires::IMUX_BUFG_GT_CE[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_WITH_OPTINVS_{idx}_CLK_OUT"),
                    30,
                    wires::OUT_BUFG_GT[i],
                );
                let bi = [
                    (0, 5, 60),
                    (135, 140, 145),
                    (215, 220, 230),
                    (235, 240, 245),
                    (250, 255, 260),
                    (265, 270, 275),
                    (285, 290, 295),
                    (300, 305, 310),
                    (315, 320, 325),
                    (330, 340, 345),
                    (115, 170, 225),
                    (280, 335, 350),
                    (355, 10, 15),
                    (20, 25, 30),
                    (35, 40, 45),
                    (50, 55, 65),
                    (70, 75, 80),
                    (85, 90, 95),
                    (100, 105, 110),
                    (120, 125, 130),
                    (150, 155, 160),
                    (165, 175, 180),
                    (185, 190, 195),
                    (200, 205, 210),
                ][i];
                for (j, w) in [
                    wires::IMUX_BUFG_GT_CE_DUMMY0,
                    wires::IMUX_BUFG_GT_CE_DUMMY1,
                    wires::IMUX_BUFG_GT_CE_DUMMY2,
                    wires::IMUX_BUFG_GT_CE_DUMMY3,
                    wires::IMUX_BUFG_GT_CE_DUMMY4,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.0 + j),
                        30,
                        w[i],
                    );
                }
                for (j, w) in [
                    wires::IMUX_BUFG_GT_CLK_DUMMY0,
                    wires::IMUX_BUFG_GT_CLK_DUMMY1,
                    wires::IMUX_BUFG_GT_CLK_DUMMY2,
                    wires::IMUX_BUFG_GT_CLK_DUMMY3,
                    wires::IMUX_BUFG_GT_CLK_DUMMY4,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.1 + j),
                        30,
                        w[i],
                    );
                }
                for (j, w) in [
                    wires::IMUX_BUFG_GT_RST_DUMMY0,
                    wires::IMUX_BUFG_GT_RST_DUMMY1,
                    wires::IMUX_BUFG_GT_RST_DUMMY2,
                    wires::IMUX_BUFG_GT_RST_DUMMY3,
                    wires::IMUX_BUFG_GT_RST_DUMMY4,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.2 + j),
                        30,
                        w[i],
                    );
                }
            }
            for i in 0..15 {
                let idx = 33 + i;
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_SYNC_BOTH_{idx}_RST_OUT"),
                    30,
                    wires::OUT_BUFG_GT_SYNC_RST[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_BUFG_GT_SYNC_BOTH_{idx}_CE_OUT"),
                    30,
                    wires::OUT_BUFG_GT_SYNC_CE[i],
                );
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, !side, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(if side == Dir::W {
                    "INTF_W_GT"
                } else {
                    "INTF_E_GT"
                });
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let gtk = match tcid {
                    tcls::GTH => "GTH",
                    tcls::GTY => "GTY",
                    _ => unreachable!(),
                };
                let mut bels = vec![];
                for i in 0..24 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFG_GT[i], "BUFG_GT", 0, i)
                            .pin_rename("RST_PRE_OPTINV", "RST"),
                    );
                }
                for i in 0..11 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFG_GT_SYNC[i], "BUFG_GT_SYNC", 0, i),
                    );
                }
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::ABUS_SWITCH_GT[i], "ABUS_SWITCH", 0, i),
                    );
                }

                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::GT3_CHANNEL[i], &format!("{gtk}E3_CHANNEL"), 0, i)
                            .pins_name_only(&[
                                "MGTREFCLK0",
                                "MGTREFCLK1",
                                "NORTHREFCLK0",
                                "NORTHREFCLK1",
                                "SOUTHREFCLK0",
                                "SOUTHREFCLK1",
                                "QDCMREFCLK0_INT",
                                "QDCMREFCLK1_INT",
                                "QDPLL0CLK0P_INT",
                                "QDPLL1CLK0P_INT",
                                "RING_OSC_CLK_INT",
                                "RXRECCLKOUT",
                            ]),
                    );
                }
                bels.push(
                    self.builder
                        .bel_xy(bslots::GT3_COMMON, &format!("{gtk}E3_COMMON"), 0, 0)
                        .pins_name_only(&[
                            "RXRECCLK0",
                            "RXRECCLK1",
                            "RXRECCLK2",
                            "RXRECCLK3",
                            "QDCMREFCLK_INT_0",
                            "QDCMREFCLK_INT_1",
                            "QDPLLCLK0P_0",
                            "QDPLLCLK0P_1",
                            "COM0_REFCLKOUT0",
                            "COM0_REFCLKOUT1",
                            "COM0_REFCLKOUT2",
                            "COM0_REFCLKOUT3",
                            "COM0_REFCLKOUT4",
                            "COM0_REFCLKOUT5",
                            "COM2_REFCLKOUT0",
                            "COM2_REFCLKOUT1",
                            "COM2_REFCLKOUT2",
                            "COM2_REFCLKOUT3",
                            "COM2_REFCLKOUT4",
                            "COM2_REFCLKOUT5",
                            "MGTREFCLK0",
                            "MGTREFCLK1",
                            "SARC_CLK0",
                            "SARC_CLK1",
                            "SARC_CLK2",
                            "SARC_CLK3",
                        ])
                        .extra_wire("CLKOUT_NORTH0", &["CLKOUT_NORTH0"])
                        .extra_wire("CLKOUT_NORTH1", &["CLKOUT_NORTH1"])
                        .extra_wire("CLKOUT_SOUTH0", &["CLKOUT_SOUTH0"])
                        .extra_wire("CLKOUT_SOUTH1", &["CLKOUT_SOUTH1"])
                        .extra_wire(
                            "NORTHREFCLK0",
                            &[
                                "GTH_CHANNEL_BLH_0_NORTHREFCLK0",
                                "GTY_CHANNEL_BLH_1_NORTHREFCLK0",
                            ],
                        )
                        .extra_wire(
                            "NORTHREFCLK1",
                            &[
                                "GTH_CHANNEL_BLH_0_NORTHREFCLK1",
                                "GTY_CHANNEL_BLH_1_NORTHREFCLK1",
                            ],
                        )
                        .extra_wire(
                            "SOUTHREFCLK0",
                            &[
                                "GTH_CHANNEL_BLH_0_SOUTHREFCLK0",
                                "GTY_CHANNEL_BLH_1_SOUTHREFCLK0",
                            ],
                        )
                        .extra_wire(
                            "SOUTHREFCLK1",
                            &[
                                "GTH_CHANNEL_BLH_0_SOUTHREFCLK1",
                                "GTY_CHANNEL_BLH_1_SOUTHREFCLK1",
                            ],
                        )
                        .extra_wire("VCC", &["VCC_WIRE"]),
                );
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::GT_INT)
                    .optin_muxes(&wires::IMUX_BUFG_GT_CLK[..])
                    .optin_muxes(&wires::IMUX_BUFG_GT_RST[..])
                    .optin_muxes(&wires::IMUX_BUFG_GT_CE[..])
                    .optin_muxes(&wires::GCLK_HDISTR[..])
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .ref_xlat(
                        int_xy.delta(0, 30),
                        if side == Dir::W {
                            &[Some(30), None, None, None]
                        } else {
                            &[None, Some(30), None, None]
                        },
                        rclk_int,
                    );
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), side, i)
                        .ref_single(
                            int_xy.delta(if side == Dir::W { -1 } else { 1 }, (i + i / 30) as i32),
                            i,
                            intf,
                        )
                }
                xn.bels(bels).extract();
            }
        }
    }
}

pub fn make_int_db(rd: &Part, dev_naming: &DeviceNaming) -> (IntDb, NamingDb) {
    let mut maker = IntMaker {
        builder: IntBuilder::new(
            rd,
            bincode::decode_from_slice(defs::ultrascale::INIT, bincode::config::standard())
                .unwrap()
                .0,
        ),
        sng_fixup_map: BTreeMap::new(),
        dev_naming,
    };

    maker
        .builder
        .inject_main_passes(EnumMap::from_fn(|dir| match dir {
            Dir::W => ccls::PASS_W,
            Dir::E => ccls::PASS_E,
            Dir::S => ccls::PASS_S,
            Dir::N => ccls::PASS_N,
        }));

    maker.fill_wires();

    maker.fill_tiles_int();
    maker.fill_terms();
    maker.fill_tiles_rclk_int();
    maker.fill_tiles_intf();
    maker.fill_tiles_clb();
    maker.fill_tiles_lag();
    maker.fill_tiles_bram();
    maker.fill_tiles_dsp();
    maker.fill_tiles_hard();
    maker.fill_tiles_cfg();
    maker.fill_tiles_cmt();
    maker.fill_tiles_xiphy();
    maker.fill_tiles_hpio();
    maker.fill_tiles_hrio();
    maker.fill_tiles_rclk();
    maker.fill_tiles_gt();

    maker.builder.build()
}
