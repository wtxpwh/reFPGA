use prjcombine_interconnect::db::WireSlotIdExt;
use prjcombine_versal::defs::{bslots, tcls, wires};

use crate::int::IntMaker;

impl<'rd> IntMaker<'rd> {
    pub fn fill_tiles_hsr_x5io(&mut self) {
        for tkn in ["CMPHY_OCTAD_CORE", "CMPHY_OCTAD_CORE_MY"] {
            for (pin, wire) in [
                ("RXFIFO_WR_CLK_0_", wires::HSR_OUT_XPIO_PHY[0]),
                ("RXFIFO_WR_CLK_1_", wires::HSR_OUT_XPIO_PHY[1]),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "CMPHY_OCTAD", 0, 0, pin, wire.cell(0), true);
            }
        }
        for tkn in ["X5PHIOBANK_XCVR_HB_X32", "X5PHIOBANK_XCVR_HB_X32_MY"] {
            for i in 0..4 {
                for j in 0..4 {
                    for (pin, wire) in [
                        ("PLL_CLK0", wires::HSR_IMUX_X5PHIO_PLL_CLK0_DCC),
                        ("PLL_CLK90", wires::HSR_IMUX_X5PHIO_PLL_CLK90_DCC),
                        ("XCV2XCV_PDQS_IN", wires::HSR_IMUX_X5PHIO_PDQS_IN),
                        ("XCV2XCV_NDQS_IN", wires::HSR_IMUX_X5PHIO_NDQS_IN),
                        ("XCV2XCV_PDQS_O", wires::HSR_OUT_X5PHIO_PDQS_O),
                        ("XCV2XCV_NDQS_O", wires::HSR_OUT_X5PHIO_NDQS_O),
                        ("XCV2XCV_PDQS_DLY_O", wires::HSR_OUT_X5PHIO_PDQS_DLY_O),
                        ("XCV2XCV_NDQS_DLY_O", wires::HSR_OUT_X5PHIO_NDQS_DLY_O),
                        (
                            "XCV2CGL_RX0_RDQS_P_OUT_CLK",
                            wires::HSR_OUT_X5PHIO_RX0_RDQS_P_OUT,
                        ),
                        (
                            "XCV2CGL_RX1_RDQS_P_OUT_CLK",
                            wires::HSR_OUT_X5PHIO_RX1_RDQS_P_OUT,
                        ),
                        ("DQS_DLY_OUT_R", wires::HSR_OUT_X5PHIO_DQS_DLY_OUT_R),
                        ("DQS_IN_R", wires::HSR_IMUX_X5PHIO_DQS_IN_R),
                        ("XCV2PHYPLL_RX_OUT_M", wires::HSR_OUT_X5IO_CCIO[0]),
                        ("XCV2PHYPLL_RX_OUT_S", wires::HSR_OUT_X5IO_CCIO[1]),
                        ("XCV2CLK_DIV64_CLK", wires::HSR_OUT_X5PHIO_DIV64_CLK),
                    ] {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "IOB",
                            0,
                            i * 4 + j,
                            pin,
                            wire.cell(0),
                            true,
                        );
                    }
                }
            }
            for &xcv_xy in self.rd.tiles_by_kind_name(tkn) {
                for i in 0..4 {
                    let phy_xy = self
                        .follow_wire(
                            xcv_xy,
                            &format!("X5PHIOBANK_XCVR_X2_{ii}_XCV2PHY_RD_DQ_0_", ii = i * 4),
                        )
                        .unwrap();

                    let bli0_xy = self.xlat_bli(phy_xy, "CMPHY_OCTAD_0_D0_0_");
                    let bli1_xy = self.xlat_bli(phy_xy, "CMPHY_OCTAD_0_D4_0_");

                    let mut bel = self.builder.bel_xy(bslots::X5IO_OCTAD, "CMPHY_OCTAD", 0, 0);
                    for pin in self.bel_pins(phy_xy, "CMPHY_OCTAD", 0, 0) {
                        if (bli0_xy.is_none() && !(pin.starts_with("RXFIFO_WR_CLK")))
                            || pin.starts_with("APB2DMC")
                            || pin.starts_with("DMC2APB")
                            || pin.starts_with("DMC_")
                            || pin.starts_with("MUX2")
                            || pin.starts_with("PHY2XCV")
                            || pin.starts_with("RIU2XCV")
                            || pin.starts_with("SCAN")
                            || pin.starts_with("SEQ_DONE")
                            || pin.starts_with("VT")
                            || pin.starts_with("XCV2PHY")
                            || pin.starts_with("XCV2RIU")
                        {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                    for j in 0..4 {
                        bel = bel.sub_xy("IOB", 0, i * 4 + j).raw_tile(1);
                        for pin in self.bel_pins(xcv_xy, "IOB", 0, i * 4 + j) {
                            if j == 1
                                && matches!(
                                    pin,
                                    "PLL_CLK0" | "PLL_CLK90" | "DQS_IN_R" | "DQS_DLY_OUT_R"
                                )
                            {
                                // already good
                            } else if j == 1
                                && matches!(
                                    pin,
                                    "XCV2XCV_PDQS_IN"
                                        | "XCV2XCV_NDQS_IN"
                                        | "XCV2XCV_PDQS_O"
                                        | "XCV2XCV_NDQS_O"
                                        | "XCV2XCV_PDQS_DLY_O"
                                        | "XCV2XCV_NDQS_DLY_O"
                                )
                            {
                                let npin = pin.strip_prefix("XCV2XCV_").unwrap();
                                bel = bel.pin_rename(pin, npin);
                            } else if j == 1
                                && matches!(
                                    pin,
                                    "XCV2CGL_RX0_RDQS_P_OUT_CLK" | "XCV2CGL_RX1_RDQS_P_OUT_CLK"
                                )
                            {
                                let npin = pin
                                    .strip_prefix("XCV2CGL_")
                                    .unwrap()
                                    .strip_suffix("_CLK")
                                    .unwrap();
                                bel = bel.pin_rename(pin, npin);
                            } else if j == 3
                                && let Some(npin) = match pin {
                                    "XCV2PHYPLL_RX_OUT_M" => Some("CCIO_0_"),
                                    "XCV2PHYPLL_RX_OUT_S" => Some("CCIO_1_"),
                                    "XCV2CLK_DIV64_CLK" => Some("DIV64_CLK"),
                                    _ => None,
                                }
                            {
                                bel = bel.pin_rename(pin, npin);
                            } else {
                                let npin = &format!("IOB{j}_{pin}");
                                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                            }
                        }
                    }

                    let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                    let tcid = if bli0_xy.is_some() {
                        tcls::X5IO
                    } else {
                        tcls::X5IO_NC
                    };
                    let mut xt = self
                        .builder
                        .xtile_id(tcid, format!("X5IO{i}"), phy_xy)
                        .raw_tile(xcv_xy)
                        .bel(bel);
                    if let Some((bli0_xy, bli1_xy)) = bli0_xy.zip(bli1_xy) {
                        xt = xt.ref_single(bli0_xy, 1, bli).ref_single(bli1_xy, 2, bli);
                    }
                    xt.extract();
                }
            }
        }
    }

    pub fn fill_tiles_hsr_x5pll(&mut self) {
        for tkn in [
            "X5PHIOBANK_CLKTREE_X32_CORE",
            "X5PHIOBANK_CLKTREE_X32_CORE_MY",
        ] {
            for i in 0..4 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_DCCIBUF",
                    0,
                    i * 2,
                    "X5PHIO_DCCIBUF_A",
                    wires::HSR_IMUX_X5PHIO_PLL_CLK0.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_DCCIBUF",
                    0,
                    i * 2 + 1,
                    "X5PHIO_DCCIBUF_A",
                    wires::HSR_IMUX_X5PHIO_PLL_CLK90.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_DCCIBUF",
                    0,
                    i * 2,
                    "X5PHIO_DCCIBUF_ZN",
                    wires::HSR_IMUX_X5PHIO_PLL_CLK0_DCC.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_DCCIBUF",
                    0,
                    i * 2 + 1,
                    "X5PHIO_DCCIBUF_ZN",
                    wires::HSR_IMUX_X5PHIO_PLL_CLK90_DCC.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_HARD_INV",
                    0,
                    i * 2,
                    "X5PHIO_O",
                    wires::HSR_IMUX_X5PHIO_PLL_CLK0_DCC_INV.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_HARD_INV",
                    0,
                    i * 2 + 1,
                    "X5PHIO_I",
                    wires::HSR_OUT_X5PHIO_RX0_RDQS_P_OUT.cell(i),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "X5PHIO_HARD_INV",
                    0,
                    i * 2 + 1,
                    "X5PHIO_O",
                    wires::HSR_OUT_X5PHIO_RX0_RDQS_P_OUT_INV.cell(i),
                    true,
                );
            }
            for (name, wire) in [
                ("CLK2PLL_CLKIN", wires::HSR_IMUX_X5PLL_CLKIN.cell(0)),
                (
                    "CLK2PLL_CLKIN_DESKEW0",
                    wires::HSR_IMUX_X5PLL_CLKIN_DESKEW0.cell(0),
                ),
                (
                    "CLK2PLL_CLKIN_DESKEW1",
                    wires::HSR_IMUX_X5PLL_CLKIN_DESKEW1.cell(0),
                ),
                (
                    "CLK2PLL_CLKFB_DESKEW0",
                    wires::HSR_IMUX_X5PLL_CLKFB_DESKEW0.cell(0),
                ),
                (
                    "CLK2PLL_CLKFB_DESKEW1",
                    wires::HSR_IMUX_X5PLL_CLKFB_DESKEW1.cell(0),
                ),
                ("PLL2CLK_CLKOUT0", wires::HSR_OUT_X5PLL[0].cell(0)),
                ("PLL2CLK_CLKOUT1", wires::HSR_OUT_X5PLL[1].cell(0)),
                ("PLL2CLK_CLKOUT2", wires::HSR_OUT_X5PLL[2].cell(0)),
                ("PLL2CLK_CLKOUT3", wires::HSR_OUT_X5PLL[3].cell(0)),
                ("PLL2CLK_TMUXOUT", wires::HSR_OUT_X5PLL[4].cell(0)),
                ("PLL2CLK_CLK0", wires::HSR_OUT_X5PLL_PHY[0].cell(0)),
                ("PLL2CLK_CLK90", wires::HSR_OUT_X5PLL_PHY[1].cell(0)),
                (
                    "CLK2CLK_CLK0_NEXT_HB_IN",
                    wires::HSR_OUT_X5PLL_PHY_NB[0].cell(0),
                ),
                (
                    "CLK2CLK_CLK90_NEXT_HB_IN",
                    wires::HSR_OUT_X5PLL_PHY_NB[1].cell(0),
                ),
                ("CLK2XCV_PLL_TESTCLK0", wires::HSR_X5IO_TESTCLK[0].cell(0)),
                ("CLK2XCV_PLL_TESTCLK1", wires::HSR_X5IO_TESTCLK[1].cell(0)),
                ("PHY2CLK_GCLK", wires::HSR_X5IO_GCLK[0].cell(0)),
                ("PHY2CLK_HSM_REFCLK0", wires::OUT_PS_HSM_REF_CLK[0].cell(4)),
                ("PHY2CLK_HSM_REFCLK1", wires::OUT_PS_HSM_REF_CLK[1].cell(4)),
                ("PHY2CLK_RXCLKIN", wires::HSR_X5IO_RXCLK.cell(0)),
                ("CLK2PHY_DMC_CLK", wires::HSR_IMUX_DMC_CLK.cell(0)),
                ("XCV2CLK_CCIO0", wires::HSR_OUT_X5IO_CCIO[0].cell(0)),
                ("XCV2CLK_CCIO1", wires::HSR_OUT_X5IO_CCIO[1].cell(0)),
                (
                    "CLK2CLK_CCIO_IN_NEXT_HB0",
                    wires::HSR_OUT_X5IO_CCIO_NB[0].cell(0),
                ),
                (
                    "CLK2CLK_CCIO_IN_NEXT_HB1",
                    wires::HSR_OUT_X5IO_CCIO_NB[1].cell(0),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_RIGHT0",
                    wires::HSR_OUT_CCIO_W[0].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_RIGHT1",
                    wires::HSR_OUT_CCIO_W[1].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_RIGHT2",
                    wires::HSR_OUT_CCIO_W[2].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_RIGHT3",
                    wires::HSR_OUT_CCIO_W[3].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_LEFT0",
                    wires::HSR_OUT_CCIO_E[0].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_LEFT1",
                    wires::HSR_OUT_CCIO_E[1].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_LEFT2",
                    wires::HSR_OUT_CCIO_E[2].cell(5),
                ),
                (
                    "CLK2CLK_CCIO_IN_BANK_LEFT3",
                    wires::HSR_OUT_CCIO_E[3].cell(5),
                ),
                ("CGL2XCV_PDQS_OCTAD0", wires::HSR_OUT_X5PHIO_PDQS_O.cell(0)),
                ("CGL2XCV_PDQS_OCTAD1", wires::HSR_OUT_X5PHIO_PDQS_O.cell(1)),
                ("CGL2XCV_PDQS_OCTAD2", wires::HSR_OUT_X5PHIO_PDQS_O.cell(2)),
                ("CGL2XCV_PDQS_OCTAD3", wires::HSR_OUT_X5PHIO_PDQS_O.cell(3)),
                ("CGL2XCV_NDQS_OCTAD0", wires::HSR_OUT_X5PHIO_NDQS_O.cell(0)),
                ("CGL2XCV_NDQS_OCTAD1", wires::HSR_OUT_X5PHIO_NDQS_O.cell(1)),
                ("CGL2XCV_NDQS_OCTAD2", wires::HSR_OUT_X5PHIO_NDQS_O.cell(2)),
                ("CGL2XCV_NDQS_OCTAD3", wires::HSR_OUT_X5PHIO_NDQS_O.cell(3)),
                (
                    "XCV2CGL_PDQS_DLY_OCTAD0",
                    wires::HSR_OUT_X5PHIO_PDQS_DLY_O.cell(0),
                ),
                (
                    "XCV2CGL_PDQS_DLY_OCTAD1",
                    wires::HSR_OUT_X5PHIO_PDQS_DLY_O.cell(1),
                ),
                (
                    "XCV2CGL_PDQS_DLY_OCTAD2",
                    wires::HSR_OUT_X5PHIO_PDQS_DLY_O.cell(2),
                ),
                (
                    "XCV2CGL_PDQS_DLY_OCTAD3",
                    wires::HSR_OUT_X5PHIO_PDQS_DLY_O.cell(3),
                ),
                (
                    "XCV2CGL_NDQS_DLY_OCTAD0",
                    wires::HSR_OUT_X5PHIO_NDQS_DLY_O.cell(0),
                ),
                (
                    "XCV2CGL_NDQS_DLY_OCTAD1",
                    wires::HSR_OUT_X5PHIO_NDQS_DLY_O.cell(1),
                ),
                (
                    "XCV2CGL_NDQS_DLY_OCTAD2",
                    wires::HSR_OUT_X5PHIO_NDQS_DLY_O.cell(2),
                ),
                (
                    "XCV2CGL_NDQS_DLY_OCTAD3",
                    wires::HSR_OUT_X5PHIO_NDQS_DLY_O.cell(3),
                ),
                (
                    "CLK2XCV_PDQS_DIST_OCTAD0",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN.cell(0),
                ),
                (
                    "CLK2XCV_PDQS_DIST_OCTAD1",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN.cell(1),
                ),
                (
                    "CLK2XCV_PDQS_DIST_OCTAD2",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN.cell(2),
                ),
                (
                    "CLK2XCV_PDQS_DIST_OCTAD3",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN.cell(3),
                ),
                (
                    "CLK2XCV_NDQS_DIST_OCTAD0",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN.cell(0),
                ),
                (
                    "CLK2XCV_NDQS_DIST_OCTAD1",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN.cell(1),
                ),
                (
                    "CLK2XCV_NDQS_DIST_OCTAD2",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN.cell(2),
                ),
                (
                    "CLK2XCV_NDQS_DIST_OCTAD3",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN.cell(3),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_EN_0_Z",
                    wires::HSR_OMUX_X5PHIO_NDQS_DLY.cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_EN_1_Z",
                    wires::HSR_OMUX_X5PHIO_PDQS_DLY.cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_EN_0_CLK_OUT",
                    wires::HSR_OMUX_X5PHIO_NDQS[0].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_EN_1_CLK_OUT",
                    wires::HSR_OMUX_X5PHIO_PDQS[0].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_EN_2_CLK_OUT",
                    wires::HSR_OMUX_X5PHIO_NDQS[1].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_EN_3_CLK_OUT",
                    wires::HSR_OMUX_X5PHIO_PDQS[1].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_1_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN_MUX.cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_4_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN_MUX.cell(1),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_7_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN_MUX.cell(2),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_10_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_NDQS_IN_MUX.cell(3),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_2_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN_MUX.cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_5_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN_MUX.cell(1),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_8_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN_MUX.cell(2),
                ),
                (
                    "X5PHIO_ROUTE_MUX4TO1_11_CLK_OUT",
                    wires::HSR_IMUX_X5PHIO_PDQS_IN_MUX.cell(3),
                ),
                (
                    "CLK2CGL_NDQS_REPLICA0",
                    wires::HSR_IMUX_X5PHIO_DQS_IN_R.cell(0),
                ),
                (
                    "CLK2CGL_NDQS_REPLICA1",
                    wires::HSR_IMUX_X5PHIO_DQS_IN_R.cell(1),
                ),
                (
                    "CLK2CGL_NDQS_REPLICA2",
                    wires::HSR_IMUX_X5PHIO_DQS_IN_R.cell(2),
                ),
                (
                    "CLK2CGL_NDQS_REPLICA3",
                    wires::HSR_IMUX_X5PHIO_DQS_IN_R.cell(3),
                ),
                (
                    "XCV2CGL_NDQS_DLY_REPLICA0",
                    wires::HSR_OUT_X5PHIO_DQS_DLY_OUT_R.cell(0),
                ),
                (
                    "XCV2CGL_NDQS_DLY_REPLICA1",
                    wires::HSR_OUT_X5PHIO_DQS_DLY_OUT_R.cell(1),
                ),
                (
                    "XCV2CGL_NDQS_DLY_REPLICA2",
                    wires::HSR_OUT_X5PHIO_DQS_DLY_OUT_R.cell(2),
                ),
                (
                    "XCV2CGL_NDQS_DLY_REPLICA3",
                    wires::HSR_OUT_X5PHIO_DQS_DLY_OUT_R.cell(3),
                ),
                (
                    "XCV2CGL_RDQS_P_REPLICA0",
                    wires::HSR_OUT_X5PHIO_RX1_RDQS_P_OUT.cell(0),
                ),
                (
                    "XCV2CGL_RDQS_P_REPLICA1",
                    wires::HSR_OUT_X5PHIO_RX1_RDQS_P_OUT.cell(1),
                ),
                (
                    "XCV2CGL_RDQS_P_REPLICA2",
                    wires::HSR_OUT_X5PHIO_RX1_RDQS_P_OUT.cell(2),
                ),
                (
                    "XCV2CGL_RDQS_P_REPLICA3",
                    wires::HSR_OUT_X5PHIO_RX1_RDQS_P_OUT.cell(3),
                ),
                (
                    "CLK2CLK_CASCADE_CLKOUT_BANK_LEFT",
                    wires::HSR_OMUX_X5PLL_CASC[0].cell(0),
                ),
                (
                    "CLK2CLK_CASCADE_CLKOUT_BANK_RIGHT",
                    wires::HSR_OMUX_X5PLL_CASC[1].cell(0),
                ),
                (
                    "CLK2CLK_CASCADE_CLKIN_BANK_LEFT0",
                    wires::HSR_X5PLL_CASC_E1[0].cell(5),
                ),
                (
                    "CLK2CLK_CASCADE_CLKIN_BANK_LEFT1",
                    wires::HSR_X5PLL_CASC_E1[1].cell(5),
                ),
                (
                    "CLK2CLK_CASCADE_CLKIN_BANK_RIGHT0",
                    wires::HSR_X5PLL_CASC_W1[0].cell(5),
                ),
                (
                    "CLK2CLK_CASCADE_CLKIN_BANK_RIGHT1",
                    wires::HSR_X5PLL_CASC_W1[1].cell(5),
                ),
                (
                    "CLK2CLK_HS_DESKEW_CLKFB_IN_PD0",
                    wires::HSR_X5PLL_HS_DESKEW_CLKFB.cell(0),
                ),
                (
                    "CLK2CLK_HS_DESKEW_CLKFB_IN_PD1",
                    wires::HSR_X5PLL_HS_DESKEW_CLKFB_NB.cell(0),
                ),
                (
                    "CLK2CLK_HS_DESKEW_CLKIN_PD0",
                    wires::HSR_X5PLL_HS_DESKEW_CLKIN.cell(0),
                ),
                (
                    "CLK2CLK_HS_DESKEW_CLKIN_PD1",
                    wires::HSR_X5PLL_HS_DESKEW_CLKIN_NB.cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
            let mut dummy = self.builder.add_dummies(tkn);
            dummy
                .insert(
                    wires::HSR_IMUX_DMC_CLK.cell(0),
                    false,
                    &[wires::HSR_IMUX_DMC_CLK_DUMMY.cell(0)],
                )
                .insert(
                    wires::HSR_IMUX_X5PLL_CLKIN.cell(0),
                    false,
                    &wires::HSR_IMUX_X5PLL_CLKIN_DUMMY.map(|w| w.cell(0)),
                )
                .insert(
                    wires::HSR_IMUX_X5PLL_CLKIN_DESKEW0.cell(0),
                    false,
                    &wires::HSR_IMUX_X5PLL_CLKIN_DESKEW0_DUMMY.map(|w| w.cell(0)),
                )
                .insert(
                    wires::HSR_IMUX_X5PLL_CLKIN_DESKEW1.cell(0),
                    false,
                    &wires::HSR_IMUX_X5PLL_CLKIN_DESKEW1_DUMMY.map(|w| w.cell(0)),
                )
                .insert(
                    wires::HSR_IMUX_X5PLL_CLKFB_DESKEW0.cell(0),
                    false,
                    &[wires::HSR_IMUX_X5PLL_CLKFB_DESKEW0_DUMMY.cell(0)],
                )
                .insert(
                    wires::HSR_IMUX_X5PLL_CLKFB_DESKEW1.cell(0),
                    false,
                    &[wires::HSR_IMUX_X5PLL_CLKFB_DESKEW1_DUMMY.cell(0)],
                );
            for i in 0..4 {
                dummy.insert(
                    wires::HSR_IMUX_X5PHIO_DQS_IN_R.cell(i),
                    false,
                    &wires::HSR_IMUX_X5PHIO_DQS_IN_R_DUMMY.map(|w| w.cell(i)),
                );
            }
        }
        for tkn in ["CMPHY_CMU_X32_CORE", "CMPHY_CMU_X32_CORE_MY"] {
            for (name, wire) in [
                ("PHY2CLK_GCLK", wires::HSR_X5IO_GCLK[0].cell(0)),
                ("PHY2CLK_GCLK_TEST1", wires::HSR_X5IO_GCLK[1].cell(0)),
                ("CMPHY_RXCLK0", wires::HSR_OUT_XPIO_PHY[0].cell(0)),
                ("CMPHY_RXCLK1", wires::HSR_OUT_XPIO_PHY[0].cell(1)),
                ("CMPHY_RXCLK2", wires::HSR_OUT_XPIO_PHY[0].cell(2)),
                ("CMPHY_RXCLK3", wires::HSR_OUT_XPIO_PHY[0].cell(3)),
                ("CMPHY_CPHY_RD_CLK0", wires::HSR_OUT_XPIO_PHY[1].cell(0)),
                ("CMPHY_CPHY_RD_CLK1", wires::HSR_OUT_XPIO_PHY[1].cell(1)),
                ("CMPHY_CPHY_RD_CLK2", wires::HSR_OUT_XPIO_PHY[1].cell(2)),
                ("CMPHY_CPHY_RD_CLK3", wires::HSR_OUT_XPIO_PHY[1].cell(3)),
                (
                    "CLK2CLK_RXCLKOUT_NEXT_HB",
                    wires::HSR_OMUX_X5IO_RXCLK.cell(0),
                ),
                (
                    "CLK2CLK_RXCLKIN_NEXT_HB",
                    wires::HSR_OMUX_X5IO_RXCLK_NB.cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_1_Z",
                    wires::HSR_OMUX_X5IO_PHY[0].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_2_Z",
                    wires::HSR_OMUX_X5IO_PHY[1].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_3_Z",
                    wires::HSR_OMUX_X5IO_PHY[2].cell(0),
                ),
                (
                    "X5PHIO_ROUTE_MUX2TO1_4_Z",
                    wires::HSR_OMUX_X5IO_PHY[3].cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
            for i in 0..24 {
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("CLB2PHY_GCLK{i}"),
                    wires::HSR_HDISTR[i].cell(4),
                );
            }
            for i in 0..4 {
                self.builder.extra_name_tile_tw(
                    tkn,
                    format!("PHY2CLB_CLKOUT{i}"),
                    wires::HSR_OMUX_X5PLL[i].cell(0),
                );
            }
            for &cmu_xy in self.rd.tiles_by_kind_name(tkn) {
                let pll_xy = self
                    .follow_wire(cmu_xy, "X5PLL_INTF_0_APB2PLL_RST")
                    .unwrap();
                let clk_xy = self.follow_wire(cmu_xy, "PHY2CLK_GCLK").unwrap();
                let dfx_xy = self
                    .follow_wire(cmu_xy, "X5PHIO_CMU_X32_INTF_1_MUX2APB_PCLK_CMU")
                    .unwrap();
                let bli1_xy = self.xlat_bli(cmu_xy, "X5PLL_INTF_0_CLB_PADDR_0_");
                let (tcid, naming, bli0_xy) = match self
                    .xlat_bli_full(pll_xy, "X5PLL_VPLL_TOP_1_RST")
                {
                    None => (tcls::X5PLL_NC, "X5PLL_NC", None),
                    Some((xy, "BLI_GRP3_A_BLI_CTRL1")) => (tcls::X5PLL_S, "X5PLL_S", Some(xy)),
                    Some((xy, "BLI_GRP4_D_BLI_IMUX_BYP11")) => (tcls::X5PLL_A, "X5PLL_A", Some(xy)),
                    Some((xy, "BLI_GRP4_D_BLI_IMUX_BYP8")) => (tcls::X5PLL_B, "X5PLL_B", Some(xy)),
                    _ => unreachable!(),
                };
                if tcid == tcls::X5PLL_NC {
                    self.builder.extra_name_tw(
                        "PHY2CLK_DESKEW_CLKFB0",
                        wires::HSR_IMUX_X5PLL_CLKFB_DESKEW_DUMMY_BLI[0].cell(0),
                    );
                    self.builder.extra_name_tw(
                        "PHY2CLK_DESKEW_CLKFB1",
                        wires::HSR_IMUX_X5PLL_CLKFB_DESKEW_DUMMY_BLI[1].cell(0),
                    );
                } else {
                    self.builder.extra_names.remove("PHY2CLK_DESKEW_CLKFB0");
                    self.builder.extra_names.remove("PHY2CLK_DESKEW_CLKFB1");
                }

                let mut bel = self
                    .builder
                    .bel_xy(bslots::X5PLL, "X5PLL", 0, 0)
                    .raw_tile(2);
                for pin in self.bel_pins(pll_xy, "X5PLL", 0, 0) {
                    if matches!(
                        pin,
                        "CLKIN"
                            | "CLKIN1_DESKEW"
                            | "CLKIN2_DESKEW"
                            | "CLKFB1_DESKEW"
                            | "CLKFB2_DESKEW"
                            | "CLKOUTPHY_0"
                            | "CLKOUTPHY_90"
                            | "CLKOUT0"
                            | "CLKOUT1"
                            | "CLKOUT2"
                            | "CLKOUT3"
                            | "TMUXOUT"
                    ) {
                        // pass
                    } else if matches!(
                        pin,
                        "CLKOUTPHYEN"
                            | "LOCKED"
                            | "LOCKED1_DESKEW"
                            | "LOCKED2_DESKEW"
                            | "LOCKED_FB"
                            | "PWRDWN"
                            | "RST"
                            | "TESTOUT"
                    ) || pin.starts_with("TESTIN")
                    {
                        if bli0_xy.is_none() {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    } else {
                        let npin = &format!("PLL_{pin}");
                        bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                    }
                }
                bel = bel.sub_xy("X5PLL_S2P", 0, 0).raw_tile(2);
                for pin in self.bel_pins(pll_xy, "X5PLL_S2P", 0, 0) {
                    let npin = &format!("S2P_{pin}");
                    bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                }
                bel = bel.sub_xy("X5PLL_INTF", 0, 0);
                for pin in self.bel_pins(cmu_xy, "X5PLL_INTF", 0, 0) {
                    if let Some(npin) = pin.strip_prefix("CLB_") {
                        bel = bel.pin_rename(pin, npin);
                        if bli0_xy.is_none() {
                            bel = bel.pin_name_only(npin, 1);
                        }
                    } else if let Some(npin) = pin.strip_prefix("FABRIC_")
                        && npin != "PSCLK"
                    {
                        bel = bel.pin_rename(pin, npin);
                        if bli0_xy.is_none() {
                            bel = bel.pin_name_only(npin, 1);
                        }
                    } else {
                        let npin = &format!("INTF_{pin}");
                        bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                    }
                }
                bel = bel.sub_virtual().raw_tile(3);
                if bli0_xy.is_none() {
                    bel = bel.extra_wire("PSCLK", &["CMPHY_DFX_HB_X32_0_ROOT_CLK_IN_71_"]);
                } else {
                    bel = bel.extra_int_in("PSCLK", &["CMPHY_DFX_HB_X32_0_ROOT_CLK_IN_71_"]);
                }

                let mut bels = vec![bel];

                for i in 0..8 {
                    bels.extend([
                        self.builder
                            .bel_xy(bslots::X5PHIO_DCCIBUF[i], "X5PHIO_DCCIBUF", 0, i)
                            .raw_tile(1)
                            .pin_name_only("X5PHIO_DCCIBUF_OVERRIDE_SEL", 1)
                            .pin_name_only("X5PHIO_DCCIBUF_OVERRIDE_CODE_0_", 1)
                            .pin_name_only("X5PHIO_DCCIBUF_OVERRIDE_CODE_1_", 1)
                            .pin_name_only("X5PHIO_DCCIBUF_OVERRIDE_CODE_2_", 1)
                            .pin_name_only("X5PHIO_DCCIBUF_OVERRIDE_CODE_3_", 1)
                            .pin_rename("X5PHIO_DCCIBUF_A", "A")
                            .pin_rename("X5PHIO_DCCIBUF_ZN", "ZN"),
                        self.builder
                            .bel_xy(bslots::X5PHIO_HARD_INV[i], "X5PHIO_HARD_INV", 0, i)
                            .raw_tile(1)
                            .pin_rename("X5PHIO_I", "I")
                            .pin_rename("X5PHIO_O", "O"),
                    ]);
                }

                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                let mut xt = self
                    .builder
                    .xtile_id(tcid, naming, cmu_xy)
                    .raw_tile(clk_xy)
                    .raw_tile(pll_xy)
                    .raw_tile(dfx_xy)
                    .switchbox(bslots::XPLL_INT)
                    .optin_muxes(&wires::HSR_OMUX_X5IO_PHY[..])
                    .optin_muxes(&wires::HSR_X5IO_GCLK[..])
                    .optin_muxes(&wires::HSR_X5IO_TESTCLK[..])
                    .optin_muxes(&wires::HSR_OMUX_X5PLL[..])
                    .optin_muxes(&wires::HSR_OMUX_X5PHIO_PDQS[..])
                    .optin_muxes(&wires::HSR_OMUX_X5PHIO_NDQS[..])
                    .optin_muxes(&wires::HSR_OMUX_X5PLL_CASC[..])
                    .optin_muxes(&[
                        wires::HSR_IMUX_DMC_CLK,
                        wires::HSR_IMUX_X5PLL_CLKIN,
                        wires::HSR_IMUX_X5PLL_CLKIN_DESKEW0,
                        wires::HSR_IMUX_X5PLL_CLKIN_DESKEW1,
                        wires::HSR_IMUX_X5PLL_CLKFB_DESKEW0,
                        wires::HSR_IMUX_X5PLL_CLKFB_DESKEW1,
                        wires::HSR_OMUX_X5IO_RXCLK,
                        wires::HSR_X5IO_RXCLK,
                        wires::HSR_IMUX_X5PHIO_PLL_CLK0,
                        wires::HSR_IMUX_X5PHIO_PLL_CLK90,
                        wires::HSR_OMUX_X5PHIO_PDQS_DLY,
                        wires::HSR_OMUX_X5PHIO_NDQS_DLY,
                        wires::HSR_IMUX_X5PHIO_DQS_IN_R,
                        wires::HSR_IMUX_X5PHIO_PDQS_IN,
                        wires::HSR_IMUX_X5PHIO_NDQS_IN,
                        wires::HSR_IMUX_X5PHIO_PDQS_IN_MUX,
                        wires::HSR_IMUX_X5PHIO_NDQS_IN_MUX,
                    ])
                    .bels(bels);
                if let Some((bli0_xy, bli1_xy)) = bli0_xy.zip(bli1_xy) {
                    xt = xt.ref_single(bli0_xy, 6, bli).ref_single(bli1_xy, 7, bli);
                }
                xt.extract();
            }
        }
    }

    pub fn fill_tiles_hsr_x5io_bank(&mut self) {
        for tkn in ["CMPHY_CMU_X32_CORE", "CMPHY_CMU_X32_CORE_MY"] {
            for &cmu_xy in self.rd.tiles_by_kind_name(tkn) {
                let io_xy = self
                    .follow_wire(cmu_xy, "X5PHIO_CMU_X32_INTF_1_APB2CMU_CK")
                    .unwrap();
                let bli_xy = self.xlat_bli(cmu_xy, "X5PHIO_CMU_X32_INTF_1_CLB2APB_CK");
                let tcid = if bli_xy.is_some() {
                    tcls::X5IO_BANK
                } else {
                    tcls::X5IO_BANK_NC
                };
                let mut bel = self
                    .builder
                    .bel_xy(bslots::X5PHIO_CMU, "X5PHIO_CMUIF", 0, 0);
                for pin in self.bel_pins(cmu_xy, "X5PHIO_CMUIF", 0, 0) {
                    if let Some(npin) = pin
                        .strip_prefix("CLB2APB_")
                        .or(pin.strip_prefix("APB2CLB_"))
                    {
                        bel = bel.pin_rename(pin, npin);
                        if bli_xy.is_none() {
                            bel = bel.pin_name_only(npin, 1);
                        }
                    } else {
                        bel = bel.pin_name_only(pin, 1);
                    }
                }
                bel = bel.sub_xy("X5PHIO_CMU_X32", 0, 0).raw_tile(1);
                for pin in self.bel_pins(io_xy, "X5PHIO_CMU_X32", 0, 0) {
                    let npin = &format!("IO_{pin}");
                    bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                }
                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                let mut xt = self
                    .builder
                    .xtile_id(tcid, "X5IO_BANK", cmu_xy)
                    .raw_tile(io_xy)
                    .bel(bel);
                if let Some(bli_xy) = bli_xy {
                    xt = xt.ref_single(bli_xy, 1, bli);
                }
                xt.extract();
            }
        }
    }

    pub fn fill_tiles_hsr_x5io_scan(&mut self) {
        for tkn in ["CMPHY_DFX_HB_X32_TILE", "CMPHY_DFX_HB_X32_TILE_MY"] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let (bli0, wn0) = self
                    .xlat_bli_full(xy, "CMPHY_DFX_HB_X32_0_SCAN_CLK_N_EXT")
                    .unwrap();
                let bli1 = self
                    .xlat_bli(xy, "CMPHY_DFX_HB_X32_0_SCAN_EN_N_EXT")
                    .unwrap();
                let tcid = match wn0 {
                    "BLI_GRP3_B_BLI_CTRL2" => tcls::X5IO_SCAN_A,
                    "BLI_GRP3_A_BLI_CTRL2" => tcls::X5IO_SCAN_B,
                    "BLI_GRP3_C_BLI_CTRL2" => tcls::X5IO_SCAN_C,
                    _ => unreachable!(),
                };
                let bli0 = if tcid == tcls::X5IO_SCAN_C {
                    None
                } else {
                    Some(bli0)
                };
                let mut bel = self.builder.bel_xy(bslots::X5IO_SCAN, "CMPHY_DFX_HB", 0, 0);
                for pin in self.bel_pins(xy, "CMPHY_DFX_HB", 0, 0) {
                    if let Some(npin) = pin.strip_prefix("CLB2PHY_") {
                        bel = bel.pin_rename(pin, npin);
                    } else if let Some((a, b)) = pin.split_once("_EXT") {
                        bel = bel.pin_rename(pin, format!("{a}{b}"));
                    } else {
                        let npin = &format!("_{pin}");
                        bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
                    }
                }
                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                let mut xt = self
                    .builder
                    .xtile_id(tcid, "X5IO_SCAN", xy)
                    .bel(bel)
                    .ref_single(bli1, 1, bli);
                if let Some(bli0) = bli0 {
                    xt = xt.ref_single(bli0, 0, bli);
                }
                xt.extract();
            }
        }
    }

    pub fn fill_tiles_hsr_ddrmc5(&mut self) {
        for (tkn, kind, tcid) in [
            ("DDRMC5_DMC_CORE", "DDRMC5", tcls::DDRMC5),
            ("DDRMC5_DMC_CORE_MY", "DDRMC5", tcls::DDRMC5),
            ("DDRMC5C_DMC_CORE", "DDRMC5C", tcls::DDRMC5C),
            ("DDRMC5C_DMC_CORE_MY", "DDRMC5C", tcls::DDRMC5C),
            ("DDRMC5E_DMC_CORE", "DDRMC5E", tcls::DDRMC5E),
            ("DDRMC5E_DMC_CORE_MY", "DDRMC5E", tcls::DDRMC5E),
            ("DDRMC5X_DMC_CORE", "DDRMC5X", tcls::DDRMC5X),
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let mut tcid = tcid;
                let mut naming = kind;
                if tcid == tcls::DDRMC5E
                    && self.rd.wires.contains("DDRMC5E_DMC_SITE_0_PLL2_CLK_PIN")
                {
                    tcid = tcls::DDRMC5E2;
                    naming = "DDRMC5E2";
                }
                if tcid == tcls::DDRMC5E2
                    && self
                        .xlat_bli(
                            xy,
                            &format!("{kind}_DMC_SITE_0_IF_UBLAZE_FABRIC_BLI2UB_CLK"),
                        )
                        .is_none()
                {
                    tcid = tcls::DDRMC5E2_CUT;
                }
                let bli0_xy = self
                    .xlat_bli(
                        xy,
                        &format!("{kind}_DMC_SITE_0_IF_DMC_FABRIC_BLI2UB_UART_RX"),
                    )
                    .unwrap();
                let mut bli1_xy = self
                    .xlat_bli(
                        xy,
                        &format!("{kind}_DMC_SITE_0_IF_UBLAZE_FABRIC_BLI2UB_CLK"),
                    )
                    .or(self.xlat_bli(xy, &format!("{kind}_DMC_SITE_0_IF_DMC_FABRIC_UB2BLI_TRIG")))
                    .unwrap();
                if bli0_xy == bli1_xy {
                    tcid = tcls::DDRMC5E_ALT;
                    bli1_xy = self
                        .xlat_bli(
                            xy,
                            &format!("{kind}_DMC_SITE_0_IF_UBLAZE_FABRIC_BLI2UB_READY"),
                        )
                        .unwrap();
                }
                for (pin, wire) in [
                    ("DBG_PLL1_CLK", wires::HSR_OMUX_X5PLL[1].cell(0)),
                    ("PLL2_CLK", wires::HSR_OMUX_X5PLL[2].cell(0)),
                    ("DMC_CLK", wires::HSR_IMUX_DMC_CLK.cell(1)),
                    ("TEST_CLK_OUT_0_", wires::HSR_OUT_DMC_TEST_CLK[0].cell(1)),
                    ("TEST_CLK_OUT_1_", wires::HSR_OUT_DMC_TEST_CLK[1].cell(1)),
                ] {
                    if pin == "PLL2_CLK" && !matches!(tcid, tcls::DDRMC5E2 | tcls::DDRMC5E2_CUT) {
                        continue;
                    }
                    self.builder
                        .extra_name_pin_xy(tkn, kind, 0, 0, pin, wire, true);
                }
                let is_cut = matches!(tcid, tcls::DDRMC5X | tcls::DDRMC5E2_CUT);
                let mut bel = self.builder.bel_xy(bslots::DDRMC5, kind, 0, 0);
                for pin in self.bel_pins(xy, kind, 0, 0) {
                    if pin.starts_with("TO_NOC")
                        || pin.starts_with("FROM_NOC")
                        || pin.starts_with("IF_DMC2X5PHYIO")
                        || pin.starts_with("DMC2PHY")
                        || pin == "DMC_CLK_DESKEW"
                        || is_cut
                            && pin.starts_with("IF_UBLAZE_FABRIC_UB2BLI")
                            && !matches!(
                                pin,
                                "IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA_27_"
                                    | "IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA_28_"
                                    | "IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA_29_"
                                    | "IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA_30_"
                                    | "IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA_31_"
                            )
                        || is_cut
                            && pin.starts_with("IF_UBLAZE_FABRIC_BLI2UB")
                            && !matches!(
                                pin,
                                "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_0_"
                                    | "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_1_"
                                    | "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_2_"
                                    | "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_3_"
                                    | "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_4_"
                                    | "IF_UBLAZE_FABRIC_BLI2UB_READ_DATA_5_"
                            )
                    {
                        bel = bel.pin_name_only(pin, 1);
                    }
                }
                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .bel(bel)
                    .ref_single(bli0_xy, 2, bli)
                    .ref_single(bli1_xy, 3, bli)
                    .extract();
            }
        }
    }

    pub fn fill_tiles_hnoc_x5io(&mut self) {
        let mut scans = Vec::from_iter(
            self.rd
                .tiles_by_kind_name("NOC2_SCAN_TOP")
                .iter()
                .chain(self.rd.tiles_by_kind_name("NOC2_SCAN_TOP_MY"))
                .copied()
                .filter(|&xy| xy.y < 9),
        );
        if scans.is_empty() {
            return;
        }
        scans.sort();
        let mut scans_row = [const { vec![] }; 3];
        for xy in scans {
            scans_row[(xy.y - 6) as usize].push(xy);
        }

        let npi_nir = self.rd.tiles_by_kind_name("NPI2_NIR_TILE")[0];
        let nps = self.rd.tiles_by_kind_name("NOC2_NPS5555_TOP");
        let xbr = self.rd.tiles_by_kind_name("NOC2_XBR4X2_TOP");
        let nmu = self.rd.tiles_by_kind_name("NOC2_NMU128_TOP");
        let nsu = self.rd.tiles_by_kind_name("NOC2_NSU128_TOP");
        let null = self.rd.tiles_by_kind_name("NULL")[0];

        let scans = if !self.rd.tiles_by_kind_name("PSS_BASE_CORE").is_empty() {
            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PS9_A,
                "HNOC_X5IO_PS9_A",
                &[
                    (npi_nir, bslots::HNOC_NPI_NIR, "NPI_NIR", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                    (nmu[3], bslots::HNOC_PS_NMU[3], "NOC2_NMU128", None),
                    (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                    (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (scans_row[1][0], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                ],
                &[],
            );

            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PS9_B,
                "HNOC_X5IO_PS9_B",
                &[
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (xbr[0], bslots::HNOC_XBR4X2, "NOC2_XBR4X2", None),
                    (scans_row[0][0], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                ],
                &[],
            );

            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PS9_C,
                "HNOC_X5IO_PS9_C",
                &[
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (nps[4], bslots::HNOC_NPS[4], "NOC2_NPS5555", None),
                    (nps[5], bslots::HNOC_NPS[5], "NOC2_NPS5555", None),
                    (nps[6], bslots::HNOC_NPS[6], "NOC2_NPS5555", None),
                    (nps[7], bslots::HNOC_NPS[7], "NOC2_NPS5555", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                    (nmu[3], bslots::HNOC_PS_NMU[3], "NOC2_NMU128", None),
                    (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                    (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                    (nsu[2], bslots::HNOC_PS_NSU[2], "NOC2_NSU128", None),
                    (nsu[3], bslots::HNOC_PS_NSU[3], "NOC2_NSU128", None),
                    (xbr[0], bslots::HNOC_XBR4X2, "NOC2_XBR4X2", None),
                    (
                        scans_row[2][0],
                        bslots::HNOC_SCAN[0],
                        "NOC2_SCAN",
                        Some((9, 0)),
                    ),
                    (
                        scans_row[2][1],
                        bslots::HNOC_SCAN[1],
                        "NOC2_SCAN",
                        Some((7, 0)),
                    ),
                    (
                        scans_row[2][2],
                        bslots::HNOC_SCAN[2],
                        "NOC2_SCAN",
                        Some((15, 0)),
                    ),
                ],
                &[
                    Some((
                        scans_row[2][2],
                        "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                    )),
                    Some((
                        scans_row[2][0],
                        "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                    )),
                ],
            );
            scans_row[1][1..].iter().copied()
        } else if self.rd.tiles_by_kind_name("PSXT_TILE").is_empty() {
            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PSX_A,
                "HNOC_X5IO_PSX_A",
                &[
                    (npi_nir, bslots::HNOC_NPI_NIR, "NPI_NIR", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                    (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                    (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (scans_row[1][0], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                ],
                &[],
            );

            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PSX_B,
                "HNOC_X5IO_PSX_B",
                &[
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                    (nmu[3], bslots::HNOC_PS_NMU[3], "NOC2_NMU128", None),
                    (xbr[0], bslots::HNOC_XBR4X2, "NOC2_XBR4X2", None),
                    (scans_row[0][1], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                ],
                &[],
            );

            if !self.rd.tiles_by_kind_name("PSXL_CORE").is_empty() {
                self.fill_tile_noc2(
                    tcls::HNOC_X5IO_PSXL_C,
                    "HNOC_X5IO_PSXL_C",
                    &[
                        (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                        (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                        (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                        (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                        (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                        (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                        (nmu[3], bslots::HNOC_PS_NMU[3], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                        (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                        (nsu[2], bslots::HNOC_PS_NSU[2], "NOC2_NSU128", None),
                        (nsu[3], bslots::HNOC_PS_NSU[3], "NOC2_NSU128", None),
                        (
                            scans_row[0][2],
                            bslots::HNOC_SCAN[0],
                            "NOC2_SCAN",
                            Some((7, 0)),
                        ),
                        (
                            scans_row[0][3],
                            bslots::HNOC_SCAN[1],
                            "NOC2_SCAN",
                            Some((10, 0)),
                        ),
                        (
                            scans_row[1][1],
                            bslots::HNOC_SCAN[2],
                            "NOC2_SCAN",
                            Some((10, 0)),
                        ),
                        (
                            scans_row[1][2],
                            bslots::HNOC_SCAN[3],
                            "NOC2_SCAN",
                            Some((11, 0)),
                        ),
                    ],
                    &[
                        Some((
                            scans_row[1][2],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                        Some((
                            scans_row[0][2],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                    ],
                );
            } else {
                self.fill_tile_noc2(
                    tcls::HNOC_X5IO_PSXC_C,
                    "HNOC_X5IO_PSXC_C",
                    &[
                        (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                        (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                        (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                        (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                        (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                        (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                        (nsu[2], bslots::HNOC_PS_NSU[2], "NOC2_NSU128", None),
                        (nsu[3], bslots::HNOC_PS_NSU[3], "NOC2_NSU128", None),
                        (
                            scans_row[0][2],
                            bslots::HNOC_SCAN[0],
                            "NOC2_SCAN",
                            Some((7, 0)),
                        ),
                        (
                            scans_row[0][3],
                            bslots::HNOC_SCAN[1],
                            "NOC2_SCAN",
                            Some((10, 0)),
                        ),
                        (
                            scans_row[1][1],
                            bslots::HNOC_SCAN[2],
                            "NOC2_SCAN",
                            Some((10, 0)),
                        ),
                        (
                            scans_row[1][2],
                            bslots::HNOC_SCAN[3],
                            "NOC2_SCAN",
                            Some((11, 0)),
                        ),
                    ],
                    &[
                        Some((
                            scans_row[1][2],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                        Some((
                            scans_row[0][2],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                    ],
                );
            }

            scans_row[1][3..].iter().copied()
        } else {
            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PSXT_A,
                "HNOC_X5IO_PSXT_A",
                &[
                    (npi_nir, bslots::HNOC_NPI_NIR, "NPI_NIR", None),
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nmu[1], bslots::HNOC_PS_NMU[1], "NOC2_NMU128", None),
                    (nmu[2], bslots::HNOC_PS_NMU[2], "NOC2_NMU128", None),
                    (nmu[3], bslots::HNOC_PS_NMU[3], "NOC2_NMU128", None),
                    (nmu[4], bslots::HNOC_PS_NMU[4], "NOC2_NMU128", None),
                    (nmu[5], bslots::HNOC_PS_NMU[5], "NOC2_NMU128", None),
                    (nmu[6], bslots::HNOC_PS_NMU[6], "NOC2_NMU128", None),
                    (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                    (scans_row[1][0], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                ],
                &[],
            );

            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PSXT_B,
                "HNOC_X5IO_PSXT_B",
                &[
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (nmu[0], bslots::HNOC_PS_NMU[0], "NOC2_NMU128", None),
                    (nsu[0], bslots::HNOC_PS_NSU[0], "NOC2_NSU128", None),
                    (nsu[1], bslots::HNOC_PS_NSU[1], "NOC2_NSU128", None),
                    (xbr[0], bslots::HNOC_XBR4X2, "NOC2_XBR4X2", None),
                    (
                        scans_row[2][0],
                        bslots::HNOC_SCAN[0],
                        "NOC2_SCAN",
                        Some((12, 12)),
                    ),
                ],
                &[
                    Some((
                        scans_row[2][0],
                        "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                    )),
                    Some((
                        scans_row[2][0],
                        "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_MASK_FROM_PL_0_",
                    )),
                ],
            );

            self.fill_tile_noc2(
                tcls::HNOC_X5IO_PSXT_C,
                "HNOC_X5IO_PSXT_C",
                &[
                    (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                    (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                    (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                    (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                    (
                        scans_row[2][1],
                        bslots::HNOC_SCAN[0],
                        "NOC2_SCAN",
                        Some((12, 12)),
                    ),
                ],
                &[
                    Some((
                        scans_row[2][1],
                        "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                    )),
                    Some((
                        scans_row[2][1],
                        "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_MASK_FROM_PL_0_",
                    )),
                ],
            );

            scans_row[1][1..].iter().copied()
        };

        for scan in scans {
            let (bli_xy, wn) = self
                .xlat_bli_full(scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
                .unwrap();
            if self
                .xlat_bli(scan, "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_FROM_PL_4_")
                .is_none()
            {
                self.fill_tile_noc2(
                    tcls::HNOC_X5IO_CAP,
                    "HNOC_X5IO_CAP",
                    &[
                        (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                        (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((4, 3))),
                    ],
                    &[
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                        None,
                    ],
                );
            } else if self
                .xlat_bli(scan, "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_FROM_PL_6_")
                .is_none()
            {
                let tcid = match wn {
                    "BLI_GRP2_A_BLI_IMUX0" => tcls::HNOC_X5IO_CROSS,
                    "BLI_GRP0_A_BLI_IMUX0" => tcls::HNOC_X5IO_CROSS_ALT,
                    _ => panic!("umm {wn}"),
                };

                self.fill_tile_noc2(
                    tcid,
                    "HNOC_X5IO_CROSS",
                    &[
                        (nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                        (nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                        (nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                        (nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((6, 0))),
                    ],
                    &[
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                        None,
                    ],
                );
            } else {
                let mut bel_scan = self
                    .builder
                    .bel_xy(bslots::HNOC_SCAN[0], "NOC2_SCAN", 0, 0)
                    .raw_tile(7);
                for i in 9..15 {
                    bel_scan = bel_scan
                        .pin_name_only(&format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"), 1)
                        .pin_name_only(&format!("NOC2_SCAN_CHNL_TO_PL_{i}_"), 1);
                }
                for i in 0..14 {
                    bel_scan =
                        bel_scan.pin_name_only(&format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"), 1);
                }
                let bels = [
                    self.bel_name_only(nps[0], bslots::HNOC_NPS[0], "NOC2_NPS5555", 0, 0),
                    self.bel_name_only(nps[1], bslots::HNOC_NPS[1], "NOC2_NPS5555", 0, 0)
                        .raw_tile(1),
                    self.bel_name_only(nps[2], bslots::HNOC_NPS[2], "NOC2_NPS5555", 0, 0)
                        .raw_tile(2),
                    self.bel_name_only(nps[3], bslots::HNOC_NPS[3], "NOC2_NPS5555", 0, 0)
                        .raw_tile(3),
                    self.bel_name_only(xbr[0], bslots::HNOC_XBR4X2, "NOC2_XBR4X2", 0, 0)
                        .raw_tile(6),
                    bel_scan,
                ];
                let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
                self.builder
                    .xtile_id(tcls::HNOC_X5IO_DDRMC, "HNOC_X5IO_DDRMC", nps[0])
                    .raw_tile(nps[1])
                    .raw_tile(nps[2])
                    .raw_tile(nps[3])
                    .raw_tile(null)
                    .raw_tile(null)
                    .raw_tile(xbr[0])
                    .raw_tile(scan)
                    .ref_single(bli_xy, 12, bli)
                    .bels(bels)
                    .extract();
            }
        }

        for xy in self
            .rd
            .tiles_by_kind_name("NOC2_NMU512_BLI_TILE")
            .iter()
            .chain(self.rd.tiles_by_kind_name("NOC2_NMU512_BLI_TILE_MY"))
            .copied()
            .filter(|xy| xy.y < 9)
        {
            let bli_xy = [
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_2_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WID_3_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_18_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_ARID_0_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_34_",
                "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_387_",
            ]
            .map(|w| self.xlat_bli(xy, w));
            if bli_xy[0].is_none() {
                continue;
            }
            let bli_xy = bli_xy.map(Option::unwrap);
            let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX_NOC");
            for i in 0..2 {
                let bel = self
                    .builder
                    .bel_xy(bslots::HNOC_NMU512[i], "NOC2_NMU512", 0, 0)
                    .raw_tile(4 + i)
                    .pin_name_only("FROM_NOC", 1)
                    .pin_name_only("TO_NOC", 1);
                self.builder
                    .xtile_id(tcls::HNOC_X5IO_DDRMC, "HNOC_X5IO_DDRMC", null)
                    .raw_tile(null)
                    .raw_tile(null)
                    .raw_tile(null)
                    .raw_tile([xy, null][i])
                    .raw_tile([null, xy][i])
                    .ref_single(bli_xy[0], i * 6, bli)
                    .ref_single(bli_xy[1], i * 6 + 1, bli)
                    .ref_single(bli_xy[2], i * 6 + 2, bli)
                    .ref_single(bli_xy[3], i * 6 + 3, bli)
                    .ref_single(bli_xy[4], i * 6 + 4, bli)
                    .ref_single(bli_xy[5], i * 6 + 5, bli)
                    .bel(bel)
                    .extract();
            }
            break;
        }
    }
}
