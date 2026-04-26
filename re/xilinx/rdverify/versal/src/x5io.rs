use prjcombine_entity::EntityId;
use prjcombine_interconnect::{db::CellSlotId, grid::BelCoord};
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    chip::{HRegId, HsrSKind, PsKind},
    defs::{bcls, bslots, tcls, tslots},
    expanded::ExpandedDevice,
};

pub fn verify_x5io_octad(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::X5IO_NC;

    let rpos = bcrd.col.to_idx() % 8;
    let (idx, cell_bank) = if rpos < 4 {
        (rpos, bcrd.cell.delta((3 - rpos) as i32, 0))
    } else {
        (7 - rpos, bcrd.cell.delta(-(rpos as i32 - 4), 0))
    };

    let scan = cell_bank.bel(bslots::X5IO_SCAN);
    let phio_cmu = cell_bank.bel(bslots::X5PHIO_CMU);
    let octad: [_; 4] = core::array::from_fn(|i| {
        edev.tile_cell(cell_bank.tile(tslots::HSR_XPLL), CellSlotId::from_idx(i))
            .bel(bslots::X5IO_OCTAD)
    });

    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("CMPHY_OCTAD")
        .skip_in(bcls::X5IO_OCTAD::PLL_CLK0)
        .skip_in(bcls::X5IO_OCTAD::PLL_CLK90)
        .skip_in(bcls::X5IO_OCTAD::PDQS_IN)
        .skip_in(bcls::X5IO_OCTAD::NDQS_IN)
        .skip_in(bcls::X5IO_OCTAD::DQS_IN_R)
        .skip_out(bcls::X5IO_OCTAD::CCIO[0])
        .skip_out(bcls::X5IO_OCTAD::CCIO[1])
        .skip_out(bcls::X5IO_OCTAD::RX0_RDQS_P_OUT)
        .skip_out(bcls::X5IO_OCTAD::RX1_RDQS_P_OUT)
        .skip_out(bcls::X5IO_OCTAD::DQS_DLY_OUT_R)
        .skip_out(bcls::X5IO_OCTAD::PDQS_O)
        .skip_out(bcls::X5IO_OCTAD::NDQS_O)
        .skip_out(bcls::X5IO_OCTAD::PDQS_DLY_O)
        .skip_out(bcls::X5IO_OCTAD::NDQS_DLY_O)
        .skip_out(bcls::X5IO_OCTAD::DIV64_CLK)
        .extra_out_far_claim_bus("VT_DLY_OUT", 21)
        .extra_in_far_from_far_bus("VT_DLY_IN0", octad[0], "VT_DLY_OUT", 21)
        .extra_in_far_from_far_bus("VT_DLY_IN1", octad[1], "VT_DLY_OUT", 21)
        .extra_in_far_from_far_bus("VT_DLY_IN2", octad[2], "VT_DLY_OUT", 21)
        .extra_in_far_from_far_bus("VT_DLY_IN3", octad[3], "VT_DLY_OUT", 21)
        .extra_out_far_claim_bus("VT_DRFT_OUT", 21)
        .extra_in_far_from_far_bus("VT_DRFT_IN0", octad[0], "VT_DRFT_OUT", 21)
        .extra_in_far_from_far_bus("VT_DRFT_IN1", octad[1], "VT_DRFT_OUT", 21)
        .extra_in_far_from_far_bus("VT_DRFT_IN2", octad[2], "VT_DRFT_OUT", 21)
        .extra_in_far_from_far_bus("VT_DRFT_IN3", octad[3], "VT_DRFT_OUT", 21)
        .extra_out_far_claim("SEQ_DONE")
        .extra_in_far_from_far("SEQ_DONE_I_0_", octad[0], "SEQ_DONE")
        .extra_in_far_from_far("SEQ_DONE_I_1_", octad[1], "SEQ_DONE")
        .extra_in_far_from_far("SEQ_DONE_I_2_", octad[2], "SEQ_DONE")
        .extra_in_far_from_far("SEQ_DONE_I_3_", octad[3], "SEQ_DONE")
        // to SCAN
        .extra_out_far_claim("MUX2APB_PCLK")
        .extra_out_far_claim_bus("MUX2RDP_CLK", 2)
        .extra_out_far_claim("MUX2WRP_CLK")
        .extra_in_far_from_far("SCAN_EN", scan, "_SCAN_EN")
        .extra_in_far_from_far("SCAN_MODE_RST_N", scan, "_SCAN_MODE_RST_N")
        .extra_in_far_from_far("SCAN_RST_BYP", scan, "_SCAN_RST_BYP")
        // dummy
        .extra_in_far("SCAN_MODE")
        // to XCV
        .extra_out_far_claim("PHY2XCV_2TO1_CLK")
        .extra_out_far_claim_bus("PHY2XCV_DYNAMIC_DCI_TS", 8)
        .extra_out_far_claim_bus("PHY2XCV_IBUF_DIS_OR_HS_RX_DIS", 8)
        .extra_out_far_claim_bus("PHY2XCV_RD_CTL0", 8)
        .extra_out_far_claim_bus("PHY2XCV_RD_CTL1", 8)
        .extra_out_far_claim_bus("PHY2XCV_RD_CTL2", 8)
        .extra_out_far_claim_bus("PHY2XCV_RD_CTL3", 8)
        .extra_out_far_claim_bus("PHY2XCV_WR_CTL0", 10)
        .extra_out_far_claim_bus("PHY2XCV_WR_CTL1", 10)
        .extra_out_far_claim_bus("PHY2XCV_WR_CTL2", 10)
        .extra_out_far_claim_bus("PHY2XCV_WR_CTL3", 10)
        .extra_out_far_claim_bus("PHY2XCV_WR_DQ0", 32)
        .extra_out_far_claim_bus("PHY2XCV_WR_DQ1", 32)
        .extra_out_far_claim_bus("PHY2XCV_WR_DQ2", 32)
        .extra_out_far_claim_bus("PHY2XCV_WR_DQ3", 32)
        .extra_in_far_from_far("XCV2PHY_LP_I_0_0_", bcrd, "IOB0_XCV2PHYPLL_RX_OUT_M")
        .extra_in_far_from_far("XCV2PHY_LP_I_0_1_", bcrd, "IOB1_XCV2PHYPLL_RX_OUT_M")
        .extra_in_far_from_far("XCV2PHY_LP_I_0_2_", bcrd, "IOB2_XCV2PHYPLL_RX_OUT_M")
        .extra_in_far_from_far("XCV2PHY_LP_I_0_3_", bcrd, "CCIO_0_")
        .extra_in_far_from_far("XCV2PHY_LP_I_1_0_", bcrd, "IOB0_XCV2PHYPLL_RX_OUT_S")
        .extra_in_far_from_far("XCV2PHY_LP_I_1_1_", bcrd, "IOB1_XCV2PHYPLL_RX_OUT_S")
        .extra_in_far_from_far("XCV2PHY_LP_I_1_2_", bcrd, "IOB2_XCV2PHYPLL_RX_OUT_S")
        .extra_in_far_from_far("XCV2PHY_LP_I_1_3_", bcrd, "CCIO_1_")
        .extra_in_far_from_far_bus("XCV2PHY_RD_CLK0", bcrd, "IOB0_XCV2PHY_RD_CLK", 2)
        .extra_in_far_from_far_bus("XCV2PHY_RD_CLK1", bcrd, "IOB1_XCV2PHY_RD_CLK", 2)
        .extra_in_far_from_far_bus("XCV2PHY_RD_CLK2", bcrd, "IOB2_XCV2PHY_RD_CLK", 2)
        .extra_in_far_from_far_bus("XCV2PHY_RD_CLK3", bcrd, "IOB3_XCV2PHY_RD_CLK", 2)
        .extra_in_far_from_far("XCV2PHY_WR_CLK0", bcrd, "IOB0_XCV2PHY_WR_CLK")
        .extra_in_far_from_far("XCV2PHY_WR_CLK1", bcrd, "IOB1_XCV2PHY_WR_CLK")
        .extra_in_far_from_far("XCV2PHY_WR_CLK2", bcrd, "IOB2_XCV2PHY_WR_CLK")
        .extra_in_far_from_far("XCV2PHY_WR_CLK3", bcrd, "IOB3_XCV2PHY_WR_CLK")
        .extra_in_far_from_far_bus("XCV2PHY_RD_DQ0", bcrd, "IOB0_XCV2PHY_RD_DQ", 16)
        .extra_in_far_from_far_bus("XCV2PHY_RD_DQ1", bcrd, "IOB1_XCV2PHY_RD_DQ", 16)
        .extra_in_far_from_far_bus("XCV2PHY_RD_DQ2", bcrd, "IOB2_XCV2PHY_RD_DQ", 16)
        .extra_in_far_from_far_bus("XCV2PHY_RD_DQ3", bcrd, "IOB3_XCV2PHY_RD_DQ", 16)
        .extra_out_far_claim("RIU2XCV_CK0")
        .extra_out_far_claim("RIU2XCV_CK1")
        .extra_out_far_claim("RIU2XCV_CK2")
        .extra_out_far_claim("RIU2XCV_CK3")
        .extra_out_far_claim("RIU2XCV_RST0")
        .extra_out_far_claim("RIU2XCV_RST1")
        .extra_out_far_claim("RIU2XCV_RST2")
        .extra_out_far_claim("RIU2XCV_RST3")
        .extra_out_far_claim("RIU2XCV_CA0")
        .extra_out_far_claim("RIU2XCV_CA1")
        .extra_out_far_claim("RIU2XCV_CA2")
        .extra_out_far_claim("RIU2XCV_CA3")
        .extra_out_far_claim("RIU2XCV_WR0")
        .extra_out_far_claim("RIU2XCV_WR1")
        .extra_out_far_claim("RIU2XCV_WR2")
        .extra_out_far_claim("RIU2XCV_WR3")
        .extra_in_far_from_far("XCV2RIU_CK0", bcrd, "IOB0_XCV2RIU_CK")
        .extra_in_far_from_far("XCV2RIU_CK1", bcrd, "IOB1_XCV2RIU_CK")
        .extra_in_far_from_far("XCV2RIU_CK2", bcrd, "IOB2_XCV2RIU_CK")
        .extra_in_far_from_far("XCV2RIU_CK3", bcrd, "IOB3_XCV2RIU_CK")
        .extra_in_far_from_far_bus("XCV2RIU_RD0", bcrd, "IOB0_XCV2RIU_RD", 4)
        .extra_in_far_from_far_bus("XCV2RIU_RD1", bcrd, "IOB1_XCV2RIU_RD", 4)
        .extra_in_far_from_far_bus("XCV2RIU_RD2", bcrd, "IOB2_XCV2RIU_RD", 4)
        .extra_in_far_from_far_bus("XCV2RIU_RD3", bcrd, "IOB3_XCV2RIU_RD", 4)
        // dealt with by DDRMC
        .extra_in_far("DMC2APB_PCLK")
        .extra_in_far("DMC2APB_PRESETN")
        .extra_in_far("DMC2APB_PENABLE")
        .extra_in_far("DMC2APB_PWRITE")
        .extra_in_far("DMC2APB_PSEL")
        .extra_in_far_bus("DMC2APB_PADDR", 10)
        .extra_in_far_bus("DMC2APB_PWDATA", 16)
        .extra_out_far_claim("APB2DMC_PREADY")
        .extra_out_far_claim_bus("DMC2APB_PRDATA", 16)
        .extra_out_far_claim("APB2DMC_PSLVERR")
        .extra_in_far("DMC_CLK")
        .extra_in_far("DMC_RST")
        .extra_out_far_claim("DMC_PD")
        .extra_in_far("DMC_FIFO_RDCLK")
        .extra_in_far("DMC_FIFO_RDEN")
        .extra_out_far_claim("DMC_FIFO_EMPTY")
        .extra_in_far_bus("DMC_PHY_WREN", 4)
        .extra_in_far_bus("DMC_PHY_WRCS0", 4)
        .extra_in_far_bus("DMC_PHY_WRCS1", 4)
        .extra_in_far_bus("DMC_PHY_RDEN", 4)
        .extra_in_far_bus("DMC_PHY_RDCS0", 4)
        .extra_in_far_bus("DMC_PHY_RDCS1", 4)
        .extra_in_far_bus("DMC_D0", 8)
        .extra_in_far_bus("DMC_D1", 8)
        .extra_in_far_bus("DMC_D2", 8)
        .extra_in_far_bus("DMC_D3", 8)
        .extra_in_far_bus("DMC_D4", 8)
        .extra_in_far_bus("DMC_D5", 8)
        .extra_in_far_bus("DMC_D6", 8)
        .extra_in_far_bus("DMC_D7", 8)
        .extra_out_far_claim_bus("DMC_Q0", 8)
        .extra_out_far_claim_bus("DMC_Q1", 8)
        .extra_out_far_claim_bus("DMC_Q2", 8)
        .extra_out_far_claim_bus("DMC_Q3", 8)
        .extra_out_far_claim_bus("DMC_Q4", 8)
        .extra_out_far_claim_bus("DMC_Q5", 8)
        .extra_out_far_claim_bus("DMC_Q6", 8)
        .extra_out_far_claim_bus("DMC_Q7", 8)
        .extra_in_far_bus("DMC_IBUF_HS_RX_DIS", 8);

    if is_nc {
        bel = bel
            .extra_in_far("PCLK")
            .extra_in_far("PRESETN")
            .extra_in_far("PENABLE")
            .extra_in_far("PWRITE")
            .extra_in_far("PSEL")
            .extra_in_far_bus("PADDR", 10)
            .extra_in_far_bus("PWDATA", 16)
            .extra_out_far_claim("PREADY")
            .extra_out_far_claim_bus("PRDATA", 16)
            .extra_out_far_claim("PSLVERR")
            .extra_in_far("PHY_CLK")
            .extra_in_far("PHY_2TO1_CLK")
            .extra_in_far("RST")
            .extra_out_far_claim("PHY_PD")
            .extra_out_far_claim("PHY_RDY")
            .extra_out_far_claim("DLY_RDY")
            .extra_in_far_bus("PHY_RDEN", 8)
            .extra_in_far_bus("PHY_RDCS0", 8)
            .extra_in_far_bus("PHY_RDCS1", 8)
            .extra_in_far_bus("PHY_WREN", 8)
            .extra_in_far_bus("PHY_WRCS0", 8)
            .extra_in_far_bus("PHY_WRCS1", 8)
            .extra_in_far_bus("RXFIFO_RDCLK", 2)
            .extra_in_far_bus("RXFIFO_RDEN", 2)
            .extra_out_far_claim_bus("FIFO_EMPTY", 2)
            .extra_in_far_bus("RX_RST", 8)
            .extra_in_far_bus("TX_RST", 8)
            .extra_in_far_bus("T", 8)
            .extra_in_far_bus("D0", 16)
            .extra_in_far_bus("D1", 16)
            .extra_in_far_bus("D2", 16)
            .extra_in_far_bus("D3", 16)
            .extra_in_far_bus("D4", 16)
            .extra_in_far_bus("D5", 16)
            .extra_in_far_bus("D6", 16)
            .extra_in_far_bus("D7", 16)
            .extra_out_far_claim_bus("Q0", 16)
            .extra_out_far_claim_bus("Q1", 16)
            .extra_out_far_claim_bus("Q2", 16)
            .extra_out_far_claim_bus("Q3", 16)
            .extra_out_far_claim_bus("Q4", 16)
            .extra_out_far_claim_bus("Q5", 16)
            .extra_out_far_claim_bus("Q6", 16)
            .extra_out_far_claim_bus("Q7", 16)
            .extra_in_far_bus("IBUFDISABLEIN", 8)
            .extra_in_far_bus("DYN_DCI_IN", 8)
            .extra_in_far_bus("CLB2PHY_KEEPER_EN", 8)
            .extra_in_far_bus("CLB2PHY_LP_RX_DIS_OR_TERM_EN", 4)
            .extra_in_far_bus("CLB2PHY_LP_TX_O_0", 4)
            .extra_in_far_bus("CLB2PHY_LP_TX_O_1", 4)
            .extra_in_far_bus("CLB2PHY_LP_TX_T", 4)
            .extra_out_far_claim_bus("PHY2CLB_LP_I_0", 4)
            .extra_out_far_claim_bus("PHY2CLB_LP_I_1", 4)
            .extra_out_far_claim("PHY2CLB_STATUS");
    }

    bel.commit();

    for i in 0..4 {
        let mut bel = vrf
            .verify_bel(bcrd)
            .sub(1 + i)
            .kind("X5PHIO_XCVR")
            .skip_auto()
            .extra_in_far_from_far(format!("IOB{i}_PHY2XCV_2TO1_CLK"), bcrd, "PHY2XCV_2TO1_CLK")
            .extra_in_far_from_far(
                format!("IOB{i}_PHY2XCV_DYNAMIC_DCI_TS_0_"),
                bcrd,
                format!("PHY2XCV_DYNAMIC_DCI_TS_{ii}_", ii = i * 2),
            )
            .extra_in_far_from_far(
                format!("IOB{i}_PHY2XCV_DYNAMIC_DCI_TS_1_"),
                bcrd,
                format!("PHY2XCV_DYNAMIC_DCI_TS_{ii}_", ii = i * 2 + 1),
            )
            .extra_in_far_from_far(
                format!("IOB{i}_PHY2XCV_IBUF_DIS_OR_HS_RX_DIS_0_"),
                bcrd,
                format!("PHY2XCV_IBUF_DIS_OR_HS_RX_DIS_{ii}_", ii = i * 2),
            )
            .extra_in_far_from_far(
                format!("IOB{i}_PHY2XCV_IBUF_DIS_OR_HS_RX_DIS_1_"),
                bcrd,
                format!("PHY2XCV_IBUF_DIS_OR_HS_RX_DIS_{ii}_", ii = i * 2 + 1),
            )
            .extra_in_far_from_far_bus(
                format!("IOB{i}_PHY2XCV_RD_CTL"),
                bcrd,
                format!("PHY2XCV_RD_CTL{i}"),
                8,
            )
            .extra_in_far_from_far_bus(
                format!("IOB{i}_PHY2XCV_WR_CTL"),
                bcrd,
                format!("PHY2XCV_WR_CTL{i}"),
                10,
            )
            .extra_in_far_from_far_bus(
                format!("IOB{i}_PHY2XCV_WR_DQ"),
                bcrd,
                format!("PHY2XCV_WR_DQ{i}"),
                32,
            )
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2PHY_RD_CLK"), 2)
            .extra_out_far_claim(format!("IOB{i}_XCV2PHY_WR_CLK"))
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2PHY_RD_DQ"), 16)
            .extra_in_far_from_far(format!("IOB{i}_RIU2XCV_CK"), bcrd, format!("RIU2XCV_CK{i}"))
            .extra_in_far_from_far(
                format!("IOB{i}_RIU2XCV_RST"),
                bcrd,
                format!("RIU2XCV_RST{i}"),
            )
            .extra_in_far_from_far(format!("IOB{i}_RIU2XCV_CA"), bcrd, format!("RIU2XCV_CA{i}"))
            .extra_in_far_from_far(format!("IOB{i}_RIU2XCV_WR"), bcrd, format!("RIU2XCV_WR{i}"))
            .extra_out_far_claim(format!("IOB{i}_XCV2RIU_CK"))
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2RIU_RD"), 4)
            .extra_out_far_claim_bus(format!("IOB{i}_DCI_BUS_BUF_OUT"), 17)
            .extra_out_far_claim(format!("IOB{i}_XCV2XCV_VREF_H1P_O"))
            .extra_out_far_claim(format!("IOB{i}_XCV2XCV_VREF_H1M_O"))
            .extra_out_far_claim(format!("IOB{i}_DIFF_P"))
            .extra_out_far_claim(format!("IOB{i}_DIFF_N"))
            .extra_in_far_from_far(
                format!("IOB{i}_XCV2XCV_PAD_P"),
                bcrd,
                if matches!(i, 1 | 2) {
                    format!("IOB{ii}_DIFF_P", ii = i ^ 1)
                } else {
                    format!("IOB{ii}_DIFF_N", ii = i ^ 1)
                },
            )
            .extra_in_far_from_far(
                format!("IOB{i}_XCV2XCV_PAD_N"),
                bcrd,
                if matches!(i, 1 | 2) {
                    format!("IOB{ii}_DIFF_N", ii = i ^ 1)
                } else {
                    format!("IOB{ii}_DIFF_P", ii = i ^ 1)
                },
            )
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2XCV_RX2TX_DOUT"), 2)
            .extra_out_far_claim(format!("IOB{i}_XCV2XCV_RX0_RDQS_OUT"))
            .extra_out_far_claim(format!("IOB{i}_XCV2XCV_RX1_RDQS_OUT"))
            .extra_out_far_claim(format!("IOB{i}_XCV2CLK_NDQS_DLY_REPLICA"))
            .extra_out_far_claim(format!("IOB{i}_DQS_DLY_OUT_M"))
            .extra_out_far_claim(format!("IOB{i}_XCV2XCV_CPHY_CLK_OUT"))
            .extra_in_far_from_far(
                format!("IOB{i}_XCV2XCV_CPHY_CLK_IN"),
                bcrd,
                format!("IOB{ii}_XCV2XCV_CPHY_CLK_OUT", ii = [1, 1, 2, 2][i]),
            )
            .extra_in_far_from_far(
                format!("IOB{i}_XCV2XCV_VREF_H1P_I"),
                bcrd,
                "IOB2_XCV2XCV_VREF_H1P_O",
            )
            .extra_in_far_from_far(
                format!("IOB{i}_XCV2XCV_VREF_H1M_I"),
                bcrd,
                "IOB2_XCV2XCV_VREF_H1M_O",
            )
            .extra_out_far_claim(format!("IOB{i}_XCV2CLK_RIU_CLK0_DCD_ADJ_SEL"))
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2CLK_RIU_CLK0_DCD_ADJ"), 4)
            .extra_out_far_claim(format!("IOB{i}_XCV2CLK_RIU_CLK90_DCD_ADJ_SEL"))
            .extra_out_far_claim_bus(format!("IOB{i}_XCV2CLK_RIU_CLK90_DCD_ADJ"), 4)
            // dummy
            .extra_in_far_claim_bus(format!("IOB{i}_XCVR_X2_ID"), 2)
            .extra_in_far_claim(format!("IOB{i}_CFGXCV_SCAN_MODE"))
            .extra_in_far_from_far(
                format!("IOB{i}_CFGXCV_SCAN_MODE_RST"),
                scan,
                "_SCAN_MODE_RST_N",
            )
            .extra_in_far(format!("IOB{i}_CFGXCV_SCAN_RST_BYP"))
            .extra_in_far(format!("IOB{i}_CFGXCV_SCAN_EN"))
            .extra_in_far_bus(format!("IOB{i}_CFGXCV_SCAN_DATA_OUT_INT"), 5);
        if i == 1 {
            bel = bel
                .extra_in("PLL_CLK0")
                .extra_in("PLL_CLK90")
                .extra_in("DQS_IN_R")
                .extra_out("DQS_DLY_OUT_R")
                .extra_in("PDQS_IN")
                .extra_in("NDQS_IN")
                .extra_out("PDQS_O")
                .extra_out("NDQS_O")
                .extra_out("PDQS_DLY_O")
                .extra_out("NDQS_DLY_O")
                .extra_out("RX0_RDQS_P_OUT")
                .extra_out("RX1_RDQS_P_OUT");
        } else {
            bel = bel
                .extra_out_far_claim(format!("IOB{i}_DQS_DLY_OUT_R"))
                .extra_out_far_claim(format!("IOB{i}_XCV2XCV_PDQS_O"))
                .extra_out_far_claim(format!("IOB{i}_XCV2XCV_NDQS_O"))
                .extra_out_far_claim(format!("IOB{i}_XCV2XCV_PDQS_DLY_O"))
                .extra_out_far_claim(format!("IOB{i}_XCV2XCV_NDQS_DLY_O"))
                .extra_out_far_claim(format!("IOB{i}_XCV2CGL_RX0_RDQS_P_OUT_CLK"))
                .extra_out_far_claim(format!("IOB{i}_XCV2CGL_RX1_RDQS_P_OUT_CLK"))
                .extra_in_far_from_far(format!("IOB{i}_PLL_CLK0"), bcrd, "PLL_CLK0")
                .extra_in_far_from_far(format!("IOB{i}_PLL_CLK90"), bcrd, "PLL_CLK90")
                .extra_in_far_from_far(format!("IOB{i}_XCV2XCV_PDQS_IN"), bcrd, "PDQS_IN")
                .extra_in_far_from_far(format!("IOB{i}_XCV2XCV_NDQS_IN"), bcrd, "NDQS_IN")
                // dummy
                .extra_in_far_claim(format!("IOB{i}_DQS_IN_R"));
        }
        if i == 3 {
            bel = bel
                .extra_out("CCIO_0_")
                .extra_out("CCIO_1_")
                .extra_out("DIV64_CLK");
        } else {
            bel = bel
                .extra_out_far_claim(format!("IOB{i}_XCV2PHYPLL_RX_OUT_M"))
                .extra_out_far_claim(format!("IOB{i}_XCV2PHYPLL_RX_OUT_S"))
                .extra_out_far_claim(format!("IOB{i}_XCV2CLK_DIV64_CLK"));
        }

        if idx == 0 {
            bel = bel.extra_in_far_from_far_bus(
                format!("IOB{i}_CMU2XCV_DCI"),
                phio_cmu,
                "IO_CMU2XCV_DCI",
                17,
            );
        } else {
            bel = bel.extra_in_far_from_far_bus(
                format!("IOB{i}_CMU2XCV_DCI"),
                octad[idx - 1],
                format!("IOB{i}_DCI_BUS_BUF_OUT"),
                17,
            );
        }

        if matches!(i, 1 | 2) {
            bel = bel
                .extra_in_far_from_far(
                    format!("IOB{i}_XCV2XCV_RX0_RDQS_IN"),
                    bcrd,
                    format!("IOB{ii}_XCV2XCV_RX0_RDQS_OUT", ii = i ^ 1),
                )
                .extra_in_far_from_far(
                    format!("IOB{i}_XCV2XCV_RX1_RDQS_IN"),
                    bcrd,
                    format!("IOB{ii}_XCV2XCV_RX1_RDQS_OUT", ii = i ^ 1),
                )
                .extra_in_far_from_far(
                    format!("IOB{i}_XCV2XCV_RX2_RDQS_IN"),
                    bcrd,
                    format!("IOB{i}_XCV2XCV_RX0_RDQS_OUT"),
                )
                .extra_in_far_from_far(
                    format!("IOB{i}_XCV2CGL_RX1_NDQS_DLY_IN"),
                    bcrd,
                    format!("IOB{ii}_XCV2CLK_NDQS_DLY_REPLICA", ii = i ^ 1),
                )
                .extra_in_far_from_far(
                    format!("IOB{i}_XCV2CGL_RX2_NDQS_DLY_IN"),
                    bcrd,
                    format!("IOB{i}_DQS_DLY_OUT_M"),
                );
        } else {
            bel = bel
                .extra_in_far_claim(format!("IOB{i}_XCV2XCV_RX0_RDQS_IN"))
                .extra_in_far_claim(format!("IOB{i}_XCV2XCV_RX1_RDQS_IN"))
                .extra_in_far_claim(format!("IOB{i}_XCV2XCV_RX2_RDQS_IN"))
                .extra_in_far_claim(format!("IOB{i}_XCV2CGL_RX1_NDQS_DLY_IN"))
                .extra_in_far_claim(format!("IOB{i}_XCV2CGL_RX2_NDQS_DLY_IN"));
        }

        if idx % 2 == 0 {
            if i == 0 {
                bel = bel
                    .extra_in_far_from_far(
                        format!("IOB{i}_XCV2XCV_RX2TX_DIN_0_"),
                        octad[idx ^ 1],
                        format!("IOB{i}_XCV2XCV_RX2TX_DOUT_1_"),
                    )
                    .extra_in_far_from_far(
                        format!("IOB{i}_XCV2XCV_RX2TX_DIN_1_"),
                        octad[idx ^ 1],
                        format!("IOB{i}_XCV2XCV_RX2TX_DOUT_0_"),
                    );
            } else {
                bel = bel.extra_in_far_from_far_bus(
                    format!("IOB{i}_XCV2XCV_RX2TX_DIN"),
                    bcrd,
                    format!("IOB{ii}_XCV2XCV_RX2TX_DOUT", ii = i - 1),
                    2,
                );
            }
        } else {
            if i == 3 {
                bel = bel
                    .extra_in_far_from_far(
                        format!("IOB{i}_XCV2XCV_RX2TX_DIN_0_"),
                        octad[idx ^ 1],
                        format!("IOB{i}_XCV2XCV_RX2TX_DOUT_1_"),
                    )
                    .extra_in_far_from_far(
                        format!("IOB{i}_XCV2XCV_RX2TX_DIN_1_"),
                        octad[idx ^ 1],
                        format!("IOB{i}_XCV2XCV_RX2TX_DOUT_0_"),
                    );
            } else {
                bel = bel.extra_in_far_from_far_bus(
                    format!("IOB{i}_XCV2XCV_RX2TX_DIN"),
                    bcrd,
                    format!("IOB{ii}_XCV2XCV_RX2TX_DOUT", ii = i + 1),
                    2,
                );
            }
        }

        bel.commit();
    }
}

pub fn verify_x5pll(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::X5PLL_NC;
    let scan = bcrd.bel(bslots::X5IO_SCAN);
    let phio_cmu = bcrd.bel(bslots::X5PHIO_CMU);
    let mut bel = vrf
        .verify_bel(bcrd)
        .skip_auto()
        .extra_in("CLKIN")
        .extra_in("CLKIN1_DESKEW")
        .extra_in("CLKIN2_DESKEW")
        .extra_in("CLKFB1_DESKEW")
        .extra_in("CLKFB2_DESKEW")
        .extra_out("CLKOUT0")
        .extra_out("CLKOUT1")
        .extra_out("CLKOUT2")
        .extra_out("CLKOUT3")
        .extra_out("TMUXOUT")
        .extra_out("CLKOUTPHY_0")
        .extra_out("CLKOUTPHY_90")
        .extra_out_far_claim("PLL_CLKOUTPHY_CASC_OUT_0")
        .extra_out_far_claim("PLL_CLKOUTPHY_CASC_OUT_90")
        .extra_in_far_from_far("PLL_RIU_CLK", bcrd, "S2P_RIU_CLK")
        .extra_in_far_from_far("PLL_RIU_NIBBLE_SEL", bcrd, "S2P_RIU_NIBBLE_SEL")
        .extra_in_far_from_far("PLL_RIU_WR_EN", bcrd, "S2P_RIU_WREN")
        .extra_in_far_from_far_bus("PLL_RIU_ADDR", bcrd, "S2P_RIU_ADDR", 8)
        .extra_in_far_from_far_bus("PLL_RIU_WR_DATA", bcrd, "S2P_RIU_WR_DATA", 16)
        .extra_out_far_claim("PLL_RIU_VALID")
        .extra_out_far_claim_bus("PLL_RIU_RD_DATA", 16)
        .extra_in_far_from_far("PLL_SCANCLK", bcrd, "INTF_APB2PLL_CK")
        .extra_in_far_from_far("PLL_SCANENB", scan, "_SCAN_EN_STUCKAT")
        .extra_in_far_from_far("PLL_SCANMODEB", scan, "_SCAN_MODE_STUCKAT")
        .extra_in_far_from_far("PLL_PSCLK", bcrd, "INTF_PSCLK")
        .extra_in_far_from_far("PLL_PSEN", bcrd, "INTF_PSEN")
        .extra_in_far_from_far("PLL_PSINCDEC", bcrd, "INTF_PSINCDEC")
        .extra_out_far_claim("PLL_PSDONE")
        // dummy conns
        .extra_in_far_claim("PLL_DCLK")
        .extra_in_far_claim("PLL_DEN")
        .extra_in_far_claim("PLL_DWE")
        .extra_in_far_claim_bus("PLL_DADDR", 7)
        .extra_in_far_claim_bus("PLL_DI", 16)
        .extra_out_far_claim("PLL_DRDY")
        .extra_out_far_claim_bus("PLL_DO", 16);

    if is_nc {
        bel = bel
            .extra_out_far_claim("LOCKED")
            .extra_out_far_claim("LOCKED1_DESKEW")
            .extra_out_far_claim("LOCKED2_DESKEW")
            .extra_out_far_claim("LOCKED_FB")
            .extra_in_far_claim("CLKOUTPHYEN")
            .extra_in_far_claim("RST")
            .extra_in_far_claim("PWRDWN")
            .extra_out_far_claim("TESTOUT")
            .extra_in_far_claim_bus("TESTIN", 8);
    } else {
        bel = bel
            .extra_out("LOCKED")
            .extra_out("LOCKED1_DESKEW")
            .extra_out("LOCKED2_DESKEW")
            .extra_out("LOCKED_FB")
            .extra_in("CLKOUTPHYEN")
            .extra_in("RST")
            .extra_in("PWRDWN")
            .extra_out("TESTOUT")
            .extra_in_bus("TESTIN", 8);
    }

    // CLKOUTPHY cascade
    let reg = HRegId::from_idx(bcrd.col.to_idx() / 8);
    let chip = edev.chips[bcrd.die];
    let HsrSKind::X5io(ref hsr) = chip.hsr_s else {
        unreachable!()
    };
    let src_dx = if let Some(io) = hsr.regions[reg].io[0]
        && io.ddrmc.is_some()
    {
        Some(-8)
    } else if let Some(io) = hsr.regions[reg].io[1]
        && io.ddrmc.is_some()
    {
        Some(8)
    } else if reg != hsr.regions.first_id().unwrap()
        && let Some(io) = hsr.regions[reg - 1].io[1]
        && io.ddrmc.is_some()
    {
        Some(0)
    } else if reg != hsr.regions.last_id().unwrap()
        && let Some(io) = hsr.regions[reg + 1].io[0]
        && io.ddrmc.is_some()
    {
        Some(0)
    } else {
        None
    };
    if let Some(dx) = src_dx {
        let src = bcrd.delta(dx, 0).bel(bslots::X5PLL);
        bel = bel
            .extra_in_far_from_far("PLL_CLKOUTPHY_CASC_IN_0", src, "PLL_CLKOUTPHY_CASC_OUT_0")
            .extra_in_far_from_far("PLL_CLKOUTPHY_CASC_IN_90", src, "PLL_CLKOUTPHY_CASC_OUT_90");
    } else {
        bel = bel
            .extra_in_far_claim("PLL_CLKOUTPHY_CASC_IN_0")
            .extra_in_far_claim("PLL_CLKOUTPHY_CASC_IN_90");
    }

    bel.commit();

    vrf.verify_bel(bcrd)
        .sub(1)
        .kind("X5PLL_S2P")
        .skip_auto()
        .extra_in_far_from_far("S2P_RIU_RST", bcrd, "INTF_APB2PLL_RST")
        .extra_in_far_from_far("S2P_RIU2XCV_CK", bcrd, "INTF_APB2PLL_CK")
        .extra_in_far_from_far("S2P_RIU2XCV_CA", bcrd, "INTF_APB2PLL_CA")
        .extra_in_far_from_far("S2P_RIU2XCV_WR", bcrd, "INTF_APB2PLL_WR")
        .extra_out_far_claim("S2P_XCV2RIU_CK")
        .extra_out_far_claim_bus("S2P_XCV2RIU_RD", 4)
        .extra_out_far_claim("S2P_RIU_CLK")
        .extra_out_far_claim("S2P_RIU_NIBBLE_SEL")
        .extra_out_far_claim("S2P_RIU_WREN")
        .extra_out_far_claim_bus("S2P_RIU_ADDR", 8)
        .extra_out_far_claim_bus("S2P_RIU_WR_DATA", 16)
        .extra_in_far_from_far_bus("S2P_RIU_RDATA", bcrd, "PLL_RIU_RD_DATA", 16)
        .commit();

    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(2)
        .kind("X5PLL_INTF")
        .skip_auto()
        .extra_out_far_claim("INTF_APB2PLL_RST")
        .extra_out_far_claim("INTF_APB2PLL_CK")
        .extra_out_far_claim("INTF_APB2PLL_CA")
        .extra_out_far_claim("INTF_APB2PLL_WR")
        .extra_in_far_from_far("INTF_PLL2APB_CK", bcrd, "S2P_XCV2RIU_CK")
        .extra_in_far_from_far_bus("INTF_PLL2APB_RD", bcrd, "S2P_XCV2RIU_RD", 4)
        .extra_in_far_from_far("INTF_FABRIC_PSCLK", scan, "_CLK_ROOT_INT_71_")
        .extra_out_far_claim("INTF_PSCLK")
        .extra_out_far_claim("INTF_PSEN")
        .extra_out_far_claim("INTF_PSINCDEC")
        .extra_in_far_from_far("INTF_PSDONE", bcrd, "PLL_PSDONE")
        // to SCAN
        .extra_out_far_claim("INTF_MUX2APB_PCLK_XPLL")
        .extra_in_far_from_far("INTF_DFX_SCAN_RST_BYP", phio_cmu, "DFX_SCAN_RST_BYP")
        .extra_in_far_from_far("INTF_DFX_SCAN_MODE_RST_N", phio_cmu, "DFX_SCAN_MODE_RST_N")
        .extra_in_far_from_far("INTF_DFX_SCAN_MODE", phio_cmu, "DFX_SCAN_MODE")
        .extra_in_far_from_far("INTF_DFX_SCAN_EN", phio_cmu, "DFX_SCAN_EN")
        .extra_in_far_from_far("INTF_SCAN_EN", phio_cmu, "SCAN_EN")
        .extra_in_far_from_far("INTF_SCAN_MODE_RST_B", phio_cmu, "SCAN_MODE_RST_B")
        .extra_in_far_from_far("INTF_SCAN_RST_BYP", phio_cmu, "SCAN_RST_BYP")
        // dummy
        .extra_in_far_claim("INTF_SCAN_MODE")
        // handled by DDRMC
        .extra_in_far_claim("INTF_DMC_PD")
        .extra_in_far_claim("INTF_DMC_PCLK")
        .extra_in_far_claim("INTF_DMC_PRESETN")
        .extra_in_far_claim("INTF_DMC_PENABLE")
        .extra_in_far_claim("INTF_DMC_PWRITE")
        .extra_in_far_claim("INTF_DMC_PSEL")
        .extra_in_far_claim_bus("INTF_DMC_PADDR", 10)
        .extra_in_far_claim_bus("INTF_DMC_PWDATA", 16)
        .extra_out_far_claim("INTF_DMC_PREADY")
        .extra_out_far_claim_bus("INTF_DMC_PRDATA", 16)
        .extra_out_far_claim("INTF_DMC_PSLVERR");

    if is_nc {
        bel.claim_net(&[bel.wire("PSCLK")]);
        bel = bel
            .extra_in_far_claim("PSEN")
            .extra_in_far_claim("PSINCDEC")
            .extra_out_far_claim("PSDONE")
            .extra_in_far_claim("PCLK")
            .extra_in_far_claim("PRESETN")
            .extra_in_far_claim("PENABLE")
            .extra_in_far_claim("PWRITE")
            .extra_in_far_claim("PSEL")
            .extra_in_far_claim_bus("PADDR", 10)
            .extra_in_far_claim_bus("PWDATA", 16)
            .extra_out_far_claim("PREADY")
            .extra_out_far_claim_bus("PRDATA", 16)
            .extra_out_far_claim("PSLVERR");
    } else {
        bel = bel
            .extra_in("PSEN")
            .extra_in("PSINCDEC")
            .extra_out("PSDONE")
            .extra_in("PCLK")
            .extra_in("PRESETN")
            .extra_in("PENABLE")
            .extra_in("PWRITE")
            .extra_in("PSEL")
            .extra_in_bus("PADDR", 10)
            .extra_in_bus("PWDATA", 16)
            .extra_out("PREADY")
            .extra_out_bus("PRDATA", 16)
            .extra_out("PSLVERR");
    }

    bel.commit();
}

pub fn verify_x5phio_cmu(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::X5IO_BANK_NC;
    let scan = bcrd.bel(bslots::X5IO_SCAN);
    let octad: [_; 4] = core::array::from_fn(|i| {
        edev.tile_cell(bcrd.tile(tslots::HSR_XPLL), CellSlotId::from_idx(i))
            .bel(bslots::X5IO_OCTAD)
    });
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("X5PHIO_CMUIF")
        .extra_out_far_claim("APB2CMU_RST")
        .extra_out_far_claim("APB2CMU_CK")
        .extra_out_far_claim("APB2CMU_CA")
        .extra_out_far_claim("APB2CMU_WR")
        .extra_in_far_from_far("CMU2APB_CK", bcrd, "IO_CMU2APB_CK")
        .extra_in_far_from_far_bus("CMU2APB_RD", bcrd, "IO_CMU2APB_RD", 4)
        .extra_in_far_from_far("DFX_SCAN_RST_BYP", scan, "_SCAN_RST_BYP")
        .extra_in_far_from_far("DFX_SCAN_MODE_RST_N", scan, "_SCAN_MODE_RST_N")
        .extra_in_far_from_far("DFX_SCAN_EN", scan, "_SCAN_EN")
        .extra_in_far_from_far("SCAN_RST_BYP", scan, "_SCAN_RST_BYP")
        .extra_in_far_from_far("SCAN_MODE_RST_B", scan, "_SCAN_MODE_RST_N")
        .extra_in_far_from_far("SCAN_EN", scan, "_SCAN_EN")
        // dummy
        .extra_in_far_claim("DFX_SCAN_MODE")
        .extra_in_far_claim("SCAN_MODE")
        // to SCAN
        .extra_out_far_claim("MUX2APB_PCLK_CMU")
        // to DMC (and handled by it)
        .extra_in_far_claim("DMC2APB_CK")
        .extra_in_far_claim("DMC2APB_RST")
        .extra_in_far_claim("DMC2APB_PENABLE")
        .extra_in_far_claim("DMC2APB_PWRITE")
        .extra_in_far_claim("DMC2APB_PSEL")
        .extra_in_far_claim_bus("DMC2APB_PADDR", 10)
        .extra_in_far_claim_bus("DMC2APB_PWDATA", 16)
        .extra_out_far_claim("APB2DMC_PREADY")
        .extra_out_far_claim_bus("APB2DMC_PRDATA", 16)
        .extra_out_far_claim("APB2DMC_PSLVERR");

    if is_nc {
        bel = bel
            .extra_in_far_claim("CK")
            .extra_in_far_claim("RST")
            .extra_in_far_claim("PENABLE")
            .extra_in_far_claim("PWRITE")
            .extra_in_far_claim("PSEL")
            .extra_in_far_claim_bus("PADDR", 10)
            .extra_in_far_claim_bus("PWDATA", 16)
            .extra_out_far_claim("PREADY")
            .extra_out_far_claim_bus("PRDATA", 16)
            .extra_out_far_claim("PSLVERR");
    }

    bel.commit();

    let prev = edev
        .bel_delta(bcrd.cell, -1, 0, bcrd.slot)
        .or(edev.bel_delta(bcrd.cell, -7, 0, bcrd.slot));
    let next = edev
        .bel_delta(bcrd.cell, 1, 0, bcrd.slot)
        .or(edev.bel_delta(bcrd.cell, 7, 0, bcrd.slot));

    let mut bel = vrf
        .verify_bel(bcrd)
        .sub(1)
        .kind("X5PHIO_CMU_X32")
        .skip_auto()
        .extra_in_far_from_far("IO_APB2CMU_RST", bcrd, "APB2CMU_RST")
        .extra_in_far_from_far("IO_APB2CMU_CK", bcrd, "APB2CMU_CK")
        .extra_in_far_from_far("IO_APB2CMU_CA", bcrd, "APB2CMU_CA")
        .extra_in_far_from_far("IO_APB2CMU_WR", bcrd, "APB2CMU_WR")
        .extra_out_far_claim("IO_CMU2APB_CK")
        .extra_out_far_claim_bus("IO_CMU2APB_RD", 4)
        .extra_out_far_claim_bus("IO_CMU2CMU_RCAL_OUT", 15)
        .extra_out_far_claim_bus("IO_CMU2XCV_DCI", 17)
        .extra_in_far_from_far("IO_CMU_SCAN_MODE_RST_B", scan, "_SCAN_MODE_RST_N")
        .extra_in_far_from_far("IO_OCTAD_VREF_H1P", octad[0], "IOB2_XCV2XCV_VREF_H1P_O")
        .extra_in_far_from_far("IO_OCTAD_VREF_H1M", octad[0], "IOB2_XCV2XCV_VREF_H1M_O")
        .extra_out_far_claim("IO_DCI_LOCK_DONE_DWN")
        .extra_out_far_claim("IO_DCI_LOCK_DONE_UP")
        // dummy
        .extra_in_far_claim("IO_MASTER_EN_PIN")
        .extra_in_far_claim("IO_CMU_SCAN_MODE")
        .extra_in_far("IO_CMU_SCAN_EN")
        .extra_in_far("IO_CMU_SCAN_RST_BYP");

    if let Some(prev) = prev {
        bel = bel
            .extra_in_far_from_far_bus("IO_CMU2CMU_RCAL_IN", prev, "IO_CMU2CMU_RCAL_OUT", 15)
            .extra_in_far_from_far("IO_DCI_LOCK_INIT_UP", prev, "IO_DCI_LOCK_DONE_UP");
    } else {
        bel = bel
            .extra_in_far_claim_bus("IO_CMU2CMU_RCAL_IN", 15)
            .extra_in_far_from_far("IO_DCI_LOCK_INIT_UP", bcrd, "IO_DCI_LOCK_DONE_DWN");
    }
    if let Some(next) = next {
        bel = bel.extra_in_far_from_far("IO_DCI_LOCK_INIT_DWN", next, "IO_DCI_LOCK_DONE_DWN");
    } else {
        bel = bel.extra_in_far_from_far("IO_DCI_LOCK_INIT_DWN", bcrd, "IO_DCI_LOCK_DONE_UP");
    }

    bel.commit();
}

pub fn verify_x5io_scan(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let chip = edev.chips[bcrd.die];
    let pll = bcrd.bel(bslots::X5PLL);
    let phio_cmu = bcrd.bel(bslots::X5PHIO_CMU);
    let octad: [_; 4] = core::array::from_fn(|i| {
        edev.tile_cell(bcrd.tile(tslots::HSR_XPLL), CellSlotId::from_idx(i))
            .bel(bslots::X5IO_OCTAD)
    });
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind("CMPHY_DFX_HB")
        .extra_out_far_claim("_CLK_ROOT_INT_71_")
        .extra_out_far_claim("_SCAN_MODE")
        .extra_out_far_claim("_SCAN_MODE_RST_N")
        .extra_out_far_claim("_SCAN_RST_BYP")
        .extra_out_far_claim("_SCAN_EN")
        .extra_out_far_claim("_SCAN_MODE_STUCKAT")
        .extra_out_far_claim("_SCAN_EN_STUCKAT")
        .extra_in_far_from_far("_ROOT_CLK_IN_1_", phio_cmu, "MUX2APB_PCLK_CMU")
        .extra_in_far_from_far("_ROOT_CLK_IN_2_", pll, "INTF_MUX2APB_PCLK_XPLL")
        .extra_in_far_from_far("_ROOT_CLK_IN_71_", pll, "PSCLK")
        // from DDRMC (and handled by it)
        .extra_in_far_claim("_CSSD_TRIG_N_0_")
        // dummy
        .extra_in_far("_DFX_RST_N")
        .extra_in_far_claim("_MCLK")
        .extra_in_far_claim("_SYS_RESET_SYNCED");

    for (base, stride, pin) in [
        (3, 1, "MUX2APB_PCLK"),
        (7, 1, "PHY_CLK"),
        (11, 1, "DMC_CLK"),
        (15, 2, "MUX2RDP_CLK_0_"),
        (16, 2, "MUX2RDP_CLK_1_"),
        (23, 2, "XCV2PHY_RD_CLK0_0_"),
        (24, 2, "XCV2PHY_RD_CLK0_1_"),
        (31, 2, "XCV2PHY_RD_CLK1_0_"),
        (32, 2, "XCV2PHY_RD_CLK1_1_"),
        (39, 2, "XCV2PHY_RD_CLK2_0_"),
        (40, 2, "XCV2PHY_RD_CLK2_1_"),
        (47, 2, "XCV2PHY_RD_CLK3_0_"),
        (48, 2, "XCV2PHY_RD_CLK3_1_"),
        (55, 1, "XCV2PHY_WR_CLK0"),
        (59, 1, "XCV2PHY_WR_CLK1"),
        (63, 1, "XCV2PHY_WR_CLK2"),
        (67, 1, "XCV2PHY_WR_CLK3"),
        (72, 1, "MUX2WRP_CLK"),
    ] {
        for (i, oct) in octad.into_iter().enumerate() {
            let oct_tcrd = edev.bel_tile(oct);
            let mut skip = false;
            if chip.ps == PsKind::Ps9Cpm6
                && edev[oct_tcrd].class == tcls::X5IO_NC
                && pin == "PHY_CLK"
            {
                skip = true;
            }
            let HsrSKind::X5io(ref hsr) = chip.hsr_s else {
                unreachable!()
            };
            if pin == "DMC_CLK"
                && hsr.regions[HRegId::from_idx(bcrd.col.to_idx() / 8)].io[1].is_none()
            {
                skip = true;
            }
            if skip {
                bel = bel.extra_in_far(format!("_ROOT_CLK_IN_{ii}_", ii = base + i * stride));
            } else {
                bel = bel.extra_in_far_from_far(
                    format!("_ROOT_CLK_IN_{ii}_", ii = base + i * stride),
                    oct,
                    pin,
                );
            }
        }
    }

    bel.commit();
}

pub fn verify_x5phio_dccibuf(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let idx = bslots::X5PHIO_DCCIBUF.index_of(bcrd.slot).unwrap();
    let obel = edev
        .tile_cell(edev.bel_tile(bcrd), CellSlotId::from_idx(idx / 2))
        .bel(bslots::X5IO_OCTAD);
    vrf.verify_bel(bcrd)
        .extra_in_far_from_far_bus(
            "X5PHIO_DCCIBUF_OVERRIDE_CODE",
            obel,
            [
                "IOB1_XCV2CLK_RIU_CLK0_DCD_ADJ",
                "IOB1_XCV2CLK_RIU_CLK90_DCD_ADJ",
            ][idx % 2],
            4,
        )
        .extra_in_far_from_far(
            "X5PHIO_DCCIBUF_OVERRIDE_SEL",
            obel,
            [
                "IOB1_XCV2CLK_RIU_CLK0_DCD_ADJ_SEL",
                "IOB1_XCV2CLK_RIU_CLK90_DCD_ADJ_SEL",
            ][idx % 2],
        )
        .commit();
}

pub fn verify_ddrmc5(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::DDRMC5 => "DDRMC5",
        tcls::DDRMC5C => "DDRMC5C",
        tcls::DDRMC5E | tcls::DDRMC5E_ALT | tcls::DDRMC5E2 => "DDRMC5E",
        _ => unreachable!(),
    };
    let octad = if bcrd.col.to_idx() % 8 == 4 {
        [-4, -3, -2, -1, 3, 2, 1, 0, 4, 5, 6, 7]
    } else {
        [4, 3, 2, 1, -3, -2, -1, 0, -4, -5, -6, -7]
    }
    .map(|dx| bcrd.delta(dx, 0).bel(bslots::X5IO_OCTAD));
    let phio_cmu = [octad[3], octad[7], octad[11]].map(|bcrd| bcrd.bel(bslots::X5PHIO_CMU));
    let scan = phio_cmu.map(|bcrd| bcrd.bel(bslots::X5IO_SCAN));
    let pll = phio_cmu.map(|bcrd| bcrd.bel(bslots::X5PLL));
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("TO_NOC_0")
        .extra_out_far_claim("TO_NOC_1")
        .extra_in_far("FROM_NOC_0")
        .extra_in_far("FROM_NOC_1")
        // dummy
        .extra_out_far_claim("DMC_CLK_DESKEW");

    for i in 0..3 {
        bel = bel
            .extra_out_far_to_far(
                format!("DMC2PHY_CSSD_TRIG_{i}_"),
                scan[i],
                "_CSSD_TRIG_N_0_",
            )
            .extra_out_far_to_far(format!("DMC2PHY_PD_HB{i}"), pll[i], "INTF_DMC_PD")
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PCLK"),
                pll[i],
                "INTF_DMC_PCLK",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PRESETN"),
                pll[i],
                "INTF_DMC_PRESETN",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PENABLE"),
                pll[i],
                "INTF_DMC_PENABLE",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PWRITE"),
                pll[i],
                "INTF_DMC_PWRITE",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PSEL"),
                pll[i],
                "INTF_DMC_PSEL",
            )
            .extra_out_far_to_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PADDR"),
                pll[i],
                "INTF_DMC_PADDR",
                10,
            )
            .extra_out_far_to_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_DMC2APB_PWDATA"),
                pll[i],
                "INTF_DMC_PWDATA",
                16,
            )
            .extra_in_far_from_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_APB2DMC_PRDATA"),
                pll[i],
                "INTF_DMC_PRDATA",
                16,
            )
            .extra_in_far_from_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_APB2DMC_PREADY"),
                pll[i],
                "INTF_DMC_PREADY",
            )
            .extra_in_far_from_far(
                format!("IF_DMC2X5PHYIO_HB{i}_XPLL_APB2DMC_PSLVERR"),
                pll[i],
                "INTF_DMC_PSLVERR",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PCLK"),
                phio_cmu[i],
                "DMC2APB_CK",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PRESETN"),
                phio_cmu[i],
                "DMC2APB_RST",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PENABLE"),
                phio_cmu[i],
                "DMC2APB_PENABLE",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PWRITE"),
                phio_cmu[i],
                "DMC2APB_PWRITE",
            )
            .extra_out_far_to_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PSEL"),
                phio_cmu[i],
                "DMC2APB_PSEL",
            )
            .extra_out_far_to_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PADDR"),
                phio_cmu[i],
                "DMC2APB_PADDR",
                10,
            )
            .extra_out_far_to_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_DMC2APB_PWDATA"),
                phio_cmu[i],
                "DMC2APB_PWDATA",
                16,
            )
            .extra_in_far_from_far_bus(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_APB2DMC_PRDATA"),
                phio_cmu[i],
                "APB2DMC_PRDATA",
                16,
            )
            .extra_in_far_from_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_APB2DMC_PREADY"),
                phio_cmu[i],
                "APB2DMC_PREADY",
            )
            .extra_in_far_from_far(
                format!("IF_DMC2X5PHYIO_HB{i}_CMU_APB2DMC_PSLVERR"),
                phio_cmu[i],
                "APB2DMC_PSLVERR",
            );
        for j in 0..4 {
            let oct = octad[i * 4 + j];
            bel = bel
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PCLK"),
                    oct,
                    "DMC2APB_PCLK",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PRESETN"),
                    oct,
                    "DMC2APB_PRESETN",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PENABLE"),
                    oct,
                    "DMC2APB_PENABLE",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PWRITE"),
                    oct,
                    "DMC2APB_PWRITE",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PSEL"),
                    oct,
                    "DMC2APB_PSEL",
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PADDR"),
                    oct,
                    "DMC2APB_PADDR",
                    10,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_DMC2APB_PWDATA"),
                    oct,
                    "DMC2APB_PWDATA",
                    16,
                )
                .extra_in_far_from_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_APB2DMC_PRDATA"),
                    oct,
                    "DMC2APB_PRDATA",
                    16,
                )
                .extra_in_far_from_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_APB2DMC_PREADY"),
                    oct,
                    "APB2DMC_PREADY",
                )
                .extra_in_far_from_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RIU_APB2DMC_PSLVERR"),
                    oct,
                    "APB2DMC_PSLVERR",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_DMC2PHY_FIFO_RDEN"),
                    oct,
                    "DMC_FIFO_RDEN",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_DMC2PHY_RXFIFO_RDCLK"),
                    oct,
                    "DMC_FIFO_RDCLK",
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_DMC2PHY_RDEN"),
                    oct,
                    "DMC_PHY_RDEN",
                    4,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_DMC2PHY_RDCS0"),
                    oct,
                    "DMC_PHY_RDCS0",
                    4,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_DMC2PHY_RDCS1"),
                    oct,
                    "DMC_PHY_RDCS1",
                    4,
                )
                .extra_in_far_from_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_PHY2DMC_FIFO_EMPTY"),
                    oct,
                    "DMC_FIFO_EMPTY",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_CLK"),
                    oct,
                    "DMC_CLK",
                )
                .extra_out_far_claim_to_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_RST"),
                    oct,
                    "DMC_RST",
                )
                .extra_in_far_from_far(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_PHY2DMC_PD"),
                    oct,
                    "DMC_PD",
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_WREN"),
                    oct,
                    "DMC_PHY_WREN",
                    4,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_WRCS0"),
                    oct,
                    "DMC_PHY_WRCS0",
                    4,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_WRCS1"),
                    oct,
                    "DMC_PHY_WRCS1",
                    4,
                )
                .extra_out_far_claim_to_far_bus(
                    format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_IBUF_DIS_OR_HS_RX_DIS"),
                    oct,
                    "DMC_IBUF_HS_RX_DIS",
                    8,
                );
            for k in 0..8 {
                bel = bel
                    .extra_in_far_from_far_bus(
                        format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_RX_PHY2DMC_RD_DQ{k}"),
                        oct,
                        format!("DMC_Q{k}"),
                        8,
                    )
                    .extra_out_far_claim_to_far_bus(
                        format!("IF_DMC2X5PHYIO_HB{i}_OCT{j}_TX_DMC2PHY_WR_DQ{k}"),
                        oct,
                        format!("DMC_D{k}"),
                        8,
                    );
            }
        }
    }
    bel.commit();
    crate::noc::verify_noc_conns(edev, vrf, bcrd);
}
