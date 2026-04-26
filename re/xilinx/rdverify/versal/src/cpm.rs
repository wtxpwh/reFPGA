use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::PinDir,
    dir::DirV,
    grid::{BelCoord, ColId, DieIdExt, RowId},
};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    chip::Chip,
    defs::{bslots, tcls},
    expanded::ExpandedDevice,
};

pub fn verify_cpm4(vrf: &mut Verifier, bcrd: BelCoord) {
    let ps = bcrd.with_row(RowId::from_idx(0)).bel(bslots::PS9);
    let naming = vrf.ngrid.get_bel_naming(bcrd);

    let mut bel = vrf.verify_bel(bcrd).kind("CPM_MAIN").skip_auto();

    for (pin, pdata) in &naming.pins {
        if pdata.tile.to_idx() != 0 {
            continue;
        }
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if pin.starts_with("CPMPLPLL")
            || pin == "CPMOSCCLKDIV2"
            || pin == "EFUSECPMCCIXDIS"
            || pin.starts_with("IFCPMPS")
            || pin.starts_with("IFPSCPM")
            || pin.starts_with("LPDCPM")
            || pin.starts_with("PERST")
        {
            // PS9
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            let pspin = if let Some(suffix) = pin.strip_prefix("CPMPLPLL") {
                format!("CPMPLL{suffix}")
            } else {
                pin.to_string()
            };
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(ps, &pspin)]);
        } else if pin.starts_with("DFXE2M")
            || pin.starts_with("IFCPMEXT")
            || pin.starts_with("IFEXTCPM")
            || pin.starts_with("PCIEAEXT")
        {
            // EXT
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_out_far_claim(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.wire_far(&format!("_EXT_{pin}"))]);
        } else if pin.starts_with("IFCPMXPIPE") {
            // XPIPE
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_out_far_claim(pin);
            }
            // connection handled in XPIPE
        } else {
            // normal interconnect
            if dir == PinDir::Input {
                bel = bel.extra_in(pin);
            } else {
                bel = bel.extra_out(pin);
            }
        }
    }

    bel.commit();

    let mut bel = vrf.verify_bel(bcrd).sub(1).kind("CPM_EXT").skip_auto();

    for (pin, pdata) in &naming.pins {
        if pdata.tile.to_idx() != 1 {
            continue;
        }
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if pin.starts_with("_EXT_") {
            // connection to core
            if dir == PinDir::Input {
                bel = bel.extra_in_claim(pin);
                bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
            } else {
                bel = bel.extra_out_claim(pin);
                bel.claim_pip(bel.wire_far(pin), bel.wire(pin));
            }
        } else {
            // normal interconnect
            if dir == PinDir::Input {
                bel = bel.extra_in(pin);
            } else {
                bel = bel.extra_out(pin);
            }
        }
    }

    bel.commit();
}

fn xlat_cpm5_gt_pin(bcrd: BelCoord, pin: &str) -> Option<(BelCoord, String)> {
    let gtyp = |quad| {
        bcrd.delta(0, (Chip::ROWS_PER_REG * quad) as i32)
            .bel(bslots::GTYP)
    };
    let (pin, idx) = if let Some(pre) = pin.strip_suffix('_') {
        let (pin, idx) = pre.rsplit_once('_').unwrap();
        let idx: usize = idx.parse().unwrap();
        (pin, Some(idx))
    } else {
        (pin, None)
    };
    if let Some(rest) = pin.strip_prefix("IFBUFGTQ") {
        let quad = rest.strip_suffix("CLKBUFGT").unwrap();
        let quad: usize = quad.parse().unwrap();
        let idx = idx.unwrap();
        if idx == 14 {
            None
        } else {
            Some((
                gtyp(quad),
                [
                    "REFCLK0_ODIV2",
                    "CH0_TXOUTCLK",
                    "CH1_TXOUTCLK",
                    "CH0_RXOUTCLK",
                    "CH1_RXOUTCLK",
                    "CH0_DMONITOROUTCLK",
                    "CH1_DMONITOROUTCLK",
                    "REFCLK1_ODIV2",
                    "CH2_TXOUTCLK",
                    "CH3_TXOUTCLK",
                    "CH2_RXOUTCLK",
                    "CH3_RXOUTCLK",
                    "CH2_DMONITOROUTCLK",
                    "CH3_DMONITOROUTCLK",
                ][idx]
                    .to_string(),
            ))
        }
    } else if let Some(rest) = pin.strip_prefix("IFFCQ") {
        let quad: usize = rest[0..1].parse().unwrap();
        let ch: usize = rest[1..2].parse().unwrap();
        let pin = &rest[2..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "APTEXTHOLD" => "DFEHOLD",
            "APTOVERWREN" => "DFEOVRD",
            "BSRSERIAL" => "BSR_SERIAL",
            "CDRBMCDREQ" => "CDRBMCDRREQ",
            "CDRHOLD" => "RXCDRHOLD",
            "CDRLOCK" => "RXCDRLOCK",
            "CDROVREN" => "RXCDROVRDEN",
            "CDRPHRESET" => "RXCDRRESET",
            "CFOKDONE" => "RXOSINTDONE",
            "CKPINRSRVD0" => "CLKRSVD0",
            "CKPINRSRVD1" => "CLKRSVD1",
            "COMINITDET" => "RXCOMINITDET",
            "COMSASDET" => "RXCOMSASDET",
            "COMWAKEDET" => "RXCOMWAKEDET",
            "DMONCLK" => "DMONITORCLK",
            "DMONOUT" => "DMONITOROUT",
            "ENPPM" => "TXPIPPMEN",
            "PERSTB" => "PCIERSTB",
            "PHYESMADAPTATIONSAVE" => "PHYESMADAPTSAVE",
            "PINRSRVDAS" => "PINRSVDAS",
            "PINRSRVD" => "GTRSVD",
            "REFCLKPMA" => "REFDEBUGOUT",
            "RXCHICORESETMASK" => "RXPHALIGNRESETMASK",
            "RXCHISALIGNED" => "RXCHANISALIGNED",
            "RXCKCORCNT" => "RXCLKCORCNT",
            "RXDELAYALIGNERR" => "RXDLYALIGNERR",
            "RXDELAYALIGNPROGRESS" => "RXDLYALIGNPROG",
            "RXDELAYALIGNREQ" => "RXDLYALIGNREQ",
            "RXETHERNETSTATOUT" => "RX10GSTAT",
            "RXOUTPCSCLK" => "RXDEBUGPCSOUT",
            "RXPOWERDOWN" => "RXPD",
            "RXPHASEALIGNDONE" => "RXPHALIGNDONE",
            "RXPHASEALIGNERR" => "RXPHALIGNERR",
            "RXPHASEALIGNPD" => "RXPHDLYPD",
            "RXPHASEALIGNREQ" => "RXPHALIGNREQ",
            "RXPHASEDELAYRESETDONE" => "RXPHDLYRESETDONE",
            "RXPHASESETINITDONE" => "RXPHSETINITDONE",
            "RXPHASESETINITREQ" => "RXPHSETINITREQ",
            "RXPHASESHIFT180DONE" => "RXPHSHIFT180DONE",
            "RXPHASESHIFT180REQ" => "RXPHSHIFT180",
            "RXUSRRDY" => "RXUSERRDY",
            "STEPSIZEPPM" => "TXPIPPMSTEPSIZE",
            "TCOCLKFSMFROUT" => "TXPHDLYTSTCLK",
            "TXCHICOOUTRSVD" => "TXPHALIGNOUTRSVD",
            "TXCHICORESETMASK" => "TXPHALIGNRESETMASK",
            "TXDELAYALIGNERR" => "TXDLYALIGNERR",
            "TXDELAYALIGNPROGRESS" => "TXDLYALIGNPROG",
            "TXDELAYALIGNREQ" => "TXDLYALIGNREQ",
            "TXDETECTRXLOOPBACK" => "TXDETECTRX",
            "TXDRVAMP" => "TXDIFFCTRL",
            "TXEMPMAIN" => "TXMAINCURSOR",
            "TXEMPPOS" => "TXPOSTCURSOR",
            "TXEMPPRE" => "TXPRECURSOR",
            "TXETHERNETSTATTXLOCALFAULT" => "TX10GSTAT",
            "TXOUTPCSCLK" => "TXDEBUGPCSOUT",
            "TXPHASEALIGNDONE" => "TXPHALIGNDONE",
            "TXPHASEALIGNERR" => "TXPHALIGNERR",
            "TXPHASEALIGNPD" => "TXPHDLYPD",
            "TXPHASEALIGNREQ" => "TXPHALIGNREQ",
            "TXPHASEDELAYRESETDONE" => "TXPHDLYRESETDONE",
            "TXPHASESETINITDONE" => "TXPHSETINITDONE",
            "TXPHASESETINITREQ" => "TXPHSETINITREQ",
            "TXPHASESHIFT180DONE" => "TXPHSHIFT180DONE",
            "TXPHASESHIFT180REQ" => "TXPHSHIFT180",
            "TXPOWERDOWN" => "TXPD",
            "TXSERPWRDN" => "TXPISOPD",
            "TXTXPICODEOVRDEN" => "TXPICODEOVRDEN",
            "TXTXPICODERESET" => "TXPICODERESET",
            "TXUSRRDY" => "TXUSERRDY",
            "XPIPE5PIPELINEEN" => "XPIPE5_PIPELINE_EN",
            _ => pin,
        };
        Some((
            gtyp(quad),
            if let Some(idx) = idx {
                format!("CH{ch}_{pin}_{idx}_")
            } else {
                format!("CH{ch}_{pin}")
            },
        ))
    } else if let Some(rest) = pin.strip_prefix("IFFCTRLQ") {
        let quad: usize = rest[0..1].parse().unwrap();
        let pin = &rest[1..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "PRESETN" => "APB3PRESETN",
            "PENABLE" => "APB3PENABLE",
            "PWRITE" => "APB3PWRITE",
            "PSEL" => "APB3PSEL",
            "PADDR" => "APB3PADDR",
            "PWDATA" => "APB3PWDATA",
            "PREADY" => "APB3PREADY",
            "PRDATA" => "APB3PRDATA",
            "PSLVERR" => "APB3PSLVERR",
            "BGBYPASS" => "BGBYPASSB",
            "BGPWRDNB" => "BGPDB",
            "BGRCALCTL" => "BGRCALOVRD",
            "BGTESTEN" => "BGMONITORENB",
            "COEREGRST" => "COESTATUSDEBUG",
            "CORRECTABLEERR" => "CORRECTERR",
            "UNCORRECTABLEERR" => "UNCORRECTERR",
            "DEBUGTRACETREADY" => "DEBUGTRACEREADY",
            "GTRSVDIN" => "CTRLRSVDIN",
            "GTRSVDOUT" => "CTRLRSVDOUT",
            "M0AXISTDATA" => "M0_AXIS_TDATA",
            "M0AXISTLAST" => "M0_AXIS_TLAST",
            "M0AXISTREADY" => "M0_AXIS_TREADY",
            "M0AXISTVALID" => "M0_AXIS_TVALID",
            "M1AXISTDATA" => "M1_AXIS_TDATA",
            "M1AXISTLAST" => "M1_AXIS_TLAST",
            "M1AXISTREADY" => "M1_AXIS_TREADY",
            "M1AXISTVALID" => "M1_AXIS_TVALID",
            "M2AXISTDATA" => "M2_AXIS_TDATA",
            "M2AXISTLAST" => "M2_AXIS_TLAST",
            "M2AXISTREADY" => "M2_AXIS_TREADY",
            "M2AXISTVALID" => "M2_AXIS_TVALID",
            "S0AXISTDATA" => "S0_AXIS_TDATA",
            "S0AXISTLAST" => "S0_AXIS_TLAST",
            "S0AXISTREADY" => "S0_AXIS_TREADY",
            "S0AXISTVALID" => "S0_AXIS_TVALID",
            "S1AXISTDATA" => "S1_AXIS_TDATA",
            "S1AXISTLAST" => "S1_AXIS_TLAST",
            "S1AXISTREADY" => "S1_AXIS_TREADY",
            "S1AXISTVALID" => "S1_AXIS_TVALID",
            "S2AXISTDATA" => "S2_AXIS_TDATA",
            "S2AXISTLAST" => "S2_AXIS_TLAST",
            "S2AXISTREADY" => "S2_AXIS_TREADY",
            "S2AXISTVALID" => "S2_AXIS_TVALID",
            "PCIELTSSMSTATE" => "PCIELTSSM",
            "RCAL" => "RCALOUT",
            "RXMARGINREQPAYLOAD" => "RXMARGINREQPAYLD",
            "RXMARGINRESPAYLOAD" => "RXMARGINRESPAYLD",
            "UBGPI" => "GPI",
            "UBGPO" => "GPO",
            _ => pin,
        };
        Some((
            gtyp(quad),
            if let Some(idx) = idx {
                format!("{pin}_{idx}_")
            } else {
                pin.to_string()
            },
        ))
    } else if let Some(rest) = pin.strip_prefix("IFFHSQ") {
        let quad: usize = rest[0..1].parse().unwrap();
        let hs: usize = rest[1..2].parse().unwrap();
        let pin = &rest[2..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "COREREFCLK0" => "LCPLLGTGREFCLK",
            "COREREFCLK1" => "RPLLGTGREFCLK",
            "LCPLLFBLOSS" => "LCPLLFBCLKLOST",
            "LCPLLFREQLOCK" => "LCPLLLOCK",
            "LCPLLPWRDN" => "LCPLLPD",
            "LCPLLREFLOSS" => "LCPLLREFCLKLOST",
            "LCPLLREFSELDYN" => "LCPLLREFCLKSEL",
            "RPLLFBLOSS" => "RPLLFBCLKLOST",
            "RPLLFREQLOCK" => "RPLLLOCK",
            "RPLLPWRDN" => "RPLLPD",
            "RPLLREFLOSS" => "RPLLREFCLKLOST",
            "RPLLREFSELDYN" => "RPLLREFCLKSEL",
            "MGTLCPLLREFCLKFA" => "LCPLLREFCLKMONITOR",
            "MGTRPLLREFCLKFA" => "RPLLREFCLKMONITOR",
            _ => pin,
        };
        Some((
            gtyp(quad),
            if let Some(idx) = idx {
                format!("HSCLK{hs}_{pin}_{idx}_")
            } else {
                format!("HSCLK{hs}_{pin}")
            },
        ))
    } else if let Some(rest) = pin.strip_prefix("IFFRCKQ") {
        let quad: usize = rest[0..1].parse().unwrap();
        let rc: usize = rest[1..2].parse().unwrap();
        let pin = &rest[2..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "HROWTESTCK" => "CLKTESTSIG",
            "REFCLKPD" => "GTREFCLKPD",
            _ => unreachable!(),
        };
        Some((gtyp(quad), format!("REFCLK{rc}_{pin}")))
    } else if pin.starts_with("SCAN") {
        let (pin, quad) = pin.rsplit_once('Q').unwrap();
        let quad: usize = quad.parse().unwrap();
        let (pref, pin) = if let Some(pin) = pin
            .strip_suffix("GTCTRLFS")
            .or(pin.strip_suffix("GTCTRLSF"))
        {
            ("", pin)
        } else {
            let (pin, idx) = pin.split_at(pin.len() - 1);
            let pref = match idx {
                "0" => "CH0_",
                "1" => "CH1_",
                "2" => "CH2_",
                "3" => "CH3_",
                _ => unreachable!(),
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            (pref, pin)
        };
        let pin = match pin {
            "SCANCLKN" => "SCANCLKB",
            "SCANMODERSTN" => "SCANRSTB",
            "SCANENN" => "SCANENB",
            "SCANEDTUPDTN" => "EDTUPDATEB",
            "SCANCHNLIN" => "SCANIN",
            "SCANCHNLOUT" => "SCANOUT",
            "SCANCNTRLCHNLIN" => "SCANCNTRLIN",
            "SCANCNTRLCHNLOUT" => "SCANCNTRLOUT",
            _ => panic!("ummm {pin}"),
        };
        let pin = if let Some(idx) = idx {
            format!("{pref}{pin}_{idx}_")
        } else {
            format!("{pref}{pin}")
        };
        Some((gtyp(quad), pin))
    } else {
        None
    }
}

pub fn verify_cpm5(vrf: &mut Verifier, bcrd: BelCoord) {
    let ps = bcrd.with_row(RowId::from_idx(0)).bel(bslots::PS9);
    let naming = vrf.ngrid.get_bel_naming(bcrd);

    let mut bel = vrf.verify_bel(bcrd);
    for (pin, pdata) in &naming.pins {
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if pin.starts_with("CPMPLPLL")
            || pin == "CPMOSCCLKDIV2"
            || pin == "EFUSECPMCCIXDIS"
            || pin.starts_with("IFCPMPS")
            || pin.starts_with("IFPSCPM")
            || pin.starts_with("LPDCPM")
            || pin.starts_with("PERST")
        {
            // PS9
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            let pspin = if let Some(suffix) = pin.strip_prefix("CPMPLPLL") {
                format!("CPMPLL{suffix}")
            } else {
                pin.to_string()
            };
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(ps, &pspin)]);
        } else if let Some((obel, opin)) = xlat_cpm5_gt_pin(bcrd, pin) {
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, &opin)]);
        } else if pin.starts_with("IFBUFGT") {
            bel = bel.extra_in_far_claim(pin);
        } else if pin.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pin);
        } else if pin.starts_with("Q") {
            // dummy
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_out_far_claim(pin);
            }
        }
    }

    bel.commit();
}

fn xlat_cpm6_gt_pin(bcrd: BelCoord, pin: &str) -> Option<(BelCoord, String)> {
    let opin = pin;
    let (pin, mut idx) = if let Some(pre) = pin.strip_suffix('_') {
        let (pin, idx) = pre.rsplit_once('_').unwrap();
        let idx: usize = idx.parse().unwrap();
        (pin, Some(idx))
    } else {
        (pin, None)
    };
    if let Some(pin) = pin
        .strip_prefix("BOTGTOCT")
        .or(pin.strip_prefix("TOPGTOCT"))
    {
        let gthi = if opin.starts_with("BOT") { 0 } else { 2 };
        let (pref, pin, gtlo) = if let Some((pin, suf)) = pin.split_once("GTCTRL") {
            (
                "",
                pin,
                match suf {
                    "FS" | "SF" => 0,
                    "FSTOP" | "SFTOP" => 1,
                    _ => unreachable!(),
                },
            )
        } else {
            let (pin, idx) = pin.split_at(pin.len() - 1);
            let pref = match idx {
                "0" => "CH0_",
                "1" => "CH1_",
                "2" => "CH2_",
                "3" => "CH3_",
                _ => unreachable!(),
            };
            let (pin, gtlo) = if let Some(pin) = pin.strip_suffix("TOP") {
                (pin, 1)
            } else {
                (pin, 0)
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            (pref, pin, gtlo)
        };
        let pin = match pin {
            "SCANCLKN" => "SCANCLKB",
            "SCANMODERSTN" => "SCANRSTB",
            "SCANENN" => "SCANENB",
            "SCANEDTUPDTN" => {
                if pref.is_empty() {
                    "SCANEDTUPDTB"
                } else {
                    "EDTUPDATEB"
                }
            }
            "SCANODCCCHNLMASKIN" => {
                if pref.is_empty() {
                    "SCANODCCCHNLMASKIN"
                } else {
                    "SCANODCCCHNLMASK"
                }
            }
            "SCANCHNLIN" => "SCANIN",
            "SCANCHNLMASKIN" => "SCANCHNLMASKIN",
            "SCANCHNLOUT" => "SCANOUT",
            "SCANCNTRLCHNLIN" => "SCANCNTRLIN",
            "SCANCNTRLCHNLOUT" => "SCANCNTRLOUT",
            _ => panic!("ummm {pin}"),
        };
        let gt = gthi + gtlo;
        let pin = if let Some(idx) = idx {
            format!("{pref}{pin}_{idx}_")
        } else {
            format!("{pref}{pin}")
        };
        Some((bcrd.delta(0, gt * 48).bel(bslots::GTM2), pin))
    } else if let Some(pin) = pin
        .strip_prefix("IFBOTGTOCT")
        .or(pin.strip_prefix("IFTOPGTOCT"))
    {
        let gthi = if opin.starts_with("IFBOT") { 0 } else { 2 };
        let (pin, gtlo) = if matches!(pin, "BUFGTCLKBUFGT" | "TOPBUFGTCLKBUFGT") {
            (
                [
                    "REFCLK0_ODIV2",
                    "CH0_TXOUTCLK",
                    "CH1_TXOUTCLK",
                    "CH0_RXOUTCLK",
                    "CH1_RXOUTCLK",
                    "CH0_DMONITOROUTCLK",
                    "CH1_DMONITOROUTCLK",
                    "REFCLK1_ODIV2",
                    "CH2_TXOUTCLK",
                    "CH3_TXOUTCLK",
                    "CH2_RXOUTCLK",
                    "CH3_RXOUTCLK",
                    "CH2_DMONITOROUTCLK",
                    "CH3_DMONITOROUTCLK",
                    "BUFG_GT_RSVD",
                ][idx.take().unwrap()]
                .to_string(),
                if pin.starts_with("TOP") { 1 } else { 0 },
            )
        } else if let Some(pin) = pin.strip_prefix("FCTRL") {
            let (pin, gtlo) = if let Some(pin) = pin.strip_prefix('T') {
                (pin, 1)
            } else {
                (pin, 0)
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            let pin = match pin {
                "CSSDSTOPCLK" => "CSSDSTOPCLKB",
                "UBGPI" => "GPI",
                "UBGPO" => "GPO",
                _ => pin,
            };
            (pin.to_string(), gtlo)
        } else if pin.starts_with("FC") {
            let (ch, pin, gtlo) = if let Some(pin) = pin.strip_prefix("FC0") {
                (0, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FC1") {
                (1, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FC2") {
                (2, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FC3") {
                (3, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FCT0") {
                (0, pin, 1)
            } else if let Some(pin) = pin.strip_prefix("FCT1") {
                (1, pin, 1)
            } else if let Some(pin) = pin.strip_prefix("FCT2") {
                (2, pin, 1)
            } else if let Some(pin) = pin.strip_prefix("FCT3") {
                (3, pin, 1)
            } else {
                panic!("ummm {pin}")
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            let pin = match pin {
                "BSRSERIAL" => "BSR_SERIAL",
                "CDRHOLD" => "RXCDRHOLD",
                "CSSDSTOPCLK" => "CSSDSTOPCLKB",
                "DMONCLK" => "DMONITORCLK",
                "DMONOUT" => "DMONITOROUT",
                "PCIELTSSMSTATE" => "PCIE_LTSSM_STATE",
                "UPI2CTRLPHYCMDCODE" => "UPI2CMDCODE",
                "UPI2CTRLPHYCMDREQ" => "UPI2CMDREQ",
                "UPI2PHYCTRLCMDERROR" => "UPI2CMDERROR",
                "UPI2PHYCTRLCMDREADY" => "UPI2CMDREADY",
                "UPI2PHYCTRLCMDRESP" => "UPI2CMDRESP",
                "UPI2PHYCTRLMSGREQ" => "UPI2MSGREQ",
                _ => pin,
            };
            (format!("CH{ch}_{pin}"), gtlo)
        } else if pin.starts_with("FHS") {
            let (hs, pin, gtlo) = if let Some(pin) = pin.strip_prefix("FHS0") {
                (0, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FHS1") {
                (1, pin, 0)
            } else if let Some(pin) = pin.strip_prefix("FHST0") {
                (0, pin, 1)
            } else if let Some(pin) = pin.strip_prefix("FHST1") {
                (1, pin, 1)
            } else {
                panic!("ummm {pin}")
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            let pin = match pin {
                "COREREFCLK0" => "LCPLLGTGREFCLK",
                "COREREFCLK1" => "RPLLGTGREFCLK",
                "MGTLCPLLREFCLKFA" => "LCPLLREFCLKMONITOR",
                "MGTRPLLREFCLKFA" => "RPLLREFCLKMONITOR",
                _ => pin,
            };
            (format!("HSCLK{hs}_{pin}"), gtlo)
        } else if pin.starts_with("FRCK") {
            let (pin, gtlo) = match pin {
                "FRCK0HROWTESTCKFS" => ("REFCLK0_CLKTESTSIG", 0),
                "FRCK1HROWTESTCKFS" => ("REFCLK1_CLKTESTSIG", 0),
                "FRCKT0HROWTESTCKFS" => ("REFCLK0_CLKTESTSIG", 1),
                "FRCKT1HROWTESTCKFS" => ("REFCLK1_CLKTESTSIG", 1),
                _ => unreachable!(),
            };
            (pin.to_string(), gtlo)
        } else {
            panic!("ummm {pin}")
        };
        let gt = gthi + gtlo;
        let pin = if let Some(idx) = idx {
            format!("{pin}_{idx}_")
        } else {
            pin.to_string()
        };
        Some((bcrd.delta(0, gt * 48).bel(bslots::GTM2), pin))
    } else {
        None
    }
}

pub fn verify_cpm6(vrf: &mut Verifier, bcrd: BelCoord) {
    let ps = bcrd.with_row(RowId::from_idx(0)).bel(bslots::PS9);
    let naming = vrf.ngrid.get_bel_naming(bcrd);

    let mut bel = vrf.verify_bel(bcrd);
    for (pin, pdata) in &naming.pins {
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if pin.starts_with("CPMPLPLL")
            || pin == "CPMOSCCLKDIV2"
            || pin == "EFUSECPMCCIXDIS"
            || pin.starts_with("IFCPMPS")
            || pin.starts_with("IFPSCPM")
            || pin.starts_with("LPDCPM")
            || pin.starts_with("PERST")
        {
            // PS9
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            let pspin = if let Some(suffix) = pin.strip_prefix("CPMPLPLL") {
                format!("CPMPLL{suffix}")
            } else {
                pin.to_string()
            };
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(ps, &pspin)]);
        } else if let Some((obel, opin)) = xlat_cpm6_gt_pin(bcrd, pin) {
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, &opin)]);
        } else if pin.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pin);
        }
    }
    bel.commit();
}

pub fn verify_cpm5n(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let ps = bcrd.with_row(RowId::from_idx(0)).bel(bslots::PSXL);
    let naming = vrf.ngrid.get_bel_naming(bcrd);

    let mut bel = vrf.verify_bel(bcrd);
    for (pin, pdata) in &naming.pins {
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if matches!(
            pin.as_str(),
            "CPM5ISPRESENT" | "CPMOSCCLKDIV2" | "CPMPSXCXSCLK" | "DBGLPDRAWRSTN"
        ) || pin.starts_with("EFUSE")
            || pin.starts_with("IFCMNCML256")
            || pin.starts_with("IFCPMPS")
            || pin.starts_with("IFCPM5NPSX")
            || pin.starts_with("IFPSCPM")
            || pin.starts_with("IFPSXCPM5N")
            || pin.starts_with("LPDCPM")
            || pin.starts_with("PERST")
        {
            // PSXL
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(ps, pin)]);
        } else if let Some((obel, opin)) = xlat_cpm5_gt_pin(bcrd, pin) {
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, &opin)]);
        } else if pin.starts_with("IFBUFGT") {
            bel = bel.extra_in_far_claim(pin);
        } else if pin.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pin);
        } else if pin.starts_with("Q") {
            // dummy
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_out_far_claim(pin);
            }
        }
    }

    let chip = edev.chips[bcrd.die];
    let cell_cdx = bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::N));
    let cell_dpu = cell_cdx.delta(1, 0);
    for (pin, obel) in [
        ("CDX_NMU_AXIM", cell_cdx.bel(bslots::HNOC_NIC_AXI_NMU)),
        ("CDX_NMU_AXIS0", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NMU[0])),
        ("CDX_NMU_AXIS1", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NMU[1])),
        ("CDX_NMU_AXIS2", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NMU[2])),
        ("CDX_NMU_AXIS3", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NMU[3])),
        ("DPU_NMU_AXIS0", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NMU[0])),
        ("DPU_NMU_AXIS1", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NMU[1])),
        ("DPU_NMU_AXIS2", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NMU[2])),
        ("DPU_NMU_AXIS3", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NMU[3])),
    ] {
        bel = bel.extra_out_far_to_far(pin, obel, "AXI_IN");
    }
    for (pin, obel) in [
        ("CDX_NSU_AXIM", cell_cdx.bel(bslots::HNOC_NIC_AXI_NSU)),
        ("CDX_NSU_AXIS0", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NSU[0])),
        ("CDX_NSU_AXIS1", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NSU[1])),
        ("CDX_NSU_AXIS2", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NSU[2])),
        ("CDX_NSU_AXIS3", cell_cdx.bel(bslots::HNOC_NIC_AXIS_NSU[3])),
        ("DPU_NSU_AXIS0", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NSU[0])),
        ("DPU_NSU_AXIS1", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NSU[1])),
        ("DPU_NSU_AXIS2", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NSU[2])),
        ("DPU_NSU_AXIS3", cell_dpu.bel(bslots::HNOC_NIC_AXIS_NSU[3])),
    ] {
        bel = bel.extra_in_far_from_far(pin, obel, "AXI_OUT");
    }

    bel.commit();
}

fn xlat_mmi_gt_pin(pin: &str) -> Option<String> {
    let pin = pin.strip_prefix("IFMMIGTYP")?;
    let (pin, idx) = if let Some(pre) = pin.strip_suffix('_') {
        let (pin, idx) = pre.rsplit_once('_').unwrap();
        let idx: usize = idx.parse().unwrap();
        (pin, Some(idx))
    } else {
        (pin, None)
    };
    if pin == "BUFGTCLKBUFGT" {
        let idx = idx.unwrap();
        if idx == 14 {
            None
        } else {
            Some(
                [
                    "REFCLK0_ODIV2",
                    "CH0_TXOUTCLK",
                    "CH1_TXOUTCLK",
                    "CH0_RXOUTCLK",
                    "CH1_RXOUTCLK",
                    "CH0_DMONITOROUTCLK",
                    "CH1_DMONITOROUTCLK",
                    "REFCLK1_ODIV2",
                    "CH2_TXOUTCLK",
                    "CH3_TXOUTCLK",
                    "CH2_RXOUTCLK",
                    "CH3_RXOUTCLK",
                    "CH2_DMONITOROUTCLK",
                    "CH3_DMONITOROUTCLK",
                ][idx]
                    .to_string(),
            )
        }
    } else if let Some(rest) = pin.strip_prefix("CH") {
        let ch: usize = rest[0..1].parse().unwrap();
        let pin = &rest[1..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "APTEXTHOLD" => "DFEHOLD",
            "APTOVERWREN" => "DFEOVRD",
            "BSRSERIAL" => "BSR_SERIAL",
            "CDRBMCDREQ" => "CDRBMCDRREQ",
            "CDRHOLD" => "RXCDRHOLD",
            "CDRLOCK" => "RXCDRLOCK",
            "CDROVREN" => "RXCDROVRDEN",
            "CDRPHRESET" => "RXCDRRESET",
            "CFOKDONE" => "RXOSINTDONE",
            "CKPINRSRVD0" => "CLKRSVD0",
            "CKPINRSRVD1" => "CLKRSVD1",
            "COMINITDET" => "RXCOMINITDET",
            "COMSASDET" => "RXCOMSASDET",
            "COMWAKEDET" => "RXCOMWAKEDET",
            "DMONCLK" => "DMONITORCLK",
            "DMONOUT" => "DMONITOROUT",
            "ENPPM" => "TXPIPPMEN",
            "PERSTB" => "PCIERSTB",
            "PHYESMADAPTATIONSAVE" => "PHYESMADAPTSAVE",
            "PINRSRVDAS" => "PINRSVDAS",
            "PINRSRVD" => "GTRSVD",
            "REFCLKPMA" => "REFDEBUGOUT",
            "RXCHICORESETMASK" => "RXPHALIGNRESETMASK",
            "RXCHISALIGNED" => "RXCHANISALIGNED",
            "RXCKCORCNT" => "RXCLKCORCNT",
            "RXDELAYALIGNERR" => "RXDLYALIGNERR",
            "RXDELAYALIGNPROGRESS" => "RXDLYALIGNPROG",
            "RXDELAYALIGNREQ" => "RXDLYALIGNREQ",
            "RXETHERNETSTATOUT" => "RX10GSTAT",
            "RXOUTPCSCLK" => "RXDEBUGPCSOUT",
            "RXPOWERDOWN" => "RXPD",
            "RXPHASEALIGNDONE" => "RXPHALIGNDONE",
            "RXPHASEALIGNERR" => "RXPHALIGNERR",
            "RXPHASEALIGNPD" => "RXPHDLYPD",
            "RXPHASEALIGNREQ" => "RXPHALIGNREQ",
            "RXPHASEDELAYRESETDONE" => "RXPHDLYRESETDONE",
            "RXPHASESETINITDONE" => "RXPHSETINITDONE",
            "RXPHASESETINITREQ" => "RXPHSETINITREQ",
            "RXPHASESHIFT180DONE" => "RXPHSHIFT180DONE",
            "RXPHASESHIFT180REQ" => "RXPHSHIFT180",
            "RXUSRRDY" => "RXUSERRDY",
            "STEPSIZEPPM" => "TXPIPPMSTEPSIZE",
            "TCOCLKFSMFROUT" => "TXPHDLYTSTCLK",
            "TXCHICOOUTRSVD" => "TXPHALIGNOUTRSVD",
            "TXCHICORESETMASK" => "TXPHALIGNRESETMASK",
            "TXDELAYALIGNERR" => "TXDLYALIGNERR",
            "TXDELAYALIGNPROGRESS" => "TXDLYALIGNPROG",
            "TXDELAYALIGNREQ" => "TXDLYALIGNREQ",
            "TXDETECTRXLOOPBACK" => "TXDETECTRX",
            "TXDRVAMP" => "TXDIFFCTRL",
            "TXEMPMAIN" => "TXMAINCURSOR",
            "TXEMPPOS" => "TXPOSTCURSOR",
            "TXEMPPRE" => "TXPRECURSOR",
            "TXETHERNETSTATTXLOCALFAULT" => "TX10GSTAT",
            "TXOUTPCSCLK" => "TXDEBUGPCSOUT",
            "TXPHASEALIGNDONE" => "TXPHALIGNDONE",
            "TXPHASEALIGNERR" => "TXPHALIGNERR",
            "TXPHASEALIGNPD" => "TXPHDLYPD",
            "TXPHASEALIGNREQ" => "TXPHALIGNREQ",
            "TXPHASEDELAYRESETDONE" => "TXPHDLYRESETDONE",
            "TXPHASESETINITDONE" => "TXPHSETINITDONE",
            "TXPHASESETINITREQ" => "TXPHSETINITREQ",
            "TXPHASESHIFT180DONE" => "TXPHSHIFT180DONE",
            "TXPHASESHIFT180REQ" => "TXPHSHIFT180",
            "TXPOWERDOWN" => "TXPD",
            "TXSERPWRDN" => "TXPISOPD",
            "TXTXPICODEOVRDEN" => "TXPICODEOVRDEN",
            "TXTXPICODERESET" => "TXPICODERESET",
            "TXUSRRDY" => "TXUSERRDY",
            "XPIPE5PIPELINEEN" => "XPIPE5_PIPELINE_EN",
            _ => pin,
        };
        Some(if let Some(idx) = idx {
            format!("CH{ch}_{pin}_{idx}_")
        } else {
            format!("CH{ch}_{pin}")
        })
    } else if let Some(pin) = pin.strip_prefix("CTRL") {
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "PRESETN" => "APB3PRESETN",
            "PENABLE" => "APB3PENABLE",
            "PWRITE" => "APB3PWRITE",
            "PSEL" => "APB3PSEL",
            "PADDR" => "APB3PADDR",
            "PWDATA" => "APB3PWDATA",
            "PREADY" => "APB3PREADY",
            "PRDATA" => "APB3PRDATA",
            "PSLVERR" => "APB3PSLVERR",
            "BGBYPASS" => "BGBYPASSB",
            "BGPWRDNB" => "BGPDB",
            "BGRCALCTL" => "BGRCALOVRD",
            "BGTESTEN" => "BGMONITORENB",
            "COEREGRST" => "COESTATUSDEBUG",
            "CORRECTABLEERR" => "CORRECTERR",
            "UNCORRECTABLEERR" => "UNCORRECTERR",
            "DEBUGTRACETREADY" => "DEBUGTRACEREADY",
            "GTRSVDIN" => "CTRLRSVDIN",
            "GTRSVDOUT" => "CTRLRSVDOUT",
            "M0AXISTDATA" => "M0_AXIS_TDATA",
            "M0AXISTLAST" => "M0_AXIS_TLAST",
            "M0AXISTREADY" => "M0_AXIS_TREADY",
            "M0AXISTVALID" => "M0_AXIS_TVALID",
            "M1AXISTDATA" => "M1_AXIS_TDATA",
            "M1AXISTLAST" => "M1_AXIS_TLAST",
            "M1AXISTREADY" => "M1_AXIS_TREADY",
            "M1AXISTVALID" => "M1_AXIS_TVALID",
            "M2AXISTDATA" => "M2_AXIS_TDATA",
            "M2AXISTLAST" => "M2_AXIS_TLAST",
            "M2AXISTREADY" => "M2_AXIS_TREADY",
            "M2AXISTVALID" => "M2_AXIS_TVALID",
            "S0AXISTDATA" => "S0_AXIS_TDATA",
            "S0AXISTLAST" => "S0_AXIS_TLAST",
            "S0AXISTREADY" => "S0_AXIS_TREADY",
            "S0AXISTVALID" => "S0_AXIS_TVALID",
            "S1AXISTDATA" => "S1_AXIS_TDATA",
            "S1AXISTLAST" => "S1_AXIS_TLAST",
            "S1AXISTREADY" => "S1_AXIS_TREADY",
            "S1AXISTVALID" => "S1_AXIS_TVALID",
            "S2AXISTDATA" => "S2_AXIS_TDATA",
            "S2AXISTLAST" => "S2_AXIS_TLAST",
            "S2AXISTREADY" => "S2_AXIS_TREADY",
            "S2AXISTVALID" => "S2_AXIS_TVALID",
            "PCIELTSSMSTATE" => "PCIELTSSM",
            "RCAL" => "RCALOUT",
            "RXMARGINREQPAYLOAD" => "RXMARGINREQPAYLD",
            "RXMARGINRESPAYLOAD" => "RXMARGINRESPAYLD",
            "UBGPI" => "GPI",
            "UBGPO" => "GPO",
            _ => pin,
        };
        Some(if let Some(idx) = idx {
            format!("{pin}_{idx}_")
        } else {
            pin.to_string()
        })
    } else if let Some(rest) = pin.strip_prefix("HS") {
        let hs: usize = rest[0..1].parse().unwrap();
        let pin = &rest[1..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "COREREFCLK0" => "LCPLLGTGREFCLK",
            "COREREFCLK1" => "RPLLGTGREFCLK",
            "LCPLLFBLOSS" => "LCPLLFBCLKLOST",
            "LCPLLFREQLOCK" => "LCPLLLOCK",
            "LCPLLPWRDN" => "LCPLLPD",
            "LCPLLREFLOSS" => "LCPLLREFCLKLOST",
            "LCPLLREFSELDYN" => "LCPLLREFCLKSEL",
            "RPLLFBLOSS" => "RPLLFBCLKLOST",
            "RPLLFREQLOCK" => "RPLLLOCK",
            "RPLLPWRDN" => "RPLLPD",
            "RPLLREFLOSS" => "RPLLREFCLKLOST",
            "RPLLREFSELDYN" => "RPLLREFCLKSEL",
            "MGTLCPLLREFCLKFA" => "LCPLLREFCLKMONITOR",
            "MGTRPLLREFCLKFA" => "RPLLREFCLKMONITOR",
            _ => pin,
        };
        Some(if let Some(idx) = idx {
            format!("HSCLK{hs}_{pin}_{idx}_")
        } else {
            format!("HSCLK{hs}_{pin}")
        })
    } else if let Some(rest) = pin.strip_prefix("REFCLK") {
        let rc: usize = rest[0..1].parse().unwrap();
        let pin = &rest[1..];
        let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
        let pin = match pin {
            "HROWTESTCK" => "CLKTESTSIG",
            "REFCLKPD" => "GTREFCLKPD",
            _ => unreachable!(),
        };
        Some(format!("REFCLK{rc}_{pin}"))
    } else if let Some(pin) = pin.strip_prefix("GTDFX") {
        let (pref, pin) = if let Some(pin) = pin
            .strip_suffix("GTCTRLFS")
            .or(pin.strip_suffix("GTCTRLSF"))
        {
            ("", pin)
        } else {
            let (pin, idx) = pin.split_at(pin.len() - 1);
            let pref = match idx {
                "0" => "CH0_",
                "1" => "CH1_",
                "2" => "CH2_",
                "3" => "CH3_",
                _ => unreachable!(),
            };
            let pin = pin.strip_suffix("FS").or(pin.strip_suffix("SF")).unwrap();
            (pref, pin)
        };
        let pin = match pin {
            "SCANCLKN" => "SCANCLKB",
            "SCANMODERSTN" => "SCANRSTB",
            "SCANENN" => "SCANENB",
            "SCANEDTUPDTN" => "EDTUPDATEB",
            "SCANCHNLIN" => "SCANIN",
            "SCANCHNLOUT" => "SCANOUT",
            "SCANCNTRLCHNLIN" => "SCANCNTRLIN",
            "SCANCNTRLCHNLOUT" => "SCANCNTRLOUT",
            _ => panic!("ummm {pin}"),
        };
        Some(if let Some(idx) = idx {
            format!("{pref}{pin}_{idx}_")
        } else {
            format!("{pref}{pin}")
        })
    } else {
        None
    }
}

pub fn verify_mmi(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let ps = bcrd.with_row(RowId::from_idx(0)).bel(bslots::PSXC);
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let has_int = edev[edev.bel_tile(bcrd)].class != tcls::MMI_NC;
    let gt = if has_int {
        bcrd.delta(0, Chip::ROWS_PER_REG as i32)
    } else {
        bcrd.cell
    }
    .bel(bslots::GTYP);

    let mut bel = vrf.verify_bel(bcrd);
    if !has_int {
        bel = bel.kind("MMI2");
    }
    for (pin, pdata) in &naming.pins {
        let &(_, dir) = pdata.orig_pin.as_ref().unwrap();
        if matches!(
            pin.as_str(),
            "EFUSEBCFMTDIS"
                | "FPDINITVCCAUXLVLSHB"
                | "FPDMMIPORN"
                | "FPXMMITOPSWCLK"
                | "GTMMIPSPLFTIDMONOUTCLK"
                | "GTPSHSDPRXOUTCLK"
                | "GTPSHSDPTXOUTCLK"
                | "HSDPPERSTN"
                | "MMIISPRESENT"
                | "MMIPSMDC"
                | "MMIPSMDIOEN"
                | "MMIPSMDIOOUT"
                | "PSAXICFGCLK"
                | "PSGEMTSURSTN"
                | "PSGTHSDPPIPECLK"
                | "PSGTHSDPRXUSRCLK"
                | "PSTSUCLK"
        ) || pin.starts_with("IFMMIPS")
            || pin.starts_with("IFPSMMI")
            || pin.starts_with("IFPSPLMMI")
            || pin.starts_with("PSPLMMI")
            || pin.starts_with("IFPSGTYP")
            || pin.starts_with("MMIRCLK")
            || pin.starts_with("PSMMI")
            || pin.starts_with("PCIE")
        {
            // PSXC
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            let pspin = if pin == "IFPSMMISOCDBGDBGLPDCLK" {
                "IFPSMMISOCDBGDBGFPDCLK"
            } else {
                pin
            };
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(ps, pspin)]);
        } else if let Some(opin) = xlat_mmi_gt_pin(pin) {
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pin);
            } else {
                bel = bel.extra_out_far(pin);
            }
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(gt, &opin)]);
        } else if pin.starts_with("UNUSEDTIEHIGH") && has_int {
            bel = bel.extra_out_far(pin);
        } else if matches!(
            pin.as_str(),
            "SYSRST1N"
                | "SYSRST2N"
                | "SYSRST3N"
                | "NPICLKEPXPIPE"
                | "PORRESETN"
                | "IFMMIGTYPBUFGTCLKBUFGT_14_"
        ) || !has_int
        {
            // dummy
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pin);
            } else {
                bel = bel.extra_out_far_claim(pin);
            }
        }
    }

    bel.commit();
}
