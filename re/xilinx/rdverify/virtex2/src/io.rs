use prjcombine_entity::EntityId;
use prjcombine_interconnect::dir::DirH;
use prjcombine_interconnect::grid::BelCoord;
use prjcombine_re_xilinx_naming::db::RawTileId;
use prjcombine_re_xilinx_naming_virtex2::ExpandedNamedDevice;
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_virtex2::chip::{ChipKind, IoDiffKind};
use prjcombine_virtex2::defs::{bcls, bslots};
use prjcombine_virtex2::iob::IobKind;

pub fn verify_ioi(endev: &ExpandedNamedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::IOI.index_of(bcrd.slot).unwrap();
    let mut bel = vrf.verify_bel(bcrd);
    let pad = bcrd.pad(bcls::IOI::PAD);
    let edge = endev.chip.get_io_edge(pad);
    let io_info = endev.chip.get_io_info(pad);
    if io_info.pad_kind == Some(IobKind::Clk) {
        bel = bel
            .kind(["CLK_P", "CLK_N"][bslots::IOI.index_of(bcrd.slot).unwrap() % 2])
            .extra_out("BREFCLK")
            .skip_out(bcls::IOI::I);
        bel.claim_net(&[bel.wire("BREFCLK")]);
        bel.claim_net(&[bel.wire_far("BREFCLK")]);
        bel.claim_pip(bel.wire_far("BREFCLK"), bel.wire("BREFCLK"));
        if idx.is_multiple_of(2) {
            bel.claim_pip(bel.wire("I"), bel.wire_far("BREFCLK"));
        }
        bel.commit();
    } else {
        let tn = &bel.ntile.names[RawTileId::from_idx(0)];
        let is_ipad = tn.contains("IBUFS") || (tn.contains("IOIB") && idx == 2);
        let kind = if matches!(
            endev.chip.kind,
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp
        ) {
            let is_tb = matches!(io_info.bank, 0 | 2);
            match (io_info.diff, is_ipad) {
                (IoDiffKind::P(_), false) => {
                    if is_tb {
                        "DIFFMTB"
                    } else {
                        "DIFFMLR"
                    }
                }
                (IoDiffKind::P(_), true) => "DIFFMI_NDT",
                (IoDiffKind::N(_), false) => {
                    if is_tb {
                        "DIFFSTB"
                    } else {
                        "DIFFSLR"
                    }
                }
                (IoDiffKind::N(_), true) => "DIFFSI_NDT",
                (IoDiffKind::None, false) => "IOB",
                (IoDiffKind::None, true) => "IBUF",
            }
        } else {
            match (io_info.diff, is_ipad) {
                (IoDiffKind::P(_), false) => "DIFFM",
                (IoDiffKind::P(_), true) => "DIFFMI",
                (IoDiffKind::N(_), false) => "DIFFS",
                (IoDiffKind::N(_), true) => "DIFFSI",
                (IoDiffKind::None, false) => "IOB",
                (IoDiffKind::None, true) => "IBUF",
            }
        };
        bel = bel
            .kind(kind)
            .extra_out("PADOUT")
            .extra_in("DIFFI_IN")
            .extra_out("DIFFO_OUT")
            .extra_in("DIFFO_IN")
            .skip_out(bcls::IOI::CLKPAD);
        if matches!(
            endev.chip.kind,
            ChipKind::Spartan3E | ChipKind::Spartan3A | ChipKind::Spartan3ADsp
        ) {
            bel = bel
                .extra_out("PCI_RDY")
                .extra_in("PCI_CE")
                .extra_out("ODDROUT1")
                .extra_out("ODDROUT2")
                .extra_in("ODDRIN1")
                .extra_in("ODDRIN2")
                .extra_in("IDDRIN1")
                .extra_in("IDDRIN2");
        }
        if endev.chip.kind == ChipKind::Spartan3ADsp {
            bel = bel.extra_in("OAUX").extra_in("TAUX");
        }
        // diff pairing
        if !endev.chip.kind.is_virtex2() || io_info.diff != IoDiffKind::None {
            for pin in ["PADOUT", "DIFFI_IN", "DIFFO_IN", "DIFFO_OUT"] {
                bel.claim_net(&[bel.wire(pin)]);
            }
            match io_info.diff {
                IoDiffKind::P(opad) => {
                    bel.claim_pip(bel.wire("DIFFI_IN"), bel.bel_wire(opad.bel, "PADOUT"));
                }
                IoDiffKind::N(opad) => {
                    bel.claim_pip(bel.wire("DIFFI_IN"), bel.bel_wire(opad.bel, "PADOUT"));
                    bel.claim_pip(bel.wire("DIFFO_IN"), bel.bel_wire(opad.bel, "DIFFO_OUT"));
                }
                IoDiffKind::None => (),
            }
        }
        if matches!(
            endev.chip.kind,
            ChipKind::Spartan3E | ChipKind::Spartan3A | ChipKind::Spartan3ADsp
        ) {
            for pin in [
                "ODDRIN1", "ODDRIN2", "ODDROUT1", "ODDROUT2", "IDDRIN1", "IDDRIN2", "PCI_CE",
                "PCI_RDY",
            ] {
                bel.claim_net(&[bel.wire(pin)]);
            }
            // ODDR, IDDR
            if let IoDiffKind::P(opad) | IoDiffKind::N(opad) = io_info.diff {
                bel.claim_pip(bel.wire("ODDRIN1"), bel.bel_wire(opad.bel, "ODDROUT2"));
                bel.claim_pip(bel.wire("ODDRIN2"), bel.bel_wire(opad.bel, "ODDROUT1"));
                bel.claim_pip(bel.wire("IDDRIN1"), bel.bel_wire(opad.bel, "IQ1"));
                bel.claim_pip(bel.wire("IDDRIN2"), bel.bel_wire(opad.bel, "IQ2"));
            }
            bel.claim_pip(bel.wire("PCI_CE"), bel.wire_far("PCI_CE"));
            let scol = if bcrd.col < endev.chip.col_clk {
                endev.chip.col_w()
            } else {
                endev.chip.col_e()
            };
            let obel = bcrd
                .with_cr(scol, endev.chip.row_mid())
                .bel(bslots::PCILOGICSE);
            bel.verify_net(&[
                bel.vrf.bel_pip_owire(obel, "PCI_CE", 0),
                bel.wire_far("PCI_CE"),
            ]);
        }
        if endev.chip.kind == ChipKind::Spartan3ADsp {
            for pin in ["OAUX", "TAUX"] {
                bel.claim_net(&[bel.wire(pin)]);
            }
        }
        let is_clock = if endev.chip.kind.is_virtex2() {
            bcrd.col == endev.chip.col_clk - 1 || bcrd.col == endev.chip.col_clk
        } else if !endev.chip.kind.is_spartan3ea() {
            (bcrd.col == endev.chip.col_clk - 1 || bcrd.col == endev.chip.col_clk) && idx < 2
        } else {
            (0..8).any(|idx| endev.chip.get_clk_io(edge, idx) == Some(pad))
        };
        if is_clock {
            bel.claim_pip(bel.wire("CLKPAD"), bel.wire("I"));
        }
        bel.commit();
    }
}

pub fn verify_pcilogicse(endev: &ExpandedNamedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in("IRDY")
        .extra_in("TRDY")
        .extra_out("PCI_CE");
    let edge = if bcrd.col == endev.chip.col_w() {
        DirH::W
    } else if bcrd.col == endev.chip.col_e() {
        DirH::E
    } else {
        unreachable!()
    };
    let pci_rdy = endev.chip.get_pci_io(edge);
    for (pin, opad) in ["IRDY", "TRDY"].into_iter().zip(pci_rdy) {
        bel.claim_net(&[bel.wire(pin)]);
        bel.claim_pip(bel.wire(pin), bel.wire_far(pin));
        bel.claim_net(&[bel.wire_far(pin), bel.bel_wire(opad.bel, "PCI_RDY_IN")]);
        bel.claim_pip(
            bel.bel_wire(opad.bel, "PCI_RDY_IN"),
            bel.bel_wire(opad.bel, "PCI_RDY"),
        );
    }
    let (wt, wf) = bel.pip("PCI_CE", 0);
    bel.claim_net(&[bel.wire("PCI_CE"), wf]);
    bel.claim_pip(wt, wf);
    bel.claim_net(&[wt]);
    bel.commit();
}
