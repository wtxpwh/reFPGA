use std::collections::{BTreeMap, BTreeSet, btree_map};

use prjcombine_interconnect::{
    db::{
        BelInfo, CellSlotId, ConnectorWire, IntDb, LegacyBel, TileWireCoord, WireSlotId,
        WireSlotIdExt,
    },
    dir::{Dir, DirMap, DirV},
};
use prjcombine_re_xilinx_rawdump::{Coord, Part};

use prjcombine_entity::{EntityId, EntityPartVec};
use prjcombine_re_xilinx_naming::db::{BelNaming, NamingDb};
use prjcombine_re_xilinx_naming_ultrascale::DeviceNaming;
use prjcombine_re_xilinx_rd2db_interconnect::{IntBuilder, PipMode, XTileInfo, XTileRef};
use prjcombine_ultrascale::defs::{
    self, bslots, ccls, tcls, ultrascaleplus::wires, wiredata::ultrascaleplus as wiredata,
};

const XLAT24: [usize; 24] = [
    0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15,
];

const BUFCE_LEAF_SWIZZLE: [[usize; 16]; 2] = [
    [0, 1, 7, 6, 5, 4, 12, 13, 14, 15, 23, 22, 21, 20, 28, 29],
    [3, 2, 8, 9, 10, 11, 19, 18, 17, 16, 24, 25, 26, 27, 31, 30],
];

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
    rd: &'a Part,
    builder: IntBuilder<'a>,
    // how many mental illnesses do you think I could be diagnosed with just from this repo?
    sng_fixup_map: BTreeMap<TileWireCoord, TileWireCoord>,
    dev_naming: &'a DeviceNaming,
}

impl IntMaker<'_> {
    fn fill_wires_long(&mut self) {
        let d2n = DirMap::from_fn(|dir| match dir {
            Dir::N => 0,
            Dir::S => 1,
            Dir::E => 2,
            Dir::W => 3,
        });

        for (w, dir, fts, ftn) in [
            (wiredata::LX12_W.as_slice(), Dir::W, None, None),
            (
                wiredata::LX12_E.as_slice(),
                Dir::E,
                Some(wires::LX12_E6_S0),
                Some(wires::LX12_E6_N7),
            ),
            (wiredata::LX12_S.as_slice(), Dir::S, None, None),
            (wiredata::LX12_N.as_slice(), Dir::N, None, None),
        ] {
            let length = w.len() - 1;
            let ftd = d2n[!dir];
            for i in 0..8 {
                self.builder
                    .wire_names(w[0][i], &[format!("{dir}{dir}12_BEG{i}")]);
                for j in 1..length {
                    let nn = (b'A' + (j as u8 - 1)) as char;
                    let wname = format!("{dir}{dir}12_{nn}_FT{ftd}_{i}");
                    self.builder.wire_names(w[j][i], &[wname]);
                }
                let wname = format!("{dir}{dir}12_END{i}");
                self.builder.wire_names(w[length][i], &[wname]);
                if i == 0
                    && let Some(fts) = fts
                {
                    self.builder
                        .wire_names(fts, &[format!("{dir}{dir}12_BLS_{i}_FT0")]);
                }
                if i == 7
                    && let Some(ftn) = ftn
                {
                    self.builder
                        .wire_names(ftn, &[format!("{dir}{dir}12_BLN_{i}_FT1")]);
                }
            }
        }
    }

    fn fill_wires_sdqnode(&mut self) {
        for i in 0..96 {
            match i {
                0 | 2 => {
                    self.builder.wire_names_pair(
                        wires::SDQNODE[i],
                        &[format!("SDQNODE_W_{i}_FT1"), format!("SDQNODE_E_{i}_FT1")],
                    );
                    self.builder.wire_names_pair(
                        wires::SDQNODE_S[i],
                        &[
                            format!("SDQNODE_W_BLS_{i}_FT0"),
                            format!("SDQNODE_E_BLS_{i}_FT0"),
                        ],
                    );
                }
                91 | 93 | 95 => {
                    self.builder.wire_names_pair(
                        wires::SDQNODE[i],
                        &[format!("SDQNODE_W_{i}_FT0"), format!("SDQNODE_E_{i}_FT0")],
                    );
                    self.builder.wire_names_pair(
                        wires::SDQNODE_N[i],
                        &[
                            format!("SDQNODE_W_BLN_{i}_FT1"),
                            format!("SDQNODE_E_BLN_{i}_FT1"),
                        ],
                    );
                }
                _ => {
                    // TODO not the true permutation
                    let a = [
                        0, 11, 22, 33, 44, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17,
                        18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 38,
                        39, 40, 41, 42, 43, 45, 46, 47,
                    ][i >> 1];
                    let aa = a + 48;
                    let b = i & 1;
                    self.builder.wire_names_pair(
                        wires::SDQNODE[i],
                        &[
                            format!("INT_NODE_SDQ_{aa}_INT_OUT{b}"),
                            format!("INT_NODE_SDQ_{a}_INT_OUT{b}"),
                        ],
                    );
                }
            }
        }
    }

    fn fill_wires_sdq(&mut self) {
        let d2n = DirMap::from_fn(|dir| match dir {
            Dir::N => 0,
            Dir::S => 1,
            Dir::E => 2,
            Dir::W => 3,
        });

        for (w, dir, fts, ftn) in [
            (wiredata::X1_E.as_slice(), Dir::E, None, None),
            (
                wiredata::X1_W.as_slice(),
                Dir::W,
                None,
                Some(wires::X1_W1_N7),
            ),
            (wiredata::X1_N.as_slice(), Dir::N, None, None),
            (wiredata::X1_S.as_slice(), Dir::S, None, None),
            (wiredata::X2_E.as_slice(), Dir::E, None, None),
            (
                wiredata::X2_W.as_slice(),
                Dir::W,
                Some(wires::X2_W2_S0),
                None,
            ),
            (wiredata::X2_N.as_slice(), Dir::N, None, None),
            (wiredata::X2_S.as_slice(), Dir::S, None, None),
            (wiredata::X4_E.as_slice(), Dir::E, None, None),
            (wiredata::X4_W.as_slice(), Dir::W, None, None),
            (
                wiredata::X4_N.as_slice(),
                Dir::N,
                None,
                Some(wires::X4_N4_N7),
            ),
            (
                wiredata::X4_S.as_slice(),
                Dir::S,
                Some(wires::X4_S4_S0),
                None,
            ),
        ] {
            let length = w.len() - 1;
            let ftd = d2n[!dir];
            for i in 0..8 {
                let name_w = if length == 1 && dir == Dir::E {
                    let (a, b) = [
                        (60, 1),
                        (4, 0),
                        (61, 1),
                        (5, 0),
                        (62, 1),
                        (6, 0),
                        (63, 1),
                        (7, 0),
                    ][i];
                    format!("INT_INT_SDQ_{a}_INT_OUT{b}")
                } else {
                    format!("{dir}{dir}{length}_W_BEG{i}")
                };
                let name_e = if length == 1 && dir == Dir::W {
                    if i == 7 {
                        format!("{dir}{dir}{length}_E_{i}_FT0")
                    } else {
                        let (a, b) = [
                            (72, 0),
                            (32, 1),
                            (73, 0),
                            (33, 1),
                            (74, 0),
                            (34, 1),
                            (75, 0),
                        ][i];
                        format!("INT_INT_SDQ_{a}_INT_OUT{b}")
                    }
                } else {
                    format!("{dir}{dir}{length}_E_BEG{i}")
                };
                self.builder.wire_names_pair(w[0][i], &[name_w, name_e]);
                for j in 1..length {
                    let nn = match dir {
                        Dir::W | Dir::E => {
                            if j.is_multiple_of(2) {
                                Some((b'A' + (j as u8 / 2 - 1)) as char)
                            } else {
                                None
                            }
                        }
                        Dir::S | Dir::N => Some((b'A' + (j as u8 - 1)) as char),
                    };
                    if let Some(nn) = nn {
                        self.builder.wire_names_pair(
                            w[j][i],
                            &[
                                format!("{dir}{dir}{length}_W_{nn}_FT{ftd}_{i}"),
                                format!("{dir}{dir}{length}_E_{nn}_FT{ftd}_{i}"),
                            ],
                        );
                    }
                }
                self.builder.wire_names_pair(
                    w[length][i],
                    &if length == 1 && matches!(dir, Dir::E | Dir::W) {
                        [
                            format!("{dir}{dir}{length}_E_END{i}"),
                            format!("{dir}{dir}{length}_W_END{i}"),
                        ]
                    } else {
                        [
                            format!("{dir}{dir}{length}_W_END{i}"),
                            format!("{dir}{dir}{length}_E_END{i}"),
                        ]
                    },
                );
                match (length, dir) {
                    (1, Dir::W) => {
                        self.sng_fixup_map.insert(w[0][i].cell(1), w[1][i].cell(0));
                    }
                    (1, Dir::E) => {
                        self.sng_fixup_map.insert(w[0][i].cell(0), w[1][i].cell(1));
                    }
                    _ => (),
                }
                if i == 0
                    && let Some(fts) = fts
                {
                    self.builder.wire_names_pair(
                        fts,
                        &[
                            format!("{dir}{dir}{length}_W_BLS_{i}_FT0"),
                            format!("{dir}{dir}{length}_E_BLS_{i}_FT0"),
                        ],
                    );
                }
                if i == 7
                    && let Some(ftn) = ftn
                {
                    self.builder.wire_names_pair(
                        ftn,
                        &if length == 1 {
                            [
                                format!("{dir}{dir}{length}_E_BLN_{i}_FT1"),
                                format!("{dir}{dir}{length}_W_BLN_{i}_FT1"),
                            ]
                        } else {
                            [
                                format!("{dir}{dir}{length}_W_BLN_{i}_FT1"),
                                format!("{dir}{dir}{length}_E_BLN_{i}_FT1"),
                            ]
                        },
                    );
                }
            }
        }
    }

    fn fill_wires_inode(&mut self) {
        for i in 0..64 {
            match i {
                1 | 3 | 5 | 9 => {
                    self.builder.wire_names_pair(
                        wires::INODE[i],
                        &[format!("INODE_W_{i}_FT1"), format!("INODE_E_{i}_FT1")],
                    );
                    self.builder.wire_names_pair(
                        wires::INODE_S[i],
                        &[
                            format!("INODE_W_BLS_{i}_FT0"),
                            format!("INODE_E_BLS_{i}_FT0"),
                        ],
                    );
                }
                54 | 58 | 60 | 62 => {
                    self.builder.wire_names_pair(
                        wires::INODE[i],
                        &[format!("INODE_W_{i}_FT0"), format!("INODE_E_{i}_FT0")],
                    );
                    self.builder.wire_names_pair(
                        wires::INODE_N[i],
                        &[
                            format!("INODE_W_BLN_{i}_FT1"),
                            format!("INODE_E_BLN_{i}_FT1"),
                        ],
                    );
                }
                _ => {
                    // TODO not the true permutation
                    let a = [
                        0, 11, 22, 30, 31, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17,
                        18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 29,
                    ][i >> 1];
                    let aa = a + 32;
                    let b = i & 1;
                    self.builder.extra_name_tile_cell(
                        "INT",
                        format!("INT_NODE_IMUX_{aa}_INT_OUT{b}"),
                        0,
                        wires::INODE[i],
                    );
                    self.builder.extra_name_tile_cell(
                        "INT",
                        format!("INT_NODE_IMUX_{a}_INT_OUT{b}"),
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
                &[format!("CTRL_W{i}"), format!("CTRL_E{i}")],
            );
        }

        for i in 0..16 {
            match i {
                0 | 2 => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BOUNCE_W_{i}_FT1"), format!("BOUNCE_E_{i}_FT1")],
                    );
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP_S[i],
                        &[
                            format!("BOUNCE_W_BLS_{i}_FT0"),
                            format!("BOUNCE_E_BLS_{i}_FT0"),
                        ],
                    );
                }
                13 | 15 => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BOUNCE_W_{i}_FT0"), format!("BOUNCE_E_{i}_FT0")],
                    );
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP_N[i],
                        &[
                            format!("BOUNCE_W_BLN_{i}_FT1"),
                            format!("BOUNCE_E_BLN_{i}_FT1"),
                        ],
                    );
                }
                _ => {
                    self.builder.wire_names_pair(
                        wires::IMUX_BYP[i],
                        &[format!("BYPASS_W{i}"), format!("BYPASS_E{i}")],
                    );
                }
            };
            self.builder
                .mark_delay(wires::IMUX_BYP[i], wires::IMUX_BYP_DELAY[i]);
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
        for i in 0..24 {
            self.builder.wire_names_pair(
                wires::IMUX_RCLK[i],
                &[
                    if i < 16 {
                        format!(
                            "CLK_LEAF_SITES_{idx}_CE_INT",
                            idx = BUFCE_LEAF_SWIZZLE[i / 8][i % 8 + 8]
                        )
                    } else {
                        format!("INT_RCLK_TO_CLK_LEFT_{a}_{b}", a = i % 2, b = (i - 16) / 2)
                    },
                    if i < 16 {
                        format!(
                            "CLK_LEAF_SITES_{idx}_CE_INT",
                            idx = BUFCE_LEAF_SWIZZLE[i / 8][i % 8]
                        )
                    } else if i < 22 {
                        format!("INT_RCLK_TO_CLK_RIGHT_{a}_{b}", a = i % 2, b = (i - 16) / 2)
                    } else if i == 22 {
                        "CLK_LEAF_SITES_0_ENSEL_PROG".into()
                    } else if i == 23 {
                        "CLK_LEAF_SITES_0_CLK_CASC_IN".into()
                    } else {
                        unreachable!()
                    },
                ],
            );
        }
        for i in 0..24 {
            for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("INT_NODE_IMUX_{a}_INT_OUT{b}", a = i / 2 + 12, b = i % 2),
                    0,
                    wires::INODE_RCLK[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("INT_NODE_IMUX_{a}_INT_OUT{b}", a = i / 2, b = i % 2),
                    1,
                    wires::INODE_RCLK[i],
                );
            }
        }
        const RCLK_GND_SWIZZLE: [(usize, usize); 24] = [
            // absolutely not the "correct" swizzle, whatever it would even mean.
            (1, 12),
            (2, 13),
            (3, 14),
            (4, 15),
            (5, 16),
            (17, 0),
            (18, 6),
            (19, 7),
            (20, 8),
            (21, 9),
            (22, 10),
            (23, 11),
            (27, 36),
            (28, 37),
            (29, 38),
            (30, 39),
            (40, 24),
            (41, 25),
            (42, 26),
            (43, 31),
            (44, 32),
            (45, 33),
            (46, 34),
            (47, 35),
        ];
        for (i, (iw, ie)) in RCLK_GND_SWIZZLE.into_iter().enumerate() {
            let w = wires::RCLK_GND[i];
            for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
                self.builder
                    .extra_name_tile_cell(tkn, format!("GND_WIRE{iw}"), 0, w);
                self.builder
                    .extra_name_tile_cell(tkn, format!("GND_WIRE{ie}"), 1, w);
            }
        }
    }

    fn fill_wires(&mut self) {
        // common wires

        self.builder.wire_names(wires::TIE_1, &["VCC_WIRE"]);

        for i in 0..16 {
            self.builder
                .wire_names(wires::GCLK[i], &[format!("GCLK_B_0_{i}")]);
            for tkn in ["RCLK_INT_L", "RCLK_INT_R"] {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!(
                        "CLK_LEAF_SITES_{idx}_CLK_LEAF",
                        idx = BUFCE_LEAF_SWIZZLE[0][i]
                    ),
                    2,
                    wires::GCLK[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!(
                        "CLK_LEAF_SITES_{idx}_CLK_IN",
                        idx = BUFCE_LEAF_SWIZZLE[1][i]
                    ),
                    0,
                    wires::IMUX_BUFCE_LEAF[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!(
                        "CLK_LEAF_SITES_{idx}_CLK_IN",
                        idx = BUFCE_LEAF_SWIZZLE[0][i]
                    ),
                    2,
                    wires::IMUX_BUFCE_LEAF[i],
                );
            }
        }

        for i in 0..16 {
            for j in 0..2 {
                self.builder.wire_names(
                    wires::GNODE[i * 2 + j],
                    &[format!("INT_NODE_GLOBAL_{i}_INT_OUT{j}")],
                );
            }
        }

        self.fill_wires_long();

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

        self.fill_wires_sdqnode();
        self.fill_wires_sdq();
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
                if wf == wires::X2_W2[0] {
                    twf = (
                        CellSlotId::from_idx(cell.to_idx() ^ 1),
                        [wires::X2_W2_E0, wires::X2_W2_W0][cell.to_idx()],
                    );
                }
                if wf == wires::X1_W1[7] {
                    twf = (
                        CellSlotId::from_idx(cell.to_idx() ^ 1),
                        [wires::X1_W1_E7, wires::X1_W1_W7][cell.to_idx()],
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

    fn fill_io_term_short(&mut self, xy_w: Coord, xy_e: Coord) {
        let mut e2w = EntityPartVec::new();
        let mut w2e = EntityPartVec::new();
        let pass_w = &self.builder.db[ccls::PASS_W];
        for (wt, &ti) in &pass_w.wires {
            let ConnectorWire::Pass(wf) = ti else {
                unreachable!()
            };
            w2e.insert(wf, wt);
            e2w.insert(wt, wf);
        }
        let pass_e = &self.builder.db[ccls::PASS_E];
        for (wt, &ti) in &pass_e.wires {
            let ConnectorWire::Pass(wf) = ti else {
                unreachable!()
            };
            e2w.insert(wf, wt);
            w2e.insert(wt, wf);
        }
        let switch_tile = [w2e, e2w];
        for (ccid_pass, ccid_io, xy_to, xy_from, cell_to, cell_from) in [
            (ccls::PASS_W, ccls::IO_W, xy_e, xy_w, 0, 1),
            (ccls::PASS_E, ccls::IO_E, xy_w, xy_e, 1, 0),
        ] {
            let pass = &self.builder.db[ccid_pass];
            let naming =
                &self.builder.ndb.tile_class_namings[self.builder.ndb.get_tile_class_naming("INT")];
            let mut node2target = BTreeMap::new();
            for &ti in pass.wires.values() {
                let ConnectorWire::Pass(wf) = ti else {
                    unreachable!()
                };
                let name = if let Some(name) = naming.wires.get(&wf.cell(cell_from)) {
                    name
                } else if let Some(&owf) = switch_tile[cell_from].get(wf) {
                    if let Some(name) = naming.wires.get(&owf.cell(cell_from ^ 1)) {
                        name
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(xy_from, &name.name);
                assert!(node2target.insert(node, wf).is_none());
            }
            let mut wires = EntityPartVec::new();
            for wt in pass.wires.ids() {
                let name = if let Some(name) = naming.wires.get(&wt.cell(cell_to)) {
                    name
                } else if let Some(&owt) = switch_tile[cell_to].get(wt) {
                    if let Some(name) = naming.wires.get(&owt.cell(cell_to ^ 1)) {
                        name
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(xy_to, &name.name);
                if let Some(&wf) = node2target.get(&node) {
                    wires.insert(wt, ConnectorWire::Pass(wf));
                }
            }
            self.builder.insert_connector(ccid_io, wires);
        }
    }

    fn fill_io_term_long(&mut self, xy_w: Coord, xy_e: Coord) {
        for (ccid_pass, ccid_io, xy_to, xy_from) in [
            (ccls::PASS_LW, ccls::IO_LW, xy_e, xy_w),
            (ccls::PASS_LE, ccls::IO_LE, xy_w, xy_e),
        ] {
            let pass = &self.builder.db[ccid_pass];
            let naming =
                &self.builder.ndb.tile_class_namings[self.builder.ndb.get_tile_class_naming("INT")];
            let mut node2target = BTreeMap::new();
            for &ti in pass.wires.values() {
                let ConnectorWire::Pass(wf) = ti else {
                    unreachable!()
                };
                let Some(name) = naming.wires.get(&wf.cell(0)) else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(xy_from, &name.name);
                assert!(node2target.insert(node, wf).is_none());
            }
            let mut wires = EntityPartVec::new();
            for wt in pass.wires.ids() {
                let Some(name) = naming.wires.get(&wt.cell(0)) else {
                    continue;
                };
                let node = self.builder.rd.lookup_wire_force(xy_to, &name.name);
                if let Some(&wf) = node2target.get(&node) {
                    wires.insert(wt, ConnectorWire::Pass(wf));
                }
            }
            self.builder.insert_connector(ccid_io, wires);
        }
    }

    fn fill_terms(&mut self) {
        for tkn in ["INT_TERM_B", "INT_TERM_P", "INT_INT_TERM_H_FT"] {
            for &xy in self.builder.rd.tiles_by_kind_name(tkn) {
                let int_xy = self.builder.walk_to_int(xy, Dir::N, true).unwrap();
                self.extract_sn_term(DirV::S, int_xy);
            }
        }
        for tkn in ["INT_TERM_T"] {
            for &xy in self.builder.rd.tiles_by_kind_name(tkn) {
                let int_xy = self.builder.walk_to_int(xy, Dir::S, true).unwrap();
                self.extract_sn_term(DirV::N, int_xy);
            }
        }

        for &xy in self.builder.rd.tiles_by_kind_name("INT_IBRK_FSR2IO") {
            let Some(xy_w) = self.builder.walk_to_int(xy, Dir::W, true) else {
                continue;
            };
            let Some(xy_e) = self.builder.walk_to_int(xy, Dir::E, true) else {
                continue;
            };
            self.fill_io_term_short(xy_w, xy_e);
            self.fill_io_term_long(xy_w, xy_e);
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
            for &xy in self.builder.rd.tiles_by_kind_name(tkn) {
                let mut bels = vec![];
                for (ud, slots) in [('D', bslots::BUFCE_LEAF_S), ('U', bslots::BUFCE_LEAF_N)] {
                    for i in 0..16 {
                        let mut bel = self
                            .builder
                            .bel_xy(
                                slots[i],
                                "BUFCE_LEAF",
                                i & 7,
                                i / 8 + 2 * usize::from(ud == 'U'),
                            )
                            .pin_rename("CE_INT", "CE")
                            .pins_name_only(&["CLK_CASC_OUT"]);
                        if i != 0 || ud == 'U' {
                            bel = bel.pins_name_only(&["CLK_CASC_IN"]);
                        }
                        bels.push(bel);
                    }
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
            (tcls::INTF, "INTF_W", Dir::W, "INT_INTF_L", false),
            (tcls::INTF, "INTF_E", Dir::E, "INT_INTF_R", false),
            (
                tcls::INTF_IO,
                "INTF_PSS",
                Dir::W,
                "INT_INTF_LEFT_TERM_PSS",
                true,
            ),
            (
                tcls::INTF_IO,
                "INTF_W_IO",
                Dir::W,
                "INT_INTF_LEFT_TERM_IO_FT",
                true,
            ),
            (tcls::INTF_IO, "INTF_W_IO", Dir::W, "INT_INTF_L_CMT", true),
            (tcls::INTF_IO, "INTF_W_IO", Dir::W, "INT_INTF_L_IO", true),
            (
                tcls::INTF_IO,
                "INTF_E_IO",
                Dir::E,
                "INT_INTF_RIGHT_TERM_IO",
                true,
            ),
            (
                tcls::INTF_IO,
                "INTF_E_IO",
                Dir::E,
                "INT_INTF_RIGHT_TERM_XP5IO_FT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_W_PCIE",
                Dir::W,
                "INT_INTF_L_PCIE4",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_PCIE",
                Dir::E,
                "INT_INTF_R_PCIE4",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_W_GT",
                Dir::W,
                "INT_INTF_L_TERM_GT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_GT",
                Dir::E,
                "INT_INTF_R_TERM_GT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_GT",
                Dir::E,
                "INT_INTF_20_2_RIGHT_TERM_GT_FT",
                true,
            ),
            (
                tcls::INTF_DELAY,
                "INTF_E_GT",
                Dir::E,
                "INT_INTF_RIGHT_TERM_HDIO_FT",
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
            ("CLEM", tcls::CLEM, "CLEM", Dir::W),
            ("CLEM_R", tcls::CLEM, "CLEM", Dir::W),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = xy.delta(if side == Dir::W { 1 } else { -1 }, 0);
                let bel = self
                    .builder
                    .bel_xy(bslots::SLICE, "SLICE", 0, 0)
                    .pin_name_only("CIN", 1)
                    .pin_name_only("COUT", 0);
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .ref_int_side(int_xy, side, 0)
                    .bel(bel)
                    .extract();
            }
        }
    }

    fn fill_tiles_lag(&mut self) {
        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("LAG_LAG").iter().next() {
            let mut bels = vec![];
            for i in 0..4 {
                let mut bel = self
                    .builder
                    .bel_xy(bslots::LAGUNA[i], "LAGUNA", i >> 1, i & 1)
                    .extra_wire("VCC", &["VCC_WIRE"]);
                for j in 0..6 {
                    bel = bel
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
                bels.push(bel);
            }
            self.builder
                .xtile_id(tcls::LAGUNA, "LAGUNA", xy)
                .ref_int_side(xy.delta(2, 0), Dir::W, 0)
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
                .pin_rename("TSTRINGEN_X", "TSTRINGEN")
                .pin_rename("TSTRINGSTART_X", "TSTRINGSTART")
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
            "RCLK_BRAM_INTF_L",
            "RCLK_BRAM_INTF_TD_L",
            "RCLK_BRAM_INTF_TD_R",
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let intf = self.builder.ndb.get_tile_class_naming("INTF_W");
                let mut bels = vec![];
                for (i, x, y) in [(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1)] {
                    bels.push(self.builder.bel_xy(bslots::HARD_SYNC[i], "HARD_SYNC", x, y));
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
                let mut bel = self.builder.bel_xy(bslots::DSP[i], "DSP48E2", 0, i);
                let buf_cnt = match i {
                    0 => 1,
                    1 => 0,
                    _ => unreachable!(),
                };
                bel = bel.pin_name_only("MULTSIGNIN", buf_cnt);
                bel = bel.pin_name_only("MULTSIGNOUT", 0);
                bel = bel.pin_name_only("CARRYCASCIN", buf_cnt);
                bel = bel.pin_name_only("CARRYCASCOUT", 0);
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

    fn fill_tiles_bli(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("BLI_BLI_FT")
            .iter()
            .next()
        {
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E");
            let bels = [
                self.builder
                    .bel_xy(bslots::BLI_HBM_APB_INTF, "BLI_HBM_APB_INTF", 0, 0),
                self.builder
                    .bel_xy(bslots::BLI_HBM_AXI_INTF, "BLI_HBM_AXI_INTF", 0, 0),
            ];
            let mut xn = self.builder.xtile_id(tcls::HBM_BLI, "HBM_BLI", xy);
            for i in 0..15 {
                xn = xn
                    .ref_int_side(xy.delta(-2, i as i32), Dir::E, i)
                    .ref_single(xy.delta(-1, i as i32), i, intf);
            }
            xn.bels(bels).extract();
        }
    }

    fn fill_tiles_uram(&mut self) {
        for tkn in ["URAM_URAM_FT", "URAM_URAM_DELAY_FT"] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E");
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W");
                let mut bels = vec![];
                for i in 0..4 {
                    let mut bel = self.builder.bel_xy(bslots::URAM[i], "URAM288", 0, i);
                    let buf_cnt = match i {
                        0 => 1,
                        _ => 0,
                    };
                    for ab in ['A', 'B'] {
                        for j in 0..23 {
                            bel = bel.pin_name_only(&format!("CAS_IN_ADDR_{ab}{j}"), buf_cnt);
                            bel = bel.pin_name_only(&format!("CAS_OUT_ADDR_{ab}{j}"), 0);
                        }
                        for j in 0..9 {
                            bel = bel.pin_name_only(&format!("CAS_IN_BWE_{ab}{j}"), buf_cnt);
                            bel = bel.pin_name_only(&format!("CAS_OUT_BWE_{ab}{j}"), 0);
                        }
                        for j in 0..72 {
                            bel = bel.pin_name_only(&format!("CAS_IN_DIN_{ab}{j}"), buf_cnt);
                            bel = bel.pin_name_only(&format!("CAS_OUT_DIN_{ab}{j}"), 0);
                            bel = bel.pin_name_only(&format!("CAS_IN_DOUT_{ab}{j}"), buf_cnt);
                            bel = bel.pin_name_only(&format!("CAS_OUT_DOUT_{ab}{j}"), 0);
                        }
                        for pin in ["EN", "RDACCESS", "RDB_WR", "DBITERR", "SBITERR"] {
                            bel = bel.pin_name_only(&format!("CAS_IN_{pin}_{ab}"), buf_cnt);
                            bel = bel.pin_name_only(&format!("CAS_OUT_{pin}_{ab}"), 0);
                        }
                    }
                    bels.push(bel);
                }
                let mut xn = self.builder.xtile_id(tcls::URAM, "URAM", xy);
                for i in 0..15 {
                    xn = xn
                        .ref_int_side(xy.delta(-2, i as i32), Dir::E, i)
                        .ref_single(xy.delta(-1, i as i32), i, intf_e);
                }
                for i in 0..15 {
                    xn = xn
                        .ref_int_side(xy.delta(2, i as i32), Dir::W, i + 15)
                        .ref_single(xy.delta(1, i as i32), i + 15, intf_w);
                }
                xn.bels(bels).extract();
            }
        }
    }

    fn fill_tiles_hard(&mut self) {
        for (slot, tcid, naming, tkn, bk) in [
            (
                bslots::PCIE4,
                tcls::PCIE4,
                "PCIE4",
                "PCIE4_PCIE4_FT",
                "PCIE40E4",
            ),
            (
                bslots::PCIE4,
                tcls::PCIE4C,
                "PCIE4C",
                "PCIE4C_PCIE4C_FT",
                "PCIE4CE4",
            ),
            (
                bslots::PCIE4,
                tcls::PCIE4CE,
                "PCIE4CE",
                "PCIE4CE_PCIE4CE_FT",
                "PCIE4CE",
            ),
            (bslots::CMAC, tcls::CMAC, "CMAC", "CMAC", "CMACE4"),
            (bslots::ILKN, tcls::ILKN, "ILKN", "ILKN_ILKN_FT", "ILKNE4"),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_w_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let int_e_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W_PCIE");
                let mut bel = self.builder.bel_xy(slot, bk, 0, 0);
                let mut naming = naming;
                if tcid == tcls::PCIE4 {
                    let mut has_mcap = false;
                    if let Some(wire) = self
                        .builder
                        .rd
                        .wires
                        .get("PCIE4_PCIE4_CORE_0_MCAP_PERST0_B_PIN")
                    {
                        let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                        if tk.wires.contains_key(&wire) {
                            has_mcap = true;
                        }
                    }
                    if has_mcap {
                        bel = bel
                            .pin_name_only("MCAP_PERST0_B", 1)
                            .pin_name_only("MCAP_PERST1_B", 1);
                    } else {
                        bel = bel
                            .pin_name_only("MCAP_PERST0_B", 0)
                            .pin_name_only("MCAP_PERST1_B", 0);
                        naming = "PCIE4_NOCFG";
                    }
                } else if tcid == tcls::PCIE4C {
                    let mut has_mcap = false;
                    if let Some(wire) = self.builder.rd.wires.get("PCIE4C_CORE_0_MCAP_PERST0_B_PIN")
                    {
                        let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                        if tk.wires.contains_key(&wire) {
                            has_mcap = true;
                        }
                    }
                    if has_mcap {
                        bel = bel
                            .pin_name_only("MCAP_PERST0_B", 1)
                            .pin_name_only("MCAP_PERST1_B", 1);
                    } else {
                        bel = bel
                            .pin_name_only("MCAP_PERST0_B", 0)
                            .pin_name_only("MCAP_PERST1_B", 0);
                        naming = "PCIE4C_NOCFG";
                    }
                } else if tcid == tcls::PCIE4CE {
                    bel = bel
                        .pin_name_only("MCAP_PERST0_B", 0)
                        .pin_name_only("MCAP_PERST1_B", 0);
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

        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("FE_FE_FT").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_W_PCIE");
            let mut bel = self.builder.bel_xy(bslots::FE, "FE", 0, 0);
            for (prefix, num) in [
                ("XIL_UNCONN_CLK", 470),
                ("XIL_UNCONN_IN", 1862),
                ("XIL_UNCONN_OUT", 711),
            ] {
                for i in 0..num {
                    bel = bel.pins_name_only(&[format!("{prefix}{i}")]);
                }
            }
            let mut xn = self.builder.xtile_id(tcls::FE, "FE", xy);
            for i in 0..60 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_xy.delta(-1, (i + i / 30) as i32), i, intf);
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_dfe(&mut self) {
        let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
        let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W_PCIE");
        for (bslots, tcid, naming, tkn, bk) in [
            (
                [
                    (bslots::DFE_FFT[0], "FFT_INST_0_"),
                    (bslots::DFE_FFT[1], "FFT_INST_1_"),
                    (bslots::DFE_PRACH, "PRACH_INST_0_"),
                ]
                .as_slice(),
                tcls::DFE_A,
                "DFE_A",
                "DFE_DFE_TILEA_FT",
                "DFE_A",
            ),
            (
                [
                    (bslots::DFE_FFT[0], "FFT_INST_0_"),
                    (bslots::DFE_FFT[1], "FFT_INST_1_"),
                    (bslots::DFE_FIR[0], "CHANNEL_FILTER_INST_0_"),
                    (bslots::DFE_FIR[1], "CHANNEL_FILTER_INST_1_"),
                ]
                .as_slice(),
                tcls::DFE_B,
                "DFE_B",
                "DFE_DFE_TILEB_FT",
                "DFE_B",
            ),
            (
                [
                    (bslots::DFE_MATRIX_FIR[0], "COMPLEX_EQ_INST_0_"),
                    (bslots::DFE_MATRIX_FIR[1], "COMPLEX_EQ_INST_1_"),
                    (bslots::DFE_DUC_DDC[0], "DUC_DDC_AND_MIXER_INST_0_"),
                    (bslots::DFE_DUC_DDC[1], "DUC_DDC_AND_MIXER_INST_1_"),
                ]
                .as_slice(),
                tcls::DFE_C,
                "DFE_C",
                "DFE_DFE_TILEC_FT",
                "DFE_C",
            ),
            (
                [
                    (bslots::DFE_FFT[0], "FFT_INST_0_"),
                    (bslots::DFE_CFR[0], "CFR_INST_0_"),
                    (bslots::DFE_CFR[1], "CFR_INST_1_"),
                    (bslots::DFE_PQ_INTERP[0], "RESAMPLER_DL_TOP_INST_0_"),
                ]
                .as_slice(),
                tcls::DFE_D,
                "DFE_D",
                "DFE_DFE_TILED_FT",
                "DFE_D",
            ),
            (
                [
                    (bslots::DFE_NL_FIR[0], "NLF_INST_0_"),
                    (bslots::DFE_NL_FIR[1], "NLF_INST_1_"),
                    (bslots::DFE_NL_FIR[2], "NLF_INST_2_"),
                ]
                .as_slice(),
                tcls::DFE_E,
                "DFE_E",
                "DFE_DFE_TILEE_FT",
                "DFE_E",
            ),
            (
                [
                    (bslots::DFE_NL_FIR[0], "NLF_INST_0_"),
                    (bslots::DFE_PQ_INTERP[0], "RESAMPLER_DL_TOP_INST_0_"),
                    (bslots::DFE_PQ_INTERP[1], "RESAMPLER_DL_TOP_INST_1_"),
                    (bslots::DFE_PQ_INTERP[2], "RESAMPLER_DL_TOP_INST_2_"),
                    (bslots::DFE_PQ_INTERP[3], "RESAMPLER_DL_TOP_INST_3_"),
                ]
                .as_slice(),
                tcls::DFE_F,
                "DFE_F",
                "DFE_DFE_TILEF_FT",
                "DFE_F",
            ),
            (
                [
                    (bslots::DFE_PQ_DECIM[0], "RESAMPLER_UL_TOP_INST_0_"),
                    (bslots::DFE_PQ_DECIM[1], "RESAMPLER_UL_TOP_INST_1_"),
                    (bslots::DFE_PQ_DECIM[2], "RESAMPLER_UL_TOP_INST_2_"),
                    (bslots::DFE_PQ_DECIM[3], "RESAMPLER_UL_TOP_INST_3_"),
                    (bslots::DFE_PQ_DECIM[4], "RESAMPLER_UL_TOP_INST_4_"),
                ]
                .as_slice(),
                tcls::DFE_G,
                "DFE_G",
                "DFE_DFE_TILEG_FT",
                "DFE_G",
            ),
        ] {
            let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() else {
                continue;
            };
            let int_w_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let int_e_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
            let bel = self.builder.bel_xy(bslots[0].0, bk, 0, 0).manual();
            let mut xt = self.builder.xtile_id(tcid, naming, xy);
            for i in 0..60 {
                xt = xt
                    .ref_int_side(int_w_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                    .ref_single(int_w_xy.delta(1, (i + i / 30) as i32), i, intf_w);
                if tcid != tcls::DFE_B {
                    xt = xt
                        .ref_int_side(int_e_xy.delta(0, (i + i / 30) as i32), Dir::W, i + 60)
                        .ref_single(int_e_xy.delta(-1, (i + i / 30) as i32), i + 60, intf_e);
                }
            }
            let mut xt = xt.bel(bel).extract();
            let (orig_bel, orig_naming) = xt.bels.pop().unwrap();
            for &(bslot, infix) in bslots {
                let mut bel = LegacyBel::default();
                let mut bnaming = orig_naming.clone();
                for (orig_pname, pin) in &orig_bel.pins {
                    let Some((pre, post)) = orig_pname.split_once(infix) else {
                        continue;
                    };
                    let pname = format!("{pre}{post}");
                    bel.pins.insert(pname.clone(), pin.clone());
                    let pn = orig_naming.pins[orig_pname].clone();
                    bnaming.pins.insert(pname, pn);
                }
                self.builder
                    .insert_tcls_bel(tcid, bslot, BelInfo::Legacy(bel));
                self.builder.insert_bel_naming(naming, bslot, bnaming);
            }
        }
    }

    fn fill_tiles_ps(&mut self) {
        for i in 0..18 {
            self.builder
                .extra_name(format!("PS_TO_PL_CLK{i}"), wires::OUT_PS_CLK[i]);
        }
        for tkn in [
            "RCLK_INTF_LEFT_TERM_ALTO",
            "RCLK_RCLK_INTF_LEFT_TERM_DA6_FT",
            "RCLK_INTF_LEFT_TERM_DA7",
            "RCLK_RCLK_INTF_LEFT_TERM_DA8_FT",
            "RCLK_RCLK_INTF_LEFT_TERM_DC12_FT",
            "RCLK_RCLK_INTF_LEFT_TERM_MX8_FT",
        ] {
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!(
                        "VCC_WIRE{ii}",
                        ii = [
                            0, 3, 8, 9, 10, 11, 13, 14, 15, 16, 1, 12, 17, 18, 19, 20, 21, 22, 23,
                            2, 4, 5, 6, 7
                        ][i]
                    ),
                    wires::IMUX_BUFG_PS_DUMMY[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFG_PS_{i}_CLK_IN"),
                    wires::IMUX_BUFG_PS[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFG_PS_{i}_CLK_OUT"),
                    wires::OUT_BUFG_PS[i],
                );
                self.builder
                    .extra_name_tile(tkn, format!("CLK_HROUTE{i}"), wires::GCLK_HROUTE[i]);
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let mut bels = vec![];
                for i in 0..24 {
                    bels.push(self.builder.bel_xy(bslots::BUFG_PS[i], "BUFG_PS", 0, i));
                }
                self.builder
                    .xtile_id(tcls::RCLK_PS, "RCLK_PS", xy)
                    .ref_xlat(xy.delta(1, 0), &[Some(0), None, None, None], rclk_int)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&wires::IMUX_BUFG_PS[..])
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .bels(bels)
                    .extract();
            }
        }

        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("PSS_ALTO").iter().next() {
            let int_r_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_PSS");
            let rclk_ps = self.builder.ndb.get_tile_class_naming("RCLK_PS");
            let mut bel = self
                .builder
                .bel_xy(bslots::PS, "PS8", 0, 0)
                .pins_name_only(&[
                    "BSCAN_RESET_TAP_B",
                    "BSCAN_CLOCKDR",
                    "BSCAN_SHIFTDR",
                    "BSCAN_UPDATEDR",
                    "BSCAN_INTEST",
                    "BSCAN_EXTEST",
                    "BSCAN_INIT_MEMORY",
                    "BSCAN_AC_TEST",
                    "BSCAN_AC_MODE",
                    "BSCAN_MISR_JTAG_LOAD",
                    "PSS_CFG_RESET_B",
                    "PSS_FST_CFG_B",
                    "PSS_GTS_CFG_B",
                    "PSS_GTS_USR_B",
                    "PSS_GHIGH_B",
                    "PSS_GPWRDWN_B",
                    "PCFG_POR_B",
                ]);
            for pin in [
                "IDCODE15",
                "IDCODE16",
                "IDCODE17",
                "IDCODE18",
                "IDCODE20",
                "IDCODE21",
                "IDCODE28",
                "IDCODE29",
                "IDCODE30",
                "IDCODE31",
                "PS_VERSION_0",
                "PS_VERSION_1",
                "PS_VERSION_2",
                "PS_VERSION_3",
            ] {
                bel = bel.pin_dummy(pin);
            }
            let mut xn = self
                .builder
                .xtile_id(tcls::PS, "PS", xy)
                .ref_single(int_r_xy.delta(-1, 30), 30, rclk_ps)
                .ref_single(int_r_xy.delta(-1, 92), 90, rclk_ps)
                .ref_single(int_r_xy.delta(-1, 154), 150, rclk_ps);
            for i in 0..180 {
                xn = xn
                    .ref_int_side(int_r_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_r_xy.delta(-1, (i + i / 30) as i32), i, intf);
            }
            xn.bel(bel).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("VCU_VCU_FT")
            .iter()
            .next()
        {
            let int_r_xy = self
                .builder
                .walk_to_int(xy.delta(0, 2), Dir::E, false)
                .unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_PSS");
            let rclk_ps = self.builder.ndb.get_tile_class_naming("RCLK_PS");
            let bel = self.builder.bel_xy(bslots::VCU, "VCU", 0, 0);
            let mut xn = self.builder.xtile_id(tcls::VCU, "VCU", xy).ref_single(
                int_r_xy.delta(-1, 30),
                30,
                rclk_ps,
            );
            for i in 0..60 {
                xn = xn
                    .ref_int_side(int_r_xy.delta(0, (i + i / 30) as i32), Dir::W, i)
                    .ref_single(int_r_xy.delta(-1, (i + i / 30) as i32), i, intf);
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_cfg(&mut self) {
        for (tcid, naming, tkn, bkind) in [
            (tcls::CFG, "CFG", "CFG_CONFIG", "CONFIG_SITE"),
            (tcls::CFG_CSEC, "CFG_CSEC", "CSEC_CONFIG_FT", "CSEC_SITE"),
            (
                tcls::CFG_CSEC_V2,
                "CFG_CSEC_V2",
                "CSEC_CONFIG_VER2_FT",
                "CSEC_SITE",
            ),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let mut bel_cfg = self.builder.bel_xy(bslots::CFG, bkind, 0, 0);
                if tcid != tcls::CFG {
                    bel_cfg = bel_cfg.pin_rename("KEY_CLEAR_B", "KEY_CLEAR");
                }
                let bels = [
                    bel_cfg,
                    self.builder
                        .bel_xy(bslots::ABUS_SWITCH_CFG, "ABUS_SWITCH", 0, 0)
                        .pins_name_only(&["TEST_ANALOGBUS_SEL_B"]),
                ];
                let mut xn = self.builder.xtile_id(tcid, naming, xy);
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
                }
                xn.bels(bels).extract();
            }
        }

        for tkn in ["CFGIO_IOB20", "CFGIOLC_IOB20_FT"] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let bels = [
                    self.builder.bel_xy(bslots::PMV, "PMV", 0, 0),
                    self.builder.bel_xy(bslots::PMV2, "PMV2", 0, 0),
                    self.builder.bel_xy(bslots::PMVIOB, "PMVIOB", 0, 0),
                    self.builder.bel_xy(bslots::MTBF3, "MTBF3", 0, 0),
                    self.builder.bel_xy(bslots::CFGIO, "CFGIO_SITE", 0, 0),
                ];
                let mut xn = self.builder.xtile_id(tcls::CFGIO, "CFGIO", xy);
                for i in 0..30 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
                }
                xn.bels(bels).extract();
            }
        }

        if let Some(&xy) = self.builder.rd.tiles_by_kind_name("AMS").iter().next() {
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
            let mut bel = self.builder.bel_xy(bslots::SYSMON, "SYSMONE4", 0, 0);
            for i in 0..16 {
                bel = bel
                    .pin_name_only(&format!("VP_AUX{i}"), 1)
                    .pin_name_only(&format!("VN_AUX{i}"), 1);
            }
            let mut xn = self.builder.xtile_id(tcls::AMS, "AMS", xy);
            for i in 0..30 {
                xn = xn
                    .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                    .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
            }
            xn.bel(bel).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("CFRM_CFRAME_TERM_H_FT")
            .iter()
            .next()
        {
            let mut bels = vec![];
            for i in 0..8 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::ABUS_SWITCH_HBM[i], "ABUS_SWITCH", i >> 1, i & 1)
                        .pins_name_only(&["TEST_ANALOGBUS_SEL_B"]),
                );
            }
            self.builder
                .xtile_id(tcls::HBM_ABUS_SWITCH, "HBM_ABUS_SWITCH", xy)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_hdio(&mut self) {
        for i in 0..4 {
            self.builder.extra_name_cell(
                format!("CLK_BUFGCE_HDIO_{i}_CLK_IN"),
                30,
                wires::IMUX_BUFGCE_HDIO[i],
            );
            self.builder.extra_name_cell(
                format!("CLK_BUFGCE_HDIO_{i}_CLK_OUT"),
                30,
                wires::OUT_BUFGCE_HDIO[i],
            );
            for tkn in [
                "RCLK_HDIO",
                "RCLK_RCLK_HDIO_R_FT",
                "RCLK_RCLK_HDIO_LAST_R_FT",
                "RCLK_RCLK_HDIOS_L_FT",
                "RCLK_RCLK_HDIOS_CMAC_L_FT",
                "RCLK_RCLK_HDIOL_L_FT",
                "RCLK_RCLK_HDIOL_MRC_L_FT",
                "RCLK_RCLK_HDIOL_R_FT",
                "RCLK_RCLK_HDIOL_MRC_R_FT",
            ] {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_4TO1_{i}_CLK_OUT"),
                    30,
                    wires::IMUX_BUFGCE_HDIO_CCIO[i],
                );
            }
        }
        {
            if let Some(&xy) = self
                .builder
                .rd
                .tiles_by_kind_name("HDIO_BOT_RIGHT")
                .iter()
                .next()
            {
                let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let mut bels = vec![];
                // 12, 14, 8, 10
                for i in 0..12 {
                    let mut bel_iob0 = self
                        .builder
                        .bel_xy(bslots::HDIOB[2 * i], "IOB", 0, 2 * (i % 6))
                        .raw_tile(i / 6)
                        .pins_name_only(&[
                            "OP",
                            "TSP",
                            "O_B",
                            "TSTATEB",
                            "OUTB_B",
                            "OUTB_B_IN",
                            "TSTATE_IN",
                            "TSTATE_OUT",
                            "LVDS_TRUE",
                            "PAD_RES",
                            "I",
                        ])
                        .pin_name_only("SWITCH_OUT", 1)
                        .pin_dummy("IO");
                    if let Some(idx) = match i {
                        6 => Some(0),
                        7 => Some(1),
                        4 => Some(2),
                        5 => Some(3),
                        _ => None,
                    } {
                        bel_iob0 = bel_iob0.extra_int_out_force(
                            "CLKPAD",
                            wires::OUT_IOB_CLKPAD[idx].cell(30),
                            [
                                "HDIO_IOBPAIR_18_I",
                                "HDIO_IOBPAIR_25_I",
                                "HDIO_IOBPAIR_46_I",
                                "HDIO_IOBPAIR_53_I",
                            ][idx],
                        );
                    }
                    bels.extend([
                        bel_iob0,
                        self.builder
                            .bel_xy(bslots::HDIOB[2 * i + 1], "IOB", 0, 2 * (i % 6) + 1)
                            .raw_tile(i / 6)
                            .pins_name_only(&[
                                "OP",
                                "TSP",
                                "O_B",
                                "TSTATEB",
                                "OUTB_B",
                                "OUTB_B_IN",
                                "TSTATE_IN",
                                "TSTATE_OUT",
                                "LVDS_TRUE",
                                "PAD_RES",
                                "I",
                            ])
                            .pin_name_only("SWITCH_OUT", 1)
                            .pin_dummy("IO"),
                        self.builder
                            .bel_xy(bslots::HDIOB_DIFF_IN[i], "HDIOBDIFFINBUF", 0, i % 6)
                            .raw_tile(i / 6)
                            .pins_name_only(&["LVDS_TRUE", "LVDS_COMP", "PAD_RES_0", "PAD_RES_1"]),
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i], "HDIOLOGIC_M", 0, i % 6)
                            .raw_tile(i / 6)
                            .pin_rename("IPFFM_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFM_CE", "IPFF_CE")
                            .pin_rename("IPFFM_CK", "IPFF_CK")
                            .pin_rename("IPFFM_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFM_Q1", "IPFF_Q1")
                            .pin_rename("IPFFM_Q2", "IPFF_Q2")
                            .pin_rename("IPFFM_RST", "IPFF_RST")
                            .pin_rename("OPFFM_CE", "OPFF_CE")
                            .pin_rename("OPFFM_CK", "OPFF_CK")
                            .pin_rename("OPFFM_D1", "OPFF_D1")
                            .pin_rename("OPFFM_D2", "OPFF_D2")
                            .pin_rename("OPFFM_RST", "OPFF_RST")
                            .pin_rename("TFFM_D1", "TFF_D1")
                            .pin_rename("TFFM_D2", "TFF_D2")
                            .pins_name_only(&["OPFFM_Q", "TFFM_Q", "IPFFM_D"]),
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i + 1], "HDIOLOGIC_S", 0, i % 6)
                            .raw_tile(i / 6)
                            .pin_rename("IPFFS_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFS_CE", "IPFF_CE")
                            .pin_rename("IPFFS_CK", "IPFF_CK")
                            .pin_rename("IPFFS_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFS_Q1", "IPFF_Q1")
                            .pin_rename("IPFFS_Q2", "IPFF_Q2")
                            .pin_rename("IPFFS_RST", "IPFF_RST")
                            .pin_rename("OPFFS_CE", "OPFF_CE")
                            .pin_rename("OPFFS_CK", "OPFF_CK")
                            .pin_rename("OPFFS_D1", "OPFF_D1")
                            .pin_rename("OPFFS_D2", "OPFF_D2")
                            .pin_rename("OPFFS_RST", "OPFF_RST")
                            .pin_rename("TFFS_D1", "TFF_D1")
                            .pin_rename("TFFS_D2", "TFF_D2")
                            .pins_name_only(&["OPFFS_Q", "TFFS_Q", "IPFFS_D"]),
                    ]);
                }
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDLOGIC_CSSD[i], "HDLOGIC_CSSD", 0, 0)
                            .raw_tile(i),
                    );
                }
                bels.extend([
                    self.builder.bel_xy(bslots::HDIO_VREF[0], "HDIO_VREF", 0, 0),
                    self.builder
                        .bel_xy(bslots::HDIO_BIAS, "HDIO_BIAS", 0, 0)
                        .raw_tile(1),
                ]);
                let mut xn = self
                    .builder
                    .xtile_id(tcls::HDIO, "HDIO", xy)
                    .raw_tile(xy.delta(0, 31));
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
                }
                xn.bels(bels).extract();
            }
        }

        for (tcid, tkn, side) in [
            (tcls::HDIOL_S, "HDIOLC_HDIOL_BOT_LEFT_FT", Dir::W),
            (tcls::HDIOL_N, "HDIOLC_HDIOL_TOP_LEFT_FT", Dir::W),
            (tcls::HDIOL_S, "HDIOLC_HDIOL_BOT_RIGHT_CFG_FT", Dir::E),
            (tcls::HDIOL_N, "HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", Dir::E),
            (tcls::HDIOL_S, "HDIOLC_HDIOL_BOT_RIGHT_AUX_FT", Dir::E),
            (tcls::HDIOL_N, "HDIOLC_HDIOL_TOP_RIGHT_AUX_FT", Dir::E),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, !side, false).unwrap();
                let intf_xy = int_xy.delta(if side == Dir::W { -1 } else { 1 }, 0);
                let intf_tile = &self.rd.tiles[&intf_xy];
                let intf = self.builder.ndb.get_tile_class_naming(
                    match self.rd.tile_kinds.key(intf_tile.kind).as_str() {
                        "INT_INTF_LEFT_TERM_IO_FT" => "INTF_W_IO",
                        "INT_INTF_RIGHT_TERM_IO" => "INTF_E_IO",
                        "INT_INTF_RIGHT_TERM_HDIO_FT" => "INTF_E_PCIE",
                        x => panic!("ummm {x}"),
                    },
                );
                let mut bels = vec![];
                for i in 0..21 {
                    let mut bel_iob0 = self
                        .builder
                        .bel_xy(bslots::HDIOB[2 * i], "IOB", 0, 2 * i)
                        .pins_name_only(&[
                            "OP",
                            "TSP",
                            "O_B",
                            "TSTATEB",
                            "OUTB_B",
                            "OUTB_B_IN",
                            "TSTATE_IN",
                            "TSTATE_OUT",
                            "LVDS_TRUE",
                            "PAD_RES",
                            "I",
                        ])
                        .pin_name_only("SWITCH_OUT", 0)
                        .pin_dummy("IO");
                    if let Some(idx) = match (tcid, i) {
                        (tcls::HDIOL_N, 0) => Some(0),
                        (tcls::HDIOL_N, 1) => Some(1),
                        (tcls::HDIOL_S, 6) => Some(2),
                        (tcls::HDIOL_S, 5) => Some(3),
                        _ => None,
                    } {
                        bel_iob0 = bel_iob0.extra_int_out_force(
                            "CLKPAD",
                            wires::OUT_IOB_CLKPAD[idx].cell(0),
                            [
                                "HDIOLC_IOBPAIR_52_I",
                                "HDIOLC_IOBPAIR_59_I",
                                "HDIOLC_IOBPAIR_94_I",
                                "HDIOLC_IOBPAIR_87_I",
                            ][idx],
                        );
                    }
                    bels.extend([
                        bel_iob0,
                        self.builder
                            .bel_xy(bslots::HDIOB[2 * i + 1], "IOB", 0, 2 * i + 1)
                            .pins_name_only(&[
                                "OP",
                                "TSP",
                                "O_B",
                                "TSTATEB",
                                "OUTB_B",
                                "OUTB_B_IN",
                                "TSTATE_IN",
                                "TSTATE_OUT",
                                "LVDS_TRUE",
                                "PAD_RES",
                                "I",
                            ])
                            .pin_name_only("SWITCH_OUT", 0)
                            .pin_dummy("IO"),
                    ]);
                }
                for i in 0..21 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDIOB_DIFF_IN[i], "HDIOBDIFFINBUF", 0, i)
                            .pins_name_only(&["LVDS_TRUE", "LVDS_COMP", "PAD_RES_0", "PAD_RES_1"]),
                    );
                }
                for i in 0..21 {
                    bels.extend([
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i], "HDIOLOGIC_M", 0, i)
                            .pin_rename("IPFFM_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFM_CE", "IPFF_CE")
                            .pin_rename("IPFFM_CK", "IPFF_CK")
                            .pin_rename("IPFFM_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFM_Q1", "IPFF_Q1")
                            .pin_rename("IPFFM_Q2", "IPFF_Q2")
                            .pin_rename("IPFFM_RST", "IPFF_RST")
                            .pin_rename("OPFFM_CE", "OPFF_CE")
                            .pin_rename("OPFFM_CK", "OPFF_CK")
                            .pin_rename("OPFFM_D1", "OPFF_D1")
                            .pin_rename("OPFFM_D2", "OPFF_D2")
                            .pin_rename("OPFFM_RST", "OPFF_RST")
                            .pin_rename("TFFM_D1", "TFF_D1")
                            .pin_rename("TFFM_D2", "TFF_D2")
                            .pins_name_only(&["OPFFM_Q", "TFFM_Q", "IPFFM_D"]),
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i + 1], "HDIOLOGIC_S", 0, i)
                            .pin_rename("IPFFS_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFS_CE", "IPFF_CE")
                            .pin_rename("IPFFS_CK", "IPFF_CK")
                            .pin_rename("IPFFS_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFS_Q1", "IPFF_Q1")
                            .pin_rename("IPFFS_Q2", "IPFF_Q2")
                            .pin_rename("IPFFS_RST", "IPFF_RST")
                            .pin_rename("OPFFS_CE", "OPFF_CE")
                            .pin_rename("OPFFS_CK", "OPFF_CK")
                            .pin_rename("OPFFS_D1", "OPFF_D1")
                            .pin_rename("OPFFS_D2", "OPFF_D2")
                            .pin_rename("OPFFS_RST", "OPFF_RST")
                            .pin_rename("TFFS_D1", "TFF_D1")
                            .pin_rename("TFFS_D2", "TFF_D2")
                            .pins_name_only(&["OPFFS_Q", "TFFS_Q", "IPFFS_D"]),
                    ]);
                }
                for i in 0..3 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDLOGIC_CSSD[i], "HDLOGIC_CSSD", 0, i),
                    );
                }
                for i in 0..2 {
                    bels.push(self.builder.bel_xy(bslots::HDIO_VREF[i], "HDIO_VREF", 0, i));
                }
                bels.push(self.builder.bel_xy(bslots::HDIO_BIAS, "HDIO_BIAS", 0, 0));
                let mut xn = self.builder.xtile_id(tcid, tkn, xy);
                for i in 0..30 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), side, i)
                        .ref_single(intf_xy.delta(0, (i + i / 30) as i32), i, intf)
                }
                xn.bels(bels).extract();
            }
        }

        for tkn in [
            "HDIOLC_HDIOS_BOT_LEFT_FT",
            "HDIOLC_HDIOS_BOT_LEFT_CFG_FT",
            "HDIOLC_HDIOS_BOT_LEFT_AUX_FT",
            "HDIOLC_HDIOS_CMAC_BOT_LEFT_FT",
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_w_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let int_e_xy = self.builder.walk_to_int(xy, Dir::E, false).unwrap();
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W_PCIE");
                let mut bels = vec![];
                for i in 0..21 {
                    let mut bel_iob0 = self
                        .builder
                        .bel_xy(bslots::HDIOB[2 * i], "IOB", 0, 2 * (i % 11))
                        .raw_tile(i / 11)
                        .pins_name_only(&[
                            "OP",
                            "TSP",
                            "O_B",
                            "TSTATEB",
                            "OUTB_B",
                            "OUTB_B_IN",
                            "TSTATE_IN",
                            "TSTATE_OUT",
                            "LVDS_TRUE",
                            "PAD_RES",
                            "I",
                        ])
                        .pin_name_only("SWITCH_OUT", 0)
                        .pin_dummy("IO");
                    if let Some(idx) = match i {
                        11 => Some(0),
                        12 => Some(1),
                        4 => Some(2),
                        5 => Some(3),
                        _ => None,
                    } {
                        bel_iob0 = bel_iob0.extra_int_out_force(
                            "CLKPAD",
                            wires::OUT_IOB_CLKPAD[idx].cell(30),
                            [
                                "HDIOLC_IOBPAIR_28_I",
                                "HDIOLC_IOBPAIR_35_I",
                                "HDIOLC_IOBPAIR_58_I",
                                "HDIOLC_IOBPAIR_65_I",
                            ][idx],
                        );
                    }
                    bels.extend([
                        bel_iob0,
                        self.builder
                            .bel_xy(bslots::HDIOB[2 * i + 1], "IOB", 0, 2 * (i % 11) + 1)
                            .raw_tile(i / 11)
                            .pins_name_only(&[
                                "OP",
                                "TSP",
                                "O_B",
                                "TSTATEB",
                                "OUTB_B",
                                "OUTB_B_IN",
                                "TSTATE_IN",
                                "TSTATE_OUT",
                                "LVDS_TRUE",
                                "PAD_RES",
                                "I",
                            ])
                            .pin_name_only("SWITCH_OUT", 0)
                            .pin_dummy("IO"),
                    ]);
                }
                for i in 0..21 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDIOB_DIFF_IN[i], "HDIOBDIFFINBUF", 0, i % 11)
                            .raw_tile(i / 11)
                            .pins_name_only(&["LVDS_TRUE", "LVDS_COMP", "PAD_RES_0", "PAD_RES_1"]),
                    );
                }
                for i in 0..21 {
                    bels.extend([
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i], "HDIOLOGIC_M", 0, i % 11)
                            .raw_tile(i / 11)
                            .pin_rename("IPFFM_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFM_CE", "IPFF_CE")
                            .pin_rename("IPFFM_CK", "IPFF_CK")
                            .pin_rename("IPFFM_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFM_Q1", "IPFF_Q1")
                            .pin_rename("IPFFM_Q2", "IPFF_Q2")
                            .pin_rename("IPFFM_RST", "IPFF_RST")
                            .pin_rename("OPFFM_CE", "OPFF_CE")
                            .pin_rename("OPFFM_CK", "OPFF_CK")
                            .pin_rename("OPFFM_D1", "OPFF_D1")
                            .pin_rename("OPFFM_D2", "OPFF_D2")
                            .pin_rename("OPFFM_RST", "OPFF_RST")
                            .pin_rename("TFFM_D1", "TFF_D1")
                            .pin_rename("TFFM_D2", "TFF_D2")
                            .pins_name_only(&["OPFFM_Q", "TFFM_Q", "IPFFM_D"]),
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[2 * i + 1], "HDIOLOGIC_S", 0, i % 11)
                            .raw_tile(i / 11)
                            .pin_rename("IPFFS_BYPASS", "IPFF_BYPASS")
                            .pin_rename("IPFFS_CE", "IPFF_CE")
                            .pin_rename("IPFFS_CK", "IPFF_CK")
                            .pin_rename("IPFFS_CK_C", "IPFF_CK_C")
                            .pin_rename("IPFFS_Q1", "IPFF_Q1")
                            .pin_rename("IPFFS_Q2", "IPFF_Q2")
                            .pin_rename("IPFFS_RST", "IPFF_RST")
                            .pin_rename("OPFFS_CE", "OPFF_CE")
                            .pin_rename("OPFFS_CK", "OPFF_CK")
                            .pin_rename("OPFFS_D1", "OPFF_D1")
                            .pin_rename("OPFFS_D2", "OPFF_D2")
                            .pin_rename("OPFFS_RST", "OPFF_RST")
                            .pin_rename("TFFS_D1", "TFF_D1")
                            .pin_rename("TFFS_D2", "TFF_D2")
                            .pins_name_only(&["OPFFS_Q", "TFFS_Q", "IPFFS_D"]),
                    ]);
                }
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::HDLOGIC_CSSD[i],
                                if i < 2 {
                                    "HDIOS_HDLOGIC_CSSD"
                                } else {
                                    "HDIOS_HDLOGIC_CSSD_TOP"
                                },
                                0,
                                i % 2,
                            )
                            .raw_tile(i / 2),
                    );
                }
                for i in 0..3 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDIO_VREF[i], "HDIO_VREF", 0, i % 2)
                            .raw_tile(i / 2),
                    );
                }
                bels.push(
                    self.builder
                        .bel_xy(bslots::HDIO_BIAS, "HDIO_BIAS", 0, 0)
                        .raw_tile(1),
                );
                let mut xn = self
                    .builder
                    .xtile_id(tcls::HDIOS, tkn, xy)
                    .raw_tile(xy.delta(0, 31));
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_w_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_single(int_w_xy.delta(1, (i + i / 30) as i32), i, intf_w);
                    xn = xn
                        .ref_int_side(int_e_xy.delta(0, (i + i / 30) as i32), Dir::E, i + 60)
                        .ref_single(int_e_xy.delta(-1, (i + i / 30) as i32), i + 60, intf_e);
                }
                xn.bels(bels).extract();
            }
        }

        for (tcid, naming, tkn) in [
            (tcls::RCLK_HDIO, "RCLK_HDIO", "RCLK_HDIO"),
            (tcls::RCLK_HDIO, "RCLK_HDIO", "RCLK_RCLK_HDIO_R_FT"),
            (tcls::RCLK_HDIO, "RCLK_HDIO", "RCLK_RCLK_HDIO_LAST_R_FT"),
            (tcls::RCLK_HDIOS, "RCLK_HDIOS", "RCLK_RCLK_HDIOS_L_FT"),
            (tcls::RCLK_HDIOS, "RCLK_HDIOS", "RCLK_RCLK_HDIOS_CMAC_L_FT"),
        ] {
            for i in 0..24 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HDISTR_FT0_{i}"),
                    30,
                    wires::GCLK_HDISTR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HROUTE_L{i}"),
                    30,
                    wires::GCLK_HROUTE[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HROUTE_R{i}"),
                    30,
                    wires::GCLK_HROUTE_FAR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = XLAT24[i] * 2 + 5),
                    30,
                    wires::GCLK_HROUTE_MUX[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = XLAT24[i] * 2 + 4),
                    30,
                    wires::GCLK_HROUTE_FAR_MUX[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = XLAT24[i] + 4),
                    30,
                    wires::OMUX_BUFG_HDIO[i],
                );
            }
            for i in 0..4 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CCIO_IO2RCLK{i}"),
                    30,
                    wires::CCIO_HDIO[i],
                );
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let top_xy = xy.delta(0, -30);
                let int_xy = self.builder.walk_to_int(top_xy, Dir::W, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming("INTF_E_PCIE");
                let mut bels = vec![];
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFGCE_HDIO[i], "BUFGCE_HDIO", i >> 1, i & 1)
                            .pin_rename("CE_PRE_OPTINV", "CE"),
                    );
                }
                for (i, x, y) in [
                    (0, 0, 0),
                    (1, 0, 1),
                    (2, 1, 0),
                    (3, 1, 1),
                    (4, 2, 0),
                    (5, 2, 1),
                    (6, 3, 0),
                ] {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::ABUS_SWITCH_HDIO[i], "ABUS_SWITCH", x, y)
                            .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                    );
                }
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&wires::IMUX_BUFGCE_HDIO[..])
                    .optin_muxes(&wires::IMUX_BUFGCE_HDIO_CCIO[..])
                    .optin_muxes(&wires::OMUX_BUFG_HDIO[..])
                    .optin_muxes(&wires::GCLK_HDISTR[..])
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .optin_muxes(&wires::GCLK_HROUTE_FAR[..])
                    .optin_muxes(&wires::GCLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::GCLK_HROUTE_FAR_MUX[..]);
                for i in 0..60 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                        .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
                }
                xn.bels(bels).extract();
            }
        }
        for tcid in [tcls::RCLK_HDIO, tcls::RCLK_HDIOS] {
            if let Some(pips) = self.builder.pips.get_mut(&(tcid, bslots::RCLK_BEL_INT)) {
                for i in 0..4 {
                    pips.pips.insert(
                        (
                            wires::CCIO_HDIO[i].cell(30),
                            wires::OUT_IOB_CLKPAD[i].cell(30).pos(),
                        ),
                        PipMode::PermaBuf,
                    );
                }
            }
        }

        for (tcid, naming, tkn, side) in [
            (
                tcls::RCLK_HDIOL_CMT_W,
                "RCLK_HDIOL_L",
                "RCLK_RCLK_HDIOL_L_FT",
                Dir::W,
            ),
            (
                tcls::RCLK_HDIOL_CMT_W,
                "RCLK_HDIOL_L",
                "RCLK_RCLK_HDIOL_MRC_L_FT",
                Dir::W,
            ),
            (
                tcls::RCLK_HDIOL,
                "RCLK_HDIOL_R",
                "RCLK_RCLK_HDIOL_R_FT",
                Dir::E,
            ),
            (
                tcls::RCLK_HDIOL_CMT_E,
                "RCLK_HDIOL_R",
                "RCLK_RCLK_HDIOL_MRC_R_FT",
                Dir::E,
            ),
        ] {
            for i in 0..24 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HDISTR_FT0_{i}"),
                    30,
                    wires::GCLK_HDISTR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HDISTR_FT1_{i}"),
                    30,
                    wires::GCLK_HDISTR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = XLAT24[i] + 4),
                    30,
                    wires::OMUX_BUFG_HDIO[i],
                );
                if side == Dir::W {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!(
                            "CLK_CMT_DRVR_TRI_HROUTE_{ii}_CLK_OUT_B",
                            ii = XLAT24[i] * 2 + 1
                        ),
                        30,
                        wires::GCLK_HROUTE_HDIOL_VOID[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HROUTE_FT1_{i}"),
                        30,
                        if tcid != tcls::RCLK_HDIOL {
                            wires::GCLK_HROUTE_FAR[i]
                        } else {
                            wires::GCLK_HROUTE[i]
                        },
                    );
                } else {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CMT_DRVR_TRI_HROUTE_{ii}_CLK_OUT_B", ii = XLAT24[i] * 2),
                        30,
                        wires::GCLK_HROUTE_HDIOL_VOID[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HROUTE_FT0_{i}"),
                        30,
                        if tcid != tcls::RCLK_HDIOL {
                            wires::GCLK_HROUTE_FAR[i]
                        } else {
                            wires::GCLK_HROUTE[i]
                        },
                    );
                }
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = XLAT24[i] * 2 + 5),
                    30,
                    wires::GCLK_HROUTE_HDIOL_W_MUX[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = XLAT24[i] * 2 + 4),
                    30,
                    wires::GCLK_HROUTE_HDIOL_E_MUX[i],
                );
            }
            for i in 0..4 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CCIO_IO2RCLK{i}"),
                    30,
                    wires::CCIO_HDIO[i],
                );
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let top_xy = xy.delta(0, -30);
                let int_xy = self.builder.walk_to_int(top_xy, !side, false).unwrap();
                let intf_xy = int_xy.delta(if side == Dir::W { -1 } else { 1 }, 0);
                let intf_tile = &self.rd.tiles[&intf_xy];
                let intf = self.builder.ndb.get_tile_class_naming(
                    match self.rd.tile_kinds.key(intf_tile.kind).as_str() {
                        "INT_INTF_LEFT_TERM_IO_FT" => "INTF_W_IO",
                        "INT_INTF_RIGHT_TERM_IO" => "INTF_E_IO",
                        "INT_INTF_RIGHT_TERM_HDIO_FT" => "INTF_E_PCIE",
                        x => panic!("ummm {x}"),
                    },
                );
                let mut bels = vec![];
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFGCE_HDIO[i], "BUFGCE_HDIO", i >> 1, i & 1)
                            .pin_rename("CE_PRE_OPTINV", "CE"),
                    );
                }
                for (i, x, y) in [
                    (0, 0, 0),
                    (1, 0, 1),
                    (2, 1, 0),
                    (3, 1, 1),
                    (4, 2, 0),
                    (5, 2, 1),
                    (6, 3, 0),
                    (7, 3, 1),
                    (8, 4, 0),
                    (9, 4, 1),
                    (10, 5, 0),
                    (11, 5, 1),
                ] {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::ABUS_SWITCH_HDIO[i], "ABUS_SWITCH", x, y)
                            .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                    );
                }
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&wires::IMUX_BUFGCE_HDIO[..])
                    .optin_muxes(&wires::IMUX_BUFGCE_HDIO_CCIO[..])
                    .optin_muxes(&wires::OMUX_BUFG_HDIO[..])
                    .optin_muxes(&wires::GCLK_HDISTR[..])
                    .optin_muxes(&wires::GCLK_HROUTE[..])
                    .optin_muxes(&wires::GCLK_HROUTE_FAR[..])
                    .optin_muxes(&wires::GCLK_HROUTE_HDIOL_VOID[..])
                    .optin_muxes(&wires::GCLK_HROUTE_HDIOL_W_MUX[..])
                    .optin_muxes(&wires::GCLK_HROUTE_HDIOL_E_MUX[..]);
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
        for tcid in [
            tcls::RCLK_HDIOL,
            tcls::RCLK_HDIOL_CMT_W,
            tcls::RCLK_HDIOL_CMT_E,
        ] {
            if let Some(pips) = self.builder.pips.get_mut(&(tcid, bslots::RCLK_BEL_INT)) {
                for i in 0..4 {
                    pips.pips.insert(
                        (
                            wires::CCIO_HDIO[i].cell(30),
                            wires::OUT_IOB_CLKPAD[i]
                                .cell(if i < 2 { 30 } else { 0 })
                                .pos(),
                        ),
                        PipMode::PermaBuf,
                    );
                }
            }
        }
    }

    fn fill_tiles_cmt(&mut self) {
        for i in 0..24 {
            self.builder.extra_name_cell(
                format!("CLK_CMT_MUX_16_ENC_{i}_CLK_OUT"),
                30,
                wires::OMUX_BUFG_CMT[i],
            );
            self.builder.extra_name_cell(
                format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 4 + i * 8),
                30,
                wires::OMUX_HDISTR_CMT[i],
            );
        }
        for (tcid, naming, tkn, side) in [
            (tcls::CMT, "CMT_L", "CMT_L", Dir::W),
            (tcls::CMT, "CMT_L", "CMT_CMT_LEFT_DL3_FT", Dir::W),
            (tcls::CMT_HBM, "CMT_L_HBM", "CMT_LEFT_H", Dir::W),
            (tcls::CMT, "CMT_R", "CMT_RIGHT", Dir::E),
            (tcls::CMT, "CMT_R", "CMT_CMT_RIGHT_DL4T_FT", Dir::E),
            (tcls::CMTXP, "CMTXP_R", "CMTXP_CMTXP_RIGHT_FT", Dir::E),
        ] {
            let is_hbm = tcid == tcls::CMT_HBM;
            let is_xp = tcid == tcls::CMTXP;
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
                    format!("CLK_HDISTR_0_{i}"),
                    30,
                    wires::GCLK_HDISTR[i],
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
                    format!("CLK_CMT_DRVR_TRI_{ii}_CLK_OUT_B", ii = i * 4),
                    30,
                    wires::GCLK_HDISTR_CMT[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = 1 + i * 8),
                    30,
                    wires::GCLK_HDISTR_MUX[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = i * 8),
                    30,
                    wires::GCLK_HDISTR_CMT_MUX[i],
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
                    format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = i * 2 + 1),
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
                    0, 3, 36, 53, 56, 59, 62, 65, 68, 71, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 39,
                    42, 45, 48,
                ][i];
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
            if is_xp {
                for byte in 0..4 {
                    for i in 0..2 {
                        self.builder.extra_name_tile_cell(
                            tkn,
                            if byte < 2 {
                                format!("CLK_CCIO_BOT{ii}", ii = byte * 2 + i)
                            } else {
                                format!("CLK_CCIO_TOP{ii}", ii = byte * 2 + i - 4)
                            },
                            byte * 15,
                            wires::OUT_PHY_CLK[i],
                        );
                    }
                }
                for i in 0..4 {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CCIO_MID{i}"),
                        30,
                        wires::CCIO_CMT[i],
                    );
                }
            } else {
                for byte in 0..4 {
                    for i in 0..2 {
                        self.builder.extra_name_tile_cell(
                            tkn,
                            format!("PHY2RCLK_SS_DIVCLK_{byte}_{i}"),
                            byte * 15,
                            wires::OUT_PHY_CLK[i],
                        );
                    }
                }
                for i in 0..4 {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("IOB2CLK_CCIO{i}"),
                        30,
                        wires::CCIO_CMT[i],
                    );
                }
            }

            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, !side, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(if side == Dir::W {
                    "INTF_W_IO"
                } else {
                    "INTF_E_IO"
                });
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
                    let sy = if i < 18 { i } else { i + 1 };
                    for (pin, wire) in [
                        ("CLK_IN", wires::GCLK_HDISTR_TEST_CMT_IN[i].cell(30)),
                        ("CLK_OUT", wires::GCLK_HDISTR_TEST_CMT[i].cell(30)),
                    ] {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "GCLK_TEST_BUFE3",
                            0,
                            sy,
                            pin,
                            wire,
                            false,
                        );
                    }
                    bels.push(self.builder.bel_xy(
                        bslots::GCLK_TEST_BUF_CMT[i],
                        "GCLK_TEST_BUFE3",
                        0,
                        sy,
                    ));
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
                    let slot_name = if is_xp { "PLLXP" } else { "PLL" };
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

                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = 56 + i * 2),
                        30,
                        wires::IMUX_PLL_CLKFB_HDISTR[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = 57 + i * 2),
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
                        format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = 60 + i * 3),
                        30,
                        wires::IMUX_PLL_CLKIN_HDISTR[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = 61 + i * 3),
                        30,
                        wires::IMUX_PLL_CLKIN_HROUTE[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = 62 + i * 3),
                        30,
                        wires::IMUX_PLL_CLKIN_HDISTR_DLY[i],
                    );

                    for (pin, wire) in [
                        ("CLKIN", wires::IMUX_PLL_CLKIN[i].cell(30)),
                        ("CLKFBIN", wires::IMUX_PLL_CLKFB[i].cell(30)),
                    ] {
                        if is_hbm && pin == "CLKFBIN" {
                            continue;
                        }
                        self.builder
                            .extra_name_pin_xy(tkn, slot_name, 0, i, pin, wire, false);
                    }

                    let mut bel = self
                        .builder
                        .bel_xy(bslots::PLL[i], slot_name, 0, i)
                        .pins_name_only(&["CLKOUTPHY_P"]);
                    if is_hbm {
                        // the muxes are repurposed for HBM reference
                        bel = bel.pin_name_only("CLKFBIN", 1);
                    }
                    if is_xp {
                        bel = bel
                            .pins_name_only(&["LOCKED_DMC"])
                            .pin_name_only("CLKOUTPHY_DMCEN", 1)
                            .pin_name_only("RST_DMC", 1);
                    }
                    bels.push(bel);
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
                        "MMCM",
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
                        .extra_name_pin_xy(tkn, "MMCM", 0, 0, pin, wire, false);
                }

                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_48_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKFB_HDISTR,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_49_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKFB_HDISTR_DLY,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "VCC_WIRE51",
                    30,
                    wires::IMUX_MMCM_CLKFB_DUMMY0,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "VCC_WIRE52",
                    30,
                    wires::IMUX_MMCM_CLKFB_DUMMY1,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_50_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN1_HDISTR,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_51_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN1_HROUTE,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_52_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN1_HDISTR_DLY,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "GND_WIRE0",
                    30,
                    wires::IMUX_MMCM_CLKIN1_DUMMY0,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_53_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN2_HDISTR,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_54_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN2_HROUTE,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "CLK_CMT_MUX_24_ENC_55_CLK_OUT",
                    30,
                    wires::IMUX_MMCM_CLKIN2_HDISTR_DLY,
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    "GND_WIRE1",
                    30,
                    wires::IMUX_MMCM_CLKIN2_DUMMY0,
                );

                bels.push(self.builder.bel_xy(bslots::MMCM, "MMCM", 0, 0));
                bels.push(
                    self.builder
                        .bel_xy(bslots::ABUS_SWITCH_CMT, "ABUS_SWITCH", 0, 0)
                        .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                );
                if is_hbm {
                    for i in 0..2 {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "HBM_REF_CLK",
                            0,
                            i,
                            "REF_CLK",
                            wires::IMUX_PLL_CLKFB[i].cell(30),
                            false,
                        );
                        bels.push(
                            self.builder
                                .bel_xy(bslots::HBM_REF_CLK[i], "HBM_REF_CLK", 0, i),
                        );
                    }
                }
                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
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
                    .optin_muxes(&wires::GCLK_HDISTR_CMT_MUX[..])
                    .optin_muxes(&wires::GCLK_HDISTR_CMT[..])
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

    fn fill_tiles_xiphy(&mut self) {
        for (tkn, side) in [("XIPHY_BYTE_L", Dir::W), ("XIPHY_BYTE_RIGHT", Dir::E)] {
            for i in 0..6 {
                self.builder
                    .extra_name_tile(tkn, format!("GCLK_FT0_{i}"), wires::GCLK_XIPHY[i]);
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, !side, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(if side == Dir::W {
                    "INTF_W_IO"
                } else {
                    "INTF_E_IO"
                });
                let mut bels = vec![];
                for i in 0..13 {
                    if matches!(i, 0 | 6) {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "BITSLICE_RX_TX",
                            0,
                            i,
                            "PHY2CLB_FIFO_WRCLK",
                            wires::OUT_PHY_CLK[i / 6].cell(0),
                            false,
                        );
                    }
                    for (pin, w) in [
                        ("RX_CLK", wires::IMUX_BITSLICE_CLK),
                        ("RX_CLK_C", wires::IMUX_BITSLICE_RX_CLK_C),
                        ("RX_CLK_C_B", wires::IMUX_BITSLICE_RX_CLK_C_B),
                        ("TX_OCLK", wires::IMUX_BITSLICE_TX_OCLK),
                    ] {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "BITSLICE_RX_TX",
                            0,
                            i,
                            pin,
                            w[i].cell(0),
                            pin == "RX_CLK",
                        );
                    }
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BITSLICE[i], "BITSLICE_RX_TX", 0, i)
                        .manual()
                        .pin_rename("RX_CLK", "CLK")
                        .pin_rename("CLB2PHY_FIFO_RDEN", "FIFO_RDEN")
                        .pin_rename("PHY2CLB_FIFO_EMPTY", "FIFO_EMPTY")
                        .pin_rename("PHY2CLB_FIFO_WRCLK", "FIFO_WRCLK")
                        .pin_rename("RX_RST", "RAW_RX_RST")
                        .pin_rename("TX_RST", "RAW_TX_RST")
                        .pins_name_only(&[
                            // aliases of CLK
                            "RX_CLKDIV",
                            "TX_CLK",
                            "TX_OCLKDIV",
                            // reclaimed from XIPHY_FEEDTHROUGH
                            "RAW_TX_RST",
                            "RAW_RX_RST",
                            // assorted
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
                            "TX_REGRST",
                            "TX_Q",
                            "RX_CLK_P",
                            "RX_CLK_N",
                            "RX_CTRL_CLK",
                            "RX_CTRL_CE",
                            "RX_CTRL_INC",
                            "RX_CTRL_LD",
                            "RX_DCC0",
                            "RX_DCC1",
                            "RX_DCC2",
                            "RX_DCC3",
                            "RX_VTC_READY",
                            "RX_RESET",
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
                        .extra_wire("DYN_DCI_OUT", &[format!("PHY2IOB_ODT_OUT_BYTE{i}")])
                        .extra_int_in(
                            "DYN_DCI_OUT_INT",
                            &[if i < 6 {
                                format!("CLB2PHY_ODT_LOW{i}")
                            } else {
                                format!("CLB2PHY_ODT_UPP{ii}", ii = i - 6)
                            }],
                        );
                    if !matches!(i, 0 | 6) {
                        bel = bel.pins_name_only(&["FIFO_WRCLK"]);
                    }
                    for i in 0..18 {
                        bel = bel.pins_name_only(&[
                            format!("BS2CTL_IDELAY_FIXED_DLY_RATIO{i}"),
                            format!("BS2CTL_ODELAY_FIXED_DLY_RATIO{i}"),
                        ]);
                    }
                    for i in 0..9 {
                        bel = bel.pins_name_only(&[
                            format!("BS2CTL_RX_CNTVALUEOUT{i}"),
                            format!("BS2CTL_TX_CNTVALUEOUT{i}"),
                            format!("RX_CTRL_DLY{i}"),
                            format!("TX_CTRL_DLY{i}"),
                        ]);
                    }
                    bels.push(bel);
                }
                for i in 0..2 {
                    self.builder.extra_name_pin_xy(
                        tkn,
                        "BITSLICE_TX",
                        0,
                        i,
                        "CLK",
                        wires::IMUX_BITSLICE_T_CLK[i].cell(0),
                        false,
                    );
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BITSLICE_T[i], "BITSLICE_TX", 0, i)
                        .manual()
                        .pin_rename("RST", "RAW_RST")
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
                            "RAW_RST",
                            "REGRST",
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
                    for i in 0..9 {
                        bel = bel.pins_name_only(&[
                            format!("BS2CTL_CNTVALUEOUT{i}"),
                            format!("CTRL_DLY{i}"),
                        ]);
                    }
                    bels.push(bel);
                }
                for i in 0..2 {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BITSLICE_CONTROL[i], "BITSLICE_CONTROL", 0, i)
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
                            "CLB2PHY_CTRL_RST",
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
                        ])
                        .pin_name_only("CLK_FROM_EXT", 1);
                    for i in 0..7 {
                        bel = bel.pins_name_only(&[
                            format!("RX_DCC{i:02}_0"),
                            format!("RX_DCC{i:02}_1"),
                            format!("RX_DCC{i:02}_2"),
                            format!("RX_DCC{i:02}_3"),
                            format!("RX_PDQ{i}_IN"),
                            format!("RX_NDQ{i}_IN"),
                            format!("IDELAY_CTRL_CLK{i}"),
                            format!("IDELAY_CE_OUT{i}"),
                            format!("IDELAY_INC_OUT{i}"),
                            format!("IDELAY_LD_OUT{i}"),
                            format!("FIXED_IDELAY{i:02}"),
                            format!("ODELAY_CE_OUT{i}"),
                            format!("ODELAY_INC_OUT{i}"),
                            format!("ODELAY_LD_OUT{i}"),
                            format!("FIXED_ODELAY{i:02}"),
                            format!("VTC_READY_IDELAY{i:02}"),
                            format!("VTC_READY_ODELAY{i:02}"),
                            format!("WL_TRAIN{i}"),
                            format!("DYN_DCI_OUT{i}"),
                            format!("DQS_IN{i}"),
                            format!("RX_BS_RESET{i}"),
                            format!("TX_BS_RESET{i}"),
                            format!("PDQS_OUT{i}"),
                            format!("NDQS_OUT{i}"),
                            format!("REFCLK_EN{i}"),
                            format!("IFIFO_BYPASS{i}"),
                            format!("BS2CTL_RIU_BS_DQS_EN{i}"),
                        ]);
                        for j in 0..9 {
                            bel = bel.pins_name_only(&[
                                format!("IDELAY{i:02}_IN{j}"),
                                format!("IDELAY{i:02}_OUT{j}"),
                                format!("ODELAY{i:02}_IN{j}"),
                                format!("ODELAY{i:02}_OUT{j}"),
                            ]);
                        }
                        for j in 0..18 {
                            bel = bel.pins_name_only(&[
                                format!("FIXDLYRATIO_IDELAY{i:02}_{j}"),
                                format!("FIXDLYRATIO_ODELAY{i:02}_{j}"),
                            ]);
                        }
                    }
                    for i in 0..8 {
                        bel = bel.pins_name_only(&[
                            format!("ODELAY_CTRL_CLK{i}"),
                            format!("DYNAMIC_MODE_EN{i}"),
                            format!("EN_DIV_DLY_OE{i}"),
                            format!("TOGGLE_DIV2_SEL{i}"),
                            format!("TX_DATA_PHASE{i}"),
                            format!("BS2CTL_RIU_TX_DATA_PHASE{i}"),
                            format!("DIV2_CLK_OUT{i}"),
                            format!("DIV_CLK_OUT{i}"),
                            format!("DDR_CLK_OUT{i}"),
                            format!("PH02_DIV2_360_{i}"),
                            format!("PH13_DIV2_360_{i}"),
                            format!("PH0_DIV_720_{i}"),
                            format!("PH1_DIV_720_{i}"),
                            format!("PH2_DIV_720_{i}"),
                            format!("PH3_DIV_720_{i}"),
                        ]);
                    }
                    for i in 0..9 {
                        bel = bel.pins_name_only(&[
                            format!("TRISTATE_ODELAY_IN{i}"),
                            format!("TRISTATE_ODELAY_OUT{i}"),
                        ]);
                    }
                    for i in 0..16 {
                        bel = bel.pins_name_only(&[format!("RIU2CLB_RD_DATA{i}")]);
                    }
                    bel = bel
                        .sub_xy("PLL_SELECT_SITE", 0, i)
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
                    .bel_xy(bslots::XIPHY_BYTE, "XIPHY_FEEDTHROUGH", 0, 0)
                    .manual()
                    .pins_name_only(&[
                        "CLB2PHY_CTRL_RST_LOW_SMX",
                        "CLB2PHY_CTRL_RST_UPP_SMX",
                        "CLB2PHY_TRISTATE_ODELAY_RST_SMX0",
                        "CLB2PHY_TRISTATE_ODELAY_RST_SMX1",
                        "CLB2PHY_TXBIT_TRI_RST_SMX0",
                        "CLB2PHY_TXBIT_TRI_RST_SMX1",
                        "SCAN_INT_LOWER",
                        "SCAN_INT_UPPER",
                        "DIV_CLK_OUT_LOW",
                        "DIV_CLK_OUT_UPP",
                        "XIPHY_CLK_STOP_CTRL_LOW",
                        "XIPHY_CLK_STOP_CTRL_UPP",
                        "RCLK2PHY_CLKDR",
                        "RCLK2PHY_SHIFTDR",
                    ]);
                for i in 0..13 {
                    bel = bel.pins_name_only(&[
                        format!("CLB2PHY_TXBIT_RST_SMX{i}"),
                        format!("CLB2PHY_RXBIT_RST_SMX{i}"),
                        format!("CLB2PHY_FIFO_CLK_SMX{i}"),
                        format!("CLB2PHY_IDELAY_RST_SMX{i}"),
                        format!("CLB2PHY_ODELAY_RST_SMX{i}"),
                    ]);
                }
                for i in 0..6 {
                    bel = bel.pins_name_only(&[format!("CTL2BS_REFCLK_EN_LOW_SMX{i}")]);
                }
                for i in 0..7 {
                    bel = bel.pins_name_only(&[
                        format!("CTL2BS_REFCLK_EN_LOW{i}"),
                        format!("CTL2BS_REFCLK_EN_UPP{i}"),
                        format!("CTL2BS_REFCLK_EN_UPP_SMX{i}"),
                    ]);
                }
                bel = bel
                    .sub_xy("RIU_OR", 0, 0)
                    .pins_name_only(&["RIU_RD_VALID_LOW", "RIU_RD_VALID_UPP"]);
                for i in 0..16 {
                    bel = bel.pins_name_only(&[
                        format!("RIU_RD_DATA_LOW{i}"),
                        format!("RIU_RD_DATA_UPP{i}"),
                    ]);
                }
                bels.push(bel);
                let mut xn = self
                    .builder
                    .xtile_id(tcls::XIPHY, "XIPHY", xy)
                    .switchbox(bslots::XIPHY_INT)
                    .optin_muxes(&wires::IMUX_BITSLICE_CLK[..])
                    .optin_muxes(&wires::IMUX_BITSLICE_RX_CLK_C[..])
                    .optin_muxes(&wires::IMUX_BITSLICE_RX_CLK_C_B[..])
                    .optin_muxes(&wires::IMUX_BITSLICE_TX_OCLK[..])
                    .optin_muxes(&wires::IMUX_BITSLICE_T_CLK[..]);
                for i in 0..15 {
                    xn = xn
                        .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), side, i)
                        .ref_single(
                            int_xy.delta(if side == Dir::W { -1 } else { 1 }, (i + i / 30) as i32),
                            i,
                            intf,
                        )
                }
                let mut xt = xn.bels(bels).extract();

                let (mut bel_byte, mut naming_byte) = xt.bels.pop().unwrap();
                let mut bels_control = xt.bels.split_off(15);
                let mut bels_bitslice_t = xt.bels.split_off(13);
                let mut bels_bitslice = xt.bels;

                for i in 0..13 {
                    for (pname, bpname) in [
                        ("FIFO_RDCLK", format!("CLB2PHY_FIFO_CLK{i}")),
                        ("RX_RST", format!("RXBIT_RST{i}")),
                        ("TX_RST", format!("TXBIT_RST{i}")),
                        ("RX_RST_DLY", format!("IDELAY_RST{i}")),
                        ("TX_RST_DLY", format!("ODELAY_RST{i}")),
                    ] {
                        let pin = bel_byte.pins.remove(&bpname).unwrap();
                        let pn = naming_byte.pins[&bpname].clone();
                        bels_bitslice[i].0.pins.insert(pname.into(), pin);
                        bels_bitslice[i].1.pins.insert(pname.into(), pn);
                    }
                }
                for i in 0..2 {
                    for (pname, bpname) in [
                        ("RST", format!("TXBIT_TRI_RST{i}")),
                        ("RST_DLY", format!("TRISTATE_ODELAY_RST{i}")),
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
                    (0, "RST", "CTRL_RST_LOW"),
                    (1, "RST", "CTRL_RST_UPP"),
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
                    // move RIU pins to XIPHY_BYTE
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
                    self.builder
                        .insert_tcls_bel(tcls::XIPHY, bslot, BelInfo::Legacy(bel));
                    self.builder.insert_bel_naming("XIPHY", bslot, bn);
                }
                for ((bel, bn), bslot) in bels_bitslice_t.into_iter().zip(bslots::BITSLICE_T) {
                    self.builder
                        .insert_tcls_bel(tcls::XIPHY, bslot, BelInfo::Legacy(bel));
                    self.builder.insert_bel_naming("XIPHY", bslot, bn);
                }
                for ((bel, bn), bslot) in bels_control.into_iter().zip(bslots::BITSLICE_CONTROL) {
                    self.builder
                        .insert_tcls_bel(tcls::XIPHY, bslot, BelInfo::Legacy(bel));
                    self.builder.insert_bel_naming("XIPHY", bslot, bn);
                }
                self.builder.insert_tcls_bel(
                    tcls::XIPHY,
                    bslots::XIPHY_BYTE,
                    BelInfo::Legacy(bel_byte),
                );
                self.builder
                    .insert_bel_naming("XIPHY", bslots::XIPHY_BYTE, naming_byte);
            }
        }

        for (tkn, naming) in [
            ("RCLK_RCLK_XIPHY_INNER_FT", "RCLK_XIPHY_L"),
            ("RCLK_XIPHY_OUTER_RIGHT", "RCLK_XIPHY_R"),
        ] {
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_ZERO{i}"),
                    wires::GCLK_HDISTR_FAR[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_ONE{i}"),
                    wires::GCLK_HDISTR[i],
                );
            }
            for i in 0..6 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_TO_XIPHY_BYTES_BOT{i}"),
                    1,
                    wires::GCLK_XIPHY[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_TO_XIPHY_BYTES_TOP{i}"),
                    0,
                    wires::GCLK_XIPHY[i],
                );
            }
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                self.builder
                    .xtile_id(tcls::RCLK_XIPHY, naming, xy)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&wires::GCLK_HDISTR[..])
                    .optin_muxes(&wires::GCLK_HDISTR_FAR[..])
                    .optin_muxes(&wires::GCLK_XIPHY[..])
                    .extract();
            }
        }
    }

    fn fill_tiles_hpio(&mut self) {
        for (tkn, side) in [("HPIO_L", Dir::W), ("HPIO_RIGHT", Dir::E)] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, !side, false).unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(if side == Dir::W {
                    "INTF_W_IO"
                } else {
                    "INTF_E_IO"
                });
                let mut bels = vec![];
                let mut is_alt = false;
                let mut is_nocfg = true;
                let mut is_noams = true;
                if let Some(wire) = self.builder.rd.wires.get("HPIO_IOBSNGL_19_TSDI_PIN") {
                    let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                    if tk.wires.contains_key(&wire) {
                        is_alt = true;
                    }
                }
                if let Some(wire) = self.builder.rd.wires.get("HPIO_IOBPAIR_26_TSDI_PIN") {
                    let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                    if tk.wires.contains_key(&wire) {
                        is_nocfg = false;
                    }
                }
                if let Some(wire) = self.builder.rd.wires.get("HPIO_IOBPAIR_26_SWITCH_OUT_PIN") {
                    let tk = &self.builder.rd.tile_kinds[self.builder.rd.tiles[&xy].kind];
                    if tk.wires.contains_key(&wire) {
                        is_noams = false;
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
                            "DOUT",
                            "IO",
                            "LVDS_TRUE",
                            "PAD_RES",
                            "O_B",
                            "TSTATEB",
                            "DYNAMIC_DCI_TS",
                            "VREF",
                        ])
                        .pin_name_only("SWITCH_OUT", if is_noams { 0 } else { 1 })
                        .pin_name_only("OP", 1)
                        .pin_name_only("TSP", 1)
                        .pin_name_only("TSDI", 1);
                    if matches!(i, 12 | 25) {
                        bel = bel
                            .pin_dummy("IO")
                            .pin_dummy("LVDS_TRUE")
                            .pin_dummy("OUTB_B_IN")
                            .pin_dummy("TSTATE_IN");
                        if !is_alt {
                            bel = bel.pin_name_only("TSDI", 0);
                        }
                    }
                    if is_nocfg {
                        bel = bel.pin_name_only("TSDI", 0);
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
                                "HPIO_IOBPAIR_26_I",
                                "HPIO_IOBPAIR_31_I",
                                "HPIO_IOBPAIR_76_I",
                                "HPIO_IOBPAIR_81_I",
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
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HPIOB_DCI[i], "HPIOB_DCI_SNGL", 0, i),
                    );
                }
                bels.push(
                    self.builder
                        .bel_xy(bslots::HPIO_VREF, "HPIO_VREF_SITE", 0, 0)
                        .pins_name_only(&["VREF1", "VREF2"]),
                );
                bels.push(self.builder.bel_xy(bslots::HPIO_BIAS, "BIAS", 0, 0));
                let naming = if is_noams {
                    "HPIO_NOAMS"
                } else if is_nocfg {
                    "HPIO_NOCFG"
                } else if is_alt {
                    "HPIO_ALTCFG"
                } else {
                    "HPIO"
                };
                let mut xn = self.builder.xtile_id(tcls::HPIO, naming, xy);
                for i in 0..30 {
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

        for (tkn, side) in [("RCLK_HPIO_L", Dir::W), ("RCLK_HPIO_R", Dir::E)] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self
                    .builder
                    .walk_to_int(xy.delta(0, -30), !side, false)
                    .unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(if side == Dir::W {
                    "INTF_W_IO"
                } else {
                    "INTF_E_IO"
                });
                let mut bels = vec![];
                for i in 0..7 {
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::ABUS_SWITCH_HPIO[i],
                                "ABUS_SWITCH",
                                if side == Dir::W {
                                    i
                                } else {
                                    [0, 6, 1, 3, 2, 4, 5][i]
                                },
                                0,
                            )
                            .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                    );
                }
                bels.push(
                    self.builder
                        .bel_xy(bslots::HPIO_ZMATCH, "HPIO_ZMATCH_BLK_HCLK", 0, 0),
                );
                bels.push(
                    self.builder
                        .bel_xy(bslots::HPIO_PRBS, "HPIO_RCLK_PRBS", 0, 0),
                );
                let mut xn = self.builder.xtile_id(tcls::RCLK_HPIO, "RCLK_HPIO", xy);
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
    }

    fn fill_tiles_xp5io(&mut self) {
        let tkn = "HSM_XP5IO_FT";
        let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let xp5io_es1 = if let Some((tk, _)) = self.rd.tile_kinds.get("HSM_XP5IO_FT")
            && self.rd.tk_slot_xy(tk, "IOB", 0, 18).is_some()
        {
            true
        } else {
            false
        };
        let naming = if !xp5io_es1 { "XP5IO" } else { "XP5IO_ES1" };

        for i in 0..4 {
            self.builder.extra_name_pin_xy(
                tkn,
                "LPDDRMC",
                0,
                0,
                format!("IF_XPIO_MMCM_DMC_OABUT_XPIO_CCIO{i}"),
                wires::CCIO_CMT[i].cell(30),
                false,
            );
        }

        for byte in 0..4 {
            for i in 0..2 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "LPDDRMC",
                    0,
                    0,
                    format!(
                        "IF_XPIO_MMCM_DMC_OABUT_XPIO_PHY_CLK{ii}",
                        ii = 1 + byte * 2 + i
                    ),
                    wires::OUT_PHY_CLK[i].cell(byte * 15),
                    false,
                );
            }
        }

        let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
        let intf = self.builder.ndb.get_tile_class_naming("INTF_E_IO");

        let mut bels = vec![];
        let tile = &self.builder.rd.tiles[&xy];
        let tk = &self.builder.rd.tile_kinds[tile.kind];

        for i in 0..33 {
            let sy = if !xp5io_es1 {
                if i >= 30 {
                    i - 30 + 19
                } else if i >= 18 {
                    i - 18 + 22
                } else {
                    i
                }
            } else {
                i
            };
            let tksite = tk
                .sites
                .get(&self.builder.rd.slot_xy("IOB", 0, sy).unwrap())
                .unwrap()
                .1;
            let mut bel = self
                .builder
                .bel_xy(bslots::XP5IOB_PAIR[i], "IOB", 0, sy)
                .manual();
            for pin in tksite.pins.keys() {
                bel = bel.pin_name_only(pin, 0);
            }
            bels.push(bel);
        }

        for i in 0..11 {
            let tksite = tk
                .sites
                .get(&self.builder.rd.slot_xy("XP5IO_VREF", 0, i).unwrap())
                .unwrap()
                .1;
            let mut bel = self
                .builder
                .bel_xy(bslots::XP5IO_VREF[i], "XP5IO_VREF", 0, i);
            for pin in tksite.pins.keys() {
                bel = bel.pin_name_only(pin, 0);
            }
            bels.push(bel);
        }

        for i in 0..11 {
            let sy = if !xp5io_es1 {
                if i >= 10 {
                    i - 10 + 6
                } else if i >= 6 {
                    i - 6 + 7
                } else {
                    i
                }
            } else {
                i
            };

            let tksite_ls = tk
                .sites
                .get(&self.builder.rd.slot_xy("X5PHY_LS", 0, sy).unwrap())
                .unwrap()
                .1;
            let mut bel = self
                .builder
                .bel_xy(bslots::X5PHY[i], "X5PHY_LS", 0, sy)
                .manual();
            for pin in tksite_ls.pins.keys() {
                let mut buf_cnt = 0;
                if pin.starts_with("RTRIM_M2L_IN") && i == 0 {
                    buf_cnt = 1;
                }
                bel = bel.pin_name_only(pin, buf_cnt);
            }

            bel = bel.sub_xy("X5PHY_HS", 0, sy);
            let tksite_hs = tk
                .sites
                .get(&self.builder.rd.slot_xy("X5PHY_HS", 0, sy).unwrap())
                .unwrap()
                .1;
            for pin in tksite_hs.pins.keys() {
                let npin = &format!("HS_{pin}");
                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 0);
            }

            bel = bel.sub_xy("X5PHY_PLL_SELECT", 0, i);
            let tksite_sel = tk
                .sites
                .get(&self.builder.rd.slot_xy("X5PHY_PLL_SELECT", 0, i).unwrap())
                .unwrap()
                .1;
            for pin in tksite_sel.pins.keys() {
                let npin = &format!("PLL_SELECT_{pin}");
                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 0);
            }
            bels.push(bel);
        }

        let tksite = tk
            .sites
            .get(&self.builder.rd.slot_xy("LPDDRMC", 0, 0).unwrap())
            .unwrap()
            .1;
        let mut bel = self
            .builder
            .bel_xy(bslots::LPDDRMC, "LPDDRMC", 0, 0)
            .manual();
        if !xp5io_es1 {
            bel = bel
                .pin_rename("IJTAG_TDO_IABUT", "_IJTAG_TDO_IABUT")
                .pin_dummy("_IJTAG_TDO_IABUT")
                .extra_int_in("IJTAG_TDO_IABUT", &["LPDDRMC_DMC_SITE_0_IJTAG_TDO_IABUT"])
        } else {
            bel = bel
                .pin_rename("IJTAG_CLOCK_DR", "_IJTAG_CLOCK_DR")
                .pin_rename("IJTAG_SHIFT_DR", "_IJTAG_SHIFT_DR")
                .pin_rename("IJTAG_UPDATE_DR", "_IJTAG_UPDATE_DR")
                .pin_name_only("_IJTAG_CLOCK_DR", 0)
                .pin_name_only("_IJTAG_SHIFT_DR", 0)
                .pin_name_only("_IJTAG_UPDATE_DR", 0)
                .extra_int_in("IJTAG_CLOCK_DR", &["GND_WIRE0"])
                .extra_int_in("IJTAG_SHIFT_DR", &["GND_WIRE9"])
                .extra_int_in("IJTAG_UPDATE_DR", &["GND_WIRE11"])
                .extra_int_out("IJTAG_CLOCK_DR_O", &["GND_WIRE1"])
                .extra_int_out("IJTAG_SHIFT_DR_O", &["GND_WIRE10"])
                .extra_int_out("IJTAG_UPDATE_DR_O", &["GND_WIRE12"]);
        }
        for pin in tksite.pins.keys() {
            if pin.starts_with("IF_DMC_CLB2PHY")
                || pin.starts_with("IF_DMC2PHY")
                || pin.starts_with("DMC2PHY")
                || pin.starts_with("DMC_XPLL")
                || pin.starts_with("PHY_DIV4_CLK")
                || pin.starts_with("IF_XPIO_DFX_DFXCNTRL_DMC_IABUT")
                || pin.starts_with("IF_XPIO_DCI_FABRIC_DMC_IABUT")
                || pin.starts_with("IF_XPIO_MMCM_DMC_IABUT")
                || matches!(
                    pin.as_str(),
                    "CLOCK_DR_OABUT"
                        | "RESET_TAP_OABUT"
                        | "SHIFT_DR_OABUT"
                        | "IJTAG_TDO_EXT"
                        | "IJTAG_TDO_IABUT"
                        | "UPDATE_DR_OABUT"
                        | "SELECT_DR_OABUT"
                        | "CAPTURE_DR_OABUT"
                        | "BSCAN_CFG2IOB_PUDC_B_IABUT"
                        | "BSCAN_GTS_USR_B_IABUT"
                        | "BSCAN_EXTEST_IABUT"
                        | "BSCAN_EXTEST_SMPL_IABUT"
                        | "PLL0_CLKOUTPHY_OABUT"
                        | "PLL1_CLKOUTPHY_OABUT"
                )
            {
                bel = bel.pin_name_only(pin, 0);
            }
        }
        for pin in [
            "PLL0_CLKOUTPHY_IABUT",
            "PLL1_CLKOUTPHY_IABUT",
            "XPLL0_DMC_LOCK",
            "XPLL1_DMC_LOCK",
        ] {
            bel = bel.pin_name_only(pin, 1);
        }
        bel = bel
            .extra_wire("VCC", &["VCC_WIRE"])
            .extra_wire("NIBBLE0_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_0_Z"])
            .extra_wire("NIBBLE1_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_1_Z"])
            .extra_wire("NIBBLE2_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_3_Z"])
            .extra_wire("NIBBLE3_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_4_Z"])
            .extra_wire("NIBBLE7_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_8_Z"])
            .extra_wire("NIBBLE8_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_9_Z"])
            .extra_wire("NIBBLE9_CCIO_DUMMY", &["XP5IO_ROUTE_MUX2TO1_EN_10_Z"])
            .pin_rename("SELECT_DR", "IJTAG_SELECT_DR")
            .pin_rename("CAPTURE_DR", "IJTAG_CAPTURE_DR")
            .pin_rename("FABRIC_APB_RST_N", "APB_RST_N")
            .pin_rename("IFABRIC_APB_CLK", "APB_CLK")
            .pin_rename(
                "IF_XPIO_DCI_FABRIC_DMC_OABUT_CLB2CTR_DCI_TEST_I_CLOCK",
                "DCI_TEST_I0",
            )
            .pin_rename(
                "IF_XPIO_DCI_FABRIC_DMC_OABUT_CLB2CTRDCI_USR_RST",
                "DCI_USR_RST",
            )
            .pin_rename("IF_XPIO_DCI_FABRIC_DMC_OABUT_CLB2IOB_PUDC_B", "DCI_PUDC_B")
            // ???
            .extra_int_in("IOB_PUDC_B", &["CFG2IOB_PUDC_B"])
            .extra_int_out("IOB_PUDC_B_O", &["CFG2IOB_PUDC_B"])
            .extra_int_out("IJTAG_CAPTURE_DR_O", &["LPDDRMC_DMC_SITE_0_CAPTURE_DR"])
            .extra_int_out("IJTAG_SELECT_DR_O", &["LPDDRMC_DMC_SITE_0_SELECT_DR"])
            .extra_int_out("IJTAG_RESET_TAP_O", &["LPDDRMC_DMC_SITE_0_IJTAG_RESET_TAP"]);
        if !xp5io_es1 {
            bel = bel
                .extra_int_out("IJTAG_CLOCK_DR_O", &["LPDDRMC_DMC_SITE_0_IJTAG_CLOCK_DR"])
                .extra_int_out("IJTAG_SHIFT_DR_O", &["LPDDRMC_DMC_SITE_0_IJTAG_SHIFT_DR"])
                .extra_int_out("IJTAG_UPDATE_DR_O", &["LPDDRMC_DMC_SITE_0_IJTAG_UPDATE_DR"]);
        }

        let tksite = tk
            .sites
            .get(&self.builder.rd.slot_xy("XP5PIO_CMU_DIG_TOP", 0, 0).unwrap())
            .unwrap()
            .1;
        bel = bel.sub_xy("XP5PIO_CMU_DIG_TOP", 0, 0);
        for pin in tksite.pins.keys() {
            let npin = &format!("CMU_DIG_{pin}");
            bel = bel.pin_rename(pin, npin).pin_name_only(npin, 0);
        }

        let tksite = tk
            .sites
            .get(&self.builder.rd.slot_xy("XP5PIO_CMU_ANA", 0, 0).unwrap())
            .unwrap()
            .1;
        bel = bel.sub_xy("XP5PIO_CMU_ANA", 0, 0);
        for pin in tksite.pins.keys() {
            let npin = &format!("CMU_ANA_{pin}");
            bel = bel.pin_rename(pin, npin).pin_name_only(npin, 0);
        }

        bels.push(bel);

        for i in 0..2 {
            bels.push(
                self.builder
                    .bel_xy(bslots::ABUS_SWITCH_XP5IO[i], "ABUS_SWITCH", 0, i)
                    .raw_tile(1)
                    .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
            )
        }

        let mut xn = self
            .builder
            .xtile_id(tcls::XP5IO, naming, xy)
            .raw_tile(xy.delta(-2, 30));
        for i in 0..60 {
            xn = xn
                .ref_int_side(int_xy.delta(0, (i + i / 30) as i32), Dir::E, i)
                .ref_single(int_xy.delta(1, (i + i / 30) as i32), i, intf)
        }
        let mut xt = xn.bels(bels).extract();

        let (mut bel_lpddrmc, mut naming_lpddrmc) = xt.bels.pop().unwrap();
        let mut bels_phy = xt.bels.split_off(33);
        let mut bels_pair = xt.bels;
        let mut bels_iob =
            Vec::from_iter((0..66).map(|_| (LegacyBel::default(), BelNaming::default())));
        let mut bels_bitslice =
            Vec::from_iter((0..66).map(|_| (LegacyBel::default(), BelNaming::default())));

        for (pname, wire) in [
            ("IOB_PUDC_B", wires::IMUX_IMUX_DELAY[27]),
            ("IJTAG_RESET_TAP", wires::IMUX_IMUX_DELAY[28]),
            ("IJTAG_CAPTURE_DR", wires::IMUX_IMUX_DELAY[30]),
            ("IJTAG_SELECT_DR", wires::IMUX_IMUX_DELAY[31]),
        ] {
            let wire = wire.cell(34);
            let bpin = bel_lpddrmc.pins.get_mut(pname).unwrap();
            bpin.wires = BTreeSet::from_iter([wire]);
            let bnpin = naming_lpddrmc.pins.get_mut(pname).unwrap();
            bnpin.is_intf = false;
        }
        if !xp5io_es1 {
            for (pname, wire) in [
                ("IJTAG_CLOCK_DR", wires::IMUX_IMUX_DELAY[6]),
                ("IJTAG_SHIFT_DR", wires::IMUX_IMUX_DELAY[29]),
                ("IJTAG_UPDATE_DR", wires::IMUX_IMUX_DELAY[7]),
            ] {
                let wire = wire.cell(34);
                let bpin = bel_lpddrmc.pins.get_mut(pname).unwrap();
                bpin.wires = BTreeSet::from_iter([wire]);
                let bnpin = naming_lpddrmc.pins.get_mut(pname).unwrap();
                bnpin.is_intf = false;
            }
        }

        let mut renames = vec![];
        let mut moves = vec![];
        for pname in bel_lpddrmc.pins.keys() {
            if let Some(suffix) = pname.strip_prefix("IF_LPDDRMC_RMUX_AXI_DC_TOP_P") {
                renames.push((pname.to_string(), format!("AXIA{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_APB_FABRIC2BRIDGE_") {
                renames.push((pname.to_string(), format!("APB_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_DMC_FABRIC_") {
                renames.push((pname.to_string(), format!("DMC_{suffix}")));
            } else if let Some(suffix) = pname
                .strip_prefix("IF_XPIO_DCI_FABRIC_DMC_OABUT_CLB2CTR_DCI_")
                .or(pname.strip_prefix("IF_XPIO_DCI_FABRIC_DMC_OABUT_CTR2CLB_DCI_"))
            {
                renames.push((pname.to_string(), format!("DCI_{suffix}")));
            } else if let Some(suffix) = pname
                .strip_prefix("IF_XPIO_DFX_DFXCNTRL_DMC_OABUT_CLB2PHY_")
                .or(pname.strip_prefix("IF_XPIO_DFX_DFXCNTRL_DMC_OABUT_PHY2CLB_"))
                .or(pname.strip_prefix("IF_XPIO_DFX_DFXCNTRL_DMC_OABUT_DFX_"))
            {
                renames.push((pname.to_string(), format!("DFX_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_UBLAZE_FABRIC_BLI2UB_") {
                renames.push((pname.to_string(), format!("UBLAZE_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_UBLAZE_FABRIC_UB2BLI_") {
                renames.push((pname.to_string(), format!("UBLAZE_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_UBLAZE_FABRIC_UB_") {
                renames.push((pname.to_string(), format!("UBLAZE_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_UBLAZE_FABRIC_M_AXI_") {
                renames.push((pname.to_string(), format!("UBLAZE_AXI_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_UBLAZE_FABRIC_BSCAN_") {
                renames.push((pname.to_string(), format!("UBLAZE_BSCAN_{suffix}")));
            } else if let Some(suffix) = pname.strip_prefix("IF_HSM_CLB2PHY_P") {
                let (idx, suffix) = suffix.split_once('_').unwrap();
                let idx: usize = idx.parse().unwrap();
                let io_pin = if let Some(suf) = suffix.strip_suffix("_BIT2") {
                    format!("{suf}_2")
                } else {
                    suffix.to_string()
                };
                moves.push((idx, io_pin, pname));
            } else if let Some(suffix) = pname
                .strip_prefix("AXI0_")
                .or(pname.strip_prefix("AXI1_"))
                .or(pname.strip_prefix("MAXI_DC"))
                .or(pname.strip_prefix("MAXI_IC"))
            {
                let (prefix, suffix) = suffix.split_once("_NIB").unwrap();
                let (io_idx, suffix) = suffix.split_once('_').unwrap();
                let io_idx: usize = io_idx.parse().unwrap();
                let (axi_pin, io_pin) = if !prefix.ends_with(|x: char| x.is_ascii_digit()) {
                    let (idx, suffix) = suffix.split_once('_').unwrap();
                    (prefix.to_string(), format!("{suffix}{idx}"))
                } else {
                    let (prefix, axi_idx_lo) = prefix.rsplit_once('_').unwrap();
                    let axi_idx_lo: usize = axi_idx_lo.parse().unwrap();
                    if !prefix.ends_with(|x: char| x.is_ascii_digit()) {
                        (format!("{prefix}{axi_idx_lo}"), suffix.to_string())
                    } else {
                        let (prefix, axi_idx_hi) = prefix.rsplit_once('_').unwrap();
                        let axi_idx_hi: usize = axi_idx_hi.parse().unwrap();
                        let len = axi_idx_hi - axi_idx_lo + 1;
                        let (io_idx_lo, suffix) =
                            if suffix.starts_with(|x: char| x.is_ascii_digit()) {
                                let (io_idx_hi, suffix) = suffix.split_once('_').unwrap();
                                let (io_idx_lo, suffix) = suffix.split_once('_').unwrap();
                                let io_idx_lo: usize = io_idx_lo.parse().unwrap();
                                let io_idx_hi: usize = io_idx_hi.parse().unwrap();
                                assert_eq!(io_idx_hi - io_idx_lo + 1, len);
                                (io_idx_lo, suffix)
                            } else {
                                (0, suffix)
                            };
                        let io_pin = suffix.trim_end_matches(|x: char| x.is_ascii_digit());
                        let idx = suffix.strip_prefix(io_pin).unwrap();
                        let idx: usize = idx.parse().unwrap();
                        assert!(idx < len);
                        (
                            format!("{prefix}{axi_idx}", axi_idx = axi_idx_lo + idx),
                            format!("{io_pin}{io_idx}", io_idx = io_idx_lo + idx),
                        )
                    }
                };
                let axi_prefix = if pname.starts_with("AXI0_") {
                    "AXIA0_"
                } else if pname.starts_with("AXI1_") {
                    "AXIA1_"
                } else if pname.starts_with("MAXI_DC") {
                    "UBLAZE_AXI_DC_"
                } else if pname.starts_with("MAXI_IC") {
                    "UBLAZE_AXI_IC_"
                } else {
                    unreachable!()
                };
                renames.push((pname.to_string(), format!("{axi_prefix}{axi_pin}")));
                moves.push((io_idx, io_pin, pname));
            }
        }

        for (nidx, io_pin, pname) in moves {
            let pin = bel_lpddrmc.pins[pname].clone();
            let pn = naming_lpddrmc.pins[pname].clone();
            if let Some(suffix) = io_pin
                .strip_prefix("CLB2RIU_")
                .or(io_pin.strip_prefix("RIU2CLB_"))
            {
                let io_pin = format!("RIU_{suffix}");
                bels_phy[nidx].0.pins.insert(io_pin.clone(), pin);
                bels_phy[nidx].1.pins.insert(io_pin.clone(), pn);
            } else if let Some(suffix) = io_pin.strip_prefix("PHY2CLB_RD_DQ") {
                let (iob, idx) = suffix.split_once('_').unwrap();
                let iob: usize = iob.parse().unwrap();
                let iob_idx = nidx * 6 + iob;
                let io_pin = format!("RD_DQ{idx}");
                bels_bitslice[iob_idx].0.pins.insert(io_pin.clone(), pin);
                bels_bitslice[iob_idx].1.pins.insert(io_pin.clone(), pn);
            } else if let Some(suffix) = io_pin.strip_prefix("CLB2PHY_WR_DQ") {
                let (iob, idx) = suffix.split_once('_').unwrap();
                let iob: usize = iob.parse().unwrap();
                let iob_idx = nidx * 6 + iob;
                let io_pin = format!("WR_DQ{idx}");
                bels_bitslice[iob_idx].0.pins.insert(io_pin.clone(), pin);
                bels_bitslice[iob_idx].1.pins.insert(io_pin.clone(), pn);
            } else if let Some(suffix) = io_pin.strip_prefix("PHY2CLB_IODELAY_CNTVALUEOUT") {
                let idx: usize = suffix.parse().unwrap();
                let iob_idx = nidx * 6 + idx / 9;
                let idx = idx % 9;
                let io_pin = format!("IODELAY_CNTVALUEOUT{idx}");
                bels_bitslice[iob_idx].0.pins.insert(io_pin.clone(), pin);
                bels_bitslice[iob_idx].1.pins.insert(io_pin.clone(), pn);
            } else if let Some(suffix) = io_pin.strip_prefix("CLB2PHY_IODELAY_CNTVALUEIN") {
                let idx: usize = suffix.parse().unwrap();
                let iob_idx = nidx * 6 + idx / 9;
                let idx = idx % 9;
                let io_pin = format!("IODELAY_CNTVALUEIN{idx}");
                bels_bitslice[iob_idx].0.pins.insert(io_pin.clone(), pin);
                bels_bitslice[iob_idx].1.pins.insert(io_pin.clone(), pn);
            } else if io_pin.contains("DELAY")
                || io_pin.contains("BIT_RST")
                || io_pin.contains("T_TXBIT")
            {
                let suffix = io_pin.strip_prefix("CLB2PHY_").unwrap();
                let io_pin = suffix.trim_end_matches(|x: char| x.is_ascii_digit());
                let idx = suffix.strip_prefix(io_pin).unwrap();
                let idx: usize = idx.parse().unwrap();
                let iob_idx = nidx * 6 + idx;
                bels_bitslice[iob_idx].0.pins.insert(io_pin.into(), pin);
                bels_bitslice[iob_idx].1.pins.insert(io_pin.into(), pn);
            } else if io_pin.contains("DYNAMIC_DCI_TS")
                || io_pin.contains("IBUF_DISABLE")
                || io_pin.contains("KEEPER_EN")
                || io_pin.contains("IOB2CLB_DFT")
            {
                let suffix = io_pin
                    .strip_prefix("CLB2PHY_")
                    .or(io_pin.strip_prefix("IOB2CLB_"))
                    .unwrap();
                let io_pin = suffix.trim_end_matches(|x: char| x.is_ascii_digit());
                let idx = suffix.strip_prefix(io_pin).unwrap();
                let idx: usize = idx.parse().unwrap();
                let iob_idx = nidx * 6 + idx;
                bels_iob[iob_idx].0.pins.insert(io_pin.into(), pin);
                bels_iob[iob_idx].1.pins.insert(io_pin.into(), pn);
            } else if let Some(suffix) = io_pin
                .strip_prefix("PHY2CLB_LP_I_")
                .or(io_pin.strip_prefix("CLB2PHY_LP_TX_O_OR_PU_EN_"))
                .or(io_pin.strip_prefix("CLB2PHY_LP_TX_T_OR_PD_EN_"))
                .or(io_pin.strip_prefix("CLB2PHY_PD_EN_"))
            {
                let (iob_idx, pair_idx) = suffix.split_once('_').unwrap();
                let iob_idx: usize = iob_idx.parse().unwrap();
                let pair_idx: usize = pair_idx.parse().unwrap();
                let iob_idx = nidx * 6 + pair_idx * 2 + iob_idx;
                let io_pin = if io_pin.contains("_LP_I_") {
                    "LP_I"
                } else if io_pin.contains("_PD_EN_") {
                    "PD_EN"
                } else if io_pin.contains("_PU_EN_") {
                    "PU_EN"
                } else {
                    unreachable!()
                };
                bels_iob[iob_idx].0.pins.insert(io_pin.into(), pin);
                bels_iob[iob_idx].1.pins.insert(io_pin.into(), pn);
            } else if let Some(suffix) = io_pin.strip_prefix("CLB2PHY_LP_RX_DIS_OR_TERM_EN") {
                let pair_idx: usize = suffix.parse().unwrap();
                let pair_idx = nidx * 3 + pair_idx;
                bels_pair[pair_idx].0.pins.insert("TERM_EN".into(), pin);
                bels_pair[pair_idx].1.pins.insert("TERM_EN".into(), pn);
            } else {
                let phy_pin = io_pin
                    .strip_prefix("CLB2PHY_")
                    .or(io_pin.strip_prefix("CLB2MC_"))
                    .or(io_pin.strip_prefix("PHY2CLB_"))
                    .unwrap();
                bels_phy[nidx].0.pins.insert(phy_pin.into(), pin);
                bels_phy[nidx].1.pins.insert(phy_pin.into(), pn);
            }
        }

        for i in 0..11 {
            let spin = &format!("IF_XPIO_MMCM_DMC_OABUT_XPIO_PHY_CLK{i}");
            let rpin = "FIFO_WRCLK";
            let pin = bel_lpddrmc.pins.remove(spin).unwrap();
            let pn = naming_lpddrmc.pins[spin].clone();
            bels_phy[i].0.pins.insert(rpin.into(), pin);
            bels_phy[i].1.pins.insert(rpin.into(), pn);
        }

        for i in 0..4 {
            let nidx = [4, 5, 10, 6][i];
            let spin = &format!("IF_XPIO_MMCM_DMC_OABUT_XPIO_CCIO{i}");
            let rpin = "CCIO";
            let pin = bel_lpddrmc.pins.remove(spin).unwrap();
            let pn = naming_lpddrmc.pins[spin].clone();
            bels_phy[nidx].0.pins.insert(rpin.into(), pin);
            bels_phy[nidx].1.pins.insert(rpin.into(), pn);
        }

        for (oname, nname) in renames {
            let pin = bel_lpddrmc.pins.remove(&oname).unwrap();
            bel_lpddrmc.pins.insert(nname.clone(), pin);
            let pn = naming_lpddrmc.pins.remove(&oname).unwrap();
            naming_lpddrmc.pins.insert(nname, pn);
        }

        bel_lpddrmc
            .pins
            .retain(|pname, _| !pname.starts_with("IF_HSM_CLB2PHY_"));

        for (bslot, (bel, bn)) in bslots::XP5IOB.into_iter().zip(bels_iob) {
            self.builder
                .insert_tcls_bel(tcls::XP5IO, bslot, BelInfo::Legacy(bel));
            self.builder.insert_bel_naming(naming, bslot, bn);
        }
        for (bslot, (bel, bn)) in bslots::XP5IOB_PAIR.into_iter().zip(bels_pair) {
            self.builder
                .insert_tcls_bel(tcls::XP5IO, bslot, BelInfo::Legacy(bel));
            self.builder.insert_bel_naming(naming, bslot, bn);
        }
        for (bslot, (bel, bn)) in bslots::XP5IO_BITSLICE.into_iter().zip(bels_bitslice) {
            self.builder
                .insert_tcls_bel(tcls::XP5IO, bslot, BelInfo::Legacy(bel));
            self.builder.insert_bel_naming(naming, bslot, bn);
        }
        for (bslot, (bel, bn)) in bslots::X5PHY.into_iter().zip(bels_phy) {
            self.builder
                .insert_tcls_bel(tcls::XP5IO, bslot, BelInfo::Legacy(bel));
            self.builder.insert_bel_naming(naming, bslot, bn);
        }
        self.builder
            .insert_tcls_bel(tcls::XP5IO, bslots::LPDDRMC, BelInfo::Legacy(bel_lpddrmc));
        self.builder
            .insert_bel_naming(naming, bslots::LPDDRMC, naming_lpddrmc);
    }

    fn fill_tiles_rclk(&mut self) {
        for tkn in [
            "RCLK_CLEL_L_L",
            "RCLK_CLEL_L_R",
            "RCLK_CLEM_L",
            "RCLK_CLEM_DMC_L",
            "RCLK_CLEM_R",
            "RCLK_LAG_L",
            "RCLK_LAG_R",
            "RCLK_LAG_DMC_L",
            "RCLK_DSP_INTF_L",
            "RCLK_DSP_INTF_R",
            "RCLK_RCLK_DSP_INTF_DC12_L_FT",
            "RCLK_RCLK_DSP_INTF_DC12_R_FT",
            "RCLK_BRAM_INTF_L",
            "RCLK_BRAM_INTF_TD_L",
            "RCLK_BRAM_INTF_TD_R",
            "RCLK_RCLK_URAM_INTF_L_FT",
        ] {
            self.builder
                .extra_name_tile(tkn, "CLK_VDISTR_BOT", wires::GCLK_VDISTR_S[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VDISTR_TOP", wires::GCLK_VDISTR_N[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VROUTE_BOT", wires::GCLK_VROUTE_S[0]);
            self.builder
                .extra_name_tile(tkn, "CLK_VROUTE_TOP", wires::GCLK_VROUTE_N[0]);
            for i in 0..4 {
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
                    format!("CLK_CMT_DRVR_TRI_ESD_{ii}_CLK_OUT_SCHMITT_B", ii = i * 4),
                    wires::GCLK_VDISTR_S_BUF[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!(
                        "CLK_CMT_DRVR_TRI_ESD_{ii}_CLK_OUT_SCHMITT_B",
                        ii = i * 4 + 1
                    ),
                    wires::GCLK_VDISTR_N_BUF[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!(
                        "CLK_CMT_DRVR_TRI_ESD_{ii}_CLK_OUT_SCHMITT_B",
                        ii = i * 4 + 2
                    ),
                    wires::GCLK_VROUTE_S_BUF[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!(
                        "CLK_CMT_DRVR_TRI_ESD_{ii}_CLK_OUT_SCHMITT_B",
                        ii = i * 4 + 3
                    ),
                    wires::GCLK_VROUTE_N_BUF[i],
                );

                let idx = i * 2;
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFCE_ROW_FSR_{idx}_CLK_IN"),
                    wires::IMUX_BUFCE_ROW_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_BUFCE_ROW_FSR_{idx}_CLK_OUT"),
                    wires::OUT_BUFCE_ROW_RCLK[i],
                );

                let idx = i * 2 + 1;
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_TEST_BUF_SITE_{idx}_CLK_IN"),
                    wires::GCLK_HDISTR_RCLK[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_TEST_BUF_SITE_{idx}_CLK_OUT"),
                    wires::GCLK_HDISTR_TEST_RCLK[i],
                );
            }
        }

        for (tcid, naming, tkn) in [
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "PCIE4_PCIE4_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "PCIE4C_PCIE4C_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "PCIE4CE_PCIE4CE_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CMAC",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "ILKN_ILKN_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "DFE_DFE_TILEA_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "DFE_DFE_TILEB_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "DFE_DFE_TILEE_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "DFE_DFE_TILEG_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CFG_CONFIG",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CSEC_CONFIG_FT",
            ),
            (
                tcls::RCLK_HROUTE_SPLITTER_HARD,
                "RCLK_HROUTE_SPLITTER_HARD",
                "CSEC_CONFIG_VER2_FT",
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
            let tkn = "RCLK_DSP_INTF_CLKBUF_L";
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

        self.builder
            .extra_name("CLK_HROUTE_CORE_OPT", wires::GCLK_HROUTE_RCLK[0]);
        self.builder
            .extra_name("CLK_HROUTE_CORE_OPT0", wires::GCLK_HROUTE_RCLK[0]);
        self.builder
            .extra_name("CLK_HROUTE_CORE_OPT1", wires::GCLK_HROUTE_RCLK[1]);
        self.builder
            .extra_name("CLK_HROUTE_CORE_OPT2", wires::GCLK_HROUTE_RCLK[2]);
        self.builder
            .extra_name("CLK_HROUTE_CORE_OPT3", wires::GCLK_HROUTE_RCLK[3]);
        for (tcid, tkn) in [
            (tcls::RCLK_V_SINGLE_CLE, "RCLK_CLEL_L_L"),
            (tcls::RCLK_V_SINGLE_CLE, "RCLK_CLEL_L_R"),
            (tcls::RCLK_V_SINGLE_CLE, "RCLK_CLEM_L"),
            (tcls::RCLK_V_SINGLE_CLE, "RCLK_CLEM_DMC_L"),
            (tcls::RCLK_V_SINGLE_CLE, "RCLK_CLEM_R"),
            (tcls::RCLK_V_SINGLE_LAG, "RCLK_LAG_L"),
            (tcls::RCLK_V_SINGLE_LAG, "RCLK_LAG_R"),
            (tcls::RCLK_V_SINGLE_LAG, "RCLK_LAG_DMC_L"),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let is_alt = self.dev_naming.rclk_alt_pins[tkn];
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let int_xy = xy.delta(if tkn.starts_with("RCLK_LAG") { 2 } else { 1 }, 0);
                let bels = vec![
                    self.builder
                        .bel_xy(bslots::BUFCE_ROW_RCLK[0], "BUFCE_ROW_FSR", 0, 0)
                        .pin_rename("CE_PRE_OPTINV", "CE")
                        .pins_name_only(&["CLK_OUT_OPT_DLY"]),
                    self.builder
                        .bel_xy(bslots::GCLK_TEST_BUF_RCLK[0], "GCLK_TEST_BUFE3", 0, 0),
                ];
                self.builder
                    .xtile_id(
                        tcid,
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
                    .optin_muxes(&wires::GCLK_VDISTR_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_BUF[..])
                    .ref_xlat(int_xy, &[Some(0), None, None, None], rclk_int)
                    .bels(bels)
                    .extract();
            }
        }

        for tkn in [
            "RCLK_DSP_INTF_L",
            "RCLK_DSP_INTF_R",
            "RCLK_RCLK_DSP_INTF_DC12_L_FT",
            "RCLK_RCLK_DSP_INTF_DC12_R_FT",
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let is_alt = self.dev_naming.rclk_alt_pins[tkn];
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let int_xy = xy.delta(-1, 0);
                let mut bels = vec![];
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFCE_ROW_RCLK[i], "BUFCE_ROW_FSR", i, 0)
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
                        tcls::RCLK_V_DOUBLE_DSP,
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
                    .optin_muxes(&wires::GCLK_VDISTR_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_BUF[..])
                    .ref_xlat(int_xy, &[None, Some(0), None, None], rclk_int)
                    .bels(bels)
                    .extract();
            }
        }

        for (tcid, tkn) in [
            (tcls::RCLK_V_QUAD_BRAM, "RCLK_BRAM_INTF_L"),
            (tcls::RCLK_V_QUAD_BRAM, "RCLK_BRAM_INTF_TD_L"),
            (tcls::RCLK_V_QUAD_BRAM, "RCLK_BRAM_INTF_TD_R"),
            (tcls::RCLK_V_QUAD_URAM, "RCLK_RCLK_URAM_INTF_L_FT"),
        ] {
            if let Some(&xy) = self.builder.rd.tiles_by_kind_name(tkn).iter().next() {
                let is_alt = self.dev_naming.rclk_alt_pins[tkn];
                let is_uram = tkn == "RCLK_RCLK_URAM_INTF_L_FT";
                let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK_INT");
                let int_xy = xy.delta(if is_uram { 3 } else { 2 }, 0);
                let mut bels = vec![];
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFCE_ROW_RCLK[i], "BUFCE_ROW_FSR", i, 0)
                            .pin_rename("CE_PRE_OPTINV", "CE")
                            .pins_name_only(&["CLK_OUT_OPT_DLY"]),
                    );
                }
                for i in 0..4 {
                    bels.push(self.builder.bel_xy(
                        bslots::GCLK_TEST_BUF_RCLK[i],
                        "GCLK_TEST_BUFE3",
                        i,
                        0,
                    ));
                }
                for (i, x, y) in [(0, 0, 0), (1, 0, 1), (2, 1, 0)] {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::VBUS_SWITCH[i], "VBUS_SWITCH", x, y)
                            .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                    );
                }
                let tcname = self.builder.db[tcid].name.clone();
                self.builder
                    .xtile_id(
                        tcid,
                        if is_alt {
                            format!("{tcname}_ALT")
                        } else {
                            tcname
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
                    .optin_muxes(&wires::GCLK_VDISTR_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VDISTR_N_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_MUX[..])
                    .optin_muxes(&wires::GCLK_VROUTE_S_BUF[..])
                    .optin_muxes(&wires::GCLK_VROUTE_N_BUF[..])
                    .ref_xlat(int_xy, &[Some(0), None, None, None], rclk_int)
                    .bels(bels)
                    .extract();
            }
        }
    }

    fn fill_tiles_gt(&mut self) {
        for (tcid, naming, tkn, side, base_bufg_gt, out_gt_clk) in [
            (
                tcls::GTH,
                "GTH_L",
                "GTH_QUAD_LEFT",
                Dir::W,
                0,
                [
                    "GTH_COMMON_BLH_46_REFCLK2HROW0",
                    "GTH_CHANNEL_BLH_48_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_47_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_48_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_47_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_48_DMONOUTCLK_INT",
                    "GTH_CHANNEL_BLH_47_DMONOUTCLK_INT",
                    "GTH_COMMON_BLH_46_REFCLK2HROW1",
                    "GTH_CHANNEL_BLH_44_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_45_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_44_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_45_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_44_DMONOUTCLK_INT",
                    "GTH_CHANNEL_BLH_45_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTH,
                "GTH_R",
                "GTH_QUAD_RIGHT",
                Dir::E,
                5,
                [
                    "GTH_COMMON_BLH_2_REFCLK2HROW0",
                    "GTH_CHANNEL_BLH_4_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_3_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_4_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_3_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_4_DMONOUTCLK_INT",
                    "GTH_CHANNEL_BLH_3_DMONOUTCLK_INT",
                    "GTH_COMMON_BLH_2_REFCLK2HROW1",
                    "GTH_CHANNEL_BLH_0_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_1_TXOUTCLK_INT",
                    "GTH_CHANNEL_BLH_0_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_1_RXRECCLK_INT",
                    "GTH_CHANNEL_BLH_0_DMONOUTCLK_INT",
                    "GTH_CHANNEL_BLH_1_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTY,
                "GTY_L",
                "GTY_L",
                Dir::W,
                0,
                [
                    "GTY_COMMON_BLH_44_REFCLK2HROW0",
                    "GTY_CHANNEL_BLH_48_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_46_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_48_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_46_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_48_DMONOUTCLK_INT",
                    "GTY_CHANNEL_BLH_46_DMONOUTCLK_INT",
                    "GTY_COMMON_BLH_44_REFCLK2HROW1",
                    "GTY_CHANNEL_BLH_45_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_47_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_45_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_47_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_45_DMONOUTCLK_INT",
                    "GTY_CHANNEL_BLH_47_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTY,
                "GTY_R",
                "GTY_R",
                Dir::E,
                5,
                [
                    "GTY_COMMON_BLH_0_REFCLK2HROW0",
                    "GTY_CHANNEL_BLH_4_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_2_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_4_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_2_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_4_DMONOUTCLK_INT",
                    "GTY_CHANNEL_BLH_2_DMONOUTCLK_INT",
                    "GTY_COMMON_BLH_0_REFCLK2HROW1",
                    "GTY_CHANNEL_BLH_1_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_3_TXOUTCLK_INT",
                    "GTY_CHANNEL_BLH_1_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_3_RXRECCLK_INT",
                    "GTY_CHANNEL_BLH_1_DMONOUTCLK_INT",
                    "GTY_CHANNEL_BLH_3_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTF,
                "GTF_L",
                "GTFY_QUAD_LEFT_FT",
                Dir::W,
                0,
                [
                    "GTF_COMMON_BLH_44_REFCLK2HROW0",
                    "GTF_CHANNEL_BLH_48_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_46_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_48_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_46_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_48_DMONOUTCLK_INT",
                    "GTF_CHANNEL_BLH_46_DMONOUTCLK_INT",
                    "GTF_COMMON_BLH_44_REFCLK2HROW1",
                    "GTF_CHANNEL_BLH_45_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_47_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_45_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_47_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_45_DMONOUTCLK_INT",
                    "GTF_CHANNEL_BLH_47_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTF,
                "GTF_R",
                "GTFY_QUAD_RIGHT_FT",
                Dir::E,
                5,
                [
                    "GTF_COMMON_BLH_0_REFCLK2HROW0",
                    "GTF_CHANNEL_BLH_4_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_2_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_4_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_2_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_4_DMONOUTCLK_INT",
                    "GTF_CHANNEL_BLH_2_DMONOUTCLK_INT",
                    "GTF_COMMON_BLH_0_REFCLK2HROW1",
                    "GTF_CHANNEL_BLH_1_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_3_TXOUTCLK_INT",
                    "GTF_CHANNEL_BLH_1_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_3_RXRECCLK_INT",
                    "GTF_CHANNEL_BLH_1_DMONOUTCLK_INT",
                    "GTF_CHANNEL_BLH_3_DMONOUTCLK_INT",
                ]
                .as_slice(),
            ),
            (
                tcls::GTM,
                "GTM_L",
                "GTM_DUAL_LEFT_FT",
                Dir::W,
                0,
                [
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT0",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT1",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT2",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT3",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT4",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_BOT5",
                    "CLK_BUFG_GT_SYNC_BOTH_30_CLK_IN",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP0",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP1",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP2",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP3",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP4",
                    "GTM_DUAL_SITE_44_CLK_BUFGT_CLK_IN_TOP5",
                    "CLK_BUFG_GT_SYNC_BOTH_37_CLK_IN",
                ]
                .as_slice(),
            ),
            (
                tcls::GTM,
                "GTM_R",
                "GTM_DUAL_RIGHT_FT",
                Dir::E,
                2,
                [
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT0",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT1",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT2",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT3",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT4",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_BOT5",
                    "CLK_BUFG_GT_SYNC_BOTH_32_CLK_IN",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP0",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP1",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP2",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP3",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP4",
                    "GTM_DUAL_SITE_0_CLK_BUFGT_CLK_IN_TOP5",
                    "CLK_BUFG_GT_SYNC_BOTH_39_CLK_IN",
                ]
                .as_slice(),
            ),
            (
                tcls::HSADC,
                "HSADC_R",
                "HSADC_HSADC_RIGHT_FT",
                Dir::E,
                1,
                [
                    "HSADC_HSADC_RIGHT_CORE_0_CLK_ADC",
                    "HSADC_HSADC_RIGHT_CORE_0_PLL_DMON_OUT",
                    "HSADC_HSADC_RIGHT_CORE_0_PLL_REFCLK_OUT",
                    "HSADC_HSADC_RIGHT_CORE_0_CLK_ADC_SPARE",
                ]
                .as_slice(),
            ),
            (
                tcls::HSDAC,
                "HSDAC_R",
                "HSDAC_HSDAC_RIGHT_FT",
                Dir::E,
                1,
                [
                    "HSDAC_HSDAC_RIGHT_CORE_0_CLK_DAC",
                    "HSDAC_HSDAC_RIGHT_CORE_0_PLL_DMON_OUT",
                    "HSDAC_HSDAC_RIGHT_CORE_0_PLL_REFCLK_OUT",
                    "HSDAC_HSDAC_RIGHT_CORE_0_CLK_DAC_SPARE",
                ]
                .as_slice(),
            ),
            (
                tcls::RFADC,
                "RFADC_R",
                "RFADC_RFADC_RIGHT_FT",
                Dir::E,
                1,
                [
                    "RFADC_RFADC_RIGHT_CORE_0_CLK_ADC",
                    "RFADC_RFADC_RIGHT_CORE_0_PLL_DMON_OUT",
                    "RFADC_RFADC_RIGHT_CORE_0_PLL_REFCLK_OUT",
                    "RFADC_RFADC_RIGHT_CORE_0_CLK_ADC_SPARE",
                ]
                .as_slice(),
            ),
            (
                tcls::RFDAC,
                "RFDAC_R",
                "RFDAC_RFDAC_RIGHT_FT",
                Dir::E,
                1,
                [
                    "RFDAC_RFDAC_RIGHT_CORE_0_CLK_DAC",
                    "RFDAC_RFDAC_RIGHT_CORE_0_PLL_DMON_OUT",
                    "RFDAC_RFDAC_RIGHT_CORE_0_PLL_REFCLK_OUT",
                    "RFDAC_RFDAC_RIGHT_CORE_0_CLK_DAC_SPARE",
                ]
                .as_slice(),
            ),
        ] {
            let is_gt = matches!(tcid, tcls::GTH | tcls::GTY | tcls::GTF | tcls::GTM);
            for (i, &n) in out_gt_clk.iter().enumerate() {
                self.builder
                    .extra_name_tile_cell(tkn, n, 30, wires::OUT_GT_CLK[i]);
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
                let idx = base_bufg_gt + i;
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
                if is_gt {
                    let bi = [
                        (0, 1, 12),
                        (27, 28, 29),
                        (43, 44, 46),
                        (47, 48, 49),
                        (50, 51, 52),
                        (53, 54, 55),
                        (57, 58, 59),
                        (60, 61, 62),
                        (63, 64, 65),
                        (66, 68, 69),
                        (23, 34, 45),
                        (56, 67, 70),
                        (71, 2, 3),
                        (4, 5, 6),
                        (7, 8, 9),
                        (10, 11, 13),
                        (14, 15, 16),
                        (17, 18, 19),
                        (20, 21, 22),
                        (24, 25, 26),
                        (30, 31, 32),
                        (33, 35, 36),
                        (37, 38, 39),
                        (40, 41, 42),
                    ][i];
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.0),
                        30,
                        wires::IMUX_BUFG_GT_CE_DUMMY0[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.1),
                        30,
                        wires::IMUX_BUFG_GT_CLK_DUMMY0[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.2),
                        30,
                        wires::IMUX_BUFG_GT_RST_DUMMY0[i],
                    );
                } else {
                    let bi = [
                        (20, 21, 82),
                        (137, 138, 149),
                        (213, 214, 226),
                        (227, 228, 239),
                        (240, 241, 252),
                        (253, 254, 265),
                        (267, 268, 279),
                        (280, 281, 292),
                        (293, 294, 305),
                        (306, 318, 329),
                        (123, 164, 225),
                        (266, 307, 330),
                        (331, 32, 43),
                        (44, 45, 56),
                        (57, 58, 69),
                        (70, 71, 83),
                        (84, 85, 96),
                        (97, 98, 109),
                        (110, 111, 122),
                        (124, 125, 136),
                        (150, 151, 162),
                        (163, 175, 186),
                        (187, 188, 199),
                        (200, 201, 212),
                    ][i];
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.0),
                        30,
                        wires::IMUX_BUFG_GT_CE_DUMMY0[i],
                    );
                    for (j, w) in [
                        wires::IMUX_BUFG_GT_CLK_DUMMY0,
                        wires::IMUX_BUFG_GT_CLK_DUMMY1,
                        wires::IMUX_BUFG_GT_CLK_DUMMY2,
                        wires::IMUX_BUFG_GT_CLK_DUMMY3,
                        wires::IMUX_BUFG_GT_CLK_DUMMY4,
                        wires::IMUX_BUFG_GT_CLK_DUMMY5,
                        wires::IMUX_BUFG_GT_CLK_DUMMY6,
                        wires::IMUX_BUFG_GT_CLK_DUMMY7,
                        wires::IMUX_BUFG_GT_CLK_DUMMY8,
                        wires::IMUX_BUFG_GT_CLK_DUMMY9,
                        wires::IMUX_BUFG_GT_CLK_DUMMY10,
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
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("VCC_WIRE{ii}", ii = bi.2),
                        30,
                        wires::IMUX_BUFG_GT_RST_DUMMY0[i],
                    );
                }
            }
            for i in 0..15 {
                let idx = base_bufg_gt + 24 + i;
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
                let mut bels = vec![];
                for i in 0..24 {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BUFG_GT[i], "BUFG_GT", 0, i)
                        .pin_rename("RST_PRE_OPTINV", "RST");
                    if !is_gt {
                        let bi = [
                            10, 43, 61, 64, 67, 70, 73, 76, 79, 13, 16, 19, 22, 25, 28, 31, 34, 37,
                            40, 46, 49, 52, 55, 58,
                        ][i];
                        bel = bel
                            .pin_name_only("DIV0", 1)
                            .pin_name_only("DIV1", 1)
                            .pin_name_only("DIV2", 1)
                            .extra_wire("DIV0_DUMMY", &[format!("GND_WIRE{bi}")])
                            .extra_wire("DIV1_DUMMY", &[format!("GND_WIRE{ii}", ii = bi + 1)])
                            .extra_wire("DIV2_DUMMY", &[format!("GND_WIRE{ii}", ii = bi + 2)]);
                    }
                    bels.push(bel);
                }
                for i in 0..15 {
                    let mut bel =
                        self.builder
                            .bel_xy(bslots::BUFG_GT_SYNC[i], "BUFG_GT_SYNC", 0, i);
                    if !is_gt && (4..14).contains(&i) {
                        bel = bel.pins_name_only(&["CLK_IN", "CE_IN", "RST_IN"]);
                    }
                    if tcid == tcls::GTM && matches!(i, 6 | 13) {
                        bel = bel.extra_wire(
                            "CLK_IN",
                            &[format!(
                                "CLK_BUFG_GT_SYNC_BOTH_{ii}_CLK_IN",
                                ii = base_bufg_gt + 24 + i,
                            )],
                        );
                    }
                    bels.push(bel);
                }
                for i in 0..5 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::ABUS_SWITCH_GT[i], "ABUS_SWITCH", 0, i)
                            .pin_rename("TEST_ANALOGBUS_SEL_B", "TEST_ANALOGBUS_SEL"),
                    );
                }

                if tcid == tcls::GTM {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::GTM_DUAL, "GTM_DUAL", 0, 0)
                        .pins_name_only(&[
                            "HROW_TEST_CK_SA",
                            "MGTREFCLK_CLEAN",
                            "RXRECCLK0_INT",
                            "RXRECCLK1_INT",
                            "REFCLKPDB_SA",
                            "RCALSEL0",
                            "RCALSEL1",
                            "REFCLK_DIST2PLL0",
                            "REFCLK_DIST2PLL1",
                            "REFCLK2HROW",
                        ])
                        .extra_wire("SOUTHCLKOUT", &["SOUTHCLKOUT"])
                        .extra_wire("SOUTHCLKOUT_DUMMY0", &["VCC_WIRE72"])
                        .extra_wire("SOUTHCLKOUT_DUMMY1", &["VCC_WIRE73"])
                        .extra_wire("NORTHCLKOUT", &["NORTHCLKOUT"])
                        .extra_wire("NORTHCLKOUT_DUMMY0", &["VCC_WIRE74"])
                        .extra_wire("NORTHCLKOUT_DUMMY1", &["VCC_WIRE75"])
                        .sub_xy("GTM_REFCLK", 0, 0);
                    for pin in [
                        "HROW_TEST_CK_FS",
                        "MGTREFCLK_CLEAN",
                        "REFCLK2HROW",
                        "REFCLKPDB_SA",
                        "RXRECCLK0_INT",
                        "RXRECCLK1_INT",
                        "RXRECCLK2_INT",
                        "RXRECCLK3_INT",
                    ] {
                        bel = bel
                            .pin_rename(pin, format!("GTM_REFCLK_{pin}"))
                            .pin_name_only(&format!("GTM_REFCLK_{pin}"), 0);
                    }
                    bels.push(bel);
                } else if is_gt {
                    let (common, channel) = match tcid {
                        tcls::GTH | tcls::GTY => (bslots::GT4_COMMON, bslots::GT4_CHANNEL),
                        tcls::GTF => (bslots::GTF_COMMON, bslots::GTF_CHANNEL),
                        _ => unreachable!(),
                    };
                    let pref = match tcid {
                        tcls::GTH => "GTHE4",
                        tcls::GTY => "GTYE4",
                        tcls::GTF => "GTF",
                        _ => unreachable!(),
                    };
                    for i in 0..4 {
                        bels.push(
                            self.builder
                                .bel_xy(channel[i], &format!("{pref}_CHANNEL"), 0, i)
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
                            .bel_xy(common, &format!("{pref}_COMMON"), 0, 0)
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
                                    "GTH_CHANNEL_BLH_44_NORTHREFCLK0",
                                    "GTF_CHANNEL_BLH_1_NORTHREFCLK0",
                                    "GTF_CHANNEL_BLH_45_NORTHREFCLK0",
                                    "GTY_CHANNEL_BLH_1_NORTHREFCLK0",
                                    "GTY_CHANNEL_BLH_45_NORTHREFCLK0",
                                ],
                            )
                            .extra_wire(
                                "NORTHREFCLK1",
                                &[
                                    "GTH_CHANNEL_BLH_0_NORTHREFCLK1",
                                    "GTH_CHANNEL_BLH_44_NORTHREFCLK1",
                                    "GTF_CHANNEL_BLH_1_NORTHREFCLK1",
                                    "GTF_CHANNEL_BLH_45_NORTHREFCLK1",
                                    "GTY_CHANNEL_BLH_1_NORTHREFCLK1",
                                    "GTY_CHANNEL_BLH_45_NORTHREFCLK1",
                                ],
                            )
                            .extra_wire(
                                "SOUTHREFCLK0",
                                &[
                                    "GTH_CHANNEL_BLH_0_SOUTHREFCLK0",
                                    "GTH_CHANNEL_BLH_44_SOUTHREFCLK0",
                                    "GTF_CHANNEL_BLH_1_SOUTHREFCLK0",
                                    "GTF_CHANNEL_BLH_45_SOUTHREFCLK0",
                                    "GTY_CHANNEL_BLH_1_SOUTHREFCLK0",
                                    "GTY_CHANNEL_BLH_45_SOUTHREFCLK0",
                                ],
                            )
                            .extra_wire(
                                "SOUTHREFCLK1",
                                &[
                                    "GTH_CHANNEL_BLH_0_SOUTHREFCLK1",
                                    "GTH_CHANNEL_BLH_44_SOUTHREFCLK1",
                                    "GTF_CHANNEL_BLH_1_SOUTHREFCLK1",
                                    "GTF_CHANNEL_BLH_45_SOUTHREFCLK1",
                                    "GTY_CHANNEL_BLH_1_SOUTHREFCLK1",
                                    "GTY_CHANNEL_BLH_45_SOUTHREFCLK1",
                                ],
                            )
                            .extra_wire("VCC", &["VCC_WIRE"]),
                    );
                } else {
                    let (slot, kind) = match tcid {
                        tcls::HSADC => (bslots::HSADC, "HSADC"),
                        tcls::HSDAC => (bslots::HSDAC, "HSDAC"),
                        tcls::RFADC => (bslots::HSADC, "RFADC"),
                        tcls::RFDAC => (bslots::HSDAC, "RFDAC"),
                        _ => unreachable!(),
                    };
                    let mut bel = self
                        .builder
                        .bel_xy(slot, kind, 0, 0)
                        .pins_name_only(&["SYSREF_OUT_SOUTH_P", "SYSREF_OUT_NORTH_P"])
                        .pin_name_only("SYSREF_IN_SOUTH_P", 1)
                        .pin_name_only("SYSREF_IN_NORTH_P", 1);
                    if kind.starts_with("RF") {
                        bel = bel
                            .pins_name_only(&[
                                "CLK_DISTR_IN_NORTH",
                                "CLK_DISTR_IN_SOUTH",
                                "CLK_DISTR_OUT_NORTH",
                                "CLK_DISTR_OUT_SOUTH",
                                "T1_ALLOWED_SOUTH",
                                "T1_ALLOWED_NORTH",
                            ])
                            .pin_name_only("CLK_DISTR_IN_NORTH", 1)
                            .pin_name_only("CLK_DISTR_IN_SOUTH", 1)
                            .pin_name_only("T1_ALLOWED_NORTH", 1);
                    }
                    bels.push(bel);
                }
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
        rd,
        builder: IntBuilder::new(
            rd,
            bincode::decode_from_slice(defs::ultrascaleplus::INIT, bincode::config::standard())
                .unwrap()
                .0,
        ),
        sng_fixup_map: BTreeMap::new(),
        dev_naming,
    };

    maker
        .builder
        .inject_main_passes(DirMap::from_fn(|dir| match dir {
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
    maker.fill_tiles_bli();
    maker.fill_tiles_uram();
    maker.fill_tiles_hard();
    maker.fill_tiles_dfe();
    maker.fill_tiles_ps();
    maker.fill_tiles_cfg();
    maker.fill_tiles_hdio();
    maker.fill_tiles_cmt();
    maker.fill_tiles_xiphy();
    maker.fill_tiles_hpio();
    maker.fill_tiles_xp5io();
    maker.fill_tiles_rclk();
    maker.fill_tiles_gt();

    maker.builder.build()
}
