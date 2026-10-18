use indicatif::ProgressBar;
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{BelInfo, PinDir, SwitchBoxItem},
    dir::DirV,
    grid::{BelCoord, ColId, DieIdExt},
};
use prjcombine_re_xilinx_naming_versal::ExpandedNamedDevice;
use prjcombine_re_xilinx_rawdump::Part;
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    chip::{Chip, DisabledPart, FabricKind, HsrSKind, X5ioHNoc},
    defs::{bcls, bslots, tcls},
    expanded::ExpandedDevice,
};

mod aie;
mod cpm;
mod gt;
mod hbm;
mod noc;
mod ps;
mod x5io;
mod xpio;

fn verify_iri(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::IRI.index_of(bcrd.slot).unwrap();
    let kind = if matches!(idx, 0 | 2) {
        "IRI_QUAD_EVEN"
    } else {
        "IRI_QUAD_ODD"
    };
    vrf.verify_bel(bcrd).kind(kind).commit();
}

fn verify_bli(_edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let (idx, grp) = if let Some(idx) = bslots::BLI_GRP0.index_of(bcrd.slot) {
        (idx, 0)
    } else if let Some(idx) = bslots::BLI_GRP1.index_of(bcrd.slot) {
        (idx, 1)
    } else if let Some(idx) = bslots::BLI_GRP2.index_of(bcrd.slot) {
        (idx, 2)
    } else {
        (
            3,
            match bcrd.slot {
                bslots::BLI_GRP4 => 4,
                bslots::BLI_GRP5 => 5,
                bslots::BLI_GRP6 => 6,
                bslots::BLI_GRP7 => 7,
                _ => unreachable!(),
            },
        )
    };
    let abcd = ['A', 'B', 'C', 'D'][idx];
    vrf.verify_bel(bcrd)
        .kind(format!("BLI_{abcd}_GRP{grp}"))
        .commit();
}

fn verify_slice(vrf: &mut Verifier, bcrd: BelCoord) {
    let is_m = bcrd.slot == bslots::SLICE[1];
    let obel_s = vrf.grid.bel_delta(bcrd.cell, 0, -1, bcrd.slot);

    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far("CIN")
        .extra_out_far_claim("COUT");

    if let Some(obel) = obel_s {
        bel.verify_net(&[bel.wire_far("CIN"), bel.bel_wire_far(obel, "COUT")]);
    } else {
        bel.claim_net(&[bel.wire_far("CIN")]);
    }

    if is_m {
        bel = bel
            .kind("SLICEM")
            .extra_in_far("SRL_IN_B")
            .extra_out_far_claim("SRL_OUT_B");
        if !bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG)
            && let Some(obel) = obel_s
        {
            bel.verify_net(&[
                bel.wire_far("SRL_IN_B"),
                bel.bel_wire_far(obel, "SRL_OUT_B"),
            ]);
        } else {
            bel.claim_net(&[bel.wire_far("SRL_IN_B")]);
        }
    } else {
        bel = bel.kind("SLICEL");
    }

    bel.commit();
}

fn verify_laguna(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    bel.claim_pip(bel.wire("RX"), bel.wire("UBUMP"));
    bel.claim_pip(bel.wire("UBUMP"), bel.wire("TX"));
    if let Some(conn) = edev.sll.get(&bcrd) {
        if !conn.cursed {
            if let Some(obel) = conn.target {
                if bcrd < obel {
                    bel.claim_net(&[bel.wire("UBUMP"), bel.bel_wire(obel, "UBUMP")]);
                } else {
                    bel.verify_net(&[bel.wire("UBUMP"), bel.bel_wire(obel, "UBUMP")]);
                }
            } else {
                bel.claim_net(&[bel.wire("UBUMP")]);
            }
        }
    } else {
        bel.claim_net(&[bel.wire("UBUMP")]);
    }
}

fn verify_dsp(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::DSP.index_of(bcrd.slot).unwrap();

    let mut bel = vrf.verify_bel(bcrd).kind("DSP58_PRIMARY");
    let mut cascs = vec![];
    let obel_odsp = bcrd.bel(bslots::DSP[idx ^ 1]);
    let lr = if idx == 1 { 'R' } else { 'L' };
    for i in 0..10 {
        let pin = &format!("AD_CPLX_{i}_");
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.wire_far(&format!("CPLX_AD_CPLX_DSP{lr}_{i}_")),
        );
        bel = bel.extra_out_far_claim(format!("AD_DATA_CPLX_{i}_"));
    }
    for i in 0..18 {
        bel = bel.extra_out_far_claim(format!("A_CPLX_{i}_"));
        bel = bel.extra_out_far_claim(format!("B2B1_CPLX_{i}_"));
        bel = bel.extra_out_far_claim(format!("A_TO_D_CPLX_{i}_"));
        let pin = &format!("D_FROM_A_CPLX_{i}_");
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire_far(obel_odsp, &format!("A_TO_D_CPLX_{i}_")),
        );
    }
    for i in 0..37 {
        for uv in ['U', 'V'] {
            let pin = &format!("{uv}_CPLX_{i}_");
            bel = bel.extra_in_claim(pin);
            bel.claim_pip(bel.wire(pin), bel.wire_far(&format!("CPLX_{uv}_CPLX_{i}_")));
        }
    }
    bel = bel
        .extra_out_far_claim("CONJ_CPLX_OUT")
        .extra_in_claim("CONJ_CPLX_MULT_IN")
        .extra_in_claim("CONJ_CPLX_PREADD_IN");
    bel.claim_pip(
        bel.wire("CONJ_CPLX_MULT_IN"),
        bel.wire_far(&format!("CPLX_CONJ_DSP_{lr}_MULT_OUT")),
    );
    bel.claim_pip(
        bel.wire("CONJ_CPLX_PREADD_IN"),
        bel.wire_far(&format!("CPLX_CONJ_DSP_{lr}_PREADD_OUT")),
    );
    for i in 0..34 {
        cascs.push((format!("ACIN_{i}_"), format!("ACOUT_{i}_")));
    }
    for i in 0..32 {
        cascs.push((format!("BCIN_{i}_"), format!("BCOUT_{i}_")));
    }
    for i in 0..58 {
        cascs.push((format!("PCIN_{i}_"), format!("PCOUT_{i}_")));
    }
    cascs.push(("MULTSIGNIN".to_string(), "MULTSIGNOUT".to_string()));
    cascs.push(("CARRYCASCIN".to_string(), "CARRYCASCOUT".to_string()));

    let obel_s = edev.bel_delta(bcrd.cell, 0, -2, bcrd.slot);
    for (ipin, opin) in &cascs {
        bel = bel.extra_in_far(ipin).extra_out_far_claim(opin);
        if let Some(obel) = obel_s {
            bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
        } else {
            bel.claim_net(&[bel.wire_far(ipin)]);
        }
    }
    bel.commit();

    if idx == 0 {
        let mut bel = vrf.verify_bel(bcrd).sub(1).skip_auto().kind("DSP58_CPLX");
        let mut inps = vec![];
        let mut outps = vec![];
        let obel_dsp0 = bcrd.bel(bslots::DSP[0]);
        let obel_dsp1 = bcrd.bel(bslots::DSP[1]);
        for i in 0..10 {
            outps.push(format!("AD_CPLX_DSPL_{i}_"));
            outps.push(format!("AD_CPLX_DSPR_{i}_"));
            inps.push((
                format!("AD_DATA_CPLX_DSPL_{i}_"),
                obel_dsp0,
                format!("AD_DATA_CPLX_{i}_"),
            ));
            inps.push((
                format!("AD_DATA_CPLX_DSPR_{i}_"),
                obel_dsp1,
                format!("AD_DATA_CPLX_{i}_"),
            ));
        }
        for i in 0..18 {
            inps.push((format!("A_CPLX_L_{i}_"), obel_dsp0, format!("A_CPLX_{i}_")));
            inps.push((
                format!("B2B1_CPLX_L_{i}_"),
                obel_dsp0,
                format!("B2B1_CPLX_{i}_"),
            ));
            inps.push((
                format!("B2B1_CPLX_R_{i}_"),
                obel_dsp1,
                format!("B2B1_CPLX_{i}_"),
            ));
        }
        for i in 0..37 {
            outps.push(format!("U_CPLX_{i}_"));
            outps.push(format!("V_CPLX_{i}_"));
        }
        inps.push((
            "CONJ_DSP_L_IN".to_string(),
            obel_dsp0,
            "CONJ_CPLX_OUT".to_string(),
        ));
        inps.push((
            "CONJ_DSP_R_IN".to_string(),
            obel_dsp1,
            "CONJ_CPLX_OUT".to_string(),
        ));
        outps.push("CONJ_DSP_L_MULT_OUT".to_string());
        outps.push("CONJ_DSP_R_MULT_OUT".to_string());
        outps.push("CONJ_DSP_L_PREADD_OUT".to_string());
        outps.push("CONJ_DSP_R_PREADD_OUT".to_string());
        for out in outps {
            let cpin = &format!("CPLX_{out}");
            bel = bel.extra_out_claim(cpin);
            bel.claim_net(&[bel.wire_far(cpin)]);
            bel.claim_pip(bel.wire_far(cpin), bel.wire(cpin));
        }
        for (inp, obel, opin) in inps {
            let cpin = &format!("CPLX_{inp}");
            bel = bel.extra_in_claim(cpin);
            bel.claim_pip(bel.wire(cpin), bel.bel_wire_far(obel, &opin));
        }
        bel.commit();
    }
}

fn verify_bram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("RAMB36");
    let mut cascs = vec![];
    for ab in ['A', 'B'] {
        for i in 0..32 {
            cascs.push((format!("CASDIN{ab}_{i}_"), format!("CASDOUT{ab}_{i}_")));
        }
        for i in 0..4 {
            cascs.push((format!("CASDINP{ab}_{i}_"), format!("CASDOUTP{ab}_{i}_")));
        }
    }
    cascs.push(("CASINSBITERR".to_string(), "CASOUTSBITERR".to_string()));
    cascs.push(("CASINDBITERR".to_string(), "CASOUTDBITERR".to_string()));
    let obel_s = edev.bel_delta(bcrd.cell, 0, -4, bcrd.slot);
    for (ipin, opin) in &cascs {
        bel = bel.extra_in_far(ipin);
        bel = bel.extra_out_far_claim(opin);
        if let Some(obel) = obel_s {
            bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
        } else {
            bel.claim_net(&[bel.wire_far(ipin)]);
        }
    }
    bel.commit();

    for h in 0..2 {
        let mut bel = vrf
            .verify_bel(bcrd)
            .kind(["RAMB18_L", "RAMB18_U"][h])
            .sub(1 + h)
            .skip_auto();

        let ul = ['L', 'U'][h];

        for (pin, fpin0, fpin1) in [
            ("CLKARDCLK", "CLKAL", "CLKAU"),
            ("CLKBWRCLK", "CLKBL", "CLKBU"),
            ("ENARDEN", "ENAL", "ENAU"),
            ("ENBWREN", "ENBL", "ENBU"),
            ("REGCEAREGCE", "REGCEAL", "REGCEAU"),
            ("REGCEB", "REGCEBL", "REGCEBU"),
            ("RSTRAMARSTRAM", "RSTRAMAL", "RSTRAMAU"),
            ("RSTRAMB", "RSTRAMBL", "RSTRAMBU"),
            ("RSTREGARSTREG", "RSTREGAL", "RSTREGAU"),
            ("RSTREGB", "RSTREGBL", "RSTREGBU"),
            ("CASDOMUXA", "CASDOMUXAL", "CASDOMUXAU"),
            ("CASDOMUXB", "CASDOMUXBL", "CASDOMUXBU"),
            ("CASDOMUXEN_A", "CASDOMUXEN_AL", "CASDOMUXEN_AU"),
            ("CASDOMUXEN_B", "CASDOMUXEN_BL", "CASDOMUXEN_BU"),
            ("CASOREGIMUXA", "CASOREGIMUXAL", "CASOREGIMUXAU"),
            ("CASOREGIMUXB", "CASOREGIMUXBL", "CASOREGIMUXBU"),
            ("CASOREGIMUXEN_A", "CASOREGIMUXEN_AL", "CASOREGIMUXEN_AU"),
            ("CASOREGIMUXEN_B", "CASOREGIMUXEN_BL", "CASOREGIMUXEN_BU"),
            ("ARST_A", "ARST_A_L", "ARST_A_U"),
            ("ARST_B", "ARST_B_L", "ARST_B_U"),
            ("SLEEP", "SLEEPL", "SLEEPU"),
            ("TST_RESET_B", "TST_RESET_B_L", "TST_RESET_B_U"),
            (
                "SCAN_DATA_EXT_IN_0_",
                "SCAN_DATA_EXT_IN_0_",
                "SCAN_DATA_EXT_IN_2_",
            ),
            (
                "SCAN_DATA_EXT_IN_1_",
                "SCAN_DATA_EXT_IN_1_",
                "SCAN_DATA_EXT_IN_3_",
            ),
        ] {
            let hpin = &format!("H{h}_{pin}");
            bel = bel.extra_in_claim(hpin);
            bel.claim_pip(bel.wire(hpin), bel.wire_far([fpin0, fpin1][h]));
        }

        for (pin, fpin0, fpin1) in [
            (
                "SCAN_DATA_EXT_OUT_0_",
                "SCAN_DATA_EXT_OUT_0_",
                "SCAN_DATA_EXT_OUT_2_",
            ),
            (
                "SCAN_DATA_EXT_OUT_1_",
                "SCAN_DATA_EXT_OUT_1_",
                "SCAN_DATA_EXT_OUT_3_",
            ),
        ] {
            let hpin = &format!("H{h}_{pin}");
            bel = bel.extra_out_claim(hpin);
            bel.claim_pip(bel.wire_far([fpin0, fpin1][h]), bel.wire(hpin));
        }

        if h == 0 {
            for pin in ["SCAN_CLK_N_EXT", "SCAN_EN_N_EXT"] {
                let hpin = &format!("H{h}_{pin}");
                bel = bel.extra_in_claim(hpin);
                bel.claim_pip(bel.wire(hpin), bel.wire_far(pin));
            }
            bel = bel.extra_out_claim("H0_CSSD_STOP_DONE_EXT");
            bel.claim_pip(
                bel.wire_far("CSSD_STOP_DONE_EXT"),
                bel.wire("H0_CSSD_STOP_DONE_EXT"),
            );
        }

        for i in 0..11 {
            for (ab, abrw) in [('A', "ARD"), ('B', "BWR")] {
                let pname = &format!("ADDR{abrw}ADDR_{i}_");
                let pin = &format!("H{h}_{pname}");
                bel = bel.extra_in_claim(pin);
                bel.claim_pip(
                    bel.wire(pin),
                    bel.wire_far(&format!("ADDR{ab}{ul}_{ii}_", ii = i + 1)),
                );
            }
        }

        for (pin, opin, num) in [
            ("DINADIN", "DIA", 16),
            ("DINBDIN", "DIB", 16),
            ("DINPADINP", "DIPA", 2),
            ("DINPBDINP", "DIPB", 2),
            ("WEA", "WEA", 4),
            ("WEBWE", "WEB", 4),
        ] {
            for i in 0..num {
                let pname = &format!("{pin}_{i}_");
                let hpin = &format!("H{h}_{pname}");
                bel = bel.extra_in_claim(hpin);
                bel.claim_pip(bel.wire(hpin), bel.wire_far(&format!("{opin}{ul}_{i}_")));
            }
        }

        for (pin, opin, num) in [
            ("DOUTADOUT", "DOA", 16),
            ("DOUTBDOUT", "DOB", 16),
            ("DOUTPADOUTP", "DOPA", 2),
            ("DOUTPBDOUTP", "DOPB", 2),
        ] {
            for i in 0..num {
                let pname = &format!("{pin}_{i}_");
                let hpin = &format!("H{h}_{pname}");
                bel = bel.extra_out_claim(hpin);
                bel.claim_pip(bel.wire_far(&format!("{opin}{ul}_{i}_")), bel.wire(hpin));
            }
        }

        for (pin, num) in [("CASDOUT", 16), ("CASDOUTP", 2)] {
            for ab in ['A', 'B'] {
                for i in 0..num {
                    let ii = i * 2 + h;
                    let opin = &format!("H{h}_{pin}{ab}_{i}_");
                    let opin_f = &format!("{pin}{ab}_{ii}_");
                    bel = bel.extra_out_claim(opin);
                    bel.claim_pip(bel.wire_far(opin_f), bel.wire(opin));
                }
            }
        }
        for (pin, num) in [("CASDIN", 16), ("CASDINP", 2)] {
            for ab in ['A', 'B'] {
                for i in 0..num {
                    let ii = i * 2 + h;
                    let opin = &format!("H{h}_{pin}{ab}_{i}_");
                    let opin_f = &format!("{pin}{ab}_{ii}_");
                    bel = bel.extra_in_claim(opin);
                    bel.claim_pip(bel.wire(opin), bel.wire_far(opin_f));
                }
            }
        }

        bel.commit();
    }
}

fn verify_uram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut cascs = vec![];
    for ab in ['A', 'B'] {
        for i in 0..72 {
            cascs.push((
                format!("CAS_IN_DIN_{ab}_{i}_"),
                format!("CAS_OUT_DIN_{ab}_{i}_"),
            ));
            cascs.push((
                format!("CAS_IN_DOUT_{ab}_{i}_"),
                format!("CAS_OUT_DOUT_{ab}_{i}_"),
            ));
        }
        for i in 0..26 {
            cascs.push((
                format!("CAS_IN_ADDR_{ab}_{i}_"),
                format!("CAS_OUT_ADDR_{ab}_{i}_"),
            ));
        }
        for i in 0..9 {
            cascs.push((
                format!("CAS_IN_BWE_{ab}_{i}_"),
                format!("CAS_OUT_BWE_{ab}_{i}_"),
            ));
        }
        for n in ["EN", "SBITERR", "DBITERR", "RDACCESS", "RDB_WR"] {
            cascs.push((format!("CAS_IN_{n}_{ab}"), format!("CAS_OUT_{n}_{ab}")));
        }
    }
    let mut bel = vrf.verify_bel(bcrd).kind("URAM288");
    let is_delay = edev.is_reg_n(bcrd.die, edev.row_to_reg(bcrd.row))
        && bcrd.row.to_idx() % Chip::ROWS_PER_REG == 44;

    for (ipin, opin) in &cascs {
        bel = bel.extra_in_far(ipin).extra_out_far_claim(opin);
    }

    if let Some(obel_s) = edev.bel_delta(bcrd.cell, 0, -4, bcrd.slot) {
        let prev_delay = edev.is_reg_n(obel_s.die, edev.row_to_reg(obel_s.row))
            && obel_s.row.to_idx() % Chip::ROWS_PER_REG == 44;

        for (ipin, opin) in &cascs {
            if !prev_delay {
                bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel_s, opin)]);
            } else {
                bel.verify_net(&[
                    bel.wire_far(ipin),
                    bel.bel_wire_far(obel_s, &format!("CAS_DLY_{opin}")),
                ]);
            }
        }
    } else {
        for (ipin, _) in &cascs {
            bel.claim_net(&[bel.wire_far(ipin)]);
        }
    }

    bel.commit();

    if is_delay {
        let mut bel = vrf.verify_bel(bcrd).sub(1).kind("URAM_CAS_DLY").skip_auto();
        for (ipin, opin) in &cascs {
            let cipin = &format!("CAS_DLY_{ipin}");
            let copin = &format!("CAS_DLY_{opin}");
            bel = bel.extra_in_claim(cipin).extra_out_claim(copin);
            bel.claim_net(&[bel.wire_far(copin)]);
            bel.claim_pip(bel.wire_far(copin), bel.wire(copin));
            bel.claim_pip(bel.wire(cipin), bel.wire_far(cipin));
            bel.verify_net(&[bel.wire_far(cipin), bel.wire_far(opin)]);
        }
        bel.commit();
    }
}

fn verify_hardip(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord, kind: &str) {
    if !edev.disabled.contains(&DisabledPart::HardIpSite(
        bcrd.die,
        bcrd.col,
        edev.row_to_reg(bcrd.row),
    )) {
        let mut bel = vrf.verify_bel(bcrd).kind(kind);
        if bcrd.slot == bslots::PCIE4 {
            for i in 0..57 {
                bel = bel.extra_out_far(format!("XILUNCONNOUT_{i}_"));
            }
        }
        if bcrd.slot == bslots::PCIE5 {
            for i in 0..22 {
                bel = bel.extra_out_far(format!("XILUNCONNOUT_{i}_"));
            }
        }
        bel.commit();
    }
}

fn verify_bufdiv_leaf(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(if chip.fabric == FabricKind::Ulvt {
            "BUFDIV_LEAF_ULVT"
        } else {
            "BUFDIV_LEAF"
        })
        .extra_out_far_claim("O_CASC");

    let idx = bslots::BUFDIV_LEAF.index_of(bcrd.slot).unwrap();
    if idx != 0 {
        bel = bel.extra_in_claim("I_CASC");
        let obel = bcrd.bel(bslots::BUFDIV_LEAF[idx - 1]);
        bel.claim_pip(bel.wire("I_CASC"), bel.bel_wire_far(obel, "O_CASC"));
    }

    bel.commit();
}

fn verify_hdiob(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::HDIOB.index_of(bcrd.slot).unwrap();
    if !idx.is_multiple_of(2) {
        return;
    }
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("RXOUT_S")
        .extra_in_claim("OP_M")
        .extra_in_claim("OP_S")
        .extra_in_claim("TRISTATE_M")
        .extra_in_claim("TRISTATE_S")
        .extra_in("IBUF_DISABLE_S")
        .extra_in("KEEPER_EN_S")
        .extra_in("PD_EN_S")
        .extra_in("PU_EN_S")
        .extra_in("DYNAMIC_DCI_TS_S");

    if !matches!(idx, 10 | 12) {
        bel = bel.extra_out_far_claim("RXOUT_M")
    } else {
        bel = bel.skip_out(bcls::HDIOB::CLKPAD).extra_out("RXOUT_M");
    }

    let obel = bcrd.bel(bslots::HDIOLOGIC_SCAN[idx / 2]);
    for (ipin, opin) in [
        ("OP_M", "OPFFM_Q"),
        ("OP_S", "OPFFS_Q"),
        ("TRISTATE_M", "TFFM_Q"),
        ("TRISTATE_S", "TFFS_Q"),
    ] {
        bel.claim_pip(bel.wire(ipin), bel.bel_wire_far(obel, opin));
    }

    bel.commit();
}

fn verify_hdiologic(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::HDIOLOGIC_SCAN.index_of(bcrd.slot).unwrap();
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HDIOLOGIC")
        .extra_out_far_claim("OPFFM_Q")
        .extra_out_far_claim("OPFFS_Q")
        .extra_out_far_claim("TFFM_Q")
        .extra_out_far_claim("TFFS_Q")
        .extra_in_claim("IPFFM_D")
        .extra_in_claim("IPFFS_D");

    let obel = bcrd.bel(bslots::HDIOB[idx * 2]);
    for (ipin, opin) in [("IPFFM_D", "RXOUT_M"), ("IPFFS_D", "RXOUT_S")] {
        bel.claim_pip(bel.wire(ipin), bel.bel_wire_far(obel, opin));
    }

    for slot in [bslots::HDIOLOGIC[idx * 2], bslots::HDIOLOGIC[idx * 2 + 1]] {
        let obel = bcrd.bel(slot);
        let naming = bel.vrf.ngrid.get_bel_naming(obel);
        for pin in naming.pins.values() {
            if let Some((ref pname, pdir)) = pin.orig_pin {
                if pdir == PinDir::Input {
                    bel = bel.extra_in(pname);
                } else {
                    bel = bel.extra_out(pname);
                }
            }
        }
    }

    bel.commit();
}

fn verify_sysmon_sat_hsr(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    if tcid == tcls::SYSMON_SAT_HSR_NC {
        for (pname, pin) in &naming.pins {
            if let Some((_, pdir)) = pin.orig_pin {
                if pdir == PinDir::Input {
                    bel = bel.extra_in_far_claim(pname);
                } else {
                    bel = bel.extra_out_far_claim(pname);
                }
            }
        }
    } else {
        bel = bel.extra_out_far_claim("DATAREADY");
        for i in 0..16 {
            bel = bel
                .extra_out_far_claim(format!("PVALUE_{i}_"))
                .extra_out_far_claim(format!("VVALUE_{i}_"))
                .extra_out_far_claim(format!("TVALUE_{i}_"));
        }
        for i in 0..8 {
            bel = bel.extra_out_far_claim(format!("ALARM_{i}_"));
        }
    }
    bel.commit();
}

fn verify_dpll(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    if edev[tcrd].class == tcls::HDIO {
        let reg = edev.row_to_reg(bcrd.row);
        if edev
            .disabled
            .contains(&DisabledPart::HdioDpll(bcrd.die, bcrd.col, reg))
        {
            return;
        }
    }
    let mut bel = vrf.verify_bel(bcrd);
    if edev[tcrd].class == tcls::DPLL_X5IO_NC {
        for (pname, pin) in &naming.pins {
            if !matches!(
                pname.as_str(),
                "CLKIN"
                    | "CLKIN_DESKEW"
                    | "CLKOUT0"
                    | "CLKOUT1"
                    | "CLKOUT2"
                    | "CLKOUT3"
                    | "TMUXOUT"
            ) && let Some((_, pdir)) = pin.orig_pin
            {
                if pdir == PinDir::Input {
                    bel = bel.extra_in_far_claim(pname);
                } else {
                    bel = bel.extra_out_far_claim(pname);
                }
            }
        }
    }
    bel.commit();
}

fn verify_vcu2(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far("NSU_VCU2")
        .extra_out_far("VCU2NOCSCANCLKN")
        .extra_out_far("VCU2NOCSCANCSSDTRIGN")
        .extra_out_far("VCU2NOCSCANEDTUPDTN")
        .extra_out_far("VCU2NOCSCANENN")
        .extra_out_far("VCU2NOCSCANMODERSTN");
    bel.verify_net(&[
        bel.wire_far("NSU_VCU2"),
        bel.bel_wire_far(bcrd.bel(bslots::VCU2_NSU), "AXI_OUT"),
    ]);
    for i in 0..4 {
        let pin = &format!("NMU_VCU2_{i}");
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[
            bel.wire_far(pin),
            bel.bel_wire_far(bcrd.bel(bslots::VCU2_NMU[i]), "AXI_IN"),
        ]);
    }
    let scan = bcrd.bel(bslots::VCU2_NOC_SCAN);
    for i in 0..12 {
        let pin_in = &format!("VCU2NOCSCANCHNLOUT_{i}_");
        let pin_out = &format!("NOCVCU2SCANCHNLIN_{i}_");
        let pin_mask = &format!("VCU2NOCSCANCHNLMASKOUT_{i}_");
        bel = bel
            .extra_in_far(pin_out)
            .extra_out_far(pin_mask)
            .extra_out_far(pin_in);
        let opin_in = &format!("NOC2_SCAN_CHNL_FROM_PL_{i}_");
        let opin_out = &format!("NOC2_SCAN_CHNL_TO_PL_{i}_");
        let opin_mask = &format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_");
        bel.verify_net(&[bel.wire_far(pin_in), bel.bel_wire_far(scan, opin_in)]);
        bel.verify_net(&[bel.wire_far(pin_mask), bel.bel_wire_far(scan, opin_mask)]);
        bel.verify_net(&[bel.wire_far(pin_out), bel.bel_wire_far(scan, opin_out)]);
    }
    for (pin, opin) in [
        ("VCU2NOCSCANCLKN", "IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N"),
        (
            "VCU2NOCSCANCSSDTRIGN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_CSSD_TRIG_N",
        ),
        (
            "VCU2NOCSCANEDTUPDTN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_EDT_UPDT_N",
        ),
        ("VCU2NOCSCANENN", "IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N"),
        (
            "VCU2NOCSCANMODERSTN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_MODE_RST_N",
        ),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(scan, opin)]);
    }
    bel.commit();
}

fn verify_isp2(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far("NSU_ISP")
        .extra_out_far("ISP2NOCSCANCLKN")
        .extra_out_far("ISP2NOCSCANCSSDTRIGN")
        .extra_out_far("ISP2NOCSCANEDTUPDTN")
        .extra_out_far("ISP2NOCSCANENN")
        .extra_out_far("ISP2NOCSCANMODERSTN");
    bel.verify_net(&[
        bel.wire_far("NSU_ISP"),
        bel.bel_wire_far(bcrd.bel(bslots::VCU2_NSU), "AXI_OUT"),
    ]);
    for i in 0..2 {
        let pin = &format!("NMU_ISP_{i}");
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[
            bel.wire_far(pin),
            bel.bel_wire_far(bcrd.bel(bslots::VCU2_NMU[i]), "AXI_IN"),
        ]);
    }
    let scan = bcrd.bel(bslots::VCU2_NOC_SCAN);
    for i in 0..7 {
        let pin_in = &format!("ISP2NOCSCANCHNLOUT_{i}_");
        let pin_out = &format!("NOCISP2SCANCHNLIN_{i}_");
        let pin_mask = &format!("ISP2NOCSCANCHNLMASKOUT_{i}_");
        bel = bel
            .extra_in_far(pin_out)
            .extra_out_far(pin_mask)
            .extra_out_far(pin_in);
        let opin_in = &format!("NOC2_SCAN_CHNL_FROM_PL_{i}_");
        let opin_out = &format!("NOC2_SCAN_CHNL_TO_PL_{i}_");
        let opin_mask = &format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_");
        bel.verify_net(&[bel.wire_far(pin_in), bel.bel_wire_far(scan, opin_in)]);
        bel.verify_net(&[bel.wire_far(pin_mask), bel.bel_wire_far(scan, opin_mask)]);
        bel.verify_net(&[bel.wire_far(pin_out), bel.bel_wire_far(scan, opin_out)]);
    }
    for (pin, opin) in [
        ("ISP2NOCSCANCLKN", "IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N"),
        (
            "ISP2NOCSCANCSSDTRIGN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_CSSD_TRIG_N",
        ),
        (
            "ISP2NOCSCANEDTUPDTN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_EDT_UPDT_N",
        ),
        ("ISP2NOCSCANENN", "IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N"),
        (
            "ISP2NOCSCANMODERSTN",
            "IF_NOC2_SCAN_CNTL_IN_SCAN_MODE_RST_N",
        ),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(scan, opin)]);
    }
    bel.commit();
}

fn verify_bufgctrl(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::CLK_XPIO_MIN | tcls::CLK_X5IO_MIN_A | tcls::CLK_X5IO_MIN_B
    ) {
        bel = bel
            .extra_in_far_claim("CE0")
            .extra_in_far_claim("CE1")
            .extra_in_far_claim("SEL0")
            .extra_in_far_claim("SEL1")
            .extra_in_far_claim("IGNORE0")
            .extra_in_far_claim("IGNORE1");
    }
    if bel.vrf.ngrid.get_bel_name(bcrd).is_some() {
        bel.commit();
    }
}

fn verify_bufgce_div(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::CLK_XPIO_MIN | tcls::CLK_X5IO_MIN_A | tcls::CLK_X5IO_MIN_B
    ) {
        bel = bel.extra_in_far_claim("CE").extra_in_far_claim("CLR");
    }
    if bel.vrf.ngrid.get_bel_name(bcrd).is_some() {
        bel.commit();
    }
}

fn verify_gclk_pd_h(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::RCLK_SPLITTER | tcls::RCLK_SPLITTER_VR
    ) {
        bel = bel
            .extra_in_far_claim("CLK_IN0")
            .extra_in_far_claim("CLK_IN1")
            .extra_out_far_claim("PD_OUT");
    }
    bel.commit();
}

fn verify_gclk_delay(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_gt = !edev.chips[bcrd.die].columns[bcrd.col].kind.is_vnoc();
    let kind = match edev.chips[bcrd.die].fabric {
        FabricKind::Orig | FabricKind::Rev2 => "GCLK_DELAY",
        FabricKind::Ssit => "GCLK_DELAY_SSIT",
        FabricKind::Ssit2 | FabricKind::Ulvt => "GCLK_DELAY_SSIT2",
    };
    let is_ssit2 = matches!(
        edev.chips[bcrd.die].fabric,
        FabricKind::Ssit2 | FabricKind::Ulvt
    );
    let idx = bslots::GCLK_DELAY.index_of(bcrd.slot).unwrap();
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_in_claim("EN_TAPS")
        .extra_in_claim("LOAD")
        .extra_in_claim("INC")
        .extra_in_claim("DEC");
    if is_gt {
        bel = bel.extra_in_far_claim("PD_LEFT");
    }
    if edev[tcrd].class == tcls::RCLK_GT_CPM6 {
        bel = bel
            .extra_in_far_claim("PD_RIGHT")
            .extra_in_far_claim("PD_BOT")
            .extra_in_far_claim("PD_TOP");
    }
    let obel_dec = if edev.has_bel(bcrd.bel(bslots::GCLK_TAPS_DECODE_VNOC)) {
        bcrd.bel(bslots::GCLK_TAPS_DECODE_VNOC)
    } else {
        bcrd.bel(bslots::GCLK_TAPS_DECODE_GT)
    };
    if matches!(idx, 23 | 47) {
        bel = bel.extra_out("LOCK");
        bel.verify_net(&[
            bel.wire("LOCK"),
            bel.bel_wire(
                obel_dec,
                if obel_dec.slot == bslots::GCLK_TAPS_DECODE_GT {
                    "LOCK"
                } else if idx < 24 {
                    "LOCK_L"
                } else {
                    "LOCK_R"
                },
            ),
        ]);
        for i in 0..9 {
            bel = bel.extra_out(format!("TAPS_{i}_"));
            bel.verify_net(&[
                bel.wire(&format!("TAPS_{i}_")),
                bel.bel_wire(
                    obel_dec,
                    &if obel_dec.slot == bslots::GCLK_TAPS_DECODE_GT {
                        format!("TAPS_{i}_")
                    } else if idx < 24 {
                        format!("TAPS_L_{i}_")
                    } else {
                        format!("TAPS_R_{i}_")
                    },
                ),
            ]);
        }
    } else {
        bel = bel.extra_out_far_claim("LOCK");
        for i in 0..9 {
            bel = bel.extra_out_far_claim(format!("TAPS_{i}_"));
        }
    }
    if is_ssit2 {
        for i in 9..12 {
            bel = bel.extra_out_far_claim(format!("TAPS_{i}_"));
        }
    }
    let num_taps = if is_ssit2 { 12 } else { 9 };
    if matches!(idx, 0 | 24) {
        bel = bel.extra_in_far_claim("LOCK_IN");
        for i in 0..num_taps {
            bel = bel.extra_in_far_claim(format!("TAPS_IN_{i}_"));
        }
    } else {
        bel = bel.extra_in_claim("LOCK_IN");
        bel.claim_pip(
            bel.wire("LOCK_IN"),
            bel.bel_wire_far(bcrd.bel(bslots::GCLK_DELAY[idx - 1]), "LOCK"),
        );
        for i in 0..num_taps {
            bel = bel.extra_in_claim(format!("TAPS_IN_{i}_"));
            bel.claim_pip(
                bel.wire(&format!("TAPS_IN_{i}_")),
                bel.bel_wire_far(bcrd.bel(bslots::GCLK_DELAY[idx - 1]), &format!("TAPS_{i}_")),
            );
        }
    }

    for pin in ["INC", "DEC"] {
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_dec, pin));
    }
    bel.claim_pip(
        bel.wire("LOAD"),
        bel.bel_wire_far(
            obel_dec,
            if obel_dec.slot == bslots::GCLK_TAPS_DECODE_GT {
                "LOAD_EN"
            } else if idx < 24 {
                "LOAD_EN_L"
            } else {
                "LOAD_EN_R"
            },
        ),
    );
    bel.claim_pip(
        bel.wire("EN_TAPS"),
        bel.bel_wire_far(
            obel_dec,
            &if obel_dec.slot == bslots::GCLK_TAPS_DECODE_GT {
                format!("EN_TAPS_{idx}_")
            } else if idx < 24 {
                format!("EN_TAPS_L_{idx}_")
            } else {
                format!("EN_TAPS_R_{i}_", i = idx - 24)
            },
        ),
    );

    bel.commit();
}

fn verify_gclk_taps_decode_vnoc(vrf: &mut Verifier, bcrd: BelCoord) {
    let is_ps = vrf.ngrid.get_bel_name(bcrd).unwrap().contains("_PS");
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(if is_ps {
            "GCLK_TAPS_DECODE_VNOC_PS"
        } else {
            "GCLK_TAPS_DECODE_VNOC"
        })
        .skip_in(bcls::GCLK_TAPS_DECODE_VNOC::INC)
        .skip_in(bcls::GCLK_TAPS_DECODE_VNOC::DEC)
        .extra_out_far_claim("LOAD_EN_L")
        .extra_out_far_claim("LOAD_EN_R")
        .skip_out(bcls::GCLK_TAPS_DECODE_VNOC::LOCK_L)
        .skip_out(bcls::GCLK_TAPS_DECODE_VNOC::LOCK_R);
    for i in 0..9 {
        bel = bel
            .skip_out(bcls::GCLK_TAPS_DECODE_VNOC::TAPS_L[i])
            .skip_out(bcls::GCLK_TAPS_DECODE_VNOC::TAPS_R[i]);
    }
    for i in 0..24 {
        bel = bel
            .extra_out_far_claim(format!("EN_TAPS_L_{i}_"))
            .extra_out_far_claim(format!("EN_TAPS_R_{i}_"));
    }
    bel.commit();
}

fn verify_gclk_taps_decode_gt(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("GCLK_TAPS_DECODE_GT")
        .skip_in(bcls::GCLK_TAPS_DECODE_GT::INC)
        .skip_in(bcls::GCLK_TAPS_DECODE_GT::DEC)
        .extra_out_far_claim("LOAD_EN")
        .skip_out(bcls::GCLK_TAPS_DECODE_GT::LOCK);
    for i in 0..9 {
        bel = bel.skip_out(bcls::GCLK_TAPS_DECODE_GT::TAPS[i]);
    }
    for i in 0..24 {
        bel = bel.extra_out_far_claim(format!("EN_TAPS_{i}_"));
    }
    bel.commit();
}

fn verify_bel(endev: &ExpandedNamedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let edev = endev.edev;
    match bcrd.slot {
        bslots::INT
        | bslots::INTF_INT
        | bslots::INTF_TESTMUX
        | bslots::INTF_TEST_INT
        | bslots::CLE_BC_INT
        | bslots::RCLK_INT
        | bslots::BEL_INT
        | bslots::RCLK_INTF_INT
        | bslots::RCLK_BEL_INT
        | bslots::CLK_INT
        | bslots::CLK_V_BUF
        | bslots::BLI_MUX
        | bslots::HSR_CLK
        | bslots::HSR_CLK_TEST
        | bslots::HSR_CLK_IN
        | bslots::XPLL_INT
        | bslots::XPLL_GASKET_INT => (),
        _ if bslots::IRI.contains(bcrd.slot) => verify_iri(vrf, bcrd),
        _ if bslots::BLI_GRP0.contains(bcrd.slot)
            || bslots::BLI_GRP1.contains(bcrd.slot)
            || bslots::BLI_GRP2.contains(bcrd.slot) =>
        {
            verify_bli(edev, vrf, bcrd)
        }
        bslots::BLI_GRP4 | bslots::BLI_GRP5 | bslots::BLI_GRP6 | bslots::BLI_GRP7 => {
            verify_bli(edev, vrf, bcrd)
        }
        bslots::BLI_TMR => vrf.verify_bel(bcrd).commit(),
        _ if bslots::SLICE.contains(bcrd.slot) => verify_slice(vrf, bcrd),
        _ if bslots::LAGUNA.contains(bcrd.slot) => verify_laguna(edev, vrf, bcrd),
        _ if bslots::DSP.contains(bcrd.slot) => verify_dsp(edev, vrf, bcrd),
        bslots::BRAM => verify_bram(edev, vrf, bcrd),
        bslots::URAM => verify_uram(edev, vrf, bcrd),
        bslots::PCIE4 => verify_hardip(edev, vrf, bcrd, "PCIE40"),
        bslots::PCIE5 => verify_hardip(edev, vrf, bcrd, "PCIE50"),
        bslots::MRMAC => verify_hardip(edev, vrf, bcrd, "MRMAC"),
        bslots::DCMAC => verify_hardip(edev, vrf, bcrd, "DCMAC"),
        bslots::ILKN => verify_hardip(edev, vrf, bcrd, "ILKNF"),
        bslots::HSC => verify_hardip(edev, vrf, bcrd, "HSC"),
        bslots::SDFEC => verify_hardip(edev, vrf, bcrd, "SDFEC_A"),
        bslots::DFE_CFC => verify_hardip(
            edev,
            vrf,
            bcrd,
            if edev.is_reg_n(bcrd.die, edev.row_to_reg(bcrd.row)) {
                "DFE_CFC_TOP"
            } else {
                "DFE_CFC_BOT"
            },
        ),
        bslots::RCLK_DFX_TEST => vrf.verify_bel(bcrd).commit(),
        bslots::SYSMON_SAT_VNOC | bslots::SYSMON_SAT_GT => vrf.verify_bel(bcrd).commit(),
        bslots::SYSMON_SAT_HSR => verify_sysmon_sat_hsr(edev, vrf, bcrd),
        _ if bslots::SYSMON_SAT_HBM.contains(bcrd.slot) => verify_sysmon_sat_hsr(edev, vrf, bcrd),
        bslots::DPLL_HDIO | bslots::DPLL_GT | bslots::DPLL_HSR => verify_dpll(edev, vrf, bcrd),
        bslots::MMCM => vrf.verify_bel(bcrd).commit(),
        bslots::VNOC_NSU512 | bslots::VNOC_NMU512 | bslots::HNOC_NSU512 => {
            noc::verify_vnoc_nxu512(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_NMU512.contains(bcrd.slot) => noc::verify_vnoc_nxu512(edev, vrf, bcrd),
        bslots::VNOC_NPS_NSU | bslots::VNOC_NPS_NMU => noc::verify_noc_nps(edev, vrf, bcrd),
        bslots::VNOC_NPS6X_NSU | bslots::VNOC_NPS6X_NMU => noc::verify_noc_nps6(edev, vrf, bcrd),
        bslots::VNOC_SCAN => noc::verify_noc_scan(edev, vrf, bcrd),
        bslots::HDIO_BIAS | bslots::RPI_HD_APB | bslots::HDLOGIC_APB | bslots::MISR => {
            vrf.verify_bel(bcrd).commit()
        }
        bslots::PS9 => ps::verify_ps9(edev, vrf, bcrd),
        bslots::CPM4 => cpm::verify_cpm4(vrf, bcrd),
        bslots::CPM5 => cpm::verify_cpm5(vrf, bcrd),
        bslots::CPM6 => cpm::verify_cpm6(vrf, bcrd),
        bslots::PSXL => ps::verify_psxl(edev, vrf, bcrd),
        bslots::CPM5N => cpm::verify_cpm5n(edev, vrf, bcrd),
        bslots::PSXC => ps::verify_psxc(edev, vrf, bcrd),
        bslots::PSXT => ps::verify_psxt(edev, vrf, bcrd),
        bslots::MMI => cpm::verify_mmi(edev, vrf, bcrd),
        bslots::HNICX => gt::verify_hnicx(edev, vrf, bcrd),
        bslots::HNOC_NPI_NIR => ps::verify_npi_nir(edev, vrf, bcrd),
        bslots::XRAM => ps::verify_xram(edev, vrf, bcrd),
        bslots::GTY | bslots::GTYP | bslots::GTM | bslots::GTM2 => gt::verify_gt(edev, vrf, bcrd),
        bslots::XPIPE | bslots::HNICPIPE => (),
        _ if bslots::RFADC.contains(bcrd.slot) || bslots::RFDAC.contains(bcrd.slot) => {
            gt::verify_rfadc_rfdac(endev, vrf, bcrd)
        }
        bslots::BFR_B | bslots::VDU => vrf.verify_bel(bcrd).commit(),
        bslots::VCU2 => verify_vcu2(vrf, bcrd),
        bslots::ISP2 => verify_isp2(vrf, bcrd),
        bslots::VCU2_NOC_SCAN => noc::verify_noc_scan(edev, vrf, bcrd),
        _ if bslots::VCU2_NMU.contains(bcrd.slot) => noc::verify_vcu2_nmu(edev, vrf, bcrd),
        _ if bslots::VCU2_NPS6X.contains(bcrd.slot) || bslots::HNOC_NPS6X.contains(bcrd.slot) => {
            noc::verify_noc_nps6(edev, vrf, bcrd)
        }
        bslots::VCU2_NSU => noc::verify_vcu2_nsu(edev, vrf, bcrd),
        _ if bslots::BUFDIV_LEAF.contains(bcrd.slot) => verify_bufdiv_leaf(edev, vrf, bcrd),
        _ if bslots::HDIOB.contains(bcrd.slot) => verify_hdiob(vrf, bcrd),
        _ if bslots::HDIOLOGIC.contains(bcrd.slot) => (),
        _ if bslots::HDIOLOGIC_SCAN.contains(bcrd.slot) => verify_hdiologic(vrf, bcrd),
        _ if bslots::BUFGCE_HDIO.contains(bcrd.slot) => {
            vrf.verify_bel(bcrd).kind("BUFGCE_HDIO").commit()
        }
        _ if bslots::BUFGCE.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFGCTRL.contains(bcrd.slot) => verify_bufgctrl(edev, vrf, bcrd),
        _ if bslots::BUFGCE_DIV.contains(bcrd.slot) => verify_bufgce_div(edev, vrf, bcrd),
        _ if bslots::BUFG_HSR_TEST.contains(bcrd.slot) => {
            let kind = if vrf
                .ngrid
                .get_bel_name(bcrd)
                .unwrap()
                .starts_with("BUFG_HSR_TEST_ULVT")
            {
                "BUFG_HSR_TEST_ULVT"
            } else {
                "BUFG_HSR_TEST"
            };
            vrf.verify_bel(bcrd).kind(kind).commit()
        }
        _ if bslots::BUFG_HBM.contains(bcrd.slot) || bslots::BUFG_PS.contains(bcrd.slot) => {
            let chip = edev.chips[bcrd.die];
            let kind = if chip.fabric == FabricKind::Ulvt {
                "BUFG_PS_ULVT"
            } else {
                "BUFG_PS"
            };
            vrf.verify_bel(bcrd).kind(kind).commit()
        }
        _ if bslots::GCLK_PD_H.contains(bcrd.slot) => verify_gclk_pd_h(edev, vrf, bcrd),
        _ if bslots::GCLK_PD_V.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFG_FABRIC.contains(bcrd.slot) => vrf
            .verify_bel(bcrd)
            .kind(if edev.chips[bcrd.die].fabric == FabricKind::Ulvt {
                "BUFG_FABRIC_ULVT"
            } else {
                "BUFG_FABRIC"
            })
            .commit(),
        _ if bslots::GCLK_TEST.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR_N.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR_LEV1.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR_LEV1_N.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR_LEV2.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_VDISTR_LEV2_N.contains(bcrd.slot) =>
        {
            vrf.verify_bel(bcrd)
                .kind(if edev.chips[bcrd.die].fabric == FabricKind::Ulvt {
                    "GCLK_TEST_ULVT"
                } else {
                    "GCLK_TEST"
                })
                .commit()
        }
        _ if bslots::GCLK_TEST_HSR.contains(bcrd.slot) => {
            vrf.verify_bel(bcrd).kind("GCLK_TEST").commit()
        }
        _ if bslots::BUFG_GT.contains(bcrd.slot) => vrf
            .verify_bel(bcrd)
            .kind(if edev.chips[bcrd.die].fabric == FabricKind::Ulvt {
                "BUFG_GT_ULVT"
            } else {
                "BUFG_GT"
            })
            .commit(),
        _ if bslots::BUFG_GT_SYNC.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::GCLK_TEST_RING_HDISTR.contains(bcrd.slot)
            || bslots::GCLK_TEST_RING_HROUTE.contains(bcrd.slot) =>
        {
            vrf.verify_bel(bcrd).kind("GCLK_TEST_RING").commit()
        }

        _ if bslots::GCLK_DELAY.contains(bcrd.slot) => verify_gclk_delay(edev, vrf, bcrd),
        bslots::GCLK_TAPS_DECODE_VNOC => verify_gclk_taps_decode_vnoc(vrf, bcrd),
        bslots::GCLK_TAPS_DECODE_GT => verify_gclk_taps_decode_gt(vrf, bcrd),

        _ if bslots::HNOC_PS_NMU.contains(bcrd.slot) || bslots::HNOC_ME_NMU.contains(bcrd.slot) => {
            noc::verify_noc_nmu128(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_PS_NSU.contains(bcrd.slot) || bslots::HNOC_ME_NSU.contains(bcrd.slot) => {
            noc::verify_noc_nsu128(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_NCRB.contains(bcrd.slot) || bslots::HNOC_ME2_NCRB.contains(bcrd.slot) => {
            noc::verify_noc_ncrb(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_NIDB_V.contains(bcrd.slot) => noc::verify_noc_nidb(edev, vrf, bcrd),
        bslots::HNOC_NIDB_H => noc::verify_noc_nidb(edev, vrf, bcrd),
        _ if bslots::HNOC_NPS.contains(bcrd.slot) || bslots::HNOC_NPS_MID.contains(bcrd.slot) => {
            noc::verify_noc_nps(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_NPP_RPTR.contains(bcrd.slot) => noc::verify_noc_npp_rptr(edev, vrf, bcrd),
        _ if bslots::HNOC_SCAN.contains(bcrd.slot) => noc::verify_noc_scan(edev, vrf, bcrd),
        bslots::HNOC_NCRB_SCAN => noc::verify_noc_scan(edev, vrf, bcrd),
        bslots::HNOC_NIC_AXI_NMU => noc::verify_noc2_nmu256(edev, vrf, bcrd),
        bslots::HNOC_NIC_AXI_NSU => noc::verify_noc2_nsu256(edev, vrf, bcrd),
        _ if bslots::HNOC_NIC_AXIS_NMU.contains(bcrd.slot) => {
            noc::verify_noc2_nmu256(edev, vrf, bcrd)
        }
        _ if bslots::HNOC_NIC_AXIS_NSU.contains(bcrd.slot) => {
            noc::verify_noc2_nsu256(edev, vrf, bcrd)
        }
        bslots::HNOC_NIC_XBR2X4 => noc::verify_noc_xbr2x4(edev, vrf, bcrd),
        bslots::HNOC_XBR4X2 => noc::verify_noc_xbr4x2(edev, vrf, bcrd),

        _ if bslots::XPLL.contains(bcrd.slot) => xpio::verify_xpll(edev, vrf, bcrd),
        _ if bslots::XPIOB.contains(bcrd.slot) => xpio::verify_xpiob(vrf, bcrd),
        _ if bslots::XPIOLOGIC.contains(bcrd.slot) => xpio::verify_xpiologic(edev, vrf, bcrd),
        _ if bslots::XPHY_NIBBLESLICE.contains(bcrd.slot) => (),
        bslots::XPHY => xpio::verify_xphy(edev, vrf, bcrd),
        bslots::XPIO_VREF => xpio::verify_xpio_vref(vrf, bcrd),
        bslots::XPIO_DCI => xpio::verify_xpio_dci(vrf, bcrd),
        bslots::XPIO_SCAN => (),
        bslots::DDRMC => xpio::verify_ddrmc(edev, vrf, bcrd),
        _ if bslots::HNOC_NPS6.contains(bcrd.slot) => noc::verify_noc_nps6(edev, vrf, bcrd),
        _ if bslots::HNOC_NPS4.contains(bcrd.slot) => noc::verify_noc_nps4(edev, vrf, bcrd),
        bslots::NOC_HBM_BLI_SCAN => vrf.verify_bel(bcrd).commit(),
        _ if bslots::NOC_NMU_HBM2E.contains(bcrd.slot) => {
            noc::verify_noc_nmu_hbm2e(edev, vrf, bcrd)
        }
        bslots::HBM_CHNL => hbm::verify_hbm_chnl(edev, vrf, bcrd),
        bslots::HBM_MS => hbm::verify_hbm_ms(vrf, bcrd),
        bslots::HPLL => hbm::verify_hpll(vrf, bcrd),

        bslots::X5IO_OCTAD => x5io::verify_x5io_octad(edev, vrf, bcrd),
        bslots::X5PLL => x5io::verify_x5pll(edev, vrf, bcrd),
        bslots::X5PHIO_CMU => x5io::verify_x5phio_cmu(edev, vrf, bcrd),
        bslots::X5IO_SCAN => x5io::verify_x5io_scan(endev, vrf, bcrd),
        _ if bslots::X5PHIO_HARD_INV.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::X5PHIO_DCCIBUF.contains(bcrd.slot) => {
            x5io::verify_x5phio_dccibuf(edev, vrf, bcrd)
        }
        bslots::DDRMC5 => x5io::verify_ddrmc5(edev, vrf, bcrd),

        bslots::AIE_CORE => aie::verify_aie_core(edev, vrf, bcrd),
        bslots::AIE_ML_CORE => aie::verify_aie_ml_core(edev, vrf, bcrd),
        bslots::AIE_MEMGRP => aie::verify_aie_memgrp(edev, vrf, bcrd),
        bslots::AIE_ML_MEM_MEMGRP => aie::verify_aie_ml_mem_memgrp(edev, vrf, bcrd),
        _ if bslots::AIE_FIFO.contains(bcrd.slot) => aie::verify_aie_fifo(edev, vrf, bcrd),
        bslots::AIE_TILECTRL => aie::verify_aie_tilectrl(edev, vrf, bcrd),
        bslots::AIE_SHIMTRACE => aie::verify_aie_shimtrace(edev, vrf, bcrd),
        bslots::AIE2PS_SHIM_CNTRL => aie::verify_aie2ps_shim_cntrl(edev, vrf, bcrd),
        bslots::AIE2PS_NMU_ARBIT => aie::verify_aie2ps_nmu_arbit(edev, vrf, bcrd),
        bslots::AIE_ARRAYSWITCH => aie::verify_aie_arrayswitch(edev, vrf, bcrd),
        bslots::AIE_ML_ARRAYSWITCH => aie::verify_aie_ml_arrayswitch(edev, vrf, bcrd),
        bslots::AIE_ML_MEM_ARRAYSWITCH => aie::verify_aie_ml_mem_arrayswitch(edev, vrf, bcrd),
        bslots::AIE_SHIMSWITCH => aie::verify_aie_shimswitch(edev, vrf, bcrd),
        bslots::AIE_ML_SHIMSWITCH => aie::verify_aie_ml_shimswitch(edev, vrf, bcrd),
        bslots::AIE2PS_SHIMSWITCH => aie::verify_aie2ps_shimswitch(edev, vrf, bcrd),
        bslots::AIE_PL => aie::verify_aie_pl(endev, vrf, bcrd),
        bslots::AIE_NOC => aie::verify_aie_noc(endev, vrf, bcrd),
        bslots::AIE_PLL => aie::verify_aie_pll(edev, vrf, bcrd),
        bslots::AIE_INTF_TEST => aie::verify_aie_intf_test(edev, vrf, bcrd),
        bslots::AIE_ML_INTF_TEST => aie::verify_aie_ml_intf_test(edev, vrf, bcrd),
        bslots::AIE2PS_INTF_TEST => aie::verify_aie2ps_intf_test(edev, vrf, bcrd),

        _ => vrf
            .pb
            .suspend(|| println!("MEOW {}", bcrd.to_string(edev.db))),
    }
}

fn prep_bli(edev: &ExpandedDevice, vrf: &mut Verifier) {
    for tcid in [tcls::BLI_S, tcls::BLI_N, tcls::BLI_CLE_S, tcls::BLI_CLE_N] {
        let Some(BelInfo::SwitchBox(sb)) = edev.db[tcid].bels.get(bslots::BEL_INT) else {
            continue;
        };
        for item in &sb.items {
            let SwitchBoxItem::PermaBuf(buf) = item else {
                continue;
            };
            vrf.skip_tcls_pip(tcid, buf.dst, buf.src);
            for &tcrd in &edev.tile_index[tcid] {
                let wt = edev.resolve_tile_wire(tcrd, buf.dst).unwrap();
                let wf = edev.resolve_tile_wire(tcrd, buf.src.tw).unwrap();
                vrf.alias_wire(wt, wf);
            }
        }
    }
}

fn prep_permabufs(vrf: &mut Verifier) {
    for (tcid, bslot) in [
        (tcls::XPLL, bslots::XPLL_INT),
        (tcls::X5PLL_A, bslots::XPLL_INT),
        (tcls::X5PLL_B, bslots::XPLL_INT),
        (tcls::X5PLL_S, bslots::XPLL_INT),
        (tcls::X5PLL_NC, bslots::XPLL_INT),
        (tcls::X5PLL_GASKET, bslots::XPLL_GASKET_INT),
        (tcls::X5PLL_GASKET_HALF, bslots::XPLL_GASKET_INT),
        (tcls::HSR_CLK_TEST_XPIO_REV1, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_XPIO_REV2, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_ME_REV1, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_ME_REV2, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_ME_SSIT, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_ME_ULVT, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_X5IO_A, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_TEST_X5IO_B, bslots::HSR_CLK_TEST),
        (tcls::HSR_CLK_HBM_EVEN, bslots::HSR_CLK_IN),
        (tcls::HSR_CLK_HBM_ODD, bslots::HSR_CLK_IN),
        (tcls::HSR_CLK_HBM_MS_EVEN, bslots::HSR_CLK_IN),
        (tcls::HSR_CLK_HBM_MS_ODD, bslots::HSR_CLK_IN),
        (tcls::HSR_CLK_AIE_PLL, bslots::HSR_CLK_IN),
    ] {
        let Some(BelInfo::SwitchBox(sb)) = vrf.grid.db[tcid].bels.get(bslot) else {
            continue;
        };
        for item in &sb.items {
            if let SwitchBoxItem::PermaBuf(buf) = item {
                vrf.skip_tcls_pip(tcid, buf.dst, buf.src);
                for &tcrd in &vrf.grid.tile_index[tcid] {
                    vrf.alias_wire(
                        vrf.grid.tile_wire(tcrd, buf.dst),
                        vrf.grid.tile_wire(tcrd, buf.src.tw),
                    );
                }
            }
        }
    }
}

fn prep_dummy_nmu(edev: &ExpandedDevice, vrf: &mut Verifier) {
    for (die, &chip) in &edev.chips {
        let HsrSKind::X5io(ref x5io) = chip.hsr_s else {
            continue;
        };
        for (reg, rd) in &x5io.regions {
            let X5ioHNoc::Ddrmc { nmu_bli, .. } = rd.hnoc else {
                continue;
            };
            for (i, bli) in nmu_bli.into_iter().enumerate() {
                if bli.is_none() {
                    let bcrd = die
                        .cell(ColId::from_idx(reg.to_idx() * 4), chip.row_hsr(DirV::S))
                        .bel(bslots::HNOC_NMU512[i]);
                    vrf.skip_bel_input(bcrd, bcls::NOC_NMU512::CLK);
                    vrf.claim_net(&[vrf.bel_wire(bcrd, "CLK")]);
                    vrf.claim_pip(vrf.bel_wire(bcrd, "CLK"), vrf.bel_wire_far(bcrd, "CLK"));
                }
            }
        }
    }
}

fn verify_extra(_edev: &ExpandedDevice, vrf: &mut Verifier) {
    for i in 0..6 {
        vrf.kill_stub_out_cond_tk("CLE_BC_CORE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("CLE_BC_CORE_MX", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("TERM_T_CLE_LAG_CORE", &format!("GND_WIRE{i}"));
    }
    for i in 0..3 {
        vrf.kill_stub_out_cond_tk("NOC_PNOC_MONO_CORE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_PNOC_SSIT_CORE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_HNOC_S3_PS_CORE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_HNOC_S3_PL_CORE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_HNOC_S3_PL_CORE_MX", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_TNOC_S2_CORE_MX", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NOC_TNOC_SSIT_S2_CORE", &format!("GND_WIRE{i}"));
    }
    for i in 0..24 {
        vrf.kill_stub_out_cond_tk("TERM_T_GTCLK_R_CORE2", &format!("GND_WIRE{i}"));
    }
    for i in 0..2 {
        vrf.kill_stub_out_cond_tk("TERM_T_GTYP_L_CORE", &format!("GND_WIRE{i}"));
    }
    vrf.kill_stub_out_cond_tk("NOC_TNOC_BRIDGE_TOP_CORE", "GND_WIRE");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_BRIDGE_BOT_CORE", "GND_WIRE");
    vrf.kill_stub_out_cond_tk("NOC2_TNOC_NIDB_CORE", "GND_WIRE");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_SSIT_NCRB_CORE", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_SSIT_NCRB_CORE", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_SSIT_NCRB_CORE_MY", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_SSIT_NCRB_CORE_MY", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NCRB_CORE_MX", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NCRB_CORE_MX", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_TNOC_NCRB_CORE_MX", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_HBM_S4_NPS6_CORE", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_HBM_S4_CORE", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_HBM_S4_CORE", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NPS8_CORE", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NPS8_CORE", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NPS8_CORE", "GND_WIRE2");
    vrf.kill_stub_out_cond_tk("NOC_HBM_NPS8_CORE", "GND_WIRE3");
    vrf.kill_stub_out_cond_tk("CLKE2_REBUF_HBM_CORE_MX", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("HSM_AMS_SAT_ME", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("RBRK_CLE_CORE", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RBRK_CLE_CORE", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("RBRK_CLE_LAG_CORE", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RBRK_CLE_LAG_CORE", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("RCLK_CLE_CORE", "VCC_WIRE2");
    vrf.kill_stub_out_cond_tk("RCLK_CLE_VR_CORE", "VCC_WIRE2");
    vrf.kill_stub_out_cond_tk("RCLK_INT_L_FT", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RCLK_INT_L_FT", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("RCLK_INT_R_FT", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RCLK_INT_R_FT", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("RCLK_INT_L_VR_FT", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RCLK_INT_L_VR_FT", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("RCLK_INT_R_VR_FT", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("RCLK_INT_R_VR_FT", "VCC_WIRE1");
    vrf.kill_stub_out_cond_tk("CLE_W_CORE", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("CLE_E_CORE", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("CLE_W_VR_CORE", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("CLE_E_VR_CORE", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("TERM_P_CLE_CORE", "VCC_WIRE4");
    vrf.kill_stub_out_cond_tk("TERM_P_CLE_CORE", "VCC_WIRE5");
    vrf.kill_stub_out_cond_tk("HDIO_TILE", "VCC_WIRE4290");
    vrf.kill_stub_out_cond_tk("HDIO_BOT_TILE", "VCC_WIRE4290");
    vrf.kill_stub_out_cond_tk("HSM_AMS_SAT_LC", "VCC_WIRE");
    vrf.kill_stub_out_cond_tk("AIE2P_S_TILE", "GND_WIRE0");
    vrf.kill_stub_out_cond_tk("AIE2P_S_TILE", "GND_WIRE1");
    vrf.kill_stub_out_cond_tk("AIE2P_S_INTF_C_CORE", "GND_WIRE5");
    vrf.kill_stub_out_cond_tk("AIE2P_S_INTF_C_CORE", "GND_WIRE6");
    vrf.kill_stub_out_cond_tk("AIE2P_S_INTF_C_CORE", "GND_WIRE7");
    vrf.kill_stub_out_cond_tk("AIE2P_S_INTF_C_CORE", "GND_WIRE8");
    vrf.kill_stub_out_cond_tk("PSXC_TILE", "VCC_WIRE0");
    vrf.kill_stub_out_cond_tk("CLKE2_VNOC_ULVT_PSS_CCA_TILE", "VCC_WIRE388");
    for i in 4..8 {
        vrf.kill_stub_out_cond_tk("GTY_QUAD_SINGLE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("GTY_QUAD_SINGLE_MY", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("GTYP_QUAD_SINGLE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("GTYP_QUAD_SINGLE_MY", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("GTM_QUAD_SINGLE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("GTM_QUAD_SINGLE_MY", &format!("VCC_WIRE{i}"));
    }
    for i in 0..2 {
        vrf.kill_stub_out_cond_tk("NPI_VNPI_BOT_TILE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_TOP_TILE", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_BOT_TILE_MY", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_TOP_TILE_MY", &format!("GND_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_BOT_TILE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_TOP_TILE", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_BOT_TILE_MY", &format!("VCC_WIRE{i}"));
        vrf.kill_stub_out_cond_tk("NPI_VNPI_TOP_TILE_MY", &format!("VCC_WIRE{i}"));
    }
    for tkn in [
        "CLK_VNOC_TILE",
        "CLK_VNOC_AAA_TILE",
        "CLK_VNOC_AAO_TILE",
        "CLK_VNOC_BBA_TILE",
        "CLK_VNOC_BBO_TILE",
        "CLK_VNOC_CCA_TILE",
        "CLK_VNOC_CCO_TILE",
        "CLK_VNOC_SSIT_TOP_CCO_TILE",
        "CLK_VNOC_SSIT_TOP_AAO_TILE",
        "CLK_VNOC_SSIT_AIE_AAO_TILE",
        "CLK_VNOC_SSIT_HBM_AAO_TILE",
        "CLK_VNOC_SSIT_MT_AAO_TILE",
        "CLK_VNOC_SSIT_MT_BBO_TILE",
        "CLK_VNOC_SSIT_MB_BBO_TILE",
        "CLK_VNOC_SSIT_BOT_CCO_TILE",
        "CLK_VNOC_SSIT_HSR_CCO_TILE",
        "CLK_VNOC_PSS_TILE",
        "CLK_VNOC_PSS_AAA_TILE",
        "CLK_VNOC_PSS_BBA_TILE",
        "CLK_VNOC_PSS_CCA_TILE",
        "CLK_VNOC_PSS_CCO_TILE",
        "CLK_VNOC_PSS_AAO_TILE",
        "CLK_VNOC_PSS_BBO_TILE",
        "CLK_VNOC_PSS_SSIT_MT_AAO_TILE",
        "CLK_VNOC_PSS_SSIT_MB_BBO_TILE",
        "CLK_VNOC_PSS_SSIT_HSR_CCO_TILE",
        "CLK_VNOC_PSS_SSIT_BOT_CCO_TILE",
    ] {
        vrf.kill_stub_out_cond_tk(tkn, "GND_WIRE20");
    }
    for tkn in [
        "CLKE2_VNOC_CCO_TILE",
        "CLKE2_VNOC_BBO_TILE",
        "CLKE2_VNOC_AAO_TILE",
        "CLKE2_VNOC_EDGE_AAO_TILE",
        "CLKE2_VNOC_PSS_CCO_TILE",
        "CLKE2_VNOC_ULVT_AAA_TILE",
        "CLKE2_VNOC_ULVT_AAO_TILE",
        "CLKE2_VNOC_ULVT_BBA_TILE",
        "CLKE2_VNOC_ULVT_BBO_TILE",
        "CLKE2_VNOC_ULVT_CCA_TILE",
        "CLKE2_VNOC_ULVT_CCO_TILE",
        "CLKE2_VNOC_ULVT_PSS_BBA_TILE",
        "CLKE2_VNOC_ULVT_PSS_CCA_TILE",
        "CLKE2_VNOC_ULVT_PSS_CCO_TILE",
    ] {
        vrf.kill_stub_out_cond_tk(tkn, "GND_WIRE26");
    }

    vrf.skip_residual_nodes();
}

pub fn verify_device(endev: &ExpandedNamedDevice, rd: &Part, pb: &ProgressBar) {
    let mut vrf = Verifier::new(rd, &endev.ngrid, pb);
    prep_bli(endev.edev, &mut vrf);
    prep_permabufs(&mut vrf);
    prep_dummy_nmu(endev.edev, &mut vrf);
    vrf.prep_int_wires();
    vrf.handle_int();
    pb.set_length(vrf.grid.tiles().count() as u64);
    pb.set_position(0);
    pb.set_message("verify bel");
    for (tcrd, tile) in endev.ngrid.egrid.tiles() {
        pb.inc(1);
        let tcls = &endev.ngrid.egrid.db[tile.class];
        for slot in tcls.bels.ids() {
            verify_bel(endev, &mut vrf, tcrd.bel(slot));
        }
    }
    pb.set_message("verify finish");
    verify_extra(endev.edev, &mut vrf);
    vrf.finish();
}
