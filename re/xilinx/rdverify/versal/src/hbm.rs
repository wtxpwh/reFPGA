use prjcombine_entity::EntityId;
use prjcombine_interconnect::{coord::BelCoord, db::PinDir};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    defs::{bcls, bslots},
    expanded::ExpandedDevice,
};

pub fn verify_hbm_chnl(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let bidx = bcrd.col.to_idx() / 2 % 8;
    let chidx = bidx / 2 + bidx % 2 * 4;
    let chabc = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][chidx];
    let ms = bcrd.delta(-(bidx as i32) * 2, 0).bel(bslots::HBM_MS);
    let mut bel = vrf.verify_bel(bcrd).kind("HBM_MC").skip_auto();

    // NOC connections (verified by verify_noc_conns)
    for i in 0..4 {
        bel = bel
            .extra_out_far_claim(format!("MC_TO_NOC_{i}"))
            .extra_in_far(format!("MC_FROM_NOC_{i}"));
    }

    // irrelevant
    for pin in [
        "MC_HBMMC_NOC_RST_N_MC_GL",
        "MC_HBMMC_NPI_CLK_MC_GL",
        "MC_HBMMC_SPARE_MC_GL_0_",
        "MC_HBMMC_SPARE_MC_GL_1_",
        "MC_HBMMC_SPARE_MC_GL_2_",
        "MC_HBMMC_SYS_RST1_N_MC_GL",
        "MC_HBMMC_SYS_RST2_N_MC_GL",
        "MC_HBMMC_SYS_RST3_N_MC_GL",
    ] {
        bel = bel.extra_in_far(pin);
    }

    // PHY connections
    let mut mc_phy_conn = vec![];
    let mut phy_mc_conn = vec![];
    for ch in 0..2 {
        for (pin, n) in [("CK", 2), ("CKE", 2), ("COL", 18), ("ROW", 14)] {
            for i in 0..n {
                mc_phy_conn.push((
                    format!("MC_CH{ch}_HBMMC_AW_{pin}_MC_DFI_CMD_{i}_"),
                    format!("PHY_CH{ch}_PHY_CHNL_CORE_AW_{pin}_MC2PHY_DFI_AW_{i}_"),
                ));
            }
        }
        for (pin, n) in [
            ("DATA_DQS", 8),
            ("DATA_DQ_EN", 8),
            ("DATA", 256),
            ("DBI", 32),
            ("MASK", 32),
            ("PAR_EN", 8),
            ("PAR", 8),
        ] {
            for i in 0..n {
                mc_phy_conn.push((
                    format!("MC_CH{ch}_HBMMC_DW_{pin}_MC_DFI_WR_{i}_"),
                    format!("PHY_CH{ch}_PHY_CHNL_CORE_DW_{pin}_MC2PHY_DFI_WR_{i}_"),
                ));
            }
        }
        for i in 0..8 {
            phy_mc_conn.push((
                format!("PHY_CH{ch}_PHY_CHNL_CORE_DW_WDATA_DERR_MC2PHY_DFI_WR_{i}_"),
                format!("MC_CH{ch}_HBMMC_DW_WDATA_DERR_MC_DFI_WR_{i}_"),
            ));
        }
        for (pin, n) in [
            ("DATA", 256),
            ("DBI", 32),
            ("MASK", 32),
            ("PAR", 8),
            ("RDDATA_VALID", 8),
        ] {
            for i in 0..n {
                phy_mc_conn.push((
                    format!("PHY_CH{ch}_PHY_CHNL_CORE_DW_{pin}_MC2PHY_DFI_RD_{i}_"),
                    format!("MC_CH{ch}_HBMMC_DW_{pin}_MC_DFI_RD_{i}_"),
                ));
            }
        }
    }
    for (pin, n) in [("AW_AERR_N", 2), ("DW_DBI_BYTE_DISABLE", 16), ("TEMP", 3)] {
        for i in 0..n {
            phy_mc_conn.push((
                format!("PHY_PHY_CHNL_CORE_{pin}_MC2PHY_MISC_{i}_"),
                format!("MC_HBMMC_{pin}_{i}_"),
            ));
        }
    }
    for pin in [
        "CTRLUPD_ACK",
        "INIT_COMPLETE",
        "TRAINING_COMPLETE",
        "PHYUPD_REQ",
        "PHY_LP_STATE",
    ] {
        phy_mc_conn.push((
            format!("PHY_PHY_CHNL_CORE_{pin}_MC2PHY_MISC"),
            format!("MC_HBMMC_{pin}"),
        ));
    }
    for pin in [
        "AW_CK_DIS",
        "AW_TX_INDX_LD",
        "CTRLUPD_REQ",
        "DW_RX_INDX_LD",
        "DW_TX_INDX_LD",
        "LP_PWR_E_REQ",
        "LP_PWR_X_REQ",
        "LP_SR_E_REQ",
        "PHYUPD_ACK",
    ] {
        mc_phy_conn.push((
            format!("MC_HBMMC_{pin}"),
            format!("PHY_PHY_CHNL_CORE_{pin}_MC2PHY_MISC"),
        ));
    }
    phy_mc_conn.extend([
        (
            "PHY_PHY_CHNL_CORE_CATTRIP_MC2PHY_MISC".to_string(),
            "MC_HBMMC_CATTRIP_DFI_MISC".to_string(),
        ),
        (
            "PHY_PHY_CHNL_CORE_DFI_CLK_OUT".to_string(),
            "MC_HBMMC_DFI_CLK".to_string(),
        ),
        (
            "PHY_PHY_CHNL_CORE_DFI_RST_N".to_string(),
            "MC_HBMMC_DFI_RESET_N".to_string(),
        ),
    ]);
    mc_phy_conn.extend([
        (
            "MC_HBMMC_INIT_START_DFI_MISC".to_string(),
            "PHY_PHY_CHNL_CORE_INIT_START_MC2PHY_MISC".to_string(),
        ),
        (
            "MC_HBMMC_DFI_CLK_OUT".to_string(),
            "PHY_PHY_CHNL_CORE_DFI_CLK_IN".to_string(),
        ),
    ]);
    for (pin_mc, _pin_phy) in &mc_phy_conn {
        bel = bel.extra_out_far_claim(pin_mc);
    }
    for (pin_phy, pin_mc) in &phy_mc_conn {
        bel = bel.extra_in_far_from_far(pin_mc, bcrd, pin_phy);
    }

    bel.commit();
    crate::noc::verify_noc_conns(edev, vrf, bcrd);

    let mut bel = vrf.verify_bel(bcrd).kind("HBM_PHY_CHNL").sub(1);

    // irrelevant
    for pin in [
        "PHY_PHY_CHNL_CORE_SYS_RST1_N",
        "PHY_PHY_CHNL_CORE_SYS_RST2_N",
        "PHY_PHY_CHNL_CORE_SYS_RST3_N",
    ] {
        bel = bel.extra_in_far(pin);
    }
    bel = bel.extra_in_far_claim_bus("PHY_PHY_CHNL_CORE_HBM_CHNL_NUM", 3);

    // NMU connections
    for i in 0..4 {
        let obel = bcrd
            .delta(1 - (i as i32 / 2), 0)
            .bel(bslots::NOC_NMU_HBM2E[i % 2]);
        bel = bel.extra_in_far_from_far(format!("PHY_TO_PHY_{i}"), obel, "TO_PHY");
    }

    // MC connections
    for (pin_mc, pin_phy) in &mc_phy_conn {
        bel = bel.extra_in_far_from_far(pin_phy, bcrd, pin_mc);
    }
    for (pin_phy, _pin_mc) in &phy_mc_conn {
        bel = bel.extra_out_far_claim(pin_phy);
    }

    // IO connections
    let mut phy_io_conn = vec![];
    let mut io_phy_conn = vec![];
    for pin in naming.pins.keys() {
        if let Some(rest) = pin.strip_prefix("PHY_PHY_CHNL_CORE_PHY2IOB_") {
            if rest.starts_with("TX_DATA") {
                continue;
            }
            let io_pin = if rest.starts_with("SNEAK_ENB") {
                format!("IO_HBM_IO_CHNL_CORE_PHY2IOB_DW_{rest}")
            } else if rest.starts_with("RDQS") {
                format!("IO_HBM_IO_CHNL_CORE_PHY2{rest}")
            } else if rest.starts_with("DFT_PNDRV") {
                format!("IO_HBM_IO_CHNL_CORE_PHY2IOB_DW_{rest}")
            } else if let Some(rest) = rest.strip_prefix("TX_DERR_TSTATE_") {
                format!("IO_HBM_IO_CHNL_CORE_PHY2IOB_TX_DER_TSTATE_{rest}")
            } else {
                format!("IO_HBM_IO_CHNL_CORE_PHY2IOB_{rest}")
            };
            phy_io_conn.push((pin.clone(), io_pin));
        }
        if let Some(rest) = pin.strip_prefix("PHY_PHY_CHNL_CORE_PHY2DLL_") {
            let io_pin = format!("IO_HBM_IO_CHNL_CORE_PHY2DLL_{rest}");
            phy_io_conn.push((pin.clone(), io_pin));
        }
        if let Some(rest) = pin.strip_prefix("PHY_PHY_CHNL_CORE_PHY2RDQS") {
            let io_pin = format!("IO_HBM_IO_CHNL_CORE_PHY2RDQS{rest}");
            phy_io_conn.push((pin.clone(), io_pin));
        }
    }
    for pin in naming.pins.keys() {
        if let Some(rest) = pin.strip_prefix("PHY_PHY_CHNL_CORE_IOB2PHY_") {
            let io_pin = format!("IO_HBM_IO_CHNL_CORE_IOB2PHY_{rest}");
            io_phy_conn.push((io_pin, pin.clone()));
        }
    }
    for (pin_io, pin_phy) in &io_phy_conn {
        bel = bel.extra_in_far_from_far(pin_phy, bcrd, pin_io);
    }
    for (pin_phy, _pin_io) in &phy_io_conn {
        bel = bel.extra_out_far_claim(pin_phy);
    }
    bel = bel.extra_in_far_from_far(
        "PHY_PHY_CHNL_CORE_HBM_PICLK",
        bcrd,
        "PHY_PHY_CHNL_CORE_IOB2PHY_RX_STBC_0_",
    );

    // MS connection
    bel = bel
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_CHNL_EN")
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_HBM_CORE_SOFT_RST")
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_HBM_RDQS_TRNG_REQ")
        .extra_out_far_claim_bus("PHY_PHY_CHNL_CORE_DEBUG_BUS_TRFC_GEN", 29)
        .extra_out_far_claim_bus("PHY_PHY_CHNL_CORE_DELTA_CALC_DEBUG_BUS", 16)
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_DFI_CATTRIP",
            ms,
            format!("PHY_PHY_MS_CORE_CH{chabc}_DFI_CATTRIP"),
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_DFI_TEMP",
            ms,
            format!("PHY_PHY_MS_CORE_CH{chabc}_DFI_TEMP"),
            3,
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_DEBUG_DW_SELECT",
            ms,
            "PHY_DEBUG_DW_SELECT",
            2,
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_PHY_TRFC_DBG_MODE",
            ms,
            "PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_MODE",
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_PHY_TRFC_DBG_LD",
            ms,
            "PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_LD",
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_HBM_PHY_TRFC_INDX",
            ms,
            "PHY_PHY_MS_CORE_HBM_PHY_TRFC_INDX",
            5,
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_HBM_PHY_TRFC_DBG_IN",
            ms,
            "PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_IN",
            12,
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_HS_CLK",
            ms,
            format!("IO_HBM_IO_MS_CORE_CH{chabc}_IOB2PHY_HS_TX_CLKDIV2"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_WS_CLK",
            ms,
            "PHY_PHY_MS_CORE_HBM_WS_CLK",
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_TILE_RST_N",
            ms,
            format!("PHY_PHY_MS_CORE_HBM_TILE_RST_N_{chidx}_"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HBM_RDQS_TRNG_GNT",
            ms,
            format!("PHY_PHY_MS_CORE_HBM_RDQS_TRNG_GNT_{chidx}_"),
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_HDLL2PHY_DL_LPF_DAT",
            ms,
            "PHY_PHY_MS_CORE_DLL2PHY_DL_LPF_DAT",
            10,
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HDLL2PHY_DL_LPF_RDY",
            ms,
            "PHY_PHY_MS_CORE_DLL2PHY_DL_LPF_RDY",
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_HDLL2PHY_LOCKED",
            ms,
            "PHY_PHY_MS_CORE_DLL2PHY_LOCKED",
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_INT_PHY2IOB_TX_AER",
            ms,
            format!("PHY_TX_AERR_{chidx}_"),
        );
    for i in 0..4 {
        bel = bel.extra_in_far_from_far(
            format!("PHY_PHY_CHNL_CORE_INT_PHY2IOB_TX_DERR_{i}_"),
            ms,
            format!("PHY_TX_DERR_{ii}_", ii = chidx * 4 + i),
        );
    }
    bel = bel
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_MS2PHY_RX_CATTRIP",
            ms,
            "PHY_PHY_MS_CORE_MS2PHY_RX_CATTRIP_DEBOUNCED",
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_MS2PHY_RX_TEMP",
            ms,
            "PHY_PHY_MS_CORE_MS2PHY_RX_TEMP_DEBOUNCED",
            3,
        )
        .extra_out_far_claim_bus("PHY_PHY_CHNL_CORE_PHY2IOB_TX_DATA_T", 4)
        .extra_in_far_claim_bus("PHY_PHY_CHNL_CORE_DEBUG_OUT", 16)
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_PHY2MSCLK_AW_DIS")
        .extra_out_far_claim_bus("PHY_PHY_CHNL_CORE_PHY2MSCLK_DW_DIS", 4)
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_PHY2PLL_PSCLK")
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_PHY2PLL_PSINCDEC")
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_PHY_NPI_PRESET_N",
            ms,
            format!("PHY_CH{chidx}_PHY_MS_CORE_PRESET_N_MS_P_CH_NPI"),
        )
        .extra_in_far_from_far("PHY_PHY_CHNL_CORE_PWRDWN", ms, "PWRDWN")
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_RD_LFSR_CMPR_DT_VLD",
            ms,
            format!("PHY_PHY_MS_CORE_RD_LFSR_CMPR_DT_VLD_{chidx}_"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_RD_LFSR_CMPR_ERR_GNT",
            ms,
            format!("PHY_PHY_MS_CORE_RD_LFSR_CMPR_ERR_GNT_{chidx}_"),
        )
        .extra_out_far_claim("PHY_PHY_CHNL_CORE_RD_LFSR_CMPR_ERR_REQ")
        .extra_in_far_from_far_bus(
            "PHY_PHY_CHNL_CORE_TAP_ADDR",
            ms,
            "PHY_PHY_MS_CORE_TAP2CHNL_ADDR",
            5,
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_TAP_CAPTUREWR",
            ms,
            format!("PHY_PHY_MS_CORE_TAP2CHNL_CAPTUREWR_{chidx}_"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_TAP_SHIFTWR",
            ms,
            format!("PHY_PHY_MS_CORE_TAP2CHNL_SHIFTWR_{chidx}_"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_TAP_UPDATEWR",
            ms,
            format!("PHY_PHY_MS_CORE_TAP2CHNL_UPDATEWR_{chidx}_"),
        )
        .extra_in_far_from_far(
            "PHY_PHY_CHNL_CORE_TAP_WSI",
            ms,
            format!("PHY_PHY_MS_CORE_TAP2CHNL_WSI_{chidx}_"),
        )
        .extra_out_far_claim_bus("PHY_PHY_CHNL_CORE_TAP_INST_TYPE", 2);

    bel.commit();

    let mut bel = vrf.verify_bel(bcrd).kind("HBM_IO_CHNL").sub(2).skip_auto();

    // PHY connection
    for (pin_io, _pin_phy) in &io_phy_conn {
        bel = bel.extra_out_far_claim(pin_io);
    }
    for (pin_phy, pin_io) in &phy_io_conn {
        bel = bel.extra_in_far(pin_io);
        bel.verify_net(&[bel.wire_far(pin_phy), bel.wire_far(pin_io)]);
    }

    // MS connection
    for i in 0..4 {
        bel = bel
            .extra_in_far_from_far(
                format!("IO_HBM_IO_CHNL_CORE_HBM_HS_TX_CLKB_{i}_"),
                ms,
                format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKB_{chabc}{i}"),
            )
            .extra_in_far_from_far(
                format!("IO_HBM_IO_CHNL_CORE_HBM_HS_TX_CLKDIV2_B_{i}_"),
                ms,
                format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKDIV2B_{chabc}{i}"),
            );
    }
    bel = bel
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_HBM_HS_TX_CLKB_4_",
            ms,
            format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKB_{chabc}"),
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_HBM_HS_TX_CLKDIV2_B_4_",
            ms,
            format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKDIV2B_{chabc}"),
        )
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_AW_PD_EN",
            ms,
            format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_AW_PD_EN_BUF"),
            6,
        )
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_AW_PU_EN",
            ms,
            format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_AW_PU_EN_BUF"),
            6,
        )
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_AW_SLICE_EN",
            ms,
            format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_SLICE_EN_BUF"),
            6,
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_POR_B_VCCINT_IO_AW",
            ms,
            format!("IO_HBM_IO_MS_CORE_POR_B_VCCINT_IO_BUF_{chabc}"),
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_POR_B_VCCO_AW",
            ms,
            format!("IO_HBM_IO_MS_CORE_POR_B_VCCO_BUF_{chabc}"),
        );
    for i in 0..4 {
        bel = bel
            .extra_in_far_from_far_bus(
                format!("IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_DW_PD_EN_{i}"),
                ms,
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_PD_EN_BUF_{chabc}{i}"),
                6,
            )
            .extra_in_far_from_far_bus(
                format!("IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_DW_PU_EN_{i}"),
                ms,
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_PU_EN_BUF_{chabc}{i}"),
                6,
            )
            .extra_in_far_from_far_bus(
                format!("IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_DW_SLICE_EN_{i}"),
                ms,
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_SLICE_EN_BUF_{chabc}{i}"),
                6,
            )
            .extra_in_far_from_far(
                format!("IO_HBM_IO_CHNL_CORE_POR_B_VCCINT_IO_DW_{i}_"),
                ms,
                format!("IO_HBM_IO_MS_CORE_POR_B_VCCINT_IO_BUF_{chabc}{i}"),
            )
            .extra_in_far_from_far(
                format!("IO_HBM_IO_CHNL_CORE_POR_B_VCCO_DW_{i}_"),
                ms,
                format!("IO_HBM_IO_MS_CORE_POR_B_VCCO_BUF_{chabc}{i}"),
            );
    }
    bel = bel
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_PBIAS",
            ms,
            "IO_HBM_IO_MS_CORE_MIDSTCK2IOB_PBIAS",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_CHNL_CORE_MIDSTCK2IOB_VREF",
            ms,
            "IO_HBM_IO_MS_CORE_MIDSTCK2IOB_VREF",
        )
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_CHNL_CORE_PHY2IOB_TX_DATA_T",
            ms,
            format!("PHY_CH{chidx}_PHY2IOB_TX_DATA_T"),
            8,
        )
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_CHNL_CORE_PHY2IOB_TX_DATA_C",
            ms,
            format!("PHY_CH{chidx}_PHY2IOB_TX_DATA_C"),
            8,
        )
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_DQ_MONOUT", 4)
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_DQ_MONOUT1", 4)
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_RDQS_MONOUT", 4)
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_RDQST_MONOUT", 4)
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_WDQS_MONOUT", 4)
        .extra_out_far_claim_bus("IO_HBM_IO_CHNL_CORE_IOB2C4_DW_WDQST_MONOUT", 4);

    bel.commit();
}

pub fn verify_hbm_ms(vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let chnl: [_; 8] = core::array::from_fn(|i| {
        bcrd.delta(i as i32 % 4 * 4 + i as i32 / 4 * 2, 0)
            .bel(bslots::HBM_CHNL)
    });
    let hpll = bcrd.bel(bslots::HPLL);
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HBM_PHY_MS")
        .skip_in(bcls::HBM_MS::CCIO_REF_CLK);

    // CHNL connection
    for (chidx, chnl) in chnl.into_iter().enumerate() {
        let chabc = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][chidx];
        bel = bel
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_TAP_CHNL_EN_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_CHNL_EN",
            )
            .extra_out_far_claim(format!("PHY_PHY_MS_CORE_HBM_TILE_RST_N_{chidx}_"))
            .extra_out_far_claim(format!("PHY_PHY_MS_CORE_HBM_RDQS_TRNG_GNT_{chidx}_"))
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_HBM_RDQS_TRNG_REQ_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_HBM_RDQS_TRNG_REQ",
            )
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_HBM_CORE_SOFT_RST_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_HBM_CORE_SOFT_RST",
            )
            .extra_in_far_from_far_bus(
                format!("PHY_PHY_MS_CORE_CH{chabc}_DEBUG_BUS_TRFC_GEN"),
                chnl,
                "PHY_PHY_CHNL_CORE_DEBUG_BUS_TRFC_GEN",
                29,
            )
            .extra_in_far_from_far_bus(
                format!("PHY_PHY_MS_CORE_CH{chabc}_DELTA_CALC_DEBUG_BUS"),
                chnl,
                "PHY_PHY_CHNL_CORE_DELTA_CALC_DEBUG_BUS",
                16,
            )
            .extra_out_far_claim(format!("PHY_PHY_MS_CORE_CH{chabc}_DFI_CATTRIP"))
            .extra_out_far_claim_bus(format!("PHY_PHY_MS_CORE_CH{chabc}_DFI_TEMP"), 3)
            .extra_in_far_from_far_bus(
                format!("PHY_CH{chidx}_IOB2PHY_RX_AW_DERR"),
                chnl,
                "PHY_PHY_CHNL_CORE_IOB2PHY_RX_AW_AERR",
                2,
            )
            .extra_in_far_from_far_bus(
                format!("PHY_CH{chidx}_IOB2PHY_RX_DW_DERR"),
                chnl,
                "PHY_PHY_CHNL_CORE_IOB2PHY_RX_DW_DERR",
                8,
            )
            .extra_in_far_from_far_bus(
                format!("PHY_CH{chidx}_IOB2PHY_RX_STB"),
                chnl,
                "IO_HBM_IO_CHNL_CORE_IOB2PHY_RX_STB",
                4,
            )
            .extra_in_far_from_far_bus(
                format!("PHY_CH{chidx}_IOB2PHY_RX_STBC"),
                chnl,
                "IO_HBM_IO_CHNL_CORE_IOB2PHY_RX_STBC",
                4,
            )
            .extra_in_far_from_far_bus(
                format!("PHY_CH{chidx}_PHY2IOB_TX_DATA_T_INT"),
                chnl,
                "PHY_PHY_CHNL_CORE_PHY2IOB_TX_DATA_T",
                4,
            )
            .extra_out_far_claim(format!("PHY_CH{chidx}_PHY_MS_CORE_PRESET_N_MS_P_CH_NPI"))
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_PHY2PLL_PSCLK_INT_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_PHY2PLL_PSCLK",
            )
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_PHY2PLL_PSINCDEC_INT_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_PHY2PLL_PSINCDEC",
            )
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_RD_LFSR_CMPR_ERR_REQ_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_RD_LFSR_CMPR_ERR_REQ",
            )
            .extra_out_far_claim_bus(format!("PHY_CH{chidx}_PHY2IOB_TX_DATA_T"), 8)
            .extra_out_far_claim_bus(format!("PHY_CH{chidx}_PHY2IOB_TX_DATA_C"), 8)
            .extra_in_far_from_far(
                format!("PHY_PHY_MS_CORE_TRAINING_COMPLETE_{chidx}_"),
                chnl,
                "PHY_PHY_CHNL_CORE_TRAINING_COMPLETE_MC2PHY_MISC",
            )
            // crap
            .extra_in_far_claim_bus(
                format!("PHY_CH{chidx}_PHY_MS_CORE_ADDR_SIZE_MS_P_CH_NPI"),
                3,
            )
            .extra_in_far_claim_bus(
                format!("PHY_CH{chidx}_PHY_MS_CORE_INTERRUPT_MS_P_CH_NPI"),
                20,
            );
    }
    bel = bel
        .extra_out_far_claim("PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_LD")
        .extra_out_far_claim("PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_MODE")
        .extra_out_far_claim_bus("PHY_DEBUG_DW_SELECT", 2)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_HBM_PHY_TRFC_INDX", 5)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_HBM_PHY_TRFC_DBG_IN", 12)
        .extra_out_far_claim("PHY_PHY_MS_CORE_HBM_WS_CLK")
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_DLL2PHY_DL_LPF_DAT", 10)
        .extra_out_far_claim("PHY_PHY_MS_CORE_DLL2PHY_DL_LPF_RDY")
        .extra_out_far_claim("PHY_PHY_MS_CORE_DLL2PHY_LOCKED")
        .extra_out_far_claim_bus("PHY_TX_AERR", 8)
        .extra_out_far_claim_bus("PHY_TX_DERR", 32)
        .extra_out_far_claim("PHY_PHY_MS_CORE_MS2PHY_RX_CATTRIP_DEBOUNCED")
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_MS2PHY_RX_TEMP_DEBOUNCED", 3)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_RD_LFSR_CMPR_DT_VLD", 8)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_RD_LFSR_CMPR_ERR_GNT", 8)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_TAP2CHNL_ADDR", 5)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_TAP2CHNL_CAPTUREWR", 8)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_TAP2CHNL_SHIFTWR", 8)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_TAP2CHNL_UPDATEWR", 8)
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_TAP2CHNL_WSI", 8);
    for i in 0..32 {
        bel = bel.extra_out_far_to_far(
            format!("PHY_PHY_MS_CORE_DEBUG_OUT_{i}_"),
            chnl[[2, 6][i / 16]],
            format!("PHY_PHY_CHNL_CORE_DEBUG_OUT_{ii}_", ii = i % 16),
        );
    }
    for i in 0..16 {
        bel = bel.extra_in_far_from_far(
            format!("PHY_PHY_MS_CORE_TAP_INST_TYPE_{i}_"),
            chnl[i / 2],
            format!("PHY_PHY_CHNL_CORE_TAP_INST_TYPE_{ii}_", ii = i % 2),
        );
    }

    // IO connection
    bel = bel
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_DLL2PHY_CLKDIV2",
            bcrd,
            "IO_HBM_IO_MS_CORE_DLL2PHY_CLKDIV2",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_DLL2PHY_PD_OUT",
            bcrd,
            "IO_HBM_IO_MS_CORE_DLL2PHY_PD_OUT",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_CCIO2PHY_REF_CLK",
            bcrd,
            "IO_HBM_IO_MS_CORE_CCIO2PHY_REF_CLK",
        )
        .extra_out_far_claim("PHY_PHY_MS_CORE_DRAM_RST_N")
        .extra_in_far_from_far_bus(
            "PHY_PHY_MS_CORE_MS2PHY_DCI_COMP_OUT",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_DCI_COMP_OUT",
            2,
        )
        .extra_in_far_from_far_bus(
            "PHY_PHY_MS_CORE_MS2PHY_RX_TEMP",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_TEMP",
            3,
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_CAPTUREWR",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_CAPTUREWR",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_SHIFTWR",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_SHIFTWR",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_UPDATEWR",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_UPDATEWR",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_SELECTWIR",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_SELECTWIR",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_WRCK",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_WRCK",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_WRST",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_WRST",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_WSI",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_WSI",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_RESET",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_RESET",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_MS2PHY_RX_CATTRIP",
            bcrd,
            "IO_HBM_IO_MS_CORE_MS2PHY_RX_CATTRIP",
        )
        .extra_in_far_from_far(
            "PHY_PHY_MS_CORE_PLL2PHY_CLKOUTPHY",
            bcrd,
            "IO_HBM_IO_MS_CORE_PLL2PHY_CLKOUTPHY_TEST",
        )
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_CAPTUREWR")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_SHIFTWR")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_UPDATEWR")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_SELECTWIR")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_WRCK")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_WRST_N")
        .extra_out_far_claim("PHY_PHY_MS_CORE_TAP_WSI")
        .extra_in_far_from_far_bus(
            "PHY_PHY_MS_CORE_TAP_WSO",
            bcrd,
            "IO_HBM_IO_MS_CORE_TAP_WSO",
            8,
        )
        .extra_out_far_claim_bus("PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL", 64);

    for pin in naming.pins.keys() {
        if pin.starts_with("PHY_PHY_MS_CORE_PHY2DLL")
            || pin.starts_with("PHY_PHY_MS_CORE_PHY2HBMEXT")
            || pin.starts_with("PHY_PHY_MS_CORE_PHY2IOB_DFT")
            || pin.starts_with("PHY_PHY_MS_CORE_PHY2MS")
        {
            bel = bel.extra_out_far_claim(pin);
        }
    }

    // HPLL connection
    for (pin, opin) in [
        ("PHY_PHY_MS_CORE_PHY2PLL_CLKOUTPHY_EN_GATED", "CLKOUTPHYEN"),
        ("PHY_PHY_MS_CORE_PHY2PLL_PWRDWN", "PWRDWN"),
        ("PHY_PHY_MS_CORE_PHY2PLL_RST", "RST"),
        ("PHY_PHY_MS_CORE_PHY2PLL_SCAN_CLK", "SCANCLK"),
        ("PHY_PHY_MS_CORE_PHY2PLL_SCAN_EN_B", "SCANENB"),
        ("PHY_PHY_MS_CORE_PHY2PLL_SCAN_IN", "SCANIN"),
        ("PHY_PHY_MS_CORE_PHY2PLL_SCAN_MODE_B", "SCANMODEB"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_0_", "TESTIN_0_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_1_", "TESTIN_1_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_2_", "TESTIN_2_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_3_", "TESTIN_3_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_4_", "TESTIN_4_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_5_", "TESTIN_5_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_6_", "TESTIN_6_"),
        ("PHY_PHY_MS_CORE_PHY2PLL_TEST_IN_7_", "TESTIN_7_"),
    ] {
        bel = bel.extra_out_far_to_far(pin, hpll, opin);
    }
    for (pin, opin) in [
        ("PHY_PHY_MS_CORE_PLL2PHY_CLKOUT0", "CLKOUT0"),
        ("PHY_PHY_MS_CORE_PLL2PHY_TMUXOUT", "TMUXOUT"),
        ("PHY_PHY_MS_CORE_PLL2PHY_PLL_LOCKED", "LOCKED"),
        ("PHY_PHY_MS_CORE_PLL2PHY_SCAN_OUT", "SCANOUT"),
        ("PHY_PHY_MS_CORE_PLL2PHY_TESTOUT_0_", "LOCKED_FB"),
        ("PHY_PHY_MS_CORE_PLL2PHY_TESTOUT_1_", "TESTOUT"),
        ("PHY_PHY_MS_CORE_PLL2PHY_TESTOUT_2_", "LOCKED1_DESKEW"),
        ("PHY_PHY_MS_CORE_PLL2PHY_TESTOUT_3_", "LOCKED2_DESKEW"),
    ] {
        bel = bel.extra_in_far_from_far(pin, hpll, opin);
    }

    // garbage
    bel = bel
        .extra_in_far_claim_bus("PHY_PHY_MS_CORE_ADDR_SIZE_MSL_NPI", 3)
        .extra_in_far_claim_bus("PHY_PHY_MS_CORE_ADDR_SIZE_MS_XPLL", 3)
        .extra_in_far("PHY_PHY_MS_CORE_NOC_RST_N_MISC")
        .extra_in_far("PHY_PHY_MS_CORE_MCLK_MISC")
        .extra_in_far("PHY_PHY_MS_CORE_SYS_RST1_N_MISC")
        .extra_in_far("PHY_PHY_MS_CORE_SYS_RST2_N_MISC")
        .extra_in_far("PHY_PHY_MS_CORE_SYS_RST3_N_MISC")
        .extra_in_far_bus("PHY_PHY_MS_CORE_SPARE_MISC", 3);

    bel.commit();

    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("HBM_IO_MS")
        .sub(1)
        .skip_auto()
        .extra_in("CCIO_REF_CLK");

    // CHNL connection
    for (chidx, chnl) in chnl.into_iter().enumerate() {
        let chabc = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][chidx];
        bel = bel
            .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_CH{chabc}_IOB2PHY_HS_TX_CLKDIV2"))
            .extra_in_far_from_far(
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_PHY2IOB_AW_RST_N"),
                chnl,
                "IO_HBM_IO_CHNL_CORE_PHY2IOB_AW_RST_N",
            )
            .extra_in_far_from_far(
                format!(
                    "IO_HBM_IO_MS_CORE_PHY2MSCLK_DIS_{ii}_",
                    ii = 32 + [5, 1, 4, 0, 7, 3, 6, 2][chidx]
                ),
                chnl,
                "PHY_PHY_CHNL_CORE_PHY2MSCLK_AW_DIS",
            );
        for i in 0..4 {
            bel = bel.extra_in_far_from_far(
                format!(
                    "IO_HBM_IO_MS_CORE_PHY2MSCLK_DIS_{ii}_",
                    ii = [9, 8, 1, 0, 25, 24, 17, 16][chidx] + (3 - i) * 2
                ),
                chnl,
                format!("PHY_PHY_CHNL_CORE_PHY2MSCLK_DW_DIS_{i}_"),
            );
        }
        for i in 0..4 {
            bel = bel
                .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKB_{chabc}{i}"))
                .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKDIV2B_{chabc}{i}"));
        }
        bel = bel
            .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKB_{chabc}"))
            .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_HBM_HS_TX_CLKDIV2B_{chabc}"))
            .extra_out_far_claim_bus(
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_AW_PD_EN_BUF"),
                6,
            )
            .extra_out_far_claim_bus(
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_AW_PU_EN_BUF"),
                6,
            )
            .extra_out_far_claim_bus(
                format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_SLICE_EN_BUF"),
                6,
            )
            .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_POR_B_VCCINT_IO_BUF_{chabc}"))
            .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_POR_B_VCCO_BUF_{chabc}"));
        for i in 0..4 {
            bel = bel
                .extra_out_far_claim_bus(
                    format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_PD_EN_BUF_{chabc}{i}"),
                    6,
                )
                .extra_out_far_claim_bus(
                    format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_PU_EN_BUF_{chabc}{i}"),
                    6,
                )
                .extra_out_far_claim_bus(
                    format!("IO_HBM_IO_MS_CORE_CH{chabc}_MIDSTCK2IOB_SLICE_EN_BUF_{chabc}{i}"),
                    6,
                )
                .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_POR_B_VCCINT_IO_BUF_{chabc}{i}"))
                .extra_out_far_claim(format!("IO_HBM_IO_MS_CORE_POR_B_VCCO_BUF_{chabc}{i}"));
        }
    }
    bel = bel
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MIDSTCK2IOB_PBIAS")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MIDSTCK2IOB_VREF");

    // PHY connection
    bel = bel
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_DLL2PHY_CLKDIV2")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_DLL2PHY_PD_OUT")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_CCIO2PHY_REF_CLK")
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DRAM_RST_N",
            bcrd,
            "PHY_PHY_MS_CORE_DRAM_RST_N",
        )
        .extra_out_far_claim_bus("IO_HBM_IO_MS_CORE_MS2PHY_DCI_COMP_OUT", 2)
        .extra_out_far_claim_bus("IO_HBM_IO_MS_CORE_MS2PHY_RX_TEMP", 3)
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_CAPTUREWR")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_SHIFTWR")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_UPDATEWR")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_SELECTWIR")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_WRCK")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_WRST")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_WSI")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_RESET")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_MS2PHY_RX_CATTRIP")
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2IOB_MS_LPBK_EN",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2MS_LPBK_EN",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_EN_PLL2PHY_CLKOUTPHY_TEST",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2MS_SPARE_7_",
        )
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_PLL2PHY_CLKOUTPHY_TEST")
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_CAPTUREWR",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_CAPTUREWR",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_SHIFTWR",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_SHIFTWR",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_UPDATEWR",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_UPDATEWR",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_SELECTWIR",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_SELECTWIR",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_WRCK",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_WRCK",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_TAP_WRST_N",
            bcrd,
            "PHY_PHY_MS_CORE_TAP_WRST_N",
        )
        .extra_in_far_from_far("IO_HBM_IO_MS_CORE_TAP_WSI", bcrd, "PHY_PHY_MS_CORE_TAP_WSI")
        .extra_out_far_claim_bus("IO_HBM_IO_MS_CORE_TAP_WSO", 8)
        .extra_in_far_from_far_bus(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_REG_CTL",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL",
            57,
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_CLK_SEL",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_57_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_IBUF_DIS_OR_HS_RX_DIS",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_58_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_IBUF_DISABLE",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_59_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_LP_RX_DIS_OR_TERM_EN",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_60_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_ODISABLE",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_61_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_T_0_",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_62_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_PHY2CCIO_T_1_",
            bcrd,
            "PHY_PHY_MS_CORE_PHY2CCIO_REG_CTL_63_",
        );

    for pin in naming.pins.keys() {
        if pin.starts_with("PHY_PHY_MS_CORE_PHY2DLL")
            || pin.starts_with("PHY_PHY_MS_CORE_PHY2HBMEXT")
            || pin.starts_with("PHY_PHY_MS_CORE_PHY2IOB_DFT")
            || (pin.starts_with("PHY_PHY_MS_CORE_PHY2MS")
                && pin != "PHY_PHY_MS_CORE_PHY2MS_LPBK_EN"
                && pin != "PHY_PHY_MS_CORE_PHY2MS_SPARE_7_")
        {
            let rest = pin.strip_prefix("PHY_PHY_MS_CORE_").unwrap();
            bel = bel.extra_in_far_from_far(format!("IO_HBM_IO_MS_CORE_{rest}"), bcrd, pin);
        }
    }

    // HPLL connection
    bel = bel
        .extra_out_far("IO_HBM_IO_MS_CORE_PLL_REF_CLK")
        .extra_in_far("IO_HBM_IO_MS_CORE_PLL2PHY_CLKOUTPHY");
    for (pin, opin) in [
        ("IO_HBM_IO_MS_CORE_PLL_REF_CLK", "CLKIN"),
        ("IO_HBM_IO_MS_CORE_PLL2PHY_CLKOUTPHY", "CLKOUTPHY"),
    ] {
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(hpll, opin)]);
    }

    // monitor
    bel = bel
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_CCIO2DFTIO_REF_CLK")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_HBM2DFTIO_HS_TX_CLK")
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_0_0_",
            bcrd,
            "IO_HBM_IO_MS_CORE_HBM2DFTIO_HS_TX_CLK",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_0_1_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_RDQS_MONOUT_0_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_0_2_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_WDQS_MONOUT_0_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_0_3_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_DQ_MONOUT_0_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_1_0_",
            bcrd,
            "IO_HBM_IO_MS_CORE_CCIO2DFTIO_REF_CLK",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_1_1_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_RDQST_MONOUT_0_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_1_2_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_WDQST_MONOUT_0_",
        )
        .extra_in_far_from_far(
            "IO_HBM_IO_MS_CORE_DIG_IN_1_3_",
            chnl[4],
            "IO_HBM_IO_CHNL_CORE_IOB2C4_DW_DQ_MONOUT1_0_",
        );

    // crap
    bel = bel
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_POR_B_VCCAUX")
        .extra_out_far_claim("IO_HBM_IO_MS_CORE_POR_B_VCCAUX_VCCINT_IO");

    bel.commit();
}

pub fn verify_hpll(vrf: &mut Verifier, bcrd: BelCoord) {
    let naming = vrf.ngrid.get_bel_naming(bcrd);
    let mut bel = vrf.verify_bel(bcrd).kind("HPLL");
    for (pname, pin) in &naming.pins {
        if let Some((_, pdir)) = pin.orig_pin {
            if pdir == PinDir::Input {
                bel = bel.extra_in_far_claim(pname);
            } else {
                bel = bel.extra_out_far_claim(pname);
            }
        }
    }
    bel.commit();
}
