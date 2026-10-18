use prjcombine_tablegen::target_defs;

target_defs! {
    bel_class IRI {
        input CLK, RST, CE[4], IMUX[24];
        output CLK_O, RST_O, CE_O[4], IMUX_O[24];
    }

    bel_class BUFDIV_LEAF {
        input I, I_CASC, CLR_B;
        output O;

        input TEST_SEL[2];
    }

    bel_class BLI_GRP0 {
        input CLK, RST, CE;
        input IMUX_I[16], OUT_CTRL_I[1];
        output IMUX_O[16], OUT_CTRL_O[1];
    }

    bel_class BLI_GRP1 {
        input CLK, RST, CE;
        input OUT_I[32], OUT_CTRL_I[2];
        output OUT_O[32], OUT_CTRL_O[2];
    }

    bel_class BLI_GRP2 {
        input CLK, RST, CE;
        input IMUX_I[17];
        output IMUX_O[17];
    }

    bel_class BLI_GRP456 {
        input CLK, RST, CE;
        input IMUX_I[4], OUT_I[4];
        output IMUX_O[4], OUT_O[4];
    }

    bel_class BLI_GRP7 {
        input CLK, RST, CE;
        input IMUX_I[5], OUT_I[5];
        output IMUX_O[5], OUT_O[5];
    }

    bel_class BLI_TMR {
    }

    bel_class SLICE {
        input A1, A2, A3, A4, A5, A6, AX, A_I;
        input B1, B2, B3, B4, B5, B6, BX, B_I;
        input C1, C2, C3, C4, C5, C6, CX, C_I;
        input D1, D2, D3, D4, D5, D6, DX, D_I;
        input E1, E2, E3, E4, E5, E6, EX, E_I;
        input F1, F2, F3, F4, F5, F6, FX, F_I;
        input G1, G2, G3, G4, G5, G6, GX, G_I;
        input H1, H2, H3, H4, H5, H6, HX, H_I;
        input LAG_E1, LAG_W1, LAG_N, LAG_S, LAG_E2, LAG_W2;
        input CLK, RST, CE[4], WE;
        output A_O, AQ, AQ2;
        output B_O, BQ, BQ2;
        output C_O, CQ, CQ2;
        output D_O, DQ, DQ2;
        output E_O, EQ, EQ2;
        output F_O, FQ, FQ2;
        output G_O, GQ, GQ2;
        output H_O, HQ, HQ2;
    }

    bel_class LAGUNA {
        input TX;
        output RX;
    }

    bel_class BRAM {
        input CLKAL, CLKAU, CLKBL, CLKBU;
        input ENAL, ENAU, ENBL, ENBU;
        input ARST_A_L, ARST_A_U, ARST_B_L, ARST_B_U;
        input REGCEAL, REGCEAU, REGCEBL, REGCEBU;
        input RSTRAMAL, RSTRAMAU, RSTRAMBL, RSTRAMBU;
        input RSTREGAL, RSTREGAU, RSTREGBL, RSTREGBU;
        input ECCPIPECEL;
        input WEAL[4], WEAU[4], WEBL[4], WEBU[4], WE_IND_PARITY;
        input ADDRAL[12], ADDRAU[12], ADDRBL[12], ADDRBU[12];

        input DIAL[16], DIAU[16], DIBL[16], DIBU[16];
        input DIPAL[2], DIPAU[2], DIPBL[2], DIPBU[2];
        output DOAL[16], DOAU[16], DOBL[16], DOBU[16];
        output DOPAL[2], DOPAU[2], DOPBL[2], DOPBU[2];

        input CASDOMUXAL, CASDOMUXAU, CASDOMUXBL, CASDOMUXBU;
        input CASDOMUXEN_AL, CASDOMUXEN_AU, CASDOMUXEN_BL, CASDOMUXEN_BU;
        input CASOREGIMUXAL, CASOREGIMUXAU, CASOREGIMUXBL, CASOREGIMUXBU;
        input CASOREGIMUXEN_AL, CASOREGIMUXEN_AU, CASOREGIMUXEN_BL, CASOREGIMUXEN_BU;

        input INJECTSBITERR, INJECTDBITERR;
        output SBITERR, DBITERR;

        input SLEEPL, SLEEPU;

        output TST_GMC_STAGGER_OUT;
        input TST_RESET_B_L, TST_RESET_B_U;
        input TST_RING_ENB;
        input TST_RING_STARTB;
        output TST_RING_OUT;
        output TST_SLEEP_CNTL;

        input SCAN_CLK_N_EXT;
        input SCAN_EN_N_EXT;
        input SCAN_DATA_EXT_IN[4];
        output SCAN_DATA_EXT_OUT[4];

        output CSSD_STOP_DONE_EXT;
    }

    bel_class URAM {
        input CLK;
        input EN_A, EN_B;
        input RST_A, RST_B;
        input RDB_WR_A, RDB_WR_B;
        input OREG_CE_A, OREG_CE_B;
        input OREG_ECC_CE_A, OREG_ECC_CE_B;
        input BWE_A[9], BWE_B[9];
        input ADDR_A[26], ADDR_B[26];

        input DIN_A[72], DIN_B[72];
        output DOUT_A[72], DOUT_B[72];
        output RDACCESS_A, RDACCESS_B;

        input INJECT_SBITERR_A, INJECT_SBITERR_B;
        input INJECT_DBITERR_A, INJECT_DBITERR_B;
        output SBITERR_A, SBITERR_B;
        output DBITERR_A, DBITERR_B;

        input SLEEP;

        output TST_GMC_STAGGER_OUT;
        input TST_RESET_B;
        input TST_RING_ENB;
        output TST_RING_OUT;
        input TST_RING_STARTB;
        output TST_SLEEP_OUT;

        input SCAN_CLK_N_EXT;
        input SCAN_EN_N_EXT;
        input SCAN_DATA_EXT_IN[5];
        output SCAN_DATA_EXT_OUT[5];

        output CSSD_STOP_DONE_EXT;
    }

    bel_class DSP {
        input CLK, CLK_CPLX;
        input ASYNC_RST, ASYNC_RST_CPLX;

        input A[34];
        input RSTA, CEA1, CEA2;

        input B[32];
        input RSTB, CEB1, CEB2;

        input C[58];
        input RSTC, CEC;

        input D[32];
        input RSTD, RSTD_CPLX, CED, CED_CPLX;

        input INMODE[5];
        input RSTINMODE, CEINMODE;
        input RSTAD, CEAD, CEAD_CPLX;

        input RSTM, CEM;

        input CARRYIN;
        input CARRYINSEL[3];
        input RSTALLCARRYIN, CECARRYIN;

        input OPMODE[9];
        input RSTCTRL, CECTRL;

        input NEGATE[3];
        input ALUMODE[4];
        input RSTALUMODE, CEALUMODE;

        input RSTP, CEP;

        output P[58];
        output CARRYOUT[4];
        output XOROUT[8];

        output PATTERNDETECT;
        output PATTERNBDETECT;
        output OVERFLOW;
        output UNDERFLOW;

        input SCAN_CLK_N_EXT;
        input SCAN_CLK_N_EXT_CPLX;
        input SCAN_EN_N_EXT;
        input SCAN_EN_N_EXT_CPLX;
        input SCAN_DATA_EXT_IN[2];
        output SCAN_DATA_EXT_OUT[2];

        output CSSD_STOP_DONE_EXT;
    }

    bel_class HDIOLOGIC {
        output IPFF_BYPASS;
        input IPFF_CK, IPFF_CK_C;
        input IPFF_RST;
        input IPFF_CE;
        output IPFF_Q1, IPFF_Q2;

        input IDLY_CLK;
        input IDLY_RST;
        input IDLY_CE;
        input IDLY_INC;
        input IDLY_LOAD;
        input IDLY_CNTVALUEIN[5];
        output IDLY_CNTVALUEOUT[5];

        input OPFF_CK;
        input OPFF_RST;
        input OPFF_CE;
        input OPFF_D1, OPFF_D2;
        input TFF_D1, TFF_D2;

        input ODLY_CLK;
        input ODLY_RST;
        input ODLY_CE;
        input ODLY_INC;
        input ODLY_LOAD;
        input ODLY_CNTVALUEIN[5];
        output ODLY_CNTVALUEOUT[5];
    }

    bel_class HDIOLOGIC_SCAN {
        output APB_LGC_SCANOUT;
        input APB_LGC_SCAN_CLK;
        input APB_LGC_SCAN_EN;
        input APB_LGC_SCAN_IN;
        input APB_LGC_SCAN_MODE;
        input APB_LGC_SCAN_RST;

        output LGC_SCANOUT;
        input LGC_SCAN_CLK;
        input LGC_SCAN_EN;
        input LGC_SCAN_IN;
        input LGC_SCAN_MODE;
        input LGC_SCAN_RST;
        input LGC_SCAN_SET;
    }

    bel_class HDIOB {
        input IBUF_DISABLE;
        input PD_EN;
        input PU_EN;
        input KEEPER_EN;
        input DYNAMIC_DCI_TS;

        output CLKPAD;

        pad PAD: inout;
    }

    bel_class HDIO_BIAS {
    }

    bel_class RPI_HD_APB {
        input PCLK;
        input PRESET_N;
        input PENABLE;
        input PSEL;
        input PWRITE;
        input PADDR[9];
        input PWDATA[16];
        output PREADY;
        output PSLVERR;
        output PRDATA[16];

        input APB_SCAN_CLK;
        input APB_SCAN_RST;
        input APB_SCAN_IN;
        output APB_SCAN_OUT;
    }

    bel_class HDLOGIC_APB {
        input HDBSCAN_DATA;
        input HDLOGIC_BSCAN_MODE;
        input HDLOGIC_SCAN_EN;
        input HDLOGIC_SCAN_MODE;
        output HDLOGIC_SCAN_OUT;
    }

    bel_class XPHY {
        input CTRL_CLK;
        input RST;

        input FIFO_RD_CLK;
        input FIFO_RDEN;
        output FIFO_EMPTY;
        // special out
        output FIFO_WR_CLK;

        input EN_VTC;
        output DLY_RDY;
        output GT_STATUS;
        output PHY_RDY;
        output MST_PD;

        input PHY_RDCS0[4];
        input PHY_RDCS1[4];
        input PHY_RDEN[4];
        input PHY_WRCS0[4];
        input PHY_WRCS1[4];
        input PHY_WREN[4];

        input RIU_WR_EN;
        input RIU_NIBBLE_SEL;
        input RIU_ADDR[8];
        input RIU_WR_DATA[16];
        output RIU_RD_VALID;
        output RIU_RD_DATA[16];

        input D0[8];
        input D1[8];
        input D2[8];
        input D3[8];
        input D4[8];
        input D5[8];

        output Q0[8];
        output Q1[8];
        output Q2[8];
        output Q3[8];
        output Q4[8];
        output Q5[8];
    }

    bel_class XPHY_NIBBLESLICE {
        input RX_RST, TX_RST;
        input D[8], T;
        output Q[8];

        input RX_EN_VTC, TX_EN_VTC;
        input RXTX_SEL, CE, INC, LD;
        input CNTVALUEIN[9];
        output CNTVALUEOUT[9];
    }

    bel_class XPIOLOGIC {
        input GCLK_SIG[6];

        output IPFF_BYPASS;
        input IPFF_RST;
        input IPFF_CE;
        output IPFF_Q1, IPFF_Q2;

        input IDLY_RST;
        input IDLY_CE;
        input IDLY_INC;
        input IDLY_LOAD;
        input IDLY_CNTVALUEIN[5];
        output IDLY_CNTVALUEOUT[5];

        input OPFF_RST;
        input OPFF_CE;
        input OPFF_D1, OPFF_D2;
        input TFF_D1, TFF_D2;

        input ODLY_RST;
        input ODLY_CE;
        input ODLY_INC;
        input ODLY_LOAD;
        input ODLY_CNTVALUEIN[5];
        output ODLY_CNTVALUEOUT[5];
    }

    bel_class XPIOB {
        input IBUF_DISABLE;
        input PD_EN;
        input PU_EN;
        input KEEPER_EN;
        input DYNAMIC_DCI_TS;
        input TERM_EN;
        output LP_I;

        output CLKPAD;

        pad PAD: inout;
    }

    bel_class XPIO_VREF {
    }

    bel_class XPIO_DCI {
        input OK_TO_UPDATE;
        output DCI_LOCK;

        input DCI_TEST_I[24];
        output DCI_TEST_O[24];
    }

    bel_class XPIO_SCAN {
        input SCAN_CLK_B;
        input SCAN_MODE_RST_B;
        input SCAN_EN_B;
        input SCAN_EDT_UPDT_B;

        input CSSD_TRIG_XPHY[9];
        input SCAN_IN_XPHY[9];
        output SCAN_OUT_XPHY[9];
        input SCAN_CNTRL_CHNL_IN_XPHY[9];
        output SCAN_CNTRL_CHNL_OUT_XPHY[9];
        input SCAN_IN_IOL[9];
        output SCAN_OUT_IOL[9];
    }

    bel_class DDRMC {
        input DMC_CLK;
        output DMC_CLK_DESKEW;
        input DBG_PLL1_CLK;
        output TEST_CLK_OUT[2];

        input IF_DMC_FABRIC_BLOCK_PERIODIC_CAL;
        output IF_CAL_CAL_BUSY;
        output IF_CAL_CAL_DONE;

        input IF_DMC_FABRIC_BLI2ILA_ACK;
        input IF_DMC_FABRIC_BLI2ILA_TRIG;
        output IF_DMC_FABRIC_ILA2BLI_ACK;
        output IF_DMC_FABRIC_ILA2BLI_TRIG;

        input IF_DMC_FABRIC_BLI2UB_TRACE_CLK;
        input IF_DMC_FABRIC_BLI2UB_TRACE_RST_N;
        input IF_DMC_FABRIC_BLI2UB_TRACE_TREADY;
        output IF_DMC_FABRIC_UB2BLI_TRACE_TVALID;
        output IF_DMC_FABRIC_UB2BLI_TRACE_TDATA[16];
        input IF_DMC_FABRIC_BLI2UB_ACK;
        input IF_DMC_FABRIC_BLI2UB_TRIG;
        output IF_DMC_FABRIC_UB2BLI_ACK;
        output IF_DMC_FABRIC_UB2BLI_TRIG;
        input IF_DMC_FABRIC_BLI2UB_UART_RX;
        output IF_DMC_FABRIC_UB2BLI_UART_TX;

        input IF_DMC_FABRIC_CSSD_TRIG_IN_N_EXT;

        input IF_DMC_FABRIC_DMC_MISC_IN[31];
        output IF_DMC_FABRIC_DMC_MISC_OUT[42];

        output IF_DMC_FABRIC_REF_USR_PORT_AVAILABLE;
        input IF_DMC_FABRIC_REF_RANK_EN_0[4];
        input IF_DMC_FABRIC_REF_RANK_EN_1[4];
        input IF_DMC_FABRIC_REF_REQ_0;
        input IF_DMC_FABRIC_REF_REQ_1;
        output IF_DMC_FABRIC_REF_ACK_0[4];
        output IF_DMC_FABRIC_REF_ACK_1[4];

        input IF_UBLAZE_FABRIC_FABRIC_CLK;
        input IF_UBLAZE_FABRIC_FABRIC_RST_N;
        output IF_UBLAZE_FABRIC_PENABLE;
        output IF_UBLAZE_FABRIC_PSELX;
        output IF_UBLAZE_FABRIC_PADDR[32];
        output IF_UBLAZE_FABRIC_PWRITE;
        output IF_UBLAZE_FABRIC_PPROT[3];
        output IF_UBLAZE_FABRIC_PWDATA[32];
        output IF_UBLAZE_FABRIC_PSTRB[4];
        input IF_UBLAZE_FABRIC_PREADY;
        input IF_UBLAZE_FABRIC_PRDATA[32];
        input IF_UBLAZE_FABRIC_PSLVERR;

        input IF_UBLAZE_FABRIC_UB_MISC_IN[3];
        output IF_UBLAZE_FABRIC_UB_MISC_OUT[2];
    }

    bel_class X5IO_OCTAD {
        input PCLK;
        input PRESETN;
        input PENABLE;
        input PWRITE;
        input PSEL;
        input PADDR[10];
        input PWDATA[16];
        output PREADY;
        output PRDATA[16];
        output PSLVERR;

        input PHY_CLK;
        input PHY_2TO1_CLK;
        input RST;
        output PHY_PD;
        output PHY_RDY;
        output DLY_RDY;

        input PHY_RDEN[8];
        input PHY_RDCS0[8];
        input PHY_RDCS1[8];
        input PHY_WREN[8];
        input PHY_WRCS0[8];
        input PHY_WRCS1[8];

        input RXFIFO_RDCLK[2];
        input RXFIFO_RDEN[2];
        output FIFO_EMPTY[2];
        output RXFIFO_WR_CLK[2];

        input RX_RST[8];
        input TX_RST[8];

        input T[8];
        for i in 0..8 {
            input "D{i}"[16];
            output "Q{i}"[16];
        }

        input PLL_CLK0, PLL_CLK90;
        input PDQS_IN, NDQS_IN;
        output PDQS_O, NDQS_O;
        output PDQS_DLY_O, NDQS_DLY_O;
        output RX0_RDQS_P_OUT, RX1_RDQS_P_OUT;
        input DQS_IN_R;
        output DQS_DLY_OUT_R;
        output CCIO[2];
        output DIV64_CLK;

        input IBUFDISABLEIN[8];
        input DYN_DCI_IN[8];
        input CLB2PHY_KEEPER_EN[8];
        input CLB2PHY_LP_RX_DIS_OR_TERM_EN[4];
        input CLB2PHY_LP_TX_O_0[4];
        input CLB2PHY_LP_TX_O_1[4];
        input CLB2PHY_LP_TX_T[4];
        output PHY2CLB_LP_I_0[4];
        output PHY2CLB_LP_I_1[4];
        output PHY2CLB_STATUS;

        pad PAD[8]: inout;
    }

    bel_class X5PHIO_CMU {
        input CK;
        input RST;
        input PENABLE;
        input PWRITE;
        input PSEL;
        input PADDR[10];
        input PWDATA[16];
        output PREADY;
        output PRDATA[16];
        output PSLVERR;
    }

    bel_class X5IO_SCAN {
        input CSSD_TRIG_B;
        input SCAN_CLK_N;
        input SCAN_MODE_RST_N;
        input SCAN_EN_N;
        input SCAN_EDT_UPDT_N;
        input SCAN_CHNL_IN[4];
        input SCAN_CHNL_MASK_IN[3];
        output SCAN_CHNL_OUT[4];
        input SCAN_CNTRL_CHNL_IN[2];
        output SCAN_CNTRL_CHNL_OUT[2];
    }

    bel_class X5PHIO_DCCIBUF {
        input A;
        output ZN;
    }

    bel_class X5PHIO_HARD_INV {
        input I;
        output O;
    }

    bel_class DDRMC5 {
        input DMC_CLK;
        input DBG_PLL1_CLK;
        input PLL2_CLK;
        output TEST_CLK_OUT[2];

        input IF_DMC_FABRIC_BLOCK_PERIODIC_CAL;
        output IF_CAL_CAL_BUSY;
        output IF_CAL_CAL_DONE;

        input IF_DMC_FABRIC_BLI2ILA_ACK;
        input IF_DMC_FABRIC_BLI2ILA_TRIG;
        output IF_DMC_FABRIC_ILA2BLI_ACK;
        output IF_DMC_FABRIC_ILA2BLI_TRIG;

        input IF_DMC_FABRIC_EXMON_CLEAR_IN;
        output IF_DMC_FABRIC_EXMON_CLEAR_OUT;

        output IF_DMC_FABRIC_REF_ACK_0[4];
        output IF_DMC_FABRIC_REF_ACK_1[2];
        input IF_DMC_FABRIC_REF_RANK_EN_0[4];
        input IF_DMC_FABRIC_REF_RANK_EN_1[2];
        output IF_DMC_FABRIC_REF_USR_PORT_AVAILABLE;

        input IF_DMC_FABRIC_CSSD_TRIG_IN_N_EXT;
        output IF_DMC_FABRIC_CSSD_CLKSTP_BCAST;
        input IF_DMC_FABRIC_SCAN_CLK_N_EXT;
        input IF_DMC_FABRIC_SCAN_MODE_RST_N_EXT;
        input IF_DMC_FABRIC_SCAN_EN_N_EXT;
        input IF_DMC_FABRIC_SCAN_EDT_UPDT_N_EXT;
        input IF_DMC_FABRIC_SCAN_CHNL_IN_EXT[13];
        input IF_DMC_FABRIC_SCAN_CHNL_MASK_IN_EXT[13];
        output IF_DMC_FABRIC_SCAN_CHNL_OUT_EXT_CRYPTO[13];
        output IF_DMC_FABRIC_SCAN_CHNL_OUT_EXT_DDRMC[10];
        input IF_DMC_FABRIC_SCAN_CNTRL_CHNL_IN_EXT_CRYPTO;
        input IF_DMC_FABRIC_SCAN_CNTRL_CHNL_IN_EXT_DDRMC;
        output IF_DMC_FABRIC_SCAN_CNTRL_CHNL_OUT_EXT_CRYPTO;
        output IF_DMC_FABRIC_SCAN_CNTRL_CHNL_OUT_EXT_DDRMC;

        input IF_DMC_FABRIC_BLI2UB_ACK;
        input IF_DMC_FABRIC_BLI2UB_TRIG;
        output IF_DMC_FABRIC_UB2BLI_TRIG;
        output IF_DMC_FABRIC_UB2BLI_ACK;

        input IF_DMC_FABRIC_BLI2UB_TRACE_CLK;
        input IF_DMC_FABRIC_BLI2UB_TRACE_RST_N;
        input IF_DMC_FABRIC_BLI2UB_TRACE_TREADY;
        output IF_DMC_FABRIC_UB2BLI_TRACE_TVALID;
        output IF_DMC_FABRIC_UB2BLI_TRACE_TDATA[16];

        input IF_DMC_FABRIC_BLI2UB_UART_RX;
        output IF_DMC_FABRIC_UB2BLI_UART_TX;

        input IF_UBLAZE_FABRIC_BLI2UB_CLK;
        input IF_UBLAZE_FABRIC_BLI2UB_RST;
        output IF_UBLAZE_FABRIC_UB2BLI_ADDRESS[32];
        output IF_UBLAZE_FABRIC_UB2BLI_ADDR_STROBE;
        output IF_UBLAZE_FABRIC_UB2BLI_READ_STROBE;
        output IF_UBLAZE_FABRIC_UB2BLI_WRITE_DATA[32];
        input IF_UBLAZE_FABRIC_BLI2UB_READY;
        input IF_UBLAZE_FABRIC_BLI2UB_READ_DATA[32];

        input IF_UBLAZE_FABRIC_UB_MISC_IN[3];
        output IF_UBLAZE_FABRIC_UB_MISC_OUT[2];
    }

    bel_class GCLK_PD {
        input CLK_IN0, CLK_IN1;
        output PD_OUT;
    }

    bel_class GCLK_DELAY {
        input I;
        output O;

        // some of those are shared with BUFG_FABRIC inputs
        input I_TEST;

        input PD_BOT;
        input PD_LEFT;
        input PD_RIGHT;
        input PD_TOP;
    }

    bel_class GCLK_TAPS_DECODE_VNOC {
        input INC, DEC;
        input LOAD;
        input TAPS_SEL_L[5], TAPS_SEL_R[5];
        output LOCK_L, LOCK_R;
        output TAPS_L[9], TAPS_R[9];
    }

    bel_class GCLK_TAPS_DECODE_GT {
        input INC, DEC;
        input LOAD;
        input TAPS_SEL[5];
        output LOCK;
        output TAPS[9];
    }

    bel_class GCLK_TEST_BUF {
        input I;
        output O;
    }

    bel_class BUFG {
        input I;
        output O;
    }

    bel_class BUFGCE {
        input I, CE;
        output O;
    }

    bel_class BUFGCE_DIV {
        input I, CLR, CE;
        output O;
    }

    bel_class BUFGCTRL {
        input I0, I1;
        input CE0, CE1;
        input IGNORE0, IGNORE1;
        input SEL0, SEL1;
        output O;
    }

    bel_class BUFG_GT {
        input I, CLR, CE;
        input CLRMASK, CEMASK;
        input DIV[3];
        output O;
    }

    bel_class BUFG_GT_SYNC {
        input CLK;
        input CE, CLR;
        output CESYNC, CLRSYNC;
    }

    bel_class MMCM {
        input CLKIN1, CLKIN2, CLKFBIN;
        input CLKIN1_DESKEW, CLKIN2_DESKEW;
        input CLKINSEL;
        input CLKFB1_DESKEW, CLKFB2_DESKEW;
        output CLKINSTOPPED, CLKFBSTOPPED;

        output CLKOUT0, CLKOUT1, CLKOUT2, CLKOUT3, CLKOUT4, CLKOUT5, CLKOUT6, CLKFBOUT, TMUXOUT;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[7];
        input DI[16];
        output DRDY;
        output DO[16];

        input RST;
        input PWRDWN;
        output LOCKED;
        output LOCKED1_DESKEW;
        output LOCKED2_DESKEW;
        output LOCKED_FB;

        input PSCLK, PSEN, PSINCDEC;
        output PSDONE;

        input SCANCLK;
        input SCANENB;
        input SCANMODEB;
        input SCANIN;
        output SCANOUT;

        input TESTIN[8];
        output TESTOUT[13];
    }

    bel_class DPLL {
        input CLKIN, CLKIN_DESKEW;
        input CLKFB_DESKEW;

        output CLKOUT0, CLKOUT1, CLKOUT2, CLKOUT3, TMUXOUT;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[7];
        input DI[16];
        output DRDY;
        output DO[16];

        input RST;
        input PWRDWN;
        output LOCKED;
        output LOCKED_DESKEW;
        output LOCKED_FB;

        input PSCLK, PSEN, PSINCDEC;
        output PSDONE;

        input SCANCLK;
        input SCANENB;
        input SCANMODEB;
        input SCANIN;
        output SCANOUT;

        input TESTIN[8];
        output TESTOUT[2];
    }

    bel_class XPLL {
        input CLKIN;
        input CLKIN1_DESKEW, CLKIN2_DESKEW;
        input CLKFB1_DESKEW, CLKFB2_DESKEW;

        output CLKOUT0, CLKOUT1, CLKOUT2, CLKOUT3, TMUXOUT;

        input CLKOUTPHYEN;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[7];
        input DI[16];
        output DRDY;
        output DO[16];

        input RST;
        input PWRDWN;
        output LOCKED;
        output LOCKED1_DESKEW;
        output LOCKED2_DESKEW;
        output LOCKED_FB;

        input PSCLK, PSEN, PSINCDEC;
        output PSDONE;

        input SCANCLK;
        input SCANENB;
        input SCANMODEB;
        input SCANIN;
        output SCANOUT;

        input TESTIN[8];
        output TESTOUT;
    }

    bel_class X5PLL {
        input CLKIN;
        input CLKIN1_DESKEW, CLKIN2_DESKEW;
        input CLKFB1_DESKEW, CLKFB2_DESKEW;

        output CLKOUTPHY_0, CLKOUTPHY_90;
        output CLKOUT0, CLKOUT1, CLKOUT2, CLKOUT3, TMUXOUT;

        input CLKOUTPHYEN;

        input PCLK;
        input PRESETN;
        input PENABLE;
        input PWRITE;
        input PSEL;
        input PADDR[10];
        input PWDATA[16];
        output PREADY;
        output PRDATA[16];
        output PSLVERR;

        input RST;
        input PWRDWN;
        output LOCKED;
        output LOCKED1_DESKEW;
        output LOCKED2_DESKEW;
        output LOCKED_FB;

        input PSCLK, PSEN, PSINCDEC;
        output PSDONE;

        input TESTIN[8];
        output TESTOUT;
    }

    bel_class MISR {
        input MISR_CLK;
        input MISR_IN[24];
    }

    bel_class NOC_NSU512 {
        input CLK;

        input IF_NOC_AXI_ARREADY;
        output IF_NOC_AXI_ARVALID;
        output IF_NOC_AXI_ARADDR[64];
        output IF_NOC_AXI_ARADDR_PAR[8];
        output IF_NOC_AXI_ARBURST[2];
        output IF_NOC_AXI_ARCACHE[4];
        output IF_NOC_AXI_ARCID[2];
        // ID_INFO not on NOC2
        output IF_NOC_AXI_ARID_INFO[16];
        output IF_NOC_AXI_ARLEN[8];
        output IF_NOC_AXI_ARLOCK[1];
        output IF_NOC_AXI_ARPROT[3];
        output IF_NOC_AXI_ARQOS[4];
        output IF_NOC_AXI_ARREGION[4];
        output IF_NOC_AXI_ARSIZE[3];
        // SRC_INFO not on NOC2
        output IF_NOC_AXI_ARSRC_INFO[12];
        output IF_NOC_AXI_ARUSER[16];

        input IF_NOC_AXI_AWREADY;
        output IF_NOC_AXI_AWVALID;
        output IF_NOC_AXI_AWADDR[64];
        output IF_NOC_AXI_AWADDR_PAR[8];
        output IF_NOC_AXI_AWBURST[2];
        output IF_NOC_AXI_AWCACHE[4];
        output IF_NOC_AXI_AWCID[2];
        // ID_INFO not on NOC2
        output IF_NOC_AXI_AWID_INFO[16];
        output IF_NOC_AXI_AWLEN[8];
        output IF_NOC_AXI_AWLOCK[1];
        output IF_NOC_AXI_AWPROT[3];
        output IF_NOC_AXI_AWQOS[4];
        output IF_NOC_AXI_AWREGION[4];
        output IF_NOC_AXI_AWSIZE[3];
        // SRC_INFO not on NOC2
        output IF_NOC_AXI_AWSRC_INFO[12];
        output IF_NOC_AXI_AWUSER[16];

        output IF_NOC_AXI_BREADY;
        input IF_NOC_AXI_BVALID;
        input IF_NOC_AXI_BCID[2];
        input IF_NOC_AXI_BRESP[2];
        input IF_NOC_AXI_BUSER[16];

        output IF_NOC_AXI_RREADY;
        input IF_NOC_AXI_RVALID;
        input IF_NOC_AXI_RDATA[512];
        input IF_NOC_AXI_RDATA_PAR[64];
        input IF_NOC_AXI_RCID[2];
        input IF_NOC_AXI_RLAST[1];
        input IF_NOC_AXI_RPOISON;
        input IF_NOC_AXI_RRESP[2];
        input IF_NOC_AXI_RUSER[16];

        input IF_NOC_AXI_WREADY;
        output IF_NOC_AXI_WVALID;
        output IF_NOC_AXI_WCID[6];
        output IF_NOC_AXI_WDATA[512];
        output IF_NOC_AXI_WDATA_PAR[64];
        // ID_INFO not on NOC2
        output IF_NOC_AXI_WID_INFO[16];
        output IF_NOC_AXI_WLAST[1];
        output IF_NOC_AXI_WPOISON;
        // SRC_INFO not on NOC2
        output IF_NOC_AXI_WSRC_INFO[12];
        output IF_NOC_AXI_WSTRB[64];
        output IF_NOC_AXI_WUSER[16];

        output IF_NOC_AXI_TDEST[10];

        input NSU_USR_INTERRUPT_IN[4];

        // NOC2 only
        output NSU_FIREWALL_ERR;

        // NOC only
        output IF_NOC_DBG_OUT_DBG64_CLK;
        output IF_NOC_DBG_OUT_DBG64_MUX[64];
        input SPARE_IN[8];
        output SPARE_OUT[63];
    }

    bel_class NOC_NMU512 {
        input CLK;

        output IF_NOC_AXI_ARREADY;
        input IF_NOC_AXI_ARVALID;
        input IF_NOC_AXI_ARADDR[64];
        input IF_NOC_AXI_ARADDR_PAR[8];
        input IF_NOC_AXI_ARBURST[2];
        input IF_NOC_AXI_ARCACHE[4];
        input IF_NOC_AXI_ARID[16];
        input IF_NOC_AXI_ARLEN[8];
        input IF_NOC_AXI_ARLOCK[1];
        input IF_NOC_AXI_ARPROT[3];
        input IF_NOC_AXI_ARQOS[4];
        input IF_NOC_AXI_ARREGION[4];
        input IF_NOC_AXI_ARSIZE[3];
        input IF_NOC_AXI_ARUSER[16];

        output IF_NOC_AXI_AWREADY;
        input IF_NOC_AXI_AWVALID;
        input IF_NOC_AXI_AWADDR[64];
        input IF_NOC_AXI_AWADDR_PAR[8];
        input IF_NOC_AXI_AWBURST[2];
        input IF_NOC_AXI_AWCACHE[4];
        input IF_NOC_AXI_AWID[16];
        input IF_NOC_AXI_AWLEN[8];
        input IF_NOC_AXI_AWLOCK[1];
        input IF_NOC_AXI_AWPROT[3];
        input IF_NOC_AXI_AWQOS[4];
        input IF_NOC_AXI_AWREGION[4];
        input IF_NOC_AXI_AWSIZE[3];
        input IF_NOC_AXI_AWUSER[16];

        input IF_NOC_AXI_BREADY;
        output IF_NOC_AXI_BVALID;
        output IF_NOC_AXI_BID[16];
        output IF_NOC_AXI_BRESP[2];
        output IF_NOC_AXI_BUSER[16];

        input IF_NOC_AXI_RREADY;
        output IF_NOC_AXI_RDATA[512];
        output IF_NOC_AXI_RDATA_PAR[64];
        output IF_NOC_AXI_RID[16];
        output IF_NOC_AXI_RLAST[1];
        output IF_NOC_AXI_RPOISON;
        output IF_NOC_AXI_RRESP[2];
        output IF_NOC_AXI_RUSER[16];
        output IF_NOC_AXI_RVALID;

        output IF_NOC_AXI_WREADY;
        input IF_NOC_AXI_WVALID;
        input IF_NOC_AXI_WDATA[512];
        input IF_NOC_AXI_WDATA_PAR[64];
        input IF_NOC_AXI_WID[16];
        input IF_NOC_AXI_WLAST[1];
        input IF_NOC_AXI_WPOISON;
        input IF_NOC_AXI_WSTRB[64];
        input IF_NOC_AXI_WUSER[16];

        input IF_NOC_AXI_TDEST[10];

        // NOC2 only has 4 bits of USR_DST
        input NMU_RD_DEST_MODE;
        input NMU_RD_USR_DST[12];
        input NMU_WR_DEST_MODE;
        input NMU_WR_USR_DST[12];
        // NOC2 only
        output NMU_FIREWALL_RD_ERR;
        output NMU_FIREWALL_WR_ERR;

        input NMU_USR_INTERRUPT_IN[4];

        // NOC only
        output IF_NOC_DBG_OUT_DBG64_CLK;
        output IF_NOC_DBG_OUT_DBG64_MUX[64];
        input SPARE_IN[8];
        output SPARE_OUT[63];
    }

    bel_class NOC_NMU256 {
    }

    bel_class NOC_NSU256 {
    }

    bel_class NOC_NMU128 {
    }

    bel_class NOC_NSU128 {
    }

    bel_class NOC_NCRB {
    }

    bel_class NPI_NIR {
    }

    bel_class NOC_NIDB {
    }

    bel_class NOC_NPP_RPTR {
    }

    bel_class NOC_NPS {
        // OG VNOC only
        input IF_NOC_DBG128_OUT_DBG_CTRL[8];
        output IF_NOC_DBG128_OUT_DBG_MUX[128];
    }

    bel_class NOC2_NPS6X {
    }

    bel_class NOC2_XBR2X4 {
    }

    bel_class NOC2_XBR4X2 {
    }

    bel_class NOC2_SCAN {
        input IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N;
        input IF_NOC2_SCAN_CNTL_IN_SCAN_CSSD_TRIG_N;
        input IF_NOC2_SCAN_CNTL_IN_SCAN_EDT_UPDT_N;
        input IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N;
        input IF_NOC2_SCAN_CNTL_IN_SCAN_MODE_RST_N;
        input NOC2_SCAN_CHNL_MASK_FROM_PL[14];
        input NOC2_SCAN_CHNL_FROM_PL[15];
        output NOC2_SCAN_CHNL_TO_PL[15];
    }

    bel_class SYSMON_SAT {
        input FABRICCLK;
        output DATAREADY;
        output ALARM[8];
        output PVALUE[16];
        output TVALUE[16];
        output VVALUE[16];

        input TESTADCIN[16];
        output TESTADCOUT[16];

        input CSSDTRIGN;
        input SCANCLKNEXT;
        input SCANENNEXT;
        input SCANMODERSTNEXT;
        input SCANEDTUPDTNEXT;
        input SCANCHNLINEXT;
        output SCANCHNLOUTEXT;
        input SCANCNTRLCHNLINEXT;
        output SCANCNTRLCHNLOUTEXT;
    }

    bel_class RCLK_DFX_TEST {
        input CSSD_TRIG_EXT;
        input TST_RESET;
    }

    bel_class PCIE4 {
        input APB3CLK;
        input APB3PRESETN;
        input APB3PENABLE;
        input APB3PWRITE;
        input APB3PSEL;
        input APB3PADDR[9];
        input APB3PWDATA[32];
        output APB3PREADY;
        output APB3PRDATA[32];
        output APB3PSLVERR;

        input CORECLK;
        input RESETN;

        input MAXISCQTREADY[22];
        output MAXISCQTVALID;
        output MAXISCQTDATA[512];
        output MAXISCQTKEEP[16];
        output MAXISCQTLAST;
        output MAXISCQTUSER[229];

        input MAXISRCTREADY[22];
        output MAXISRCTVALID;
        output MAXISRCTDATA[512];
        output MAXISRCTKEEP[16];
        output MAXISRCTLAST;
        output MAXISRCTUSER[161];

        output SAXISCCTREADY[4];
        input SAXISCCTVALID;
        input SAXISCCTDATA[512];
        input SAXISCCTKEEP[16];
        input SAXISCCTLAST;
        input SAXISCCTUSER[81];

        output SAXISRQTREADY[4];
        input SAXISRQTVALID;
        input SAXISRQTDATA[512];
        input SAXISRQTKEEP[16];
        input SAXISRQTLAST;
        input SAXISRQTUSER[137];

        input AXIUSERIN[16];
        output AXIUSEROUT[16];

        output CFG10BTAGREQUESTERENABLE[4];
        output CFGATOMICREQUESTERENABLE[4];
        output CFGBUSNUMBER[8];
        input CFGCONFIGSPACEENABLE;
        output CFGCURRENTSPEED[2];
        input CFGDEVIDPF0[16];
        input CFGDEVIDPF1[16];
        input CFGDEVIDPF2[16];
        input CFGDEVIDPF3[16];
        input CFGDSBUSNUMBER[8];
        input CFGDSDEVICENUMBER[5];
        input CFGDSFUNCTIONNUMBER[3];
        input CFGDSN[64];
        input CFGDSPORTNUMBER[8];
        input CFGERRCORIN;
        output CFGERRCOROUT;
        output CFGERRFATALOUT;
        output CFGERRNONFATALOUT;
        input CFGERRUNCORIN;
        output CFGEXTFUNCTIONNUMBER[8];
        input CFGEXTREADDATAVALID;
        input CFGEXTREADDATA[32];
        output CFGEXTREADRECEIVED;
        output CFGEXTREGISTERNUMBER[10];
        output CFGEXTTAGENABLE;
        output CFGEXTWRITEBYTEENABLE[4];
        output CFGEXTWRITEDATA[32];
        output CFGEXTWRITERECEIVED;
        output CFGFCCPLDSCALE[2];
        output CFGFCCPLD[12];
        output CFGFCCPLHSCALE[2];
        output CFGFCCPLH[8];
        output CFGFCNPDSCALE[2];
        output CFGFCNPD[12];
        output CFGFCNPHSCALE[2];
        output CFGFCNPH[8];
        output CFGFCPDSCALE[2];
        output CFGFCPD[12];
        output CFGFCPHSCALE[2];
        output CFGFCPH[8];
        input CFGFCSEL[3];
        input CFGFLRDONE[4];
        output CFGFLRINPROCESS[4];
        output CFGFUNCTIONPOWERSTATE[12];
        output CFGFUNCTIONSTATUS[16];
        input CFGHOTRESETIN;
        output CFGHOTRESETOUT;
        input CFGINTERRUPTINT[4];
        input CFGINTERRUPTMSIATTR[3];
        output CFGINTERRUPTMSIDATA[32];
        output CFGINTERRUPTMSIENABLE[4];
        output CFGINTERRUPTMSIFAIL;
        input CFGINTERRUPTMSIFUNCTIONNUMBER[8];
        input CFGINTERRUPTMSIINT[32];
        output CFGINTERRUPTMSIMASKUPDATE;
        output CFGINTERRUPTMSIMMENABLE[12];
        input CFGINTERRUPTMSIPENDINGSTATUSDATAENABLE;
        input CFGINTERRUPTMSIPENDINGSTATUSFUNCTIONNUM[2];
        input CFGINTERRUPTMSIPENDINGSTATUS[32];
        input CFGINTERRUPTMSISELECT[2];
        output CFGINTERRUPTMSISENT;
        input CFGINTERRUPTMSITPHPRESENT;
        input CFGINTERRUPTMSITPHSTTAG[8];
        input CFGINTERRUPTMSITPHTYPE[2];
        input CFGINTERRUPTMSIXADDRESS[64];
        input CFGINTERRUPTMSIXDATA[32];
        output CFGINTERRUPTMSIXENABLE[4];
        input CFGINTERRUPTMSIXINT;
        output CFGINTERRUPTMSIXMASK[4];
        output CFGINTERRUPTMSIXVECPENDINGSTATUS;
        input CFGINTERRUPTMSIXVECPENDING[2];
        input CFGINTERRUPTPENDING[4];
        output CFGINTERRUPTSENT;
        output CFGLINKPOWERSTATE[2];
        input CFGLINKTRAININGENABLE;
        output CFGLOCALERROROUT[5];
        output CFGLOCALERRORVALID;
        output CFGLTSSMSTATE[6];
        output CFGMAXPAYLOAD[2];
        output CFGMAXREADREQ[3];
        input CFGMGMTADDR[10];
        input CFGMGMTBYTEENABLE[4];
        input CFGMGMTDEBUGACCESS;
        input CFGMGMTFUNCTIONNUMBER[8];
        input CFGMGMTREAD;
        output CFGMGMTREADDATA[32];
        output CFGMGMTREADWRITEDONE;
        input CFGMGMTWRITE;
        input CFGMGMTWRITEDATA[32];
        output CFGMSGRECEIVED;
        output CFGMSGRECEIVEDDATA[8];
        output CFGMSGRECEIVEDTYPE[5];
        input CFGMSGTRANSMIT;
        input CFGMSGTRANSMITDATA[32];
        output CFGMSGTRANSMITDONE;
        input CFGMSGTRANSMITTYPE[3];
        output CFGMSIXRAMADDRESS[13];
        input CFGMSIXRAMERRCOR;
        input CFGMSIXRAMERRUNCOR;
        input CFGMSIXRAMREADDATA[36];
        output CFGMSIXRAMREADENABLE;
        output CFGMSIXRAMWRITEBYTEENABLE[4];
        output CFGMSIXRAMWRITEDATA[36];
        output CFGNEGOTIATEDWIDTH[3];
        output CFGPASIDENABLE[4];
        output CFGPASIDEXECPERMISSIONENABLE[4];
        output CFGPASIDPRIVILMODEENABLE[4];
        output CFGPHYLINKDOWN;
        output CFGPHYLINKSTATUS[2];
        output CFGPLSTATUSCHANGE;
        input CFGPMASPML1ENTRYREJECT;
        input CFGPMASPMTXL0SENTRYDISABLE;
        input CFGPOWERSTATECHANGEACK;
        output CFGPOWERSTATECHANGEINTERRUPT;
        output CFGRCBSTATUS[4];
        input CFGREQPMTRANSITIONL23READY;
        input CFGREVIDPF0[8];
        input CFGREVIDPF1[8];
        input CFGREVIDPF2[8];
        input CFGREVIDPF3[8];
        output CFGRXPMSTATE[2];
        input CFGSUBSYSIDPF0[16];
        input CFGSUBSYSIDPF1[16];
        input CFGSUBSYSIDPF2[16];
        input CFGSUBSYSIDPF3[16];
        input CFGSUBSYSVENDID[16];
        output CFGTPHRAMADDRESS[12];
        input CFGTPHRAMERRCOR;
        input CFGTPHRAMERRUNCOR;
        input CFGTPHRAMREADDATA[36];
        output CFGTPHRAMREADENABLE;
        output CFGTPHRAMWRITEBYTEENABLE[4];
        output CFGTPHRAMWRITEDATA[36];
        output CFGTPHREQUESTERENABLE[4];
        output CFGTPHSTMODE[12];
        output CFGTXPMSTATE[2];
        output CFGVC1ENABLE;
        output CFGVC1NEGOTIATIONPENDING;
        input CFGVENDID[16];
        input CFGVFFLRDONE;
        input CFGVFFLRFUNCNUM[8];

        output DBGCTRL0OUT[32];
        output DBGCTRL1OUT[32];
        output DBGDATA0OUT[256];
        output DBGDATA1OUT[256];
        input DBGSEL0[6];
        input DBGSEL1[6];

        input MGMTRESETN;
        input MGMTSTICKYRESETN;

        input CORECLKMIREPLAYRAM0;
        input CORECLKMIREPLAYRAM1;
        output MIREPLAYRAMADDRESS0[10];
        output MIREPLAYRAMADDRESS1[10];
        input MIREPLAYRAMERRCOR[12];
        input MIREPLAYRAMERRUNCOR[12];
        input MIREPLAYRAMREADDATA0[128];
        input MIREPLAYRAMREADDATA1[128];
        output MIREPLAYRAMREADENABLE0;
        output MIREPLAYRAMREADENABLE1;
        output MIREPLAYRAMWRITEDATA0[128];
        output MIREPLAYRAMWRITEDATA1[128];
        output MIREPLAYRAMWRITEENABLE0;
        output MIREPLAYRAMWRITEENABLE1;

        input CORECLKMIRXCOMPLETIONRAM0;
        input CORECLKMIRXCOMPLETIONRAM1;
        input MIRXCOMPLETIONRAMERRCOR[24];
        input MIRXCOMPLETIONRAMERRUNCOR[24];
        output MIRXCOMPLETIONRAMREADADDRESS0[10];
        output MIRXCOMPLETIONRAMREADADDRESS1[10];
        input MIRXCOMPLETIONRAMREADDATA0[144];
        input MIRXCOMPLETIONRAMREADDATA1[144];
        output MIRXCOMPLETIONRAMREADENABLE0[2];
        output MIRXCOMPLETIONRAMREADENABLE1[2];
        output MIRXCOMPLETIONRAMWRITEADDRESS0[10];
        output MIRXCOMPLETIONRAMWRITEADDRESS1[10];
        output MIRXCOMPLETIONRAMWRITEDATA0[144];
        output MIRXCOMPLETIONRAMWRITEDATA1[144];
        output MIRXCOMPLETIONRAMWRITEENABLE0[2];
        output MIRXCOMPLETIONRAMWRITEENABLE1[2];

        input CORECLKMIRXPOSTEDREQUESTRAM0;
        input CORECLKMIRXPOSTEDREQUESTRAM1;
        input MIRXPOSTEDREQUESTRAMERRCOR[12];
        input MIRXPOSTEDREQUESTRAMERRUNCOR[12];
        output MIRXPOSTEDREQUESTRAMREADADDRESS0[10];
        output MIRXPOSTEDREQUESTRAMREADADDRESS1[10];
        input MIRXPOSTEDREQUESTRAMREADDATA0[144];
        input MIRXPOSTEDREQUESTRAMREADDATA1[144];
        output MIRXPOSTEDREQUESTRAMREADENABLE0;
        output MIRXPOSTEDREQUESTRAMREADENABLE1;
        output MIRXPOSTEDREQUESTRAMWRITEADDRESS0[10];
        output MIRXPOSTEDREQUESTRAMWRITEADDRESS1[10];
        output MIRXPOSTEDREQUESTRAMWRITEDATA0[144];
        output MIRXPOSTEDREQUESTRAMWRITEDATA1[144];
        output MIRXPOSTEDREQUESTRAMWRITEENABLE0;
        output MIRXPOSTEDREQUESTRAMWRITEENABLE1;

        input PCIECOMPLDELIVEREDTAG0[8];
        input PCIECOMPLDELIVEREDTAG1[8];
        input PCIECOMPLDELIVERED[2];
        output PCIECQNPREQCOUNT[6];
        input PCIECQNPREQ[2];
        input PCIECQNPUSERCREDITRCVD;
        input PCIECQPIPELINEEMPTY;
        input PCIEPOSTEDREQDELIVERED;
        output PCIERQSEQNUM0[6];
        output PCIERQSEQNUM1[6];
        output PCIERQSEQNUMVLD0;
        output PCIERQSEQNUMVLD1;
        output PCIERQTAG0[10];
        output PCIERQTAG1[10];
        output PCIERQTAGAV[4];
        output PCIERQTAGVLD0;
        output PCIERQTAGVLD1;
        output PCIETFCNPDAV[4];
        output PCIETFCNPHAV[4];

        input PIPECLK;
        input PIPECLKEN;
        input PIPERESETN;
        input PIPEEQFS[6];
        input PIPEEQLF[6];
        output PIPELTSSM[6];

        for i in 0..16 {
            input "PIPERX{i:02}CHARISK"[2];
            input "PIPERX{i:02}DATAVALID";
            input "PIPERX{i:02}DATA"[32];
            input "PIPERX{i:02}ELECIDLE";
            output "PIPERX{i:02}EQCONTROL"[2];
            input "PIPERX{i:02}EQDONE";
            input "PIPERX{i:02}EQLPADAPTDONE";
            input "PIPERX{i:02}EQLPLFFSSEL";
            input "PIPERX{i:02}EQLPNEWTXCOEFFORPRESET"[18];
            output "PIPERX{i:02}EQPRESETREJECT";
            input "PIPERX{i:02}PHYSTATUS";
            output "PIPERX{i:02}POLARITY";
            input "PIPERX{i:02}STARTBLOCK"[2];
            input "PIPERX{i:02}STATUS"[3];
            input "PIPERX{i:02}SYNCHEADER"[2];
            input "PIPERX{i:02}VALID";
        }

        output PIPERXEQLPLFFS[6];
        output PIPERXEQLPTXPRESET[4];
        for i in 0..4 {
            input "PIPERXMARGIN{i}REQACK";
            output "PIPERXMARGIN{i}REQCMD"[4];
            output "PIPERXMARGIN{i}REQLANENUM"[2];
            output "PIPERXMARGIN{i}REQPAYLOAD"[8];
            output "PIPERXMARGIN{i}REQREQ";
            output "PIPERXMARGIN{i}RESACK";
            input "PIPERXMARGIN{i}RESCMD"[4];
            input "PIPERXMARGIN{i}RESLANENUM"[2];
            input "PIPERXMARGIN{i}RESPAYLOAD"[8];
            input "PIPERXMARGIN{i}RESREQ";
        }
        output PIPERXTERMINATION;

        for i in 0..16 {
            output "PIPETX{i:02}CHARISK"[2];
            output "PIPETX{i:02}COMPLIANCE";
            output "PIPETX{i:02}DATAVALID";
            output "PIPETX{i:02}DATA"[32];
            output "PIPETX{i:02}ELECIDLE";
            input "PIPETX{i:02}EQCOEFF"[18];
            output "PIPETX{i:02}EQCONTROL"[2];
            output "PIPETX{i:02}EQDEEMPH"[6];
            input "PIPETX{i:02}EQDONE";
            output "PIPETX{i:02}POWERDOWN"[2];
            output "PIPETX{i:02}STARTBLOCK";
            output "PIPETX{i:02}SYNCHEADER"[2];
        }

        output PIPETXDEEMPH;
        output PIPETXMARGIN[3];
        output PIPETXRATE[3];
        output PIPETXRCVRDET;
        output PIPETXRESET;
        output PIPETXSWING;

        output PLEQINPROGRESS;
        output PLEQPHASE[2];
        input PLEQRESETEIEOSCOUNT;
        input PLGEN2UPSTREAMPREFERDEEMPH;
        output PLGEN34EQMISMATCH;
        input PLGEN34REDOEQSPEED;
        input PLGEN34REDOEQUALIZATION;
        output PLRCVDEIOSANYLANE;

        input USERCLK;
        input USERCLK2;
        input USERCLKEN;
        input USERSPAREIN[32];
        output USERSPAREOUT[32];

        input CSSDTRIGINNEXT[6];
        input SCANCLKNEXT;
        input SCANMODERSTNEXT;
        input SCANENNEXT;
        input SCANEDTUPDTNEXT;
        input SCANCHNLINEXT[8];
        output SCANCHNLOUTEXT[8];
        input SCANCNTRLCHNLINEXT;
        output SCANCNTRLCHNLOUTEXT;
    }

    bel_class PCIE5 {
        input APB3CLK;
        input APB3PRESETN;
        input APB3PENABLE;
        input APB3PWRITE;
        input APB3PSEL;
        input APB3PADDR[9];
        input APB3PWDATA[32];
        output APB3PREADY;
        output APB3PRDATA[32];
        output APB3PSLVERR;

        input CORECLK;
        input RESETN;

        output MAXISCCIXRXTVALID;
        output MAXISCCIXRXTUSER[47];

        input MAXISCQTREADY[22];
        output MAXISCQTVALID;
        output MAXISCQTDATA[512];
        output MAXISCQTKEEP[16];
        output MAXISCQTLAST;
        output MAXISCQTUSER[231];

        input MAXISRCTREADY[22];
        output MAXISRCTVALID;
        output MAXISRCTDATA[512];
        output MAXISRCTKEEP[16];
        output MAXISRCTLAST;
        output MAXISRCTUSER[161];

        input SAXISCCIXTXTVALID;
        input SAXISCCIXTXTDATA[256];
        input SAXISCCIXTXTUSER[47];

        output SAXISCCTREADY[4];
        input SAXISCCTVALID;
        input SAXISCCTDATA[512];
        input SAXISCCTKEEP[16];
        input SAXISCCTLAST;
        input SAXISCCTUSER[81];

        output SAXISRQTREADY[4];
        input SAXISRQTVALID;
        input SAXISRQTDATA[512];
        input SAXISRQTKEEP[16];
        input SAXISRQTLAST;
        input SAXISRQTUSER[183];

        input AXIUSERIN[16];
        output AXIUSEROUT[16];

        input CCIXOPTIMIZEDTLPTXANDRXENABLE;
        input CCIXRXCORRECTABLEERRORDETECTED;
        input CCIXRXFIFOOVERFLOW;
        input CCIXRXTLPFORWARDED0;
        input CCIXRXTLPFORWARDED1;
        input CCIXRXTLPFORWARDEDLENGTH0[6];
        input CCIXRXTLPFORWARDEDLENGTH1[6];
        input CCIXRXUNCORRECTABLEERRORDETECTED;
        output CCIXTXACTIVEACK;
        input CCIXTXACTIVEREQ;
        output CCIXTXCREDITGRANT;
        input CCIXTXCREDITRETURN;
        output CCIXTXDEACTHINT;

        output CFG10BTAGREQUESTERENABLE;
        output CFGATOMICREQUESTERENABLE;
        output CFGBUSNUMBER[8];
        input CFGCONFIGSPACEENABLE;
        output CFGCURRENTSPEED[3];
        input CFGDEVIDPF0[16];
        input CFGDEVIDPF1[16];
        input CFGDEVIDPF2[16];
        input CFGDEVIDPF3[16];
        input CFGDSN[64];
        input CFGDSPORTNUMBER[8];
        input CFGERRCORIN;
        output CFGERRCOROUT;
        output CFGERRFATALOUT;
        output CFGERRNONFATALOUT;
        input CFGERRUNCORIN;
        output CFGEXTFUNCTIONNUMBER[16];
        input CFGEXTREADDATAVALID;
        input CFGEXTREADDATA[32];
        output CFGEXTREADRECEIVED;
        output CFGEXTREGISTERNUMBER[10];
        output CFGEXTTAGENABLE;
        output CFGEXTWRITEBYTEENABLE[4];
        output CFGEXTWRITEDATA[32];
        output CFGEXTWRITERECEIVED;
        output CFGFCCPLDSCALE[2];
        output CFGFCCPLD[12];
        output CFGFCCPLHSCALE[2];
        output CFGFCCPLH[8];
        output CFGFCNPDSCALE[2];
        output CFGFCNPD[12];
        output CFGFCNPHSCALE[2];
        output CFGFCNPH[8];
        output CFGFCPDSCALE[2];
        output CFGFCPD[12];
        output CFGFCPHSCALE[2];
        output CFGFCPH[8];
        input CFGFCSEL[3];
        input CFGFCVCSEL;
        input CFGFLRDONE;
        input CFGFLRDONEFUNCNUM[16];
        output CFGFUNCTIONPOWERSTATE[3];
        output CFGFUNCTIONSTATUS[4];
        input CFGHOTRESETIN;
        output CFGHOTRESETOUT;
        input CFGINTERRUPTINT[4];
        input CFGINTERRUPTMSIATTR[3];
        output CFGINTERRUPTMSIDATA[32];
        output CFGINTERRUPTMSIENABLE;
        output CFGINTERRUPTMSIFAIL;
        input CFGINTERRUPTMSIFUNCTIONNUMBER[16];
        input CFGINTERRUPTMSIINT[32];
        output CFGINTERRUPTMSIMASKUPDATE;
        output CFGINTERRUPTMSIMMENABLE[3];
        input CFGINTERRUPTMSIPENDINGSTATUSDATAENABLE;
        input CFGINTERRUPTMSIPENDINGSTATUSFUNCTIONNUM[3];
        input CFGINTERRUPTMSIPENDINGSTATUS[32];
        input CFGINTERRUPTMSISELECT[3];
        output CFGINTERRUPTMSISENT;
        input CFGINTERRUPTMSITPHPRESENT;
        input CFGINTERRUPTMSITPHSTTAG[8];
        input CFGINTERRUPTMSITPHTYPE[2];
        input CFGINTERRUPTMSIXADDRESS[64];
        input CFGINTERRUPTMSIXDATA[32];
        output CFGINTERRUPTMSIXENABLE;
        input CFGINTERRUPTMSIXINT;
        output CFGINTERRUPTMSIXMASK;
        output CFGINTERRUPTMSIXVECPENDINGSTATUS;
        input CFGINTERRUPTMSIXVECPENDING[2];
        input CFGINTERRUPTPENDING[8];
        output CFGINTERRUPTSENT;
        output CFGLINKPOWERSTATE[2];
        input CFGLINKTRAININGENABLE;
        output CFGLOCALERROROUT[5];
        output CFGLOCALERRORVALID;
        output CFGLTSSMSTATE[6];
        output CFGMAXPAYLOAD[2];
        output CFGMAXREADREQ[3];
        input CFGMGMTADDR[10];
        input CFGMGMTBYTEENABLE[4];
        input CFGMGMTDEBUGACCESS;
        input CFGMGMTFUNCTIONNUMBER[16];
        input CFGMGMTREAD;
        output CFGMGMTREADDATA[32];
        output CFGMGMTREADWRITEDONE;
        input CFGMGMTWRITE;
        input CFGMGMTWRITEDATA[32];
        output CFGMSGRECEIVED;
        output CFGMSGRECEIVEDDATA[8];
        output CFGMSGRECEIVEDTYPE[5];
        input CFGMSGTRANSMIT;
        input CFGMSGTRANSMITDATA[32];
        output CFGMSGTRANSMITDONE;
        input CFGMSGTRANSMITTYPE[3];
        output CFGMSIXRAMADDRESS[16];
        input CFGMSIXRAMERRCOR;
        input CFGMSIXRAMERRUNCOR;
        input CFGMSIXRAMREADDATA[64];
        output CFGMSIXRAMREADENABLE;
        output CFGMSIXRAMWRITEBYTEENABLE[8];
        output CFGMSIXRAMWRITEDATA[64];
        output CFGMSIXRAMWRITEENABLE;
        output CFGNEGOTIATEDWIDTH[3];
        output CFGPASIDENABLE;
        output CFGPASIDEXECPERMISSIONENABLE;
        output CFGPASIDPRIVILMODEENABLE;
        input CFGPERFUNCFUNCNUM[16];
        output CFGPERFUNCOUT[24];
        input CFGPERFUNCREQ;
        output CFGPERFUNCVLD;
        output CFGPHYLINKDOWN;
        output CFGPHYLINKSTATUS[2];
        output CFGPLSTATUSCHANGE;
        input CFGPMASPML1ENTRYREJECT;
        input CFGPMASPMTXL0SENTRYDISABLE;
        input CFGPOWERSTATECHANGEACK;
        output CFGPOWERSTATECHANGEINTERRUPT;
        output CFGRCBSTATUS;
        input CFGREQPMTRANSITIONL23READY;
        input CFGREVIDPF0[8];
        input CFGREVIDPF1[8];
        input CFGREVIDPF2[8];
        input CFGREVIDPF3[8];
        output CFGRXPMSTATE[2];
        input CFGSUBSYSIDPF0[16];
        input CFGSUBSYSIDPF1[16];
        input CFGSUBSYSIDPF2[16];
        input CFGSUBSYSIDPF3[16];
        input CFGSUBSYSVENDID[16];
        output CFGTXPMSTATE[2];
        output CFGVC1ENABLE;
        output CFGVC1NEGOTIATIONPENDING;
        input CFGVENDID[16];
        output CFGWRREQBMEVLD;
        output CFGWRREQFLRVLD;
        output CFGWRREQFUNCNUM[16];
        output CFGWRREQMSIVLD;
        output CFGWRREQMSIXVLD;
        output CFGWRREQOUTVALUE[4];
        output CFGWRREQVFEVLD;

        output DBGCTRL0OUT[32];
        output DBGCTRL1OUT[32];
        output DBGDATA0OUT[256];
        output DBGDATA1OUT[256];
        input DBGSEL0[6];
        input DBGSEL1[6];

        input MGMTRESETN;
        input MGMTSTICKYRESETN;

        input CORECLKMIREPLAYRAM0;
        input CORECLKMIREPLAYRAM1;
        output MIREPLAYRAMADDRESS0[10];
        output MIREPLAYRAMADDRESS1[10];
        input MIREPLAYRAMERRCOR[12];
        input MIREPLAYRAMERRUNCOR[12];
        input MIREPLAYRAMREADDATA0[128];
        input MIREPLAYRAMREADDATA1[128];
        output MIREPLAYRAMREADENABLE0;
        output MIREPLAYRAMREADENABLE1;
        output MIREPLAYRAMWRITEDATA0[128];
        output MIREPLAYRAMWRITEDATA1[128];
        output MIREPLAYRAMWRITEENABLE0;
        output MIREPLAYRAMWRITEENABLE1;

        input CORECLKMIRXCOMPLETIONRAM0;
        input CORECLKMIRXCOMPLETIONRAM1;
        input MIRXCOMPLETIONRAMERRCOR[24];
        input MIRXCOMPLETIONRAMERRUNCOR[24];
        output MIRXCOMPLETIONRAMREADADDRESS0[10];
        output MIRXCOMPLETIONRAMREADADDRESS1[10];
        input MIRXCOMPLETIONRAMREADDATA0[144];
        input MIRXCOMPLETIONRAMREADDATA1[144];
        output MIRXCOMPLETIONRAMREADENABLE0[2];
        output MIRXCOMPLETIONRAMREADENABLE1[2];
        output MIRXCOMPLETIONRAMWRITEADDRESS0[10];
        output MIRXCOMPLETIONRAMWRITEADDRESS1[10];
        output MIRXCOMPLETIONRAMWRITEDATA0[144];
        output MIRXCOMPLETIONRAMWRITEDATA1[144];
        output MIRXCOMPLETIONRAMWRITEENABLE0[2];
        output MIRXCOMPLETIONRAMWRITEENABLE1[2];

        input CORECLKMIRXPOSTEDREQUESTRAM0;
        input CORECLKMIRXPOSTEDREQUESTRAM1;
        input MIRXPOSTEDREQUESTRAMERRCOR[12];
        input MIRXPOSTEDREQUESTRAMERRUNCOR[12];
        output MIRXPOSTEDREQUESTRAMREADADDRESS0[10];
        output MIRXPOSTEDREQUESTRAMREADADDRESS1[10];
        input MIRXPOSTEDREQUESTRAMREADDATA0[144];
        input MIRXPOSTEDREQUESTRAMREADDATA1[144];
        output MIRXPOSTEDREQUESTRAMREADENABLE0;
        output MIRXPOSTEDREQUESTRAMREADENABLE1;
        output MIRXPOSTEDREQUESTRAMWRITEADDRESS0[10];
        output MIRXPOSTEDREQUESTRAMWRITEADDRESS1[10];
        output MIRXPOSTEDREQUESTRAMWRITEDATA0[144];
        output MIRXPOSTEDREQUESTRAMWRITEDATA1[144];
        output MIRXPOSTEDREQUESTRAMWRITEENABLE0;
        output MIRXPOSTEDREQUESTRAMWRITEENABLE1;

        output PCIECQNPREQCOUNT[6];
        input PCIECQNPREQ[2];
        output PCIERQSEQNUM0[6];
        output PCIERQSEQNUM1[6];
        output PCIERQSEQNUMVLD0;
        output PCIERQSEQNUMVLD1;
        output PCIERQTAG0[10];
        output PCIERQTAG1[10];
        output PCIERQTAGAV[4];
        output PCIERQTAGVLD0;
        output PCIERQTAGVLD1;
        output PCIETFCNPDAV[4];
        output PCIETFCNPHAV[4];

        input PIPECLKEN;
        input PIPERESETN;
        input PIPEEQFS[6];
        input PIPEEQLF[6];
        output PIPELTSSM[6];
        for i in 0..16 {
            input "PIPERX{i:02}CHARISK"[2];
            input "PIPERX{i:02}DATAVALID";
            input "PIPERX{i:02}DATA"[32];
            input "PIPERX{i:02}ELECIDLE";
            output "PIPERX{i:02}EQCONTROL"[2];
            input "PIPERX{i:02}EQDONE";
            input "PIPERX{i:02}EQLPADAPTDONE";
            input "PIPERX{i:02}EQLPLFFSSEL";
            input "PIPERX{i:02}EQLPNEWTXCOEFFORPRESET"[18];
            output "PIPERX{i:02}EQPRESETREJECT";
            input "PIPERX{i:02}PHYSTATUS";
            output "PIPERX{i:02}POLARITY";
            input "PIPERX{i:02}SKPEND";
            input "PIPERX{i:02}STARTBLOCK"[2];
            input "PIPERX{i:02}STATUS"[3];
            input "PIPERX{i:02}SYNCHEADER"[2];
            input "PIPERX{i:02}VALID";
        }

        output PIPERXEQLPLFFS[6];
        output PIPERXEQLPTXPRESET[4];
        for i in 0..4 {
            input "PIPERXMARGIN{i}REQACK";
            output "PIPERXMARGIN{i}REQCMD"[4];
            output "PIPERXMARGIN{i}REQLANENUM"[2];
            output "PIPERXMARGIN{i}REQPAYLOAD"[8];
            output "PIPERXMARGIN{i}REQREQ";
            output "PIPERXMARGIN{i}RESACK";
            input "PIPERXMARGIN{i}RESCMD"[4];
            input "PIPERXMARGIN{i}RESLANENUM"[2];
            input "PIPERXMARGIN{i}RESPAYLOAD"[8];
            input "PIPERXMARGIN{i}RESREQ";
        }
        output PIPERXTERMINATION;
        for i in 0..16 {
            output "PIPETX{i:02}CHARISK"[2];
            output "PIPETX{i:02}COMPLIANCE";
            output "PIPETX{i:02}DATAVALID";
            output "PIPETX{i:02}DATA"[32];
            output "PIPETX{i:02}ELECIDLE";
            input "PIPETX{i:02}EQCOEFF"[18];
            output "PIPETX{i:02}EQCONTROL"[2];
            output "PIPETX{i:02}EQDEEMPH"[6];
            input "PIPETX{i:02}EQDONE";
            output "PIPETX{i:02}POWERDOWN"[2];
            output "PIPETX{i:02}RCVRDET";
            output "PIPETX{i:02}STARTBLOCK";
            output "PIPETX{i:02}SYNCHEADER"[2];
        }
        output PIPETXDEEMPH;
        output PIPETXMARGIN[3];
        output PIPETXRATE[3];
        output PIPETXRESET;
        output PIPETXSWING;

        output PLEQINPROGRESS;
        output PLEQPHASE[2];
        input PLEQRESETEIEOSCOUNT;
        input PLGEN2UPSTREAMPREFERDEEMPH;
        output PLGEN34EQMISMATCH;
        input PLGEN34REDOEQSPEED;
        input PLGEN34REDOEQUALIZATION;
        output PLRCVDEIOSANYLANE;

        input USERCLKEN;
        input USERSPAREIN[32];
        output USERSPAREOUT[32];

        input CSSDTRIGINNEXT[6];
        input SCANCLKNEXT;
        input SCANMODERSTNEXT;
        input SCANMODERSTAPBN;
        input SCANMODEAPBN;
        input SCANRSTBYPAPBN;
        input SCANENNEXT;
        input SCANENAPBN;
        input SCANEDTUPDTNEXT;
        input SCANCHNLINEXT[9];
        output SCANCHNLOUTEXT[9];
        input SCANCNTRLCHNLINEXT;
        output SCANCNTRLCHNLOUTEXT;
        input SCANINAPB;
        output SCANOUTAPB;
    }

    bel_class MRMAC {
        input APB3_CLK;
        input APB3_PRESET;
        input APB3_PENABLE;
        input APB3_PWRITE;
        input APB3_PSEL;
        input APB3_PADDR[16];
        input APB3_PWDATA[32];
        output APB3_PREADY;
        output APB3_PRDATA[32];
        output APB3_PSLVERR;

        for i in 0..4 {
            input "CTL_RX_FEC_FC32_RA_MODE_{i}";
            input "CTL_RX_PAUSE_ACK_{i}"[9];
            input "CTL_RX_PAUSE_ENABLE_{i}"[9];
            input "CTL_RX_PTP_ST_ADJUST_{i}"[32];
            input "CTL_RX_PTP_ST_ADJUST_TYPE_{i}"[2];
            input "CTL_RX_PTP_ST_ADJUST_VLD_{i}";
            input "CTL_RX_PTP_ST_OVERWRITE_{i}";
            input "CTL_RX_PTP_ST_SYNC_{i}";
            input "CTL_RX_PTP_SYSTEMTIMER_{i}"[55];
            input "CTL_TX_PAUSE_ENABLE_{i}"[9];
            input "CTL_TX_PAUSE_REQ_{i}"[9];
            input "CTL_TX_PTP_ST_ADJUST_{i}"[32];
            input "CTL_TX_PTP_ST_ADJUST_TYPE_{i}"[2];
            input "CTL_TX_PTP_ST_ADJUST_VLD_{i}";
            input "CTL_TX_PTP_ST_OVERWRITE_{i}";
            input "CTL_TX_PTP_ST_SYNC_{i}";
            input "CTL_TX_PTP_SYSTEMTIMER_{i}"[55];
            input "CTL_TX_RESEND_PAUSE_{i}";
            input "CTL_TX_SEND_IDLE_IN_{i}";
            input "CTL_TX_SEND_LFI_IN_{i}";
            input "CTL_TX_SEND_RFI_IN_{i}";
        }
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE_01;
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE_23;
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE_VALUE_01[8];
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE_VALUE_23[8];
        input CTL_RSVD_IN[128];

        output EXT_COUNT_ADDR[9];
        output EXT_COUNT_FLAGS[5];
        output EXT_COUNT_INC;

        for i in 0..4 {
            input "FEC_RX_DIN_START_{i}";
            output "FEC_RX_DOUT_FLAGS_{i}"[4];
            output "FEC_RX_DOUT_IS_AM_{i}";
            output "FEC_RX_DOUT_START_{i}";
            output "FEC_RX_DOUT_VALID_{i}";
            input "FEC_TX_DIN_IS_AM_{i}";
            input "FEC_TX_DIN_START_{i}";
            input "FEC_TX_DIN_VALID_{i}";
            output "FEC_TX_DOUT_START_{i}";
        }

        output PM_RDY[4];
        input PM_TICK[4];

        input RX_CORE_CLK[4];
        input RX_CORE_RESET[4];

        input RX_AXI_CLK[4];
        for i in 0..4 {
            output "RX_AXIS_TVALID_{i}";
            output "RX_AXIS_TLAST_{i}";
        }
        for i in 0..8 {
            output "RX_AXIS_TDATA{i}"[64];
            output "RX_AXIS_TKEEP_USER{i}"[11];
        }

        input RX_FLEXIF_CLK[4];
        input RX_FLEXIF_RESET[4];
        for i in 0..4 {
            input "RX_FLEX_CM_ENA_{i}";
            output "RX_FLEX_CM_STALLOUT_{i}";
            output "RX_FLEX_ENA_{i}";
        }
        for i in 0..8 {
            output "RX_FLEX_ALMARKER{i}";
            output "RX_FLEX_BIP8{i}"[8];
            input "RX_FLEX_CM_DATA{i}"[66];
            output "RX_FLEX_DATA{i}"[66];
        }
        output RX_FLEX_LANE0;
        for i in 0..20 {
            output "RX_LANE_ALIGNER_FILL_{i}"[8];
        }

        for i in 0..4 {
            output "RX_PREAMBLEOUT_{i}"[56];
            output "RX_PTP_RSFEC_OFFSET_OUT_{i}"[7];
            output "RX_PTP_TSTAMP_OUT_{i}"[55];
        }

        input RX_SERDES_CLK[4];
        input RX_SERDES_RESET[4];
        input RX_ALT_SERDES_CLK[4];
        for i in 0..4 {
            input "RX_SERDES_DATA{i}"[80];
        }
        input RX_TS_CLK[4];

        output STAT_RSVD_OUT[217];
        output STAT_RX01_FEC_DEGRADED_SER;
        output STAT_RX23_FEC_DEGRADED_SER;
        output STAT_RX_ALIGNED_0;
        output STAT_RX_ALIGNED_2;
        output STAT_RX_ALIGNED_ERR_0;
        output STAT_RX_ALIGNED_ERR_2;
        for i in 0..4 {
            output "STAT_RX_AXIS_ERR_{i}";
            output "STAT_RX_AXIS_FIFO_OVERFLOW_{i}";
            output "STAT_RX_BAD_CODE_{i}";
            output "STAT_RX_BAD_FCS_{i}";
            output "STAT_RX_BAD_PREAMBLE_{i}";
            output "STAT_RX_BAD_SFD_{i}";
            output "STAT_RX_CL49_82_CONVERT_ERR_{i}";
            output "STAT_RX_ECC_ERR_{i}"[2];
            output "STAT_RX_FEC_ALIGNED_{i}";
            output "STAT_RX_FEC_DELAY_{i}"[15];
            output "STAT_RX_FEC_HI_SER_{i}";
            output "STAT_RX_FEC_MAPPING_{i}"[2];
            output "STAT_RX_FLEXIF_ERR_{i}";
            output "STAT_RX_FLEX_FIFO_OVF_{i}";
            output "STAT_RX_FLEX_FIFO_UDF_{i}";
            output "STAT_RX_FLEX_MON_FIFO_OVF_{i}";
            output "STAT_RX_FLEX_MON_FIFO_UDF_{i}";
            output "STAT_RX_GOT_SIGNAL_OS_{i}";
            output "STAT_RX_HI_BER_{i}";
            output "STAT_RX_INTERNAL_LOCAL_FAULT_{i}";
            output "STAT_RX_INVALID_START_{i}";
            output "STAT_RX_LOCAL_FAULT_{i}";
            for j in 0..9 {
                output "STAT_RX_PAUSE_QUANTA{j}_{i}"[16];
            }
            output "STAT_RX_PAUSE_REQ_{i}"[9];
            output "STAT_RX_PAUSE_VALID_{i}"[9];
            output "STAT_RX_PCS_BAD_CODE_{i}";
            output "STAT_RX_PTP_ST_SYNC_{i}";
            output "STAT_RX_PTP_SYSTEMTIMER_{i}"[55];
            output "STAT_RX_RECEIVED_LOCAL_FAULT_{i}";
            output "STAT_RX_REMOTE_FAULT_{i}";
            output "STAT_RX_STATUS_{i}";
            output "STAT_RX_TEST_PATTERN_MISMATCH_{i}";
            output "STAT_RX_TRUNCATED_{i}";
            output "STAT_RX_TSN_FRAGMENT_{i}";
            output "STAT_RX_TSN_PREEMPTED_PKT_{i}";
            output "STAT_RX_VALID_CTRL_CODE_{i}";
        }
        output STAT_RX_BIP_ERR_0[20];
        output STAT_RX_BIP_ERR_2[4];
        output STAT_RX_BLOCK_LOCK_0[20];
        output STAT_RX_BLOCK_LOCK_1;
        output STAT_RX_BLOCK_LOCK_2[4];
        output STAT_RX_BLOCK_LOCK_3;
        output STAT_RX_FEC_CORRECTED_CW_0_0;
        output STAT_RX_FEC_CORRECTED_CW_0_1;
        output STAT_RX_FEC_CORRECTED_CW_0_2;
        output STAT_RX_FEC_CORRECTED_CW_0_3;
        output STAT_RX_FEC_CORRECTED_CW_1;
        output STAT_RX_FEC_CORRECTED_CW_2_0;
        output STAT_RX_FEC_CORRECTED_CW_2_1;
        output STAT_RX_FEC_CORRECTED_CW_2_2;
        output STAT_RX_FEC_CORRECTED_CW_2_3;
        output STAT_RX_FEC_CORRECTED_CW_3;
        output STAT_RX_FEC_CW_0_0;
        output STAT_RX_FEC_CW_0_1;
        output STAT_RX_FEC_CW_0_2;
        output STAT_RX_FEC_CW_0_3;
        output STAT_RX_FEC_CW_1;
        output STAT_RX_FEC_CW_2_0;
        output STAT_RX_FEC_CW_2_1;
        output STAT_RX_FEC_CW_2_2;
        output STAT_RX_FEC_CW_2_3;
        output STAT_RX_FEC_CW_3;
        output STAT_RX_FEC_ERR_COUNT_0_0[4];
        output STAT_RX_FEC_ERR_COUNT_0_1[4];
        output STAT_RX_FEC_ERR_COUNT_0_2[4];
        output STAT_RX_FEC_ERR_COUNT_0_3[4];
        output STAT_RX_FEC_ERR_COUNT_1[4];
        output STAT_RX_FEC_ERR_COUNT_2_0[4];
        output STAT_RX_FEC_ERR_COUNT_2_1[4];
        output STAT_RX_FEC_ERR_COUNT_3[4];
        output STAT_RX_FEC_LANE_LOCK_0[4];
        output STAT_RX_FEC_LANE_LOCK_1;
        output STAT_RX_FEC_LANE_LOCK_2[4];
        output STAT_RX_FEC_LANE_LOCK_3;
        output STAT_RX_FEC_UNCORRECTED_CW_0_0;
        output STAT_RX_FEC_UNCORRECTED_CW_0_1;
        output STAT_RX_FEC_UNCORRECTED_CW_0_2;
        output STAT_RX_FEC_UNCORRECTED_CW_0_3;
        output STAT_RX_FEC_UNCORRECTED_CW_1;
        output STAT_RX_FEC_UNCORRECTED_CW_2_0;
        output STAT_RX_FEC_UNCORRECTED_CW_2_1;
        output STAT_RX_FEC_UNCORRECTED_CW_2_2;
        output STAT_RX_FEC_UNCORRECTED_CW_2_3;
        output STAT_RX_FEC_UNCORRECTED_CW_3;
        output STAT_RX_FRAMING_ERR_0[20];
        output STAT_RX_FRAMING_ERR_1;
        output STAT_RX_FRAMING_ERR_2[4];
        output STAT_RX_FRAMING_ERR_3;
        output STAT_RX_LANE0_VLM_BIP7_0[8];
        output STAT_RX_LANE0_VLM_BIP7_2[8];
        output STAT_RX_LANE0_VLM_BIP7_VALID_0;
        output STAT_RX_LANE0_VLM_BIP7_VALID_2;
        output STAT_RX_MF_ERR_0[20];
        output STAT_RX_MF_ERR_2[4];
        output STAT_RX_MISALIGNED_0;
        output STAT_RX_MISALIGNED_2;
        output STAT_RX_SYNCED_0[20];
        output STAT_RX_SYNCED_2[4];
        output STAT_RX_SYNCED_ERR_0[20];
        output STAT_RX_SYNCED_ERR_2[4];
        output STAT_RX_VL_DEMUXED_0;
        output STAT_RX_VL_DEMUXED_2;

        input TX_CORE_CLK[4];
        input TX_CORE_RESET[4];

        input TX_AXI_CLK[4];
        for i in 0..4 {
            output "TX_AXIS_TREADY_{i}";
            input "TX_AXIS_TVALID_{i}";
            input "TX_AXIS_TLAST_{i}";
        }
        for i in 0..8 {
            input "TX_AXIS_TDATA{i}"[64];
            input "TX_AXIS_TKEEP_USER{i}"[11];
        }

        input TX_FLEXIF_CLK[4];
        for i in 0..8 {
            input "TX_FLEX_DATA{i}"[66];
            input "TX_FLEX_ALMARKER{i}";
        }
        for i in 0..4 {
            input "TX_FLEX_ENA_{i}";
            output "TX_FLEX_STALL_{i}";
            input "TX_PREAMBLEIN_{i}"[56];
            input "TX_PTP_1588OP_IN_{i}"[2];
            input "TX_PTP_CF_OFFSET_IN_{i}"[16];
            input "TX_PTP_FLEX_1588OP_IN_{i}";
            input "TX_PTP_FLEX_TAG_FIELD_IN_{i}"[16];
            output "TX_PTP_RSFEC_OFFSET_OUT_{i}"[7];
            input "TX_PTP_TAG_FIELD_IN_{i}"[16];
            output "TX_PTP_TSTAMP_OUT_{i}"[55];
            output "TX_PTP_TSTAMP_TAG_OUT_{i}"[16];
            output "TX_PTP_TSTAMP_VALID_OUT_{i}";
            input "TX_PTP_UPD_CHKSUM_IN_{i}";
        }
        input TX_PTP_FLEX_1588LOC_IN_0[3];
        input TX_PTP_FLEX_1588LOC_IN_1[2];
        input TX_PTP_FLEX_1588LOC_IN_2[2];
        input TX_PTP_FLEX_1588LOC_IN_3[2];

        input TX_ALT_SERDES_CLK[4];
        for i in 0..4 {
            output "TX_SERDES_DATA{i}"[80];
        }
        output TX_SERDES_IS_AM_0[9];
        output TX_SERDES_IS_AM_1[8];
        output TX_SERDES_IS_AM_2[9];
        output TX_SERDES_IS_AM_3[8];
        input TX_SERDES_RESET[4];
        input TX_TS_CLK[4];

        for i in 0..4 {
            output "STAT_TX_AXIS_ERR_{i}";
            output "STAT_TX_AXIS_UNF_{i}";
            output "STAT_TX_BAD_FCS_{i}";
            output "STAT_TX_CL82_49_CONVERT_ERR_{i}";
            output "STAT_TX_ECC_ERR_{i}"[2];
            output "STAT_TX_FEC_PCS_AM_LOCK_{i}"[5];
            output "STAT_TX_FEC_PCS_BLOCK_LOCK_{i}"[5];
            output "STAT_TX_FEC_PCS_LANE_ALIGN_{i}";
            output "STAT_TX_FLEXIF_ERR_{i}";
            output "STAT_TX_FLEX_FIFO_OVF_{i}";
            output "STAT_TX_FLEX_FIFO_UDF_{i}";
            output "STAT_TX_FRAME_ERROR_{i}";
            output "STAT_TX_LOCAL_FAULT_{i}";
            output "STAT_TX_PAUSE_VALID_{i}"[9];
            output "STAT_TX_PCS_BAD_CODE_{i}"[3];
            output "STAT_TX_PTP_ST_SYNC_{i}";
            output "STAT_TX_PTP_SYSTEMTIMER_{i}"[55];
            output "STAT_TX_TSN_FRAGMENT_{i}";
            output "STAT_TX_TSN_PREEMPTED_PKT_{i}";
        }

        input BISR_W_CLK;
        input BISR_W_DONE;
        output BISR_ACCESS;
        output BISR_ADDRESS[6];
        input BISR_R_MODE;
        output BISR_SELECT;
        output BISR_WRITE;

        input CSSD_TRIG_N[128];

        output MBIST_DONE;
        output MBIST_ERROR;

        input SCAN_CLK_N;
        input SCAN_MODE_RST_N;
        input SCAN_EN_N;
        input SCAN_EDT_UPDT_N;
        input SCAN_CHNL_IN[12];
        output SCAN_CHNL_OUT[12];
        input SCAN_CNTRL_CHNL_IN[2];
        output SCAN_CNTRL_CHNL_OUT[2];
    }

    bel_class SDFEC {
        input LD_CORE_CLK;
        input LD_CORE_ENABLE;
        input LD_RESET_N;
        output LD_INTERRUPT;
        input LD_REG_CACHE_CLR_N;
        input LD_SHUTDOWN;
        input LD_SHUTDOWN_OVERRIDE_N;

        input S_AXIS_LD_CTRL_ACLK;
        output S_AXIS_LD_CTRL_TREADY;
        input S_AXIS_LD_CTRL_TVALID;
        input S_AXIS_LD_CTRL_TDATA[64];
        output S_AXIS_LD_CTRL_CHAR;

        input M_AXIS_LD_STATUS_ACLK;
        input M_AXIS_LD_STATUS_TREADY;
        output M_AXIS_LD_STATUS_TVALID;
        output M_AXIS_LD_STATUS_TDATA[64];
        input M_AXIS_LD_STATUS_CHAR;

        input S_AXIS_LD_DIN_ACLK;
        output S_AXIS_LD_DIN_TREADY;
        input S_AXIS_LD_DIN_TVALID;
        input S_AXIS_LD_DIN_TDATA[512];
        input S_AXIS_LD_DIN_TLAST;
        output S_AXIS_LD_DIN_CHAR;

        input S_AXIS_LD_DIN_WORDS_ACLK;
        output S_AXIS_LD_DIN_WORDS_TREADY;
        input S_AXIS_LD_DIN_WORDS_TVALID;
        input S_AXIS_LD_DIN_WORDS_TDATA[32];
        input S_AXIS_LD_DIN_WORDS_TLAST;
        output S_AXIS_LD_DIN_WORDS_CHAR;

        input M_AXIS_LD_DOUT_ACLK;
        input M_AXIS_LD_DOUT_TREADY;
        output M_AXIS_LD_DOUT_TVALID;
        output M_AXIS_LD_DOUT_TDATA[512];
        output M_AXIS_LD_DOUT_TKEEP[64];
        output M_AXIS_LD_DOUT_TLAST;
        input M_AXIS_LD_DOUT_CHAR;

        input S_AXIS_LD_DOUT_WORDS_ACLK;
        output S_AXIS_LD_DOUT_WORDS_TREADY;
        input S_AXIS_LD_DOUT_WORDS_TVALID;
        input S_AXIS_LD_DOUT_WORDS_TDATA[32];
        input S_AXIS_LD_DOUT_WORDS_TLAST;
        output S_AXIS_LD_DOUT_WORDS_CHAR;

        input S_AXI_LD_ACLK;
        input S_AXI_LD_ARESETN;

        output S_AXI_LD_ARREADY;
        input S_AXI_LD_ARVALID;
        input S_AXI_LD_ARADDR[18];
        output S_AXI_LD_ARCHAR;

        output S_AXI_LD_AWREADY;
        input S_AXI_LD_AWVALID;
        input S_AXI_LD_AWADDR[18];
        output S_AXI_LD_AWCHAR;

        input S_AXI_LD_BREADY;
        output S_AXI_LD_BVALID;

        input S_AXI_LD_RREADY;
        output S_AXI_LD_RVALID;
        output S_AXI_LD_RDATA[32];
        input S_AXI_LD_RCHAR;

        output S_AXI_LD_WREADY;
        input S_AXI_LD_WVALID;
        input S_AXI_LD_WDATA[32];
        output S_AXI_LD_WCHAR;
        input LD_SPARE_IN[64];
        output LD_SPARE_OUT[64];

        input LD_CSSD_TRIG_N;
        input LD_SCAN_CLK_N_IN_EXT;
        input LD_SCAN_MODE_RST_N_IN_EXT;
        input LD_SCAN_EN_N_IN_EXT;
        input LD_SCAN_EDT_UPDT_N_IN_EXT;
        input LD_SCAN_CHNL_IN_EXT[10];
        output LD_SCAN_CHNL_OUT_EXT[10];
        input LD_SCAN_CNTRL_CHNL_IN_EXT;
        output LD_SCAN_CNTRL_CHNL_OUT_EXT;

        input LD_TEST_MODE_PIN_CHAR_N;
    }

    bel_class DFE_CFC {
        for i in 0..4 {
            input "CH{i}_CLK";
            input "CH{i}_CLK_IO"[4];
            input "CH{i}_RESET_N";
            input "CH{i}_ENABLE";

            input "CH{i}_CONTROL0"[22];
            input "CH{i}_CONTROL1"[22];
            input "CH{i}_CONTROL2"[22];
            input "CH{i}_CONTROL3"[22];
            input "CH{i}_CTRL_DBL"[14];
            output "CH{i}_STATUS0"[2];
            output "CH{i}_STATUS1"[2];
            output "CH{i}_STATUS_DBL"[15];
            input "CH{i}_MODE0"[32];
            input "CH{i}_MODE1"[32];

            for j in 0..4 {
                output "CH{i}_S_AXIS_DIN{j}_TREADY";
                input "CH{i}_S_AXIS_DIN{j}_TVALID";
                input "CH{i}_S_AXIS_DIN{j}_TDATA"[72];
                input "CH{i}_S_AXIS_DIN{j}_TID"[8];
                input "CH{i}_S_AXIS_DIN{j}_TKEEP"[2];
                input "CH{i}_S_AXIS_DIN{j}_TLAST";
                input "CH{i}_S_AXIS_DIN{j}_TUSER"[30];
                output "CH{i}_S_AXIS_DIN{j}_CHAR";

                input "CH{i}_S_AXIS_COEFF{j}_TVALID";
                output "CH{i}_S_AXIS_COEFF{j}_TREADY";
                input "CH{i}_S_AXIS_COEFF{j}_TDATA"[32];
                input "CH{i}_S_AXIS_COEFF{j}_TLAST";
                input "CH{i}_S_AXIS_COEFF{j}_TUSER"[16];
                output "CH{i}_S_AXIS_COEFF{j}_CHAR";

                output "CH{i}_M_AXIS_DOUT{j}_TVALID";
                output "CH{i}_M_AXIS_DOUT{j}_TDATA"[72];
                output "CH{i}_M_AXIS_DOUT{j}_TID"[8];
                output "CH{i}_M_AXIS_DOUT{j}_TKEEP"[2];
                output "CH{i}_M_AXIS_DOUT{j}_TUSER"[32];
                input "CH{i}_M_AXIS_DOUT{j}_CHAR";
            }

            output "CH{i}_SPARE_OUT0"[6];
            output "CH{i}_SPARE_OUT1"[6];

            input "CH{i}_CSSD_TRIG_N";
            input "CH{i}_SCAN_CLK_N_IN_EXT";
            input "CH{i}_SCAN_MODE_RST_N_IN_EXT";
            input "CH{i}_SCAN_EN_N_IN_EXT";
            input "CH{i}_SCAN_EDT_UPDT_N_IN_EXT";
            input "CH{i}_SCAN_CHNL_IN_EXT"[10];
            output "CH{i}_SCAN_CHNL_OUT_EXT"[10];
            input "CH{i}_SCAN_CNTRL_CHNL_IN_EXT";
            output "CH{i}_SCAN_CNTRL_CHNL_OUT_EXT";

            input "CH{i}_TEST_MODE_PIN_CHAR_N";
        }

        for i in 0..2 {
            input "FT{i}_CLK";
            input "FT{i}_CLK_IO"[16];
            input "FT{i}_D_CLK";
            input "FT{i}_RESET_N";
            input "FT{i}_ENABLE";
            input "FT{i}_CTRL"[128];
            input "FT{i}_CTRL_DBL"[14];
            input "FT{i}_EMA"[7];
            input "FT{i}_FABRIC_CNTL_IN"[15];
            input "FT{i}_REG_CACHE_CLR_N";
            input "FT{i}_STATIC_CTRL"[8];
            output "FT{i}_STATUS"[10];
            output "FT{i}_STATUS_DBL"[15];
            input "FT{i}_SHUTDOWN_IN";
            output "FT{i}_SHUTDOWN_OUT";

            input "FT{i}_SCAN_CLK_N_IN_EXT";
            input "FT{i}_SCAN_MODE_RST_N_IN_EXT";
            input "FT{i}_SCAN_EN_N_IN_EXT";
            input "FT{i}_SCAN_EDT_UPDT_N_IN_EXT";
            input "FT{i}_SCAN_CHNL_IN_EXT"[2];
            output "FT{i}_SCAN_CHNL_OUT_EXT"[2];
            input "FT{i}_SCAN_CNTRL_CHNL_IN_EXT";
            output "FT{i}_SCAN_CNTRL_CHNL_OUT_EXT";

            input "FT{i}_TEST_BYPASS_N";
            input "FT{i}_TEST_CSSD_TRIG_N";
            input "FT{i}_TEST_PIN_CHAR_IN";
            output "FT{i}_TEST_PIN_CHAR_OUT";

            input "IF_M_AXIS_FT{i}_DOUT_TREADY";
            output "IF_M_AXIS_FT{i}_DOUT_TVALID";
            output "IF_M_AXIS_FT{i}_DOUT_TDATA"[496];
            output "IF_M_AXIS_FT{i}_DOUT_TLAST";
            output "IF_M_AXIS_FT{i}_DOUT_TUSER"[108];

            output "IF_S_AXIS_FT{i}_DIN_TREADY";
            input "IF_S_AXIS_FT{i}_DIN_TVALID";
            input "IF_S_AXIS_FT{i}_DIN_TDATA"[288];
            input "IF_S_AXIS_FT{i}_DIN_TLAST";
            input "IF_S_AXIS_FT{i}_DIN_TUSER"[38];

            output "IF_S_AXIS_FT{i}_PH2_CTRL_TREADY";
            input "IF_S_AXIS_FT{i}_PH2_CTRL_TVALID";
            input "IF_S_AXIS_FT{i}_PH2_CTRL_TDATA"[124];
            input "IF_S_AXIS_FT{i}_PH2_CTRL_TLAST";
        }
    }

    bel_class DCMAC {
        input APB3_CLK;
        input APB3_PRESET;
        input APB3_PENABLE;
        input APB3_PWRITE;
        input APB3_PSEL;
        input APB3_PADDR[18];
        input APB3_PWDATA[32];
        output APB3_PREADY;
        output APB3_PRDATA[32];
        output APB3_PSLVERR;

        for i in 0..6 {
            input "C{i}_CTL_RX_PAUSE_ENABLE"[9];
            input "C{i}_CTL_TX_LANE0_VLM_BIP7_OVERRIDE";
            input "C{i}_CTL_TX_LANE0_VLM_BIP7_OVERRIDE_VALUE"[8];
            input "C{i}_CTL_TX_PAUSE_ENABLE"[9];
            input "C{i}_CTL_TX_SEND_IDLE_PIN";
            input "C{i}_CTL_TX_SEND_LFI_PIN";
            input "C{i}_CTL_TX_SEND_RFI_PIN";

            input "C{i}_RX_PTP_ST_ADJUST_VLD";
            input "C{i}_RX_PTP_ST_ADJUST"[32];
            input "C{i}_RX_PTP_ST_ADJUST_TYPE"[2];
            input "C{i}_RX_PTP_ST_OVERWRITE";
            output "C{i}_RX_PTP_ST_SAMPLE_EDGE_OUT";
            input "C{i}_RX_PTP_ST_SYNC";
            output "C{i}_RX_PTP_ST_SYNC_OUT";
            input "C{i}_RX_PTP_SYSTEMTIMER"[55];
            output "C{i}_RX_PTP_SYSTEMTIMER_OUT"[32];

            input "C{i}_TX_PTP_ST_ADJUST_VLD";
            input "C{i}_TX_PTP_ST_ADJUST"[32];
            input "C{i}_TX_PTP_ST_ADJUST_TYPE"[2];
            input "C{i}_TX_PTP_ST_OVERWRITE";
            output "C{i}_TX_PTP_ST_SAMPLE_EDGE_OUT";
            input "C{i}_TX_PTP_ST_SYNC";
            output "C{i}_TX_PTP_ST_SYNC_OUT";
            input "C{i}_TX_PTP_SYSTEMTIMER"[55];
            output "C{i}_TX_PTP_SYSTEMTIMER_OUT"[32];
        }

        input CTL_RSVD_IN[120];
        input CTL_RX_CUSTOM_VL_LENGTH_MINUS1[16];
        input CTL_TX_CUSTOM_VL_LENGTH_MINUS1[16];
        for i in 0..20 {
            input "CTL_VL_MARKER_ID{i}"[64];
        }

        for i in 0..6 {
            input "FEC_RX_DIN_START_{i}";
            input "FEC_RX_DIN_START_{i}_BH";
            output "FEC_TX_DOUT_START_{i}";
            output "FEC_TX_DOUT_START_{i}_BH";
        }

        input RSVD_IN_RX_MAC[8];
        input RSVD_IN_RX_PHY[8];
        output RSVD_OUT[60];
        output RSVD_OUT_RX_MAC[29];
        output RSVD_OUT_RX_PHY[131];
        output RSVD_OUT_TX_MAC[67];
        output RSVD_OUT_TX_PHY[65];

        input RX_CORE_CLK;
        input RX_CORE_RESET;

        output RX_ALL_CHANNEL_MAC_PM_RDY;
        input RX_ALL_CHANNEL_MAC_PM_TICK;

        input RX_AXI_CLK;
        for i in 0..6 {
            output "RX_AXIS_TVALID_{i}";
        }
        output RX_AXIS_TID[6];
        for i in 0..12 {
            output "RX_AXIS_TDATA{i}"[128];
            output "RX_AXIS_TUSER_ENA{i}";
            output "RX_AXIS_TUSER_SOP{i}";
            output "RX_AXIS_TUSER_EOP{i}";
            output "RX_AXIS_TUSER_ERR{i}";
            output "RX_AXIS_TUSER_MTY{i}"[4];
        }

        input RX_CHANNEL_FLUSH[6];
        output RX_FEC_AM_SF_0[3];
        output RX_FEC_AM_SF_2[3];
        output RX_FEC_AM_SF_4[3];

        input RX_FLEXIF_CLK[6];
        for i in 0..6 {
            output "RX_FLEX_AMFLAG_{i}";
            output "RX_FLEX_ENA_{i}";
            output "RX_FLEX_FEC_RX_DOUT_FLAGSB_{i}"[4];
            output "RX_FLEX_FEC_RX_DOUT_FLAGS_{i}"[4];
            output "RX_FLEX_LANE0_{i}";
            output "RX_FLEX_STARTB_{i}";
            output "RX_FLEX_START_{i}";
        }
        output RX_FLEX_DATA_0[330];
        output RX_FLEX_DATA_1[330];
        output RX_FLEX_DATA_2[330];
        output RX_FLEX_DATA_3[330];
        output RX_FLEX_DATA_4[320];
        output RX_FLEX_DATA_5[320];

        output RX_LANE_ALIGNER_FILL[7];
        output RX_LANE_ALIGNER_FILL_START;
        output RX_LANE_ALIGNER_FILL_VALID;

        input RX_MACIF_CLK;
        input RX_MACIF_ENA;
        input RX_MACIF_STATUS;
        for i in 0..24 {
            input "RX_MACIF_DATA_{i}"[66];
        }
        input RX_MACIF_TS_ID[6];

        input RX_PAUSE_ACK[9];
        input RX_PAUSE_ACK_ID[3];
        for i in 0..9 {
            output "RX_PAUSE_QUANTA{i}"[16];
        }
        output RX_PAUSE_REQ[9];
        output RX_PAUSE_REQ_ID[3];
        output RX_PAUSE_VALID[9];
        output RX_PAUSE_VALID_ID[3];

        output RX_PCS_TDM_STATS_VALID;
        output RX_PCS_TDM_STATS_START;
        output RX_PCS_TDM_STATS_DATA[44];

        output RX_PORT_PM_RDY[6];
        input RX_PORT_PM_TICK[6];

        for i in 0..6 {
            output "RX_PREAMBLEOUT_{i}"[56];
            output "RX_PTP_TSTAMP_OUT_{i}"[32];
        }
        output RX_RSFEC_TDM_STATS_VALID;
        output RX_RSFEC_TDM_STATS_START;
        output RX_RSFEC_TDM_STATS_DATA[16];

        output RX_TSMAC_TDM_STATS_VALID;
        output RX_TSMAC_TDM_STATS_DATA[79];
        output RX_TSMAC_TDM_STATS_ID[6];

        input TS_CLK[6];

        input TX_CORE_CLK;
        input TX_CORE_RESET;

        output TX_ALL_CHANNEL_MAC_PM_RDY;
        input TX_ALL_CHANNEL_MAC_PM_TICK;

        input TX_AXI_CLK;
        output TX_AXIS_CH_STATUS_VLD;
        output TX_AXIS_CH_STATUS_ID[6];
        output TX_AXIS_CH_STATUS_SKIP_REQ;
        output TX_AXIS_ID_REQ_VLD;
        output TX_AXIS_ID_REQ[6];
        for i in 0..6 {
            output "TX_AXIS_TREADY_{i}";
            input "TX_AXIS_TVALID_{i}";
            output "TX_AXIS_TAF_{i}";
        }
        input TX_AXIS_TID[6];
        for i in 0..12 {
            input "TX_AXIS_TDATA{i}"[128];
            input "TX_AXIS_TUSER_ENA{i}";
            input "TX_AXIS_TUSER_EOP{i}";
            input "TX_AXIS_TUSER_ERR{i}";
            input "TX_AXIS_TUSER_MTY{i}"[4];
            input "TX_AXIS_TUSER_SOP{i}";
        }
        input TX_AXIS_TUSER_SKIP_RESPONSE;

        input TX_CHANNEL_FLUSH[6];

        input TX_FEC_AM_SF_0[3];
        input TX_FEC_AM_SF_2[3];
        input TX_FEC_AM_SF_4[3];

        input TX_FLEXIF_CLK[6];
        for i in 0..6 {
            input "TX_FLEX_AMFLAG_{i}";
            input "TX_FLEX_ENAB_{i}";
            input "TX_FLEX_ENA_{i}";
            output "TX_FLEX_STALL_{i}";
            input "TX_FLEX_STARTB_{i}";
            input "TX_FLEX_START_{i}";
        }
        input TX_FLEX_DATA_0[330];
        input TX_FLEX_DATA_1[330];
        input TX_FLEX_DATA_2[330];
        input TX_FLEX_DATA_3[330];
        input TX_FLEX_DATA_4[320];
        input TX_FLEX_DATA_5[320];

        input TX_MACIF_CLK;
        output TX_MACIF_ENA;
        for i in 0..24 {
            output "TX_MACIF_DATA_{i}"[66];
        }
        output TX_MACIF_TS_ID[6];
        input TX_MACIF_TS_ID_REQ[6];
        output TX_MACIF_TS_ID_REQ_RDY;
        input TX_MACIF_TS_ID_REQ_SIC[6];
        input TX_MACIF_TS_ID_REQ_VLD;

        output TX_PAUSE_PACKET_SENT[9];
        output TX_PAUSE_PACKET_SENT_ID[3];
        input TX_PAUSE_REQ_0[9];
        input TX_PAUSE_REQ_1[9];
        input TX_PAUSE_REQ_2[9];
        input TX_PAUSE_REQ_3[9];
        input TX_PAUSE_REQ_4[9];
        input TX_PAUSE_REQ_5[9];

        output TX_PCS_TDM_STATS_VALID;
        output TX_PCS_TDM_STATS_START;
        output TX_PCS_TDM_STATS_DATA[22];

        output TX_PORT_PM_RDY[6];
        input TX_PORT_PM_TICK[6];

        input TX_PREAMBLEIN_0[56];
        input TX_PREAMBLEIN_1[56];
        input TX_PREAMBLEIN_2[56];
        input TX_PREAMBLEIN_3[56];
        input TX_PREAMBLEIN_4[56];
        input TX_PREAMBLEIN_5[56];

        input TX_PTP_1588OP_IN_0[2];
        input TX_PTP_1588OP_IN_1[2];
        input TX_PTP_1588OP_IN_2[2];
        input TX_PTP_1588OP_IN_3[2];
        input TX_PTP_1588OP_IN_4[2];
        input TX_PTP_1588OP_IN_5[2];
        input TX_PTP_CF_OFFSET_IN_0[14];
        input TX_PTP_CF_OFFSET_IN_1[14];
        input TX_PTP_CF_OFFSET_IN_2[14];
        input TX_PTP_CF_OFFSET_IN_3[14];
        input TX_PTP_CF_OFFSET_IN_4[14];
        input TX_PTP_CF_OFFSET_IN_5[14];
        input TX_PTP_FLEX_1588LOC_IN_0[3];
        input TX_PTP_FLEX_1588LOC_IN_1[3];
        input TX_PTP_FLEX_1588LOC_IN_2[3];
        input TX_PTP_FLEX_1588LOC_IN_3[3];
        input TX_PTP_FLEX_1588LOC_IN_4[3];
        input TX_PTP_FLEX_1588LOC_IN_5[2];
        input TX_PTP_FLEX_1588OP_IN_0;
        input TX_PTP_FLEX_1588OP_IN_1;
        input TX_PTP_FLEX_1588OP_IN_2;
        input TX_PTP_FLEX_1588OP_IN_3;
        input TX_PTP_FLEX_1588OP_IN_4;
        input TX_PTP_FLEX_1588OP_IN_5;
        input TX_PTP_FLEX_TAG_FIELD_IN_0[8];
        input TX_PTP_FLEX_TAG_FIELD_IN_1[8];
        input TX_PTP_FLEX_TAG_FIELD_IN_2[8];
        input TX_PTP_FLEX_TAG_FIELD_IN_3[8];
        input TX_PTP_FLEX_TAG_FIELD_IN_4[8];
        input TX_PTP_FLEX_TAG_FIELD_IN_5[8];
        input TX_PTP_TAG_FIELD_IN_0[8];
        input TX_PTP_TAG_FIELD_IN_1[8];
        input TX_PTP_TAG_FIELD_IN_2[8];
        input TX_PTP_TAG_FIELD_IN_3[8];
        input TX_PTP_TAG_FIELD_IN_4[8];
        input TX_PTP_TAG_FIELD_IN_5[8];
        output TX_PTP_TSTAMP_0[32];
        output TX_PTP_TSTAMP_1[32];
        output TX_PTP_TSTAMP_2[32];
        output TX_PTP_TSTAMP_PORT_ID[3];
        output TX_PTP_TSTAMP_TAG_0[8];
        output TX_PTP_TSTAMP_TAG_1[8];
        output TX_PTP_TSTAMP_TAG_2[8];
        output TX_PTP_TSTAMP_VALID[3];
        for i in 0..6 {
            input "TX_PTP_UDP_CHKSUM_IN_{i}";
            input "TX_RESEND_PAUSE_{i}";
        }

        output TX_TSMAC_TDM_STATS_VALID;
        output TX_TSMAC_TDM_STATS_DATA[56];
        output TX_TSMAC_TDM_STATS_ID[6];


        input RX_SERDES_CLK[6];
        input RX_ALT_SERDES_CLK[6];
        input RX_SERDES_RESET[6];
        input TX_SERDES_CLK[6];
        input TX_SERDES_RESET[6];
        input TX_ALT_SERDES_CLK[6];
        for i in 0..6 {
            output "TX_SERDES_IS_AM_{i}";
            output "TX_SERDES_IS_AM_PREFIFO_{i}";
            output "RX_SERDES_ALBUF_RESTART_{i}";
        }
        for i in 0..24 {
            input "RX_SERDES_DATA{i}"[80];
            output "RX_SERDES_ALBUF_SLIP_{i}";
            input "RX_SERDES_FIFO_FLAGIN_{i}";
            output "RX_SERDES_FIFO_FLAGOUT_{i}";
            output "TX_SERDES_DATA{i}"[80];
        }

        input CSSD_TRIG_N;

        input SCAN_CLK_N;
        input SCAN_EN_N;
        input SCAN_EN_APB_N;
        input SCAN_RST_BYP_APB_N;
        input SCAN_MODE_APB_N;
        input SCAN_MODE_RST_APB_N;
        input SCAN_MODE_RST_N;
        input SCAN_EDT_UPDT_N;
        input SCAN_CHNL_IN[28];
        output SCAN_CHNL_OUT[28];
        input SCAN_CHNL_IN_APB;
        output SCAN_CHNL_OUT_APB;
        input SCAN_CNTRL_CHNL_IN[3];
        output SCAN_CNTRL_CHNL_OUT[3];
    }

    bel_class ILKN {
        input APB3_CLK;
        input APB3_PRESET;
        input APB3_PSEL;
        input APB3_PENABLE;
        input APB3_PWRITE;
        input APB3_PADDR[16];
        input APB3_PWDATA[32];
        output APB3_PRDATA[32];
        output APB3_PREADY;
        output APB3_PSLVERR;

        input C0_CORE_CLK;
        input C0_AXI_CLK;

        input C0_RX_RESET;
        output C0_RX_OVFOUT;
        input C0_TX_RESET;
        output C0_TX_RDYOUT;
        output C0_TX_OVFOUT;

        input C0_CTL_RX_FORCE_RESYNC;
        input C0_CTL_TX_DIAGWORD_INTFSTAT;
        input C0_CTL_TX_ENABLE;
        input C0_CTL_TX_FC_STAT[256];
        input C0_CTL_TX_MUBITS[8];

        output C0_STAT_RX_ALIGNED;
        output C0_STAT_RX_ALIGNED_ERR;
        output C0_STAT_RX_BURSTMAX_ERR;
        output C0_STAT_RX_BURST_ERR;
        output C0_STAT_RX_ERR;
        output C0_STAT_RX_FC_STAT[256];
        output C0_STAT_RX_MEOP_ERR;
        output C0_STAT_RX_MISALIGNED;
        output C0_STAT_RX_MSOP_ERR;
        output C0_STAT_RX_MUBITS[8];
        output C0_STAT_RX_MUBITS_UPDATED;
        output C0_STAT_RX_OVERFLOW_ERR;
        output C0_STAT_TX_BURST_ERR;
        output C0_STAT_TX_ENAIN_ERR;
        output C0_STAT_TX_FIFO1_OFLOW_ERR;
        output C0_STAT_TX_FIFO1_UFLOW_ERR;
        output C0_STAT_TX_FIFO2_OFLOW_ERR;
        output C0_STAT_TX_FIFO2_UFLOW_ERR;
        output C0_STAT_TX_OVERFLOW_ERR;
        output C0_STAT_TX_UNDERFLOW_ERR;

        input CTL_RSVD1_IN[128];
        input CTL_RSVD_IN_RX_TOP[512];
        input CTL_RSVD_IN_TX_TOP[32];
        input CTL_TX_DIAGWORD_LANESTAT[24];

        for i in 0..16 {
            output "RX_AXIS_TDATA{i}"[128];
            output "RX_AXIS_TUSER_CHAN{i}"[11];
            output "RX_AXIS_TUSER_ENA{i}";
            output "RX_AXIS_TUSER_SOP{i}";
            output "RX_AXIS_TUSER_EOP{i}";
            output "RX_AXIS_TUSER_ERR{i}";
            output "RX_AXIS_TUSER_MTY{i}"[4];

            input "TX_AXIS_TDATA{i}"[128];
            input "TX_AXIS_TUSER_BCTL{i}";
            input "TX_AXIS_TUSER_CHAN{i}"[11];
            input "TX_AXIS_TUSER_ENA{i}";
            input "TX_AXIS_TUSER_SOP{i}";
            input "TX_AXIS_TUSER_EOP{i}";
            input "TX_AXIS_TUSER_ERR{i}";
            input "TX_AXIS_TUSER_MTY{i}"[4];
        }

        output STAT_RSVD_OUT[217];
        output STAT_RSVD_OUT_RX_TOP[64];
        output STAT_RSVD_OUT_TX_TOP[512];
        output STAT_RX_BAD_TYPE_ERR[24];
        output STAT_RX_CRC32_ERR[24];
        output STAT_RX_CRC32_VALID[24];
        output STAT_RX_DESCRAM_ERR[24];
        output STAT_RX_DIAGWORD_INTFSTAT[24];
        output STAT_RX_DIAGWORD_LANESTAT[24];
        output STAT_RX_FRAMING_ERR[24];
        output STAT_RX_MF_ERR[24];
        output STAT_RX_MF_LEN_ERR[24];
        output STAT_RX_MF_REPEAT_ERR[24];
        output STAT_RX_SYNCED[24];
        output STAT_RX_SYNCED_ERR[24];
        output STAT_RX_WORD_SYNC[24];

        output STAT_RX_FEC_LANE_DELAY_SYNC;
        output STAT_RX_FEC_LANE_DELAY_VALUE[15];
        output STAT_RX_FEC_LANE_MAPPING_SYNC;
        output STAT_RX_FEC_LANE_MAPPING_VALUE[8];

        for i in 0..6 {
            for j in 0..2 {
                input "RX_FEC{i}_SLICE{j}_DIN_START";
                output "RX_FEC{i}_SLICE{j}_DOUT_START";
                output "RX_FEC{i}_SLICE{j}_DOUT"[160];
                output "RX_FEC{i}_SLICE{j}_DOUT_FLAGS"[4];

                input "TX_FEC{i}_SLICE{j}_DIN_START";
                input "TX_FEC{i}_SLICE{j}_DIN"[160];
                output "TX_FEC{i}_SLICE{j}_DOUT_START";
            }

            output "STAT_RX_FEC{i}_ALIGNED";
            output "STAT_RX_FEC{i}_AM_LOCK"[4];
            output "STAT_RX_FEC{i}_ECC_ERR"[4];
            for j in 0..4 {
                output "STAT_RX_FEC{i}_LANE{j}_ERR_0TO1_INC"[8];
                output "STAT_RX_FEC{i}_LANE{j}_ERR_1TO0_INC"[8];
                output "STAT_RX_FEC{i}_LANE{j}_ERR_COUNT_INC"[4];
            }
            for j in 0..2 {
                output "STAT_RX_FEC{i}_SLICE{j}_CORRECTED_CW_INC";
                output "STAT_RX_FEC{i}_SLICE{j}_CW_INC";
                output "STAT_RX_FEC{i}_SLICE{j}_UNCORRECTED_CW_INC";
            }
        }

        input RX_SERDES_CLK[6];
        input RX_ALT_SERDES_CLK[6];
        input RX_SERDES_RESET[6];
        input TX_SERDES_CLK[6];
        input TX_ALT_SERDES_CLK[6];
        input TX_SERDES_RESET[6];
        for i in 0..24 {
            input "RX_SERDES_DATA{i}"[80];
            output "TX_SERDES_DATA{i}"[80];
        }

        input TEST_MODE;
        input TEST_RESET;

        input CSSD_TRIG_N;

        input SCAN_CLK_N;
        input SCAN_RST_BYP_APB_N;
        input SCAN_MODE_RST_APB_N;
        input SCAN_MODE_RST_N;
        input SCAN_MODE_APB_N;
        input SCAN_EN_N;
        input SCAN_EN_APB_N;
        input SCAN_EDT_UPDT_N;
        input SCAN_CHNL_IN[21];
        input SCAN_CHNL_IN_APB;
        output SCAN_CHNL_OUT[21];
        output SCAN_CHNL_OUT_APB;
        input SCAN_CNTRL_CHNL_IN[2];
        output SCAN_CNTRL_CHNL_OUT[2];
    }

    bel_class HSC {
        // APB

        input APB3_CLK;
        input APB3_PRESET;
        input APB3_PENABLE;
        input APB3_PWRITE;
        input APB3_PSEL;
        input APB3_PADDR[15];
        input APB3_PWDATA[32];
        output APB3_PREADY;
        output APB3_PRDATA[32];
        output APB3_PSLVERR;

        // decryption

        input DEC_CORE_CLK;
        input DEC_CORE_RESET;

        input DEC_IGR_AXIS_CLK;
        input DEC_IGR_AXIS_RESET;

        output DEC_IGR_AXIS_TREADY_0;
        output DEC_IGR_AXIS_TREADY_1;
        output DEC_IGR_AXIS_TREADY_2;
        output DEC_IGR_AXIS_TREADY_3;
        input DEC_IGR_AXIS_TVALID_0;
        input DEC_IGR_AXIS_TVALID_1;
        input DEC_IGR_AXIS_TVALID_2;
        input DEC_IGR_AXIS_TVALID_3;
        output DEC_IGR_AXIS_SKIP_REQ;
        output DEC_IGR_AXIS_SKIP_ID[6];
        output DEC_IGR_AXIS_TAF_0;
        output DEC_IGR_AXIS_TAF_1;
        output DEC_IGR_AXIS_TAF_2;
        output DEC_IGR_AXIS_TAF_3;
        input DEC_IGR_AXIS_TID[6];
        for i in 0..8 {
            input "DEC_IGR_AXIS_TDATA{i}"[128];
            input "DEC_IGR_AXIS_TUSER_ENA{i}";
            input "DEC_IGR_AXIS_TUSER_EOP{i}";
            input "DEC_IGR_AXIS_TUSER_ERR{i}";
            input "DEC_IGR_AXIS_TUSER_MTY{i}"[4];
            input "DEC_IGR_AXIS_TUSER_SOP{i}";
        }

        input DEC_IGR_CH_FLUSH[4];
        for i in 0..4 {
            input "DEC_IGR_PRTIF_CRYPTO_AUTH_ONLY_P{i}";
            input "DEC_IGR_PRTIF_CRYPTO_BYP_P{i}";
            input "DEC_IGR_PRTIF_CRYPTO_CIPHER_SUITE_P{i}"[2];
            input "DEC_IGR_PRTIF_CRYPTO_CONF_OFFSET_P{i}"[6];
            input "DEC_IGR_PRTIF_CRYPTO_ICV_P{i}"[128];
            input "DEC_IGR_PRTIF_CRYPTO_IV_SALT_P{i}"[96];
            input "DEC_IGR_PRTIF_CRYPTO_MODE_P{i}"[2];
            input "DEC_IGR_PRTIF_CRYPTO_REPLAY_PROT_EN_P{i}";
            input "DEC_IGR_PRTIF_CRYPTO_SA_INDEX_P{i}"[20];
            input "DEC_IGR_PRTIF_CRYPTO_SPARE_IN_P{i}"[32];
            input "DEC_IGR_PRTIF_CRYPTO_ZLEN_P{i}";
            input "DEC_IGR_PRTIF_EXT_KEY_P{i}"[256];
            input "DEC_IGR_PRTIF_MACSEC_SA_IN_USE_P{i}";
            input "DEC_IGR_PRTIF_MACSEC_VALIDATION_MODE_P{i}"[2];
        }

        input DEC_EGR_AXIS_CLK;
        input DEC_EGR_AXIS_RESET;
        output DEC_EGR_AXIS_TVALID_0;
        output DEC_EGR_AXIS_TVALID_1;
        output DEC_EGR_AXIS_TVALID_2;
        output DEC_EGR_AXIS_TVALID_3;
        output DEC_EGR_AXIS_TID[6];
        for i in 0..8 {
            output "DEC_EGR_AXIS_TDATA{i}"[128];
            output "DEC_EGR_AXIS_TUSER_ENA{i}";
            output "DEC_EGR_AXIS_TUSER_SOP{i}";
            output "DEC_EGR_AXIS_TUSER_EOP{i}";
            output "DEC_EGR_AXIS_TUSER_ERR{i}";
            output "DEC_EGR_AXIS_TUSER_MTY{i}"[4];
        }

        for i in 0..4 {
            output "DEC_EGR_PRTIF_CRYPTO_AUTH_ONLY_P{i}";
            output "DEC_EGR_PRTIF_CRYPTO_BYP_P{i}";
            output "DEC_EGR_PRTIF_CRYPTO_DISCARD_P{i}";
            output "DEC_EGR_PRTIF_CRYPTO_ICV_CHK_FAIL_P{i}";
            output "DEC_EGR_PRTIF_CRYPTO_PKT_NUM_P{i}"[64];
            output "DEC_EGR_PRTIF_CRYPTO_REPLAY_CHK_FAIL_P{i}";
            output "DEC_EGR_PRTIF_CRYPTO_SA_INDEX_P{i}"[20];
            output "DEC_EGR_PRTIF_CRYPTO_SPARE_OUT_P{i}"[32];
            output "DEC_EGR_PRTIF_CRYPTO_ZLEN_P{i}";
            output "DEC_EGR_PRTIF_IPSEC_NEXT_HEADER_P{i}"[8];
            output "DEC_EGR_PRTIF_MACSEC_SECTAG_AN_P{i}"[2];
        }

        // encryption

        input ENC_CORE_CLK;
        input ENC_CORE_RESET;

        input ENC_IGR_AXIS_CLK;
        input ENC_IGR_AXIS_RESET;
        output ENC_IGR_AXIS_TREADY_0;
        output ENC_IGR_AXIS_TREADY_1;
        output ENC_IGR_AXIS_TREADY_2;
        output ENC_IGR_AXIS_TREADY_3;
        input ENC_IGR_AXIS_TVALID_0;
        input ENC_IGR_AXIS_TVALID_1;
        input ENC_IGR_AXIS_TVALID_2;
        input ENC_IGR_AXIS_TVALID_3;
        output ENC_IGR_AXIS_SKIP_REQ;
        output ENC_IGR_AXIS_SKIP_ID[6];
        output ENC_IGR_AXIS_TAF_0;
        output ENC_IGR_AXIS_TAF_1;
        output ENC_IGR_AXIS_TAF_2;
        output ENC_IGR_AXIS_TAF_3;
        input ENC_IGR_AXIS_TID[6];
        for i in 0..8 {
            input "ENC_IGR_AXIS_TDATA{i}"[128];
            input "ENC_IGR_AXIS_TUSER_ENA{i}";
            input "ENC_IGR_AXIS_TUSER_EOP{i}";
            input "ENC_IGR_AXIS_TUSER_ERR{i}";
            input "ENC_IGR_AXIS_TUSER_MTY{i}"[4];
            input "ENC_IGR_AXIS_TUSER_SOP{i}";
        }

        input ENC_IGR_CH_FLUSH[4];

        for i in 0..4 {
            input "ENC_IGR_PRTIF_CRYPTO_AUTH_ONLY_P{i}";
            input "ENC_IGR_PRTIF_CRYPTO_BYP_P{i}";
            input "ENC_IGR_PRTIF_CRYPTO_CIPHER_SUITE_P{i}"[2];
            input "ENC_IGR_PRTIF_CRYPTO_CONF_OFFSET_P{i}"[6];
            input "ENC_IGR_PRTIF_CRYPTO_IV_SALT_P{i}"[96];
            input "ENC_IGR_PRTIF_CRYPTO_MODE_P{i}"[2];
            input "ENC_IGR_PRTIF_CRYPTO_PKT_NUM_P{i}"[64];
            input "ENC_IGR_PRTIF_CRYPTO_SA_INDEX_P{i}"[20];
            input "ENC_IGR_PRTIF_CRYPTO_SPARE_IN_P{i}"[32];
            input "ENC_IGR_PRTIF_CRYPTO_ZLEN_P{i}";
            input "ENC_IGR_PRTIF_EXT_KEY_P{i}"[256];
            input "ENC_IGR_PRTIF_MACSEC_SECTAG_AN_P{i}"[2];
            input "ENC_IGR_PRTIF_MACSEC_SECTAG_SCI_P{i}"[64];
            input "ENC_IGR_PRTIF_MACSEC_SECTAG_SHORTLEN_P{i}"[8];
            input "ENC_IGR_PRTIF_MACSEC_SECTAG_SSCI_P{i}"[32];
            input "ENC_IGR_PRTIF_MACSEC_SECTAG_TCI_P{i}"[6];
        }

        input ENC_EGR_AXIS_CLK;
        input ENC_EGR_AXIS_RESET;
        output ENC_EGR_AXIS_TID[6];
        output ENC_EGR_AXIS_TVALID_0;
        output ENC_EGR_AXIS_TVALID_1;
        output ENC_EGR_AXIS_TVALID_2;
        output ENC_EGR_AXIS_TVALID_3;
        for i in 0..8 {
            output "ENC_EGR_AXIS_TDATA{i}"[128];
            output "ENC_EGR_AXIS_TUSER_ENA{i}";
            output "ENC_EGR_AXIS_TUSER_EOP{i}";
            output "ENC_EGR_AXIS_TUSER_ERR{i}";
            output "ENC_EGR_AXIS_TUSER_MTY{i}"[4];
            output "ENC_EGR_AXIS_TUSER_SOP{i}";
        }

        for i in 0..4 {
            output "ENC_EGR_PRTIF_CRYPTO_AUTH_ONLY_P{i}";
            output "ENC_EGR_PRTIF_CRYPTO_BYP_P{i}";
            output "ENC_EGR_PRTIF_CRYPTO_ICV_P{i}"[128];
            output "ENC_EGR_PRTIF_CRYPTO_SA_INDEX_P{i}"[20];
            output "ENC_EGR_PRTIF_CRYPTO_SPARE_OUT_P{i}"[32];
            output "ENC_EGR_PRTIF_CRYPTO_ZLEN_P{i}";
            output "ENC_EGR_PRTIF_MACSEC_MTU_CHECK_FAILURE_P{i}";
        }

        // status

        output STAT_DEC_ECC_CORRECTABLE_ERR;
        output STAT_DEC_ECC_UNCORRECTABLE_ERR;
        output STAT_ENC_ECC_CORRECTABLE_ERR;
        output STAT_ENC_ECC_UNCORRECTABLE_ERR;

        output STAT_RSVD_OUT_RX_TOP[128];
        output STAT_RSVD_OUT_TX_TOP[128];

        output STAT_RX_SC_DELAYED_PKT_A[2];
        output STAT_RX_SC_DELAYED_PKT_B;
        output STAT_RX_SC_INVALID_PKT_A[2];
        output STAT_RX_SC_INVALID_PKT_B;
        output STAT_RX_SC_LATE_PKT_A[2];
        output STAT_RX_SC_LATE_PKT_B;
        output STAT_RX_SC_NOT_VALID_PKT_A[2];
        output STAT_RX_SC_NOT_VALID_PKT_B;
        output STAT_RX_SC_OK_PKT_A[2];
        output STAT_RX_SC_OK_PKT_B;
        output STAT_RX_SC_UNCHECKED_PKT_A[2];
        output STAT_RX_SC_UNCHECKED_PKT_B;
        output STAT_RX_SECY_BAD_TAG_PKT_A[2];
        output STAT_RX_SECY_BAD_TAG_PKT_B;
        output STAT_RX_SECY_DECR_OCTETS_A[16];
        output STAT_RX_SECY_DECR_OCTETS_B[16];
        output STAT_RX_SECY_NO_SA_ERROR_PKT_A[2];
        output STAT_RX_SECY_NO_SA_ERROR_PKT_B;
        output STAT_RX_SECY_NO_SA_PKT_A[2];
        output STAT_RX_SECY_NO_SA_PKT_B;
        output STAT_RX_SECY_NO_TAG_PKT_A[2];
        output STAT_RX_SECY_NO_TAG_PKT_B;
        output STAT_RX_SECY_UNTAGGED_PKT_A[2];
        output STAT_RX_SECY_UNTAGGED_PKT_B;
        output STAT_RX_SECY_VALD_OCTETS_A[16];
        output STAT_RX_SECY_VALD_OCTETS_B[16];

        output STAT_TX_SC_ENCR_PKT_A[2];
        output STAT_TX_SC_ENCR_PKT_B;
        output STAT_TX_SC_PROT_PKT_A[2];
        output STAT_TX_SC_PROT_PKT_B;
        output STAT_TX_SECY_ENCR_OCTETS_A[16];
        output STAT_TX_SECY_ENCR_OCTETS_B[16];
        output STAT_TX_SECY_PROT_OCTETS_A[16];
        output STAT_TX_SECY_PROT_OCTETS_B[16];
        output STAT_TX_SECY_TOO_LONG_PKT_A[2];
        output STAT_TX_SECY_TOO_LONG_PKT_B;
        output STAT_TX_SECY_UNTAGGED_PKT_A[2];
        output STAT_TX_SECY_UNTAGGED_PKT_B;

        // stats

        output RX_STATS_VALID_A;
        output RX_SC_STATS_VALID_B;
        output RX_SC_STATS_NUM_A[19];
        output RX_SC_STATS_NUM_B[19];
        output RX_SECY_STATS_VALID_B;
        output RX_SECY_STATS_NUM_A[6];
        output RX_SECY_STATS_NUM_B[6];

        output TX_STATS_VALID_A;
        output TX_SC_STATS_VALID_B;
        output TX_SC_STATS_NUM_A[19];
        output TX_SC_STATS_NUM_B[19];
        output TX_SECY_STATS_VALID_B;
        output TX_SECY_STATS_NUM_A[6];
        output TX_SECY_STATS_NUM_B[6];

        // misc

        input CSSD_TRIG_N;
        input CTL_RSVD_IN_RX_TOP[64];
        input CTL_RSVD_IN_TX_TOP[64];

        input SCAN_CLK_N;
        input SCAN_MODE_RST_N;
        input SCAN_MODE_RST_APB_N;
        input SCAN_RST_BYP_APB_N;
        input SCAN_EN_N;
        input SCAN_EN_APB_N;
        input SCAN_MODE_APB_N;
        input SCAN_EDT_UPDT_N;
        input SCAN_CHNL_IN[28];
        input SCAN_CHNL_IN_APB;
        output SCAN_CHNL_OUT[28];
        output SCAN_CHNL_OUT_APB;
        input SCAN_CNTRL_CHNL_IN[3];
        output SCAN_CNTRL_CHNL_OUT[3];
    }

    bel_class GTY {
        input APB3CLK;
        input APB3PRESETN;
        input APB3PENABLE;
        input APB3PWRITE;
        input APB3PSEL;
        input APB3PADDR[16];
        input APB3PWDATA[32];
        output APB3PREADY;
        output APB3PRDATA[32];
        output APB3PSLVERR;
        input ALTCLK;

        input AXISCLK;
        for i in 0..3 {
            output "S{i}_AXIS_TREADY";
            input "S{i}_AXIS_TVALID";
            input "S{i}_AXIS_TDATA"[32];
            input "S{i}_AXIS_TLAST";

            input "M{i}_AXIS_TREADY";
            output "M{i}_AXIS_TVALID";
            output "M{i}_AXIS_TDATA"[32];
            output "M{i}_AXIS_TLAST";
        }

        for i in 0..2 {
            input "REFCLK{i}_CLKTESTSIG";
            input "REFCLK{i}_GTREFCLKPD";
            output "REFCLK{i}_ODIV2";
        }

        for i in 0..4 {
            input "CH{i}_BSR_SERIAL";
            output "CH{i}_BUFGTCE";
            output "CH{i}_BUFGTCEMASK"[4];
            output "CH{i}_BUFGTDIV"[12];
            output "CH{i}_BUFGTRST";
            output "CH{i}_BUFGTRSTMASK"[4];
            input "CH{i}_CDRBMCDRREQ";
            input "CH{i}_CDRFREQOS";
            input "CH{i}_CDRINCPCTRL";
            input "CH{i}_CDRSTEPDIR";
            input "CH{i}_CDRSTEPSQ";
            input "CH{i}_CDRSTEPSX";
            input "CH{i}_CFOKOVRDFINISH";
            input "CH{i}_CFOKOVRDPULSE";
            output "CH{i}_CFOKOVRDRDY0";
            output "CH{i}_CFOKOVRDRDY1";
            input "CH{i}_CFOKOVRDSTART";
            input "CH{i}_CLKRSVD0";
            input "CH{i}_CLKRSVD1";
            input "CH{i}_DMONFIFORESET";
            input "CH{i}_DMONITORCLK";
            output "CH{i}_DMONITOROUTCLK";
            output "CH{i}_DMONITOROUT"[32];
            output "CH{i}_EYESCANDATAERROR";
            input "CH{i}_EYESCANRESET";
            input "CH{i}_EYESCANTRIGGER";
            input "CH{i}_GTRSVD"[16];
            input "CH{i}_GTRXRESET";
            input "CH{i}_GTTXRESET";
            input "CH{i}_HSDPPCSRESET";
            input "CH{i}_ILORESET";
            output "CH{i}_ILORESETDONE";
            input "CH{i}_ILORESETMASK";
            input "CH{i}_LOOPBACK"[3];
            input "CH{i}_PCIERSTB";
            input "CH{i}_PCSRSVDIN"[16];
            output "CH{i}_PCSRSVDOUT"[16];
            input "CH{i}_PHYESMADAPTSAVE";
            output "CH{i}_PHYREADY";
            output "CH{i}_PHYSTATUS";
            output "CH{i}_PINRSVDAS"[16];
            output "CH{i}_RESETEXCEPTION";

            output "CH{i}_RX10GSTAT"[8];
            output "CH{i}_RXBUFSTATUS"[3];
            output "CH{i}_RXBYTEISALIGNED";
            output "CH{i}_RXBYTEREALIGN";
            input "CH{i}_RXCDRHOLD";
            output "CH{i}_RXCDRLOCK";
            input "CH{i}_RXCDROVRDEN";
            output "CH{i}_RXCDRPHDONE";
            input "CH{i}_RXCDRRESET";
            output "CH{i}_RXCHANBONDSEQ";
            output "CH{i}_RXCHANISALIGNED";
            output "CH{i}_RXCHANREALIGN";
            input "CH{i}_RXCHBONDI"[5];
            output "CH{i}_RXCHBONDO"[5];
            output "CH{i}_RXCLKCORCNT"[2];
            output "CH{i}_RXCOMINITDET";
            output "CH{i}_RXCOMMADET";
            output "CH{i}_RXCOMSASDET";
            output "CH{i}_RXCOMWAKEDET";
            output "CH{i}_RXCTRL0"[16];
            output "CH{i}_RXCTRL1"[16];
            output "CH{i}_RXCTRL2"[8];
            output "CH{i}_RXCTRL3"[8];
            input "CH{i}_RXDAPICODEOVRDEN";
            input "CH{i}_RXDAPICODERESET";
            output "CH{i}_RXDATAEXTENDRSVD"[8];
            output "CH{i}_RXDATAVALID"[2];
            output "CH{i}_RXDATA"[128];
            output "CH{i}_RXDCCDONE";
            output "CH{i}_RXDLYALIGNERR";
            output "CH{i}_RXDLYALIGNPROG";
            input "CH{i}_RXDLYALIGNREQ";
            output "CH{i}_RXELECIDLE";
            input "CH{i}_RXEQTRAINING";
            output "CH{i}_RXFINEALIGNDONE";
            input "CH{i}_RXGEARBOXSLIP";
            output "CH{i}_RXHEADERVALID"[2];
            output "CH{i}_RXHEADER"[6];
            input "CH{i}_RXLATCLK";
            input "CH{i}_RXLPMEN";
            input "CH{i}_RXMLDCHAINDONE";
            input "CH{i}_RXMLDCHAINREQ";
            input "CH{i}_RXMLFINEALIGNREQ";
            input "CH{i}_RXOOBRESET";
            output "CH{i}_RXOSINTDONE";
            output "CH{i}_RXOSINTSTARTED";
            output "CH{i}_RXOSINTSTROBEDONE";
            output "CH{i}_RXOSINTSTROBESTARTED";
            output "CH{i}_RXOUTCLK";
            input "CH{i}_RXPCSRESETMASK"[5];
            input "CH{i}_RXPD"[2];
            output "CH{i}_RXPHALIGNDONE";
            output "CH{i}_RXPHALIGNERR";
            input "CH{i}_RXPHALIGNREQ";
            input "CH{i}_RXPHALIGNRESETMASK"[2];
            input "CH{i}_RXPHDLYPD";
            input "CH{i}_RXPHDLYRESET";
            output "CH{i}_RXPHDLYRESETDONE";
            output "CH{i}_RXPHSETINITDONE";
            input "CH{i}_RXPHSETINITREQ";
            input "CH{i}_RXPHSHIFT180";
            output "CH{i}_RXPHSHIFT180DONE";
            output "CH{i}_RXPMARESETDONE";
            input "CH{i}_RXPMARESETMASK"[7];
            input "CH{i}_RXPOLARITY";
            input "CH{i}_RXPRBSCNTRESET";
            output "CH{i}_RXPRBSERR";
            output "CH{i}_RXPRBSLOCKED";
            input "CH{i}_RXPRBSSEL"[4];
            input "CH{i}_RXPROGDIVRESET";
            output "CH{i}_RXPROGDIVRESETDONE";
            input "CH{i}_RXRATE"[8];
            output "CH{i}_RXRESETDONE";
            input "CH{i}_RXRESETMODE"[2];
            input "CH{i}_RXSLIDE";
            output "CH{i}_RXSLIDERDY";
            output "CH{i}_RXSTARTOFSEQ"[2];
            output "CH{i}_RXSTATUS"[3];
            input "CH{i}_RXSYNCALLIN";
            output "CH{i}_RXSYNCDONE";
            input "CH{i}_RXTERMINATION";
            input "CH{i}_RXUSERRDY";
            input "CH{i}_RXUSRCLK";
            output "CH{i}_RXVALID";

            output "CH{i}_TX10GSTAT";
            output "CH{i}_TXBUFSTATUS"[2];
            output "CH{i}_TXCOMFINISH";
            input "CH{i}_TXCOMINIT";
            input "CH{i}_TXCOMSAS";
            input "CH{i}_TXCOMWAKE";
            input "CH{i}_TXCTRL0"[16];
            input "CH{i}_TXCTRL1"[16];
            input "CH{i}_TXCTRL2"[8];
            input "CH{i}_TXDAPICODEOVRDEN";
            input "CH{i}_TXDAPICODERESET";
            input "CH{i}_TXDATAEXTENDRSVD"[8];
            input "CH{i}_TXDATA"[128];
            output "CH{i}_TXDCCDONE";
            input "CH{i}_TXDEEMPH"[2];
            input "CH{i}_TXDETECTRX";
            input "CH{i}_TXDIFFCTRL"[5];
            output "CH{i}_TXDLYALIGNERR";
            output "CH{i}_TXDLYALIGNPROG";
            input "CH{i}_TXDLYALIGNREQ";
            input "CH{i}_TXELECIDLE";
            input "CH{i}_TXHEADER"[6];
            input "CH{i}_TXINHIBIT";
            input "CH{i}_TXLATCLK";
            input "CH{i}_TXMAINCURSOR"[7];
            input "CH{i}_TXMARGIN"[3];
            input "CH{i}_TXMLDCHAINDONE";
            input "CH{i}_TXMLDCHAINREQ";
            input "CH{i}_TXONESZEROS";
            output "CH{i}_TXOUTCLK";
            input "CH{i}_TXPAUSEDELAYALIGN";
            input "CH{i}_TXPCSRESETMASK";
            input "CH{i}_TXPD"[2];
            output "CH{i}_TXPHALIGNDONE";
            output "CH{i}_TXPHALIGNERR";
            output "CH{i}_TXPHALIGNOUTRSVD";
            input "CH{i}_TXPHALIGNREQ";
            input "CH{i}_TXPHALIGNRESETMASK"[2];
            input "CH{i}_TXPHDLYPD";
            input "CH{i}_TXPHDLYRESET";
            output "CH{i}_TXPHDLYRESETDONE";
            input "CH{i}_TXPHDLYTSTCLK";
            output "CH{i}_TXPHSETINITDONE";
            input "CH{i}_TXPHSETINITREQ";
            input "CH{i}_TXPHSHIFT180";
            output "CH{i}_TXPHSHIFT180DONE";
            input "CH{i}_TXPICODEOVRDEN";
            input "CH{i}_TXPICODERESET";
            input "CH{i}_TXPIPPMEN";
            input "CH{i}_TXPIPPMSTEPSIZE"[5];
            input "CH{i}_TXPISOPD";
            output "CH{i}_TXPMARESETDONE";
            input "CH{i}_TXPMARESETMASK"[3];
            input "CH{i}_TXPOLARITY";
            input "CH{i}_TXPOSTCURSOR"[5];
            input "CH{i}_TXPRBSFORCEERR";
            input "CH{i}_TXPRBSSEL"[4];
            input "CH{i}_TXPRECURSOR"[5];
            input "CH{i}_TXPROGDIVRESET";
            output "CH{i}_TXPROGDIVRESETDONE";
            input "CH{i}_TXRATE"[8];
            output "CH{i}_TXRESETDONE";
            input "CH{i}_TXRESETMODE"[2];
            input "CH{i}_TXSEQUENCE"[7];
            input "CH{i}_TXSWING";
            input "CH{i}_TXSYNCALLIN";
            output "CH{i}_TXSYNCDONE";
            input "CH{i}_TXUSERRDY";
            input "CH{i}_TXUSRCLK";

            input "CH{i}_CSSDRSTB";
            input "CH{i}_CSSDSTOPCLK";
            output "CH{i}_CSSDSTOPCLKDONE";
            input "CH{i}_SCANCLKB";
            input "CH{i}_SCANRSTB";
            input "CH{i}_SCANENB";
            input "CH{i}_EDTUPDATEB";
            input "CH{i}_SCANCNTRLIN";
            output "CH{i}_SCANCNTRLOUT";
            input "CH{i}_SCANIN"[4];
            output "CH{i}_SCANOUT"[4];
            input "CH{i}_TSTCLK0";
            input "CH{i}_TSTCLK1";
            input "CH{i}_TSTIN"[20];
        }

        for i in 0..2 {
            input "HSCLK{i}_LCPLLCLKRSVD0";
            input "HSCLK{i}_LCPLLCLKRSVD1";
            output "HSCLK{i}_LCPLLFBCLKLOST";
            input "HSCLK{i}_LCPLLFBDIV"[8];
            input "HSCLK{i}_LCPLLGTGREFCLK";
            output "HSCLK{i}_LCPLLLOCK";
            input "HSCLK{i}_LCPLLPD";
            output "HSCLK{i}_LCPLLREFCLKLOST";
            output "HSCLK{i}_LCPLLREFCLKMONITOR";
            input "HSCLK{i}_LCPLLREFCLKSEL"[3];
            input "HSCLK{i}_LCPLLRESET";
            input "HSCLK{i}_LCPLLRESETBYPASSMODE";
            input "HSCLK{i}_LCPLLRESETMASK"[2];
            input "HSCLK{i}_LCPLLRSVD0"[8];
            input "HSCLK{i}_LCPLLRSVD1"[8];
            output "HSCLK{i}_LCPLLRSVDOUT"[8];
            input "HSCLK{i}_LCPLLSDMDATA"[26];
            input "HSCLK{i}_LCPLLSDMTOGGLE";

            input "HSCLK{i}_RPLLCLKRSVD0";
            input "HSCLK{i}_RPLLCLKRSVD1";
            output "HSCLK{i}_RPLLFBCLKLOST";
            input "HSCLK{i}_RPLLFBDIV"[8];
            input "HSCLK{i}_RPLLGTGREFCLK";
            output "HSCLK{i}_RPLLLOCK";
            input "HSCLK{i}_RPLLPD";
            output "HSCLK{i}_RPLLREFCLKLOST";
            output "HSCLK{i}_RPLLREFCLKMONITOR";
            input "HSCLK{i}_RPLLREFCLKSEL"[3];
            input "HSCLK{i}_RPLLRESET";
            input "HSCLK{i}_RPLLRESETBYPASSMODE";
            input "HSCLK{i}_RPLLRESETMASK"[2];
            input "HSCLK{i}_RPLLRSVD0"[8];
            input "HSCLK{i}_RPLLRSVD1"[8];
            output "HSCLK{i}_RPLLRSVDOUT"[8];
            input "HSCLK{i}_RPLLSDMDATA"[26];
            input "HSCLK{i}_RPLLSDMTOGGLE";
        }

        input BGBYPASSB;
        input BGMONITORENB;
        input BGPDB;
        input BGRCALOVRDENB;
        input BGRCALOVRD[5];

        output CORRECTERR;
        output UNCORRECTERR;

        input CTRLRSVDIN0[16];
        input CTRLRSVDIN1[14];
        output CTRLRSVDOUT[32];

        input DEBUGTRACECLK;
        input DEBUGTRACEREADY;
        output DEBUGTRACETVALID;
        output DEBUGTRACETDATA[16];

        input GPI[16];
        output GPO[16];

        output GTPOWERGOOD;

        input MSTRXRESET[4];
        output MSTRXRESETDONE[4];
        input MSTTXRESET[4];
        output MSTTXRESETDONE[4];

        input PCIELINKREACHTARGET;
        input PCIELTSSM[6];

        output RCALCMP;
        input RCALENB;
        output RCALOUT[5];

        input RXMARGINCLK;
        output RXMARGINREQACK;
        input RXMARGINREQCMD[4];
        input RXMARGINREQLANENUM[2];
        input RXMARGINREQPAYLD[8];
        input RXMARGINREQREQ;
        input RXMARGINRESACK;
        output RXMARGINRESCMD[4];
        output RXMARGINRESLANENUM[2];
        output RXMARGINRESPAYLD[8];
        output RXMARGINRESREQ;

        input TRIGIN0;
        output TRIGACKIN0;
        output TRIGOUT0;
        input TRIGACKOUT0;

        input UBENABLE;
        input UBMBRST;
        input UBIOLMBRST;
        input UBINTR[12];
        output UBINTERRUPT;
        input UBRXUART;
        output UBTXUART;

        input XPSCANCLK;
        input XPSCANRSTB;
        input XPSCANMODEB;
        input XPSCANENB;
        input XPSCANIN[16];
        output XPSCANOUT[16];

        input CSSDRSTB;
        input CSSDSTOPCLK;
        output CSSDSTOPCLKDONE;
        input SCANCLKB;
        input SCANRSTB;
        input SCANENB;
        input EDTUPDATEB;
        input SCANCNTRLIN;
        output SCANCNTRLOUT;
        input SCANIN[4];
        output SCANOUT[4];
    }

    bel_class GTYP {
        input APB3CLK;
        input APB3PRESETN;
        input APB3PENABLE;
        input APB3PWRITE;
        input APB3PSEL;
        input APB3PADDR[16];
        input APB3PWDATA[32];
        output APB3PREADY;
        output APB3PRDATA[32];
        output APB3PSLVERR;

        input AXISCLK;
        for i in 0..4 {
            output "S{i}_AXIS_TREADY";
            input "S{i}_AXIS_TVALID";
            input "S{i}_AXIS_TDATA"[32];
            input "S{i}_AXIS_TLAST";

            input "M{i}_AXIS_TREADY";
            output "M{i}_AXIS_TVALID";
            output "M{i}_AXIS_TDATA"[32];
            output "M{i}_AXIS_TLAST";
        }

        for i in 0..2 {
            input "REFCLK{i}_CLKTESTSIG";
            input "REFCLK{i}_GTREFCLKPD";
            output "REFCLK{i}_ODIV2";
        }

        for i in 0..4 {
            input "CH{i}_BSR_SERIAL";
            output "CH{i}_BUFGTCE";
            output "CH{i}_BUFGTCEMASK"[4];
            output "CH{i}_BUFGTDIV"[12];
            output "CH{i}_BUFGTRST";
            output "CH{i}_BUFGTRSTMASK"[4];
            input "CH{i}_CDRBMCDRREQ";
            input "CH{i}_CDRFREQOS";
            input "CH{i}_CDRINCPCTRL";
            input "CH{i}_CDRSTEPDIR";
            input "CH{i}_CDRSTEPSQ";
            input "CH{i}_CDRSTEPSX";
            input "CH{i}_CLKRSVD0";
            input "CH{i}_CLKRSVD1";
            input "CH{i}_DFEHOLD";
            input "CH{i}_DFEOVRD";
            input "CH{i}_DMONFIFORESET";
            input "CH{i}_DMONITORCLK";
            output "CH{i}_DMONITOROUTCLK";
            output "CH{i}_DMONITOROUT"[32];
            output "CH{i}_EYESCANDATAERROR";
            input "CH{i}_EYESCANRESET";
            input "CH{i}_EYESCANTRIGGER";
            input "CH{i}_GTRSVD"[16];
            input "CH{i}_GTRXRESET";
            input "CH{i}_GTTXRESET";
            input "CH{i}_HSDPPCSRESET";
            input "CH{i}_ILORESET";
            output "CH{i}_ILORESETDONE";
            input "CH{i}_ILORESETMASK";
            input "CH{i}_LOOPBACK"[3];
            input "CH{i}_PCIERSTB";
            input "CH{i}_PCSRSVDIN"[16];
            output "CH{i}_PCSRSVDOUT"[16];
            input "CH{i}_PHYESMADAPTSAVE";
            output "CH{i}_PHYREADY";
            output "CH{i}_PHYSTATUS";
            output "CH{i}_PINRSVDAS"[16];
            output "CH{i}_REFDEBUGOUT"[2];
            output "CH{i}_RESETEXCEPTION";

            output "CH{i}_RX10GSTAT"[8];
            output "CH{i}_RXBUFSTATUS"[3];
            output "CH{i}_RXBYTEISALIGNED";
            output "CH{i}_RXBYTEREALIGN";
            input "CH{i}_RXCDRHOLD";
            output "CH{i}_RXCDRLOCK";
            input "CH{i}_RXCDROVRDEN";
            output "CH{i}_RXCDRPHDONE";
            input "CH{i}_RXCDRRESET";
            output "CH{i}_RXCHANBONDSEQ";
            output "CH{i}_RXCHANISALIGNED";
            output "CH{i}_RXCHANREALIGN";
            input "CH{i}_RXCHBONDI"[5];
            output "CH{i}_RXCHBONDO"[5];
            output "CH{i}_RXCLKCORCNT"[2];
            output "CH{i}_RXCOMINITDET";
            output "CH{i}_RXCOMMADET";
            output "CH{i}_RXCOMSASDET";
            output "CH{i}_RXCOMWAKEDET";
            output "CH{i}_RXCTRL0"[16];
            output "CH{i}_RXCTRL1"[16];
            output "CH{i}_RXCTRL2"[8];
            output "CH{i}_RXCTRL3"[8];
            input "CH{i}_RXDAPICODEOVRDEN";
            input "CH{i}_RXDAPICODERESET";
            input "CH{i}_RXDAPIRESET";
            output "CH{i}_RXDAPIRESETDONE";
            input "CH{i}_RXDAPIRESETMASK"[2];
            output "CH{i}_RXDATAVALID"[2];
            output "CH{i}_RXDATA"[128];
            output "CH{i}_RXDEBUGPCSOUT";
            output "CH{i}_RXDLYALIGNERR";
            output "CH{i}_RXDLYALIGNPROG";
            input "CH{i}_RXDLYALIGNREQ";
            output "CH{i}_RXELECIDLE";
            input "CH{i}_RXEQTRAINING";
            output "CH{i}_RXFINEALIGNDONE";
            input "CH{i}_RXGEARBOXSLIP";
            output "CH{i}_RXHEADERVALID"[2];
            output "CH{i}_RXHEADER"[6];
            input "CH{i}_RXLATCLK";
            input "CH{i}_RXLPMEN";
            input "CH{i}_RXMLDCHAINDONE";
            input "CH{i}_RXMLDCHAINREQ";
            input "CH{i}_RXMLFINEALIGNREQ";
            input "CH{i}_RXOOBRESET";
            output "CH{i}_RXOSINTDONE";
            output "CH{i}_RXOUTCLK";
            input "CH{i}_RXPCSRESETMASK"[5];
            input "CH{i}_RXPD"[2];
            output "CH{i}_RXPHALIGNDONE";
            output "CH{i}_RXPHALIGNERR";
            input "CH{i}_RXPHALIGNREQ";
            input "CH{i}_RXPHALIGNRESETMASK"[2];
            input "CH{i}_RXPHDLYPD";
            input "CH{i}_RXPHDLYRESET";
            output "CH{i}_RXPHDLYRESETDONE";
            output "CH{i}_RXPHSETINITDONE";
            input "CH{i}_RXPHSETINITREQ";
            input "CH{i}_RXPHSHIFT180";
            output "CH{i}_RXPHSHIFT180DONE";
            output "CH{i}_RXPKDET";
            output "CH{i}_RXPMARESETDONE";
            input "CH{i}_RXPMARESETMASK"[7];
            input "CH{i}_RXPOLARITY";
            input "CH{i}_RXPRBSCNTRESET";
            output "CH{i}_RXPRBSERR";
            output "CH{i}_RXPRBSLOCKED";
            input "CH{i}_RXPRBSSEL"[4];
            input "CH{i}_RXPROGDIVRESET";
            output "CH{i}_RXPROGDIVRESETDONE";
            input "CH{i}_RXQPIEN";
            output "CH{i}_RXQPISENN";
            output "CH{i}_RXQPISENP";
            input "CH{i}_RXRATE"[8];
            output "CH{i}_RXRESETDONE";
            input "CH{i}_RXRESETMODE"[2];
            output "CH{i}_RXSIMPLEXPHYSTATUS";
            input "CH{i}_RXSLIDE";
            output "CH{i}_RXSLIDERDY";
            output "CH{i}_RXSLIPDONE";
            output "CH{i}_RXSTARTOFSEQ"[2];
            output "CH{i}_RXSTATUS"[3];
            input "CH{i}_RXSYNCALLIN";
            output "CH{i}_RXSYNCDONE";
            input "CH{i}_RXTERMINATION";
            input "CH{i}_RXUSERRDY";
            input "CH{i}_RXUSRCLK";
            output "CH{i}_RXVALID";

            output "CH{i}_TX10GSTAT";
            output "CH{i}_TXBUFSTATUS"[2];
            output "CH{i}_TXCOMFINISH";
            input "CH{i}_TXCOMINIT";
            input "CH{i}_TXCOMSAS";
            input "CH{i}_TXCOMWAKE";
            input "CH{i}_TXCTRL0"[16];
            input "CH{i}_TXCTRL1"[16];
            input "CH{i}_TXCTRL2"[8];
            input "CH{i}_TXDAPICODEOVRDEN";
            input "CH{i}_TXDAPICODERESET";
            input "CH{i}_TXDAPIRESET";
            output "CH{i}_TXDAPIRESETDONE";
            input "CH{i}_TXDAPIRESETMASK"[2];
            input "CH{i}_TXDATA"[128];
            output "CH{i}_TXDCCDONE";
            output "CH{i}_TXDEBUGPCSOUT";
            input "CH{i}_TXDEEMPH"[2];
            input "CH{i}_TXDETECTRX";
            input "CH{i}_TXDIFFCTRL"[5];
            output "CH{i}_TXDLYALIGNERR";
            output "CH{i}_TXDLYALIGNPROG";
            input "CH{i}_TXDLYALIGNREQ";
            input "CH{i}_TXELECIDLE";
            input "CH{i}_TXHEADER"[6];
            input "CH{i}_TXINHIBIT";
            input "CH{i}_TXLATCLK";
            input "CH{i}_TXMAINCURSOR"[7];
            input "CH{i}_TXMARGIN"[3];
            input "CH{i}_TXMLDCHAINDONE";
            input "CH{i}_TXMLDCHAINREQ";
            input "CH{i}_TXONESZEROS";
            output "CH{i}_TXOUTCLK";
            input "CH{i}_TXPAUSEDELAYALIGN";
            input "CH{i}_TXPCSRESETMASK";
            input "CH{i}_TXPD"[2];
            output "CH{i}_TXPHALIGNDONE";
            output "CH{i}_TXPHALIGNERR";
            output "CH{i}_TXPHALIGNOUTRSVD";
            input "CH{i}_TXPHALIGNREQ";
            input "CH{i}_TXPHALIGNRESETMASK"[2];
            input "CH{i}_TXPHDLYPD";
            input "CH{i}_TXPHDLYRESET";
            output "CH{i}_TXPHDLYRESETDONE";
            input "CH{i}_TXPHDLYTSTCLK";
            output "CH{i}_TXPHSETINITDONE";
            input "CH{i}_TXPHSETINITREQ";
            input "CH{i}_TXPHSHIFT180";
            output "CH{i}_TXPHSHIFT180DONE";
            input "CH{i}_TXPICODEOVRDEN";
            input "CH{i}_TXPICODERESET";
            input "CH{i}_TXPIPPMEN";
            input "CH{i}_TXPIPPMSTEPSIZE"[5];
            input "CH{i}_TXPISOPD";
            output "CH{i}_TXPMARESETDONE";
            input "CH{i}_TXPMARESETMASK"[3];
            input "CH{i}_TXPOLARITY";
            input "CH{i}_TXPOSTCURSOR"[5];
            input "CH{i}_TXPRBSFORCEERR";
            input "CH{i}_TXPRBSSEL"[4];
            input "CH{i}_TXPRECURSOR"[5];
            input "CH{i}_TXPROGDIVRESET";
            output "CH{i}_TXPROGDIVRESETDONE";
            input "CH{i}_TXQPIBIASEN";
            output "CH{i}_TXQPISENN";
            output "CH{i}_TXQPISENP";
            input "CH{i}_TXQPIWEAKPU";
            input "CH{i}_TXRATE"[8];
            output "CH{i}_TXRESETDONE";
            input "CH{i}_TXRESETMODE"[2];
            input "CH{i}_TXSEQUENCE"[7];
            output "CH{i}_TXSIMPLEXPHYSTATUS";
            input "CH{i}_TXSWING";
            output "CH{i}_TXSWINGOUTHIGH";
            output "CH{i}_TXSWINGOUTLOW";
            input "CH{i}_TXSYNCALLIN";
            output "CH{i}_TXSYNCDONE";
            input "CH{i}_TXUSERRDY";
            input "CH{i}_TXUSRCLK";
            output "CH{i}_XPIPE5_PIPELINE_EN";

            input "CH{i}_CSSDSTOPCLK";
            input "CH{i}_EDTUPDATEB";
            input "CH{i}_SCANCLKB";
            input "CH{i}_SCANRSTB";
            input "CH{i}_SCANENB";
            input "CH{i}_SCANCNTRLIN";
            output "CH{i}_SCANCNTRLOUT";
            input "CH{i}_SCANIN"[4];
            output "CH{i}_SCANOUT"[4];
            input "CH{i}_TSTCLK0";
            input "CH{i}_TSTCLK1";
        }

        for i in 0..2 {
            output "HSCLK{i}_LCPLLFBCLKLOST";
            input "HSCLK{i}_LCPLLFBDIV"[8];
            input "HSCLK{i}_LCPLLGTGREFCLK";
            output "HSCLK{i}_LCPLLLOCK";
            input "HSCLK{i}_LCPLLPD";
            output "HSCLK{i}_LCPLLREFCLKLOST";
            output "HSCLK{i}_LCPLLREFCLKMONITOR";
            input "HSCLK{i}_LCPLLREFCLKSEL"[3];
            input "HSCLK{i}_LCPLLRESET";
            input "HSCLK{i}_LCPLLRESETBYPASSMODE";
            input "HSCLK{i}_LCPLLRESETMASK"[2];
            input "HSCLK{i}_LCPLLSDMDATA"[26];
            input "HSCLK{i}_LCPLLSDMTOGGLE";

            output "HSCLK{i}_RPLLFBCLKLOST";
            input "HSCLK{i}_RPLLFBDIV"[8];
            input "HSCLK{i}_RPLLGTGREFCLK";
            output "HSCLK{i}_RPLLLOCK";
            input "HSCLK{i}_RPLLPD";
            output "HSCLK{i}_RPLLREFCLKLOST";
            output "HSCLK{i}_RPLLREFCLKMONITOR";
            input "HSCLK{i}_RPLLREFCLKSEL"[3];
            input "HSCLK{i}_RPLLRESET";
            input "HSCLK{i}_RPLLRESETBYPASSMODE";
            input "HSCLK{i}_RPLLRESETMASK"[2];
            input "HSCLK{i}_RPLLSDMDATA"[26];
            input "HSCLK{i}_RPLLSDMTOGGLE";
        }

        input BGBYPASSB;
        input BGMONITORENB;
        input BGPDB;
        input BGRCALOVRDENB;
        input BGRCALOVRD[5];

        input COESTATUSDEBUG;

        output CORRECTERR;
        output UNCORRECTERR;

        input CTRLRSVDIN[8];
        output CTRLRSVDOUT[8];

        input DEBUGTRACECLK;
        input DEBUGTRACEREADY;
        output DEBUGTRACETVALID;
        output DEBUGTRACETDATA[16];

        input GPI[32];
        output GPO[32];

        output GTPOWERGOOD;

        output MSTRXRESETDONE[4];
        input MSTRXRESET[4];
        output MSTTXRESETDONE[4];
        input MSTTXRESET[4];

        input PCIELINKREACHTARGET;
        input PCIELTSSM[6];

        output RCALCMP;
        input RCALENB;
        output RCALOUT[5];

        input RXMARGINCLK;
        output RXMARGINREQACK;
        input RXMARGINREQCMD[4];
        input RXMARGINREQLANENUM[2];
        input RXMARGINREQPAYLD[8];
        input RXMARGINREQREQ;
        input RXMARGINRESACK;
        output RXMARGINRESCMD[4];
        output RXMARGINRESLANENUM[2];
        output RXMARGINRESPAYLD[8];
        output RXMARGINRESREQ;

        input TRIGIN0;
        output TRIGACKIN0;
        output TRIGOUT0;
        input TRIGACKOUT0;

        input UBENABLE;
        input UBMBRST;
        input UBIOLMBRST;
        input UBINTR[12];
        output UBINTERRUPT;
        input UBRXUART;
        output UBTXUART;

        input CSSDSTOPCLK;
        input EDTUPDATEB;
        input SCANCLKB;
        input SCANRSTB;
        input SCANENB;
        input SCANCNTRLIN;
        output SCANCNTRLOUT;
        input SCANIN[4];
        output SCANOUT[4];
    }

    bel_class GTM {
        input APB3CLK;
        input APB3PRESETN;
        input APB3PENABLE;
        input APB3PWRITE;
        input APB3PSEL;
        input APB3PADDR[16];
        input APB3PWDATA[32];
        output APB3PREADY;
        output APB3PRDATA[32];
        output APB3PSLVERR;

        input AXISCLK;
        for i in 0..4 {
            output "S{i}_AXIS_TREADY";
            input "S{i}_AXIS_TVALID";
            input "S{i}_AXIS_TDATA"[32];
            input "S{i}_AXIS_TLAST";

            input "M{i}_AXIS_TREADY";
            output "M{i}_AXIS_TVALID";
            output "M{i}_AXIS_TDATA"[32];
            output "M{i}_AXIS_TLAST";
        }

        for i in 0..2 {
            input "REFCLK{i}_CLKTESTSIG";
            input "REFCLK{i}_GTREFCLKPD";
            output "REFCLK{i}_ODIV2";
        }

        for i in 0..4 {
            input "CH{i}_BSR_SERIAL";
            input "CH{i}_CDRFREQOS";
            input "CH{i}_CDRINCPCTRL";
            input "CH{i}_CDRSTEPDIR";
            input "CH{i}_CDRSTEPSQ";
            input "CH{i}_CDRSTEPSX";
            input "CH{i}_CLKRSVD0";
            input "CH{i}_CLKRSVD1";
            input "CH{i}_DMONFIFORESET";
            input "CH{i}_DMONITORCLK";
            output "CH{i}_DMONITOROUTCLK";
            output "CH{i}_DMONITOROUT"[32];
            output "CH{i}_EYESCANDATAERROR";
            input "CH{i}_EYESCANRESET";
            input "CH{i}_EYESCANTRIGGER";
            input "CH{i}_GTRSVD"[16];
            input "CH{i}_GTRXRESET";
            input "CH{i}_GTTXRESET";
            input "CH{i}_ILORESET";
            output "CH{i}_ILORESETDONE";
            input "CH{i}_ILORESETMASK";
            input "CH{i}_LOOPBACK"[3];
            input "CH{i}_PCSRSVDIN"[16];
            output "CH{i}_PCSRSVDOUT"[16];
            output "CH{i}_PINRSVDAS"[16];
            output "CH{i}_REFDEBUGOUT"[2];
            output "CH{i}_RESETEXCEPTION";

            output "CH{i}_RXBUFSTATUS"[3];
            input "CH{i}_RXCDRHOLD";
            output "CH{i}_RXCDRLOCK";
            input "CH{i}_RXCDROVRDEN";
            output "CH{i}_RXCDRPHDONE";
            input "CH{i}_RXCDRRESET";
            output "CH{i}_RXDATA"[256];
            output "CH{i}_RXDEBUGPCSOUT";
            input "CH{i}_RXLATCLK";
            output "CH{i}_RXOUTCLK";
            input "CH{i}_RXPCSRESETMASK"[3];
            input "CH{i}_RXPD"[2];
            input "CH{i}_RXPERST";
            output "CH{i}_RXPMARESETDONE";
            input "CH{i}_RXPMARESETMASK"[8];
            input "CH{i}_RXPOLARITY";
            input "CH{i}_RXPRBSCNTRESET";
            input "CH{i}_RXPRBSCNTSTOP";
            output "CH{i}_RXPRBSERR";
            output "CH{i}_RXPRBSLOCKED";
            input "CH{i}_RXPRBSSEL"[4];
            input "CH{i}_RXPROGDIVRESET";
            output "CH{i}_RXPROGDIVRESETDONE";
            input "CH{i}_RXQPRBSEN";
            input "CH{i}_RXRATE"[8];
            output "CH{i}_RXRESETDONE";
            input "CH{i}_RXRESETMODE"[2];
            input "CH{i}_RXRSV0";
            input "CH{i}_RXRSV1";
            input "CH{i}_RXRSV2";
            input "CH{i}_RXUSERRDY";
            input "CH{i}_RXUSRCLK";

            output "CH{i}_TXBUFSTATUS"[3];
            input "CH{i}_TXDATA"[256];
            output "CH{i}_TXDCCDONE";
            output "CH{i}_TXDEBUGPCSOUT";
            input "CH{i}_TXINHIBIT";
            input "CH{i}_TXLATCLK";
            input "CH{i}_TXMAINCURSOR"[7];
            output "CH{i}_TXOUTCLK";
            input "CH{i}_TXPCSRESETMASK";
            input "CH{i}_TXPD"[2];
            input "CH{i}_TXPERST";
            input "CH{i}_TXPISOPD";
            output "CH{i}_TXPMARESETDONE";
            input "CH{i}_TXPMARESETMASK"[2];
            input "CH{i}_TXPOLARITY";
            input "CH{i}_TXPOSTCURSOR"[6];
            input "CH{i}_TXPRBSFORCEERR";
            input "CH{i}_TXPRBSSEL"[4];
            input "CH{i}_TXPRECURSOR"[6];
            input "CH{i}_TXPRECURSOR2"[6];
            input "CH{i}_TXPRECURSOR3"[6];
            input "CH{i}_TXPROGDIVRESET";
            output "CH{i}_TXPROGDIVRESETDONE";
            input "CH{i}_TXQPRBSEN";
            input "CH{i}_TXRATE"[8];
            output "CH{i}_TXRESETDONE";
            input "CH{i}_TXRESETMODE"[2];
            input "CH{i}_TXRSV0"[5];
            input "CH{i}_TXRSV1"[4];
            input "CH{i}_TXUSERRDY";
            input "CH{i}_TXUSRCLK";

            input "CH{i}_CSSDSTOPCLKB";
            input "CH{i}_SCANCLKB";
            input "CH{i}_SCANRSTB";
            input "CH{i}_SCANENB";
            input "CH{i}_EDTUPDATEB";
            input "CH{i}_SCANCNTRLIN";
            output "CH{i}_SCANCNTRLOUT";
            input "CH{i}_SCANIN"[4];
            output "CH{i}_SCANOUT"[4];
            input "CH{i}_TSTCLK0";
            input "CH{i}_TSTCLK1";
        }

        for i in 0..2 {
            output "HSCLK{i}_LCPLLFBCLKLOST";
            input "HSCLK{i}_LCPLLFBDIV"[8];
            input "HSCLK{i}_LCPLLGTGREFCLK";
            output "HSCLK{i}_LCPLLLOCK";
            input "HSCLK{i}_LCPLLPD";
            output "HSCLK{i}_LCPLLREFCLKLOST";
            output "HSCLK{i}_LCPLLREFCLKMONITOR";
            input "HSCLK{i}_LCPLLREFCLKSEL"[3];
            input "HSCLK{i}_LCPLLRESET";
            input "HSCLK{i}_LCPLLRESETBYPASSMODE";
            input "HSCLK{i}_LCPLLRESETMASK"[2];
            input "HSCLK{i}_LCPLLSDMDATA"[26];
            input "HSCLK{i}_LCPLLSDMTOGGLE";
        }

        input BGBYPASSB;
        input BGMONITORENB;
        input BGPDB;
        input BGRCALOVRDENB;
        input BGRCALOVRD[5];

        input COESTATUSDEBUG;

        output CORRECTERR;
        output UNCORRECTERR;

        input CTRLRSVDIN[8];
        output CTRLRSVDOUT[8];

        input DEBUGTRACECLK;
        input DEBUGTRACEREADY;
        output DEBUGTRACETVALID;
        output DEBUGTRACETDATA[16];

        input GPI[32];
        output GPO[32];

        output GTPOWERGOOD;

        input MSTRXRESET[4];
        output MSTRXRESETDONE[4];
        input MSTTXRESET[4];
        output MSTTXRESETDONE[4];

        output RCALCMP;
        output RCALOUT[5];

        input TRIGIN0;
        output TRIGACKIN0;
        output TRIGOUT0;
        input TRIGACKOUT0;

        input UBENABLE;
        input UBMBRST;
        input UBIOLMBRST;
        input UBINTR[12];
        output UBINTERRUPT;
        input UBRXUART;
        output UBTXUART;

        input CSSDSTOPCLKB;
        input EDTUPDATEB;
        input SCANCLKB;
        input SCANRSTB;
        input SCANENB;
        input SCANCNTRLIN;
        output SCANCNTRLOUT;
        input SCANIN[4];
        output SCANOUT[4];
    }

    bel_class GTM2 {
        for i in 0..2 {
            input "REFCLK{i}_CLKTESTSIG";
            output "REFCLK{i}_ODIV2";
        }

        for i in 0..4 {
            input "CH{i}_BSR_SERIAL";

            output "CH{i}_BUFGTCE";
            output "CH{i}_BUFGTCEMASK"[4];
            output "CH{i}_BUFGTDIV"[12];
            output "CH{i}_BUFGTRST";
            output "CH{i}_BUFGTRSTMASK"[4];

            input "CH{i}_DMONFIFORESET";
            input "CH{i}_DMONITORCLK";
            output "CH{i}_DMONITOROUTCLK";
            output "CH{i}_DMONITOROUT"[32];

            input "CH{i}_GTRXRESET";
            input "CH{i}_GTTXRESET";
            input "CH{i}_HSDPPCSRESET";
            input "CH{i}_MNGPWRTOKENIN";
            output "CH{i}_MNGPWRTOKENOUT";
            input "CH{i}_PCIE_LTSSM_STATE"[8];
            output "CH{i}_PHYREADY";
            output "CH{i}_PHYSTATUS";

            output "CH{i}_RXBUFSTATUS"[3];
            output "CH{i}_RXBYTEISALIGNED";
            output "CH{i}_RXBYTEREALIGN";
            input "CH{i}_RXCDRHOLD";
            output "CH{i}_RXCHANBONDSEQ";
            input "CH{i}_RXCHBONDI"[5];
            output "CH{i}_RXCHBONDO"[5];
            output "CH{i}_RXCOMMADET";
            output "CH{i}_RXDATA"[320];
            output "CH{i}_RXELECIDLE";
            input "CH{i}_RXGEARBOXSLIP";
            input "CH{i}_RXLATCLK";
            output "CH{i}_RXOUTCLK";
            output "CH{i}_RXOUTCLKVALID";
            input "CH{i}_RXPOLARITY";
            output "CH{i}_RXRESETDONE";
            input "CH{i}_RXSLIDE";
            output "CH{i}_RXSLIDERDY";
            output "CH{i}_RXSLIPDONE";
            output "CH{i}_RXSSCLK";
            output "CH{i}_RXSTATUS"[3];
            input "CH{i}_RXTERMINATION";
            input "CH{i}_RXUSRCLK";
            output "CH{i}_RXVALID";

            input "CH{i}_TSTCLK0";
            input "CH{i}_TSTCLK1";

            output "CH{i}_TXBUFSTATUS"[2];
            output "CH{i}_TXCOMFINISH";
            input "CH{i}_TXDATA"[320];
            input "CH{i}_TXDETECTRXLOOPBACK";
            input "CH{i}_TXELECIDLE";
            input "CH{i}_TXEMPMAIN"[6];
            input "CH{i}_TXEMPPOS"[6];
            input "CH{i}_TXEMPPRE"[6];
            input "CH{i}_TXLATCLK";
            output "CH{i}_TXOUTCLK";
            input "CH{i}_TXPOWERDOWN"[2];
            input "CH{i}_TXRATE"[8];
            output "CH{i}_TXRESETDONE";
            input "CH{i}_TXUSRCLK";

            input "CH{i}_UPI2CMDREQ";
            input "CH{i}_UPI2CMDCODE"[20];
            output "CH{i}_UPI2CMDERROR";
            output "CH{i}_UPI2CMDREADY";
            output "CH{i}_UPI2CMDRESP"[32];
            output "CH{i}_UPI2MSGREQ";

            input "CH{i}_CSSDSTOPCLKB";
            input "CH{i}_EDTUPDATEB";
            input "CH{i}_SCANCLKB";
            input "CH{i}_SCANRSTB";
            input "CH{i}_SCANENB";
            input "CH{i}_SCANCNTRLIN";
            output "CH{i}_SCANCNTRLOUT";
            input "CH{i}_SCANIN"[4];
            input "CH{i}_SCANCHNLMASKIN"[4];
            output "CH{i}_SCANOUT"[4];
            input "CH{i}_SCANODCCCHNLMASK";
        }

        for i in 0..2 {
            output "HSCLK{i}_LCPLLFREQLOCK";
            input "HSCLK{i}_LCPLLGTGREFCLK";
            output "HSCLK{i}_LCPLLREFCLKMONITOR";
            input "HSCLK{i}_LCPLLSDMDATA"[26];
            input "HSCLK{i}_LCPLLSDMTOGGLE";

            output "HSCLK{i}_RPLLFREQLOCK";
            input "HSCLK{i}_RPLLGTGREFCLK";
            output "HSCLK{i}_RPLLREFCLKMONITOR";
        }

        output BUFG_GT_RSVD;

        input GPI[32];
        output GPO[32];

        output GTPOWERGOOD;

        output MCAEVENTVAL;
        output MCAEVENT[16];

        input PCIELINKREACHTARGET;

        input RXMARGINCLK;
        output RXMARGINREQACK;
        input RXMARGINREQCMD[4];
        input RXMARGINREQLANENUM[2];
        input RXMARGINREQPAYLOAD[8];
        input RXMARGINREQREQ;
        input RXMARGINRESACK;
        output RXMARGINRESCMD[4];
        output RXMARGINRESLANENUM[2];
        output RXMARGINRESPAYLOAD[8];
        output RXMARGINRESREQ;

        input CSSDSTOPCLKB;
        input SCANCLKB;
        input SCANRSTB;
        input SCANENB;
        input SCANEDTUPDTB;
        input SCANCNTRLIN;
        output SCANCNTRLOUT;
        input SCANIN[4];
        input SCANCHNLMASKIN[4];
        output SCANOUT[4];
        input SCANODCCCHNLMASKIN;
    }

    bel_class XPIPE {
        output TXOUTCLKDBG;
    }

    bel_class HNICPIPE {
        input CSSD_TRIG_N;
        input EDT_UPDT_N;
        input SCAN_CLK_N;
        input SCAN_RST_N;
        input SCAN_EN_N;
        input SCAN_CNTRL_CHNL_IN;
        input SCAN_CHNL_IN[3];
        input SCAN_CHNL_MASK_IN[3];
    }

    bel_class RFADC {
        input PCLK;
        input PRESET_N;
        input PENABLE;
        input PWRITE;
        input PSEL;
        input PADDR[16];
        input PWDATA[32];
        output PREADY;
        output PRDATA[32];
        output PSLVERR;

        input FABRIC_CLK;
        input FABRIC_CLK_IO[4];
        input FABRIC_CLK_OBS;
        input FABRIC_CLK_OBS_IO[4];

        output CLK_ADC;
        output CLK_ADC_SPARE;

        input PLL_MONCLK;
        input PLL_REFCLK_IN;

        output PLL_DMON_OUT;
        output PLL_DMON_OUT_SPARE;

        input CONTROL_COMMON[16];
        output STATUS_COMMON[24];

        for i in 0..4 {
            input "CONTROL_ADC{i}"[23];
            output "STATUS_ADC{i}"[50];

            output "DATA_ADC{i}"[256];
        }

        input DLC_CLK;
        input DLC_COEFF[41];
        output DLC_VALID;
        output DLC_DATA[273];

        output TEST_STATUS[16];

        input CSSD_TRIG_N;
        input SCAN_CLK_N_IN_EXT;
        input SCAN_MODE_RST_N_IN_EXT;
        input SCAN_EN_N_IN_EXT;
        input SCAN_EDT_UPDT_N_IN_EXT;
        input SCAN_CHNL_IN_EXT[15];
        output SCAN_CHNL_OUT_EXT[15];
        input SCAN_CNTRL_CHNL_IN_EXT;
        output SCAN_CNTRL_CHNL_OUT_EXT;
    }

    bel_class RFDAC {
        input PCLK;
        input PRESET_N;
        input PENABLE;
        input PWRITE;
        input PSEL;
        input PADDR[16];
        input PWDATA[32];
        output PREADY;
        output PRDATA[32];
        output PSLVERR;

        input FABRIC_CLK;
        input FABRIC_CLK_IO[4];

        output CLK_DAC;
        output CLK_DAC_SPARE;
        output CLK_CAL;
        output CLK_CAL_SPARE;

        input PLL_MONCLK;
        input PLL_REFCLK_IN;

        output PLL_DMON_OUT;
        output PLL_DMON_OUT_SPARE;

        input CONTROL_COMMON[16];
        output STATUS_COMMON[24];

        for i in 0..4 {
            input "CONTROL_DAC{i}"[22];
            output "STATUS_DAC{i}"[24];

            input "DATA_DAC{i}"[512];

            output "CSA_CAL_CONTROL_DAC{i}"[8];
            output "CSA_CAL_DATA_DAC{i}"[8];
            output "TM_CAL_CONTROL_DAC{i}"[8];
            output "TM_CAL_DATA_DAC{i}"[8];
        }

        output TEST_STATUS[16];

        input CSSD_TRIG_N;
        input SCAN_CLK_N_IN_EXT;
        input SCAN_MODE_RST_N_IN_EXT;
        input SCAN_EN_N_IN_EXT;
        input SCAN_EDT_UPDT_N_IN_EXT;
        input SCAN_CHNL_IN_EXT[15];
        output SCAN_CHNL_OUT_EXT[15];
        input SCAN_CNTRL_CHNL_IN_EXT;
        output SCAN_CNTRL_CHNL_OUT_EXT;
    }

    bel_class VDU {
        input VDUCORECLK;
        input VDUMCUCLK;

        input PLVDUCORECLK;
        input PLVDUMCUCLK;
        input PLVDURAWRSTN;

        output VDUPLCORESTATUSCLKPLL;
        output VDUPLMCUSTATUSCLKPLL;

        output VDUPLPINTREQ;

        output VDUPLPORSTATUSLVLSHPL;
        output VDUPLPWRSUPPLYSTATUSLVLSHPL;

        // decoder AXI out

        input PLVDUAXIDECCLK;

        for i in 0..2 {
            input "IFVDUPLDECAXI{i}ARREADY";
            output "IFVDUPLDECAXI{i}ARVALID";
            output "IFVDUPLDECAXI{i}ARADDR"[64];
            output "IFVDUPLDECAXI{i}ARBURST"[2];
            output "IFVDUPLDECAXI{i}ARCACHE"[4];
            output "IFVDUPLDECAXI{i}ARID"[16];
            output "IFVDUPLDECAXI{i}ARLEN"[8];
            output "IFVDUPLDECAXI{i}ARLOCK"[1];
            output "IFVDUPLDECAXI{i}ARPROT"[3];
            output "IFVDUPLDECAXI{i}ARQOS"[4];
            output "IFVDUPLDECAXI{i}ARREGION"[4];
            output "IFVDUPLDECAXI{i}ARSIZE"[3];
            output "IFVDUPLDECAXI{i}ARUSER"[18];

            input "IFVDUPLDECAXI{i}AWREADY";
            output "IFVDUPLDECAXI{i}AWVALID";
            output "IFVDUPLDECAXI{i}AWADDR"[64];
            output "IFVDUPLDECAXI{i}AWBURST"[2];
            output "IFVDUPLDECAXI{i}AWCACHE"[4];
            output "IFVDUPLDECAXI{i}AWID"[16];
            output "IFVDUPLDECAXI{i}AWLEN"[8];
            output "IFVDUPLDECAXI{i}AWLOCK"[1];
            output "IFVDUPLDECAXI{i}AWPROT"[3];
            output "IFVDUPLDECAXI{i}AWQOS"[4];
            output "IFVDUPLDECAXI{i}AWREGION"[4];
            output "IFVDUPLDECAXI{i}AWSIZE"[3];
            output "IFVDUPLDECAXI{i}AWUSER"[18];

            output "IFVDUPLDECAXI{i}BREADY";
            input "IFVDUPLDECAXI{i}BVALID";
            input "IFVDUPLDECAXI{i}BID"[16];
            input "IFVDUPLDECAXI{i}BRESP"[2];
            input "IFVDUPLDECAXI{i}BUSER"[16];

            output "IFVDUPLDECAXI{i}RREADY";
            input "IFVDUPLDECAXI{i}RVALID";
            input "IFVDUPLDECAXI{i}RDATA"[128];
            input "IFVDUPLDECAXI{i}RID"[16];
            input "IFVDUPLDECAXI{i}RLAST"[1];
            input "IFVDUPLDECAXI{i}RRESP"[2];
            input "IFVDUPLDECAXI{i}RUSER"[17];

            input "IFVDUPLDECAXI{i}WREADY";
            output "IFVDUPLDECAXI{i}WVALID";
            output "IFVDUPLDECAXI{i}WDATA"[128];
            output "IFVDUPLDECAXI{i}WID"[16];
            output "IFVDUPLDECAXI{i}WLAST"[1];
            output "IFVDUPLDECAXI{i}WSTRB"[16];
            output "IFVDUPLDECAXI{i}WUSER"[17];
        }

        // MCU axi out

        input PLVDUAXIMCUCLK;

        input IFVDUPLMCUAXIARREADY;
        output IFVDUPLMCUAXIARVALID;
        output IFVDUPLMCUAXIARADDR[64];
        output IFVDUPLMCUAXIARBURST[2];
        output IFVDUPLMCUAXIARCACHE[4];
        output IFVDUPLMCUAXIARID[16];
        output IFVDUPLMCUAXIARLEN[8];
        output IFVDUPLMCUAXIARLOCK[1];
        output IFVDUPLMCUAXIARPROT[3];
        output IFVDUPLMCUAXIARQOS[4];
        output IFVDUPLMCUAXIARREGION[4];
        output IFVDUPLMCUAXIARSIZE[3];
        output IFVDUPLMCUAXIARUSER[16];

        input IFVDUPLMCUAXIAWREADY;
        output IFVDUPLMCUAXIAWVALID;
        output IFVDUPLMCUAXIAWADDR[64];
        output IFVDUPLMCUAXIAWBURST[2];
        output IFVDUPLMCUAXIAWCACHE[4];
        output IFVDUPLMCUAXIAWID[16];
        output IFVDUPLMCUAXIAWLEN[8];
        output IFVDUPLMCUAXIAWLOCK[1];
        output IFVDUPLMCUAXIAWPROT[3];
        output IFVDUPLMCUAXIAWQOS[4];
        output IFVDUPLMCUAXIAWREGION[4];
        output IFVDUPLMCUAXIAWSIZE[3];
        output IFVDUPLMCUAXIAWUSER[16];

        output IFVDUPLMCUAXIBREADY;
        input IFVDUPLMCUAXIBVALID;
        input IFVDUPLMCUAXIBID[16];
        input IFVDUPLMCUAXIBRESP[2];
        input IFVDUPLMCUAXIBUSER[16];

        output IFVDUPLMCUAXIRREADY;
        input IFVDUPLMCUAXIRVALID;
        input IFVDUPLMCUAXIRDATA[32];
        input IFVDUPLMCUAXIRID[16];
        input IFVDUPLMCUAXIRLAST[1];
        input IFVDUPLMCUAXIRRESP[2];
        input IFVDUPLMCUAXIRUSER[16];

        input IFVDUPLMCUAXIWREADY;
        output IFVDUPLMCUAXIWVALID;
        output IFVDUPLMCUAXIWDATA[32];
        output IFVDUPLMCUAXIWID[16];
        output IFVDUPLMCUAXIWLAST[1];
        output IFVDUPLMCUAXIWSTRB[4];
        output IFVDUPLMCUAXIWUSER[16];

        // AXI-lite in

        input PLVDUAXILITECLK;

        output IFPLVDUAXILITEAPBARREADY;
        input IFPLVDUAXILITEAPBARVALID;
        input IFPLVDUAXILITEAPBARADDR[32];
        input IFPLVDUAXILITEAPBARPROT[3];

        output IFPLVDUAXILITEAPBAWREADY;
        input IFPLVDUAXILITEAPBAWVALID;
        input IFPLVDUAXILITEAPBAWADDR[32];
        input IFPLVDUAXILITEAPBAWPROT[3];

        input IFPLVDUAXILITEAPBBREADY;
        output IFPLVDUAXILITEAPBBVALID;
        output IFPLVDUAXILITEAPBBRESP[2];

        input IFPLVDUAXILITEAPBRREADY;
        output IFPLVDUAXILITEAPBRVALID;
        output IFPLVDUAXILITEAPBRDATA[32];
        output IFPLVDUAXILITEAPBRRESP[2];

        output IFPLVDUAXILITEAPBWREADY;
        input IFPLVDUAXILITEAPBWVALID;
        input IFPLVDUAXILITEAPBWDATA[32];
        input IFPLVDUAXILITEAPBWSTRB[4];

        // misc

        output ISO1VDUPL;
        output ISO2VDUPL;

        input PLVDUCSSDTRIGN[2];

        input PLVDUIOCHARDATAINSELN;
        input PLVDUIOCHARDECAXI0DATAIN;
        input PLVDUIOCHARDECAXI1DATAIN;
        input PLVDUIOCHARMCUAXIDATAIN;
        output VDUPLIOCHARDECAXI0DATAOUT;
        output VDUPLIOCHARDECAXI1DATAOUT;
        output VDUPLIOCHARMCUAXIDATAOUT;

        input PLVDUMCUVDECDEBUGCLK;
        input PLVDUMCUVDECDEBUGRST;
        input PLVDUMCUVDECDEBUGSYSRST;
        input PLVDUMCUVDECDEBUGCAPTURE;
        input PLVDUMCUVDECDEBUGSHIFT;
        input PLVDUMCUVDECDEBUGUPDATE;
        input PLVDUMCUVDECDEBUGREGEN[8];
        input PLVDUMCUVDECDEBUGTDI;
        output VDUPLMCUVDECDEBUGTDO;

        output VDUPLDEBUG[20];

        input PLVDUSCANCLKNEXT;
        input PLVDUSCANMODERSTNEXT;
        input PLVDUSCANENNEXT;
        input PLVDUSCANEDTUPDTNEXT;
        input PLVDUSCANCHNLINEXTTOP[9];
        input PLVDUSCANCNTRLCHNLINEXTTOP;
        output VDUPLSCANCHNLOUTEXTTOP[9];
        output VDUPLSCANCNTRLCHNLOUTEXTTOP;

        input PLVDUSPAREPORTIN1[6];
        input PLVDUSPAREPORTIN2[6];
        input PLVDUSPAREPORTIN3[6];
        input PLVDUSPAREPORTIN4[6];
        input PLVDUSPAREPORTIN5[6];
        input PLVDUSPAREPORTIN6[6];
        input PLVDUSPAREPORTIN7[6];
        input PLVDUSPAREPORTIN8[6];
        input PLVDUSPAREPORTIN9[6];
        input PLVDUSPAREPORTIN10[6];
        input PLVDUSPAREPORTIN11[6];
        input PLVDUSPAREPORTIN12[6];
        input PLVDUSPAREPORTIN13[6];
        output VDUPLSPAREPORTOUT1[2];
        output VDUPLSPAREPORTOUT2[2];
        output VDUPLSPAREPORTOUT3[2];
        output VDUPLSPAREPORTOUT4[2];
        output VDUPLSPAREPORTOUT5[2];
        output VDUPLSPAREPORTOUT6[2];
        output VDUPLSPAREPORTOUT7[2];
        output VDUPLSPAREPORTOUT8[2];
        output VDUPLSPAREPORTOUT9[6];
        output VDUPLSPAREPORTOUT10[6];
        output VDUPLSPAREPORTOUT11[6];
        output VDUPLSPAREPORTOUT12[6];
        output VDUPLSPAREPORTOUT13[6];
    }

    bel_class BFR_B {
        input CLK;
        input CLK_IO[48];
        input D_CLK;
        input RESET_N;

        input CTRL_DBL[14];
        output STATUS_DBL[15];

        for i in 0..8 {
            input "IF_M_AXIS_UNIT{i}_DOUT_TREADY";
            output "IF_M_AXIS_UNIT{i}_DOUT_TVALID";
            output "IF_M_AXIS_UNIT{i}_DOUT_TDATA"[256];
            output "IF_M_AXIS_UNIT{i}_DOUT_TUSER"[2];

            output "IF_S_AXIS_UNIT{i}_CTRL_A_TREADY";
            input "IF_S_AXIS_UNIT{i}_CTRL_A_TVALID";
            input "IF_S_AXIS_UNIT{i}_CTRL_A_TDATA"[38];

            output "IF_S_AXIS_UNIT{i}_CTRL_B_TREADY";
            input "IF_S_AXIS_UNIT{i}_CTRL_B_TVALID";
            input "IF_S_AXIS_UNIT{i}_CTRL_B_TDATA"[46];

            output "IF_S_AXIS_UNIT{i}_DIN_A_TREADY";
            input "IF_S_AXIS_UNIT{i}_DIN_A_TVALID";
            input "IF_S_AXIS_UNIT{i}_DIN_A_TDATA"[256];
            input "IF_S_AXIS_UNIT{i}_DIN_A_TKEEP"[2];

            output "IF_S_AXIS_UNIT{i}_DIN_B_TREADY";
            input "IF_S_AXIS_UNIT{i}_DIN_B_TVALID";
            input "IF_S_AXIS_UNIT{i}_DIN_B_TDATA"[128];
            input "IF_S_AXIS_UNIT{i}_DIN_B_TKEEP"[2];
            input "IF_S_AXIS_UNIT{i}_DIN_B_TUSER"[16];

            input "UNIT{i}_ENABLE";
            input "UNIT{i}_CTRL"[12];
            output "UNIT{i}_STATUS"[3];

            input "UNIT{i}_TEST_PIN_CHAR_IN";
            output "UNIT{i}_TEST_PIN_CHAR_OUT";
        }

        for i in 0..2 {
            output "IF_S_AXIS_CTRL_BCAST{i}_RD_TREADY";
            input "IF_S_AXIS_CTRL_BCAST{i}_RD_TVALID";
            input "IF_S_AXIS_CTRL_BCAST{i}_RD_TDATA"[40];

            output "IF_S_AXIS_CTRL_BCAST{i}_WR_TREADY";
            input "IF_S_AXIS_CTRL_BCAST{i}_WR_TVALID";
            input "IF_S_AXIS_CTRL_BCAST{i}_WR_TDATA"[40];
        }

        input S_AXI_PARAM_ACLK;
        input S_AXI_PARAM_ARESETN;

        output IF_S_AXI_PARAM_ARREADY;
        input IF_S_AXI_PARAM_ARVALID;
        input IF_S_AXI_PARAM_ARADDR[14];

        output IF_S_AXI_PARAM_AWREADY;
        input IF_S_AXI_PARAM_AWVALID;
        input IF_S_AXI_PARAM_AWADDR[14];

        input IF_S_AXI_PARAM_BREADY;
        output IF_S_AXI_PARAM_BVALID;

        input IF_S_AXI_PARAM_RREADY;
        output IF_S_AXI_PARAM_RVALID;
        output IF_S_AXI_PARAM_RDATA[16];

        output IF_S_AXI_PARAM_WREADY;
        input IF_S_AXI_PARAM_WVALID;
        input IF_S_AXI_PARAM_WDATA[16];

        input SHUTDOWN_IN;
        output SHUTDOWN_OUT;

        input TEST_BYPASS_N[3];
        input TEST_CSSD_TRIG_N;

        input SCAN_CLK_N_IN_EXT;
        input SCAN_MODE_RST_N_IN_EXT;
        input SCAN_EN_N_IN_EXT;
        input SCAN_EDT_UPDT_N_IN_EXT;
        input SCAN_CHNL_IN_EXT[6];
        output SCAN_CHNL_OUT_EXT[6];
        input SCAN_CNTRL_CHNL_IN_EXT;
        output SCAN_CNTRL_CHNL_OUT_EXT;
    }

    bel_class VCU2 {
        input VCU2AXISLAVECLK;
        input VCU2AXISLAVEDEBUGCLK;

        input VCU2DECCLK;
        input VCU2DECDEBUGCLK;

        input VCU2ENCCLK;
        input VCU2ENCDEBUGCLK;

        input PLVCU2RAWRSTN;

        output VCU2DECPINTBUS[32];
        output VCU2DECPINTREQ;

        output VCU2DECSYNCY[14];

        input VCU2ENCMCUACKNEXT;
        output VCU2ENCMCUACKPREV;
        output VCU2ENCMCUINTNEXT;
        input VCU2ENCMCUINTPREV;

        output VCU2ENCPINTBUS[32];
        output VCU2ENCPINTREQ;

        input VCU2ENCSYNCEOF[2];
        input VCU2ENCSYNCEOL[2];

        input VCU2DECMCUACKNEXT;
        output VCU2DECMCUACKPREV;
        output VCU2DECMCUINTNEXT;
        input VCU2DECMCUINTPREV;

        input PLVCU2SPAREIN[128];
        output PLVCU2SPAREOUT[128];
        input PLVCU2SPARE[5];
        output VCU2PLSPARE[5];

        input VCU2DECJTAGCLK;
        input VCU2DECJTAGMS;
        input VCU2DECJTAGDI;
        output VCU2DECJTAGDO;

        input VCU2ENCJTAGCLK;
        input VCU2ENCJTAGMS;
        input VCU2ENCJTAGDI;
        output VCU2ENCJTAGDO;

        // most likely this is just passed straight to NOC2_SCAN
        input PLVCU2SCANCSSDTRIGN;
        input PLVCU2SCANCLKN;
        input PLVCU2SCANMODERSTN;
        input PLVCU2SCANENN;
        input PLVCU2SCANEDTUPDTN;
        input PLVCU2SCANCHNLOUT[12];
        input PLVCU2SCANCHNLMASKOUT[12];
        output VCU2PLSCANCHNLIN[12];

        input PLVCU2SCANCLKNEXT;
        input PLVCU2SCANMODERSTNEXT;
        input PLVCU2SCANENNEXT;
        input PLVCU2SCANEDTUPDTNEXT;
        input PLVCU2SCANCHNLINEXTTOP[3];
        output VCU2PLSCANCHNLOUTEXTTOP[3];
        input PLVCU2SCANCNTRLCHNLINEXTTOP;
        output VCU2PLSCANCNTRLCHNLOUTEXTTOP;

        input VCU2DECDFXSCANCHNLINEXT[8];
        input VCU2DECDFXSCANCHNLMASKINEXT[8];
        output VCU2DECDFXSCANCHNLOUTEXT[8];
        input VCU2DECDFXSCANODCCCHNLINEXT;
        input VCU2DECDFXSCANODCCCHNLMASKINEXT;
        output VCU2DECDFXSCANODCCCHNLOUTEXT;
        output VCU2DECDFXUNEXPECTEDACTIVATION;

        input VCU2ENCCOREDFXSCANCHNLINEXT[24];
        input VCU2ENCCOREDFXSCANCHNLMASKINEXT[24];
        output VCU2ENCCOREDFXSCANCHNLOUTEXT[24];
        input VCU2ENCCOREDFXSCANODCCCHNLINEXT[2];
        input VCU2ENCCOREDFXSCANODCCCHNLMASKINEXT[2];
        output VCU2ENCCOREDFXSCANODCCCHNLOUTEXT[2];
        output VCU2ENCCOREDFXUNEXPECTEDACTIVATION[2];

        input VCU2INTWRAPCSSDTRIGN[8];
        input VCU2INTWRAPDFXSCANCHNLMASKINEXT[3];
        input VCU2INTWRAPDFXSCANODCCCHNLMASKINEXT;
        output VCU2INTWRAPDFXDFXUNEXPECTEDACTIVATION;
    }

    bel_class ISP2 {
        input PLISP2RAWRSTN;

        output FUSAISRIRQCORR;
        output FUSAISRIRQUNCOR;

        output ISPISRIRQ;
        output ISPXMPUINTERRUPT;

        for i in 0..2 {
            input "DPLLISPCLK{i}";
            input "PLDEBUGCLK{i}";
            input "PLISPVIDOUTCLK{i}";
            output "ISPPLCLK{i}";
            output "ISP{i}CORERESETNRSTMON";

            output "CLKOUT{i}"[8];

            output "ISP{i}FEIRQ";
            output "ISP{i}FUSAIRQ";
            output "ISP{i}FUSAURGENTIRQ";
            output "ISP{i}ISPIRQ";
            output "ISP{i}MIIRQ";

            for j in 1..3 {
                input "IFISP{i}{j}AXISOUTSAXISTREADY";
                output "IFISP{i}{j}AXISOUTSAXISTVALID";
                output "IFISP{i}{j}AXISOUTSAXISTDATA"[128];
                output "IFISP{i}{j}AXISOUTSAXISTDEST"[2];
                output "IFISP{i}{j}AXISOUTSAXISTKEEP"[6];
                output "IFISP{i}{j}AXISOUTSAXISTLAST";
                output "IFISP{i}{j}AXISOUTSAXISTUSER"[96];
            }
        }

        for i in 0..5 {
            input "PLISPVIDIN{i}CLK";

            input "IFISPMIPIVIDIN{i}HEADERVALID";
            input "IFISPMIPIVIDIN{i}HEADERDATA"[32];

            output "IFISPMIPIVIDIN{i}TREADY";
            input "IFISPMIPIVIDIN{i}TVALID";
            input "IFISPMIPIVIDIN{i}TDATA"[64];
            input "IFISPMIPIVIDIN{i}TDEST"[10];
            input "IFISPMIPIVIDIN{i}TLAST";
            input "IFISPMIPIVIDIN{i}TUSER"[96];
        }

        input PLISP2SPARE[3];
        output ISP2PLSPARE[3];
        input ISP2SPAREIN[200];
        output ISP2SPAREOUT[200];

        input ISP2CSSDTRIGN[8];

        input ISP2SCANCLKNEXT;
        input SCANMODERSTNEXT;
        input ISP2SCANENNEXT;
        input SCANEDTUPDTEXT;

        input ISP2COREDFXSCANODCCCHNLINEXT;
        input ISP2COREDFXSCANODCCCHNLMASKINEXT;
        output ISP2COREDFXSCANODCCCHNLOUTEXT;

        input ISP2CORESCANCHNLINEXT[4];
        input ISP2CORESCANCHNLMASKINEXT[4];
        output ISP2CORESCANCHNLOUTEXT[4];

        for i in 0..2 {
            input "ISP2INST{i}DFXSCANODCCCHNLINEXT";
            input "ISP2INST{i}DFXSCANODCCCHNLMASKINEXT";
            output "ISP2INST{i}DFXSCANODCCCHNLOUTEXT";

            input "ISP2INST{i}SCANCHNLINEXT"[9];
            input "ISP2INST{i}SCANCHNLMASKINEXT"[9];
            output "ISP2INST{i}SCANCHNLOUTEXT"[9];
        }

        // NOC2_SCAN

        input PLISP2SCANCHNLMASKOUT[7];
        input PLISP2SCANCHNLOUT[7];
        output ISP2PLSCANCHNLIN[7];
        input PLISP2SCANCLKN;
        input PLISP2SCANCSSDTRIGN;
        input PLISP2SCANEDTUPDTN;
        input PLISP2SCANENN;
        input PLISP2SCANMODERSTN;
    }

    bel_class PS9 {
        // Clock outputs (BUFG_PS)

        output PMCRCLKCLK[6];
        output LPDRCLKCLK[6];

        // Clock inputs

        input PLPMCAUXREFCLK;
        input PLLPDAUXREFCLK;
        input PLFPDAUXREFCLK;

        // Reset outputs

        output PLRST0N;
        output PLRST1N;
        output PLRST2N;
        output PLRST3N;

        // CPUs

        input PLPSAPUGICFIQ[2];
        input PLPSAPUGICIRQ[2];
        input PLPSEVENTI;
        output PSPLEVENTO;
        output PSPLSTANDBYWFE[2];
        output PSPLSTANDBYWFI[2];

        input NFIQ0LPDRPU;
        input NFIQ1LPDRPU;
        input NIRQ0LPDRPU;
        input NIRQ1LPDRPU;
        input RPUEVENTI0;
        input RPUEVENTI1;
        output RPUEVENTO0;
        output RPUEVENTO1;

        // IRQs

        input PMCPLIRQ[8];
        input PLPSIRQ0[8];
        input PLPSIRQ1[8];
        output PSPLIRQPMC[64];
        output PSPLIRQLPD[100];
        output PSPLIRQFPD[32];

        // GPIO

        input PMCPLGPI[32];
        output PMCPLGPO[32];

        // AXI

        // PL_ACE_FPD

        input PLACECLK;

        input SACEFPDACREADY;
        output SACEFPDACVALID;
        output SACEFPDACADDR[44];
        output SACEFPDACPROT[3];
        output SACEFPDACSNOOP[4];

        output SACEFPDARREADY;
        input SACEFPDARVALID;
        input SACEFPDARADDR[44];
        input SACEFPDARBAR[2];
        input SACEFPDARBURST[2];
        input SACEFPDARCACHE[4];
        input SACEFPDARDOMAIN[2];
        input SACEFPDARID[6];
        input SACEFPDARLEN[8];
        input SACEFPDARLOCK;
        input SACEFPDARPROT[3];
        input SACEFPDARQOS[4];
        input SACEFPDARREGION[4];
        input SACEFPDARSIZE[3];
        input SACEFPDARSNOOP[4];
        input SACEFPDARUSER[16];

        output SACEFPDAWREADY;
        input SACEFPDAWVALID;
        input SACEFPDAWADDR[44];
        input SACEFPDAWBAR[2];
        input SACEFPDAWBURST[2];
        input SACEFPDAWCACHE[4];
        input SACEFPDAWDOMAIN[2];
        input SACEFPDAWID[6];
        input SACEFPDAWLEN[8];
        input SACEFPDAWLOCK;
        input SACEFPDAWPROT[3];
        input SACEFPDAWQOS[4];
        input SACEFPDAWREGION[4];
        input SACEFPDAWSIZE[3];
        input SACEFPDAWSNOOP[3];
        input SACEFPDAWUSER[16];

        input SACEFPDBREADY;
        output SACEFPDBVALID;
        output SACEFPDBID[6];
        output SACEFPDBRESP[2];
        output SACEFPDBUSER;

        output SACEFPDCDREADY;
        input SACEFPDCDVALID;
        input SACEFPDCDDATA[128];
        input SACEFPDCDLAST;

        output SACEFPDCRREADY;
        input SACEFPDCRVALID;
        input SACEFPDCRRESP[5];

        input SACEFPDRREADY;
        output SACEFPDRVALID;
        input SACEFPDRACK;
        output SACEFPDRDATA[128];
        output SACEFPDRID[6];
        output SACEFPDRLAST;
        output SACEFPDRRESP[4];
        output SACEFPDRUSER;

        output SACEFPDWREADY;
        input SACEFPDWVALID;
        input SACEFPDWACK;
        input SACEFPDWDATA[128];
        input SACEFPDWLAST;
        input SACEFPDWSTRB[16];
        input SACEFPDWUSER;

        // PL_ACP_FPD

        input PLACPINACT;

        input SAXIACPACLK;

        output SAXIACPARREADY;
        input SAXIACPARVALID;
        input SAXIACPARADDR[44];
        input SAXIACPARCACHE[4];
        input SAXIACPARID[5];
        input SAXIACPARLEN[8];
        input SAXIACPARPROT[3];
        input SAXIACPARUSER[2];

        output SAXIACPAWREADY;
        input SAXIACPAWVALID;
        input SAXIACPAWADDR[44];
        input SAXIACPAWCACHE[4];
        input SAXIACPAWID[5];
        input SAXIACPAWLEN[8];
        input SAXIACPAWPROT[3];
        input SAXIACPAWUSER[2];

        input SAXIACPBREADY;
        output SAXIACPBVALID;
        output SAXIACPBID[5];
        output SAXIACPBRESP[2];

        input SAXIACPRREADY;
        output SAXIACPRVALID;
        output SAXIACPRDATA[128];
        output SAXIACPRID[5];
        output SAXIACPRLAST;
        output SAXIACPRRESP[2];

        output SAXIACPWREADY;
        input SAXIACPWVALID;
        input SAXIACPWDATA[128];
        input SAXIACPWLAST;
        input SAXIACPWSTRB[16];

        // PL_AXI_FPD

        input SAXIGP0RCLK;

        output SAXIGP0ARREADY;
        input SAXIGP0ARVALID;
        output SAXIGP0RACOUNT[4];
        input SAXIGP0ARADDR[49];
        input SAXIGP0ARBURST[2];
        input SAXIGP0ARCACHE[4];
        input SAXIGP0ARID[6];
        input SAXIGP0ARLEN[8];
        input SAXIGP0ARLOCK;
        input SAXIGP0ARPROT[3];
        input SAXIGP0ARQOS[4];
        input SAXIGP0ARSIZE[3];
        input SAXIGP0ARUSER[10];

        output SAXIGP0RVALID;
        input SAXIGP0RREADY;
        output SAXIGP0RCOUNT[8];
        output SAXIGP0RDATA[128];
        output SAXIGP0RID[6];
        output SAXIGP0RLAST;
        output SAXIGP0RRESP[2];

        input SAXIGP0WCLK;

        output SAXIGP0AWREADY;
        input SAXIGP0AWVALID;
        output SAXIGP0WACOUNT[4];
        input SAXIGP0AWADDR[49];
        input SAXIGP0AWBURST[2];
        input SAXIGP0AWCACHE[4];
        input SAXIGP0AWID[6];
        input SAXIGP0AWLEN[8];
        input SAXIGP0AWLOCK;
        input SAXIGP0AWPROT[3];
        input SAXIGP0AWQOS[4];
        input SAXIGP0AWSIZE[3];
        input SAXIGP0AWUSER[10];

        input SAXIGP0BREADY;
        output SAXIGP0BVALID;
        output SAXIGP0BID[6];
        output SAXIGP0BRESP[2];

        output SAXIGP0WREADY;
        input SAXIGP0WVALID;
        output SAXIGP0WCOUNT[8];
        input SAXIGP0WDATA[128];
        input SAXIGP0WLAST;
        input SAXIGP0WSTRB[16];

        // PL_ACELITE_FPD

        input SAXIGP2RCLK;

        output SAXIGP2ARREADY;
        input SAXIGP2ARVALID;
        output SAXIGP2RACOUNT[4];
        input SAXIGP2ARADDR[49];
        input SAXIGP2ARBURST[2];
        input SAXIGP2ARCACHE[4];
        input SAXIGP2ARID[6];
        input SAXIGP2ARLEN[8];
        input SAXIGP2ARLOCK;
        input SAXIGP2ARPROT[3];
        input SAXIGP2ARQOS[4];
        input SAXIGP2ARSIZE[3];
        input SAXIGP2ARUSER[18];

        input SAXIGP2RREADY;
        output SAXIGP2RVALID;
        output SAXIGP2RCOUNT[8];
        output SAXIGP2RDATA[128];
        output SAXIGP2RID[6];
        output SAXIGP2RLAST;
        output SAXIGP2RRESP[2];

        input SAXIGP2WCLK;

        output SAXIGP2AWREADY;
        input SAXIGP2AWVALID;
        output SAXIGP2WACOUNT[4];
        input SAXIGP2AWADDR[49];
        input SAXIGP2AWBURST[2];
        input SAXIGP2AWCACHE[4];
        input SAXIGP2AWID[6];
        input SAXIGP2AWLEN[8];
        input SAXIGP2AWLOCK;
        input SAXIGP2AWPROT[3];
        input SAXIGP2AWQOS[4];
        input SAXIGP2AWSIZE[3];
        input SAXIGP2AWUSER[18];

        input SAXIGP2BREADY;
        output SAXIGP2BVALID;
        output SAXIGP2BID[6];
        output SAXIGP2BRESP[2];

        output SAXIGP2WREADY;
        input SAXIGP2WVALID;
        output SAXIGP2WCOUNT[8];
        input SAXIGP2WDATA[128];
        input SAXIGP2WLAST;
        input SAXIGP2WSTRB[16];

        // PL_AXI_LPD

        input AXDS4COHDISABLE;

        input SAXIGP4RCLK;

        output SAXIGP4ARREADY;
        input SAXIGP4ARVALID;
        output SAXIGP4RACOUNT[4];
        input SAXIGP4ARADDR[49];
        input SAXIGP4ARBURST[2];
        input SAXIGP4ARCACHE[4];
        input SAXIGP4ARID[6];
        input SAXIGP4ARLEN[8];
        input SAXIGP4ARLOCK;
        input SAXIGP4ARPROT[3];
        input SAXIGP4ARQOS[4];
        input SAXIGP4ARSIZE[3];
        input SAXIGP4ARUSER[11];

        input SAXIGP4RREADY;
        output SAXIGP4RVALID;
        output SAXIGP4RCOUNT[8];
        output SAXIGP4RDATA[128];
        output SAXIGP4RID[6];
        output SAXIGP4RLAST;
        output SAXIGP4RRESP[2];

        input SAXIGP4WCLK;

        output SAXIGP4AWREADY;
        input SAXIGP4AWVALID;
        output SAXIGP4WACOUNT[4];
        input SAXIGP4AWADDR[49];
        input SAXIGP4AWBURST[2];
        input SAXIGP4AWCACHE[4];
        input SAXIGP4AWID[6];
        input SAXIGP4AWLEN[8];
        input SAXIGP4AWLOCK;
        input SAXIGP4AWPROT[3];
        input SAXIGP4AWQOS[4];
        input SAXIGP4AWSIZE[3];
        input SAXIGP4AWUSER[11];

        input SAXIGP4BREADY;
        output SAXIGP4BVALID;
        output SAXIGP4BID[6];
        output SAXIGP4BRESP[2];

        output SAXIGP4WREADY;
        input SAXIGP4WVALID;
        output SAXIGP4WCOUNT[8];
        input SAXIGP4WDATA[128];
        input SAXIGP4WLAST;
        input SAXIGP4WSTRB[16];

        // FPD_AXI_PL

        input MAXIGP0ACLK;

        input MAXIGP0ARREADY;
        output MAXIGP0ARVALID;
        output MAXIGP0ARADDR[44];
        output MAXIGP0ARBURST[2];
        output MAXIGP0ARCACHE[4];
        output MAXIGP0ARID[16];
        output MAXIGP0ARLEN[8];
        output MAXIGP0ARLOCK;
        output MAXIGP0ARPROT[3];
        output MAXIGP0ARQOS[4];
        output MAXIGP0ARSIZE[3];
        output MAXIGP0ARUSER[16];

        input MAXIGP0AWREADY;
        output MAXIGP0AWVALID;
        output MAXIGP0AWADDR[44];
        output MAXIGP0AWBURST[2];
        output MAXIGP0AWCACHE[4];
        output MAXIGP0AWID[16];
        output MAXIGP0AWLEN[8];
        output MAXIGP0AWLOCK;
        output MAXIGP0AWPROT[3];
        output MAXIGP0AWQOS[4];
        output MAXIGP0AWSIZE[3];
        output MAXIGP0AWUSER[16];

        output MAXIGP0BREADY;
        input MAXIGP0BVALID;
        input MAXIGP0BID[16];
        input MAXIGP0BRESP[2];

        output MAXIGP0RREADY;
        input MAXIGP0RVALID;
        input MAXIGP0RDATA[128];
        input MAXIGP0RID[16];
        input MAXIGP0RLAST;
        input MAXIGP0RRESP[2];

        input MAXIGP0WREADY;
        output MAXIGP0WVALID;
        output MAXIGP0WDATA[128];
        output MAXIGP0WLAST;
        output MAXIGP0WSTRB[16];

        // LPD_AXI_PL

        input MAXIGP2ACLK;

        input MAXIGP2ARREADY;
        output MAXIGP2ARVALID;
        output MAXIGP2ARADDR[44];
        output MAXIGP2ARBURST[2];
        output MAXIGP2ARCACHE[4];
        output MAXIGP2ARID[16];
        output MAXIGP2ARLEN[8];
        output MAXIGP2ARLOCK;
        output MAXIGP2ARPROT[3];
        output MAXIGP2ARQOS[4];
        output MAXIGP2ARSIZE[3];
        output MAXIGP2ARUSER[16];

        input MAXIGP2AWREADY;
        output MAXIGP2AWVALID;
        output MAXIGP2AWADDR[44];
        output MAXIGP2AWBURST[2];
        output MAXIGP2AWCACHE[4];
        output MAXIGP2AWID[16];
        output MAXIGP2AWLEN[8];
        output MAXIGP2AWLOCK;
        output MAXIGP2AWPROT[3];
        output MAXIGP2AWQOS[4];
        output MAXIGP2AWSIZE[3];
        output MAXIGP2AWUSER[16];

        output MAXIGP2BREADY;
        input MAXIGP2BVALID;
        input MAXIGP2BID[16];
        input MAXIGP2BRESP[2];

        output MAXIGP2RREADY;
        input MAXIGP2RVALID;
        input MAXIGP2RDATA[128];
        input MAXIGP2RID[16];
        input MAXIGP2RLAST;
        input MAXIGP2RRESP[2];

        input MAXIGP2WREADY;
        output MAXIGP2WVALID;
        output MAXIGP2WDATA[128];
        output MAXIGP2WLAST;
        output MAXIGP2WSTRB[16];

        // SMMU address translation

        input PLPSSMMUCLOCK;

        output PLPSSMMUARREADY;
        input PLPSSMMUARVALID;
        input PLPSSMMUARADDR[49];
        input PLPSSMMUARCACHE[4];
        input PLPSSMMUARID[8];

        output PLPSSMMUAWREADY;
        input PLPSSMMUAWVALID;
        input PLPSSMMUAWADDR[49];
        input PLPSSMMUAWCACHE[4];
        input PLPSSMMUAWID[8];

        output PLPSSMMUCOMPRDREADY;
        input PLPSSMMUCOMPRDVAL;
        input PLPSSMMUCOMPRDID[8];

        output PLPSSMMUCOMPWRREADY;
        input PLPSSMMUCOMPWRVAL;
        input PLPSSMMUCOMPWRID[8];

        input PLPSSMMURNS;
        input PLPSSMMURSMID[10];
        input PLPSSMMUWNS;
        input PLPSSMMUWSMID[10];

        input PLSMMUTESTINPUT;
        output PLSMMUTESTOUTPUT;

        input PSSMMUPLARREADY;
        output PSSMMUPLARVALID;
        output PSSMMUPLARADDR[48];
        output PSSMMUPLARCACHE[4];
        output PSSMMUPLARID[8];

        input PSSMMUPLAWREADY;
        output PSSMMUPLAWVALID;
        output PSSMMUPLAWADDR[48];
        output PSSMMUPLAWCACHE[4];
        output PSSMMUPLAWID[8];

        input PSSMMUPLBREADY;
        output PSSMMUPLBVALID;
        output PSSMMUPLBID[8];
        output PSSMMUPLBRESP[2];

        input PSSMMUPLRREADY;
        output PSSMMUPLRVALID;
        output PSSMMUPLRERR;
        output PSSMMUPLRID[8];
        output PSSMMUPLRNS;
        output PSSMMUPLRRESP[2];

        output PSSMMUPLWERR;
        output PSSMMUPLWNS;

        // ADMA

        input ADMAFCICLK[8];
        output ADMA2PLTVLD[8];
        input PL2ADMATACK[8];
        input PL2ADMACVLD[8];
        output ADMA2PLCACK[8];

        // CFG etc.

        input GCAPCLK;
        input CFUGCAPB;
        input CFUUSRGSRB;
        input CFUUSRGTSB;
        output CFGMCLK;
        output CFUEOS;

        output USRSEL[4];
        output USRDRCK[4];
        output USRRESET[4];
        output USRRUNTEST[4];
        output USRCAPTURE[4];
        output USRSHIFT[4];
        output USRUPDATE[4];
        output USRTCK[4];
        output USRTMS[4];
        output USRTDI[4];
        input USRTDO[4];

        input MJTAGTCK;
        input MJTAGTMS;
        input MJTAGTDI;
        output MJTAGTDO;

        output IFPMCCFUSEUCFUSEUCRCERROR;
        output IFPMCCFUSEUCFUSEUECCERROR;
        output IFPMCCFUSEUCFUSEUENDOFCALIB;
        output IFPMCCFUSEUCFUSEUHALTED;
        output IFPMCCFUSEUCFUSEUHEARTBEAT;

        // reverse EMIO

        output FMIOLPDPMCEMIOIN[52];
        input FMIOLPDPMCEMIOOUT[52];
        input FMIOLPDPMCEMIOOE[52];
        output FMIOFPDLPDEMIOIN[26];
        input FMIOFPDLPDEMIOOUT[26];
        input FMIOFPDLPDEMIOOE[26];

        // EMIO

        input FMIOGPIOIN[64];
        output FMIOGPIOOUT[64];
        output FMIOGPIOOEN[64];

        input EMIOGPIO2I[32];
        output EMIOGPIO2O[32];
        output EMIOGPIO2TN[32];

        input FMIOWWDTCLKIN;
        output FMIOWWDTINTERRUPT;
        output FMIOWWDTRESET;
        output FMIOWWDTRESETPENDING;
        output FMIOGWDTWS0;
        output FMIOGWDTWS1;

        input FMIOFPDWWDTCLKIN;
        output FMIOFPDWWDTINTERRUPT;
        output FMIOFPDWWDTRESET;
        output FMIOFPDWWDTRESETPENDING;
        output FMIOFPDGWDTWS0;
        output FMIOFPDGWDTWS1;

        for i in 0..4 {
            input "EMIOTTC{i}CLKI"[3];
            output "EMIOTTC{i}WAVEO"[3];
        }

        for i in 0..2 {
            input "FMIOUART{i}RXD";
            output "FMIOUART{i}TXD";
            input "FMIOUART{i}NUARTCTS";
            output "FMIOUART{i}NUARTRTS";
            input "FMIOUART{i}NUARTDSR";
            output "FMIOUART{i}NUARTDTR";
            input "FMIOUART{i}NUARTDCD";
            input "FMIOUART{i}NUARTRI";
            input "FMIOUART{i}SIRIN";
            output "FMIOUART{i}NSIROUT";
            output "FMIOUART{i}NUARTOUT1";
            output "FMIOUART{i}NUARTOUT2";
        }

        input FMIOI2CSCLIN;
        output FMIOI2CSCLOEN;
        output FMIOI2CSCLOUT;
        input FMIOI2CSDAIN;
        output FMIOI2CSDAOEN;
        output FMIOI2CSDAOUT;

        for i in 0..2 {
            input "EMIOI2C{i}SCLI";
            output "EMIOI2C{i}SCLO";
            output "EMIOI2C{i}SCLTN";
            input "EMIOI2C{i}SDAI";
            output "EMIOI2C{i}SDAO";
            output "EMIOI2C{i}SDATN";
        }

        input FMIOSYSMONI2CSCLINPUT;
        output FMIOSYSMONI2CSCLTRIB;
        input FMIOSYSMONI2CSDAINPUT;
        output FMIOSYSMONI2CSDATRIB;
        output FMIOSYSMONI2CSMBALERTTRIB;

        for i in 0..2 {
            input "EMIOSPI{i}SCLKI";
            output "EMIOSPI{i}SCLKO";
            output "EMIOSPI{i}SCLKTN";
            input "EMIOSPI{i}SSIN";
            output "EMIOSPI{i}SSON"[3];
            output "EMIOSPI{i}SSNTN";
            input "EMIOSPI{i}MI";
            output "EMIOSPI{i}MO";
            output "EMIOSPI{i}MOTN";
            input "EMIOSPI{i}SI";
            output "EMIOSPI{i}SO";
            output "EMIOSPI{i}STN";
        }
        input FMIOCHARSPITESTSELECTN;
        input FMIOCHARSPISELECTION;
        input FMIOCHARSPITESTINPUT;
        output FMIOCHARSPITESTOUTPUT;

        for i in 0..2 {
            input "EMIOCAN{i}PHYRX";
            output "EMIOCAN{i}PHYTX";
        }

        for i in 0..2 {
            output "FMIOSD{i}BUSPOWEROUT";
            output "FMIOSD{i}BUSVOLTAGEOUT"[3];
            output "FMIOSD{i}LEDCONTROLOUT";
            input "FMIOSD{i}SDIFCDNIN";
            output "FMIOSD{i}SDIFCLKOUT";
            input "FMIOSD{i}SDIFCMDIN";
            output "FMIOSD{i}SDIFCMDOE";
            output "FMIOSD{i}SDIFCMDOUT";
            input "FMIOSD{i}SDIFDATIN"[8];
            output "FMIOSD{i}SDIFDATOE"[8];
            output "FMIOSD{i}SDIFDATOUT"[8];
            input "FMIOSD{i}SDIFWPIN";
            input "FMIOSDIO{i}RXCLKIN";

            output "FMIOSD{i}DLLTESTCLK0";
            output "FMIOSD{i}DLLTESTCLKRX";
            output "FMIOSD{i}DLLTESTCLKTX";
            input "FMIOSD{i}DLLTESTIN"[4];
            output "FMIOSD{i}DLLTESTOUT"[8];
        }
        input FMIOCHARSDTESTSELECTN;
        input FMIOCHARSDSELECTION;
        input FMIOCHARSDTESTINPUT;
        output FMIOCHARSDTESTOUTPUT;

        input EMIOENETTSUCLK;
        output EMIOENET0GEMTSUTIMERCNT[94];
        input FMIOGEMTSUCLKFROMPL;
        for i in 0..2 {
            output "EMIOENET{i}DMABUSWIDTH"[2];
            output "EMIOENET{i}DMATXENDTOG";
            input "EMIOENET{i}DMATXSTATUSTOG";
            input "EMIOENET{i}EXTINTIN";
            input "EMIOENET{i}GMIICOL";
            input "EMIOENET{i}GMIICRS";
            input "EMIOENET{i}GMIIRXCLK";
            input "EMIOENET{i}GMIIRXDV";
            input "EMIOENET{i}GMIIRXD"[8];
            input "EMIOENET{i}GMIIRXER";
            input "EMIOENET{i}GMIITXCLK";
            output "EMIOENET{i}GMIITXD"[8];
            output "EMIOENET{i}GMIITXEN";
            output "EMIOENET{i}GMIITXER";
            input "EMIOENET{i}MDIOI";
            output "EMIOENET{i}MDIOMDC";
            output "EMIOENET{i}MDIOO";
            output "EMIOENET{i}MDIOTN";
            output "EMIOENET{i}RXWDATA"[8];
            output "EMIOENET{i}RXWEOP";
            output "EMIOENET{i}RXWERR";
            output "EMIOENET{i}RXWFLUSH";
            input "EMIOENET{i}RXWOVERFLOW";
            output "EMIOENET{i}RXWSOP";
            output "EMIOENET{i}RXWSTATUS"[45];
            output "EMIOENET{i}RXWWR";
            output "EMIOENET{i}SPEEDMODE"[4];
            input "EMIOENET{i}TXRCONTROL";
            input "EMIOENET{i}TXRDATARDY"[2];
            input "EMIOENET{i}TXRDATA"[8];
            input "EMIOENET{i}TXREOP";
            input "EMIOENET{i}TXRERR";
            input "EMIOENET{i}TXRFLUSHED";
            output "EMIOENET{i}TXRRD"[2];
            input "EMIOENET{i}TXRSOP";
            output "EMIOENET{i}TXRSTATUS"[4];
            input "EMIOENET{i}TXRUNDERFLOW";
            input "EMIOENET{i}TXRVALID";

            output "EMIOGEM{i}DELAYREQRX";
            output "EMIOGEM{i}DELAYREQTX";
            output "EMIOGEM{i}PDELAYREQRX";
            output "EMIOGEM{i}PDELAYREQTX";
            output "EMIOGEM{i}PDELAYRESPRX";
            output "EMIOGEM{i}PDELAYRESPTX";
            output "EMIOGEM{i}RXSOF";
            output "EMIOGEM{i}SYNCFRAMERX";
            output "EMIOGEM{i}SYNCFRAMETX";
            input "EMIOGEM{i}TSUINCCTRL"[2];
            output "EMIOGEM{i}TSUTIMERCMPVAL";
            output "EMIOGEM{i}TXRFIXEDLAT";
            output "EMIOGEM{i}TXSOF";

            output "FMIOGEM{i}ADDMATCHVEC"[4];
            input "FMIOGEM{i}FIFORXCLKFROMPL";
            input "FMIOGEM{i}FIFOTXCLKFROMPL";
            output "FMIOGEM{i}RXDATABUFWRQ0";
            output "FMIOGEM{i}RXDATABUFWRQ1";
            output "FMIOGEM{i}RXWQUEUE"[4];
            output "FMIOGEM{i}TXRQUEUE"[4];
            output "FMIOGEM{i}TXRTIMESTAMP"[78];
        }
        input FMIOCHARGEMTESTSELECTN;
        input FMIOCHARGEMSELECTION;
        input FMIOCHARGEMTESTINPUT;
        output FMIOCHARGEMTESTOUTPUT;

        input EMIOHUBPORTOVERCRNTUSB20;
        output EMIOU2DSPORTVBUSCTRLUSB30;

        input FMIOCHARAFIFSLPDTESTSELECTN;
        input FMIOCHARAFIFSLPDTESTINPUT;
        output FMIOCHARAFIFSLPDTESTOUTPUT;
        input FMIOCHARAFIFSFPDTESTSELECTN;
        input FMIOCHARAFIFSFPDTESTINPUT;
        output FMIOCHARAFIFSFPDTESTOUTPUT;

        // SYSMONs

        output IFSYSMONROOTUSERALARM[8];
        output IFSYSMONROOTUSERDATAREADY;
        output IFSYSMONROOTUSERMUXADDR[4];
        output IFSYSMONROOTUSERPVALUE[16];
        output IFSYSMONROOTUSERTVALUE[16];
        output IFSYSMONROOTUSERVVALUE[16];
        output PMCPLSYSMONROOTALARM[10];
        output PMCPLSYSMONROOTNEWDATA[4];

        input IFSYSMONROOTTESTFABRICCLK;
        input IFSYSMONROOTTESTTESTADCINCLK;
        input IFSYSMONROOTTESTTESTADCIN[15];
        output IFSYSMONROOTTESTTESTADCOUT[16];

        input IFLPDSYSMONSATTESTFABRICCLK;
        input IFLPDSYSMONSATTESTTESTADCINCLK;
        input IFLPDSYSMONSATTESTTESTADCIN[15];
        output IFLPDSYSMONSATTESTTESTADCOUT[16];

        input IFFPDSYSMONSATTESTFABRICCLK;
        input IFFPDSYSMONSATTESTTESTADCINCLK;
        input IFFPDSYSMONSATTESTTESTADCIN[15];
        output IFFPDSYSMONSATTESTTESTADCOUT[16];

        // Tracing etc

        input FTMGPI[32];

        input STMEVENT[60];

        output PSPLTRIGACK[4];
        output PSPLTRIGGER[4];
        input PLPSTRIGACK[4];
        input PLPSTRIGGER[4];

        // ATB

        input PLPSATBCLK;

        output PSPLATREADY;
        input PLPSATVALID;
        input PLPSATBYTES[2];
        input PLPSATDATA[32];
        input PLPSATID[7];

        input PLPSAFREADY;
        output PSPLAFVALID;

        output PSPLSYNCREQ;

        // Trace out

        input PLPSTRACECLK;
        output PSPLTRACECTL;
        output PSPLTRACEDATA[32];

        // HSDP

        input PLHSDPCLK;

        input PLHSDPEGRESSTREADY;
        output PLHSDPEGRESSTVALID;
        output PLHSDPEGRESSTDATA[64];
        output PLHSDPEGRESSTKEEP[8];
        output PLHSDPEGRESSTLAST;
        output PLHSDPEGRESSTUSER[12];

        output PLHSDPINGRESSTREADY;
        input PLHSDPINGRESSTVALID;
        input PLHSDPINGRESSTDATA[64];
        input PLHSDPINGRESSTKEEP[8];
        input PLHSDPINGRESSTLAST;

        // PMU

        input AIBPMUAFIFMFPDACK;
        input AIBPMUAFIFMLPDACK;
        output PMUAIBAFIFMFPDREQ;
        output PMUAIBAFIFMLPDREQ;

        // PPU

        output PPU1PLTRIGACKIN;
        input PPU1PLTRIGACKOUT;
        input PPU1PLTRIGIN;
        output PPU1PLTRIGOUT;

        // Errors

        output PMCERRORTOPL[64];
        input PMUERRORFROMPL[4];
        output PMUERRORTOPL[47];
        output PSMERRORTOPL[64];

        // Special clock outs to XPLLs and north HNOC only

        output HSMREFCLK[2];

        // Special test-only clock outputs

        output MCLK;
        output NOCCLK;
        output NPICLK;
        output NTISCANCLKIN;

        // PLL test

        input PLPMCPLLTESTOUTSEL;
        output PMCPLPLLTESTOUT[16];
        output PMCNOCPPLLTESTCLKOUT[2];
        output PMCNOCNPLLTESTCLKOUT[2];

        input PLLPDPLLTESTOUTSEL;
        output LPDPLLTESTOUT[16];
        output FPDNOCLPDPLLTESTCLKOUT[2];

        output FPDPLLTESTOUT[16];
        output FPDNOCFPDPLLTESTCLKOUT[2];

        output FPDNOCCPMPLLTESTCLKOUT[2];

        // Unsorted

        input PLFPDSPARE0IN;
        input PLFPDSPARE1IN;
        output FPDPLSPARE0OUT;
        output FPDPLSPARE1OUT;

        input GDFABRICGL[4];

        output LPDPLSPARE0OUT;
        output LPDPLSPARE1OUT;
        output LPDPLSPARE2OUT;
        output LPDPLSPARE3OUT;
        output LPDPLSPARE4OUT;
        input PLLPDSPARE0IN;
        input PLLPDSPARE1IN;
        input PLLPDSPARE2IN;
        input PLLPDSPARE3IN;
        input PLLPDSPARE4IN;
        output LPDPLDEBUG[67];

        output OSCRTCCLK;

        input PLATTESTSELN;

        input PLCONFIGDONE;

        output PLREFCLKMUXMONITOR;

        output PLTESTPATTERNDOUT[32];

        input PLPMCCSSDTRIG;
        input PLPMCCSSDTRIGGERN;
        input PLPMCEDTUPDATEN;
        input PLPMCSCANENN;
        input PLPMCSCANRESETN;
        input PLPMCSCANSYNCBYP;

        output PMCPLCSSDTRIGGERNR;
        output PMCPLEDTUPDATENR;
        output PMCPLSCANENNR;
        output PMCPLSCANRESETNR;

        input PLPMCNTISCANCLOCK;
        input PLPMCNTITESTEN[3];
        input PLPMCNTIBUSIN[16];
        output PMCPLNTIBUSOUT[16];

        input PMCTPPLCLK[4];
        output PMCTPPLIN[38];
        input PMCTPPLOEN[38];
        input PMCTPPLOUT[38];

        input PLPMCCTRLTESTAAG;
        input PLPMCFPGACLOCKSTOP[4];

        input PMVCAPTURECLKINTIP;
        output PMVCAPTUREQINTOP[5];
        input PMVDATAININTIP;
        input PMVEN1INTIP;
        input PMVFFCLKINTIP;
        output PMVFFQINTOP[5];
        input PMVIMUXININT[5];
        output PMVOUT1INTOP;
        output PMVOUT2INTOP;
        output PMVOUT3INTOP;
        output PMVOUT4INTOP;
        output PMVOUTSCLB[3];
        output PMVOUTSINT[3];
        output PMVPULSEOUTINTOP;
        input PMVRESETINTIP;
        input PMVSPAREIN1INTIP[6];
        input PMVSYNCENBINTIP;

        input SCANCHNLINAPUEXT[3];
        input SCANCHNLINCPU01EXT[4];
        input SCANCHNLINEXTFPDSMON;
        input SCANCHNLINEXTFPD[10];
        output SCANCHNLOUTAPUEXT[3];
        output SCANCHNLOUTCPU0EXT[4];
        output SCANCHNLOUTCPU1EXT[4];
        output SCANCHNLOUTEXTFPDSMON;
        output SCANCHNLOUTEXTFPD[10];
        output SCANCHNLSPAREOUTEXTCCI[8];
        input SCANCLKDIVRSTNEXTFPD;
        input SCANCLKNEXT;
        input SCANCLKNEXTFPD;
        input SCANCNTRLCHNLINEXTAPU;
        input SCANCNTRLCHNLINEXTFPD;
        output SCANCNTRLCHNLOUTCPU1;
        output SCANCNTRLCHNLOUTEXTFPD;
        input SCANEDTUPDTNEXT;
        input SCANEDTUPDTNEXTFPD;
        input SCANENNEXT;
        input SCANENNEXTFPD;
        input SCANMODERSTNEXT;
        input SCANMODERSTNEXTFPD;

        input SCANIOUCHNLINEXT[2];
        output SCANIOUCHNLOUTEXT[2];
        input SCANIOUCNTRLCHNLINEXT;
        output SCANIOUCNTRLCHNLOUTEXT;
        input SCANIOUSPAREINEXT[3];
        output SCANIOUSPAREOUTEXT[3];

        input SCANLPDINTCHNLINEXT[15];
        output SCANLPDINTCHNLOUTEXT[15];
        input SCANLPDINTCLKDIVRSTNEXT;
        input SCANLPDINTCNTRLCHNLINEXT;
        output SCANLPDINTCNTRLCHNLOUTEXT;
        input SCANLPDINTPLLRESETEXT;
        input SCANLPDINTSPAREINEXT[3];
        output SCANLPDINTSPAREOUTEXT[3];
        input SCANLPDSMONCHNLINEXT;
        output SCANLPDSMONCHNLOUTEXT;

        input SCANPLLRESETEXTFPD;

        input SCANRPUCHNLINEXT[2];
        output SCANRPUCHNLOUTEXT[2];
        input SCANRPUCNTRLCHNLINEXT;
        output SCANRPUCNTRLCHNLOUTEXT;
        input SCANRPUSPAREINEXT[3];
        output SCANRPUSPAREOUTEXT[3];
        input SCANSPAREINAPU[3];
        input SCANSPAREINCPU0[3];
        input SCANSPAREINCPU1[3];
        input SCANSPAREINEXTFPD[3];
        output SCANSPAREOUTAPU[3];
        output SCANSPAREOUTCPU0[3];
        output SCANSPAREOUTCPU1[3];
        output SCANSPAREOUTEXTFPD[3];

        input SCANXRAMCHNLINEXT[5];
        output SCANXRAMCHNLOUTEXT[5];
        input SCANXRAMCLKNEXT;
        input SCANXRAMCNTRLCHNLINEXT;
        output SCANXRAMCNTRLCHNLOUTEXT;
        input SCANXRAMENNEXT;
        input SCANXRAMMODEEDTUPDTN;
        input SCANXRAMMODERSTNEXT;
        input SCANXRAMSPAREINEXT[2];
        output SCANXRAMSPAREOUTEXT[2];

        input SMMUPLTESTINPUT;
        output SMMUPLTESTOUTPUT;

        input TESTBSCANACMODE;
        input TESTBSCANACTEST;
        input TESTBSCANCLOCKDR;
        input TESTBSCANENN;
        input TESTBSCANEXTEST;
        input TESTBSCANEXTESTSMPL;
        input TESTBSCANINITMEMORY;
        input TESTBSCANMODEC;
        input TESTBSCANRESETTAPB;
        input TESTBSCANSHIFTDR;
        input TESTBSCANTDI;
        output TESTBSCANTDO;
        input TESTBSCANUPDATEDR;

        input TESTUSB0FUNCMUX0N;
        input TESTUSB0SCANMUX0N;
        input TESTUSB1FUNCMUX0N;
        input TESTUSB1SCANMUX0N;

        input TKLBCLKDIVCONTROL;
        input TKLBLBISTCAPTUREDR;
        input TKLBLBISTCLK;
        input TKLBLBISTSHIFTDR;
        input TKLBLBISTTCK;
        input TKLBLBISTTESTLOGICRESET;
        input TKLBLBISTUPDATEDR;
        input TKLBLPDINTCHANNELIN[10];
        output TKLBLPDINTCHANNELOUT[10];
        input TKLBLPDINTCONTROLLERCHAINEN;
        input TKLBLPDINTCONTROLLERCHAINSCANIN;
        output TKLBLPDINTCONTROLLERCHAINSCANOUT;
        input TKLBLPDINTCTRLCHAINSI;
        output TKLBLPDINTCTRLCHAINSO;
        input TKLBLPDINTEXTESTCHAINSI;
        output TKLBLPDINTEXTESTCHAINSO;
        input TKLBLPDINTLBISTSHIFTSCANIN;
        output TKLBLPDINTLBISTSHIFTSCANOUT;
        input TKLBLPDINTLBISTTAPINSTDECODE;
        input TKLBLPDINTMFGPOSTCLOCK;
        output TKLBLPDINTMFGPOSTLBISTDONE;
        input TKLBLPDINTMFGPOSTRESETN;
        input TKLBLPDINTPOSTISOLATIONEN;
        input TKLBLPDINTSPAREINS[2];
        output TKLBLPDINTSPAREOUTS[2];
        input TKLBPLLRESET;
        input TKLBRPUCHANNELIN[10];
        output TKLBRPUCHANNELOUT[10];
        input TKLBRPUCONTROLLERCHAINEN;
        input TKLBRPUCONTROLLERCHAINSCANIN;
        output TKLBRPUCONTROLLERCHAINSCANOUT;
        input TKLBRPUCTRLCHAINSI;
        output TKLBRPUCTRLCHAINSO;
        input TKLBRPUFUNCPOSTCLOCK;
        input TKLBRPUFUNCPOSTEFUSEENABLE;
        input TKLBRPUFUNCPOSTRESETN;
        input TKLBRPULBISTSHIFTSCANIN;
        output TKLBRPULBISTSHIFTSCANOUT;
        input TKLBRPULBISTTAPINSTDECODE;
        input TKLBRPUMFGPOSTCLOCK;
        output TKLBRPUMFGPOSTLBISTDONE;
        input TKLBRPUMFGPOSTRESETN;
        input TKLBRPUSPAREINS[2];
        output TKLBRPUSPAREOUTS[2];
        input TKLBSCANEDTCLK;
        input TKLBSCANEDTRESET;
        input TKLBSCANEDTUPDATE;
        input TKLBSCANRESET;
        input TKLBSHIFTCLOCK;
        input TKLBSHIFTENABLE;

        input TSTRTCCALIBREGIN[21];
        output TSTRTCCALIBREGOUT[21];
        input TSTRTCCALIBREGWE;
        input TSTRTCCLK;
        input TSTRTCDISABLEBATOP;
        output TSTRTCOSCCLKOUT;
        input TSTRTCOSCCNTRLIN[4];
        output TSTRTCOSCCNTRLOUT[4];
        input TSTRTCOSCCNTRLWE;
        output TSTRTCSECCOUNTEROUT[32];
        output TSTRTCSECONDSRAWINT;
        input TSTRTCSECRELOAD;
        input TSTRTCTESTCLOCKSELECTN;
        input TSTRTCTESTMODEN;
        output TSTRTCTICKCOUNTEROUT[16];
        input TSTRTCTIMESETREGIN[32];
        output TSTRTCTIMESETREGOUT[32];
        input TSTRTCTIMESETREGWE;
    }

    bel_class XRAM {
        input PLXRAMMAINCLK;
        input PLXRAMSYSRSTN;

        output XRAMPLMAINOUTCLK;
        output XRAMPLLSBUSOUTCLK;

        // main AXI

        for i in 0..3 {
            input "PLXRAMAXI{i}CLK";

            input "IFPLXRAMAXI{i}ARVALID";
            output "IFPLXRAMAXI{i}ARREADY";
            input "IFPLXRAMAXI{i}ARADDR"[32];
            input "IFPLXRAMAXI{i}ARBURST"[2];
            input "IFPLXRAMAXI{i}ARCACHE"[4];
            input "IFPLXRAMAXI{i}ARID"[16];
            input "IFPLXRAMAXI{i}ARLEN"[8];
            input "IFPLXRAMAXI{i}ARLOCK"[1];
            input "IFPLXRAMAXI{i}ARPROT"[3];
            input "IFPLXRAMAXI{i}ARQOS"[4];
            input "IFPLXRAMAXI{i}ARREGION"[4];
            input "IFPLXRAMAXI{i}ARSIZE"[3];
            input "IFPLXRAMAXI{i}ARUSER"[16];

            input "IFPLXRAMAXI{i}AWVALID";
            output "IFPLXRAMAXI{i}AWREADY";
            input "IFPLXRAMAXI{i}AWADDR"[32];
            input "IFPLXRAMAXI{i}AWBURST"[2];
            input "IFPLXRAMAXI{i}AWCACHE"[4];
            input "IFPLXRAMAXI{i}AWID"[16];
            input "IFPLXRAMAXI{i}AWLEN"[8];
            input "IFPLXRAMAXI{i}AWLOCK"[1];
            input "IFPLXRAMAXI{i}AWPROT"[3];
            input "IFPLXRAMAXI{i}AWQOS"[4];
            input "IFPLXRAMAXI{i}AWREGION"[4];
            input "IFPLXRAMAXI{i}AWSIZE"[3];
            input "IFPLXRAMAXI{i}AWUSER"[16];

            input "IFPLXRAMAXI{i}BREADY";
            output "IFPLXRAMAXI{i}BVALID";
            output "IFPLXRAMAXI{i}BID"[16];
            output "IFPLXRAMAXI{i}BRESP"[2];
            output "IFPLXRAMAXI{i}BUSER"[1];

            input "IFPLXRAMAXI{i}RREADY";
            output "IFPLXRAMAXI{i}RVALID";
            output "IFPLXRAMAXI{i}RDATA"[256];
            output "IFPLXRAMAXI{i}RID"[16];
            output "IFPLXRAMAXI{i}RLAST"[1];
            output "IFPLXRAMAXI{i}RRESP"[2];
            output "IFPLXRAMAXI{i}RUSER"[32];

            output "IFPLXRAMAXI{i}WREADY";
            input "IFPLXRAMAXI{i}WVALID";
            input "IFPLXRAMAXI{i}WDATA"[256];
            input "IFPLXRAMAXI{i}WID"[16];
            input "IFPLXRAMAXI{i}WLAST"[1];
            input "IFPLXRAMAXI{i}WSTRB"[32];
            input "IFPLXRAMAXI{i}WUSER"[32];
        }

        // AXI-lite

        input PLXRAMAXILITECLK;

        output IFPLXRAMAXIARREADY;
        input IFPLXRAMAXIARVALID;
        input IFPLXRAMAXIARADDR[32];
        input IFPLXRAMAXIARPROT[3];

        output IFPLXRAMAXIAWREADY;
        input IFPLXRAMAXIAWVALID;
        input IFPLXRAMAXIAWADDR[32];
        input IFPLXRAMAXIAWPROT[3];

        input IFPLXRAMAXIBREADY;
        output IFPLXRAMAXIBVALID;
        output IFPLXRAMAXIBRESP[2];

        input IFPLXRAMAXIRREADY;
        output IFPLXRAMAXIRVALID;
        output IFPLXRAMAXIRDATA[32];
        output IFPLXRAMAXIRRESP[2];

        output IFPLXRAMAXIWREADY;
        input IFPLXRAMAXIWVALID;
        input IFPLXRAMAXIWDATA[32];
        input IFPLXRAMAXIWSTRB[4];

        // misc

        output XRAMPLCORRERR;
        output XRAMPLUNCORRERR;
        output XRAMPLERRSTATUS[128];

        input PLXRAMCSSDTRIGN[2];
        input PLXRAMSPARE[5];
        output XRAMPLSPARE[5];
    }

    bel_class CPM4 {
        // special clock outs

        output CPMDPLLPCIE0USERCLK;
        output CPMDPLLPCIE1USERCLK;

        output IFCPMPCIEADPLLCPMDPLL0RSTN;
        output IFCPMPCIEADPLLCPMDPLL1RSTN;

        input IFCPMPCIEADPLLDPLL0CPMLOCKED;
        input IFCPMPCIEADPLLDPLL1CPMLOCKED;

        input PLREFCLK;

        input PLCPMIRQ0;
        input PLCPMIRQ1;

        input PLCPMLOOPBACKEN;

        output IFCPMPLISRCORREVENT;
        output IFCPMPLISRUNCORREVENT;
        output IFCPMPLISRMISCEVENT;

        for i in 0..2 {
            input "IFCPMPCIEA{i}CFGHPICFGCCIXEDRDATARATECHANGEACK";
            input "IFCPMPCIEA{i}CFGHPICFGERRCORIN";
            input "IFCPMPCIEA{i}CFGHPICFGERRUNCORIN";
            input "IFCPMPCIEA{i}CFGHPICFGEXTREADDATAVALID";
            input "IFCPMPCIEA{i}CFGHPICFGEXTREADDATA"[32];
            input "IFCPMPCIEA{i}CFGHPICFGFCSEL"[3];
            input "IFCPMPCIEA{i}CFGHPICFGFCVCSEL";
            input "IFCPMPCIEA{i}CFGHPICFGFLRDONE"[4];
            input "IFCPMPCIEA{i}CFGHPICFGHOTRESETIN";
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTINT"[4];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIATTR"[3];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIFUNCTIONNUMBER"[8];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIINT"[32];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIPENDINGSTATUSDATAENABLE";
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIPENDINGSTATUSFUNCTIONNUM"[2];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIPENDINGSTATUS"[32];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSISELECT"[2];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSITPHPRESENT";
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSITPHSTTAG"[8];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSITPHTYPE"[2];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIXADDRESS"[64];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIXDATA"[32];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIXINT";
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTMSIXVECPENDING"[2];
            input "IFCPMPCIEA{i}CFGHPICFGINTERRUPTPENDING"[4];
            input "IFCPMPCIEA{i}CFGHPICFGMGMTADDR"[10];
            input "IFCPMPCIEA{i}CFGHPICFGMGMTBYTEENABLE"[4];
            input "IFCPMPCIEA{i}CFGHPICFGMGMTDEBUGACCESS";
            input "IFCPMPCIEA{i}CFGHPICFGMGMTFUNCTIONNUMBER"[8];
            input "IFCPMPCIEA{i}CFGHPICFGMGMTREAD";
            input "IFCPMPCIEA{i}CFGHPICFGMGMTWRITE";
            input "IFCPMPCIEA{i}CFGHPICFGMGMTWRITEDATA"[32];
            input "IFCPMPCIEA{i}CFGHPICFGMSGTRANSMIT";
            input "IFCPMPCIEA{i}CFGHPICFGMSGTRANSMITDATA"[32];
            input "IFCPMPCIEA{i}CFGHPICFGMSGTRANSMITTYPE"[3];
            input "IFCPMPCIEA{i}CFGHPICFGPOWERSTATECHANGEACK";
            input "IFCPMPCIEA{i}CFGHPICFGVFFLRDONE";
            input "IFCPMPCIEA{i}CFGHPICFGVFFLRFUNCNUM"[8];
        }

        output IFPCIE0PLCCAXISAXISCCTREADY[4];
        input IFPCIE0PLCCAXISAXISCCTVALID;
        input IFPCIE0PLCCAXISAXISCCTDATA[512];
        input IFPCIE0PLCCAXISAXISCCTKEEP[16];
        input IFPCIE0PLCCAXISAXISCCTLAST;
        input IFPCIE0PLCCAXISAXISCCTUSER[81];

        output IFPCIE0PLRQAXISAXISRQTREADY[4];
        input IFPCIE0PLRQAXISAXISRQTVALID;
        input IFPCIE0PLRQAXISAXISRQTDATA[512];
        input IFPCIE0PLRQAXISAXISRQTKEEP[16];
        input IFPCIE0PLRQAXISAXISRQTLAST;
        input IFPCIE0PLRQAXISAXISRQTUSER[137];

        output IFPCIE1PLCCAXISAXISCCTREADY[4];
        input IFPCIE1PLCCAXISAXISCCTVALID;
        input IFPCIE1PLCCAXISAXISCCTDATA[512];
        input IFPCIE1PLCCAXISAXISCCTKEEP[16];
        input IFPCIE1PLCCAXISAXISCCTLAST;
        input IFPCIE1PLCCAXISAXISCCTUSER[81];

        output IFPCIE1PLRQAXISAXISRQTREADY[4];
        input IFPCIE1PLRQAXISAXISRQTVALID;
        input IFPCIE1PLRQAXISAXISRQTDATA[512];
        input IFPCIE1PLRQAXISAXISRQTKEEP[16];
        input IFPCIE1PLRQAXISAXISRQTLAST;
        input IFPCIE1PLRQAXISAXISRQTUSER[137];

        for i in 0..2 {
            input "PLCHI{i}CLK";

            input "IFPLCPMP{i}CHIREQFLITPEND";
            input "IFPLCPMP{i}CHIREQFLITV";
            input "IFPLCPMP{i}CHIREQFLIT"[121];
            output "IFPLCPMP{i}CHIREQLCRDV";

            input "IFPLCPMP{i}CHIWDATFLITPEND";
            input "IFPLCPMP{i}CHIWDATFLITV";
            input "IFPLCPMP{i}CHIWDATFLIT"[705];
            output "IFPLCPMP{i}CHIWDATLCRDV";

            input "IFPLCPMP{i}CHISRSPFLITPEND";
            input "IFPLCPMP{i}CHISRSPFLITV";
            input "IFPLCPMP{i}CHISRSPFLIT"[51];
            output "IFPLCPMP{i}CHISRSPLCRDV";

            output "IFPLCPMP{i}CHICRSPFLITPEND";
            output "IFPLCPMP{i}CHICRSPFLITV";
            output "IFPLCPMP{i}CHICRSPFLIT"[51];
            input "IFPLCPMP{i}CHICRSPLCRDV";

            output "IFPLCPMP{i}CHIRDATFLITPEND";
            output "IFPLCPMP{i}CHIRDATFLITV";
            output "IFPLCPMP{i}CHIRDATFLIT"[705];
            input "IFPLCPMP{i}CHIRDATLCRDV";

            output "IFPLCPMP{i}CHISNPFLITPEND";
            output "IFPLCPMP{i}CHISNPFLITV";
            output "IFPLCPMP{i}CHISNPFLIT"[88];
            input "IFPLCPMP{i}CHISNPLCRDV";

            input "IFPLCPMP{i}CHIMLINKACTIVEREQ";
            output "IFPLCPMP{i}CHIMLINKACTIVEACK";
            output "IFPLCPMP{i}CHISLINKACTIVEREQ";
            input "IFPLCPMP{i}CHISLINKACTIVEACK";
            input "IFPLCPMP{i}CHIMSACTIVE";
            output "IFPLCPMP{i}CHISSACTIVE";
            input "IFPLCPMP{i}CHISYSCOREQ";
            output "IFPLCPMP{i}CHISYSCOACK";
        }

        output IFEXTPLPCIE0CFGHPOCFGCCIXEDRDATARATECHANGEREQ;
        output IFEXTPLPCIE0CFGHPOCFGCURRENTSPEED[2];
        output IFEXTPLPCIE0CFGHPOCFGEDRENABLE;
        output IFEXTPLPCIE0CFGHPOCFGERRCOROUT;
        output IFEXTPLPCIE0CFGHPOCFGERRFATALOUT;
        output IFEXTPLPCIE0CFGHPOCFGERRNONFATALOUT;
        output IFEXTPLPCIE0CFGHPOCFGEXTFUNCTIONNUMBER[8];
        output IFEXTPLPCIE0CFGHPOCFGEXTREADRECEIVED;
        output IFEXTPLPCIE0CFGHPOCFGEXTREGISTERNUMBER[10];
        output IFEXTPLPCIE0CFGHPOCFGEXTWRITEBYTEENABLE[4];
        output IFEXTPLPCIE0CFGHPOCFGEXTWRITEDATA[32];
        output IFEXTPLPCIE0CFGHPOCFGEXTWRITERECEIVED;
        output IFEXTPLPCIE0CFGHPOCFGFCNPDSCALE[2];
        output IFEXTPLPCIE0CFGHPOCFGFCNPD[12];
        output IFEXTPLPCIE0CFGHPOCFGFCNPHSCALE[2];
        output IFEXTPLPCIE0CFGHPOCFGFCNPH[8];
        output IFEXTPLPCIE0CFGHPOCFGFCPDSCALE[2];
        output IFEXTPLPCIE0CFGHPOCFGFCPD[12];
        output IFEXTPLPCIE0CFGHPOCFGFCPHSCALE[2];
        output IFEXTPLPCIE0CFGHPOCFGFCPH[8];
        output IFEXTPLPCIE0CFGHPOCFGHOTRESETOUT;
        output IFEXTPLPCIE0CFGHPOCFGINTERRUPTSENT;
        output IFEXTPLPCIE0CFGHPOCFGLINKPOWERSTATE[2];
        output IFEXTPLPCIE0CFGHPOCFGLOCALERROROUT[5];
        output IFEXTPLPCIE0CFGHPOCFGLOCALERRORVALID;
        output IFEXTPLPCIE0CFGHPOCFGLTSSMSTATE[6];
        output IFEXTPLPCIE0CFGHPOCFGMGMTREADDATA[32];
        output IFEXTPLPCIE0CFGHPOCFGMGMTREADWRITEDONE;
        output IFEXTPLPCIE0CFGHPOCFGNEGOTIATEDWIDTH[3];
        output IFEXTPLPCIE0CFGHPOCFGPASIDENABLE[4];
        output IFEXTPLPCIE0CFGHPOCFGPASIDEXECPERMISSIONENABLE[4];
        output IFEXTPLPCIE0CFGHPOCFGPASIDPRIVILMODEENABLE[4];
        output IFEXTPLPCIE0CFGHPOCFGPHYLINKDOWN;
        output IFEXTPLPCIE0CFGHPOCFGPHYLINKSTATUS[2];
        output IFEXTPLPCIE0CFGHPOCFGPLSTATUSCHANGE;
        output IFEXTPLPCIE0CFGHPOCFGPOWERSTATECHANGEINTERRUPT;
        output IFEXTPLPCIE0CFGHPOCFGRXPMSTATE[2];
        output IFEXTPLPCIE0CFGHPOCFGTXPMSTATE[2];

        output IFEXTPLPCIE0CFGLPOCFG10BTAGREQUESTERENABLE[4];
        output IFEXTPLPCIE0CFGLPOCFGATOMICREQUESTERENABLE[4];
        output IFEXTPLPCIE0CFGLPOCFGEXTTAGENABLE;
        output IFEXTPLPCIE0CFGLPOCFGFCCPLDSCALE[2];
        output IFEXTPLPCIE0CFGLPOCFGFCCPLD[12];
        output IFEXTPLPCIE0CFGLPOCFGFCCPLHSCALE[2];
        output IFEXTPLPCIE0CFGLPOCFGFCCPLH[8];
        output IFEXTPLPCIE0CFGLPOCFGFLRINPROCESS[4];
        output IFEXTPLPCIE0CFGLPOCFGFUNCTIONPOWERSTATE[12];
        output IFEXTPLPCIE0CFGLPOCFGFUNCTIONSTATUS[16];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIDATA[32];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIENABLE[4];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIFAIL;
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIMASKUPDATE;
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIMMENABLE[12];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSISENT;
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIXENABLE[4];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIXMASK[4];
        output IFEXTPLPCIE0CFGLPOCFGINTERRUPTMSIXVECPENDINGSTATUS;
        output IFEXTPLPCIE0CFGLPOCFGMAXPAYLOAD[2];
        output IFEXTPLPCIE0CFGLPOCFGMAXREADREQ[3];
        output IFEXTPLPCIE0CFGLPOCFGMSGRECEIVED;
        output IFEXTPLPCIE0CFGLPOCFGMSGRECEIVEDDATA[8];
        output IFEXTPLPCIE0CFGLPOCFGMSGRECEIVEDTYPE[5];
        output IFEXTPLPCIE0CFGLPOCFGMSGTRANSMITDONE;
        output IFEXTPLPCIE0CFGLPOCFGRCBSTATUS[4];
        output IFEXTPLPCIE0CFGLPOCFGTPHREQUESTERENABLE[4];
        output IFEXTPLPCIE0CFGLPOCFGTPHSTMODE[12];
        output IFEXTPLPCIE0CFGLPOCFGVC1ENABLE;
        output IFEXTPLPCIE0CFGLPOCFGVC1NEGOTIATIONPENDING;

        output IFEXTPLPCIE0UNIFIEDCQAXISAXISCQTDATA[512];
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISCQTKEEP[16];
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISCQTUSER[229];
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISPORT0CQTLAST;
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISPORT0RCTLAST;
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISPORT1CQTLAST;
        output IFEXTPLPCIE0UNIFIEDCQAXISAXISPORT1RCTLAST;
        input IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT0CQCRDT;
        output IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT0CQVLD;
        input IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT0RCCRDT;
        output IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT0RCVLD;
        input IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT1CQCRDT;
        output IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT1CQVLD;
        input IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT1RCCRDT;
        output IFEXTPLPCIE0UNIFIEDCQAXISPCIEAXISPORT1RCVLD;

        input IFPLEXTPCIE0AXIPCIECOMPLDELIVEREDTAG0[8];
        input IFPLEXTPCIE0AXIPCIECOMPLDELIVEREDTAG1[8];
        input IFPLEXTPCIE0AXIPCIECOMPLDELIVERED[2];
        output IFPLEXTPCIE0AXIPCIECQNPREQCOUNT[6];
        input IFPLEXTPCIE0AXIPCIECQNPREQ[2];
        input IFPLEXTPCIE0AXIPCIECQNPUSERCREDITRCVD;
        input IFPLEXTPCIE0AXIPCIECQPIPELINEEMPTY;
        input IFPLEXTPCIE0AXIPCIEPOSTEDREQDELIVERED;
        output IFPLEXTPCIE0AXIPCIERQSEQNUM0[6];
        output IFPLEXTPCIE0AXIPCIERQSEQNUM1[6];
        output IFPLEXTPCIE0AXIPCIERQSEQNUMVLD0;
        output IFPLEXTPCIE0AXIPCIERQSEQNUMVLD1;
        output IFPLEXTPCIE0AXIPCIERQTAG0[10];
        output IFPLEXTPCIE0AXIPCIERQTAG1[10];
        output IFPLEXTPCIE0AXIPCIERQTAGAV[4];
        output IFPLEXTPCIE0AXIPCIERQTAGVLD0;
        output IFPLEXTPCIE0AXIPCIERQTAGVLD1;
        output IFPLEXTPCIE0AXIPCIETFCNPDAV[4];
        output IFPLEXTPCIE0AXIPCIETFCNPHAV[4];
        input IFPLEXTPCIE0AXITLRXCOMPLCREDITRELEASED;
        input IFPLEXTPCIE0AXITLRXCOMPLHEADERCREDITRELEASEDVALUE[2];
        input IFPLEXTPCIE0AXITLRXCOMPLPAYLOADCREDITRELEASEDVALUE[7];
        input IFPLEXTPCIE0AXITLRXNONPOSTEDCREDITRELEASED;
        input IFPLEXTPCIE0AXITLRXNONPOSTEDPAYLOADCREDITRELEASEDVALUE[2];
        input IFPLEXTPCIE0AXITLRXPOSTEDCREDITRELEASED;
        input IFPLEXTPCIE0AXITLRXPOSTEDPAYLOADCREDITRELEASEDVALUE[7];

        output IFEXTPLPCIE1CFGHPOCFGCCIXEDRDATARATECHANGEREQ;
        output IFEXTPLPCIE1CFGHPOCFGCURRENTSPEED[2];
        output IFEXTPLPCIE1CFGHPOCFGEDRENABLE;
        output IFEXTPLPCIE1CFGHPOCFGERRCOROUT;
        output IFEXTPLPCIE1CFGHPOCFGERRFATALOUT;
        output IFEXTPLPCIE1CFGHPOCFGERRNONFATALOUT;
        output IFEXTPLPCIE1CFGHPOCFGEXTFUNCTIONNUMBER[8];
        output IFEXTPLPCIE1CFGHPOCFGEXTREADRECEIVED;
        output IFEXTPLPCIE1CFGHPOCFGEXTREGISTERNUMBER[10];
        output IFEXTPLPCIE1CFGHPOCFGEXTWRITEBYTEENABLE[4];
        output IFEXTPLPCIE1CFGHPOCFGEXTWRITEDATA[32];
        output IFEXTPLPCIE1CFGHPOCFGEXTWRITERECEIVED;
        output IFEXTPLPCIE1CFGHPOCFGFCNPDSCALE[2];
        output IFEXTPLPCIE1CFGHPOCFGFCNPD[12];
        output IFEXTPLPCIE1CFGHPOCFGFCNPHSCALE[2];
        output IFEXTPLPCIE1CFGHPOCFGFCNPH[8];
        output IFEXTPLPCIE1CFGHPOCFGFCPDSCALE[2];
        output IFEXTPLPCIE1CFGHPOCFGFCPD[12];
        output IFEXTPLPCIE1CFGHPOCFGFCPHSCALE[2];
        output IFEXTPLPCIE1CFGHPOCFGFCPH[8];
        output IFEXTPLPCIE1CFGHPOCFGHOTRESETOUT;
        output IFEXTPLPCIE1CFGHPOCFGINTERRUPTSENT;
        output IFEXTPLPCIE1CFGHPOCFGLINKPOWERSTATE[2];
        output IFEXTPLPCIE1CFGHPOCFGLOCALERROROUT[5];
        output IFEXTPLPCIE1CFGHPOCFGLOCALERRORVALID;
        output IFEXTPLPCIE1CFGHPOCFGLTSSMSTATE[6];
        output IFEXTPLPCIE1CFGHPOCFGMGMTREADDATA[32];
        output IFEXTPLPCIE1CFGHPOCFGMGMTREADWRITEDONE;
        output IFEXTPLPCIE1CFGHPOCFGNEGOTIATEDWIDTH[3];
        output IFEXTPLPCIE1CFGHPOCFGPASIDENABLE[4];
        output IFEXTPLPCIE1CFGHPOCFGPASIDEXECPERMISSIONENABLE[4];
        output IFEXTPLPCIE1CFGHPOCFGPASIDPRIVILMODEENABLE[4];
        output IFEXTPLPCIE1CFGHPOCFGPHYLINKDOWN;
        output IFEXTPLPCIE1CFGHPOCFGPHYLINKSTATUS[2];
        output IFEXTPLPCIE1CFGHPOCFGPLSTATUSCHANGE;
        output IFEXTPLPCIE1CFGHPOCFGPOWERSTATECHANGEINTERRUPT;
        output IFEXTPLPCIE1CFGHPOCFGRXPMSTATE[2];
        output IFEXTPLPCIE1CFGHPOCFGTXPMSTATE[2];

        output IFEXTPLPCIE1UNIFIEDCQAXISAXISCQTDATA[512];
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISCQTKEEP[16];
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISCQTUSER[229];
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISPORT0CQTLAST;
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISPORT0RCTLAST;
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISPORT1CQTLAST;
        output IFEXTPLPCIE1UNIFIEDCQAXISAXISPORT1RCTLAST;
        input IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT0CQCRDT;
        output IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT0CQVLD;
        input IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT0RCCRDT;
        output IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT0RCVLD;
        input IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT1CQCRDT;
        output IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT1CQVLD;
        input IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT1RCCRDT;
        output IFEXTPLPCIE1UNIFIEDCQAXISPCIEAXISPORT1RCVLD;

        input IFPLEXTPCIE1AXIPCIECOMPLDELIVEREDTAG0[8];
        input IFPLEXTPCIE1AXIPCIECOMPLDELIVEREDTAG1[8];
        input IFPLEXTPCIE1AXIPCIECOMPLDELIVERED[2];
        output IFPLEXTPCIE1AXIPCIECQNPREQCOUNT[6];
        input IFPLEXTPCIE1AXIPCIECQNPREQ[2];
        input IFPLEXTPCIE1AXIPCIECQNPUSERCREDITRCVD;
        input IFPLEXTPCIE1AXIPCIECQPIPELINEEMPTY;
        input IFPLEXTPCIE1AXIPCIEPOSTEDREQDELIVERED;
        output IFPLEXTPCIE1AXIPCIERQSEQNUM0[6];
        output IFPLEXTPCIE1AXIPCIERQSEQNUM1[6];
        output IFPLEXTPCIE1AXIPCIERQSEQNUMVLD0;
        output IFPLEXTPCIE1AXIPCIERQSEQNUMVLD1;
        output IFPLEXTPCIE1AXIPCIERQTAG0[10];
        output IFPLEXTPCIE1AXIPCIERQTAG1[10];
        output IFPLEXTPCIE1AXIPCIERQTAGAV[4];
        output IFPLEXTPCIE1AXIPCIERQTAGVLD0;
        output IFPLEXTPCIE1AXIPCIERQTAGVLD1;
        output IFPLEXTPCIE1AXIPCIETFCNPDAV[4];
        output IFPLEXTPCIE1AXIPCIETFCNPHAV[4];
        input IFPLEXTPCIE1AXITLRXCOMPLCREDITRELEASED;
        input IFPLEXTPCIE1AXITLRXCOMPLHEADERCREDITRELEASEDVALUE[2];
        input IFPLEXTPCIE1AXITLRXCOMPLPAYLOADCREDITRELEASEDVALUE[7];
        input IFPLEXTPCIE1AXITLRXNONPOSTEDCREDITRELEASED;
        input IFPLEXTPCIE1AXITLRXNONPOSTEDPAYLOADCREDITRELEASEDVALUE[2];
        input IFPLEXTPCIE1AXITLRXPOSTEDCREDITRELEASED;
        input IFPLEXTPCIE1AXITLRXPOSTEDPAYLOADCREDITRELEASEDVALUE[7];

        input DFXCOMSCANCLKNEXT;
        input DFXCOMSCANEDTUPDTNEXT;
        input DFXCOMSCANENNEXT;
        input DFXCOMSCANMODERSTNEXT;

        input DFXDMASCANCHNLINEXT[16];
        output DFXDMASCANCHNLOUTEXT[16];
        input DFXDMASCANCNTRLCHNLINEXT;
        output DFXDMASCANCNTRLCHNLOUTEXT;
        input DFXDMASCANSPAREINEXT[3];
        output DFXDMASCANSPAREOUTEXT[3];

        input DFXINTSCANCHNLINEXT[17];
        output DFXINTSCANCHNLOUTEXT[17];
        input DFXINTSCANCLKDIVRSTNEXT;
        input DFXINTSCANCNTRLCHNLINEXT;
        output DFXINTSCANCNTRLCHNLOUTEXT;
        input DFXINTSCANPLLRESETEXT;
        input DFXINTSCANSPAREINEXT[3];
        output DFXINTSCANSPAREOUTEXT[3];

        input DFXL2SCANCHNLINEXT[10];
        output DFXL2SCANCHNLOUTEXT[10];
        input DFXL2SCANCNTRLCHNLINEXT;
        output DFXL2SCANCNTRLCHNLOUTEXT;
        input DFXL2SCANSPAREINEXT[3];
        output DFXL2SCANSPAREOUTEXT[3];

        input DFXPCIESCANCHNLINEXT[17];
        output DFXPCIESCANCHNLOUTEXT[17];
        input DFXPCIESCANCNTRLCHNLINEXT;
        output DFXPCIESCANCNTRLCHNLOUTEXT;
        input DFXPCIESCANSPAREINEXT[3];
        output DFXPCIESCANSPAREOUTEXT[3];

        input DFXPO0SCANCHNLINEXT[5];
        output DFXPO0SCANCHNLOUTEXT[5];
        input DFXPO0SCANCNTRLCHNLINEXT;
        output DFXPO0SCANCNTRLCHNLOUTEXT;
        input DFXPO0SCANSPAREINEXT[3];
        output DFXPO0SCANSPAREOUTEXT[3];

        input DFXPO1SCANCHNLINEXT[5];
        output DFXPO1SCANCHNLOUTEXT[5];
        input DFXPO1SCANCNTRLCHNLINEXT;
        output DFXPO1SCANCNTRLCHNLOUTEXT;
        input DFXPO1SCANSPAREINEXT[3];
        output DFXPO1SCANSPAREOUTEXT[3];
    }

    bel_class CPM5 {
        input PLCPM5DMA0USERCLK;
        input PLCPM5DMA1USERCLK;
        input PLCPM5IRQ0;
        input PLCPM5IRQ1;
        input PLCPM5LOOPBACK0EN;
        input PLCPM5LOOPBACK1EN;
        input PLCPM5PCIE0USERCLKFB;
        input PLCPM5PCIE1USERCLKFB;
        input PLCPM5REFCLK;

        output CPM5PLDMA0RESETN;
        output CPM5PLDMA1RESETN;

        input PLCPM5GPI0[32];
        input PLCPM5GPI1[32];
        output CPM5PLGPO0[32];
        output CPM5PLGPO1[32];

        output CPM5RCLK0[12];
        output CPM5RCLK1[12];

        output ISOCPM5PLN;
        output ISODFXCPM5PLN;

        output IFCPM5PLISRCORREVENT;
        output IFCPM5PLISRMISCEVENT;
        output IFCPM5PLISRUNCORREVENT;

        for i in 0..2 {
            input "PLCPM5AXI{i}CLK";

            input "IFCPM5PLAXI{i}ARREADY";
            output "IFCPM5PLAXI{i}ARVALID";
            output "IFCPM5PLAXI{i}ARADDR"[64];
            output "IFCPM5PLAXI{i}ARBURST"[2];
            output "IFCPM5PLAXI{i}ARCACHE"[4];
            output "IFCPM5PLAXI{i}ARID"[16];
            output "IFCPM5PLAXI{i}ARLEN"[8];
            output "IFCPM5PLAXI{i}ARLOCK"[1];
            output "IFCPM5PLAXI{i}ARPROT"[3];
            output "IFCPM5PLAXI{i}ARQOS"[4];
            output "IFCPM5PLAXI{i}ARREGION"[4];
            output "IFCPM5PLAXI{i}ARSIZE"[3];
            output "IFCPM5PLAXI{i}ARUSER"[32];

            input "IFCPM5PLAXI{i}AWREADY";
            output "IFCPM5PLAXI{i}AWVALID";
            output "IFCPM5PLAXI{i}AWADDR"[64];
            output "IFCPM5PLAXI{i}AWBURST"[2];
            output "IFCPM5PLAXI{i}AWCACHE"[4];
            output "IFCPM5PLAXI{i}AWID"[16];
            output "IFCPM5PLAXI{i}AWLEN"[8];
            output "IFCPM5PLAXI{i}AWLOCK"[1];
            output "IFCPM5PLAXI{i}AWPROT"[3];
            output "IFCPM5PLAXI{i}AWQOS"[4];
            output "IFCPM5PLAXI{i}AWREGION"[4];
            output "IFCPM5PLAXI{i}AWSIZE"[3];
            output "IFCPM5PLAXI{i}AWUSER"[32];

            output "IFCPM5PLAXI{i}BREADY";
            input "IFCPM5PLAXI{i}BVALID";
            input "IFCPM5PLAXI{i}BID"[16];
            input "IFCPM5PLAXI{i}BRESP"[2];
            input "IFCPM5PLAXI{i}BUSER"[1];

            output "IFCPM5PLAXI{i}RREADY";
            input "IFCPM5PLAXI{i}RVALID";
            input "IFCPM5PLAXI{i}RDATA"[512];
            input "IFCPM5PLAXI{i}RID"[16];
            input "IFCPM5PLAXI{i}RLAST"[1];
            input "IFCPM5PLAXI{i}RRESP"[2];
            input "IFCPM5PLAXI{i}RUSER"[64];

            input "IFCPM5PLAXI{i}WREADY";
            output "IFCPM5PLAXI{i}WVALID";
            output "IFCPM5PLAXI{i}WDATA"[512];
            output "IFCPM5PLAXI{i}WID"[16];
            output "IFCPM5PLAXI{i}WLAST"[1];
            output "IFCPM5PLAXI{i}WSTRB"[64];
            output "IFCPM5PLAXI{i}WUSER"[64];
        }

        for i in 0..2 {
            output "IFCPM5PLMPIO{i}AXISCCTREADY"[8];
            input "IFCPM5PLMPIO{i}AXISCCTVALID";
            input "IFCPM5PLMPIO{i}AXISCCTDATA"[1024];
            input "IFCPM5PLMPIO{i}AXISCCTKEEP"[32];
            input "IFCPM5PLMPIO{i}AXISCCTLAST";
            input "IFCPM5PLMPIO{i}AXISCCTUSER"[165];

            output "IFCPM5PLMPIO{i}AXISRQTREADY"[8];
            input "IFCPM5PLMPIO{i}AXISRQTVALID";
            input "IFCPM5PLMPIO{i}AXISRQTDATA"[1024];
            input "IFCPM5PLMPIO{i}AXISRQTKEEP"[32];
            input "IFCPM5PLMPIO{i}AXISRQTLAST";
            input "IFCPM5PLMPIO{i}AXISRQTUSER"[373];

            input "IFCPM5PLMPIO{i}PCIEAXISCQREADY"[60];
            input "IFCPM5PLMPIO{i}PCIEAXISCQRTS";
            output "IFCPM5PLMPIO{i}PCIEAXISCQVLD";
            input "IFCPM5PLMPIO{i}PCIEAXISRCREADY"[60];
            input "IFCPM5PLMPIO{i}PCIEAXISRCRTS";
            output "IFCPM5PLMPIO{i}PCIEAXISRCVLD";

            output "IFCPM5PLMPIO{i}AXISCQTDATA"[1024];
            output "IFCPM5PLMPIO{i}AXISCQTKEEP"[32];
            output "IFCPM5PLMPIO{i}AXISCQTLAST";
            output "IFCPM5PLMPIO{i}AXISCQTUSER"[465];
            output "IFCPM5PLMPIO{i}AXISRCTLAST";

            output "IFCPM5PLMPIO{i}PCIECQNPREQCOUNT"[6];
            input "IFCPM5PLMPIO{i}PCIECQNPREQ"[3];
            output "IFCPM5PLMPIO{i}PCIERQSEQNUM0"[6];
            output "IFCPM5PLMPIO{i}PCIERQSEQNUM1"[6];
            output "IFCPM5PLMPIO{i}PCIERQSEQNUM2"[6];
            output "IFCPM5PLMPIO{i}PCIERQSEQNUM3"[6];
            output "IFCPM5PLMPIO{i}PCIERQSEQNUMVLD"[4];
            output "IFCPM5PLMPIO{i}PCIERQTAGAV"[4];
            output "IFCPM5PLMPIO{i}PCIERQTAGVLD"[4];
            output "IFCPM5PLMPIO{i}PCIERQTAG"[40];
            output "IFCPM5PLMPIO{i}PCIETFCNPDAV"[4];
            output "IFCPM5PLMPIO{i}PCIETFCNPHAV"[4];

            input "IFCPM5PLMPIO{i}CFGMGMTFUNCTIONNUMBER"[16];
            input "IFCPM5PLMPIO{i}CFGMGMTADDR"[10];
            input "IFCPM5PLMPIO{i}CFGMGMTREAD";
            input "IFCPM5PLMPIO{i}CFGMGMTWRITE";
            input "IFCPM5PLMPIO{i}CFGMGMTDEBUGACCESS";
            input "IFCPM5PLMPIO{i}CFGMGMTWRITEDATA"[32];
            input "IFCPM5PLMPIO{i}CFGMGMTBYTEENABLE"[4];
            output "IFCPM5PLMPIO{i}CFGMGMTREADDATA"[32];
            output "IFCPM5PLMPIO{i}CFGMGMTREADWRITEDONE";

            input "IFCPM5PLMPIO{i}CFGMGMT2FUNCTIONNUMBER"[16];
            input "IFCPM5PLMPIO{i}CFGMGMT2ADDR"[10];
            input "IFCPM5PLMPIO{i}CFGMGMT2READ";
            input "IFCPM5PLMPIO{i}CFGMGMT2WRITE";
            input "IFCPM5PLMPIO{i}CFGMGMT2DEBUGACCESS";
            input "IFCPM5PLMPIO{i}CFGMGMT2WRITEDATA"[32];
            input "IFCPM5PLMPIO{i}CFGMGMT2BYTEENABLE"[4];
            output "IFCPM5PLMPIO{i}CFGMGMT2READDATA"[32];
            output "IFCPM5PLMPIO{i}CFGMGMT2READWRITEDONE";

            output "IFCPM5PLMPIO{i}CFGEXTFUNCTIONNUMBER"[16];
            output "IFCPM5PLMPIO{i}CFGEXTREGISTERNUMBER"[10];
            output "IFCPM5PLMPIO{i}CFGEXTREADRECEIVED";
            output "IFCPM5PLMPIO{i}CFGEXTWRITERECEIVED";
            output "IFCPM5PLMPIO{i}CFGEXTWRITEDATA"[32];
            output "IFCPM5PLMPIO{i}CFGEXTWRITEBYTEENABLE"[4];
            input "IFCPM5PLMPIO{i}CFGEXTREADDATAVALID";
            input "IFCPM5PLMPIO{i}CFGEXTREADDATA"[32];

            output "IFCPM5PLMPIO{i}CFG10BTAGREQUESTERENABLE";
            output "IFCPM5PLMPIO{i}CFGATOMICREQUESTERENABLE";
            output "IFCPM5PLMPIO{i}CFGBUSNUMBER"[8];
            input "IFCPM5PLMPIO{i}CFGCCIXEDRDATARATECHANGEACK";
            output "IFCPM5PLMPIO{i}CFGCCIXEDRDATARATECHANGEREQ";
            output "IFCPM5PLMPIO{i}CFGCURRENTSPEED"[3];
            output "IFCPM5PLMPIO{i}CFGEDRENABLE";
            input "IFCPM5PLMPIO{i}CFGERRCORIN";
            output "IFCPM5PLMPIO{i}CFGERRCOROUT";
            output "IFCPM5PLMPIO{i}CFGERRFATALOUT";
            output "IFCPM5PLMPIO{i}CFGERRMSGFUNCTIONNUMBER"[10];
            output "IFCPM5PLMPIO{i}CFGERRMSGTYPEVLD";
            output "IFCPM5PLMPIO{i}CFGERRMSGTYPE"[6];
            output "IFCPM5PLMPIO{i}CFGERRNONFATALOUT";
            input "IFCPM5PLMPIO{i}CFGERRUNCORIN";
            output "IFCPM5PLMPIO{i}CFGEXTTAGENABLE";

            output "IFCPM5PLMPIO{i}CFGFCCPLDSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCCPLD"[12];
            output "IFCPM5PLMPIO{i}CFGFCCPLHSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCCPLH"[8];
            output "IFCPM5PLMPIO{i}CFGFCNPDSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCNPD"[12];
            output "IFCPM5PLMPIO{i}CFGFCNPHSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCNPH"[8];
            output "IFCPM5PLMPIO{i}CFGFCPDSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCPD"[12];
            output "IFCPM5PLMPIO{i}CFGFCPHSCALE"[2];
            output "IFCPM5PLMPIO{i}CFGFCPH"[8];
            input "IFCPM5PLMPIO{i}CFGFCSEL"[3];
            input "IFCPM5PLMPIO{i}CFGFCVCSEL";

            input "IFCPM5PLMPIO{i}CFGFLRDONE";
            input "IFCPM5PLMPIO{i}CFGFLRDONEFUNCNUM"[16];
            output "IFCPM5PLMPIO{i}CFGFUNCTIONPOWERSTATE"[3];
            output "IFCPM5PLMPIO{i}CFGFUNCTIONSTATUS"[4];
            input "IFCPM5PLMPIO{i}CFGHOTRESETIN";
            output "IFCPM5PLMPIO{i}CFGHOTRESETOUT";

            input "IFCPM5PLMPIO{i}CFGINTERRUPTINT"[4];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIATTR"[3];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIDATA"[32];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIENABLE";
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIFAIL";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIFUNCTIONNUMBER"[16];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIINT"[32];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIMASKUPDATE";
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIMMENABLE"[3];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIPENDINGSTATUSDATAENABLE";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIPENDINGSTATUSFUNCTIONNUM"[4];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIPENDINGSTATUS"[32];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSISELECT"[4];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSISENT";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSITPHPRESENT";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSITPHSTTAG"[8];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSITPHTYPE"[2];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXADDRESS"[64];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXDATA"[32];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXENABLE";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXINT";
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXMASK";
            output "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXVECPENDINGSTATUS";
            input "IFCPM5PLMPIO{i}CFGINTERRUPTMSIXVECPENDING"[2];
            input "IFCPM5PLMPIO{i}CFGINTERRUPTPENDING"[16];
            output "IFCPM5PLMPIO{i}CFGINTERRUPTSENT";

            output "IFCPM5PLMPIO{i}CFGLINKPOWERSTATE"[2];
            output "IFCPM5PLMPIO{i}CFGLOCALERROROUT"[5];
            output "IFCPM5PLMPIO{i}CFGLOCALERRORVALID";
            output "IFCPM5PLMPIO{i}CFGLTSSMSTATE"[6];
            output "IFCPM5PLMPIO{i}CFGMAXPAYLOAD"[2];
            output "IFCPM5PLMPIO{i}CFGMAXREADREQ"[3];

            output "IFCPM5PLMPIO{i}CFGMSGRECEIVED";
            output "IFCPM5PLMPIO{i}CFGMSGRECEIVEDDATA"[8];
            output "IFCPM5PLMPIO{i}CFGMSGRECEIVEDTYPE"[5];
            input "IFCPM5PLMPIO{i}CFGMSGTRANSMIT";
            input "IFCPM5PLMPIO{i}CFGMSGTRANSMITDATA"[32];
            output "IFCPM5PLMPIO{i}CFGMSGTRANSMITDONE";
            input "IFCPM5PLMPIO{i}CFGMSGTRANSMITTYPE"[3];

            output "IFCPM5PLMPIO{i}CFGNEGOTIATEDWIDTH"[3];
            output "IFCPM5PLMPIO{i}CFGPASIDENABLE";
            output "IFCPM5PLMPIO{i}CFGPASIDEXECPERMISSIONENABLE";
            output "IFCPM5PLMPIO{i}CFGPASIDPRIVILMODEENABLE";
            input "IFCPM5PLMPIO{i}CFGPERFUNCFUNCNUM"[16];
            output "IFCPM5PLMPIO{i}CFGPERFUNCOUT"[24];
            input "IFCPM5PLMPIO{i}CFGPERFUNCREQ";
            output "IFCPM5PLMPIO{i}CFGPERFUNCVLD";
            output "IFCPM5PLMPIO{i}CFGPHYLINKDOWN";
            output "IFCPM5PLMPIO{i}CFGPHYLINKSTATUS"[2];
            output "IFCPM5PLMPIO{i}CFGPLSTATUSCHANGE";
            input "IFCPM5PLMPIO{i}CFGPOWERSTATECHANGEACK";
            output "IFCPM5PLMPIO{i}CFGPOWERSTATECHANGEINTERRUPT";
            output "IFCPM5PLMPIO{i}CFGRCBSTATUS";

            input "IFCPM5PLMPIO{i}CFGREQRXTREADY";
            output "IFCPM5PLMPIO{i}CFGREQRXTVALID";
            output "IFCPM5PLMPIO{i}CFGREQRXTDATA"[128];
            output "IFCPM5PLMPIO{i}CFGREQRXTUSER"[17];
            input "IFCPM5PLMPIO{i}CFGREQRXACK";

            output "IFCPM5PLMPIO{i}CFGREQTXTREADY";
            input "IFCPM5PLMPIO{i}CFGREQTXTVALID";
            input "IFCPM5PLMPIO{i}CFGREQTXTDATA"[128];
            input "IFCPM5PLMPIO{i}CFGREQTXTUSER"[17];

            output "IFCPM5PLMPIO{i}CFGRXPMSTATE"[2];
            output "IFCPM5PLMPIO{i}CFGTXPMSTATE"[2];
            output "IFCPM5PLMPIO{i}CFGVC1ENABLE";
            output "IFCPM5PLMPIO{i}CFGVC1NEGOTIATIONPENDING";
            output "IFCPM5PLMPIO{i}CFGWRREQBMEVLD";
            output "IFCPM5PLMPIO{i}CFGWRREQFLRVLD";
            output "IFCPM5PLMPIO{i}CFGWRREQFUNCNUM"[16];
            output "IFCPM5PLMPIO{i}CFGWRREQMSIVLD";
            output "IFCPM5PLMPIO{i}CFGWRREQMSIXVLD";
            output "IFCPM5PLMPIO{i}CFGWRREQOUTVALUE"[4];
            output "IFCPM5PLMPIO{i}CFGWRREQVFEVLD";
        }

        for i in 0..2 {
            input "IFDPLL{i}PLTSTDADDR"[7];
            input "IFDPLL{i}PLTSTDCLK";
            input "IFDPLL{i}PLTSTDEN";
            input "IFDPLL{i}PLTSTDI"[16];
            output "IFDPLL{i}PLTSTDOUT"[16];
            output "IFDPLL{i}PLTSTDRDY";
            input "IFDPLL{i}PLTSTDWE";
            output "IFDPLL{i}PLTSTLOCKED";
            input "IFDPLL{i}PLTSTPSCLK";
            output "IFDPLL{i}PLTSTPSDONE";
            input "IFDPLL{i}PLTSTPSEN";
            input "IFDPLL{i}PLTSTPSINCDEC";
            input "IFDPLL{i}PLTSTPWRDWN";
            input "IFDPLL{i}PLTSTRST";
            input "IFDPLL{i}PLTSTTESTIN"[8];
            output "IFDPLL{i}PLTSTTESTOUT"[4];
        }

        for i in 0..2 {
            input "PLCPM5CHI{i}CLK";
            output "IFPLCPM5P{i}CHICRSPFLITPEND";
            output "IFPLCPM5P{i}CHICRSPFLITV";
            output "IFPLCPM5P{i}CHICRSPFLIT"[51];
            input "IFPLCPM5P{i}CHICRSPLCRDV";
            output "IFPLCPM5P{i}CHIMLINKACTIVEACK";
            input "IFPLCPM5P{i}CHIMLINKACTIVEREQ";
            input "IFPLCPM5P{i}CHIMSACTIVE";
            output "IFPLCPM5P{i}CHIRDATFLITPEND";
            output "IFPLCPM5P{i}CHIRDATFLITV";
            output "IFPLCPM5P{i}CHIRDATFLIT"[705];
            input "IFPLCPM5P{i}CHIRDATLCRDV";
            input "IFPLCPM5P{i}CHIREQFLITPEND";
            input "IFPLCPM5P{i}CHIREQFLITV";
            input "IFPLCPM5P{i}CHIREQFLIT"[121];
            output "IFPLCPM5P{i}CHIREQLCRDV";
            input "IFPLCPM5P{i}CHISLINKACTIVEACK";
            output "IFPLCPM5P{i}CHISLINKACTIVEREQ";
            output "IFPLCPM5P{i}CHISNPFLITPEND";
            output "IFPLCPM5P{i}CHISNPFLITV";
            output "IFPLCPM5P{i}CHISNPFLIT"[88];
            input "IFPLCPM5P{i}CHISNPLCRDV";
            input "IFPLCPM5P{i}CHISRSPFLITPEND";
            input "IFPLCPM5P{i}CHISRSPFLITV";
            input "IFPLCPM5P{i}CHISRSPFLIT"[51];
            output "IFPLCPM5P{i}CHISRSPLCRDV";
            output "IFPLCPM5P{i}CHISSACTIVE";
            output "IFPLCPM5P{i}CHISYSCOACK";
            input "IFPLCPM5P{i}CHISYSCOREQ";
            input "IFPLCPM5P{i}CHIWDATFLITPEND";
            input "IFPLCPM5P{i}CHIWDATFLITV";
            input "IFPLCPM5P{i}CHIWDATFLIT"[705];
            output "IFPLCPM5P{i}CHIWDATLCRDV";
        }

        input IFPLCPM5BOTGTYPFTI64APB3CLK;
        input IFPLCPM5BOTGTYPFTI64APB3PADDR[16];
        input IFPLCPM5BOTGTYPFTI64APB3PENABLE;
        output IFPLCPM5BOTGTYPFTI64APB3PRDATA[32];
        output IFPLCPM5BOTGTYPFTI64APB3PREADY;
        input IFPLCPM5BOTGTYPFTI64APB3PRESETN;
        input IFPLCPM5BOTGTYPFTI64APB3PSEL[2];
        output IFPLCPM5BOTGTYPFTI64APB3PSLVERR;
        input IFPLCPM5BOTGTYPFTI64APB3PWDATA[32];
        input IFPLCPM5BOTGTYPFTI64APB3PWRITE;
        for i in 0..4 {
            input "IFPLCPM5BOTGTYPFTI64CH{i}EYESCANRESET";
            input "IFPLCPM5BOTGTYPFTI64CH{i}GTTRXRESET";
            input "IFPLCPM5BOTGTYPFTI64CH{i}GTTXRESET";
            input "IFPLCPM5BOTGTYPFTI64CH{i}HSDPPCSRESET";
            input "IFPLCPM5BOTGTYPFTI64CH{i}ILORESET";
            output "IFPLCPM5BOTGTYPFTI64CH{i}ILORESETDONE";
            output "IFPLCPM5BOTGTYPFTI64CH{i}PHYSTATUS";
            input "IFPLCPM5BOTGTYPFTI64CH{i}RXCDRRESET";
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXPMARESETDONE";
            input "IFPLCPM5BOTGTYPFTI64CH{i}RXPMARESETMASK"[7];
            input "IFPLCPM5BOTGTYPFTI64CH{i}RXPRBSCNTRESET";
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXPRBSERR";
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXPRBSLOCKED";
            input "IFPLCPM5BOTGTYPFTI64CH{i}RXPROGDIVRESET";
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXPROGDIVRESETDONE";
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXRESETDONE";
            input "IFPLCPM5BOTGTYPFTI64CH{i}RXRESETMODE"[2];
            output "IFPLCPM5BOTGTYPFTI64CH{i}RXSTATUS"[3];
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXDETECTRX";
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXELECIDLE";
            output "IFPLCPM5BOTGTYPFTI64CH{i}TXPMARESETDONE";
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXPMARESETMASK"[3];
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXPRBSFORCEERR";
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXPROGDIVRESET";
            output "IFPLCPM5BOTGTYPFTI64CH{i}TXPROGDIVRESETDONE";
            output "IFPLCPM5BOTGTYPFTI64CH{i}TXRESETDONE";
            input "IFPLCPM5BOTGTYPFTI64CH{i}TXRESETMODE"[2];
        }
        input IFPLCPM5BOTGTYPFTI64DMONFIFORESET;
        input IFPLCPM5BOTGTYPFTI64DMONITORCLK;
        output IFPLCPM5BOTGTYPFTI64DMONITOROUT[64];
        input IFPLCPM5BOTGTYPFTI64GTPLLRST;
        output IFPLCPM5BOTGTYPFTI64HSCLK0LCPLLLOCK;
        output IFPLCPM5BOTGTYPFTI64HSCLK0RPLLLOCK;
        output IFPLCPM5BOTGTYPFTI64HSCLK1LCPLLLOCK;
        output IFPLCPM5BOTGTYPFTI64HSCLK1RPLLLOCK;
        input IFPLCPM5BOTGTYPFTI64USRCLK;

        input IFPLCPM5TOPGTYPFTI64APB3CLK;
        input IFPLCPM5TOPGTYPFTI64APB3PADDR[16];
        input IFPLCPM5TOPGTYPFTI64APB3PENABLE;
        output IFPLCPM5TOPGTYPFTI64APB3PRDATA[32];
        output IFPLCPM5TOPGTYPFTI64APB3PREADY;
        input IFPLCPM5TOPGTYPFTI64APB3PRESETN;
        input IFPLCPM5TOPGTYPFTI64APB3PSEL[2];
        output IFPLCPM5TOPGTYPFTI64APB3PSLVERR;
        input IFPLCPM5TOPGTYPFTI64APB3PWDATA[32];
        input IFPLCPM5TOPGTYPFTI64APB3PWRITE;
        for i in 0..4 {
            input "IFPLCPM5TOPGTYPFTI64CH{i}EYESCANRESET";
            input "IFPLCPM5TOPGTYPFTI64CH{i}GTTRXRESET";
            input "IFPLCPM5TOPGTYPFTI64CH{i}GTTXRESET";
            input "IFPLCPM5TOPGTYPFTI64CH{i}HSDPPCSRESET";
            input "IFPLCPM5TOPGTYPFTI64CH{i}ILORESET";
            output "IFPLCPM5TOPGTYPFTI64CH{i}ILORESETDONE";
            output "IFPLCPM5TOPGTYPFTI64CH{i}PHYSTATUS";
            input "IFPLCPM5TOPGTYPFTI64CH{i}RXCDRRESET";
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXPMARESETDONE";
            input "IFPLCPM5TOPGTYPFTI64CH{i}RXPMARESETMASK"[7];
            input "IFPLCPM5TOPGTYPFTI64CH{i}RXPRBSCNTRESET";
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXPRBSERR";
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXPRBSLOCKED";
            input "IFPLCPM5TOPGTYPFTI64CH{i}RXPROGDIVRESET";
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXPROGDIVRESETDONE";
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXRESETDONE";
            input "IFPLCPM5TOPGTYPFTI64CH{i}RXRESETMODE"[2];
            output "IFPLCPM5TOPGTYPFTI64CH{i}RXSTATUS"[3];
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXDETECTRX";
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXELECIDLE";
            output "IFPLCPM5TOPGTYPFTI64CH{i}TXPMARESETDONE";
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXPMARESETMASK"[3];
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXPRBSFORCEERR";
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXPROGDIVRESET";
            output "IFPLCPM5TOPGTYPFTI64CH{i}TXPROGDIVRESETDONE";
            output "IFPLCPM5TOPGTYPFTI64CH{i}TXRESETDONE";
            input "IFPLCPM5TOPGTYPFTI64CH{i}TXRESETMODE"[2];
        }
        input IFPLCPM5TOPGTYPFTI64DMONFIFORESET;
        input IFPLCPM5TOPGTYPFTI64DMONITORCLK;
        output IFPLCPM5TOPGTYPFTI64DMONITOROUT[64];
        input IFPLCPM5TOPGTYPFTI64GTPLLRST;
        output IFPLCPM5TOPGTYPFTI64HSCLK0LCPLLLOCK;
        output IFPLCPM5TOPGTYPFTI64HSCLK0RPLLLOCK;
        output IFPLCPM5TOPGTYPFTI64HSCLK1LCPLLLOCK;
        output IFPLCPM5TOPGTYPFTI64HSCLK1RPLLLOCK;
        input IFPLCPM5TOPGTYPFTI64USRCLK;

        for i in 0..2 {
            input "IFCMN{i}DFXSCW3F2CSCANCHNLINEXT"[3];
            input "IFCMN{i}DFXSCW3F2CSCANCNTRLCHNLINEXT";
            input "IFCMN{i}DFXSCW3F2CSCANSPAREINEXT"[3];
            output "IFCMN{i}DFXSCW3C2FSCANCHNLOUTEXT"[3];
            output "IFCMN{i}DFXSCW3C2FSCANCNTRLCHNLOUTEXT";
            output "IFCMN{i}DFXSCW3C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..2 {
            input "IFCMNCXLA{i}DFXSCW3F2CSCANCHNLINEXT"[3];
            input "IFCMNCXLA{i}DFXSCW3F2CSCANCNTRLCHNLINEXT";
            input "IFCMNCXLA{i}DFXSCW3F2CSCANSPAREINEXT"[3];
            output "IFCMNCXLA{i}DFXSCW3C2FSCANCHNLOUTEXT"[3];
            output "IFCMNCXLA{i}DFXSCW3C2FSCANCNTRLCHNLOUTEXT";
            output "IFCMNCXLA{i}DFXSCW3C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..2 {
            input "IFCMNCXRH{i}DFXSCW6F2CSCANCHNLINEXT"[6];
            input "IFCMNCXRH{i}DFXSCW6F2CSCANCNTRLCHNLINEXT";
            input "IFCMNCXRH{i}DFXSCW6F2CSCANSPAREINEXT"[3];
            output "IFCMNCXRH{i}DFXSCW6C2FSCANCHNLOUTEXT"[6];
            output "IFCMNCXRH{i}DFXSCW6C2FSCANCNTRLCHNLOUTEXT";
            output "IFCMNCXRH{i}DFXSCW6C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..2 {
            input "IFCMNHNF{i}DFXSCW3F2CSCANCHNLINEXT"[3];
            input "IFCMNHNF{i}DFXSCW3F2CSCANCNTRLCHNLINEXT";
            input "IFCMNHNF{i}DFXSCW3F2CSCANSPAREINEXT"[3];
            output "IFCMNHNF{i}DFXSCW3C2FSCANCHNLOUTEXT"[3];
            output "IFCMNHNF{i}DFXSCW3C2FSCANCNTRLCHNLOUTEXT";
            output "IFCMNHNF{i}DFXSCW3C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..2 {
            input "IFDMA{i}DFXCTRLF2CSCANCLKNEXT";
            input "IFDMA{i}DFXCTRLF2CSCANMODERSTNEXT";
            input "IFDMA{i}DFXCTRLF2CSCANENNEXT";
            input "IFDMA{i}DFXCTRLF2CSCANEDTUPDTNEXT";
            input "IFDMA{i}INTDFXSCW7F2CSCANCHNLINEXT"[7];
            input "IFDMA{i}INTDFXSCW7F2CSCANCNTRLCHNLINEXT";
            input "IFDMA{i}INTDFXSCW7F2CSCANSPAREINEXT"[3];
            output "IFDMA{i}INTDFXSCW7C2FSCANCHNLOUTEXT"[7];
            output "IFDMA{i}INTDFXSCW7C2FSCANCNTRLCHNLOUTEXT";
            output "IFDMA{i}INTDFXSCW7C2FSCANSPAREOUTEXT"[3];
            input "IFDMA{i}TOPDFXSCW10F2CSCANCHNLINEXT"[10];
            input "IFDMA{i}TOPDFXSCW10F2CSCANCNTRLCHNLINEXT";
            input "IFDMA{i}TOPDFXSCW10F2CSCANSPAREINEXT"[3];
            output "IFDMA{i}TOPDFXSCW10C2FSCANCHNLOUTEXT"[10];
            output "IFDMA{i}TOPDFXSCW10C2FSCANCNTRLCHNLOUTEXT";
            output "IFDMA{i}TOPDFXSCW10C2FSCANSPAREOUTEXT"[3];
        }

        input IFINTWBDFXSCW6F2CSCANCHNLINEXT[6];
        input IFINTWBDFXSCW6F2CSCANCNTRLCHNLINEXT;
        input IFINTWBDFXSCW6F2CSCANSPAREINEXT[3];
        output IFINTWBDFXSCW6C2FSCANCHNLOUTEXT[6];
        output IFINTWBDFXSCW6C2FSCANCNTRLCHNLOUTEXT;
        output IFINTWBDFXSCW6C2FSCANSPAREOUTEXT[3];
        input IFINTWTDFXSCW6F2CSCANCHNLINEXT[6];
        input IFINTWTDFXSCW6F2CSCANCNTRLCHNLINEXT;
        input IFINTWTDFXSCW6F2CSCANSPAREINEXT[3];
        output IFINTWTDFXSCW6C2FSCANCHNLOUTEXT[6];
        output IFINTWTDFXSCW6C2FSCANCNTRLCHNLOUTEXT;
        output IFINTWTDFXSCW6C2FSCANSPAREOUTEXT[3];

        for i in 0..2 {
            input "IFL2{i}DFXCTRLF2CSCANCLKNEXT";
            input "IFL2{i}DFXCTRLF2CSCANEDTUPDTNEXT";
            input "IFL2{i}DFXCTRLF2CSCANENNEXT";
            input "IFL2{i}DFXCTRLF2CSCANMODERSTNEXT";
            input "IFL2{i}DFXSCW5F2CSCANCHNLINEXT"[5];
            input "IFL2{i}DFXSCW5F2CSCANCNTRLCHNLINEXT";
            input "IFL2{i}DFXSCW5F2CSCANSPAREINEXT"[3];
            output "IFL2{i}DFXSCW5C2FSCANCHNLOUTEXT"[5];
            output "IFL2{i}DFXSCW5C2FSCANCNTRLCHNLOUTEXT";
            output "IFL2{i}DFXSCW5C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..2 {
            input "IFPCIE{i}ATTRDFXSCW6F2CSCANCHNLINEXT"[6];
            input "IFPCIE{i}ATTRDFXSCW6F2CSCANCNTRLCHNLINEXT";
            input "IFPCIE{i}ATTRDFXSCW6F2CSCANSPAREINEXT"[3];
            output "IFPCIE{i}ATTRDFXSCW6C2FSCANCHNLOUTEXT"[6];
            output "IFPCIE{i}ATTRDFXSCW6C2FSCANCNTRLCHNLOUTEXT";
            output "IFPCIE{i}ATTRDFXSCW6C2FSCANSPAREOUTEXT"[3];
            input "IFPCIE{i}PSBDFXSCW10F2CSCANCHNLINEXT"[10];
            input "IFPCIE{i}PSBDFXSCW10F2CSCANCNTRLCHNLINEXT";
            input "IFPCIE{i}PSBDFXSCW10F2CSCANSPAREINEXT"[3];
            output "IFPCIE{i}PSBDFXSCW10C2FSCANCHNLOUTEXT"[10];
            output "IFPCIE{i}PSBDFXSCW10C2FSCANCNTRLCHNLOUTEXT";
            output "IFPCIE{i}PSBDFXSCW10C2FSCANSPAREOUTEXT"[3];
        }

        for i in 0..4 {
            input "IFPLCPM5DFXBOTSCANCLKNFSBQ{i}";
            input "IFPLCPM5DFXBOTSCANMODERSTNFSBQ{i}";
            input "IFPLCPM5DFXBOTSCANENNFSBQ{i}";
            input "IFPLCPM5DFXBOTSCANEDTUPDTNFSBQ{i}";
            input "IFPLCPM5DFXBOTSCANCNTRLCHNLINFSBQ{i}";
            output "IFPLCPM5DFXBOTSCANCNTRLCHNLOUTSFBQ{i}";
            input "IFPLCPM5DFXBOTSCANCHNLINFSBQ{i}"[4];
            output "IFPLCPM5DFXBOTSCANCHNLOUTSFBQ{i}"[4];
        }

        input IFPLCPM5DFXBOTSCANCLKNGTCTRLFSBQ;
        input IFPLCPM5DFXBOTSCANMODERSTNGTCTRLFSBQ;
        input IFPLCPM5DFXBOTSCANENNGTCTRLFSBQ;
        input IFPLCPM5DFXBOTSCANEDTUPDTNGTCTRLFSBQ;
        input IFPLCPM5DFXBOTSCANCNTRLCHNLINGTCTRLFSBQ;
        output IFPLCPM5DFXBOTSCANCNTRLCHNLOUTGTCTRLSFBQ;
        input IFPLCPM5DFXBOTSCANCHNLINGTCTRLFSBQ[4];
        output IFPLCPM5DFXBOTSCANCHNLOUTGTCTRLSFBQ[4];

        for i in 0..4 {
            input "IFPLCPM5DFXBOTSCANCLKNFSTQ{i}";
            input "IFPLCPM5DFXBOTSCANMODERSTNFSTQ{i}";
            input "IFPLCPM5DFXBOTSCANENNFSTQ{i}";
            input "IFPLCPM5DFXBOTSCANEDTUPDTNFSTQ{i}";
            input "IFPLCPM5DFXBOTSCANCNTRLCHNLINFSTQ{i}";
            output "IFPLCPM5DFXBOTSCANCNTRLCHNLOUTSFTQ{i}";
            input "IFPLCPM5DFXBOTSCANCHNLINFSTQ{i}"[4];
            output "IFPLCPM5DFXBOTSCANCHNLOUTSFTQ{i}"[4];
        }

        input IFPLCPM5DFXBOTSCANCLKNGTCTRLFSTQ;
        input IFPLCPM5DFXBOTSCANMODERSTNGTCTRLFSTQ;
        input IFPLCPM5DFXBOTSCANENNGTCTRLFSTQ;
        input IFPLCPM5DFXBOTSCANEDTUPDTNGTCTRLFSTQ;
        input IFPLCPM5DFXBOTSCANCNTRLCHNLINGTCTRLFSTQ;
        output IFPLCPM5DFXBOTSCANCNTRLCHNLOUTGTCTRLSFTQ;
        input IFPLCPM5DFXBOTSCANCHNLINGTCTRLFSTQ[4];
        output IFPLCPM5DFXBOTSCANCHNLOUTGTCTRLSFTQ[4];

        for i in 0..4 {
            input "IFPLCPM5DFXTOPSCANCLKNFSBQ{i}";
            input "IFPLCPM5DFXTOPSCANMODERSTNFSBQ{i}";
            input "IFPLCPM5DFXTOPSCANENNFSBQ{i}";
            input "IFPLCPM5DFXTOPSCANEDTUPDTNFSBQ{i}";
            input "IFPLCPM5DFXTOPSCANCNTRLCHNLINFSBQ{i}";
            output "IFPLCPM5DFXTOPSCANCNTRLCHNLOUTSFBQ{i}";
            input "IFPLCPM5DFXTOPSCANCHNLINFSBQ{i}"[4];
            output "IFPLCPM5DFXTOPSCANCHNLOUTSFBQ{i}"[4];
        }

        input IFPLCPM5DFXTOPSCANCLKNGTCTRLFSBQ;
        input IFPLCPM5DFXTOPSCANMODERSTNGTCTRLFSBQ;
        input IFPLCPM5DFXTOPSCANENNGTCTRLFSBQ;
        input IFPLCPM5DFXTOPSCANEDTUPDTNGTCTRLFSBQ;
        input IFPLCPM5DFXTOPSCANCNTRLCHNLINGTCTRLFSBQ;
        output IFPLCPM5DFXTOPSCANCNTRLCHNLOUTGTCTRLSFBQ;
        input IFPLCPM5DFXTOPSCANCHNLINGTCTRLFSBQ[4];
        output IFPLCPM5DFXTOPSCANCHNLOUTGTCTRLSFBQ[4];

        for i in 0..4 {
            input "IFPLCPM5DFXTOPSCANCLKNFSTQ{i}";
            input "IFPLCPM5DFXTOPSCANMODERSTNFSTQ{i}";
            input "IFPLCPM5DFXTOPSCANENNFSTQ{i}";
            input "IFPLCPM5DFXTOPSCANEDTUPDTNFSTQ{i}";
            input "IFPLCPM5DFXTOPSCANCNTRLCHNLINFSTQ{i}";
            output "IFPLCPM5DFXTOPSCANCNTRLCHNLOUTSFTQ{i}";
            input "IFPLCPM5DFXTOPSCANCHNLINFSTQ{i}"[4];
            output "IFPLCPM5DFXTOPSCANCHNLOUTSFTQ{i}"[4];
        }

        input IFPLCPM5DFXTOPSCANCLKNGTCTRLFSTQ;
        input IFPLCPM5DFXTOPSCANMODERSTNGTCTRLFSTQ;
        input IFPLCPM5DFXTOPSCANENNGTCTRLFSTQ;
        input IFPLCPM5DFXTOPSCANEDTUPDTNGTCTRLFSTQ;
        input IFPLCPM5DFXTOPSCANCNTRLCHNLINGTCTRLFSTQ;
        output IFPLCPM5DFXTOPSCANCNTRLCHNLOUTGTCTRLSFTQ;
        input IFPLCPM5DFXTOPSCANCHNLINGTCTRLFSTQ[4];
        output IFPLCPM5DFXTOPSCANCHNLOUTGTCTRLSFTQ[4];
    }

    bel_class CPM6 {
        output CPMCLKBUFGT0[30];
        output CPMCLKBUFGT1[30];
        output CPMCLKDPLL0[10];
        output CPMCLKDPLL1[10];

        input PLCPMREFCLK;

        input PLAXIMMCLK0;
        input PLAXIMMCLK1;
        output PLAXIMMRSTN0;
        output PLAXIMMRSTN1;

        input PLCXLCLK0;
        input PLCXLCLK1;
        output PLCXLRSTN0;
        output PLCXLRSTN1;

        input PLDBICLK0;
        input PLDBICLK1;
        output PLDBIRSTN0;
        output PLDBIRSTN1;

        input PLSTRMCLK0;
        input PLSTRMCLK1;
        output PLSTRMRSTN0;
        output PLSTRMRSTN1;

        input PLRXUSRCLKQ0[4];
        input PLRXUSRCLKQ1[4];
        input PLRXUSRCLKQ2[4];
        input PLRXUSRCLKQ3[4];
        input PLTXUSRCLKQ0[4];
        input PLTXUSRCLKQ1[4];
        input PLTXUSRCLKQ2[4];
        input PLTXUSRCLKQ3[4];

        input PLMUXINBOTFSR[5312];
        input PLMUXINTOPFSR[6440];
        output PLMUXOUTBOTFSR[4104];
        output PLMUXOUTTOPFSR[4554];

        output ISOCPMPLN;

        input BOTCPM6SCANF2CSCANCLKNEXT;
        input TOPCPM6SCANF2CSCANCLKNEXT;

        input BOTGTSCANSCANCLKNFS0M;
        input BOTGTSCANSCANCLKNFS1M;
        input BOTGTSCANSCANCLKNFS2M;
        input BOTGTSCANSCANCLKNFS3M;
        input BOTGTSCANSCANCLKNFSTOP0M;
        input BOTGTSCANSCANCLKNFSTOP1M;
        input BOTGTSCANSCANCLKNFSTOP2M;
        input BOTGTSCANSCANCLKNFSTOP3M;
        input BOTGTSCANSCANCLKNGTCTRLFSM;
        input BOTGTSCANSCANCLKNGTCTRLFSTOPM;

        input TOPGTSCANSCANCLKNFS0M;
        input TOPGTSCANSCANCLKNFS1M;
        input TOPGTSCANSCANCLKNFS2M;
        input TOPGTSCANSCANCLKNFS3M;
        input TOPGTSCANSCANCLKNFSTOP0M;
        input TOPGTSCANSCANCLKNFSTOP1M;
        input TOPGTSCANSCANCLKNFSTOP2M;
        input TOPGTSCANSCANCLKNFSTOP3M;
        input TOPGTSCANSCANCLKNGTCTRLFSM;
        input TOPGTSCANSCANCLKNGTCTRLFSTOPM;
    }

    bel_class PSXL {
        // Clock outputs (BUFG_PS)

        output LPDRCLKCLK[6];
        output PMCRCLKCLK[6];
        output PSPLFPDRCLKCLK[12];

        // Clock inputs

        input PLPMCAUXREFCLK;
        input PLLPDAUXREFCLK;
        input PLFPDAUXREFCLK;

        // Reset outputs

        output PLRST0N;
        output PLRST1N;
        output PLRST2N;
        output PLRST3N;

        // CPUs

        input PLPSAPU0GICFIQ[4];
        input PLPSAPU0GICIRQ[4];
        input PLPSAPU1GICFIQ[4];
        input PLPSAPU1GICIRQ[4];
        input PLPSAPU2GICFIQ[4];
        input PLPSAPU2GICIRQ[4];
        input PLPSAPU3GICFIQ[4];
        input PLPSAPU3GICIRQ[4];

        input PLPSEVENTIREQ[4];
        input PLPSEVENTIREQCHK[4];
        output PSPLEVENTIACK[4];
        output PSPLEVENTIACKCHK[4];

        input PLRPUA0NFIQ;
        input PLRPUA0NIRQ;
        input PLRPUA1NFIQ;
        input PLRPUA1NIRQ;
        input PLRPUB0NFIQ;
        input PLRPUB0NIRQ;
        input PLRPUB1NFIQ;
        input PLRPUB1NIRQ;

        input RPUEVENTI[2];
        output RPUEVENTO[2];

        input PLLPXEXEREQ[2];
        output LPXPLEXEACK[2];

        // IRQs

        input PMCPLIRQ[8];
        input PLPSIRQ0[8];
        input PLPSIRQ1[8];
        output PSPLIRQPMC[78];
        output PSPLIRQLPD[112];
        output PSPLIRQFPD[66];

        // GPIO

        input PMCPLGPI[32];
        output PMCPLGPO[32];
        output PSPLGPIO[32];

        // Timestamp

        output PSPLTIMESTAMP[64];

        // AXI

        // ACP

        input SAXIACPCLK;

        input PLPSACPDESTID[2];

        input SAXIACPAWAKEUP;
        input SAXIACPAWAKEUPCHK;
        input SAXIACPWAKEUP;

        output SAXIACPARREADY;
        output SAXIACPARREADYCHK;
        input SAXIACPARVALID;
        input SAXIACPARVALIDCHK;
        input SAXIACPARADDR[48];
        input SAXIACPARADDRCHK[6];
        input SAXIACPARID[8];
        input SAXIACPARIDCHK;
        input SAXIACPARLEN[8];
        input SAXIACPARLENCHK;
        input SAXIACPARCACHE[4];
        input SAXIACPARDOMAIN[2];
        input SAXIACPARPROT[3];
        input SAXIACPARCTLCHK0;
        input SAXIACPARCTLCHK1;
        input SAXIACPARCTLCHK2;

        output SAXIACPAWREADY;
        output SAXIACPAWREADYCHK;
        input SAXIACPAWVALID;
        input SAXIACPAWVALIDCHK;
        input SAXIACPAWADDR[48];
        input SAXIACPAWADDRCHK[6];
        input SAXIACPAWID[8];
        input SAXIACPAWIDCHK;
        input SAXIACPAWLEN[8];
        input SAXIACPAWLENCHK;
        input SAXIACPAWCACHE[4];
        input SAXIACPAWDOMAIN[2];
        input SAXIACPAWPROT[3];
        input SAXIACPAWSNOOP[4];
        input SAXIACPAWCTLCHK0;
        input SAXIACPAWCTLCHK1;
        input SAXIACPAWCTLCHK2;
        input SAXIACPAWSTASHLPIDEN;
        input SAXIACPAWSTASHLPID[4];
        input SAXIACPAWSTASHCHK;

        input SAXIACPBREADY;
        input SAXIACPBREADYCHK;
        output SAXIACPBVALID;
        output SAXIACPBVALIDCHK;
        output SAXIACPBID[8];
        output SAXIACPBIDCHK;
        output SAXIACPBRESP[2];
        output SAXIACPBRESPCHK;

        input SAXIACPRREADY;
        input SAXIACPRREADYCHK;
        output SAXIACPRVALID;
        output SAXIACPRVALIDCHK;
        output SAXIACPRDATA[128];
        output SAXIACPRDATACHK[16];
        output SAXIACPRID[8];
        output SAXIACPRIDCHK;
        output SAXIACPRLAST;
        output SAXIACPRLASTCHK;
        output SAXIACPRRESP[2];
        output SAXIACPRRESPCHK;

        output SAXIACPWREADY;
        output SAXIACPWREADYCHK;
        input SAXIACPWVALID;
        input SAXIACPWVALIDCHK;
        input SAXIACPWDATA[128];
        input SAXIACPWDATACHK[16];
        input SAXIACPWLAST;
        input SAXIACPWLASTCHK;
        input SAXIACPWSTRB[16];
        input SAXIACPWSTRBCHK[2];

        // PL_AXI*_FPD

        for i in 0..4 {
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWAKEUP";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWAKEUPCHK";

            input "SACE5LGP{i}RCLK";

            output "SACE5LGP{i}RACOUNT"[4];
            output "SACE5LGP{i}RACOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARADDR"[49];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARADDRCHK"[7];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARIDCHK"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLEN"[8];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLENCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARBURST"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCACHE"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARDOMAIN"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLOCK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARPROT"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARQOS"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARSIZE"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARSNOOP"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK0";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK1";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK2";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARUSER"[12];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARUSERCHK"[2];

            output "SACE5LGP{i}RCOUNT"[4];
            output "SACE5LGP{i}RCOUNTCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RREADY";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RREADYCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RVALID";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RVALIDCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RDATACHK"[32];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RDATA"[256];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RID"[11];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RIDCHK"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RLAST";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RLASTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RRESP"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RRESPCHK";
            output "IFPLAFIFM{i}ACE5LITERPOISON";
            output "IFPLAFIFM{i}ACE5LITERPOISONCHK";

            input "SACE5LGP{i}WCLK";

            output "SACE5LGP{i}WACOUNT"[4];
            output "SACE5LGP{i}WACOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWADDR"[49];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWADDRCHK"[7];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWIDCHK"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLEN"[8];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLENCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWBURST"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCACHE"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWDOMAIN"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLOCK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWPROT"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWQOS"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSIZE"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSNOOP"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK0";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK1";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK2";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPIDEN";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPID"[5];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNIDEN";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWUSER"[12];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWUSERCHK"[2];

            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BREADY";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BREADYCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BVALID";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BVALIDCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BID"[11];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BIDCHK"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BRESP"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BRESPCHK";

            output "SACE5LGP{i}WCOUNT"[7];
            output "SACE5LGP{i}WCOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WDATA"[256];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WDATACHK"[32];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WLAST";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WLASTCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WSTRB"[32];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WSTRBCHK"[4];
            input "IFPLAFIFM{i}ACE5LITEWPOISON";
            input "IFPLAFIFM{i}ACE5LITEWPOISONCHK";
        }

        // PL_AXI_LPD

        input PLLPXAFIFMCOHDISABLE;

        input SAXIGP4AWAKEUPCHK;

        input SAXIGP4RCLK;

        output SAXIGP4RACOUNT[4];
        output SAXIGP4RACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4ARREADY;
        output IFPLAFIFM4AXISAXIGP4ARREADYCHK;
        input IFPLAFIFM4AXISAXIGP4ARVALID;
        input IFPLAFIFM4AXISAXIGP4ARVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4ARADDR[64];
        input IFPLAFIFM4AXISAXIGP4ARADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4ARID[16];
        input IFPLAFIFM4AXISAXIGP4ARIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4ARLEN[8];
        input IFPLAFIFM4AXISAXIGP4ARLENCHK;
        input IFPLAFIFM4AXISAXIGP4ARBURST[2];
        input IFPLAFIFM4AXISAXIGP4ARCACHE[4];
        input IFPLAFIFM4AXISAXIGP4ARLOCK;
        input IFPLAFIFM4AXISAXIGP4ARPROT[3];
        input IFPLAFIFM4AXISAXIGP4ARQOS[4];
        input IFPLAFIFM4AXISAXIGP4ARREGION[4];
        input IFPLAFIFM4AXISAXIGP4ARSIZE[3];
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4ARUSER[32];
        input IFPLAFIFM4AXISAXIGP4ARUSERCHK[4];

        output SAXIGP4RCOUNTCHK;
        output SAXIGP4RCOUNT[8];
        input IFPLAFIFM4AXISAXIGP4RREADY;
        input IFPLAFIFM4AXISAXIGP4RREADYCHK;
        output IFPLAFIFM4AXISAXIGP4RVALID;
        output IFPLAFIFM4AXISAXIGP4RVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4RDATA[256];
        output IFPLAFIFM4AXISAXIGP4RDATACHK[32];
        output IFPLAFIFM4AXISAXIGP4RID[16];
        output IFPLAFIFM4AXISAXIGP4RIDCHK[4];
        output IFPLAFIFM4AXISAXIGP4RLAST;
        output IFPLAFIFM4AXISAXIGP4RLASTCHK;
        output IFPLAFIFM4AXISAXIGP4RRESP[2];
        output IFPLAFIFM4AXISAXIGP4RRESPCHK;
        output IFPLAFIFM4AXISAXIGP4RUSER[18];
        output IFPLAFIFM4AXISAXIGP4RUSERCHK[3];

        input SAXIGP4WCLK;

        output SAXIGP4WACOUNT[4];
        output SAXIGP4WACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4AWREADY;
        output IFPLAFIFM4AXISAXIGP4AWREADYCHK;
        input IFPLAFIFM4AXISAXIGP4AWVALID;
        input IFPLAFIFM4AXISAXIGP4AWVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4AWADDR[64];
        input IFPLAFIFM4AXISAXIGP4AWADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4AWID[16];
        input IFPLAFIFM4AXISAXIGP4AWIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4AWLEN[8];
        input IFPLAFIFM4AXISAXIGP4AWLENCHK;
        input IFPLAFIFM4AXISAXIGP4AWBURST[2];
        input IFPLAFIFM4AXISAXIGP4AWCACHE[4];
        input IFPLAFIFM4AXISAXIGP4AWLOCK;
        input IFPLAFIFM4AXISAXIGP4AWPROT[3];
        input IFPLAFIFM4AXISAXIGP4AWQOS[4];
        input IFPLAFIFM4AXISAXIGP4AWREGION[4];
        input IFPLAFIFM4AXISAXIGP4AWSIZE[3];
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4AWUSER[32];
        input IFPLAFIFM4AXISAXIGP4AWUSERCHK[4];

        output SAXIGP4WCOUNTCHK;
        output SAXIGP4WCOUNT[8];
        input IFPLAFIFM4AXISAXIGP4BREADY;
        input IFPLAFIFM4AXISAXIGP4BREADYCHK;
        output IFPLAFIFM4AXISAXIGP4BVALID;
        output IFPLAFIFM4AXISAXIGP4BVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4BID[16];
        output IFPLAFIFM4AXISAXIGP4BIDCHK[2];
        output IFPLAFIFM4AXISAXIGP4BRESP[2];
        output IFPLAFIFM4AXISAXIGP4BRESPCHK[1];
        output IFPLAFIFM4AXISAXIGP4BUSER;
        output IFPLAFIFM4AXISAXIGP4BUSERCHK;

        output IFPLAFIFM4AXISAXIGP4WREADY;
        output IFPLAFIFM4AXISAXIGP4WREADYCHK;
        input IFPLAFIFM4AXISAXIGP4WVALID;
        input IFPLAFIFM4AXISAXIGP4WVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4WDATA[256];
        input IFPLAFIFM4AXISAXIGP4WDATACHK[32];
        input IFPLAFIFM4AXISAXIGP4WID[16];
        input IFPLAFIFM4AXISAXIGP4WIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4WLAST;
        input IFPLAFIFM4AXISAXIGP4WLASTCHK;
        input IFPLAFIFM4AXISAXIGP4WSTRB[32];
        input IFPLAFIFM4AXISAXIGP4WSTRBCHK[4];
        input IFPLAFIFM4AXISAXIGP4WUSER[18];
        input IFPLAFIFM4AXISAXIGP4WUSERCHK[3];

        // FPD_AXI_PL

        input MAXIGP0ACLK;

        input IFAFIFS0PLAXIMAXIGP0ARREADY;
        input IFAFIFS0PLAXIMAXIGP0ARREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0ARVALID;
        output IFAFIFS0PLAXIMAXIGP0ARVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0ARADDR[64];
        output IFAFIFS0PLAXIMAXIGP0ARADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0ARID[16];
        output IFAFIFS0PLAXIMAXIGP0ARIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0ARLEN[8];
        output IFAFIFS0PLAXIMAXIGP0ARLENCHK;
        output IFAFIFS0PLAXIMAXIGP0ARBURST[2];
        output IFAFIFS0PLAXIMAXIGP0ARCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0ARLOCK;
        output IFAFIFS0PLAXIMAXIGP0ARPROT[3];
        output IFAFIFS0PLAXIMAXIGP0ARQOS[4];
        output IFAFIFS0PLAXIMAXIGP0ARREGION[4];
        output IFAFIFS0PLAXIMAXIGP0ARSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0ARUSER[16];
        output IFAFIFS0PLAXIMAXIGP0ARUSERCHK[2];

        input IFAFIFS0PLAXIMAXIGP0AWREADY;
        input IFAFIFS0PLAXIMAXIGP0AWREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0AWVALID;
        output IFAFIFS0PLAXIMAXIGP0AWVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0AWADDR[64];
        output IFAFIFS0PLAXIMAXIGP0AWADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0AWID[16];
        output IFAFIFS0PLAXIMAXIGP0AWIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0AWLEN[8];
        output IFAFIFS0PLAXIMAXIGP0AWLENCHK;
        output IFAFIFS0PLAXIMAXIGP0AWBURST[2];
        output IFAFIFS0PLAXIMAXIGP0AWCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0AWLOCK;
        output IFAFIFS0PLAXIMAXIGP0AWPROT[3];
        output IFAFIFS0PLAXIMAXIGP0AWQOS[4];
        output IFAFIFS0PLAXIMAXIGP0AWREGION[4];
        output IFAFIFS0PLAXIMAXIGP0AWSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0AWUSER[16];
        output IFAFIFS0PLAXIMAXIGP0AWUSERCHK[2];

        output IFAFIFS0PLAXIMAXIGP0BREADY;
        output IFAFIFS0PLAXIMAXIGP0BREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0BVALID;
        input IFAFIFS0PLAXIMAXIGP0BVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0BID[16];
        input IFAFIFS0PLAXIMAXIGP0BIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0BRESP[2];
        input IFAFIFS0PLAXIMAXIGP0BRESPCHK[1];
        input IFAFIFS0PLAXIMAXIGP0BUSER;
        input IFAFIFS0PLAXIMAXIGP0BUSERCHK;

        output IFAFIFS0PLAXIMAXIGP0RREADY;
        output IFAFIFS0PLAXIMAXIGP0RREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0RVALID;
        input IFAFIFS0PLAXIMAXIGP0RVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0RDATA[128];
        input IFAFIFS0PLAXIMAXIGP0RDATACHK[16];
        input IFAFIFS0PLAXIMAXIGP0RID[16];
        input IFAFIFS0PLAXIMAXIGP0RIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0RLAST;
        input IFAFIFS0PLAXIMAXIGP0RLASTCHK;
        input IFAFIFS0PLAXIMAXIGP0RRESP[2];
        input IFAFIFS0PLAXIMAXIGP0RRESPCHK;
        input IFAFIFS0PLAXIMAXIGP0RUSER[18];
        input IFAFIFS0PLAXIMAXIGP0RUSERCHK[3];

        input IFAFIFS0PLAXIMAXIGP0WREADY;
        input IFAFIFS0PLAXIMAXIGP0WREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0WVALID;
        output IFAFIFS0PLAXIMAXIGP0WVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0WDATA[128];
        output IFAFIFS0PLAXIMAXIGP0WDATACHK[16];
        output IFAFIFS0PLAXIMAXIGP0WID[16];
        output IFAFIFS0PLAXIMAXIGP0WIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WLAST;
        output IFAFIFS0PLAXIMAXIGP0WLASTCHK;
        output IFAFIFS0PLAXIMAXIGP0WSTRB[16];
        output IFAFIFS0PLAXIMAXIGP0WSTRBCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WUSER[18];
        output IFAFIFS0PLAXIMAXIGP0WUSERCHK[3];

        // LPD_AXI_PL

        input MAXIGP2ACLK;

        input IFAXIPLPORT2MAXIGP2ARREADY;
        input IFAXIPLPORT2MAXIGP2ARREADYCHK;
        output IFAXIPLPORT2MAXIGP2ARVALID;
        output IFAXIPLPORT2MAXIGP2ARVALIDCHK;
        output IFAXIPLPORT2MAXIGP2ARADDR[64];
        output IFAXIPLPORT2MAXIGP2ARADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2ARID[16];
        output IFAXIPLPORT2MAXIGP2ARIDCHK[2];
        output IFAXIPLPORT2MAXIGP2ARLEN[8];
        output IFAXIPLPORT2MAXIGP2ARLENCHK;
        output IFAXIPLPORT2MAXIGP2ARBURST[2];
        output IFAXIPLPORT2MAXIGP2ARCACHE[4];
        output IFAXIPLPORT2MAXIGP2ARLOCK;
        output IFAXIPLPORT2MAXIGP2ARPROT[3];
        output IFAXIPLPORT2MAXIGP2ARQOS[4];
        output IFAXIPLPORT2MAXIGP2ARREGION[4];
        output IFAXIPLPORT2MAXIGP2ARSIZE[3];
        output IFAXIPLPORT2MAXIGP2ARCTLCHK1;
        output IFAXIPLPORT2MAXIGP2ARCTLCHK2;
        output IFAXIPLPORT2MAXIGP2ARUSER[16];
        output IFAXIPLPORT2MAXIGP2ARUSERCHK[2];

        input IFAXIPLPORT2MAXIGP2AWREADY;
        input IFAXIPLPORT2MAXIGP2AWREADYCHK;
        output IFAXIPLPORT2MAXIGP2AWVALID;
        output IFAXIPLPORT2MAXIGP2AWVALIDCHK;
        output IFAXIPLPORT2MAXIGP2AWID[16];
        output IFAXIPLPORT2MAXIGP2AWIDCHK[2];
        output IFAXIPLPORT2MAXIGP2AWLEN[8];
        output IFAXIPLPORT2MAXIGP2AWLENCHK;
        output IFAXIPLPORT2MAXIGP2AWADDR[64];
        output IFAXIPLPORT2MAXIGP2AWADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2AWBURST[2];
        output IFAXIPLPORT2MAXIGP2AWCACHE[4];
        output IFAXIPLPORT2MAXIGP2AWLOCK;
        output IFAXIPLPORT2MAXIGP2AWPROT[3];
        output IFAXIPLPORT2MAXIGP2AWQOS[4];
        output IFAXIPLPORT2MAXIGP2AWREGION[4];
        output IFAXIPLPORT2MAXIGP2AWSIZE[3];
        output IFAXIPLPORT2MAXIGP2AWCTLCHK1;
        output IFAXIPLPORT2MAXIGP2AWCTLCHK2;
        output IFAXIPLPORT2MAXIGP2AWUSER[16];
        output IFAXIPLPORT2MAXIGP2AWUSERCHK[2];

        output IFAXIPLPORT2MAXIGP2BREADY;
        output IFAXIPLPORT2MAXIGP2BREADYCHK;
        input IFAXIPLPORT2MAXIGP2BVALID;
        input IFAXIPLPORT2MAXIGP2BVALIDCHK;
        input IFAXIPLPORT2MAXIGP2BID[16];
        input IFAXIPLPORT2MAXIGP2BIDCHK[2];
        input IFAXIPLPORT2MAXIGP2BRESP[2];
        input IFAXIPLPORT2MAXIGP2BRESPCHK[1];
        input IFAXIPLPORT2MAXIGP2BUSER;
        input IFAXIPLPORT2MAXIGP2BUSERCHK;

        output IFAXIPLPORT2MAXIGP2RREADY;
        output IFAXIPLPORT2MAXIGP2RREADYCHK;
        input IFAXIPLPORT2MAXIGP2RVALID;
        input IFAXIPLPORT2MAXIGP2RVALIDCHK;
        input IFAXIPLPORT2MAXIGP2RDATA[128];
        input IFAXIPLPORT2MAXIGP2RDATACHK[16];
        input IFAXIPLPORT2MAXIGP2RID[16];
        input IFAXIPLPORT2MAXIGP2RIDCHK[2];
        input IFAXIPLPORT2MAXIGP2RLAST;
        input IFAXIPLPORT2MAXIGP2RLASTCHK;
        input IFAXIPLPORT2MAXIGP2RRESP[2];
        input IFAXIPLPORT2MAXIGP2RRESPCHK;
        input IFAXIPLPORT2MAXIGP2RUSER[18];
        input IFAXIPLPORT2MAXIGP2RUSERCHK[3];

        input IFAXIPLPORT2MAXIGP2WREADY;
        input IFAXIPLPORT2MAXIGP2WREADYCHK;
        output IFAXIPLPORT2MAXIGP2WVALID;
        output IFAXIPLPORT2MAXIGP2WVALIDCHK;
        output IFAXIPLPORT2MAXIGP2WDATA[128];
        output IFAXIPLPORT2MAXIGP2WDATACHK[16];
        output IFAXIPLPORT2MAXIGP2WID[16];
        output IFAXIPLPORT2MAXIGP2WIDCHK[2];
        output IFAXIPLPORT2MAXIGP2WLAST;
        output IFAXIPLPORT2MAXIGP2WLASTCHK;
        output IFAXIPLPORT2MAXIGP2WSTRB[16];
        output IFAXIPLPORT2MAXIGP2WSTRBCHK[2];
        output IFAXIPLPORT2MAXIGP2WUSER[18];
        output IFAXIPLPORT2MAXIGP2WUSERCHK[3];

        // RPU LLPP

        input PLRPULLPPAXICLK;

        input IFRPULLPPAXIARREADY;
        output IFRPULLPPAXIARVALID;
        output IFRPULLPPAXIARADDR[64];
        output IFRPULLPPAXIARBURST[2];
        output IFRPULLPPAXIARCACHE[4];
        output IFRPULLPPAXIARID[16];
        output IFRPULLPPAXIARLEN[8];
        output IFRPULLPPAXIARLOCK[1];
        output IFRPULLPPAXIARPROT[3];
        output IFRPULLPPAXIARQOS[4];
        output IFRPULLPPAXIARREGION[4];
        output IFRPULLPPAXIARSIZE[3];
        output IFRPULLPPAXIARUSER[18];

        input IFRPULLPPAXIAWREADY;
        output IFRPULLPPAXIAWVALID;
        output IFRPULLPPAXIAWADDR[64];
        output IFRPULLPPAXIAWBURST[2];
        output IFRPULLPPAXIAWCACHE[4];
        output IFRPULLPPAXIAWID[16];
        output IFRPULLPPAXIAWLEN[8];
        output IFRPULLPPAXIAWLOCK[1];
        output IFRPULLPPAXIAWPROT[3];
        output IFRPULLPPAXIAWQOS[4];
        output IFRPULLPPAXIAWREGION[4];
        output IFRPULLPPAXIAWSIZE[3];
        output IFRPULLPPAXIAWUSER[18];

        output IFRPULLPPAXIBREADY;
        input IFRPULLPPAXIBVALID;
        input IFRPULLPPAXIBID[16];
        input IFRPULLPPAXIBRESP[2];
        input IFRPULLPPAXIBUSER[1];

        output IFRPULLPPAXIRREADY;
        input IFRPULLPPAXIRVALID;
        input IFRPULLPPAXIRDATA[32];
        input IFRPULLPPAXIRID[16];
        input IFRPULLPPAXIRLAST[1];
        input IFRPULLPPAXIRRESP[2];
        input IFRPULLPPAXIRUSER[6];

        input IFRPULLPPAXIWREADY;
        output IFRPULLPPAXIWVALID;
        output IFRPULLPPAXIWDATA[32];
        output IFRPULLPPAXIWID[16];
        output IFRPULLPPAXIWLAST[1];
        output IFRPULLPPAXIWSTRB[4];
        output IFRPULLPPAXIWUSER[6];

        // CHI

        input PLPSXCHICLK;

        input IFPLPSXCHIMLINKACTIVEREQ;
        input IFPLPSXCHIMLINKACTIVEREQCHK;
        output IFPLPSXCHIMLINKACTIVEACK;
        output IFPLPSXCHIMLINKACTIVEACKCHK;
        output IFPLPSXCHISLINKACTIVEREQ;
        output IFPLPSXCHISLINKACTIVEREQCHK;
        input IFPLPSXCHISLINKACTIVEACK;
        input IFPLPSXCHISLINKACTIVEACKCHK;

        input IFPLPSXCHIMSACTIVE;
        input IFPLPSXCHIMSACTIVECHK;
        output IFPLPSXCHISSACTIVE;
        output IFPLPSXCHISSACTIVECHK;

        input IFPLPSXCHISYSCOREQ;
        input IFPLPSXCHISYSCOREQCHK;
        output IFPLPSXCHISYSCOACK;
        output IFPLPSXCHISYSCOACKCHK;

        input IFPLPSXCHIREQFLITPEND;
        input IFPLPSXCHIREQFLITPENDCHK;
        input IFPLPSXCHIREQFLITV;
        input IFPLPSXCHIREQFLITVCHK;
        input IFPLPSXCHIREQFLIT[145];
        input IFPLPSXCHIREQFLITCHK[19];
        output IFPLPSXCHIREQLCRDV;
        output IFPLPSXCHIREQLCRDVCHK;

        input IFPLPSXCHIWDATFLITPEND;
        input IFPLPSXCHIWDATFLITPENDCHK;
        input IFPLPSXCHIWDATFLITV;
        input IFPLPSXCHIWDATFLITVCHK;
        input IFPLPSXCHIWDATFLIT[653];
        input IFPLPSXCHIWDATFLITCHK[82];
        output IFPLPSXCHIWDATLCRDV;
        output IFPLPSXCHIWDATLCRDVCHK;

        output IFPLPSXCHIRDATFLITPEND;
        output IFPLPSXCHIRDATFLITPENDCHK;
        output IFPLPSXCHIRDATFLITV;
        output IFPLPSXCHIRDATFLITVCHK;
        output IFPLPSXCHIRDATFLIT[653];
        output IFPLPSXCHIRDATFLITCHK[82];
        input IFPLPSXCHIRDATLCRDV;
        input IFPLPSXCHIRDATLCRDVCHK;

        output IFPLPSXCHICRSPFLITPEND;
        output IFPLPSXCHICRSPFLITPENDCHK;
        output IFPLPSXCHICRSPFLITV;
        output IFPLPSXCHICRSPFLITVCHK;
        output IFPLPSXCHICRSPFLIT[59];
        output IFPLPSXCHICRSPFLITCHK[8];
        input IFPLPSXCHICRSPLCRDV;
        input IFPLPSXCHICRSPLCRDVCHK;

        output IFPLPSXCHISNPFLITPEND;
        output IFPLPSXCHISNPFLITPENDCHK;
        output IFPLPSXCHISNPFLITV;
        output IFPLPSXCHISNPFLITVCHK;
        output IFPLPSXCHISNPFLIT[96];
        output IFPLPSXCHISNPFLITCHK[12];
        input IFPLPSXCHISNPLCRDV;
        input IFPLPSXCHISNPLCRDVCHK;

        input IFPLPSXCHISRSPFLITPEND;
        input IFPLPSXCHISRSPFLITPENDCHK;
        input IFPLPSXCHISRSPFLITV;
        input IFPLPSXCHISRSPFLITVCHK;
        input IFPLPSXCHISRSPFLIT[59];
        input IFPLPSXCHISRSPFLITCHK[8];
        output IFPLPSXCHISRSPLCRDV;
        output IFPLPSXCHISRSPLCRDVCHK;

        // DTI

        for i in 0..2 {
            input "PLPSXDTI{i}CLK";

            input "IFPLPSDTIDN{i}TWAKEUP";
            input "IFPLPSDTIDN{i}TWAKEUPCHK";
            output "IFPLPSDTIDN{i}TREADY";
            output "IFPLPSDTIDN{i}TREADYCHK";
            input "IFPLPSDTIDN{i}TVALID";
            input "IFPLPSDTIDN{i}TVALIDCHK";
            input "IFPLPSDTIDN{i}TDATA"[160];
            input "IFPLPSDTIDN{i}TDATACHK"[160];
            input "IFPLPSDTIDN{i}TID"[5];
            input "IFPLPSDTIDN{i}TIDCHK"[5];
            input "IFPLPSDTIDN{i}TKEEP"[20];
            input "IFPLPSDTIDN{i}TKEEPCHK"[20];
            input "IFPLPSDTIDN{i}TLAST";
            input "IFPLPSDTIDN{i}TLASTCHK";

            output "IFPSPLDTIUP{i}TWAKEUP";
            output "IFPSPLDTIUP{i}TWAKEUPCHK";
            input "IFPSPLDTIUP{i}TREADY";
            input "IFPSPLDTIUP{i}TREADYCHK";
            output "IFPSPLDTIUP{i}TVALID";
            output "IFPSPLDTIUP{i}TVALIDCHK";
            output "IFPSPLDTIUP{i}TDATA"[160];
            output "IFPSPLDTIUP{i}TDATACHK"[160];
            output "IFPSPLDTIUP{i}TDEST"[5];
            output "IFPSPLDTIUP{i}TDESTCHK"[5];
            output "IFPSPLDTIUP{i}TKEEP"[20];
            output "IFPSPLDTIUP{i}TKEEPCHK"[20];
            output "IFPSPLDTIUP{i}TLAST";
            output "IFPSPLDTIUP{i}TLASTCHK";

            input "PLDTI{i}TESTSELN";
        }
        input MMTPSDTIFMUERROUT;
        input SYSPSDTIFMUERROUT;

        // ADMA

        input ADMAFCICLK[8];
        output ADMA2PLTVLD[8];
        input PL2ADMATACK[8];
        input PL2ADMACVLD[8];
        output ADMA2PLCACK[8];

        // CFG etc.

        input GCAPCLK;
        input CFUGCAPB;
        input CFUUSRGSRB;
        input CFUUSRGTSB;
        output PSPLCFGMCLK;
        output CFUEOS;

        output USRSEL[4];
        output USRDRCK[4];
        output USRRESET[4];
        output USRRUNTEST[4];
        output USRCAPTURE[4];
        output USRSHIFT[4];
        output USRUPDATE[4];
        output USRTCK[4];
        output USRTMS[4];
        output USRTDI[4];
        input USRTDO[4];

        input MJTAGTCK;
        input MJTAGTMS;
        input MJTAGTDI;
        output MJTAGTDO;

        output IFPMCCFUSEUCFUSEUCRCERROR;
        output IFPMCCFUSEUCFUSEUECCERROR;
        output IFPMCCFUSEUCFUSEUENDOFCALIB;
        output IFPMCCFUSEUCFUSEUHALTED;
        output IFPMCCFUSEUCFUSEUHEARTBEAT;

        // EMIO

        input FMIOGPIOIN[32];
        output FMIOGPIOOEN[32];
        output FMIOGPIOOUT[32];

        input EMIOGPIO2I[32];
        output EMIOGPIO2O[32];
        output EMIOGPIO2TN[32];

        output FMIOWWDT0INTERRUPT;
        output FMIOWWDT0RESET;
        output FMIOWWDT0RESETPENDING;
        output FMIOGWDT0WS0;
        output FMIOGWDT0WS1;

        output FMIOFPDWWDTINTERRUPT;
        output FMIOFPDWWDTRESET;
        output FMIOFPDWWDTRESETPENDING;
        output FMIOFPDGWDTWS0;
        output FMIOFPDGWDTWS1;

        for i in 0..4 {
            input "EMIOTTC{i}CLKI"[3];
            output "EMIOTTC{i}WAVEO"[3];
        }

        for i in 0..2 {
            input "FMIOUART{i}RXD";
            output "FMIOUART{i}TXD";
            input "FMIOUART{i}NUARTCTS";
            output "FMIOUART{i}NUARTRTS";
            input "FMIOUART{i}NUARTDSR";
            output "FMIOUART{i}NUARTDTR";
            input "FMIOUART{i}NUARTDCD";
            input "FMIOUART{i}NUARTRI";
            input "FMIOUART{i}SIRIN";
            output "FMIOUART{i}NSIROUT";
            output "FMIOUART{i}NUARTOUT1";
            output "FMIOUART{i}NUARTOUT2";
        }

        input FMIOI2CSCLIN;
        output FMIOI2CSCLOEN;
        output FMIOI2CSCLOUT;
        input FMIOI2CSDAIN;
        output FMIOI2CSDAOEN;
        output FMIOI2CSDAOUT;

        for i in 0..2 {
            input "EMIOI2C{i}SCLI";
            output "EMIOI2C{i}SCLO";
            output "EMIOI2C{i}SCLTN";
            input "EMIOI2C{i}SDAI";
            output "EMIOI2C{i}SDAO";
            output "EMIOI2C{i}SDATN";
        }

        input FMIOSYSMONI2CSCLINPUT;
        output FMIOSYSMONI2CSCLTRIB;
        input FMIOSYSMONI2CSDAINPUT;
        output FMIOSYSMONI2CSDATRIB;
        output FMIOSYSMONI2CSMBALERTTRIB;

        for i in 0..2 {
            input "EMIOSPI{i}SCLKI";
            output "EMIOSPI{i}SCLKO";
            output "EMIOSPI{i}SCLKTN";
            input "EMIOSPI{i}SSIN";
            output "EMIOSPI{i}SSON"[3];
            output "EMIOSPI{i}SSNTN";
            input "EMIOSPI{i}MI";
            output "EMIOSPI{i}MO";
            output "EMIOSPI{i}MOTN";
            input "EMIOSPI{i}SI";
            output "EMIOSPI{i}SO";
            output "EMIOSPI{i}STN";
        }
        input FMIOCHARSPITESTSELECTN;
        input FMIOCHARSPISELECTION;
        input FMIOCHARSPITESTINPUT;
        output FMIOCHARSPITESTOUTPUT;

        for i in 0..2 {
            input "EMIOCAN{i}PHYRX";
            output "EMIOCAN{i}PHYTX";
        }

        for i in 0..2 {
            output "FMIOSD{i}BUSPOWEROUT";
            output "FMIOSD{i}BUSVOLTAGEOUT"[3];
            output "FMIOSD{i}LEDCONTROLOUT";
            input "FMIOSD{i}SDIFCDNIN";
            output "FMIOSD{i}SDIFCLKOUT";
            input "FMIOSD{i}SDIFCMDIN";
            output "FMIOSD{i}SDIFCMDOE";
            output "FMIOSD{i}SDIFCMDOUT";
            input "FMIOSD{i}SDIFDATIN"[8];
            output "FMIOSD{i}SDIFDATOE"[8];
            output "FMIOSD{i}SDIFDATOUT"[8];
            input "FMIOSD{i}SDIFWPIN";
            input "FMIOSDIO{i}RXCLKIN";

            output "FMIOSD{i}DLLTESTCLK0";
            output "FMIOSD{i}DLLTESTCLKRX";
            output "FMIOSD{i}DLLTESTCLKTX";
            input "FMIOSD{i}DLLTESTIN"[4];
            output "FMIOSD{i}DLLTESTOUT"[8];
        }
        input FMIOCHARSDSELECTION;
        input FMIOCHARSDTESTINPUT;
        output FMIOCHARSDTESTOUTPUT;
        input FMIOCHARSDTESTSELECTN;

        input EMIOENETTSUCLK;
        output EMIOENET0GEMTSUTIMERCNT[94];
        input FMIOGEMTSUCLKFROMPL;
        for i in 0..2 {
            output "EMIOENET{i}DMABUSWIDTH"[2];
            output "EMIOENET{i}DMATXENDTOG";
            input "EMIOENET{i}DMATXSTATUSTOG";
            input "EMIOENET{i}EXTINTIN";
            input "EMIOENET{i}GMIICOL";
            input "EMIOENET{i}GMIICRS";
            input "EMIOENET{i}GMIIRXCLK";
            input "EMIOENET{i}GMIIRXDV";
            input "EMIOENET{i}GMIIRXD"[8];
            input "EMIOENET{i}GMIIRXER";
            input "EMIOENET{i}GMIITXCLK";
            output "EMIOENET{i}GMIITXD"[8];
            output "EMIOENET{i}GMIITXEN";
            output "EMIOENET{i}GMIITXER";
            input "EMIOENET{i}MDIOI";
            output "EMIOENET{i}MDIOMDC";
            output "EMIOENET{i}MDIOO";
            output "EMIOENET{i}MDIOTN";
            output "EMIOENET{i}RXWDATA"[8];
            output "EMIOENET{i}RXWEOP";
            output "EMIOENET{i}RXWERR";
            output "EMIOENET{i}RXWFLUSH";
            input "EMIOENET{i}RXWOVERFLOW";
            output "EMIOENET{i}RXWSOP";
            output "EMIOENET{i}RXWSTATUS"[45];
            output "EMIOENET{i}RXWWR";
            output "EMIOENET{i}SPEEDMODE"[4];
            input "EMIOENET{i}TXRCONTROL";
            input "EMIOENET{i}TXRDATARDY"[2];
            input "EMIOENET{i}TXRDATA"[8];
            input "EMIOENET{i}TXREOP";
            input "EMIOENET{i}TXRERR";
            input "EMIOENET{i}TXRFLUSHED";
            output "EMIOENET{i}TXRRD";
            input "EMIOENET{i}TXRSOP";
            output "EMIOENET{i}TXRSTATUS"[4];
            input "EMIOENET{i}TXRUNDERFLOW";
            input "EMIOENET{i}TXRVALID";

            output "EMIOGEM{i}DELAYREQRX";
            output "EMIOGEM{i}DELAYREQTX";
            output "EMIOGEM{i}PDELAYREQRX";
            output "EMIOGEM{i}PDELAYREQTX";
            output "EMIOGEM{i}PDELAYRESPRX";
            output "EMIOGEM{i}PDELAYRESPTX";
            output "EMIOGEM{i}RXSOF";
            output "EMIOGEM{i}SYNCFRAMERX";
            output "EMIOGEM{i}SYNCFRAMETX";
            input "EMIOGEM{i}TSUINCCTRL"[2];
            output "EMIOGEM{i}TSUTIMERCMPVAL";
            output "EMIOGEM{i}TXRFIXEDLAT";
            output "EMIOGEM{i}TXSOF";

            output "FMIOGEM{i}ADDMATCHVEC"[4];
            input "FMIOGEM{i}FIFORXCLKFROMPL";
            input "FMIOGEM{i}FIFOTXCLKFROMPL";
            output "FMIOGEM{i}RXDATABUFWRQ0";
            output "FMIOGEM{i}RXDATABUFWRQ1";
            output "FMIOGEM{i}RXWQUEUE"[4];
            output "FMIOGEM{i}TXRQUEUE"[4];
            output "FMIOGEM{i}TXRTIMESTAMP"[78];
        }
        input FMIOCHARGEMSELECTION;
        input FMIOCHARGEMTESTINPUT;
        output FMIOCHARGEMTESTOUTPUT;
        input FMIOCHARGEMTESTSELECTN;

        input EMIOHUBPORTOVERCRNTUSB20;
        output EMIOU2DSPORTVBUSCTRLUSB30;

        input FMIOCHARAFIFSFPDTESTINPUT;
        output FMIOCHARAFIFSFPDTESTOUTPUT;
        input FMIOCHARAFIFSFPDTESTSELECTN;
        input FMIOCHARAFIFSLPDTESTINPUT;
        output FMIOCHARAFIFSLPDTESTOUTPUT;
        input FMIOCHARAFIFSLPDTESTSELECTN;

        // SYSMONs

        output IFSYSMONROOTUSERALARM[8];
        output IFSYSMONROOTUSERDATAREADY;
        output IFSYSMONROOTUSERMUXADDR[4];
        output IFSYSMONROOTUSERPVALUE[16];
        output IFSYSMONROOTUSERTVALUE[16];
        output IFSYSMONROOTUSERVVALUE[16];
        output PMCPLSYSMONROOTALARM[10];
        output PMCPLSYSMONROOTNEWDATA[4];

        input IFSYSMONROOTTESTFABRICCLK;
        input IFSYSMONROOTTESTTESTADCINCLK;
        input IFSYSMONROOTTESTTESTADCIN[15];
        output IFSYSMONROOTTESTTESTADCOUT[16];

        input IFLPDSYSMONSATTESTFABRICCLK;
        input IFLPDSYSMONSATTESTTESTADCINCLK;
        input IFLPDSYSMONSATTESTTESTADCIN[15];
        output IFLPDSYSMONSATTESTTESTADCOUT[16];

        for i in 0..4 {
            input "IFFPDSYSMON{i}SATTESTFABRICCLK";
            input "IFFPDSYSMON{i}SATTESTTESTADCIN"[16];
            output "IFFPDSYSMON{i}SATTESTTESTADCOUT"[16];
        }

        // Tracing etc

        input FTMGPI[32];

        input STMEVENT[60];

        output PSPLTRIGACK[4];
        output PSPLTRIGGER[4];
        input PLPSTRIGACK[4];
        input PLPSTRIGGER[4];

        // ATB

        input PLPSATBCLK;

        output PSPLATREADY;
        input PLPSATVALID;
        input PLPSATBYTES[2];
        input PLPSATDATA[32];
        input PLPSATID[7];

        input PLPSAFREADY;
        output PSPLAFVALID;

        output PSPLSYNCREQ;

        // Trace out

        input PLPSTRACECLK;
        output PSPLTRACECTL;
        output PSPLTRACEDATA[32];

        // HSDP

        input PLHSDPCLK;
        output LPXPLHSDPRAWRSTN;

        input PLHSDPEGRESSTREADY;
        output PLHSDPEGRESSTVALID;
        output PLHSDPEGRESSTDATA[64];
        output PLHSDPEGRESSTKEEP[8];
        output PLHSDPEGRESSTLAST;
        output PLHSDPEGRESSTUSER[12];

        output PLHSDPINGRESSTREADY;
        input PLHSDPINGRESSTVALID;
        input PLHSDPINGRESSTDATA[64];
        input PLHSDPINGRESSTKEEP[8];
        input PLHSDPINGRESSTLAST;

        // PMU

        input AIBPMUAFIFMFPDACK;
        input AIBPMUAFIFMLPDACK;
        output PMUAIBAFIFMFPDREQ;
        output PMUAIBAFIFMLPDREQ;

        input PLPMXU2PMUOVERCURRENTUSB1;
        output PMXPLU2PMUVBUSCTRLUSB1;

        // PPU

        output PPU1PLTRIGACKIN;
        input PPU1PLTRIGACKOUT;
        input PPU1PLTRIGIN;
        output PPU1PLTRIGOUT;

        // Errors

        output PMCERRORTOPL[64];
        input PMUERRORFROMPL[4];
        output PMUERRORTOPL[47];

        // Special clock outs to XPLLs and north HNOC only

        output HSMREFCLK[2];

        // Special test-only clock outputs

        output MCLK;
        output NPICLK;
        output NTISCANCLKIN;

        // PLL test

        input PLPMCPLLTESTOUTSEL;
        output PMCPLPLLTESTOUT[16];

        input PLFPXPLLTESTOUTSEL;
        output FPDPLLTESTOUT[16];

        input PLLPDPLLTESTOUTSEL[2];
        output LPDPLLTESTOUT[16];

        // Unsorted

        output FPXCMN2PLSPARE[32];
        output FPXINTPLSPARE[16];
        input PLFPXCMN2SPARE[32];
        input PLFPXINTSPARE[16];

        input GDFABRICGL[4];

        output LPDPLDEBUG[67];
        output LPDPLDEBUGUSB1[67];

        input PLLPXSPARE[32];
        output LPXPLSPARE[32];

        input PLLPXGD0FABRIC[2];
        input PLLPXGD1FABRIC[2];

        output OSCRTCCLK;

        input PL0CLKTOLCMON;
        input PL1CLKTOLCMON;
        input PL2CLKTOLCMON;

        input PLCPILOOPBACKENN;

        input PLCSSDTRIGGER;
        input PLPMCCSSDTRIGGERN;
        input PLPMCEDTUPDATEN;
        input PLPMCFPGACLOCKSTOP[4];
        input PLPMCSCANENN;
        input PLPMCSCANRESETN;
        input PLPMCSCANSYNCBYP;

        input PLPMXWWDTCLK;
        output PMXPLWWDTINTERRUPT;

        input PLPMCCTRLTESTAAG;

        input PLPSCMN12SCANSPARE[20];
        input PLPSFPXINTSCANSPARE[20];
        output PSPLCMN12SCANSPARE[20];
        output PSPLFPXINTSCANSPARE[20];

        output PLTESTPATTERNDOUT[32];

        output PMCPLCSSDTRIGGERNR;
        output PMCPLEDTUPDATENR;
        output PMCPLSCANENNR;
        output PMCPLSCANRESETNR;

        input PLPMCNTISCANCLOCK;
        input PLPMCNTIBUSIN[16];
        input PLPMCNTITESTEN[3];
        output PMCPLNTIBUSOUT[16];

        input PMCTPPLCLK[4];
        output PMCTPPLIN[38];
        input PMCTPPLOEN[38];
        input PMCTPPLOUT[38];

        output PSPLPKIPIPESTATUS[32];
        output PSPLPKIPKBUSY[26];

        input PMVCAPTURECLKINTIP;
        output PMVCAPTUREQINTOP[5];
        input PMVDATAININTIP;
        input PMVEN1INTIP;
        input PMVFFCLKINTIP;
        output PMVFFQINTOP[5];
        input PMVIMUXININT[5];
        output PMVOUT1INTOP;
        output PMVOUT2INTOP;
        output PMVOUT3INTOP;
        output PMVOUT4INTOP;
        output PMVOUTSCLB[3];
        output PMVOUTSINT[3];
        output PMVPULSEOUTINTOP;
        input PMVRESETINTIP;
        input PMVSPAREIN1INTIP[6];
        input PMVSYNCENBINTIP;

        input SCANCLKNEXT;
        input SCANMODERSTNEXT;
        input SCANENNEXT;
        input SCANEDTUPDTNEXT;
        input SCANCLKNEXTFPD;
        input SCANENNEXTFPD;
        input SCANMODERSTNEXTFPD;
        input SCANEDTUPDTNEXTFPD;

        input SCANCHNLINAMS;
        output SCANCHNLOUTAMS;
        input SCANCNTRLCHNLINAMS;
        output SCANCNTRLCHNLOUTAMS;

        for i in 0..4 {
            input "SCANCHNLINEXTAPU{i}"[10];
            input "SCANCHNLMASKINEXTAPU{i}"[10];
            output "SCANCHNLOUTEXTAPU{i}"[10];
            input "SCANCNTRLCHNLINEXTAPU{i}";
            output "SCANCNTRLCHNLOUTEXTAPU{i}";

            for j in 0..4 {
                input "SCANCHNLINEXTAPU{i}CPU{j}"[9];
                input "SCANCHNLMASKINEXTAPU{i}CPU{j}"[9];
                output "SCANCHNLOUTEXTAPU{i}CPU{j}"[9];
                input "SCANCNTRLCHNLINEXTAPU{i}CPU{j}";
                output "SCANCNTRLCHNLOUTEXTAPU{i}CPU{j}";
            }
        }

        input SCANCHNLINEXTCML[14];
        input SCANCHNLMASKINEXTCML[14];
        output SCANCHNLOUTEXTCML[14];
        input SCANCNTRLCHNLINEXTCML;
        output SCANCNTRLCHNLOUTEXTCML;

        input SCANCHNLINEXTCMN1[14];
        input SCANCHNLINEXTCMN2[14];
        input SCANCHNLINEXTCMN3P0[14];
        input SCANCHNLINEXTCMN3P1[11];
        input SCANCHNLMASKINEXTCMN1[14];
        input SCANCHNLMASKINEXTCMN2[14];
        input SCANCHNLMASKINEXTCMN3P0[14];
        input SCANCHNLMASKINEXTCMN3P1[11];
        output SCANCHNLOUTEXTCMN1[14];
        output SCANCHNLOUTEXTCMN2[14];
        output SCANCHNLOUTEXTCMN3P0[14];
        output SCANCHNLOUTEXTCMN3P1[11];
        input SCANCNTRLCHNLINEXTCMN1;
        input SCANCNTRLCHNLINEXTCMN2;
        input SCANCNTRLCHNLINEXTCMN3P0;
        input SCANCNTRLCHNLINEXTCMN3P1;
        output SCANCNTRLCHNLOUTEXTCMN1;
        output SCANCNTRLCHNLOUTEXTCMN2;
        output SCANCNTRLCHNLOUTEXTCMN3P0;
        output SCANCNTRLCHNLOUTEXTCMN3P1;

        for i in 0..4 {
            input "SCANCHNLINEXTFPDSYSMON{i}";
            output "SCANCHNLOUTEXTFPDSYSMON{i}";
            input "SCANCNTRLCHNLINFPDSYSMON{i}";
            output "SCANCNTRLCHNLOUTFPDSYSMON{i}";
        }

        input SCANCHNLINEXTFPD[14];
        input SCANCHNLMASKINEXTFPD[14];
        output SCANCHNLOUTEXTFPD[14];
        input SCANCNTRLCHNLINEXTFPD;
        output SCANCNTRLCHNLOUTEXTFPD;

        input SCANCHNLINEXTMMU[14];
        input SCANCHNLMASKINEXTMMU[14];
        output SCANCHNLOUTEXTMMU[14];
        input SCANCNTRLCHNLINEXTMMU;
        output SCANCNTRLCHNLOUTEXTMMU;

        input SCANCHNLINEXTPKI[11];
        input SCANCHNLMASKINEXTPKI[11];
        output SCANCHNLOUTEXTPKI[11];
        input SCANCNTRLCHNLINEXTPKI;
        output SCANCNTRLCHNLOUTEXTPKI;

        input SCANCHNLINIOU[5];
        input SCANCHNLMASKINIOU[5];
        output SCANCHNLOUTIOU[5];
        input SCANCNTRLCHNLINIOU;
        output SCANCNTRLCHNLOUTIOU;
        input SCANSPAREINEXTIOU[5];
        output SCANSPAREOUTEXTIOU[5];

        input SCANCHNLINLPX[15];
        input SCANCHNLMASKINLPX[15];
        output SCANCHNLOUTLPX[15];
        input SCANCNTRLCHNLINLPX[2];
        output SCANCNTRLCHNLOUTLPX[2];
        input SCANSPAREINEXTLPX[12];
        output SCANSPAREOUTEXTLPX[12];

        input SCANCHNLINOCM[13];
        input SCANCHNLMASKINOCM[13];
        output SCANCHNLOUTOCM[13];
        input SCANCNTRLCHNLINOCM;
        output SCANCNTRLCHNLOUTOCM;
        input SCANSPAREINEXTOCM[10];
        output SCANSPAREOUTEXTOCM[10];

        input SCANCHNLINRPU[4];
        input SCANCHNLMASKINRPU[4];
        output SCANCHNLOUTRPUA[4];
        output SCANCHNLOUTRPUB[4];
        input SCANCNTRLCHNLINRPUA;
        input SCANCNTRLCHNLINRPUB;
        output SCANCNTRLCHNLOUTRPUA;
        output SCANCNTRLCHNLOUTRPUB;
        input SCANSPAREINEXTRPUA[5];
        input SCANSPAREINEXTRPUB[5];
        output SCANSPAREOUTEXTRPUA[5];
        output SCANSPAREOUTEXTRPUB[5];

        input TESTBSCANACMODE;
        input TESTBSCANACTEST;
        input TESTBSCANCLOCKDR;
        input TESTBSCANENN;
        input TESTBSCANEXTEST;
        input TESTBSCANEXTESTSMPL;
        input TESTBSCANINITMEMORY;
        input TESTBSCANMODEC;
        input TESTBSCANRESETTAPB;
        input TESTBSCANSHIFTDR;
        input TESTBSCANTDI;
        output TESTBSCANTDO;
        input TESTBSCANUPDATEDR;

        input TESTUSB0FUNCMUX0N;
        input TESTUSB0SCANMUX0N;
        input TESTUSB1FUNCMUX0N;
        input TESTUSB1SCANMUX0N;

        input TSTRTCCALIBREGIN[21];
        output TSTRTCCALIBREGOUT[21];
        input TSTRTCCALIBREGWE;
        input TSTRTCCLK;
        input TSTRTCDISABLEBATOP;
        output TSTRTCOSCCLKOUT;
        input TSTRTCOSCCNTRLIN[4];
        output TSTRTCOSCCNTRLOUT[4];
        input TSTRTCOSCCNTRLWE;
        output TSTRTCSECCOUNTEROUT[32];
        output TSTRTCSECONDSRAWINT;
        input TSTRTCSECRELOAD;
        input TSTRTCTESTCLOCKSELECTN;
        input TSTRTCTESTMODEN;
        output TSTRTCTICKCOUNTEROUT[16];
        input TSTRTCTIMESETREGIN[32];
        output TSTRTCTIMESETREGOUT[32];
        input TSTRTCTIMESETREGWE;
    }

    bel_class CPM5N {
        input PLCPMGPI[32];
        output CPMPLGPO[32];

        input PLCPMREFCLK;
        input PLCPMBOTUSERCLK;
        input PLCPMTOPUSERCLK;
        input PLCPMIRQ0;
        input PLCPMIRQ1;
        input PLCPMLOOPBACKEN;
        input PLCPMPCIE0CXSUSERCLK;
        input PLCPMPCIE2CXSUSERCLK;

        output CPMRCLK0[12];
        output CPMRCLK1[12];

        input IFCPM5NPLINDFXSCWCONCATSCWIN[652];
        input IFCPM5NPLMPIOBOTIN[5593];
        output IFCPM5NPLMPIOBOTOUT[4059];
        input IFCPM5NPLMPIOTOPIN[5765];
        output IFCPM5NPLMPIOTOPOUT[4335];

        output IFCPMPLISRCORREVENT;
        output IFCPMPLISRMISCEVENT;
        output IFCPMPLISRUNCORREVENT;

        input IFDFXCTRLF2CSCANCLKNEXT;
        input IFDFXCTRLF2CSCANEDTUPDTNEXT;
        input IFDFXCTRLF2CSCANENNEXT;
        input IFDFXCTRLF2CSCANMODERSTNEXT;

        for i in 0..4 {
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2ADDR"[10];
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2BYTEENABLE"[4];
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2DEBUGACCESS";
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2FUNCTIONNUMBER"[16];
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2READ";
            output "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2READDATA"[32];
            output "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2READWRITEDONE";
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2WRITE";
            input "IFEXT{i}IOMUXCFGMGMT2CFGMGMT2WRITEDATA"[32];
            input "IFEXT{i}IOMUXCFGMGMT2SPAREINPUTS"[8];
            output "IFEXT{i}IOMUXCFGMGMT2SPAREOUTPUTS"[2];
        }

        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACE11RCRB;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEADDR[16];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDESTREAMID[8];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDETBIT;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDEVALID;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACELDIDVALID;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACELDID[16];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEREAD;
        input IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADDATAVALID;
        input IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADDATA[32];
        input IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADPKTINPROGRESS;
        input IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADY;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACESIZE[2];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEVALID;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITE;
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEBYTEENABLE[4];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEDATA[32];
        output IFEXT0IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEPKTINPROGRESS;
        input IFEXT0IOMUXCXLREGMEMSPACESPAREINPUTS[8];
        output IFEXT0IOMUXCXLREGMEMSPACESPAREOUTPUTS[8];

        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACE11RCRB;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEADDR[16];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDESTREAMID[8];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDETBIT;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEIDEVALID;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACELDIDVALID;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACELDID[16];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEREAD;
        input IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADDATAVALID;
        input IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADDATA[32];
        input IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADPKTINPROGRESS;
        input IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEREADY;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACESIZE[2];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEVALID;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITE;
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEBYTEENABLE[4];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEDATA[32];
        output IFEXT2IOMUXCXLREGMEMSPACECXLREGMEMSPACEWRITEPKTINPROGRESS;
        input IFEXT2IOMUXCXLREGMEMSPACESPAREINPUTS[8];
        output IFEXT2IOMUXCXLREGMEMSPACESPAREOUTPUTS[8];

        input IFPLCPMGTYPFTI64BOTINAPB3CLK;
        input IFPLCPMGTYPFTI64BOTINAPB3PADDR[16];
        input IFPLCPMGTYPFTI64BOTINAPB3PENABLE;
        input IFPLCPMGTYPFTI64BOTINAPB3PRESETN;
        input IFPLCPMGTYPFTI64BOTINAPB3PSEL[2];
        input IFPLCPMGTYPFTI64BOTINAPB3PWDATA[32];
        input IFPLCPMGTYPFTI64BOTINAPB3PWRITE;
        for i in 0..4 {
            input "IFPLCPMGTYPFTI64BOTINCH{i}EYESCANRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}GTTRXRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}GTTXRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}HSDPPCSRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}ILORESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}RXCDRRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}RXPMARESETMASK"[7];
            input "IFPLCPMGTYPFTI64BOTINCH{i}RXPRBSCNTRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}RXPROGDIVRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}RXRESETMODE"[2];
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXDETECTRX";
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXELECIDLE";
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXPMARESETMASK"[3];
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXPRBSFORCEERR";
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXPROGDIVRESET";
            input "IFPLCPMGTYPFTI64BOTINCH{i}TXRESETMODE"[2];
        }
        input IFPLCPMGTYPFTI64BOTINDMONFIFORESET;
        input IFPLCPMGTYPFTI64BOTINDMONITORCLK;
        input IFPLCPMGTYPFTI64BOTINGTPLLRST;
        input IFPLCPMGTYPFTI64BOTINUSRCLK;

        input IFPLCPMGTYPFTI64TOPINAPB3CLK;
        input IFPLCPMGTYPFTI64TOPINAPB3PADDR[16];
        input IFPLCPMGTYPFTI64TOPINAPB3PENABLE;
        input IFPLCPMGTYPFTI64TOPINAPB3PRESETN;
        input IFPLCPMGTYPFTI64TOPINAPB3PSEL[2];
        input IFPLCPMGTYPFTI64TOPINAPB3PWDATA[32];
        input IFPLCPMGTYPFTI64TOPINAPB3PWRITE;
        for i in 0..4 {
            input "IFPLCPMGTYPFTI64TOPINCH{i}EYESCANRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}GTTRXRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}GTTXRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}HSDPPCSRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}ILORESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}RXCDRRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}RXPMARESETMASK"[7];
            input "IFPLCPMGTYPFTI64TOPINCH{i}RXPRBSCNTRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}RXPROGDIVRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}RXRESETMODE"[2];
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXDETECTRX";
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXELECIDLE";
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXPMARESETMASK"[3];
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXPRBSFORCEERR";
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXPROGDIVRESET";
            input "IFPLCPMGTYPFTI64TOPINCH{i}TXRESETMODE"[2];
        }
        input IFPLCPMGTYPFTI64TOPINDMONFIFORESET;
        input IFPLCPMGTYPFTI64TOPINDMONITORCLK;
        input IFPLCPMGTYPFTI64TOPINGTPLLRST;
        input IFPLCPMGTYPFTI64TOPINUSRCLK;

        input IFPLCPMSYSMONTESTFABRICCLK;
        input IFPLCPMSYSMONTESTTESTADCINCLK;
        input IFPLCPMSYSMONTESTTESTADCIN[15];
        output IFPLCPMSYSMONTESTTESTADCOUT[16];

        output ISOCPMPLN;
        input PPSIN;
        output PPSOUT;

        for i in 0..4 {
            input "IFPLCPMGTYDFXBOTINSCANCLKNFSBQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANMODERSTNFSBQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANENNFSBQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANEDTUPDTNFSBQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANCNTRLCHNLINFSBQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANCHNLINFSBQ{i}"[4];
        }

        input IFPLCPMGTYDFXBOTINSCANCLKNGTCTRLFSBQ;
        input IFPLCPMGTYDFXBOTINSCANMODERSTNGTCTRLFSBQ;
        input IFPLCPMGTYDFXBOTINSCANENNGTCTRLFSBQ;
        input IFPLCPMGTYDFXBOTINSCANEDTUPDTNGTCTRLFSBQ;
        input IFPLCPMGTYDFXBOTINSCANCNTRLCHNLINGTCTRLFSBQ;
        input IFPLCPMGTYDFXBOTINSCANCHNLINGTCTRLFSBQ[4];

        for i in 0..4 {
            input "IFPLCPMGTYDFXBOTINSCANCLKNFSTQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANMODERSTNFSTQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANENNFSTQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANEDTUPDTNFSTQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANCNTRLCHNLINFSTQ{i}";
            input "IFPLCPMGTYDFXBOTINSCANCHNLINFSTQ{i}"[4];
        }

        input IFPLCPMGTYDFXBOTINSCANCLKNGTCTRLFSTQ;
        input IFPLCPMGTYDFXBOTINSCANMODERSTNGTCTRLFSTQ;
        input IFPLCPMGTYDFXBOTINSCANENNGTCTRLFSTQ;
        input IFPLCPMGTYDFXBOTINSCANEDTUPDTNGTCTRLFSTQ;
        input IFPLCPMGTYDFXBOTINSCANCNTRLCHNLINGTCTRLFSTQ;
        input IFPLCPMGTYDFXBOTINSCANCHNLINGTCTRLFSTQ[4];

        for i in 0..4 {
            input "IFPLCPMGTYDFXTOPINSCANCLKNFSBQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANMODERSTNFSBQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANENNFSBQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANEDTUPDTNFSBQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANCNTRLCHNLINFSBQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANCHNLINFSBQ{i}"[4];
        }

        input IFPLCPMGTYDFXTOPINSCANCLKNGTCTRLFSBQ;
        input IFPLCPMGTYDFXTOPINSCANMODERSTNGTCTRLFSBQ;
        input IFPLCPMGTYDFXTOPINSCANENNGTCTRLFSBQ;
        input IFPLCPMGTYDFXTOPINSCANEDTUPDTNGTCTRLFSBQ;
        input IFPLCPMGTYDFXTOPINSCANCNTRLCHNLINGTCTRLFSBQ;
        input IFPLCPMGTYDFXTOPINSCANCHNLINGTCTRLFSBQ[4];

        for i in 0..4 {
            input "IFPLCPMGTYDFXTOPINSCANCLKNFSTQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANMODERSTNFSTQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANENNFSTQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANEDTUPDTNFSTQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANCNTRLCHNLINFSTQ{i}";
            input "IFPLCPMGTYDFXTOPINSCANCHNLINFSTQ{i}"[4];
        }

        input IFPLCPMGTYDFXTOPINSCANCLKNGTCTRLFSTQ;
        input IFPLCPMGTYDFXTOPINSCANMODERSTNGTCTRLFSTQ;
        input IFPLCPMGTYDFXTOPINSCANENNGTCTRLFSTQ;
        input IFPLCPMGTYDFXTOPINSCANEDTUPDTNGTCTRLFSTQ;
        input IFPLCPMGTYDFXTOPINSCANCNTRLCHNLINGTCTRLFSTQ;
        input IFPLCPMGTYDFXTOPINSCANCHNLINGTCTRLFSTQ[4];
    }

    bel_class PSXC {
        // Clock outputs (BUFG_PS)

        output LPDRCLKCLK[6];
        output PMCRCLKCLK[6];
        output PSPLFPDRCLKCLK[12];

        // Clock inputs

        input PLPMCAUXREFCLK;
        input PLLPDAUXREFCLK;
        input PLFPDAUXREFCLK;

        // Reset outputs

        output PLRST0N;
        output PLRST1N;
        output PLRST2N;
        output PLRST3N;

        // CPUs

        input PLPSAPU0GICFIQ[2];
        input PLPSAPU0GICIRQ[2];
        input PLPSAPU1GICFIQ[2];
        input PLPSAPU1GICIRQ[2];
        input PLPSAPU2GICFIQ[2];
        input PLPSAPU2GICIRQ[2];
        input PLPSAPU3GICFIQ[2];
        input PLPSAPU3GICIRQ[2];

        input PLPSEVENTIREQ[4];
        input PLPSEVENTIREQCHK[4];
        output PSPLEVENTIACK[4];
        output PSPLEVENTIACKCHK[4];

        input PLRPUA0NFIQ;
        input PLRPUA0NIRQ;
        input PLRPUA1NFIQ;
        input PLRPUA1NIRQ;
        input PLRPUB0NFIQ;
        input PLRPUB0NIRQ;
        input PLRPUB1NFIQ;
        input PLRPUB1NIRQ;
        input PLRPUC0NFIQ;
        input PLRPUC0NIRQ;
        input PLRPUC1NFIQ;
        input PLRPUC1NIRQ;
        input PLRPUD0NFIQ;
        input PLRPUD0NIRQ;
        input PLRPUD1NFIQ;
        input PLRPUD1NIRQ;
        input PLRPUE0NFIQ;
        input PLRPUE0NIRQ;
        input PLRPUE1NFIQ;
        input PLRPUE1NIRQ;

        input RPUEVENTI[5];
        output RPUEVENTO[5];

        input PLLPXEXEREQ[5];
        output LPXPLEXEACK[5];

        // IRQs

        input PMCPLIRQ[8];
        input PLPSIRQ0[8];
        input PLPSIRQ1[8];
        output PSPLIRQPMC[40];
        output PSPLIRQLPD[140];
        output PSPLIRQFPD[96];

        // GPIO

        input PMCPLGPI[32];
        output PMCPLGPO[32];

        // Timestamp

        output PSPLTIMESTAMP[64];

        // AXI

        // ACP

        input SAXIACPCLK;

        input PLPSACPDESTID[2];

        input SAXIACPAWAKEUP;
        input SAXIACPAWAKEUPCHK;

        output SAXIACPARREADY;
        output SAXIACPARREADYCHK;
        input SAXIACPARVALID;
        input SAXIACPARVALIDCHK;
        input SAXIACPARADDR[48];
        input SAXIACPARADDRCHK[6];
        input SAXIACPARID[8];
        input SAXIACPARIDCHK;
        input SAXIACPARLEN[8];
        input SAXIACPARLENCHK;
        input SAXIACPARCACHE[4];
        input SAXIACPARDOMAIN[2];
        input SAXIACPARPROT[3];
        input SAXIACPARCTLCHK0;
        input SAXIACPARCTLCHK1;
        input SAXIACPARCTLCHK2;

        output SAXIACPAWREADY;
        output SAXIACPAWREADYCHK;
        input SAXIACPAWVALID;
        input SAXIACPAWVALIDCHK;
        input SAXIACPAWADDR[48];
        input SAXIACPAWADDRCHK[6];
        input SAXIACPAWID[8];
        input SAXIACPAWIDCHK;
        input SAXIACPAWLEN[8];
        input SAXIACPAWLENCHK;
        input SAXIACPAWCACHE[4];
        input SAXIACPAWDOMAIN[2];
        input SAXIACPAWPROT[3];
        input SAXIACPAWSNOOP[4];
        input SAXIACPAWCTLCHK0;
        input SAXIACPAWCTLCHK1;
        input SAXIACPAWCTLCHK2;
        input SAXIACPAWSTASHLPIDEN;
        input SAXIACPAWSTASHLPID[4];
        input SAXIACPAWSTASHCHK;

        input SAXIACPBREADY;
        input SAXIACPBREADYCHK;
        output SAXIACPBVALID;
        output SAXIACPBVALIDCHK;
        output SAXIACPBID[8];
        output SAXIACPBIDCHK;
        output SAXIACPBRESP[2];
        output SAXIACPBRESPCHK;

        input SAXIACPRREADY;
        input SAXIACPRREADYCHK;
        output SAXIACPRVALID;
        output SAXIACPRVALIDCHK;
        output SAXIACPRDATA[128];
        output SAXIACPRDATACHK[16];
        output SAXIACPRID[8];
        output SAXIACPRIDCHK;
        output SAXIACPRLAST;
        output SAXIACPRLASTCHK;
        output SAXIACPRRESP[2];
        output SAXIACPRRESPCHK;

        output SAXIACPWREADY;
        output SAXIACPWREADYCHK;
        input SAXIACPWVALID;
        input SAXIACPWVALIDCHK;
        input SAXIACPWDATA[128];
        input SAXIACPWDATACHK[16];
        input SAXIACPWLAST;
        input SAXIACPWLASTCHK;
        input SAXIACPWSTRB[16];
        input SAXIACPWSTRBCHK[2];

        // PL_AXI*_FPD

        for i in 0..4 {
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWAKEUP";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWAKEUPCHK";

            input "SACE5LGP{i}RCLK";

            output "SACE5LGP{i}RACOUNT"[4];
            output "SACE5LGP{i}RACOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARADDR"[49];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARADDRCHK"[7];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARIDCHK"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLEN"[8];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLENCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARBURST"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCACHE"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARDOMAIN"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARLOCK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARPROT"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARQOS"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARSIZE"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARSNOOP"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK0";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK1";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARCTLCHK2";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARUSER"[12];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}ARUSERCHK"[2];

            output "SACE5LGP{i}RCOUNT"[4];
            output "SACE5LGP{i}RCOUNTCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RREADY";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RREADYCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RVALID";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RVALIDCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RDATA"[256];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RDATACHK"[32];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RID"[11];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RIDCHK"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RLAST";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RLASTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RRESP"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}RRESPCHK";
            output "IFPLAFIFM{i}ACE5LITERPOISON";
            output "IFPLAFIFM{i}ACE5LITERPOISONCHK";

            input "SACE5LGP{i}WCLK";

            output "SACE5LGP{i}WACOUNT"[4];
            output "SACE5LGP{i}WACOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWADDR"[49];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWADDRCHK"[7];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWIDCHK"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLEN"[8];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLENCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWBURST"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCACHE"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWDOMAIN"[2];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWLOCK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWPROT"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWQOS"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSIZE"[3];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSNOOP"[4];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK0";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK1";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWCTLCHK2";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPIDEN";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPID"[5];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHLPIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNIDEN";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNID"[11];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWSTASHNIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWUSER"[12];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}AWUSERCHK"[2];

            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BREADY";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BREADYCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BVALID";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BVALIDCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BID"[11];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BIDCHK"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BRESP"[2];
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}BRESPCHK";

            output "SACE5LGP{i}WCOUNT"[7];
            output "SACE5LGP{i}WCOUNTCHK";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WREADY";
            output "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WREADYCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WVALID";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WVALIDCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WDATA"[256];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WDATACHK"[32];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WLAST";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WLASTCHK";
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WSTRB"[32];
            input "IFPLAFIFM{i}ACE5LITESACE5LGP{i}WSTRBCHK"[4];
            input "IFPLAFIFM{i}ACE5LITEWPOISON";
            input "IFPLAFIFM{i}ACE5LITEWPOISONCHK";
        }

        // PL_AXI_LPD

        input PLLPXAFIFMCOHDISABLE;

        input SAXIGP4AWAKEUPCHK;

        input SAXIGP4RCLK;

        output SAXIGP4RACOUNT[4];
        output SAXIGP4RACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4ARREADY;
        output IFPLAFIFM4AXISAXIGP4ARREADYCHK;
        input IFPLAFIFM4AXISAXIGP4ARVALID;
        input IFPLAFIFM4AXISAXIGP4ARVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4ARADDR[64];
        input IFPLAFIFM4AXISAXIGP4ARADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4ARID[16];
        input IFPLAFIFM4AXISAXIGP4ARIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4ARLEN[8];
        input IFPLAFIFM4AXISAXIGP4ARLENCHK;
        input IFPLAFIFM4AXISAXIGP4ARBURST[2];
        input IFPLAFIFM4AXISAXIGP4ARCACHE[4];
        input IFPLAFIFM4AXISAXIGP4ARLOCK;
        input IFPLAFIFM4AXISAXIGP4ARPROT[3];
        input IFPLAFIFM4AXISAXIGP4ARQOS[4];
        input IFPLAFIFM4AXISAXIGP4ARREGION[4];
        input IFPLAFIFM4AXISAXIGP4ARSIZE[3];
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4ARUSER[32];
        input IFPLAFIFM4AXISAXIGP4ARUSERCHK[4];

        output SAXIGP4RCOUNTCHK;
        output SAXIGP4RCOUNT[8];
        input IFPLAFIFM4AXISAXIGP4RREADY;
        input IFPLAFIFM4AXISAXIGP4RREADYCHK;
        output IFPLAFIFM4AXISAXIGP4RVALID;
        output IFPLAFIFM4AXISAXIGP4RVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4RDATA[256];
        output IFPLAFIFM4AXISAXIGP4RDATACHK[32];
        output IFPLAFIFM4AXISAXIGP4RID[16];
        output IFPLAFIFM4AXISAXIGP4RIDCHK[4];
        output IFPLAFIFM4AXISAXIGP4RLAST;
        output IFPLAFIFM4AXISAXIGP4RLASTCHK;
        output IFPLAFIFM4AXISAXIGP4RRESP[2];
        output IFPLAFIFM4AXISAXIGP4RRESPCHK;
        output IFPLAFIFM4AXISAXIGP4RUSER[18];
        output IFPLAFIFM4AXISAXIGP4RUSERCHK[3];

        input SAXIGP4WCLK;

        output SAXIGP4WACOUNT[4];
        output SAXIGP4WACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4AWREADY;
        output IFPLAFIFM4AXISAXIGP4AWREADYCHK;
        input IFPLAFIFM4AXISAXIGP4AWVALID;
        input IFPLAFIFM4AXISAXIGP4AWVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4AWADDR[64];
        input IFPLAFIFM4AXISAXIGP4AWADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4AWID[16];
        input IFPLAFIFM4AXISAXIGP4AWIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4AWLEN[8];
        input IFPLAFIFM4AXISAXIGP4AWLENCHK;
        input IFPLAFIFM4AXISAXIGP4AWBURST[2];
        input IFPLAFIFM4AXISAXIGP4AWCACHE[4];
        input IFPLAFIFM4AXISAXIGP4AWLOCK;
        input IFPLAFIFM4AXISAXIGP4AWPROT[3];
        input IFPLAFIFM4AXISAXIGP4AWQOS[4];
        input IFPLAFIFM4AXISAXIGP4AWREGION[4];
        input IFPLAFIFM4AXISAXIGP4AWSIZE[3];
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4AWUSER[32];
        input IFPLAFIFM4AXISAXIGP4AWUSERCHK[4];

        output SAXIGP4WCOUNTCHK;
        output SAXIGP4WCOUNT[8];
        input IFPLAFIFM4AXISAXIGP4BREADY;
        input IFPLAFIFM4AXISAXIGP4BREADYCHK;
        output IFPLAFIFM4AXISAXIGP4BVALID;
        output IFPLAFIFM4AXISAXIGP4BVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4BID[16];
        output IFPLAFIFM4AXISAXIGP4BIDCHK[2];
        output IFPLAFIFM4AXISAXIGP4BRESP[2];
        output IFPLAFIFM4AXISAXIGP4BRESPCHK[1];
        output IFPLAFIFM4AXISAXIGP4BUSER;
        output IFPLAFIFM4AXISAXIGP4BUSERCHK;

        output IFPLAFIFM4AXISAXIGP4WREADY;
        output IFPLAFIFM4AXISAXIGP4WREADYCHK;
        input IFPLAFIFM4AXISAXIGP4WVALID;
        input IFPLAFIFM4AXISAXIGP4WVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4WDATA[256];
        input IFPLAFIFM4AXISAXIGP4WDATACHK[32];
        input IFPLAFIFM4AXISAXIGP4WID[16];
        input IFPLAFIFM4AXISAXIGP4WIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4WLAST;
        input IFPLAFIFM4AXISAXIGP4WLASTCHK;
        input IFPLAFIFM4AXISAXIGP4WSTRB[32];
        input IFPLAFIFM4AXISAXIGP4WSTRBCHK[4];
        input IFPLAFIFM4AXISAXIGP4WUSER[18];
        input IFPLAFIFM4AXISAXIGP4WUSERCHK[3];

        // FPD_AXI_PL

        input MAXIGP0ACLK;

        input IFAFIFS0PLAXIMAXIGP0ARREADY;
        input IFAFIFS0PLAXIMAXIGP0ARREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0ARVALID;
        output IFAFIFS0PLAXIMAXIGP0ARVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0ARADDR[64];
        output IFAFIFS0PLAXIMAXIGP0ARADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0ARID[16];
        output IFAFIFS0PLAXIMAXIGP0ARIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0ARLEN[8];
        output IFAFIFS0PLAXIMAXIGP0ARLENCHK;
        output IFAFIFS0PLAXIMAXIGP0ARBURST[2];
        output IFAFIFS0PLAXIMAXIGP0ARCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0ARLOCK;
        output IFAFIFS0PLAXIMAXIGP0ARPROT[3];
        output IFAFIFS0PLAXIMAXIGP0ARQOS[4];
        output IFAFIFS0PLAXIMAXIGP0ARREGION[4];
        output IFAFIFS0PLAXIMAXIGP0ARSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0ARUSER[16];
        output IFAFIFS0PLAXIMAXIGP0ARUSERCHK[2];

        input IFAFIFS0PLAXIMAXIGP0AWREADY;
        input IFAFIFS0PLAXIMAXIGP0AWREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0AWVALID;
        output IFAFIFS0PLAXIMAXIGP0AWVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0AWADDR[64];
        output IFAFIFS0PLAXIMAXIGP0AWADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0AWID[16];
        output IFAFIFS0PLAXIMAXIGP0AWIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0AWLEN[8];
        output IFAFIFS0PLAXIMAXIGP0AWLENCHK;
        output IFAFIFS0PLAXIMAXIGP0AWBURST[2];
        output IFAFIFS0PLAXIMAXIGP0AWCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0AWLOCK;
        output IFAFIFS0PLAXIMAXIGP0AWPROT[3];
        output IFAFIFS0PLAXIMAXIGP0AWQOS[4];
        output IFAFIFS0PLAXIMAXIGP0AWREGION[4];
        output IFAFIFS0PLAXIMAXIGP0AWSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0AWUSER[16];
        output IFAFIFS0PLAXIMAXIGP0AWUSERCHK[2];

        output IFAFIFS0PLAXIMAXIGP0BREADY;
        output IFAFIFS0PLAXIMAXIGP0BREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0BVALID;
        input IFAFIFS0PLAXIMAXIGP0BVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0BID[16];
        input IFAFIFS0PLAXIMAXIGP0BIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0BRESP[2];
        input IFAFIFS0PLAXIMAXIGP0BRESPCHK[1];
        input IFAFIFS0PLAXIMAXIGP0BUSER;
        input IFAFIFS0PLAXIMAXIGP0BUSERCHK;

        output IFAFIFS0PLAXIMAXIGP0RREADY;
        output IFAFIFS0PLAXIMAXIGP0RREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0RVALID;
        input IFAFIFS0PLAXIMAXIGP0RVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0RDATA[128];
        input IFAFIFS0PLAXIMAXIGP0RDATACHK[16];
        input IFAFIFS0PLAXIMAXIGP0RID[16];
        input IFAFIFS0PLAXIMAXIGP0RIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0RLAST;
        input IFAFIFS0PLAXIMAXIGP0RLASTCHK;
        input IFAFIFS0PLAXIMAXIGP0RRESP[2];
        input IFAFIFS0PLAXIMAXIGP0RRESPCHK;
        input IFAFIFS0PLAXIMAXIGP0RUSER[18];
        input IFAFIFS0PLAXIMAXIGP0RUSERCHK[3];

        input IFAFIFS0PLAXIMAXIGP0WREADY;
        input IFAFIFS0PLAXIMAXIGP0WREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0WVALID;
        output IFAFIFS0PLAXIMAXIGP0WVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0WDATA[128];
        output IFAFIFS0PLAXIMAXIGP0WDATACHK[16];
        output IFAFIFS0PLAXIMAXIGP0WID[16];
        output IFAFIFS0PLAXIMAXIGP0WIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WLAST;
        output IFAFIFS0PLAXIMAXIGP0WLASTCHK;
        output IFAFIFS0PLAXIMAXIGP0WSTRB[16];
        output IFAFIFS0PLAXIMAXIGP0WSTRBCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WUSER[18];
        output IFAFIFS0PLAXIMAXIGP0WUSERCHK[3];

        // LPD_AXI_PL

        input MAXIGP2ACLK;

        input IFAXIPLPORT2MAXIGP2ARREADY;
        input IFAXIPLPORT2MAXIGP2ARREADYCHK;
        output IFAXIPLPORT2MAXIGP2ARVALID;
        output IFAXIPLPORT2MAXIGP2ARVALIDCHK;
        output IFAXIPLPORT2MAXIGP2ARADDR[64];
        output IFAXIPLPORT2MAXIGP2ARADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2ARID[16];
        output IFAXIPLPORT2MAXIGP2ARIDCHK[2];
        output IFAXIPLPORT2MAXIGP2ARLEN[8];
        output IFAXIPLPORT2MAXIGP2ARLENCHK;
        output IFAXIPLPORT2MAXIGP2ARBURST[2];
        output IFAXIPLPORT2MAXIGP2ARCACHE[4];
        output IFAXIPLPORT2MAXIGP2ARLOCK;
        output IFAXIPLPORT2MAXIGP2ARPROT[3];
        output IFAXIPLPORT2MAXIGP2ARQOS[4];
        output IFAXIPLPORT2MAXIGP2ARREGION[4];
        output IFAXIPLPORT2MAXIGP2ARSIZE[3];
        output IFAXIPLPORT2MAXIGP2ARCTLCHK1;
        output IFAXIPLPORT2MAXIGP2ARCTLCHK2;
        output IFAXIPLPORT2MAXIGP2ARUSER[16];
        output IFAXIPLPORT2MAXIGP2ARUSERCHK[2];

        input IFAXIPLPORT2MAXIGP2AWREADY;
        input IFAXIPLPORT2MAXIGP2AWREADYCHK;
        output IFAXIPLPORT2MAXIGP2AWVALID;
        output IFAXIPLPORT2MAXIGP2AWVALIDCHK;
        output IFAXIPLPORT2MAXIGP2AWID[16];
        output IFAXIPLPORT2MAXIGP2AWIDCHK[2];
        output IFAXIPLPORT2MAXIGP2AWLEN[8];
        output IFAXIPLPORT2MAXIGP2AWLENCHK;
        output IFAXIPLPORT2MAXIGP2AWADDR[64];
        output IFAXIPLPORT2MAXIGP2AWADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2AWBURST[2];
        output IFAXIPLPORT2MAXIGP2AWCACHE[4];
        output IFAXIPLPORT2MAXIGP2AWLOCK;
        output IFAXIPLPORT2MAXIGP2AWPROT[3];
        output IFAXIPLPORT2MAXIGP2AWQOS[4];
        output IFAXIPLPORT2MAXIGP2AWREGION[4];
        output IFAXIPLPORT2MAXIGP2AWSIZE[3];
        output IFAXIPLPORT2MAXIGP2AWCTLCHK1;
        output IFAXIPLPORT2MAXIGP2AWCTLCHK2;
        output IFAXIPLPORT2MAXIGP2AWUSER[16];
        output IFAXIPLPORT2MAXIGP2AWUSERCHK[2];

        output IFAXIPLPORT2MAXIGP2BREADY;
        output IFAXIPLPORT2MAXIGP2BREADYCHK;
        input IFAXIPLPORT2MAXIGP2BVALID;
        input IFAXIPLPORT2MAXIGP2BVALIDCHK;
        input IFAXIPLPORT2MAXIGP2BID[16];
        input IFAXIPLPORT2MAXIGP2BIDCHK[2];
        input IFAXIPLPORT2MAXIGP2BRESP[2];
        input IFAXIPLPORT2MAXIGP2BRESPCHK[1];
        input IFAXIPLPORT2MAXIGP2BUSER;
        input IFAXIPLPORT2MAXIGP2BUSERCHK;

        output IFAXIPLPORT2MAXIGP2RREADY;
        output IFAXIPLPORT2MAXIGP2RREADYCHK;
        input IFAXIPLPORT2MAXIGP2RVALID;
        input IFAXIPLPORT2MAXIGP2RVALIDCHK;
        input IFAXIPLPORT2MAXIGP2RDATA[128];
        input IFAXIPLPORT2MAXIGP2RDATACHK[16];
        input IFAXIPLPORT2MAXIGP2RID[16];
        input IFAXIPLPORT2MAXIGP2RIDCHK[2];
        input IFAXIPLPORT2MAXIGP2RLAST;
        input IFAXIPLPORT2MAXIGP2RLASTCHK;
        input IFAXIPLPORT2MAXIGP2RRESP[2];
        input IFAXIPLPORT2MAXIGP2RRESPCHK;
        input IFAXIPLPORT2MAXIGP2RUSER[18];
        input IFAXIPLPORT2MAXIGP2RUSERCHK[3];

        input IFAXIPLPORT2MAXIGP2WREADY;
        input IFAXIPLPORT2MAXIGP2WREADYCHK;
        output IFAXIPLPORT2MAXIGP2WVALID;
        output IFAXIPLPORT2MAXIGP2WVALIDCHK;
        output IFAXIPLPORT2MAXIGP2WDATA[128];
        output IFAXIPLPORT2MAXIGP2WDATACHK[16];
        output IFAXIPLPORT2MAXIGP2WID[16];
        output IFAXIPLPORT2MAXIGP2WIDCHK[2];
        output IFAXIPLPORT2MAXIGP2WLAST;
        output IFAXIPLPORT2MAXIGP2WLASTCHK;
        output IFAXIPLPORT2MAXIGP2WSTRB[16];
        output IFAXIPLPORT2MAXIGP2WSTRBCHK[2];
        output IFAXIPLPORT2MAXIGP2WUSER[18];
        output IFAXIPLPORT2MAXIGP2WUSERCHK[3];

        // CHI

        input PLPSXCHICLK;

        input IFPLPSXCHIMLINKACTIVEREQ;
        input IFPLPSXCHIMLINKACTIVEREQCHK;
        output IFPLPSXCHIMLINKACTIVEACK;
        output IFPLPSXCHIMLINKACTIVEACKCHK;
        output IFPLPSXCHISLINKACTIVEREQ;
        output IFPLPSXCHISLINKACTIVEREQCHK;
        input IFPLPSXCHISLINKACTIVEACK;
        input IFPLPSXCHISLINKACTIVEACKCHK;

        input IFPLPSXCHIMSACTIVE;
        input IFPLPSXCHIMSACTIVECHK;
        output IFPLPSXCHISSACTIVE;
        output IFPLPSXCHISSACTIVECHK;

        input IFPLPSXCHISYSCOREQ;
        input IFPLPSXCHISYSCOREQCHK;
        output IFPLPSXCHISYSCOACK;
        output IFPLPSXCHISYSCOACKCHK;

        input IFPLPSXCHIREQFLITPEND;
        input IFPLPSXCHIREQFLITPENDCHK;
        input IFPLPSXCHIREQFLITV;
        input IFPLPSXCHIREQFLITVCHK;
        input IFPLPSXCHIREQFLIT[145];
        input IFPLPSXCHIREQFLITCHK[19];
        output IFPLPSXCHIREQLCRDV;
        output IFPLPSXCHIREQLCRDVCHK;

        input IFPLPSXCHIWDATFLITPEND;
        input IFPLPSXCHIWDATFLITPENDCHK;
        input IFPLPSXCHIWDATFLITV;
        input IFPLPSXCHIWDATFLITVCHK;
        input IFPLPSXCHIWDATFLIT[653];
        input IFPLPSXCHIWDATFLITCHK[82];
        output IFPLPSXCHIWDATLCRDV;
        output IFPLPSXCHIWDATLCRDVCHK;

        output IFPLPSXCHIRDATFLITPEND;
        output IFPLPSXCHIRDATFLITPENDCHK;
        output IFPLPSXCHIRDATFLITV;
        output IFPLPSXCHIRDATFLITVCHK;
        output IFPLPSXCHIRDATFLIT[653];
        output IFPLPSXCHIRDATFLITCHK[82];
        input IFPLPSXCHIRDATLCRDV;
        input IFPLPSXCHIRDATLCRDVCHK;

        output IFPLPSXCHICRSPFLITPEND;
        output IFPLPSXCHICRSPFLITPENDCHK;
        output IFPLPSXCHICRSPFLITV;
        output IFPLPSXCHICRSPFLITVCHK;
        output IFPLPSXCHICRSPFLIT[59];
        output IFPLPSXCHICRSPFLITCHK[8];
        input IFPLPSXCHICRSPLCRDV;
        input IFPLPSXCHICRSPLCRDVCHK;

        output IFPLPSXCHISNPFLITPEND;
        output IFPLPSXCHISNPFLITPENDCHK;
        output IFPLPSXCHISNPFLITV;
        output IFPLPSXCHISNPFLITVCHK;
        output IFPLPSXCHISNPFLIT[96];
        output IFPLPSXCHISNPFLITCHK[12];
        input IFPLPSXCHISNPLCRDV;
        input IFPLPSXCHISNPLCRDVCHK;

        input IFPLPSXCHISRSPFLITPEND;
        input IFPLPSXCHISRSPFLITPENDCHK;
        input IFPLPSXCHISRSPFLITV;
        input IFPLPSXCHISRSPFLITVCHK;
        input IFPLPSXCHISRSPFLIT[59];
        input IFPLPSXCHISRSPFLITCHK[8];
        output IFPLPSXCHISRSPLCRDV;
        output IFPLPSXCHISRSPLCRDVCHK;

        // DTI

        for i in 0..2 {
            input "PLPSXDTI{i}CLK";

            input "IFPLPSDTIDN{i}TWAKEUP";
            input "IFPLPSDTIDN{i}TWAKEUPCHK";
            output "IFPLPSDTIDN{i}TREADY";
            output "IFPLPSDTIDN{i}TREADYCHK";
            input "IFPLPSDTIDN{i}TVALID";
            input "IFPLPSDTIDN{i}TVALIDCHK";
            input "IFPLPSDTIDN{i}TDATA"[160];
            input "IFPLPSDTIDN{i}TDATACHK"[160];
            input "IFPLPSDTIDN{i}TID"[5];
            input "IFPLPSDTIDN{i}TIDCHK"[5];
            input "IFPLPSDTIDN{i}TKEEP"[20];
            input "IFPLPSDTIDN{i}TKEEPCHK"[20];
            input "IFPLPSDTIDN{i}TLAST";
            input "IFPLPSDTIDN{i}TLASTCHK";

            output "IFPSPLDTIUP{i}TWAKEUP";
            output "IFPSPLDTIUP{i}TWAKEUPCHK";
            input "IFPSPLDTIUP{i}TREADY";
            input "IFPSPLDTIUP{i}TREADYCHK";
            output "IFPSPLDTIUP{i}TVALID";
            output "IFPSPLDTIUP{i}TVALIDCHK";
            output "IFPSPLDTIUP{i}TDATA"[160];
            output "IFPSPLDTIUP{i}TDATACHK"[160];
            output "IFPSPLDTIUP{i}TDEST"[5];
            output "IFPSPLDTIUP{i}TDESTCHK"[5];
            output "IFPSPLDTIUP{i}TKEEP"[20];
            output "IFPSPLDTIUP{i}TKEEPCHK"[20];
            output "IFPSPLDTIUP{i}TLAST";
            output "IFPSPLDTIUP{i}TLASTCHK";

            input "PLDTI{i}TESTSELN";
        }
        input MMTPSDTIFMUERROUT;
        input SYSPSDTIFMUERROUT;

        // ADMA

        input ADMAFCICLK[8];
        output ADMA2PLTVLD[8];
        input PL2ADMATACK[8];
        input PL2ADMACVLD[8];
        output ADMA2PLCACK[8];

        // SDMA

        input SDMAFCICLK[8];
        output SDMA2PLTVLD[8];
        input PL2SDMATACK[8];
        input PL2SDMACVLD[8];
        output SDMA2PLCACK[8];

        // CFG etc.

        input GCAPCLK;
        input CFUGCAPB;
        input CFUUSRGSRB;
        input CFUUSRGTSB;
        output PSPLCFGMCLK;
        output CFUEOS;

        output USRSEL[4];
        output USRDRCK[4];
        output USRRESET[4];
        output USRRUNTEST[4];
        output USRCAPTURE[4];
        output USRSHIFT[4];
        output USRUPDATE[4];
        output USRTCK[4];
        output USRTMS[4];
        output USRTDI[4];
        input USRTDO[4];

        input MJTAGTCK;
        input MJTAGTMS;
        input MJTAGTDI;
        output MJTAGTDO;

        output IFPMCCFUSEUCFUSEUCRCERROR;
        output IFPMCCFUSEUCFUSEUECCERROR;
        output IFPMCCFUSEUCFUSEUENDOFCALIB;
        output IFPMCCFUSEUCFUSEUHALTED;
        output IFPMCCFUSEUCFUSEUHEARTBEAT;

        // ASU

        input PLASUAPBCLK;
        output IFASUPLCTRLAPBPENABLE;
        output IFASUPLCTRLAPBPWRITE;
        output IFASUPLCTRLAPBPPROT[3];
        output IFASUPLCTRLAPBPSEL;
        output IFASUPLCTRLAPBPADDR[32];
        output IFASUPLCTRLAPBPWDATA[32];
        output IFASUPLCTRLAPBPSTRB[4];
        input IFASUPLCTRLAPBPREADY;
        input IFASUPLCTRLAPBPRDATA[32];
        input IFASUPLCTRLAPBPSLVERR;

        input PLASUSTREAMCLK;

        input ASUPLREADYB;
        output ASUPLVALID;
        output ASUPLDATA[128];
        output ASUPLSTRB[16];
        output ASUPLKEYOUT[256];
        output ASUPLLAST;

        output PLASUREADYB;
        input PLASUVALID;
        input PLASUDATA[128];
        input PLASUSTRB[16];
        input PLASULAST;

        input PLASUIRQ0;
        input PLASUIRQ1;
        input PLASUIRQ2;
        input PLASUIRQ3;
        input PLASUNONFATALERROR0;
        input PLASUNONFATALERROR1;
        input PLASUGDFABRIC[2];

        input PLASUSCANSPAREIN[16];
        output ASUPLSCANSPAREOUT[16];

        // EMIO

        input FMIOGPIOIN[32];
        output FMIOGPIOOEN[32];
        output FMIOGPIOOUT[32];

        input EMIOGPIO1I[32];
        output EMIOGPIO1O[32];
        output EMIOGPIO1TN[32];

        input EMIOGPIO2I[32];
        output EMIOGPIO2O[32];
        output EMIOGPIO2TN[32];

        output FMIOWWDT0INTERRUPT;
        output FMIOWWDT0RESET;
        output FMIOWWDT0RESETPENDING;
        output FMIOGWDT0WS0;
        output FMIOGWDT0WS1;

        output FMIOFPDWWDTINTERRUPT;
        output FMIOFPDWWDTRESET;
        output FMIOFPDWWDTRESETPENDING;
        output FMIOFPDGWDTWS0;
        output FMIOFPDGWDTWS1;

        for i in 0..8 {
            input "EMIOTTC{i}CLKI"[3];
            output "EMIOTTC{i}WAVEO"[3];
        }

        for i in 0..2 {
            input "FMIOUART{i}RXD";
            output "FMIOUART{i}TXD";
            input "FMIOUART{i}NUARTCTS";
            output "FMIOUART{i}NUARTRTS";
            input "FMIOUART{i}NUARTDSR";
            output "FMIOUART{i}NUARTDTR";
            input "FMIOUART{i}NUARTDCD";
            input "FMIOUART{i}NUARTRI";
            input "FMIOUART{i}SIRIN";
            output "FMIOUART{i}NSIROUT";
            output "FMIOUART{i}NUARTOUT1";
            output "FMIOUART{i}NUARTOUT2";
        }

        input FMIOI2CSCLIN;
        output FMIOI2CSCLOEN;
        output FMIOI2CSCLOUT;
        input FMIOI2CSDAIN;
        output FMIOI2CSDAOEN;
        output FMIOI2CSDAOUT;

        for i in 0..8 {
            input "EMIOI2C{i}SCLI";
            output "EMIOI2C{i}SCLO";
            output "EMIOI2C{i}SCLTN";
            input "EMIOI2C{i}SDAI";
            output "EMIOI2C{i}SDAO";
            output "EMIOI2C{i}SDATN";
        }

        input FMIOSYSMONI2CSCLINPUT;
        output FMIOSYSMONI2CSCLTRIB;
        input FMIOSYSMONI2CSDAINPUT;
        output FMIOSYSMONI2CSDATRIB;
        output FMIOSYSMONI2CSMBALERTTRIB;

        for i in 0..2 {
            input "EMIOSPI{i}SCLKI";
            output "EMIOSPI{i}SCLKO";
            output "EMIOSPI{i}SCLKTN";
            input "EMIOSPI{i}SSIN";
            output "EMIOSPI{i}SSON"[3];
            output "EMIOSPI{i}SSNTN";
            input "EMIOSPI{i}MI";
            output "EMIOSPI{i}MO";
            output "EMIOSPI{i}MOTN";
            input "EMIOSPI{i}SI";
            output "EMIOSPI{i}SO";
            output "EMIOSPI{i}STN";
        }
        input FMIOCHARSPITESTSELECTN;
        input FMIOCHARSPISELECTION;
        input FMIOCHARSPITESTINPUT;
        output FMIOCHARSPITESTOUTPUT;

        for i in 0..4 {
            input "EMIOCAN{i}PHYRX";
            output "EMIOCAN{i}PHYTX";
        }

        for i in 0..2 {
            output "FMIOSD{i}BUSPOWEROUT";
            output "FMIOSD{i}BUSVOLTAGEOUT"[3];
            output "FMIOSD{i}LEDCONTROLOUT";
            input "FMIOSD{i}SDIFCDNIN";
            output "FMIOSD{i}SDIFCLKOUT";
            input "FMIOSD{i}SDIFCMDIN";
            output "FMIOSD{i}SDIFCMDOE";
            output "FMIOSD{i}SDIFCMDOUT";
            input "FMIOSD{i}SDIFDATIN"[8];
            output "FMIOSD{i}SDIFDATOE"[8];
            output "FMIOSD{i}SDIFDATOUT"[8];
            input "FMIOSD{i}SDIFWPIN";
            input "FMIOSDIO{i}RXCLKIN";

            output "FMIOSD{i}DLLTESTCLK0";
            output "FMIOSD{i}DLLTESTCLKRX";
            output "FMIOSD{i}DLLTESTCLKTX";
            input "FMIOSD{i}DLLTESTIN"[4];
            output "FMIOSD{i}DLLTESTOUT"[8];
        }
        input FMIOCHARSDSELECTION;
        input FMIOCHARSDTESTINPUT;
        output FMIOCHARSDTESTOUTPUT;
        input FMIOCHARSDTESTSELECTN;

        input EMIOENETTSUCLK;
        output EMIOENET0GEMTSUTIMERCNT[94];
        input FMIOGEMTSUCLKFROMPL;
        for i in 0..2 {
            output "EMIOENET{i}DMABUSWIDTH"[2];
            output "EMIOENET{i}DMATXENDTOG";
            input "EMIOENET{i}DMATXSTATUSTOG";
            input "EMIOENET{i}EXTINTIN";
            input "EMIOENET{i}GMIICOL";
            input "EMIOENET{i}GMIICRS";
            input "EMIOENET{i}GMIIRXCLK";
            input "EMIOENET{i}GMIIRXDV";
            input "EMIOENET{i}GMIIRXD"[8];
            input "EMIOENET{i}GMIIRXER";
            input "EMIOENET{i}GMIITXCLK";
            output "EMIOENET{i}GMIITXD"[8];
            output "EMIOENET{i}GMIITXEN";
            output "EMIOENET{i}GMIITXER";
            input "EMIOENET{i}MDIOI";
            output "EMIOENET{i}MDIOMDC";
            output "EMIOENET{i}MDIOO";
            output "EMIOENET{i}MDIOTN";
            output "EMIOENET{i}RXWDATA"[8];
            output "EMIOENET{i}RXWEOP";
            output "EMIOENET{i}RXWERR";
            output "EMIOENET{i}RXWFLUSH";
            input "EMIOENET{i}RXWOVERFLOW";
            output "EMIOENET{i}RXWSOP";
            output "EMIOENET{i}RXWSTATUS"[45];
            output "EMIOENET{i}RXWWR";
            output "EMIOENET{i}SPEEDMODE"[4];
            input "EMIOENET{i}TXRCONTROL";
            input "EMIOENET{i}TXRDATARDY"[2];
            input "EMIOENET{i}TXRDATA"[8];
            input "EMIOENET{i}TXREOP";
            input "EMIOENET{i}TXRERR";
            input "EMIOENET{i}TXRFLUSHED";
            output "EMIOENET{i}TXRRD";
            input "EMIOENET{i}TXRSOP";
            output "EMIOENET{i}TXRSTATUS"[4];
            input "EMIOENET{i}TXRUNDERFLOW";
            input "EMIOENET{i}TXRVALID";

            output "EMIOGEM{i}DELAYREQRX";
            output "EMIOGEM{i}DELAYREQTX";
            output "EMIOGEM{i}PDELAYREQRX";
            output "EMIOGEM{i}PDELAYREQTX";
            output "EMIOGEM{i}PDELAYRESPRX";
            output "EMIOGEM{i}PDELAYRESPTX";
            output "EMIOGEM{i}RXSOF";
            output "EMIOGEM{i}SYNCFRAMERX";
            output "EMIOGEM{i}SYNCFRAMETX";
            input "EMIOGEM{i}TSUINCCTRL"[2];
            output "EMIOGEM{i}TSUTIMERCMPVAL";
            output "EMIOGEM{i}TXRFIXEDLAT";
            output "EMIOGEM{i}TXSOF";

            output "FMIOGEM{i}ADDMATCHVEC"[4];
            input "FMIOGEM{i}FIFORXCLKFROMPL";
            input "FMIOGEM{i}FIFOTXCLKFROMPL";
            output "FMIOGEM{i}RXDATABUFWRQ0";
            output "FMIOGEM{i}RXDATABUFWRQ1";
            output "FMIOGEM{i}RXWQUEUE"[4];
            output "FMIOGEM{i}TXRQUEUE"[4];
            output "FMIOGEM{i}TXRTIMESTAMP"[78];
        }
        input FMIOCHARGEMSELECTION;
        input FMIOCHARGEMTESTINPUT;
        output FMIOCHARGEMTESTOUTPUT;
        input FMIOCHARGEMTESTSELECTN;

        input EMIOHUBPORTOVERCRNTUSB20;
        output EMIOU2DSPORTVBUSCTRLUSB30;

        output FMIODTBOUT[2];

        output FMIOPHYREFDIGCLK;

        input FMIOMPHYJTAGTRSTN;
        input FMIOMPHYJTAGTCK;
        input FMIOMPHYJTAGTMS;
        input FMIOMPHYJTAGTDI;
        output FMIOMPHYJTAGTDO;
        output FMIOMPHYJTAGTDOEN;

        output FMIOOCLAMONCLK;
        input FMIOOCLAMONCLKFROMPL;
        output FMIOOCLAMONCLKTOPL;
        output FMIOOCLAMONVALID;
        output FMIOOCLAMONOUT[128];

        output FMIORXTSTRTOBSERVE0[8];
        output FMIORXTSTRTOBSERVE1[8];
        output FMIOTXTSTRTOBSERVE0[8];
        output FMIOTXTSTRTOBSERVE1[8];

        input FMIOCHARAFIFSFPDTESTINPUT;
        output FMIOCHARAFIFSFPDTESTOUTPUT;
        input FMIOCHARAFIFSFPDTESTSELECTN;
        input FMIOCHARAFIFSLPDTESTINPUT;
        output FMIOCHARAFIFSLPDTESTOUTPUT;
        input FMIOCHARAFIFSLPDTESTSELECTN;

        // SYSMONs

        output IFSYSMONROOTUSERALARM[8];
        output IFSYSMONROOTUSERDATAREADY;
        output IFSYSMONROOTUSERMUXADDR[4];
        output IFSYSMONROOTUSERPVALUE[16];
        output IFSYSMONROOTUSERTVALUE[16];
        output IFSYSMONROOTUSERVVALUE[16];
        output PMCPLSYSMONROOTALARM[10];
        output PMCPLSYSMONROOTNEWDATA[4];

        input IFSYSMONROOTTESTFABRICCLK;
        input IFSYSMONROOTTESTTESTADCINCLK;
        input IFSYSMONROOTTESTTESTADCIN[15];
        output IFSYSMONROOTTESTTESTADCOUT[16];

        input IFLPDSYSMONSATTESTFABRICCLK;
        input IFLPDSYSMONSATTESTTESTADCINCLK;
        input IFLPDSYSMONSATTESTTESTADCIN[15];
        output IFLPDSYSMONSATTESTTESTADCOUT[16];

        for i in 0..4 {
            input "IFFPDSYSMON{i}SATTESTFABRICCLK";
            input "IFFPDSYSMON{i}SATTESTTESTADCIN"[16];
            output "IFFPDSYSMON{i}SATTESTTESTADCOUT"[16];
        }

        // Tracing etc

        input STMEVENT[60];

        output PSPLTRIGACK[4];
        output PSPLTRIGGER[4];
        input PLPSTRIGACK[4];
        input PLPSTRIGGER[4];

        // ATB

        input PLPSATBCLK;

        output PSPLATREADY;
        input PLPSATVALID;
        input PLPSATBYTES[2];
        input PLPSATDATA[32];
        input PLPSATID[7];

        input PLPSAFREADY;
        output PSPLAFVALID;

        output PSPLSYNCREQ;

        // Trace out

        input PLPSTRACECLK;
        output PSPLTRACECTL;
        output PSPLTRACEDATA[32];

        // HSDP

        input PLHSDPCLK;
        output LPXPLHSDPRAWRSTN;

        input PLHSDPEGRESSTREADY;
        output PLHSDPEGRESSTVALID;
        output PLHSDPEGRESSTDATA[64];
        output PLHSDPEGRESSTKEEP[8];
        output PLHSDPEGRESSTLAST;
        output PLHSDPEGRESSTUSER[12];

        output PLHSDPINGRESSTREADY;
        input PLHSDPINGRESSTVALID;
        input PLHSDPINGRESSTDATA[64];
        input PLHSDPINGRESSTKEEP[8];
        input PLHSDPINGRESSTLAST;

        // PMU

        input AIBPMUAFIFMFPDACK;
        input AIBPMUAFIFMLPDACK;
        output PMUAIBAFIFMFPDREQ;
        output PMUAIBAFIFMLPDREQ;

        input PLPMXU2PMUOVERCURRENTUSB1;
        output PMXPLU2PMUVBUSCTRLUSB1;

        // PPU

        output PPU1PLTRIGACKIN;
        input PPU1PLTRIGACKOUT;
        input PPU1PLTRIGIN;
        output PPU1PLTRIGOUT;

        // Errors

        output PMCERRORTOPL[72];
        input PMUERRORFROMPL[4];
        output PMUERRORTOPL[47];

        // Special clock outs to XPLLs and north HNOC only

        output HSMREFCLK[2];

        // Special test-only clock outputs

        output MCLK;
        output NPICLK;
        output NTISCANCLKIN;

        // MMI

        output PLMMIRCLK[12];
        input PLPSPLMMIAUXREFCLK;
        input PLPSPLMMIDCT10MODECLK;
        input PLPSPLMMIDCT10MODEI2SCLK;
        input PLPSPLMMIGEMTSUINCCTRLTOLPX[2];
        input PLPSPLMMIGEMTSUINCCTRL[2];
        input PLPSPLMMIRCLK0MMIVBG;
        input PLPSPLMMIRCLK0MMIVBGHALF;

        input IFPLPSPLMMISYSMONTESTFABRICCLK;
        input IFPLPSPLMMISYSMONTESTTESTADCIN[16];
        output IFPLPSPLMMISYSMONTESTTESTADCOUT[16];

        input IFPLPSPLMMIGTYPFTI64APB3CLK;
        input IFPLPSPLMMIGTYPFTI64APB3PRESETN;
        input IFPLPSPLMMIGTYPFTI64APB3PENABLE;
        input IFPLPSPLMMIGTYPFTI64APB3PWRITE;
        input IFPLPSPLMMIGTYPFTI64APB3PSEL[2];
        input IFPLPSPLMMIGTYPFTI64APB3PADDR[16];
        input IFPLPSPLMMIGTYPFTI64APB3PWDATA[32];
        output IFPLPSPLMMIGTYPFTI64APB3PREADY;
        output IFPLPSPLMMIGTYPFTI64APB3PRDATA[32];
        output IFPLPSPLMMIGTYPFTI64APB3PSLVERR;

        input PLPSPLMMIGTFTIUSRCLK;
        input PLPSPLMMIGTFTIDMONITORCLK;
        output PLGTMMIPSPLFTIDMONOUTCLK;
        input IFPLPSPLMMIGTYPFTI64DMONFIFORESET;
        output IFPLPSPLMMIGTYPFTI64DMONITOROUT[64];
        input IFPLPSPLMMIGTYPFTI64GTPLLRST;
        output IFPLPSPLMMIGTYPFTI64HSCLK0LCPLLLOCK;
        output IFPLPSPLMMIGTYPFTI64HSCLK0RPLLLOCK;
        output IFPLPSPLMMIGTYPFTI64HSCLK1LCPLLLOCK;
        output IFPLPSPLMMIGTYPFTI64HSCLK1RPLLLOCK;

        for i in 0..4 {
            input "IFPLPSPLMMIGTYPFTI64CH{i}EYESCANRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}GTRXRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}GTTXRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}HSDPPCSRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}ILORESET";
            output "IFPLPSPLMMIGTYPFTI64CH{i}ILORESETDONE";
            output "IFPLPSPLMMIGTYPFTI64CH{i}PHYSTATUS";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXCDRRESET";
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXPMARESETDONE";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPMARESETMASK"[7];
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPRBSCNTRESET";
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXPRBSERR";
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXPRBSLOCKED";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPROGDIVRESET";
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXPROGDIVRESETDONE";
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXRESETDONE";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXRESETMODE"[2];
            output "IFPLPSPLMMIGTYPFTI64CH{i}RXSTATUS"[3];
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXDETECTRX";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXELECIDLE";
            output "IFPLPSPLMMIGTYPFTI64CH{i}TXPMARESETDONE";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPMARESETMASK"[3];
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPRBSFORCEERR";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPROGDIVRESET";
            output "IFPLPSPLMMIGTYPFTI64CH{i}TXPROGDIVRESETDONE";
            output "IFPLPSPLMMIGTYPFTI64CH{i}TXRESETDONE";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXRESETMODE"[2];
        }

        input IFPLPSPLMMIDFXCTRLF2CSCANCLKNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANMODERSTNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANENNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANEDTUPDTNEXT;
        input IFPLPSPLMMIDFXSCWCONCATSCWIN[450];
        output IFPLPSPLMMIDFXSCWCONCATSCWOUT[202];

        input IFPLPSPLMMIGTDFXSCANCLKNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANMODERSTNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANENNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANEDTUPDTNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANCHNLINGTCTRLFS[4];
        output IFPLPSPLMMIGTDFXSCANCHNLOUTGTCTRLSF[4];
        input IFPLPSPLMMIGTDFXSCANCNTRLCHNLINGTCTRLFS;
        output IFPLPSPLMMIGTDFXSCANCNTRLCHNLOUTGTCTRLSF;

        for i in 0..4 {
            input "IFPLPSPLMMIGTDFXSCANCLKNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANMODERSTNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANENNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANEDTUPDTNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANCHNLINFS{i}"[4];
            output "IFPLPSPLMMIGTDFXSCANCHNLOUTSF{i}"[4];
            input "IFPLPSPLMMIGTDFXSCANCNTRLCHNLINFS{i}";
            output "IFPLPSPLMMIGTDFXSCANCNTRLCHNLOUTSF{i}";
        }

        // PLL test

        input PLPMCPLLTESTOUTSEL;
        output PMCPLPLLTESTOUT[16];

        input PLFPXPLLTESTOUTSEL;
        output FPDPLLTESTOUT[16];

        input PLLPDPLLTESTOUTSEL[2];
        output LPDPLLTESTOUT[16];

        // Unsorted

        output FPXCMN2PLSPARE[32];
        output FPXINTPLSPARE[16];
        input PLFPXCMN2SPARE[32];
        input PLFPXINTSPARE[16];

        input GDFABRICGL[4];

        output LPDPLDEBUG[67];
        output LPDPLDEBUGUSB1[67];

        input PLLPXSPARE[32];
        output LPXPLSPARE[32];

        input PLLPXGD0FABRIC[2];
        input PLLPXGD1FABRIC[2];

        output OSCRTCCLK;

        input PL0CLKTOLCMON;
        input PL1CLKTOLCMON;
        input PL2CLKTOLCMON;

        input PLCPILOOPBACKENN;

        input PLCSSDTRIGGER;
        input PLPMCCSSDTRIGGERN;
        input PLPMCEDTUPDATEN;
        input PLPMCFPGACLOCKSTOP[4];
        input PLPMCSCANENN;
        input PLPMCSCANRESETN;
        input PLPMCSCANSYNCBYP;

        input PLPMXWWDTCLK;
        output PMXPLWWDTINTERRUPT;

        input PLPMCCTRLTESTAAG;

        input PLPSCMN12SCANSPARE[20];
        input PLPSFPXINTSCANSPARE[20];
        output PSPLCMN12SCANSPARE[20];
        output PSPLFPXINTSCANSPARE[20];

        output PLTESTPATTERNDOUT[32];

        output PMCPLCSSDTRIGGERNR;
        output PMCPLEDTUPDATENR;
        output PMCPLSCANENNR;
        output PMCPLSCANRESETNR;

        input PLPMCNTISCANCLOCK;
        input PLPMCNTIBUSIN[16];
        input PLPMCNTITESTEN[3];
        output PMCPLNTIBUSOUT[16];

        input PMCTPPLCLK[4];
        output PMCTPPLIN[38];
        input PMCTPPLOEN[38];
        input PMCTPPLOUT[38];

        input PMVCAPTURECLKINTIP;
        output PMVCAPTUREQINTOP[5];
        input PMVDATAININTIP;
        input PMVEN1INTIP;
        input PMVFFCLKINTIP;
        output PMVFFQINTOP[5];
        input PMVIMUXININT[5];
        output PMVOUT1INTOP;
        output PMVOUT2INTOP;
        output PMVOUT3INTOP;
        output PMVOUT4INTOP;
        output PMVOUTSCLB[3];
        output PMVOUTSINT[3];
        output PMVPULSEOUTINTOP;
        input PMVRESETINTIP;
        input PMVSPAREIN1INTIP[6];
        input PMVSYNCENBINTIP;

        input SCANCLKNEXT;
        input SCANMODERSTNEXT;
        input SCANENNEXT;
        input SCANEDTUPDTNEXT;
        input SCANCLKNEXTFPD;
        input SCANENNEXTFPD;
        input SCANMODERSTNEXTFPD;
        input SCANEDTUPDTNEXTFPD;

        input SCANCHNLINAMS;
        output SCANCHNLOUTAMS;
        input SCANCNTRLCHNLINAMS;
        output SCANCNTRLCHNLOUTAMS;

        for i in 0..4 {
            input "SCANCHNLINEXTAPU{i}"[10];
            input "SCANCHNLMASKINEXTAPU{i}"[10];
            output "SCANCHNLOUTEXTAPU{i}"[10];
            input "SCANCNTRLCHNLINEXTAPU{i}";
            input "SCANCNTRLCHNLMASKINEXTAPU{i}";
            output "SCANCNTRLCHNLOUTEXTAPU{i}";

            for j in 0..2 {
                input "SCANCHNLINEXTAPU{i}CPU{j}"[10];
                input "SCANCHNLMASKINEXTAPU{i}CPU{j}"[10];
                output "SCANCHNLOUTEXTAPU{i}CPU{j}"[10];
                input "SCANCNTRLCHNLINEXTAPU{i}CPU{j}";
                input "SCANCNTRLCHNLMASKINEXTAPU{i}CPU{j}";
                output "SCANCNTRLCHNLOUTEXTAPU{i}CPU{j}";
            }
        }

        input SCANCHNLINEXTASU[10];
        input SCANCHNLMASKINEXTASU[10];
        output SCANCHNLOUTEXTASU[10];
        input SCANODCCCHNLINEXTASU;
        input SCANODCCCHNLMASKINEXTTOASU;
        output SCANODCCCHNLOUTEXTASU;

        input SCANCHNLINEXTCML[15];
        input SCANCHNLMASKINEXTCML[15];
        output SCANCHNLOUTEXTCML[15];
        input SCANCNTRLCHNLINEXTCML;
        input SCANCNTRLCHNLMASKINEXTCML;
        output SCANCNTRLCHNLOUTEXTCML;

        input SCANCHNLINEXTCMN1[15];
        input SCANCHNLINEXTCMN2[15];
        input SCANCHNLINEXTCMN3P0[15];
        input SCANCHNLINEXTCMN3P1[15];
        input SCANCHNLMASKINEXTCMN1[15];
        input SCANCHNLMASKINEXTCMN2[15];
        input SCANCHNLMASKINEXTCMN3P0[15];
        input SCANCHNLMASKINEXTCMN3P1[15];
        output SCANCHNLOUTEXTCMN1[15];
        output SCANCHNLOUTEXTCMN2[15];
        output SCANCHNLOUTEXTCMN3P0[15];
        output SCANCHNLOUTEXTCMN3P1[15];
        input SCANCNTRLCHNLINEXTCMN1;
        input SCANCNTRLCHNLINEXTCMN2;
        input SCANCNTRLCHNLINEXTCMN3P0;
        input SCANCNTRLCHNLINEXTCMN3P1;
        input SCANCNTRLCHNLMASKINEXTCMN1;
        input SCANCNTRLCHNLMASKINEXTCMN2;
        input SCANCNTRLCHNLMASKINEXTCMN3P0;
        input SCANCNTRLCHNLMASKINEXTCMN3P1;
        output SCANCNTRLCHNLOUTEXTCMN1;
        output SCANCNTRLCHNLOUTEXTCMN2;
        output SCANCNTRLCHNLOUTEXTCMN3P0;
        output SCANCNTRLCHNLOUTEXTCMN3P1;

        for i in 0..4 {
            input "SCANCHNLINEXTFPDSYSMON{i}";
            output "SCANCHNLOUTEXTFPDSYSMON{i}";
            input "SCANCNTRLCHNLINFPDSYSMON{i}";
            output "SCANCNTRLCHNLOUTFPDSYSMON{i}";
        }

        input SCANCHNLINEXTFPD[15];
        input SCANCHNLMASKINEXTFPD[15];
        output SCANCHNLOUTEXTFPD[15];
        input SCANCNTRLCHNLINEXTFPD;
        output SCANCNTRLCHNLOUTEXTFPD;
        input SCANODCCCHNLMASKINEXTFPD;
        input SCANODCCCHNLMASKINEXTFPD1;

        input SCANCHNLINEXTMMU[15];
        input SCANCHNLMASKINEXTMMU[15];
        output SCANCHNLOUTEXTMMU[15];
        input SCANCNTRLCHNLINEXTMMU;
        output SCANCNTRLCHNLOUTEXTMMU;
        input SCANODCCCHNLMASKINEXTMMU;

        input SCANCHNLINEXTMMU0[4];
        input SCANCHNLMASKINEXTMMU0[4];
        output SCANCHNLOUTEXTMMU0[4];
        input SCANODCCCHNLINEXTMMU0;
        output SCANODCCCHNLOUTEXTMMU0;

        input SCANCHNLINEXTPKI[11];
        input SCANCHNLMASKINEXTPKI[11];
        input SCANCNTRLCHNLINEXTPKI;

        input SCANCHNLINIOU[5];
        input SCANCHNLMASKINIOU[5];
        output SCANCHNLOUTIOU[5];
        input SCANSPAREINEXTIOU[5];
        output SCANSPAREOUTEXTIOU[5];
        input SCANODCCCHNLINIOU;
        output SCANODCCCHNLOUTIOU;
        input SCANODCCCHNLMASKINEXTTOIOU;

        input SCANCHNLINLPX[15];
        input SCANCHNLMASKINLPX[15];
        output SCANCHNLOUTLPX[15];
        input SCANCNTRLCHNLINLPX;
        output SCANCNTRLCHNLOUTLPX;
        input SCANSPAREINEXTLPX[12];
        output SCANSPAREOUTEXTLPX[12];
        input SCANODCCCHNLMASKINEXTTOLPX;

        input SCANCHNLINOCM[15];
        input SCANCHNLMASKINOCM[15];
        output SCANCHNLOUTOCM[15];
        input SCANCNTRLCHNLINOCM;
        output SCANCNTRLCHNLOUTOCM;
        input SCANSPAREINEXTOCM[10];
        output SCANSPAREOUTEXTOCM[10];
        input SCANODCCCHNLMASKINEXTTOOCM;

        input SCANCHNLINRPU[6];
        input SCANCHNLMASKINRPU[6];
        output SCANCHNLOUTRPUA[6];
        output SCANCHNLOUTRPUB[6];
        output SCANCHNLOUTRPUC[6];
        output SCANCHNLOUTRPUD[6];
        output SCANCHNLOUTRPUE[6];
        input SCANCNTRLCHNLINRPUA;
        input SCANCNTRLCHNLINRPUB;
        input SCANCNTRLCHNLINRPUC;
        input SCANCNTRLCHNLINRPUD;
        input SCANCNTRLCHNLINRPUE;
        output SCANCNTRLCHNLOUTRPUA;
        output SCANCNTRLCHNLOUTRPUB;
        output SCANCNTRLCHNLOUTRPUC;
        output SCANCNTRLCHNLOUTRPUD;
        output SCANCNTRLCHNLOUTRPUE;
        input SCANODCCCHNLMASKINEXTTORPUA;
        input SCANODCCCHNLMASKINEXTTORPUB;
        input SCANODCCCHNLMASKINEXTTORPUC;
        input SCANODCCCHNLMASKINEXTTORPUD;
        input SCANODCCCHNLMASKINEXTTORPUE;
        input SCANSPAREINEXTRPUA[5];
        input SCANSPAREINEXTRPUB[5];
        input SCANSPAREINEXTRPUC[5];
        input SCANSPAREINEXTRPUD[5];
        input SCANSPAREINEXTRPUE[5];
        output SCANSPAREOUTEXTRPUA[5];
        output SCANSPAREOUTEXTRPUB[5];
        output SCANSPAREOUTEXTRPUC[5];
        output SCANSPAREOUTEXTRPUD[5];
        output SCANSPAREOUTEXTRPUE[5];

        input TESTBSCANACMODE;
        input TESTBSCANACTEST;
        input TESTBSCANCLOCKDR;
        input TESTBSCANENN;
        input TESTBSCANEXTEST;
        input TESTBSCANEXTESTSMPL;
        input TESTBSCANINITMEMORY;
        input TESTBSCANMODEC;
        input TESTBSCANRESETTAPB;
        input TESTBSCANSHIFTDR;
        input TESTBSCANTDI;
        output TESTBSCANTDO;
        input TESTBSCANUPDATEDR;

        input TESTUSB0FUNCMUX0N;
        input TESTUSB0SCANMUX0N;
        input TESTUSB1FUNCMUX0N;
        input TESTUSB1SCANMUX0N;

        input TSTRTCCALIBREGIN[21];
        output TSTRTCCALIBREGOUT[21];
        input TSTRTCCALIBREGWE;
        input TSTRTCCLK;
        input TSTRTCDISABLEBATOP;
        output TSTRTCOSCCLKOUT;
        input TSTRTCOSCCNTRLIN[4];
        output TSTRTCOSCCNTRLOUT[4];
        input TSTRTCOSCCNTRLWE;
        output TSTRTCSECCOUNTEROUT[32];
        output TSTRTCSECONDSRAWINT;
        input TSTRTCSECRELOAD;
        input TSTRTCTESTCLOCKSELECTN;
        input TSTRTCTESTMODEN;
        output TSTRTCTICKCOUNTEROUT[16];
        input TSTRTCTIMESETREGIN[32];
        output TSTRTCTIMESETREGOUT[32];
        input TSTRTCTIMESETREGWE;
    }

    bel_class MMI {
        // DisplayPort

        input DPEXTERNALCUSTOMEVENT1;
        input DPEXTERNALCUSTOMEVENT2;
        input DPEXTERNALVSYNCEVENT;

        input PLMMIDC1XCLK;
        input PLMMIDC2XCLK;
        input PLMMIDCI2SS0CLK;
        input PLMMIDCI2SS1CLK;
        input PLMMIDCI2SS2CLK;
        input PLMMIDCI2SS3CLK;
        input PLMMIDCS1CLK;
        input PLMMIDCS2CLK;
        input PLMMIDCS3CLK;

        output IFMMIPLI2S0I2SLRCLKI;
        output IFMMIPLI2S0I2SDATAI[4];

        input IFPLMMII2S0I2SLRCLKI;
        input IFPLMMII2S0I2SDATAI[4];

        input IFPLMMII2S1I2SLRCLKI;
        input IFPLMMII2S1I2SDATAI[4];

        input IFPLMMII2S2I2SLRCLKI;
        input IFPLMMII2S2I2SDATAI[4];

        input IFPLMMII2S3I2SLRCLKI;
        input IFPLMMII2S3I2SDATAI[4];

        for i in 0..2 {
            output "IFMMIPLSDPFB{i}EXTSDPREQI";
            input "IFMMIPLSDPFB{i}EXTSDPACKO";
            output "IFMMIPLSDPFB{i}EXTSDPDATAI"[72];
            input "IFMMIPLSDPFB{i}EXTSDPHORIZONTALBLANKINGO";
            input "IFMMIPLSDPFB{i}EXTSDPVERTICALBLANKINGO";
            output "IFMMIPLSDPFB{i}EXTDPSDPACKI";
            output "IFMMIPLSDPFB{i}EXTDPSDPHORIZONTALBLANKINGI";
            output "IFMMIPLSDPFB{i}EXTDPSDPVERTICALBLANKINGI";
        }

        input IFPLMMISDP00EXTSDPREQI;
        output IFPLMMISDP00EXTSDPACKO;
        input IFPLMMISDP00EXTSDPDATAI[72];
        output IFPLMMISDP00EXTSDPLINECNTMAT[2];
        output IFPLMMISDP00EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP00EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP01EXTSDPREQI;
        output IFPLMMISDP01EXTSDPACKO;
        input IFPLMMISDP01EXTSDPDATAI[72];
        output IFPLMMISDP01EXTSDPLINECNTMAT[2];
        output IFPLMMISDP01EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP01EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP1EXTSDPREQI;
        output IFPLMMISDP1EXTSDPACKO;
        input IFPLMMISDP1EXTSDPDATAI[72];
        output IFPLMMISDP1EXTSDPLINECNTMAT[2];
        output IFPLMMISDP1EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP1EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP2EXTSDPREQI;
        output IFPLMMISDP2EXTSDPACKO;
        input IFPLMMISDP2EXTSDPDATAI[72];
        output IFPLMMISDP2EXTSDPLINECNTMAT[2];
        output IFPLMMISDP2EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP2EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP3EXTSDPREQI;
        output IFPLMMISDP3EXTSDPACKO;
        input IFPLMMISDP3EXTSDPDATAI[72];
        output IFPLMMISDP3EXTSDPLINECNTMAT[2];
        output IFPLMMISDP3EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP3EXTSDPVERTICALBLANKINGO;

        for i in 0..2 {
            output "IFMMIPLVIDEOS0P{i}HSYNCI";
            output "IFMMIPLVIDEOS0P{i}VSYNCI";
            output "IFMMIPLVIDEOS0P{i}DATAENI";
            output "IFMMIPLVIDEOS0P{i}VDATAALPHAI"[8];
            output "IFMMIPLVIDEOS0P{i}VDATARGBI"[36];
        }
        for i in 0..4 {
            input "IFPLMMIVIDEOS0P{i}DATAENI";
            input "IFPLMMIVIDEOS0P{i}HSYNCI";
            input "IFPLMMIVIDEOS0P{i}VSYNCI";
            input "IFPLMMIVIDEOS0P{i}VDATAALPHAI"[8];
            input "IFPLMMIVIDEOS0P{i}VDATARGBI"[36];
        }
        for i in 1..4 {
            for j in 0..2 {
                input "IFPLMMIVIDEOS{i}P{j}DATAENI";
                input "IFPLMMIVIDEOS{i}P{j}HSYNCI";
                input "IFPLMMIVIDEOS{i}P{j}VSYNCI";
                input "IFPLMMIVIDEOS{i}P{j}VDATAALPHAI"[8];
                input "IFPLMMIVIDEOS{i}P{j}VDATARGBI"[36];
            }
        }

        // Ethernet

        input PLGEMRXCLK;
        input PLGEMTXCLK;
        output GEMPLRXCLK;
        output GEMPLTXCLK;
        input GEMEXTINTERRUPTIN;

        output IFPLMMIGEM1588PTPDELAYREQRX;
        output IFPLMMIGEM1588PTPDELAYREQTX;
        output IFPLMMIGEM1588PTPPDELAYREQRX;
        output IFPLMMIGEM1588PTPPDELAYREQTX;
        output IFPLMMIGEM1588PTPPDELAYRESPRX;
        output IFPLMMIGEM1588PTPPDELAYRESPTX;
        output IFPLMMIGEM1588PTPSOFRX;
        output IFPLMMIGEM1588PTPSOFTX;
        output IFPLMMIGEM1588PTPSYNCFRAMERX;
        output IFPLMMIGEM1588PTPSYNCFRAMETX;

        output IFPLMMIGEMEXTFIFOADDMATCHVEC[4];
        output IFPLMMIGEMEXTFIFODMATXENDTOG;
        input IFPLMMIGEMEXTFIFODMATXSTATUSTOG;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ0;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ1;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ2;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ3;
        output IFPLMMIGEMEXTFIFORXWDATA[64];
        output IFPLMMIGEMEXTFIFORXWEOP;
        output IFPLMMIGEMEXTFIFORXWERR;
        output IFPLMMIGEMEXTFIFORXWFLUSH;
        output IFPLMMIGEMEXTFIFORXWMOD[4];
        input IFPLMMIGEMEXTFIFORXWOVERFLOW;
        output IFPLMMIGEMEXTFIFORXWQUEUE[4];
        output IFPLMMIGEMEXTFIFORXWSOP;
        output IFPLMMIGEMEXTFIFORXWSTATUS[45];
        output IFPLMMIGEMEXTFIFORXWTIMESTAMP[78];
        output IFPLMMIGEMEXTFIFORXWWR;
        input IFPLMMIGEMEXTFIFOTXRCONTROL;
        input IFPLMMIGEMEXTFIFOTXRDATARDY[4];
        input IFPLMMIGEMEXTFIFOTXRDATA[64];
        input IFPLMMIGEMEXTFIFOTXREOP;
        input IFPLMMIGEMEXTFIFOTXREOPEARLY;
        input IFPLMMIGEMEXTFIFOTXRERR;
        input IFPLMMIGEMEXTFIFOTXRFLUSHED;
        input IFPLMMIGEMEXTFIFOTXRFRAMESIZEVLD[4];
        input IFPLMMIGEMEXTFIFOTXRFRAMESIZE[56];
        input IFPLMMIGEMEXTFIFOTXRMOD[4];
        output IFPLMMIGEMEXTFIFOTXRQUEUE[4];
        output IFPLMMIGEMEXTFIFOTXRRD[4];
        input IFPLMMIGEMEXTFIFOTXRSOP;
        output IFPLMMIGEMEXTFIFOTXRSTATUS[4];
        output IFPLMMIGEMEXTFIFOTXRTIMESTAMP[78];
        input IFPLMMIGEMEXTFIFOTXRUNDERFLOW;
        input IFPLMMIGEMEXTFIFOTXRVALID;

        input MMIPLMMITSUCLK;
        input MMIPLMMITSUINCCTRL[2];
        output MMIMMIPLTSUCLK;
        output MMIMMIPLTSUTIMERCMPVAL;
        output MMIMMIPLTSUTIMERCNT[94];

        // Misc

        output IFMMIPLISRCORREVENT;
        output IFMMIPLISRMISCEVENT;
        output IFMMIPLISRUNCORREVENT;

        output ISOMMIPLN;

        input PLMMIGPI[16];
        output MMIPLGPO[32];
        input PLMMIIRQ0;
        input PLMMIIRQ1;

        input MMIPLMMIPTMCLK;
        input MMIPLMMIPTMLOAD;

        input PLMMIDEBUGCLK;
        output MMIPLDEBUGCLK;
        output MMIPLDEBUGO[260];
        input PLMMISPARE[32];
        output MMIPLSPARE[32];

        input PLMMILOOPBACKEN;
    }

    bel_class PSXT {
        // Clock outputs (BUFG_PS)

        output LPDRCLKCLK[6];
        output PMCRCLKCLK[6];
        output PSPLFPDRCLKCLK[12];

        // Clock inputs

        input PLPMCAUXREFCLK;
        input PLLPDAUXREFCLK;
        input PLFPDAUXREFCLK;

        // Reset outputs

        output PLRST0N;
        output PLRST1N;
        output PLRST2N;
        output PLRST3N;

        // CPUs

        input PLPSAPU0GICFIQ[4];
        input PLPSAPU0GICIRQ[4];

        input EVENTIPLFPX[4];
        input EVENTICHKPLFPX[4];
        output EVENTOFPXPL[4];
        output EVENTOCHKFPXPL[4];

        input PLRPUA0NFIQ;
        input PLRPUA0NIRQ;
        input PLRPUA1NFIQ;
        input PLRPUA1NIRQ;
        input PLRPUB0NFIQ;
        input PLRPUB0NIRQ;
        input PLRPUB1NFIQ;
        input PLRPUB1NIRQ;
        input PLRPUC0NFIQ;
        input PLRPUC0NIRQ;
        input PLRPUC1NFIQ;
        input PLRPUC1NIRQ;

        input RPUEVENTI[5];
        output RPUEVENTO[5];

        input PLLPXEXEREQ[5];
        output LPXPLEXEACK[5];

        // IRQs

        input PMCPLIRQ[8];
        input PLPSIRQ0[8];
        input PLPSIRQ1[8];
        output PSPLIRQLPD[14];

        // GPIO

        input PMCPLGPI[32];
        output PMCPLGPO[32];

        // Timestamp

        output PSPLTIMESTAMP[64];

        // AXI

        // ACP

        input SAXIACPCLK;

        input PLPSACPDESTID[2];

        input SAXIACPAWAKEUP;
        input SAXIACPAWAKEUPCHK;

        output SAXIACPARREADY;
        output SAXIACPARREADYCHK;
        input SAXIACPARVALID;
        input SAXIACPARVALIDCHK;
        input SAXIACPARADDR[48];
        input SAXIACPARADDRCHK[6];
        input SAXIACPARID[8];
        input SAXIACPARIDCHK;
        input SAXIACPARLEN[8];
        input SAXIACPARLENCHK;
        input SAXIACPARCACHE[4];
        input SAXIACPARDOMAIN[2];
        input SAXIACPARPROT[3];
        input SAXIACPARCTLCHK0;
        input SAXIACPARCTLCHK1;
        input SAXIACPARCTLCHK2;

        output SAXIACPAWREADY;
        output SAXIACPAWREADYCHK;
        input SAXIACPAWVALID;
        input SAXIACPAWVALIDCHK;
        input SAXIACPAWADDR[48];
        input SAXIACPAWADDRCHK[6];
        input SAXIACPAWID[8];
        input SAXIACPAWIDCHK;
        input SAXIACPAWLEN[8];
        input SAXIACPAWLENCHK;
        input SAXIACPAWCACHE[4];
        input SAXIACPAWDOMAIN[2];
        input SAXIACPAWPROT[3];
        input SAXIACPAWSNOOP[4];
        input SAXIACPAWCTLCHK0;
        input SAXIACPAWCTLCHK1;
        input SAXIACPAWCTLCHK2;
        input SAXIACPAWSTASHLPIDEN;
        input SAXIACPAWSTASHLPID[4];
        input SAXIACPAWSTASHCHK;

        input SAXIACPBREADY;
        input SAXIACPBREADYCHK;
        output SAXIACPBVALID;
        output SAXIACPBVALIDCHK;
        output SAXIACPBID[8];
        output SAXIACPBIDCHK;
        output SAXIACPBRESP[2];
        output SAXIACPBRESPCHK;

        input SAXIACPRREADY;
        input SAXIACPRREADYCHK;
        output SAXIACPRVALID;
        output SAXIACPRVALIDCHK;
        output SAXIACPRDATA[128];
        output SAXIACPRDATACHK[16];
        output SAXIACPRID[8];
        output SAXIACPRIDCHK;
        output SAXIACPRLAST;
        output SAXIACPRLASTCHK;
        output SAXIACPRRESP[2];
        output SAXIACPRRESPCHK;

        output SAXIACPWREADY;
        output SAXIACPWREADYCHK;
        input SAXIACPWVALID;
        input SAXIACPWVALIDCHK;
        input SAXIACPWDATA[128];
        input SAXIACPWDATACHK[16];
        input SAXIACPWLAST;
        input SAXIACPWLASTCHK;
        input SAXIACPWSTRB[16];
        input SAXIACPWSTRBCHK[2];

        // PL_AXI_FPD

        input IFPLAFIFM0ACE5LITESACE5LGP0AWAKEUP;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWAKEUPCHK;

        input SACE5LGP0RCLK;

        output SACE5LGP0RACOUNT[4];
        output SACE5LGP0RACOUNTCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0ARREADY;
        output IFPLAFIFM0ACE5LITESACE5LGP0ARREADYCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARVALID;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARVALIDCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARADDR[49];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARADDRCHK[7];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARID[11];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARIDCHK[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARLEN[8];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARLENCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARBURST[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARCACHE[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARDOMAIN[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARLOCK;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARPROT[3];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARQOS[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARSIZE[3];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARSNOOP[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARCTLCHK0;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARCTLCHK1;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARCTLCHK2;
        input IFPLAFIFM0ACE5LITESACE5LGP0ARUSER[12];
        input IFPLAFIFM0ACE5LITESACE5LGP0ARUSERCHK[2];

        output SACE5LGP0RCOUNT[4];
        output SACE5LGP0RCOUNTCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0RREADY;
        input IFPLAFIFM0ACE5LITESACE5LGP0RREADYCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0RVALID;
        output IFPLAFIFM0ACE5LITESACE5LGP0RVALIDCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0RDATA[256];
        output IFPLAFIFM0ACE5LITESACE5LGP0RDATACHK[32];
        output IFPLAFIFM0ACE5LITESACE5LGP0RID[11];
        output IFPLAFIFM0ACE5LITESACE5LGP0RIDCHK[2];
        output IFPLAFIFM0ACE5LITESACE5LGP0RLAST;
        output IFPLAFIFM0ACE5LITESACE5LGP0RLASTCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0RRESP[2];
        output IFPLAFIFM0ACE5LITESACE5LGP0RRESPCHK;
        output IFPLAFIFM0ACE5LITERPOISON;
        output IFPLAFIFM0ACE5LITERPOISONCHK;

        input SACE5LGP0WCLK;

        output SACE5LGP0WACOUNT[4];
        output SACE5LGP0WACOUNTCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0AWREADY;
        output IFPLAFIFM0ACE5LITESACE5LGP0AWREADYCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWVALID;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWVALIDCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWADDR[49];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWADDRCHK[7];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWID[11];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWIDCHK[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWLEN[8];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWLENCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWBURST[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWCACHE[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWDOMAIN[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWLOCK;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWPROT[3];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWQOS[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSIZE[3];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSNOOP[4];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWCTLCHK0;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWCTLCHK1;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWCTLCHK2;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWUSER[12];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWUSERCHK[2];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHLPIDEN;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHLPID[5];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHLPIDCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHNIDEN;
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHNID[11];
        input IFPLAFIFM0ACE5LITESACE5LGP0AWSTASHNIDCHK;

        input IFPLAFIFM0ACE5LITESACE5LGP0BREADY;
        input IFPLAFIFM0ACE5LITESACE5LGP0BREADYCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0BVALID;
        output IFPLAFIFM0ACE5LITESACE5LGP0BVALIDCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0BID[11];
        output IFPLAFIFM0ACE5LITESACE5LGP0BIDCHK[2];
        output IFPLAFIFM0ACE5LITESACE5LGP0BRESP[2];
        output IFPLAFIFM0ACE5LITESACE5LGP0BRESPCHK;

        output SACE5LGP0WCOUNT[7];
        output SACE5LGP0WCOUNTCHK;
        output IFPLAFIFM0ACE5LITESACE5LGP0WREADY;
        output IFPLAFIFM0ACE5LITESACE5LGP0WREADYCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0WVALID;
        input IFPLAFIFM0ACE5LITESACE5LGP0WVALIDCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0WDATA[256];
        input IFPLAFIFM0ACE5LITESACE5LGP0WDATACHK[32];
        input IFPLAFIFM0ACE5LITESACE5LGP0WLAST;
        input IFPLAFIFM0ACE5LITESACE5LGP0WLASTCHK;
        input IFPLAFIFM0ACE5LITESACE5LGP0WSTRB[32];
        input IFPLAFIFM0ACE5LITESACE5LGP0WSTRBCHK[4];
        input IFPLAFIFM0ACE5LITEWPOISON;
        input IFPLAFIFM0ACE5LITEWPOISONCHK;

        // PL_AXI_LPD

        input PLLPXAFIFMCOHDISABLE;

        input SAXIGP4AWAKEUPCHK;

        input SAXIGP4RCLK;

        output SAXIGP4RACOUNT[4];
        output SAXIGP4RACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4ARREADY;
        output IFPLAFIFM4AXISAXIGP4ARREADYCHK;
        input IFPLAFIFM4AXISAXIGP4ARVALID;
        input IFPLAFIFM4AXISAXIGP4ARVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4ARADDR[64];
        input IFPLAFIFM4AXISAXIGP4ARADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4ARID[16];
        input IFPLAFIFM4AXISAXIGP4ARIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4ARLEN[8];
        input IFPLAFIFM4AXISAXIGP4ARLENCHK;
        input IFPLAFIFM4AXISAXIGP4ARBURST[2];
        input IFPLAFIFM4AXISAXIGP4ARCACHE[4];
        input IFPLAFIFM4AXISAXIGP4ARLOCK;
        input IFPLAFIFM4AXISAXIGP4ARPROT[3];
        input IFPLAFIFM4AXISAXIGP4ARQOS[4];
        input IFPLAFIFM4AXISAXIGP4ARREGION[4];
        input IFPLAFIFM4AXISAXIGP4ARSIZE[3];
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4ARCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4ARUSER[16];
        input IFPLAFIFM4AXISAXIGP4ARUSERCHK[2];

        output SAXIGP4RCOUNT[8];
        output SAXIGP4RCOUNTCHK;
        input IFPLAFIFM4AXISAXIGP4RREADY;
        input IFPLAFIFM4AXISAXIGP4RREADYCHK;
        output IFPLAFIFM4AXISAXIGP4RVALID;
        output IFPLAFIFM4AXISAXIGP4RVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4RDATA[128];
        output IFPLAFIFM4AXISAXIGP4RDATACHK[16];
        output IFPLAFIFM4AXISAXIGP4RID[16];
        output IFPLAFIFM4AXISAXIGP4RIDCHK[2];
        output IFPLAFIFM4AXISAXIGP4RLAST;
        output IFPLAFIFM4AXISAXIGP4RLASTCHK;
        output IFPLAFIFM4AXISAXIGP4RRESP[2];
        output IFPLAFIFM4AXISAXIGP4RRESPCHK;
        output IFPLAFIFM4AXISAXIGP4RUSER[18];
        output IFPLAFIFM4AXISAXIGP4RUSERCHK[3];

        input SAXIGP4WCLK;

        output SAXIGP4WACOUNT[4];
        output SAXIGP4WACOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4AWREADY;
        output IFPLAFIFM4AXISAXIGP4AWREADYCHK;
        input IFPLAFIFM4AXISAXIGP4AWVALID;
        input IFPLAFIFM4AXISAXIGP4AWVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4AWADDR[64];
        input IFPLAFIFM4AXISAXIGP4AWADDRCHK[8];
        input IFPLAFIFM4AXISAXIGP4AWID[16];
        input IFPLAFIFM4AXISAXIGP4AWIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4AWLEN[8];
        input IFPLAFIFM4AXISAXIGP4AWLENCHK;
        input IFPLAFIFM4AXISAXIGP4AWBURST[2];
        input IFPLAFIFM4AXISAXIGP4AWCACHE[4];
        input IFPLAFIFM4AXISAXIGP4AWLOCK;
        input IFPLAFIFM4AXISAXIGP4AWPROT[3];
        input IFPLAFIFM4AXISAXIGP4AWQOS[4];
        input IFPLAFIFM4AXISAXIGP4AWREGION[4];
        input IFPLAFIFM4AXISAXIGP4AWSIZE[3];
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK0;
        input IFPLAFIFM4AXISAXIGP4AWCTLCHK1;
        input IFPLAFIFM4AXISAXIGP4AWUSER[16];
        input IFPLAFIFM4AXISAXIGP4AWUSERCHK[2];

        input IFPLAFIFM4AXISAXIGP4BREADY;
        input IFPLAFIFM4AXISAXIGP4BREADYCHK;
        output IFPLAFIFM4AXISAXIGP4BVALID;
        output IFPLAFIFM4AXISAXIGP4BVALIDCHK;
        output IFPLAFIFM4AXISAXIGP4BID[16];
        output IFPLAFIFM4AXISAXIGP4BIDCHK[2];
        output IFPLAFIFM4AXISAXIGP4BRESP[2];
        output IFPLAFIFM4AXISAXIGP4BRESPCHK[1];
        output IFPLAFIFM4AXISAXIGP4BUSER;
        output IFPLAFIFM4AXISAXIGP4BUSERCHK;

        output SAXIGP4WCOUNT[8];
        output SAXIGP4WCOUNTCHK;
        output IFPLAFIFM4AXISAXIGP4WREADY;
        output IFPLAFIFM4AXISAXIGP4WREADYCHK;
        input IFPLAFIFM4AXISAXIGP4WVALID;
        input IFPLAFIFM4AXISAXIGP4WVALIDCHK;
        input IFPLAFIFM4AXISAXIGP4WDATA[128];
        input IFPLAFIFM4AXISAXIGP4WDATACHK[16];
        input IFPLAFIFM4AXISAXIGP4WID[16];
        input IFPLAFIFM4AXISAXIGP4WIDCHK[2];
        input IFPLAFIFM4AXISAXIGP4WLAST;
        input IFPLAFIFM4AXISAXIGP4WLASTCHK;
        input IFPLAFIFM4AXISAXIGP4WSTRB[16];
        input IFPLAFIFM4AXISAXIGP4WSTRBCHK[2];
        input IFPLAFIFM4AXISAXIGP4WUSER[18];
        input IFPLAFIFM4AXISAXIGP4WUSERCHK[3];

        // FPD_AXI_PL

        input MAXIGP0ACLK;

        input IFAFIFS0PLAXIMAXIGP0ARREADY;
        input IFAFIFS0PLAXIMAXIGP0ARREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0ARVALID;
        output IFAFIFS0PLAXIMAXIGP0ARVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0ARADDR[64];
        output IFAFIFS0PLAXIMAXIGP0ARADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0ARID[16];
        output IFAFIFS0PLAXIMAXIGP0ARIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0ARLEN[8];
        output IFAFIFS0PLAXIMAXIGP0ARLENCHK;
        output IFAFIFS0PLAXIMAXIGP0ARBURST[2];
        output IFAFIFS0PLAXIMAXIGP0ARCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0ARLOCK;
        output IFAFIFS0PLAXIMAXIGP0ARPROT[3];
        output IFAFIFS0PLAXIMAXIGP0ARQOS[4];
        output IFAFIFS0PLAXIMAXIGP0ARREGION[4];
        output IFAFIFS0PLAXIMAXIGP0ARSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0ARCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0ARUSER[16];
        output IFAFIFS0PLAXIMAXIGP0ARUSERCHK[2];

        input IFAFIFS0PLAXIMAXIGP0AWREADY;
        input IFAFIFS0PLAXIMAXIGP0AWREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0AWVALID;
        output IFAFIFS0PLAXIMAXIGP0AWVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0AWADDR[64];
        output IFAFIFS0PLAXIMAXIGP0AWADDRCHK[8];
        output IFAFIFS0PLAXIMAXIGP0AWID[16];
        output IFAFIFS0PLAXIMAXIGP0AWIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0AWLEN[8];
        output IFAFIFS0PLAXIMAXIGP0AWLENCHK;
        output IFAFIFS0PLAXIMAXIGP0AWBURST[2];
        output IFAFIFS0PLAXIMAXIGP0AWCACHE[4];
        output IFAFIFS0PLAXIMAXIGP0AWLOCK;
        output IFAFIFS0PLAXIMAXIGP0AWPROT[3];
        output IFAFIFS0PLAXIMAXIGP0AWQOS[4];
        output IFAFIFS0PLAXIMAXIGP0AWREGION[4];
        output IFAFIFS0PLAXIMAXIGP0AWSIZE[3];
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK0;
        output IFAFIFS0PLAXIMAXIGP0AWCTLCHK1;
        output IFAFIFS0PLAXIMAXIGP0AWUSER[16];
        output IFAFIFS0PLAXIMAXIGP0AWUSERCHK[2];

        output IFAFIFS0PLAXIMAXIGP0BREADY;
        output IFAFIFS0PLAXIMAXIGP0BREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0BVALID;
        input IFAFIFS0PLAXIMAXIGP0BVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0BID[16];
        input IFAFIFS0PLAXIMAXIGP0BIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0BRESP[2];
        input IFAFIFS0PLAXIMAXIGP0BRESPCHK[1];
        input IFAFIFS0PLAXIMAXIGP0BUSER;
        input IFAFIFS0PLAXIMAXIGP0BUSERCHK;

        output IFAFIFS0PLAXIMAXIGP0RREADY;
        output IFAFIFS0PLAXIMAXIGP0RREADYCHK;
        input IFAFIFS0PLAXIMAXIGP0RVALID;
        input IFAFIFS0PLAXIMAXIGP0RVALIDCHK;
        input IFAFIFS0PLAXIMAXIGP0RDATA[128];
        input IFAFIFS0PLAXIMAXIGP0RDATACHK[16];
        input IFAFIFS0PLAXIMAXIGP0RID[16];
        input IFAFIFS0PLAXIMAXIGP0RIDCHK[2];
        input IFAFIFS0PLAXIMAXIGP0RLAST;
        input IFAFIFS0PLAXIMAXIGP0RLASTCHK;
        input IFAFIFS0PLAXIMAXIGP0RRESP[2];
        input IFAFIFS0PLAXIMAXIGP0RRESPCHK;
        input IFAFIFS0PLAXIMAXIGP0RUSER[18];
        input IFAFIFS0PLAXIMAXIGP0RUSERCHK[3];

        input IFAFIFS0PLAXIMAXIGP0WREADY;
        input IFAFIFS0PLAXIMAXIGP0WREADYCHK;
        output IFAFIFS0PLAXIMAXIGP0WVALID;
        output IFAFIFS0PLAXIMAXIGP0WVALIDCHK;
        output IFAFIFS0PLAXIMAXIGP0WDATA[128];
        output IFAFIFS0PLAXIMAXIGP0WDATACHK[16];
        output IFAFIFS0PLAXIMAXIGP0WID[16];
        output IFAFIFS0PLAXIMAXIGP0WIDCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WLAST;
        output IFAFIFS0PLAXIMAXIGP0WLASTCHK;
        output IFAFIFS0PLAXIMAXIGP0WSTRB[16];
        output IFAFIFS0PLAXIMAXIGP0WSTRBCHK[2];
        output IFAFIFS0PLAXIMAXIGP0WUSER[18];
        output IFAFIFS0PLAXIMAXIGP0WUSERCHK[3];

        // LPD_AXI_PL

        input MAXIGP2ACLK;

        input IFAXIPLPORT2MAXIGP2ARREADY;
        input IFAXIPLPORT2MAXIGP2ARREADYCHK;
        output IFAXIPLPORT2MAXIGP2ARVALID;
        output IFAXIPLPORT2MAXIGP2ARVALIDCHK;
        output IFAXIPLPORT2MAXIGP2ARADDR[64];
        output IFAXIPLPORT2MAXIGP2ARADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2ARID[16];
        output IFAXIPLPORT2MAXIGP2ARIDCHK[2];
        output IFAXIPLPORT2MAXIGP2ARLEN[8];
        output IFAXIPLPORT2MAXIGP2ARLENCHK;
        output IFAXIPLPORT2MAXIGP2ARBURST[2];
        output IFAXIPLPORT2MAXIGP2ARCACHE[4];
        output IFAXIPLPORT2MAXIGP2ARLOCK;
        output IFAXIPLPORT2MAXIGP2ARPROT[3];
        output IFAXIPLPORT2MAXIGP2ARQOS[4];
        output IFAXIPLPORT2MAXIGP2ARREGION[4];
        output IFAXIPLPORT2MAXIGP2ARSIZE[3];
        output IFAXIPLPORT2MAXIGP2ARCTLCHK1;
        output IFAXIPLPORT2MAXIGP2ARCTLCHK2;
        output IFAXIPLPORT2MAXIGP2ARUSER[16];
        output IFAXIPLPORT2MAXIGP2ARUSERCHK[2];

        input IFAXIPLPORT2MAXIGP2AWREADY;
        input IFAXIPLPORT2MAXIGP2AWREADYCHK;
        output IFAXIPLPORT2MAXIGP2AWVALID;
        output IFAXIPLPORT2MAXIGP2AWVALIDCHK;
        output IFAXIPLPORT2MAXIGP2AWADDR[64];
        output IFAXIPLPORT2MAXIGP2AWADDRCHK[8];
        output IFAXIPLPORT2MAXIGP2AWID[16];
        output IFAXIPLPORT2MAXIGP2AWIDCHK[2];
        output IFAXIPLPORT2MAXIGP2AWLEN[8];
        output IFAXIPLPORT2MAXIGP2AWLENCHK;
        output IFAXIPLPORT2MAXIGP2AWBURST[2];
        output IFAXIPLPORT2MAXIGP2AWCACHE[4];
        output IFAXIPLPORT2MAXIGP2AWLOCK;
        output IFAXIPLPORT2MAXIGP2AWPROT[3];
        output IFAXIPLPORT2MAXIGP2AWQOS[4];
        output IFAXIPLPORT2MAXIGP2AWREGION[4];
        output IFAXIPLPORT2MAXIGP2AWSIZE[3];
        output IFAXIPLPORT2MAXIGP2AWCTLCHK1;
        output IFAXIPLPORT2MAXIGP2AWCTLCHK2;
        output IFAXIPLPORT2MAXIGP2AWUSER[16];
        output IFAXIPLPORT2MAXIGP2AWUSERCHK[2];

        output IFAXIPLPORT2MAXIGP2BREADY;
        output IFAXIPLPORT2MAXIGP2BREADYCHK;
        input IFAXIPLPORT2MAXIGP2BVALID;
        input IFAXIPLPORT2MAXIGP2BVALIDCHK;
        input IFAXIPLPORT2MAXIGP2BID[16];
        input IFAXIPLPORT2MAXIGP2BIDCHK[2];
        input IFAXIPLPORT2MAXIGP2BRESP[2];
        input IFAXIPLPORT2MAXIGP2BRESPCHK[1];
        input IFAXIPLPORT2MAXIGP2BUSER;
        input IFAXIPLPORT2MAXIGP2BUSERCHK;

        output IFAXIPLPORT2MAXIGP2RREADY;
        output IFAXIPLPORT2MAXIGP2RREADYCHK;
        input IFAXIPLPORT2MAXIGP2RVALID;
        input IFAXIPLPORT2MAXIGP2RVALIDCHK;
        input IFAXIPLPORT2MAXIGP2RDATA[128];
        input IFAXIPLPORT2MAXIGP2RDATACHK[16];
        input IFAXIPLPORT2MAXIGP2RID[16];
        input IFAXIPLPORT2MAXIGP2RIDCHK[2];
        input IFAXIPLPORT2MAXIGP2RLAST;
        input IFAXIPLPORT2MAXIGP2RLASTCHK;
        input IFAXIPLPORT2MAXIGP2RRESP[2];
        input IFAXIPLPORT2MAXIGP2RRESPCHK;
        input IFAXIPLPORT2MAXIGP2RUSER[18];
        input IFAXIPLPORT2MAXIGP2RUSERCHK[3];

        input IFAXIPLPORT2MAXIGP2WREADY;
        input IFAXIPLPORT2MAXIGP2WREADYCHK;
        output IFAXIPLPORT2MAXIGP2WVALID;
        output IFAXIPLPORT2MAXIGP2WVALIDCHK;
        output IFAXIPLPORT2MAXIGP2WDATA[128];
        output IFAXIPLPORT2MAXIGP2WDATACHK[16];
        output IFAXIPLPORT2MAXIGP2WID[16];
        output IFAXIPLPORT2MAXIGP2WIDCHK[2];
        output IFAXIPLPORT2MAXIGP2WLAST;
        output IFAXIPLPORT2MAXIGP2WLASTCHK;
        output IFAXIPLPORT2MAXIGP2WSTRB[16];
        output IFAXIPLPORT2MAXIGP2WSTRBCHK[2];
        output IFAXIPLPORT2MAXIGP2WUSER[18];
        output IFAXIPLPORT2MAXIGP2WUSERCHK[3];

        // DTI

        input PLPSXDTI0CLK;

        input IFPLPSDTIDN0TWAKEUP;
        input IFPLPSDTIDN0TWAKEUPCHK;
        output IFPLPSDTIDN0TREADY;
        output IFPLPSDTIDN0TREADYCHK;
        input IFPLPSDTIDN0TVALID;
        input IFPLPSDTIDN0TVALIDCHK;
        input IFPLPSDTIDN0TDATA[160];
        input IFPLPSDTIDN0TDATACHK[160];
        input IFPLPSDTIDN0TID[5];
        input IFPLPSDTIDN0TIDCHK[5];
        input IFPLPSDTIDN0TKEEP[20];
        input IFPLPSDTIDN0TKEEPCHK[20];
        input IFPLPSDTIDN0TLAST;
        input IFPLPSDTIDN0TLASTCHK;

        output IFPSPLDTIUP0TWAKEUP;
        output IFPSPLDTIUP0TWAKEUPCHK;
        input IFPSPLDTIUP0TREADY;
        input IFPSPLDTIUP0TREADYCHK;
        output IFPSPLDTIUP0TVALID;
        output IFPSPLDTIUP0TVALIDCHK;
        output IFPSPLDTIUP0TDATA[160];
        output IFPSPLDTIUP0TDATACHK[160];
        output IFPSPLDTIUP0TDEST[5];
        output IFPSPLDTIUP0TDESTCHK[5];
        output IFPSPLDTIUP0TKEEP[20];
        output IFPSPLDTIUP0TKEEPCHK[20];
        output IFPSPLDTIUP0TLAST;
        output IFPSPLDTIUP0TLASTCHK;

        input PLDTI0TESTSELN;

        input MMTPSDTIFMUERROUT;

        // ADMA

        input ADMAFCICLK[8];
        output ADMA2PLTVLD[8];
        input PL2ADMATACK[8];
        input PL2ADMACVLD[8];
        output ADMA2PLCACK[8];

        // SDMA

        input SDMAFCICLK[8];
        output SDMA2PLTVLD[8];
        input PL2SDMATACK[8];
        input PL2SDMACVLD[8];
        output SDMA2PLCACK[8];

        // CFG etc.

        input GCAPCLK;
        input CFUGCAPB;
        input CFUUSRGSRB;
        input CFUUSRGTSB;
        output PSPLCFGMCLK;
        output CFUEOS;

        output USRSEL[4];
        output USRDRCK[4];
        output USRRESET[4];
        output USRRUNTEST[4];
        output USRCAPTURE[4];
        output USRSHIFT[4];
        output USRUPDATE[4];
        output USRTCK[4];
        output USRTMS[4];
        output USRTDI[4];
        input USRTDO[4];

        input MJTAGTCK;
        input MJTAGTMS;
        input MJTAGTDI;
        output MJTAGTDO;

        output IFPMCCFUSEUCFUSEUCRCERROR;
        output IFPMCCFUSEUCFUSEUECCERROR;
        output IFPMCCFUSEUCFUSEUENDOFCALIB;
        output IFPMCCFUSEUCFUSEUHALTED;
        output IFPMCCFUSEUCFUSEUHEARTBEAT;

        // ASU

        input PLASUAPBCLK;
        output IFASUPLCTRLAPBPENABLE;
        output IFASUPLCTRLAPBPWRITE;
        output IFASUPLCTRLAPBPPROT[3];
        output IFASUPLCTRLAPBPSEL;
        output IFASUPLCTRLAPBPADDR[32];
        output IFASUPLCTRLAPBPWDATA[32];
        output IFASUPLCTRLAPBPSTRB[4];
        input IFASUPLCTRLAPBPREADY;
        input IFASUPLCTRLAPBPRDATA[32];
        input IFASUPLCTRLAPBPSLVERR;

        input PLASUSTREAMCLK;

        input ASUPLREADYB;
        output ASUPLVALID;
        output ASUPLDATA[128];
        output ASUPLSTRB[16];
        output ASUPLKEYOUT[256];
        output ASUPLLAST;

        output PLASUREADYB;
        input PLASUVALID;
        input PLASUDATA[128];
        input PLASUSTRB[16];
        input PLASULAST;

        input PLASUIRQ0;
        input PLASUIRQ1;
        input PLASUIRQ2;
        input PLASUIRQ3;
        input PLASUNONFATALERROR0;
        input PLASUNONFATALERROR1;
        input PLASUGDFABRIC[2];

        input PLASUSCANSPAREIN[16];

        // EMIO

        input FMIOGPIOIN[32];
        output FMIOGPIOOEN[32];
        output FMIOGPIOOUT[32];

        input EMIOGPIO1I[32];
        output EMIOGPIO1O[32];
        output EMIOGPIO1TN[32];

        output FMIOWWDT0INTERRUPT;
        output FMIOWWDT0RESET;
        output FMIOWWDT0RESETPENDING;
        output FMIOGWDT0WS0;
        output FMIOGWDT0WS1;

        output FMIOFPDWWDTINTERRUPT;
        output FMIOFPDWWDTRESET;
        output FMIOFPDWWDTRESETPENDING;
        output FMIOFPDGWDTWS0;
        output FMIOFPDGWDTWS1;

        for i in 0..8 {
            input "EMIOTTC{i}CLKI"[3];
            output "EMIOTTC{i}WAVEO"[3];
        }

        for i in 0..2 {
            input "FMIOUART{i}RXD";
            output "FMIOUART{i}TXD";
            input "FMIOUART{i}NUARTCTS";
            output "FMIOUART{i}NUARTRTS";
            input "FMIOUART{i}NUARTDSR";
            output "FMIOUART{i}NUARTDTR";
            input "FMIOUART{i}NUARTDCD";
            input "FMIOUART{i}NUARTRI";
            input "FMIOUART{i}SIRIN";
            output "FMIOUART{i}NSIROUT";
            output "FMIOUART{i}NUARTOUT1";
            output "FMIOUART{i}NUARTOUT2";
        }

        for i in 0..8 {
            input "EMIOI2C{i}SCLI";
            output "EMIOI2C{i}SCLO";
            output "EMIOI2C{i}SCLTN";
            input "EMIOI2C{i}SDAI";
            output "EMIOI2C{i}SDAO";
            output "EMIOI2C{i}SDATN";
        }

        input FMIOSYSMONI2CSCLINPUT;
        output FMIOSYSMONI2CSCLTRIB;
        input FMIOSYSMONI2CSDAINPUT;
        output FMIOSYSMONI2CSDATRIB;
        output FMIOSYSMONI2CSMBALERTTRIB;

        for i in 0..2 {
            input "EMIOSPI{i}MI";
            output "EMIOSPI{i}MO";
            output "EMIOSPI{i}MOTN";
            input "EMIOSPI{i}SCLKI";
            output "EMIOSPI{i}SCLKO";
            output "EMIOSPI{i}SCLKTN";
            input "EMIOSPI{i}SI";
            output "EMIOSPI{i}SO";
            input "EMIOSPI{i}SSIN";
            output "EMIOSPI{i}SSNTN";
            output "EMIOSPI{i}SSON"[3];
            output "EMIOSPI{i}STN";
        }

        input FMIOCHARSPITESTSELECTN;
        input FMIOCHARSPISELECTION;
        input FMIOCHARSPITESTINPUT;
        output FMIOCHARSPITESTOUTPUT;

        for i in 0..6 {
            input "EMIOCAN{i}PHYRX";
            output "EMIOCAN{i}PHYTX";
        }

        for i in 0..2 {
            output "FMIOSD{i}BUSPOWEROUT";
            output "FMIOSD{i}BUSVOLTAGEOUT"[3];
            input "FMIOSD{i}DLLTESTIN"[4];
            output "FMIOSD{i}LEDCONTROLOUT";
        }


        input EMIOENETTSUCLK;
        output EMIOENET0GEMTSUTIMERCNT[94];
        input FMIOGEMTSUCLKFROMPL;

        for i in 0..2 {
            output "EMIOENET{i}DMABUSWIDTH"[2];
            input "EMIOENET{i}EXTINTIN";
            input "EMIOENET{i}GMIIRXCLK";
            input "EMIOENET{i}GMIITXCLK";
            input "EMIOENET{i}MDIOI";
            output "EMIOENET{i}MDIOMDC";
            output "EMIOENET{i}MDIOO";
            output "EMIOENET{i}MDIOTN";
            output "EMIOENET{i}SPEEDMODE"[4];
            output "EMIOGEM{i}DELAYREQRX";
            output "EMIOGEM{i}DELAYREQTX";
            output "EMIOGEM{i}PDELAYREQRX";
            output "EMIOGEM{i}PDELAYREQTX";
            output "EMIOGEM{i}PDELAYRESPRX";
            output "EMIOGEM{i}PDELAYRESPTX";
            output "EMIOGEM{i}RXSOF";
            output "EMIOGEM{i}SYNCFRAMERX";
            output "EMIOGEM{i}SYNCFRAMETX";
            input "EMIOGEM{i}TSUINCCTRL"[2];
            output "EMIOGEM{i}TSUTIMERCMPVAL";
            output "EMIOGEM{i}TXSOF";

            input "FMIOGEM{i}FIFORXCLKFROMPL";
            input "FMIOGEM{i}FIFOTXCLKFROMPL";
            output "FMIOGEM{i}RXDATABUFWRQ0";
            output "FMIOGEM{i}RXDATABUFWRQ1";
        }
        input FMIOCHARGEMSELECTION;
        input FMIOCHARGEMTESTINPUT;
        output FMIOCHARGEMTESTOUTPUT;
        input FMIOCHARGEMTESTSELECTN;

        input EMIOHUBPORTOVERCRNTUSB20;
        output EMIOU2DSPORTVBUSCTRLUSB30;

        output FMIODTBOUT[2];

        output FMIOPHYREFDIGCLK;

        input FMIOMPHYJTAGTRSTN;
        input FMIOMPHYJTAGTCK;
        input FMIOMPHYJTAGTMS;
        input FMIOMPHYJTAGTDI;
        output FMIOMPHYJTAGTDO;
        output FMIOMPHYJTAGTDOEN;

        input FMIOOCLAMONCLKFROMPL;
        output FMIOOCLAMONCLKTOPL;

        input FMIOCHARAFIFSFPDTESTINPUT;
        output FMIOCHARAFIFSFPDTESTOUTPUT;
        input FMIOCHARAFIFSFPDTESTSELECTN;
        input FMIOCHARAFIFSLPDTESTINPUT;
        output FMIOCHARAFIFSLPDTESTOUTPUT;
        input FMIOCHARAFIFSLPDTESTSELECTN;

        // SYSMONs

        output IFSYSMONROOTUSERALARM[8];
        output IFSYSMONROOTUSERDATAREADY;
        output IFSYSMONROOTUSERMUXADDR[4];
        output IFSYSMONROOTUSERPVALUE[16];
        output IFSYSMONROOTUSERTVALUE[16];
        output IFSYSMONROOTUSERVVALUE[16];
        output PMCPLSYSMONROOTALARM[10];
        output PMCPLSYSMONROOTNEWDATA[4];

        input IFSYSMONROOTTESTFABRICCLK;
        input IFSYSMONROOTTESTTESTADCINCLK;
        input IFSYSMONROOTTESTTESTADCIN[15];
        output IFSYSMONROOTTESTTESTADCOUT[16];

        input IFLPDSYSMONSATTESTFABRICCLK;
        input IFLPDSYSMONSATTESTTESTADCINCLK;
        input IFLPDSYSMONSATTESTTESTADCIN[15];

        input IFFPDSYSMON0SATTESTFABRICCLK;
        input IFFPDSYSMON0SATTESTTESTADCINCLK;
        input IFFPDSYSMON0SATTESTTESTADCIN[15];

        // Tracing etc

        input STMEVENT[60];

        output PSPLTRIGACK[4];
        output PSPLTRIGGER[4];
        input PLPSTRIGACK[4];
        input PLPSTRIGGER[4];

        // ATB

        input PLPSATBCLK;

        output PSPLATREADY;
        input PLPSATVALID;
        input PLPSATBYTES[2];
        input PLPSATDATA[32];
        input PLPSATID[7];

        input PLPSAFREADY;
        output PSPLAFVALID;

        output PSPLSYNCREQ;

        // Trace out

        input PLPSTRACECLK;
        output PSPLTRACECTL;
        output PSPLTRACEDATA[32];

        // HSDP

        input PLHSDPCLK;
        output LPXPLHSDPRAWRSTN;

        input PLHSDPEGRESSTREADY;
        output PLHSDPEGRESSTVALID;
        output PLHSDPEGRESSTDATA[64];
        output PLHSDPEGRESSTKEEP[8];
        output PLHSDPEGRESSTLAST;
        output PLHSDPEGRESSTUSER[12];

        output PLHSDPINGRESSTREADY;
        input PLHSDPINGRESSTVALID;
        input PLHSDPINGRESSTDATA[64];
        input PLHSDPINGRESSTKEEP[8];
        input PLHSDPINGRESSTLAST;

        // PMU

        input AIBPMUAFIFMFPDACK;
        input AIBPMUAFIFMLPDACK;
        output PMUAIBAFIFMFPDREQ;
        output PMUAIBAFIFMLPDREQ;

        input PLPMXU2PMUOVERCURRENTUSB1;
        output PMXPLU2PMUVBUSCTRLUSB1;

        // PPU

        output PPU1PLTRIGACKIN;
        input PPU1PLTRIGACKOUT;
        input PPU1PLTRIGIN;
        output PPU1PLTRIGOUT;

        // Errors

        output PMCERRORTOPL[72];
        input PMUERRORFROMPL[4];
        output PMUERRORTOPL[47];

        // Special clock outs to XPLLs and north HNOC only

        output HSMREFCLK[2];

        // Special test-only clock outputs

        output MCLK;
        output NPICLK;
        output NTISCANCLKIN;

        // MMI

        output PLMMIRCLK[12];
        input PLPSPLMMIAUXREFCLK;
        input PLPSPLMMIDCT10MODECLK;
        input PLPSPLMMIDCT10MODEI2SCLK;
        input PLPSPLMMIGEMTSUINCCTRLTOLPX[2];
        input PLPSPLMMIGEMTSUINCCTRL[2];
        input PLPSPLMMIRCLK0MMIVBG;
        input PLPSPLMMIRCLK0MMIVBGHALF;

        input IFPLPSPLMMISYSMONTESTFABRICCLK;
        input IFPLPSPLMMISYSMONTESTTESTADCINCLK;
        input IFPLPSPLMMISYSMONTESTTESTADCIN[15];

        input IFPLPSPLMMIGTYPFTI64APB3CLK;
        input IFPLPSPLMMIGTYPFTI64APB3PRESETN;
        input IFPLPSPLMMIGTYPFTI64APB3PENABLE;
        input IFPLPSPLMMIGTYPFTI64APB3PWRITE;
        input IFPLPSPLMMIGTYPFTI64APB3PSEL[2];
        input IFPLPSPLMMIGTYPFTI64APB3PADDR[16];
        input IFPLPSPLMMIGTYPFTI64APB3PWDATA[32];

        input PLPSPLMMIGTFTIUSRCLK;
        input PLPSPLMMIGTFTIDMONITORCLK;
        output PLGTMMIPSPLFTIDMONOUTCLK;
        input IFPLPSPLMMIGTYPFTI64DMONFIFORESET;
        input IFPLPSPLMMIGTYPFTI64GTPLLRST;

        for i in 0..4 {
            input "IFPLPSPLMMIGTYPFTI64CH{i}EYESCANRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}GTRXRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}GTTXRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}HSDPPCSRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}ILORESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXCDRRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPMARESETMASK"[7];
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPRBSCNTRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXPROGDIVRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}RXRESETMODE"[2];
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXDETECTRX";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXELECIDLE";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPMARESETMASK"[3];
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPRBSFORCEERR";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXPROGDIVRESET";
            input "IFPLPSPLMMIGTYPFTI64CH{i}TXRESETMODE"[2];
        }

        input IFPLPSPLMMIDFXCTRLF2CSCANCLKNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANMODERSTNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANENNEXT;
        input IFPLPSPLMMIDFXCTRLF2CSCANEDTUPDTNEXT;
        input IFPLPSPLMMIDFXSCWCONCATSCWIN[450];

        input IFPLPSPLMMIGTDFXSCANCLKNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANMODERSTNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANENNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANEDTUPDTNGTCTRLFS;
        input IFPLPSPLMMIGTDFXSCANCHNLINGTCTRLFS[4];
        input IFPLPSPLMMIGTDFXSCANCNTRLCHNLINGTCTRLFS;

        for i in 0..4 {
            input "IFPLPSPLMMIGTDFXSCANCHNLINFS{i}"[4];
            input "IFPLPSPLMMIGTDFXSCANCLKNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANCNTRLCHNLINFS{i}";
            input "IFPLPSPLMMIGTDFXSCANEDTUPDTNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANENNFS{i}";
            input "IFPLPSPLMMIGTDFXSCANMODERSTNFS{i}";
        }

        // PLL test

        input PLFPXPLLTESTOUTSEL;
        input PLPMCPLLTESTOUTSEL;
        input PLLPDPLLTESTOUTSEL[2];

        // Unsorted

        output FPXCMN2PLSPARE0;
        output FPXCMN2PLSPARE1;
        output FPXCMN2PLSPARE2;
        output FPXCMN2PLSPARE3;
        output FPXCMN2PLSPARE4[2];
        output FPXCMN2PLSPARE5[2];
        output FPXCMN2PLSPARE6[4];
        output FPXCMN2PLSPARE7[4];
        output FPXCMN2PLSPARE[16];
        output FPXINTPLSPARE[16];
        input PLFPXCMN2SPARE[32];
        input PLFPXINTSPARE[16];

        input GDFABRICGL[4];

        input PLLPXSPARE1[16];
        input PLLPXSPARE2[17:16];
        input PLLPXSPARE3[31:18];
        output LPXPLSPARE0;
        output LPXPLSPARE1;
        output LPXPLSPARE2;
        output LPXPLSPARE3;
        output LPXPLSPARE4[2];
        output LPXPLSPARE5[2];
        output LPXPLSPARE6[4];
        output LPXPLSPARE7[4];
        output LPXPLSPARE[16];

        input PLLPXGD0FABRIC[2];
        input PLLPXGD1FABRIC[2];

        output OSCRTCCLK;

        input PL0CLKTOLCMON;
        input PL1CLKTOLCMON;
        input PL2CLKTOLCMON;

        input PLCSSDTRIGGER;
        input PLPMCCSSDTRIGGERN;
        input PLPMCEDTUPDATEN;
        input PLPMCFPGACLOCKSTOP[4];
        input PLPMCSCANENN;
        input PLPMCSCANRESETN;
        input PLPMCSCANSYNCBYP;

        input PLPMXWWDTCLK;
        output PMXPLWWDTINTERRUPT;

        input PLPMCCTRLTESTAAG;

        input PLPSCMN12SCANSPARE[20];
        input PLPSFPXINTSCANSPARE[20];

        input PLPMCNTISCANCLOCK;
        input PLPMCNTIBUSIN[16];
        input PLPMCNTITESTEN[3];

        input PMCTPPLCLK[4];
        output PMCTPPLIN[38];
        input PMCTPPLOEN[38];
        input PMCTPPLOUT[38];

        input PMVCAPTURECLKINTIP;
        input PMVDATAININTIP;
        input PMVEN1INTIP;
        input PMVFFCLKINTIP;
        input PMVIMUXININT[5];
        input PMVRESETINTIP;
        input PMVSPAREIN1INTIP[6];
        input PMVSYNCENBINTIP;

        input SCANCLKNEXT;
        input SCANEDTUPDTNEXT;
        input SCANENNEXT;
        input SCANMODERSTNEXT;
        input SCANCLKNEXTFPD;
        input SCANEDTUPDTNEXTFPD;
        input SCANENNEXTFPD;
        input SCANMODERSTNEXTFPD;

        input SCANCHNLINAMS;
        input SCANCNTRLCHNLINAMS;

        input SCANCHNLINEXTAPU0[10];
        input SCANCHNLMASKINEXTAPU0[10];
        input SCANCNTRLCHNLINEXTAPU0;
        input SCANCNTRLCHNLMASKINEXTAPU0;

        for i in 0..4 {
            input "SCANCHNLINEXTAPU0CPU{i}"[10];
            input "SCANCHNLMASKINEXTAPU0CPU{i}"[10];
            input "SCANCNTRLCHNLINEXTAPU0CPU{i}";
            input "SCANCNTRLCHNLMASKINEXTAPU0CPU{i}";
        }

        input SCANCHNLINEXTASU[10];
        input SCANCHNLMASKINEXTASU[10];
        input SCANODCCCHNLINEXTASU;
        input SCANODCCCHNLMASKINEXTTOASU;

        input SCANCHNLINEXTCML[15];
        input SCANCHNLMASKINEXTCML[15];
        input SCANCNTRLCHNLINEXTCML;
        input SCANCNTRLCHNLMASKINEXTCML;

        input SCANCHNLINEXTCMN1[15];
        input SCANCHNLINEXTCMN2[15];
        input SCANCHNLINEXTCMN3P1[15];
        input SCANCHNLINEXTCMN4P0[15];
        input SCANCHNLMASKINEXTCMN1[15];
        input SCANCHNLMASKINEXTCMN2[15];
        input SCANCHNLMASKINEXTCMN3P1[15];
        input SCANCHNLMASKINEXTCMN4P0[15];
        input SCANCNTRLCHNLINEXTCMN1;
        input SCANCNTRLCHNLINEXTCMN2;
        input SCANCNTRLCHNLINEXTCMN3P1;
        input SCANCNTRLCHNLINEXTCMN4P0;
        input SCANCNTRLCHNLMASKINEXTCMN1;
        input SCANCNTRLCHNLMASKINEXTCMN2;
        input SCANCNTRLCHNLMASKINEXTCMN3P1;
        input SCANCNTRLCHNLMASKINEXTCMN4P0;

        input SCANCHNLINEXTFPDSYSMON0;
        input SCANCNTRLCHNLINFPDSYSMON0;

        input SCANCHNLINEXTFPD[15];
        input SCANCHNLMASKINEXTFPD[15];
        input SCANCNTRLCHNLINEXTFPD;
        input SCANODCCCHNLMASKINEXTFPD;
        input SCANODCCCHNLMASKINEXTFPD1;

        input SCANCHNLINEXTMMU[15];
        input SCANCHNLMASKINEXTMMU[15];
        input SCANCNTRLCHNLINEXTMMU;
        input SCANODCCCHNLMASKINEXTMMU;

        input SCANCHNLINEXTMMU0[4];
        input SCANCHNLMASKINEXTMMU0[4];
        input SCANODCCCHNLINEXTMMU0;

        input SCANCHNLINEXTPKI[11];
        input SCANCHNLMASKINEXTPKI[11];
        input SCANCNTRLCHNLINEXTPKI;

        input SCANCHNLINEXTSPLITTER[15];
        input SCANCHNLMASKINEXTSPLITTER[15];
        input SCANODCCCHNLINEXTSPLITTER;
        input SCANODCCCHNLMASKINEXTSPLITTER;

        input SCANCHNLINIOU[5];
        input SCANCHNLMASKINIOU[5];
        input SCANODCCCHNLINIOU;
        input SCANODCCCHNLMASKINEXTTOIOU;
        input SCANSPAREINEXTIOU[5];

        input SCANCHNLINLPX[15];
        input SCANCHNLMASKINLPX[15];
        input SCANCNTRLCHNLINLPX;
        input SCANODCCCHNLMASKINEXTTOLPX;
        input SCANSPAREINEXTLPX[12];

        input SCANCHNLINOCM[15];
        input SCANCHNLMASKINOCM[15];
        input SCANCNTRLCHNLINOCM;
        input SCANODCCCHNLMASKINEXTTOOCM;
        input SCANSPAREINEXTOCM[10];

        input SCANCHNLINRPU[6];
        input SCANCHNLMASKINRPU[6];
        input SCANCNTRLCHNLINRPUA;
        input SCANCNTRLCHNLINRPUB;
        input SCANCNTRLCHNLINRPUC;
        input SCANODCCCHNLMASKINEXTTORPUA;
        input SCANODCCCHNLMASKINEXTTORPUB;
        input SCANODCCCHNLMASKINEXTTORPUC;
        input SCANSPAREINEXTRPUA[5];
        input SCANSPAREINEXTRPUB[5];
        input SCANSPAREINEXTRPUC[5];

        input TESTBSCANACMODE;
        input TESTBSCANACTEST;
        input TESTBSCANCLOCKDR;
        input TESTBSCANENN;
        input TESTBSCANEXTEST;
        input TESTBSCANEXTESTSMPL;
        input TESTBSCANINITMEMORY;
        input TESTBSCANMODEC;
        input TESTBSCANRESETTAPB;
        input TESTBSCANSHIFTDR;
        input TESTBSCANTDI;
        input TESTBSCANUPDATEDR;

        input TESTUSB0FUNCMUX0N;
        input TESTUSB0SCANMUX0N;
        input TESTUSB1FUNCMUX0N;
        input TESTUSB1SCANMUX0N;

        input TSTRTCCALIBREGIN[21];
        input TSTRTCCALIBREGWE;
        input TSTRTCCLK;
        input TSTRTCDISABLEBATOP;
        input TSTRTCOSCCNTRLIN[4];
        input TSTRTCOSCCNTRLWE;
        input TSTRTCSECRELOAD;
        input TSTRTCTESTCLOCKSELECTN;
        input TSTRTCTESTMODEN;
        input TSTRTCTIMESETREGIN[32];
        input TSTRTCTIMESETREGWE;

        // -- MMI part --

        // DisplayPort

        input DPEXTERNALCUSTOMEVENT1;
        input DPEXTERNALCUSTOMEVENT2;
        input DPEXTERNALVSYNCEVENT;

        input PLMMIDC1XCLK;
        input PLMMIDC2XCLK;
        input PLMMIDCI2SS0CLK;
        input PLMMIDCI2SS1CLK;
        input PLMMIDCI2SS2CLK;
        input PLMMIDCI2SS3CLK;
        input PLMMIDCS1CLK;
        input PLMMIDCS2CLK;
        input PLMMIDCS3CLK;

        output IFMMIPLI2S0I2SLRCLKI;
        output IFMMIPLI2S0I2SDATAI[4];

        input IFPLMMII2S0I2SLRCLKI;
        input IFPLMMII2S0I2SDATAI[4];
        input IFPLMMII2S1I2SLRCLKI;
        input IFPLMMII2S1I2SDATAI[4];
        input IFPLMMII2S2I2SLRCLKI;
        input IFPLMMII2S2I2SDATAI[4];
        input IFPLMMII2S3I2SLRCLKI;
        input IFPLMMII2S3I2SDATAI[4];

        for i in 0..2 {
        output "IFMMIPLSDPFB{i}EXTSDPREQI";
        input "IFMMIPLSDPFB{i}EXTSDPACKO";
        output "IFMMIPLSDPFB{i}EXTSDPDATAI"[72];
        input "IFMMIPLSDPFB{i}EXTSDPHORIZONTALBLANKINGO";
        input "IFMMIPLSDPFB{i}EXTSDPVERTICALBLANKINGO";
        output "IFMMIPLSDPFB{i}EXTDPSDPACKI";
        output "IFMMIPLSDPFB{i}EXTDPSDPHORIZONTALBLANKINGI";
        output "IFMMIPLSDPFB{i}EXTDPSDPVERTICALBLANKINGI";
        }

        input IFPLMMISDP00EXTSDPREQI;
        output IFPLMMISDP00EXTSDPACKO;
        input IFPLMMISDP00EXTSDPDATAI[72];
        output IFPLMMISDP00EXTSDPLINECNTMAT[2];
        output IFPLMMISDP00EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP00EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP01EXTSDPREQI;
        output IFPLMMISDP01EXTSDPACKO;
        input IFPLMMISDP01EXTSDPDATAI[72];
        output IFPLMMISDP01EXTSDPLINECNTMAT[2];
        output IFPLMMISDP01EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP01EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP1EXTSDPREQI;
        output IFPLMMISDP1EXTSDPACKO;
        input IFPLMMISDP1EXTSDPDATAI[72];
        output IFPLMMISDP1EXTSDPLINECNTMAT[2];
        output IFPLMMISDP1EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP1EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP2EXTSDPREQI;
        output IFPLMMISDP2EXTSDPACKO;
        input IFPLMMISDP2EXTSDPDATAI[72];
        output IFPLMMISDP2EXTSDPLINECNTMAT[2];
        output IFPLMMISDP2EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP2EXTSDPVERTICALBLANKINGO;

        input IFPLMMISDP3EXTSDPREQI;
        output IFPLMMISDP3EXTSDPACKO;
        input IFPLMMISDP3EXTSDPDATAI[72];
        output IFPLMMISDP3EXTSDPLINECNTMAT[2];
        output IFPLMMISDP3EXTSDPHORIZONTALBLANKINGO;
        output IFPLMMISDP3EXTSDPVERTICALBLANKINGO;

        for i in 0..2 {
            output "IFMMIPLVIDEOS0P{i}DATAENI";
            output "IFMMIPLVIDEOS0P{i}HSYNCI";
            output "IFMMIPLVIDEOS0P{i}VDATAALPHAI"[8];
            output "IFMMIPLVIDEOS0P{i}VDATARGBI"[36];
            output "IFMMIPLVIDEOS0P{i}VSYNCI";
        }

        for i in 0..4 {
            input "IFPLMMIVIDEOS0P{i}DATAENI";
            input "IFPLMMIVIDEOS0P{i}HSYNCI";
            input "IFPLMMIVIDEOS0P{i}VSYNCI";
            input "IFPLMMIVIDEOS0P{i}VDATAALPHAI"[8];
            input "IFPLMMIVIDEOS0P{i}VDATARGBI"[36];
        }
        for i in 0..2 {
            input "IFPLMMIVIDEOS1P{i}DATAENI";
            input "IFPLMMIVIDEOS1P{i}HSYNCI";
            input "IFPLMMIVIDEOS1P{i}VSYNCI";
            input "IFPLMMIVIDEOS1P{i}VDATAALPHAI"[8];
            input "IFPLMMIVIDEOS1P{i}VDATARGBI"[36];
        }
        input IFPLMMIVIDEOS2P0DATAENI;
        input IFPLMMIVIDEOS2P0HSYNCI;
        input IFPLMMIVIDEOS2P0VSYNCI;
        input IFPLMMIVIDEOS2P0VDATAALPHAI[8];
        input IFPLMMIVIDEOS2P0VDATARGBI[36];
        input IFPLMMIVIDEOS3P0DATAENI;
        input IFPLMMIVIDEOS3P0HSYNCI;
        input IFPLMMIVIDEOS3P0VSYNCI;
        input IFPLMMIVIDEOS3P0VDATAALPHAI[8];
        input IFPLMMIVIDEOS3P0VDATARGBI[36];

        // Ethernet

        input PLGEMRXCLK;
        input PLGEMTXCLK;
        output GEMPLRXCLK;
        output GEMPLTXCLK;
        input GEMEXTINTERRUPTIN;

        output IFPLMMIGEM1588PTPDELAYREQRX;
        output IFPLMMIGEM1588PTPDELAYREQTX;
        output IFPLMMIGEM1588PTPPDELAYREQRX;
        output IFPLMMIGEM1588PTPPDELAYREQTX;
        output IFPLMMIGEM1588PTPPDELAYRESPRX;
        output IFPLMMIGEM1588PTPPDELAYRESPTX;
        output IFPLMMIGEM1588PTPSOFRX;
        output IFPLMMIGEM1588PTPSOFTX;
        output IFPLMMIGEM1588PTPSYNCFRAMERX;
        output IFPLMMIGEM1588PTPSYNCFRAMETX;

        output IFPLMMIGEMEXTFIFOADDMATCHVEC[4];
        output IFPLMMIGEMEXTFIFODMATXENDTOG;
        input IFPLMMIGEMEXTFIFODMATXSTATUSTOG;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ0;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ1;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ2;
        output IFPLMMIGEMEXTFIFORXDATABUFWRQ3;
        output IFPLMMIGEMEXTFIFORXWDATA[64];
        output IFPLMMIGEMEXTFIFORXWEOP;
        output IFPLMMIGEMEXTFIFORXWERR;
        output IFPLMMIGEMEXTFIFORXWFLUSH;
        output IFPLMMIGEMEXTFIFORXWMOD[4];
        input IFPLMMIGEMEXTFIFORXWOVERFLOW;
        output IFPLMMIGEMEXTFIFORXWQUEUE[4];
        output IFPLMMIGEMEXTFIFORXWSOP;
        output IFPLMMIGEMEXTFIFORXWSTATUS[45];
        output IFPLMMIGEMEXTFIFORXWTIMESTAMP[78];
        output IFPLMMIGEMEXTFIFORXWWR;
        input IFPLMMIGEMEXTFIFOTXRCONTROL;
        input IFPLMMIGEMEXTFIFOTXRDATARDY[4];
        input IFPLMMIGEMEXTFIFOTXRDATA[64];
        input IFPLMMIGEMEXTFIFOTXREOP;
        input IFPLMMIGEMEXTFIFOTXREOPEARLY;
        input IFPLMMIGEMEXTFIFOTXRERR;
        input IFPLMMIGEMEXTFIFOTXRFLUSHED;
        input IFPLMMIGEMEXTFIFOTXRFRAMESIZEVLD[4];
        input IFPLMMIGEMEXTFIFOTXRFRAMESIZE[56];
        input IFPLMMIGEMEXTFIFOTXRMOD[4];
        output IFPLMMIGEMEXTFIFOTXRQUEUE[4];
        output IFPLMMIGEMEXTFIFOTXRRD[4];
        input IFPLMMIGEMEXTFIFOTXRSOP;
        output IFPLMMIGEMEXTFIFOTXRSTATUS[4];
        output IFPLMMIGEMEXTFIFOTXRTIMESTAMP[78];
        input IFPLMMIGEMEXTFIFOTXRUNDERFLOW;
        input IFPLMMIGEMEXTFIFOTXRVALID;

        input MMIPLMMITSUCLK;
        input MMIPLMMITSUINCCTRL[2];
        output MMIMMIPLTSUCLK;
        output MMIMMIPLTSUTIMERCMPVAL;
        output MMIMMIPLTSUTIMERCNT[94];

        // Misc

        output IFMMIPLISRCORREVENT;
        output IFMMIPLISRMISCEVENT;
        output IFMMIPLISRUNCORREVENT;

        input PLMMIGPI[16];
        output MMIPLGPO[32];
        input PLMMIIRQ0;
        input PLMMIIRQ1;

        input MMIPLMMIPTMCLK;
        input MMIPLMMIPTMLOAD;

        input PLMMIDEBUGCLK;
        output MMIPLDEBUGCLK;
        input PLMMISPARE[32];
        output MMIPLSPARE0;
        output MMIPLSPARE1;
        output MMIPLSPARE2;
        output MMIPLSPARE3;
        output MMIPLSPARE4[2];
        output MMIPLSPARE5[2];
        output MMIPLSPARE6[4];
        output MMIPLSPARE7[4];
        output MMIPLSPARE[16];

        input PLMMILOOPBACKEN;
    }

    bel_class HNICX {
        // special clock out

        output SLV_CLK_HNIC;

        input FHX_MAC_FLOW_CTRL_CLK;

        output FHX_MAC_STATUS_CLK_OUT; // clock out
        output FHX_MAC_STATUS_VECTOR[512];

        output FHX_BCS_ID_REQ_STALL;
        input FHX_BCS_LU_REQ_STALL;
        output FHX_BCS_LU_RESP_STALL;

        input FHX_CCS1_REQ_STALL;
        output FHX_CCS1_RESP_STALL;
        input FHX_CCS2_REQ_STALL;
        output FHX_CCS2_RESP_STALL;

        output FHX_SCS_ID_REQ_STALL;
        input FHX_SCS_LU_REQ_STALL;
        output FHX_SCS_LU_RESP_STALL;

        input FHX_DBG_TRIG_IN;

        input FHX_LCS_CACHE_MISS_CLK;
        input FHX_ONE_SEC_PULSE_IN;
        output FHX_ONE_SEC_PULSE_OUT;

        input FHX_TLPBK_SEL_HUB_0;
        input FHX_TLPBK_SEL_HUB_1;
        input FHX_TLPBK_SEL_HUB_2;
        input FHX_TLPBK_SEL_HUB_3;
        input FHX_TLPBK_SEL_LCS_0;
        input FHX_TLPBK_SEL_LCS_1;
        input FHX_TLPBK_SEL_LCS_2;
        input FHX_TLPBK_SEL_NMAC_0;
        input FHX_TLPBK_SEL_NMAC_1;
        input FHX_TLPBK_SEL_NMAC_2;
        input FHX_TLPBK_SEL_VNX_0;
        input FHX_TLPBK_SEL_VNX_1;
        input FHX_TLPBK_SEL_VNX_2;

        input FHX_DPU_CMD_CREDIT_3_0_CLK;
        input FHX_DPU_CMD_CREDIT_7_4_CLK;
        input FHX_DPU_CMD_CREDIT_11_8_CLK;
        for i in 0..12 {
            output "FHX_DPU_COMMAND_{i}_CREDIT"[8];
            output "FHX_DPU_DATA_{i}_CREDIT"[8];
        }
        input FHX_DPU_ING_CMD_CH0_CH3_CLK;
        input FHX_DPU_ING_CMD_CH4_CH7_CLK;
        input FHX_DPU_ING_CMD_CH8_CH9_CLK;
        input FHX_DPU_ING_CMD_CH10_CH11_CLK;
        for i in 0..12 {
            output "FHX_DPU_ING_CMD_CH{i}_TAF";
            input "FHX_DPU_ING_CMD_CH{i}_TDATA"[256];
            input "FHX_DPU_ING_CMD_CH{i}_TID"[2];
            input "FHX_DPU_ING_CMD_CH{i}_TLAST";
            input "FHX_DPU_ING_CMD_CH{i}_TVALID";
        }
        input FHX_DPU_ING_DATA_CH0_CH3_CLK;
        input FHX_DPU_ING_DATA_CH4_CH7_CLK;
        input FHX_DPU_ING_DATA_CH8_CH11_CLK;
        input FHX_DPU_ING_DATA_CH12_CH15_CLK;
        for i in 0..16 {
            output "FHX_DPU_ING_DATA_CH{i}_TAF";
            input "FHX_DPU_ING_DATA_CH{i}_TDATA"[256];
            input "FHX_DPU_ING_DATA_CH{i}_TLAST";
            input "FHX_DPU_ING_DATA_CH{i}_TVALID";
        }

        input FHX_FCI_CLK;
        output FHX_FCI_A_REQ_AF;
        input FHX_FCI_A_REQ_VLD;
        input FHX_FCI_A_REQ[632];
        output FHX_FCI_B_REQ_AF;
        input FHX_FCI_B_REQ_VLD;
        input FHX_FCI_B_REQ[480];

        for i in 0..2 {
            // HMAE

            input "FHX_P2HMAE{i}_CLK";

            output "FHX_P2HMAE{i}_ICSB_AF";
            input "FHX_P2HMAE{i}_ICSB_VALID";
            input "FHX_P2HMAE{i}_ICSB_DATA"[1024];
            input "FHX_P2HMAE{i}_ICSB_CH_VC"[2];
            input "FHX_P2HMAE{i}_ICSB_EOP";
            input "FHX_P2HMAE{i}_ICSB_ERR"[2];
            input "FHX_P2HMAE{i}_ICSB_MTY"[7];
            input "FHX_P2HMAE{i}_ICSB_SOP";

            output "FHX_P2HMAE{i}_RD_NOTIFY_VLD";
            output "FHX_P2HMAE{i}_RD_NOTIFY"[6];

            input "FHX_HMAE2P{i}_CLK";

            output "FHX_HMAE2P{i}_DST_CRDT_MSG_AF";
            input "FHX_HMAE2P{i}_DST_CRDT_MSG_VLD";
            input "FHX_HMAE2P{i}_DST_CRDT_MSG"[22];

            input "FHX_HMAE2P{i}_ICSB_AF";
            output "FHX_HMAE2P{i}_ICSB_VALID";
            output "FHX_HMAE2P{i}_ICSB_DATA"[1024];
            output "FHX_HMAE2P{i}_ICSB_EOP";
            output "FHX_HMAE2P{i}_ICSB_ERR"[2];
            output "FHX_HMAE2P{i}_ICSB_MTY"[7];
            output "FHX_HMAE2P{i}_ICSB_SOP";

            // NMAE

            input "FHX_P2NMAE{i}_CLK";

            output "FHX_P2NMAE{i}_ICSB_AF";
            input "FHX_P2NMAE{i}_ICSB_VALID";
            input "FHX_P2NMAE{i}_ICSB_DATA"[1024];
            input "FHX_P2NMAE{i}_ICSB_CH_VC"[2];
            input "FHX_P2NMAE{i}_ICSB_EOP";
            input "FHX_P2NMAE{i}_ICSB_ERR"[2];
            input "FHX_P2NMAE{i}_ICSB_MTY"[7];
            input "FHX_P2NMAE{i}_ICSB_SOP";

            output "FHX_P2NMAE{i}_RD_NOTIFY_VLD";
            output "FHX_P2NMAE{i}_RD_NOTIFY"[6];

            input "FHX_NMAE2P{i}_CLK";

            output "FHX_NMAE2P{i}_DST_CRDT_MSG_AF";
            input "FHX_NMAE2P{i}_DST_CRDT_MSG_VLD";
            input "FHX_NMAE2P{i}_DST_CRDT_MSG"[22];

            input "FHX_NMAE2P{i}_ICSB_AF";
            output "FHX_NMAE2P{i}_ICSB_VALID";
            output "FHX_NMAE2P{i}_ICSB_DATA"[1024];
            output "FHX_NMAE2P{i}_ICSB_EOP";
            output "FHX_NMAE2P{i}_ICSB_ERR"[2];
            output "FHX_NMAE2P{i}_ICSB_MTY"[7];
            output "FHX_NMAE2P{i}_ICSB_SOP";

            // VNR

            input "FHX_P2VNR{i}_CLK";

            output "FHX_P2VNR{i}_ICSB_AF";
            input "FHX_P2VNR{i}_ICSB_VALID";
            input "FHX_P2VNR{i}_ICSB_DATA"[1024];
            input "FHX_P2VNR{i}_ICSB_CH_VC"[2];
            input "FHX_P2VNR{i}_ICSB_EOP";
            input "FHX_P2VNR{i}_ICSB_ERR"[2];
            input "FHX_P2VNR{i}_ICSB_MTY"[7];
            input "FHX_P2VNR{i}_ICSB_SOP";

            output "FHX_P2VNR{i}_RD_NOTIFY_VLD";
            output "FHX_P2VNR{i}_RD_NOTIFY"[6];

            input "FHX_VNR2P{i}_CLK";

            output "FHX_VNR2P{i}_DST_CRDT_MSG_AF";
            input "FHX_VNR2P{i}_DST_CRDT_MSG_VLD";
            input "FHX_VNR2P{i}_DST_CRDT_MSG"[22];

            input "FHX_VNR2P{i}_ICSB_AF";
            output "FHX_VNR2P{i}_ICSB_VALID";
            output "FHX_VNR2P{i}_ICSB_DATA"[1024];
            output "FHX_VNR2P{i}_ICSB_EOP";
            output "FHX_VNR2P{i}_ICSB_ERR"[2];
            output "FHX_VNR2P{i}_ICSB_MTY"[7];
            output "FHX_VNR2P{i}_ICSB_SOP";

            // VNT

            input "FHX_P2VNT{i}_CLK";

            output "FHX_P2VNT{i}_ICSB_AF";
            input "FHX_P2VNT{i}_ICSB_VALID";
            input "FHX_P2VNT{i}_ICSB_DATA"[1024];
            input "FHX_P2VNT{i}_ICSB_CH_VC"[2];
            input "FHX_P2VNT{i}_ICSB_EOP";
            input "FHX_P2VNT{i}_ICSB_ERR"[2];
            input "FHX_P2VNT{i}_ICSB_MTY"[7];
            input "FHX_P2VNT{i}_ICSB_SOP";

            output "FHX_P2VNT{i}_RD_NOTIFY_VLD";
            output "FHX_P2VNT{i}_RD_NOTIFY"[6];

            input "FHX_VNT2P{i}_CLK";

            output "FHX_VNT2P{i}_DST_CRDT_MSG_AF";
            input "FHX_VNT2P{i}_DST_CRDT_MSG_VLD";
            input "FHX_VNT2P{i}_DST_CRDT_MSG"[22];

            input "FHX_VNT2P{i}_ICSB_AF";
            output "FHX_VNT2P{i}_ICSB_VALID";
            output "FHX_VNT2P{i}_ICSB_DATA"[1024];
            output "FHX_VNT2P{i}_ICSB_EOP";
            output "FHX_VNT2P{i}_ICSB_ERR"[2];
            output "FHX_VNT2P{i}_ICSB_MTY"[7];
            output "FHX_VNT2P{i}_ICSB_SOP";
        }

        input FHX_M_AXIS_A2FAB_TAF;
        output FHX_M_AXIS_A2FAB_TVALID;
        output FHX_M_AXIS_A2FAB_TDATA[256];
        output FHX_M_AXIS_A2FAB_TDEST[3];
        output FHX_M_AXIS_A2FAB_TLAST;

        output FHX_S_AXIS_FAB2A_TAF;
        input FHX_S_AXIS_FAB2A_TVALID;
        input FHX_S_AXIS_FAB2A_TDATA[256];
        input FHX_S_AXIS_FAB2A_TDEST[3];
        input FHX_S_AXIS_FAB2A_TLAST;

        for i in 0..2 {
            input "FHX_MAC_RX_CLK{i}";

            output "FHX_RX_AXIS_TVALID{i}";
            for j in 0..4 {
                output "FHX_RX_AXIS_TDATA{j}{i}"[128];
                output "FHX_RX_AXIS_TUSER_ENA{j}{i}";
                output "FHX_RX_AXIS_TUSER_EOP{j}{i}";
                output "FHX_RX_AXIS_TUSER_ERR{j}{i}";
                output "FHX_RX_AXIS_TUSER_MTY{j}{i}"[4];
                output "FHX_RX_AXIS_TUSER_SOP{j}{i}";
            }
            output "FHX_RX_AXIS_TID{i}";

            output "FHX_RX_PREAMBLEOUT{i}"[56];
            output "FHX_RX_PTP_TSTAMP_OUT{i}"[32];
        }

        for i in 0..9 {
            output "FHX_RX_PAUSE_REQ_{i}";
            output "FHX_RX_PAUSE_QUANTA_{i}"[16];
        }
        for i in 0..2 {
            input "FHX_RX_PAUSE_ACK{i}"[9];
            input "FHX_RX_PAUSE_ACK_ID{i}";
        }
        output FHX_RX_PAUSE_REQ_ID[2];
        output FHX_RX_PAUSE_VALID[9];
        output FHX_RX_PAUSE_VALID_ID[2];

        for i in 0..2 {
            input "FHX_MAC_TX_CLK{i}";

            output "FHX_TX_AXIS_ALMOST_FULL0{i}";
            output "FHX_TX_AXIS_ALMOST_FULL1{i}";
            input "FHX_TX_AXIS_TVALID{i}";
            for j in 0..4 {
                input "FHX_TX_AXIS_TDATA{j}{i}"[128];
                input "FHX_TX_AXIS_TUSER_ENA{j}{i}";
                input "FHX_TX_AXIS_TUSER_EOP{j}{i}";
                input "FHX_TX_AXIS_TUSER_ERR{j}{i}";
                input "FHX_TX_AXIS_TUSER_MTY{j}{i}"[4];
                input "FHX_TX_AXIS_TUSER_SOP{j}{i}";
            }
            input "FHX_TX_AXIS_TID{i}";

            input "FHX_TX_PREAMBLEIN{i}"[56];
            input "FHX_TX_PTP_1588OP_IN{i}"[2];
            input "FHX_TX_PTP_CF_OFFSET_IN{i}"[14];
            input "FHX_TX_PTP_TAG_FIELD_IN{i}"[8];
            output "FHX_TX_PTP_TSTAMP{i}"[32];
            output "FHX_TX_PTP_TSTAMP_PORT_ID{i}"[2];
            output "FHX_TX_PTP_TSTAMP_TAG{i}"[8];
            output "FHX_TX_PTP_TSTAMP_VALID{i}";
            input "FHX_TX_PTP_UDP_CHKSUM_IN{i}";
        }

        output FHX_TX_PAUSE_PACKET_SENT[9];
        output FHX_TX_PAUSE_PACKET_SENT_ID[2];
        input FHX_TX_PAUSE_REQ_0[9];
        input FHX_TX_PAUSE_REQ_1[9];
        input FHX_TX_PAUSE_REQ_2[9];
        input FHX_TX_PAUSE_REQ_3[9];
        input FHX_TX_RESEND_PAUSE_0;
        input FHX_TX_RESEND_PAUSE_1;
        input FHX_TX_RESEND_PAUSE_2;
        input FHX_TX_RESEND_PAUSE_3;

        // special clk out
        output TEST_PLL_CLK_OUT[2];
        output TEST_PLL_OUT[16];

        input FHX_HNICX_CSSD_TRIG_IN_N_EXT;

        input FHX_F2C_SCAN_R1_CLK_N_EXT;
        input FHX_F2C_SCAN_R1_EDT_UPDT_N_EXT;
        input FHX_F2C_SCAN_R1_EN_N_EXT;
        input FHX_F2C_SCAN_R1_MODE_RST_N_EXT;
        input FHX_F2C_SCAN_R2_CLK_N_EXT;
        input FHX_F2C_SCAN_R2_EDT_UPDT_N_EXT;
        input FHX_F2C_SCAN_R2_EN_N_EXT;
        input FHX_F2C_SCAN_R2_MODE_RST_N_EXT;
        input FHX_F2C_SCAN_R3_CLK_N_EXT;
        input FHX_F2C_SCAN_R3_EDT_UPDT_N_EXT;
        input FHX_F2C_SCAN_R3_EN_N_EXT;
        input FHX_F2C_SCAN_R3_MODE_RST_N_EXT;
        input FHX_F2C_SCAN_R4_CLK_N_EXT;
        input FHX_F2C_SCAN_R4_EDT_UPDT_N_EXT;
        input FHX_F2C_SCAN_R4_EN_N_EXT;
        input FHX_F2C_SCAN_R4_MODE_RST_N_EXT;

        input FHX_F2C_DPU_SCAN_CHNL_IN_EXT[100];
        input FHX_F2C_DPU_SCAN_CHNL_MASK_IN_EXT[100];
        input FHX_F2C_DPU_SCAN_CNTRL_CHNL_IN_EXT[8];
        input FHX_F2C_DPU_SCAN_SPARE_IN_EXT[32];
        output FHX_C2F_DPU_SCAN_CHNL_OUT_EXT[100];
        output FHX_C2F_DPU_SCAN_CNTRL_CHNL_OUT_EXT[8];
        output FHX_C2F_DPU_SCAN_SPARE_OUT_EXT[16];

        input FHX_F2C_NIC_HUB_SCAN_CHNL_IN_EXT[12];
        input FHX_F2C_NIC_HUB_SCAN_CHNL_MASK_IN_EXT[12];
        input FHX_F2C_NIC_HUB_SCAN_CNTRL_CHNL_IN_EXT;
        input FHX_F2C_NIC_HUB_SCAN_SPARE_IN_EXT[2];
        output FHX_C2F_NIC_HUB_SCAN_CHNL_OUT_EXT[12];
        output FHX_C2F_NIC_HUB_SCAN_CNTRL_CHNL_OUT_EXT;
        output FHX_C2F_NIC_HUB_SCAN_SPARE_OUT_EXT;

        input FHX_F2C_NIC_LCS_SCAN_CHNL_IN_EXT[26];
        input FHX_F2C_NIC_LCS_SCAN_CHNL_MASK_IN_EXT[26];
        input FHX_F2C_NIC_LCS_SCAN_CNTRL_CHNL_IN_EXT[2];
        input FHX_F2C_NIC_LCS_SCAN_SPARE_IN_EXT[4];
        output FHX_C2F_NIC_LCS_SCAN_CHNL_OUT_EXT[26];
        output FHX_C2F_NIC_LCS_SCAN_CNTRL_CHNL_OUT_EXT[2];
        output FHX_C2F_NIC_LCS_SCAN_SPARE_OUT_EXT[2];

        input FHX_F2C_NIC_MAE_1_SCAN_CHNL_IN_EXT[33];
        input FHX_F2C_NIC_MAE_1_SCAN_CHNL_MASK_IN_EXT[33];
        input FHX_F2C_NIC_MAE_1_SCAN_CNTRL_CHNL_IN_EXT[3];
        input FHX_F2C_NIC_MAE_1_SCAN_SPARE_IN_EXT[6];
        output FHX_C2F_NIC_MAE_1_SCAN_CHNL_OUT_EXT[33];
        output FHX_C2F_NIC_MAE_1_SCAN_CNTRL_CHNL_OUT_EXT[3];
        output FHX_C2F_NIC_MAE_1_SCAN_SPARE_OUT_EXT[3];

        input FHX_F2C_NIC_MAE_2_SCAN_CHNL_IN_EXT[33];
        input FHX_F2C_NIC_MAE_2_SCAN_CHNL_MASK_IN_EXT[33];
        input FHX_F2C_NIC_MAE_2_SCAN_CNTRL_CHNL_IN_EXT[3];
        input FHX_F2C_NIC_MAE_2_SCAN_SPARE_IN_EXT[6];
        output FHX_C2F_NIC_MAE_2_SCAN_CHNL_OUT_EXT[33];
        output FHX_C2F_NIC_MAE_2_SCAN_CNTRL_CHNL_OUT_EXT[3];
        output FHX_C2F_NIC_MAE_2_SCAN_SPARE_OUT_EXT[3];

        input FHX_F2C_NIC_NETHUB_SCAN_CHNL_IN_EXT[7];
        input FHX_F2C_NIC_NETHUB_SCAN_CHNL_MASK_IN_EXT[7];
        input FHX_F2C_NIC_NETHUB_SCAN_CNTRL_CHNL_IN_EXT;
        input FHX_F2C_NIC_NETHUB_SCAN_SPARE_IN_EXT[2];
        output FHX_C2F_NIC_NETHUB_SCAN_CHNL_OUT_EXT[7];
        output FHX_C2F_NIC_NETHUB_SCAN_CNTRL_CHNL_OUT_EXT;
        output FHX_C2F_NIC_NETHUB_SCAN_SPARE_OUT_EXT;

        input FHX_F2C_NIC_NICMAC_SCAN_CHNL_IN_EXT[26];
        input FHX_F2C_NIC_NICMAC_SCAN_CHNL_MASK_IN_EXT[26];
        input FHX_F2C_NIC_NICMAC_SCAN_CNTRL_CHNL_IN_EXT[2];
        input FHX_F2C_NIC_NICMAC_SCAN_SPARE_IN_EXT[8];
        output FHX_C2F_NIC_NICMAC_SCAN_CHNL_OUT_EXT[26];
        output FHX_C2F_NIC_NICMAC_SCAN_CNTRL_CHNL_OUT_EXT[2];
        output FHX_C2F_NIC_NICMAC_SCAN_SPARE_OUT_EXT[4];

        input FHX_F2C_NIC_VNX_PLL_SCAN_CHNL_IN_EXT;
        output FHX_C2F_NIC_VNX_PLL_SCAN_CHNL_OUT_EXT;

        input FHX_F2C_NIC_VNX_SCAN_CHNL_IN_EXT[30];
        input FHX_F2C_NIC_VNX_SCAN_CHNL_MASK_IN_EXT[30];
        input FHX_F2C_NIC_VNX_SCAN_CNTRL_CHNL_IN_EXT[2];
        input FHX_F2C_NIC_VNX_SCAN_SPARE_IN_EXT[4];
        output FHX_C2F_NIC_VNX_SCAN_CHNL_OUT_EXT[30];
        output FHX_C2F_NIC_VNX_SCAN_CNTRL_CHNL_OUT_EXT[2];
        output FHX_C2F_NIC_VNX_SCAN_SPARE_OUT_EXT[2];
    }

    bel_class AIE_TILECTRL {
    }

    bel_class AIE_FIFO {
    }

    bel_class AIE_ARRAYSWITCH {
    }

    bel_class AIE_ML_ARRAYSWITCH {
    }

    bel_class AIE_ML_MEM_ARRAYSWITCH {
    }

    bel_class AIE_CORE {
    }

    bel_class AIE_ML_CORE {
    }

    bel_class AIE_MEMGRP {
    }

    bel_class AIE_ML_MEM_MEMGRP {
    }

    bel_class AIE_SHIMSWITCH {
    }

    bel_class AIE_ML_SHIMSWITCH {
    }

    bel_class AIE2PS_SHIMSWITCH {
    }

    bel_class AIE_SHIMTRACE {
    }

    bel_class AIE2PS_SHIM_CNTRL {
    }

    bel_class AIE2PS_NMU_ARBIT {
    }

    bel_class AIE_PLL {
        input PL_CLK;
        input REF_CLK;
        output AIE_PLL_TEST_CLK_OUT[2];
        output AIE_PLL_TEST_OUT[16];
    }

    bel_class AIE_PL {
        for i in 0..6 {
            input "M_CLK_{i}";
            input "M_TREADY_{i}";
            output "M_TVALID_{i}";
            output "M_TDATA_{i}"[64];
            output "M_TKEEP_{i}";
            output "M_TLAST_{i}";
        }
        for i in 0..8 {
            input "S_CLK_{i}";
            output "S_TREADY_{i}";
            input "S_TVALID_{i}";
            input "S_TDATA_{i}"[64];
            input "S_TKEEP_{i}";
            input "S_TLAST_{i}";
        }
        for i in 0..2 {
            input "TRIG_{i}_IN_CLK";
            input "TRIG_{i}_IN"[16];
            input "TRIG_{i}_OUT_CLK";
            output "TRIG_{i}_OUT"[16];
        }
    }

    bel_class AIE_NOC {
        output M_AXI_ACLK;
        output S_AXI_ACLK;
    }

    bel_class AIE_INTF_TEST {
        input PL_AIE_SCAN_PLL_RESET;
        input PL_AIE_SCAN_CLKDIV_RSTN;
        input PL_AIE_SCAN_CLK_N;
        input PL_AIE_SCAN_MODE_RST_N;
        input PL_AIE_SCAN_EN_N;
        input PL_AIE_SCAN_EDT_UPDT_N;
        input PL_AIE_SCAN_CHNL_IN[5];
        output AIE_PL_SCAN_CHNL_OUT[33];
        input PL_AIE_SCAN_SPARE_IN[2];
        output AIE_PL_SCAN_SPARE_OUT[2];
    }

    bel_class AIE_ML_INTF_TEST {
        input PL_AIE_SCAN_CLK_N;
        input PL_AIE_SCAN_MODE_RST_N;
        input PL_AIE_SCAN_EN_N;
        input PL_AIE_SCAN_EDT_UPDT_N;
        input PL_AIE_SCAN_CHNL_IN[9];
        output AIE_PL_SCAN_CHNL_OUT[35];
    }

    bel_class AIE2PS_INTF_TEST {
        input PL_AIE_SCAN_CLK_N;
        input PL_AIE_SCAN_MODE_RST_N;
        input PL_AIE_SCAN_EN_N;
        input PL_AIE_SCAN_EDT_UPDT_N;
        input PL_AIE_SCAN_CHNL_IN[32];
        output AIE_PL_SCAN_CHNL_OUT[12];
    }

    bel_class NOC_HBM_BLI_SCAN {
        for i in 0..3 {
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CLK_N";
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_MODE_RST_N";
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_EN_N";
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_EDT_UPDT_N";
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CSSD_TRIG_N";
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CHNL_IN"[4];
            output "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CHNL_OUT"[4];
            input "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CNTRL_CHNL_IN";
            output "IF_HBM_BLI_DFX_IN{i}_DFX_SCAN_CNTRL_CHNL_OUT";
        }
    }

    bel_class NOC_NMU_HBM2E {
        input CLK;

        input NMU_USR_INTERRUPT_IN[2];

        input IF_NOC_AXI_TOP_RD_DEST_MODE;
        input IF_NOC_AXI_TOP_RD_USR_DST[3];

        output IF_NOC_AXI_TOP_ARREADY;
        input IF_NOC_AXI_TOP_ARVALID;
        input IF_NOC_AXI_TOP_ARADDR[48];
        input IF_NOC_AXI_TOP_ARADDR_PAR[2];
        input IF_NOC_AXI_TOP_ARBURST[2];
        input IF_NOC_AXI_TOP_ARCACHE[4];
        input IF_NOC_AXI_TOP_ARID[7];
        input IF_NOC_AXI_TOP_ARLEN[8];
        input IF_NOC_AXI_TOP_ARLOCK;
        input IF_NOC_AXI_TOP_ARPROT[3];
        input IF_NOC_AXI_TOP_ARSIZE[3];
        input IF_NOC_AXI_TOP_ARUSER[11];

        input IF_NOC_AXI_TOP_WR_DEST_MODE;
        input IF_NOC_AXI_TOP_WR_USR_DST[3];

        output IF_NOC_AXI_TOP_AWREADY;
        input IF_NOC_AXI_TOP_AWVALID;
        input IF_NOC_AXI_TOP_AWADDR[48];
        input IF_NOC_AXI_TOP_AWADDR_PAR[2];
        input IF_NOC_AXI_TOP_AWBURST[2];
        input IF_NOC_AXI_TOP_AWCACHE[4];
        input IF_NOC_AXI_TOP_AWID[7];
        input IF_NOC_AXI_TOP_AWLEN[8];
        input IF_NOC_AXI_TOP_AWLOCK;
        input IF_NOC_AXI_TOP_AWPROT[3];
        input IF_NOC_AXI_TOP_AWSIZE[3];
        input IF_NOC_AXI_TOP_AWUSER[11];

        input IF_NOC_AXI_TOP_BREADY;
        output IF_NOC_AXI_TOP_BVALID;
        output IF_NOC_AXI_TOP_BID[7];
        output IF_NOC_AXI_TOP_BRESP[2];
        output IF_NOC_AXI_TOP_BUSER[16];

        input IF_NOC_AXI_TOP_RREADY;
        output IF_NOC_AXI_TOP_RVALID;
        output IF_NOC_AXI_TOP_RDATA[256];
        output IF_NOC_AXI_TOP_RDATA_PAR[8];
        output IF_NOC_AXI_TOP_RID[7];
        output IF_NOC_AXI_TOP_RLAST;
        output IF_NOC_AXI_TOP_RPOISON;
        output IF_NOC_AXI_TOP_RRESP[2];

        output IF_NOC_AXI_TOP_WREADY;
        input IF_NOC_AXI_TOP_WDATA[256];
        input IF_NOC_AXI_TOP_WDATA_PAR[8];
        input IF_NOC_AXI_TOP_WLAST;
        input IF_NOC_AXI_TOP_WPOISON;
        input IF_NOC_AXI_TOP_WSTRB[32];
        input IF_NOC_AXI_TOP_WVALID;

        output IF_BLI_PHY_DIR_IN_PHY2BLI[51];
        input IF_BLI_PHY_DIR_IN_SPARE_CTRL[4];
    }

    bel_class HBM_MS {
        input CCIO_REF_CLK;
        output CLKOUT_CCIO, CLKOUT, CLKOUTPHY, TMUXOUT;
        input CLKOUTPHY_EN;
        input PWRDWN;
        input DLL_TEST_IN[5];
        output DLL_TEST_OUT[24];
        input PLL_TEST_IN[8];
        output PLL_TEST_OUT[4];
    }

    bel_class HBM_CHNL {
        output DFI_CLK_MC2PHY;
        output DFI_CLK_NOC2PHY;
    }

    bel_class NOC_NPS4 {
    }

    bel_class NOC_NPS6 {
    }

    region_slot CLE;
    region_slot LEAF;
    region_slot HDISTR_LOC;
    region_slot HDISTR;
    region_slot HROUTE;
    region_slot GCLK_PD_H;
    region_slot GCLK_PD_V;
    region_slot PS;
    region_slot CPM;
    region_slot HSM;
    region_slot XPIO_LEAF;
    region_slot HSR_SSIT2_H;

    // wires common between two halves of the interconnect tile

    wire TIE_1: tie 1;

    wire X6_W0[8]: mux;
    for i in 1..=3 {
        wire "X6_W{i}"[8]: branch LE;
    }
    wire X6_E0[8]: mux;
    for i in 1..=3 {
        wire "X6_E{i}"[8]: branch LW;
    }
    wire X7_S0[8]: mux;
    for i in 1..=7 {
        wire "X7_S{i}"[8]: branch N;
    }
    wire X7_N0[8]: mux;
    for i in 1..=7 {
        wire "X7_N{i}"[8]: branch S;
    }
    wire X6_E3_S0: branch N;

    wire X10_W0[8]: mux;
    for i in 1..=5 {
        wire "X10_W{i}"[8]: branch LE;
    }
    wire X10_E0[8]: mux;
    for i in 1..=5 {
        wire "X10_E{i}"[8]: branch LW;
    }
    wire X12_S0[8]: mux;
    for i in 1..=12 {
        wire "X12_S{i}"[8]: branch N;
    }
    wire X12_N0[8]: mux;
    for i in 1..=12 {
        wire "X12_N{i}"[8]: branch S;
    }
    wire X10_E5_N7: branch S;
    wire X10_E5_E7: branch W;

    wire IMUX_LAG[6]: mux;
    wire OUT_LAG[6]: regional CLE;

    // wires belonging to interconnect left/right half-tiles

    wire OUT[48]: bel;
    // only a few actually exist
    wire OUT_S[48]: branch N;
    wire OUT_BEL[48]: bel;
    wire OUT_TEST[48]: test;
    wire OUT_CLE[48]: branch INTF;

    wire SDQNODE[128]: mux;
    wire SDQNODE_S[128]: branch N;
    wire SDQNODE_N[128]: branch S;

    wire X1_W0[16]: mux;
    wire X1_W1[16]: branch E;
    wire X1_W0_S0: branch N;
    wire X1_E0[16]: mux;
    wire X1_E1[16]: branch W;
    wire X1_S0[16]: mux;
    wire X1_S1[16]: branch N;
    wire X1_N0[16]: mux;
    wire X1_N1[16]: branch S;

    wire X2_W0[8]: mux;
    wire X2_W1[8]: branch E;
    wire X2_W2[8]: branch E;
    wire X2_E0[8]: mux;
    wire X2_E1[8]: branch W;
    wire X2_E2[8]: branch W;
    wire X2_S0[8]: mux;
    wire X2_S1[8]: branch N;
    wire X2_S2[8]: branch N;
    wire X2_N0[8]: mux;
    wire X2_N1[8]: branch S;
    wire X2_N2[8]: branch S;

    wire X4_W0[8]: mux;
    for i in 1..=4 {
        wire "X4_W{i}"[8]: branch E;
    }
    wire X4_E0[8]: mux;
    for i in 1..=4 {
        wire "X4_E{i}"[8]: branch W;
    }
    wire X4_S0[8]: mux;
    for i in 1..=4 {
        wire "X4_S{i}"[8]: branch N;
    }
    wire X4_N0[8]: mux;
    for i in 1..=4 {
        wire "X4_N{i}"[8]: branch S;
    }
    wire X4_E3_S0: branch N;

    wire INODE[128]: mux;
    wire IMUX_IMUX[96]: mux;
    wire IMUX_BOUNCE[32]: mux;
    wire BNODE[64]: branch INTF;

    wire TEST_TMR_DFT: test;

    wire BNODE_CLE[32]: mux;
    wire CNODE_CLE[12]: mux;
    wire IMUX_CLE_CTRL[13]: mux;
    for i in 0..4 {
        wire "IMUX_BLI_CLE_IRI{i}_FAKE_CE"[4]: tie 0;
    }
    wire GCLK_CLE[16]: regional LEAF;
    wire GCLK_CLE_S[16]: multi_branch CLK_HALF_S;

    for i in 0..4 {
        wire "IMUX_INTF_IRI{i}_CLK": mux;
        wire "IMUX_INTF_IRI{i}_RST": mux;
        wire "IMUX_INTF_IRI{i}_CE"[4]: mux;
    }
    wire CNODE_INTF[24]: mux;
    wire GCLK_INTF[16]: regional LEAF;
    wire GCLK_INTF_S[16]: multi_branch CLK_HALF_S;

    wire INODE_RCLK[40]: mux;
    wire IMUX_RCLK[40]: mux;

    for i in 0..4 {
        wire "IRI{i}_CLK": mux;
        wire "IRI{i}_RST": mux;
        wire "IRI{i}_CE"[4]: mux;
        wire "IRI{i}_IMUX"[24]: mux;
        wire "IRI{i}_IMUX_DELAY"[24]: mux;
    }

    // BLI wires

    wire BLI_A_GRP0_IMUX[16]: mux;
    wire BLI_A_GRP0_OUT_CTRL: mux;
    wire BLI_A_GRP1_OUT[32]: mux;
    wire BLI_A_GRP1_OUT_CTRL[2]: mux;
    wire BLI_A_GRP2_IMUX[17]: mux;
    wire BLI_A_GRP3_IMUX_CTRL[3]: mux;
    wire BLI_B_GRP0_IMUX[16]: mux;
    wire BLI_B_GRP0_OUT_CTRL: mux;
    wire BLI_B_GRP1_OUT[32]: mux;
    wire BLI_B_GRP1_OUT_CTRL[2]: mux;
    wire BLI_B_GRP2_IMUX[17]: mux;
    wire BLI_B_GRP3_IMUX_CTRL[3]: mux;
    wire BLI_C_GRP0_IMUX[16]: mux;
    wire BLI_C_GRP0_OUT_CTRL: mux;
    wire BLI_C_GRP1_OUT[32]: mux;
    wire BLI_C_GRP1_OUT_CTRL[2]: mux;
    wire BLI_C_GRP2_IMUX[17]: mux;
    wire BLI_C_GRP3_IMUX_CTRL[3]: mux;
    wire BLI_D_GRP4_IMUX[4]: mux;
    wire BLI_D_GRP4_IMUX_BYP[12]: mux;
    wire BLI_D_GRP4_OUT[4]: mux;
    wire BLI_D_GRP5_IMUX[4]: mux;
    wire BLI_D_GRP5_IMUX_BYP[12]: mux;
    wire BLI_D_GRP5_OUT[4]: mux;
    wire BLI_D_GRP6_IMUX[4]: mux;
    wire BLI_D_GRP6_IMUX_BYP[12]: mux;
    wire BLI_D_GRP6_OUT[4]: mux;
    wire BLI_D_GRP7_IMUX[5]: mux;
    wire BLI_D_GRP7_IMUX_BYP[11]: mux;
    wire BLI_D_GRP7_OUT[5]: mux;

    wire BLI_A_GRP0_OUT_CTRL_BEL: bel;
    wire BLI_A_GRP1_OUT_BEL[32]: bel;
    wire BLI_A_GRP1_OUT_CTRL_BEL[2]: bel;
    wire BLI_B_GRP0_OUT_CTRL_BEL: bel;
    wire BLI_B_GRP1_OUT_BEL[32]: bel;
    wire BLI_B_GRP1_OUT_CTRL_BEL[2]: bel;
    wire BLI_C_GRP0_OUT_CTRL_BEL: bel;
    wire BLI_C_GRP1_OUT_BEL[32]: bel;
    wire BLI_C_GRP1_OUT_CTRL_BEL[2]: bel;
    wire BLI_D_GRP4_OUT_BEL[4]: bel;
    wire BLI_D_GRP5_OUT_BEL[4]: bel;
    wire BLI_D_GRP6_OUT_BEL[4]: bel;
    wire BLI_D_GRP7_OUT_BEL[5]: bel;

    wire BLI_A_GRP0_OUT_CTRL_NOC: bel;
    wire BLI_A_GRP1_OUT_NOC[32]: bel;
    wire BLI_A_GRP1_OUT_CTRL_NOC[2]: bel;
    wire BLI_B_GRP0_OUT_CTRL_NOC: bel;
    wire BLI_B_GRP1_OUT_NOC[32]: bel;
    wire BLI_B_GRP1_OUT_CTRL_NOC[2]: bel;
    wire BLI_C_GRP0_OUT_CTRL_NOC: bel;
    wire BLI_C_GRP1_OUT_NOC[32]: bel;
    wire BLI_C_GRP1_OUT_CTRL_NOC[2]: bel;
    wire BLI_D_GRP4_OUT_NOC[4]: bel;
    wire BLI_D_GRP5_OUT_NOC[4]: bel;
    wire BLI_D_GRP6_OUT_NOC[4]: bel;
    wire BLI_D_GRP7_OUT_NOC[5]: bel;

    // clock routing

    wire CLK_HDISTR_LOC[24]: regional HDISTR_LOC;
    wire CLK_HDISTR_LOC_DRV[24]: multi_branch CLK_DRV;
    wire CLK_HDISTR_LOC_DRV_N[24]: branch CLK_N;
    wire CLK_HDISTR_LOC_DRV_S[24]: branch CLK_S;

    wire CLK_HDISTR[24] {
        0..8 => regional HROUTE,
        8..12 => regional HDISTR,
        12..20 => regional HROUTE,
        20..24 => regional HDISTR,
    }
    wire CLK_HDISTR_VOID[24]: mux;
    wire CLK_HDISTR_FAKE[8]: mux;

    wire CLK_HROUTE[12]: regional HROUTE;

    wire CLK_VROUTE[24]: multi_root;
    wire CLK_VROUTE_N[24]: multi_branch CLK_N;
    wire CLK_VDISTR[24]: multi_root;
    wire CLK_VDISTR_N[24]: multi_branch CLK_N;
    wire CLK_VDISTR_S[24]: mux;
    wire CLK_VDISTR_LEV1[24]: multi_root;
    wire CLK_VDISTR_LEV1_N[24]: multi_branch CLK_N;
    wire CLK_VDISTR_LEV2[24]: multi_root;
    wire CLK_VDISTR_LEV2_N[24]: multi_branch CLK_N;
    wire CLK_VDISTR_LEV3[24]: multi_root;
    wire CLK_VDISTR_LEV3_N[24]: multi_branch CLK_N;

    wire CLK_PD_H_SPLITTER[24]: regional GCLK_PD_H;
    wire CLK_PD_H_VNOC[24]: bel;
    wire CLK_PD_V[24]: regional GCLK_PD_V;
    wire CLK_PD_V_S[24]: branch CLK_S;

    wire CLK_HDISTR_MUX[24]: mux;
    wire CLK_HDISTR_MUX_DUMMY0[24]: tie 0;
    wire CLK_HDISTR_MUX_DUMMY1[24]: tie 0;
    wire CLK_HDISTR_MUX_DUMMY2[24]: tie 0;
    wire CLK_HDISTR_MUX_DUMMY3[24]: tie 0;

    wire CLK_HROUTE_MUX[12]: mux;
    wire CLK_HROUTE_MUX_DUMMY0[12]: tie 0;
    wire CLK_HROUTE_MUX_DUMMY1[12]: tie 0;
    wire CLK_HROUTE_MUX_DUMMY2[12]: tie 0;
    wire CLK_HROUTE_MUX_DUMMY3[12]: tie 0;

    wire CLK_VROUTE_MUX[24]: mux;
    wire CLK_VROUTE_MUX_DUMMY[24]: tie 1;

    wire CLK_HDISTR_TEST[24]: bel;
    wire CLK_HDISTR_TEST_RING[24]: bel;

    wire CLK_HROUTE_TEST[24]: bel;
    wire CLK_HROUTE_TEST_RING[12]: bel;

    wire CLK_HDISTR_LOC_SSIT2_H[24]: regional GCLK_PD_H;
    wire CLK_HDISTR_LOC_SSIT2_V[24]: regional GCLK_PD_V;
    wire CLK_HDISTR_LOC_SSIT2_LOOP[24]: mux;
    wire CLK_HDISTR_SSIT2_H[24]: regional GCLK_PD_H;
    wire CLK_HROUTE_SSIT2_H[12]: regional GCLK_PD_H;
    wire OUT_GCLK_PD[24]: bel;
    wire CLK_VROUTE_SSIT2_V[24]: regional GCLK_PD_V;
    wire CLK_VDISTR_SSIT2_V[24]: regional GCLK_PD_V;


    wire CLK_VDISTR_PRE_MUX[24]: mux;
    wire CLK_VDISTR_N_PRE_MUX[24]: mux;
    wire CLK_VDISTR_MUX[24]: mux;
    wire CLK_VDISTR_N_MUX[24]: mux;
    wire CLK_VDISTR_LEV1_MUX[24]: mux;
    wire CLK_VDISTR_LEV1_N_MUX[24]: mux;
    wire CLK_VDISTR_LEV2_MUX[24]: mux;
    wire CLK_VDISTR_LEV2_N_MUX[24]: mux;

    wire CLK_VDISTR_MUX_DUMMY0[24]: tie 0;
    wire CLK_VDISTR_MUX_DUMMY1[24]: tie 0;
    wire CLK_VDISTR_N_MUX_DUMMY0[24]: tie 0;
    wire CLK_VDISTR_N_MUX_DUMMY1[24]: tie 0;
    wire CLK_VDISTR_N_MUX_DUMMY2[24]: tie 0;
    wire CLK_VDISTR_LEV1_MUX_DUMMY0[24]: tie 0;
    wire CLK_VDISTR_LEV1_MUX_DUMMY1[24]: tie 0;
    wire CLK_VDISTR_LEV1_MUX_DUMMY2[24]: tie 0;
    wire CLK_VDISTR_LEV1_N_MUX_DUMMY0[24]: tie 0;
    wire CLK_VDISTR_LEV1_N_MUX_DUMMY1[24]: tie 0;
    wire CLK_VDISTR_LEV1_N_MUX_DUMMY2[24]: tie 0;
    wire CLK_VDISTR_LEV2_MUX_DUMMY0[24]: tie 0;
    wire CLK_VDISTR_LEV2_MUX_DUMMY1[24]: tie 0;
    wire CLK_VDISTR_LEV2_N_MUX_DUMMY[24]: tie 0;

    wire CLK_VDISTR_TMUX[24]: mux;
    wire CLK_VDISTR_N_TMUX[24]: mux;
    wire CLK_VDISTR_LEV1_TMUX[24]: mux;
    wire CLK_VDISTR_LEV1_N_TMUX[24]: mux;
    wire CLK_VDISTR_LEV2_TMUX[24]: mux;
    wire CLK_VDISTR_LEV2_N_TMUX[24]: mux;

    wire CLK_VDISTR_TMUX_DUMMY[24]: tie 0;
    wire CLK_VDISTR_N_TMUX_DUMMY[24]: tie 0;
    wire CLK_VDISTR_LEV1_TMUX_DUMMY[24]: tie 0;
    wire CLK_VDISTR_LEV1_N_TMUX_DUMMY[24]: tie 0;
    wire CLK_VDISTR_LEV2_TMUX_DUMMY[24]: tie 0;
    wire CLK_VDISTR_LEV2_N_TMUX_DUMMY[24]: tie 0;

    wire CLK_VDISTR_TEST[24]: mux;
    wire CLK_VDISTR_N_TEST[24]: mux;
    wire CLK_VDISTR_LEV1_TEST[24]: mux;
    wire CLK_VDISTR_LEV1_N_TEST[24]: mux;
    wire CLK_VDISTR_LEV2_TEST[24]: mux;
    wire CLK_VDISTR_LEV2_N_TEST[24]: mux;

    wire IMUX_BUFG_GT_I[24]: mux;
    wire IMUX_BUFG_GT_CLR[24]: mux;
    wire IMUX_BUFG_GT_CE[24]: mux;
    wire IMUX_BUFG_GT_CLR_DUMMY[24]: tie 0;
    wire IMUX_BUFG_GT_CE_DUMMY[24]: tie 1;
    wire OUT_BUFG_FABRIC[24]: bel;
    wire OUT_BUFG_GT[24]: bel;
    wire OUT_BUFG_PS[12]: bel;
    wire OUT_BUFG_GT_SYNC_CE[41]: bel;
    wire OUT_BUFG_GT_SYNC_CLR[41]: bel;
    wire IN_BUFG_GT_SYNC_DUMMY[4]: mux;

    // RCLK leaf clock muxes
    wire IMUX_BUFDIV_LEAF[32]: mux;

    // HDIO connection
    wire IMUX_BUFGCE_HDIO[4]: mux;
    wire IMUX_BUFGCE_HDIO_DUMMY0[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY1[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY2[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY3[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY4[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY5[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY6[4]: tie 0;
    wire IMUX_BUFGCE_HDIO_DUMMY7[4]: tie 0;
    wire HDIO_CCIO[2]: bel;
    wire OUT_BUFGCE_HDIO[4]: bel;
    wire OUT_BUFGCE_HDIO_S[4]: branch CLK_HALF_S;
    wire IMUX_DPLL_HDIO_RCLK: branch CLK_HALF_N;
    wire IMUX_DPLL_HDIO_RCLK_S: mux;

    wire IMUX_DPLL_CLKIN: branch CLK_HALF_N;
    wire IMUX_DPLL_CLKIN_S: mux;
    wire IMUX_DPLL_CLKIN_DUMMY[4]: tie 1;
    wire IMUX_DPLL_CLKIN_S_DUMMY[4]: tie 1;
    wire IMUX_DPLL_CLKIN_DUMMY_GND[2]: tie 0;
    wire IMUX_DPLL_CLKIN_S_DUMMY_GND[2]: tie 0;
    wire IMUX_DPLL_CLKIN_DESKEW: branch CLK_HALF_N;
    wire IMUX_DPLL_CLKIN_DESKEW_S: mux;
    wire IMUX_DPLL_CLKIN_DESKEW_DUMMY[2]: tie 1;
    wire IMUX_DPLL_CLKIN_DESKEW_S_DUMMY[2]: tie 1;
    wire OUT_DPLL[5]: bel;
    wire OUT_DPLL_GT[5]: branch DPLL_GT;
    wire OUT_DPLL_GT_S[5]: branch CLK_HALF_S;
    wire OUT_GT_CLK[15]: bel;
    wire OUT_GT_CLK_S[15]: branch CLK_HALF_S;
    wire OUT_CPM_CLK[2]: regional CPM;

    wire OUT_PS_SCAN_CLK: regional PS;
    wire OUT_PS_MCLK: regional PS;
    wire OUT_PS_NPI_CLK: regional PS;
    wire OUT_PS_NOC_CLK: regional PS;
    wire OUT_PS_HSM_REF_CLK[2]: regional HSM;

    // HSR clocking
    wire HSR_HROUTE[24]: multi_root;
    wire HSR_HROUTE_W[24]: multi_branch HSR_W;
    wire HSR_HDISTR[24]: multi_root;
    wire HSR_HDISTR_W[24]: multi_branch HSR_W;
    wire HSR_HROUTE_SLR[24]: regional HSR_SSIT2_H;
    wire HSR_HDISTR_SLR[24]: regional HSR_SSIT2_H;
    wire HSR_HDISTR_MUX[24]: mux;
    wire HSR_HDISTR_MUX_DUMMY0[24]: tie 1;
    wire HSR_HDISTR_MUX_DUMMY1[24]: tie 1;
    wire HSR_HDISTR_MUX_DUMMY2[24]: tie 1;
    wire HSR_HROUTE_TEST_MUX[24]: mux;
    wire HSR_HROUTE_TEST[24]: bel;
    wire HSR_HROUTE_MUX[24]: mux;
    wire HSR_CLK_TEST_IN[6]: bel;
    wire HSR_OUT_BUFG_TEST[6]: bel;

    // XPIO / X5IO clocking shared stuff
    wire HSR_IMUX_BUFG[24]: mux;
    for i in 0..10 {
        wire "HSR_IMUX_BUFG_DUMMY{i}"[24]: tie 1;
    }
    wire HSR_IMUX_BUFG_HDISTR[24]: mux;
    wire HSR_IMUX_BUFG_XPIO[24]: mux;
    for i in 0..21 {
        wire "HSR_IMUX_BUFG_XPIO_DUMMY{i}"[24]: tie 1;
    }
    wire HSR_IMUX_BUFGCTRL_I0[8]: mux;
    wire HSR_IMUX_BUFGCTRL_I1[8]: mux;
    wire HSR_OUT_BUFGCE[24]: bel;
    wire HSR_OUT_BUFGCTRL[8]: bel;
    wire HSR_OUT_BUFGCE_DIV[4]: bel;
    wire HSR_IMUX_MMCM_CLKIN1: mux;
    wire HSR_IMUX_MMCM_CLKIN2: mux;
    wire HSR_IMUX_MMCM_CLKFB: mux;
    wire HSR_IMUX_MMCM_CLKIN1_HDISTR: mux;
    wire HSR_IMUX_MMCM_CLKIN2_HDISTR: mux;
    wire HSR_IMUX_MMCM_CLKFB_HDISTR: mux;
    wire HSR_IMUX_MMCM_CLKIN1_DUMMY[7]: tie 1;
    wire HSR_IMUX_MMCM_CLKIN2_DUMMY[7]: tie 1;
    wire HSR_IMUX_MMCM_CLKFB_DUMMY[7]: tie 1;
    wire HSR_IMUX_MMCM_CLKFB_DESKEW[2]: mux;
    wire HSR_IMUX_MMCM_CLKFB_DESKEW0_DUMMY[11]: tie 1;
    wire HSR_IMUX_MMCM_CLKFB_DESKEW1_DUMMY[11]: tie 1;
    wire HSR_IMUX_MMCM_CLKIN_DESKEW[2]: mux;
    wire HSR_IMUX_MMCM_CLKIN_DESKEW_DUMMY[2]: tie 1;
    wire HSR_OUT_MMCM[9]: bel;

    wire HSR_IMUX_DPLL_CLKIN: mux;
    wire HSR_IMUX_DPLL_CLKIN_DUMMY[7]: tie 1;
    wire HSR_IMUX_DPLL_CLKIN_DESKEW: mux;
    wire HSR_IMUX_DPLL_CLKIN_DESKEW_DUMMY[3]: tie 1;
    wire HSR_IMUX_DPLL_CLKIN_HDISTR: mux;
    wire HSR_OUT_DPLL[5]: bel;

    wire HSR_OUT_XPLL0[5]: bel;
    wire HSR_OUT_XPLL1[5]: bel;
    wire HSR_OUT_XPLL_W[2]: branch XPIO_E;
    wire HSR_OUT_XPLL_E[2]: branch XPIO_W;
    wire HSR_OUT_PHY[9]: bel;
    wire HSR_OUT_CLK_CCIO[4]: bel;
    wire HSR_OUT_CCIO[4]: bel;
    wire HSR_OUT_CCIO_W[4]: branch XPIO_E;
    wire HSR_OUT_CCIO_E[4]: branch XPIO_W;

    // XPIO clocking
    wire HSR_OUT_XPIO_PHY[2]: bel;
    wire HSR_OUT_XPIO_CCIO: bel;

    // X5IO clocking
    wire HSR_GCLK_XPIO[6]: regional XPIO_LEAF;
    wire HSR_OUT_DMC_CLK_DESKEW: bel;
    wire HSR_IMUX_XPLL_CLKIN[2]: mux;
    wire HSR_IMUX_XPLL_CLKIN_MUX[2]: mux;
    wire HSR_IMUX_XPLL_CLKIN_DESKEW0[2]: mux;
    wire HSR_IMUX_XPLL_CLKIN_DESKEW1[2]: mux;
    wire HSR_IMUX_XPLL_CLKFB_DESKEW0[2]: mux;
    wire HSR_IMUX_XPLL_CLKFB_DESKEW1[2]: mux;
    wire HSR_IMUX_XPLL0_CLKIN_DESKEW0_DUMMY[2]: tie 1;
    wire HSR_IMUX_XPLL0_CLKIN_DESKEW1_DUMMY[2]: tie 1;
    wire HSR_IMUX_XPLL1_CLKIN_DESKEW0_DUMMY[2]: tie 1;
    wire HSR_IMUX_XPLL1_CLKIN_DESKEW1_DUMMY[2]: tie 1;
    wire HSR_IMUX_XPLL0_CLKFB_DESKEW0_DUMMY[10]: tie 1;
    wire HSR_IMUX_XPLL0_CLKFB_DESKEW1_DUMMY[10]: tie 1;
    wire HSR_IMUX_XPLL1_CLKFB_DESKEW0_DUMMY[10]: tie 1;
    wire HSR_IMUX_XPLL1_CLKFB_DESKEW1_DUMMY[10]: tie 1;

    // X5IO clocking
    wire HSR_OUT_X5IO_CCIO[2]: bel;
    wire HSR_OUT_X5IO_CCIO_NB[2]: branch X5IO_PAIR;
    wire HSR_IMUX_X5PLL_CLKIN: mux;
    wire HSR_IMUX_X5PLL_CLKIN_DUMMY[12]: tie 0;
    wire HSR_IMUX_X5PLL_CLKIN_DESKEW0: mux;
    wire HSR_IMUX_X5PLL_CLKIN_DESKEW0_DUMMY[2]: tie 0;
    wire HSR_IMUX_X5PLL_CLKIN_DESKEW1: mux;
    wire HSR_IMUX_X5PLL_CLKIN_DESKEW1_DUMMY[2]: tie 0;
    wire HSR_IMUX_X5PLL_CLKFB_DESKEW0: mux;
    wire HSR_IMUX_X5PLL_CLKFB_DESKEW0_DUMMY: tie 0;
    wire HSR_IMUX_X5PLL_CLKFB_DESKEW1: mux;
    wire HSR_IMUX_X5PLL_CLKFB_DESKEW1_DUMMY: tie 0;
    wire HSR_IMUX_X5PLL_CLKFB_DESKEW_DUMMY_BLI[2]: mux;
    wire HSR_IMUX_X5PHIO_PLL_CLK0: mux;
    wire HSR_IMUX_X5PHIO_PLL_CLK0_DCC: mux;
    wire HSR_IMUX_X5PHIO_PLL_CLK0_DCC_INV: mux;
    wire HSR_IMUX_X5PHIO_PLL_CLK90: mux;
    wire HSR_IMUX_X5PHIO_PLL_CLK90_DCC: mux;
    wire HSR_IMUX_X5PHIO_PDQS_IN: mux;
    wire HSR_IMUX_X5PHIO_NDQS_IN: mux;
    wire HSR_IMUX_X5PHIO_PDQS_IN_MUX: mux;
    wire HSR_IMUX_X5PHIO_NDQS_IN_MUX: mux;
    wire HSR_IMUX_X5PHIO_DQS_IN_R: mux;
    wire HSR_IMUX_X5PHIO_DQS_IN_R_DUMMY[2]: tie 0;
    wire HSR_OUT_X5PHIO_RX0_RDQS_P_OUT: bel;
    wire HSR_OUT_X5PHIO_RX0_RDQS_P_OUT_INV: bel;
    wire HSR_OUT_X5PHIO_RX1_RDQS_P_OUT: bel;
    wire HSR_OUT_X5PHIO_DQS_DLY_OUT_R: bel;
    wire HSR_OUT_X5PHIO_PDQS_O: bel;
    wire HSR_OUT_X5PHIO_NDQS_O: bel;
    wire HSR_OUT_X5PHIO_PDQS_DLY_O: bel;
    wire HSR_OUT_X5PHIO_NDQS_DLY_O: bel;
    wire HSR_OMUX_X5PHIO_PDQS[2]: mux;
    wire HSR_OMUX_X5PHIO_NDQS[2]: mux;
    wire HSR_OMUX_X5PHIO_PDQS_DLY: mux;
    wire HSR_OMUX_X5PHIO_NDQS_DLY: mux;
    wire HSR_OUT_X5PHIO_DIV64_CLK: bel;
    wire HSR_OUT_X5PLL[5]: bel;
    wire HSR_OUT_X5PLL_PHY[2]: bel;
    wire HSR_OUT_X5PLL_PHY_NB[2]: branch X5IO_PAIR;
    wire HSR_OMUX_X5PLL[4]: mux;
    wire HSR_OMUX_X5PLL_CASC[2]: mux;
    wire HSR_OMUX_X5PLL_CASC_NB[2]: branch X5IO_PAIR;
    wire HSR_X5PLL_CASC_W0[2]: mux;
    wire HSR_X5PLL_CASC_W1[2]: branch XPIO_E;
    wire HSR_X5PLL_CASC_E0[2]: mux;
    wire HSR_X5PLL_CASC_E1[2]: branch XPIO_W;
    wire HSR_OMUX_X5IO_PHY[4]: mux;
    wire HSR_OMUX_X5IO_RXCLK: mux;
    wire HSR_OMUX_X5IO_RXCLK_NB: branch X5IO_PAIR;
    wire HSR_X5IO_TESTCLK[2]: mux;
    wire HSR_X5IO_GCLK[2]: mux;
    wire HSR_X5IO_RXCLK: mux;
    wire HSR_IMUX_DMC_CLK: mux;
    wire HSR_IMUX_DMC_CLK_DUMMY: tie 0;
    wire HSR_OUT_DMC_TEST_CLK[2]: bel;
    wire HSR_X5PLL_HS_DESKEW_CLKIN: branch X5IO_HS_DESKEW_CLKIN;
    wire HSR_X5PLL_HS_DESKEW_CLKIN_NB: branch X5IO_PAIR;
    wire HSR_X5PLL_HS_DESKEW_CLKFB: branch X5IO_HS_DESKEW_CLKFB;
    wire HSR_X5PLL_HS_DESKEW_CLKFB_NB: branch X5IO_PAIR;

    // ME clocking
    wire HSR_IMUX_TO_ME: mux;
    wire HSR_IMUX_HSM_TO_ME: mux;

    // HBM clocking
    wire HSR_IMUX_TO_HBM[2]: mux;
    wire HSR_IMUX_TO_HBM_HROUTE[2]: mux;
    wire HSR_OUT_BUFG_HBM[2]: bel;
    wire HSR_IN_BUFG_HBM[2]: mux;
    wire HSR_OUT_HBM_CCIO: bel;
    wire HSR_OUT_HBM_CLK_NOC2PHY: bel;
    wire HSR_OUT_HBM_CLK_MC2PHY: bel;

    wire AIE_OUT_PLL_TEST_CLK_OUT[2]: bel;

    tile_slot INT {
        bel_slot INT: routing;

        tile_class INT {
            cell CELL[2];
        }
    }

    tile_slot CLE_BC {
        bel_slot CLE_BC_INT: routing;
        bel_slot LAGUNA[6]: LAGUNA;

        tile_class CLE_BC, CLE_BC_SLL, CLE_BC_SLL2 {
            cell CELL[2];
        }
    }

    tile_slot INTF {
        bel_slot INTF_INT: routing;
        bel_slot INTF_TESTMUX: routing;
        bel_slot INTF_TEST_INT: routing;
        bel_slot IRI[4]: IRI;

        tile_class INTF_W, INTF_W_HB, INTF_W_HDIO, INTF_W_PSS {
            cell CELL;
        }
        tile_class INTF_W_TERM_PSS, INTF_W_TERM_GT {
            cell CELL;
        }
        tile_class INTF_E, INTF_E_HB, INTF_E_HDIO {
            cell CELL;
        }
        tile_class INTF_E_TERM_GT {
            cell CELL;
        }
        tile_class INTF_BLI_CLE_ABC_S, INTF_BLI_CLE_ABC_N, INTF_BLI_CLE_D_S, INTF_BLI_CLE_D_N {
            cell CELL;
        }
    }

    tile_slot BEL {
        bel_slot BEL_INT: routing;

        bel_slot BLI_GRP0[3]: BLI_GRP0;
        bel_slot BLI_GRP1[3]: BLI_GRP1;
        bel_slot BLI_GRP2[3]: BLI_GRP2;
        bel_slot BLI_GRP4: BLI_GRP456;
        bel_slot BLI_GRP5: BLI_GRP456;
        bel_slot BLI_GRP6: BLI_GRP456;
        bel_slot BLI_GRP7: BLI_GRP7;
        bel_slot BLI_TMR: BLI_TMR;
        tile_class BLI_CLE_S, BLI_CLE_N, BLI_S, BLI_N {
            cell CELL[4];
        }

        bel_slot SLICE[2]: SLICE;
        tile_class CLE, CLE_VR {
            cell CELL;
        }

        bel_slot BRAM: BRAM;
        bel_slot URAM: URAM;
        tile_class BRAM, URAM {
            cell CELL[4];
        }

        bel_slot DSP[2]: DSP;
        tile_class DSP {
            cell CELL_W[2];
            cell CELL_E[2];
        }

        bel_slot PCIE4: PCIE4;
        bel_slot PCIE5: PCIE5;
        bel_slot MRMAC: MRMAC;
        bel_slot SDFEC: SDFEC;
        bel_slot DFE_CFC: DFE_CFC;
        tile_class PCIE4, PCIE5, MRMAC, SDFEC, DFE_CFC {
            cell CELL_W[48];
            cell CELL_E[48];
        }

        bel_slot DCMAC: DCMAC;
        bel_slot ILKN: ILKN;
        bel_slot HSC: HSC;
        tile_class DCMAC, ILKN, HSC {
            cell CELL_W[96];
            cell CELL_E[96];
        }

        bel_slot HDIOB[22]: HDIOB;
        bel_slot HDIOLOGIC[22]: HDIOLOGIC;
        bel_slot HDIOLOGIC_SCAN[11]: HDIOLOGIC_SCAN;
        bel_slot BUFGCE_HDIO[4]: BUFGCE;
        bel_slot DPLL_HDIO: DPLL;
        bel_slot HDIO_BIAS: HDIO_BIAS;
        bel_slot RPI_HD_APB: RPI_HD_APB;
        bel_slot HDLOGIC_APB: HDLOGIC_APB;
        tile_class HDIO {
            cell CELL_W[48];
            cell CELL_E[48];
        }

        bel_slot MISR: MISR;
        tile_class MISR {
            cell CELL_W[48];
            cell CELL_E[48];
        }

        bel_slot VNOC_NSU512: NOC_NSU512;
        bel_slot VNOC_NMU512: NOC_NMU512;
        bel_slot VNOC_NPS_NSU: NOC_NPS;
        bel_slot VNOC_NPS_NMU: NOC_NPS;
        bel_slot VNOC_NPS6X_NSU: NOC2_NPS6X;
        bel_slot VNOC_NPS6X_NMU: NOC2_NPS6X;
        bel_slot VNOC_SCAN: NOC2_SCAN;
        tile_class VNOC, VNOC2, VNOC4 {
            cell CELL_W[48];
            cell CELL_E[48];
        }

        bel_slot RFADC[2]: RFADC;
        bel_slot RFDAC[2]: RFDAC;
        tile_class RFADC, RFDAC {
            cell CELL[96];
        }

        bel_slot VDU: VDU;
        tile_class VDU {
            cell CELL[48];
        }

        bel_slot BFR_B: BFR_B;
        tile_class BFR_B {
            cell CELL[48];
        }

        bel_slot VCU2: VCU2;
        bel_slot VCU2_NOC_SCAN: NOC2_SCAN;
        bel_slot VCU2_NPS6X[4]: NOC2_NPS6X;
        bel_slot VCU2_NMU[4]: NOC_NMU128;
        bel_slot VCU2_NSU: NOC_NSU128;
        tile_class VCU2 {
            cell CELL[96];
        }

        bel_slot ISP2: ISP2;
        tile_class ISP2 {
            cell CELL[48];
        }

        bel_slot PS9: PS9;
        tile_class PS9 {
            cell CELL[96];
            cell HSR[2];
        }

        bel_slot XRAM: XRAM;
        tile_class XRAM {
            cell CELL[48];
        }

        tile_class XRAM_NC {
        }

        bel_slot XPIPE: XPIPE;
        tile_class XPIPE {
            cell CELL[48];
        }

        bel_slot CPM4: CPM4;
        tile_class CPM4 {
            cell CELL[48];
            cell EXT_W[48];
            cell EXT_E[48];
        }

        bel_slot CPM5: CPM5;
        tile_class CPM5 {
            cell CELL[192];
        }

        bel_slot CPM6: CPM6;
        tile_class CPM6 {
            cell CELL[192];
        }

        bel_slot PSXL: PSXL;
        tile_class PSXL {
            cell CELL[240];
            cell HSR;
        }

        bel_slot CPM5N: CPM5N;
        tile_class CPM5N {
            cell CELL[192];
        }

        bel_slot PSXC: PSXC;
        tile_class PSXC {
            cell CELL[192];
            cell HSR;
        }

        bel_slot PSXT: PSXT;
        tile_class PSXT {
            cell CELL[192];
            cell HSR;
        }

        bel_slot MMI: MMI;
        tile_class MMI {
            cell CELL[96];
        }

        tile_class MMI_NC {
        }

        bel_slot HNICX: HNICX;
        tile_class HNICX {
            cell CELL[336];
        }
    }

    tile_slot GT {
        bel_slot GTY: GTY;
        bel_slot GTYP: GTYP;
        bel_slot GTM: GTM;
        bel_slot GTM2: GTM2;
        bel_slot HNICPIPE: HNICPIPE;
        tile_class GTY, GTYP, GTYP_XRAM, GTM, GTM_HNIC, GTM2_S, GTM2_N {
            cell CELL[48];
        }
        tile_class GTYP_PS, GTM2_PS {
        }
    }

    tile_slot SYSMON_SAT {
        bel_slot SYSMON_SAT_VNOC: SYSMON_SAT;
        tile_class SYSMON_SAT_VNOC {
            cell CELL_W[48];
            cell CELL_E[48];
        }

        bel_slot SYSMON_SAT_GT: SYSMON_SAT;
        tile_class SYSMON_SAT_GT {
            cell CELL[48];
        }
    }

    tile_slot DPLL {
        bel_slot DPLL_GT: DPLL;

        tile_class DPLL_GT {
            cell CELL[48];
        }
    }

    tile_slot RCLK_INT {
        bel_slot RCLK_INT: routing;

        tile_class RCLK {
            cell CELL[2];
        }
    }

    tile_slot RCLK_INTF {
        bel_slot RCLK_INTF_INT: routing;
        bel_slot BUFDIV_LEAF[32]: BUFDIV_LEAF;

        tile_class RCLK_INTF {
            cell CELL;
        }

        tile_class RCLK_CLE {
            cell CELL;
        }
    }

    tile_slot RCLK_BEL {
        bel_slot RCLK_BEL_INT: routing;

        bel_slot RCLK_DFX_TEST: RCLK_DFX_TEST;
        tile_class RCLK_DFX {
            cell CELL;
        }

        tile_class RCLK_HDIO, RCLK_HDIO_VR, RCLK_HB_HDIO, RCLK_HB_HDIO_VR {
            cell CELL;
        }
    }

    tile_slot CLK_INT {
        bel_slot CLK_INT: routing;
        bel_slot GCLK_PD_H[24]: GCLK_PD;

        tile_class RCLK_SPLITTER_PD, RCLK_SPLITTER_PD_VR, RCLK_SPLITTER, RCLK_SPLITTER_VR {
            cell W, E;
        }

        tile_class RCLK_SPLITTER_SSIT {
            cell CELL;
        }

        bel_slot GCLK_DELAY[48]: GCLK_DELAY;
        bel_slot GCLK_TAPS_DECODE_VNOC: GCLK_TAPS_DECODE_VNOC;
        bel_slot GCLK_TEST[48]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_HDISTR[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_HROUTE[12]: GCLK_TEST_BUF;
        bel_slot BUFG_FABRIC[24]: BUFG;
        bel_slot BUFG_PS[12]: BUFG;
        tile_class RCLK_VNOC, RCLK_VNOC_PS {
            cell W[48], E[48];
        }

        bel_slot GCLK_TAPS_DECODE_GT: GCLK_TAPS_DECODE_GT;
        bel_slot BUFG_GT[24]: BUFG_GT;
        bel_slot BUFG_GT_SYNC[41]: BUFG_GT_SYNC;
        tile_class RCLK_GT, RCLK_GT_CPM6 {
            cell CELL[48];
        }
    }

    tile_slot CLK_V_BUF {
        bel_slot CLK_V_BUF: routing;
        bel_slot GCLK_PD_V[48]: GCLK_PD;
        bel_slot GCLK_TEST_RING_VDISTR[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_VDISTR_N[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_VDISTR_LEV1[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_VDISTR_LEV1_N[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_VDISTR_LEV2[24]: GCLK_TEST_BUF;
        bel_slot GCLK_TEST_RING_VDISTR_LEV2_N[24]: GCLK_TEST_BUF;

        tile_class
            CLK_V_BUF_VNOC,
            CLK_V_BUF_VNOC_SSIT_HNOC_S,
            CLK_V_BUF_VNOC_SSIT_S,
            CLK_V_BUF_VNOC_SSIT_MID_S,
            CLK_V_BUF_VNOC_SSIT_MID_N,
            CLK_V_BUF_VNOC_SSIT_N,
            CLK_V_BUF_VNOC_SSIT2_EDGE
        {
            cell W, E;
        }
        tile_class
            CLK_V_BUF_GT,
            CLK_V_BUF_GT_SSIT_HNOC_S,
            CLK_V_BUF_GT_SSIT_S,
            CLK_V_BUF_GT_SSIT_MID_S,
            CLK_V_BUF_GT_SSIT_MID_N,
            CLK_V_BUF_GT_SSIT_N,
            CLK_V_BUF_GT_SSIT2_EDGE
        {
            cell CELL;
        }

        tile_class CLK_S_BUF_VNOC, CLK_S_BUF_GT {
            cell CELL;
            cell HSR_ROUTE;
            cell HSR_DISTR;
        }
    }

    tile_slot BLI_MUX {
        bel_slot BLI_MUX: routing;

        tile_class BLI_MUX, BLI_MUX_NOC {
            cell CELL;
        }
    }

    tile_slot HSR_CLK {
        bel_slot HSR_CLK: routing;
        bel_slot BUFG_HSR_TEST[6]: BUFG;
        bel_slot GCLK_TEST_HSR[24]: GCLK_TEST_BUF;
        bel_slot BUFGCE[24]: BUFGCE;
        bel_slot BUFGCTRL[8]: BUFGCTRL;
        bel_slot BUFGCE_DIV[4]: BUFGCE_DIV;
        bel_slot BUFG_HBM[2]: BUFG;
        tile_class CLK_XPIO_FULL, CLK_XPIO_MIN, CLK_X5IO_FULL, CLK_X5IO_MIN_A, CLK_X5IO_MIN_B {
            cell HSR;
            cell BLI[2];
            cell BLI_MMCM[2];
        }

        tile_class CLK_ME, CLK_HBM {
            cell HSR;
        }

        tile_class CLK_HSR_SSIT2_H {
            cell HSR;
        }
    }

    tile_slot HSR_CLK_TEST {
        bel_slot HSR_CLK_TEST: routing;

        tile_class HSR_CLK_TEST_XPIO_REV1, HSR_CLK_TEST_XPIO_REV2 {
            cell HSR;
            switchbox HSR_CLK_TEST {
                permabuf HSR_CLK_TEST_IN[0] = OUT_PS_SCAN_CLK;
                if tile_class HSR_CLK_TEST_XPIO_REV1 {
                    permabuf HSR_CLK_TEST_IN[1] = OUT_PS_MCLK;
                }
                permabuf HSR_CLK_TEST_IN[2] = OUT_PS_NPI_CLK;
                permabuf HSR_CLK_TEST_IN[3] = OUT_PS_NOC_CLK;
            }
        }

        tile_class HSR_CLK_TEST_ME_REV1, HSR_CLK_TEST_ME_REV2, HSR_CLK_TEST_ME_SSIT, HSR_CLK_TEST_ME_ULVT {
            cell HSR;
            switchbox HSR_CLK_TEST {
                if tile_class HSR_CLK_TEST_ME_ULVT {
                } else {
                    permabuf HSR_CLK_TEST_IN[2] = OUT_PS_NOC_CLK;
                }
                if tile_class HSR_CLK_TEST_ME_REV1 {
                    permabuf HSR_CLK_TEST_IN[3] = OUT_PS_MCLK;
                }
                if tile_class HSR_CLK_TEST_ME_SSIT {
                } else {
                    permabuf HSR_CLK_TEST_IN[0] = OUT_PS_NPI_CLK;
                    permabuf HSR_CLK_TEST_IN[4] = OUT_PS_SCAN_CLK;
                }
            }
        }

        tile_class HSR_CLK_TEST_X5IO_A {
            cell HSR_CLK, HSR_DDRMC;
            switchbox HSR_CLK_TEST {
                permabuf HSR_CLK.HSR_CLK_TEST_IN[0] = HSR_DDRMC.HSR_OUT_DMC_TEST_CLK[0];
                permabuf HSR_CLK.HSR_CLK_TEST_IN[1] = HSR_DDRMC.HSR_OUT_DMC_TEST_CLK[1];
            }
        }
        tile_class HSR_CLK_TEST_X5IO_B {
            cell HSR_CLK, HSR_DDRMC;
            switchbox HSR_CLK_TEST {
                permabuf HSR_CLK.HSR_CLK_TEST_IN[2] = HSR_DDRMC.HSR_OUT_DMC_TEST_CLK[0];
                permabuf HSR_CLK.HSR_CLK_TEST_IN[3] = HSR_DDRMC.HSR_OUT_DMC_TEST_CLK[1];
            }
        }
    }

    tile_slot HSR_CLK_IN {
        bel_slot HSR_CLK_IN: routing;
        tile_class HSR_CLK_HBM_EVEN, HSR_CLK_HBM_ODD {
            cell HSR_CLK;
            cell HSR_CHNL[2];
            switchbox HSR_CLK_IN {
                if tile_class HSR_CLK_HBM_EVEN {
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[0] = HSR_CHNL[0].HSR_OUT_HBM_CLK_NOC2PHY;
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[1] = HSR_CHNL[1].HSR_OUT_HBM_CLK_MC2PHY;
                } else {
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[0] = HSR_CHNL[0].HSR_OUT_HBM_CLK_MC2PHY;
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[1] = HSR_CHNL[1].HSR_OUT_HBM_CLK_NOC2PHY;
                }
            }
        }
        tile_class HSR_CLK_HBM_MS_EVEN, HSR_CLK_HBM_MS_ODD {
            cell HSR_CLK;
            cell HSR_MS;
            cell HSR_CHNL;
            switchbox HSR_CLK_IN {
                permabuf HSR_CLK.HSR_IN_BUFG_HBM[0] = HSR_MS.HSR_OUT_HBM_CCIO;
                if tile_class HSR_CLK_HBM_MS_EVEN {
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[1] = HSR_CHNL.HSR_OUT_HBM_CLK_NOC2PHY;
                } else {
                    permabuf HSR_CLK.HSR_IN_BUFG_HBM[1] = HSR_CHNL.HSR_OUT_HBM_CLK_MC2PHY;
                }
            }
        }
        tile_class HSR_CLK_AIE_PLL {
            cell HSR_CLK;
            cell AIE;
            switchbox HSR_CLK_IN {
                permabuf HSR_CLK.HSR_CLK_TEST_IN[1] = AIE.AIE_OUT_PLL_TEST_CLK_OUT[0];
                permabuf HSR_CLK.HSR_CLK_TEST_IN[5] = AIE.AIE_OUT_PLL_TEST_CLK_OUT[1];
            }
        }
    }

    tile_slot HSR_MMCM {
        bel_slot MMCM: MMCM;
        tile_class MMCM_XPIO, MMCM_X5IO {
            cell HSR;
            cell BLI[2];
        }
    }

    tile_slot HSR_DPLL {
        bel_slot DPLL_HSR: DPLL;
        tile_class DPLL_XPIO, DPLL_X5IO {
            cell HSR;
            cell BLI[2];
        }
        tile_class DPLL_X5IO_NC {
            cell HSR;
        }
    }

    tile_slot HSR_XPLL {
        bel_slot XPLL_INT: routing;
        bel_slot XPLL[2]: XPLL;
        tile_class XPLL {
            cell HSR[9];
            cell BLI[2];
        }

        bel_slot X5PLL: X5PLL;
        bel_slot X5PHIO_DCCIBUF[8]: X5PHIO_DCCIBUF;
        bel_slot X5PHIO_HARD_INV[8]: X5PHIO_HARD_INV;
        tile_class X5PLL_A, X5PLL_B, X5PLL_S {
            cell HSR_IO[4];
            cell HSR_CLK;
            cell HSR_PAIR;
            cell BLI[2];
        }
        tile_class X5PLL_NC {
            cell HSR_IO[4];
            cell HSR_CLK;
            cell HSR_PAIR;
        }
    }

    tile_slot HSR_XPLL_GASKET {
        bel_slot XPLL_GASKET_INT: routing;
        tile_class X5PLL_GASKET {
            cell HSR[8];
            switchbox XPLL_GASKET_INT {
                permabuf HSR[0].HSR_OUT_CLK_CCIO[0] = HSR[0].HSR_OUT_X5IO_CCIO[0];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[1] = HSR[0].HSR_OUT_X5IO_CCIO[1];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[2] = HSR[4].HSR_OUT_X5IO_CCIO[0];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[3] = HSR[4].HSR_OUT_X5IO_CCIO[1];
                for i in 0..4 {
                    permabuf HSR[0].HSR_OUT_XPLL0[i] = HSR[0].HSR_OMUX_X5PLL[i];
                    permabuf HSR[0].HSR_OUT_XPLL1[i] = HSR[4].HSR_OMUX_X5PLL[i];
                }
                permabuf HSR[0].HSR_OUT_XPLL0[4] = HSR[0].HSR_OUT_X5PLL[4];
                permabuf HSR[0].HSR_OUT_XPLL1[4] = HSR[4].HSR_OUT_X5PLL[4];
                for i in 0..8 {
                    permabuf HSR[0].HSR_OUT_PHY[i] = HSR[i].HSR_OUT_XPIO_PHY[0];
                }
            }
        }

        tile_class X5PLL_GASKET_HALF {
            cell HSR[4];
            switchbox XPLL_GASKET_INT {
                permabuf HSR[0].HSR_OUT_CLK_CCIO[0] = HSR[0].HSR_OUT_X5IO_CCIO[0];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[1] = HSR[0].HSR_OUT_X5IO_CCIO[1];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[2] = HSR[0].HSR_OUT_X5IO_CCIO[0];
                permabuf HSR[0].HSR_OUT_CLK_CCIO[3] = HSR[0].HSR_OUT_X5IO_CCIO[1];
                for i in 0..4 {
                    permabuf HSR[0].HSR_OUT_XPLL0[i] = HSR[0].HSR_OMUX_X5PLL[i];
                    permabuf HSR[0].HSR_OUT_XPLL1[i] = HSR[0].HSR_OMUX_X5PLL[i];
                }
                permabuf HSR[0].HSR_OUT_XPLL0[4] = HSR[0].HSR_OUT_X5PLL[4];
                permabuf HSR[0].HSR_OUT_XPLL1[4] = HSR[0].HSR_OUT_X5PLL[4];
                for i in 0..4 {
                    permabuf HSR[0].HSR_OUT_PHY[i] = HSR[i].HSR_OUT_XPIO_PHY[0];
                    permabuf HSR[0].HSR_OUT_PHY[4 + i] = HSR[i].HSR_OUT_XPIO_PHY[0];
                }
            }
        }
    }

    tile_slot HSR_IO {
        bel_slot XPHY: XPHY;
        bel_slot XPHY_NIBBLESLICE[6]: XPHY_NIBBLESLICE;
        bel_slot XPIOLOGIC[6]: XPIOLOGIC;
        bel_slot XPIOB[6]: XPIOB;
        bel_slot XPIO_VREF: XPIO_VREF;
        tile_class XPIO {
            cell HSR;
            cell BLI[2];
        }
        tile_class XPIO_NC {
            cell HSR;
        }

        bel_slot X5IO_OCTAD: X5IO_OCTAD;
        tile_class X5IO {
            cell HSR;
            cell BLI[2];
        }
        tile_class X5IO_NC {
            cell HSR;
        }

        bel_slot HBM_CHNL: HBM_CHNL;
        tile_class HBM_CHNL {
            cell HSR;
        }
    }

    tile_slot HSR_BANK {
        bel_slot XPIO_DCI: XPIO_DCI;
        tile_class XPIO_DCI_A, XPIO_DCI_B, XPIO_DCI_C, XPIO_DCI_S {
            cell BLI[2];
        }

        bel_slot X5PHIO_CMU: X5PHIO_CMU;
        tile_class X5IO_BANK {
            cell BLI[2];
        }
        tile_class X5IO_BANK_NC;

        bel_slot HBM_MS: HBM_MS;
        bel_slot SYSMON_SAT_HBM[2]: SYSMON_SAT;
        bel_slot HPLL: XPLL;
        tile_class HBM_MS {
            cell HSR[2];
            cell BLI[2];
        }
    }
    tile_slot HSR_SCAN {
        bel_slot XPIO_SCAN: XPIO_SCAN;
        tile_class XPIO_SCAN_A, XPIO_SCAN_B, XPIO_SCAN_C, XPIO_SCAN_S {
            cell BLI[2];
        }

        bel_slot X5IO_SCAN: X5IO_SCAN;
        tile_class X5IO_SCAN_A, X5IO_SCAN_B, X5IO_SCAN_C {
            cell BLI[2];
        }
    }

    tile_slot HSR_NMU {
        bel_slot NOC_NMU_HBM2E[2]: NOC_NMU_HBM2E;
        bel_slot NOC_HBM_BLI_SCAN: NOC_HBM_BLI_SCAN;

        tile_class HBM_INTF {
            cell BLI[6];
        }
    }

    tile_slot HSR_DDRMC {
        bel_slot DDRMC: DDRMC;
        tile_class DDRMC {
            cell HSR;
            cell BLI[4];
        }

        bel_slot DDRMC5: DDRMC5;
        tile_class DDRMC5, DDRMC5C, DDRMC5E, DDRMC5E_ALT, DDRMC5E2, DDRMC5E2_CUT, DDRMC5X {
            cell HSR[2];
            cell BLI[2];
        }
    }

    tile_slot HSR_SYSMON_SAT {
        bel_slot SYSMON_SAT_HSR: SYSMON_SAT;
        tile_class SYSMON_SAT_HSR_XPIO, SYSMON_SAT_HSR_X5IO {
            cell BLI[2];
        }
        tile_class SYSMON_SAT_HSR_ME {
            cell BLI[4];
        }
        tile_class SYSMON_SAT_HSR_NC;
    }

    tile_slot HNOC {
        bel_slot HNOC_NPS[8]: NOC_NPS;
        bel_slot HNOC_NPS_MID[2]: NOC_NPS;
        bel_slot HNOC_NPS6X[3]: NOC2_NPS6X;
        bel_slot HNOC_SCAN[4]: NOC2_SCAN;
        bel_slot HNOC_NMU512[2]: NOC_NMU512;
        bel_slot HNOC_NSU512: NOC_NSU512;

        bel_slot HNOC_NPI_NIR: NPI_NIR;
        bel_slot HNOC_PS_NMU[7]: NOC_NMU128;
        bel_slot HNOC_PS_NSU[4]: NOC_NSU128;

        tile_class HNOC_XPIO_PS9_A, HNOC_XPIO_PS9_B, HNOC_XPIO_PS9_C, HNOC_XPIO;

        tile_class HNOC_SSIT_PS9_A, HNOC_SSIT_PS9_BD, HNOC_SSIT_PS9_C;

        tile_class
            HNOC_X5IO_PS9_A, HNOC_X5IO_PS9_B,
            HNOC_X5IO_PSX_A, HNOC_X5IO_PSX_B,
            HNOC_X5IO_PSXT_A;
        tile_class
            HNOC_X5IO_PS9_C, HNOC_X5IO_PSXL_C, HNOC_X5IO_PSXC_C, HNOC_X5IO_PSXT_B, HNOC_X5IO_PSXT_C,
            HNOC_X5IO_CROSS, HNOC_X5IO_CROSS_ALT, HNOC_X5IO_CAP
        {
            cell BLI_SCAN[2];
        }

        bel_slot HNOC_XBR4X2: NOC2_XBR4X2;
        tile_class HNOC_X5IO_DDRMC {
            cell BLI_NMU[12];
            cell BLI_SCAN[2];
        }

        bel_slot HNOC_NIDB_V[2]: NOC_NIDB;
        bel_slot HNOC_NPP_RPTR[4]: NOC_NPP_RPTR;
        tile_class HNOC_SSIT, HNOC_SSIT2_V;

        bel_slot HNOC_NIDB_H: NOC_NIDB;
        tile_class HNOC_SSIT2_H;

        bel_slot HNOC_ME_NMU[4]: NOC_NMU128;
        bel_slot HNOC_ME_NSU[2]: NOC_NSU128;
        tile_class HNOC_ME, HNOC_ME_VNOC;

        bel_slot HNOC_ME2_NCRB[2]: NOC_NCRB;
        tile_class HNOC_ME2, HNOC_ME2_NCRB, HNOC_ME2_VNOC4, HNOC_ME2_CAP_W, HNOC_ME2_CAP_E {
            cell BLI_SCAN[2];
        }

        bel_slot HNOC_NPS4[2]: NOC_NPS4;
        bel_slot HNOC_NPS6[4]: NOC_NPS6;
        tile_class HNOC_HBM;
        tile_class HNOC_HBM_VNOC;

        bel_slot HNOC_NIC_XBR2X4: NOC2_XBR2X4;
        bel_slot HNOC_NIC_AXIS_NMU[4]: NOC_NMU256;
        bel_slot HNOC_NIC_AXIS_NSU[4]: NOC_NSU256;
        bel_slot HNOC_NIC_AXI_NMU: NOC_NMU256;
        bel_slot HNOC_NIC_AXI_NSU: NOC_NSU256;
        tile_class HNOC_NIC_CPM_CDX;
        tile_class HNOC_NIC_CPM_DPU {
            cell BLI_SCAN[2];
        }
        tile_class HNOC_NIC_BLI {
            cell BLI_NMU[6];
            cell BLI_NSU[8];
            cell BLI_SCAN[2];
        }
        tile_class HNOC_NIC_VNOC {
            cell BLI_SCAN[2];
        }
        tile_class HNOC_NIC_HNICX {
            cell BLI_SCAN[2];
        }
    }

    tile_slot HNOC_NCRB {
        bel_slot HNOC_NCRB[2]: NOC_NCRB;
        bel_slot HNOC_NCRB_SCAN: NOC2_SCAN;
        tile_class HNOC_NCRB;
        tile_class HNOC2_NCRB {
            cell BLI[2];
        }
    }

    tile_slot AIE {
        // in all AIE tiles (incl. AIE_INTF)
        bel_slot AIE_TILECTRL: AIE_TILECTRL;
        bel_slot AIE_FIFO[2]: AIE_FIFO;

        // AIE tiles proper
        bel_slot AIE_ARRAYSWITCH: AIE_ARRAYSWITCH;
        bel_slot AIE_ML_ARRAYSWITCH: AIE_ML_ARRAYSWITCH;
        bel_slot AIE_CORE: AIE_CORE;
        bel_slot AIE_ML_CORE: AIE_ML_CORE;
        bel_slot AIE_MEMGRP: AIE_MEMGRP;
        tile_class AIE, AIE_ML, AIE2PS;

        bel_slot AIE_ML_MEM_MEMGRP: AIE_ML_MEM_MEMGRP;
        bel_slot AIE_ML_MEM_ARRAYSWITCH: AIE_ML_MEM_ARRAYSWITCH;
        tile_class AIE_ML_MEM, AIE2PS_MEM;

        // in all AIE_INTF tiles
        bel_slot AIE_SHIMSWITCH: AIE_SHIMSWITCH;
        bel_slot AIE_ML_SHIMSWITCH: AIE_ML_SHIMSWITCH;
        bel_slot AIE2PS_SHIMSWITCH: AIE2PS_SHIMSWITCH;
        bel_slot AIE_SHIMTRACE: AIE_SHIMTRACE;
        bel_slot AIE2PS_SHIM_CNTRL: AIE2PS_SHIM_CNTRL;
        bel_slot AIE2PS_NMU_ARBIT: AIE2PS_NMU_ARBIT;

        tile_class AIE_SHIM, AIE_ML_SHIM;

        bel_slot AIE_PL: AIE_PL;
        bel_slot AIE_NOC: AIE_NOC;
        tile_class AIE_SHIM_PL, AIE_ML_SHIM_PL, AIE2PS_SHIM_NOC_PL {
            cell BLI[4];
        }
        tile_class AIE_SHIM_NOC_PL, AIE_ML_SHIM_NOC_PL {
            cell BLI[4];
            cell HSR_TEST;
        }
    }

    tile_slot AIE_PLL {
        bel_slot AIE_PLL: AIE_PLL;
        tile_class AIE_PLL, AIE_ML_PLL, AIE2PS_PLL {
            cell HSR;
            cell BLI[2];
        }
    }

    tile_slot AIE_TEST {
        bel_slot AIE_INTF_TEST: AIE_INTF_TEST;
        bel_slot AIE_ML_INTF_TEST: AIE_ML_INTF_TEST;
        bel_slot AIE2PS_INTF_TEST: AIE2PS_INTF_TEST;
        tile_class AIE_INTF_TEST, AIE_ML_INTF_TEST, AIE2PS_INTF_TEST {
            cell BLI[4];
        }
        tile_class AIE_INTF_TEST_NC, AIE_ML_INTF_TEST_NC, AIE2PS_INTF_TEST_NC;
    }

    connector_slot W {
        opposite E;
        connector_class PASS_W {
            pass X1_E1 = X1_E0;
            pass X2_E1 = X2_E0;
            pass X2_E2 = X2_E1;
            pass X4_E1 = X4_E0;
            pass X4_E2 = X4_E1;
            pass X4_E3 = X4_E2;
            pass X4_E4 = X4_E3;
            pass X10_E5_E7 = X10_E5[7];
        }
        connector_class TERM_W {
            reflect X1_E1 = X1_W0;
            reflect X2_E1 = X2_W0;
            reflect X2_E2 = X2_W1;
            reflect X4_E1 = X4_W0;
            reflect X4_E2 = X4_W1;
            reflect X4_E3 = X4_W2;
            reflect X4_E4 = X4_W3;
        }
    }
    connector_slot E {
        opposite W;
        connector_class PASS_E {
            pass X1_W1 = X1_W0;
            pass X2_W1 = X2_W0;
            pass X2_W2 = X2_W1;
            pass X4_W1 = X4_W0;
            pass X4_W2 = X4_W1;
            pass X4_W3 = X4_W2;
            pass X4_W4 = X4_W3;
        }
        connector_class TERM_E {
            reflect X1_W1 = X1_E0;
            reflect X2_W1 = X2_E0;
            reflect X2_W2 = X2_E1;
            reflect X4_W1 = X4_E0;
            reflect X4_W2 = X4_E1;
            reflect X4_W3 = X4_E2;
            reflect X4_W4 = X4_E3;
        }
    }
    connector_slot LW {
        opposite LE;
        connector_class PASS_LW {
            for i in 0..3 {
                pass "X6_E{i+1}" = "X6_E{i}";
            }
            for i in 0..5 {
                pass "X10_E{i+1}" = "X10_E{i}";
            }
        }
        connector_class TERM_LW {
            for i in 0..3 {
                reflect "X6_E{i+1}" = "X6_W{i}";
            }
            for i in 0..5 {
                reflect "X10_E{i+1}" = "X10_W{i}";
            }
        }
    }
    connector_slot LE {
        opposite LW;
        connector_class PASS_LE {
            for i in 0..3 {
                pass "X6_W{i+1}" = "X6_W{i}";
            }
            for i in 0..5 {
                pass "X10_W{i+1}" = "X10_W{i}";
            }
        }
        connector_class TERM_LE {
            for i in 0..3 {
                reflect "X6_W{i+1}" = "X6_E{i}";
            }
            for i in 0..5 {
                reflect "X10_W{i+1}" = "X10_E{i}";
            }
        }
    }
    connector_slot S {
        opposite N;
        connector_class PASS_S {
            pass SDQNODE_N = SDQNODE;
            pass X1_N1 = X1_N0;
            pass X2_N1 = X2_N0;
            pass X2_N2 = X2_N1;
            pass X4_N1 = X4_N0;
            pass X4_N2 = X4_N1;
            pass X4_N3 = X4_N2;
            pass X4_N4 = X4_N3;
            for i in 0..7 {
                pass "X7_N{i+1}" = "X7_N{i}";
            }
            for i in 0..12 {
                pass "X12_N{i+1}" = "X12_N{i}";
            }
            pass X10_E5_N7 = X10_E5[7];
        }
        connector_class TERM_S {
            reflect X1_N1 = X1_S0;
            reflect X2_N1 = X2_S0;
            reflect X2_N2 = X2_S1;
            reflect X4_N1 = X4_S0;
            reflect X4_N2 = X4_S1;
            reflect X4_N3 = X4_S2;
            reflect X4_N4 = X4_S3;
            for i in 0..7 {
                reflect "X7_N{i+1}" = "X7_S{i}";
            }
            for i in 0..12 {
                reflect "X12_N{i+1}" = "X12_S{i}";
            }
            reflect X10_E5_N7 = X6_E3[0];
            reflect SDQNODE_N[29] = SDQNODE[0];
            reflect SDQNODE_N[31] = SDQNODE[2];
            reflect SDQNODE_N[95] = SDQNODE[32];
            reflect SDQNODE_N[127] = SDQNODE[96];
        }
    }
    connector_slot N {
        opposite S;
        connector_class PASS_N {
            pass OUT_S = OUT;
            pass SDQNODE_S = SDQNODE;
            pass X1_S1 = X1_S0;
            pass X1_W0_S0 = X1_W0[0];
            pass X2_S1 = X2_S0;
            pass X2_S2 = X2_S1;
            pass X4_S1 = X4_S0;
            pass X4_S2 = X4_S1;
            pass X4_S3 = X4_S2;
            pass X4_S4 = X4_S3;
            pass X4_E3_S0 = X4_E3[0];
            for i in 0..7 {
                pass "X7_S{i+1}" = "X7_S{i}";
            }
            for i in 0..12 {
                pass "X12_S{i+1}" = "X12_S{i}";
            }
            pass X6_E3_S0 = X6_E3[0];
        }
        connector_class TERM_N {
            reflect X1_S1 = X1_N0;
            reflect X2_S1 = X2_N0;
            reflect X2_S2 = X2_N1;
            reflect X4_S1 = X4_N0;
            reflect X4_S2 = X4_N1;
            reflect X4_S3 = X4_N2;
            reflect X4_S4 = X4_N3;
            for i in 0..7 {
                reflect "X7_S{i+1}" = "X7_N{i}";
            }
            for i in 0..12 {
                reflect "X12_S{i+1}" = "X12_N{i}";
            }
            reflect X6_E3_S0 = X10_E5[7];
            reflect OUT_S[1] = X10_E5_E7;
            reflect SDQNODE_S[0] = SDQNODE[29];
            reflect SDQNODE_S[2] = SDQNODE[31];
            reflect SDQNODE_S[32] = SDQNODE[95];
            reflect SDQNODE_S[96] = SDQNODE[127];
            reflect SDQNODE_S[98] = SDQNODE[127];
        }
    }
    connector_slot INTF {
        opposite INTF;
        connector_class CLE_W, CLE_BLI_W {
            if connector_class CLE_W {
                reflect OUT_CLE = OUT;
            }
        }
        connector_class CLE_E, CLE_BLI_E {
            if connector_class CLE_E {
                reflect OUT_CLE = OUT;
            }
        }
    }

    connector_slot CLK_HALF_S {
        opposite CLK_HALF_N;
        connector_class CLK_HALF_S {
            pass GCLK_CLE_S = GCLK_CLE;
            pass GCLK_INTF_S = GCLK_INTF;
            pass OUT_BUFGCE_HDIO_S = OUT_BUFGCE_HDIO;
            pass OUT_DPLL_GT_S = OUT_DPLL_GT;
            pass OUT_GT_CLK_S = OUT_GT_CLK;
        }
    }
    connector_slot CLK_HALF_N {
        opposite CLK_HALF_S;
        connector_class CLK_HALF_N {
            pass IMUX_DPLL_HDIO_RCLK = IMUX_DPLL_HDIO_RCLK_S;
            pass IMUX_DPLL_CLKIN = IMUX_DPLL_CLKIN_S;
            pass IMUX_DPLL_CLKIN_DESKEW = IMUX_DPLL_CLKIN_DESKEW_S;
        }
    }

    connector_slot CLK_S {
        opposite CLK_N;
        connector_class CLK_S {
            pass CLK_PD_V_S = CLK_PD_V;
            pass CLK_HDISTR_LOC_DRV_S = CLK_HDISTR_LOC_DRV;
        }
        connector_class CLK_S_HSR;
    }
    connector_slot CLK_N {
        opposite CLK_S;
        connector_class CLK_N_VNOC {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
            pass CLK_VROUTE_N = CLK_VROUTE;
            pass CLK_VDISTR_N = CLK_VDISTR;
            pass CLK_VDISTR_LEV1_N = CLK_VDISTR_LEV1;
            pass CLK_VDISTR_LEV2_N = CLK_VDISTR_LEV2;
        }
        connector_class CLK_N_VNOC_LEV3 {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
            pass CLK_VROUTE_N = CLK_VROUTE;
            pass CLK_VDISTR_N = CLK_VDISTR;
            pass CLK_VDISTR_LEV1_N = CLK_VDISTR_LEV1;
            pass CLK_VDISTR_LEV2_N = CLK_VDISTR_LEV2;
            pass CLK_VDISTR_LEV3_N = CLK_VDISTR_LEV3;
        }
        connector_class CLK_N_GT {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
            pass CLK_VROUTE_N = CLK_VROUTE;
            pass CLK_VDISTR_N = CLK_VDISTR_S;
            pass CLK_VDISTR_LEV1_N = CLK_VDISTR_LEV1;
            pass CLK_VDISTR_LEV2_N = CLK_VDISTR_LEV2;
        }
        connector_class CLK_N_GT_LEV3 {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
            pass CLK_VROUTE_N = CLK_VROUTE;
            pass CLK_VDISTR_N = CLK_VDISTR_S;
            pass CLK_VDISTR_LEV1_N = CLK_VDISTR_LEV1;
            pass CLK_VDISTR_LEV2_N = CLK_VDISTR_LEV2;
            pass CLK_VDISTR_LEV3_N = CLK_VDISTR_LEV3;
        }
        connector_class CLK_N_SSIT {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
            pass CLK_VROUTE_N = CLK_VROUTE;
            pass CLK_VDISTR_N = CLK_VDISTR;
            pass CLK_VDISTR_LEV1_N = CLK_VDISTR_LEV1;
            pass CLK_VDISTR_LEV2_N = CLK_VDISTR_LEV2;
            pass CLK_VDISTR_LEV3_N = CLK_VDISTR_LEV3;
        }
        connector_class CLK_N_SSIT_HBM {
            pass CLK_HDISTR_LOC_DRV_N = CLK_HDISTR_LOC_DRV;
        }
        connector_class CLK_N_HSR {
            pass CLK_VROUTE_N = HSR_HROUTE;
            pass CLK_VDISTR_LEV2_N = HSR_HDISTR;
        }
    }

    connector_slot DPLL_GT {
        opposite DPLL_GT;
        connector_class DPLL_GT {
            reflect OUT_DPLL_GT = OUT_DPLL;
        }
        connector_class DPLL_GT_VDU {
            reflect OUT_DPLL_GT[0] = OUT_DPLL[0];
            reflect OUT_DPLL_GT[1] = OUT_DPLL[1];
            reflect OUT_DPLL_GT[4] = OUT_DPLL[4];
        }
    }

    connector_slot CLK_DRV {
        opposite CLK_DRV;
        connector_class CLK_DRV {
            reflect CLK_HDISTR_LOC_DRV = CLK_HDISTR_LOC;
        }
    }

    connector_slot HSR_W {
        opposite HSR_E;
        connector_class HSR_W {
            pass HSR_HROUTE_W = HSR_HROUTE;
            pass HSR_HDISTR_W = HSR_HDISTR;
        }
    }
    connector_slot HSR_E {
        opposite HSR_W;
        connector_class HSR_E;
    }

    connector_slot XPIO_W {
        opposite XPIO_E;
        connector_class XPIO_W {
            pass HSR_OUT_CCIO_E = HSR_OUT_CCIO;
            pass HSR_OUT_XPLL_E[0] = HSR_OUT_XPLL0[0];
            pass HSR_OUT_XPLL_E[1] = HSR_OUT_XPLL1[0];
        }
        connector_class X5IO_W {
            pass HSR_OUT_CCIO_E[0] = HSR_OUT_X5IO_CCIO_NB[0];
            pass HSR_OUT_CCIO_E[1] = HSR_OUT_X5IO_CCIO_NB[1];
            pass HSR_OUT_CCIO_E[2] = HSR_OUT_X5IO_CCIO[0];
            pass HSR_OUT_CCIO_E[3] = HSR_OUT_X5IO_CCIO[1];
            pass HSR_X5PLL_CASC_E1[0] = HSR_OMUX_X5PLL_CASC_NB[0];
            pass HSR_X5PLL_CASC_E1[1] = HSR_OMUX_X5PLL_CASC[1];
        }
        connector_class X5IO_W_SINGLE {
            pass HSR_OUT_CCIO_E[0] = HSR_OUT_X5IO_CCIO[0];
            pass HSR_OUT_CCIO_E[1] = HSR_OUT_X5IO_CCIO[1];
            pass HSR_X5PLL_CASC_E1[0] = HSR_OMUX_X5PLL_CASC_NB[0];
            pass HSR_X5PLL_CASC_E1[1] = HSR_OMUX_X5PLL_CASC[1];
        }
    }
    connector_slot XPIO_E {
        opposite XPIO_W;
        connector_class XPIO_E {
            pass HSR_OUT_CCIO_W = HSR_OUT_CCIO;
            pass HSR_OUT_XPLL_W[0] = HSR_OUT_XPLL0[0];
            pass HSR_OUT_XPLL_W[1] = HSR_OUT_XPLL1[0];
        }
        connector_class X5IO_E {
            pass HSR_OUT_CCIO_W[0] = HSR_OUT_X5IO_CCIO[0];
            pass HSR_OUT_CCIO_W[1] = HSR_OUT_X5IO_CCIO[1];
            pass HSR_OUT_CCIO_W[2] = HSR_OUT_X5IO_CCIO_NB[0];
            pass HSR_OUT_CCIO_W[3] = HSR_OUT_X5IO_CCIO_NB[1];
            pass HSR_X5PLL_CASC_W1[0] = HSR_OMUX_X5PLL_CASC[0];
            pass HSR_X5PLL_CASC_W1[1] = HSR_OMUX_X5PLL_CASC_NB[1];
        }
    }

    connector_slot X5IO_PAIR {
        opposite X5IO_PAIR;
        connector_class X5IO_PAIR {
            pass HSR_OMUX_X5IO_RXCLK_NB = HSR_OMUX_X5IO_RXCLK;
            pass HSR_OUT_X5PLL_PHY_NB = HSR_OUT_X5PLL_PHY;
            pass HSR_OUT_X5IO_CCIO_NB = HSR_OUT_X5IO_CCIO;
            pass HSR_X5PLL_HS_DESKEW_CLKIN_NB = HSR_X5PLL_HS_DESKEW_CLKIN;
            pass HSR_X5PLL_HS_DESKEW_CLKFB_NB = HSR_X5PLL_HS_DESKEW_CLKFB;
            pass HSR_OMUX_X5PLL_CASC_NB = HSR_OMUX_X5PLL_CASC;
        }
    }

    connector_slot X5IO_HS_DESKEW_CLKIN {
        opposite X5IO_HS_DESKEW_CLKIN_REV;
        connector_class X5IO_HS_DESKEW_CLKIN {
            pass HSR_X5PLL_HS_DESKEW_CLKIN = HSR_OUT_X5PHIO_DIV64_CLK;
        }
    }
    connector_slot X5IO_HS_DESKEW_CLKIN_REV {
        opposite X5IO_HS_DESKEW_CLKIN;
        connector_class X5IO_HS_DESKEW_CLKIN_REV;
    }
    connector_slot X5IO_HS_DESKEW_CLKFB {
        opposite X5IO_HS_DESKEW_CLKFB_REV;
        connector_class X5IO_HS_DESKEW_CLKFB {
            pass HSR_X5PLL_HS_DESKEW_CLKFB = HSR_OUT_X5PHIO_DIV64_CLK;
        }
    }
    connector_slot X5IO_HS_DESKEW_CLKFB_REV {
        opposite X5IO_HS_DESKEW_CLKFB;
        connector_class X5IO_HS_DESKEW_CLKFB_REV;
    }
}

pub mod wiredata {
    use prjcombine_entity::id::EntityStaticRange;
    use prjcombine_interconnect::db::WireSlotId;

    use crate::defs::wires;

    pub const X1_W: [EntityStaticRange<WireSlotId, 16>; 2] = [wires::X1_W0, wires::X1_W1];
    pub const X1_E: [EntityStaticRange<WireSlotId, 16>; 2] = [wires::X1_E0, wires::X1_E1];
    pub const X1_S: [EntityStaticRange<WireSlotId, 16>; 2] = [wires::X1_S0, wires::X1_S1];
    pub const X1_N: [EntityStaticRange<WireSlotId, 16>; 2] = [wires::X1_N0, wires::X1_N1];

    pub const X2_W: [EntityStaticRange<WireSlotId, 8>; 3] =
        [wires::X2_W0, wires::X2_W1, wires::X2_W2];
    pub const X2_E: [EntityStaticRange<WireSlotId, 8>; 3] =
        [wires::X2_E0, wires::X2_E1, wires::X2_E2];
    pub const X2_S: [EntityStaticRange<WireSlotId, 8>; 3] =
        [wires::X2_S0, wires::X2_S1, wires::X2_S2];
    pub const X2_N: [EntityStaticRange<WireSlotId, 8>; 3] =
        [wires::X2_N0, wires::X2_N1, wires::X2_N2];

    pub const X4_W: [EntityStaticRange<WireSlotId, 8>; 5] = [
        wires::X4_W0,
        wires::X4_W1,
        wires::X4_W2,
        wires::X4_W3,
        wires::X4_W4,
    ];
    pub const X4_E: [EntityStaticRange<WireSlotId, 8>; 5] = [
        wires::X4_E0,
        wires::X4_E1,
        wires::X4_E2,
        wires::X4_E3,
        wires::X4_E4,
    ];
    pub const X4_S: [EntityStaticRange<WireSlotId, 8>; 5] = [
        wires::X4_S0,
        wires::X4_S1,
        wires::X4_S2,
        wires::X4_S3,
        wires::X4_S4,
    ];
    pub const X4_N: [EntityStaticRange<WireSlotId, 8>; 5] = [
        wires::X4_N0,
        wires::X4_N1,
        wires::X4_N2,
        wires::X4_N3,
        wires::X4_N4,
    ];

    pub const X6_W: [EntityStaticRange<WireSlotId, 8>; 4] =
        [wires::X6_W0, wires::X6_W1, wires::X6_W2, wires::X6_W3];
    pub const X6_E: [EntityStaticRange<WireSlotId, 8>; 4] =
        [wires::X6_E0, wires::X6_E1, wires::X6_E2, wires::X6_E3];
    pub const X7_S: [EntityStaticRange<WireSlotId, 8>; 8] = [
        wires::X7_S0,
        wires::X7_S1,
        wires::X7_S2,
        wires::X7_S3,
        wires::X7_S4,
        wires::X7_S5,
        wires::X7_S6,
        wires::X7_S7,
    ];
    pub const X7_N: [EntityStaticRange<WireSlotId, 8>; 8] = [
        wires::X7_N0,
        wires::X7_N1,
        wires::X7_N2,
        wires::X7_N3,
        wires::X7_N4,
        wires::X7_N5,
        wires::X7_N6,
        wires::X7_N7,
    ];
    pub const X10_W: [EntityStaticRange<WireSlotId, 8>; 6] = [
        wires::X10_W0,
        wires::X10_W1,
        wires::X10_W2,
        wires::X10_W3,
        wires::X10_W4,
        wires::X10_W5,
    ];
    pub const X10_E: [EntityStaticRange<WireSlotId, 8>; 6] = [
        wires::X10_E0,
        wires::X10_E1,
        wires::X10_E2,
        wires::X10_E3,
        wires::X10_E4,
        wires::X10_E5,
    ];

    pub const X12_S: [EntityStaticRange<WireSlotId, 8>; 13] = [
        wires::X12_S0,
        wires::X12_S1,
        wires::X12_S2,
        wires::X12_S3,
        wires::X12_S4,
        wires::X12_S5,
        wires::X12_S6,
        wires::X12_S7,
        wires::X12_S8,
        wires::X12_S9,
        wires::X12_S10,
        wires::X12_S11,
        wires::X12_S12,
    ];
    pub const X12_N: [EntityStaticRange<WireSlotId, 8>; 13] = [
        wires::X12_N0,
        wires::X12_N1,
        wires::X12_N2,
        wires::X12_N3,
        wires::X12_N4,
        wires::X12_N5,
        wires::X12_N6,
        wires::X12_N7,
        wires::X12_N8,
        wires::X12_N9,
        wires::X12_N10,
        wires::X12_N11,
        wires::X12_N12,
    ];

    pub const IRI_CLK: [WireSlotId; 4] = [
        wires::IRI0_CLK,
        wires::IRI1_CLK,
        wires::IRI2_CLK,
        wires::IRI3_CLK,
    ];
    pub const IRI_RST: [WireSlotId; 4] = [
        wires::IRI0_RST,
        wires::IRI1_RST,
        wires::IRI2_RST,
        wires::IRI3_RST,
    ];
    pub const IRI_CE: [EntityStaticRange<WireSlotId, 4>; 4] = [
        wires::IRI0_CE,
        wires::IRI1_CE,
        wires::IRI2_CE,
        wires::IRI3_CE,
    ];
    pub const IRI_IMUX: [EntityStaticRange<WireSlotId, 24>; 4] = [
        wires::IRI0_IMUX,
        wires::IRI1_IMUX,
        wires::IRI2_IMUX,
        wires::IRI3_IMUX,
    ];
    pub const IRI_IMUX_DELAY: [EntityStaticRange<WireSlotId, 24>; 4] = [
        wires::IRI0_IMUX_DELAY,
        wires::IRI1_IMUX_DELAY,
        wires::IRI2_IMUX_DELAY,
        wires::IRI3_IMUX_DELAY,
    ];
}
