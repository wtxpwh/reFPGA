use std::collections::HashMap;

use prjcombine_entity::id::EntityStaticRange;
use prjcombine_interconnect::{
    db::{WireSlotId, WireSlotIdExt},
    dir::{Dir, DirH},
};
use prjcombine_re_xilinx_rawdump::TkSiteSlot;
use prjcombine_versal::defs::{wiredata, wires};

use crate::int::{
    IntBuilderExt, IntMaker,
    intf::{BLI_CLE_INTF_KINDS, INTF_KINDS},
};

impl<'rd> IntMaker<'rd> {
    fn fill_wires_long(&mut self) {
        for (fwd, ll, wires) in [
            (Dir::W, 6, wiredata::X6_W.as_slice()),
            (Dir::E, 6, wiredata::X6_E.as_slice()),
            (Dir::S, 7, wiredata::X7_S.as_slice()),
            (Dir::N, 7, wiredata::X7_N.as_slice()),
            (Dir::W, 10, wiredata::X10_W.as_slice()),
            (Dir::E, 10, wiredata::X10_E.as_slice()),
            (Dir::S, 12, wiredata::X12_S.as_slice()),
            (Dir::N, 12, wiredata::X12_N.as_slice()),
        ] {
            let l = wires.len() - 1;
            for i in 0..8 {
                self.builder
                    .wire_names(wires[0][i], &[format!("OUT_{fwd}{fwd}{ll}_BEG{i}")]);
                self.builder
                    .extra_name(format!("IN_{fwd}{fwd}{ll}_END{i}"), wires[l][i]);
            }
        }
        self.builder.wire_names(wires::X6_E3_S0, &["IN_EE6_BLS_0"]);
        self.builder
            .wire_names(wires::X10_E5_N7, &["IN_EE10_BLN_7"]);
    }

    fn fill_wires_sdqnode(&mut self) {
        for (iq, q) in ['E', 'N', 'S', 'W'].into_iter().enumerate() {
            for i in 0..32 {
                let ii = iq * 32 + i;
                match (q, i) {
                    ('E', 0 | 2) | ('W', 0 | 2) | ('N', 0) => {
                        self.builder.wire_names_pair(
                            wires::SDQNODE[ii],
                            &[format!("OUT_{q}NODE_W_{i}"), format!("OUT_{q}NODE_E_{i}")],
                        );
                        self.builder.wire_names_pair(
                            wires::SDQNODE_S[ii],
                            &[
                                format!("IN_{q}NODE_W_BLS_{i}"),
                                format!("IN_{q}NODE_E_BLS_{i}"),
                            ],
                        );
                    }
                    ('E', 29 | 31) | ('W', 31) | ('S', 31) => {
                        self.builder.wire_names_pair(
                            wires::SDQNODE[ii],
                            &[format!("OUT_{q}NODE_W_{i}"), format!("OUT_{q}NODE_E_{i}")],
                        );
                        self.builder.wire_names_pair(
                            wires::SDQNODE_N[ii],
                            &[
                                format!("IN_{q}NODE_W_BLN_{i}"),
                                format!("IN_{q}NODE_E_BLN_{i}"),
                            ],
                        );
                    }
                    _ => {
                        // TODO not the true permutation
                        let a = [0, 11, 1, 2, 3, 4, 5, 6, 7, 8, 9, 13, 14, 15, 10, 12][i >> 1];
                        let aaw = a + 16 + iq * 32;
                        let aae = a + iq * 32;
                        let b = i & 1;
                        self.builder.wire_names_pair(
                            wires::SDQNODE[ii],
                            &[
                                format!("INT_NODE_SDQ_ATOM_{aaw}_INT_OUT{b}"),
                                format!("INT_NODE_SDQ_ATOM_{aae}_INT_OUT{b}"),
                            ],
                        );
                    }
                }
            }
        }
    }

    fn fill_wireset_sdq<const N: usize>(
        &mut self,
        dir: Dir,
        wires: &[EntityStaticRange<WireSlotId, N>],
    ) {
        let fwd = dir;
        let bwd = !dir;
        let length = wires.len() - 1;
        for i in 0..N {
            self.builder.wire_names_pair(
                wires[0][i],
                &[
                    format!("OUT_{fwd}{fwd}{length}_W_BEG{i}"),
                    format!("OUT_{fwd}{fwd}{length}_E_BEG{i}"),
                ],
            );
            if matches!(dir, Dir::H(_)) && length == 1 {
                self.builder
                    .extra_name(format!("IF_HBUS_{fwd}BUS{i}"), wires[0][i]);
                self.builder
                    .extra_name(format!("IF_HBUS_{bwd}_{fwd}BUS{i}"), wires[0][i]);
            }
            for j in 1..=length {
                match (fwd, length, j) {
                    (Dir::H(_), 2, 1) => {
                        for &(side, tkn, _, _, term, _) in INTF_KINDS {
                            if term && side == DirH::W {
                                let ii = i + 24;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            } else {
                                let ii = i + 24;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            }
                            if term && side == DirH::E {
                                let ii = i + 16;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            } else {
                                let ii = i + 16;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            }
                        }
                    }
                    (Dir::H(_), 4, 3) => {
                        for &(side, tkn, _, _, term, _) in INTF_KINDS {
                            if term && side == DirH::W {
                                let ii = i + 56;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            } else {
                                let ii = i + 56;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            }
                            if term && side == DirH::E {
                                let ii = i + 40;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            } else {
                                let ii = i + 40;
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                                self.builder.extra_name_tile(
                                    tkn,
                                    format!("IF_HBUS_{bwd}_{fwd}BUS{ii}"),
                                    wires[j][i],
                                );
                            }
                        }
                    }
                    (Dir::S, 1, 1) => {
                        let ii = i;
                        self.builder
                            .extra_name(format!("IF_INT_VSINGLE{ii}"), wires[j][i]);
                    }
                    (Dir::N, 1, 1) => {
                        let ii = i + 16;
                        self.builder
                            .extra_name(format!("IF_INT_VSINGLE{ii}"), wires[j][i]);
                    }
                    (Dir::N, 2, 1) => {
                        let ii = i + 32;
                        self.builder
                            .extra_name_cell(format!("IF_VBUS_S_NBUS{ii}"), 1, wires[j][i]);
                        let ii = i + 48;
                        self.builder
                            .extra_name_cell(format!("IF_VBUS_S_NBUS{ii}"), 0, wires[j][i]);
                    }
                    (Dir::N, 4, 1) => {
                        let ii = i + 64;
                        self.builder
                            .extra_name_cell(format!("IF_VBUS_S_NBUS{ii}"), 1, wires[j][i]);
                        let ii = i + 96;
                        self.builder
                            .extra_name_cell(format!("IF_VBUS_S_NBUS{ii}"), 0, wires[j][i]);
                    }
                    _ => (),
                }
            }
            let swz = [0, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1, 2, 3, 4, 5, 6][i];
            if length == 1 && fwd == Dir::E {
                self.builder.extra_name_cell(
                    format!("INT_SDQ_RED_ATOM_{ii}_INT_OUT0", ii = 16 + swz),
                    0,
                    wires[0][i],
                );
                self.sng_fixup_map
                    .insert(wires[0][i].cell(0), wires[1][i].cell(1));

                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_E_END{i}"),
                    0,
                    wires[length][i],
                );
                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_W_END{i}"),
                    1,
                    wires[length][i],
                );
            } else if length == 1 && fwd == Dir::W {
                self.builder.extra_name_cell(
                    format!("INT_SDQ_RED_ATOM_{ii}_INT_OUT0", ii = 32 + swz),
                    1,
                    wires[0][i],
                );
                self.sng_fixup_map
                    .insert(wires[0][i].cell(1), wires[1][i].cell(0));
                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_E_END{i}"),
                    0,
                    wires[length][i],
                );
                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_W_END{i}"),
                    1,
                    wires[length][i],
                );
            } else {
                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_W_END{i}"),
                    0,
                    wires[length][i],
                );
                self.builder.extra_name_cell(
                    format!("IN_{fwd}{fwd}{length}_E_END{i}"),
                    1,
                    wires[length][i],
                );
            }
        }
    }

    fn fill_wires_sdq(&mut self) {
        self.fill_wireset_sdq(Dir::W, &wiredata::X1_W);
        self.fill_wireset_sdq(Dir::E, &wiredata::X1_E);
        self.fill_wireset_sdq(Dir::S, &wiredata::X1_S);
        self.fill_wireset_sdq(Dir::N, &wiredata::X1_N);
        self.fill_wireset_sdq(Dir::W, &wiredata::X2_W);
        self.fill_wireset_sdq(Dir::E, &wiredata::X2_E);
        self.fill_wireset_sdq(Dir::S, &wiredata::X2_S);
        self.fill_wireset_sdq(Dir::N, &wiredata::X2_N);
        self.fill_wireset_sdq(Dir::W, &wiredata::X4_W);
        self.fill_wireset_sdq(Dir::E, &wiredata::X4_E);
        self.fill_wireset_sdq(Dir::S, &wiredata::X4_S);
        self.fill_wireset_sdq(Dir::N, &wiredata::X4_N);
        self.builder
            .extra_name_tile_cell("CLE_BC_CORE", "BNODE_TAP0", 1, wires::X1_W0_S0);
        self.builder
            .extra_name_tile_cell("CLE_BC_CORE_MX", "BNODE_TAP0", 1, wires::X1_W0_S0);
        self.builder
            .extra_name_tile_cell("SLL", "BNODE_TAP0", 1, wires::X1_W0_S0);
        self.builder
            .extra_name_tile_cell("SLL2", "BNODE_TAP0", 1, wires::X1_W0_S0);
        self.builder
            .extra_name("IF_LBC_N_BNODE_SOUTHBUS", wires::X4_E3_S0);
    }

    fn fill_wires_cle_imux(&mut self) {
        for i in 0..13 {
            self.builder.wire_names_pair(
                wires::IMUX_CLE_CTRL[i],
                &[format!("CTRL_L_B{i}"), format!("CTRL_R_B{i}")],
            );
            for tkn in ["CLE_W_CORE", "CLE_E_CORE"] {
                self.builder.extra_name_tile(
                    tkn,
                    match i {
                        0 => "CLE_SLICEL_TOP_0_CLK",
                        1 => "CLE_SLICEM_TOP_1_CLK",
                        2 => "CLE_SLICEL_TOP_0_RST",
                        3 => "CLE_SLICEM_TOP_1_RST",
                        4 => "CLE_SLICEL_TOP_0_CKEN1",
                        5 => "CLE_SLICEL_TOP_0_CKEN2",
                        6 => "CLE_SLICEL_TOP_0_CKEN3",
                        7 => "CLE_SLICEL_TOP_0_CKEN4",
                        8 => "CLE_SLICEM_TOP_1_WE",
                        9 => "CLE_SLICEM_TOP_1_CKEN1",
                        10 => "CLE_SLICEM_TOP_1_CKEN2",
                        11 => "CLE_SLICEM_TOP_1_CKEN3",
                        12 => "CLE_SLICEM_TOP_1_CKEN4",
                        _ => unreachable!(),
                    },
                    wires::IMUX_CLE_CTRL[i],
                );
            }
            for tkn in ["CLE_W_VR_CORE", "CLE_E_VR_CORE"] {
                self.builder.extra_name_tile(
                    tkn,
                    match i {
                        0 => "CLE_SLICEL_VR_TOP_0_CLK",
                        1 => "CLE_SLICEM_VR_TOP_1_CLK",
                        2 => "CLE_SLICEL_VR_TOP_0_RST",
                        3 => "CLE_SLICEM_VR_TOP_1_RST",
                        4 => "CLE_SLICEL_VR_TOP_0_CKEN1",
                        5 => "CLE_SLICEL_VR_TOP_0_CKEN2",
                        6 => "CLE_SLICEL_VR_TOP_0_CKEN3",
                        7 => "CLE_SLICEL_VR_TOP_0_CKEN4",
                        8 => "CLE_SLICEM_VR_TOP_1_WE",
                        9 => "CLE_SLICEM_VR_TOP_1_CKEN1",
                        10 => "CLE_SLICEM_VR_TOP_1_CKEN2",
                        11 => "CLE_SLICEM_VR_TOP_1_CKEN3",
                        12 => "CLE_SLICEM_VR_TOP_1_CKEN4",
                        _ => unreachable!(),
                    },
                    wires::IMUX_CLE_CTRL[i],
                );
            }
        }
    }

    pub fn fill_wires(&mut self) {
        // common wires
        self.builder.wire_names(wires::TIE_1, &["VCC_WIRE"]);
        self.fill_wires_long();

        for i in 0..6 {
            self.builder.extra_name_tile_cell(
                "SLL",
                format!("LAG_CASCOUT_TXI{i}"),
                0,
                wires::IMUX_LAG[i],
            );
            self.builder.extra_name_tile_cell(
                "SLL2",
                format!("LAG_CASCOUT_TXI{i}"),
                0,
                wires::IMUX_LAG[i],
            );
        }

        for i in 0..6 {
            self.builder.extra_name_tile_cell(
                "CLE_BC_CORE",
                format!("VCC_WIRE{i}"),
                0,
                wires::OUT_LAG[i],
            );
            self.builder.extra_name_tile_cell(
                "CLE_BC_CORE_MX",
                format!("VCC_WIRE{i}"),
                0,
                wires::OUT_LAG[i],
            );
            self.builder
                .extra_name_tile_cell("SLL", format!("LAG_OUT{i}"), 0, wires::OUT_LAG[i]);
            self.builder
                .extra_name_tile_cell("SLL2", format!("LAG_OUT{i}"), 0, wires::OUT_LAG[i]);
        }

        // wires belonging to interconnect left/right half-tiles
        for i in 0..48 {
            self.builder
                .mark_test_mux_in(wires::OUT_BEL[i], wires::OUT[i]);
            self.builder
                .mark_test_mux_in_test(wires::OUT_TEST[i], wires::OUT[i]);
            self.builder
                .extra_name_tile_cell("INT", format!("LOGIC_OUTS_W{i}"), 0, wires::OUT[i]);
            self.builder
                .extra_name_tile_cell("INT", format!("LOGIC_OUTS_E{i}"), 1, wires::OUT[i]);
            match i {
                1 | 4 | 5 => {
                    self.builder.wire_names_pair(
                        wires::OUT_S[i],
                        &[
                            format!("IN_LOGIC_OUTS_W_BLS_{i}"),
                            format!("IN_LOGIC_OUTS_E_BLS_{i}"),
                        ],
                    );
                }
                _ => (),
            }
            self.builder.test_mux_pass(wires::OUT_CLE[i]);
            for tkn in ["CLE_BC_CORE", "CLE_BC_CORE_MX", "SLL", "SLL2"] {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("LOGIC_OUTS_W{i}"),
                    0,
                    wires::OUT_CLE[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("LOGIC_OUTS_E{i}"),
                    1,
                    wires::OUT_CLE[i],
                );
            }
        }

        self.fill_wires_sdqnode();
        self.fill_wires_sdq();

        for i in 0..64 {
            for j in 0..2 {
                self.builder.wire_names_pair(
                    wires::INODE[i * 2 + j],
                    &[
                        format!(
                            "INT_NODE_IMUX_ATOM_{ii}_INT_OUT{j}",
                            ii = 32 + (i % 32) + (i / 32) * 64
                        ),
                        format!(
                            "INT_NODE_IMUX_ATOM_{ii}_INT_OUT{j}",
                            ii = (i % 32) + (i / 32) * 64
                        ),
                    ],
                );
            }
        }

        for i in 0..96 {
            self.builder.wire_names_pair(
                wires::IMUX_IMUX[i],
                &[format!("IMUX_B_W{i}"), format!("IMUX_B_E{i}")],
            );
        }

        for i in 0..32 {
            self.builder.extra_name_tile_cell(
                "INT",
                format!("BOUNCE_W{i}"),
                0,
                wires::IMUX_BOUNCE[i],
            );
            self.builder.extra_name_tile_cell(
                "INT",
                format!("BOUNCE_E{i}"),
                1,
                wires::IMUX_BOUNCE[i],
            );
        }

        for i in 0..64 {
            self.builder
                .extra_name_tile_cell("INT", format!("BNODE_W{i}"), 0, wires::BNODE[i]);
            self.builder
                .extra_name_tile_cell("INT", format!("BNODE_E{i}"), 1, wires::BNODE[i]);
        }

        self.builder
            .wire_names(wires::TEST_TMR_DFT, &["INTF_MUX2_TMR_GREEN_TMR_DFT"]);
        for &(_, tkn, _, _, _, _, _) in BLI_CLE_INTF_KINDS {
            for i in 0..4 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("INTF_MUX2_TMR_GREEN_{i}_TMR_DFT"),
                    wires::TEST_TMR_DFT,
                );
            }
        }

        for i in 0..32 {
            self.builder
                .extra_name_cell(format!("BNODE_OUTS_W{i}"), 0, wires::BNODE_CLE[i]);
            self.builder
                .extra_name_cell(format!("BNODE_OUTS_E{i}"), 1, wires::BNODE_CLE[i]);
        }

        for i in 0..12 {
            self.builder
                .extra_name_cell(format!("CNODE_OUTS_W{i}"), 0, wires::CNODE_CLE[i]);
            self.builder
                .extra_name_cell(format!("CNODE_OUTS_E{i}"), 1, wires::CNODE_CLE[i]);
        }

        self.fill_wires_cle_imux();

        for (i, w) in [
            wires::IMUX_BLI_CLE_IRI0_FAKE_CE,
            wires::IMUX_BLI_CLE_IRI1_FAKE_CE,
            wires::IMUX_BLI_CLE_IRI2_FAKE_CE,
            wires::IMUX_BLI_CLE_IRI3_FAKE_CE,
        ]
        .into_iter()
        .enumerate()
        {
            for j in 1..4 {
                for (_, tkn, _, _, _, _, _) in BLI_CLE_INTF_KINDS {
                    let idxs = match (i, j) {
                        (0, 1) => [24, 30, 39, 45],
                        (0, 2) => [25, 31, 40, 46],
                        (0, 3) => [26, 32, 41, 47],
                        (1, 1) => [0, 6, 15, 21],
                        (1, 2) => [1, 7, 16, 22],
                        (1, 3) => [2, 8, 17, 23],
                        (2, 1) => [27, 33, 36, 42],
                        (2, 2) => [28, 34, 37, 43],
                        (2, 3) => [29, 35, 38, 44],
                        (3, 1) => [3, 9, 12, 18],
                        (3, 2) => [4, 10, 13, 19],
                        (3, 3) => [5, 11, 14, 20],
                        _ => unreachable!(),
                    };
                    for idx in idxs {
                        self.builder
                            .extra_name_tile(tkn, format!("GND_WIRE{idx}"), w[j]);
                    }
                }
            }
        }

        for i in 0..16 {
            self.builder
                .extra_name_cell(format!("GCLK_B{i}"), 1, wires::GCLK_CLE[i]);
        }

        for (i, wclk, wrst, wce) in [
            (
                0,
                wires::IMUX_INTF_IRI0_CLK,
                wires::IMUX_INTF_IRI0_RST,
                wires::IMUX_INTF_IRI0_CE,
            ),
            (
                1,
                wires::IMUX_INTF_IRI1_CLK,
                wires::IMUX_INTF_IRI1_RST,
                wires::IMUX_INTF_IRI1_CE,
            ),
            (
                2,
                wires::IMUX_INTF_IRI2_CLK,
                wires::IMUX_INTF_IRI2_RST,
                wires::IMUX_INTF_IRI2_CE,
            ),
            (
                3,
                wires::IMUX_INTF_IRI3_CLK,
                wires::IMUX_INTF_IRI3_RST,
                wires::IMUX_INTF_IRI3_CE,
            ),
        ] {
            let rg = match i % 2 {
                0 => "RED",
                1 => "GREEN",
                _ => unreachable!(),
            };
            for (_, tkn, _, _, _, _) in INTF_KINDS {
                self.builder
                    .extra_name_tile(tkn, format!("INTF_IRI_QUADRANT_{rg}_{i}_CLK"), wclk);
            }
            for (_, tkn, _, _, _, _) in INTF_KINDS {
                self.builder
                    .extra_name_tile(tkn, format!("INTF_IRI_QUADRANT_{rg}_{i}_RST"), wrst);
            }
            for j in 0..4 {
                for (_, tkn, _, _, _, _) in INTF_KINDS {
                    self.builder.extra_name_tile(
                        tkn,
                        format!("INTF_IRI_QUADRANT_{rg}_{i}_CE{j}"),
                        wce[j],
                    );
                }
            }
        }

        for i in 0..12 {
            for j in 0..2 {
                self.builder.wire_names(
                    wires::CNODE_INTF[i * 2 + j],
                    &[format!("INTF_CNODE_ATOM_{i}_INT_OUT{j}")],
                );
            }
        }

        for i in 0..16 {
            self.builder
                .wire_names(wires::GCLK_INTF[i], &[format!("IF_GCLK_GCLK_B{i}")]);
        }

        for i in 0..20 {
            for j in 0..2 {
                self.builder.wire_names_pair(
                    wires::INODE_RCLK[2 * i + j],
                    &[
                        format!(
                            "INT_NODE_IMUX_ATOM_RCLK_{ii}_INT_OUT{j}",
                            ii = i % 10 + i / 10 * 20 + 10
                        ),
                        format!(
                            "INT_NODE_IMUX_ATOM_RCLK_{ii}_INT_OUT{j}",
                            ii = i % 10 + i / 10 * 20
                        ),
                    ],
                );
            }
        }

        for i in 0..2 {
            for j in 0..20 {
                self.builder.wire_names_pair(
                    wires::IMUX_RCLK[20 * i + j],
                    &[
                        format!("IF_INT2COE_W_INT_RCLK_TO_CLK_B_{i}_{j}"),
                        format!("IF_INT2COE_E_INT_RCLK_TO_CLK_B_{i}_{j}"),
                    ],
                );
            }
        }

        for iri in 0..4 {
            self.iri_wires.push(wiredata::IRI_CLK[iri]);
            self.iri_wires.push(wiredata::IRI_RST[iri]);
            for j in 0..4 {
                self.iri_wires.push(wiredata::IRI_CE[iri][j]);
            }
            for j in 0..24 {
                self.iri_wires.push(wiredata::IRI_IMUX[iri][j]);
                self.builder
                    .mark_delay(wiredata::IRI_IMUX[iri][j], wiredata::IRI_IMUX_DELAY[iri][j]);
            }
        }
        let slot_iri = self.rd.slot_kinds.get("IRI_QUAD").unwrap();
        for (_, tkn, tk) in &self.rd.tile_kinds {
            let mut buf_outs = HashMap::new();
            for &(wf, wt) in tk.pips.keys() {
                buf_outs.insert(wf, wt);
            }
            for (_, slot, site) in &tk.sites {
                let TkSiteSlot::Xy(slot, x, y) = *slot else {
                    continue;
                };
                if slot != slot_iri {
                    continue;
                }
                assert_eq!(x, 0);
                let sub = if tkn.starts_with("BLI_CLE_BOT") {
                    3 - y / 4
                } else {
                    y / 4
                } as usize;
                let iri = usize::from(y % 4);

                let name = &self.rd.wires[buf_outs[&site.pins["CLK_O"].wire.unwrap()]];
                self.builder
                    .extra_name_tile_cell(tkn, name, sub, wiredata::IRI_CLK[iri]);
                let name = &self.rd.wires[buf_outs[&site.pins["RST_O"].wire.unwrap()]];
                self.builder
                    .extra_name_tile_cell(tkn, name, sub, wiredata::IRI_RST[iri]);
                for j in 0..4 {
                    let name =
                        &self.rd.wires[buf_outs[&site.pins[&format!("CE{j}_O")].wire.unwrap()]];
                    self.builder
                        .extra_name_tile_cell(tkn, name, sub, wiredata::IRI_CE[iri][j]);
                }
                for j in 0..24 {
                    let name =
                        &self.rd.wires[buf_outs[&site.pins[&format!("IMUX_O{j}")].wire.unwrap()]];
                    self.builder
                        .extra_name_tile_cell(tkn, name, sub, wiredata::IRI_IMUX[iri][j]);
                }
            }
        }
    }

    pub fn fill_wires_bli(&mut self) {
        for (w, prefix) in [
            (wires::BLI_A_GRP0_IMUX.as_slice(), "BLI_GRP0_A_BLI_IMUX"),
            (
                [wires::BLI_A_GRP0_OUT_CTRL].as_slice(),
                "BLI_GRP0_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_A_GRP1_OUT.as_slice(),
                "BLI_GRP1_A_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_A_GRP1_OUT_CTRL.as_slice(),
                "BLI_GRP1_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (wires::BLI_A_GRP2_IMUX.as_slice(), "BLI_GRP2_A_BLI_IMUX"),
            (
                wires::BLI_A_GRP3_IMUX_CTRL.as_slice(),
                "BLI_GRP3_A_BLI_CTRL",
            ),
            (wires::BLI_B_GRP0_IMUX.as_slice(), "BLI_GRP0_B_BLI_IMUX"),
            (
                [wires::BLI_B_GRP0_OUT_CTRL].as_slice(),
                "BLI_GRP0_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_B_GRP1_OUT.as_slice(),
                "BLI_GRP1_B_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_B_GRP1_OUT_CTRL.as_slice(),
                "BLI_GRP1_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (wires::BLI_B_GRP2_IMUX.as_slice(), "BLI_GRP2_B_BLI_IMUX"),
            (
                wires::BLI_B_GRP3_IMUX_CTRL.as_slice(),
                "BLI_GRP3_B_BLI_CTRL",
            ),
            (wires::BLI_C_GRP0_IMUX.as_slice(), "BLI_GRP0_C_BLI_IMUX"),
            (
                [wires::BLI_C_GRP0_OUT_CTRL].as_slice(),
                "BLI_GRP0_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_C_GRP1_OUT.as_slice(),
                "BLI_GRP1_C_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_C_GRP1_OUT_CTRL.as_slice(),
                "BLI_GRP1_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (wires::BLI_C_GRP2_IMUX.as_slice(), "BLI_GRP2_C_BLI_IMUX"),
            (
                wires::BLI_C_GRP3_IMUX_CTRL.as_slice(),
                "BLI_GRP3_C_BLI_CTRL",
            ),
            (wires::BLI_D_GRP4_IMUX.as_slice(), "BLI_GRP4_D_BLI_IMUX"),
            (
                wires::BLI_D_GRP4_IMUX_BYP.as_slice(),
                "BLI_GRP4_D_BLI_IMUX_BYP",
            ),
            (
                wires::BLI_D_GRP4_OUT.as_slice(),
                "BLI_GRP4_D_BLI_LOGIC_OUTS",
            ),
            (wires::BLI_D_GRP5_IMUX.as_slice(), "BLI_GRP5_D_BLI_IMUX"),
            (
                wires::BLI_D_GRP5_IMUX_BYP.as_slice(),
                "BLI_GRP5_D_BLI_IMUX_BYP",
            ),
            (
                wires::BLI_D_GRP5_OUT.as_slice(),
                "BLI_GRP5_D_BLI_LOGIC_OUTS",
            ),
            (wires::BLI_D_GRP6_IMUX.as_slice(), "BLI_GRP6_D_BLI_IMUX"),
            (
                wires::BLI_D_GRP6_IMUX_BYP.as_slice(),
                "BLI_GRP6_D_BLI_IMUX_BYP",
            ),
            (
                wires::BLI_D_GRP6_OUT.as_slice(),
                "BLI_GRP6_D_BLI_LOGIC_OUTS",
            ),
            (wires::BLI_D_GRP7_IMUX.as_slice(), "BLI_GRP7_D_BLI_IMUX"),
            (
                wires::BLI_D_GRP7_IMUX_BYP.as_slice(),
                "BLI_GRP7_D_BLI_IMUX_BYP",
            ),
            (
                wires::BLI_D_GRP7_OUT.as_slice(),
                "BLI_GRP7_D_BLI_LOGIC_OUTS",
            ),
            (
                [wires::BLI_A_GRP0_OUT_CTRL_BEL].as_slice(),
                "HSR_GRP0_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_A_GRP1_OUT_BEL.as_slice(),
                "HSR_GRP1_A_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_A_GRP1_OUT_CTRL_BEL.as_slice(),
                "HSR_GRP1_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                [wires::BLI_B_GRP0_OUT_CTRL_BEL].as_slice(),
                "HSR_GRP0_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_B_GRP1_OUT_BEL.as_slice(),
                "HSR_GRP1_B_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_B_GRP1_OUT_CTRL_BEL.as_slice(),
                "HSR_GRP1_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                [wires::BLI_C_GRP0_OUT_CTRL_BEL].as_slice(),
                "HSR_GRP0_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_C_GRP1_OUT_BEL.as_slice(),
                "HSR_GRP1_C_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_C_GRP1_OUT_CTRL_BEL.as_slice(),
                "HSR_GRP1_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_D_GRP4_OUT_BEL.as_slice(),
                "HSR_GRP4_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP5_OUT_BEL.as_slice(),
                "HSR_GRP5_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP6_OUT_BEL.as_slice(),
                "HSR_GRP6_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP7_OUT_BEL.as_slice(),
                "HSR_GRP7_D_BLI_LOGIC_OUTS",
            ),
            (
                [wires::BLI_A_GRP0_OUT_CTRL_NOC].as_slice(),
                "HSR1_GRP0_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_A_GRP1_OUT_NOC.as_slice(),
                "HSR1_GRP1_A_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_A_GRP1_OUT_CTRL_NOC.as_slice(),
                "HSR1_GRP1_A_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                [wires::BLI_B_GRP0_OUT_CTRL_NOC].as_slice(),
                "HSR1_GRP0_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_B_GRP1_OUT_NOC.as_slice(),
                "HSR1_GRP1_B_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_B_GRP1_OUT_CTRL_NOC.as_slice(),
                "HSR1_GRP1_B_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                [wires::BLI_C_GRP0_OUT_CTRL_NOC].as_slice(),
                "HSR1_GRP0_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_C_GRP1_OUT_NOC.as_slice(),
                "HSR1_GRP1_C_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_C_GRP1_OUT_CTRL_NOC.as_slice(),
                "HSR1_GRP1_C_BLI_LOGIC_OUTS_CTRL",
            ),
            (
                wires::BLI_D_GRP4_OUT_NOC.as_slice(),
                "HSR1_GRP4_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP5_OUT_NOC.as_slice(),
                "HSR1_GRP5_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP6_OUT_NOC.as_slice(),
                "HSR1_GRP6_D_BLI_LOGIC_OUTS",
            ),
            (
                wires::BLI_D_GRP7_OUT_NOC.as_slice(),
                "HSR1_GRP7_D_BLI_LOGIC_OUTS",
            ),
        ] {
            for (i, w) in w.iter().copied().enumerate() {
                let wn = format!("{prefix}{i}");
                self.builder.wire_names(w, &[wn]);
            }
        }
    }
}
