use prjcombine_interconnect::db::WireSlotIdExt;
use prjcombine_versal::defs::{bslots, tcls, wires};

use crate::int::IntMaker;

impl<'rd> IntMaker<'rd> {
    fn fill_tiles_aie(&mut self) {
        for tkn in ["AIE_TILE", "AIE_TILE_MY"] {
            let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
                continue;
            };
            let mut bels = vec![];
            for (bslot, slot, x) in [
                (bslots::AIE_CORE, "AIE_CORE", 0),
                (bslots::AIE_MEMGRP, "AIE_MEMGRP", 0),
                (bslots::AIE_TILECTRL, "AIE_TILECTRL", 0),
                (bslots::AIE_ARRAYSWITCH, "AIE_ARRAYSWITCH", 0),
                (bslots::AIE_FIFO[0], "AIE_FIFO", 0),
                (bslots::AIE_FIFO[1], "AIE_FIFO", 1),
            ] {
                bels.push(self.bel_name_only(xy, bslot, slot, x, 0));
            }
            self.builder
                .xtile_id(tcls::AIE, "AIE", xy)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_aie_pll(&mut self) {
        let tkn = "AIE_INTF_A_CORE";
        for (name, wire) in [
            ("AIE_PLL_PL_CLK", wires::HSR_IMUX_TO_ME.cell(0)),
            ("AIE_PLL_REF_CLK", wires::HSR_IMUX_HSM_TO_ME.cell(0)),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_0_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[0].cell(0),
            ),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_1_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[1].cell(0),
            ),
        ] {
            self.builder.extra_name_tile_tw(tkn, name, wire);
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            let wi0 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_0_").unwrap();
            let (bli0_xy, _) = self.bli[&(xy, wi0)];
            let wi1 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_8_").unwrap();
            let (bli1_xy, _) = self.bli[&(xy, wi1)];

            let bel = self.builder.bel_xy(bslots::AIE_PLL, "AIE_PLL", 0, 0);

            let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
            self.builder
                .xtile_id(tcls::AIE_PLL, "AIE_PLL", xy)
                .bel(bel)
                .ref_single(bli0_xy, 1, bli)
                .ref_single(bli1_xy, 2, bli)
                .extract();

            let mut bels = vec![];
            for (bslot, slot, x) in [
                (bslots::AIE_TILECTRL, "AIE_TILECTRL", 0),
                (bslots::AIE_SHIMSWITCH, "AIE_SHIMSWITCH", 0),
                (bslots::AIE_SHIMTRACE, "AIE_SHIMTRACE", 0),
                (bslots::AIE_FIFO[0], "AIE_FIFO", 0),
                (bslots::AIE_FIFO[1], "AIE_FIFO", 1),
            ] {
                bels.push(self.bel_name_only(xy, bslot, slot, x, 0));
            }

            self.builder
                .xtile_id(tcls::AIE_SHIM, "AIE_SHIM", xy)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_aie_intf(&mut self) {
        for (tcid, naming, tkn) in [
            (tcls::AIE_SHIM_PL, "AIE_SHIM_PL", "AIE_INTF_B0_CORE"),
            (tcls::AIE_SHIM_NOC_PL, "AIE_SHIM_NOC_PL", "AIE_INTF_B1_CORE"),
            (tcls::AIE_SHIM_NOC_PL, "AIE_SHIM_NOC_PL", "AIE_INTF_B2_CORE"),
            (tcls::AIE_SHIM_PL, "AIE_SHIM_PL", "AIE_INTF_B3_CORE"),
        ] {
            if tcid == tcls::AIE_SHIM_NOC_PL {
                for (name, wire) in [
                    ("AIE_NOC_S_AXI_ACLK", wires::HSR_CLK_TEST_IN[1].cell(4)),
                    ("AIE_NOC_M_AXI_ACLK", wires::HSR_CLK_TEST_IN[5].cell(4)),
                ] {
                    self.builder.extra_name_tile_tw(tkn, name, wire);
                }
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("AIE_PL_S_TDATA_0_0_").unwrap();
                let Some(&(bli0_xy, _)) = self.bli.get(&(xy, wi0)) else {
                    continue;
                };
                let wi1 = self.rd.wires.get("AIE_PL_S_TDATA_0_32_").unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];
                let wi2 = self.rd.wires.get("AIE_PL_S_TDATA_4_0_").unwrap();
                let Some(&(bli2_xy, _)) = self.bli.get(&(xy, wi2)) else {
                    continue;
                };
                let wi3 = self.rd.wires.get("AIE_PL_S_TDATA_4_32_").unwrap();
                let (bli3_xy, _) = self.bli[&(xy, wi3)];

                let mut bels = vec![];

                let mut bel = self.builder.bel_xy(bslots::AIE_PL, "AIE_PL", 0, 0);
                for i in 0..6 {
                    bel = bel.pin_name_only(&format!("FROM_AIE_{i}"), 1);
                }
                for i in 0..8 {
                    bel = bel.pin_name_only(&format!("TO_AIE_{i}"), 1);
                }
                bels.push(bel);

                if tcid == tcls::AIE_SHIM_NOC_PL {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::AIE_NOC, "AIE_NOC", 0, 0)
                        .pin_name_only("FROM_NOC", 1)
                        .pin_name_only("TO_NOC", 1);
                    for i in 2..6 {
                        bel = bel
                            .pin_name_only(&format!("TO_PL_{i}"), 1)
                            .pin_name_only(&format!("FROM_AIE_{i}"), 1);
                    }
                    for i in [2, 3, 6, 7] {
                        bel = bel
                            .pin_name_only(&format!("FROM_PL_{i}"), 1)
                            .pin_name_only(&format!("TO_AIE_{i}"), 1);
                    }
                    bels.push(bel);
                }

                for (bslot, slot, x) in [
                    (bslots::AIE_TILECTRL, "AIE_TILECTRL", 0),
                    (bslots::AIE_SHIMSWITCH, "AIE_SHIMSWITCH", 0),
                    (bslots::AIE_SHIMTRACE, "AIE_SHIMTRACE", 0),
                    (bslots::AIE_FIFO[0], "AIE_FIFO", 0),
                    (bslots::AIE_FIFO[1], "AIE_FIFO", 1),
                ] {
                    bels.push(self.bel_name_only(xy, bslot, slot, x, 0));
                }

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .bels(bels)
                    .ref_single(bli0_xy, 0, bli)
                    .ref_single(bli1_xy, 1, bli)
                    .ref_single(bli2_xy, 2, bli)
                    .ref_single(bli3_xy, 3, bli)
                    .extract();
            }
        }
    }

    fn fill_tiles_aie_test(&mut self) {
        let tkn = "AIE_INTF_B0_CORE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let wi0 = self.rd.wires.get("PL_ME_SCAN_CLKDIV_RSTN_EXT").unwrap();
        let (bli0_xy, _) = self.bli[&(xy, wi0)];
        let wi1 = self.rd.wires.get("PL_ME_SCAN_CLK_N_EXT").unwrap();
        let (bli1_xy, _) = self.bli[&(xy, wi1)];
        let wi2 = self.rd.wires.get("ME_PL_SCAN_CHNL_OUT_EXT21").unwrap();
        let (bli2_xy, _) = self.bli[&(xy, wi2)];
        let wi3 = self.rd.wires.get("ME_PL_SCAN_CHNL_OUT_EXT29").unwrap();
        let (bli3_xy, _) = self.bli[&(xy, wi3)];

        let bel = self
            .builder
            .bel_xy(bslots::AIE_INTF_TEST, "AIE_INTF_B_TEST", 0, 0);
        let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
        self.builder
            .xtile_id(tcls::AIE_INTF_TEST, "AIE_INTF_TEST", xy)
            .bel(bel)
            .ref_single(bli0_xy, 0, bli)
            .ref_single(bli1_xy, 1, bli)
            .ref_single(bli2_xy, 2, bli)
            .ref_single(bli3_xy, 3, bli)
            .extract();
    }

    fn fill_tiles_aie_test_nc(&mut self) {
        let tkn = "AIE_INTF_C_CORE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let bel = self.bel_name_only(xy, bslots::AIE_INTF_TEST, "AIE_INTF_C_TEST", 0, 0);
        self.builder
            .xtile_id(tcls::AIE_INTF_TEST_NC, "AIE_INTF_TEST_NC", xy)
            .bel(bel)
            .extract();
    }

    fn fill_tiles_aie_ml(&mut self) {
        let tkn = "AIE_ML_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let mut bels = vec![];
        for (bslot, slot) in [
            (bslots::AIE_ML_CORE, "AIE_ML_CORE"),
            (bslots::AIE_MEMGRP, "AIE_ML_MEMGRP"),
            (bslots::AIE_TILECTRL, "AIE_ML_TILECTRL"),
            (bslots::AIE_ML_ARRAYSWITCH, "AIE_ML_ARRAYSWITCH"),
            (bslots::AIE_FIFO[0], "AIE_ML_FIFO"),
        ] {
            bels.push(self.bel_name_only(xy, bslot, slot, 0, 0));
        }
        self.builder
            .xtile_id(tcls::AIE_ML, "AIE_ML", xy)
            .bels(bels)
            .extract();
    }

    fn fill_tiles_aie_ml_mem(&mut self) {
        let tkn = "AIE_ML_MEM_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let mut bels = vec![];
        for (bslot, slot) in [
            (bslots::AIE_ML_MEM_MEMGRP, "AIE_ML_MEM_MEMGRP"),
            (bslots::AIE_TILECTRL, "AIE_ML_TILECTRL"),
            (bslots::AIE_ML_MEM_ARRAYSWITCH, "AIE_ML_MEM_ARRAYSWITCH"),
        ] {
            bels.push(self.bel_name_only(xy, bslot, slot, 0, 0));
        }
        self.builder
            .xtile_id(tcls::AIE_ML_MEM, "AIE_ML_MEM", xy)
            .bels(bels)
            .extract();
    }

    fn fill_tiles_aie_ml_pll(&mut self) {
        let tkn = "AIE_ML_INTF_A_CORE";
        for (name, wire) in [
            ("AIE_PLL_PL_CLK", wires::HSR_IMUX_TO_ME.cell(0)),
            ("AIE_PLL_REF_CLK", wires::HSR_IMUX_HSM_TO_ME.cell(0)),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_0_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[0].cell(0),
            ),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_1_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[1].cell(0),
            ),
        ] {
            self.builder.extra_name_tile_tw(tkn, name, wire);
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            let wi0 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_0_").unwrap();
            let (bli0_xy, _) = self.bli[&(xy, wi0)];
            let wi1 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_8_").unwrap();
            let (bli1_xy, _) = self.bli[&(xy, wi1)];

            let bel = self.builder.bel_xy(bslots::AIE_PLL, "AIE_ML_PLL", 0, 0);

            let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
            self.builder
                .xtile_id(tcls::AIE_ML_PLL, "AIE_ML_PLL", xy)
                .bel(bel)
                .ref_single(bli0_xy, 1, bli)
                .ref_single(bli1_xy, 2, bli)
                .extract();

            let mut bels = vec![];

            for (bslot, slot) in [
                (bslots::AIE_TILECTRL, "AIE_ML_TILECTRL"),
                (bslots::AIE_ML_SHIMSWITCH, "AIE_ML_SHIMSWITCH"),
                (bslots::AIE_SHIMTRACE, "AIE_ML_SHIMTRACE"),
                (bslots::AIE_FIFO[0], "AIE_ML_FIFO"),
            ] {
                bels.push(self.bel_name_only(xy, bslot, slot, 0, 0));
            }

            self.builder
                .xtile_id(tcls::AIE_ML_SHIM, "AIE_ML_SHIM", xy)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_aie_ml_intf(&mut self) {
        for (tcid, naming, tkn) in [
            (
                tcls::AIE_ML_SHIM_PL,
                "AIE_ML_SHIM_PL",
                "AIE_ML_SHIM_PL_TILE",
            ),
            (
                tcls::AIE_ML_SHIM_NOC_PL,
                "AIE_ML_SHIM_NOC_PL",
                "AIE_ML_SHIM_NOC_TILE",
            ),
        ] {
            if tcid == tcls::AIE_ML_SHIM_NOC_PL {
                for (name, wire) in [
                    ("AIE_NOC_S_AXI_ACLK", wires::HSR_CLK_TEST_IN[1].cell(4)),
                    ("AIE_NOC_M_AXI_ACLK", wires::HSR_CLK_TEST_IN[5].cell(4)),
                ] {
                    self.builder.extra_name_tile_tw(tkn, name, wire);
                }
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("AIE_PL_S_TDATA_0_0_").unwrap();
                let Some(&(bli0_xy, _)) = self.bli.get(&(xy, wi0)) else {
                    continue;
                };
                let wi1 = self.rd.wires.get("AIE_PL_S_TDATA_0_32_").unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];
                let wi2 = self.rd.wires.get("AIE_PL_S_TDATA_4_0_").unwrap();
                let Some(&(bli2_xy, _)) = self.bli.get(&(xy, wi2)) else {
                    continue;
                };
                let wi3 = self.rd.wires.get("AIE_PL_S_TDATA_4_32_").unwrap();
                let (bli3_xy, _) = self.bli[&(xy, wi3)];

                let mut bels = vec![];

                let mut bel = self.builder.bel_xy(bslots::AIE_PL, "AIE_ML_PL", 0, 0);
                for i in 0..6 {
                    bel = bel.pin_name_only(&format!("FROM_AIE_{i}"), 1);
                }
                for i in 0..8 {
                    bel = bel.pin_name_only(&format!("TO_AIE_{i}"), 1);
                }
                bels.push(bel);

                if tcid == tcls::AIE_ML_SHIM_NOC_PL {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::AIE_NOC, "AIE_ML_NOC", 0, 0)
                        .pin_name_only("FROM_NOC", 1)
                        .pin_name_only("TO_NOC", 1);
                    for i in 2..6 {
                        bel = bel
                            .pin_name_only(&format!("TO_PL_{i}"), 1)
                            .pin_name_only(&format!("FROM_AIE_{i}"), 1);
                    }
                    for i in [2, 3, 6, 7] {
                        bel = bel
                            .pin_name_only(&format!("FROM_PL_{i}"), 1)
                            .pin_name_only(&format!("TO_AIE_{i}"), 1);
                    }
                    bels.push(bel);
                }

                for (bslot, slot, x) in [
                    (bslots::AIE_TILECTRL, "AIE_ML_TILECTRL", 0),
                    (bslots::AIE_ML_SHIMSWITCH, "AIE_ML_SHIMSWITCH", 0),
                    (bslots::AIE_SHIMTRACE, "AIE_ML_SHIMTRACE", 0),
                    (bslots::AIE_FIFO[0], "AIE_ML_FIFO", 0),
                ] {
                    bels.push(self.bel_name_only(xy, bslot, slot, x, 0));
                }

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .bels(bels)
                    .ref_single(bli0_xy, 0, bli)
                    .ref_single(bli1_xy, 1, bli)
                    .ref_single(bli2_xy, 2, bli)
                    .ref_single(bli3_xy, 3, bli)
                    .extract();
            }
        }
    }

    fn fill_tiles_aie_ml_test(&mut self) {
        let tkn = "AIE_ML_INTF_B_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let bli0_xy = self.xlat_bli(xy, "PL_ME_SCAN_CHNL_IN_EXT8").unwrap();
        let bli1_xy = self.xlat_bli(xy, "PL_ME_SCAN_CLK_N_EXT").unwrap();
        let bli2_xy = self.xlat_bli(xy, "ME_PL_SCAN_CHNL_OUT_EXT23").unwrap();
        let bli3_xy = self.xlat_bli(xy, "ME_PL_SCAN_CHNL_OUT_EXT31").unwrap();

        let bel = self
            .builder
            .bel_xy(bslots::AIE_ML_INTF_TEST, "AIE_ML_INTF_B_TEST", 0, 0);
        let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
        self.builder
            .xtile_id(tcls::AIE_ML_INTF_TEST, "AIE_ML_INTF_TEST", xy)
            .bel(bel)
            .ref_single(bli0_xy, 0, bli)
            .ref_single(bli1_xy, 1, bli)
            .ref_single(bli2_xy, 2, bli)
            .ref_single(bli3_xy, 3, bli)
            .extract();
    }

    fn fill_tiles_aie_ml_test_nc(&mut self) {
        let tkn = "AIE_ML_INTF_C_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let bel = self.bel_name_only(xy, bslots::AIE_ML_INTF_TEST, "AIE_ML_INTF_C_TEST", 0, 0);
        self.builder
            .xtile_id(tcls::AIE_ML_INTF_TEST_NC, "AIE_ML_INTF_TEST_NC", xy)
            .bel(bel)
            .extract();
    }

    fn fill_tiles_aie2ps(&mut self) {
        let tkn = "AIE2P_S_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let mut bels = vec![];
        for (bslot, slot) in [
            (bslots::AIE_ML_CORE, "AIE2PS_CORE"),
            (bslots::AIE_MEMGRP, "AIE2PS_MEMGRP"),
            (bslots::AIE_TILECTRL, "AIE2P_TILECTRL"),
            (bslots::AIE_ML_ARRAYSWITCH, "AIE2PS_ARRAYSWITCH"),
            (bslots::AIE_FIFO[0], "AIE2P_FIFO"),
        ] {
            bels.push(self.bel_name_only(xy, bslot, slot, 0, 0));
        }
        self.builder
            .xtile_id(tcls::AIE2PS, "AIE2PS", xy)
            .bels(bels)
            .extract();
    }

    fn fill_tiles_aie2ps_mem(&mut self) {
        let tkn = "AIE2P_S_MEM_TILE";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let mut bels = vec![];
        for (bslot, slot) in [
            (bslots::AIE_ML_MEM_MEMGRP, "AIE2PS_MEM_MEMGRP"),
            (bslots::AIE_TILECTRL, "AIE2P_TILECTRL"),
            (bslots::AIE_ML_MEM_ARRAYSWITCH, "AIE2PS_MEM_ARRAYSWITCH"),
        ] {
            bels.push(self.bel_name_only(xy, bslot, slot, 0, 0));
        }
        self.builder
            .xtile_id(tcls::AIE2PS_MEM, "AIE2PS_MEM", xy)
            .bels(bels)
            .extract();
    }

    fn fill_tiles_aie2ps_pll(&mut self) {
        let tkn = "AIE2P_S_INTF_A_CORE";
        for (name, wire) in [
            ("AIE_PLL_PL_CLK", wires::HSR_IMUX_TO_ME.cell(0)),
            ("AIE_PLL_REF_CLK", wires::HSR_IMUX_HSM_TO_ME.cell(0)),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_0_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[0].cell(0),
            ),
            (
                "AIE_PLL_AIE_PLL_TEST_CLK_OUT_1_",
                wires::AIE_OUT_PLL_TEST_CLK_OUT[1].cell(0),
            ),
        ] {
            self.builder.extra_name_tile_tw(tkn, name, wire);
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            let wi0 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_0_").unwrap();
            let (bli0_xy, _) = self.bli[&(xy, wi0)];
            let wi1 = self.rd.wires.get("AIE_PLL_AIE_PLL_TEST_OUT_8_").unwrap();
            let (bli1_xy, _) = self.bli[&(xy, wi1)];

            let bel = self.builder.bel_xy(bslots::AIE_PLL, "AIE2PS_PLL", 0, 0);

            let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
            self.builder
                .xtile_id(tcls::AIE2PS_PLL, "AIE2PS_PLL", xy)
                .bel(bel)
                .ref_single(bli0_xy, 1, bli)
                .ref_single(bli1_xy, 2, bli)
                .extract();
        }
    }

    fn fill_tiles_aie2ps_intf(&mut self) {
        let tcid = tcls::AIE2PS_SHIM_NOC_PL;
        for (tkn, naming) in [
            ("AIE2P_S_INTF_A_CORE", "AIE2PS_SHIM_NOC_PL_A"),
            ("AIE2P_S_INTF_C_CORE", "AIE2PS_SHIM_NOC_PL"),
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let bli_xy = [
                    "AIE_PL_S_TDATA_0_0_",
                    "AIE_PL_S_TDATA_0_32_",
                    "AIE_PL_S_TDATA_4_0_",
                    "AIE_PL_S_TDATA_4_32_",
                ]
                .map(|w| self.xlat_bli(xy, w));

                let is_nc = bli_xy.iter().any(Option::is_none);

                let mut bels = vec![];

                if is_nc {
                    bels.push(
                        self.bel_name_only(xy, bslots::AIE_PL, "AIE2P_PL", 0, 0)
                            .naming_only(),
                    );
                } else {
                    let mut bel = self.builder.bel_xy(bslots::AIE_PL, "AIE2P_PL", 0, 0);
                    for i in 0..6 {
                        bel = bel.pin_name_only(&format!("FROM_AIE_{i}"), 1);
                    }
                    for i in 0..8 {
                        bel = bel.pin_name_only(&format!("TO_AIE_{i}"), 1);
                    }
                    bels.push(bel);
                }

                let mut bel = self
                    .builder
                    .bel_xy(bslots::AIE_NOC, "AIE2PS_NOC", 0, 0)
                    .pin_name_only("FROM_NOC", 1)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("M_AXI_ACLK", 1)
                    .pin_name_only("S_AXI_ACLK", 1);
                for i in 2..6 {
                    bel = bel
                        .pin_name_only(&format!("TO_PL_{i}"), 1)
                        .pin_name_only(&format!("FROM_AIE_{i}"), 1);
                }
                for i in [2, 3, 6, 7] {
                    bel = bel
                        .pin_name_only(&format!("FROM_PL_{i}"), 1)
                        .pin_name_only(&format!("TO_AIE_{i}"), 1);
                }
                bels.push(bel);

                for (bslot, slot, x) in [
                    (bslots::AIE_TILECTRL, "AIE2P_TILECTRL", 0),
                    (bslots::AIE2PS_SHIMSWITCH, "AIE2PS_SHIMSWITCH", 0),
                    (bslots::AIE2PS_SHIM_CNTRL, "AIE2PS_SHIM_CNTRL", 0),
                    (bslots::AIE_SHIMTRACE, "AIE2PS_SHIMTRACE", 0),
                    (bslots::AIE_FIFO[0], "AIE2P_FIFO", 0),
                    (bslots::AIE2PS_NMU_ARBIT, "AIE2PS_NMU_ARBIT", 0),
                ] {
                    bels.push(self.bel_name_only(xy, bslot, slot, x, 0));
                }

                if is_nc {
                    self.builder.xtile_id(tcid, naming, xy).bels(bels).extract();
                } else {
                    let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                    self.builder
                        .xtile_id(tcid, naming, xy)
                        .bels(bels)
                        .ref_single(bli_xy[0].unwrap(), 0, bli)
                        .ref_single(bli_xy[1].unwrap(), 1, bli)
                        .ref_single(bli_xy[2].unwrap(), 2, bli)
                        .ref_single(bli_xy[3].unwrap(), 3, bli)
                        .extract();
                }
            }
        }
    }

    fn fill_tiles_aie2ps_test(&mut self) {
        for (tkn, bk) in [
            ("AIE2P_S_INTF_A_CORE", "AIE2PS_INTF_A_TEST"),
            ("AIE2P_S_INTF_C_CORE", "AIE2PS_INTF_C_TEST"),
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                if self
                    .xlat_bli(xy, "IF_ME_DFX0_FABRIC_PL_ME_SCAN_EN_N")
                    .is_some()
                {
                    let bli_xy = [
                        "IF_ME_DFX0_FABRIC_PL_ME_SCAN_EN_N",
                        "IF_ME_DFX0_FABRIC_PL_ME_SCAN_CLK_N",
                        "IF_ME_DFX0_FABRIC_PL_ME_SCAN_CHNL_IN20",
                    ]
                    .map(|w| self.xlat_bli(xy, w).unwrap());
                    let bel = self.builder.bel_xy(bslots::AIE2PS_INTF_TEST, bk, 0, 0);
                    let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                    self.builder
                        .xtile_id(tcls::AIE2PS_INTF_TEST, "AIE2PS_INTF_TEST", xy)
                        .ref_single(bli_xy[0], 0, bli)
                        .ref_single(bli_xy[1], 1, bli)
                        .ref_single(bli_xy[2], 2, bli)
                        .bel(bel)
                        .extract();
                } else {
                    let bel = self.bel_name_only(xy, bslots::AIE2PS_INTF_TEST, bk, 0, 0);
                    self.builder
                        .xtile_id(tcls::AIE2PS_INTF_TEST_NC, "AIE2PS_INTF_TEST", xy)
                        .bel(bel)
                        .extract();
                }
            }
        }
    }

    pub fn fill_tiles_ai(&mut self) {
        self.fill_tiles_aie();
        self.fill_tiles_aie_pll();
        self.fill_tiles_aie_intf();
        self.fill_tiles_aie_test();
        self.fill_tiles_aie_test_nc();
        self.fill_tiles_aie_ml();
        self.fill_tiles_aie_ml_mem();
        self.fill_tiles_aie_ml_pll();
        self.fill_tiles_aie_ml_intf();
        self.fill_tiles_aie_ml_test();
        self.fill_tiles_aie_ml_test_nc();
        self.fill_tiles_aie2ps();
        self.fill_tiles_aie2ps_mem();
        self.fill_tiles_aie2ps_pll();
        self.fill_tiles_aie2ps_intf();
        self.fill_tiles_aie2ps_test();
    }
}
