use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::PinDir,
    dir::DirV,
    grid::{BelCoord, ColId, DieIdExt, RowId},
};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    chip::{Chip, FabricKind, HsrSKind, PsKind},
    defs::{bslots, tcls},
    expanded::ExpandedDevice,
};

use crate::cpm::xlat_mmi_gt_pin;

pub fn verify_ps9(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    for (pn, pin) in &naming.pins {
        let mut is_spec = false;
        if pn == "NOCPORN" {
            is_spec = true;
        }
        // NPI
        if pn == "NPIRSTN"
            || pn.starts_with("NPIINTERRUPTOUT")
            || pn == "PSPMCNOCNPIAXICLK"
            || pn.starts_with("IFPMCNOCNPI")
        {
            is_spec = true;
        }
        // NOC
        if matches!(
            pn.as_str(),
            "PCIE_NOC_0"
                | "PCIE_NOC_1"
                | "PMC_NOC_0"
                | "RPU_NOC_0"
                | "NCI_NOC_0"
                | "NCI_NOC_1"
                | "CCI_NOC_0"
                | "CCI_NOC_1"
                | "CCI_NOC_2"
                | "CCI_NOC_3"
                | "NOC_PCIE_0"
                | "NOC_PMC_0"
                | "NOC_CCI_0"
                | "NOC_CCI_1"
                | "NOC_NCI_0"
                | "NOC_NCI_1"
        ) {
            continue;
        }
        // XRAM
        if pn.starts_with("CPUR5")
            || pn.contains("OCM2")
            || pn.starts_with("LPDLSBUS")
            || pn.starts_with("LPDXRAM")
            || pn.starts_with("OCMRAWRSTN")
            || pn == "PSXRAMSUPPLYSTATUSVCCINTLPD"
            || pn.starts_with("SCANOCM2")
            || pn.starts_with("XRAMLPD")
        {
            is_spec = true;
        }
        // CPM
        if pn.starts_with("CPM")
            || pn == "EFUSECPMCCIXDIS"
            || pn.starts_with("IFCPMPS")
            || pn.starts_with("IFPSCPM")
            || pn.starts_with("LPDCPM")
            || pn.starts_with("PERST")
        {
            is_spec = true;
        }
        if is_spec {
            let &(_, dir) = pin.orig_pin.as_ref().unwrap();
            if dir == PinDir::Input {
                if pn == "NPIINTERRUPTOUT_16_" && edev.chips[bcrd.die].fabric == FabricKind::Ulvt {
                    bel = bel.extra_in_far(pn);
                } else {
                    bel = bel.extra_in_far_claim(pn);
                }
            } else {
                bel = bel.extra_out_far_claim(pn);
            }
        }
    }
    let chip = edev.chips[bcrd.die];
    match chip.hsr_s {
        HsrSKind::Xpio(_) | HsrSKind::Ssit => {
            let cells = [
                bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::S)),
                bcrd.die.cell(
                    ColId::from_idx(if chip.hsr_s == HsrSKind::Ssit { 2 } else { 9 }),
                    chip.row_hsr(DirV::S),
                ),
            ];
            bel = bel
                .extra_out_far_to_far("PCIE_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
                .extra_out_far_to_far("PCIE_NOC_1", cells[0].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
                .extra_out_far_to_far("PMC_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
                .extra_out_far_to_far("RPU_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
                .extra_out_far_to_far("NCI_NOC_0", cells[1].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
                .extra_out_far_to_far("NCI_NOC_1", cells[1].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_0", cells[1].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_1", cells[1].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_2", cells[1].bel(bslots::HNOC_PS_NMU[4]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_3", cells[1].bel(bslots::HNOC_PS_NMU[5]), "AXI_IN")
                .extra_in_far_from_far(
                    "NOC_PCIE_0",
                    cells[0].bel(bslots::HNOC_PS_NSU[0]),
                    "AXI_OUT",
                )
                .extra_in_far_from_far("NOC_PMC_0", cells[0].bel(bslots::HNOC_PS_NSU[1]), "AXI_OUT")
                .extra_in_far_from_far("NOC_CCI_0", cells[1].bel(bslots::HNOC_PS_NSU[0]), "AXI_OUT")
                .extra_in_far_from_far("NOC_CCI_1", cells[1].bel(bslots::HNOC_PS_NSU[1]), "AXI_OUT")
                .extra_in_far_from_far("NOC_NCI_0", cells[1].bel(bslots::HNOC_PS_NSU[2]), "AXI_OUT")
                .extra_in_far_from_far("NOC_NCI_1", cells[1].bel(bslots::HNOC_PS_NSU[3]), "AXI_OUT")
        }
        HsrSKind::X5io(_) => {
            let cells = [
                bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::S)),
                bcrd.die.cell(ColId::from_idx(4), chip.row_hsr(DirV::S)),
                bcrd.die.cell(ColId::from_idx(16), chip.row_hsr(DirV::S)),
            ];
            bel = bel
                .extra_out_far_to_far("PCIE_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
                .extra_out_far_to_far("PCIE_NOC_1", cells[0].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
                .extra_out_far_to_far("PMC_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
                .extra_out_far_to_far("RPU_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
                .extra_out_far_to_far("NCI_NOC_0", cells[2].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
                .extra_out_far_to_far("NCI_NOC_1", cells[2].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_0", cells[2].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_1", cells[1].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_2", cells[2].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
                .extra_out_far_to_far("CCI_NOC_3", cells[1].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
                .extra_in_far_from_far(
                    "NOC_PCIE_0",
                    cells[0].bel(bslots::HNOC_PS_NSU[0]),
                    "AXI_OUT",
                )
                .extra_in_far_from_far("NOC_PMC_0", cells[0].bel(bslots::HNOC_PS_NSU[1]), "AXI_OUT")
                .extra_in_far_from_far("NOC_NCI_0", cells[2].bel(bslots::HNOC_PS_NSU[0]), "AXI_OUT")
                .extra_in_far_from_far("NOC_NCI_1", cells[2].bel(bslots::HNOC_PS_NSU[1]), "AXI_OUT")
                .extra_in_far_from_far("NOC_CCI_0", cells[2].bel(bslots::HNOC_PS_NSU[2]), "AXI_OUT")
                .extra_in_far_from_far("NOC_CCI_1", cells[2].bel(bslots::HNOC_PS_NSU[3]), "AXI_OUT")
        }
    }

    bel.commit();
}

pub fn verify_psxl(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd).kind("PSX");
    for (pn, pin) in &naming.pins {
        let mut is_spec = false;
        if pn.starts_with("NOCLPXSPARE")
            || pn == "NOCPORN"
            || pn == "NOCRSTN"
            || pn.starts_with("SYSRST")
        {
            is_spec = true;
        }
        // NPI
        if pn == "NPIRSTN"
            || pn.starts_with("NPIINTERRUPTOUT")
            || pn == "PSXLPMXNOCNPIAXICLK"
            || pn.starts_with("IFPMXNOCNPI")
        {
            is_spec = true;
        }
        // NOC
        if matches!(
            pn.as_str(),
            "PSXL_NOC_PCIE_0"
                | "PSXL_NOC_PCIE_1"
                | "PSXL_NOC_PCIE_2"
                | "PSXL_NOC_PCIE_3"
                | "PMC_NOC_0"
                | "RPU_NOC_0"
                | "CMN_NOC_0"
                | "CMN_NOC_1"
                | "CMN_NOC_2"
                | "CMN_NOC_3"
                | "CMN_NOC_4"
                | "CMN_NOC_5"
                | "CMN_NOC_6"
                | "CMN_NOC_7"
                | "CMN_NOC_8"
                | "NOC_PCIE_0"
                | "NOC_PMC_0"
                | "CMN_NOC_CCI_0"
                | "CMN_NOC_CCI_1"
                | "CMN_NOC_CCI_2"
                | "CMN_NOC_CCI_3"
        ) {
            continue;
        }
        // CPM
        if matches!(
            pn.as_str(),
            "CPM5ISPRESENT" | "CPMOSCCLKDIV2" | "CPMPSXCXSCLK" | "DBGLPDRAWRSTN"
        ) || pn.starts_with("EFUSE")
            || pn.starts_with("IFCMNCML256")
            || pn.starts_with("IFCPMPS")
            || pn.starts_with("IFCPM5NPSX")
            || pn.starts_with("IFPSCPM")
            || pn.starts_with("IFPSXCPM5N")
            || pn.starts_with("LPDCPM")
            || pn.starts_with("PERST")
        {
            is_spec = true;
        }
        if pn.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pn);
            continue;
        }
        if is_spec {
            let &(_, dir) = pin.orig_pin.as_ref().unwrap();
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pn);
            } else {
                bel = bel.extra_out_far_claim(pn);
            }
        }
    }

    let chip = edev.chips[bcrd.die];
    let cells = [
        bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(4), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(16), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(20), chip.row_hsr(DirV::S)),
    ];
    bel = bel
        .extra_out_far_to_far("CMN_NOC_8", cells[0].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("PMC_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("RPU_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_in_far_from_far("NOC_PMC_0", cells[0].bel(bslots::HNOC_PS_NSU[0]), "AXI_OUT")
        .extra_in_far_from_far(
            "NOC_PCIE_0",
            cells[0].bel(bslots::HNOC_PS_NSU[1]),
            "AXI_OUT",
        )
        .extra_out_far_to_far("CMN_NOC_0", cells[1].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_1", cells[1].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_2", cells[1].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_3", cells[1].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_4", cells[2].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_5", cells[2].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_6", cells[2].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_7", cells[2].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far(
            "PSXL_NOC_PCIE_0",
            cells[3].bel(bslots::HNOC_PS_NMU[0]),
            "AXI_IN",
        )
        .extra_out_far_to_far(
            "PSXL_NOC_PCIE_1",
            cells[3].bel(bslots::HNOC_PS_NMU[1]),
            "AXI_IN",
        )
        .extra_out_far_to_far(
            "PSXL_NOC_PCIE_2",
            cells[3].bel(bslots::HNOC_PS_NMU[2]),
            "AXI_IN",
        )
        .extra_out_far_to_far(
            "PSXL_NOC_PCIE_3",
            cells[3].bel(bslots::HNOC_PS_NMU[3]),
            "AXI_IN",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_0",
            cells[3].bel(bslots::HNOC_PS_NSU[0]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_1",
            cells[3].bel(bslots::HNOC_PS_NSU[1]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_2",
            cells[3].bel(bslots::HNOC_PS_NSU[2]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_3",
            cells[3].bel(bslots::HNOC_PS_NSU[3]),
            "AXI_OUT",
        );

    bel.commit();
}

pub fn verify_psxc(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd).kind("PS11");
    for (pn, pin) in &naming.pins {
        let mut is_spec = false;
        if pn == "NOCPORN"
            || pn == "NOCRSTN"
            || pn.starts_with("SYSRST")
            || pn.starts_with("PSPLFPDRCLKCLKAPEX")
        {
            is_spec = true;
        }
        // NPI
        if pn == "NPIRSTN"
            || pn.starts_with("NPIINTERRUPTOUT")
            || pn == "PSXLPMXNOCNPIAXICLK"
            || pn.starts_with("IFPMXNOCNPI")
        {
            is_spec = true;
        }
        // NOC
        if matches!(
            pn.as_str(),
            "PMC_NOC_0"
                | "RPU_NOC_0"
                | "CMN_NOC_0"
                | "CMN_NOC_1"
                | "CMN_NOC_2"
                | "CMN_NOC_3"
                | "CMN_NOC_4"
                | "CMN_NOC_5"
                | "CMN_NOC_6"
                | "CMN_NOC_7"
                | "CMN_NOC_8"
                | "MMI_NOC_0"
                | "NOC_PMC_0"
                | "CMN_NOC_CCI_0"
                | "CMN_NOC_CCI_1"
                | "CMN_NOC_CCI_2"
                | "CMN_NOC_CCI_3"
        ) {
            continue;
        }
        // MMI
        if matches!(
            pn.as_str(),
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
        ) || pn.starts_with("IFMMIPS")
            || pn.starts_with("IFPSMMI")
            || pn.starts_with("IFPSPLMMI")
            || pn.starts_with("PSPLMMI")
            || pn.starts_with("IFPSGTYP")
            || pn.starts_with("MMIRCLK")
            || pn.starts_with("PSMMI")
            || pn.starts_with("PCIE")
        {
            is_spec = true;
        }
        if pn.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pn);
            continue;
        }
        if is_spec {
            let &(_, dir) = pin.orig_pin.as_ref().unwrap();
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pn);
            } else {
                bel = bel.extra_out_far_claim(pn);
            }
        }
    }

    let chip = edev.chips[bcrd.die];
    let cells = [
        bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(4), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(16), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(20), chip.row_hsr(DirV::S)),
    ];
    bel = bel
        .extra_out_far_to_far("CMN_NOC_8", cells[0].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("PMC_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("RPU_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_in_far_from_far("NOC_PMC_0", cells[0].bel(bslots::HNOC_PS_NSU[0]), "AXI_OUT")
        .extra_out_far_to_far("CMN_NOC_0", cells[1].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_1", cells[1].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_2", cells[1].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_3", cells[1].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_4", cells[2].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_5", cells[2].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_6", cells[2].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_7", cells[2].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("MMI_NOC_0", cells[3].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_in_far_from_far(
            "CMN_NOC_CCI_0",
            cells[3].bel(bslots::HNOC_PS_NSU[0]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_1",
            cells[3].bel(bslots::HNOC_PS_NSU[1]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_2",
            cells[3].bel(bslots::HNOC_PS_NSU[2]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_3",
            cells[3].bel(bslots::HNOC_PS_NSU[3]),
            "AXI_OUT",
        );

    bel.commit();
}

pub fn verify_psxt(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd).kind("PS11_B");
    let gt = bcrd
        .delta(0, Chip::ROWS_PER_REG as i32 * 3)
        .bel(bslots::GTYP);

    for (pn, pin) in &naming.pins {
        let &(_, dir) = pin.orig_pin.as_ref().unwrap();
        let mut is_spec = false;
        if pn == "NOCPORN"
            || pn == "NOCRSTN"
            || pn.starts_with("SYSRST")
            || pn.starts_with("PSPLFPDRCLKCLKAPEX")
        {
            is_spec = true;
        }
        // NPI
        if pn == "NPIRSTN"
            || pn.starts_with("NPIINTERRUPTOUT")
            || pn == "PSXLPMXNOCNPIAXICLK"
            || pn.starts_with("IFPMXNOCNPI")
        {
            is_spec = true;
        }
        // MMI specials
        if matches!(
            pn.as_str(),
            "GTYPSYSRST1N"
                | "GTYPSYSRST2N"
                | "GTYPSYSRST3N"
                | "NPICLKEPXPIPE"
                | "PORRESETN"
                | "IFMMIGTYPBUFGTCLKBUFGT_14_"
        ) {
            is_spec = true;
        }

        // NOC
        if matches!(
            pn.as_str(),
            "PMC_NOC_0"
                | "RPU_NOC_0"
                | "CMN_NOC_0"
                | "CMN_NOC_1"
                | "CMN_NOC_2"
                | "CMN_NOC_3"
                | "CMN_NOC_4"
                | "CMN_NOC_5"
                | "CMN_NOC_6"
                | "CMN_NOC_7"
                | "CMN_NOC_8"
                | "MMI_NOC_0"
                | "NOC_PMC_0"
                | "CMN_NOC_CCI_0"
                | "CMN_NOC_CCI_1"
                | "CMN_NOC_CCI_2"
                | "CMN_NOC_CCI_3"
        ) {
            continue;
        }
        if let Some(opin) = xlat_mmi_gt_pin(pn) {
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pn);
            } else {
                bel = bel.extra_out_far(pn);
            }
            bel.verify_net(&[bel.wire_far(pn), bel.bel_wire_far(gt, &opin)]);
            continue;
        }
        if matches!(
            pn.as_str(),
            "IFPMXNOCNPIAXIARREADY"
                | "IFPMXNOCNPIAXIAWREADY"
                | "IFPMXNOCNPIAXIBVALID"
                | "IFPMXNOCNPIAXIRVALID"
                | "IFPMXNOCNPIAXIWREADY"
        ) {
            // oh my god what is wrong with this chip did they slop it into existence
            bel = bel.extra_out_claim(pn);
            continue;
        }

        if pn.starts_with("UNUSEDTIEHIGH") {
            bel = bel.extra_out_far(pn);
            continue;
        }
        if is_spec {
            let &(_, dir) = pin.orig_pin.as_ref().unwrap();
            if dir == PinDir::Input {
                bel = bel.extra_in_far_claim(pn);
            } else {
                bel = bel.extra_out_far_claim(pn);
            }
        }
    }

    let chip = edev.chips[bcrd.die];
    let cells = [
        bcrd.die.cell(ColId::from_idx(0), chip.row_hsr(DirV::S)),
        bcrd.die.cell(ColId::from_idx(4), chip.row_hsr(DirV::S)),
    ];
    bel = bel
        .extra_out_far_to_far("CMN_NOC_8", cells[0].bel(bslots::HNOC_PS_NMU[4]), "AXI_IN")
        .extra_out_far_to_far("PMC_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[5]), "AXI_IN")
        .extra_out_far_to_far("RPU_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[6]), "AXI_IN")
        .extra_in_far_from_far("NOC_PMC_0", cells[0].bel(bslots::HNOC_PS_NSU[0]), "AXI_OUT")
        .extra_out_far_to_far("CMN_NOC_0", cells[0].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_1", cells[0].bel(bslots::HNOC_PS_NMU[1]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_2", cells[0].bel(bslots::HNOC_PS_NMU[2]), "AXI_IN")
        .extra_out_far_to_far("CMN_NOC_3", cells[0].bel(bslots::HNOC_PS_NMU[3]), "AXI_IN")
        .extra_out_far_to_far("MMI_NOC_0", cells[1].bel(bslots::HNOC_PS_NMU[0]), "AXI_IN")
        .extra_in_far_from_far(
            "CMN_NOC_CCI_0",
            cells[1].bel(bslots::HNOC_PS_NSU[0]),
            "AXI_OUT",
        )
        .extra_in_far_from_far(
            "CMN_NOC_CCI_1",
            cells[1].bel(bslots::HNOC_PS_NSU[1]),
            "AXI_OUT",
        );

    bel.commit();
}

pub fn verify_xram(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let is_nc = edev[edev.bel_tile(bcrd)].class == tcls::XRAM_NC;
    let mut bel = vrf.verify_bel(bcrd);
    let obel = bcrd
        .die
        .cell(edev.col_cfrm[bcrd.die], RowId::from_idx(0))
        .bel(bslots::PS9);
    for (pn, pin) in &naming.pins {
        let &(_, dir) = pin.orig_pin.as_ref().unwrap();
        if pn.contains("PLXRAM") || pn.contains("XRAMPL") {
            if is_nc {
                if dir == PinDir::Input {
                    bel = bel.extra_in_far_claim(pn);
                } else {
                    bel = bel.extra_out_far_claim(pn);
                }
            }
        } else {
            bel.verify_net(&[bel.wire_far(pn), bel.bel_wire_far(obel, pn)]);
            if dir == PinDir::Input {
                bel = bel.extra_in_far(pn);
            } else {
                bel = bel.extra_out_far(pn);
            }
        }
    }
    bel.commit();
}

pub fn verify_npi_nir(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd);
    let ps = bcrd
        .die
        .cell(edev.col_cfrm[bcrd.die], chip.row_s())
        .bel(match chip.ps {
            PsKind::Ps9 | PsKind::Ps9Xram | PsKind::Ps9Cpm4 | PsKind::Ps9Cpm5 | PsKind::Ps9Cpm6 => {
                bslots::PS9
            }
            PsKind::PsXCpm5N => bslots::PSXL,
            PsKind::PsXcMmi => bslots::PSXC,
            PsKind::PsXt => bslots::PSXT,
        });
    for (pn, pin) in &naming.pins {
        let &(_, dir) = pin.orig_pin.as_ref().unwrap();
        let opn = if let Some(suffix) = pn.strip_prefix("IF_NPI_AXI_") {
            if ps.slot == bslots::PS9 {
                format!("IFPMCNOCNPIAXI{suffix}")
            } else {
                format!("IFPMXNOCNPIAXI{suffix}")
            }
        } else if let Some(suffix) = pn.strip_prefix("NPI_INTERRUPT_OUT") {
            format!("NPIINTERRUPTOUT{suffix}")
        } else if pn == "NPI_CLK" {
            "NPICLK".to_string()
        } else if pn == "NPI_RST_N" {
            "NPIRSTN".to_string()
        } else if pn == "PMC_CLK" {
            if ps.slot == bslots::PS9 {
                "PSPMCNOCNPIAXICLK".to_string()
            } else {
                "PSXLPMXNOCNPIAXICLK".to_string()
            }
        } else {
            unreachable!()
        };
        let mut skip = false;
        if opn == "NPIINTERRUPTOUT_16_"
            && edev.chips[bcrd.die].fabric == FabricKind::Ulvt
            && ps.slot == bslots::PS9
        {
            skip = true;
        }
        if ps.slot == bslots::PSXT
            && matches!(
                opn.as_str(),
                "IFPMXNOCNPIAXIARREADY"
                    | "IFPMXNOCNPIAXIAWREADY"
                    | "IFPMXNOCNPIAXIBVALID"
                    | "IFPMXNOCNPIAXIRVALID"
                    | "IFPMXNOCNPIAXIWREADY"
            )
        {
            skip = true;
        }
        if !skip {
            bel.verify_net(&[bel.wire_far(pn), bel.bel_wire_far(ps, &opn)]);
        }
        if dir == PinDir::Input {
            bel = bel.extra_in_far(pn);
        } else {
            bel = bel.extra_out_far(pn);
        }
    }
    bel.commit();
}
