use std::collections::BTreeMap;

use prjcombine_interconnect::db::WireSlotIdExt;
use prjcombine_re_xilinx_naming::db::BelNaming;
use prjcombine_re_xilinx_rawdump::NodeOrWire;
use prjcombine_re_xilinx_rd2db_interconnect::{PipMode, bel::TempBel};
use prjcombine_versal::defs::{bslots, tcls, wires};

use crate::int::IntMaker;

impl<'rd> IntMaker<'rd> {
    pub fn fill_tiles_hnoc_xpio(&mut self) {
        let tiles = self.rd.tiles_by_kind_name("NOC_HNOC_S3_PS_CORE");
        if !tiles.is_empty() {
            let &pnoc_xy = self
                .rd
                .tiles_by_kind_name("NOC_PNOC_MONO_CORE")
                .iter()
                .next()
                .unwrap();

            assert!(tiles.len() >= 2);
            for (tcid, naming, nmu, nsu) in [
                (tcls::HNOC_XPIO_PS9_A, "HNOC_XPIO_PS9_A", 0..4, 0..2),
                (tcls::HNOC_XPIO_PS9_B, "HNOC_XPIO_PS9_B", 4..10, 2..6),
                (tcls::HNOC_XPIO_PS9_C, "HNOC_XPIO_PS9_C", 0..0, 0..0),
            ] {
                let mut bels = vec![];
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS5555", 0, i % 2)
                            .raw_tile(i / 2)
                            .pin_name_only("OUT_0", 1)
                            .pin_name_only("OUT_1", 1)
                            .pin_name_only("OUT_2", 1)
                            .pin_name_only("OUT_3", 1)
                            .pin_name_only("IN_0", 1)
                            .pin_name_only("IN_1", 1)
                            .pin_name_only("IN_2", 1)
                            .pin_name_only("IN_3", 1),
                    );
                }

                if tcid == tcls::HNOC_XPIO_PS9_A {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_NPI_NIR, "NPI_NIR", 0, 0)
                            .raw_tile(2),
                    );
                }
                for (i, sy) in nmu.enumerate() {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_PS_NMU[i], "NOC_NMU128", 0, sy)
                            .raw_tile(2),
                    );
                }
                for (i, sy) in nsu.enumerate() {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_PS_NSU[i], "NOC_NSU128", 0, sy)
                            .raw_tile(2),
                    );
                }

                self.builder
                    .xtile_id(tcid, naming, tiles[0])
                    .raw_tile(tiles[1])
                    .raw_tile(pnoc_xy)
                    .bels(bels)
                    .extract();
            }
        }
        let tiles = self.rd.tiles_by_kind_name("NOC_HNOC_S3_PL_CORE");
        if !tiles.is_empty() {
            assert!(tiles.len() >= 2);
            let tcid = tcls::HNOC_XPIO;
            let naming = "HNOC_XPIO";
            let mut bels = vec![];
            for i in 0..4 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS5555", 0, i % 2 * 2)
                        .raw_tile(i / 2)
                        .pin_name_only("OUT_0", 1)
                        .pin_name_only("OUT_1", 1)
                        .pin_name_only("OUT_2", 1)
                        .pin_name_only("OUT_3", 1)
                        .pin_name_only("IN_0", 1)
                        .pin_name_only("IN_1", 1)
                        .pin_name_only("IN_2", 1)
                        .pin_name_only("IN_3", 1),
                );
            }
            for i in 0..2 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_NPS_MID[i], "NOC_NPS5555", 0, 1)
                        .raw_tile(i)
                        .pin_name_only("OUT_0", 1)
                        .pin_name_only("OUT_1", 1)
                        .pin_name_only("OUT_2", 1)
                        .pin_name_only("OUT_3", 1)
                        .pin_name_only("IN_0", 1)
                        .pin_name_only("IN_1", 1)
                        .pin_name_only("IN_2", 1)
                        .pin_name_only("IN_3", 1),
                );
            }
            self.builder
                .xtile_id(tcid, naming, tiles[0])
                .raw_tile(tiles[1])
                .bels(bels)
                .extract();
        }
    }

    pub fn fill_tiles_hsr_mmcm(&mut self) {
        for tkn in ["CMT_MMCM", "CMT_MMCM_MX"] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("CMT_MMCM_TOP_0_DI_0_").unwrap();
                let (bli0_xy, _) = self.bli[&(xy, wi0)];
                let wi1 = self.rd.wires.get("CMT_MMCM_TOP_0_DI_4_").unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];
                let (tcid, bli_xy, bli_naming) = if bli0_xy == bli1_xy {
                    (tcls::MMCM_XPIO, vec![None, Some(bli0_xy)], "BLI_MUX")
                } else {
                    (
                        tcls::MMCM_X5IO,
                        vec![Some(bli1_xy), Some(bli0_xy)],
                        "BLI_MUX_NOC",
                    )
                };
                for (pin, wire) in [
                    ("CLKIN1", wires::HSR_IMUX_MMCM_CLKIN1.cell(0)),
                    ("CLKIN2", wires::HSR_IMUX_MMCM_CLKIN2.cell(0)),
                    ("CLKFBIN", wires::HSR_IMUX_MMCM_CLKFB.cell(0)),
                    (
                        "CLKIN1_DESKEW",
                        wires::HSR_IMUX_MMCM_CLKIN_DESKEW[0].cell(0),
                    ),
                    (
                        "CLKIN2_DESKEW",
                        wires::HSR_IMUX_MMCM_CLKIN_DESKEW[1].cell(0),
                    ),
                    (
                        "CLKFB1_DESKEW",
                        wires::HSR_IMUX_MMCM_CLKFB_DESKEW[0].cell(0),
                    ),
                    (
                        "CLKFB2_DESKEW",
                        wires::HSR_IMUX_MMCM_CLKFB_DESKEW[1].cell(0),
                    ),
                    ("CLKOUT0", wires::HSR_OUT_MMCM[0].cell(0)),
                    ("CLKOUT1", wires::HSR_OUT_MMCM[1].cell(0)),
                    ("CLKOUT2", wires::HSR_OUT_MMCM[2].cell(0)),
                    ("CLKOUT3", wires::HSR_OUT_MMCM[3].cell(0)),
                    ("CLKOUT4", wires::HSR_OUT_MMCM[4].cell(0)),
                    ("CLKOUT5", wires::HSR_OUT_MMCM[5].cell(0)),
                    ("CLKOUT6", wires::HSR_OUT_MMCM[6].cell(0)),
                    ("CLKFBOUT", wires::HSR_OUT_MMCM[7].cell(0)),
                    ("TMUXOUT", wires::HSR_OUT_MMCM[8].cell(0)),
                ] {
                    self.builder
                        .extra_name_pin_xy(tkn, "MMCM", 0, 0, pin, wire, true);
                }
                let bel = self.builder.bel_xy(bslots::MMCM, "MMCM", 0, 0);
                let bli = self.builder.ndb.get_tile_class_naming(bli_naming);
                let mut xn = self.builder.xtile_id(tcid, "MMCM", xy);
                for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                    if let Some(bli_xy) = bli_xy {
                        xn = xn.ref_single(bli_xy, i + 1, bli);
                    }
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_hsr_dpll(&mut self) {
        for tkn in ["CMT_DPLL", "CMT_DPLL_MX"] {
            for (pin, wire) in [
                ("CLKIN", wires::HSR_IMUX_DPLL_CLKIN.cell(0)),
                ("CLKIN_DESKEW", wires::HSR_IMUX_DPLL_CLKIN_DESKEW.cell(0)),
                ("CLKOUT0", wires::HSR_OUT_DPLL[0].cell(0)),
                ("CLKOUT1", wires::HSR_OUT_DPLL[1].cell(0)),
                ("CLKOUT2", wires::HSR_OUT_DPLL[2].cell(0)),
                ("CLKOUT3", wires::HSR_OUT_DPLL[3].cell(0)),
                ("TMUXOUT", wires::HSR_OUT_DPLL[4].cell(0)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "DPLL", 0, 0, pin, wire, true);
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("CMT_DPLL_TOP_0_DI_4_").unwrap();
                if let Some(&(bli0_xy, bli0_wi)) = self.bli.get(&(xy, wi0)) {
                    let wn = self.rd.wires[bli0_wi].as_str();
                    let (tcid, bli_xy, bli_naming) = match wn {
                        "BLI_GRP4_D_BLI_IMUX_BYP0" => {
                            (tcls::DPLL_XPIO, vec![None, Some(bli0_xy)], "BLI_MUX")
                        }
                        "BLI_GRP0_B_BLI_IMUX0" => {
                            (tcls::DPLL_X5IO, vec![Some(bli0_xy), None], "BLI_MUX_NOC")
                        }
                        _ => panic!("oops {wn}"),
                    };
                    let bel = self.builder.bel_xy(bslots::DPLL_HSR, "DPLL", 0, 0);
                    let bli = self.builder.ndb.get_tile_class_naming(bli_naming);
                    let mut xn = self.builder.xtile_id(tcid, "DPLL", xy);
                    for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                        if let Some(bli_xy) = bli_xy {
                            xn = xn.ref_single(bli_xy, i + 1, bli);
                        }
                    }
                    xn.bel(bel).extract();
                } else {
                    let mut bel = self.builder.bel_xy(bslots::DPLL_HSR, "DPLL", 0, 0);
                    for pin in self.bel_pins(xy, "DPLL", 0, 0) {
                        if !matches!(
                            pin,
                            "CLKIN"
                                | "CLKIN_DESKEW"
                                | "CLKOUT0"
                                | "CLKOUT1"
                                | "CLKOUT2"
                                | "CLKOUT3"
                                | "TMUXOUT"
                        ) {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                    self.builder
                        .xtile_id(tcls::DPLL_X5IO_NC, "DPLL", xy)
                        .bel(bel)
                        .extract();
                }
            }
        }
    }

    pub fn fill_tiles_hsr_clk_xpio(&mut self) {
        for tkn in ["CLK_REBUF_BUFGS_HSR_CORE", "CLK_REBUF_BUFGS_HSR_CORE_MX"] {
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HROUTE_0_{i}"),
                    wires::HSR_HROUTE_W[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HROUTE_1_{i}"),
                    wires::HSR_HROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_0_{i}"),
                    wires::HSR_HDISTR_W[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_1_{i}"),
                    wires::HSR_HDISTR[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_16_ENC_{i}_CLK_OUT"),
                    wires::HSR_HDISTR_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = i * 2),
                    wires::HSR_IMUX_BUFG_HDISTR[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_24_ENC_{ii}_CLK_OUT", ii = i * 2 + 1),
                    wires::HSR_HROUTE_TEST_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_36_ENC_{ii}_CLK_OUT", ii = i * 2),
                    wires::HSR_IMUX_BUFG_XPIO[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_36_ENC_{ii}_CLK_IN27", ii = i * 2),
                    wires::HSR_HROUTE_TEST[i],
                );
            }
            for i in 0..6 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_IN_TEST{i}"),
                    wires::HSR_CLK_TEST_IN[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_DRVR_TRI_{ii}_CLK_IN", ii = i + 144),
                    wires::HSR_OUT_BUFG_TEST[i],
                );
            }
            for i in 0..5 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("DPLL_CLKOUT{i}"),
                    wires::HSR_OUT_DPLL[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_XPIO_XPIO_PLL0_CLK{i}"),
                    wires::HSR_OUT_XPLL0[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_XPIO_XPIO_PLL1_CLK{i}"),
                    wires::HSR_OUT_XPLL1[i],
                );
            }
            for i in 0..9 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_XPIO_XPIO_PHY_CLK{i}"),
                    wires::HSR_OUT_PHY[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("MMCM_CLKOUT{i}"),
                    wires::HSR_OUT_MMCM[i],
                );
            }
            for i in 0..4 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_XPIO_XPIO_CCIO{i}"),
                    wires::HSR_OUT_CLK_CCIO[i],
                );
            }
            for (wire, name) in [
                (wires::HSR_IMUX_DPLL_CLKIN, "DPLL_CLKIN"),
                (wires::HSR_IMUX_DPLL_CLKIN_DESKEW, "DPLL_CLKIN_DESKEW"),
                (wires::HSR_IMUX_MMCM_CLKFB, "MMCM_CLKFBIN"),
                (wires::HSR_IMUX_MMCM_CLKFB_DESKEW[0], "MMCM_CLKFB_DESKEW0"),
                (wires::HSR_IMUX_MMCM_CLKFB_DESKEW[1], "MMCM_CLKFB_DESKEW1"),
                (wires::HSR_IMUX_MMCM_CLKIN1, "MMCM_CLKIN1"),
                (wires::HSR_IMUX_MMCM_CLKIN2, "MMCM_CLKIN2"),
                (wires::HSR_IMUX_MMCM_CLKIN_DESKEW[0], "MMCM_CLKIN_DESKEW0"),
                (wires::HSR_IMUX_MMCM_CLKIN_DESKEW[1], "MMCM_CLKIN_DESKEW1"),
                (
                    wires::HSR_IMUX_DPLL_CLKIN_HDISTR,
                    "CLK_CMT_MUX_24_ENC_48_CLK_OUT",
                ),
                (
                    wires::HSR_IMUX_MMCM_CLKFB_HDISTR,
                    "CLK_CMT_MUX_24_ENC_49_CLK_OUT",
                ),
                (
                    wires::HSR_IMUX_MMCM_CLKIN1_HDISTR,
                    "CLK_CMT_MUX_24_ENC_50_CLK_OUT",
                ),
                (
                    wires::HSR_IMUX_MMCM_CLKIN2_HDISTR,
                    "CLK_CMT_MUX_24_ENC_51_CLK_OUT",
                ),
            ] {
                self.builder.extra_name_tile(tkn, name, wire);
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                if self.rd.lookup_wire(xy, "IF_XPIO_XPIO_PLL0_CLK0")
                    == self.rd.lookup_wire(xy, "IF_XPIO_XPIO_PLL1_CLK0")
                {
                    continue;
                }
                let wi0 = self
                    .builder
                    .rd
                    .wires
                    .get("IF_BUFGS_FABRIC_CLK_INT")
                    .unwrap();
                let (bli0_xy, bli0_wi) = self.bli[&(xy, wi0)];
                let wn = self.rd.wires[bli0_wi].as_str();
                let (tcid, bli_xy, bli_naming, is_min) = match wn {
                    "BLI_GRP3_A_BLI_CTRL0" => {
                        let wi1 = self
                            .builder
                            .rd
                            .wires
                            .get("CLK_BUFGCTRL_WITH_OPTINVS_64_IGNORE0")
                            .unwrap();
                        let (bli1_xy, _) = self.bli[&(xy, wi1)];
                        (
                            tcls::CLK_XPIO_FULL,
                            vec![Some(bli1_xy), Some(bli0_xy)],
                            "BLI_MUX",
                            false,
                        )
                    }
                    "BLI_GRP3_A_BLI_CTRL2" => (
                        tcls::CLK_XPIO_MIN,
                        vec![None, Some(bli0_xy)],
                        "BLI_MUX",
                        true,
                    ),
                    "BLI_GRP3_B_BLI_CTRL2" => {
                        let wi1 = self
                            .builder
                            .rd
                            .wires
                            .get("CLK_BUFGCTRL_WITH_OPTINVS_64_IGNORE0")
                            .unwrap();
                        if let Some(&(bli1_xy, _)) = self.bli.get(&(xy, wi1)) {
                            (
                                tcls::CLK_X5IO_FULL,
                                vec![Some(bli0_xy), Some(bli1_xy)],
                                "BLI_MUX_NOC",
                                false,
                            )
                        } else {
                            (
                                tcls::CLK_X5IO_MIN_A,
                                vec![Some(bli0_xy), None],
                                "BLI_MUX_NOC",
                                true,
                            )
                        }
                    }
                    "BLI_GRP3_B_BLI_CTRL1" => {
                        let wi1 = self.rd.wires.get("CLK_BUFGCE_48_CE").unwrap();
                        let (bli1_xy, _) = self.bli[&(xy, wi1)];
                        (
                            tcls::CLK_X5IO_MIN_B,
                            vec![Some(bli0_xy), Some(bli1_xy)],
                            "BLI_MUX_NOC",
                            true,
                        )
                    }
                    _ => panic!("oops {wn}"),
                };
                if matches!(tcid, tcls::CLK_XPIO_MIN | tcls::CLK_XPIO_FULL) {
                    for (wire, name) in [
                        (
                            wires::BLI_B_GRP3_IMUX_CTRL[0].cell(3),
                            "CLB_CLKFB_DESKEW0_MMCM",
                        ),
                        (
                            wires::BLI_A_GRP3_IMUX_CTRL[0].cell(4),
                            "CLB_CLKFB_DESKEW1_MMCM",
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                } else {
                    for (wire, name) in [
                        (
                            wires::BLI_A_GRP3_IMUX_CTRL[2].cell(4),
                            "CLB_CLKFB_DESKEW0_MMCM",
                        ),
                        (
                            wires::BLI_B_GRP3_IMUX_CTRL[1].cell(4),
                            "CLB_CLKFB_DESKEW1_MMCM",
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                let mut bels = vec![];
                for i in 0..24 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::GCLK_TEST_HSR[i], "GCLK_TEST", 0, i)
                            .pin_rename("CLKIN", "I")
                            .pin_rename("CLKOUT", "O"),
                    );
                }
                for i in 0..24 {
                    for (pin, wire) in [
                        ("I", wires::HSR_IMUX_BUFG[i].cell(0)),
                        ("O", wires::HSR_OUT_BUFGCE[i].cell(0)),
                    ] {
                        self.builder
                            .extra_name_pin_xy(tkn, "BUFGCE", 0, i, pin, wire, true);
                    }
                    bels.push(self.builder.bel_xy(bslots::BUFGCE[i], "BUFGCE", 0, i));
                }
                for i in 0..8 {
                    for (pin, wire) in [
                        ("I0", wires::HSR_IMUX_BUFGCTRL_I0[i].cell(0)),
                        ("I1", wires::HSR_IMUX_BUFGCTRL_I1[i].cell(0)),
                        ("O", wires::HSR_OUT_BUFGCTRL[i].cell(0)),
                    ] {
                        self.builder
                            .extra_name_pin_xy(tkn, "BUFGCTRL", 0, i, pin, wire, true);
                    }
                    let mut bel = self.builder.bel_xy(bslots::BUFGCTRL[i], "BUFGCTRL", 0, i);
                    if is_min {
                        bel = bel
                            .pin_name_only("CE0", 1)
                            .pin_name_only("CE1", 1)
                            .pin_name_only("SEL0", 1)
                            .pin_name_only("SEL1", 1)
                            .pin_name_only("IGNORE0", 1)
                            .pin_name_only("IGNORE1", 1);
                    }
                    bels.push(bel);
                }
                for i in 0..4 {
                    self.builder.extra_name_pin_xy(
                        tkn,
                        "BUFGCE_DIV",
                        0,
                        i,
                        "O",
                        wires::HSR_OUT_BUFGCE_DIV[i].cell(0),
                        true,
                    );
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BUFGCE_DIV[i], "BUFGCE_DIV", 0, i);
                    if is_min {
                        bel = bel.pin_name_only("CE", 1).pin_name_only("CLR", 1);
                    }
                    bels.push(bel);
                }
                for i in 0..6 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFG_HSR_TEST[i], "BUFG_HSR_TEST", 0, i),
                    );
                }
                let bli = self.builder.ndb.get_tile_class_naming(bli_naming);
                let mut dummy = self.builder.add_dummies(tkn);
                dummy
                    .insert(
                        wires::HSR_IMUX_DPLL_CLKIN.cell(0),
                        true,
                        &wires::HSR_IMUX_DPLL_CLKIN_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_DPLL_CLKIN_DESKEW.cell(0),
                        true,
                        &wires::HSR_IMUX_DPLL_CLKIN_DESKEW_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKIN1.cell(0),
                        true,
                        &wires::HSR_IMUX_MMCM_CLKIN1_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKIN2.cell(0),
                        true,
                        &wires::HSR_IMUX_MMCM_CLKIN2_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKFB.cell(0),
                        true,
                        &wires::HSR_IMUX_MMCM_CLKFB_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKIN_DESKEW[0].cell(0),
                        true,
                        &[wires::HSR_IMUX_MMCM_CLKIN_DESKEW_DUMMY[0].cell(0)],
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKIN_DESKEW[1].cell(0),
                        true,
                        &[wires::HSR_IMUX_MMCM_CLKIN_DESKEW_DUMMY[1].cell(0)],
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKFB_DESKEW[1].cell(0),
                        true,
                        &wires::HSR_IMUX_MMCM_CLKFB_DESKEW1_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_MMCM_CLKFB_DESKEW[0].cell(0),
                        true,
                        &wires::HSR_IMUX_MMCM_CLKFB_DESKEW0_DUMMY.map(|w| w.cell(0)),
                    );
                for i in 0..24 {
                    dummy.insert(
                        wires::HSR_IMUX_BUFG[i].cell(0),
                        true,
                        &[
                            wires::HSR_IMUX_BUFG_DUMMY0[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY1[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY2[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY3[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY4[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY5[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY6[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY7[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY8[i].cell(0),
                            wires::HSR_IMUX_BUFG_DUMMY9[i].cell(0),
                        ],
                    );
                    dummy.insert(
                        wires::HSR_IMUX_BUFG_XPIO[i].cell(0),
                        true,
                        &[
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY0[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY1[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY2[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY3[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY4[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY5[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY6[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY7[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY8[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY9[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY10[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY11[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY12[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY13[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY14[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY15[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY16[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY17[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY18[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY19[i].cell(0),
                            wires::HSR_IMUX_BUFG_XPIO_DUMMY20[i].cell(0),
                        ],
                    );
                    dummy.insert(
                        wires::HSR_HDISTR_MUX[i].cell(0),
                        true,
                        &[
                            wires::HSR_HDISTR_MUX_DUMMY0[i].cell(0),
                            wires::HSR_HDISTR_MUX_DUMMY1[i].cell(0),
                            wires::HSR_HDISTR_MUX_DUMMY2[i].cell(0),
                        ],
                    );
                }
                let mut xn = self.builder.xtile_id(tcid, "CLK_XPIO", xy);
                for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                    if let Some(bli_xy) = bli_xy {
                        xn = xn.ref_single(bli_xy, i + 1, bli);
                    }
                }
                xn.bels(bels)
                    .switchbox(bslots::HSR_CLK)
                    .optin_muxes(&wires::HSR_HDISTR[..])
                    .optin_muxes(&wires::HSR_HDISTR_W[..])
                    .optin_muxes(&wires::HSR_HDISTR_MUX[..])
                    .optin_muxes(&wires::HSR_HROUTE[..])
                    .optin_muxes(&wires::HSR_HROUTE_W[..])
                    .optin_muxes(&wires::HSR_HROUTE_TEST_MUX[..])
                    .optin_muxes(&wires::HSR_IMUX_BUFG[..])
                    .optin_muxes(&wires::HSR_IMUX_BUFG_HDISTR[..])
                    .optin_muxes(&wires::HSR_IMUX_BUFG_XPIO[..])
                    .optin_muxes(&wires::HSR_IMUX_BUFGCTRL_I0[..])
                    .optin_muxes(&wires::HSR_IMUX_BUFGCTRL_I1[..])
                    .optin_muxes(&[
                        wires::HSR_IMUX_DPLL_CLKIN,
                        wires::HSR_IMUX_DPLL_CLKIN_DESKEW,
                        wires::HSR_IMUX_MMCM_CLKIN1,
                        wires::HSR_IMUX_MMCM_CLKIN2,
                        wires::HSR_IMUX_MMCM_CLKFB,
                        wires::HSR_IMUX_DPLL_CLKIN_HDISTR,
                        wires::HSR_IMUX_MMCM_CLKIN1_HDISTR,
                        wires::HSR_IMUX_MMCM_CLKIN2_HDISTR,
                        wires::HSR_IMUX_MMCM_CLKFB_HDISTR,
                    ])
                    .optin_muxes(&wires::HSR_IMUX_MMCM_CLKIN_DESKEW[..])
                    .optin_muxes(&wires::HSR_IMUX_MMCM_CLKFB_DESKEW[..])
                    .extract();
            }
        }
    }

    pub fn fill_tiles_hsr_xpll(&mut self) {
        for tkn in ["CLK_PLL_AND_PHY", "CLK_PLL_AND_PHY_MX"] {
            for (name, wire) in [
                ("PLL0_CLKIN", wires::HSR_IMUX_XPLL_CLKIN[0].cell(0)),
                ("PLL1_CLKIN", wires::HSR_IMUX_XPLL_CLKIN[1].cell(0)),
                (
                    "PLL0_CLKIN_DESKEW0_MUXED",
                    wires::HSR_IMUX_XPLL_CLKIN_DESKEW0[0].cell(0),
                ),
                (
                    "PLL1_CLKIN_DESKEW0_MUXED",
                    wires::HSR_IMUX_XPLL_CLKIN_DESKEW0[1].cell(0),
                ),
                (
                    "PLL0_CLKIN_DESKEW1_MUXED",
                    wires::HSR_IMUX_XPLL_CLKIN_DESKEW1[0].cell(0),
                ),
                (
                    "PLL1_CLKIN_DESKEW1_MUXED",
                    wires::HSR_IMUX_XPLL_CLKIN_DESKEW1[1].cell(0),
                ),
                (
                    "PLL0_CLKFB_DESKEW0_MUXED",
                    wires::HSR_IMUX_XPLL_CLKFB_DESKEW0[0].cell(0),
                ),
                (
                    "PLL1_CLKFB_DESKEW0_MUXED",
                    wires::HSR_IMUX_XPLL_CLKFB_DESKEW0[1].cell(0),
                ),
                (
                    "PLL0_CLKFB_DESKEW1_MUXED",
                    wires::HSR_IMUX_XPLL_CLKFB_DESKEW1[0].cell(0),
                ),
                (
                    "PLL1_CLKFB_DESKEW1_MUXED",
                    wires::HSR_IMUX_XPLL_CLKFB_DESKEW1[1].cell(0),
                ),
                (
                    "CLK_CMT_MUX_36_ENC_0_CLK_OUT",
                    wires::HSR_IMUX_XPLL_CLKIN_MUX[0].cell(0),
                ),
                (
                    "CLK_CMT_MUX_36_ENC_1_CLK_OUT",
                    wires::HSR_IMUX_XPLL_CLKIN_MUX[1].cell(0),
                ),
                ("PLL0_CLKOUT0", wires::HSR_OUT_XPLL0[0].cell(0)),
                ("PLL0_CLKOUT1", wires::HSR_OUT_XPLL0[1].cell(0)),
                ("PLL0_CLKOUT2", wires::HSR_OUT_XPLL0[2].cell(0)),
                ("PLL0_CLKOUT3", wires::HSR_OUT_XPLL0[3].cell(0)),
                ("PLL1_CLKOUT0", wires::HSR_OUT_XPLL1[0].cell(0)),
                ("PLL1_CLKOUT1", wires::HSR_OUT_XPLL1[1].cell(0)),
                ("PLL1_CLKOUT2", wires::HSR_OUT_XPLL1[2].cell(0)),
                ("PLL1_CLKOUT3", wires::HSR_OUT_XPLL1[3].cell(0)),
                ("DMC_CLK_DESKEW", wires::HSR_OUT_DMC_CLK_DESKEW.cell(0)),
                ("HSM_REF_CLK0", wires::OUT_PS_HSM_REF_CLK[0].cell(0)),
                ("HSM_REF_CLK1", wires::OUT_PS_HSM_REF_CLK[1].cell(0)),
                ("PLL_CASCADE_IN0", wires::HSR_OUT_XPLL_E[0].cell(0)),
                ("PLL_CASCADE_IN1", wires::HSR_OUT_XPLL_E[1].cell(0)),
                ("PLL_CASCADE_IN2", wires::HSR_OUT_XPLL_W[0].cell(0)),
                ("PLL_CASCADE_IN3", wires::HSR_OUT_XPLL_W[1].cell(0)),
                ("CCIO_CASCADE_IN0", wires::HSR_OUT_CCIO_E[0].cell(0)),
                ("CCIO_CASCADE_IN1", wires::HSR_OUT_CCIO_E[1].cell(0)),
                ("CCIO_CASCADE_IN2", wires::HSR_OUT_CCIO_E[2].cell(0)),
                ("CCIO_CASCADE_IN3", wires::HSR_OUT_CCIO_E[3].cell(0)),
                ("CCIO_CASCADE_IN4", wires::HSR_OUT_CCIO_W[0].cell(0)),
                ("CCIO_CASCADE_IN5", wires::HSR_OUT_CCIO_W[1].cell(0)),
                ("CCIO_CASCADE_IN6", wires::HSR_OUT_CCIO_W[2].cell(0)),
                ("CCIO_CASCADE_IN7", wires::HSR_OUT_CCIO_W[3].cell(0)),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
            for i in 0..24 {
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("CLK_HDISTR{i}"),
                    wires::HSR_HDISTR[i].cell(0),
                );
            }
            for i in 0..6 {
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("GCLK2PHY{i}"),
                    wires::HSR_GCLK_XPIO[i].cell(3),
                );
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("GCLK2PHY{ii}", ii = i + 6),
                    wires::HSR_GCLK_XPIO[i].cell(4),
                );
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("PLL1_CLKFB_DESKEW0").unwrap();
                let (bli0_xy, _) = self.bli[&(xy, wi0)];
                let wi1 = self.rd.wires.get("PLL1_CLKFB_DESKEW1").unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];

                let mut xpll_xy = vec![];
                for i in 0..2 {
                    xpll_xy.push(self.follow_wire(xy, &format!("PLL{i}_CLKIN")).unwrap());
                }

                let clk_xy = self
                    .follow_wire_filter(xy, "CCIO_IN0", |n| n.starts_with("CLK_REBUF_BUFGS"))
                    .unwrap();

                let mut bels = vec![];
                for i in 0..2 {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::XPLL[i], "XPLL", 0, 0)
                        .raw_tile(1 + i)
                        .pin_name_only("CLKOUTPHY", 1)
                        .pin_name_only("CLKOUTPHY_CASC_IN", 1)
                        .pin_name_only("CLKOUTPHY_CASC_OUT", 1)
                        .pin_name_only("RIU_CLK", 1)
                        .pin_name_only("RIU_NIBBLE_SEL", 1)
                        .pin_name_only("RIU_WR_EN", 1)
                        .pin_name_only("RIU_VALID", 1);
                    for j in 0..8 {
                        bel = bel.pin_name_only(&format!("RIU_ADDR_{j}_"), 1);
                    }
                    for j in 0..16 {
                        bel = bel
                            .pin_name_only(&format!("RIU_RD_DATA_{j}_"), 1)
                            .pin_name_only(&format!("RIU_WR_DATA_{j}_"), 1);
                    }
                    bels.push(bel);
                }

                self.builder
                    .add_dummies(tkn)
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKIN_DESKEW0[0].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL0_CLKIN_DESKEW0_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKIN_DESKEW0[1].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL1_CLKIN_DESKEW0_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKIN_DESKEW1[0].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL0_CLKIN_DESKEW1_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKIN_DESKEW1[1].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL1_CLKIN_DESKEW1_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKFB_DESKEW0[0].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL0_CLKFB_DESKEW0_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKFB_DESKEW0[1].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL1_CLKFB_DESKEW0_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKFB_DESKEW1[0].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL0_CLKFB_DESKEW1_DUMMY.map(|w| w.cell(0)),
                    )
                    .insert(
                        wires::HSR_IMUX_XPLL_CLKFB_DESKEW1[1].cell(0),
                        true,
                        &wires::HSR_IMUX_XPLL1_CLKFB_DESKEW1_DUMMY.map(|w| w.cell(0)),
                    );

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                let clk = self.builder.ndb.get_tile_class_naming("CLK_XPIO");
                self.builder
                    .xtile_id(tcls::XPLL, "XPLL", xy)
                    .raw_tile(xpll_xy[0])
                    .raw_tile(xpll_xy[1])
                    .bels(bels)
                    .switchbox(bslots::XPLL_INT)
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKIN[..])
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKIN_MUX[..])
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKIN_DESKEW0[..])
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKIN_DESKEW1[..])
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKFB_DESKEW0[..])
                    .optin_muxes(&wires::HSR_IMUX_XPLL_CLKFB_DESKEW1[..])
                    .optin_muxes(&wires::HSR_GCLK_XPIO[..])
                    .ref_single(bli0_xy, 9, bli)
                    .ref_single(bli1_xy, 10, bli)
                    .ref_single(clk_xy, 0, clk)
                    .extract();
                let pips = self
                    .builder
                    .pips
                    .get_mut(&(tcls::XPLL, bslots::XPLL_INT))
                    .unwrap();
                for i in 0..9 {
                    pips.pips.insert(
                        (
                            wires::HSR_OUT_PHY[i].cell(0),
                            wires::HSR_OUT_XPIO_PHY[0]
                                .cell([0, 1, 2, 3, 5, 6, 7, 8, 4][i])
                                .pos(),
                        ),
                        PipMode::PermaBuf,
                    );
                }
                for i in 0..4 {
                    pips.pips.insert(
                        (
                            wires::HSR_OUT_CCIO[i].cell(0),
                            wires::HSR_OUT_XPIO_CCIO.cell(i + 2).pos(),
                        ),
                        PipMode::PermaBuf,
                    );
                    pips.pips.insert(
                        (
                            wires::HSR_OUT_CLK_CCIO[i].cell(0),
                            wires::HSR_OUT_XPIO_CCIO.cell(i + 2).pos(),
                        ),
                        PipMode::PermaBuf,
                    );
                }
            }
        }
    }

    pub fn fill_tiles_hsr_xpio(&mut self) {
        for tkn in ["XPIO_NIBBLE_SC", "XPIO_NIBBLE_SC_MX"] {
            for i in 0..6 {
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("XPIO_LOGICPAIR_0_GCLK_SIG_{i}_"),
                    wires::HSR_GCLK_XPIO[i].cell(0),
                );
            }
            self.builder.extra_name_tile_tw(
                tkn,
                "XPHY_CORE_3_FIFO_WR_CLK",
                wires::HSR_OUT_XPIO_PHY[0].cell(0),
            );
            self.builder.extra_name_tile_tw(
                tkn,
                "XPIO_IOBPAIR_5_RXOUT_M",
                wires::HSR_OUT_XPIO_CCIO.cell(0),
            );
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];

                let mut bels = vec![self.bel_name_only(xy, bslots::XPIO_VREF, "XPIO_VREF", 0, 0)];

                let wi0 = self
                    .builder
                    .rd
                    .wires
                    .get("XPIO_LOGICPAIR_0_IDLYM_CNTVALUEIN_0_")
                    .unwrap();
                if let Some(&(bli0_xy, _)) = self.bli.get(&(xy, wi0)) {
                    let wi1 = self
                        .builder
                        .rd
                        .wires
                        .get("XPIO_LOGICPAIR_1_IDLYS_CNTVALUEIN_0_")
                        .unwrap();
                    let (bli1_xy, _) = self.bli[&(xy, wi1)];

                    for i in 0..3 {
                        let mut bel = self
                            .builder
                            .bel_xy(bslots::XPIOB[i * 2], "IOB", 0, i)
                            .manual()
                            .pin_rename("TERM_EN", "_ORIG_TERM_EN")
                            .pin_name_only("_ORIG_TERM_EN", 1);
                        let slot = self.rd.slot_xy("IOB", 0, i).unwrap();
                        let tksite = tk.sites.get(&slot).unwrap().1;
                        for pin in tksite.pins.keys() {
                            bel = bel.pin_name_only(pin, 1);
                        }
                        if i == 0 {
                            bel = bel.extra_int_out("CLKPAD", &["XPIO_IOBPAIR_5_RXOUT_M"]);
                        }
                        bels.push(bel);
                        bels.push(self.builder.bel_virtual(bslots::XPIOB[i * 2 + 1]).manual());
                    }

                    for i in 0..3 {
                        let mut bel = self
                            .builder
                            .bel_xy(bslots::XPIOLOGIC[i * 2], "XPIOLOGIC", 0, i)
                            .manual();
                        let slot = self.rd.slot_xy("XPIOLOGIC", 0, i).unwrap();
                        let tksite = tk.sites.get(&slot).unwrap().1;
                        for pin in tksite.pins.keys() {
                            if pin.starts_with("XPHY")
                                || pin.starts_with("PHY")
                                || pin.starts_with("LGC_SCAN")
                                || matches!(
                                    pin.as_str(),
                                    "IPFFM_D"
                                        | "IPFFS_D"
                                        | "IPFFM_LSPD"
                                        | "IPFFS_LSPD"
                                        | "OPFFM_Q"
                                        | "OPFFS_Q"
                                        | "TFFM_Q"
                                        | "TFFS_Q"
                                )
                            {
                                bel = bel.pin_name_only(pin, 1);
                            }
                        }
                        bels.push(bel);
                        let mut bel = self
                            .builder
                            .bel_virtual(bslots::XPIOLOGIC[i * 2 + 1])
                            .manual();
                        for i in 0..6 {
                            bel = bel.extra_int_in(
                                format!("GCLK_SIG_{i}_"),
                                &[format!("XPIO_LOGICPAIR_0_GCLK_SIG_{i}_")],
                            )
                        }
                        bels.push(bel);
                    }

                    let mut bel = self.builder.bel_xy(bslots::XPHY, "XPHY", 0, 0).manual();
                    let slot = self.rd.slot_xy("XPHY", 0, 0).unwrap();
                    let tksite = tk.sites.get(&slot).unwrap().1;
                    for pin in tksite.pins.keys() {
                        if pin.starts_with("CE_")
                            || pin.starts_with("INC_")
                            || pin.starts_with("LD_")
                            || pin.starts_with("RX")
                            || pin.starts_with("TX")
                            || pin.starts_with("T_")
                            || pin.starts_with("CNTVALUE")
                            || pin.starts_with("CFG2PHY")
                            || pin.starts_with("PHY2CFG")
                            || pin.starts_with("PHY2CLB_SCAN_")
                            || pin.starts_with("CLB2PHY_SCAN_")
                            || pin.starts_with("IOB2PHY")
                            || pin.starts_with("PHY2IOB")
                            || pin.starts_with("DATAIN")
                            || pin.starts_with("O0_")
                            || pin.starts_with("DMC")
                            || (pin.starts_with("DYN_DCI") && !pin.starts_with("DYN_DCI_IN"))
                            || (pin.starts_with("IBUF_DISABLE")
                                && !pin.starts_with("IBUF_DISABLE_IN"))
                            || ((pin.starts_with("D0_")
                                || pin.starts_with("D1_")
                                || pin.starts_with("D2_")
                                || pin.starts_with("D3_")
                                || pin.starts_with("D4_")
                                || pin.starts_with("D5_"))
                                && !pin.ends_with("_7_"))
                            || (pin.starts_with("Q")
                                && (pin.ends_with("_0_")
                                    || pin.ends_with("_5_")
                                    || pin.ends_with("_6_")
                                    || pin.ends_with("_7_")))
                            || matches!(
                                pin.as_str(),
                                "CLK_TO_LOWER"
                                    | "CLK_TO_UPPER"
                                    | "CLK_FROM_OTHER_XPHY"
                                    | "NCLK_NIBBLE_IN"
                                    | "NCLK_NIBBLE_OUT"
                                    | "PCLK_NIBBLE_IN"
                                    | "PCLK_NIBBLE_OUT"
                                    | "PLL0_CLK"
                                    | "PLL1_CLK"
                                    | "CLB2PHY_CSSD_TRIG"
                            )
                        {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                    bels.push(bel);

                    let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                    let mut xt = self
                        .builder
                        .xtile_id(tcls::XPIO, "XPIO", xy)
                        .ref_single(bli0_xy, 1, bli)
                        .ref_single(bli1_xy, 2, bli)
                        .bels(bels)
                        .extract();

                    let (mut bel_xphy, bn_xphy) = xt.bels.pop().unwrap();
                    let mut bels_xpiologic = xt.bels.split_off(6);
                    let mut bels_xpiob = xt.bels;
                    let mut bels_nibbleslice = vec![
                        (
                            TempBel::default(),
                            BelNaming {
                                tiles: bn_xphy.tiles.clone(),
                                ..Default::default()
                            }
                        );
                        6
                    ];

                    for i in 0..3 {
                        let [(bel0, bn0), (bel1, bn1)] =
                            bels_xpiologic.get_disjoint_mut([2 * i, 2 * i + 1]).unwrap();
                        let mut new_pins = BTreeMap::new();
                        bel0.pins.retain(|pname, pin| {
                            if pname.starts_with("GCLK") {
                                true
                            } else {
                                let (pref, suf) = pname.split_once('_').unwrap();
                                let (pref, j) = if let Some(pref) = pref.strip_suffix('M') {
                                    (pref, 0)
                                } else if let Some(pref) = pref.strip_suffix('S') {
                                    (pref, 1)
                                } else {
                                    unreachable!()
                                };
                                let npname = format!("{pref}_{suf}");
                                let pn = bn0.pins[pname].clone();
                                if j == 0 {
                                    bn0.pins.insert(npname.clone(), pn);
                                    new_pins.insert(npname, pin.clone());
                                } else {
                                    bn1.pins.insert(npname.clone(), pn);
                                    bel1.pins.insert(npname, pin.clone());
                                }
                                false
                            }
                        });
                        bel0.pins.extend(new_pins);
                    }

                    let mut move_pin = |idx: usize, dst: &str, src: &str| {
                        let pin = bel_xphy.pins.remove(src).unwrap();
                        let pn = bn_xphy.pins[src].clone();
                        bels_xpiob[idx].0.pins.insert(dst.into(), pin);
                        bels_xpiob[idx].1.pins.insert(dst.into(), pn);
                    };
                    for i in 0..6 {
                        move_pin(i, "KEEPER_EN", &format!("CLB2PHY_KEEPER_EN_{i}_"));
                        move_pin(i, "DYNAMIC_DCI_TS", &format!("DYN_DCI_IN_{i}_"));
                        move_pin(i, "IBUF_DISABLE", &format!("IBUF_DISABLE_IN_{i}_"));
                    }
                    for i in 0..3 {
                        move_pin(2 * i, "PD_EN", &format!("CLB2PHY_LP_TX_T_OR_PD_EN_0_{i}_"));
                        move_pin(2 * i + 1, "PD_EN", &format!("CLB2PHY_PD_EN_1_{i}_"));
                        move_pin(2 * i, "PU_EN", &format!("CLB2PHY_LP_TX_O_OR_PU_EN_0_{i}_"));
                        move_pin(
                            2 * i + 1,
                            "PU_EN",
                            &format!("CLB2PHY_LP_TX_O_OR_PU_EN_1_{i}_"),
                        );
                        move_pin(
                            2 * i,
                            "TERM_EN",
                            &format!("CLB2PHY_LP_RX_DIS_OR_TERM_EN_{i}_"),
                        );
                        move_pin(2 * i, "LP_I", &format!("PHY2CLB_LP_I_0_{i}_"));
                        move_pin(2 * i + 1, "LP_I", &format!("PHY2CLB_LP_I_1_{i}_"));
                    }
                    let mut move_pin = |idx: usize, dst: &str, src: &str| {
                        let pin = bel_xphy.pins.remove(src).unwrap();
                        let pn = bn_xphy.pins[src].clone();
                        bels_nibbleslice[idx].0.pins.insert(dst.into(), pin);
                        bels_nibbleslice[idx].1.pins.insert(dst.into(), pn);
                    };
                    for i in 0..6 {
                        for j in 1..=4 {
                            move_pin(i, &format!("Q_{j}_"), &format!("Q{i}_{j}_"));
                        }
                        move_pin(i, "D_7_", &format!("D{i}_7_"));
                    }
                    for i in 0..6 {
                        for (dst, src) in [
                            ("D_0_", "OPFF_D1"),
                            ("D_1_", "OPFF_D2"),
                            ("D_2_", "TFF_D2"),
                            ("D_3_", "OPFF_RST"),
                            ("D_4_", "ODLY_CNTVALUEIN_4_"),
                            ("D_5_", "IPFF_CE"),
                            ("D_6_", "IPFF_RST"),
                            ("Q_0_", "ODLY_CNTVALUEOUT_4_"),
                            ("Q_5_", "IPFF_BYPASS"),
                            ("Q_6_", "IPFF_Q2"),
                            ("Q_7_", "IPFF_Q1"),
                            ("CNTVALUEIN_0_", "IDLY_CNTVALUEIN_0_"),
                            ("CNTVALUEIN_1_", "IDLY_CNTVALUEIN_1_"),
                            ("CNTVALUEIN_2_", "IDLY_CNTVALUEIN_2_"),
                            ("CNTVALUEIN_3_", "IDLY_CNTVALUEIN_3_"),
                            ("CNTVALUEIN_4_", "IDLY_CNTVALUEIN_4_"),
                            ("CNTVALUEIN_5_", "ODLY_CNTVALUEIN_0_"),
                            ("CNTVALUEIN_6_", "ODLY_CNTVALUEIN_1_"),
                            ("CNTVALUEIN_7_", "ODLY_CNTVALUEIN_2_"),
                            ("CNTVALUEIN_8_", "ODLY_CNTVALUEIN_3_"),
                            ("CNTVALUEOUT_0_", "IDLY_CNTVALUEOUT_0_"),
                            ("CNTVALUEOUT_1_", "IDLY_CNTVALUEOUT_1_"),
                            ("CNTVALUEOUT_2_", "IDLY_CNTVALUEOUT_2_"),
                            ("CNTVALUEOUT_3_", "IDLY_CNTVALUEOUT_3_"),
                            ("CNTVALUEOUT_4_", "IDLY_CNTVALUEOUT_4_"),
                            ("CNTVALUEOUT_5_", "ODLY_CNTVALUEOUT_0_"),
                            ("CNTVALUEOUT_6_", "ODLY_CNTVALUEOUT_1_"),
                            ("CNTVALUEOUT_7_", "ODLY_CNTVALUEOUT_2_"),
                            ("CNTVALUEOUT_8_", "ODLY_CNTVALUEOUT_3_"),
                            // TODO: RX/TX_RST, T, *_EN_VTC, RXTX_SEL, CE, INC, LD
                        ] {
                            let pin = bels_xpiologic[i].0.pins[src].clone();
                            let mut pn = bels_xpiologic[i].1.pins[src].clone();
                            pn.pips.clear();
                            pn.name = pn.name_far.clone();
                            bels_nibbleslice[i].0.pins.insert(dst.into(), pin);
                            bels_nibbleslice[i].1.pins.insert(dst.into(), pn);
                        }
                    }

                    for (i, (bel, bn)) in bels_xpiob.into_iter().enumerate() {
                        self.builder
                            .insert_tcls_bel(tcls::XPIO, bslots::XPIOB[i], bel.into());
                        self.builder.insert_bel_naming("XPIO", bslots::XPIOB[i], bn);
                    }
                    for (i, (bel, bn)) in bels_xpiologic.into_iter().enumerate() {
                        self.builder
                            .insert_tcls_bel(tcls::XPIO, bslots::XPIOLOGIC[i], bel.into());
                        self.builder
                            .insert_bel_naming("XPIO", bslots::XPIOLOGIC[i], bn);
                    }
                    for (i, (bel, bn)) in bels_nibbleslice.into_iter().enumerate() {
                        self.builder.insert_tcls_bel(
                            tcls::XPIO,
                            bslots::XPHY_NIBBLESLICE[i],
                            bel.into(),
                        );
                        self.builder
                            .insert_bel_naming("XPIO", bslots::XPHY_NIBBLESLICE[i], bn);
                    }
                    self.builder
                        .insert_tcls_bel(tcls::XPIO, bslots::XPHY, bel_xphy.into());
                    self.builder
                        .insert_bel_naming("XPIO", bslots::XPHY, bn_xphy);
                } else {
                    for i in 0..3 {
                        let mut bel = self
                            .builder
                            .bel_xy(bslots::XPIOB[i * 2], "IOB", 0, i)
                            .pin_rename("TERM_EN", "_ORIG_TERM_EN")
                            .pin_name_only("_ORIG_TERM_EN", 1);
                        let slot = self.rd.slot_xy("IOB", 0, i).unwrap();
                        let tksite = tk.sites.get(&slot).unwrap().1;
                        for pin in tksite.pins.keys() {
                            bel = bel.pin_name_only(pin, 1);
                        }
                        if i == 0 {
                            bel = bel.extra_int_out("CLKPAD", &["XPIO_IOBPAIR_5_RXOUT_M"]);
                        }
                        bels.push(bel);
                        bels.push(self.builder.bel_virtual(bslots::XPIOB[i * 2 + 1]));
                    }

                    for i in 0..3 {
                        let mut bel =
                            self.builder
                                .bel_xy(bslots::XPIOLOGIC[i * 2], "XPIOLOGIC", 0, i);
                        let slot = self.rd.slot_xy("XPIOLOGIC", 0, i).unwrap();
                        let tksite = tk.sites.get(&slot).unwrap().1;
                        for pin in tksite.pins.keys() {
                            if !pin.starts_with("GCLK_SIG") {
                                bel = bel.pin_name_only(pin, 1);
                            }
                        }
                        bels.push(bel);
                        let mut bel = self.builder.bel_virtual(bslots::XPIOLOGIC[i * 2 + 1]);
                        for i in 0..6 {
                            bel = bel.extra_int_in(
                                format!("GCLK_SIG_{i}_"),
                                &[format!("XPIO_LOGICPAIR_0_GCLK_SIG_{i}_")],
                            )
                        }
                        bels.push(bel);
                    }

                    for i in 0..6 {
                        bels.push(self.builder.bel_virtual(bslots::XPHY_NIBBLESLICE[i]));
                    }

                    let mut bel = self.builder.bel_xy(bslots::XPHY, "XPHY", 0, 0);
                    let slot = self.rd.slot_xy("XPHY", 0, 0).unwrap();
                    let tksite = tk.sites.get(&slot).unwrap().1;
                    for pin in tksite.pins.keys() {
                        if pin != "FIFO_WR_CLK" {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                    bels.push(bel);

                    self.builder
                        .xtile_id(tcls::XPIO_NC, "XPIO_NC", xy)
                        .bels(bels)
                        .extract();
                }
            }
        }
    }

    pub fn fill_tiles_hsr_xpio_dci(&mut self) {
        for tkn in ["XPIO_MISC_CORE", "XPIO_MISC_CORE_MX"] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let wi0 = self.rd.wires.get("DCI_OK_TO_UPDATE").unwrap();
                let (bli0_xy, bli0_wi) = self.bli[&(xy, wi0)];
                let wn0 = self.rd.wires[bli0_wi].as_str();
                let (tcid, bli_xy) = match wn0 {
                    "BLI_GRP0_A_BLI_IMUX15" => (tcls::XPIO_DCI_S, vec![None, Some(bli0_xy)]),
                    "BLI_GRP0_C_BLI_IMUX0" => (tcls::XPIO_DCI_A, vec![Some(bli0_xy), None]),
                    "BLI_GRP0_B_BLI_IMUX0" => (tcls::XPIO_DCI_B, vec![Some(bli0_xy), None]),
                    "BLI_GRP0_A_BLI_IMUX0" => (tcls::XPIO_DCI_C, vec![Some(bli0_xy), None]),
                    _ => panic!("ummm {wn0}"),
                };

                let bel = self
                    .builder
                    .bel_xy(bslots::XPIO_DCI, "XPIO_DCI", 0, 0)
                    .pin_name_only("CLB2MISC_SCAN_CLK_B", 1)
                    .pin_name_only("CLB2MISC_SCAN_MODE_RST_B", 1)
                    .pin_name_only("CLB2MISC_SCAN_EN_B", 1);

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                let mut xn = self.builder.xtile_id(tcid, "XPIO_DCI", xy);
                for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                    if let Some(bli_xy) = bli_xy {
                        xn = xn.ref_single(bli_xy, i, bli);
                    }
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_hsr_xpio_scan(&mut self) {
        for tkn in ["XPIO_MISC_CORE", "XPIO_MISC_CORE_MX"] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let ni = self.rd.lookup_wire(xy, "CLB2MISC_SCAN_CLK_B").unwrap();
                let NodeOrWire::Node(ni) = ni else {
                    unreachable!()
                };
                let mut nibble_xy = Vec::from_iter(
                    self.rd
                        .iter_node(ni)
                        .map(|(crd, _)| crd)
                        .filter(|&crd| self.rd.tiles[&crd].name.starts_with("XPIO_NIBBLE")),
                );
                if self.builder.is_mirror_square && xy.x >= self.rd.width / 2 {
                    nibble_xy.sort_by_key(|crd| core::cmp::Reverse(crd.x));
                } else {
                    nibble_xy.sort_by_key(|crd| crd.x);
                }
                assert_eq!(nibble_xy.len(), 9);

                let wi0 = self.rd.wires.get("CLB2MISC_SCAN_EN_B").unwrap();
                let (bli0_xy, bli0_wi) = self.bli[&(xy, wi0)];
                let wi1 = self.rd.wires.get("CLB2MISC_SCAN_CLK_B").unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];
                let wi2 = self.rd.wires.get("XPHY_CORE_3_CLB2PHY_CSSD_TRIG").unwrap();
                let (bli2_xy, _) = self.bli[&(nibble_xy[0], wi2)];
                let wn0 = self.rd.wires[bli0_wi].as_str();
                let (tcid, bli_xy) = match wn0 {
                    "BLI_GRP3_A_BLI_CTRL2" => {
                        (tcls::XPIO_SCAN_S, vec![Some(bli1_xy), Some(bli0_xy)])
                    }
                    "BLI_GRP2_A_BLI_IMUX12" => {
                        (tcls::XPIO_SCAN_A, vec![Some(bli2_xy), Some(bli0_xy)])
                    }
                    "BLI_GRP2_B_BLI_IMUX12" => {
                        (tcls::XPIO_SCAN_B, vec![Some(bli2_xy), Some(bli0_xy)])
                    }
                    "BLI_GRP2_C_BLI_IMUX12" => {
                        (tcls::XPIO_SCAN_C, vec![Some(bli2_xy), Some(bli0_xy)])
                    }
                    _ => panic!("ummm {wn0}"),
                };

                let mut bel = self
                    .builder
                    .bel_virtual(bslots::XPIO_SCAN)
                    .extra_int_in("SCAN_CLK_B", &["CLB2MISC_SCAN_CLK_B"])
                    .extra_int_in("SCAN_EN_B", &["CLB2MISC_SCAN_EN_B"])
                    .extra_int_in("SCAN_MODE_RST_B", &["CLB2MISC_SCAN_MODE_RST_B"]);
                for i in 0..9 {
                    bel = bel
                        .sub_virtual()
                        .raw_tile(i + 1)
                        .extra_int_in(
                            format!("CSSD_TRIG_XPHY_{i}_"),
                            &["XPHY_CORE_3_CLB2PHY_CSSD_TRIG"],
                        )
                        .extra_int_in(
                            format!("SCAN_IN_XPHY_{i}_"),
                            &["XPHY_CORE_3_CLB2PHY_SCAN_IN"],
                        )
                        .extra_int_in(
                            format!("SCAN_CNTRL_CHNL_IN_XPHY_{i}_"),
                            &["XPHY_CORE_3_CLB2PHY_SCAN_CNTRL_CHNL_IN"],
                        )
                        .extra_int_in(
                            format!("SCAN_IN_IOL_{i}_"),
                            &["XPIO_LOGICPAIR_0_LGC_SCAN_IN"],
                        )
                        .extra_int_out(
                            format!("SCAN_OUT_XPHY_{i}_"),
                            &["XPHY_CORE_3_PHY2CLB_SCAN_OUT"],
                        )
                        .extra_int_out(
                            format!("SCAN_CNTRL_CHNL_OUT_XPHY_{i}_"),
                            &["XPHY_CORE_3_PHY2CLB_SCAN_CNTRL_CHNL_OUT"],
                        )
                        .extra_int_out(
                            format!("SCAN_OUT_IOL_{i}_"),
                            &["XPIO_LOGICPAIR_2_LGC_SCANOUT"],
                        );
                    if i == 0 {
                        bel = bel.extra_int_in(
                            "SCAN_EDT_UPDT_B",
                            &["XPHY_CORE_3_CLB2PHY_SCAN_EDT_UPDT_B"],
                        )
                    }
                }

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                let mut xn = self.builder.xtile_id(tcid, "XPIO_SCAN", xy);
                for nxy in nibble_xy {
                    xn = xn.raw_tile(nxy);
                }
                for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                    if let Some(bli_xy) = bli_xy {
                        xn = xn.ref_single(bli_xy, i, bli);
                    }
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_hsr_ddrmc(&mut self) {
        for (tkn, tkn_riu) in [
            ("DDRMC_DMC_CORE", "DDRMC_RIU_CORE"),
            ("DDRMC_DMC_CORE_MX", "DDRMC_RIU_CORE_MX"),
        ] {
            for (name, wire) in [
                ("DDRMC_MAIN_0_DBG_PLL1_CLK", wires::HSR_OUT_XPLL1[1].cell(0)),
                ("DDRMC_MAIN_0_DMC_CLK", wires::HSR_OUT_XPLL0[1].cell(0)),
                (
                    "DDRMC_MAIN_0_DMC_CLK_DESKEW",
                    wires::HSR_OUT_DMC_CLK_DESKEW.cell(0),
                ),
                (
                    "DDRMC_MAIN_0_TEST_CLK_OUT_0_",
                    wires::HSR_CLK_TEST_IN[0].cell(0),
                ),
                (
                    "DDRMC_MAIN_0_TEST_CLK_OUT_1_",
                    wires::HSR_CLK_TEST_IN[1].cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let &riu_xy = self.rd.tiles_by_kind_name(tkn_riu).iter().next().unwrap();

                let wi0 = self
                    .rd
                    .wires
                    .get("DDRMC_MAIN_0_IF_UBLAZE_FABRIC_UB_MISC_OUT_0_")
                    .unwrap();
                let (bli0_xy, _) = self.bli[&(xy, wi0)];
                let wi1 = self
                    .rd
                    .wires
                    .get("DDRMC_MAIN_0_IF_UBLAZE_FABRIC_FABRIC_CLK")
                    .unwrap();
                let (bli1_xy, _) = self.bli[&(xy, wi1)];
                let wi2 = self
                    .rd
                    .wires
                    .get("DDRMC_MAIN_0_IF_DMC_FABRIC_DMC_MISC_OUT_16_")
                    .unwrap();
                let (bli2_xy, _) = self.bli[&(xy, wi2)];
                let wi3 = self
                    .rd
                    .wires
                    .get("DDRMC_MAIN_0_IF_DMC_FABRIC_BLI2UB_UART_RX")
                    .unwrap();
                let (bli3_xy, _) = self.bli[&(xy, wi3)];

                let mut bel = self.builder.bel_xy(bslots::DDRMC, "DDRMC", 0, 0);

                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                let slot = self.rd.slot_xy("DDRMC", 0, 0).unwrap();
                let tksite = tk.sites.get(&slot).unwrap().1;
                for pin in tksite.pins.keys() {
                    if pin.starts_with("DMC2PHY")
                        || pin.starts_with("PHY2DMC")
                        || pin.starts_with("FROM_NOC")
                        || pin.starts_with("TO_NOC")
                    {
                        bel = bel.pin_name_only(pin, 1);
                    }
                }

                // what. the fuck. is it. this time.
                for i in 25..32 {
                    bel = bel.pin_rename(
                        format!("IF_DMC_FABRIC_PADDR_{i}_"),
                        format!("IF_UBLAZE_FABRIC_PADDR_{i}_"),
                    );
                }
                for i in 29..32 {
                    bel = bel.pin_rename(
                        format!("IF_DMC_FABRIC_PRDATA_{i}_"),
                        format!("IF_UBLAZE_FABRIC_PRDATA_{i}_"),
                    );
                }

                bel = bel.sub_xy("DDRMC_RIU", 0, 0).raw_tile(1);
                let tile = &self.rd.tiles[&riu_xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                let slot = self.rd.slot_xy("DDRMC_RIU", 0, 0).unwrap();
                let tksite = tk.sites.get(&slot).unwrap().1;
                for pin in tksite.pins.keys() {
                    bel = bel.pin_name_only(pin, 1);
                }

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
                self.builder
                    .xtile_id(tcls::DDRMC, "DDRMC", xy)
                    .raw_tile(riu_xy)
                    .bel(bel)
                    .ref_single(bli0_xy, 1, bli)
                    .ref_single(bli1_xy, 2, bli)
                    .ref_single(bli2_xy, 3, bli)
                    .ref_single(bli3_xy, 4, bli)
                    .extract();
            }
        }
    }
}
