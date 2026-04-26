use prjcombine_interconnect::{
    db::PinDir,
    dir::DirV,
    grid::{BelCoord, DieIdExt},
};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    chip::{Chip, HsrNKind, HsrSKind},
    defs::{bcls, bslots, tcls},
    expanded::ExpandedDevice,
};

pub fn verify_gt(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let (kind, kind_refclk) = match bcrd.slot {
        bslots::GTY => ("GTY_QUAD", "GTY_REFCLK"),
        bslots::GTYP => ("GTYP_QUAD", "GTYP_REFCLK"),
        bslots::GTM => ("GTM_QUAD", "GTM_REFCLK"),
        bslots::GTM2 => ("GTM2_QUAD", "GTM2_REFCLK"),
        _ => unreachable!(),
    };
    let reg = edev.row_to_reg(bcrd.row);
    let chip = edev.chips[bcrd.die];
    let is_n = edev.is_reg_n(bcrd.die, reg);
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let has_xpipe = edev.has_bel(bcrd.bel(bslots::XPIPE));
    let has_hpipe = edev.has_bel(bcrd.bel(bslots::HNICPIPE));
    let has_pipe = has_xpipe || has_hpipe;
    let is_ps = matches!(
        edev[edev.bel_tile(bcrd)].class,
        tcls::GTYP_PS | tcls::GTM2_PS
    );

    let mut bel = vrf.verify_bel(bcrd).kind(kind);
    if is_ps {
        for (pn, pin) in &naming.pins {
            if pn.starts_with("PIPE")
                || pn.starts_with("RXPI")
                || pn.starts_with("TXPI")
                || pn.starts_with("RESETDONE")
                || (pn.starts_with("REFCLK") && pn.ends_with("INT"))
                || pn.contains("RXRECCLK")
                || pn.ends_with("REFCLK0")
                || pn.ends_with("REFCLK1")
                || (pn.ends_with("_ODIV2") && bcrd.slot != bslots::GTM2)
                || pn.starts_with("REFCLKOUT")
                || pn.starts_with("_REFCLK")
            {
                continue;
            }
            let &(_, dir) = pin.orig_pin.as_ref().unwrap();
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pn);
            } else {
                bel = bel.extra_out_far_claim(pn);
            }
        }
    } else if has_pipe {
        bel = bel.skip_auto();
        for i in 0..4 {
            bel = bel
                .extra_out(format!("CH{i}_RXOUTCLK"))
                .extra_out(format!("CH{i}_TXOUTCLK"))
                .extra_out(format!("CH{i}_DMONITOROUTCLK"));
        }
        for (pn, pin) in &naming.pins {
            if pn.starts_with("_GT_") {
                let &(_, dir) = pin.orig_pin.as_ref().unwrap();
                if dir == PinDir::Input {
                    bel = bel.extra_in_claim(pn);
                    bel.claim_pip(bel.wire(pn), bel.wire_far(pn));
                } else {
                    bel = bel.extra_out_claim(pn);
                    bel.claim_pip(bel.wire_far(pn), bel.wire(pn));
                }
            }
        }
    } else {
        match bcrd.slot {
            bslots::GTY => {
                bel = bel
                    .skip_out(bcls::GTY::REFCLK0_ODIV2)
                    .skip_out(bcls::GTY::REFCLK1_ODIV2);
            }
            bslots::GTYP => {
                bel = bel
                    .skip_out(bcls::GTYP::REFCLK0_ODIV2)
                    .skip_out(bcls::GTYP::REFCLK1_ODIV2);
            }
            bslots::GTM => {
                bel = bel
                    .skip_out(bcls::GTM::REFCLK0_ODIV2)
                    .skip_out(bcls::GTM::REFCLK1_ODIV2);
            }
            _ => (),
        }
    }

    let obel_s = edev.bel_gt_neighbour(bcrd, DirV::S);
    let obel_n = edev.bel_gt_neighbour(bcrd, DirV::N);
    if bcrd.slot != bslots::GTM2 {
        for (bus, num) in [("PIPE", 6), ("RXPI", 4), ("TXPI", 4), ("RESETDONE_", 2)] {
            if bcrd.slot == bslots::GTM && bus != "RESETDONE_" {
                continue;
            }
            for (dir, obel) in [("SOUTH", obel_n), ("NORTH", obel_s)] {
                for i in 0..num {
                    let opin = &format!("{bus}{dir}OUT_{i}_");
                    let ipin = &format!("{bus}{dir}IN_{i}_");
                    bel = bel.extra_out_far_claim(opin);
                    if let Some(obel) = obel
                        && obel.die == bcrd.die
                    {
                        bel = bel.extra_in_far(ipin);
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                    } else {
                        bel = bel.extra_in_far_claim(ipin);
                    }
                }
            }
        }
    }
    for pll in ["HSCLK0_LCPLL", "HSCLK1_LCPLL", "HSCLK0_RPLL", "HSCLK1_RPLL"] {
        if bcrd.slot == bslots::GTM && pll.contains("RPLL") {
            continue;
        }
        for (dir, obel) in [("SOUTH", obel_n), ("NORTH", obel_s)] {
            for i in 0..2 {
                let opin = &format!("REFCLKOUT{dir}{i}");
                let ipin = &format!("{pll}{dir}REFCLK{i}");
                let ipin0 = &format!("HSCLK0_LCPLL{dir}REFCLK{i}");
                bel = bel.extra_in_far(ipin);
                if let Some(obel) = obel {
                    bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire(obel, opin)]);
                } else {
                    if dir == "NORTH"
                        && reg == chip.regs().first().unwrap()
                        && chip.hsr_s == HsrSKind::Ssit
                        && bcrd.col == chip.col_e()
                    {
                        // nothing
                    } else if dir == "SOUTH"
                        && reg == chip.regs().last().unwrap()
                        && chip.hsr_n == HsrNKind::Ssit
                        && bcrd.col == chip.col_e()
                    {
                        // also nothing
                    } else {
                        bel.verify_net(&[bel.wire_far(ipin), bel.wire_far(ipin0)]);
                        if ipin == ipin0 {
                            bel.claim_net(&[bel.wire_far(ipin)]);
                        }
                    }
                }
            }
        }
    }
    for i in 0..2 {
        for dir in ["SOUTH", "NORTH"] {
            let opin = &format!("REFCLKOUT{dir}{i}");
            let ipin = &format!("HSCLK0_LCPLL{dir}REFCLK{i}");
            let dpin = &format!("REFCLKOUT{dir}{i}_DUMMY");
            bel.claim_net(&[bel.wire(opin)]);
            bel.claim_net(&[bel.wire(dpin)]);
            bel.claim_pip(bel.wire(opin), bel.wire_far("_REFCLK0_REFCLK_GTREFCLK"));
            bel.claim_pip(bel.wire(opin), bel.wire_far("_REFCLK1_REFCLK_GTREFCLK"));
            bel.claim_pip(bel.wire(opin), bel.wire_far(ipin));
            bel.claim_pip(bel.wire(opin), bel.wire(dpin));
        }
    }

    for i in 0..2 {
        for j in 0..2 {
            let pin_r = &format!("HSCLK{i}_RPLLGTREFCLK{j}");
            let pin_lc = &format!("HSCLK{i}_LCPLLGTREFCLK{j}");
            let opin = &format!("_REFCLK{j}_REFCLK_GTREFCLK");
            bel = bel
                .extra_out_far_claim(format!("HSCLK{i}_RXRECCLKOUT{j}"))
                .extra_out_far_claim(format!("HSCLK{i}_RXRECCLKSEL_{j}_"))
                .extra_in_far(pin_lc);
            bel.verify_net(&[bel.wire_far(pin_lc), bel.wire_far(opin)]);
            if bcrd.slot != bslots::GTM {
                bel = bel.extra_in_far(pin_r);
                bel.verify_net(&[bel.wire_far(pin_r), bel.wire_far(opin)]);
            }
        }
        if bcrd.slot == bslots::GTM2 {
            let pin = &format!("REFCLK{i}_ODIV2INT");
            let opin = &format!("_REFCLK{i}_REFCLK_ODIV2");
            bel = bel.extra_in_far(pin);
            bel.verify_net(&[bel.wire_far(pin), bel.wire_far(opin)]);
        } else {
            bel = bel.extra_out_far_claim(format!("REFCLK{i}_GTREFCLKPDINT"));
        }
        bel = bel.extra_out_far_claim(format!("REFCLK{i}_CLKTESTSIGINT"));
    }

    bel.commit();

    for i in 0..2 {
        let mut bel = vrf
            .verify_bel(bcrd)
            .sub(1 + i)
            .kind(kind_refclk)
            .skip_auto();
        for pin in ["REFCLK_GTREFCLK", "REFCLK_ODIV2"] {
            if bcrd.slot != bslots::GTM2 && pin == "REFCLK_ODIV2" {
                let rpin = &format!("REFCLK{i}_ODIV2");
                if is_ps {
                    bel = bel.extra_out_claim(rpin);
                    bel.claim_net(&[bel.wire_far(rpin)]);
                    bel.claim_pip(bel.wire_far(rpin), bel.wire(rpin));
                } else {
                    bel = bel.extra_out(rpin);
                }
            } else {
                let rpin = &format!("_REFCLK{i}_{pin}");
                bel = bel.extra_out_claim(rpin);
                bel.claim_net(&[bel.wire_far(rpin)]);
                bel.claim_pip(bel.wire_far(rpin), bel.wire(rpin));
            }
        }
        for (pin, opin) in [
            ("REFCLK_CEB", format!("REFCLK{i}_GTREFCLKPDINT")),
            ("REFCLK_CLKTESTSIG", format!("REFCLK{i}_CLKTESTSIGINT")),
            ("REFCLK_RXRECCLK0", "HSCLK0_RXRECCLKOUT0".to_string()),
            ("REFCLK_RXRECCLK1", "HSCLK0_RXRECCLKOUT1".to_string()),
            ("REFCLK_RXRECCLK2", "HSCLK1_RXRECCLKOUT0".to_string()),
            ("REFCLK_RXRECCLK3", "HSCLK1_RXRECCLKOUT1".to_string()),
            ("RXRECCLKSEL_0_", format!("HSCLK{i}_RXRECCLKSEL_0_")),
            ("RXRECCLKSEL_1_", format!("HSCLK{i}_RXRECCLKSEL_1_")),
        ] {
            if bcrd.slot == bslots::GTM2 && pin == "REFCLK_CEB" {
                continue;
            }
            let rpin = &format!("_REFCLK{i}_{pin}");
            bel = bel.extra_in_claim(rpin);
            bel.claim_pip(bel.wire(rpin), bel.wire_far(&opin));
        }
        bel.commit();
    }

    if has_pipe {
        let mut bel = vrf.verify_bel(bcrd).sub(3);
        if has_xpipe {
            bel = bel
                .kind("XPIPE_QUAD")
                .skip_out(bcls::GTY::REFCLK0_ODIV2)
                .skip_out(bcls::GTY::REFCLK1_ODIV2)
                .skip_out(bcls::GTY::CH0_TXOUTCLK)
                .skip_out(bcls::GTY::CH1_TXOUTCLK)
                .skip_out(bcls::GTY::CH2_TXOUTCLK)
                .skip_out(bcls::GTY::CH3_TXOUTCLK)
                .skip_out(bcls::GTY::CH0_RXOUTCLK)
                .skip_out(bcls::GTY::CH1_RXOUTCLK)
                .skip_out(bcls::GTY::CH2_RXOUTCLK)
                .skip_out(bcls::GTY::CH3_RXOUTCLK)
                .skip_out(bcls::GTY::CH0_DMONITOROUTCLK)
                .skip_out(bcls::GTY::CH1_DMONITOROUTCLK)
                .skip_out(bcls::GTY::CH2_DMONITOROUTCLK)
                .skip_out(bcls::GTY::CH3_DMONITOROUTCLK)
                .extra_in_far("CH0_RXOUTCLK_M")
                .extra_in_far("CH0_TXOUTCLK_M");
            if !is_n && obel_n.is_some() {
                bel.claim_net(&[bel.wire_far("CH0_RXOUTCLK_M")]);
                bel.claim_net(&[bel.wire_far("CH0_TXOUTCLK_M")]);
            } else {
                bel.verify_net(&[bel.wire_far("CH0_RXOUTCLK_M"), bel.wire_far("CH0_RXOUTCLK")]);
                bel.verify_net(&[bel.wire_far("CH0_TXOUTCLK_M"), bel.wire_far("CH0_TXOUTCLK")]);
            }
            if !is_n {
                bel = bel
                    .extra_out_far_claim("TXOUTCLKDBG_SOUTH")
                    .extra_out("TXOUTCLKDBG_NORTH");
            } else {
                bel = bel
                    .extra_out_far_claim("TXOUTCLKDBG_NORTH")
                    .extra_out("TXOUTCLKDBG_SOUTH");
            }
            for (pn, pin) in &naming.pins {
                if !pn.starts_with("XPIPE_") {
                    continue;
                }
                let (pre, suf) = if let Some(opn) = pn.strip_suffix("_M") {
                    (opn, "".to_string())
                } else if let Some((a, b)) = pn.split_once("_M_") {
                    (a, format!("_{b}"))
                } else {
                    continue;
                };
                let opn = &format!("{pre}{suf}");
                let &(_, dir) = pin.orig_pin.as_ref().unwrap();
                if dir == PinDir::Input {
                    bel = bel.extra_in_far_claim(pn).extra_out_far(opn);
                } else {
                    bel = bel.extra_out_far_claim(pn).extra_in_far(opn);
                }
                if let Some(obel) = obel_s {
                    bel.verify_net(&[bel.wire_far(opn), bel.bel_wire_far(obel, pn)]);
                } else {
                    let obel = bcrd
                        .with_col(edev.col_cfrm[bcrd.die])
                        .delta(0, -48)
                        .bel(bslots::CPM4);
                    let (sidx, suf) = if suf == "_1" {
                        (1, "")
                    } else if suf == "_2" {
                        (2, "")
                    } else if suf == "_3" {
                        (3, "")
                    } else if suf.starts_with("_1_") && suf != "_1_" {
                        (1, suf.strip_prefix("_1").unwrap())
                    } else if suf.starts_with("_2_") && suf != "_2_" {
                        (2, suf.strip_prefix("_2").unwrap())
                    } else if suf.starts_with("_3_") && suf != "_3_" {
                        (3, suf.strip_prefix("_3").unwrap())
                    } else {
                        (0, suf.as_str())
                    };
                    let cpn = if pre == "XPIPE_BUFGTCE_MASK" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEBUFGTCEMASK{suf}")
                    } else if pre == "XPIPE_BUFGTRST_MASK" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEBUFGTRSTMASK{suf}")
                    } else if let Some(pre) = pre.strip_prefix("XPIPE_BUFGT") {
                        format!("IFCPMXPIPELINK{sidx}XPIPEBUFGT{pre}{suf}")
                    } else if pre == "XPIPE_GT_OUTCLK" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEGTOUTCLK")
                    } else if pre == "XPIPE_GT_PIPECLK" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEGTPIPECLK")
                    } else if pre == "XPIPE_GT_RXOUTCLK" {
                        "IFCPMXPIPEHSDPLINKXPIPEGTRXOUTCLK".to_string()
                    } else if pre == "XPIPE_GT_RXUSRCLK" {
                        "IFCPMXPIPEHSDPLINKXPIPEGTRXUSRCLK".to_string()
                    } else if let Some(pre) = pre.strip_prefix("XPIPE_HSDP_") {
                        format!("IFCPMXPIPEHSDPCHANNEL{sidx}XPIPE{pre}{suf}")
                    } else if pre == "XPIPE_PCIE_PERST_N" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEPCIEPERSTN")
                    } else if pre == "XPIPE_PCIE_RATE" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEPIPERATE{suf}")
                    } else if pre == "XPIPE_PHYESMADAPTSAVE" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEPHYESMADAPTATIONSAVE{suf}")
                    } else if pre == "XPIPE_PHY_READY" {
                        format!("IFCPMXPIPELINK{sidx}XPIPEPHYREADY{suf}")
                    } else if pre == "XPIPE_PHYREADY_FR_BOT" {
                        "IFCPMXPIPEINTQUADXPIPEPHYREADYFRBOT".to_string()
                    } else if pre == "XPIPE_PHYREADY_TO_BOT" {
                        "IFCPMXPIPEINTQUADXPIPEPHYREADYTOBOT".to_string()
                    } else if let Some(pre) = pre.strip_prefix("XPIPE_PCIE") {
                        format!("IFCPMXPIPELINK{sidx}XPIPEPCIE{pre}{suf}")
                    } else if let Some(pre) = pre.strip_prefix("XPIPE_RXMARGIN") {
                        format!("IFCPMXPIPEQUAD{sidx}XPIPERXMARGIN{pre}{suf}")
                    } else if let Some(pre) = pre.strip_prefix("XPIPE_PIPE_CH") {
                        let (idx, rest) = pre.split_once('_').unwrap();
                        format!("IFCPMXPIPECHANNEL{idx}XPIPE{rest}{suf}")
                    } else {
                        panic!("oops {pn}")
                    };
                    bel.verify_net(&[bel.wire_far(opn), bel.bel_wire_far(obel, &cpn)]);
                }
            }
        } else {
            bel = bel
                .kind("HNICPIPE_QUAD")
                .skip_out(bcls::GTM::REFCLK0_ODIV2)
                .skip_out(bcls::GTM::REFCLK1_ODIV2)
                .skip_out(bcls::GTM::CH0_TXOUTCLK)
                .skip_out(bcls::GTM::CH1_TXOUTCLK)
                .skip_out(bcls::GTM::CH2_TXOUTCLK)
                .skip_out(bcls::GTM::CH3_TXOUTCLK)
                .skip_out(bcls::GTM::CH0_RXOUTCLK)
                .skip_out(bcls::GTM::CH1_RXOUTCLK)
                .skip_out(bcls::GTM::CH2_RXOUTCLK)
                .skip_out(bcls::GTM::CH3_RXOUTCLK)
                .skip_out(bcls::GTM::CH0_DMONITOROUTCLK)
                .skip_out(bcls::GTM::CH1_DMONITOROUTCLK)
                .skip_out(bcls::GTM::CH2_DMONITOROUTCLK)
                .skip_out(bcls::GTM::CH3_DMONITOROUTCLK);

            for i in 0..4 {
                for j in 0..16 {
                    let ipin = &format!("HNICPIPE_ANLT_HNICX_STATUS_{i}_M_{j}_");
                    let opin = &format!("HNICPIPE_ANLT_HNICX_STATUS_{i}_{j}_");
                    bel = bel.extra_in_far_claim(ipin).extra_out_far(opin);
                    if let Some(obel) = obel_n {
                        bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, ipin)]);
                    } else {
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        let hpin = &format!("ANLT_FEC_STATUS_P{i}_{j}_");
                        bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, hpin)]);
                    }
                }

                for (name, hpin) in [
                    ("FEC_ALIGNED", format!("RX_PCS_BS_STATS_C{i}_FEC_ALIGNED")),
                    ("MAC_PHY_STATUS_OK", format!("RX_PCS_BS_STATS_C{i}_STATUS")),
                    (
                        "RSFEC_CORRECTED_CW_INC",
                        format!("STAT_RSFEC_RX{i}_CORRECTED_CW_INC"),
                    ),
                    (
                        "RSFEC_CORRECTED_CWB_INC",
                        format!("STAT_RSFEC_RX{i}_CORRECTED_CWB_INC"),
                    ),
                    (
                        "RSFEC_UNCORRECTED_CW_INC",
                        format!("STAT_RSFEC_RX{i}_UNCORRECTED_CW_INC"),
                    ),
                    (
                        "RSFEC_UNCORRECTED_CWB_INC",
                        format!("STAT_RSFEC_RX{i}_UNCORRECTED_CWB_INC"),
                    ),
                    ("RSFEC_CW_INC", format!("STAT_RSFEC_RX{i}_CW_INC")),
                    ("RSFEC_CWB_INC", format!("STAT_RSFEC_RX{i}_CWB_INC")),
                ] {
                    let opin = &format!("HNICPIPE_{name}_{i}_M");
                    let ipin = &format!("HNICPIPE_{name}_{i}");
                    bel = bel.extra_out_far_claim(opin).extra_in_far(ipin);
                    if let Some(obel) = obel_n {
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                    } else {
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, &hpin)]);
                    }
                }

                for (name, hname, num) in [
                    ("CWA_ERR_CNT_INC", "CWA_ERR_COUNT_INC", 5),
                    ("CWB_ERR_CNT_INC", "CWB_ERR_COUNT_INC", 5),
                    ("L0_ERR_CNT_INC", "LANE0_ERR_COUNT_INC", 5),
                    ("L1_ERR_CNT_INC", "LANE1_ERR_COUNT_INC", 5),
                    ("L2_ERR_CNT_INC", "LANE2_ERR_COUNT_INC", 5),
                    ("L3_ERR_CNT_INC", "LANE3_ERR_COUNT_INC", 5),
                    ("MAPPING", "MAPPING", 12),
                ] {
                    for j in 0..num {
                        let opin = &format!("HNICPIPE_RSFEC_{name}_{i}_M_{j}_");
                        let ipin = &format!("HNICPIPE_RSFEC_{name}_{i}_{j}_");
                        bel = bel.extra_out_far_claim(opin).extra_in_far(ipin);
                        if let Some(obel) = obel_n {
                            bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                        } else {
                            let hpin = &format!("STAT_RSFEC_RX{i}_{hname}_{j}_");
                            let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                            bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, hpin)]);
                        }
                    }
                }
                for j in 0..4 {
                    let opin = &format!("HNICPIPE_FEC_LANE_LOCK_{i}_M_{j}_");
                    let ipin = &format!("HNICPIPE_FEC_LANE_LOCK_{i}_{j}_");
                    bel = bel.extra_out_far_claim(opin).extra_in_far(ipin);
                    if let Some(obel) = obel_n {
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                    } else {
                        let hpin = &format!("RX_PCS_BS_STATS_C{i}_FEC_LANE_LOCK_{j}_");
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, hpin)]);
                    }
                }
            }

            for i in 0..8 {
                for j in 0..80 {
                    // RX

                    let pin = &format!("HNICPIPE_HNICMAC_RXDATA{i}_{j}_");
                    let opin = &format!("HNICPIPE_RXDATA_BOT2TOP{i}_{j}_");
                    let hpin = &format!("RX_SERDES_DATA{i}_{j}_");
                    bel = bel.extra_out_far(pin);
                    if i < 4 {
                        bel = bel.extra_in_far_claim(opin);
                    }
                    if let Some(obel) = obel_n {
                        if i < 4 {
                            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
                        } else {
                            bel.claim_net(&[bel.wire_far(pin)]);
                        }
                    } else {
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, hpin)]);
                    }

                    // TX
                    let pin = &format!("HNICPIPE_HNICMAC_TXDATA{i}_{j}_");
                    let opin = &format!("HNICPIPE_TXDATA_TOP2BOT{i}_{j}_");
                    let hpin = &format!("TX_SERDES_DATA{i}_{j}_");
                    bel = bel.extra_in_far(pin);
                    if i < 4 {
                        bel = bel.extra_out_far_claim(opin);
                    }
                    if let Some(obel) = obel_n {
                        if i < 4 {
                            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
                        } else {
                            bel.claim_net(&[bel.wire_far(pin)]);
                        }
                    } else {
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, hpin)]);
                    }
                }

                // RX

                let opin = &format!("HNICPIPE_GT_RX_SERDES_CLK_GOOD_OUT_{i}_");
                let ipin = &format!("HNICPIPE_GT_RX_SERDES_CLK_GOOD_IN_{i}_");
                bel = bel.extra_out_far(opin);
                if i < 4 {
                    bel = bel.extra_in_far_claim(ipin);
                }
                if let Some(obel) = obel_n {
                    if i < 4 {
                        bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, ipin)]);
                    } else {
                        bel.claim_net(&[bel.wire_far(opin)]);
                    }
                } else {
                    let hpin = &format!("RX_SERDES_RESET_{i}_");
                    let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                    bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, hpin)]);
                }

                if i < 4 {
                    // TX
                    let opin = &format!("HNICPIPE_GT_TX_SERDES_CLK_GOOD_OUT_{i}_");
                    let ipin = &format!("HNICPIPE_GT_TX_SERDES_CLK_GOOD_IN_{i}_");
                    bel = bel.extra_out_far(opin).extra_in_far_claim(ipin);
                    if let Some(obel) = obel_n {
                        bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, ipin)]);
                    } else {
                        let hpin = &format!("TX_SERDES_RESET_{i}_");
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(opin), bel.bel_wire_far(obel, hpin)]);
                    }
                }
            }
            for (name, hname, num) in [
                ("RX_SERDES_CLK", "RX_SERDES_CLK", 8),
                ("TX_SERDES_CLK", "TX_SERDES_CLK", 4),
            ] {
                for j in 0..num {
                    let pin = &format!("HNICPIPE_{name}_{j}_");
                    let hpin = &format!("{hname}_{j}_");
                    bel = bel.extra_out_far(pin);
                    if obel_n.is_none() {
                        let obel = bcrd.delta(0, Chip::ROWS_PER_REG as i32).bel(bslots::HNICX);
                        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, hpin)]);
                    } else {
                        bel.claim_net(&[bel.wire_far(pin)]);
                    }
                }
            }

            for (iname, oname, num) in [
                ("RXUSRCLK_BOT2TOP_IN", "RXUSRCLK_BOT2TOP_OUT", 4),
                ("TXUSRCLK_BOT2TOP_IN", "TXUSRCLK_BOT2TOP_OUT", 4),
                ("RX_RST_BOT2TOP_IN", "RX_RST_BOT2TOP_OUT", 4),
                ("TX_RST_BOT2TOP_IN", "TX_RST_BOT2TOP_OUT", 4),
            ] {
                for j in 0..num {
                    let ipin = &format!("HNICPIPE_{iname}_{j}_");
                    let opin = &format!("HNICPIPE_{oname}_{j}_");
                    bel = bel.extra_in_far(ipin).extra_out_far_claim(opin);
                    if let Some(obel) = obel_s {
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                    } else {
                        bel.claim_net(&[bel.wire_far(ipin)]);
                    }
                }
            }

            for (i, obel) in [obel_s, obel_n].into_iter().enumerate() {
                let oi = i ^ 1;
                let mut pairs = vec![
                    (format!("ACK_{oi}"), format!("ACK_{i}_M")),
                    (format!("DEN_{oi}_M"), format!("DEN_{i}")),
                ];
                for j in 0..60 {
                    pairs.push((format!("DATA_{oi}_M_{j}_"), format!("DATA_{i}_{j}_")));
                }
                for (opin, ipin) in pairs {
                    let opin = &format!("HNICPIPE_UBLAZE_{opin}");
                    let ipin = &format!("HNICPIPE_UBLAZE_{ipin}");
                    bel = bel.extra_in_far(ipin).extra_out_far_claim(opin);
                    if let Some(obel) = obel {
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                    } else {
                        bel.claim_net(&[bel.wire_far(ipin)]);
                    }
                }
            }

            for rxtx in ["RX", "TX"] {
                for i in 0..4 {
                    let pin_sb = &format!("HNICPIPE_SOUTH_BOT_{rxtx}OUTCLK_{i}_");
                    let pin_st = &format!("HNICPIPE_SOUTH_TOP_{rxtx}OUTCLK_{i}_");
                    let pin_nb = &format!("HNICPIPE_NORTH_BOT_{rxtx}OUTCLK_{i}_");
                    let pin_nt = &format!("HNICPIPE_NORTH_TOP_{rxtx}OUTCLK_{i}_");
                    let pin_gt = &format!("CH{i}_{rxtx}OUTCLK");
                    bel = bel
                        .extra_in_far(pin_sb)
                        .extra_in_far(pin_st)
                        .extra_in_far(pin_nb)
                        .extra_in_far(pin_nt);
                    if let Some(obel_n) = obel_n {
                        bel.claim_net(&[bel.wire_far(pin_sb)]);
                        bel.claim_net(&[bel.wire_far(pin_st)]);
                        bel.verify_net(&[bel.wire_far(pin_nb), bel.wire_far(pin_gt)]);
                        bel.verify_net(&[bel.wire_far(pin_nt), bel.bel_wire_far(obel_n, pin_gt)]);
                    } else {
                        let obel_s = obel_s.unwrap();
                        bel.verify_net(&[bel.wire_far(pin_sb), bel.bel_wire_far(obel_s, pin_gt)]);
                        bel.verify_net(&[bel.wire_far(pin_st), bel.wire_far(pin_gt)]);
                        bel.verify_net(&[bel.wire_far(pin_nb), bel.wire("HNICPIPE_GND")]);
                        bel.verify_net(&[bel.wire_far(pin_nt), bel.wire("HNICPIPE_GND")]);
                    }
                }
            }
            bel.claim_net(&[bel.wire("HNICPIPE_GND")]);

            for pin in [
                "HNICPIPE_CSSD_TRIG_N",
                "HNICPIPE_EDT_UPDT_N",
                "HNICPIPE_SCAN_CLK_N",
                "HNICPIPE_SCAN_RST_N",
                "HNICPIPE_SCAN_EN_N",
                "HNICPIPE_SCAN_CNTRL_CHNL_IN",
                "HNICPIPE_SCAN_CHNL_IN_0_",
                "HNICPIPE_SCAN_CHNL_IN_1_",
                "HNICPIPE_SCAN_CHNL_IN_2_",
                "HNICPIPE_SCAN_CHNL_MASK_IN_0_",
                "HNICPIPE_SCAN_CHNL_MASK_IN_1_",
                "HNICPIPE_SCAN_CHNL_MASK_IN_2_",
            ] {
                bel = bel.extra_in(pin);
            }
        }

        for (pn, pin) in &naming.pins {
            if let Some(rpn) = pn.strip_prefix("_GT_") {
                let stem = if let Some(stem) = rpn.strip_suffix('_') {
                    stem.rsplit_once('_').unwrap().0
                } else {
                    rpn
                };
                let suffix = rpn.strip_prefix(stem).unwrap();
                let mpn = &format!("{stem}_M{suffix}");
                let &(_, dir) = pin.orig_pin.as_ref().unwrap();
                if dir == PinDir::Input {
                    bel = bel.extra_out_far_claim(mpn);
                } else {
                    bel = bel.extra_in_far_claim(mpn);
                }
                bel.verify_net(&[bel.wire_far(pn), bel.wire_far(mpn)]);
            }
        }

        bel.commit();
    }
}

pub fn verify_rfadc_rfdac(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far("CLK_DISTR_IN_LEFT_N")
        .extra_in_far("CLK_DISTR_IN_LEFT_P")
        .extra_in_far("CLK_DISTR_IN_LEFT_PLL_N")
        .extra_in_far("CLK_DISTR_IN_LEFT_PLL_P")
        .extra_in_far("CLK_DISTR_IN_LEFT_T1_N")
        .extra_in_far("CLK_DISTR_IN_LEFT_T1_P")
        .extra_in_far("CLK_DISTR_IN_RIGHT_N")
        .extra_in_far("CLK_DISTR_IN_RIGHT_P")
        .extra_out_far_claim("CLK_DISTR_OUT_LEFT_N")
        .extra_out_far_claim("CLK_DISTR_OUT_LEFT_P")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_N")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_P")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_PLL_N")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_PLL_P")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_T1_N")
        .extra_out_far_claim("CLK_DISTR_OUT_RIGHT_T1_P")
        .extra_in_far("SYSREF_IN1_L_N")
        .extra_in_far("SYSREF_IN1_L_P")
        .extra_in_far("SYSREF_IN2_L_N")
        .extra_in_far("SYSREF_IN2_L_P")
        .extra_in_far("SYSREF_IN_R_N")
        .extra_in_far("SYSREF_IN_R_P")
        .extra_out_far_claim("SYSREF_OUT1_R_N")
        .extra_out_far_claim("SYSREF_OUT1_R_P")
        .extra_out_far_claim("SYSREF_OUT2_R_N")
        .extra_out_far_claim("SYSREF_OUT2_R_P")
        .extra_out_far_claim("SYSREF_OUT_L_N")
        .extra_out_far_claim("SYSREF_OUT_L_P");

    let (bslots, idx) = if let Some(idx) = bslots::RFADC.index_of(bcrd.slot) {
        (bslots::RFADC, idx)
    } else {
        (bslots::RFDAC, bslots::RFDAC.index_of(bcrd.slot).unwrap())
    };

    if bcrd.slot == bslots::RFADC[0] {
        bel = bel
            .extra_out_far_claim("CLK_ADC_SPARE")
            .extra_out_far_claim("PLL_DMON_OUT_SPARE");
    } else if bcrd.slot == bslots::RFADC[1] {
        bel = bel
            .extra_out_far_claim("CLK_ADC")
            .extra_out_far_claim("PLL_DMON_OUT");
    } else if bcrd.slot == bslots::RFDAC[0] {
        bel = bel
            .extra_out_far_claim("CLK_DAC_SPARE")
            .extra_out_far_claim("CLK_CAL_SPARE")
            .extra_out_far_claim("PLL_DMON_OUT_SPARE");
    } else if bcrd.slot == bslots::RFDAC[1] {
        bel = bel
            .extra_out_far_claim("CLK_DAC")
            .extra_out_far_claim("CLK_CAL")
            .extra_out_far_claim("PLL_DMON_OUT");
    }

    let obel_prev = if idx == 0 {
        edev.bel_delta(bcrd.cell, 0, -96, bslots::RFDAC[1])
            .or(edev.bel_delta(bcrd.cell, 0, -96, bslots::RFADC[1]))
    } else {
        Some(bcrd.bel(bslots[0]))
    };

    let obel_next = if idx == 0 {
        Some(bcrd.bel(bslots[1]))
    } else {
        edev.bel_delta(bcrd.cell, 0, 96, bslots::RFDAC[0])
            .or(edev.bel_delta(bcrd.cell, 0, 96, bslots::RFADC[0]))
    };

    for pin in [
        "CLOCK_DR_IN_EXT",
        "RESET_TAP_B_IN_EXT",
        "CAPTURE_DR_IN_EXT",
        "SHIFT_DR_IN_EXT",
        "UPDATE_DR_IN_EXT",
        "SYS_RST1_N",
        "SYS_RST2_N",
        "SYS_RST3_N",
    ] {
        bel = bel.extra_in_far(pin);
        if let Some(obel) = obel_prev {
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, pin)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin)]);
        }
    }

    if let Some(obel) = obel_prev {
        for (pin, opin, brk_adc_dac) in [
            ("CLK_DISTR_IN_RIGHT_P", "CLK_DISTR_OUT_LEFT_P", true),
            ("CLK_DISTR_IN_RIGHT_N", "CLK_DISTR_OUT_LEFT_N", true),
            ("SYSREF_IN_R_P", "SYSREF_OUT_L_P", false),
            ("SYSREF_IN_R_N", "SYSREF_OUT_L_N", false),
        ] {
            if bcrd.slot == bslots::RFDAC[0] && obel.slot == bslots::RFADC[1] && brk_adc_dac {
                bel.claim_net(&[bel.wire_far(pin)])
            } else {
                bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)])
            }
        }
    } else {
        for (pin, opin) in [
            ("CLK_DISTR_IN_RIGHT_P", "CLK_DISTR_OUT_RIGHT_P"),
            ("CLK_DISTR_IN_RIGHT_N", "CLK_DISTR_OUT_RIGHT_N"),
            ("SYSREF_IN_R_P", "SYSREF_OUT1_R_P"),
            ("SYSREF_IN_R_N", "SYSREF_OUT1_R_N"),
        ] {
            bel.verify_net(&[bel.wire_far(pin), bel.wire_far(opin)])
        }
    }

    if let Some(obel) = obel_next {
        for (pin, opin) in [
            ("CLK_DISTR_IN_LEFT_P", "CLK_DISTR_OUT_RIGHT_P"),
            ("CLK_DISTR_IN_LEFT_N", "CLK_DISTR_OUT_RIGHT_N"),
            ("CLK_DISTR_IN_LEFT_PLL_P", "CLK_DISTR_OUT_RIGHT_PLL_P"),
            ("CLK_DISTR_IN_LEFT_PLL_N", "CLK_DISTR_OUT_RIGHT_PLL_N"),
            ("CLK_DISTR_IN_LEFT_T1_P", "CLK_DISTR_OUT_RIGHT_T1_P"),
            ("CLK_DISTR_IN_LEFT_T1_N", "CLK_DISTR_OUT_RIGHT_T1_N"),
            ("SYSREF_IN1_L_P", "SYSREF_OUT1_R_P"),
            ("SYSREF_IN1_L_N", "SYSREF_OUT1_R_N"),
            ("SYSREF_IN2_L_P", "SYSREF_OUT2_R_P"),
            ("SYSREF_IN2_L_N", "SYSREF_OUT2_R_N"),
        ] {
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)])
        }
    } else {
        for (pin, opin) in [
            ("CLK_DISTR_IN_LEFT_P", "CLK_DISTR_OUT_LEFT_P"),
            ("CLK_DISTR_IN_LEFT_N", "CLK_DISTR_OUT_LEFT_N"),
            ("CLK_DISTR_IN_LEFT_PLL_P", "CLK_DISTR_OUT_LEFT_P"),
            ("CLK_DISTR_IN_LEFT_PLL_N", "CLK_DISTR_OUT_LEFT_N"),
            ("CLK_DISTR_IN_LEFT_T1_P", "CLK_DISTR_OUT_LEFT_P"),
            ("CLK_DISTR_IN_LEFT_T1_N", "CLK_DISTR_OUT_LEFT_N"),
            ("SYSREF_IN1_L_P", "SYSREF_OUT_L_P"),
            ("SYSREF_IN1_L_N", "SYSREF_OUT_L_N"),
            ("SYSREF_IN2_L_P", "SYSREF_OUT_L_P"),
            ("SYSREF_IN2_L_N", "SYSREF_OUT_L_N"),
        ] {
            bel.verify_net(&[bel.wire_far(pin), bel.wire_far(opin)])
        }
    }

    bel.commit();
}

pub fn verify_hnicx(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);

    for i in 0..4 {
        for j in 0..16 {
            bel = bel.extra_in_far_claim(format!("ANLT_FEC_STATUS_P{i}_{j}_"));
        }

        for name in [
            format!("RX_PCS_BS_STATS_C{i}_FEC_ALIGNED"),
            format!("RX_PCS_BS_STATS_C{i}_STATUS"),
            format!("STAT_RSFEC_RX{i}_CORRECTED_CWB_INC"),
            format!("STAT_RSFEC_RX{i}_CORRECTED_CW_INC"),
            format!("STAT_RSFEC_RX{i}_CWB_INC"),
            format!("STAT_RSFEC_RX{i}_CW_INC"),
            format!("STAT_RSFEC_RX{i}_UNCORRECTED_CWB_INC"),
            format!("STAT_RSFEC_RX{i}_UNCORRECTED_CW_INC"),
        ] {
            bel = bel.extra_out_far_claim(name);
        }
        for (name, num) in [
            (format!("RX_PCS_BS_STATS_C{i}_FEC_LANE_LOCK"), 4),
            (format!("STAT_RSFEC_RX{i}_CWA_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_CWB_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_LANE0_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_LANE1_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_LANE2_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_LANE3_ERR_COUNT_INC"), 5),
            (format!("STAT_RSFEC_RX{i}_MAPPING"), 12),
        ] {
            for j in 0..num {
                bel = bel.extra_out_far_claim(format!("{name}_{j}_"));
            }
        }
    }

    for i in 0..4 {
        bel = bel
            .extra_in_far_claim(format!("TX_SERDES_CLK_{i}_"))
            .extra_in_far_claim(format!("TX_SERDES_RESET_{i}_"));
    }
    for i in 0..8 {
        bel = bel
            .extra_in_far_claim(format!("RX_SERDES_CLK_{i}_"))
            .extra_in_far_claim(format!("RX_SERDES_RESET_{i}_"));
        for j in 0..80 {
            bel = bel
                .extra_in_far_claim(format!("RX_SERDES_DATA{i}_{j}_"))
                .extra_out_far_claim(format!("TX_SERDES_DATA{i}_{j}_"));
        }
    }

    let chip = edev.chips[bcrd.die];
    let HsrNKind::Nic(ref nic) = chip.hsr_n else {
        unreachable!()
    };
    let cell_hnoc = bcrd
        .die
        .cell(nic.blocks.last_id().unwrap(), chip.row_hsr(DirV::N));
    for (pin, obel) in [
        ("HNICX_NOC_AXIMM_M", cell_hnoc.bel(bslots::HNOC_NIC_AXI_NMU)),
        (
            "HNICX_NOC_AXIS_M0",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NMU[0]),
        ),
        (
            "HNICX_NOC_AXIS_M1",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NMU[1]),
        ),
        (
            "HNICX_NOC_AXIS_M2",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NMU[2]),
        ),
        (
            "HNICX_NOC_AXIS_M3",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NMU[3]),
        ),
    ] {
        bel = bel.extra_out_far_to_far(pin, obel, "AXI_IN");
    }
    for (pin, obel) in [
        ("HNICX_NOC_AXIMM_S", cell_hnoc.bel(bslots::HNOC_NIC_AXI_NSU)),
        (
            "HNICX_NOC_AXIS_S0",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NSU[0]),
        ),
        (
            "HNICX_NOC_AXIS_S1",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NSU[1]),
        ),
        (
            "HNICX_NOC_AXIS_S2",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NSU[2]),
        ),
        (
            "HNICX_NOC_AXIS_S3",
            cell_hnoc.bel(bslots::HNOC_NIC_AXIS_NSU[3]),
        ),
    ] {
        bel = bel.extra_in_far_from_far(pin, obel, "AXI_OUT");
    }

    bel.commit();
}
