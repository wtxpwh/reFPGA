use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{BelInfo, SwitchBoxItem, WireSlotIdExt},
    dir::Dir,
    grid::{BelCoord, RowId},
};
use prjcombine_re_xilinx_naming::db::RawTileId;
use prjcombine_re_xilinx_naming_virtex4::ExpandedNamedDevice;
use prjcombine_re_xilinx_rawdump::{Part, Source};
use prjcombine_re_xilinx_rdverify::{RawWireCoord, Verifier};
use prjcombine_virtex4::{
    chip::{ColumnKind, DisabledPart},
    defs::{
        bcls::{self, BRAM_V5 as BRAM, GTX_CHANNEL},
        bslots, cslots,
        virtex7::{ccls, tcls, wires},
    },
    expanded::ExpandedDevice,
};

fn verify_slice(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::SLICE.index_of(bcrd.slot).unwrap();
    let kind = if edev.chips[bcrd.die].columns[bcrd.col] == ColumnKind::ClbLM && idx == 0 {
        "SLICEM"
    } else {
        "SLICEL"
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_in("CIN")
        .extra_out_claim("COUT");
    if let Some(obel) = edev.bel_carry_prev(bcrd) {
        bel.claim_net(&[bel.wire("CIN"), bel.bel_wire_far(obel, "COUT")]);
        bel.claim_pip(bel.bel_wire_far(obel, "COUT"), bel.bel_wire(obel, "COUT"));
    } else {
        bel.claim_net(&[bel.wire("CIN")]);
    }
    if bel.vrf.rd.source == Source::Vivado {
        if edev.bel_carry_next(bcrd).is_some() {
            // ok
        } else {
            bel.claim_net(&[bel.wire_far("COUT")]);
            bel.claim_pip(bel.wire_far("COUT"), bel.wire("COUT"));
        }
    }
    bel.commit();
}

fn verify_dsp(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("DSP48E1");
    let mut pairs = vec![];
    pairs.push(("MULTSIGNIN".to_string(), "MULTSIGNOUT".to_string()));
    pairs.push(("CARRYCASCIN".to_string(), "CARRYCASCOUT".to_string()));
    for i in 0..30 {
        pairs.push((format!("ACIN{i}"), format!("ACOUT{i}")));
    }
    for i in 0..18 {
        pairs.push((format!("BCIN{i}"), format!("BCOUT{i}")));
    }
    for i in 0..48 {
        pairs.push((format!("PCIN{i}"), format!("PCOUT{i}")));
    }
    for (ipin, opin) in &pairs {
        bel = bel.extra_in(ipin).extra_out_claim(opin);
        if bcrd.slot == bslots::DSP[0] {
            if bel.vrf.rd.source == Source::ISE && edev.bel_carry_prev(bcrd).is_none() {
                bel.claim_net(&[bel.wire(ipin)]);
            }
        } else {
            bel.claim_net(&[bel.wire(ipin)]);
            let obel = bcrd.bel(bslots::DSP[0]);
            bel.claim_pip(bel.wire(ipin), bel.bel_wire(obel, opin));

            if let Some(obel) = edev.bel_carry_next(bcrd) {
                bel.claim_pip(bel.wire_far(opin), bel.wire(opin));
                bel.claim_net(&[bel.bel_wire(obel, ipin), bel.wire_far(opin)]);
            } else if bel.vrf.rd.source == Source::Vivado {
                bel.claim_pip(bel.wire_far(opin), bel.wire(opin));
                bel.claim_net(&[bel.wire_far(opin)]);
            }
        }
    }
    bel.commit();
}

fn verify_bram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("RAMBFIFO36E1")
        .extra_in("CASCADEINA")
        .extra_in("CASCADEINB")
        .extra_out_claim("CASCADEOUTA")
        .extra_out_claim("CASCADEOUTB")
        .rename_in(BRAM::CLKAL, "CLKARDCLKL")
        .rename_in(BRAM::CLKAU, "CLKARDCLKU")
        .rename_in(BRAM::CLKBL, "CLKBWRCLKL")
        .rename_in(BRAM::CLKBU, "CLKBWRCLKU")
        .rename_in(BRAM::ENAL, "ENARDENL")
        .rename_in(BRAM::ENAU, "ENARDENU")
        .rename_in(BRAM::ENBL, "ENBWRENL")
        .rename_in(BRAM::ENBU, "ENBWRENU")
        .rename_in(BRAM::REGCEAL, "REGCEAREGCEL")
        .rename_in(BRAM::REGCEAU, "REGCEAREGCEU")
        .rename_in(BRAM::REGCLKAL, "REGCLKARDRCLKL")
        .rename_in(BRAM::REGCLKAU, "REGCLKARDRCLKU")
        .rename_in(BRAM::RSTRAMAL, "RSTRAMARSTRAMLRST")
        .rename_in(BRAM::RSTRAMAU, "RSTRAMARSTRAMU")
        .rename_in(BRAM::RSTREGAL, "RSTREGARSTREGL")
        .rename_in(BRAM::RSTREGAU, "RSTREGARSTREGU");
    for i in 0..16 {
        bel = bel
            .rename_in(BRAM::ADDRAL[i], format!("ADDRARDADDRL{i}"))
            .rename_in(BRAM::ADDRBL[i], format!("ADDRBWRADDRL{i}"));
    }
    for i in 0..15 {
        bel = bel
            .rename_in(BRAM::ADDRAU[i], format!("ADDRARDADDRU{i}"))
            .rename_in(BRAM::ADDRBU[i], format!("ADDRBWRADDRU{i}"));
    }
    for i in 0..8 {
        bel = bel
            .rename_in(BRAM::WEBL[i], format!("WEBWEL{i}"))
            .rename_in(BRAM::WEBU[i], format!("WEBWEU{i}"));
    }
    for i in 0..32 {
        bel = bel
            .rename_in([BRAM::DIAL, BRAM::DIAU][i % 2][i / 2], format!("DIADI{i}"))
            .rename_in([BRAM::DIBL, BRAM::DIBU][i % 2][i / 2], format!("DIBDI{i}"))
            .rename_out([BRAM::DOAL, BRAM::DOAU][i % 2][i / 2], format!("DOADO{i}"))
            .rename_out([BRAM::DOBL, BRAM::DOBU][i % 2][i / 2], format!("DOBDO{i}"));
    }
    for i in 0..4 {
        bel = bel
            .rename_in(
                [BRAM::DIPAL, BRAM::DIPAU][i % 2][i / 2],
                format!("DIPADIP{i}"),
            )
            .rename_in(
                [BRAM::DIPBL, BRAM::DIPBU][i % 2][i / 2],
                format!("DIPBDIP{i}"),
            )
            .rename_out(
                [BRAM::DOPAL, BRAM::DOPAU][i % 2][i / 2],
                format!("DOPADOP{i}"),
            )
            .rename_out(
                [BRAM::DOPBL, BRAM::DOPBU][i % 2][i / 2],
                format!("DOPBDOP{i}"),
            );
    }
    if let Some(obel) = edev.bel_carry_prev(bcrd) {
        for (ipin, opin) in [("CASCADEINA", "CASCADEOUTA"), ("CASCADEINB", "CASCADEOUTB")] {
            bel.claim_net(&[bel.wire(ipin), bel.bel_wire_far(obel, opin)]);
            bel.claim_pip(bel.bel_wire_far(obel, opin), bel.bel_wire(obel, opin));
        }
    } else if bel.vrf.rd.source == Source::ISE {
        for ipin in ["CASCADEINA", "CASCADEINB"] {
            bel.claim_net(&[bel.wire(ipin)]);
        }
    }
    if bel.vrf.rd.source == Source::Vivado && edev.bel_carry_next(bcrd).is_none() {
        for opin in ["CASCADEOUTA", "CASCADEOUTB"] {
            bel.claim_net(&[bel.wire_far(opin)]);
            bel.claim_pip(bel.wire_far(opin), bel.wire(opin));
        }
    }
    bel.commit();

    let mut bel = vrf.verify_bel(bcrd).sub(1).skip_auto().kind("FIFO18E1");
    for (hpin, fpin) in [
        ("RDCLK", "CLKAL"),
        ("WRCLK", "CLKBL"),
        ("RDEN", "ENAL"),
        ("WREN", "ENBL"),
        ("RDRCLK", "REGCLKAL"),
        ("REGCLKB", "REGCLKBL"),
        ("RSTREG", "RSTREGAL"),
        ("RSTREGB", "RSTREGBL"),
        ("RST", "RSTRAMAL"),
        ("RSTRAMB", "RSTRAMBL"),
        ("REGCE", "REGCEAL"),
        ("REGCEB", "REGCEBL"),
        ("ADDRATIEHIGH0", "ADDRAL0"),
        ("ADDRBTIEHIGH0", "ADDRBL0"),
        ("ADDRATIEHIGH1", "ADDRAL15"),
        ("ADDRBTIEHIGH1", "ADDRBL15"),
    ] {
        bel = bel.extra_in_claim_rename(hpin, format!("H0_{hpin}"));
        bel.claim_pip(bel.wire(&format!("H0_{hpin}")), bel.wire_far(fpin));
    }
    for pin in [
        "FULL",
        "EMPTY",
        "ALMOSTFULL",
        "ALMOSTEMPTY",
        "RDERR",
        "WRERR",
    ] {
        bel = bel.extra_out_claim_rename(pin, format!("H0_{pin}"));
        bel.claim_pip(bel.wire_far(pin), bel.wire(&format!("H0_{pin}")));
    }
    for (hpin, fpin, width) in [
        ("WEA", "WEAL", 4),
        ("WEBWE", "WEBL", 8),
        ("DIADI", "DIAL", 16),
        ("DIPADIP", "DIPAL", 2),
        ("DIBDI", "DIBL", 16),
        ("DIPBDIP", "DIPBL", 2),
    ] {
        for i in 0..width {
            bel = bel.extra_in_claim_rename(format!("{hpin}{i}"), format!("H0_{hpin}{i}"));
            bel.claim_pip(
                bel.wire(&format!("H0_{hpin}{i}")),
                bel.wire_far(&format!("{fpin}{i}")),
            );
        }
    }
    for (hpin, fpin, width, delta) in [
        ("DO", "DOAL", 16, 0),
        ("DOP", "DOPAL", 2, 0),
        ("DO", "DOBL", 16, 16),
        ("DOP", "DOPBL", 2, 2),
    ] {
        for i in 0..width {
            let ii = i + delta;
            bel = bel.extra_out_claim_rename(format!("{hpin}{ii}"), format!("H0_{hpin}{ii}"));
            bel.claim_pip(
                bel.wire_far(&format!("{fpin}{i}")),
                bel.wire(&format!("H0_{hpin}{ii}")),
            );
        }
    }
    for i in 0..12 {
        bel = bel
            .extra_out_claim_rename(format!("RDCOUNT{i}"), format!("H0_RDCOUNT{i}"))
            .extra_out_claim_rename(format!("WRCOUNT{i}"), format!("H0_WRCOUNT{i}"));
        bel.claim_pip(
            bel.wire_far(&format!("RDCOUNT{i}")),
            bel.wire(&format!("H0_RDCOUNT{i}")),
        );
        bel.claim_pip(
            bel.wire_far(&format!("WRCOUNT{i}")),
            bel.wire(&format!("H0_WRCOUNT{i}")),
        );
    }
    for i in 0..14 {
        bel = bel
            .extra_in_claim_rename(format!("ADDRARDADDR{i}"), format!("H0_ADDRARDADDR{i}"))
            .extra_in_claim_rename(format!("ADDRBWRADDR{i}"), format!("H0_ADDRBWRADDR{i}"));
        bel.claim_pip(
            bel.wire(&format!("H0_ADDRARDADDR{i}")),
            bel.wire_far(&format!("ADDRAL{ii}", ii = i + 1)),
        );
        bel.claim_pip(
            bel.wire(&format!("H0_ADDRBWRADDR{i}")),
            bel.wire_far(&format!("ADDRBL{ii}", ii = i + 1)),
        );
    }
    bel.commit();

    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(2)
        .skip_auto()
        .kind("RAMB18E1")
        .extra_out_dummy("FULL")
        .extra_out_dummy("EMPTY")
        .extra_out_dummy("ALMOSTFULL")
        .extra_out_dummy("ALMOSTEMPTY")
        .extra_out_dummy("RDERR")
        .extra_out_dummy("WRERR");
    for (hpin, fpin) in [
        ("CLKARDCLK", "CLKAU"),
        ("CLKBWRCLK", "CLKBU"),
        ("ENARDEN", "ENAU"),
        ("ENBWREN", "ENBU"),
        ("REGCLKARDRCLK", "REGCLKAU"),
        ("REGCLKB", "REGCLKBU"),
        ("RSTREGARSTREG", "RSTREGAU"),
        ("RSTREGB", "RSTREGBU"),
        ("RSTRAMARSTRAM", "RSTRAMAU"),
        ("RSTRAMB", "RSTRAMBU"),
        ("REGCEAREGCE", "REGCEAU"),
        ("REGCEB", "REGCEBU"),
        ("ADDRATIEHIGH0", "ADDRAU0"),
        ("ADDRBTIEHIGH0", "ADDRBU0"),
        ("ADDRATIEHIGH1", "ADDRAL15"),
        ("ADDRBTIEHIGH1", "ADDRBL15"),
    ] {
        bel = bel.extra_in_claim_rename(hpin, format!("H1_{hpin}"));
        bel.claim_pip(bel.wire(&format!("H1_{hpin}")), bel.wire_far(fpin));
    }
    for (hpin, fpin, width) in [
        ("WEA", "WEAU", 4),
        ("WEBWE", "WEBU", 8),
        ("DIADI", "DIAU", 16),
        ("DIPADIP", "DIPAU", 2),
        ("DIBDI", "DIBU", 16),
        ("DIPBDIP", "DIPBU", 2),
    ] {
        for i in 0..width {
            bel = bel.extra_in_claim_rename(format!("{hpin}{i}"), format!("H1_{hpin}{i}"));
            bel.claim_pip(
                bel.wire(&format!("H1_{hpin}{i}")),
                bel.wire_far(&format!("{fpin}{i}")),
            );
        }
    }
    for (hpin, fpin, width) in [
        ("DOADO", "DOAU", 16),
        ("DOPADOP", "DOPAU", 2),
        ("DOBDO", "DOBU", 16),
        ("DOPBDOP", "DOPBU", 2),
    ] {
        for i in 0..width {
            bel = bel.extra_out_claim_rename(format!("{hpin}{i}"), format!("H1_{hpin}{i}"));
            bel.claim_pip(
                bel.wire_far(&format!("{fpin}{i}")),
                bel.wire(&format!("H1_{hpin}{i}")),
            );
        }
    }
    for i in 0..14 {
        bel = bel
            .extra_in_claim_rename(format!("ADDRARDADDR{i}"), format!("H1_ADDRARDADDR{i}"))
            .extra_in_claim_rename(format!("ADDRBWRADDR{i}"), format!("H1_ADDRBWRADDR{i}"));
        bel.claim_pip(
            bel.wire(&format!("H1_ADDRARDADDR{i}")),
            bel.wire_far(&format!("ADDRAU{ii}", ii = i + 1)),
        );
        bel.claim_pip(
            bel.wire(&format!("H1_ADDRBWRADDR{i}")),
            bel.wire_far(&format!("ADDRBU{ii}", ii = i + 1)),
        );
    }
    for i in 0..12 {
        bel = bel
            .extra_out_dummy(format!("RDCOUNT{i}"))
            .extra_out_dummy(format!("WRCOUNT{i}"));
    }
    bel.commit();
}

fn verify_pmvbram(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("PMVBRAM");
    let tcrd = bel.vrf.grid.bel_tile(bcrd);
    if bel.vrf.grid[tcrd].class == tcls::PMVBRAM_NC {
        bel = bel
            .extra_out_claim("O")
            .extra_out_claim("ODIV2")
            .extra_out_claim("ODIV4");
        if bel.vrf.rd.source == Source::Vivado {
            bel = bel
                .extra_in("SELECT1")
                .extra_in("SELECT2")
                .extra_in("SELECT3")
                .extra_in("SELECT4");
        } else {
            bel = bel
                .extra_in_claim("SELECT1")
                .extra_in_claim("SELECT2")
                .extra_in_claim("SELECT3")
                .extra_in_claim("SELECT4");
        }
    }
    bel.commit();
}

fn verify_hclk(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let has_s = edev.has_bel(bcrd.delta(0, -1).bel(bslots::INT));
    let mut bel = vrf.verify_bel(bcrd);
    for i in 0..12 {
        bel.vrf.pin_int_wire(
            bel.wire(&format!("HCLK{i}")),
            edev.resolve_tile_wire(tcrd, wires::HCLK_ROW[i].cell(1))
                .unwrap(),
        );
        if has_s {
            bel.vrf.pin_int_wire(
                bel.wire(&format!("LCLK{i}_S")),
                edev.resolve_tile_wire(tcrd, wires::LCLK[i].cell(0))
                    .unwrap(),
            );
        }
        bel.vrf.pin_int_wire(
            bel.wire(&format!("LCLK{i}_N")),
            edev.resolve_tile_wire(tcrd, wires::LCLK[i].cell(1))
                .unwrap(),
        );
    }
    for i in 0..4 {
        bel.vrf.pin_int_wire(
            bel.wire(&format!("RCLK{i}")),
            edev.resolve_tile_wire(tcrd, wires::RCLK_ROW[i].cell(1))
                .unwrap(),
        );
    }
    for i in 0..6 {
        for sn in ['S', 'N'] {
            if sn == 'S' && !has_s {
                continue;
            }
            for j in 0..8 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("HCLK{j}")),
                );
            }
            for j in 8..12 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("HCLK{j}_I")),
                );
            }
            for j in 0..4 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("RCLK{j}_I")),
                );
            }
        }
    }
    for i in 6..12 {
        for sn in ['S', 'N'] {
            if sn == 'S' && !has_s {
                continue;
            }
            for j in 0..8 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("HCLK{j}_I")),
                );
            }
            for j in 8..12 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("HCLK{j}")),
                );
            }
            for j in 0..4 {
                bel.claim_pip(
                    bel.wire(&format!("LCLK{i}_{sn}")),
                    bel.wire(&format!("RCLK{j}")),
                );
            }
        }
    }
    for i in 8..12 {
        bel.claim_net(&[
            bel.wire(&format!("HCLK{i}_O")),
            bel.wire(&format!("HCLK{i}_I")),
        ]);
        bel.claim_pip(
            bel.wire(&format!("HCLK{i}_O")),
            bel.wire(&format!("HCLK{i}")),
        );
    }
    for i in 0..4 {
        bel.claim_net(&[
            bel.wire(&format!("RCLK{i}_O")),
            bel.wire(&format!("RCLK{i}_I")),
        ]);
        bel.claim_pip(
            bel.wire(&format!("RCLK{i}_O")),
            bel.wire(&format!("RCLK{i}")),
        );
    }
    for i in 0..8 {
        bel.claim_net(&[
            bel.wire(&format!("HCLK{i}_O")),
            bel.wire(&format!("HCLK{i}_I")),
        ]);
        bel.claim_pip(
            bel.wire(&format!("HCLK{i}_O")),
            bel.wire(&format!("HCLK{i}")),
        );
    }
}

fn verify_clk_rebuf(vrf: &mut Verifier, bcrd: BelCoord) {
    for i in 0..32 {
        let kind = if vrf
            .ngrid
            .get_bel_name_sub(bcrd, i)
            .unwrap()
            .starts_with("BUFG")
        {
            "BUFG_LB"
        } else {
            "GCLK_TEST_BUF"
        };
        let mut bel = vrf.verify_bel(bcrd).sub(i).kind(kind).skip_auto();
        let ii = i / 2;
        if i.is_multiple_of(2) {
            bel = bel
                .extra_in_claim_rename("CLKIN", format!("BUF{ii}_S_CLKIN"))
                .extra_out_rename("CLKOUT", format!("BUF{ii}_S_CLKOUT"));
            bel.claim_pip(
                bel.wire(&format!("BUF{ii}_S_CLKIN")),
                bel.wire(&format!("GCLK{i}_S")),
            );
        } else {
            bel = bel
                .extra_in_claim_rename("CLKIN", format!("BUF{ii}_N_CLKIN"))
                .extra_out_rename("CLKOUT", format!("BUF{ii}_N_CLKOUT"));
            bel.claim_pip(
                bel.wire(&format!("BUF{ii}_N_CLKIN")),
                bel.wire(&format!("GCLK{i}_N")),
            );
        }
        bel.commit();
    }
}

fn verify_clk_hrow(vrf: &mut Verifier, bcrd: BelCoord) {
    for i in 0..32 {
        let clkin = &format!("GCLK{i}_TEST_CLKIN");
        let clkout = &format!("GCLK{i}_TEST_CLKOUT");

        let mut bel = vrf
            .verify_bel(bcrd)
            .sub(i)
            .kind("GCLK_TEST_BUF")
            .skip_auto()
            .extra_in_claim_rename("CLKIN", clkin)
            .extra_out_claim_rename("CLKOUT", clkout);
        bel.claim_pip(bel.wire(clkin), bel.wire(&format!("GCLK{i}_TEST_IN")));
        bel.claim_net(&[bel.wire(&format!("GCLK{i}_TEST_OUT"))]);
        bel.claim_pip(bel.wire(&format!("GCLK{i}_TEST_OUT")), bel.wire(clkout));
        bel.claim_pip(
            bel.wire(&format!("GCLK_TEST{i}")),
            bel.wire(&format!("GCLK{i}_TEST_OUT")),
        );
        bel.claim_pip(
            bel.wire(&format!("GCLK_TEST{ii}", ii = i ^ 1)),
            bel.wire(&format!("GCLK{i}_TEST_OUT")),
        );
        bel.commit();
    }

    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(32)
        .kind("GCLK_TEST_BUF")
        .skip_auto()
        .extra_in_claim_rename("CLKIN", "HCLK_TEST_W_CLKIN")
        .extra_out_claim_rename("CLKOUT", "HCLK_TEST_W_CLKOUT");
    bel.claim_pip(bel.wire("HCLK_TEST_W_CLKIN"), bel.wire("HCLK_TEST_IN_W"));
    bel.claim_pip(bel.wire("HCLK_TEST_OUT_W"), bel.wire("HCLK_TEST_W_CLKOUT"));
    bel.commit();

    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(33)
        .kind("GCLK_TEST_BUF")
        .skip_auto()
        .extra_in_claim_rename("CLKIN", "HCLK_TEST_E_CLKIN")
        .extra_out_claim_rename("CLKOUT", "HCLK_TEST_E_CLKOUT");
    bel.claim_pip(bel.wire("HCLK_TEST_E_CLKIN"), bel.wire("HCLK_TEST_IN_E"));
    bel.claim_pip(bel.wire("HCLK_TEST_OUT_E"), bel.wire("HCLK_TEST_E_CLKOUT"));
    bel.commit();
}

fn verify_dci(vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_claim("DCIDATA")
        .extra_out_claim("DCIADDRESS0")
        .extra_out_claim("DCIADDRESS1")
        .extra_out_claim("DCIADDRESS2")
        .extra_out_claim("DCIIOUPDATE")
        .extra_out_claim("DCIREFIOUPDATE")
        .extra_out_claim("DCISCLK")
        .commit();
}

fn verify_ilogic(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::ILOGIC.index_of(bcrd.slot).unwrap();
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let is_hp = matches!(tcid, tcls::IO_HP_PAIR | tcls::IO_HP_S | tcls::IO_HP_N);
    let is_single = !matches!(tcid, tcls::IO_HP_PAIR | tcls::IO_HR_PAIR);
    let kind = if is_hp { "ILOGICE2" } else { "ILOGICE3" };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .skip_out(bcls::ILOGIC::CLKPAD)
        .extra_in_claim("OCLK")
        .extra_in_claim("OCLKB")
        .extra_in_claim("D")
        .extra_in_claim("DDLY")
        .extra_in_claim("OFB")
        .extra_in_claim("TFB")
        .extra_out_claim("SHIFTOUT1")
        .extra_out_claim("SHIFTOUT2")
        .extra_in_dummy("REV");
    if bcrd.slot != bslots::ILOGIC[0] || is_single {
        bel = bel.extra_in_dummy("SHIFTIN1").extra_in_dummy("SHIFTIN2");
    } else {
        bel = bel.extra_in_claim("SHIFTIN1").extra_in_claim("SHIFTIN2");
    }

    let obel_ologic = bcrd.bel(bslots::OLOGIC[idx]);

    bel.claim_pip(bel.wire("OCLK"), bel.bel_wire(obel_ologic, "CLK"));
    bel.claim_pip(bel.wire("OCLKB"), bel.bel_wire(obel_ologic, "CLK"));
    bel.claim_pip(bel.wire("OCLKB"), bel.bel_wire(obel_ologic, "CLKB"));
    bel.claim_pip(bel.wire("OFB"), bel.bel_wire(obel_ologic, "OFB"));
    bel.claim_pip(bel.wire("TFB"), bel.bel_wire(obel_ologic, "TFB"));

    let obel_idelay = bcrd.bel(bslots::IDELAY[idx]);
    bel.claim_pip(bel.wire("DDLY"), bel.bel_wire(obel_idelay, "DATAOUT"));

    let obel_iob = bcrd.bel(bslots::IOB[idx]);
    bel.claim_pip(bel.wire("D"), bel.wire("IOB_I_BUF"));
    bel.claim_net(&[bel.wire("IOB_I_BUF")]);
    bel.claim_pip(bel.wire("IOB_I_BUF"), bel.wire("IOB_I"));
    bel.verify_net(&[bel.wire("IOB_I"), bel.bel_wire(obel_iob, "I")]);

    if bcrd.slot == bslots::ILOGIC[0] && !is_single {
        let obel = bcrd.bel(bslots::ILOGIC[1]);
        bel.claim_pip(bel.wire("SHIFTIN1"), bel.bel_wire(obel, "SHIFTOUT1"));
        bel.claim_pip(bel.wire("SHIFTIN2"), bel.bel_wire(obel, "SHIFTOUT2"));
    }

    if bcrd.slot == bslots::ILOGIC[1] {
        let has_clkout = match bel.vrf.rd.source {
            Source::ISE => matches!(bcrd.row.to_idx() % 50, 7 | 19 | 31 | 43 | 21 | 23 | 25 | 27),
            Source::Vivado => !matches!(bcrd.row.to_idx() % 50, 13 | 37),
        };
        if has_clkout {
            bel.claim_pip(bel.wire("CLKPAD"), bel.wire("O"));
        }
    }

    bel.commit()
}

fn verify_ologic(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::OLOGIC.index_of(bcrd.slot).unwrap();
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    let is_hp = matches!(tcid, tcls::IO_HP_PAIR | tcls::IO_HP_S | tcls::IO_HP_N);

    let kind = if is_hp { "OLOGICE2" } else { "OLOGICE3" };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .skip_in(bcls::OLOGIC::CLK)
        .skip_in(bcls::OLOGIC::CLKB)
        .skip_out(bcls::OLOGIC::TFB)
        .extra_in_claim_rename("CLK", "CLK_FAKE")
        .extra_in_claim_rename("CLKB", "CLKB_FAKE")
        .extra_out_claim("OFB")
        .extra_out_claim_rename("TFB", "TFB_FAKE")
        .extra_out_claim("OQ")
        .extra_out_claim("TQ")
        .extra_out_claim("SHIFTOUT1")
        .extra_out_claim("SHIFTOUT2")
        .extra_in_dummy("REV")
        .extra_in_claim("TBYTEIN")
        .extra_out_claim("TBYTEOUT");
    if bcrd.slot != bslots::OLOGIC[1] {
        bel = bel.extra_in_dummy("SHIFTIN1").extra_in_dummy("SHIFTIN2")
    } else {
        bel = bel.extra_in_claim("SHIFTIN1").extra_in_claim("SHIFTIN2")
    }

    bel.claim_pip(bel.wire("CLK_FAKE"), bel.wire("CLK"));
    bel.claim_pip(bel.wire("CLKB_FAKE"), bel.wire("CLK"));
    bel.claim_pip(bel.wire("CLKB_FAKE"), bel.wire("CLKB"));

    bel.claim_pip(bel.wire("TFB"), bel.wire("TFB_FAKE"));

    let obel_iob = bcrd.bel(bslots::IOB[idx]);
    bel.claim_pip(bel.wire("IOB_T"), bel.wire("TQ"));
    bel.claim_pip(bel.wire("IOB_O"), bel.wire("OQ"));
    if kind == "OLOGICE2" {
        let obel_odelay = bcrd.bel(bslots::ODELAY[idx]);
        bel.claim_pip(bel.wire("IOB_O"), bel.bel_wire(obel_odelay, "DATAOUT"));
    }
    bel.verify_net(&[bel.wire("IOB_O"), bel.bel_wire(obel_iob, "O")]);
    bel.verify_net(&[bel.wire("IOB_T"), bel.bel_wire(obel_iob, "T")]);

    if bcrd.slot == bslots::OLOGIC[1] {
        let obel = bcrd.bel(bslots::OLOGIC[0]);
        bel.claim_pip(bel.wire("SHIFTIN1"), bel.bel_wire(obel, "SHIFTOUT1"));
        bel.claim_pip(bel.wire("SHIFTIN2"), bel.bel_wire(obel, "SHIFTOUT2"));
    }

    bel.claim_pip(bel.wire("TBYTEIN"), bel.wire("TBYTEIN_IOI"));

    let y = bcrd.row.to_idx() % 50 + idx;
    let tbyte_sy = match y {
        0..13 => 7,
        13..25 => 19,
        25..37 => 31,
        37..50 => 43,
        _ => unreachable!(),
    };
    let tbyte_srow = RowId::from_idx(bcrd.row.to_idx() / 50 * 50 + tbyte_sy);
    if tbyte_sy == y {
        bel.claim_net(&[bel.wire("TBYTEIN_IOI")]);
        bel.claim_pip(bel.wire("TBYTEIN_IOI"), bel.wire("TBYTEOUT"));
    } else {
        let obel = bcrd.cell.with_row(tbyte_srow).bel(bslots::OLOGIC[0]);
        bel.verify_net(&[bel.wire("TBYTEIN_IOI"), bel.bel_wire(obel, "TBYTEIN_IOI")]);
    }

    bel.commit();
}

fn verify_idelay(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::IDELAY.index_of(bcrd.slot).unwrap();
    let tcrd = vrf.grid.bel_tile(bcrd);
    let tcid = vrf.grid[tcrd].class;
    let is_hp = matches!(tcid, tcls::IO_HP_PAIR | tcls::IO_HP_S | tcls::IO_HP_N);
    let kind = if is_hp {
        "IDELAYE2_FINEDELAY"
    } else {
        "IDELAYE2"
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_in_claim("IDATAIN")
        .extra_out_claim("DATAOUT");

    let obel_ilogic = bcrd.bel(bslots::ILOGIC[idx]);
    bel.claim_pip(bel.wire("IDATAIN"), bel.bel_wire(obel_ilogic, "IOB_I_BUF"));

    let obel_ologic = bcrd.bel(bslots::OLOGIC[idx]);
    bel.claim_pip(bel.wire("IDATAIN"), bel.bel_wire(obel_ologic, "OFB"));

    bel.commit();
}

fn verify_odelay(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::ODELAY.index_of(bcrd.slot).unwrap();
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("ODELAYE2")
        .extra_in_claim("CLKIN")
        .extra_in_claim("ODATAIN");

    let obel_ologic = bcrd.bel(bslots::OLOGIC[idx]);
    bel.claim_pip(bel.wire("CLKIN"), bel.bel_wire(obel_ologic, "CLK"));
    bel.claim_pip(bel.wire("ODATAIN"), bel.bel_wire(obel_ologic, "OFB"));

    bel.commit();
}

fn verify_iob(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::IOB.index_of(bcrd.slot).unwrap();
    let tcrd = vrf.grid.bel_tile(bcrd);
    let tcid = vrf.grid[tcrd].class;
    let (kind, is_single) = match (idx, tcid) {
        (1, tcls::IO_HP_PAIR) => ("IOB18M", false),
        (0, tcls::IO_HP_PAIR) => ("IOB18S", false),
        (1, tcls::IO_HR_PAIR) => ("IOB33M", false),
        (0, tcls::IO_HR_PAIR) => ("IOB33S", false),
        (0, tcls::IO_HP_S) => ("IOB18", true),
        (0, tcls::IO_HP_N) => ("IOB18", true),
        (0, tcls::IO_HR_S) => ("IOB33", true),
        (0, tcls::IO_HR_N) => ("IOB33", true),
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_out_claim("I")
        .extra_in_claim("O")
        .extra_in_claim("T")
        .extra_out_claim("O_OUT")
        .extra_out_claim("T_OUT")
        .extra_out_claim("DIFFO_OUT")
        .extra_out_claim("PADOUT");
    if is_single {
        bel = bel
            .extra_in_dummy("DIFFI_IN")
            .extra_in_dummy("DIFFO_IN")
            .extra_in_dummy("O_IN")
            .extra_in_dummy("T_IN")
            .extra_in_dummy("DIFF_TERM_INT_EN");
    } else if idx == 0 {
        bel = bel
            .extra_in_claim("DIFFI_IN")
            .extra_in_claim("DIFFO_IN")
            .extra_in_claim("O_IN")
            .extra_in_claim("T_IN");
    } else {
        bel = bel
            .extra_in_claim("DIFFI_IN")
            .extra_in_dummy("DIFFO_IN")
            .extra_in_dummy("O_IN")
            .extra_in_dummy("T_IN")
            .extra_in_dummy("DIFF_TERM_INT_EN");
    }
    if !is_single {
        let oslot = bslots::IOB[idx ^ 1];
        let obel = bcrd.bel(oslot);
        if idx == 0 {
            bel.claim_pip(bel.wire("O_IN"), bel.bel_wire(obel, "O_OUT"));
            bel.claim_pip(bel.wire("T_IN"), bel.bel_wire(obel, "T_OUT"));
            bel.claim_pip(bel.wire("DIFFO_IN"), bel.bel_wire(obel, "DIFFO_OUT"));
        }
        bel.claim_pip(bel.wire("DIFFI_IN"), bel.bel_wire(obel, "PADOUT"));
    }
    bel.commit();
}

fn verify_phaser_in(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::PHASER_IN.index_of(bcrd.slot).unwrap();
    let abcd = ['A', 'B', 'C', 'D'][idx];
    let mut bel = vrf.verify_bel(bcrd).kind("PHASER_IN_PHY");
    let obel_pc = bcrd.bel(bslots::PHY_CONTROL);
    for (pin, opin) in [
        ("ENCALIBPHY0", "PCENABLECALIB0"),
        ("ENCALIBPHY1", "PCENABLECALIB1"),
        ("RANKSELPHY0", &format!("INRANK{abcd}0")),
        ("RANKSELPHY1", &format!("INRANK{abcd}1")),
        ("BURSTPENDINGPHY", &format!("INBURSTPENDING{idx}")),
    ] {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel_pc, opin)]);
    }
    bel.commit();
}

fn verify_phaser_out(vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::PHASER_OUT.index_of(bcrd.slot).unwrap();
    let mut bel = vrf.verify_bel(bcrd).kind("PHASER_OUT_PHY");
    let obel_pc = bcrd.bel(bslots::PHY_CONTROL);
    for (pin, opin) in [
        ("ENCALIBPHY0", "PCENABLECALIB0"),
        ("ENCALIBPHY1", "PCENABLECALIB1"),
        ("BURSTPENDINGPHY", &format!("OUTBURSTPENDING{idx}")),
    ] {
        bel = bel.extra_in_claim(pin);
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel_pc, opin)]);
    }
    bel.commit();
}

fn verify_phy_control(vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_claim_far("INRANKA0")
        .extra_out_claim_far("INRANKA1")
        .extra_out_claim_far("INRANKB0")
        .extra_out_claim_far("INRANKB1")
        .extra_out_claim_far("INRANKC0")
        .extra_out_claim_far("INRANKC1")
        .extra_out_claim_far("INRANKD0")
        .extra_out_claim_far("INRANKD1")
        .extra_out_claim_far("PCENABLECALIB0")
        .extra_out_claim_far("PCENABLECALIB1")
        .extra_out_claim_far("INBURSTPENDING0")
        .extra_out_claim_far("INBURSTPENDING1")
        .extra_out_claim_far("INBURSTPENDING2")
        .extra_out_claim_far("INBURSTPENDING3")
        .extra_out_claim_far("OUTBURSTPENDING0")
        .extra_out_claim_far("OUTBURSTPENDING1")
        .extra_out_claim_far("OUTBURSTPENDING2")
        .extra_out_claim_far("OUTBURSTPENDING3")
        .commit();
}

fn verify_mmcm(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("MMCME2_ADV");
    bel.claim_net(&[bel.wire("CLKFB")]);
    bel.claim_pip(bel.wire("CLKFB"), bel.wire("CLKFBOUT"));
    bel.claim_pip(bel.wire("CLKFBIN"), bel.wire("CLKFB"));
    bel.commit()
}

fn verify_pll(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("PLLE2_ADV");
    bel.claim_net(&[bel.wire("CLKFB")]);
    bel.claim_pip(bel.wire("CLKFB"), bel.wire("CLKFBOUT"));
    bel.claim_pip(bel.wire("CLKFBIN"), bel.wire("CLKFB"));
    bel.commit()
}

fn verify_xadc(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("XADC")
        .ipad("VP", 1)
        .ipad("VN", 2);

    for i in 0..16 {
        let vauxp = &format!("VAUXP{i}");
        let vauxn = &format!("VAUXN{i}");
        let Some((iop, _)) = edev.get_sysmon_vaux(bcrd.cell, i) else {
            bel = bel.extra_in_dummy(vauxp).extra_in_dummy(vauxn);
            continue;
        };
        bel = bel.extra_in_claim(vauxp).extra_in_claim(vauxn);
        let (ow0p, iw0p) = bel.pip(vauxp, 0);
        let (ow1p, iw1p) = bel.pip(vauxp, 1);
        let (ow0n, iw0n) = bel.pip(vauxn, 0);
        let (ow1n, iw1n) = bel.pip(vauxn, 1);
        bel.claim_pip(ow0p, iw0p);
        bel.claim_pip(ow1p, iw1p);
        bel.claim_pip(ow0n, iw0n);
        bel.claim_pip(ow1n, iw1n);
        bel.claim_net(&[iw0p, ow1p]);
        bel.claim_net(&[iw0n, ow1n]);
        let obel = iop.cell.bel(bslots::IOB[1]);
        bel.claim_net(&[iw1p, bel.bel_wire(obel, "MONITOR")]);
        bel.claim_pip(bel.bel_wire(obel, "MONITOR"), bel.bel_wire(obel, "PADOUT"));
        let obel = iop.cell.bel(bslots::IOB[0]);
        bel.claim_net(&[iw1n, bel.bel_wire(obel, "MONITOR")]);
        bel.claim_pip(bel.bel_wire(obel, "MONITOR"), bel.bel_wire(obel, "PADOUT"));
    }

    if bcrd.die == edev.interposer.unwrap().primary || bel.vrf.rd.source != Source::Vivado {
        bel.commit();
    }
}

fn verify_ps(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("PS7");
    let mut pins = vec![];
    pins.push("DDRWEB".to_string());
    pins.push("DDRVRN".to_string());
    pins.push("DDRVRP".to_string());
    for i in 0..13 {
        pins.push(format!("DDRA{i}"));
    }
    pins.push("DDRA14".to_string());
    pins.push("DDRA13".to_string());
    for i in 0..3 {
        pins.push(format!("DDRBA{i}"));
    }
    pins.push("DDRCASB".to_string());
    pins.push("DDRCKE".to_string());
    pins.push("DDRCKN".to_string());
    pins.push("DDRCKP".to_string());
    pins.push("PSCLK".to_string());
    pins.push("DDRCSB".to_string());
    for i in 0..4 {
        pins.push(format!("DDRDM{i}"));
    }
    for i in 0..32 {
        pins.push(format!("DDRDQ{i}"));
    }
    for i in 0..4 {
        pins.push(format!("DDRDQSN{i}"));
    }
    for i in 0..4 {
        pins.push(format!("DDRDQSP{i}"));
    }
    pins.push("DDRDRSTB".to_string());
    for i in 0..54 {
        pins.push(format!("MIO{i}"));
    }
    pins.push("DDRODT".to_string());
    pins.push("PSPORB".to_string());
    pins.push("DDRRASB".to_string());
    pins.push("PSSRSTB".to_string());

    for (i, pin) in pins.into_iter().enumerate() {
        bel = bel.iopad(pin, i + 1);
    }
    bel.commit();
}

fn verify_gtclk(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("IBUFDS_GTE2")
        .skip_out(bcls::GTCLK::CLKOUT)
        .extra_out_claim("O")
        .extra_out_claim("ODIV2");

    bel.claim_pip(bel.wire("CLKOUT"), bel.wire("O"));
    bel.claim_pip(bel.wire("CLKOUT"), bel.wire("ODIV2"));

    if !edev.disabled.contains(&DisabledPart::Gtp) || bel.vrf.rd.source == Source::ISE {
        bel.ipad("I", 1).ipad("IB", 2).commit();
    }
}

fn verify_gtp_channel(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("GTPE2_CHANNEL")
        .ipad("GTPRXP", 1)
        .ipad("GTPRXN", 2)
        .opad("GTPTXP", 3)
        .opad("GTPTXN", 4)
        .extra_in_claim("PLL0CLK")
        .extra_in_claim("PLL1CLK")
        .extra_in_claim("PLL0REFCLK")
        .extra_in_claim("PLL1REFCLK");

    let obel = bcrd
        .with_row(edev.chips[bcrd.die].row_hclk(bcrd.row))
        .bel(bslots::GTP_COMMON);
    for (pin, opin) in [
        ("PLL0CLK", "PLL0OUTCLK"),
        ("PLL1CLK", "PLL1OUTCLK"),
        ("PLL0REFCLK", "PLL0OUTREFCLK"),
        ("PLL1REFCLK", "PLL1OUTREFCLK"),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
    }
    if !edev.disabled.contains(&DisabledPart::Gtp) || bel.vrf.rd.source == Source::ISE {
        bel.commit();
    }
}

fn verify_gtx_channel(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = vrf.grid.bel_tile(bcrd);
    let is_gth = vrf.grid[tcrd].class == tcls::GTH_CHANNEL;

    let rxp = if is_gth { "GTHRXP" } else { "GTXRXP" };
    let rxn = if is_gth { "GTHRXN" } else { "GTXRXN" };
    let txp = if is_gth { "GTHTXP" } else { "GTXTXP" };
    let txn = if is_gth { "GTHTXN" } else { "GTXTXN" };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(if is_gth {
            "GTHE2_CHANNEL"
        } else {
            "GTXE2_CHANNEL"
        })
        .extra_in_claim("GTREFCLK0")
        .extra_in_claim("GTREFCLK1")
        .extra_in_claim("GTNORTHREFCLK0")
        .extra_in_claim("GTNORTHREFCLK1")
        .extra_in_claim("GTSOUTHREFCLK0")
        .extra_in_claim("GTSOUTHREFCLK1")
        .extra_in_claim("QPLLCLK")
        .extra_in_claim("QPLLREFCLK")
        .ipad(rxp, 1)
        .ipad(rxn, 2)
        .opad(txp, 3)
        .opad(txn, 4);

    if !is_gth {
        bel = bel
            .rename_out(GTX_CHANNEL::RXDATAVALID[0], "RXDATAVALID")
            .rename_out(GTX_CHANNEL::RXHEADERVALID[0], "RXHEADERVALID")
            .rename_out(GTX_CHANNEL::RXSTARTOFSEQ[0], "RXSTARTOFSEQ")
    }

    let obel = bcrd
        .with_row(edev.chips[bcrd.die].row_hclk(bcrd.row))
        .bel(bslots::GTX_COMMON);
    let has_s = bel
        .vrf
        .grid
        .bel_delta(obel.cell, 0, -50, bslots::GTX_COMMON)
        .is_some();
    let has_n = bel
        .vrf
        .grid
        .bel_delta(obel.cell, 0, 50, bslots::GTX_COMMON)
        .is_some();
    for (pin, opin, present) in [
        ("QPLLCLK", "QPLLOUTCLK", true),
        ("QPLLREFCLK", "QPLLOUTREFCLK", true),
        ("GTREFCLK0", "GTREFCLK0", true),
        ("GTREFCLK1", "GTREFCLK1", true),
        ("GTNORTHREFCLK0", "GTNORTHREFCLK0", has_s),
        ("GTNORTHREFCLK1", "GTNORTHREFCLK1", has_s),
        ("GTSOUTHREFCLK0", "GTSOUTHREFCLK0", has_n),
        ("GTSOUTHREFCLK1", "GTSOUTHREFCLK1", has_n),
    ] {
        if present {
            bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(obel, opin)]);
        } else {
            bel.vrf.claim_dummy_in(bel.wire_far(pin));
        }
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
    }
    bel.commit();
}

fn verify_gtp_common(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("GTPE2_COMMON")
        .extra_in_claim("GTREFCLK0")
        .extra_in_claim("GTREFCLK1")
        .extra_out_claim("PLL0OUTCLK")
        .extra_out_claim("PLL1OUTCLK")
        .extra_out_claim("PLL0OUTREFCLK")
        .extra_out_claim("PLL1OUTREFCLK");
    let mut is_mid_l = false;
    let mut is_mid_r = false;
    if let Some((cl, cr)) = edev.col_gt_m {
        if cl == bcrd.col {
            bel = bel
                .extra_in_dummy("GTEASTREFCLK0")
                .extra_in_dummy("GTEASTREFCLK1")
                .extra_in_claim("GTWESTREFCLK0")
                .extra_in_claim("GTWESTREFCLK1");
            is_mid_l = true;
        } else {
            assert_eq!(bcrd.col, cr);
            bel = bel
                .extra_in_claim("GTEASTREFCLK0")
                .extra_in_claim("GTEASTREFCLK1")
                .extra_in_dummy("GTWESTREFCLK0")
                .extra_in_dummy("GTWESTREFCLK1");
            is_mid_r = true;
        }
    } else {
        bel = bel
            .extra_in_dummy("GTEASTREFCLK0")
            .extra_in_dummy("GTEASTREFCLK1")
            .extra_in_dummy("GTWESTREFCLK0")
            .extra_in_dummy("GTWESTREFCLK1");
    }

    for i in 0..2 {
        bel.claim_net(&[bel.wire(&format!("REFCLK{i}"))]);
        bel.claim_pip(
            bel.wire(&format!("GTREFCLK{i}")),
            bel.wire(&format!("REFCLK{i}")),
        );
        let obel = bcrd.bel(bslots::GTCLK[i]);
        bel.claim_pip(bel.wire(&format!("REFCLK{i}")), bel.bel_wire(obel, "O"));
    }

    for pin in ["PLL0OUTCLK", "PLL1OUTCLK", "PLL0OUTREFCLK", "PLL1OUTREFCLK"] {
        bel.claim_net(&[bel.wire_far(pin)]);
        bel.claim_pip(bel.wire_far(pin), bel.wire(pin));
    }

    if is_mid_l {
        bel.claim_net(&[bel.wire("EASTCLK0")]);
        bel.claim_net(&[bel.wire("EASTCLK1")]);
        bel.claim_pip(bel.wire("EASTCLK0"), bel.wire("REFCLK0"));
        bel.claim_pip(bel.wire("EASTCLK0"), bel.wire("REFCLK1"));
        bel.claim_pip(bel.wire("EASTCLK1"), bel.wire("REFCLK0"));
        bel.claim_pip(bel.wire("EASTCLK1"), bel.wire("REFCLK1"));

        bel.claim_pip(bel.wire("GTWESTREFCLK0"), bel.wire("WESTCLK0"));
        bel.claim_pip(bel.wire("GTWESTREFCLK1"), bel.wire("WESTCLK1"));
        let obel = bcrd
            .with_col(edev.col_gt_m.unwrap().1)
            .bel(bslots::GTP_COMMON);
        bel.verify_net(&[bel.wire("WESTCLK0"), bel.bel_wire(obel, "WESTCLK0")]);
        bel.verify_net(&[bel.wire("WESTCLK1"), bel.bel_wire(obel, "WESTCLK1")]);
    }
    if is_mid_r {
        bel.claim_net(&[bel.wire("WESTCLK0")]);
        bel.claim_net(&[bel.wire("WESTCLK1")]);
        bel.claim_pip(bel.wire("WESTCLK0"), bel.wire("REFCLK0"));
        bel.claim_pip(bel.wire("WESTCLK0"), bel.wire("REFCLK1"));
        bel.claim_pip(bel.wire("WESTCLK1"), bel.wire("REFCLK0"));
        bel.claim_pip(bel.wire("WESTCLK1"), bel.wire("REFCLK1"));

        bel.claim_pip(bel.wire("GTEASTREFCLK0"), bel.wire("EASTCLK0"));
        bel.claim_pip(bel.wire("GTEASTREFCLK1"), bel.wire("EASTCLK1"));
        let obel = bcrd
            .with_col(edev.col_gt_m.unwrap().0)
            .bel(bslots::GTP_COMMON);
        bel.verify_net(&[bel.wire("EASTCLK0"), bel.bel_wire(obel, "EASTCLK0")]);
        bel.verify_net(&[bel.wire("EASTCLK1"), bel.bel_wire(obel, "EASTCLK1")]);
    }

    if !edev.disabled.contains(&DisabledPart::Gtp) || bel.vrf.rd.source == Source::ISE {
        bel.commit();
    }
}

fn verify_gtx_common(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = vrf.grid.bel_tile(bcrd);
    let is_gth = vrf.grid[tcrd].class == tcls::GTH_COMMON;
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(if is_gth {
            "GTHE2_COMMON"
        } else {
            "GTXE2_COMMON"
        })
        .extra_in_claim("GTREFCLK0")
        .extra_in_claim("GTREFCLK1")
        .extra_in_claim("GTNORTHREFCLK0")
        .extra_in_claim("GTNORTHREFCLK1")
        .extra_in_claim("GTSOUTHREFCLK0")
        .extra_in_claim("GTSOUTHREFCLK1")
        .extra_out_claim("QPLLOUTCLK")
        .extra_out_claim("QPLLOUTREFCLK");

    for i in 0..2 {
        bel.claim_net(&[bel.wire_far(&format!("GTREFCLK{i}"))]);
        bel.claim_pip(
            bel.wire(&format!("GTREFCLK{i}")),
            bel.wire_far(&format!("GTREFCLK{i}")),
        );
        let obel = bcrd.bel(bslots::GTCLK[i]);
        bel.claim_pip(
            bel.wire_far(&format!("GTREFCLK{i}")),
            bel.bel_wire(obel, "O"),
        );
        bel.claim_pip(
            bel.wire(&format!("GTSOUTHREFCLK{i}")),
            bel.wire_far(&format!("GTSOUTHREFCLK{i}")),
        );
        bel.claim_pip(
            bel.wire(&format!("GTNORTHREFCLK{i}")),
            bel.wire_far(&format!("GTNORTHREFCLK{i}")),
        );
    }

    for pin in ["QPLLOUTCLK", "QPLLOUTREFCLK"] {
        bel.claim_net(&[bel.wire_far(pin)]);
        bel.claim_pip(bel.wire_far(pin), bel.wire(pin));
    }

    if let Some(obel_s) = edev.bel_gtclk_neighbour(bcrd, Dir::S) {
        bel.claim_net(&[bel.wire("BRKH_S_SOUTHREFCLK0_S")]);
        bel.claim_net(&[bel.wire("BRKH_S_SOUTHREFCLK1_S")]);
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK0_S"),
            bel.wire("BRKH_S_SOUTHREFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK0_S"),
            bel.wire("BRKH_S_REFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK0_S"),
            bel.wire("BRKH_S_REFCLK1"),
        );
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK1_S"),
            bel.wire("BRKH_S_SOUTHREFCLK1"),
        );
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK1_S"),
            bel.wire("BRKH_S_REFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_S_SOUTHREFCLK1_S"),
            bel.wire("BRKH_S_REFCLK1"),
        );

        bel.verify_net(&[bel.wire("BRKH_S_REFCLK0"), bel.wire_far("GTREFCLK0")]);
        bel.verify_net(&[bel.wire("BRKH_S_REFCLK1"), bel.wire_far("GTREFCLK1")]);

        for i in 0..2 {
            bel.verify_net(&[
                bel.wire(&format!("BRKH_S_SOUTHREFCLK{i}")),
                bel.wire_far(&format!("GTSOUTHREFCLK{i}")),
            ]);
        }

        for i in 0..2 {
            bel.verify_net(&[
                bel.wire(&format!("BRKH_S_SOUTHREFCLK{i}_S")),
                bel.bel_wire_far(obel_s, &format!("GTSOUTHREFCLK{i}")),
            ]);
        }
    } else {
        for i in 0..2 {
            bel.vrf
                .claim_dummy_in(bel.wire_far(&format!("GTNORTHREFCLK{i}")));
        }
    }

    if let Some(obel_n) = edev.bel_gtclk_neighbour(bcrd, Dir::N) {
        bel.claim_net(&[bel.wire("BRKH_N_NORTHREFCLK0_N")]);
        bel.claim_net(&[bel.wire("BRKH_N_NORTHREFCLK1_N")]);
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK0_N"),
            bel.wire("BRKH_N_NORTHREFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK0_N"),
            bel.wire("BRKH_N_REFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK0_N"),
            bel.wire("BRKH_N_REFCLK1"),
        );
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK1_N"),
            bel.wire("BRKH_N_NORTHREFCLK1"),
        );
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK1_N"),
            bel.wire("BRKH_N_REFCLK0"),
        );
        bel.claim_pip(
            bel.wire("BRKH_N_NORTHREFCLK1_N"),
            bel.wire("BRKH_N_REFCLK1"),
        );

        bel.verify_net(&[bel.wire("BRKH_N_REFCLK0"), bel.wire_far("GTREFCLK0")]);
        bel.verify_net(&[bel.wire("BRKH_N_REFCLK1"), bel.wire_far("GTREFCLK1")]);

        for i in 0..2 {
            bel.verify_net(&[
                bel.wire(&format!("BRKH_N_NORTHREFCLK{i}")),
                bel.wire_far(&format!("GTNORTHREFCLK{i}")),
            ]);
        }

        for i in 0..2 {
            bel.verify_net(&[
                bel.wire(&format!("BRKH_N_NORTHREFCLK{i}_N")),
                bel.bel_wire_far(obel_n, &format!("GTNORTHREFCLK{i}")),
            ]);
        }
    } else {
        for i in 0..2 {
            bel.vrf
                .claim_dummy_in(bel.wire_far(&format!("GTSOUTHREFCLK{i}")));
        }
    }

    bel.commit();
}

fn verify_gtz_octal(vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd).kind("GTZE2_OCTAL");
    for i in 0..2 {
        bel = bel
            .ipad(format!("GTREFCLK{i}P"), 1 + i * 2)
            .ipad(format!("GTREFCLK{i}N"), 2 + i * 2);
    }
    for i in 0..8 {
        bel = bel
            .ipad(format!("GTZRXP{i}"), 5 + i * 2)
            .ipad(format!("GTZRXN{i}"), 6 + i * 2)
            .opad(format!("GTZTXP{i}"), 21 + i * 2)
            .opad(format!("GTZTXN{i}"), 22 + i * 2);
    }
    bel.commit();
}

fn verify_bel(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcid = edev[tcrd].class;
    match bcrd.slot {
        bslots::SPEC_INT if matches!(tcid, tcls::CLK_BUFG_REBUF | tcls::CLK_BALI_REBUF) => {
            verify_clk_rebuf(vrf, bcrd)
        }
        bslots::SPEC_INT if tcid == tcls::CLK_HROW => verify_clk_hrow(vrf, bcrd),
        bslots::INT
        | bslots::INTF_INT
        | bslots::INTF_TESTMUX
        | bslots::SPEC_INT
        | bslots::CMT_FIFO_INT
        | bslots::CLK_INT
        | bslots::HROW_INT
        | bslots::HCLK_IO_INT
        | bslots::MISC_CFG
        | bslots::BANK
        | bslots::GLOBAL
        | bslots::HCLK_DRP_GTP_MID => (),
        _ if bslots::HCLK_DRP.contains(bcrd.slot) => (),
        _ if bslots::SLICE.contains(bcrd.slot) => verify_slice(edev, vrf, bcrd),
        _ if bslots::DSP.contains(bcrd.slot) => verify_dsp(edev, vrf, bcrd),
        bslots::BRAM => verify_bram(edev, vrf, bcrd),
        bslots::PCIE => {
            if !edev.disabled.contains(&DisabledPart::Gtp) || vrf.rd.source == Source::ISE {
                vrf.verify_bel(bcrd).kind("PCIE_2_1").commit();
            }
        }
        bslots::PCIE3 => vrf.verify_bel(bcrd).kind("PCIE_3_0").commit(),
        bslots::PMVBRAM => verify_pmvbram(vrf, bcrd),
        _ if bslots::PMV_CFG.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        bslots::PMV_CLK => vrf.verify_bel(bcrd).commit(),
        bslots::PMV2 | bslots::MTBF2 | bslots::STARTUP | bslots::CAPTURE | bslots::USR_ACCESS => {
            vrf.verify_bel(bcrd).commit()
        }
        bslots::PMV2_SVT => vrf.verify_bel(bcrd).kind("PMV2_SVT").commit(),
        bslots::CFG_IO_ACCESS => vrf.verify_bel(bcrd).kind("CFG_IO_ACCESS").commit(),
        bslots::FRAME_ECC => vrf.verify_bel(bcrd).kind("FRAME_ECC").commit(),
        bslots::PMVIOB_CFG | bslots::PMVIOB_CLK => vrf.verify_bel(bcrd).commit(),
        bslots::DCIRESET | bslots::DNA_PORT | bslots::EFUSE_USR => {
            if bcrd.die == edev.interposer.unwrap().primary || vrf.rd.source != Source::Vivado {
                vrf.verify_bel(bcrd).commit()
            }
        }
        _ if bslots::BSCAN.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::ICAP.contains(bcrd.slot) => vrf
            .verify_bel(bcrd)
            .kind("ICAP")
            .rename_in(bcls::ICAP_V6::CSB, "CSIB")
            .commit(),

        bslots::HCLK => verify_hclk(edev, vrf, bcrd),
        _ if bslots::BUFHCE_W.contains(bcrd.slot) || bslots::BUFHCE_E.contains(bcrd.slot) => {
            vrf.verify_bel(bcrd).commit()
        }
        _ if bslots::BUFGCTRL.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),

        _ if bslots::BUFIO.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        _ if bslots::BUFR.contains(bcrd.slot) => vrf.verify_bel(bcrd).commit(),
        bslots::IDELAYCTRL => vrf.verify_bel(bcrd).commit(),
        bslots::DCI => verify_dci(vrf, bcrd),

        _ if bslots::ILOGIC.contains(bcrd.slot) => verify_ilogic(edev, vrf, bcrd),
        _ if bslots::OLOGIC.contains(bcrd.slot) => verify_ologic(edev, vrf, bcrd),
        _ if bslots::IDELAY.contains(bcrd.slot) => verify_idelay(vrf, bcrd),
        _ if bslots::ODELAY.contains(bcrd.slot) => verify_odelay(vrf, bcrd),
        _ if bslots::IOB.contains(bcrd.slot) => verify_iob(vrf, bcrd),

        _ if bslots::PHASER_IN.contains(bcrd.slot) => verify_phaser_in(vrf, bcrd),
        _ if bslots::PHASER_OUT.contains(bcrd.slot) => verify_phaser_out(vrf, bcrd),
        bslots::PHASER_REF => vrf.verify_bel(bcrd).commit(),
        bslots::PHY_CONTROL => verify_phy_control(vrf, bcrd),
        _ if bcrd.slot == bslots::PLL[0] => verify_mmcm(vrf, bcrd),
        _ if bcrd.slot == bslots::PLL[1] => verify_pll(vrf, bcrd),
        _ if bslots::BUFMRCE.contains(bcrd.slot) => vrf.verify_bel(bcrd).kind("BUFMRCE").commit(),
        bslots::IN_FIFO | bslots::OUT_FIFO => vrf.verify_bel(bcrd).commit(),

        bslots::SYSMON => verify_xadc(edev, vrf, bcrd),
        bslots::PS => verify_ps(vrf, bcrd),

        bslots::GTP_CHANNEL => verify_gtp_channel(edev, vrf, bcrd),
        bslots::GTP_COMMON => verify_gtp_common(edev, vrf, bcrd),
        bslots::GTX_CHANNEL => verify_gtx_channel(edev, vrf, bcrd),
        bslots::GTX_COMMON => verify_gtx_common(edev, vrf, bcrd),
        _ if bslots::GTCLK.contains(bcrd.slot) => verify_gtclk(edev, vrf, bcrd),

        bslots::GTZ_OCTAL => verify_gtz_octal(vrf, bcrd),

        _ => println!("MEOW {}", bcrd.to_string(edev.db)),
    }
}

fn verify_extra(vrf: &mut Verifier) {
    let mut stub_out_cond = vec![
        "IOI_IMUX_RC0",
        "IOI_IMUX_RC1",
        "IOI_IMUX_RC2",
        "IOI_IMUX_RC3",
        "IOI_RCLK_DIV_CE0",
        "IOI_RCLK_DIV_CE1",
        "IOI_RCLK_DIV_CE2_1",
        "IOI_RCLK_DIV_CE3_1",
        "IOI_RCLK_DIV_CLR0_1",
        "IOI_RCLK_DIV_CLR1_1",
        "IOI_RCLK_DIV_CLR2",
        "IOI_RCLK_DIV_CLR3",
        "IOI_IDELAYCTRL_RST",
        "IOI_IDELAYCTRL_DNPULSEOUT",
        "IOI_IDELAYCTRL_UPPULSEOUT",
        "IOI_IDELAYCTRL_RDY",
        "IOI_IDELAYCTRL_OUTN1",
        "IOI_IDELAYCTRL_OUTN65",
        "LIOB_MONITOR_P",
        "LIOB_MONITOR_N",
        "RIOB_MONITOR_P",
        "RIOB_MONITOR_N",
    ];
    if vrf.rd.source == Source::Vivado {
        stub_out_cond.extend([
            "BRAM_PMVBRAM_SELECT1",
            "BRAM_PMVBRAM_SELECT2",
            "BRAM_PMVBRAM_SELECT3",
            "BRAM_PMVBRAM_SELECT4",
            // hmmmmm.
            "IOI_INT_DCI_EN",
            "IOI_DCI_TSTRST",
            "IOI_DCI_TSTHLP",
            "IOI_DCI_TSTHLN",
            "IOI_DCI_TSTCLK",
            "IOI_DCI_TSTRST0",
        ]);
    }
    for w in stub_out_cond {
        vrf.kill_stub_out_cond(w);
    }
    for prefix in ["PSS0", "PSS1", "PSS2"] {
        for i in 0..40 {
            for j in 0..2 {
                vrf.kill_stub_out_cond(&format!("{prefix}_CLK_B{j}_{i}"));
            }
            for j in 0..48 {
                vrf.kill_stub_out_cond(&format!("{prefix}_IMUX_B{j}_{i}"));
            }
            for j in 0..24 {
                vrf.kill_stub_in_cond(&format!("{prefix}_LOGIC_OUTS{j}_{i}"));
            }
        }
    }
    for i in 0..15 {
        for tb in ["BOT", "TOP"] {
            for lr in ['L', 'R'] {
                for j in 0..2 {
                    vrf.kill_stub_out_cond(&format!("PCIE3_{tb}_CLK{j}_{lr}_{i}"));
                    vrf.kill_stub_out_cond(&format!("PCIE3_{tb}_CTRL{j}_{lr}_{i}"));
                }
                for j in 0..48 {
                    vrf.kill_stub_out_cond(&format!("PCIE3_{tb}_IMUX{j}_{lr}_{i}"));
                }
            }
        }
    }
    for w in [
        "BRKH_GTX_REFCLK0_UPPER",
        "BRKH_GTX_REFCLK1_UPPER",
        "BRKH_GTX_REFCLK0_LOWER",
        "BRKH_GTX_REFCLK1_LOWER",
        "BRKH_GTX_SOUTHREFCLK0_UPPER",
        "BRKH_GTX_SOUTHREFCLK1_UPPER",
        "BRKH_GTX_NORTHREFCLK0_LOWER",
        "BRKH_GTX_NORTHREFCLK1_LOWER",
    ] {
        vrf.kill_stub_in_cond(w);
    }
    for w in [
        "BRKH_GTX_SOUTHREFCLK0_LOWER",
        "BRKH_GTX_SOUTHREFCLK1_LOWER",
        "BRKH_GTX_NORTHREFCLK0_UPPER",
        "BRKH_GTX_NORTHREFCLK1_UPPER",
    ] {
        vrf.kill_stub_out_cond(w);
    }
    if vrf.rd.source == Source::Vivado {
        for &crd in vrf.rd.tiles_by_kind_name("BRKH_INT") {
            if crd.y == vrf.rd.height - 1 {
                for w in [
                    "BRKH_INT_SL1END0",
                    "BRKH_INT_SL1END1",
                    "BRKH_INT_SL1END2",
                    "BRKH_INT_SL1END3",
                    "BRKH_INT_SR1END1",
                    "BRKH_INT_SR1END2",
                    "BRKH_INT_SR1END3",
                    "BRKH_INT_NL1BEG0_SLOW",
                    "BRKH_INT_NL1BEG1_SLOW",
                    "BRKH_INT_NL1BEG2_SLOW",
                    "BRKH_INT_NR1BEG0_SLOW",
                    "BRKH_INT_NR1BEG1_SLOW",
                    "BRKH_INT_NR1BEG2_SLOW",
                    "BRKH_INT_NR1BEG3_SLOW",
                ] {
                    vrf.claim_net(&[RawWireCoord { crd, wire: w }]);
                }
            }
        }
    }
}

fn verify_pre_hclk(vrf: &mut Verifier) {
    vrf.skip_bslot(bslots::HCLK);
    for tkn in ["INT_L", "INT_L_SLV", "INT_L_SLV_FLY"] {
        for i in 6..12 {
            vrf.mark_merge_pip(tkn, &format!("GCLK_L_B{i}_WEST"), &format!("GCLK_L_B{i}"));
            vrf.mark_merge_pip(tkn, &format!("GCLK_L_B{i}_EAST"), &format!("GCLK_L_B{i}"));
        }
    }
    for tkn in ["INT_R", "INT_R_SLV", "INT_R_SLV_FLY"] {
        for i in 0..6 {
            vrf.mark_merge_pip(tkn, &format!("GCLK_B{i}_WEST"), &format!("GCLK_B{i}"));
            vrf.mark_merge_pip(tkn, &format!("GCLK_B{i}_EAST"), &format!("GCLK_B{i}"));
        }
    }
}

fn verify_pre_clk_bufg(vrf: &mut Verifier) {
    for tkn in ["CLK_BUFG_BOT_R", "CLK_BUFG_TOP_R"] {
        for i in 0..16 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CLK_BUFG_LOGIC_OUTS_B{j}_{k}", j = 4 + i % 4, k = i / 4),
                &format!("CLK_BUFG_R_CK_FB_TEST0_{i}"),
            );
            vrf.mark_merge_pip(
                tkn,
                &format!("CLK_BUFG_LOGIC_OUTS_B{j}_{k}", j = i % 4, k = i / 4),
                &format!("CLK_BUFG_R_CK_FB_TEST1_{i}"),
            );
        }
    }

    // very likely a case of wrong-direction pip
    for tcid in [tcls::CLK_BUFG_S, tcls::CLK_BUFG_N] {
        if vrf.grid.tile_index[tcid].is_empty() {
            continue;
        }
        let BelInfo::SwitchBox(ref sb) = vrf.grid.db[tcid].bels[bslots::SPEC_INT] else {
            unreachable!()
        };
        for item in &sb.items {
            let SwitchBoxItem::ProgBuf(buf) = item else {
                continue;
            };
            if !wires::OUT_BEL.contains(buf.dst.wire) {
                continue;
            }
            vrf.skip_tcls_pip(tcid, buf.dst, buf.src.tw);
            vrf.inject_tcls_pip(tcid, buf.src.tw, buf.dst);
        }
    }
}

fn verify_pre_clk_hrow(vrf: &mut Verifier) {
    for &tcrd in &vrf.grid.tile_index[tcls::CLK_HROW] {
        // insert invisible buffers
        let BelInfo::SwitchBox(ref sb) = vrf.grid.db[tcls::CLK_HROW].bels[bslots::SPEC_INT] else {
            unreachable!()
        };
        for item in &sb.items {
            let SwitchBoxItem::ProgBuf(buf) = item else {
                continue;
            };
            if wires::GCLK_TEST_IN.contains(buf.dst.wire)
                || wires::CKINT_HROW.contains(buf.dst.wire)
            {
                continue;
            }
            vrf.skip_tcls_pip(tcls::CLK_HROW, buf.dst, buf.src.tw);
            let wt = vrf.grid.resolve_tile_wire(tcrd, buf.dst).unwrap();
            let wf = vrf.grid.resolve_tile_wire(tcrd, buf.src.tw).unwrap();
            vrf.alias_wire(wt, wf);
        }
    }
    for i in 0..32 {
        vrf.skip_tcls_pip(
            tcls::CLK_HROW,
            wires::IMUX_BUFG_O[i].cell(1),
            wires::GCLK_TEST[i ^ 1].cell(1),
        );
    }
}

fn verify_pre_ps(vrf: &mut Verifier) {
    for i in 0..4 {
        vrf.mark_merge_pip(
            "PSS1",
            &format!("PSS_HCLK_CK_IN{i}"),
            &format!("PSS_FCLKCLK{i}"),
        );
    }
    vrf.mark_merge_pip("PSS1", "PSS_LOGIC_OUTS1_19", "PSS1_LOGIC_OUTS1_39");
    vrf.mark_merge_pip("PSS1", "PSS_LOGIC_OUTS2_19", "PSS1_LOGIC_OUTS2_39");
    vrf.mark_merge_pip("PSS3", "PSS_LOGIC_OUTS0_1", "PSS1_LOGIC_OUTS0_1");
    vrf.mark_merge_pip("PSS3", "PSS_LOGIC_OUTS1_1", "PSS1_LOGIC_OUTS1_1");
}

fn verify_pre_cmt(endev: &ExpandedNamedDevice, vrf: &mut Verifier) {
    for (tkn, lr) in [("CMT_TOP_L_LOWER_B", 'L'), ("CMT_TOP_R_LOWER_B", 'R')] {
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
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_{lr}_LOWER_B_CLK_MMCM{i}"),
                &format!("CMT_LR_LOWER_B_MMCM_{pin}"),
            );
        }
        for i in 0..4 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_{lr}_LOWER_B_CLK_FREQ_BB{i}"),
                &format!("MMCM_CLK_FREQ_BB_NS{ii}", ii = 3 - i),
            );
        }
    }

    for tkn in ["CMT_TOP_L_UPPER_B", "CMT_TOP_R_UPPER_B"] {
        for i in 0..4 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_FREQ_BB_PREF_IN{i}"),
                &format!("PLL_CLK_FREQBB_REBUFOUT{i}"),
            );
        }
    }
    for (tkn, lr) in [("CMT_TOP_L_UPPER_T", 'L'), ("CMT_TOP_R_UPPER_T", 'R')] {
        for (i, pin) in [
            "CLKOUT0", "CLKOUT1", "CLKOUT2", "CLKOUT3", "CLKOUT4", "CLKOUT5", "CLKFBOUT", "TMUXOUT",
        ]
        .into_iter()
        .enumerate()
        {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_{lr}_UPPER_T_CLKPLL{i}"),
                &format!("CMT_TOP_R_UPPER_T_PLLE2_{pin}"),
            );
        }
        for i in 0..4 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_{lr}_UPPER_T_FREQ_BB{i}"),
                &format!("PLL_CLK_FREQ_BB{i}_NS"),
            );
        }
    }
    vrf.mark_merge_pip(
        "CMT_TOP_R_LOWER_B",
        "CMT_R_LOWER_B_CLK_IN1_INT",
        "CMT_TOP_CLK0_15",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_LOWER_B",
        "CMT_R_LOWER_B_CLK_IN2_INT",
        "CMT_TOP_CLK1_15",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_LOWER_B",
        "CMT_R_LOWER_B_CLK_IN3_INT",
        "CMT_TOP_CLK0_14",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_LOWER_B",
        "CMT_L_LOWER_B_CLK_IN1_INT",
        "CMT_TOP_CLK0_15",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_LOWER_B",
        "CMT_L_LOWER_B_CLK_IN2_INT",
        "CMT_TOP_CLK1_15",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_LOWER_B",
        "CMT_L_LOWER_B_CLK_IN3_INT",
        "CMT_TOP_CLK0_14",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_T",
        "CMT_TOP_R_UPPER_T_PLLE2_CLK_IN1_INT",
        "CMT_TOP_CLK1_0",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_T",
        "CMT_TOP_R_UPPER_T_PLLE2_CLK_IN2_INT",
        "CMT_TOP_CLK0_0",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_T",
        "CMT_TOP_R_UPPER_T_PLLE2_CLK_FB_INT",
        "CMT_TOP_CLK0_1",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_T",
        "CMT_TOP_L_UPPER_T_PLLE2_CLK_IN1_INT",
        "CMT_TOP_CLK1_0",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_T",
        "CMT_TOP_L_UPPER_T_PLLE2_CLK_IN2_INT",
        "CMT_TOP_CLK0_0",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_T",
        "CMT_TOP_L_UPPER_T_PLLE2_CLK_FB_INT",
        "CMT_TOP_CLK0_1",
    );

    for tkn in ["CMT_TOP_L_LOWER_B", "CMT_TOP_R_LOWER_B"] {
        for i in 0..13 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLKDIV_{i}"),
                "CMT_PHASER_A_ICLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_ICLK_{i}"), "CMT_PHASER_A_ICLK_TOIOI");
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLKDIV_{i}"),
                "CMT_PHASER_A_OCLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_OCLK_{i}"), "CMT_PHASER_A_OCLK_TOIOI");
        }
        for i in 13..16 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLKDIV_{i}"),
                "CMT_MMCM_PHASER_IN_B_ICLKDIV",
            );
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLK_{i}"),
                "CMT_MMCM_PHASER_IN_B_ICLK",
            );
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLKDIV_{i}"),
                "CMT_MMCM_PHASER_OUT_B_OCLKDIV",
            );
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLK_{i}"),
                "CMT_MMCM_PHASER_OUT_B_OCLK",
            );
        }
        vrf.mark_merge_pip(tkn, "CMT_PHASER_A_ICLK_TOIOI", "CMT_MMCM_PHASER_IN_A_ICLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_A_ICLKDIV_TOIOI",
            "CMT_MMCM_PHASER_IN_A_ICLKDIV",
        );
        vrf.mark_merge_pip(tkn, "CMT_PHASER_A_OCLK_TOIOI", "CMT_MMCM_PHASER_OUT_A_OCLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_A_OCLKDIV_TOIOI",
            "CMT_MMCM_PHASER_OUT_A_OCLKDIV",
        );
    }
    for tkn in ["CMT_TOP_L_LOWER_T", "CMT_TOP_R_LOWER_T"] {
        for i in 0..9 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLKDIV_{i}"),
                "CMT_PHASER_B_ICLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_ICLK_{i}"), "CMT_PHASER_B_ICLK_TOIOI");
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLKDIV_{i}"),
                "CMT_PHASER_B_OCLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_OCLK_{i}"), "CMT_PHASER_B_OCLK_TOIOI");
        }
        vrf.mark_merge_pip(tkn, "CMT_PHASER_B_ICLK_TOIOI", "CMT_PHASER_IN_B_ICLK");
        vrf.mark_merge_pip(tkn, "CMT_PHASER_B_ICLKDIV_TOIOI", "CMT_PHASER_IN_B_ICLKDIV");
        vrf.mark_merge_pip(tkn, "CMT_PHASER_B_OCLK_TOIOI", "CMT_PHASER_OUT_B_OCLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_B_OCLKDIV_TOIOI",
            "CMT_PHASER_OUT_B_OCLKDIV",
        );
        vrf.mark_merge_pip(tkn, "CMT_PHASER_B_TOMMCM_ICLK", "CMT_PHASER_B_ICLK_TOIOI");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_B_TOMMCM_ICLKDIV",
            "CMT_PHASER_B_ICLKDIV_TOIOI",
        );
        vrf.mark_merge_pip(tkn, "CMT_PHASER_B_TOMMCM_OCLK", "CMT_PHASER_B_OCLK_TOIOI");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_B_TOMMCM_OCLKDIV",
            "CMT_PHASER_B_OCLKDIV_TOIOI",
        );
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_IN_A_ICLKDIV",
            "CMT_PHASER_IN_A_WRCLK_TOFIFO",
        );
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_OUT_A_OCLKDIV",
            "CMT_PHASER_OUT_A_RDCLK_TOFIFO",
        );
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_IN_B_ICLKDIV",
            "CMT_PHASER_IN_B_WRCLK_TOFIFO",
        );
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_OUT_B_OCLKDIV",
            "CMT_PHASER_OUT_B_RDCLK_TOFIFO",
        );
    }
    for tkn in ["CMT_TOP_L_UPPER_B", "CMT_TOP_R_UPPER_B"] {
        for i in 0..12 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLKDIV_{i}"),
                "CMT_PHASER_C_ICLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_ICLK_{i}"), "CMT_PHASER_C_ICLK_TOIOI");
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLKDIV_{i}"),
                "CMT_PHASER_C_OCLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_OCLK_{i}"), "CMT_PHASER_C_OCLK_TOIOI");
        }
        vrf.mark_merge_pip(tkn, "CMT_PHASER_C_ICLK_TOIOI", "CMT_PHASER_IN_C_ICLK");
        vrf.mark_merge_pip(tkn, "CMT_PHASER_C_ICLKDIV_TOIOI", "CMT_PHASER_IN_C_ICLKDIV");
        vrf.mark_merge_pip(tkn, "CMT_PHASER_C_OCLK_TOIOI", "CMT_PHASER_OUT_C_OCLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_C_OCLKDIV_TOIOI",
            "CMT_PHASER_OUT_C_OCLKDIV",
        );
    }
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_B",
        "CMT_PHASER_IN_C_ICLKDIV",
        "CMT_PHASER_IN_C_WRCLK_TOFIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_B",
        "CMT_PHASER_OUT_C_OCLKDIV",
        "CMT_PHASER_OUT_C_RDCLK_TOFIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_B",
        "CMT_PHASER_IN_D_ICLKDIV",
        "CMT_PHASER_IN_D_WRCLK_TOFIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_R_UPPER_B",
        "CMT_PHASER_OUT_D_OCLKDIV",
        "CMT_PHASER_OUT_D_RDCLK_TOFIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_B",
        "CMT_PHASER_IN_C_ICLKDIV",
        "CMT_R_PHASER_IN_C_WRCLK_FIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_B",
        "CMT_PHASER_OUT_C_OCLKDIV",
        "CMT_R_PHASER_OUT_C_RDCLK_FIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_B",
        "CMT_PHASER_IN_D_ICLKDIV",
        "CMT_R_PHASER_IN_D_WRCLK_TOFIFO",
    );
    vrf.mark_merge_pip(
        "CMT_TOP_L_UPPER_B",
        "CMT_PHASER_OUT_D_OCLKDIV",
        "CMT_R_PHASER_OUT_D_RDCLK_TOFIFO",
    );

    for tkn in ["CMT_TOP_L_UPPER_T", "CMT_TOP_R_UPPER_T"] {
        for i in 0..13 {
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_ICLKDIV_{i}"),
                "CMT_PHASER_D_ICLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_ICLK_{i}"), "CMT_PHASER_D_ICLK_TOIOI");
            vrf.mark_merge_pip(
                tkn,
                &format!("CMT_TOP_OCLKDIV_{i}"),
                "CMT_PHASER_D_OCLKDIV_TOIOI",
            );
            vrf.mark_merge_pip(tkn, &format!("CMT_TOP_OCLK_{i}"), "CMT_PHASER_D_OCLK_TOIOI");
        }
        vrf.mark_merge_pip(tkn, "CMT_PHASER_D_ICLK_TOIOI", "CMT_PLL_PHASER_IN_D_ICLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_D_ICLKDIV_TOIOI",
            "CMT_PLL_PHASER_IN_D_ICLKDIV",
        );
        vrf.mark_merge_pip(tkn, "CMT_PHASER_D_OCLK_TOIOI", "CMT_PLL_PHASER_OUT_D_OCLK");
        vrf.mark_merge_pip(
            tkn,
            "CMT_PHASER_D_OCLKDIV_TOIOI",
            "CMT_PLL_PHASER_OUT_D_OCLKDIV",
        );
    }

    for &tcrd in &endev.edev.tile_index[tcls::CMT] {
        // insert invisible buffers
        let BelInfo::SwitchBox(ref sb) = endev.edev.db[tcls::CMT].bels[bslots::SPEC_INT] else {
            unreachable!()
        };
        for item in &sb.items {
            let SwitchBoxItem::ProgBuf(buf) = item else {
                continue;
            };
            if !wires::HROW_I_CMT.contains(buf.dst.wire)
                && !wires::CCIO_CMT.contains(buf.dst.wire)
                && !wires::HCLK_CMT.contains(buf.dst.wire)
                && !wires::RCLK_CMT.contains(buf.dst.wire)
                && !wires::PERF_IN_PLL.contains(buf.dst.wire)
                && !wires::PERF_IN_PHASER.contains(buf.dst.wire)
            {
                continue;
            }
            vrf.skip_tcls_pip(tcls::CMT, buf.dst, buf.src.tw);
            let wt = endev.edev.resolve_tile_wire(tcrd, buf.dst).unwrap();
            let wf = endev.edev.resolve_tile_wire(tcrd, buf.src.tw).unwrap();
            vrf.alias_wire(wt, wf);
        }
        // fix up OMUX_CCIO
        for i in 0..4 {
            vrf.skip_tcls_pip(
                tcls::CMT,
                wires::OMUX_CCIO[i].cell(25),
                wires::CCIO_CMT[i].cell(25),
            );
        }
        for i in 0..14 {
            for j in 0..4 {
                vrf.inject_tcls_pip(
                    tcls::CMT,
                    wires::HROW_O[i].cell(25),
                    wires::CCIO_CMT[j].cell(25),
                );
            }
        }
        for wt in [
            wires::IMUX_PLL_CLKIN1_HCLK,
            wires::IMUX_PLL_CLKIN2_HCLK,
            wires::IMUX_PLL_CLKFB_HCLK,
        ]
        .into_iter()
        .flatten()
        {
            for j in 0..4 {
                vrf.inject_tcls_pip(tcls::CMT, wt.cell(25), wires::CCIO_CMT[j].cell(25));
            }
        }
        // fix up unconnected FREQ_BB/SYNC_BB at the edges
        if !endev.edev[tcrd.cell].conns.contains_id(cslots::IO_S) {
            for i in 0..4 {
                vrf.skip_tile_pip(
                    tcrd,
                    wires::CMT_FREQ_BB[i].cell(25),
                    wires::CMT_FREQ_BB_S[i].cell(25),
                );
                vrf.skip_tile_pip(
                    tcrd,
                    wires::CMT_FREQ_BB_S[i].cell(25),
                    wires::CMT_FREQ_BB[i].cell(25),
                );
            }
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB.cell(25),
                wires::CMT_SYNC_BB_S.cell(25),
            );
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB_S.cell(25),
                wires::CMT_SYNC_BB.cell(25),
            );
        }
        if let Some(conn) = endev.edev[tcrd.cell].conns.get(cslots::IO_S)
            && conn.class == ccls::IO_S_SLR
        {
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB.cell(25),
                wires::CMT_SYNC_BB_S.cell(25),
            );
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB_S.cell(25),
                wires::CMT_SYNC_BB.cell(25),
            );
        }
        if !endev.edev[tcrd.cell].conns.contains_id(cslots::IO_N) {
            for i in 0..4 {
                vrf.skip_tile_pip(
                    tcrd,
                    wires::CMT_FREQ_BB[i].cell(25),
                    wires::CMT_FREQ_BB_N[i].cell(25),
                );
                vrf.skip_tile_pip(
                    tcrd,
                    wires::CMT_FREQ_BB_N[i].cell(25),
                    wires::CMT_FREQ_BB[i].cell(25),
                );
            }
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB.cell(25),
                wires::CMT_SYNC_BB_N.cell(25),
            );
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB_N.cell(25),
                wires::CMT_SYNC_BB.cell(25),
            );
        }
        if let Some(conn) = endev.edev[tcrd.cell].conns.get(cslots::IO_N)
            && conn.class == ccls::IO_N_SLR
        {
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB.cell(25),
                wires::CMT_SYNC_BB_N.cell(25),
            );
            vrf.skip_tile_pip(
                tcrd,
                wires::CMT_SYNC_BB_N.cell(25),
                wires::CMT_SYNC_BB.cell(25),
            );
        }
    }
}

fn verify_pre_io(endev: &ExpandedNamedDevice, vrf: &mut Verifier) {
    for tcid in [tcls::HCLK_IO_HP, tcls::HCLK_IO_HR] {
        for &tcrd in &endev.edev.tile_index[tcid] {
            // insert invisible buffers
            for (wt, wf) in wires::PERF_IO.into_iter().zip(wires::PERF) {
                let dst = wt.cell(4);
                let src = wf.cell(4);
                vrf.skip_tcls_pip(tcid, dst, src);
                let wt = endev.edev.resolve_tile_wire(tcrd, dst).unwrap();
                let wf = endev.edev.resolve_tile_wire(tcrd, src).unwrap();
                vrf.alias_wire(wt, wf);
            }
            let crds = vrf.get_tile_crds(tcrd).unwrap();
            let crd = crds[RawTileId::from_idx(3)];
            vrf.mark_merge_single_pip(crd, "IOI_IMUX_RC0", "IOI_BYP3_0");
            vrf.mark_merge_single_pip(crd, "IOI_IMUX_RC1", "IOI_BYP4_0");
            let crd = crds[RawTileId::from_idx(2)];
            vrf.mark_merge_single_pip(crd, "IOI_IMUX_RC2", "IOI_BYP4_1");
            vrf.mark_merge_single_pip(crd, "IOI_IMUX_RC3", "IOI_BYP3_1");
        }
    }

    for (tcid, num_cells) in [
        (tcls::IO_HP_S, 1),
        (tcls::IO_HR_S, 1),
        (tcls::IO_HP_N, 1),
        (tcls::IO_HR_N, 1),
        (tcls::IO_HP_PAIR, 2),
        (tcls::IO_HR_PAIR, 2),
    ] {
        for c in 0..num_cells {
            vrf.skip_tcls_pip(
                tcid,
                wires::IMUX_IOI_OCLK[1].cell(c),
                wires::PHASER_OCLK90.cell(c),
            );
            for i in 0..2 {
                vrf.skip_tcls_pip(
                    tcid,
                    wires::IMUX_IOI_OCLKDIV[i].cell(c),
                    wires::IMUX_IOI_OCLKDIVF[i].cell(c),
                );
                vrf.inject_tcls_pip(
                    tcid,
                    wires::IMUX_IOI_OCLKDIV[i].cell(c),
                    wires::IMUX_IMUX[8].cell(c),
                );
                for j in 0..4 {
                    vrf.inject_tcls_pip(
                        tcid,
                        wires::IMUX_IOI_OCLKDIV[i].cell(c),
                        wires::RCLK_IO[j].cell(0),
                    );
                }
                for j in 0..6 {
                    vrf.inject_tcls_pip(
                        tcid,
                        wires::IMUX_IOI_OCLKDIV[i].cell(c),
                        wires::LCLK_IO[j].cell(0),
                    );
                }
            }
        }
    }
}

fn verify_pre_gtp(endev: &ExpandedNamedDevice, vrf: &mut Verifier) {
    let tcid = tcls::GTP_COMMON_MID;
    for &tcrd in &endev.edev.tile_index[tcid] {
        // insert invisible buffers
        for i in 0..14 {
            let dst = wires::HROW_I_GTP[i].cell(3);
            let src = wires::HROW_I[i].cell(3);
            vrf.skip_tcls_pip(tcid, dst, src);
            let wt = endev.edev.resolve_tile_wire(tcrd, dst).unwrap();
            let wf = endev.edev.resolve_tile_wire(tcrd, src).unwrap();
            vrf.alias_wire(wt, wf);
        }
    }
}

fn verify_pre_bram(endev: &ExpandedNamedDevice, vrf: &mut Verifier) {
    let BelInfo::Bel(ref bel) = endev.edev.db[tcls::BRAM].bels[bslots::BRAM] else {
        unreachable!()
    };
    for (ab, ul, pins) in [
        ("ARD", 'L', BRAM::ADDRAL.as_slice()),
        ("ARD", 'U', BRAM::ADDRAU.as_slice()),
        ("BWR", 'L', BRAM::ADDRBL.as_slice()),
        ("BWR", 'U', BRAM::ADDRBU.as_slice()),
    ] {
        for i in 0..15 {
            let wire = bel.inputs[pins[i]].wire();
            let idx = wires::IMUX_BRAM.index_of(wire.wire).unwrap();
            let cell = wire.cell.to_idx();
            vrf.mark_merge_pip(
                "BRAM_L",
                &format!("BRAM_IMUX{idx}_UTURN_{cell}"),
                &format!("BRAM_UTURN_ADDR{ab}ADDR{ul}{i}"),
            );
            vrf.mark_merge_pip(
                "BRAM_R",
                &format!("BRAM_IMUX{idx}_UTURN_{cell}"),
                &format!("BRAM_UTURN_ADDR{ab}ADDR{ul}{i}"),
            );
            vrf.mark_merge_pip(
                "BRAM_L",
                &format!("BRAM_UTURN_ADDR{ab}ADDR{ul}{i}"),
                &format!("BRAM_ADDR{ab}ADDR{ul}{i}"),
            );
            vrf.mark_merge_pip(
                "BRAM_R",
                &format!("BRAM_UTURN_ADDR{ab}ADDR{ul}{i}"),
                &format!("BRAM_ADDR{ab}ADDR{ul}{i}"),
            );
            vrf.mark_merge_pip(
                "BRAM_L",
                &format!("BRAM_IMUX_ADDR{ab}ADDR{ul}{i}"),
                &format!("BRAM_IMUX{idx}_{cell}"),
            );
            vrf.mark_merge_pip(
                "BRAM_R",
                &format!("BRAM_R_IMUX_ADDR{ab}ADDR{ul}{i}"),
                &format!("BRAM_IMUX{idx}_{cell}"),
            );
        }
        if ul == 'L' {
            let wire = bel.inputs[pins[15]].wire();
            let idx = wires::IMUX_IMUX.index_of(wire.wire).unwrap();
            let cell = wire.cell.to_idx();
            vrf.mark_merge_pip(
                "BRAM_L",
                &format!("BRAM_IMUX{idx}_UTURN_{cell}"),
                &format!("BRAM_UTURN_ADDR{ab}ADDRL15"),
            );
            vrf.mark_merge_pip(
                "BRAM_R",
                &format!("BRAM_IMUX{idx}_UTURN_{cell}"),
                &format!("BRAM_UTURN_ADDR{ab}ADDRL15"),
            );
            vrf.mark_merge_pip(
                "BRAM_L",
                &format!("BRAM_IMUX_ADDR{ab}ADDRL15"),
                &format!("BRAM_IMUX{idx}_{cell}"),
            );
            vrf.mark_merge_pip(
                "BRAM_R",
                &format!("BRAM_IMUX_R_ADDR{ab}ADDRL15"),
                &format!("BRAM_IMUX{idx}_{cell}"),
            );
        }
    }
}

fn verify_pre_gtz(vrf: &mut Verifier) {
    for (tkn, t) in [
        ("GTZ_INT_L", "LB"),
        ("GTZ_INT_R", "RB"),
        ("GTZ_INT_LB", "LT"),
        ("GTZ_INT_RB", "RT"),
    ] {
        for &crd in vrf.rd.tiles_by_kind_name(tkn) {
            for c in 0..85 {
                for r in 0..49 {
                    let wl = RawWireCoord {
                        crd,
                        wire: &format!("GTZ_INT_L_SLV_{c}_{r}"),
                    };
                    let wr = RawWireCoord {
                        crd,
                        wire: &format!("GTZ_INT_R_SLV_{cc}_{r}", cc = c + 1),
                    };
                    vrf.merge_node(wl, wr);
                    vrf.claim_pip(wl, wr);
                    vrf.claim_pip(wr, wl);
                }
            }
            for r in 0..49 {
                let rr = if tkn.ends_with('B') { 48 - r } else { r };
                let wl = RawWireCoord {
                    crd,
                    wire: &format!("GTZ_INT_{t}_SLV_{rr}"),
                };
                let wr = RawWireCoord {
                    crd,
                    wire: &format!("GTZ_INT_R_SLV_0_{r}"),
                };
                vrf.merge_node(wl, wr);
                vrf.claim_pip(wl, wr);
                vrf.claim_pip(wr, wl);
            }
        }
    }
    for &crd in vrf.rd.tiles_by_kind_name("GTZ_CLK") {
        for i in 0..32 {
            let wz = RawWireCoord {
                crd,
                wire: &format!("GTZ_CLK_GCLK{i}"),
            };
            let wi = RawWireCoord {
                crd,
                wire: &format!("GTZ_CLK_BOT_IN_GCLK{i}"),
            };
            vrf.merge_node(wz, wi);
            if matches!(i, 0 | 2 | 6 | 8 | 10 | 12) {
                vrf.claim_pip(wi, wz);
            } else {
                vrf.claim_pip(wz, wi);
            }
        }
    }
    for &crd in vrf.rd.tiles_by_kind_name("GTZ_CLK_B") {
        for i in 0..32 {
            let wz = RawWireCoord {
                crd,
                wire: &format!("GTZ_CLK_GCLK{i}"),
            };
            let wi = RawWireCoord {
                crd,
                wire: &format!("GTZ_CLK_TOP_IN_GCLK{i}"),
            };
            vrf.merge_node(wz, wi);
            if matches!(i, 31 | 29 | 25 | 23 | 21 | 19) {
                vrf.claim_pip(wi, wz);
            } else {
                vrf.claim_pip(wz, wi);
            }
        }
    }
    for r in 0..49 {
        vrf.kill_stub_cond(&format!("GTZ_INT_L_SLV_85_{r}"));
    }
}

fn verify_pre(endev: &ExpandedNamedDevice, vrf: &mut Verifier) {
    verify_pre_hclk(vrf);
    verify_pre_clk_bufg(vrf);
    verify_pre_clk_hrow(vrf);
    verify_pre_ps(vrf);
    verify_pre_cmt(endev, vrf);
    verify_pre_io(endev, vrf);
    verify_pre_gtp(endev, vrf);
    verify_pre_bram(endev, vrf);
    verify_pre_gtz(vrf);
}

pub fn verify_device(endev: &ExpandedNamedDevice, rd: &Part) {
    let mut vrf = Verifier::new(rd, &endev.ngrid);
    verify_pre(endev, &mut vrf);
    vrf.prep_int_wires();
    vrf.handle_int();
    for (tcrd, tile) in endev.edev.tiles() {
        let tcls = &endev.edev.db[tile.class];
        for slot in tcls.bels.ids() {
            verify_bel(endev.edev, &mut vrf, tcrd.bel(slot));
        }
    }
    verify_extra(&mut vrf);
    vrf.finish();
}
