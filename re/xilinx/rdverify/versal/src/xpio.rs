use prjcombine_entity::EntityId;
use prjcombine_interconnect::{coord::BelCoord, dir::Dir};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    defs::{bslots, tcls},
    expanded::ExpandedDevice,
};

pub fn verify_xpll(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("CLKOUTPHY")
        .extra_out_far_claim("CLKOUTPHY_CASC_OUT")
        .extra_in_far("CLKOUTPHY_CASC_IN")
        .extra_in_far_claim("RIU_CLK")
        .extra_in_far_claim("RIU_WR_EN")
        .extra_in_far_claim("RIU_NIBBLE_SEL")
        .extra_out_far_claim("RIU_VALID");
    for i in 0..16 {
        bel = bel
            .extra_in_far_claim(format!("RIU_WR_DATA_{i}_"))
            .extra_out_far_claim(format!("RIU_RD_DATA_{i}_"));
    }
    for i in 0..8 {
        bel = bel.extra_in_far_claim(format!("RIU_ADDR_{i}_"));
    }
    if bcrd.slot == bslots::XPLL[0]
        && let Some(bcrd_ddrmc) = edev
            .bel_delta(bcrd.cell, -9, 0, bslots::DDRMC)
            .or(edev.bel_delta(bcrd.cell, 0, 0, bslots::DDRMC))
            .or(edev.bel_delta(bcrd.cell, 9, 0, bslots::DDRMC))
    {
        let obel_src = bcrd_ddrmc.bel(bslots::XPLL[0]);
        bel.verify_net(&[
            bel.wire_far("CLKOUTPHY_CASC_IN"),
            bel.bel_wire_far(obel_src, "CLKOUTPHY_CASC_OUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire_far("CLKOUTPHY_CASC_IN")]);
    }
    bel.commit();
}

pub fn verify_xpiob(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::XPIOB.index_of(bcrd.slot).unwrap();
    if !idx.is_multiple_of(2) {
        return;
    }
    let mut bel = vrf.verify_bel(bcrd).skip_auto();

    if idx == 0 {
        bel = bel.extra_out_far("RXOUT_M");
        bel.verify_net(&[bel.wire_far("RXOUT_M"), bel.wire("CLKPAD")]);
    } else {
        bel = bel.extra_out_far_claim("RXOUT_M");
    }

    let obel = bcrd.bel(bslots::XPHY);
    bel = bel
        .extra_in_far("OP_M")
        .extra_in_far("OP_S")
        .extra_in_far("TRISTATE_M")
        .extra_in_far("TRISTATE_S")
        .extra_out_far_claim("LPRX_O_P")
        .extra_out_far_claim("LPRX_O_N")
        .extra_out_far_claim("RXOUT_S")
        .extra_out_far_claim("RXOUT_LSPD_M")
        .extra_out_far_claim("RXOUT_LSPD_S")
        .extra_in_far("DYNAMIC_DCI_TS_M")
        .extra_in_far("DYNAMIC_DCI_TS_S")
        .extra_in_far("IBUF_DISABLE_M")
        .extra_in_far("IBUF_DISABLE_S")
        .extra_in_far("PD_EN_M")
        .extra_in_far("PD_EN_S")
        .extra_in_far("PU_EN_M")
        .extra_in_far("PU_EN_S")
        .extra_in_far("KEEPER_EN_M")
        .extra_in_far("KEEPER_EN_S")
        .extra_in_claim("_ORIG_TERM_EN");

    let i0 = idx;
    let i1 = idx + 1;
    let ih = idx / 2;

    bel.claim_pip(bel.wire("_ORIG_TERM_EN"), bel.wire_far("_ORIG_TERM_EN"));

    for (pin, opin) in [
        ("OP_M", format!("O0_{i0}_")),
        ("OP_S", format!("O0_{i1}_")),
        ("TRISTATE_M", format!("T_OUT_{i0}_")),
        ("TRISTATE_S", format!("T_OUT_{i1}_")),
        ("DYNAMIC_DCI_TS_M", format!("DYN_DCI_{i0}_")),
        ("DYNAMIC_DCI_TS_S", format!("DYN_DCI_{i1}_")),
        ("IBUF_DISABLE_M", format!("IBUF_DISABLE_{i0}_")),
        ("IBUF_DISABLE_S", format!("IBUF_DISABLE_{i1}_")),
        ("PD_EN_M", format!("PHY2IOB_LP_TX_T_OR_PD_EN_0_{ih}_")),
        ("PD_EN_S", format!("PHY2IOB_PD_EN_1_{ih}_")),
        ("PU_EN_M", format!("PHY2IOB_LP_TX_O_OR_PU_EN_0_{ih}_")),
        ("PU_EN_S", format!("PHY2IOB_LP_TX_O_OR_PU_EN_1_{ih}_")),
        ("KEEPER_EN_M", format!("PHY2IOB_KEEPER_EN_{i0}_")),
        ("KEEPER_EN_S", format!("PHY2IOB_KEEPER_EN_{i1}_")),
        (
            "_ORIG_TERM_EN",
            format!("PHY2IOB_LP_RX_DIS_OR_TERM_EN_{ih}_"),
        ),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, &opin)]);
    }

    let obel = bcrd.bel(bslots::XPIO_VREF);
    bel = bel.extra_in_far("VREF");
    bel.verify_net(&[bel.wire_far("VREF"), bel.bel_wire_far(obel, "VREF_XP")]);

    bel.commit();
}

pub fn verify_xpiologic(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::XPIO_NC;
    let idx = bslots::XPIOLOGIC.index_of(bcrd.slot).unwrap();
    if !idx.is_multiple_of(2) {
        return;
    }

    let iob = bcrd.bel(bslots::XPIOB[idx]);
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far("LGC_SCAN_IN")
        .extra_in_far("LGC_SCAN_CLK")
        .extra_in_far("LGC_SCAN_EN")
        .extra_in_far("LGC_SCAN_MODE")
        .extra_in_far("LGC_SCAN_RST")
        .extra_in_far("LGC_SCAN_SET")
        .extra_in_far("IPFFM_LSPD")
        .extra_in_far("IPFFS_LSPD")
        .extra_in_far("IPFFM_D")
        .extra_in_far("IPFFS_D")
        .extra_in_far("PHY_IPFFM_BYPASS")
        .extra_in_far("PHY_IPFFS_BYPASS")
        .extra_in_far("PHY_IPFFM_EVEN")
        .extra_in_far("PHY_IPFFS_EVEN")
        .extra_in_far("PHY_IPFFM_ODD")
        .extra_in_far("PHY_IPFFS_ODD")
        .extra_out_far_claim("OPFFM_Q")
        .extra_out_far_claim("OPFFS_Q")
        .extra_out_far_claim("TFFM_Q")
        .extra_out_far_claim("TFFS_Q")
        .extra_out_far_claim("XPHYM_IDELAY_EN_VTC")
        .extra_out_far_claim("XPHYS_IDELAY_EN_VTC")
        .extra_out_far_claim("XPHYM_ODELAY_EN_VTC")
        .extra_out_far_claim("XPHYS_ODELAY_EN_VTC")
        .extra_out_far_claim("XPHYM_IODELAY_CE")
        .extra_out_far_claim("XPHYS_IODELAY_CE")
        .extra_out_far_claim("XPHYM_IODELAY_INC")
        .extra_out_far_claim("XPHYS_IODELAY_INC")
        .extra_out_far_claim("XPHYM_IODELAY_LD")
        .extra_out_far_claim("XPHYS_IODELAY_LD")
        .extra_out_far_claim("XPHYM_IODELAY_SEL")
        .extra_out_far_claim("XPHYS_IODELAY_SEL")
        .extra_out_far_claim("XPHYM_RXBIT_RST")
        .extra_out_far_claim("XPHYS_RXBIT_RST")
        .extra_out_far_claim("XPHYM_TXBIT_RST")
        .extra_out_far_claim("XPHYS_TXBIT_RST");

    for (pin, opin) in [
        ("IPFFM_LSPD", "RXOUT_LSPD_M"),
        ("IPFFS_LSPD", "RXOUT_LSPD_S"),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(iob, opin)]);
    }

    let xphy = bcrd.bel(bslots::XPHY);

    let i0 = idx;
    let i1 = idx + 1;

    for i in 0..5 {
        bel = bel
            .extra_in_far(format!("PHY_IDLYM_CNTVALUEOUT_{i}_"))
            .extra_in_far(format!("PHY_IDLYS_CNTVALUEOUT_{i}_"))
            .extra_in_far(format!("PHY_ODLYM_CNTVALUEOUT_{i}_"))
            .extra_in_far(format!("PHY_ODLYS_CNTVALUEOUT_{i}_"))
            .extra_out_far_claim(format!("XPHYM_IDELAY_CNTVALUEIN_{i}_"))
            .extra_out_far_claim(format!("XPHYS_IDELAY_CNTVALUEIN_{i}_"))
            .extra_out_far_claim(format!("XPHYM_ODELAY_CNTVALUEIN_{i}_"))
            .extra_out_far_claim(format!("XPHYS_ODELAY_CNTVALUEIN_{i}_"))
            .extra_out_far_claim(format!("XPHY_WR_DQ0_{i}_"))
            .extra_out_far_claim(format!("XPHY_WR_DQ1_{i}_"));
        bel.verify_net(&[
            bel.wire_far(&format!("PHY_IDLYM_CNTVALUEOUT_{i}_")),
            bel.bel_wire_far(xphy, &format!("CNTVALUEOUT_{ii}_", ii = i0 * 9 + i)),
        ]);
        bel.verify_net(&[
            bel.wire_far(&format!("PHY_IDLYS_CNTVALUEOUT_{i}_")),
            bel.bel_wire_far(xphy, &format!("CNTVALUEOUT_{ii}_", ii = i1 * 9 + i)),
        ]);
        if i < 4 {
            bel.verify_net(&[
                bel.wire_far(&format!("PHY_ODLYM_CNTVALUEOUT_{i}_")),
                bel.bel_wire_far(xphy, &format!("CNTVALUEOUT_{ii}_", ii = i0 * 9 + i + 5)),
            ]);
            bel.verify_net(&[
                bel.wire_far(&format!("PHY_ODLYS_CNTVALUEOUT_{i}_")),
                bel.bel_wire_far(xphy, &format!("CNTVALUEOUT_{ii}_", ii = i1 * 9 + i + 5)),
            ]);
        }
    }

    for (pin, opin) in [
        ("PHY_ODLYM_CNTVALUEOUT_4_", format!("Q{i0}_0_")),
        ("PHY_ODLYS_CNTVALUEOUT_4_", format!("Q{i1}_0_")),
        ("IPFFM_D", format!("Q{i0}_4_")),
        ("IPFFS_D", format!("Q{i1}_4_")),
        ("PHY_IPFFM_BYPASS", format!("Q{i0}_5_")),
        ("PHY_IPFFS_BYPASS", format!("Q{i1}_5_")),
        ("PHY_IPFFM_EVEN", format!("Q{i0}_6_")),
        ("PHY_IPFFS_EVEN", format!("Q{i1}_6_")),
        ("PHY_IPFFM_ODD", format!("Q{i0}_7_")),
        ("PHY_IPFFS_ODD", format!("Q{i1}_7_")),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(xphy, &opin)]);
    }

    if idx == 4 {
        bel = bel.extra_out_far("LGC_SCANOUT");
    } else {
        bel = bel.extra_out_far_claim("LGC_SCANOUT");
    }
    if idx != 0 {
        let obel = bcrd.bel(bslots::XPIOLOGIC[idx - 2]);
        bel.verify_net(&[
            bel.wire_far("LGC_SCAN_IN"),
            bel.bel_wire_far(obel, "LGC_SCANOUT"),
        ]);
    }
    for (pin, opin) in [
        ("LGC_SCAN_CLK", "CLB2PHY_SCAN_CLK_B"),
        ("LGC_SCAN_EN", "CLB2PHY_SCAN_EN_B"),
        ("LGC_SCAN_MODE", "CLB2PHY_CSSD_TRIG"),
        ("LGC_SCAN_RST", "CLB2PHY_SCAN_MODE_RST_B"),
        ("LGC_SCAN_SET", "CLB2PHY_SCAN_EDT_UPDT_B"),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(xphy, opin)]);
    }

    for pin in [
        "IPFFS_RST",
        "IPFFS_CE",
        "OPFFS_RST",
        "OPFFS_CE",
        "OPFFS_D1",
        "OPFFS_D2",
        "TFFS_D1",
        "TFFS_D2",
        "IDLYS_RST",
        "IDLYS_CE",
        "IDLYS_INC",
        "IDLYS_LOAD",
        "ODLYS_RST",
        "ODLYS_CE",
        "ODLYS_INC",
        "ODLYS_LOAD",
    ] {
        if is_nc {
            bel = bel.extra_in_far_claim(pin);
        } else {
            bel = bel.extra_in(pin);
        }
    }
    for pin in ["IPFFS_Q1", "IPFFS_Q2", "IPFFS_BYPASS"] {
        if is_nc {
            bel = bel.extra_out_far_claim(pin);
        } else {
            bel = bel.extra_out(pin);
        }
    }
    for i in 0..5 {
        if is_nc {
            bel = bel
                .extra_in_far_claim(format!("IDLYM_CNTVALUEIN_{i}_"))
                .extra_in_far_claim(format!("ODLYM_CNTVALUEIN_{i}_"))
                .extra_out_far_claim(format!("IDLYM_CNTVALUEOUT_{i}_"))
                .extra_out_far_claim(format!("ODLYM_CNTVALUEOUT_{i}_"))
                .extra_in_far_claim(format!("IDLYS_CNTVALUEIN_{i}_"))
                .extra_in_far_claim(format!("ODLYS_CNTVALUEIN_{i}_"))
                .extra_out_far_claim(format!("IDLYS_CNTVALUEOUT_{i}_"))
                .extra_out_far_claim(format!("ODLYS_CNTVALUEOUT_{i}_"));
        } else {
            bel = bel
                .extra_in(format!("IDLYS_CNTVALUEIN_{i}_"))
                .extra_in(format!("ODLYS_CNTVALUEIN_{i}_"))
                .extra_out(format!("IDLYS_CNTVALUEOUT_{i}_"))
                .extra_out(format!("ODLYS_CNTVALUEOUT_{i}_"));
        }
    }
    if is_nc {
        for pin in [
            "IPFFM_RST",
            "IPFFM_CE",
            "OPFFM_RST",
            "OPFFM_CE",
            "OPFFM_D1",
            "OPFFM_D2",
            "TFFM_D1",
            "TFFM_D2",
            "IDLYM_RST",
            "IDLYM_CE",
            "IDLYM_INC",
            "IDLYM_LOAD",
            "ODLYM_RST",
            "ODLYM_CE",
            "ODLYM_INC",
            "ODLYM_LOAD",
        ] {
            bel = bel.extra_in_far_claim(pin);
        }
        bel = bel
            .extra_out_far_claim("IPFFM_Q1")
            .extra_out_far_claim("IPFFM_Q2")
            .extra_out_far_claim("IPFFM_BYPASS");
    }

    bel.commit();
}

pub fn verify_xphy(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let bidx = bcrd.col.to_idx() % 9;
    let bank_cell = bcrd.step(Dir::W, bidx);
    let is_nc = edev[tcrd].class == tcls::XPIO_NC;
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("PHY2CFG_SCAN_CLK")
        .extra_out_far_claim("PHY2CFG_SCAN_EN")
        .extra_out_far_claim("PHY2CFG_SCAN_MODE")
        .extra_out_far_claim("PHY2CFG_SCAN_MODE_RST_B")
        .extra_out_far_claim("PHY2CFG_SCAN_RST_BYP")
        .extra_in_far("CFG2PHY_SCAN_CLEAR_CLK")
        .extra_in_far("DMC_FIFO_RD_CLK")
        .extra_in_far_claim("DMC_FIFO_RDEN")
        .extra_out_far_claim("DMC_FIFO_EMPTY")
        .extra_out_far_claim("DMC_GT_STATUS")
        .extra_in_far_claim("DMC_CTRL_CLK")
        .extra_in_far_claim("DMC_RIU_WR_EN")
        .extra_in_far_claim("DMC_RIU_NIBBLE_SEL")
        .extra_out_far_claim("DMC_RIU_RD_VALID")
        .extra_out_far_claim("CLK_TO_UPPER")
        .extra_out_far_claim("CLK_TO_LOWER")
        .extra_in_far("CLK_FROM_OTHER_XPHY")
        .extra_out_far_claim("PCLK_NIBBLE_OUT")
        .extra_out_far_claim("NCLK_NIBBLE_OUT");
    for i in 0..16 {
        bel = bel
            .extra_in_far_claim(format!("DMC_RIU_WR_DATA_{i}_"))
            .extra_out_far_claim(format!("DMC_RIU_RD_DATA_{i}_"));
    }
    for i in 0..8 {
        bel = bel.extra_in_far_claim(format!("DMC_RIU_ADDR_{i}_"));
    }
    for i in 0..2 {
        bel = bel
            .extra_in_far_claim(format!("DMC_PHY_RDEN_{i}_"))
            .extra_in_far_claim(format!("DMC_PHY_RDCS0_{i}_"))
            .extra_in_far_claim(format!("DMC_PHY_RDCS1_{i}_"))
            .extra_in_far_claim(format!("DMC_PHY_WREN_{i}_"))
            .extra_in_far_claim(format!("DMC_PHY_WRCS0_{i}_"))
            .extra_in_far_claim(format!("DMC_PHY_WRCS1_{i}_"));
    }
    for i in 0..6 {
        bel = bel.extra_in_far_claim(format!("DMC_T_{i}_"));
        for j in 0..4 {
            bel = bel
                .extra_in_far_claim(format!("DMC_D{i}_{j}_"))
                .extra_out_far_claim(format!("DMC_Q{i}_{j}_"));
        }
    }

    for i in 0..4 {
        bel = bel
            .extra_out_far_claim(format!("PHY2CFG_SCAN_DATA_IN_INT_{i}_"))
            .extra_in_far(format!("CFG2PHY_SCAN_DATA_OUT_INT_{i}_"));
    }
    for i in 0..10 {
        bel = bel
            .extra_out_far_claim(format!("PHY2IOB_SCAN_DATA_IN_INT_{i}_"))
            .extra_in_far(format!("IOB2PHY_SCAN_DATA_OUT_INT_{i}_"));
    }

    for i in 0..3 {
        let iob = bcrd.bel(bslots::XPIOB[i * 2]);
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        for (pin, opin) in [
            (format!("DATAIN_{i0}_"), "RXOUT_M"),
            (format!("DATAIN_{i1}_"), "RXOUT_S"),
            (format!("IOB2PHY_LP_I_0_{i}_"), "LPRX_O_P"),
            (format!("IOB2PHY_LP_I_1_{i}_"), "LPRX_O_N"),
        ] {
            bel = bel.extra_in_far(&pin);
            bel.verify_net(&[bel.wire_far(&pin), bel.bel_wire_far(iob, opin)]);
        }
        bel = bel
            .extra_out_far_claim(format!("PHY2IOB_LP_TX_T_OR_PD_EN_0_{i}_"))
            .extra_out_far_claim(format!("PHY2IOB_PD_EN_1_{i}_"))
            .extra_out_far_claim(format!("PHY2IOB_LP_TX_O_OR_PU_EN_0_{i}_"))
            .extra_out_far_claim(format!("PHY2IOB_LP_TX_O_OR_PU_EN_1_{i}_"))
            .extra_out_far_claim(format!("PHY2IOB_LP_RX_DIS_OR_TERM_EN_{i}_"));
        for pin in [
            format!("CLB2PHY_LP_TX_T_OR_PD_EN_0_{i}_"),
            format!("CLB2PHY_PD_EN_1_{i}_"),
            format!("CLB2PHY_LP_TX_O_OR_PU_EN_0_{i}_"),
            format!("CLB2PHY_LP_TX_O_OR_PU_EN_1_{i}_"),
            format!("CLB2PHY_LP_RX_DIS_OR_TERM_EN_{i}_"),
        ] {
            if is_nc {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_in(pin);
            }
        }
        if is_nc {
            bel = bel
                .extra_out_far_claim(format!("PHY2CLB_LP_I_0_{i}_"))
                .extra_out_far_claim(format!("PHY2CLB_LP_I_1_{i}_"));
        } else {
            bel = bel
                .extra_out(format!("PHY2CLB_LP_I_0_{i}_"))
                .extra_out(format!("PHY2CLB_LP_I_1_{i}_"));
        }
    }
    for i in 0..6 {
        bel = bel
            .extra_out_far_claim(format!("O0_{i}_"))
            .extra_out_far_claim(format!("T_OUT_{i}_"))
            .extra_out_far_claim(format!("DYN_DCI_{i}_"))
            .extra_out_far_claim(format!("IBUF_DISABLE_{i}_"))
            .extra_out_far_claim(format!("PHY2IOB_KEEPER_EN_{i}_"));
        if is_nc {
            bel = bel
                .extra_in_far_claim(format!("DYN_DCI_IN_{i}_"))
                .extra_in_far_claim(format!("IBUF_DISABLE_IN_{i}_"))
                .extra_in_far_claim(format!("CLB2PHY_KEEPER_EN_{i}_"))
                .extra_out_far_claim(format!("Q{i}_1_"))
                .extra_out_far_claim(format!("Q{i}_2_"))
                .extra_out_far_claim(format!("Q{i}_3_"))
                .extra_out_far_claim(format!("Q{i}_4_"))
                .extra_in_far_claim(format!("D{i}_7_"));
        } else {
            bel = bel
                .extra_in(format!("DYN_DCI_IN_{i}_"))
                .extra_in(format!("IBUF_DISABLE_IN_{i}_"))
                .extra_in(format!("CLB2PHY_KEEPER_EN_{i}_"))
                .extra_out(format!("Q{i}_1_"))
                .extra_out(format!("Q{i}_2_"))
                .extra_out(format!("Q{i}_3_"))
                .extra_out(format!("Q{i}_4_"))
                .extra_in(format!("D{i}_7_"));
        }
    }

    for i in 0..6 {
        let obel = bcrd.bel(bslots::XPIOLOGIC[i / 2 * 2]);
        let ms = ['M', 'S'][i % 2];
        for j in 0..9 {
            let ii = i * 9 + j;
            let pin = &format!("CNTVALUEIN_{ii}_");
            bel = bel
                .extra_in_far(pin)
                .extra_out_far_claim(format!("CNTVALUEOUT_{ii}_"));
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(
                    obel,
                    &if j < 5 {
                        format!("XPHY{ms}_IDELAY_CNTVALUEIN_{j}_")
                    } else {
                        format!("XPHY{ms}_ODELAY_CNTVALUEIN_{jj}_", jj = j - 5)
                    },
                ),
            ]);
        }
        for (pin, opin) in [
            (format!("CE_{i}_"), format!("XPHY{ms}_IODELAY_CE")),
            (format!("INC_{i}_"), format!("XPHY{ms}_IODELAY_INC")),
            (format!("LD_{i}_"), format!("XPHY{ms}_IODELAY_LD")),
            (format!("RXTX_SEL_{i}_"), format!("XPHY{ms}_IODELAY_SEL")),
            (format!("RX_EN_VTC_{i}_"), format!("XPHY{ms}_IDELAY_EN_VTC")),
            (format!("TX_EN_VTC_{i}_"), format!("XPHY{ms}_ODELAY_EN_VTC")),
            (format!("RX_RST_{i}_"), format!("XPHY{ms}_RXBIT_RST")),
            (format!("TX_RST_{i}_"), format!("XPHY{ms}_TXBIT_RST")),
            (format!("D{i}_0_"), format!("OPFF{ms}_Q")),
            (format!("D{i}_1_"), format!("XPHY_WR_DQ{ii}_0_", ii = i % 2)),
            (format!("D{i}_2_"), format!("XPHY_WR_DQ{ii}_1_", ii = i % 2)),
            (format!("D{i}_3_"), format!("XPHY_WR_DQ{ii}_2_", ii = i % 2)),
            (format!("D{i}_4_"), format!("XPHY{ms}_ODELAY_CNTVALUEIN_4_")),
            (format!("D{i}_5_"), format!("XPHY_WR_DQ{ii}_3_", ii = i % 2)),
            (format!("D{i}_6_"), format!("XPHY_WR_DQ{ii}_4_", ii = i % 2)),
            (format!("T_{i}_"), format!("TFF{ms}_Q")),
        ] {
            bel = bel.extra_in_far(&pin);
            bel.verify_net(&[bel.wire_far(&pin), bel.bel_wire_far(obel, &opin)]);
        }
        bel = bel
            .extra_out_far_claim(format!("Q{i}_0_"))
            .extra_out_far_claim(format!("Q{i}_5_"))
            .extra_out_far_claim(format!("Q{i}_6_"))
            .extra_out_far_claim(format!("Q{i}_7_"));
    }

    bel = bel.extra_in_far("PLL0_CLK").extra_in_far("PLL1_CLK");
    bel.verify_net(&[
        bel.wire_far("PLL0_CLK"),
        bel.bel_wire_far(bank_cell.bel(bslots::XPLL[0]), "CLKOUTPHY"),
    ]);
    bel.verify_net(&[
        bel.wire_far("PLL1_CLK"),
        bel.bel_wire_far(bank_cell.bel(bslots::XPLL[1]), "CLKOUTPHY"),
    ]);

    let ddrmc = edev
        .bel_delta(bank_cell, -9, 0, bslots::DDRMC)
        .or(edev.bel_delta(bank_cell, 0, 0, bslots::DDRMC))
        .or(edev.bel_delta(bank_cell, 9, 0, bslots::DDRMC));
    let cell_src = if let Some(ddrmc) = ddrmc {
        ddrmc.cell
    } else {
        bank_cell
    };
    let obel = cell_src.bel(bslots::XPLL[0]);
    bel.verify_net(&[
        bel.wire_far("DMC_FIFO_RD_CLK"),
        bel.bel_wire_far(obel, "CLKOUT2"),
    ]);

    if is_nc {
        bel = bel
            .extra_in_far_claim("CTRL_CLK")
            .extra_in_far_claim("RST")
            .extra_in_far_claim("FIFO_RD_CLK")
            .extra_in_far_claim("FIFO_RDEN")
            .extra_out_far_claim("FIFO_EMPTY")
            .extra_in_far_claim("EN_VTC")
            .extra_out_far_claim("DLY_RDY")
            .extra_out_far_claim("PHY_RDY")
            .extra_out_far_claim("MST_PD")
            .extra_out_far_claim("GT_STATUS")
            .extra_in_far_claim("RIU_WR_EN")
            .extra_in_far_claim("RIU_NIBBLE_SEL")
            .extra_out_far_claim("RIU_RD_VALID");
        for i in 0..4 {
            bel = bel
                .extra_in_far_claim(format!("PHY_RDCS0_{i}_"))
                .extra_in_far_claim(format!("PHY_RDCS1_{i}_"))
                .extra_in_far_claim(format!("PHY_RDEN_{i}_"))
                .extra_in_far_claim(format!("PHY_WRCS0_{i}_"))
                .extra_in_far_claim(format!("PHY_WRCS1_{i}_"))
                .extra_in_far_claim(format!("PHY_WREN_{i}_"));
        }
        for i in 0..8 {
            bel = bel.extra_in_far_claim(format!("RIU_ADDR_{i}_"));
        }
        for i in 0..16 {
            bel = bel
                .extra_in_far_claim(format!("RIU_WR_DATA_{i}_"))
                .extra_out_far_claim(format!("RIU_RD_DATA_{i}_"));
        }
    }

    if bidx == 4 {
        bel = bel
            .extra_in_far_claim("NCLK_NIBBLE_IN")
            .extra_in_far_claim("PCLK_NIBBLE_IN");
    } else {
        bel = bel
            .extra_in_far("NCLK_NIBBLE_IN")
            .extra_in_far("PCLK_NIBBLE_IN");
        let obidx = [1, 0, 3, 2, 4, 6, 5, 8, 7][bidx];
        let obel = bank_cell.delta(obidx, 0).bel(bslots::XPHY);
        bel.verify_net(&[
            bel.wire_far("NCLK_NIBBLE_IN"),
            bel.bel_wire_far(obel, "NCLK_NIBBLE_OUT"),
        ]);
        bel.verify_net(&[
            bel.wire_far("PCLK_NIBBLE_IN"),
            bel.bel_wire_far(obel, "PCLK_NIBBLE_OUT"),
        ]);
    }
    let obidx = [2, 3, 5, 6, 7, 2, 3, 5, 6][bidx];
    let obel = bank_cell.delta(obidx, 0).bel(bslots::XPHY);
    bel.verify_net(&[
        bel.wire_far("CLK_FROM_OTHER_XPHY"),
        bel.bel_wire_far(
            obel,
            if bidx < obidx {
                "CLK_TO_LOWER"
            } else {
                "CLK_TO_UPPER"
            },
        ),
    ]);

    let obel = bank_cell.bel(bslots::XPIO_SCAN);
    for (pin, opin) in [
        ("CLB2PHY_SCAN_CLK_B", "SCAN_CLK_B"),
        ("CLB2PHY_SCAN_MODE_RST_B", "SCAN_MODE_RST_B"),
        ("CLB2PHY_SCAN_EN_B", "SCAN_EN_B"),
        ("CLB2PHY_SCAN_EDT_UPDT_B", "SCAN_EDT_UPDT_B"),
        ("CLB2PHY_CSSD_TRIG", &format!("CSSD_TRIG_XPHY_{bidx}_")),
        ("CLB2PHY_SCAN_IN", &format!("SCAN_IN_XPHY_{bidx}_")),
        (
            "CLB2PHY_SCAN_CNTRL_CHNL_IN",
            &format!("SCAN_CNTRL_CHNL_IN_XPHY_{bidx}_"),
        ),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
    }
    for (pin, opin) in [
        ("PHY2CLB_SCAN_OUT", &format!("SCAN_OUT_XPHY_{bidx}_")),
        (
            "PHY2CLB_SCAN_CNTRL_CHNL_OUT",
            &format!("SCAN_CNTRL_CHNL_OUT_XPHY_{bidx}_"),
        ),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
    }

    bel.commit();
}

pub fn verify_xpio_vref(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).extra_out_far_claim("VREF_XP");

    let obel = bcrd.bel(bslots::XPHY);
    for i in 0..10 {
        let pin = &format!("FABRIC_VREF_TUNE_{i}_");
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[
            bel.wire_far(pin),
            bel.bel_wire_far(
                obel,
                [
                    "PHY2IOB_KEEPER_EN_0_",
                    "PHY2IOB_KEEPER_EN_1_",
                    "PHY2IOB_KEEPER_EN_2_",
                    "PHY2IOB_KEEPER_EN_3_",
                    "PHY2IOB_KEEPER_EN_4_",
                    "PHY2IOB_KEEPER_EN_5_",
                    "PHY2IOB_PD_EN_1_0_",
                    "PHY2IOB_PD_EN_1_1_",
                    "PHY2IOB_PD_EN_1_2_",
                    "PHY2IOB_LP_TX_T_OR_PD_EN_0_0_",
                ][i],
            ),
        ]);
    }

    bel.commit();
}

pub fn verify_xpio_dci(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    let obel = bcrd.bel(bslots::XPIO_SCAN);
    for (pin, opin) in [
        ("CLB2MISC_SCAN_CLK_B", "SCAN_CLK_B"),
        ("CLB2MISC_SCAN_MODE_RST_B", "SCAN_MODE_RST_B"),
        ("CLB2MISC_SCAN_EN_B", "SCAN_EN_B"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
    }
    bel.commit();
}

pub fn verify_ddrmc(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("TO_NOC_0")
        .extra_out_far_claim("TO_NOC_1")
        .extra_out_far_claim("TO_NOC_2")
        .extra_out_far_claim("TO_NOC_3")
        .extra_in_far("FROM_NOC_0")
        .extra_in_far("FROM_NOC_1")
        .extra_in_far("FROM_NOC_2")
        .extra_in_far("FROM_NOC_3")
        .extra_out_far_claim("DMC2PHY_CSSD_TRIG");

    for r in 0..3 {
        for i in 0..9 {
            let dx = [0, 1, 2, 3, 5, 6, 7, 8, 4][i];
            let obel = bcrd.delta(-9 + r as i32 * 9 + dx, 0).bel(bslots::XPHY);
            let ii = r * 9 + i;
            let rden = &format!("DMC2PHY_FIFO_RDEN_{ii}_");
            let empty = &format!("PHY2DMC_FIFO_EMPTY_{ii}_");
            let gtstat = &format!("PHY2DMC_GT_STATUS_{ii}_");
            bel = bel
                .extra_out_far(rden)
                .extra_in_far(empty)
                .extra_in_far(gtstat);
            for (pin, opin) in [
                (rden, "DMC_FIFO_RDEN"),
                (empty, "DMC_FIFO_EMPTY"),
                (gtstat, "DMC_GT_STATUS"),
            ] {
                bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
            }
            for j in 0..2 {
                let iii = ii * 2 + j;
                let rden = &format!("DMC2PHY_RDEN_{iii}_");
                let rdcs0 = &format!("DMC2PHY_RDCS0_{iii}_");
                let rdcs1 = &format!("DMC2PHY_RDCS1_{iii}_");
                let wren = &format!("DMC2PHY_T_B_{iii}_");
                let wrcs0 = &format!("DMC2PHY_WRCS0_{iii}_");
                let wrcs1 = &format!("DMC2PHY_WRCS1_{iii}_");
                bel = bel
                    .extra_out_far(rden)
                    .extra_out_far(rdcs0)
                    .extra_out_far(rdcs1)
                    .extra_out_far(wren)
                    .extra_out_far(wrcs0)
                    .extra_out_far(wrcs1);
                for (pin, opin) in [
                    (rden, format!("DMC_PHY_RDEN_{j}_")),
                    (rdcs0, format!("DMC_PHY_RDCS0_{j}_")),
                    (rdcs1, format!("DMC_PHY_RDCS1_{j}_")),
                    (wren, format!("DMC_PHY_WREN_{j}_")),
                    (wrcs0, format!("DMC_PHY_WRCS0_{j}_")),
                    (wrcs1, format!("DMC_PHY_WRCS1_{j}_")),
                ] {
                    bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, &opin)]);
                }
            }
            for j in 0..6 {
                let t = &format!("DMC2PHY_T_TXBIT_{r}_{iii}_", iii = i * 6 + j);
                bel = bel.extra_out_far(t);
                bel.verify_net(&[
                    bel.wire_far(t),
                    bel.bel_wire_far(obel, &format!("DMC_T_{j}_")),
                ]);
                for k in 0..4 {
                    let iii = ii * 4 + k;
                    let d = &format!("DMC2PHY_WR_DQ{j}_{iii}_");
                    let q = &format!("PHY2DMC_RD_DQ{j}_{iii}_");
                    bel = bel.extra_out_far(d).extra_in_far(q);
                    bel.verify_net(&[
                        bel.wire_far(d),
                        bel.bel_wire_far(obel, &format!("DMC_D{j}_{k}_")),
                    ]);
                    bel.verify_net(&[
                        bel.wire_far(q),
                        bel.bel_wire_far(obel, &format!("DMC_Q{j}_{k}_")),
                    ]);
                }
            }
        }
    }

    bel.commit();
    crate::noc::verify_noc_conns(edev, vrf, bcrd);

    let mut bel = vrf.verify_bel(bcrd).sub(1).kind("DDRMC_RIU").skip_auto();
    for r in 0..3 {
        for i in 0..10 {
            let obel = if i == 9 {
                bcrd.delta(-9 + r * 9, 0).bel(bslots::XPLL[0])
            } else {
                let dx = [0, 1, 2, 3, 5, 6, 7, 8, 4][i];
                bcrd.delta(-9 + r * 9 + dx, 0).bel(bslots::XPHY)
            };
            let clk = &format!("IF_DMC2PHY{r}_RIU_DMC2PHY_CTRL_CLK_{i}_");
            let sel = &format!("IF_DMC2PHY{r}_RIU_DMC2RIU_NIBBLE_SEL_{i}_");
            let wren = &format!("IF_DMC2PHY{r}_RIU_DMC2RIU_WR_EN_{i}_");
            let valid = &format!("IF_DMC2PHY{r}_RIU_RIU2DMC_VALID_{i}_");
            bel = bel
                .extra_out_far(clk)
                .extra_out_far(sel)
                .extra_out_far(wren)
                .extra_in_far(valid);
            for (pin, opin, opin_pll) in [
                (clk, "DMC_CTRL_CLK", "RIU_CLK"),
                (sel, "DMC_RIU_NIBBLE_SEL", "RIU_NIBBLE_SEL"),
                (wren, "DMC_RIU_WR_EN", "RIU_WR_EN"),
                (valid, "DMC_RIU_RD_VALID", "RIU_VALID"),
            ] {
                let opin = if i == 9 { opin_pll } else { opin };
                bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
            }
            for j in 0..8 {
                let pin = &format!("IF_DMC2PHY{r}_RIU_DMC2RIU_ADDR_{ii}_", ii = i * 8 + j);
                bel = bel.extra_out_far(pin);
                bel.verify_net(&[
                    bel.wire_far(pin),
                    bel.bel_wire_far(
                        obel,
                        &if i == 9 {
                            format!("RIU_ADDR_{j}_")
                        } else {
                            format!("DMC_RIU_ADDR_{j}_")
                        },
                    ),
                ]);
            }
            for j in 0..16 {
                let wd = &format!("IF_DMC2PHY{r}_RIU_DMC2RIU_WR_DATA_{ii}_", ii = i * 16 + j);
                let rd = &format!("IF_DMC2PHY{r}_RIU_RIU2DMC_RD_DATA_{ii}_", ii = i * 16 + j);
                bel = bel.extra_out_far(wd).extra_in_far(rd);
                bel.verify_net(&[
                    bel.wire_far(rd),
                    bel.bel_wire_far(
                        obel,
                        &if i == 9 {
                            format!("RIU_RD_DATA_{j}_")
                        } else {
                            format!("DMC_RIU_RD_DATA_{j}_")
                        },
                    ),
                ]);
                bel.verify_net(&[
                    bel.wire_far(wd),
                    bel.bel_wire_far(
                        obel,
                        &if i == 9 {
                            format!("RIU_WR_DATA_{j}_")
                        } else {
                            format!("DMC_RIU_WR_DATA_{j}_")
                        },
                    ),
                ]);
            }
        }
        bel = bel.extra_out_far_claim(format!("IF_DMC2PHY{r}_RIU_DMC2RIU_NIBBLE_SEL_10_"));
    }
    bel.commit();
}
