use indicatif::ProgressBar;
use prjcombine_entity::EntityId;
use prjcombine_interconnect::db::{BelInfo, PinDir, SwitchBoxItem};
use prjcombine_interconnect::grid::{BelCoord, DieId, DieIdExt, RowId, TileIobId};
use prjcombine_re_xilinx_naming_ultrascale::ExpandedNamedDevice;
use prjcombine_re_xilinx_rawdump::Part;
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_ultrascale::bond::SharedCfgPad;
use prjcombine_ultrascale::chip::{Chip, ChipKind, CleMKind, ColumnKind, DisabledPart};
use prjcombine_ultrascale::defs::ultrascale::tcls;
use prjcombine_ultrascale::defs::{bcls, bslots};
use prjcombine_ultrascale::expanded::{ExpandedDevice, HdioCoord, HpioCoord, IoCoord};

mod xp5io;

fn is_cut_d(edev: &ExpandedDevice, die: DieId, row: RowId) -> bool {
    let reg = edev.chips[die].row_to_reg(row);
    if reg.to_idx() == 0 {
        false
    } else {
        edev.disabled.contains(&DisabledPart::Region(die, reg - 1))
    }
}

fn is_cut_u(edev: &ExpandedDevice, die: DieId, row: RowId) -> bool {
    let reg = edev.chips[die].row_to_reg(row);
    edev.disabled.contains(&DisabledPart::Region(die, reg + 1))
}

fn get_hpiob_bel(edev: &ExpandedDevice, crd: IoCoord) -> BelCoord {
    let IoCoord::Hpio(crd) = crd else {
        unreachable!();
    };
    let row = crd.cell.row - if crd.iob.to_idx() < 26 { 30 } else { 0 };
    let bel_hpio = crd
        .cell
        .with_row(row)
        .bel(bslots::HPIOB[crd.iob.to_idx() % 26]);
    let bel_hrio = crd
        .cell
        .with_row(row)
        .bel(bslots::HRIOB[crd.iob.to_idx() % 26]);
    if edev.has_bel(bel_hpio) {
        bel_hpio
    } else {
        bel_hrio
    }
}

fn verify_slice(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = if edev[tcrd].class == tcls::CLEM {
        "SLICEM"
    } else {
        "SLICEL"
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_in_claim("CIN")
        .extra_out_claim("COUT");
    bel.claim_pip(bel.wire("CIN"), bel.wire_far("CIN"));
    if let Some(obel) = edev.bel_delta(bcrd.cell, 0, -1, bcrd.slot) {
        bel.verify_net(&[bel.wire_far("CIN"), bel.bel_wire(obel, "COUT")]);
    } else if !is_cut_d(edev, bcrd.die, bcrd.row)
        || (edev.chips[bcrd.die].columns[bcrd.col].kind == ColumnKind::CleM(CleMKind::Laguna)
            && bcrd.row.to_idx() == Chip::ROWS_PER_REG)
    {
        bel.claim_net(&[bel.wire_far("CIN")]);
    }
    if bel.vrf.ngrid.get_bel_name(bcrd).is_some() {
        bel.commit();
    }
}

fn verify_dsp(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("DSP48E2");
    let mut pairs = vec![];
    pairs.push(("MULTSIGNIN".to_string(), "MULTSIGNOUT".to_string()));
    pairs.push(("CARRYCASCIN".to_string(), "CARRYCASCOUT".to_string()));
    for i in 0..30 {
        pairs.push((format!("ACIN_B{i}"), format!("ACOUT_B{i}")));
    }
    for i in 0..18 {
        pairs.push((format!("BCIN_B{i}"), format!("BCOUT_B{i}")));
    }
    for i in 0..48 {
        pairs.push((format!("PCIN{i}"), format!("PCOUT{i}")));
    }
    for (ipin, opin) in &pairs {
        bel = bel.extra_in_claim(ipin).extra_out_claim(opin);
        if bcrd.slot == bslots::DSP[0] {
            bel.claim_pip(bel.wire(ipin), bel.wire_far(ipin));
            if let Some(obel) = edev.bel_delta(bcrd.cell, 0, -5, bslots::DSP[1]) {
                bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire(obel, opin)]);
            } else if !is_cut_d(edev, bcrd.die, bcrd.row) {
                bel.claim_net(&[bel.wire_far(ipin)]);
            }
        } else {
            let obel = bcrd.bel(bslots::DSP[0]);
            bel.claim_pip(bel.wire(ipin), bel.bel_wire(obel, opin));
        }
    }
    if bel.vrf.ngrid.get_bel_name(bcrd).is_some() {
        bel.commit();
    }
}

fn verify_bram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);

    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("RAMBFIFO36")
        .extra_out_claim("CASMBIST12OUT");

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Mode {
        Up,
        DownHalfReg,
        UpBuf,
        DownBuf,
    }
    let mut pairs = vec![
        (
            "ENABLE_BIST".to_string(),
            "START_RSR_NEXT".to_string(),
            Mode::DownHalfReg,
        ),
        (
            "CASINSBITERR".to_string(),
            "CASOUTSBITERR".to_string(),
            Mode::Up,
        ),
        (
            "CASINDBITERR".to_string(),
            "CASOUTDBITERR".to_string(),
            Mode::Up,
        ),
        (
            "CASPRVEMPTY".to_string(),
            "CASNXTEMPTY".to_string(),
            Mode::UpBuf,
        ),
        (
            "CASNXTRDEN".to_string(),
            "CASPRVRDEN".to_string(),
            Mode::DownBuf,
        ),
    ];
    for ab in ['A', 'B'] {
        for ul in ['U', 'L'] {
            for i in 0..16 {
                pairs.push((
                    format!("CASDI{ab}{ul}{i}"),
                    format!("CASDO{ab}{ul}{i}"),
                    Mode::UpBuf,
                ));
            }
            for i in 0..2 {
                pairs.push((
                    format!("CASDIP{ab}{ul}{i}"),
                    format!("CASDOP{ab}{ul}{i}"),
                    Mode::UpBuf,
                ));
            }
        }
    }
    let bel_prev = edev.bel_delta(bcrd.cell, 0, -5, bslots::BRAM);
    let bel_next = edev.bel_delta(bcrd.cell, 0, 5, bslots::BRAM);
    for (ipin, opin, mode) in &pairs {
        bel = bel.extra_in_claim(ipin).extra_out_claim(opin);
        bel.claim_pip(bel.wire(ipin), bel.wire_far(ipin));
        match mode {
            Mode::UpBuf => {
                if !edev.is_cut
                    || (edev.kind == ChipKind::UltrascalePlus && edev.is_cut_d)
                    || bel_next.is_some()
                {
                    bel.claim_net(&[bel.wire_far(opin)]);
                    bel.claim_pip(bel.wire_far(opin), bel.wire(opin));
                }
                if let Some(obel) = bel_prev {
                    bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                } else if !is_cut_d(edev, bcrd.die, bcrd.row) {
                    bel.claim_net(&[bel.wire_far(ipin)]);
                }
            }
            Mode::Up => {
                if let Some(obel) = bel_prev {
                    bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire(obel, opin)]);
                } else if !is_cut_d(edev, bcrd.die, bcrd.row) {
                    bel.claim_net(&[bel.wire_far(ipin)]);
                }
            }
            Mode::DownBuf => {
                if !edev.is_cut
                    || (edev.kind == ChipKind::UltrascalePlus && edev.is_cut_d)
                    || bel_prev.is_some()
                {
                    bel.claim_net(&[bel.wire_far(opin)]);
                    bel.claim_pip(bel.wire_far(opin), bel.wire(opin));
                }
                if let Some(obel) = bel_next {
                    bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire_far(obel, opin)]);
                } else if !is_cut_u(edev, bcrd.die, bcrd.row) {
                    bel.claim_net(&[bel.wire_far(ipin)]);
                }
            }
            Mode::DownHalfReg => match bcrd.row.to_idx() % Chip::ROWS_PER_REG {
                25 => (),
                55 => {
                    if !(edev.kind == ChipKind::UltrascalePlus
                        && edev.is_cut_d
                        && is_cut_u(edev, bcrd.die, bcrd.row))
                    {
                        bel.claim_net(&[bel.wire_far(ipin)]);
                    }
                }
                _ => {
                    if let Some(obel) = bel_next {
                        bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire(obel, opin)]);
                    } else {
                        bel.claim_net(&[bel.wire_far(ipin)]);
                    }
                }
            },
        }
    }
    bel.commit();

    for i in 0..2 {
        let prefix = &format!("H{i}_");
        let mut bel = vrf
            .verify_bel(bcrd)
            .sub(i + 1)
            .kind(["RAMBFIFO18", "RAMB181"][i])
            .skip_auto();
        for pin in naming.pins.keys() {
            if let Some(rpin) = pin.strip_prefix(prefix) {
                if rpin.starts_with("DO")
                    || rpin.starts_with("RDCOUNT")
                    || rpin.starts_with("WRCOUNT")
                    || rpin.starts_with("CASDOA")
                    || rpin.starts_with("CASDOB")
                    || rpin.starts_with("CASDOP")
                    || matches!(
                        rpin,
                        "FULL"
                            | "EMPTY"
                            | "PROGFULL"
                            | "PROGEMPTY"
                            | "RDERR"
                            | "RDRSTBUSY"
                            | "WRERR"
                            | "WRRSTBUSY"
                            | "CASNXTEMPTY"
                            | "CASPRVRDEN"
                    )
                {
                    bel = bel.extra_out_claim(pin);
                    bel.claim_pip(bel.wire_far(rpin), bel.wire(pin));
                } else {
                    bel = bel.extra_in_claim(pin);
                    let spin = rpin.strip_suffix("_X").unwrap_or(rpin);
                    bel.claim_pip(bel.wire(pin), bel.wire_far(spin));
                }
            }
        }
        bel.commit();
    }
}

fn verify_uram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::URAM.index_of(bcrd.slot).unwrap();
    let mut bel = vrf.verify_bel(bcrd).kind("URAM288");
    let mut pairs = vec![];
    for ab in ['A', 'B'] {
        for i in 0..23 {
            pairs.push((
                format!("CAS_IN_ADDR_{ab}{i}"),
                format!("CAS_OUT_ADDR_{ab}{i}"),
            ));
        }
        for i in 0..9 {
            pairs.push((
                format!("CAS_IN_BWE_{ab}{i}"),
                format!("CAS_OUT_BWE_{ab}{i}"),
            ));
        }
        for i in 0..72 {
            pairs.push((
                format!("CAS_IN_DIN_{ab}{i}"),
                format!("CAS_OUT_DIN_{ab}{i}"),
            ));
            pairs.push((
                format!("CAS_IN_DOUT_{ab}{i}"),
                format!("CAS_OUT_DOUT_{ab}{i}"),
            ));
        }
        for pin in ["EN", "RDACCESS", "RDB_WR", "DBITERR", "SBITERR"] {
            pairs.push((format!("CAS_IN_{pin}_{ab}"), format!("CAS_OUT_{pin}_{ab}")));
        }
    }
    for (ipin, opin) in &pairs {
        bel = bel.extra_in_claim(ipin).extra_out_claim(opin);
        if bcrd.slot == bslots::URAM[0] {
            bel.claim_pip(bel.wire(ipin), bel.wire_far(ipin));
            if let Some(obel) = edev.bel_delta(bcrd.cell, 0, -15, bslots::URAM[3]) {
                bel.verify_net(&[bel.wire_far(ipin), bel.bel_wire(obel, opin)]);
            } else if !is_cut_d(edev, bcrd.die, bcrd.row) {
                bel.claim_net(&[bel.wire_far(ipin)]);
            }
        } else {
            let obel = bcrd.bel(bslots::URAM[idx - 1]);
            bel.claim_pip(bel.wire(ipin), bel.bel_wire(obel, opin));
        }
    }
    bel.commit();
}

fn verify_laguna(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);

    let skip = if bcrd.row.to_idx() < Chip::ROWS_PER_REG {
        bcrd.die.to_idx() == 0
    } else {
        bcrd.die.to_idx() == edev.chips.len() - 1
    };
    let mut obel = None;

    if bcrd.row.to_idx() < Chip::ROWS_PER_REG && !skip {
        let odie = bcrd.die - 1;
        let orow = RowId::from_idx(edev.rows(odie).len() - Chip::ROWS_PER_REG + bcrd.row.to_idx());
        obel = Some(odie.cell(bcrd.col, orow).bel(bcrd.slot));
    }
    for i in 0..6 {
        bel = bel
            .extra_in_claim(format!("RXD{i}"))
            .extra_out_claim(format!("RXQ{i}"))
            .extra_out_claim(format!("TXQ{i}"))
            .skip_out(bcls::LAGUNA::RXOUT[i]);
        bel.claim_net(&[bel.wire(&format!("TXOUT{i}"))]);
        bel.claim_pip(bel.wire(&format!("TXOUT{i}")), bel.wire(&format!("TXD{i}")));
        bel.claim_pip(bel.wire(&format!("TXOUT{i}")), bel.wire(&format!("TXQ{i}")));
        bel.claim_pip(bel.wire(&format!("RXOUT{i}")), bel.wire(&format!("RXD{i}")));
        bel.claim_pip(bel.wire(&format!("RXOUT{i}")), bel.wire(&format!("RXQ{i}")));
        bel.claim_pip(bel.wire(&format!("RXD{i}")), bel.wire(&format!("TXOUT{i}")));
        bel.claim_pip(bel.wire(&format!("RXD{i}")), bel.wire(&format!("UBUMP{i}")));
        bel.claim_pip(bel.wire(&format!("UBUMP{i}")), bel.wire("VCC"));
        bel.claim_pip(
            bel.wire(&format!("UBUMP{i}")),
            bel.wire(&format!("TXOUT{i}")),
        );
        if let Some(obel) = obel {
            bel.claim_net(&[
                bel.wire(&format!("UBUMP{i}")),
                bel.bel_wire(obel, &format!("UBUMP{i}")),
            ]);
        } else if skip {
            bel.claim_net(&[bel.wire(&format!("UBUMP{i}"))]);
        }
    }

    if bcrd.slot == bslots::LAGUNA[0] {
        bel.claim_net(&[bel.wire("VCC")]);
        if edev.kind == ChipKind::Ultrascale {
            if !edev.is_cut {
                bel.claim_net(&[bel.wire("EXTRA_RXD")]);
                bel.claim_pip(bel.wire("EXTRA_RXD"), bel.wire("EXTRA_TXOUT"));
                bel.claim_pip(bel.wire("EXTRA_RXD"), bel.wire("EXTRA_UBUMP"));
            }
            bel.claim_net(&[bel.wire("EXTRA_TXOUT")]);
            bel.claim_pip(bel.wire("EXTRA_UBUMP"), bel.wire("VCC"));
            bel.claim_pip(bel.wire("EXTRA_UBUMP"), bel.wire("EXTRA_TXOUT"));
            if let Some(obel) = obel {
                bel.claim_net(&[bel.wire("EXTRA_UBUMP"), bel.bel_wire(obel, "EXTRA_UBUMP")]);
            } else if skip {
                bel.claim_net(&[bel.wire("EXTRA_UBUMP")]);
            }
        }
    }

    if !skip {
        bel.commit();
    }
}

fn verify_pcie(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    if edev
        .disabled
        .contains(&DisabledPart::HardIp(bcrd.die, bcrd.col, reg))
    {
        return;
    }
    let kind = match edev[tcrd].class {
        tcls::PCIE3 => "PCIE_3_1",
        tcls::PCIE4 => "PCIE40E4",
        tcls::PCIE4C => "PCIE4CE4",
        tcls::PCIE4CE => "PCIE4CE",
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_in("MCAP_PERST0_B")
        .extra_in("MCAP_PERST1_B");
    if bcrd.slot == bslots::PCIE3 {
        for i in 0..951 {
            bel = bel.extra_in(format!("XIL_UNCONN_CLK_B{i}"));
        }
        for i in 0..3189 {
            bel = bel.extra_in(format!("XIL_UNCONN_IN{i}"));
        }
        for i in 0..1920 {
            bel = bel.extra_in(format!("XIL_UNCONN_BYP{i}"));
        }
        for i in 0..861 {
            bel = bel.extra_out(format!("XIL_UNCONN_OUT{i}"));
        }
        for i in 0..480 {
            bel = bel.extra_out(format!("XIL_UNCONN_BOUT{i}"));
        }
    }
    if bel.wire("MCAP_PERST0_B") != bel.wire_far("MCAP_PERST0_B") {
        bel.claim_net(&[bel.wire("MCAP_PERST0_B")]);
        bel.claim_net(&[bel.wire("MCAP_PERST1_B")]);
        bel.claim_pip(bel.wire("MCAP_PERST0_B"), bel.wire_far("MCAP_PERST0_B"));
        bel.claim_pip(bel.wire("MCAP_PERST1_B"), bel.wire_far("MCAP_PERST1_B"));
    }
    if edev
        .bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, bslots::CFG)
        .is_some()
        && !edev.chips[bcrd.die].is_nocfg()
    {
        if edev.chips[bcrd.die].config_kind.is_csec() {
            // nothingness
        } else {
            let obel = get_hpiob_bel(
                edev,
                *edev.cfg_io[bcrd.die]
                    .get_by_left(&SharedCfgPad::PerstN0)
                    .unwrap(),
            );
            bel.verify_net(&[bel.wire_far("MCAP_PERST0_B"), bel.bel_wire(obel, "DOUT")]);
            let obel = get_hpiob_bel(
                edev,
                *edev.cfg_io[bcrd.die]
                    .get_by_left(&match edev.kind {
                        ChipKind::Ultrascale => SharedCfgPad::PerstN1,
                        ChipKind::UltrascalePlus => SharedCfgPad::I2cSda,
                    })
                    .unwrap(),
            );
            bel.verify_net(&[bel.wire_far("MCAP_PERST1_B"), bel.bel_wire(obel, "DOUT")]);
        }
    } else {
        bel.claim_net(&[bel.wire_far("MCAP_PERST0_B")]);
        bel.claim_net(&[bel.wire_far("MCAP_PERST1_B")]);
    }
    bel.commit();
}

fn verify_cmac(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    if edev
        .disabled
        .contains(&DisabledPart::HardIp(bcrd.die, bcrd.col, reg))
    {
        return;
    }
    vrf.verify_bel(bcrd)
        .kind(if edev.kind == ChipKind::Ultrascale {
            "CMAC_SITE"
        } else {
            "CMACE4"
        })
        .commit();
}

fn verify_ilkn(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    if edev
        .disabled
        .contains(&DisabledPart::HardIp(bcrd.die, bcrd.col, reg))
    {
        return;
    }
    vrf.verify_bel(bcrd)
        .kind(if edev.kind == ChipKind::Ultrascale {
            "ILKN_SITE"
        } else {
            "ILKNE4"
        })
        .commit();
}

fn verify_ps(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("PS8");
    for pin in [
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
    ] {
        bel = bel.extra_in_claim(pin);
    }
    for pin in [
        "IDCODE15",
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
        bel = bel.extra_in_dummy(pin);
    }
    let tk = bel.vrf.rd.tile_kinds.get("PSS_ALTO").unwrap().1;
    let site = tk.sites.values().next().unwrap();
    if site.pins.contains_key("IDCODE16") {
        bel = bel.extra_in_dummy("IDCODE16");
    }
    if !edev.disabled.contains(&DisabledPart::Ps) {
        bel.commit();
    }
}

fn verify_dfe(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let dis = if tcid == tcls::DFE_A {
        let chip = edev.chips[bcrd.die];
        let reg = chip.row_to_reg(bcrd.row);
        DisabledPart::HardIp(bcrd.die, bcrd.col, reg)
    } else {
        DisabledPart::Dfe
    };
    if edev.disabled.contains(&dis) {
        return;
    }
    let (kind, bslots) = match tcid {
        tcls::DFE_A => (
            "DFE_A",
            vec![bslots::DFE_FFT[0], bslots::DFE_FFT[1], bslots::DFE_PRACH],
        ),
        tcls::DFE_B => (
            "DFE_B",
            vec![
                bslots::DFE_FFT[0],
                bslots::DFE_FFT[1],
                bslots::DFE_FIR[0],
                bslots::DFE_FIR[1],
            ],
        ),
        tcls::DFE_C => (
            "DFE_C",
            vec![
                bslots::DFE_MATRIX_FIR[0],
                bslots::DFE_MATRIX_FIR[1],
                bslots::DFE_DUC_DDC[0],
                bslots::DFE_DUC_DDC[1],
            ],
        ),
        tcls::DFE_D => (
            "DFE_D",
            vec![
                bslots::DFE_FFT[0],
                bslots::DFE_CFR[0],
                bslots::DFE_CFR[1],
                bslots::DFE_PQ_INTERP[0],
            ],
        ),
        tcls::DFE_E => (
            "DFE_E",
            vec![
                bslots::DFE_NL_FIR[0],
                bslots::DFE_NL_FIR[1],
                bslots::DFE_NL_FIR[2],
            ],
        ),
        tcls::DFE_F => (
            "DFE_F",
            vec![
                bslots::DFE_NL_FIR[0],
                bslots::DFE_PQ_INTERP[0],
                bslots::DFE_PQ_INTERP[1],
                bslots::DFE_PQ_INTERP[2],
                bslots::DFE_PQ_INTERP[3],
            ],
        ),
        tcls::DFE_G => (
            "DFE_G",
            vec![
                bslots::DFE_PQ_DECIM[0],
                bslots::DFE_PQ_DECIM[1],
                bslots::DFE_PQ_DECIM[2],
                bslots::DFE_PQ_DECIM[3],
                bslots::DFE_PQ_DECIM[4],
            ],
        ),
        _ => unreachable!(),
    };
    if bcrd.slot != bslots[0] {
        return;
    }
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd).kind(kind).skip_auto();
    for (pname, pin) in &naming.pins {
        if !pname.contains("_INST_") {
            continue;
        }
        let &(_, dir) = pin.orig_pin.as_ref().unwrap();
        if dir == PinDir::Input {
            bel = bel.extra_in(pname);
        } else {
            bel = bel.extra_out(pname);
        }
    }
    bel.commit();
}

fn verify_sysmon(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind(match edev.kind {
        ChipKind::Ultrascale => "SYSMONE1",
        ChipKind::UltrascalePlus => "SYSMONE4",
    });
    if edev.kind == ChipKind::Ultrascale {
        bel = bel
            .extra_in_claim("I2C_SCLK_IN")
            .extra_out_claim("I2C_SCLK_TS")
            .extra_in_claim("I2C_SDA_IN")
            .extra_out_claim("I2C_SDA_TS");
        let obel = get_hpiob_bel(
            edev,
            *edev.cfg_io[bcrd.die]
                .get_by_left(&SharedCfgPad::I2cSclk)
                .unwrap(),
        );
        bel.verify_net(&[bel.wire("I2C_SCLK_TS"), bel.bel_wire_far(obel, "TSDI")]);
        bel.verify_net(&[bel.wire_far("I2C_SCLK_IN"), bel.bel_wire(obel, "DOUT")]);
        let obel = get_hpiob_bel(
            edev,
            *edev.cfg_io[bcrd.die]
                .get_by_left(&SharedCfgPad::I2cSda)
                .unwrap(),
        );
        bel.verify_net(&[bel.wire("I2C_SDA_TS"), bel.bel_wire_far(obel, "TSDI")]);
        bel.verify_net(&[bel.wire_far("I2C_SDA_IN"), bel.bel_wire(obel, "DOUT")]);
        bel.claim_pip(bel.wire("I2C_SCLK_IN"), bel.wire_far("I2C_SCLK_IN"));
        bel.claim_pip(bel.wire("I2C_SDA_IN"), bel.wire_far("I2C_SDA_IN"));
    }
    for i in 0..16 {
        for pn in ['P', 'N'] {
            let pin = &format!("V{pn}_AUX{i}");
            bel = bel.extra_in_claim(pin);
            if edev.kind == ChipKind::UltrascalePlus {
                bel.claim_net(&[bel.wire_far(pin)]);
                bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
            }
        }
    }
    bel.commit();
}

fn verify_abus_switch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    if edev.kind == ChipKind::UltrascalePlus
        && (bcrd.slot == bslots::ABUS_SWITCH_CFG || bslots::ABUS_SWITCH_HBM.contains(bcrd.slot))
    {
        bel = bel.extra_in_claim("TEST_ANALOGBUS_SEL_B");
    }
    let tcrd = edev.bel_tile(bcrd);
    let reg = edev.chips[bcrd.die].row_to_reg(bcrd.row);
    let mut skip = false;
    if edev[tcrd].class == tcls::GTM {
        skip = edev
            .disabled
            .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg));
    }
    if edev
        .disabled
        .contains(&DisabledPart::GtBufs(bcrd.die, bcrd.col, reg))
    {
        skip = true;
    }

    if !skip {
        bel.commit();
    }
}

fn verify_bufce_leaf(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).extra_out_claim("CLK_CASC_OUT");
    let prev = if let Some(idx) = bslots::BUFCE_LEAF_S.index_of(bcrd.slot) {
        if idx == 0 {
            None
        } else {
            Some(bslots::BUFCE_LEAF_S[idx - 1])
        }
    } else {
        let idx = bslots::BUFCE_LEAF_N.index_of(bcrd.slot).unwrap();
        if idx == 0 {
            Some(bslots::BUFCE_LEAF_S[15])
        } else {
            Some(bslots::BUFCE_LEAF_N[idx - 1])
        }
    };
    if let Some(oslot) = prev {
        bel = bel.extra_in_claim("CLK_CASC_IN");
        let obel = bcrd.bel(oslot);
        bel.claim_pip(bel.wire("CLK_CASC_IN"), bel.bel_wire(obel, "CLK_CASC_OUT"));
    }
    bel.commit();
}

fn verify_bufce_row(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    if bslots::BUFCE_ROW_RCLK.contains(bcrd.slot) {
        bel = bel.extra_out_claim("CLK_OUT_OPT_DLY");
        if edev.kind == ChipKind::UltrascalePlus {
            bel = bel.kind("BUFCE_ROW_FSR");
        }
    }
    bel.commit();
}

fn verify_hdiob(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::HDIOB.index_of(bcrd.slot).unwrap();
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let is_m = idx.is_multiple_of(2);
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let hid = TileIobId::from_idx(idx);
    let iocrd = if tcid == tcls::HDIO {
        IoCoord::Hdio(HdioCoord {
            cell: bcrd.cell,
            iob: hid,
        })
    } else {
        IoCoord::HdioLc(HdioCoord {
            cell: bcrd.cell,
            iob: hid,
        })
    };
    let has_clkpad = matches!(
        (tcid, idx),
        (tcls::HDIO, 8 | 10 | 12 | 14)
            | (tcls::HDIOS, 8 | 10 | 22 | 24)
            | (tcls::HDIOL_S, 10 | 12)
            | (tcls::HDIOL_N, 0 | 2)
    );
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(if is_m { "HDIOB_M" } else { "HDIOB_S" })
        .extra_in_claim("OP")
        .extra_in_claim("TSP")
        .extra_out_claim("O_B")
        .extra_out_claim("TSTATEB")
        .extra_out_claim("OUTB_B")
        .extra_in_claim("OUTB_B_IN")
        .extra_out_claim("TSTATE_OUT")
        .extra_in_claim("TSTATE_IN")
        .extra_in_claim("LVDS_TRUE")
        .extra_out_claim("PAD_RES")
        .extra_in_dummy("IO")
        .extra_out("SWITCH_OUT");
    if has_clkpad {
        bel = bel.extra_out("I").skip_out(bcls::HDIOB::CLKPAD);
        bel.verify_net(&[bel.wire("I"), bel.wire("CLKPAD")]);
    } else {
        bel = bel.extra_out_claim("I");
    }
    let obel = bcrd.bel(bslots::HDIOB[idx ^ 1]);
    bel.claim_pip(bel.wire("OUTB_B_IN"), bel.bel_wire(obel, "OUTB_B"));
    bel.claim_pip(bel.wire("TSTATE_IN"), bel.bel_wire(obel, "TSTATE_OUT"));

    let sysmon = bcrd
        .with_cr(chip.col_cfg(), chip.row_ams())
        .bel(bslots::SYSMON);
    let io_info = edev.get_io_info(iocrd);
    if tcid == tcls::HDIO {
        bel.claim_net(&[bel.wire("SWITCH_OUT")]);
        bel.claim_pip(bel.wire_far("SWITCH_OUT"), bel.wire("SWITCH_OUT"));
        if let Some(ams_idx) = io_info.sm_pair {
            bel.verify_net(&[
                bel.wire_far("SWITCH_OUT"),
                bel.bel_wire_far(
                    sysmon,
                    &format!("V{pn}_AUX{ams_idx}", pn = if is_m { 'P' } else { 'N' }),
                ),
            ]);
        }
    } else {
        if let Some(ams_idx) = io_info.sm_pair {
            bel.verify_net(&[
                bel.wire("SWITCH_OUT"),
                bel.bel_wire_far(
                    sysmon,
                    &format!("V{pn}_AUX{ams_idx}", pn = if is_m { 'P' } else { 'N' }),
                ),
            ]);
        } else {
            bel.claim_net(&[bel.wire("SWITCH_OUT")]);
        }
    }

    if !edev
        .disabled
        .contains(&DisabledPart::HdioIob(bcrd.die, bcrd.col, reg, hid))
    {
        bel.commit();
    }
}

fn verify_hdiodiffin(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::HDIOB_DIFF_IN.index_of(bcrd.slot).unwrap();
    let obel_m = bcrd.bel(bslots::HDIOB[2 * idx]);
    let obel_s = bcrd.bel(bslots::HDIOB[2 * idx + 1]);
    let hid = TileIobId::from_idx(2 * idx);
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);

    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HDIOBDIFFINBUF")
        .extra_out_claim("LVDS_TRUE")
        .extra_out_claim("LVDS_COMP")
        .extra_in_claim("PAD_RES_0")
        .extra_in_claim("PAD_RES_1");
    bel.claim_pip(bel.bel_wire(obel_m, "LVDS_TRUE"), bel.wire("LVDS_TRUE"));
    bel.claim_pip(bel.bel_wire(obel_s, "LVDS_TRUE"), bel.wire("LVDS_COMP"));
    bel.claim_pip(bel.wire("PAD_RES_0"), bel.bel_wire(obel_m, "PAD_RES"));
    bel.claim_pip(bel.wire("PAD_RES_1"), bel.bel_wire(obel_s, "PAD_RES"));
    if !edev
        .disabled
        .contains(&DisabledPart::HdioIob(bcrd.die, bcrd.col, reg, hid))
    {
        bel.commit();
    }
}

fn verify_hdiologic(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::HDIOLOGIC.index_of(bcrd.slot).unwrap();
    let is_m = idx.is_multiple_of(2);
    let kind = if is_m { "HDIOLOGIC_M" } else { "HDIOLOGIC_S" };
    let mut bel = vrf.verify_bel(bcrd).kind(kind);
    if is_m {
        bel = bel
            .extra_in_claim("IPFFM_D")
            .extra_out_claim("OPFFM_Q")
            .extra_out_claim("TFFM_Q");
    } else {
        bel = bel
            .extra_in_claim("IPFFS_D")
            .extra_out_claim("OPFFS_Q")
            .extra_out_claim("TFFS_Q");
    }
    let obel = bcrd.bel(bslots::HDIOB[idx]);
    bel.claim_pip(
        bel.bel_wire(obel, "OP"),
        bel.wire(if is_m { "OPFFM_Q" } else { "OPFFS_Q" }),
    );
    bel.claim_pip(
        bel.bel_wire(obel, "TSP"),
        bel.wire(if is_m { "TFFM_Q" } else { "TFFS_Q" }),
    );
    bel.claim_pip(
        bel.wire(if is_m { "IPFFM_D" } else { "IPFFS_D" }),
        bel.bel_wire(obel, "I"),
    );
    bel.commit();
}

fn verify_mmcm(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    if edev.kind == ChipKind::Ultrascale {
        bel = bel.kind("MMCME3_ADV");
    }
    bel.commit();
}

fn verify_pll(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;

    let mut bel = vrf.verify_bel(bcrd).extra_out_claim("CLKOUTPHY_P");
    if edev.kind == ChipKind::Ultrascale {
        bel = bel.kind("PLLE3_ADV");
    }

    if tcid == tcls::CMT_HBM {
        bel = bel.extra_in_claim("CLKFBIN");
        bel.claim_net(&[bel.wire_far("CLKFBIN")]);
        bel.claim_pip(bel.wire("CLKFBIN"), bel.wire_far("CLKFBIN"));
    }

    if tcid == tcls::CMTXP {
        bel = bel
            .kind("PLLXP")
            .extra_in_claim("CLKOUTPHY_DMCEN")
            .extra_in_claim("RST_DMC")
            .extra_out_claim("LOCKED_DMC");

        let obel_lpddrmc = bcrd.bel(bslots::LPDDRMC);
        for pin in ["CLKOUTPHY_DMCEN", "RST_DMC"] {
            bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
        }
        let idx = bslots::PLL.index_of(bcrd.slot).unwrap();
        bel.verify_net(&[
            bel.wire_far("CLKOUTPHY_DMCEN"),
            bel.bel_wire(obel_lpddrmc, &format!("DMC_XPLL{idx}_CLKOUTPHY_EN")),
        ]);
        bel.verify_net(&[
            bel.wire_far("RST_DMC"),
            bel.bel_wire(obel_lpddrmc, &format!("DMC_XPLL{idx}_RESET")),
        ]);
    }

    bel.commit();
}

fn verify_bitslice(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::BITSLICE.index_of(bcrd.slot).unwrap();
    let nidx = usize::from(idx >= 6);
    let sidx = idx - nidx * 6;
    let obel_bsctl = bcrd.bel(bslots::BITSLICE_CONTROL[nidx]);
    let obel_feed = bcrd.bel(bslots::XIPHY_BYTE);
    let obel_bstx = bcrd.bel(bslots::BITSLICE_T[nidx]);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("BITSLICE_RX_TX")
        // to IOB
        .extra_out_claim("TX_Q")
        .extra_in_claim("RX_D")
        // to BSCTL
        .extra_out_claim("BS2CTL_TX_DDR_PHASE_SEL")
        .extra_out_claim("TX_VTC_READY")
        .extra_out_claim("RX_VTC_READY")
        .extra_out_claim("BS2CTL_IDELAY_DELAY_FORMAT")
        .extra_out_claim("BS2CTL_ODELAY_DELAY_FORMAT")
        .extra_out_claim("RX_DQS_OUT")
        .extra_out_claim("BS2CTL_RX_DDR_EN_DQS")
        .extra_out_claim("BS2CTL_RX_P0_DQ_OUT")
        .extra_out_claim("BS2CTL_RX_N0_DQ_OUT")
        // cascade stuff
        .extra_in_claim("RX2TX_CASC_RETURN_IN")
        .extra_in_claim("TX2RX_CASC_IN")
        .extra_out_claim("TX2RX_CASC_OUT")
        // aliases of CLK
        .extra_in_claim("RX_CLKDIV")
        .extra_in_claim("TX_CLK")
        .extra_in_claim("TX_OCLKDIV")
        .skip_in(bcls::BITSLICE::DYN_DCI_OUT_INT)
        // reclaimed back from XIPHY_FEEDTHROUGH
        .skip_in(bcls::BITSLICE::FIFO_RDCLK)
        .skip_in(bcls::BITSLICE::RX_RST)
        .skip_in(bcls::BITSLICE::TX_RST)
        .skip_in(bcls::BITSLICE::RX_RST_DLY)
        .skip_in(bcls::BITSLICE::TX_RST_DLY);
    if !matches!(idx, 0 | 6) {
        bel = bel
            // stub
            .extra_out_claim("FIFO_WRCLK");
    }

    bel.claim_pip(bel.wire("RX_CLKDIV"), bel.wire_far("CLK"));
    bel.claim_pip(bel.wire("TX_OCLKDIV"), bel.wire_far("CLK"));
    bel.claim_pip(bel.wire("TX_CLK"), bel.wire_far("CLK"));

    for (pin, opin) in [
        ("TX_CTRL_CLK", format!("ODELAY_CTRL_CLK{sidx}")),
        ("TX_CTRL_CE", format!("ODELAY_CE_OUT{sidx}")),
        ("TX_CTRL_INC", format!("ODELAY_INC_OUT{sidx}")),
        ("TX_CTRL_LD", format!("ODELAY_LD_OUT{sidx}")),
        ("TX_DIV2_CLK", format!("DIV2_CLK_OUT{sidx}")),
        ("TX_DIV4_CLK", format!("DIV_CLK_OUT{sidx}")),
        ("TX_DDR_CLK", format!("DDR_CLK_OUT{sidx}")),
        ("CTL2BS_DYNAMIC_MODE_EN", format!("DYNAMIC_MODE_EN{sidx}")),
        ("CTL2BS_TX_DDR_PHASE_SEL", format!("TX_DATA_PHASE{sidx}")),
        ("TX_TOGGLE_DIV2_SEL", format!("TOGGLE_DIV2_SEL{sidx}")),
        ("TX_MUX_360_P_SEL", format!("PH02_DIV2_360_{sidx}")),
        ("TX_MUX_360_N_SEL", format!("PH13_DIV2_360_{sidx}")),
        ("TX_MUX_720_P0_SEL", format!("PH0_DIV_720_{sidx}")),
        ("TX_MUX_720_P1_SEL", format!("PH1_DIV_720_{sidx}")),
        ("TX_MUX_720_P2_SEL", format!("PH2_DIV_720_{sidx}")),
        ("TX_MUX_720_P3_SEL", format!("PH3_DIV_720_{sidx}")),
        ("TX_WL_TRAIN", format!("WL_TRAIN{sidx}")),
        ("TX_BS_RESET", format!("TX_BS_RESET{sidx}")),
        ("RX_CLK_P", format!("PDQS_OUT{sidx}")),
        ("RX_CLK_N", format!("NDQS_OUT{sidx}")),
        ("RX_CTRL_CLK", format!("IDELAY_CTRL_CLK{sidx}")),
        ("RX_CTRL_CE", format!("IDELAY_CE_OUT{sidx}")),
        ("RX_CTRL_INC", format!("IDELAY_INC_OUT{sidx}")),
        ("RX_CTRL_LD", format!("IDELAY_LD_OUT{sidx}")),
        ("RX_DCC0", format!("RX_DCC{sidx:02}_0")),
        ("RX_DCC1", format!("RX_DCC{sidx:02}_1")),
        ("RX_DCC2", format!("RX_DCC{sidx:02}_2")),
        ("RX_DCC3", format!("RX_DCC{sidx:02}_3")),
        ("RX_BS_RESET", format!("RX_BS_RESET{sidx}")),
        ("CTL2BS_FIFO_BYPASS", format!("IFIFO_BYPASS{sidx}")),
    ] {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_bsctl, &opin));
    }

    let ul = if nidx == 1 { "UPP" } else { "LOW" };
    let mut pins_feed = vec![
        (
            "CTL2BS_RX_RECALIBRATE_EN",
            format!("CTL2BS_REFCLK_EN_{ul}_SMX{sidx}"),
        ),
        ("CLB2PHY_FIFO_CLK", format!("CLB2PHY_FIFO_CLK_SMX{idx}")),
    ];

    if edev.kind == ChipKind::Ultrascale {
        pins_feed.extend([
            ("RX_RESET_B", format!("CLB2PHY_IDELAY_RST_B_SMX{idx}")),
            ("TX_REGRST_B", format!("CLB2PHY_ODELAY_RST_B_SMX{idx}")),
            ("RX_RST_B", format!("CLB2PHY_RXBIT_RST_B_SMX{idx}")),
            ("TX_RST_B", format!("CLB2PHY_TXBIT_RST_B_SMX{idx}")),
        ]);
    } else {
        pins_feed.extend([
            ("RX_RESET", format!("CLB2PHY_IDELAY_RST_SMX{idx}")),
            ("TX_REGRST", format!("CLB2PHY_ODELAY_RST_SMX{idx}")),
        ]);
        bel = bel
            .extra_in_claim("RAW_RX_RST")
            .extra_in_claim("RAW_TX_RST");
        bel.claim_pip(
            bel.wire("RAW_RX_RST"),
            bel.bel_wire(obel_feed, &format!("CLB2PHY_RXBIT_RST_SMX{idx}")),
        );
        bel.claim_pip(
            bel.wire("RAW_TX_RST"),
            bel.bel_wire(obel_feed, &format!("CLB2PHY_TXBIT_RST_SMX{idx}")),
        );
    }

    for (pin, opin) in pins_feed {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_feed, &opin));
    }

    let rx_ctrl_dly: [_; 9] = core::array::from_fn(|i| format!("RX_CTRL_DLY{i}"));
    let tx_ctrl_dly: [_; 9] = core::array::from_fn(|i| format!("TX_CTRL_DLY{i}"));
    for (i, pin) in rx_ctrl_dly.iter().enumerate() {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire(obel_bsctl, &format!("IDELAY{sidx:02}_OUT{i}")),
        );
    }
    for (i, pin) in tx_ctrl_dly.iter().enumerate() {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire(obel_bsctl, &format!("ODELAY{sidx:02}_OUT{i}")),
        );
    }
    // to BSCTL
    for i in 0..9 {
        bel = bel
            .extra_out_claim(format!("BS2CTL_RX_CNTVALUEOUT{i}"))
            .extra_out_claim(format!("BS2CTL_TX_CNTVALUEOUT{i}"));
    }
    for i in 0..18 {
        bel = bel
            .extra_out_claim(format!("BS2CTL_IDELAY_FIXED_DLY_RATIO{i}"))
            .extra_out_claim(format!("BS2CTL_ODELAY_FIXED_DLY_RATIO{i}"));
    }

    bel = bel.extra_in_claim("TX_TBYTE_IN");
    bel.claim_pip(bel.wire("TX_TBYTE_IN"), bel.bel_wire(obel_bstx, "Q"));

    bel.claim_pip(bel.wire("RX_D"), bel.wire_far("RX_D"));

    if idx != 12 {
        let obel_bs = bcrd.bel(bslots::BITSLICE[idx + 1]);
        bel.claim_pip(
            bel.wire("TX2RX_CASC_IN"),
            bel.bel_wire(obel_bs, "TX2RX_CASC_OUT"),
        );
    }
    if idx != 0 {
        let obel_bs = bcrd.bel(bslots::BITSLICE[idx - 1]);
        bel.claim_pip(
            bel.wire("RX2TX_CASC_RETURN_IN"),
            bel.bel_wire(obel_bs, "RX_Q5"),
        );
    }

    bel.claim_pip(bel.wire("DYN_DCI_OUT"), bel.wire("DYN_DCI_OUT_INT"));
    bel.claim_pip(
        bel.wire("DYN_DCI_OUT"),
        bel.bel_wire(obel_bsctl, &format!("DYN_DCI_OUT{sidx}")),
    );
    bel.commit();
}

fn verify_bitslice_t(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::BITSLICE_T.index_of(bcrd.slot).unwrap();
    let obel_bsctl = bcrd.bel(bslots::BITSLICE_CONTROL[idx]);
    let obel_feed = bcrd.bel(bslots::XIPHY_BYTE);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("BITSLICE_TX")
        // to BITSLICE_RX_TX.TBYTE_IN [all of them]
        .extra_out_claim("Q")
        // to BSCTL
        .extra_out_claim("BS2CTL_TX_DDR_PHASE_SEL")
        .extra_out_claim("VTC_READY")
        // dummy stuff?
        .extra_in_claim("CDATAIN0")
        .extra_in_claim("CDATAIN1")
        .extra_out_claim("CDATAOUT")
        // reclaimed from XIPHY_FEEDTHROUGH
        .skip_in(bcls::BITSLICE_T::RST)
        .skip_in(bcls::BITSLICE_T::RST_DLY);

    for (pin, opin) in [
        ("CTRL_CE", "TRISTATE_ODELAY_CE_OUT"),
        ("CTRL_INC", "TRISTATE_ODELAY_INC_OUT"),
        ("CTRL_LD", "TRISTATE_ODELAY_LD_OUT"),
        ("CTRL_CLK", "ODELAY_CTRL_CLK7"),
        ("DIV2_CLK", "DIV2_CLK_OUT7"),
        ("DIV4_CLK", "DIV_CLK_OUT7"),
        ("DDR_CLK", "DDR_CLK_OUT7"),
        ("CTL2BS_DYNAMIC_MODE_EN", "DYNAMIC_MODE_EN7"),
        ("CTL2BS_TX_DDR_PHASE_SEL", "TX_DATA_PHASE7"),
        ("FORCE_OE_B", "FORCE_OE_B"),
        ("TOGGLE_DIV2_SEL", "TOGGLE_DIV2_SEL7"),
        ("TX_MUX_360_P_SEL", "PH02_DIV2_360_7"),
        ("TX_MUX_360_N_SEL", "PH13_DIV2_360_7"),
        ("TX_MUX_720_P0_SEL", "PH0_DIV_720_7"),
        ("TX_MUX_720_P1_SEL", "PH1_DIV_720_7"),
        ("TX_MUX_720_P2_SEL", "PH2_DIV_720_7"),
        ("TX_MUX_720_P3_SEL", "PH3_DIV_720_7"),
        ("BS_RESET", "BS_RESET_TRI"),
    ] {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_bsctl, opin));
    }
    let feed_pins = if edev.kind == ChipKind::Ultrascale {
        [
            (
                "REGRST_B",
                "CLB2PHY_TRISTATE_ODELAY_RST_B_SMX0",
                "CLB2PHY_TRISTATE_ODELAY_RST_B_SMX1",
            ),
            (
                "RST_B",
                "CLB2PHY_TXBIT_TRI_RST_B_SMX0",
                "CLB2PHY_TXBIT_TRI_RST_B_SMX1",
            ),
        ]
    } else {
        [
            (
                "REGRST",
                "CLB2PHY_TRISTATE_ODELAY_RST_SMX0",
                "CLB2PHY_TRISTATE_ODELAY_RST_SMX1",
            ),
            (
                "RAW_RST",
                "CLB2PHY_TXBIT_TRI_RST_SMX0",
                "CLB2PHY_TXBIT_TRI_RST_SMX1",
            ),
        ]
    };
    for (pin, opin_l, opin_u) in feed_pins {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire(obel_feed, [opin_l, opin_u][idx]),
        );
    }
    for i in 0..9 {
        let pin = &format!("CTRL_DLY{i}");
        bel = bel
            .extra_in_claim(pin)
            // to BSCTL
            .extra_out_claim(format!("BS2CTL_CNTVALUEOUT{i}"));
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire(obel_bsctl, &format!("TRISTATE_ODELAY_OUT{i}")),
        );
    }
    for i in 0..8 {
        let pin = &format!("D{i}");
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(
            bel.wire(pin),
            bel.bel_wire(obel_bsctl, &format!("EN_DIV_DLY_OE{i}")),
        );
    }

    bel.commit();
}

fn verify_bitslice_control(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let idx = bslots::BITSLICE_CONTROL.index_of(bcrd.slot).unwrap();
    let obel_bs_t = bcrd.bel(bslots::BITSLICE_T[idx]);
    let obel_feed = bcrd.bel(bslots::XIPHY_BYTE);

    let mut bel = vrf
        .verify_bel(bcrd)
        .skip_in(bcls::BITSLICE_CONTROL::CLK)
        .skip_in(bcls::BITSLICE_CONTROL::RST)
        // to PLL_SELECT
        .extra_out_claim("REFCLK_DFD")
        .extra_out_claim("PLL_CLK_EN")
        // to RIU
        .extra_out_claim("RIU2CLB_VALID");
    for i in 0..16 {
        bel = bel.extra_out_claim(format!("RIU2CLB_RD_DATA{i}"));
    }
    bel = bel
        // to BITSLICE_TX
        .extra_out_claim("TRISTATE_ODELAY_CE_OUT")
        .extra_out_claim("TRISTATE_ODELAY_INC_OUT")
        .extra_out_claim("TRISTATE_ODELAY_LD_OUT")
        .extra_out_claim("FORCE_OE_B")
        .extra_out_claim("BS_RESET_TRI");
    for i in 0..8 {
        bel = bel.extra_out_claim(format!("EN_DIV_DLY_OE{i}"));
    }
    for i in 0..9 {
        bel = bel.extra_out_claim(format!("TRISTATE_ODELAY_OUT{i}"));
    }
    // to BITSLICE_TX and BITSLICE_RX_TX
    for i in 0..8 {
        for p in [
            format!("ODELAY_CTRL_CLK{i}"),
            format!("DYNAMIC_MODE_EN{i}"),
            format!("TOGGLE_DIV2_SEL{i}"),
            format!("TX_DATA_PHASE{i}"),
            format!("DIV2_CLK_OUT{i}"),
            format!("DIV_CLK_OUT{i}"),
            format!("DDR_CLK_OUT{i}"),
            format!("PH02_DIV2_360_{i}"),
            format!("PH13_DIV2_360_{i}"),
            format!("PH0_DIV_720_{i}"),
            format!("PH1_DIV_720_{i}"),
            format!("PH2_DIV_720_{i}"),
            format!("PH3_DIV_720_{i}"),
        ] {
            bel = bel.extra_out_claim(p);
        }
    }
    // to BITSLICE_RX_TX
    for i in 0..7 {
        for p in [
            format!("IDELAY_CTRL_CLK{i}"),
            format!("IDELAY_CE_OUT{i}"),
            format!("IDELAY_INC_OUT{i}"),
            format!("IDELAY_LD_OUT{i}"),
            format!("ODELAY_CE_OUT{i}"),
            format!("ODELAY_INC_OUT{i}"),
            format!("ODELAY_LD_OUT{i}"),
            format!("WL_TRAIN{i}"),
            format!("RX_BS_RESET{i}"),
            format!("TX_BS_RESET{i}"),
            format!("PDQS_OUT{i}"),
            format!("NDQS_OUT{i}"),
            format!("RX_DCC{i:02}_0"),
            format!("RX_DCC{i:02}_1"),
            format!("RX_DCC{i:02}_2"),
            format!("RX_DCC{i:02}_3"),
            format!("IFIFO_BYPASS{i}"),
        ] {
            bel = bel.extra_out_claim(p);
        }
        for j in 0..9 {
            for p in [
                format!("IDELAY{i:02}_OUT{j}"),
                format!("ODELAY{i:02}_OUT{j}"),
            ] {
                bel = bel.extra_out_claim(p);
            }
        }
    }
    // to IOB, via a mux associated with BITSLICE_RX_TX
    for i in 0..7 {
        bel = bel.extra_out_claim(format!("DYN_DCI_OUT{i}"));
    }
    // to XIPHY_FEEDTHROUGH
    for i in 0..7 {
        bel = bel.extra_out_claim(format!("REFCLK_EN{i}"));
    }
    for pin in [
        // to other BSCTL
        "CLK_TO_EXT_SOUTH",
        "CLK_TO_EXT_NORTH",
        "PDQS_GT_OUT",
        "NDQS_GT_OUT",
        // to XIPHY_FEEDTHROUGH
        "LOCAL_DIV_CLK",
    ] {
        bel = bel.extra_out_claim(pin.to_string());
    }

    // from PLL_SELECT
    bel = bel.extra_in_claim("PLL_CLK");
    bel.claim_pip(bel.wire("PLL_CLK"), bel.wire("PLL_SELECT_Z"));
    // from BITSLICE_TX
    bel = bel
        .extra_in_claim("BS2CTL_RIU_TX_DATA_PHASE7")
        .extra_in_claim("TRISTATE_VTC_READY");
    bel.claim_pip(
        bel.wire("BS2CTL_RIU_TX_DATA_PHASE7"),
        bel.bel_wire(obel_bs_t, "BS2CTL_TX_DDR_PHASE_SEL"),
    );
    bel.claim_pip(
        bel.wire("TRISTATE_VTC_READY"),
        bel.bel_wire(obel_bs_t, "VTC_READY"),
    );
    for i in 0..9 {
        let pin = format!("TRISTATE_ODELAY_IN{i}");
        bel.claim_pip(
            bel.wire(&pin),
            bel.bel_wire(obel_bs_t, &format!("BS2CTL_CNTVALUEOUT{i}")),
        );
        bel = bel.extra_in_claim(pin);
    }

    // from BITSLICE_RX_TX
    for i in 0..7 {
        let obel = if idx == 0 && i == 6 {
            None
        } else {
            Some(bcrd.bel(bslots::BITSLICE[idx * 6 + i]))
        };
        for (pin, opin) in [
            (
                format!("BS2CTL_RIU_TX_DATA_PHASE{i}"),
                "BS2CTL_TX_DDR_PHASE_SEL",
            ),
            (format!("RX_PDQ{i}_IN"), "BS2CTL_RX_P0_DQ_OUT"),
            (format!("RX_NDQ{i}_IN"), "BS2CTL_RX_N0_DQ_OUT"),
            (format!("FIXED_IDELAY{i:02}"), "BS2CTL_IDELAY_DELAY_FORMAT"),
            (format!("FIXED_ODELAY{i:02}"), "BS2CTL_ODELAY_DELAY_FORMAT"),
            (format!("VTC_READY_IDELAY{i:02}"), "RX_VTC_READY"),
            (format!("VTC_READY_ODELAY{i:02}"), "TX_VTC_READY"),
            (format!("DQS_IN{i}"), "RX_DQS_OUT"),
            (format!("BS2CTL_RIU_BS_DQS_EN{i}"), "BS2CTL_RX_DDR_EN_DQS"),
        ] {
            if let Some(obel) = obel {
                bel.claim_pip(bel.wire(&pin), bel.bel_wire(obel, opin));
            }
            bel = bel.extra_in_claim(pin);
        }
        for j in 0..9 {
            for (pin, opin) in [
                (
                    format!("IDELAY{i:02}_IN{j}"),
                    format!("BS2CTL_RX_CNTVALUEOUT{j}"),
                ),
                (
                    format!("ODELAY{i:02}_IN{j}"),
                    format!("BS2CTL_TX_CNTVALUEOUT{j}"),
                ),
            ] {
                if let Some(obel) = obel {
                    bel.claim_pip(bel.wire(&pin), bel.bel_wire(obel, &opin));
                }
                bel = bel.extra_in_claim(pin);
            }
        }
        for j in 0..18 {
            for (pin, opin) in [
                (
                    format!("FIXDLYRATIO_IDELAY{i:02}_{j}"),
                    format!("BS2CTL_IDELAY_FIXED_DLY_RATIO{j}"),
                ),
                (
                    format!("FIXDLYRATIO_ODELAY{i:02}_{j}"),
                    format!("BS2CTL_ODELAY_FIXED_DLY_RATIO{j}"),
                ),
            ] {
                if let Some(obel) = obel {
                    bel.claim_pip(bel.wire(&pin), bel.bel_wire(obel, &opin));
                }
                bel = bel.extra_in_claim(pin);
            }
        }
    }

    for (pin, opin) in [
        (
            "CLK_STOP",
            ["XIPHY_CLK_STOP_CTRL_LOW", "XIPHY_CLK_STOP_CTRL_UPP"][idx],
        ),
        ("SCAN_INT", ["SCAN_INT_LOWER", "SCAN_INT_UPPER"][idx]),
        if edev.kind == ChipKind::Ultrascale {
            (
                "CLB2PHY_CTRL_RST_B",
                ["CLB2PHY_CTRL_RST_B_LOW_SMX", "CLB2PHY_CTRL_RST_B_UPP_SMX"][idx],
            )
        } else {
            (
                "CLB2PHY_CTRL_RST",
                ["CLB2PHY_CTRL_RST_LOW_SMX", "CLB2PHY_CTRL_RST_UPP_SMX"][idx],
            )
        },
    ] {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_feed, opin));
    }

    for pin in [
        // from other BSCTL in byte
        "PDQS_GT_IN",
        "NDQS_GT_IN",
        // from BSCTL in another byte
        "CLK_FROM_EXT",
    ] {
        bel = bel.extra_in_claim(pin);
    }
    let obel_bsctl_on = bcrd.bel(bslots::BITSLICE_CONTROL[idx ^ 1]);
    bel.claim_pip(
        bel.wire("PDQS_GT_IN"),
        bel.bel_wire(obel_bsctl_on, "PDQS_GT_OUT"),
    );
    bel.claim_pip(
        bel.wire("NDQS_GT_IN"),
        bel.bel_wire(obel_bsctl_on, "NDQS_GT_OUT"),
    );
    let is_from_n = bcrd.row < chip.row_rclk(bcrd.row);
    let obel_bsctl_ob = edev
        .bel_delta(bcrd.cell, 0, if is_from_n { 15 } else { -15 }, bcrd.slot)
        .unwrap();
    if edev.kind == ChipKind::Ultrascale {
        bel.claim_pip(
            bel.wire("CLK_FROM_EXT"),
            bel.bel_wire(
                obel_bsctl_ob,
                if is_from_n {
                    "CLK_TO_EXT_SOUTH"
                } else {
                    "CLK_TO_EXT_NORTH"
                },
            ),
        );
    } else {
        bel.claim_pip(bel.wire("CLK_FROM_EXT"), bel.wire_far("CLK_FROM_EXT"));
        bel.verify_net(&[
            bel.wire_far("CLK_FROM_EXT"),
            bel.bel_wire(
                obel_bsctl_ob,
                if is_from_n {
                    "CLK_TO_EXT_SOUTH"
                } else {
                    "CLK_TO_EXT_NORTH"
                },
            ),
        ]);
    }

    let naming = bel.naming;
    for pin in naming.pins.keys() {
        let Some(stem) = pin.strip_prefix("CLB2RIU_") else {
            continue;
        };
        if stem == "NIBBLE_SEL" {
            bel = bel.extra_in(pin.clone());
        } else {
            let spin = format!("RIU_{stem}");
            bel.claim_pip(bel.wire(pin), bel.bel_wire(obel_feed, &spin));
            bel = bel.extra_in_claim(pin.clone());
        }
    }

    bel = bel.extra_in_claim("RIU_CLK");
    bel.claim_pip(bel.wire("RIU_CLK"), bel.wire_far("RIU_CLK"));

    bel.commit();

    // PLL_SELECT
    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(1)
        .kind("PLL_SELECT_SITE")
        .skip_auto()
        .extra_in_claim("PLL_SELECT_D0")
        .extra_in_claim("PLL_SELECT_D1")
        .extra_in_claim("PLL_SELECT_REFCLK_DFD")
        .extra_in_claim("PLL_SELECT_PLL_CLK_EN")
        .extra_out_claim("PLL_SELECT_Z");
    let obel_pll0 = bcrd.with_row(chip.row_rclk(bcrd.row)).bel(bslots::PLL[0]);
    let obel_pll1 = bcrd.with_row(chip.row_rclk(bcrd.row)).bel(bslots::PLL[1]);
    bel.claim_pip(bel.wire("PLL_SELECT_D0"), bel.wire_far("PLL_SELECT_D0"));
    bel.claim_pip(bel.wire("PLL_SELECT_D1"), bel.wire_far("PLL_SELECT_D1"));
    bel.verify_net(&[
        bel.wire_far("PLL_SELECT_D0"),
        bel.bel_wire(obel_pll0, "CLKOUTPHY_P"),
    ]);
    bel.verify_net(&[
        bel.wire_far("PLL_SELECT_D1"),
        bel.bel_wire(obel_pll1, "CLKOUTPHY_P"),
    ]);

    bel.claim_pip(bel.wire("PLL_SELECT_REFCLK_DFD"), bel.wire("REFCLK_DFD"));
    bel.claim_pip(bel.wire("PLL_SELECT_PLL_CLK_EN"), bel.wire("PLL_CLK_EN"));

    bel.commit();
}

fn verify_xiphy_byte(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let obel_bsctl_l = bcrd.bel(bslots::BITSLICE_CONTROL[0]);
    let obel_bsctl_u = bcrd.bel(bslots::BITSLICE_CONTROL[1]);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("XIPHY_FEEDTHROUGH")
        // to BSCTL
        .extra_out_claim("SCAN_INT_LOWER")
        .extra_out_claim("SCAN_INT_UPPER")
        .extra_out_claim("XIPHY_CLK_STOP_CTRL_LOW")
        .extra_out_claim("XIPHY_CLK_STOP_CTRL_UPP")
        // from BSCTL
        .extra_in_claim("DIV_CLK_OUT_LOW")
        .extra_in_claim("DIV_CLK_OUT_UPP")
        // dummy ins
        .extra_in_claim("RCLK2PHY_CLKDR")
        .extra_in_claim("RCLK2PHY_SHIFTDR")
        // stuff from other sub-bel
        .skip_out(bcls::XIPHY_BYTE::RIU_RD_VALID)
        .skip_in(bcls::XIPHY_BYTE::RIU_WR_EN);
    for i in 0..16 {
        bel = bel
            .skip_in(bcls::XIPHY_BYTE::RIU_WR_DATA[i])
            .skip_out(bcls::XIPHY_BYTE::RIU_RD_DATA[i]);
    }
    for pin in bcls::XIPHY_BYTE::RIU_ADDR
        .into_iter()
        .chain(bcls::XIPHY_BYTE::RIU_NIBBLE_SEL)
    {
        bel = bel.skip_in(pin);
    }
    if edev.kind == ChipKind::Ultrascale {
        bel = bel
            // to BSCTL
            .extra_out_claim("CLB2PHY_CTRL_RST_B_LOW_SMX")
            .extra_out_claim("CLB2PHY_CTRL_RST_B_UPP_SMX")
            // to BITSLICE_TX
            .extra_out_claim("CLB2PHY_TRISTATE_ODELAY_RST_B_SMX0")
            .extra_out_claim("CLB2PHY_TRISTATE_ODELAY_RST_B_SMX1")
            .extra_out_claim("CLB2PHY_TXBIT_TRI_RST_B_SMX0")
            .extra_out_claim("CLB2PHY_TXBIT_TRI_RST_B_SMX1");
    } else {
        bel = bel
            // to BSCTL
            .extra_out_claim("CLB2PHY_CTRL_RST_LOW_SMX")
            .extra_out_claim("CLB2PHY_CTRL_RST_UPP_SMX")
            // to BITSLICE_TX
            .extra_out_claim("CLB2PHY_TRISTATE_ODELAY_RST_SMX0")
            .extra_out_claim("CLB2PHY_TRISTATE_ODELAY_RST_SMX1")
            .extra_out_claim("CLB2PHY_TXBIT_TRI_RST_SMX0")
            .extra_out_claim("CLB2PHY_TXBIT_TRI_RST_SMX1");
    }
    bel.claim_pip(
        bel.wire("DIV_CLK_OUT_LOW"),
        bel.bel_wire(obel_bsctl_l, "LOCAL_DIV_CLK"),
    );
    bel.claim_pip(
        bel.wire("DIV_CLK_OUT_UPP"),
        bel.bel_wire(obel_bsctl_u, "LOCAL_DIV_CLK"),
    );
    // to BITSLICE_RX_TX
    for i in 0..13 {
        bel = bel.extra_out_claim(format!("CLB2PHY_FIFO_CLK_SMX{i}"));
        if edev.kind == ChipKind::Ultrascale {
            bel = bel
                .extra_out_claim(format!("CLB2PHY_TXBIT_RST_B_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_RXBIT_RST_B_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_IDELAY_RST_B_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_ODELAY_RST_B_SMX{i}"));
        } else {
            bel = bel
                .extra_out_claim(format!("CLB2PHY_TXBIT_RST_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_RXBIT_RST_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_IDELAY_RST_SMX{i}"))
                .extra_out_claim(format!("CLB2PHY_ODELAY_RST_SMX{i}"));
        }
        // reclaimed by BITSLICE
        if edev.kind == ChipKind::Ultrascale {
            bel = bel
                .extra_in(format!("RXBIT_RST_B{i}"))
                .extra_in(format!("TXBIT_RST_B{i}"))
                .extra_in(format!("IDELAY_RST_B{i}"))
                .extra_in(format!("ODELAY_RST_B{i}"));
        } else {
            bel = bel
                .extra_in(format!("RXBIT_RST{i}"))
                .extra_in(format!("TXBIT_RST{i}"))
                .extra_in(format!("IDELAY_RST{i}"))
                .extra_in(format!("ODELAY_RST{i}"));
        }
        bel = bel.extra_in(format!("CLB2PHY_FIFO_CLK{i}"));
    }
    for i in 0..6 {
        bel = bel.extra_out_claim(format!("CTL2BS_REFCLK_EN_LOW_SMX{i}"));
    }
    for i in 0..7 {
        bel = bel.extra_out_claim(format!("CTL2BS_REFCLK_EN_UPP_SMX{i}"));
    }
    // reclaimed by BITSLICE_T
    for i in 0..2 {
        if edev.kind == ChipKind::Ultrascale {
            bel = bel
                .extra_in(format!("TXBIT_TRI_RST_B{i}"))
                .extra_in(format!("TRISTATE_ODELAY_RST_B{i}"));
        } else {
            bel = bel
                .extra_in(format!("TXBIT_TRI_RST{i}"))
                .extra_in(format!("TRISTATE_ODELAY_RST{i}"));
        }
    }
    // reclaimed by BITSLICE_CONTROL
    bel = bel
        .extra_in("CLB2PHY_CTRL_CLK_LOW")
        .extra_in("CLB2PHY_CTRL_CLK_UPP");
    if edev.kind == ChipKind::Ultrascale {
        bel = bel.extra_in("CTRL_RST_B_LOW").extra_in("CTRL_RST_B_UPP");
    } else {
        bel = bel.extra_in("CTRL_RST_LOW").extra_in("CTRL_RST_UPP");
    }

    // from BSCTL
    for i in 0..7 {
        let pin_l = format!("CTL2BS_REFCLK_EN_LOW{i}");
        let pin_u = format!("CTL2BS_REFCLK_EN_UPP{i}");
        bel.claim_pip(
            bel.wire(&pin_l),
            bel.bel_wire(obel_bsctl_l, &format!("REFCLK_EN{i}")),
        );
        bel.claim_pip(
            bel.wire(&pin_u),
            bel.bel_wire(obel_bsctl_u, &format!("REFCLK_EN{i}")),
        );
        bel = bel.extra_in_claim(pin_l).extra_in_claim(pin_u);
    }

    bel.commit();

    // RIU_OR
    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(1)
        .kind("RIU_OR")
        .skip_auto()
        .extra_out("RIU_RD_VALID");
    for i in 0..16 {
        bel = bel.extra_out(format!("RIU_RD_DATA{i}"));
    }

    let obel_l = bcrd.bel(bslots::BITSLICE_CONTROL[0]);
    let obel_u = bcrd.bel(bslots::BITSLICE_CONTROL[1]);
    for (obel, ul) in [(obel_l, "LOW"), (obel_u, "UPP")] {
        let pin = format!("RIU_RD_VALID_{ul}");
        bel.claim_pip(bel.wire(&pin), bel.bel_wire(obel, "RIU2CLB_VALID"));
        bel = bel.extra_in_claim(pin);
        for i in 0..16 {
            let pin = format!("RIU_RD_DATA_{ul}{i}");
            bel.claim_pip(
                bel.wire(&pin),
                bel.bel_wire(obel, &format!("RIU2CLB_RD_DATA{i}")),
            );
            bel = bel.extra_in_claim(pin);
        }
    }

    bel.commit();
}

fn verify_hpiob(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let idx = bslots::HPIOB.index_of(bcrd.slot).unwrap();
    let pidx = if matches!(idx, 12 | 25) {
        None
    } else if idx < 12 {
        Some(idx ^ 1)
    } else {
        Some(((idx - 1) ^ 1) + 1)
    };
    let is_single = pidx.is_none();
    let is_m = !is_single
        && if idx < 13 {
            idx.is_multiple_of(2)
        } else {
            !idx.is_multiple_of(2)
        };
    let kind = if edev.kind == ChipKind::Ultrascale {
        "HPIOB"
    } else if is_single {
        "HPIOB_SNGL"
    } else if is_m {
        "HPIOB_M"
    } else {
        "HPIOB_S"
    };
    let fidx = if bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
        idx
    } else {
        idx + 26
    };
    let hid = TileIobId::from_idx(fidx);
    let reg = chip.row_to_reg(bcrd.row);

    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        // to/from PHY
        .extra_in_claim("OP")
        .extra_in_claim("TSP")
        .extra_in_claim("DYNAMIC_DCI_TS")
        // to AMS
        .extra_out_claim("SWITCH_OUT")
        // to/from paired IOB
        .extra_out_claim("OUTB_B")
        .extra_out_claim("TSTATE_OUT")
        // to/from differential out
        .extra_out_claim("O_B")
        .extra_out_claim("TSTATEB")
        // from VREF
        .extra_in_claim("VREF")
        // dummies
        .extra_out_claim("DOUT")
        .extra_in("TSDI");

    // to differential in
    if edev.kind == ChipKind::Ultrascale {
        bel = bel.extra_out_claim("CTLE_IN");
    }
    if edev.kind == ChipKind::Ultrascale || !matches!(idx, 12 | 25) {
        bel = bel.extra_out_claim("PAD_RES");
    }

    if is_single {
        bel = bel.extra_in_dummy("IO")
    } else {
        bel = bel.extra_in_claim("IO")
    }
    if is_single && edev.kind == ChipKind::UltrascalePlus {
        bel = bel
            .extra_in_dummy("LVDS_TRUE")
            .extra_in_dummy("OUTB_B_IN")
            .extra_in_dummy("TSTATE_IN");
    } else {
        bel = bel
            .extra_in_claim("LVDS_TRUE")
            .extra_in_claim("OUTB_B_IN")
            .extra_in_claim("TSTATE_IN");
    }
    let has_clkpad = matches!(idx, 0 | 2 | 21 | 23);
    if has_clkpad {
        bel = bel.extra_out("I").skip_out(bcls::HPIOB::CLKPAD);
        bel.verify_net(&[bel.wire("I"), bel.wire("CLKPAD")]);
    } else {
        bel = bel.extra_out_claim("I");
    }

    if let Some(pidx) = pidx {
        let obel = bcrd.bel(bslots::HPIOB[pidx]);
        bel.claim_pip(bel.wire("OUTB_B_IN"), bel.bel_wire(obel, "OUTB_B"));
        bel.claim_pip(bel.wire("TSTATE_IN"), bel.bel_wire(obel, "TSTATE_OUT"));
    } else if edev.kind == ChipKind::Ultrascale {
        bel.claim_pip(bel.wire("OUTB_B_IN"), bel.wire("OUTB_B"));
    }

    let srow = if idx < 13 { bcrd.row } else { bcrd.row + 15 };
    let obel_bs = bcrd.with_row(srow).bel(bslots::BITSLICE[idx % 13]);

    bel.claim_pip(bel.wire("OP"), bel.wire_far("OP"));
    bel.claim_pip(bel.wire("TSP"), bel.wire_far("TSP"));
    bel.verify_net(&[
        bel.wire("DYNAMIC_DCI_TS"),
        bel.bel_wire(obel_bs, "DYN_DCI_OUT"),
    ]);
    bel.verify_net(&[bel.wire("I"), bel.bel_wire_far(obel_bs, "RX_D")]);
    bel.verify_net(&[bel.wire_far("OP"), bel.bel_wire(obel_bs, "TX_Q")]);
    bel.verify_net(&[bel.wire_far("TSP"), bel.bel_wire(obel_bs, "TX_T_OUT")]);

    let crd = IoCoord::Hpio(HpioCoord {
        cell: bcrd.with_row(chip.row_reg_rclk(reg)),
        iob: hid,
    });
    let cfg = edev.cfg_io[bcrd.die].get_by_right(&crd);
    let is_cfg = match cfg {
        Some(SharedCfgPad::PerstN0) => false,
        Some(SharedCfgPad::PerstN1) => false,
        Some(SharedCfgPad::EmCclk) => false,
        Some(_) => true,
        None => false,
    };
    let is_ams = matches!(cfg, Some(SharedCfgPad::I2cSda | SharedCfgPad::I2cSclk))
        && edev.kind == ChipKind::Ultrascale;
    if is_ams {
        bel.claim_net(&[bel.wire("TSDI")]);
        bel.claim_pip(bel.wire("TSDI"), bel.wire_far("TSDI"));
    } else if edev.is_cut {
        if !(edev.is_cut_d
            && edev.kind == ChipKind::UltrascalePlus
            && bel.wire("TSDI") == bel.wire_far("TSDI")
            && is_cfg)
        {
            bel.claim_net(&[bel.wire("TSDI")]);
        }
    } else if !is_cfg {
        bel.claim_net(&[bel.wire("TSDI")]);
        if bel.wire("TSDI") != bel.wire_far("TSDI") {
            bel.claim_pip(bel.wire("TSDI"), bel.wire_far("TSDI"));
            bel.claim_net(&[bel.wire_far("TSDI")]);
        }
    } else if bel.wire("TSDI") != bel.wire_far("TSDI") {
        bel.claim_net(&[bel.wire("TSDI")]);
    }

    if !bel.naming.pins["SWITCH_OUT"].pips.is_empty() && !chip.config_kind.is_csec() {
        bel.claim_pip(bel.wire_far("SWITCH_OUT"), bel.wire("SWITCH_OUT"));

        let ams_idx = match fidx {
            4 | 5 => Some(15),
            6 | 7 => Some(7),
            8 | 9 => Some(14),
            10 | 11 => Some(6),
            13 | 14 => Some(13),
            15 | 16 => Some(5),
            17 | 18 => Some(12),
            19 | 20 => Some(4),
            30 | 31 => Some(11),
            32 | 33 => Some(3),
            34 | 35 => Some(10),
            36 | 37 => Some(2),
            39 | 40 => Some(9),
            41 | 42 => Some(1),
            43 | 44 => Some(8),
            45 | 46 => Some(0),
            _ => None,
        };

        if let Some(ams_idx) = ams_idx {
            let scol = chip.col_cfg();
            let srow = chip.row_ams();
            let obel = bcrd.cell.with_cr(scol, srow).bel(bslots::SYSMON);
            bel.verify_net(&[
                bel.wire_far("SWITCH_OUT"),
                bel.bel_wire_far(
                    obel,
                    &format!("V{pn}_AUX{ams_idx}", pn = if is_m { 'P' } else { 'N' }),
                ),
            ]);
        }
    }

    let obel_vref = bcrd.bel(bslots::HPIO_VREF);
    bel.claim_pip(
        bel.wire("VREF"),
        bel.bel_wire(obel_vref, if idx < 13 { "VREF1" } else { "VREF2" }),
    );

    if !edev
        .disabled
        .contains(&DisabledPart::HpioIob(bcrd.die, bcrd.col, reg, hid))
    {
        bel.commit();
    }
}

fn verify_hpiodiffin(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let idx = bslots::HPIOB_DIFF_IN.index_of(bcrd.slot).unwrap();
    let (pidx, nidx) = if idx < 6 {
        (idx * 2, idx * 2 + 1)
    } else {
        (idx * 2 + 1, idx * 2 + 2)
    };
    let reg = chip.row_to_reg(bcrd.row);
    let mut disabled = false;
    for sidx in [pidx, nidx] {
        let fidx = if bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
            sidx
        } else {
            sidx + 26
        };
        let hid = TileIobId::from_idx(fidx);
        disabled |= edev
            .disabled
            .contains(&DisabledPart::HpioIob(bcrd.die, bcrd.col, reg, hid));
    }
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HPIOBDIFFINBUF")
        .extra_in_claim("PAD_RES_0")
        .extra_in_claim("PAD_RES_1")
        .extra_out_claim("LVDS_TRUE")
        .extra_out_claim("LVDS_COMP")
        .extra_in_claim("VREF");
    if edev.kind == ChipKind::Ultrascale {
        bel = bel.extra_in_claim("CTLE_IN_1");
    }

    let obel_p = bcrd.bel(bslots::HPIOB[pidx]);
    let obel_n = bcrd.bel(bslots::HPIOB[nidx]);
    bel.claim_pip(bel.wire("PAD_RES_0"), bel.bel_wire(obel_p, "PAD_RES"));
    bel.claim_pip(bel.wire("PAD_RES_1"), bel.bel_wire(obel_n, "PAD_RES"));
    bel.claim_pip(bel.bel_wire(obel_p, "LVDS_TRUE"), bel.wire("LVDS_TRUE"));
    bel.claim_pip(bel.bel_wire(obel_n, "LVDS_TRUE"), bel.wire("LVDS_COMP"));
    if edev.kind == ChipKind::Ultrascale {
        bel.claim_pip(bel.wire("CTLE_IN_1"), bel.bel_wire(obel_n, "CTLE_IN"));
    }

    let obel_vref = bcrd.bel(bslots::HPIO_VREF);
    bel.claim_pip(
        bel.wire("VREF"),
        bel.bel_wire(obel_vref, if idx < 6 { "VREF1" } else { "VREF2" }),
    );

    if !disabled {
        bel.commit();
    }
}

fn verify_hpiodiffout(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let idx = bslots::HPIOB_DIFF_OUT.index_of(bcrd.slot).unwrap();
    let (pidx, nidx) = if idx < 6 {
        (idx * 2, idx * 2 + 1)
    } else {
        (idx * 2 + 1, idx * 2 + 2)
    };
    let reg = chip.row_to_reg(bcrd.row);
    let mut disabled = false;
    for sidx in [pidx, nidx] {
        let fidx = if bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
            sidx
        } else {
            sidx + 26
        };
        let hid = TileIobId::from_idx(fidx);
        disabled |= edev
            .disabled
            .contains(&DisabledPart::HpioIob(bcrd.die, bcrd.col, reg, hid));
    }
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HPIOBDIFFOUTBUF")
        .extra_in_claim("O_B")
        .extra_in_claim("TSTATEB")
        .extra_out_claim("AOUT")
        .extra_out_claim("BOUT");

    let obel_p = bcrd.bel(bslots::HPIOB[pidx]);
    let obel_n = bcrd.bel(bslots::HPIOB[nidx]);
    bel.claim_pip(bel.wire("O_B"), bel.bel_wire(obel_p, "O_B"));
    bel.claim_pip(bel.wire("TSTATEB"), bel.bel_wire(obel_p, "TSTATEB"));
    bel.claim_pip(bel.bel_wire(obel_p, "IO"), bel.wire("AOUT"));
    bel.claim_pip(bel.bel_wire(obel_n, "IO"), bel.wire("BOUT"));

    if !disabled {
        bel.commit();
    }
}

fn verify_hpio_vref(vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .kind("HPIO_VREF_SITE")
        .extra_out_claim("VREF1")
        .extra_out_claim("VREF2")
        .commit();
}

fn verify_hpio_dci(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    if !edev
        .disabled
        .contains(&DisabledPart::HpioDci(bcrd.die, bcrd.col, reg))
    {
        vrf.verify_bel(bcrd).kind("HPIOB_DCI_SNGL").commit();
    }
}

fn verify_hriob(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let idx = bslots::HRIOB.index_of(bcrd.slot).unwrap();
    let pidx = if matches!(idx, 12 | 25) {
        None
    } else if idx < 12 {
        Some(idx ^ 1)
    } else {
        Some(((idx - 1) ^ 1) + 1)
    };
    let is_single = pidx.is_none();
    let is_m = !is_single
        && if idx < 13 {
            idx.is_multiple_of(2)
        } else {
            !idx.is_multiple_of(2)
        };
    let fidx = if bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
        idx
    } else {
        idx + 26
    };
    let hid = TileIobId::from_idx(fidx);
    let reg = chip.row_to_reg(bcrd.row);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HRIO")
        // to/from PHY
        .extra_in_claim("OP")
        .extra_in_claim("TSP")
        .extra_in_claim("DYNAMIC_DCI_TS")
        // to AMS
        .extra_out_claim("SWITCH_OUT")
        // to/from pair
        .extra_in_claim("OUTB_B_IN")
        .extra_out_claim("OUTB_B")
        .extra_in_claim("TSTATEIN")
        .extra_out_claim("TSTATEOUT")
        // to/from differential out
        .extra_out_claim("O_B")
        .extra_out_claim("TSTATEB")
        // to/from differential in
        .extra_in_claim("TMDS_IBUF_OUT")
        .extra_out_claim("DRIVER_BOT_IBUF")
        // dummies
        .extra_in_claim("TSDI");
    if is_single {
        bel = bel.extra_in_dummy("IO");
    } else {
        bel = bel.extra_in_claim("IO");
    }
    let has_clkpad = matches!(idx, 0 | 2 | 21 | 23);
    if has_clkpad {
        bel = bel.extra_out("DOUT").skip_out(bcls::HRIOB::CLKPAD);
        bel.verify_net(&[bel.wire("DOUT"), bel.wire("CLKPAD")]);
    } else {
        bel = bel.extra_out_claim("DOUT");
    }

    if let Some(pidx) = pidx {
        let obel = bcrd.bel(bslots::HRIOB[pidx]);
        bel.claim_pip(bel.wire("OUTB_B_IN"), bel.bel_wire(obel, "OUTB_B"));
        bel.claim_pip(bel.wire("TSTATEIN"), bel.bel_wire(obel, "TSTATEOUT"));
    } else {
        bel.claim_pip(bel.wire("OUTB_B_IN"), bel.wire("OUTB_B"));
    }

    let srow = if idx < 13 { bcrd.row } else { bcrd.row + 15 };
    let obel_bs = bcrd.with_row(srow).bel(bslots::BITSLICE[idx % 13]);

    bel.claim_pip(bel.wire("OP"), bel.wire_far("OP"));
    bel.claim_pip(bel.wire("TSP"), bel.wire_far("TSP"));
    bel.verify_net(&[
        bel.wire("DYNAMIC_DCI_TS"),
        bel.bel_wire(obel_bs, "DYN_DCI_OUT"),
    ]);
    bel.verify_net(&[bel.wire("DOUT"), bel.bel_wire_far(obel_bs, "RX_D")]);
    bel.verify_net(&[bel.wire_far("OP"), bel.bel_wire(obel_bs, "TX_Q")]);
    bel.verify_net(&[bel.wire_far("TSP"), bel.bel_wire(obel_bs, "TX_T_OUT")]);

    let crd = IoCoord::Hpio(HpioCoord {
        cell: bcrd.with_row(chip.row_reg_rclk(reg)),
        iob: hid,
    });
    let cfg = edev.cfg_io[bcrd.die].get_by_right(&crd);
    let is_cfg = match cfg {
        Some(SharedCfgPad::PerstN0) => false,
        Some(SharedCfgPad::PerstN1) => false,
        Some(SharedCfgPad::EmCclk) => false,
        Some(_) => true,
        None => false,
    };
    let is_ams = matches!(cfg, Some(SharedCfgPad::I2cSda | SharedCfgPad::I2cSclk))
        && edev.kind == ChipKind::Ultrascale;
    if is_ams {
        bel.claim_pip(bel.wire("TSDI"), bel.wire_far("TSDI"));
    } else if !edev.is_cut && !is_cfg && bel.wire("TSDI") != bel.wire_far("TSDI") {
        bel.claim_pip(bel.wire("TSDI"), bel.wire_far("TSDI"));
        bel.claim_net(&[bel.wire_far("TSDI")]);
    }

    if !bel.naming.pins["SWITCH_OUT"].pips.is_empty() {
        bel.claim_pip(bel.wire_far("SWITCH_OUT"), bel.wire("SWITCH_OUT"));
        let fidx = if bcrd.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
            idx
        } else {
            idx + 26
        };

        let ams_idx = match fidx {
            4 | 5 => Some(15),
            6 | 7 => Some(7),
            8 | 9 => Some(14),
            10 | 11 => Some(6),
            13 | 14 => Some(13),
            15 | 16 => Some(5),
            17 | 18 => Some(12),
            19 | 20 => Some(4),
            30 | 31 => Some(11),
            32 | 33 => Some(3),
            34 | 35 => Some(10),
            36 | 37 => Some(2),
            39 | 40 => Some(9),
            41 | 42 => Some(1),
            43 | 44 => Some(8),
            45 | 46 => Some(0),
            _ => None,
        };

        if let Some(ams_idx) = ams_idx {
            let scol = chip.col_cfg();
            let srow = chip.row_ams();
            let obel = bcrd.with_cr(scol, srow).bel(bslots::SYSMON);
            bel.verify_net(&[
                bel.wire_far("SWITCH_OUT"),
                bel.bel_wire_far(
                    obel,
                    &format!("V{pn}_AUX{ams_idx}", pn = if is_m { 'P' } else { 'N' }),
                ),
            ]);
        }
    }
    bel.commit();
}

fn verify_hriodiffin(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HRIODIFFINBUF")
        .extra_in_claim("LVDS_IN_P")
        .extra_in_claim("LVDS_IN_N")
        .extra_out_claim("LVDS_IBUF_OUT")
        .extra_out_claim("LVDS_IBUF_OUT_B");
    let idx = bslots::HRIOB_DIFF_IN.index_of(bcrd.slot).unwrap();
    let (pidx, nidx) = if idx < 6 {
        (idx * 2, idx * 2 + 1)
    } else {
        (idx * 2 + 1, idx * 2 + 2)
    };

    let obel_p = bcrd.bel(bslots::HRIOB[pidx]);
    let obel_n = bcrd.bel(bslots::HRIOB[nidx]);
    bel.claim_pip(
        bel.wire("LVDS_IN_P"),
        bel.bel_wire(obel_p, "DRIVER_BOT_IBUF"),
    );
    bel.claim_pip(
        bel.wire("LVDS_IN_N"),
        bel.bel_wire(obel_n, "DRIVER_BOT_IBUF"),
    );
    bel.claim_pip(
        bel.bel_wire(obel_p, "TMDS_IBUF_OUT"),
        bel.wire("LVDS_IBUF_OUT"),
    );
    bel.claim_pip(
        bel.bel_wire(obel_n, "TMDS_IBUF_OUT"),
        bel.wire("LVDS_IBUF_OUT_B"),
    );
    bel.commit();
}

fn verify_hriodiffout(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HRIODIFFOUTBUF")
        .extra_in_claim("O_B")
        .extra_in_claim("TSTATEB")
        .extra_out_claim("AOUT")
        .extra_out_claim("BOUT");
    let idx = bslots::HRIOB_DIFF_OUT.index_of(bcrd.slot).unwrap();
    let (pidx, nidx) = if idx < 6 {
        (idx * 2, idx * 2 + 1)
    } else {
        (idx * 2 + 1, idx * 2 + 2)
    };

    let obel_p = bcrd.bel(bslots::HRIOB[pidx]);
    let obel_n = bcrd.bel(bslots::HRIOB[nidx]);
    bel.claim_pip(bel.wire("O_B"), bel.bel_wire(obel_p, "O_B"));
    bel.claim_pip(bel.wire("TSTATEB"), bel.bel_wire(obel_p, "TSTATEB"));
    bel.claim_pip(bel.bel_wire(obel_p, "IO"), bel.wire("AOUT"));
    bel.claim_pip(bel.bel_wire(obel_n, "IO"), bel.wire("BOUT"));

    bel.commit();
}

fn verify_bufg_gt(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let tcrd = edev.bel_tile(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::HSADC | tcls::HSDAC | tcls::RFADC | tcls::RFDAC
    ) {
        bel = bel
            .extra_in_claim("DIV0")
            .extra_in_claim("DIV1")
            .extra_in_claim("DIV2");
        bel.claim_pip(bel.wire("DIV0"), bel.wire("DIV0_DUMMY"));
        bel.claim_pip(bel.wire("DIV1"), bel.wire("DIV1_DUMMY"));
        bel.claim_pip(bel.wire("DIV2"), bel.wire("DIV2_DUMMY"));
        bel.claim_net(&[bel.wire("DIV0_DUMMY")]);
        bel.claim_net(&[bel.wire("DIV1_DUMMY")]);
        bel.claim_net(&[bel.wire("DIV2_DUMMY")]);
    }
    if !edev
        .disabled
        .contains(&DisabledPart::GtBufs(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
}

fn verify_bufg_gt_sync(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let tcrd = edev.bel_tile(bcrd);
    let idx = bslots::BUFG_GT_SYNC.index_of(bcrd.slot).unwrap();

    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::HSADC | tcls::HSDAC | tcls::RFADC | tcls::RFDAC
    ) && (4..14).contains(&idx)
    {
        bel = bel
            .extra_in_claim("CLK_IN")
            .extra_in_claim("RST_IN")
            .extra_in_claim("CE_IN");
    }
    if edev[tcrd].class == tcls::GTM && matches!(idx, 6 | 13) {
        bel = bel.extra_in_dummy("CLK_IN");
    }

    let skip = (edev
        .disabled
        .contains(&DisabledPart::GtmSpareBufs(bcrd.die, bcrd.col, reg))
        && matches!(idx, 6 | 13))
        || edev
            .disabled
            .contains(&DisabledPart::GtBufs(bcrd.die, bcrd.col, reg));
    if !skip {
        bel.commit();
    }
}

fn verify_gt_channel(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let (common, channel) = if tcid == tcls::GTF {
        (bslots::GTF_COMMON, bslots::GTF_CHANNEL)
    } else if edev.kind == ChipKind::Ultrascale {
        (bslots::GT3_COMMON, bslots::GT3_CHANNEL)
    } else {
        (bslots::GT4_COMMON, bslots::GT4_CHANNEL)
    };
    let idx = channel.index_of(bcrd.slot).unwrap();
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let kind = match tcid {
        tcls::GTH => match edev.kind {
            ChipKind::Ultrascale => "GTHE3_CHANNEL",
            ChipKind::UltrascalePlus => "GTHE4_CHANNEL",
        },
        tcls::GTY => match edev.kind {
            ChipKind::Ultrascale => "GTYE3_CHANNEL",
            ChipKind::UltrascalePlus => "GTYE4_CHANNEL",
        },
        tcls::GTF => "GTF_CHANNEL",
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        // from COMMON
        .extra_in_claim("MGTREFCLK0")
        .extra_in_claim("MGTREFCLK1")
        .extra_in_claim("NORTHREFCLK0")
        .extra_in_claim("NORTHREFCLK1")
        .extra_in_claim("SOUTHREFCLK0")
        .extra_in_claim("SOUTHREFCLK1")
        .extra_in_claim("QDCMREFCLK0_INT")
        .extra_in_claim("QDCMREFCLK1_INT")
        .extra_in_claim("QDPLL0CLK0P_INT")
        .extra_in_claim("QDPLL1CLK0P_INT")
        .extra_in_claim("RING_OSC_CLK_INT")
        // to COMMON
        .extra_out_claim("RXRECCLKOUT");

    let obel = bcrd.bel(common);
    let cross_qdpll = edev.kind == ChipKind::Ultrascale && tcid == tcls::GTH;
    for (pin, opin) in [
        ("MGTREFCLK0", "MGTREFCLK0"),
        ("MGTREFCLK1", "MGTREFCLK1"),
        ("NORTHREFCLK0", "NORTHREFCLK0"),
        ("NORTHREFCLK1", "NORTHREFCLK1"),
        ("SOUTHREFCLK0", "SOUTHREFCLK0"),
        ("SOUTHREFCLK1", "SOUTHREFCLK1"),
        ("QDCMREFCLK0_INT", "QDCMREFCLK_INT_0"),
        ("QDCMREFCLK1_INT", "QDCMREFCLK_INT_1"),
        (
            "QDPLL0CLK0P_INT",
            if cross_qdpll {
                "QDPLLCLK0P_1"
            } else {
                "QDPLLCLK0P_0"
            },
        ),
        (
            "QDPLL1CLK0P_INT",
            if cross_qdpll {
                "QDPLLCLK0P_0"
            } else {
                "QDPLLCLK0P_1"
            },
        ),
    ] {
        bel.claim_pip(bel.wire(pin), bel.bel_wire(obel, opin));
    }
    bel.claim_pip(
        bel.wire("RING_OSC_CLK_INT"),
        bel.bel_wire(obel, &format!("SARC_CLK{idx}")),
    );

    if !edev
        .disabled
        .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
}

fn verify_gt_common(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let channel = if tcid == tcls::GTF {
        bslots::GTF_CHANNEL
    } else if edev.kind == ChipKind::Ultrascale {
        bslots::GT3_CHANNEL
    } else {
        bslots::GT4_CHANNEL
    };

    let chip = edev.chips[bcrd.die];
    let kind = match tcid {
        tcls::GTH => match edev.kind {
            ChipKind::Ultrascale => "GTHE3_COMMON",
            ChipKind::UltrascalePlus => "GTHE4_COMMON",
        },
        tcls::GTY => match edev.kind {
            ChipKind::Ultrascale => "GTYE3_COMMON",
            ChipKind::UltrascalePlus => "GTYE4_COMMON",
        },
        tcls::GTF => "GTF_COMMON",
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        // from CHANNEL
        .extra_in_claim("RXRECCLK0")
        .extra_in_claim("RXRECCLK1")
        .extra_in_claim("RXRECCLK2")
        .extra_in_claim("RXRECCLK3")
        // to CHANNEL, broadcast
        .extra_out_claim("QDCMREFCLK_INT_0")
        .extra_out_claim("QDCMREFCLK_INT_1")
        .extra_out_claim("QDPLLCLK0P_0")
        .extra_out_claim("QDPLLCLK0P_1")
        .extra_out_claim("MGTREFCLK0")
        .extra_out_claim("MGTREFCLK1")
        // to CHANNEL, specific
        .extra_out_claim("SARC_CLK0")
        .extra_out_claim("SARC_CLK1")
        .extra_out_claim("SARC_CLK2")
        .extra_out_claim("SARC_CLK3")
        // from self and up/down
        .extra_in_claim("COM0_REFCLKOUT0")
        .extra_in_claim("COM0_REFCLKOUT1")
        .extra_in_claim("COM0_REFCLKOUT2")
        .extra_in_claim("COM0_REFCLKOUT3")
        .extra_in_claim("COM0_REFCLKOUT4")
        .extra_in_claim("COM0_REFCLKOUT5")
        .extra_in_claim("COM2_REFCLKOUT0")
        .extra_in_claim("COM2_REFCLKOUT1")
        .extra_in_claim("COM2_REFCLKOUT2")
        .extra_in_claim("COM2_REFCLKOUT3")
        .extra_in_claim("COM2_REFCLKOUT4")
        .extra_in_claim("COM2_REFCLKOUT5");
    let reg = chip.row_to_reg(bcrd.row);

    for i in 0..4 {
        let obel = bcrd.bel(channel[i]);
        bel.claim_pip(
            bel.wire(&format!("RXRECCLK{i}")),
            bel.bel_wire(obel, "RXRECCLKOUT"),
        );
    }

    for (i, pin) in [
        (0, "MGTREFCLK0"),
        (1, "MGTREFCLK1"),
        (2, "NORTHREFCLK0"),
        (3, "NORTHREFCLK1"),
        (4, "SOUTHREFCLK0"),
        (5, "SOUTHREFCLK1"),
    ] {
        for j in [0, 2] {
            bel.claim_pip(bel.wire(&format!("COM{j}_REFCLKOUT{i}")), bel.wire(pin));
        }
    }

    for pin in [
        "CLKOUT_NORTH0",
        "CLKOUT_NORTH1",
        "CLKOUT_SOUTH0",
        "CLKOUT_SOUTH1",
    ] {
        bel.claim_pip(bel.wire(pin), bel.wire("VCC"));
        bel.claim_pip(bel.wire(pin), bel.wire("MGTREFCLK0"));
        bel.claim_pip(bel.wire(pin), bel.wire("MGTREFCLK1"));
    }
    bel.claim_pip(bel.wire("CLKOUT_NORTH0"), bel.wire("NORTHREFCLK0"));
    bel.claim_pip(bel.wire("CLKOUT_NORTH1"), bel.wire("NORTHREFCLK1"));
    bel.claim_pip(bel.wire("CLKOUT_SOUTH0"), bel.wire("SOUTHREFCLK0"));
    bel.claim_pip(bel.wire("CLKOUT_SOUTH1"), bel.wire("SOUTHREFCLK1"));

    if let Some(obel_n) = edev.bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, bcrd.slot) {
        bel.verify_net(&[
            bel.wire("SOUTHREFCLK0"),
            bel.bel_wire(obel_n, "CLKOUT_SOUTH0"),
        ]);
        bel.verify_net(&[
            bel.wire("SOUTHREFCLK1"),
            bel.bel_wire(obel_n, "CLKOUT_SOUTH1"),
        ]);
        bel.claim_net(&[bel.wire("CLKOUT_NORTH0")]);
        bel.claim_net(&[bel.wire("CLKOUT_NORTH1")]);
    } else {
        bel.vrf.claim_dummy_in(bel.wire("SOUTHREFCLK0"));
        bel.vrf.claim_dummy_in(bel.wire("SOUTHREFCLK1"));
        bel.vrf.claim_dummy_out(bel.wire("CLKOUT_NORTH0"));
        bel.vrf.claim_dummy_out(bel.wire("CLKOUT_NORTH1"));
    }
    if let Some(obel_s) = edev.bel_delta(bcrd.cell, 0, -(Chip::ROWS_PER_REG as i32), bcrd.slot) {
        bel.verify_net(&[
            bel.wire("NORTHREFCLK0"),
            bel.bel_wire(obel_s, "CLKOUT_NORTH0"),
        ]);
        bel.verify_net(&[
            bel.wire("NORTHREFCLK1"),
            bel.bel_wire(obel_s, "CLKOUT_NORTH1"),
        ]);
        bel.claim_net(&[bel.wire("CLKOUT_SOUTH0")]);
        bel.claim_net(&[bel.wire("CLKOUT_SOUTH1")]);
    } else {
        bel.vrf.claim_dummy_in(bel.wire("NORTHREFCLK0"));
        bel.vrf.claim_dummy_in(bel.wire("NORTHREFCLK1"));
        bel.vrf.claim_dummy_out(bel.wire("CLKOUT_SOUTH0"));
        bel.vrf.claim_dummy_out(bel.wire("CLKOUT_SOUTH1"));
    }

    if !edev
        .disabled
        .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
}

fn verify_gtm_dual(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let mut bel = vrf
        .verify_bel(bcrd)
        // to/from GTM_REFCLK
        .extra_out_claim("HROW_TEST_CK_SA")
        .extra_out_claim("REFCLKPDB_SA")
        .extra_out_claim("RXRECCLK0_INT")
        .extra_out_claim("RXRECCLK1_INT")
        .extra_in_claim("MGTREFCLK_CLEAN")
        .extra_in_claim("REFCLK2HROW")
        // from s/n
        .extra_in_claim("REFCLK_DIST2PLL0")
        // dummy ins
        .extra_in_claim("RCALSEL0")
        .extra_in_claim("RCALSEL1");
    if is_cut_u(edev, bcrd.die, bcrd.row) {
        bel = bel.extra_in("REFCLK_DIST2PLL1");
    } else {
        bel = bel.extra_in_claim("REFCLK_DIST2PLL1");
    }
    for (pin, opin) in [
        ("REFCLK2HROW", "GTM_REFCLK_REFCLK2HROW"),
        ("MGTREFCLK_CLEAN", "GTM_REFCLK_MGTREFCLK_CLEAN"),
        ("GTM_REFCLK_HROW_TEST_CK_FS", "HROW_TEST_CK_SA"),
        ("GTM_REFCLK_REFCLKPDB_SA", "REFCLKPDB_SA"),
        ("GTM_REFCLK_RXRECCLK0_INT", "RXRECCLK0_INT"),
        ("GTM_REFCLK_RXRECCLK1_INT", "RXRECCLK1_INT"),
    ] {
        bel.claim_pip(bel.wire(pin), bel.wire(opin));
    }
    if let Some(obel_n) = edev.bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, bcrd.slot) {
        bel.verify_net(&[
            bel.wire("REFCLK_DIST2PLL1"),
            bel.bel_wire(obel_n, "SOUTHCLKOUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire("NORTHCLKOUT")]);
    }
    if let Some(obel_s) = edev.bel_delta(bcrd.cell, 0, -(Chip::ROWS_PER_REG as i32), bcrd.slot) {
        bel.verify_net(&[
            bel.wire("REFCLK_DIST2PLL0"),
            bel.bel_wire(obel_s, "NORTHCLKOUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire("SOUTHCLKOUT")]);
    }
    bel.claim_pip(bel.wire("NORTHCLKOUT"), bel.wire("REFCLK_DIST2PLL0"));
    bel.claim_pip(
        bel.wire("NORTHCLKOUT"),
        bel.wire("GTM_REFCLK_MGTREFCLK_CLEAN"),
    );
    bel.claim_pip(bel.wire("NORTHCLKOUT"), bel.wire("NORTHCLKOUT_DUMMY0"));
    bel.claim_pip(bel.wire("NORTHCLKOUT"), bel.wire("NORTHCLKOUT_DUMMY1"));
    bel.claim_pip(bel.wire("SOUTHCLKOUT"), bel.wire("REFCLK_DIST2PLL1"));
    bel.claim_pip(
        bel.wire("SOUTHCLKOUT"),
        bel.wire("GTM_REFCLK_MGTREFCLK_CLEAN"),
    );
    bel.claim_pip(bel.wire("SOUTHCLKOUT"), bel.wire("SOUTHCLKOUT_DUMMY0"));
    bel.claim_pip(bel.wire("SOUTHCLKOUT"), bel.wire("SOUTHCLKOUT_DUMMY1"));
    bel.claim_net(&[bel.wire("NORTHCLKOUT_DUMMY0")]);
    bel.claim_net(&[bel.wire("NORTHCLKOUT_DUMMY1")]);
    bel.claim_net(&[bel.wire("SOUTHCLKOUT_DUMMY0")]);
    bel.claim_net(&[bel.wire("SOUTHCLKOUT_DUMMY1")]);
    if !edev
        .disabled
        .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
    let bel = vrf
        .verify_bel(bcrd)
        .sub(1)
        .kind("GTM_REFCLK")
        .skip_auto()
        .extra_in_claim("GTM_REFCLK_HROW_TEST_CK_FS")
        .extra_in_claim("GTM_REFCLK_REFCLKPDB_SA")
        .extra_in_claim("GTM_REFCLK_RXRECCLK0_INT")
        .extra_in_claim("GTM_REFCLK_RXRECCLK1_INT")
        .extra_in_claim("GTM_REFCLK_RXRECCLK2_INT")
        .extra_in_claim("GTM_REFCLK_RXRECCLK3_INT")
        .extra_out_claim("GTM_REFCLK_MGTREFCLK_CLEAN")
        .extra_out_claim("GTM_REFCLK_REFCLK2HROW");
    if !edev
        .disabled
        .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
}

fn verify_hsadc_hsdac(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let chip = edev.chips[bcrd.die];
    let reg = chip.row_to_reg(bcrd.row);
    let is_rf = matches!(tcid, tcls::RFDAC | tcls::RFADC);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(edev.db.tile_classes.key(tcid))
        // to/from north/south
        .extra_in_claim("SYSREF_IN_SOUTH_P")
        .extra_in_claim("SYSREF_IN_NORTH_P")
        .extra_out_claim("SYSREF_OUT_SOUTH_P")
        .extra_out_claim("SYSREF_OUT_NORTH_P");
    for pin in ["SYSREF_IN_SOUTH_P", "SYSREF_IN_NORTH_P"] {
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
    }
    if is_rf {
        bel = bel
            .extra_in_claim("CLK_DISTR_IN_NORTH")
            .extra_in_claim("CLK_DISTR_IN_SOUTH")
            .extra_out_claim("CLK_DISTR_OUT_NORTH")
            .extra_out_claim("CLK_DISTR_OUT_SOUTH")
            .extra_in_claim("T1_ALLOWED_NORTH")
            .extra_out_claim("T1_ALLOWED_SOUTH");
        for pin in [
            "CLK_DISTR_IN_NORTH",
            "CLK_DISTR_IN_SOUTH",
            "T1_ALLOWED_NORTH",
        ] {
            bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
        }
    }

    let oslot = match bcrd.slot {
        bslots::HSADC => bslots::HSDAC,
        bslots::HSDAC => bslots::HSADC,
        _ => unreachable!(),
    };

    if let Some(obel_n) = edev
        .bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, bcrd.slot)
        .or_else(|| edev.bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, oslot))
    {
        bel.verify_net(&[
            bel.wire_far("SYSREF_IN_NORTH_P"),
            bel.bel_wire(obel_n, "SYSREF_OUT_SOUTH_P"),
        ]);
        if is_rf {
            if obel_n.slot == bcrd.slot {
                bel.verify_net(&[
                    bel.wire_far("CLK_DISTR_IN_NORTH"),
                    bel.bel_wire(obel_n, "CLK_DISTR_OUT_SOUTH"),
                ]);
            }
            bel.verify_net(&[
                bel.wire_far("T1_ALLOWED_NORTH"),
                bel.bel_wire(obel_n, "T1_ALLOWED_SOUTH"),
            ]);
        }
    } else if chip.row_to_reg(bcrd.row).to_idx() == chip.regs - 1 {
        bel.verify_net(&[
            bel.wire_far("SYSREF_IN_NORTH_P"),
            bel.wire("SYSREF_OUT_NORTH_P"),
        ]);
        if is_rf {
            bel.verify_net(&[
                bel.wire_far("CLK_DISTR_IN_NORTH"),
                bel.wire("CLK_DISTR_OUT_NORTH"),
            ]);
        }
    }
    if let Some(obel_s) = edev
        .bel_delta(bcrd.cell, 0, -(Chip::ROWS_PER_REG as i32), bcrd.slot)
        .or_else(|| edev.bel_delta(bcrd.cell, 0, -(Chip::ROWS_PER_REG as i32), oslot))
    {
        bel.verify_net(&[
            bel.wire_far("SYSREF_IN_SOUTH_P"),
            bel.bel_wire(obel_s, "SYSREF_OUT_NORTH_P"),
        ]);
        if is_rf {
            if obel_s.slot == bcrd.slot {
                bel.verify_net(&[
                    bel.wire_far("CLK_DISTR_IN_SOUTH"),
                    bel.bel_wire(obel_s, "CLK_DISTR_OUT_NORTH"),
                ]);
            } else {
                bel.verify_net(&[
                    bel.wire_far("CLK_DISTR_IN_SOUTH"),
                    bel.wire("T1_ALLOWED_SOUTH"),
                ]);
            }
        }
    } else {
        bel.verify_net(&[
            bel.wire_far("SYSREF_IN_SOUTH_P"),
            bel.wire("SYSREF_OUT_SOUTH_P"),
        ]);
        if is_rf {
            bel.verify_net(&[
                bel.wire_far("CLK_DISTR_IN_SOUTH"),
                bel.wire("CLK_DISTR_OUT_SOUTH"),
            ]);
        }
    }

    if !edev
        .disabled
        .contains(&DisabledPart::Gt(bcrd.die, bcrd.col, reg))
    {
        bel.commit();
    }
}

fn verify_bel(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    match bcrd.slot {
        bslots::INT
        | bslots::INTF_INT
        | bslots::INTF_TESTMUX
        | bslots::XIPHY_INT
        | bslots::GT_INT
        | bslots::CMT_INT
        | bslots::RCLK_INT
        | bslots::RCLK_V_INT
        | bslots::RCLK_SPLITTER
        | bslots::RCLK_BEL_INT
        | bslots::RCLK_IOB_INT => (),
        bslots::SLICE => verify_slice(edev, vrf, bcrd),
        _ if bslots::DSP.contains(bcrd.slot) => verify_dsp(edev, vrf, bcrd),
        bslots::BRAM => verify_bram(edev, vrf, bcrd),
        _ if bslots::HARD_SYNC.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::URAM.contains(bcrd.slot) => verify_uram(edev, vrf, bcrd),
        _ if bslots::LAGUNA.contains(bcrd.slot) => verify_laguna(edev, vrf, bcrd),

        bslots::PCIE3 | bslots::PCIE4 => verify_pcie(edev, vrf, bcrd),
        bslots::CMAC => verify_cmac(edev, vrf, bcrd),
        bslots::ILKN => verify_ilkn(edev, vrf, bcrd),
        bslots::BLI_HBM_APB_INTF
        | bslots::BLI_HBM_AXI_INTF
        | bslots::PMV
        | bslots::PMV2
        | bslots::PMVIOB
        | bslots::MTBF3
        | bslots::HDIO_BIAS => vrf.verify_bel(bcrd).commit(),
        bslots::CFGIO => vrf.verify_bel(bcrd).kind("CFGIO_SITE").commit(),
        bslots::HPIO_ZMATCH => vrf.verify_bel(bcrd).kind("HPIO_ZMATCH_BLK_HCLK").commit(),
        bslots::HPIO_PRBS => vrf.verify_bel(bcrd).kind("HPIO_RCLK_PRBS").commit(),
        _ if bslots::HDLOGIC_CSSD.contains(bcrd.slot) => {
            let idx = bslots::HDLOGIC_CSSD.index_of(bcrd.slot).unwrap();
            let kind = if edev[edev.bel_tile(bcrd)].class == tcls::HDIOS {
                if idx >= 2 {
                    "HDIOS_HDLOGIC_CSSD_TOP"
                } else {
                    "HDIOS_HDLOGIC_CSSD"
                }
            } else {
                "HDLOGIC_CSSD"
            };
            vrf.verify_bel(bcrd).kind(kind).commit()
        }
        _ if bslots::HDIO_VREF.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        bslots::FE => {
            if !edev.disabled.contains(&DisabledPart::Sdfec) {
                let mut bel = vrf.verify_bel(bcrd);
                for i in 0..470 {
                    bel = bel.extra_in(format!("XIL_UNCONN_CLK{i}"));
                }
                for i in 0..1862 {
                    bel = bel.extra_in(format!("XIL_UNCONN_IN{i}"));
                }
                for i in 0..711 {
                    bel = bel.extra_out(format!("XIL_UNCONN_OUT{i}"));
                }
                bel.commit();
            }
        }
        bslots::DFE_PRACH => verify_dfe(edev, vrf, bcrd),
        _ if bslots::DFE_FFT.contains(bcrd.slot)
            || bslots::DFE_FIR.contains(bcrd.slot)
            || bslots::DFE_MATRIX_FIR.contains(bcrd.slot)
            || bslots::DFE_NL_FIR.contains(bcrd.slot)
            || bslots::DFE_DUC_DDC.contains(bcrd.slot)
            || bslots::DFE_CFR.contains(bcrd.slot)
            || bslots::DFE_PQ_DECIM.contains(bcrd.slot)
            || bslots::DFE_PQ_INTERP.contains(bcrd.slot) =>
        {
            verify_dfe(edev, vrf, bcrd)
        }
        bslots::CFG => vrf
            .verify_bel(bcrd)
            .kind(if edev.chips[bcrd.die].config_kind.is_csec() {
                "CSEC_SITE"
            } else {
                "CONFIG_SITE"
            })
            .commit(),
        bslots::SYSMON => verify_sysmon(edev, vrf, bcrd),
        bslots::PS => verify_ps(edev, vrf, bcrd),
        bslots::VCU => {
            if !edev.disabled.contains(&DisabledPart::Vcu) {
                vrf.verify_bel(bcrd).commit()
            }
        }
        bslots::ABUS_SWITCH_CFG | bslots::ABUS_SWITCH_CMT => verify_abus_switch(edev, vrf, bcrd),
        _ if bslots::ABUS_SWITCH_GT.contains(bcrd.slot)
            || bslots::ABUS_SWITCH_HDIO.contains(bcrd.slot)
            || bslots::ABUS_SWITCH_HPIO.contains(bcrd.slot)
            || bslots::ABUS_SWITCH_HRIO.contains(bcrd.slot)
            || bslots::ABUS_SWITCH_HBM.contains(bcrd.slot)
            || bslots::ABUS_SWITCH_XP5IO.contains(bcrd.slot) =>
        {
            verify_abus_switch(edev, vrf, bcrd)
        }
        _ if bslots::VBUS_SWITCH.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),

        bslots::BUFCE_LEAF_X16_S | bslots::BUFCE_LEAF_X16_N => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFCE_LEAF_S.contains(bcrd.slot)
            || bslots::BUFCE_LEAF_N.contains(bcrd.slot) =>
        {
            verify_bufce_leaf(vrf, bcrd)
        }

        _ if bslots::BUFCE_ROW_CMT.contains(bcrd.slot)
            || bslots::BUFCE_ROW_RCLK.contains(bcrd.slot) =>
        {
            verify_bufce_row(edev, vrf, bcrd)
        }
        _ if bslots::GCLK_TEST_BUF_CMT.contains(bcrd.slot)
            || bslots::GCLK_TEST_BUF_RCLK.contains(bcrd.slot) =>
        {
            vrf.verify_bel(bcrd).kind("GCLK_TEST_BUFE3").commit()
        }

        _ if bslots::BUFG_PS.contains(bcrd.slot) => {
            vrf.verify_bel(bcrd).commit();
        }

        _ if bslots::HDIOB_DIFF_IN.contains(bcrd.slot) => verify_hdiodiffin(edev, vrf, bcrd),
        _ if bslots::HDIOB.contains(bcrd.slot) => verify_hdiob(edev, vrf, bcrd),
        _ if bslots::HDIOLOGIC.contains(bcrd.slot) => verify_hdiologic(vrf, bcrd),
        _ if bslots::BUFGCE_HDIO.contains(bcrd.slot) => {
            vrf.verify_bel(bcrd).kind("BUFGCE_HDIO").commit()
        }

        _ if bslots::BUFGCE_DIV.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFGCE.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFGCTRL.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        bslots::MMCM => verify_mmcm(edev, vrf, bcrd),
        _ if bslots::PLL.contains(bcrd.slot) => verify_pll(edev, vrf, bcrd),
        _ if bslots::HBM_REF_CLK.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),

        _ if bslots::BITSLICE_T.contains(bcrd.slot) => verify_bitslice_t(edev, vrf, bcrd),
        _ if bslots::BITSLICE_CONTROL.contains(bcrd.slot) => {
            verify_bitslice_control(edev, vrf, bcrd)
        }
        _ if bslots::BITSLICE.contains(bcrd.slot) => verify_bitslice(edev, vrf, bcrd),
        bslots::XIPHY_BYTE => verify_xiphy_byte(edev, vrf, bcrd),

        _ if bslots::HPIOB_DCI.contains(bcrd.slot) => verify_hpio_dci(edev, vrf, bcrd),
        _ if bslots::HPIOB_DIFF_IN.contains(bcrd.slot) => verify_hpiodiffin(edev, vrf, bcrd),
        _ if bslots::HPIOB_DIFF_OUT.contains(bcrd.slot) => verify_hpiodiffout(edev, vrf, bcrd),
        _ if bslots::HPIOB.contains(bcrd.slot) => verify_hpiob(edev, vrf, bcrd),
        bslots::HPIO_VREF => verify_hpio_vref(vrf, bcrd),
        bslots::HPIO_BIAS => vrf.verify_bel(bcrd).kind("BIAS").commit(),

        _ if bslots::HRIOB_DIFF_IN.contains(bcrd.slot) => verify_hriodiffin(vrf, bcrd),
        _ if bslots::HRIOB_DIFF_OUT.contains(bcrd.slot) => verify_hriodiffout(vrf, bcrd),
        _ if bslots::HRIOB.contains(bcrd.slot) => verify_hriob(edev, vrf, bcrd),

        _ if bslots::XP5IOB.contains(bcrd.slot) => (),
        _ if bslots::XP5IO_BITSLICE.contains(bcrd.slot) => (),
        _ if bslots::XP5IOB_PAIR.contains(bcrd.slot) => xp5io::verify_xp5iob_pair(vrf, bcrd),
        _ if bslots::XP5IO_VREF.contains(bcrd.slot) => xp5io::verify_xp5io_vref(vrf, bcrd),
        _ if bslots::X5PHY.contains(bcrd.slot) => xp5io::verify_x5phy(vrf, bcrd),
        bslots::LPDDRMC => xp5io::verify_lpddrmc(vrf, bcrd),

        _ if bslots::BUFG_GT_SYNC.contains(bcrd.slot) => verify_bufg_gt_sync(edev, vrf, bcrd),
        _ if bslots::BUFG_GT.contains(bcrd.slot) => verify_bufg_gt(edev, vrf, bcrd),
        _ if bslots::GT3_CHANNEL.contains(bcrd.slot)
            || bslots::GT4_CHANNEL.contains(bcrd.slot)
            || bslots::GTF_CHANNEL.contains(bcrd.slot) =>
        {
            verify_gt_channel(edev, vrf, bcrd)
        }
        bslots::GT3_COMMON | bslots::GT4_COMMON | bslots::GTF_COMMON => {
            verify_gt_common(edev, vrf, bcrd)
        }
        bslots::GTM_DUAL => verify_gtm_dual(edev, vrf, bcrd),
        bslots::HSADC | bslots::HSDAC => verify_hsadc_hsdac(edev, vrf, bcrd),

        _ => println!("MEOW {}", bcrd.to_string(edev.db)),
    }
}

fn verify_extra(edev: &ExpandedDevice, vrf: &mut Verifier) {
    for w in [
        "CLK_VDISTR_FT0",
        "CLK_VROUTE_FT0",
        "CLK_VDISTR_FT0_0",
        "CLK_VROUTE_FT0_0",
        "CLK_VDISTR_FT0_1",
        "CLK_VROUTE_FT0_1",
        "CASMBIST12IN",
        "SYNC_CLK_B_TOP0",
        "SYNC_CLK_B_TOP1",
        "SYNC_CLK_B_TOP2",
        "SYNC_CLK_B_TOP3",
        "SYNC_CLK_TOP0",
        "SYNC_CLK_TOP1",
        "SYNC_CLK_TOP2",
        "SYNC_CLK_TOP3",
        "SYNC_DIN_TOP0",
        "SYNC_DIN_TOP1",
        "SYNC_DIN_TOP2",
        "SYNC_DIN_TOP3",
        "SYNC_SR_TOP0",
        "SYNC_SR_TOP1",
        "SYNC_SR_TOP2",
        "SYNC_SR_TOP3",
        "SYNC_DOUT_BOT0",
        "SYNC_DOUT_BOT1",
        "SYNC_DOUT_BOT2",
        "SYNC_DOUT_BOT3",
        "SYNC_DOUT_TERM0",
        "SYNC_DOUT_TERM1",
        "SYNC_DOUT_TERM2",
        "SYNC_DOUT_TERM3",
        "CLOCK_DR_FT0",
        "SHIFT_DR_FT0",
        "UPDATE_DR_FT0",
        "UPDATE_DR_FT1",
        "EXTEST_FT0",
        "EXTEST_FT1",
        "INTEST_FT0",
        "INTEST_FT1",
        "MISR_JTAG_LOAD_FT0",
        "AC_MODE_FT0",
        "RESET_TAP_B_FT0",
        "FST_CFG_B_FT0",
        "FST_CFG_B_FT1",
        "GPWRDWN_B_FT0",
        "GPWRDWN_B_FT1",
        "GTS_CFG_B_FT0",
        "GTS_CFG_B_FT1",
        "GTS_USR_B_FT0",
        "GTS_USR_B_FT1",
        "POR_B_FT0",
        "POR_B_FT1",
        "CLOCK_DR_IN",
        "SHIFT_DR_IN",
        "RESET_TAP_BP",
        "EXTEST_SMPL_FT0",
        "HSWAPEN_FT0",
        "IO_TO_CTR_FT0_0",
        "IO_TO_CTR_FT0_1",
        "IO_TO_CTR_FT0_2",
        "IO_TO_CTR_FT0_3",
        "REMOTE_DIODE_FN_FT0_0",
        "REMOTE_DIODE_FN_FT0_1",
        "REMOTE_DIODE_FN_FT0_2",
        "REMOTE_DIODE_FP_FT0",
        "REMOTE_DIODE_SN_FT0_0",
        "REMOTE_DIODE_SN_FT0_1",
        "REMOTE_DIODE_SN_FT0_2",
        "REMOTE_DIODE_SP_FT0",
        "DIODE_N_OPT",
        "DIODE_P_OPT",
        "GHIGH_B_PSS_0",
        "GHIGH_B_PSS_1",
        "CFG_RESET_B_FT1_0",
        "CFG_RESET_B_FT1_1",
        "GPWRDWN_B_FT1_0",
        "GPWRDWN_B_FT1_1",
        "POR_B_FT1_0",
        "POR_B_FT1_1",
        "IDCODE19",
        "PSS_ALTO_CORE_0_IDCODE15_PIN",
        "PSS_ALTO_CORE_0_IDCODE16_PIN",
        "PSS_ALTO_CORE_0_IDCODE17_PIN",
        "PSS_ALTO_CORE_0_IDCODE18_PIN",
        "PSS_ALTO_CORE_0_IDCODE20_PIN",
        "PSS_ALTO_CORE_0_IDCODE21_PIN",
        "PSS_ALTO_CORE_0_IDCODE28_PIN",
        "PSS_ALTO_CORE_0_IDCODE29_PIN",
        "PSS_ALTO_CORE_0_PS_VERSION_0_PIN",
        "PSS_ALTO_CORE_0_PS_VERSION_1_PIN",
        "PSS_ALTO_CORE_0_IDCODE15",
        "PSS_ALTO_CORE_0_IDCODE16",
        "PSS_ALTO_CORE_0_IDCODE17",
        "PSS_ALTO_CORE_0_IDCODE18",
        "PSS_ALTO_CORE_0_IDCODE20",
        "PSS_ALTO_CORE_0_IDCODE21",
        "PSS_ALTO_CORE_0_IDCODE28",
        "PSS_ALTO_CORE_0_IDCODE29",
        "PSS_ALTO_CORE_0_IDCODE30",
        "PSS_ALTO_CORE_0_IDCODE31",
        "PSS_ALTO_CORE_0_PS_VERSION_0",
        "PSS_ALTO_CORE_0_PS_VERSION_1",
        "PSS_ALTO_CORE_0_PS_VERSION_2",
        "PSS_ALTO_CORE_0_PS_VERSION_3",
    ] {
        vrf.kill_stub_in_cond(w);
    }
    if edev.kind == ChipKind::Ultrascale {
        for i in 0..104 {
            vrf.kill_stub_in_cond_tk("INT_TERM_L_IO", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_TERM_L", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_K3_TERM_L_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_VH_TERM_L_FT", &format!("GND_WIRE{i}"));
        }
        for i in 0..4 {
            vrf.kill_stub_in_cond_tk("INT_IBRK_LEFT_L_FT", &format!("GND_WIRE{i}"));
        }
        vrf.kill_stub_in_cond_tk("CFRM_L_TERM_T", "GND_WIRE0");
        vrf.kill_stub_in_cond_tk("CFRM_L_TERM_T", "GND_WIRE2");
    } else {
        for i in 0..84 {
            vrf.kill_stub_in_cond_tk("INT_INTF_LEFT_TERM_IO_FT", &format!("GND_WIRE{i}"));
        }
        for i in 0..104 {
            vrf.kill_stub_in_cond_tk("INT_INTF_RIGHT_TERM_HDIO_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("INT_INTF_RIGHT_TERM_HDIO_FT", &format!("WESTBUS_NOCONN{i}"));
        }
        for i in 0..10785 {
            vrf.kill_stub_in_cond_tk("HDIO_OUTER_TERM_R_FT", &format!("GND_WIRE{i}"));
        }
        for i in 0..3120 {
            vrf.kill_stub_in_cond_tk("HPIO_TERM_L_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_TERM_L_DA6_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_AUX_IO_TERM_L_BOT_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_AUX_IO_TERM_L_TOP_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_CFG_TERM_L_BOT_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_CFG_TERM_L_TOP_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_AUX_IO_TERM_L_BOT_DA6_FT", &format!("GND_WIRE{i}"));
            vrf.kill_stub_in_cond_tk("HPIO_AUX_IO_TERM_L_TOP_DA6_FT", &format!("GND_WIRE{i}"));
        }
        for i in 0..24 {
            vrf.kill_stub_in_cond(&format!("OPTION2IOB_TX_DATA_OPT{i}"));
            vrf.kill_stub_in_cond(&format!("OPTION2IOB_PERSIST_MODE_EN_OPT{i}"));
        }
        for i in 0..94 {
            vrf.kill_stub_in_cond_tk("AMS", &format!("UNCONN_INOUTS{i}"));
        }
        for i in 0..32 {
            vrf.kill_stub_in_cond_tk("CFG_CONFIG", &format!("PCAP_WDATA{i}"));
        }
        for w in [
            "PCAP_CS_B",
            "PCAP_PR",
            "PCAP_WDATA_CLK",
            "PCAP_RDWR_B",
            "PCFG_BOOT0",
            "PCFG_BOOT1",
            "PCFG_BOOT2",
            "PCFG_GSR",
            "PCFG_GTS",
            "PCFG_INIT_B",
            "PCFG_JTAG_CFG_DISABLE0",
            "PCFG_JTAG_CFG_DISABLE1",
            "PCFG_POR_CNT_4K",
            "PCFG_PROG",
            "PCFG_TCK",
            "PCFG_TDI",
            "PCFG_TMS",
            "UNCONN_INOUTS0",
            "UNCONN_INOUTS1",
            "UNCONN_INOUTS2",
            "UNCONN_INOUTS3",
            "UNCONN_INOUTS4",
            "UNCONN_INOUTS5",
            "UNCONN_INOUTS6",
            "UNCONN_INOUTS7",
            "UNCONN_INOUTS8",
        ] {
            vrf.kill_stub_in_cond_tk("CFG_CONFIG", w);
        }
        for i in 0..61 {
            if i == 30 {
                continue;
            }
            for j in 0..32 {
                vrf.kill_stub_in_cond_tk("HDIO_OUTER_TERM_R_FT", &format!("LOGIC_OUTS_{i}_{j}"));
            }
        }
        vrf.kill_stub_in_cond("OPTION2IOB_PERSIST_MODE_EN_VR1_OPT");
        vrf.kill_stub_in_cond("OPTION2IOB_PERSIST_MODE_EN_VR2_OPT");
        vrf.kill_stub_in_cond("OPTION2IOB_TX_DATA_VR1_OPT");
        vrf.kill_stub_in_cond("OPTION2IOB_TX_DATA_VR2_OPT");
        vrf.kill_stub_in_cond("CLK_FROM_EXT_LOW_SOUTH");
        vrf.kill_stub_in_cond("CLK_FROM_EXT_UPP_SOUTH");
        vrf.kill_stub_in_cond("CLK_FROM_EXT_LOW_NORTH");
        vrf.kill_stub_in_cond("CLK_FROM_EXT_UPP_NORTH");
        vrf.kill_stub_in_cond_tk("RCLK_AMS_CFGIO", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_AMS_CFGIO", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("RCLK_AMS_CFGIO", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("CFG_CONFIG", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("CFG_CONFIG", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("CFG_CONFIG", "VCC_WIRE4");
        vrf.kill_stub_in_cond_tk("CSEC_CONFIG_FT", "VCC_WIRE1603");
        vrf.kill_stub_in_cond_tk("CSEC_CONFIG_FT", "VCC_WIRE1604");
        vrf.kill_stub_in_cond_tk("RCLK_INT_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_INT_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_CLEL_L_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_CLEL_L_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_CLEL_R_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_CLEL_R_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_CLEM_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_CLEM_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_CLEM_CLKBUF_L", "VCC_WIRE4");
        vrf.kill_stub_in_cond_tk("RCLK_HDIO", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_HDIO", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_DSP_INTF_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_DSP_INTF_CLKBUF_L", "VCC_WIRE9");
        vrf.kill_stub_in_cond_tk("RCLK_RCLK_XIPHY_INNER_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_INTF_R_IBRK_L", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("RCLK_INTF_L_IBRK_IO_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_RCLK_INTF_LEFT_IBRK_FE_L_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_INTF_L_IBRK_PCIE4_L", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_INTF_L_TERM_GT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("RCLK_RCLK_INTF_LEFT_TERM_IO_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("HPIO_RIGHT_TERM_T", "VCC_WIRE");
        vrf.kill_stub_in_cond_tk("HPIO_RIGHT_TERM_T", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("HPIO_RIGHT_TERM_T", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("HPIO_RIGHT_TERM_T", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("HPIO_L_TERM_T", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("HPIO_L_TERM_T", "GND_WIRE2");
        vrf.kill_stub_in_cond_tk("HPIO_L_TERM_T", "GND_WIRE5");
        vrf.kill_stub_in_cond_tk("HPIO_HPIO_LEFT_TERM_T_L_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("HPIO_HPIO_LEFT_TERM_T_L_FT", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("HPIO_HPIO_LEFT_TERM_T_L_FT", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE0");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE2");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE4");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE5");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE6");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE7");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE8");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE9");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE10");
        vrf.kill_stub_in_cond_tk("HDIOLC_HDIOL_TOP_RIGHT_CFG_FT", "GND_WIRE11");
        vrf.kill_stub_in_cond_tk("PCIE4_TERM_T_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("PCIE4_TERM_T_FT", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("PCIE4_TERM_T_FT", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("CMAC_CMAC_TERM_T_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("CMAC_CMAC_TERM_T_FT", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("CMAC_CMAC_TERM_T_FT", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "VCC_WIRE4");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "VCC_WIRE6");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "GND_WIRE0");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "GND_WIRE2");
        vrf.kill_stub_in_cond_tk("HDIO_BOT_RIGHT", "GND_WIRE5");
        vrf.kill_stub_in_cond_tk("CMAC", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("CMAC", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("CMAC", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("CMAC", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("PCIE4_PCIE4_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("PCIE4_PCIE4_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("PCIE4_PCIE4_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("PCIE4_PCIE4_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("PCIE4C_PCIE4C_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("PCIE4C_PCIE4C_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("PCIE4C_PCIE4C_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("PCIE4C_PCIE4C_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("ILKN_ILKN_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("ILKN_ILKN_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("ILKN_ILKN_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("ILKN_ILKN_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("FE_FE_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEA_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEA_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEA_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEA_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEB_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEB_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEB_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEB_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEC_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEC_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILED_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILED_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEE_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEE_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEF_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEF_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEG_FT", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEG_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEG_FT", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("DFE_DFE_TILEG_FT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "VCC_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "VCC_WIRE5");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "VCC_WIRE7");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "GND_WIRE1");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "GND_WIRE3");
        vrf.kill_stub_in_cond_tk("HDIO_TOP_RIGHT", "GND_WIRE6");
        vrf.kill_stub_in_cond_tk("VCU_VCU_FT", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("VCU_VCU_FT", "VCC_WIRE4");
        vrf.kill_stub_in_cond_tk("VCU_VCU_FT", "GHIGH_B_PSS");
        vrf.kill_stub_in_cond_tk("VCU_VCU_FT", "PL_VCU_GLOBAL_CFG_RESET_B");
        vrf.kill_stub_in_cond_tk("BRAM", "VCC_WIRE0");
        vrf.kill_stub_in_cond_tk("BRAM", "VCC_WIRE1");
        vrf.kill_stub_in_cond_tk("BRAM", "VCC_WIRE2");
        vrf.kill_stub_in_cond_tk("BRAM", "VCC_WIRE3");
    }
}

fn verify_pre(vrf: &mut Verifier) {
    for (tcid, bslot) in [
        (tcls::RCLK_HDIO, bslots::RCLK_BEL_INT),
        (tcls::RCLK_HDIOS, bslots::RCLK_BEL_INT),
        (tcls::RCLK_HDIOL, bslots::RCLK_BEL_INT),
        (tcls::RCLK_HDIOL_CMT, bslots::RCLK_BEL_INT),
        (tcls::RCLK_HPIO, bslots::RCLK_IOB_INT),
        (tcls::RCLK_HRIO, bslots::RCLK_IOB_INT),
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

    let tcls = &vrf.db.tile_classes[tcls::XP5IO];
    if let Some(bel) = tcls.bels.get(bslots::LPDDRMC) {
        let BelInfo::Bel(bel) = bel else {
            unreachable!()
        };
        for &tcrd in &vrf.grid.tile_index[tcls::XP5IO] {
            for (opin, ipin) in [
                (bcls::LPDDRMC::IOB_PUDC_B_O, bcls::LPDDRMC::IOB_PUDC_B),
                (
                    bcls::LPDDRMC::IJTAG_RESET_TAP_O,
                    bcls::LPDDRMC::IJTAG_RESET_TAP,
                ),
                (
                    bcls::LPDDRMC::IJTAG_CAPTURE_DR_O,
                    bcls::LPDDRMC::IJTAG_CAPTURE_DR,
                ),
                (
                    bcls::LPDDRMC::IJTAG_SELECT_DR_O,
                    bcls::LPDDRMC::IJTAG_SELECT_DR,
                ),
            ] {
                let wo = *bel.outputs[opin].iter().next().unwrap();
                let wo = vrf.grid.tile_wire(tcrd, wo);
                let wi = bel.inputs[ipin].wire();
                let wi = vrf.grid.tile_wire(tcrd, wi);
                vrf.alias_wire(wo, wi);
            }
        }
    }
}

pub fn verify_device(endev: &ExpandedNamedDevice, rd: &Part, pb: &ProgressBar) {
    let mut vrf = Verifier::new(rd, &endev.ngrid, pb);
    verify_pre(&mut vrf);
    vrf.prep_int_wires();
    vrf.handle_int();
    for (tcrd, tile) in endev.ngrid.egrid.tiles() {
        let tcls = &endev.ngrid.egrid.db[tile.class];
        for slot in tcls.bels.ids() {
            verify_bel(endev.edev, &mut vrf, tcrd.bel(slot));
        }
    }
    verify_extra(endev.edev, &mut vrf);
    vrf.finish();
}
