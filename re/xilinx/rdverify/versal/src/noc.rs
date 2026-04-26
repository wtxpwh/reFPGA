use prjcombine_entity::EntityId;
use prjcombine_interconnect::{db::BelInfo, grid::BelCoord};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    defs::{bcls, bslots, tcls},
    expanded::ExpandedDevice,
    noc::BelCoordExt,
};

pub fn verify_vnoc_nxu512(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match (edev[tcrd].class, bcrd.slot) {
        (tcls::VNOC, bslots::VNOC_NSU512) => "NOC_NSU512",
        (tcls::VNOC, bslots::VNOC_NMU512) => "NOC_NMU512",
        (tcls::VNOC2, bslots::VNOC_NSU512) => "NOC2_NSU512",
        (tcls::VNOC2, bslots::VNOC_NMU512) => "NOC2_NMU512",
        (tcls::VNOC4, bslots::VNOC_NSU512) => "NOC2_NSU512",
        (tcls::VNOC4, bslots::VNOC_NMU512) => "NOC2_NMU512",
        (tcls::HNOC_NIC_BLI, bslots::HNOC_NSU512) => "NOC2_NSU512",
        (tcls::HNOC_NIC_BLI, _) => "NOC2_NMU512",
        (tcls::HNOC_X5IO_DDRMC, _) => "NOC2_NMU512",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nps(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::VNOC => "NOC_NPS_VNOC",
        tcls::VNOC2
        | tcls::HNOC_NIC_CPM_CDX
        | tcls::HNOC_X5IO_PS9_A
        | tcls::HNOC_X5IO_PS9_B
        | tcls::HNOC_X5IO_PS9_C
        | tcls::HNOC_X5IO_PSX_A
        | tcls::HNOC_X5IO_PSX_B
        | tcls::HNOC_X5IO_PSXL_C
        | tcls::HNOC_X5IO_PSXC_C
        | tcls::HNOC_X5IO_CROSS
        | tcls::HNOC_X5IO_CROSS_ALT
        | tcls::HNOC_X5IO_DDRMC
        | tcls::HNOC_X5IO_CAP => "NOC2_NPS5555",
        tcls::HNOC_NIC_CPM_DPU
        | tcls::HNOC_NIC_BLI
        | tcls::HNOC_NIC_VNOC
        | tcls::HNOC_NIC_HNICX => "NOC2_NPS7575",
        tcls::HNOC_XPIO
        | tcls::HNOC_XPIO_PS9_A
        | tcls::HNOC_XPIO_PS9_B
        | tcls::HNOC_XPIO_PS9_C
        | tcls::HNOC_ME
        | tcls::HNOC_ME_VNOC
        | tcls::HNOC_HBM_VNOC => "NOC_NPS5555",
        tcls::HNOC_SSIT_PS9_A
        | tcls::HNOC_SSIT_PS9_BD
        | tcls::HNOC_SSIT_PS9_C
        | tcls::HNOC_SSIT
        | tcls::HNOC_SSIT2_V => "NOC_NPS7575",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("OUT_1")
        .extra_out_far_claim("OUT_2")
        .extra_out_far_claim("OUT_3")
        .extra_in_far("IN_0")
        .extra_in_far("IN_1")
        .extra_in_far("IN_2")
        .extra_in_far("IN_3")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nps6(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("OUT_1")
        .extra_out_far_claim("OUT_2")
        .extra_out_far_claim("OUT_3")
        .extra_out_far_claim("OUT_4")
        .extra_out_far_claim("OUT_5")
        .extra_in_far("IN_0")
        .extra_in_far("IN_1")
        .extra_in_far("IN_2")
        .extra_in_far("IN_3")
        .extra_in_far("IN_4")
        .extra_in_far("IN_5")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nps4(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("OUT_REQ_0")
        .extra_out_far_claim("OUT_REQ_1")
        .extra_out_far_claim("OUT_REQ_2")
        .extra_out_far_claim("OUT_REQ_3")
        .extra_out_far_claim("OUT_RESP_0")
        .extra_out_far_claim("OUT_RESP_1")
        .extra_out_far_claim("OUT_RESP_2")
        .extra_out_far_claim("OUT_RESP_3")
        .extra_in_far("IN_REQ_0")
        .extra_in_far("IN_REQ_1")
        .extra_in_far("IN_REQ_2")
        .extra_in_far("IN_REQ_3")
        .extra_in_far("IN_RESP_0")
        .extra_in_far("IN_RESP_1")
        .extra_in_far("IN_RESP_2")
        .extra_in_far("IN_RESP_3")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_xbr2x4(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("OUT_1")
        .extra_out_far_claim("OUT_2")
        .extra_out_far_claim("OUT_3")
        .extra_out_far_claim("OUT_NMU")
        .extra_out_far_claim("OUT_NSU")
        .extra_in_far("IN_0")
        .extra_in_far("IN_1")
        .extra_in_far("IN_2")
        .extra_in_far("IN_3")
        .extra_in_far("IN_NMU")
        .extra_in_far("IN_NSU")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_xbr4x2(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("OUT_1")
        .extra_out_far_claim("OUT_2")
        .extra_out_far_claim("OUT_3")
        .extra_out_far_claim("OUT_DMC_0")
        .extra_out_far_claim("OUT_DMC_1")
        .extra_in_far("IN_0")
        .extra_in_far("IN_1")
        .extra_in_far("IN_2")
        .extra_in_far("IN_3")
        .extra_in_far("IN_DMC_0")
        .extra_in_far("IN_DMC_1")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nmu_hbm2e(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("TO_PHY")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .extra_in_far("IF_BLI_PHY_DIR_IN_BLI_CLK");
    bel.verify_net(&[
        bel.wire_far("IF_BLI_PHY_DIR_IN_BLI_CLK"),
        bel.wire_far("CLK"),
    ]);
    bel.commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_scan(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let tcls = &edev.db[edev[tcrd].class];
    let BelInfo::Bel(ref bel_info) = tcls.bels[bcrd.slot] else {
        unreachable!()
    };
    let mut bel = vrf.verify_bel(bcrd);
    if matches!(
        edev[tcrd].class,
        tcls::HNOC_NIC_CPM_CDX
            | tcls::HNOC_X5IO_PS9_A
            | tcls::HNOC_X5IO_PS9_B
            | tcls::HNOC_X5IO_PSX_A
            | tcls::HNOC_X5IO_PSX_B
    ) {
        bel = bel
            .extra_in_far("IF_NOC2_SCAN_CNTL_IN_SCAN_CSSD_TRIG_N")
            .extra_in_far("IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
            .extra_in_far("IF_NOC2_SCAN_CNTL_IN_SCAN_MODE_RST_N")
            .extra_in_far("IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N")
            .extra_in_far("IF_NOC2_SCAN_CNTL_IN_SCAN_EDT_UPDT_N");
        for i in 0..15 {
            bel = bel
                .extra_in_far(format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"))
                .extra_out_far_claim(format!("NOC2_SCAN_CHNL_TO_PL_{i}_"));
        }
        for i in 0..14 {
            bel = bel.extra_in_far(format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"));
        }
    } else {
        if !bel_info
            .inputs
            .contains_id(bcls::NOC2_SCAN::IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N)
        {
            bel = bel
                .extra_in_far_claim("IF_NOC2_SCAN_CNTL_IN_SCAN_CSSD_TRIG_N")
                .extra_in_far_claim("IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
                .extra_in_far_claim("IF_NOC2_SCAN_CNTL_IN_SCAN_MODE_RST_N")
                .extra_in_far_claim("IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N")
                .extra_in_far_claim("IF_NOC2_SCAN_CNTL_IN_SCAN_EDT_UPDT_N");
        }
        for i in 0..15 {
            if !bel_info
                .inputs
                .contains_id(bcls::NOC2_SCAN::NOC2_SCAN_CHNL_FROM_PL[i])
            {
                bel = bel
                    .extra_in_far_claim(format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"))
                    .extra_out_far_claim(format!("NOC2_SCAN_CHNL_TO_PL_{i}_"));
            }
        }
        for i in 0..14 {
            if !bel_info
                .inputs
                .contains_id(bcls::NOC2_SCAN::NOC2_SCAN_CHNL_MASK_FROM_PL[i])
            {
                bel = bel.extra_in_far_claim(format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"));
            }
        }
    }
    bel.commit();
}

pub fn verify_vcu2_nmu(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .kind("NOC2_NMU128")
        .extra_in_far_claim("AXI_IN")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_vcu2_nsu(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .kind("NOC2_NSU128")
        .extra_out_far_claim("AXI_OUT")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc2_nmu256(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .kind("NOC2_NMU256")
        .extra_in_far_claim("AXI_IN")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc2_nsu256(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .kind("NOC2_NSU256")
        .extra_out_far_claim("AXI_OUT")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nmu128(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let name = vrf.ngrid.get_bel_name(bcrd).unwrap();
    let kind = if name.starts_with("NOC2") {
        "NOC2_NMU128"
    } else {
        "NOC_NMU128"
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_in_far_claim("AXI_IN")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nsu128(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let name = vrf.ngrid.get_bel_name(bcrd).unwrap();
    let kind = if name.starts_with("NOC2") {
        "NOC2_NSU128"
    } else {
        "NOC_NSU128"
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("AXI_OUT")
        .extra_out_far_claim("TO_NOC")
        .extra_in_far("FROM_NOC")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_ncrb(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let kind = if edev.has_bel(bcrd.bel(bslots::HNOC_NIDB_V[0])) {
        "NOC_NCRB_SSIT"
    } else {
        "NOC_NCRB"
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("OUT_1")
        .extra_in_far("IN_0")
        .extra_in_far("IN_1")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_nidb(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::HNOC_SSIT => "NOC_NIDB",
        tcls::HNOC_SSIT2_V => "NOC2_NIDBV",
        tcls::HNOC_SSIT2_H => "NOC2_NIDBH",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("OUT_0")
        .extra_out_far_claim("TX")
        .extra_in_far("IN_0")
        .extra_in_far("RX")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_npp_rptr(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("OUT_LEFT")
        .extra_out_far_claim("OUT_RIGHT")
        .extra_in_far("IN_LEFT")
        .extra_in_far("IN_RIGHT")
        .commit();
    verify_noc_conns(edev, vrf, bcrd);
}

pub fn verify_noc_conns(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let bcls = edev.db.bel_class(bcrd.slot);
    let num = match bcls {
        bcls::NOC_NMU128
        | bcls::NOC_NMU256
        | bcls::NOC_NMU512
        | bcls::NOC_NSU128
        | bcls::NOC_NSU256
        | bcls::NOC_NSU512
        | bcls::NOC_NMU_HBM2E => 1,
        bcls::NOC_NCRB | bcls::NOC_NPP_RPTR | bcls::NOC_NIDB | bcls::DDRMC5 => 2,
        bcls::NOC_NPS | bcls::DDRMC | bcls::HBM_CHNL => 4,
        bcls::NOC_NPS6 | bcls::NOC2_NPS6X => 6,
        bcls::NOC_NPS4 => 8,
        bcls::NOC2_XBR2X4 => 6,
        bcls::NOC2_XBR4X2 => 6,
        _ => unreachable!(),
    };
    for i in 0..num {
        let port = bcrd.noc_port(i);
        let pin_i = match bcls {
            bcls::NOC_NMU128
            | bcls::NOC_NMU256
            | bcls::NOC_NMU512
            | bcls::NOC_NSU128
            | bcls::NOC_NSU256
            | bcls::NOC_NSU512
            | bcls::NOC_NMU_HBM2E => "FROM_NOC".to_string(),
            bcls::NOC_NPS | bcls::NOC_NPS6 | bcls::NOC2_NPS6X | bcls::NOC_NCRB => {
                format!("IN_{i}")
            }
            bcls::DDRMC | bcls::DDRMC5 => format!("FROM_NOC_{i}"),
            bcls::HBM_CHNL => format!("MC_FROM_NOC_{i}"),
            bcls::NOC_NPP_RPTR => ["IN_LEFT", "IN_RIGHT"][i].to_string(),
            bcls::NOC_NIDB => ["IN_0", "RX"][i].to_string(),
            bcls::NOC_NPS4 => {
                if i < 4 {
                    format!("IN_REQ_{i}")
                } else {
                    format!("IN_RESP_{ii}", ii = i - 4)
                }
            }
            bcls::NOC2_XBR2X4 => {
                if i < 4 {
                    format!("IN_{i}")
                } else {
                    ["IN_NMU", "IN_NSU"][i - 4].to_string()
                }
            }
            bcls::NOC2_XBR4X2 => {
                if i < 4 {
                    format!("IN_{i}")
                } else {
                    format!("IN_DMC_{ii}", ii = i - 4)
                }
            }
            _ => unreachable!(),
        };
        if let Some(&oport) = edev.noc.get(&port) {
            let obcls = edev.db.bel_class(oport.slot);
            let idx = oport.port.to_idx();
            let pin_o = match obcls {
                bcls::NOC_NMU128
                | bcls::NOC_NMU256
                | bcls::NOC_NMU512
                | bcls::NOC_NSU128
                | bcls::NOC_NSU256
                | bcls::NOC_NSU512
                | bcls::NOC_NMU_HBM2E => "TO_NOC".to_string(),
                bcls::NOC_NPS | bcls::NOC_NPS6 | bcls::NOC2_NPS6X | bcls::NOC_NCRB => {
                    format!("OUT_{idx}")
                }
                bcls::DDRMC | bcls::DDRMC5 => format!("TO_NOC_{idx}"),
                bcls::HBM_CHNL => format!("MC_TO_NOC_{idx}"),
                bcls::NOC_NPP_RPTR => ["OUT_LEFT", "OUT_RIGHT"][idx].to_string(),
                bcls::NOC_NIDB => ["OUT_0", "TX"][idx].to_string(),
                bcls::NOC_NPS4 => {
                    if idx < 4 {
                        format!("OUT_RESP_{idx}")
                    } else {
                        format!("OUT_REQ_{ii}", ii = idx - 4)
                    }
                }
                bcls::NOC2_XBR2X4 => {
                    if idx < 4 {
                        format!("OUT_{idx}")
                    } else {
                        ["OUT_NMU", "OUT_NSU"][idx - 4].to_string()
                    }
                }
                bcls::NOC2_XBR4X2 => {
                    if idx < 4 {
                        format!("OUT_{idx}")
                    } else {
                        format!("OUT_DMC_{ii}", ii = idx - 4)
                    }
                }
                _ => unreachable!(),
            };
            vrf.verify_net(&[
                vrf.bel_wire_far(bcrd, &pin_i),
                vrf.bel_wire_far(oport.bel, &pin_o),
            ]);
        } else {
            if bcls != bcls::NOC_NIDB {
                vrf.claim_net(&[vrf.bel_wire_far(bcrd, &pin_i)]);
            }
        }
    }
}
