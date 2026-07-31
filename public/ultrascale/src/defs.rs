use prjcombine_tablegen::target_defs;

target_defs! {
    variant ultrascale;
    variant ultrascaleplus;

    bel_class SLICE {
        input A1, A2, A3, A4, A5, A6, AX, A_I;
        input B1, B2, B3, B4, B5, B6, BX, B_I;
        input C1, C2, C3, C4, C5, C6, CX, C_I;
        input D1, D2, D3, D4, D5, D6, DX, D_I;
        input E1, E2, E3, E4, E5, E6, EX, E_I;
        input F1, F2, F3, F4, F5, F6, FX, F_I;
        input G1, G2, G3, G4, G5, G6, GX, G_I;
        input H1, H2, H3, H4, H5, H6, HX, H_I;
        output A_O, AMUX, AQ, AQ2;
        output B_O, BMUX, BQ, BQ2;
        output C_O, CMUX, CQ, CQ2;
        output D_O, DMUX, DQ, DQ2;
        output E_O, EMUX, EQ, EQ2;
        output F_O, FMUX, FQ, FQ2;
        output G_O, GMUX, GQ, GQ2;
        output H_O, HMUX, HQ, HQ2;
        input CLK1, CLK2;
        input SRST1, SRST2;
        input CKEN1, CKEN2, CKEN3, CKEN4;
        // SLICEM only
        input LCLK, WCKEN;

        // TODO attribute
    }

    bel_class LAGUNA {
        input TXD[6];
        input TX_CLK, TX_SR, TX_CE;

        output RXOUT[6];
        input RX_CLK, RX_SR, RX_CE;

        // TODO attributes
    }

    bel_class BRAM {
        input CLKAL, CLKAU, CLKBL, CLKBU;
        input ENAL, ENAU, ENBL, ENBU;
        // REGCLK gone on ultrascale+
        input REGCLKAL, REGCLKAU, REGCLKBL, REGCLKBU;
        input REGCEAL, REGCEAU, REGCEBL, REGCEBU;
        input RSTRAMAL, RSTRAMAU, RSTRAMBL, RSTRAMBU;
        input RSTREGAL, RSTREGAU, RSTREGBL, RSTREGBU;
        input WEAL[4], WEAU[4], WEBL[4], WEBU[4];

        input ADDRAL[15], ADDRAU[15], ADDRBL[15], ADDRBU[15];
        input ADDRENAL, ADDRENAU, ADDRENBL, ADDRENBU;

        input DIAL[16], DIAU[16], DIBL[16], DIBU[16];
        input DIPAL[2], DIPAU[2], DIPBL[2], DIPBU[2];
        output DOAL[16], DOAU[16], DOBL[16], DOBU[16];
        output DOPAL[2], DOPAU[2], DOPBL[2], DOPBU[2];

        input CASDIMUXAL, CASDIMUXAU, CASDIMUXBL, CASDIMUXBU;
        input CASDOMUXAL, CASDOMUXAU, CASDOMUXBL, CASDOMUXBU;
        input CASDOMUXENAL, CASDOMUXENAU, CASDOMUXENBL, CASDOMUXENBU;
        input CASOREGIMUXAL, CASOREGIMUXAU, CASOREGIMUXBL, CASOREGIMUXBU;
        input CASOREGIMUXENAL, CASOREGIMUXENAU, CASOREGIMUXENBL, CASOREGIMUXENBU;

        input INJECTSBITERR, INJECTDBITERR;
        output SBITERR, DBITERR;
        input ECCPIPECE;
        output ECCPARITY[8];
        output RDADDRECC[9];

        input RSTFIFO;
        output EMPTY, FULL, PROGEMPTY, PROGFULL;
        output RDCOUNT[14];
        output RDERR;
        output RDRSTBUSY;
        output WRCOUNT[14];
        output WRERR;
        output WRRSTBUSY;

        input SLEEPL, SLEEPU;

        input TSTBRAMRST;
        input TSTCNT[13];
        input TSTFLAGIN;
        input TSTOFF;
        input TSTRDCNTOFF;
        input TSTRDOS[13];
        input TSTRINGEN;
        output TSTRINGOUTDIV4;
        input TSTRINGSTART;
        output TSTSTAGEVALID;
        input TSTWRCNTOFF;
        input TSTWROS[13];
        output TST_QUIETENAL, TST_QUIETENAU, TST_QUIETENBL, TST_QUIETENBU;
        // added on ultrascale+
        output TST_SLEEP_CNTL;

        // TODO attributes
    }

    bel_class HARD_SYNC {
        input CLK, SR;
        input DIN;
        output DOUT;

        // TODO attributes
    }

    bel_class DSP {
        input CLK;

        input A[30];
        input RSTA, CEA1, CEA2;

        input B[18];
        input RSTB, CEB1, CEB2;

        input C[48];
        input RSTC, CEC;

        input DIN[27];
        input RSTD, CED;

        input INMODE[5];
        input RSTINMODE, CEINMODE;
        input CEAD;

        input RSTM, CEM;

        input CARRYIN;
        input CARRYINSEL[3];
        input RSTALLCARRYIN, CECARRYIN;

        input OPMODE[9];
        input RSTCTRL, CECTRL;

        input ALUMODE[4];
        input RSTALUMODE, CEALUMODE;

        input RSTP, CEP;

        output P[48];
        output CARRYOUT[4];
        output XOROUT[8];

        output OVERFLOW, UNDERFLOW;
        output PATTERN_DETECT;
        output PATTERN_B_DETECT;

        // ultrascale+ only
        input MSCAN_ENABLE;
        input MSCAN_IN;
        output MSCAN_OUT;

        // TODO attributes
    }

    bel_class URAM {
        input CLK;
        input RST_A, RST_B;
        input EN_A, EN_B;
        input OREG_CAS_CE_A, OREG_CAS_CE_B;
        input OREG_CE_A, OREG_CE_B;
        input OREG_ECC_CE_A, OREG_ECC_CE_B;
        input RDB_WR_A, RDB_WR_B;
        output RDACCESS_A, RDACCESS_B;
        input BWE_A[9], BWE_B[9];
        input ADDR_A[23], ADDR_B[23];
        input DIN_A[72], DIN_B[72];
        output DOUT_A[72], DOUT_B[72];

        input INJECT_DBITERR_A, INJECT_DBITERR_B;
        input INJECT_SBITERR_A, INJECT_SBITERR_B;
        output SBITERR_A, SBITERR_B;
        output DBITERR_A, DBITERR_B;

        input DEEPSLEEP;
        input SHUTDOWN;
        input SLEEP;

        output TST_DEEPSLEEP_OUT;
        input TST_RING_ENB;
        output TST_RING_OUT;
        input TST_RING_STARTB;
        output TST_SHUTDOWN_OUT;
        output TST_SLEEP_OUT;

        // TODO: attributes
    }

    bel_class ABUS_SWITCH {
        // ultrascale+ only; not present on _CFG and _HBM slots
        input TEST_ANALOGBUS_SEL;
        // TODO: attributes
    }

    bel_class VBUS_SWITCH {
        input TEST_ANALOGBUS_SEL;
        // TODO: attributes
    }

    // A quick guide to Ultrascale I/O banks.
    //
    // Ultrascale devices have many kinds of I/O:
    //
    // - dedicated configuration pins (bank 0)
    // - dedicated PS pins, including:
    //   - MIO (multiplexed I/O; banks 500, 501, 502)
    //   - assorted special-purpose pins (bank 503)
    //   - DDR controller pins (bank 504)
    //   - GTR, multi-gigabit transceivers integrated into the PS (4 GTRs per device; bank 505)
    // - I/O banks proper (banks 1-119), which come in several kinds:
    //   - HPIO and HRIO
    //     - HRIO is ultrascale only and supports 1.2V-3.3V
    //     - HPIO supports 1.0V-1.8V
    //     - located either at an edge column or in an inner column
    //     - HPIO and HRIO can be mixed within a column; if at the edge, the column may also be
    //       shared with multi-gigabit transceivers
    //     - banks are natively 26 pins each and come in pairs; the VCCO of the two paired banks is almost
    //       always wired together at package level, but there are rare packages with separate power
    //       for one of the bank pairs
    //     - a bank vertically occupies half a clock region
    //     - associated with CMT tiles (one CMT tile : pair of HPIO/HRIO banks) and XIPHY blocks
    //     - each bank is made of two 13-pin bytes
    //     - each byte is made of two nibbles; a lower 6-pin nibble and an upper 7-bit nibble
    //     - a nibble is made of 3 differential pairs; the upper nibble additionally has one unpaired pin
    //     - logically, the pins within the byte are numbered:
    //       - 0-5: lower nibble (differential)
    //       - 6-11: upper nibble (differential)
    //       - 12: upper nibble (single-ended)
    //     - physically, the layout is actually (from south): 0-5, 12, 6-11
    //     - the four differential pairs closest to the clock row are capable of driving BUFGs; these are:
    //       - byte 1 IOBs 8 and 10
    //       - byte 2 IOBs 0 and 2
    //   - HDIO (ultrascaleplus only)
    //     - supports 1.2V-3.3V
    //     - comes in three different subkinds:
    //       - small (gen1 devices)
    //         - located within hard IP columns (a bank fits into a single hard IP "slot")
    //         - a bank vertically occupies a whole clock region
    //         - a bank has 24 pins, which are grouped into 12 differential pairs
    //         - composed of two vertical "chains" of 6 diffpairs each:
    //           - chain 0 to the south of the clock row
    //           - chain 1 to the north of the clock row
    //         - a bank is associated with four BUFGCEs
    //         - the four diffpairs closest to the clock row are capable of driving BUFGCEs; these are:
    //           - IOB 8 and 10 (chain 0 IOB 8 and 10)
    //           - IOB 12 and 14 (chain 1 IOB 0 and 2)
    //       - large inner (gen2 devices)
    //         - located in its own inner column
    //         - a bank vertically occupies a whole clock region
    //         - a bank has 42 pins, which are grouped into 21 differential pairs
    //         - composed of four vertical "chains" of varying sizes:
    //           - south-west chain 0 of 6 diffpairs (IOBs 0-11)
    //           - south-east chain 1 of 5 diffpairs (IOBs 12-21)
    //           - north-west chain 2 of 6 diffpairs (IOBs 22-33)
    //           - north-east chain 3 of 4 diffpairs (IOBs 34-41)
    //         - a bank is associated with four BUFGCEs
    //         - the four diffpairs in west chains closest to the clock row are capable of driving BUFGCEs; these are:
    //           - IOB 8 and 10 (chain 0 IOB 8 and 10)
    //           - IOB 22 and 24 (chain 2 IOB 0 and 2)
    //       - large edge (gen2 devices)
    //         - located at an edge column; the column may be shared with other kinds of I/O banks
    //         - a bank vertically occupies half a clock region; banks come in pairs, but have independent VCCO
    //         - a bank has 42 pins, which are grouped into 21 differential pairs
    //         - a given bank pair may or may not be associated with a CMT tile
    //         - combosed of three side-by-side vertical "chains" of 7 diffpairs each:
    //           - chain 0: inner
    //           - chain 1: middle
    //           - chain 2: outer (closest to the edge)
    //         - the two diffpairs (four diffpairs in total across a bank pair) in inner chains closest to
    //           the clock row are capable of driving BUFGCEs; these are:
    //           - south bank: IOBs 10 and 12
    //           - norht bank: IOBs 0 and 2
    //   - XP5IO (gen2 devices; 66 pins per bank)
    //     - supports 1.0V-1.5V
    //     - located at en edge column; the column may be shared with other kinds of I/O banks
    //     - a bank vertically occupies a whole clock region
    //     - a bank has 66 pins total
    //     - a bank is made of 11 nibbles
    //     - a nibble is made of 3 differential pairs
    //     - each bank is associated with a CMT tile and an LPDDRMC block
    // - multi-gigabit transceivers and RF DAC/ADCs (banks 120-319), which come in several kinds:
    //   - GTH, GTY, GTF transceiver quads
    //     - each bank has four channels and two differential clock inputs
    //     - a channel has a RX differential pair and a TX differential pair
    //     - GTF is an obscure undocumented transceiver only used on the xcvu2p device
    //   - GTM transceiver dual (ultrascaleplus only)
    //     - each bank has two channels and one differential clock input
    //   - HSADC, RFADC high-speed analog input
    //     - HSADC is first-generation ADC, RFADC is second-generation
    //     - each bank has four differential input channels
    //     - depending on package, the four channels can be either bonded out separately,
    //       or tied together in pairs at package level; the tied configuration reduces the effective
    //       channel count to two per bank, but doubles the effective sampling rate
    //   - HSDAC, RFDAC high-speed analog output
    //     - HSDAC is first-generation DAC, RFDAC is second-generation
    //     - each bank has four differential output channels
    // - HBM (sometimes referred to as banks 43, 83)
    //   - the memory is located on-package, and the I/O lines are not externally accessible
    //   - the HBM I/O lines aren't directly accessible from the fabric either; the only interface
    //     to HBM is through the integrated memory controller
    //   - there are up to two stacks of HBM per device, located on the south edge of the device
    //   - there are 16 *channels* per HBM stack; each channel consists of an AXI bus and a control APB bus
    //   - a single channel replaces 3 DSP tiles at the bottom of a DSP columns

    bel_class HDIOB {
        input IBUF_DISABLE;
        input PD_EN;
        input PU_EN;
        input KEEPER_EN;
        input DYNAMIC_DCI_TS;

        output CLKPAD;

        pad PAD: inout;

        // TODO: attributes
    }

    bel_class HDIOB_DIFF_IN {
        // TODO: attributes
    }

    bel_class HDIOLOGIC {
        output IPFF_BYPASS;
        input IPFF_CK, IPFF_CK_C;
        input IPFF_RST;
        input IPFF_CE;
        output IPFF_Q1, IPFF_Q2;

        input OPFF_CK;
        input OPFF_RST;
        input OPFF_CE;
        input OPFF_D1, OPFF_D2;
        input TFF_D1, TFF_D2;

        // TODO: attributes
    }

    bel_class HDLOGIC_CSSD {
        input FBRC2BSCAN_CTRL;
        input FBRC2BSCAN_DATA;

        input FBRC2IOL_RX_CLK;
        input FBRC2IOL_RX_RST;
        input FBRC2IOL_RX_SET;
        input FBRC2IOL_TX_DATA_ODD;
        input FBRC2IO_SCAN_EN;
        output IO2FBRC_SCAN_OUT;

        // TODO: attributes
    }

    bel_class HDIO_VREF {
        // TODO: attributes
    }

    bel_class HDIO_BIAS {
        // TODO: attributes
    }

    bel_class HRIOB {
        input IBUF_DISABLE;
        input PD_EN;
        input PU_EN;
        input KEEPER_EN;

        output CLKPAD;

        pad PAD: inout;

        // TODO: attributes
    }

    bel_class HRIOB_DIFF_IN {
        // TODO: attributes
    }

    bel_class HRIOB_DIFF_OUT {
        input TERM_EN_TEST;

        // TODO: attributes
    }

    bel_class HPIOB {
        input IBUF_DISABLE;
        input PD_EN;
        input PU_EN;
        input KEEPER_EN;
        input OSC_EN;
        input OSC[4];

        output CLKPAD;

        pad PAD: inout;

        // TODO: attributes
    }

    bel_class HPIOB_DIFF_IN {
        input OSC_EN;
        input OSC[4];
        // u+ only
        input HSRX_DISABLE;
        input LPRX_DISABLE;
        output LPRX_O_P;
        output LPRX_O_N;
        input CTLE_IN_1;

        // TODO: attributes
    }

    bel_class HPIOB_DIFF_OUT {
        input TERM_EN;
        // u+ only
        input LPTX_I_P;
        input LPTX_I_N;
        input LPTX_T;

        // TODO: attributes
    }

    bel_class HPIOB_DCI {
        output DCI_HL_B[2];

        // TODO: attributes
    }

    bel_class HPIO_VREF {
        input FABRIC_VREF1_TUNE[7];
        input FABRIC_VREF2_TUNE[7];

        // TODO: attributes
    }

    bel_class HPIO_BIAS {
        // TODO: attributes
    }

    bel_class HPIO_ZMATCH {
        input OK_TO_UPDATE;
        input DCI_TEST_I[24];
        output DCI_TEST_O[24];

        // TODO: attributes
    }

    bel_class HPIO_PRBS {
        // TODO: attributes
    }

    bel_class BITSLICE {
        input CLK;

        input DYN_DCI_OUT_INT;

        input FIFO_RDCLK;
        input FIFO_RDEN;
        output FIFO_EMPTY;
        output FIFO_WRCLK;

        input RX_CLK_C, RX_CLK_C_B;
        input RX_RST;
        input RX_DATAIN1;
        output RX_Q[8];

        input RX_RST_DLY;
        input RX_CE_IDELAY;
        input RX_CE_IFD;
        input RX_INC;
        input RX_LD;
        input RX_EN_VTC;
        input RX_CNTVALUEIN[9];
        output RX_CNTVALUEOUT[9];

        input TX_OCLK;
        input TX_RST;
        input TX_D[8];
        input TX_T;
        output TX_T_OUT;

        input TX_RST_DLY;
        input TX_CE_ODELAY;
        input TX_CE_OFD;
        input TX_INC;
        input TX_LD;
        input TX_EN_VTC;
        input TX_CNTVALUEIN[9];
        output TX_CNTVALUEOUT[9];

        // TODO: attributes
    }

    bel_class BITSLICE_T {
        input CLK;
        input RST;

        input RST_DLY;
        input CE_ODELAY;
        input CE_OFD;
        input INC;
        input LD;
        input EN_VTC;
        input CNTVALUEIN[9];
        output CNTVALUEOUT[9];

        // TODO: attributes
    }

    bel_class BITSLICE_CONTROL {
        input CLK;
        input RST;

        input T_B[4];
        input RDEN[4];
        input RDCS0[4];
        input RDCS1[4];
        input WRCS0[4];
        input WRCS1[4];
        output PHY_RDY;

        input REFCLK;
        input CTRL_DLY_TEST_IN;
        output CTRL_DLY_TEST_OUT;
        input EN_VTC;
        output MASTER_PD_OUT;
        output FIXDLY_RDY;

        // TODO: attributes
    }

    bel_class XIPHY_BYTE {
        input RIU_NIBBLE_SEL[2];
        input RIU_WR_EN;
        input RIU_ADDR[6];
        input RIU_WR_DATA[16];
        output RIU_RD_VALID;
        output RIU_RD_DATA[16];

        input CLB2PHY_DBG_CLK_STOP_FLG_DLY_OUT;
        input CLB2PHY_DBG_CLK_STOP_FLG_OUT;
        input CLB2PHY_DBG_CT_START_EN;
        output PHY2CLB_DBG_CLK_STOP_FLG_DLY_OUT;
        output PHY2CLB_DBG_CLK_STOP_FLG_OUT;
        output PHY2CLB_DBG_CLK_STOP_OUT;

        input SCAN_CLK_DIV2;
        input SCAN_CLK_DIV4;
        input SCAN_CLK_SDR;
        input SCAN_EN_B;
        input SCAN_IN[8];
        input SCAN_MODE_B;
        input SCAN_RST_MASK_B;
        output SCAN_OUT[8];

        input TEST_DIV2_CLK_SEL_B;
        input TEST_DIV4_CLK_SEL_B;
        input TEST_SDR_CLK_SEL_B;
        input TEST_SPARE_B[4];

        // TODO: attributes
    }

    bel_class XP5IOB {
        input IBUF_DISABLE;
        // IOB0: doubles as LP_TX_T
        input PD_EN;
        // doubles as LP_TX_O
        input PU_EN;
        input KEEPER_EN;
        input DYNAMIC_DCI_TS;

        output LP_I;
        output DFT;

        pad PAD: inout;
    }

    bel_class XP5IOB_PAIR {
        // doubles as LP_RX_DIS
        input TERM_EN;
    }

    bel_class XP5IO_BITSLICE {
        input RXBIT_RST;
        output RD_DQ[8];

        input TXBIT_RST;
        input WR_DQ[8];
        input T_TXBIT;

        input IDELAY_EN_VTC;
        input ODELAY_EN_VTC;
        input IODELAY_CE;
        input IODELAY_INC;
        input IODELAY_LD;
        input IODELAY_SEL;
        input IODELAY_CNTVALUEIN[9];
        output IODELAY_CNTVALUEOUT[9];
    }

    // corresponds to a nibble of 6 pins, or 3 diffpairs
    bel_class X5PHY {
        input DIV_CLK;

        input CTRL_CLK;
        input RST;
        input DLYCTL_EN_VTC;
        input FIFO_CLK;
        input FIFO_RDEN;
        input RDEN[4];
        input WREN[4];
        output FIFO_EMPTY;
        output FIXDLY_RDY;
        output GT_STATUS;
        output MASTER_PD;
        output PHY_RDY;

        output FIFO_WRCLK;
        output CCIO;

        input RIU_ADDR[8];
        input RIU_NIBBLE_SEL;
        input RIU_WR_EN;
        input RIU_WR_DATA[16];
        output RIU_RD_DATA[16];
        output RIU_VALID;
    }

    bel_class XP5IO_VREF {
    }

    bel_class LPDDRMC {
        for i in 0..2 {
            input "AXIA{i}_CLK";

            output "AXIA{i}_ARREADY";
            input "AXIA{i}_ARVALID";
            input "AXIA{i}_ARADDR"[44];
            input "AXIA{i}_ARADDR_PAR"[2];
            input "AXIA{i}_ARID"[8];
            input "AXIA{i}_ARLEN"[8];
            input "AXIA{i}_ARUSER"[11];
            input "AXIA{i}_ARBURST"[2];
            input "AXIA{i}_ARLOCK";
            input "AXIA{i}_ARPROT"[3];
            input "AXIA{i}_ARSIZE"[3];

            output "AXIA{i}_AWREADY";
            input "AXIA{i}_AWVALID";
            input "AXIA{i}_AWADDR"[44];
            input "AXIA{i}_AWADDR_PAR"[2];
            input "AXIA{i}_AWID"[8];
            input "AXIA{i}_AWLEN"[8];
            input "AXIA{i}_AWUSER"[11];
            input "AXIA{i}_AWBURST"[2];
            input "AXIA{i}_AWLOCK";
            input "AXIA{i}_AWPROT"[3];
            input "AXIA{i}_AWSIZE"[3];

            input "AXIA{i}_BREADY";
            output "AXIA{i}_BVALID";
            output "AXIA{i}_BID"[8];
            output "AXIA{i}_BRESP"[2];

            input "AXIA{i}_RREADY";
            output "AXIA{i}_RVALID";
            output "AXIA{i}_RDATA"[256];
            output "AXIA{i}_RDATA_PAR"[8];
            output "AXIA{i}_RID"[8];
            output "AXIA{i}_RLAST";
            output "AXIA{i}_RPOISON";
            output "AXIA{i}_RRESP"[2];

            output "AXIA{i}_WREADY";
            input "AXIA{i}_WVALID";
            input "AXIA{i}_WDATA"[256];
            input "AXIA{i}_WDATA_PAR"[8];
            input "AXIA{i}_WSTRB"[32];
            input "AXIA{i}_WLAST";
            input "AXIA{i}_WPOISON";
        }

        input APB_CLK;
        input APB_RST_N;
        input APB_PENABLE;
        input APB_PWRITE;
        input APB_PSELX;
        input APB_PADDR[32];
        output APB_PREADY;
        output APB_PRDATA[32];
        output APB_PSLVERR;
        input APB_PWDATA[32];

        input DMC_BLI2ILA_ACK;
        input DMC_BLI2ILA_TRIG;
        output DMC_ILA2BLI_ACK;
        output DMC_ILA2BLI_TRIG;

        input DMC_BLI2UB_TRIG;
        input DMC_BLI2UB_ACK;
        output DMC_UB2BLI_TRIG;
        output DMC_UB2BLI_ACK;
        input DMC_BLI2UB_TRACE_CLK;
        input DMC_BLI2UB_TRACE_RST_N;
        input DMC_BLI2UB_TRACE_TREADY;
        output DMC_UB2BLI_TRACE_TVALID;
        output DMC_UB2BLI_TRACE_TDATA[16];
        input DMC_BLI2UB_UART_RX;
        output DMC_UB2BLI_UART_TX;

        input DMC_BLOCK_PERIODIC_CAL;
        output DMC_CAL_BUSY;
        output DMC_CAL_DONE;
        output DMC_CSSD_CLKSTP_BCAST;
        input DMC_CSSD_TRIG_IN_N_EXT;
        input DMC_EXMON_CLEAR_IN;
        output DMC_EXMON_CLEAR_OUT;
        output DMC_LPDDRMC_SPARE0[6];
        output DMC_LPDDRMC_SPARE1;
        input DMC_LPDDRMC_SPARE2;
        output DMC_REF_ACK_0[4];
        output DMC_REF_ACK_1[2];
        input DMC_REF_RANK_EN_0[4];
        input DMC_REF_RANK_EN_1[2];
        output DMC_REF_USR_PORT_AVAILABLE;
        input DMC_SCAN_CHNL_IN_EXT[13];
        input DMC_SCAN_CHNL_MASK_IN_EXT[13];
        output DMC_SCAN_CHNL_OUT_EXT_DDRMC[10];
        input DMC_SCAN_CLK_N_EXT;
        input DMC_SCAN_CNTRL_CHNL_IN_EXT_DDRMC[2];
        output DMC_SCAN_CNTRL_CHNL_OUT_EXT_DDRMC[2];
        input DMC_SCAN_EDT_UPDT_N_EXT;
        input DMC_SCAN_EN_N_EXT;
        input DMC_SCAN_MODE_RST_N_EXT;

        input UBLAZE_CLK;
        input UBLAZE_RST;
        input UBLAZE_DRAM_POWER_OFF_REQ;
        input UBLAZE_SELF_REF_REQ;
        output UBLAZE_DRAM_POWER_OFF_DONE;
        output UBLAZE_SELF_REF_DONE;
        input UBLAZE_MISC_IN[3];
        output UBLAZE_MISC_OUT[2];

        input UBLAZE_BSCAN_EXT_CAPTURE;
        input UBLAZE_BSCAN_EXT_DRCK;
        input UBLAZE_BSCAN_EXT_RESET;
        input UBLAZE_BSCAN_EXT_SEL;
        input UBLAZE_BSCAN_EXT_SHIFT;
        input UBLAZE_BSCAN_EXT_TCK;
        input UBLAZE_BSCAN_EXT_TDI;
        output UBLAZE_BSCAN_EXT_TDO;
        input UBLAZE_BSCAN_EXT_TMS;
        input UBLAZE_BSCAN_EXT_UPDATE;

        input UBLAZE_AXI_DC_ARREADY;
        output UBLAZE_AXI_DC_ARVALID;
        output UBLAZE_AXI_DC_ARADDR[16];
        output UBLAZE_AXI_DC_ARLEN[8];
        output UBLAZE_AXI_DC_ARBURST[2];
        output UBLAZE_AXI_DC_ARSIZE[3];

        input UBLAZE_AXI_DC_AWREADY;
        output UBLAZE_AXI_DC_AWVALID;
        output UBLAZE_AXI_DC_AWADDR[16];
        output UBLAZE_AXI_DC_AWLEN[8];
        output UBLAZE_AXI_DC_AWBURST[2];
        output UBLAZE_AXI_DC_AWSIZE[3];

        output UBLAZE_AXI_DC_BREADY;
        input UBLAZE_AXI_DC_BVALID;
        input UBLAZE_AXI_DC_BRESP[2];

        output UBLAZE_AXI_DC_RREADY;
        input UBLAZE_AXI_DC_RVALID;
        input UBLAZE_AXI_DC_RDATA[32];
        input UBLAZE_AXI_DC_RLAST;
        input UBLAZE_AXI_DC_RRESP[2];

        input UBLAZE_AXI_DC_WREADY;
        output UBLAZE_AXI_DC_WVALID;
        output UBLAZE_AXI_DC_WDATA[32];
        output UBLAZE_AXI_DC_WSTRB[4];
        output UBLAZE_AXI_DC_WLAST;

        input UBLAZE_AXI_IC_ARREADY;
        output UBLAZE_AXI_IC_ARVALID;
        output UBLAZE_AXI_IC_ARADDR[16];
        output UBLAZE_AXI_IC_ARLEN[8];
        output UBLAZE_AXI_IC_ARBURST[2];
        output UBLAZE_AXI_IC_ARSIZE[3];

        output UBLAZE_AXI_IC_RREADY;
        input UBLAZE_AXI_IC_RVALID;
        input UBLAZE_AXI_IC_RDATA[32];
        input UBLAZE_AXI_IC_RLAST;
        input UBLAZE_AXI_IC_RRESP[2];

        input DCI_USR_RST;
        input DCI_RST_N;
        input DCI_TEST_I[24];
        output DCI_LOCK;
        output DCI_TEST_O[24];

        input DCI_PUDC_B;

        input DFX_CSSD_TRIG[11];
        input DFX_SCAN_CLK_B;
        input DFX_SCAN_EN_B;
        input DFX_SCAN_MODE_RST_B;
        input DFX_SCAN_EDT_UPDT_B;
        input DFX_SCAN_CNTRL_CHNL_IN[11];
        input DFX_SCAN_COMP_IN[11];
        input DFX_SCAN_IN[11];
        output DFX_SCAN_CNTRL_CHNL_OUT[11];
        output DFX_SCAN_COMP_OUT[11];
        output DFX_SCAN_OUT[11];

        output DFX_UNEXPECTED_ACTIVATION[12];

        output CFI_NPI_BRIDGE_TO_FABRIC_STATUS[16];

        input FABRIC_MUX_SELECT_EN;

        input IJTAG_RESET_TAP;
        output IJTAG_RESET_TAP_O;
        input IJTAG_TDI_EXT;
        input IJTAG_TDI_RETURN_EXT;
        output IJTAG_TDO_OABUT;
        output IJTAG_TDO_RETURN_EXT;
        input IJTAG_CLOCK_DR;
        output IJTAG_CLOCK_DR_O;
        input IJTAG_SELECT_DR;
        output IJTAG_SELECT_DR_O;
        input IJTAG_CAPTURE_DR;
        output IJTAG_CAPTURE_DR_O;
        input IJTAG_SHIFT_DR;
        output IJTAG_SHIFT_DR_O;
        input IJTAG_UPDATE_DR;
        output IJTAG_UPDATE_DR_O;

        input IOB_PUDC_B;
        output IOB_PUDC_B_O;

        output TEST_CLK_OUT[2];
    }

    bel_class BUFGCE {
        input CLK_IN, CE;
        output CLK_OUT;

        // TODO: attributes
    }

    bel_class BUFGCE_DIV {
        input CLK_IN, RST, CE;
        output CLK_OUT;

        // TODO: attributes
    }

    bel_class BUFGCTRL {
        input CLK_I0, CLK_I1;
        input CE0, CE1;
        input IGNORE0, IGNORE1;
        input SEL0, SEL1;
        output CLK_OUT;

        // TODO: attributes
    }

    bel_class BUFG_GT {
        input CLK_IN, RST, CE;
        input RSTMASK, CEMASK;
        input DIV[3];
        output CLK_OUT;

        // TODO: attributes
    }

    bel_class BUFG_GT_SYNC {
        input CLK_IN;
        input CE_IN, RST_IN;
        output CE_OUT, RST_OUT;

        // TODO: attributes
    }

    bel_class BUFG_PS {
        input CLK_IN;
        output CLK_OUT;
    }

    bel_class GCLK_TEST_BUF {
        input CLK_IN;
        output CLK_OUT;

        // TODO: attributes?
    }

    bel_class BUFCE_ROW {
        input CLK_IN, CE;
        input OPT_DELAY_TEST[3];
        output CLK_OUT, CLK_OUT_OPT_DLY;

        // TODO: attributes
    }

    bel_class BUFCE_LEAF_X16 {
        input CLK_IN[16], CE[16];
        output CLK_OUT[16];

        // TODO: attributes; split?
    }

    bel_class BUFCE_LEAF {
        input CLK_IN, CE, ENSEL_PROG, CLK_CASC_IN;
        output CLK_LEAF;

        // TODO: attributes
    }

    bel_class MMCM {
        input CLKIN1, CLKIN2, CLKINSEL;
        input CLKFBIN;
        output CLKFBSTOPPED;
        output CLKINSTOPPED;

        output CLKOUT0, CLKOUT0B;
        output CLKOUT1, CLKOUT1B;
        output CLKOUT2, CLKOUT2B;
        output CLKOUT3, CLKOUT3B;
        output CLKOUT4, CLKOUT5, CLKOUT6;
        output CLKFBOUT, CLKFBOUTB;
        output TMUXOUT;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[7];
        input DI[16];
        output DOUT[16];
        output DRDY;

        input RST;
        input PWRDWN;
        output LOCKED;

        input PSCLK, PSEN, PSINCDEC;
        output PSDONE;

        input CDDCREQ;
        output CDDCDONE;

        input SCANCLK;
        input SCANENB;
        input SCANMODEB;
        input SCANIN;
        output SCANOUT;

        input TESTIN[32];
        // u+ only has 16 bits
        output TESTOUT[37];

        // TODO: attributes
    }

    bel_class PLL {
        input CLKIN, CLKFBIN;

        output CLKOUT0, CLKOUT0B;
        output CLKOUT1, CLKOUT1B;
        output CLKFBOUT;
        output TMUXOUT;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[7];
        input DI[16];
        output DRDY;
        output DOUT[16];

        input RST;
        input PWRDWN;
        input CLKOUTPHY_EN;
        output LOCKED;

        input SCANCLK;
        input SCANENB;
        input SCANMODEB;
        input SCANIN;
        output SCANOUT;

        input TESTIN[32];
        output TESTOUT[37];

        // TODO: attributes
    }

    bel_class HBM_REF_CLK {
        input REF_CLK;
    }

    bel_class CFG {
        // STARTUP
        output EOS;
        input KEY_CLEAR;
        output PROG_REQ;
        input PROG_ACK;
        output RBCRC_ERROR;
        output START_CFG_CLK;
        output START_CFG_MCLK;
        input USR_GSR;
        input USR_GTS;

        input USR_CCLK_O;
        input USR_CCLK_TS;
        input USR_DONE_O;
        input USR_DONE_TS;
        input USR_D_O_CFGIO[4];
        output USR_D_PIN_CFGIO[4];
        input USR_D_TS_CFGIO[4];
        input USR_FCS_B_O;
        input USR_FCS_B_TS;
        input USR_TCK;
        input USR_TMS;
        input USR_TDI;
        output USR_TDO;

        // ???
        output IOX_CCLK;
        output IOX_CFGDATA[32];
        output IOX_CFGMASTER;
        output IOX_INITBI;
        input IOX_INITBO;
        output IOX_MODE[3];
        output IOX_PUDCB;
        output IOX_RDWRB;
        input IOX_TDO;
        output IOX_VGG_COMP_OUT;

        // BSCAN×4
        for i in 1..5 {
            output "BSCAN_TCK{i}";
            output "BSCAN_TMS{i}";
            output "BSCAN_TDI{i}";
            output "BSCAN_CLKDR{i}";
            output "BSCAN_SEL{i}";
            output "BSCAN_TLR{i}";
            output "BSCAN_RTI{i}";
            output "BSCAN_CDR{i}";
            output "BSCAN_SDR{i}";
            output "BSCAN_UDR{i}";
            input "BSCAN_TDO{i}";
        }

        // ICAP×2
        output ICAP_AVAIL_BOT;
        output ICAP_AVAIL_TOP;
        input ICAP_CLK_BOT;
        input ICAP_CLK_TOP;
        input ICAP_CS_B_BOT;
        input ICAP_CS_B_TOP;
        input ICAP_RDWR_B_BOT;
        input ICAP_RDWR_B_TOP;
        input ICAP_DATA_BOT[32];
        input ICAP_DATA_TOP[32];
        output ICAP_OUT_BOT[32];
        output ICAP_OUT_TOP[32];
        output ICAP_PR_DONE_BOT;
        output ICAP_PR_DONE_TOP;
        output ICAP_PR_ERROR_BOT;
        output ICAP_PR_ERROR_TOP;

        // DNA_PORT
        input USR_DNA_CLK;
        input USR_DNA_READ;
        input USR_DNA_SHIFT;
        input USR_DNA_DIN;
        output USR_DNA_OUT;

        // FRAME_ECC
        output ECC_END_OF_FRAME;
        output ECC_END_OF_SCAN;
        output ECC_ERROR_NOTSINGLE;
        output ECC_ERROR_SINGLE;
        output ECC_FAR[27];
        input ECC_FAR_SEL[2];

        // USR_ACCESS
        output USR_ACCESS_CLK;
        output USR_ACCESS_VALID;
        output USR_ACCESS_DATA[32];

        // EFUSE_USR
        output USR_EFUSE[32];

        // DCI
        input DCI_USR_RESET_IN;
        output DCI_LOCK;

        // AXI (CSEC only)
        input AXI_CLK;

        output ARREADY;
        input ARVALID;
        input ARADDR[28];
        input ARBURST[2];
        input ARCACHE[4];
        input ARID[8];
        input ARLEN[4];
        input ARLOCK;
        input ARPROT[3];
        input ARQOS[4];
        input ARSIZE[3];

        output AWREADY;
        input AWVALID;
        input AWADDR[28];
        input AWBURST[2];
        input AWCACHE[4];
        input AWID[8];
        input AWLEN[4];
        input AWLOCK;
        input AWPROT[3];
        input AWQOS[4];
        input AWSIZE[3];

        input BREADY;
        output BVALID;
        output BID[8];
        output BRESP[2];

        input RREADY;
        output RVALID;
        output RDATA[32];
        output RID[8];
        output RLAST;
        output RRESP[2];

        output WREADY;
        input WVALID;
        input WDATA[32];
        input WSTRB[4];
        input WID[8];
        input WLAST;

        // TODO: attributes
    }

    bel_class CFGIO {
        // TODO: attributes
    }

    bel_class PMV {
        input PMV_EN1_INTIP;
        input SPARE_IN1_INTIP[6];
        output OUT1_INTOP;
        output OUT2_INTOP;
        output OUT3_INTOP;
        output OUT4_INTOP;

        // TODO: attributes
    }

    bel_class PMV2 {
        input IMUX_IN_INT[4];
        output OUTS_INT[3];

        // TODO: attributes
    }

    bel_class PMVIOB {
        input PMV_A_HPIO_INTIP[2];
        input PMV_EN_HPIO_INTIP;
        output OUT_HPIO_INTOP;
        output OUT_DIV2_HPIO_INTOP;
        output OUT_DIV4_HPIO_INTOP;

        // TODO: attributes
    }

    bel_class MTBF3 {
        input FF_CLK_INTIP;
        input RESET_INTIP;
        input SYNC_ENABLE_INTIP;
        input TOGGLE_SEL_INTIP;
        input OUTPUT_SEL_INTIP[4];
        input DATAIN_INTIP;
        output FF_Q_INTOP[5];
        input CAPTURE_CLK_INTIP;
        output CAPTURE_Q_INTOP[5];

        // TODO: attributes
    }

    bel_class SYSMON {
        input CONVST;
        input CONVST_CLK;
        input RESET_USER;
        output ALM[16];
        output BUSY;
        output CHANNEL[6];
        output EOC;
        output EOS;
        output MUX_ADDR[5];
        output OT;
        // ultrascale+ only
        output ADC_DATA[16];

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[8];
        input DI[16];
        output DRDY;
        output DOUT[16];

        // ultrascale
        input DATA_READY_ADC1_F;
        input DATA_READY_ADC2_F;
        input DEC_OUT_ADC1_F[16];
        input DEC_OUT_ADC2_F[16];
        // ultrascale+
        input DATA_READY_ADC_F;
        input DEC_OUT_ADC_F[16];

        // ultrascale+ only
        input I2C_SCLK_IN;
        output I2C_SCLK_TS;
        input I2C_SDA_IN;
        output I2C_SDA_TS;
        output SMBALERT_TS;

        output JTAG_BUSY;
        output JTAG_LOCKED;
        output JTAG_MODIFIED;

        input TEST_ADC_CLK[4];
        // ultrascale uses only 20 bits of each
        input TEST_ADC_IN[32];
        input TEST_ADC_IN2[32];
        output TEST_ADC_OUT[20];
        output TEST_DB[16];
        input TEST_SCAN_CLK[5];
        input TEST_SCAN_RESET;
        input TEST_SCAN_MODE[5];
        input TEST_SE[5];
        // ultrascale uses only 5 bits of each
        input TEST_SI[10];
        output TEST_SO[10];

        input TEST_EN_JTAG;
        input TEST_DRCK;
        input TEST_RST;
        input TEST_SEL;
        input TEST_CAPTURE;
        input TEST_SHIFT;
        input TEST_UPDATE;
        input TEST_TDI;
        output TEST_TDO;

        // TODO: attributes
    }

    bel_class PCIE3 {
        input CORE_CLK;
        input USER_CLK;
        input RESET_N;

        input DRP_CLK;
        input DRP_EN;
        input DRP_WE;
        input DRP_ADDR[10];
        input DRP_DI[16];
        output DRP_RDY;
        output DRP_DO[16];

        input M_AXIS_CQ_TREADY[22];
        output M_AXIS_CQ_TVALID;
        output M_AXIS_CQ_TDATA[256];
        output M_AXIS_CQ_TKEEP[8];
        output M_AXIS_CQ_TLAST;
        output M_AXIS_CQ_TUSER[85];

        input M_AXIS_RC_TREADY[22];
        output M_AXIS_RC_TVALID;
        output M_AXIS_RC_TDATA[256];
        output M_AXIS_RC_TKEEP[8];
        output M_AXIS_RC_TLAST;
        output M_AXIS_RC_TUSER[75];

        output S_AXIS_CC_TREADY[4];
        input S_AXIS_CC_TVALID;
        input S_AXIS_CC_TDATA[256];
        input S_AXIS_CC_TKEEP[8];
        input S_AXIS_CC_TLAST;
        input S_AXIS_CC_TUSER[33];

        output S_AXIS_RQ_TREADY[4];
        input S_AXIS_RQ_TVALID;
        input S_AXIS_RQ_TDATA[256];
        input S_AXIS_RQ_TKEEP[8];
        input S_AXIS_RQ_TLAST;
        input S_AXIS_RQ_TUSER[60];

        input CFG_MGMT_ADDR[19];
        input CFG_MGMT_BYTE_ENABLE[4];
        input CFG_MGMT_READ;
        output CFG_MGMT_READ_DATA[32];
        output CFG_MGMT_READ_WRITE_DONE;
        input CFG_MGMT_TYPE1_CFG_REG_ACCESS;
        input CFG_MGMT_WRITE;
        input CFG_MGMT_WRITE_DATA[32];
        input MGMT_RESET_N;
        input MGMT_STICKY_RESET_N;

        input CFG_CONFIG_SPACE_ENABLE;
        output CFG_CURRENT_SPEED[3];
        input CFG_DEV_ID[16];
        output CFG_DPA_SUBSTATE_CHANGE[4];
        input CFG_DSN[64];
        input CFG_DS_BUS_NUMBER[8];
        input CFG_DS_DEVICE_NUMBER[5];
        input CFG_DS_FUNCTION_NUMBER[3];
        input CFG_DS_PORT_NUMBER[8];
        input CFG_ERR_COR_IN;
        output CFG_ERR_COR_OUT;
        output CFG_ERR_FATAL_OUT;
        output CFG_ERR_NONFATAL_OUT;
        input CFG_ERR_UNCOR_IN;

        output CFG_EXT_FUNCTION_NUMBER[8];
        input CFG_EXT_READ_DATA[32];
        input CFG_EXT_READ_DATA_VALID;
        output CFG_EXT_READ_RECEIVED;
        output CFG_EXT_REGISTER_NUMBER[10];
        output CFG_EXT_WRITE_BYTE_ENABLE[4];
        output CFG_EXT_WRITE_DATA[32];
        output CFG_EXT_WRITE_RECEIVED;

        output CFG_FC_CPLD[12];
        output CFG_FC_CPLH[8];
        output CFG_FC_NPD[12];
        output CFG_FC_NPH[8];
        output CFG_FC_PD[12];
        output CFG_FC_PH[8];
        input CFG_FC_SEL[3];

        input CFG_FLR_DONE[4];
        output CFG_FLR_IN_PROCESS[4];
        output CFG_FUNCTION_POWER_STATE[12];
        output CFG_FUNCTION_STATUS[16];
        input CFG_HOT_RESET_IN;
        output CFG_HOT_RESET_OUT;

        input CFG_INTERRUPT_INT[4];
        input CFG_INTERRUPT_MSIX_ADDRESS[64];
        input CFG_INTERRUPT_MSIX_DATA[32];
        output CFG_INTERRUPT_MSIX_ENABLE[4];
        output CFG_INTERRUPT_MSIX_FAIL;
        input CFG_INTERRUPT_MSIX_INT;
        output CFG_INTERRUPT_MSIX_MASK[4];
        output CFG_INTERRUPT_MSIX_SENT;
        output CFG_INTERRUPT_MSIX_VF_ENABLE[8];
        output CFG_INTERRUPT_MSIX_VF_MASK[8];
        input CFG_INTERRUPT_MSI_ATTR[3];
        output CFG_INTERRUPT_MSI_DATA[32];
        output CFG_INTERRUPT_MSI_ENABLE[4];
        output CFG_INTERRUPT_MSI_FAIL;
        input CFG_INTERRUPT_MSI_FUNCTION_NUMBER[4];
        input CFG_INTERRUPT_MSI_INT[32];
        output CFG_INTERRUPT_MSI_MASK_UPDATE;
        output CFG_INTERRUPT_MSI_MMENABLE[12];
        input CFG_INTERRUPT_MSI_PENDING_STATUS[32];
        input CFG_INTERRUPT_MSI_PENDING_STATUS_DATA_ENABLE;
        input CFG_INTERRUPT_MSI_PENDING_STATUS_FUNCTION_NUM[4];
        input CFG_INTERRUPT_MSI_SELECT[4];
        output CFG_INTERRUPT_MSI_SENT;
        input CFG_INTERRUPT_MSI_TPH_PRESENT;
        input CFG_INTERRUPT_MSI_TPH_ST_TAG[9];
        input CFG_INTERRUPT_MSI_TPH_TYPE[2];
        output CFG_INTERRUPT_MSI_VF_ENABLE[8];
        input CFG_INTERRUPT_PENDING[4];
        output CFG_INTERRUPT_SENT;

        output CFG_LINK_POWER_STATE[2];
        input CFG_LINK_TRAINING_ENABLE;
        output CFG_LOCAL_ERROR;
        output CFG_LTR_ENABLE;
        output CFG_LTSSM_STATE[6];
        output CFG_MAX_PAYLOAD[3];
        output CFG_MAX_READ_REQ[3];

        output CFG_MSG_RECEIVED;
        output CFG_MSG_RECEIVED_DATA[8];
        output CFG_MSG_RECEIVED_TYPE[5];
        input CFG_MSG_TRANSMIT;
        input CFG_MSG_TRANSMIT_DATA[32];
        output CFG_MSG_TRANSMIT_DONE;
        input CFG_MSG_TRANSMIT_TYPE[3];

        output CFG_NEGOTIATED_WIDTH[4];
        output CFG_OBFF_ENABLE[2];
        input CFG_PER_FUNCTION_NUMBER[4];
        input CFG_PER_FUNCTION_OUTPUT_REQUEST;
        output CFG_PER_FUNCTION_UPDATE_DONE;
        input CFG_PER_FUNC_STATUS_CONTROL[3];
        output CFG_PER_FUNC_STATUS_DATA[16];
        output CFG_PHY_LINK_DOWN;
        output CFG_PHY_LINK_STATUS[2];
        output CFG_PL_STATUS_CHANGE;
        input CFG_POWER_STATE_CHANGE_ACK;
        output CFG_POWER_STATE_CHANGE_INTERRUPT;
        output CFG_RCB_STATUS[4];
        input CFG_REQ_PM_TRANSITION_L23_READY;
        input CFG_REV_ID[8];
        input CFG_SUBSYS_ID[16];
        input CFG_SUBSYS_VEND_ID[16];

        output CFG_TPH_FUNCTION_NUM[4];
        output CFG_TPH_REQUESTER_ENABLE[4];
        output CFG_TPH_STT_ADDRESS[5];
        input CFG_TPH_STT_READ_DATA[32];
        input CFG_TPH_STT_READ_DATA_VALID;
        output CFG_TPH_STT_READ_ENABLE;
        output CFG_TPH_STT_WRITE_BYTE_VALID[4];
        output CFG_TPH_STT_WRITE_DATA[32];
        output CFG_TPH_STT_WRITE_ENABLE;
        output CFG_TPH_ST_MODE[12];

        input CFG_VEND_ID[16];
        input CFG_VF_FLR_DONE[8];
        output CFG_VF_FLR_IN_PROCESS[8];
        output CFG_VF_POWER_STATE[24];
        output CFG_VF_STATUS[16];
        output CFG_VF_TPH_REQUESTER_ENABLE[8];
        output CFG_VF_TPH_ST_MODE[24];

        output CONF_MCAP_DESIGN_SWITCH;
        output CONF_MCAP_EOS;
        output CONF_MCAP_IN_USE_BY_PCIE;
        input CONF_MCAP_REQUEST_BY_CONF;

        input CONF_REQ_DATA[32];
        output CONF_REQ_READY;
        input CONF_REQ_REG_NUM[4];
        input CONF_REQ_TYPE[2];
        input CONF_REQ_VALID;
        output CONF_RESP_RDATA[32];
        output CONF_RESP_VALID;

        input CORE_CLK_MI_COMPLETION_RAM_L;
        input CORE_CLK_MI_COMPLETION_RAM_U;
        output MI_COMPLETION_RAM_READ_ADDRESS_A_L[10];
        output MI_COMPLETION_RAM_READ_ADDRESS_A_U[10];
        output MI_COMPLETION_RAM_READ_ADDRESS_B_L[10];
        output MI_COMPLETION_RAM_READ_ADDRESS_B_U[10];
        input MI_COMPLETION_RAM_READ_DATA[144];
        output MI_COMPLETION_RAM_READ_ENABLE_L[4];
        output MI_COMPLETION_RAM_READ_ENABLE_U[4];
        output MI_COMPLETION_RAM_WRITE_ADDRESS_A_L[10];
        output MI_COMPLETION_RAM_WRITE_ADDRESS_A_U[10];
        output MI_COMPLETION_RAM_WRITE_ADDRESS_B_L[10];
        output MI_COMPLETION_RAM_WRITE_ADDRESS_B_U[10];
        output MI_COMPLETION_RAM_WRITE_DATA_L[72];
        output MI_COMPLETION_RAM_WRITE_DATA_U[72];
        output MI_COMPLETION_RAM_WRITE_ENABLE_L[4];
        output MI_COMPLETION_RAM_WRITE_ENABLE_U[4];

        input CORE_CLK_MI_REPLAY_RAM;
        output MI_REPLAY_RAM_ADDRESS[9];
        input MI_REPLAY_RAM_READ_DATA[144];
        output MI_REPLAY_RAM_READ_ENABLE[2];
        output MI_REPLAY_RAM_WRITE_DATA[144];
        output MI_REPLAY_RAM_WRITE_ENABLE[2];

        input CORE_CLK_MI_REQUEST_RAM;
        output MI_REQUEST_RAM_READ_ADDRESS_A[9];
        output MI_REQUEST_RAM_READ_ADDRESS_B[9];
        input MI_REQUEST_RAM_READ_DATA[144];
        output MI_REQUEST_RAM_READ_ENABLE[4];
        output MI_REQUEST_RAM_WRITE_ADDRESS_A[9];
        output MI_REQUEST_RAM_WRITE_ADDRESS_B[9];
        output MI_REQUEST_RAM_WRITE_DATA[144];
        output MI_REQUEST_RAM_WRITE_ENABLE[4];

        input DBG_CFG_LOCAL_MGMT_REG_OVERRIDE;
        output DBG_DATA_OUT[16];
        input DBG_DATA_SEL[4];

        output DBG_MCAP_CS_B;
        output DBG_MCAP_DATA[32];
        output DBG_MCAP_EOS;
        output DBG_MCAP_ERROR;
        output DBG_MCAP_MODE;
        output DBG_MCAP_RDATA_VALID;
        output DBG_MCAP_RDWR_B;
        output DBG_MCAP_RESET;

        output DBG_PL_DATA_BLOCK_RECEIVED_AFTER_EDS;
        output DBG_PL_GEN3_FRAMING_ERROR_DETECTED;
        output DBG_PL_GEN3_SYNC_HEADER_ERROR_DETECTED;
        output DBG_PL_INFERRED_RX_ELECTRICAL_IDLE[8];

        output LL2LM_MASTER_TLP_SENT[2];
        output LL2LM_MASTER_TLP_SENT_TLP_ID0[4];
        output LL2LM_MASTER_TLP_SENT_TLP_ID1[4];

        output LL2LM_M_AXIS_RX_TVALID[8];
        output LL2LM_M_AXIS_RX_TDATA[256];
        output LL2LM_M_AXIS_RX_TUSER[18];

        output LL2LM_S_AXIS_TX_TREADY[8];
        input LL2LM_S_AXIS_TX_TVALID;
        input LL2LM_S_AXIS_TX_TUSER[14];

        input LL2LM_TX_TLP_ID0[4];
        input LL2LM_TX_TLP_ID1[4];

        input MCAP_CLK;

        input PCIE_CQ_NP_REQ;
        output PCIE_CQ_NP_REQ_COUNT[6];
        output PCIE_PERST0_B;
        output PCIE_PERST1_B;
        output PCIE_RQ_SEQ_NUM[4];
        output PCIE_RQ_SEQ_NUM_VLD;
        output PCIE_RQ_TAG[6];
        output PCIE_RQ_TAG_AV[2];
        output PCIE_RQ_TAG_VLD;
        output PCIE_TFC_NPD_AV[2];
        output PCIE_TFC_NPH_AV[2];

        input PIPE_CLK;
        input PIPE_RESET_N;
        input PIPE_EQ_FS[6];
        input PIPE_EQ_LF[6];

        for ch in 0..8 {
            input "PIPE_RX{ch}_CHAR_IS_K"[2];
            input "PIPE_RX{ch}_DATA"[32];
            input "PIPE_RX{ch}_DATA_VALID";
            input "PIPE_RX{ch}_ELEC_IDLE";
            output "PIPE_RX{ch}_EQ_CONTROL"[2];
            input "PIPE_RX{ch}_EQ_DONE";
            input "PIPE_RX{ch}_EQ_LP_ADAPT_DONE";
            output "PIPE_RX{ch}_EQ_LP_LF_FS"[6];
            input "PIPE_RX{ch}_EQ_LP_LF_FS_SEL";
            input "PIPE_RX{ch}_EQ_LP_NEW_TX_COEFF_OR_PRESET"[18];
            output "PIPE_RX{ch}_EQ_LP_TX_PRESET"[4];
            output "PIPE_RX{ch}_EQ_PRESET"[3];
            input "PIPE_RX{ch}_PHY_STATUS";
            output "PIPE_RX{ch}_POLARITY";
            input "PIPE_RX{ch}_START_BLOCK";
            input "PIPE_RX{ch}_STATUS"[3];
            input "PIPE_RX{ch}_SYNC_HEADER"[2];
            input "PIPE_RX{ch}_VALID";

            output "PIPE_TX{ch}_CHAR_IS_K"[2];
            output "PIPE_TX{ch}_COMPLIANCE";
            output "PIPE_TX{ch}_DATA"[32];
            output "PIPE_TX{ch}_DATA_VALID";
            output "PIPE_TX{ch}_DEEMPH";
            output "PIPE_TX{ch}_ELEC_IDLE";
            input "PIPE_TX{ch}_EQ_COEFF"[18];
            output "PIPE_TX{ch}_EQ_CONTROL"[2];
            output "PIPE_TX{ch}_EQ_DEEMPH"[6];
            input "PIPE_TX{ch}_EQ_DONE";
            output "PIPE_TX{ch}_EQ_PRESET"[4];
            output "PIPE_TX{ch}_MARGIN"[3];
            output "PIPE_TX{ch}_POWERDOWN"[2];
            output "PIPE_TX{ch}_RATE"[2];
            output "PIPE_TX{ch}_RCVR_DET";
            output "PIPE_TX{ch}_RESET";
            output "PIPE_TX{ch}_START_BLOCK";
            output "PIPE_TX{ch}_SWING";
            output "PIPE_TX{ch}_SYNC_HEADER"[2];
        }

        output PL_EQ_IN_PROGRESS;
        output PL_EQ_PHASE[2];
        input PL_EQ_RESET_EIEOS_COUNT;
        input PL_GEN2_UPSTREAM_PREFER_DEEMPH;

        input PMV_ENABLE_N;
        input PMV_SELECT[3];
        input PMV_DIVIDE[2];
        output PMV_OUT;

        input SCANENABLE_N;
        input SCANMODE_N;
        input SCANIN[96];
        output SCANOUT[96];

        input SPARE_IN[32];
        output SPARE_OUT[32];

        // TODO: attributes
    }

    bel_class PCIE4 {
        input CORE_CLK;
        input RESET_N;
        input USER_CLK;
        input USER_CLK2;
        input USER_CLK_EN;

        input DRP_CLK;
        input DRP_EN;
        input DRP_WE;
        input DRP_ADDR[10];
        input DRP_DI[16];
        output DRP_RDY;
        output DRP_DO[16];

        input M_AXIS_CQ_TREADY[22];
        output M_AXIS_CQ_TVALID;
        output M_AXIS_CQ_TDATA[256];
        output M_AXIS_CQ_TKEEP[8];
        output M_AXIS_CQ_TLAST;
        output M_AXIS_CQ_TUSER[88];

        input M_AXIS_RC_TREADY[22];
        output M_AXIS_RC_TVALID;
        output M_AXIS_RC_TDATA[256];
        output M_AXIS_RC_TKEEP[8];
        output M_AXIS_RC_TLAST;
        output M_AXIS_RC_TUSER[75];

        output S_AXIS_CC_TREADY[4];
        input S_AXIS_CC_TVALID;
        input S_AXIS_CC_TDATA[256];
        input S_AXIS_CC_TKEEP[8];
        input S_AXIS_CC_TLAST;
        input S_AXIS_CC_TUSER[33];

        output S_AXIS_RQ_TREADY[4];
        input S_AXIS_RQ_TVALID;
        input S_AXIS_RQ_TDATA[256];
        input S_AXIS_RQ_TKEEP[8];
        input S_AXIS_RQ_TLAST;
        input S_AXIS_RQ_TUSER[62];

        input AXI_USER_IN[8];
        output AXI_USER_OUT[8];

        output CFG_BUS_NUMBER[8];
        input CFG_CONFIG_SPACE_ENABLE;
        output CFG_CURRENT_SPEED[2];
        input CFG_DEV_ID_PF0[16];
        input CFG_DEV_ID_PF1[16];
        input CFG_DEV_ID_PF2[16];
        input CFG_DEV_ID_PF3[16];
        input CFG_DSN[64];
        input CFG_DS_BUS_NUMBER[8];
        input CFG_DS_DEVICE_NUMBER[5];
        input CFG_DS_FUNCTION_NUMBER[3];
        input CFG_DS_PORT_NUMBER[8];
        input CFG_ERR_COR_IN;
        output CFG_ERR_COR_OUT;
        output CFG_ERR_FATAL_OUT;
        output CFG_ERR_NONFATAL_OUT;
        input CFG_ERR_UNCOR_IN;

        output CFG_EXT_FUNCTION_NUMBER[8];
        input CFG_EXT_READ_DATA[32];
        input CFG_EXT_READ_DATA_VALID;
        output CFG_EXT_READ_RECEIVED;
        output CFG_EXT_REGISTER_NUMBER[10];
        output CFG_EXT_WRITE_BYTE_ENABLE[4];
        output CFG_EXT_WRITE_DATA[32];
        output CFG_EXT_WRITE_RECEIVED;

        output CFG_FC_CPLD[12];
        output CFG_FC_CPLH[8];
        output CFG_FC_NPD[12];
        output CFG_FC_NPH[8];
        output CFG_FC_PD[12];
        output CFG_FC_PH[8];
        input CFG_FC_SEL[3];
        // PCIE4C only
        input CFG_FC_VC_SEL;

        input CFG_FLR_DONE[4];
        output CFG_FLR_IN_PROCESS[4];
        output CFG_FUNCTION_POWER_STATE[12];
        output CFG_FUNCTION_STATUS[16];
        input CFG_HOT_RESET_IN;
        output CFG_HOT_RESET_OUT;

        input CFG_INTERRUPT_INT[4];
        input CFG_INTERRUPT_MSIX_ADDRESS[64];
        input CFG_INTERRUPT_MSIX_DATA[32];
        output CFG_INTERRUPT_MSIX_ENABLE[4];
        input CFG_INTERRUPT_MSIX_INT;
        output CFG_INTERRUPT_MSIX_MASK[4];
        input CFG_INTERRUPT_MSIX_VEC_PENDING[2];
        output CFG_INTERRUPT_MSIX_VEC_PENDING_STATUS;
        input CFG_INTERRUPT_MSI_ATTR[3];
        output CFG_INTERRUPT_MSI_DATA[32];
        output CFG_INTERRUPT_MSI_ENABLE[4];
        output CFG_INTERRUPT_MSI_FAIL;
        input CFG_INTERRUPT_MSI_FUNCTION_NUMBER[8];
        input CFG_INTERRUPT_MSI_INT[32];
        output CFG_INTERRUPT_MSI_MASK_UPDATE;
        output CFG_INTERRUPT_MSI_MMENABLE[12];
        input CFG_INTERRUPT_MSI_PENDING_STATUS[32];
        input CFG_INTERRUPT_MSI_PENDING_STATUS_DATA_ENABLE;
        input CFG_INTERRUPT_MSI_PENDING_STATUS_FUNCTION_NUM[2];
        input CFG_INTERRUPT_MSI_SELECT[2];
        output CFG_INTERRUPT_MSI_SENT;
        input CFG_INTERRUPT_MSI_TPH_PRESENT;
        input CFG_INTERRUPT_MSI_TPH_ST_TAG[8];
        input CFG_INTERRUPT_MSI_TPH_TYPE[2];
        input CFG_INTERRUPT_PENDING[4];
        output CFG_INTERRUPT_SENT;

        output CFG_LINK_POWER_STATE[2];
        input CFG_LINK_TRAINING_ENABLE;
        output CFG_LOCAL_ERROR_OUT[5];
        output CFG_LOCAL_ERROR_VALID;
        output CFG_LTR_ENABLE;
        output CFG_LTSSM_STATE[6];
        output CFG_MAX_PAYLOAD[2];
        output CFG_MAX_READ_REQ[3];

        input CFG_MGMT_ADDR[10];
        input CFG_MGMT_BYTE_ENABLE[4];
        input CFG_MGMT_DEBUG_ACCESS;
        input CFG_MGMT_FUNCTION_NUMBER[8];
        input CFG_MGMT_READ;
        output CFG_MGMT_READ_DATA[32];
        output CFG_MGMT_READ_WRITE_DONE;
        input CFG_MGMT_WRITE;
        input CFG_MGMT_WRITE_DATA[32];

        output CFG_MSG_RECEIVED;
        output CFG_MSG_RECEIVED_DATA[8];
        output CFG_MSG_RECEIVED_TYPE[5];
        input CFG_MSG_TRANSMIT;
        input CFG_MSG_TRANSMIT_DATA[32];
        output CFG_MSG_TRANSMIT_DONE;
        input CFG_MSG_TRANSMIT_TYPE[3];

        output CFG_MSIX_RAM_ADDRESS[13];
        input CFG_MSIX_RAM_READ_DATA[36];
        output CFG_MSIX_RAM_READ_ENABLE;
        output CFG_MSIX_RAM_WRITE_BYTE_ENABLE[4];
        output CFG_MSIX_RAM_WRITE_DATA[36];

        output CFG_NEGOTIATED_WIDTH[3];
        output CFG_OBFF_ENABLE[2];
        output CFG_PHY_LINK_DOWN;
        output CFG_PHY_LINK_STATUS[2];
        output CFG_PL_STATUS_CHANGE;
        input CFG_PM_ASPM_L1_ENTRY_REJECT;
        input CFG_PM_ASPM_TX_L0S_ENTRY_DISABLE;
        input CFG_POWER_STATE_CHANGE_ACK;
        output CFG_POWER_STATE_CHANGE_INTERRUPT;
        output CFG_RCB_STATUS[4];
        input CFG_REQ_PM_TRANSITION_L23_READY;
        input CFG_REV_ID_PF0[8];
        input CFG_REV_ID_PF1[8];
        input CFG_REV_ID_PF2[8];
        input CFG_REV_ID_PF3[8];
        output CFG_RX_PM_STATE[2];
        input CFG_SUBSYS_ID_PF0[16];
        input CFG_SUBSYS_ID_PF1[16];
        input CFG_SUBSYS_ID_PF2[16];
        input CFG_SUBSYS_ID_PF3[16];
        input CFG_SUBSYS_VEND_ID[16];

        output CFG_TPH_RAM_ADDRESS[12];
        input CFG_TPH_RAM_READ_DATA[36];
        output CFG_TPH_RAM_READ_ENABLE;
        output CFG_TPH_RAM_WRITE_BYTE_ENABLE[4];
        output CFG_TPH_RAM_WRITE_DATA[36];

        output CFG_TPH_REQUESTER_ENABLE[4];
        output CFG_TPH_ST_MODE[12];
        output CFG_TX_PM_STATE[2];
        input CFG_VEND_ID[16];
        input CFG_VF_FLR_DONE;
        input CFG_VF_FLR_FUNC_NUM[8];

        // PCIE4C only
        output CFG_VC1_ENABLE;
        output CFG_VC1_NEGOTIATION_PENDING;

        output CONF_MCAP_DESIGN_SWITCH;
        output CONF_MCAP_EOS;
        output CONF_MCAP_IN_USE_BY_PCIE;
        input CONF_MCAP_REQUEST_BY_CONF;
        input CONF_REQ_DATA[32];
        output CONF_REQ_READY;
        input CONF_REQ_REG_NUM[4];
        input CONF_REQ_TYPE[2];
        input CONF_REQ_VALID;
        output CONF_RESP_RDATA[32];
        output CONF_RESP_VALID;

        output DBG_CTRL0_OUT[32];
        output DBG_CTRL1_OUT[32];
        output DBG_DATA0_OUT[256];
        output DBG_DATA1_OUT[256];
        input DBG_SEL0[6];
        input DBG_SEL1[6];

        input MCAP_CLK;
        input MGMT_RESET_N;
        input MGMT_STICKY_RESET_N;

        input CORE_CLK_MI_REPLAY_RAM0;
        input CORE_CLK_MI_REPLAY_RAM1;
        output MI_REPLAY_RAM_ADDRESS0[9];
        output MI_REPLAY_RAM_ADDRESS1[9];
        input MI_REPLAY_RAM_ERR_COR[6];
        input MI_REPLAY_RAM_ERR_UNCOR[6];
        input MI_REPLAY_RAM_READ_DATA0[128];
        input MI_REPLAY_RAM_READ_DATA1[128];
        output MI_REPLAY_RAM_READ_ENABLE0;
        output MI_REPLAY_RAM_READ_ENABLE1;
        output MI_REPLAY_RAM_WRITE_DATA0[128];
        output MI_REPLAY_RAM_WRITE_DATA1[128];
        output MI_REPLAY_RAM_WRITE_ENABLE0;
        output MI_REPLAY_RAM_WRITE_ENABLE1;

        input CORE_CLK_MI_RX_COMPLETION_RAM0;
        input CORE_CLK_MI_RX_COMPLETION_RAM1;
        input MI_RX_COMPLETION_RAM_ERR_COR[12];
        input MI_RX_COMPLETION_RAM_ERR_UNCOR[12];
        output MI_RX_COMPLETION_RAM_READ_ADDRESS0[9];
        output MI_RX_COMPLETION_RAM_READ_ADDRESS1[9];
        input MI_RX_COMPLETION_RAM_READ_DATA0[144];
        input MI_RX_COMPLETION_RAM_READ_DATA1[144];
        output MI_RX_COMPLETION_RAM_READ_ENABLE0[2];
        output MI_RX_COMPLETION_RAM_READ_ENABLE1[2];
        output MI_RX_COMPLETION_RAM_WRITE_ADDRESS0[9];
        output MI_RX_COMPLETION_RAM_WRITE_ADDRESS1[9];
        output MI_RX_COMPLETION_RAM_WRITE_DATA0[144];
        output MI_RX_COMPLETION_RAM_WRITE_DATA1[144];
        output MI_RX_COMPLETION_RAM_WRITE_ENABLE0[2];
        output MI_RX_COMPLETION_RAM_WRITE_ENABLE1[2];

        input CORE_CLK_MI_RX_POSTED_REQUEST_RAM0;
        input CORE_CLK_MI_RX_POSTED_REQUEST_RAM1;
        input MI_RX_POSTED_REQUEST_RAM_ERR_COR[6];
        input MI_RX_POSTED_REQUEST_RAM_ERR_UNCOR[6];
        output MI_RX_POSTED_REQUEST_RAM_READ_ADDRESS0[9];
        output MI_RX_POSTED_REQUEST_RAM_READ_ADDRESS1[9];
        input MI_RX_POSTED_REQUEST_RAM_READ_DATA0[144];
        input MI_RX_POSTED_REQUEST_RAM_READ_DATA1[144];
        output MI_RX_POSTED_REQUEST_RAM_READ_ENABLE0;
        output MI_RX_POSTED_REQUEST_RAM_READ_ENABLE1;
        output MI_RX_POSTED_REQUEST_RAM_WRITE_ADDRESS0[9];
        output MI_RX_POSTED_REQUEST_RAM_WRITE_ADDRESS1[9];
        output MI_RX_POSTED_REQUEST_RAM_WRITE_DATA0[144];
        output MI_RX_POSTED_REQUEST_RAM_WRITE_DATA1[144];
        output MI_RX_POSTED_REQUEST_RAM_WRITE_ENABLE0;
        output MI_RX_POSTED_REQUEST_RAM_WRITE_ENABLE1;

        input PCIE_COMPL_DELIVERED[2];
        input PCIE_COMPL_DELIVERED_TAG0[8];
        input PCIE_COMPL_DELIVERED_TAG1[8];
        input PCIE_CQ_NP_REQ[2];
        output PCIE_CQ_NP_REQ_COUNT[6];
        input PCIE_CQ_NP_USER_CREDIT_RCVD;
        input PCIE_CQ_PIPELINE_EMPTY;
        output PCIE_PERST0_B;
        output PCIE_PERST1_B;
        input PCIE_POSTED_REQ_DELIVERED;
        output PCIE_RQ_SEQ_NUM0[6];
        output PCIE_RQ_SEQ_NUM1[6];
        output PCIE_RQ_SEQ_NUM_VLD[2];
        output PCIE_RQ_TAG0[8];
        output PCIE_RQ_TAG1[8];
        output PCIE_RQ_TAG_AV[4];
        output PCIE_RQ_TAG_VLD[2];
        output PCIE_TFC_NPD_AV[4];
        output PCIE_TFC_NPH_AV[4];

        input PIPE_CLK;
        input PIPE_CLK_EN;
        input PIPE_RESET_N;
        input PIPE_EQ_FS[6];
        input PIPE_EQ_LF[6];

        for ch in 0..16 {
            input "PIPE_RX{ch:02}_CHAR_IS_K"[2];
            input "PIPE_RX{ch:02}_DATA"[32];
            input "PIPE_RX{ch:02}_DATA_VALID";
            input "PIPE_RX{ch:02}_ELEC_IDLE";
            output "PIPE_RX{ch:02}_EQ_CONTROL"[2];
            input "PIPE_RX{ch:02}_EQ_DONE";
            input "PIPE_RX{ch:02}_EQ_LP_ADAPT_DONE";
            input "PIPE_RX{ch:02}_EQ_LP_LF_FS_SEL";
            input "PIPE_RX{ch:02}_EQ_LP_NEW_TX_COEFF_OR_PRESET"[18];
            input "PIPE_RX{ch:02}_PHY_STATUS";
            output "PIPE_RX{ch:02}_POLARITY";
            input "PIPE_RX{ch:02}_START_BLOCK"[2];
            input "PIPE_RX{ch:02}_STATUS"[3];
            input "PIPE_RX{ch:02}_SYNC_HEADER"[2];
            input "PIPE_RX{ch:02}_VALID";

            output "PIPE_TX{ch:02}_CHAR_IS_K"[2];
            output "PIPE_TX{ch:02}_COMPLIANCE";
            output "PIPE_TX{ch:02}_DATA"[32];
            output "PIPE_TX{ch:02}_DATA_VALID";
            output "PIPE_TX{ch:02}_ELEC_IDLE";
            input "PIPE_TX{ch:02}_EQ_COEFF"[18];
            output "PIPE_TX{ch:02}_EQ_CONTROL"[2];
            output "PIPE_TX{ch:02}_EQ_DEEMPH"[6];
            input "PIPE_TX{ch:02}_EQ_DONE";
            output "PIPE_TX{ch:02}_POWERDOWN"[2];
            output "PIPE_TX{ch:02}_START_BLOCK";
            output "PIPE_TX{ch:02}_SYNC_HEADER"[2];
        }

        output PIPE_RX_EQ_LP_LF_FS[6];
        output PIPE_RX_EQ_LP_TX_PRESET[4];

        output PIPE_TX_DEEMPH;
        output PIPE_TX_MARGIN[3];
        output PIPE_TX_RATE[2];
        output PIPE_TX_RCVR_DET;
        output PIPE_TX_RESET;
        output PIPE_TX_SWING;

        output PL_EQ_IN_PROGRESS;
        output PL_EQ_PHASE[2];
        input PL_EQ_RESET_EIEOS_COUNT;
        input PL_GEN2_UPSTREAM_PREFER_DEEMPH;
        output PL_GEN34_EQ_MISMATCH;
        input PL_GEN34_REDO_EQUALIZATION;
        input PL_GEN34_REDO_EQ_SPEED;

        input PMV_ENABLE_N;
        input PMV_SELECT[3];
        input PMV_DIVIDE[2];
        output PMV_OUT;

        input SCANENABLE_N;
        input SCANMODE_N;
        input SCANIN[173];
        output SCANOUT[173];

        input USER_SPARE_IN[32];
        output USER_SPARE_OUT[32];

        // PCIE4C new stuff from now on
        input CORE_CLK_CCIX;

        input CCIX_OPTIMIZED_TLP_TX_AND_RX_ENABLE;
        input CCIX_RX_CORRECTABLE_ERROR_DETECTED;
        input CCIX_RX_FIFO_OVERFLOW;
        input CCIX_RX_TLP_FORWARDED[2];
        input CCIX_RX_TLP_FORWARDED_LENGTH0[6];
        input CCIX_RX_TLP_FORWARDED_LENGTH1[6];
        input CCIX_RX_UNCORRECTABLE_ERROR_DETECTED;
        output CCIX_TX_CREDIT;

        output M_AXIS_CCIX_RX_TVALID;
        output M_AXIS_CCIX_RX_TUSER[46];

        input S_AXIS_CCIX_TX_TVALID;
        input S_AXIS_CCIX_TX_TDATA[256];
        input S_AXIS_CCIX_TX_TUSER[46];

        output DBG_CCIX_OUT[130];

        // TODO: attributes
    }

    bel_class CMAC {
        input CTL_CAUI4_MODE;

        input CTL_RX_CHECK_ETYPE_GCP;
        input CTL_RX_CHECK_ETYPE_GPP;
        input CTL_RX_CHECK_ETYPE_PCP;
        input CTL_RX_CHECK_ETYPE_PPP;
        input CTL_RX_CHECK_MCAST_GCP;
        input CTL_RX_CHECK_MCAST_GPP;
        input CTL_RX_CHECK_MCAST_PCP;
        input CTL_RX_CHECK_MCAST_PPP;
        input CTL_RX_CHECK_OPCODE_GCP;
        input CTL_RX_CHECK_OPCODE_GPP;
        input CTL_RX_CHECK_OPCODE_PCP;
        input CTL_RX_CHECK_OPCODE_PPP;
        input CTL_RX_CHECK_SA_GCP;
        input CTL_RX_CHECK_SA_GPP;
        input CTL_RX_CHECK_SA_PCP;
        input CTL_RX_CHECK_SA_PPP;
        input CTL_RX_CHECK_UCAST_GCP;
        input CTL_RX_CHECK_UCAST_GPP;
        input CTL_RX_CHECK_UCAST_PCP;
        input CTL_RX_CHECK_UCAST_PPP;
        input CTL_RX_ENABLE;
        input CTL_RX_ENABLE_GCP;
        input CTL_RX_ENABLE_GPP;
        input CTL_RX_ENABLE_PCP;
        input CTL_RX_ENABLE_PPP;
        input CTL_RX_FORCE_RESYNC;
        input CTL_RX_PAUSE_ACK[9];
        input CTL_RX_PAUSE_ENABLE[9];
        input CTL_RX_SYSTEMTIMERIN[80];
        input CTL_RX_TEST_PATTERN;

        input CTL_TX_ENABLE;
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE;
        input CTL_TX_LANE0_VLM_BIP7_OVERRIDE_VALUE[8];
        input CTL_TX_PAUSE_ENABLE[9];
        input CTL_TX_PAUSE_QUANTA0[16];
        input CTL_TX_PAUSE_QUANTA1[16];
        input CTL_TX_PAUSE_QUANTA2[16];
        input CTL_TX_PAUSE_QUANTA3[16];
        input CTL_TX_PAUSE_QUANTA4[16];
        input CTL_TX_PAUSE_QUANTA5[16];
        input CTL_TX_PAUSE_QUANTA6[16];
        input CTL_TX_PAUSE_QUANTA7[16];
        input CTL_TX_PAUSE_QUANTA8[16];
        input CTL_TX_PAUSE_REFRESH_TIMER0[16];
        input CTL_TX_PAUSE_REFRESH_TIMER1[16];
        input CTL_TX_PAUSE_REFRESH_TIMER2[16];
        input CTL_TX_PAUSE_REFRESH_TIMER3[16];
        input CTL_TX_PAUSE_REFRESH_TIMER4[16];
        input CTL_TX_PAUSE_REFRESH_TIMER5[16];
        input CTL_TX_PAUSE_REFRESH_TIMER6[16];
        input CTL_TX_PAUSE_REFRESH_TIMER7[16];
        input CTL_TX_PAUSE_REFRESH_TIMER8[16];
        input CTL_TX_PAUSE_REQ[9];
        input CTL_TX_PTP_VLANE_ADJUST_MODE;
        input CTL_TX_RESEND_PAUSE;
        input CTL_TX_SEND_IDLE;
        input CTL_TX_SEND_RFI;
        input CTL_TX_SYSTEMTIMERIN[80];
        input CTL_TX_TEST_PATTERN;

        input DRP_CLK;
        input DRP_EN;
        input DRP_WE;
        input DRP_ADDR[10];
        input DRP_DI[16];
        output DRP_DO[16];
        output DRP_RDY;

        input RX_CLK;
        input RX_RESET;
        output RX_DATAOUT0[128];
        output RX_DATAOUT1[128];
        output RX_DATAOUT2[128];
        output RX_DATAOUT3[128];
        output RX_ENAOUT[4];
        output RX_EOPOUT[4];
        output RX_ERROUT[4];
        output RX_LANE_ALIGNER_FILL_0[7];
        output RX_LANE_ALIGNER_FILL_1[7];
        output RX_LANE_ALIGNER_FILL_2[7];
        output RX_LANE_ALIGNER_FILL_3[7];
        output RX_LANE_ALIGNER_FILL_4[7];
        output RX_LANE_ALIGNER_FILL_5[7];
        output RX_LANE_ALIGNER_FILL_6[7];
        output RX_LANE_ALIGNER_FILL_7[7];
        output RX_LANE_ALIGNER_FILL_8[7];
        output RX_LANE_ALIGNER_FILL_9[7];
        output RX_LANE_ALIGNER_FILL_10[7];
        output RX_LANE_ALIGNER_FILL_11[7];
        output RX_LANE_ALIGNER_FILL_12[7];
        output RX_LANE_ALIGNER_FILL_13[7];
        output RX_LANE_ALIGNER_FILL_14[7];
        output RX_LANE_ALIGNER_FILL_15[7];
        output RX_LANE_ALIGNER_FILL_16[7];
        output RX_LANE_ALIGNER_FILL_17[7];
        output RX_LANE_ALIGNER_FILL_18[7];
        output RX_LANE_ALIGNER_FILL_19[7];
        output RX_MTYOUT0[4];
        output RX_MTYOUT1[4];
        output RX_MTYOUT2[4];
        output RX_MTYOUT3[4];
        output RX_PTP_PCSLANE_OUT[5];
        output RX_PTP_TSTAMP_OUT[80];
        input RX_SERDES_ALT_DATA0[16];
        input RX_SERDES_ALT_DATA1[16];
        input RX_SERDES_ALT_DATA2[16];
        input RX_SERDES_ALT_DATA3[16];
        input RX_SERDES_CLK[10];
        input RX_SERDES_DATA0[64];
        input RX_SERDES_DATA1[64];
        input RX_SERDES_DATA2[64];
        input RX_SERDES_DATA3[64];
        input RX_SERDES_DATA4[32];
        input RX_SERDES_DATA5[32];
        input RX_SERDES_DATA6[32];
        input RX_SERDES_DATA7[32];
        input RX_SERDES_DATA8[32];
        input RX_SERDES_DATA9[32];
        input RX_SERDES_RESET[10];
        output RX_SOPOUT[4];

        input TX_CLK;
        input TX_RESET;
        input TX_DATAIN0[128];
        input TX_DATAIN1[128];
        input TX_DATAIN2[128];
        input TX_DATAIN3[128];
        input TX_ENAIN[4];
        input TX_EOPIN[4];
        input TX_ERRIN[4];
        input TX_MTYIN0[4];
        input TX_MTYIN1[4];
        input TX_MTYIN2[4];
        input TX_MTYIN3[4];
        output TX_OVFOUT;
        input TX_PTP_1588OP_IN[2];
        input TX_PTP_CHKSUM_OFFSET_IN[16];
        output TX_PTP_PCSLANE_OUT[5];
        input TX_PTP_RXTSTAMP_IN[64];
        input TX_PTP_TAG_FIELD_IN[16];
        input TX_PTP_TSTAMP_OFFSET_IN[16];
        output TX_PTP_TSTAMP_OUT[80];
        output TX_PTP_TSTAMP_TAG_OUT[16];
        output TX_PTP_TSTAMP_VALID_OUT;
        input TX_PTP_UPD_CHKSUM_IN;
        output TX_RDYOUT;
        output TX_SERDES_ALT_DATA0[16];
        output TX_SERDES_ALT_DATA1[16];
        output TX_SERDES_ALT_DATA2[16];
        output TX_SERDES_ALT_DATA3[16];
        output TX_SERDES_DATA0[64];
        output TX_SERDES_DATA1[64];
        output TX_SERDES_DATA2[64];
        output TX_SERDES_DATA3[64];
        output TX_SERDES_DATA4[32];
        output TX_SERDES_DATA5[32];
        output TX_SERDES_DATA6[32];
        output TX_SERDES_DATA7[32];
        output TX_SERDES_DATA8[32];
        output TX_SERDES_DATA9[32];
        input TX_SOPIN[4];
        output TX_UNFOUT;

        // a bunch of those are shorter in u+
        output STAT_RX_ALIGNED;
        output STAT_RX_ALIGNED_ERR;
        output STAT_RX_BAD_CODE[7];
        output STAT_RX_BAD_FCS[4];
        output STAT_RX_BAD_PREAMBLE;
        output STAT_RX_BAD_SFD;
        output STAT_RX_BIP_ERR_0;
        output STAT_RX_BIP_ERR_1;
        output STAT_RX_BIP_ERR_2;
        output STAT_RX_BIP_ERR_3;
        output STAT_RX_BIP_ERR_4;
        output STAT_RX_BIP_ERR_5;
        output STAT_RX_BIP_ERR_6;
        output STAT_RX_BIP_ERR_7;
        output STAT_RX_BIP_ERR_8;
        output STAT_RX_BIP_ERR_9;
        output STAT_RX_BIP_ERR_10;
        output STAT_RX_BIP_ERR_11;
        output STAT_RX_BIP_ERR_12;
        output STAT_RX_BIP_ERR_13;
        output STAT_RX_BIP_ERR_14;
        output STAT_RX_BIP_ERR_15;
        output STAT_RX_BIP_ERR_16;
        output STAT_RX_BIP_ERR_17;
        output STAT_RX_BIP_ERR_18;
        output STAT_RX_BIP_ERR_19;
        output STAT_RX_BLOCK_LOCK[20];
        output STAT_RX_BROADCAST;
        output STAT_RX_FRAGMENT[4];
        output STAT_RX_FRAMING_ERR_0[4];
        output STAT_RX_FRAMING_ERR_1[4];
        output STAT_RX_FRAMING_ERR_2[4];
        output STAT_RX_FRAMING_ERR_3[4];
        output STAT_RX_FRAMING_ERR_4[4];
        output STAT_RX_FRAMING_ERR_5[4];
        output STAT_RX_FRAMING_ERR_6[4];
        output STAT_RX_FRAMING_ERR_7[4];
        output STAT_RX_FRAMING_ERR_8[4];
        output STAT_RX_FRAMING_ERR_9[4];
        output STAT_RX_FRAMING_ERR_10[4];
        output STAT_RX_FRAMING_ERR_11[4];
        output STAT_RX_FRAMING_ERR_12[4];
        output STAT_RX_FRAMING_ERR_13[4];
        output STAT_RX_FRAMING_ERR_14[4];
        output STAT_RX_FRAMING_ERR_15[4];
        output STAT_RX_FRAMING_ERR_16[4];
        output STAT_RX_FRAMING_ERR_17[4];
        output STAT_RX_FRAMING_ERR_18[4];
        output STAT_RX_FRAMING_ERR_19[4];
        output STAT_RX_FRAMING_ERR_VALID_0;
        output STAT_RX_FRAMING_ERR_VALID_1;
        output STAT_RX_FRAMING_ERR_VALID_2;
        output STAT_RX_FRAMING_ERR_VALID_3;
        output STAT_RX_FRAMING_ERR_VALID_4;
        output STAT_RX_FRAMING_ERR_VALID_5;
        output STAT_RX_FRAMING_ERR_VALID_6;
        output STAT_RX_FRAMING_ERR_VALID_7;
        output STAT_RX_FRAMING_ERR_VALID_8;
        output STAT_RX_FRAMING_ERR_VALID_9;
        output STAT_RX_FRAMING_ERR_VALID_10;
        output STAT_RX_FRAMING_ERR_VALID_11;
        output STAT_RX_FRAMING_ERR_VALID_12;
        output STAT_RX_FRAMING_ERR_VALID_13;
        output STAT_RX_FRAMING_ERR_VALID_14;
        output STAT_RX_FRAMING_ERR_VALID_15;
        output STAT_RX_FRAMING_ERR_VALID_16;
        output STAT_RX_FRAMING_ERR_VALID_17;
        output STAT_RX_FRAMING_ERR_VALID_18;
        output STAT_RX_FRAMING_ERR_VALID_19;
        output STAT_RX_GOT_SIGNAL_OS;
        output STAT_RX_HI_BER;
        output STAT_RX_INRANGEERR;
        output STAT_RX_INTERNAL_LOCAL_FAULT;
        output STAT_RX_JABBER;
        output STAT_RX_LANE0_VLM_BIP7[8];
        output STAT_RX_LANE0_VLM_BIP7_VALID;
        output STAT_RX_LOCAL_FAULT;
        output STAT_RX_MF_ERR[20];
        output STAT_RX_MF_LEN_ERR[20];
        output STAT_RX_MF_REPEAT_ERR[20];
        output STAT_RX_MISALIGNED;
        output STAT_RX_MULTICAST;
        output STAT_RX_OVERSIZE;
        output STAT_RX_PACKET_1024_1518_BYTES;
        output STAT_RX_PACKET_128_255_BYTES;
        output STAT_RX_PACKET_1519_1522_BYTES;
        output STAT_RX_PACKET_1523_1548_BYTES;
        output STAT_RX_PACKET_1549_2047_BYTES;
        output STAT_RX_PACKET_2048_4095_BYTES;
        output STAT_RX_PACKET_256_511_BYTES;
        output STAT_RX_PACKET_4096_8191_BYTES;
        output STAT_RX_PACKET_512_1023_BYTES;
        output STAT_RX_PACKET_64_BYTES;
        output STAT_RX_PACKET_65_127_BYTES;
        output STAT_RX_PACKET_8192_9215_BYTES;
        output STAT_RX_PACKET_BAD_FCS;
        output STAT_RX_PACKET_LARGE;
        output STAT_RX_PACKET_SMALL[4];
        output STAT_RX_PAUSE;
        output STAT_RX_PAUSE_QUANTA0[16];
        output STAT_RX_PAUSE_QUANTA1[16];
        output STAT_RX_PAUSE_QUANTA2[16];
        output STAT_RX_PAUSE_QUANTA3[16];
        output STAT_RX_PAUSE_QUANTA4[16];
        output STAT_RX_PAUSE_QUANTA5[16];
        output STAT_RX_PAUSE_QUANTA6[16];
        output STAT_RX_PAUSE_QUANTA7[16];
        output STAT_RX_PAUSE_QUANTA8[16];
        output STAT_RX_PAUSE_REQ[9];
        output STAT_RX_PAUSE_VALID[9];
        output STAT_RX_RECEIVED_LOCAL_FAULT;
        output STAT_RX_REMOTE_FAULT;
        output STAT_RX_STATUS;
        output STAT_RX_STOMPED_FCS[4];
        output STAT_RX_SYNCED[20];
        output STAT_RX_SYNCED_ERR[20];
        output STAT_RX_TEST_PATTERN_MISMATCH[3];
        output STAT_RX_TOOLONG;
        output STAT_RX_TOTAL_BYTES[8];
        output STAT_RX_TOTAL_GOOD_BYTES[14];
        output STAT_RX_TOTAL_GOOD_PACKETS;
        output STAT_RX_TOTAL_PACKETS[4];
        output STAT_RX_TRUNCATED;
        output STAT_RX_UNDERSIZE[4];
        output STAT_RX_UNICAST;
        output STAT_RX_USER_PAUSE;
        output STAT_RX_VLAN;
        output STAT_RX_VL_DEMUXED[20];
        output STAT_RX_VL_NUMBER_0[5];
        output STAT_RX_VL_NUMBER_1[5];
        output STAT_RX_VL_NUMBER_2[5];
        output STAT_RX_VL_NUMBER_3[5];
        output STAT_RX_VL_NUMBER_4[5];
        output STAT_RX_VL_NUMBER_5[5];
        output STAT_RX_VL_NUMBER_6[5];
        output STAT_RX_VL_NUMBER_7[5];
        output STAT_RX_VL_NUMBER_8[5];
        output STAT_RX_VL_NUMBER_9[5];
        output STAT_RX_VL_NUMBER_10[5];
        output STAT_RX_VL_NUMBER_11[5];
        output STAT_RX_VL_NUMBER_12[5];
        output STAT_RX_VL_NUMBER_13[5];
        output STAT_RX_VL_NUMBER_14[5];
        output STAT_RX_VL_NUMBER_15[5];
        output STAT_RX_VL_NUMBER_16[5];
        output STAT_RX_VL_NUMBER_17[5];
        output STAT_RX_VL_NUMBER_18[5];
        output STAT_RX_VL_NUMBER_19[5];
        output STAT_TX_BAD_FCS;
        output STAT_TX_BROADCAST;
        output STAT_TX_FRAME_ERROR;
        output STAT_TX_LOCAL_FAULT;
        output STAT_TX_MULTICAST;
        output STAT_TX_PACKET_1024_1518_BYTES;
        output STAT_TX_PACKET_128_255_BYTES;
        output STAT_TX_PACKET_1519_1522_BYTES;
        output STAT_TX_PACKET_1523_1548_BYTES;
        output STAT_TX_PACKET_1549_2047_BYTES;
        output STAT_TX_PACKET_2048_4095_BYTES;
        output STAT_TX_PACKET_256_511_BYTES;
        output STAT_TX_PACKET_4096_8191_BYTES;
        output STAT_TX_PACKET_512_1023_BYTES;
        output STAT_TX_PACKET_64_BYTES;
        output STAT_TX_PACKET_65_127_BYTES;
        output STAT_TX_PACKET_8192_9215_BYTES;
        output STAT_TX_PACKET_LARGE;
        output STAT_TX_PACKET_SMALL;
        output STAT_TX_PAUSE;
        output STAT_TX_PAUSE_VALID[9];
        output STAT_TX_PTP_FIFO_READ_ERROR;
        output STAT_TX_PTP_FIFO_WRITE_ERROR;
        output STAT_TX_TOTAL_BYTES[7];
        output STAT_TX_TOTAL_GOOD_BYTES[14];
        output STAT_TX_TOTAL_GOOD_PACKETS;
        output STAT_TX_TOTAL_PACKETS;
        output STAT_TX_UNICAST;
        output STAT_TX_USER_PAUSE;
        output STAT_TX_VLAN;

        input TEST_MODE_N;
        input TEST_RESET;

        input SCAN_EN_N;
        // u+ only
        input SCAN_CLK;
        input SCAN_IN[268];
        output SCAN_OUT[268];
        // u only
        input SCAN_IN_CMAC[182];
        input SCAN_IN_DRPCTRL[13];
        output SCAN_OUT_CMAC[182];
        output SCAN_OUT_DRPCTRL[13];

        // u+ only stuff from now on
        output CFG_RESET_CSSD;
        output CSSD_CLK_STOP_DONE;
        input CSSD_CLK_STOP_EVENT;
        input CSSD_RESETN;
        output GRESTORE_CSSD;
        output GWE_CSSD;

        input CTL_RSFEC_ENABLE_TRANSCODER_BYPASS_MODE;
        input CTL_RSFEC_IEEE_ERROR_INDICATION_MODE;
        input CTL_RX_RSFEC_ENABLE;
        input CTL_RX_RSFEC_ENABLE_CORRECTION;
        input CTL_RX_RSFEC_ENABLE_INDICATION;
        input CTL_TX_RSFEC_ENABLE;
        input CTL_TX_SEND_LFI;

        input RSFEC_BYPASS_RX_DIN[330];
        input RSFEC_BYPASS_RX_DIN_CW_START;
        output RSFEC_BYPASS_RX_DOUT[330];
        output RSFEC_BYPASS_RX_DOUT_CW_START;
        output RSFEC_BYPASS_RX_DOUT_VALID;
        input RSFEC_BYPASS_TX_DIN[330];
        input RSFEC_BYPASS_TX_DIN_CW_START;
        output RSFEC_BYPASS_TX_DOUT[330];
        output RSFEC_BYPASS_TX_DOUT_CW_START;
        output RSFEC_BYPASS_TX_DOUT_VALID;

        output RX_OTN_BIP8_0[8];
        output RX_OTN_BIP8_1[8];
        output RX_OTN_BIP8_2[8];
        output RX_OTN_BIP8_3[8];
        output RX_OTN_BIP8_4[8];
        output RX_OTN_DATA_0[66];
        output RX_OTN_DATA_1[66];
        output RX_OTN_DATA_2[66];
        output RX_OTN_DATA_3[66];
        output RX_OTN_DATA_4[66];
        output RX_OTN_ENA;
        output RX_OTN_LANE0;
        output RX_OTN_VLMARKER;

        output RX_PREOUT[56];
        input TX_PREIN[56];

        output STAT_RX_RSFEC_AM_LOCK[4];
        output STAT_RX_RSFEC_CORRECTED_CW_INC;
        output STAT_RX_RSFEC_CW_INC;
        output STAT_RX_RSFEC_ERR_COUNT0_INC[3];
        output STAT_RX_RSFEC_ERR_COUNT1_INC[3];
        output STAT_RX_RSFEC_ERR_COUNT2_INC[3];
        output STAT_RX_RSFEC_ERR_COUNT3_INC[3];
        output STAT_RX_RSFEC_HI_SER;
        output STAT_RX_RSFEC_LANE_ALIGNMENT_STATUS;
        output STAT_RX_RSFEC_LANE_FILL_0[14];
        output STAT_RX_RSFEC_LANE_FILL_1[14];
        output STAT_RX_RSFEC_LANE_FILL_2[14];
        output STAT_RX_RSFEC_LANE_FILL_3[14];
        output STAT_RX_RSFEC_LANE_MAPPING[8];
        output STAT_RX_RSFEC_RSVD[32];
        output STAT_RX_RSFEC_UNCORRECTED_CW_INC;

        // TODO: attributes
    }

    bel_class ILKN {
        input DRP_CLK;
        input DRP_EN;
        input DRP_WE;
        input DRP_ADDR[10];
        input DRP_DI[16];
        output DRP_RDY;
        output DRP_DO[16];

        input CORE_CLK;
        input LBUS_CLK;

        input CTL_RX_FORCE_RESYNC;
        input CTL_RX_RETRANS_ACK;
        input CTL_RX_RETRANS_ENABLE;
        input CTL_RX_RETRANS_ERRIN;
        input CTL_RX_RETRANS_FORCE_REQ;
        input CTL_RX_RETRANS_RESET;
        input CTL_RX_RETRANS_RESET_MODE;
        input CTL_TX_DIAGWORD_INTFSTAT;
        input CTL_TX_DIAGWORD_LANESTAT[12];
        input CTL_TX_ENABLE;
        input CTL_TX_ERRINJ_BITERR_GO;
        input CTL_TX_ERRINJ_BITERR_LANE[4];
        input CTL_TX_FC_STAT[256];
        input CTL_TX_MUBITS[8];
        input CTL_TX_RETRANS_ENABLE;
        input CTL_TX_RETRANS_RAM_PERRIN;
        input CTL_TX_RETRANS_RAM_RDATA[644];
        input CTL_TX_RETRANS_REQ;
        input CTL_TX_RETRANS_REQ_VALID;
        input CTL_TX_RLIM_DELTA[12];
        input CTL_TX_RLIM_ENABLE;
        input CTL_TX_RLIM_INTV[8];
        input CTL_TX_RLIM_MAX[12];

        output RX_BYPASS_DATAOUT0[66];
        output RX_BYPASS_DATAOUT1[66];
        output RX_BYPASS_DATAOUT2[66];
        output RX_BYPASS_DATAOUT3[66];
        output RX_BYPASS_DATAOUT4[66];
        output RX_BYPASS_DATAOUT5[66];
        output RX_BYPASS_DATAOUT6[66];
        output RX_BYPASS_DATAOUT7[66];
        output RX_BYPASS_DATAOUT8[66];
        output RX_BYPASS_DATAOUT9[66];
        output RX_BYPASS_DATAOUT10[66];
        output RX_BYPASS_DATAOUT11[66];
        output RX_BYPASS_ENAOUT[12];
        input RX_BYPASS_FORCE_REALIGNIN;
        output RX_BYPASS_IS_AVAILOUT[12];
        output RX_BYPASS_IS_BADLYFRAMEDOUT[12];
        output RX_BYPASS_IS_OVERFLOWOUT[12];
        output RX_BYPASS_IS_SYNCEDOUT[12];
        output RX_BYPASS_IS_SYNCWORDOUT[12];
        input RX_BYPASS_RDIN;
        output RX_CHANOUT0[11];
        output RX_CHANOUT1[11];
        output RX_CHANOUT2[11];
        output RX_CHANOUT3[11];
        output RX_DATAOUT0[128];
        output RX_DATAOUT1[128];
        output RX_DATAOUT2[128];
        output RX_DATAOUT3[128];
        output RX_ENAOUT[4];
        output RX_EOPOUT[4];
        output RX_ERROUT[4];
        output RX_MTYOUT0[4];
        output RX_MTYOUT1[4];
        output RX_MTYOUT2[4];
        output RX_MTYOUT3[4];
        output RX_OVFOUT;
        input RX_RESET;
        input RX_SERDES_CLK[12];
        input RX_SERDES_DATA0[64];
        input RX_SERDES_DATA1[64];
        input RX_SERDES_DATA2[64];
        input RX_SERDES_DATA3[64];
        input RX_SERDES_DATA4[64];
        input RX_SERDES_DATA5[64];
        input RX_SERDES_DATA6[64];
        input RX_SERDES_DATA7[64];
        input RX_SERDES_DATA8[64];
        input RX_SERDES_DATA9[64];
        input RX_SERDES_DATA10[64];
        input RX_SERDES_DATA11[64];
        input RX_SERDES_RESET[12];
        output RX_SOPOUT[4];

        output STAT_RX_ALIGNED;
        output STAT_RX_ALIGNED_ERR;
        output STAT_RX_BAD_TYPE_ERR[12];
        output STAT_RX_BURSTMAX_ERR;
        output STAT_RX_BURST_ERR;
        output STAT_RX_CRC24_ERR;
        output STAT_RX_CRC32_ERR[12];
        output STAT_RX_CRC32_VALID[12];
        output STAT_RX_DESCRAM_ERR[12];
        output STAT_RX_DIAGWORD_INTFSTAT[12];
        output STAT_RX_DIAGWORD_LANESTAT[12];
        output STAT_RX_FC_STAT[256];
        output STAT_RX_FRAMING_ERR[12];
        output STAT_RX_MEOP_ERR;
        output STAT_RX_MF_ERR[12];
        output STAT_RX_MF_LEN_ERR[12];
        output STAT_RX_MF_REPEAT_ERR[12];
        output STAT_RX_MISALIGNED;
        output STAT_RX_MSOP_ERR;
        output STAT_RX_MUBITS[8];
        output STAT_RX_MUBITS_UPDATED;
        output STAT_RX_OVERFLOW_ERR;
        output STAT_RX_RETRANS_CRC24_ERR;
        output STAT_RX_RETRANS_DISC;
        output STAT_RX_RETRANS_LATENCY[16];
        output STAT_RX_RETRANS_REQ;
        output STAT_RX_RETRANS_RETRY_ERR;
        output STAT_RX_RETRANS_SEQ[8];
        output STAT_RX_RETRANS_SEQ_UPDATED;
        output STAT_RX_RETRANS_STATE[3];
        output STAT_RX_RETRANS_SUBSEQ[5];
        output STAT_RX_RETRANS_WDOG_ERR;
        output STAT_RX_RETRANS_WRAP_ERR;
        output STAT_RX_SYNCED[12];
        output STAT_RX_SYNCED_ERR[12];
        output STAT_RX_WORD_SYNC[12];
        output STAT_TX_BURST_ERR;
        output STAT_TX_ERRINJ_BITERR_DONE;
        output STAT_TX_OVERFLOW_ERR;
        output STAT_TX_RETRANS_BURST_ERR;
        output STAT_TX_RETRANS_BUSY;
        output STAT_TX_RETRANS_RAM_PERROUT;
        output STAT_TX_RETRANS_RAM_RADDR[9];
        output STAT_TX_RETRANS_RAM_RD_B[4];
        output STAT_TX_RETRANS_RAM_RSEL[2];
        output STAT_TX_RETRANS_RAM_WADDR[9];
        output STAT_TX_RETRANS_RAM_WDATA[644];
        output STAT_TX_RETRANS_RAM_WE_B[4];
        output STAT_TX_UNDERFLOW_ERR;

        input TX_BCTLIN[4];
        input TX_BYPASS_CTRLIN[12];
        input TX_BYPASS_DATAIN0[64];
        input TX_BYPASS_DATAIN1[64];
        input TX_BYPASS_DATAIN2[64];
        input TX_BYPASS_DATAIN3[64];
        input TX_BYPASS_DATAIN4[64];
        input TX_BYPASS_DATAIN5[64];
        input TX_BYPASS_DATAIN6[64];
        input TX_BYPASS_DATAIN7[64];
        input TX_BYPASS_DATAIN8[64];
        input TX_BYPASS_DATAIN9[64];
        input TX_BYPASS_DATAIN10[64];
        input TX_BYPASS_DATAIN11[64];
        input TX_BYPASS_ENAIN;
        input TX_BYPASS_GEARBOX_SEQIN[8];
        input TX_BYPASS_MFRAMER_STATEIN[4];
        input TX_CHANIN0[11];
        input TX_CHANIN1[11];
        input TX_CHANIN2[11];
        input TX_CHANIN3[11];
        input TX_DATAIN0[128];
        input TX_DATAIN1[128];
        input TX_DATAIN2[128];
        input TX_DATAIN3[128];
        input TX_ENAIN[4];
        input TX_EOPIN[4];
        input TX_ERRIN[4];
        input TX_MTYIN0[4];
        input TX_MTYIN1[4];
        input TX_MTYIN2[4];
        input TX_MTYIN3[4];
        output TX_OVFOUT;
        output TX_RDYOUT;
        input TX_RESET;
        output TX_SERDES_DATA0[64];
        output TX_SERDES_DATA1[64];
        output TX_SERDES_DATA2[64];
        output TX_SERDES_DATA3[64];
        output TX_SERDES_DATA4[64];
        output TX_SERDES_DATA5[64];
        output TX_SERDES_DATA6[64];
        output TX_SERDES_DATA7[64];
        output TX_SERDES_DATA8[64];
        output TX_SERDES_DATA9[64];
        output TX_SERDES_DATA10[64];
        output TX_SERDES_DATA11[64];
        input TX_SERDES_REFCLK;
        input TX_SERDES_REFCLK_RESET;
        input TX_SOPIN[4];

        input TEST_MODE_N;
        input TEST_RESET;
        input SCAN_EN_N;
        // u only
        input SCAN_IN_DRPCTRL[15];
        input SCAN_IN_ILMAC[250];
        output SCAN_OUT_DRPCTRL[15];
        output SCAN_OUT_ILMAC[250];
        // u+ only
        input SCAN_CLK;
        input SCAN_IN[265];
        output SCAN_OUT[265];
        output CFG_RESET_CSSD;
        output CSSD_CLK_STOP_DONE;
        input CSSD_CLK_STOP_EVENT;
        input CSSD_RESETN;
        output GRESTORE_CSSD;
        output GWE_CSSD;

        // TODO: attributes
    }

    bel_class BLI_HBM_APB_INTF {
        input PCLK;
        input PRESET_N;
        input PENABLE;
        input PSEL;
        input PWRITE;
        input PADDR[22];
        input PWDATA[32];
        output PRDATA_PIPE[32];
        output PREADY_PIPE;
        output PSLVERR_PIPE;

        output TEMP_PIPE[3];
        output CATTRIP_PIPE;

        // TODO: attributes
    }

    bel_class BLI_HBM_AXI_INTF {
        input ACLK;
        input ARESET_N;

        output ARREADY_PIPE;
        input ARVALID;
        input ARADDR[37];
        input ARBURST[2];
        input ARID[6];
        input ARLEN[4];
        input ARSIZE[3];

        output AWREADY_PIPE;
        input AWVALID;
        input AWADDR[37];
        input AWBURST[2];
        input AWID[6];
        input AWLEN[4];
        input AWSIZE[3];

        input BREADY;
        output BVALID_PIPE;
        output BID_PIPE[6];
        output BRESP_PIPE[2];

        input RREADY;
        output RVALID_PIPE;
        output RDATA_PIPE[256];
        output RDATA_PARITY_PIPE[32];
        output RID_PIPE[6];
        output RLAST_PIPE;
        output RRESP_PIPE[2];

        output WREADY_PIPE;
        input WVALID;
        input WDATA[256];
        input WDATA_PARITY[32];
        input WLAST;
        input WSTRB[32];

        output DFI_CLK_BUF;
        output DFI_RST_N_BUF;
        output DFI_AW_AERR_N_PIPE[2];
        output DFI_CTRLUPD_ACK_PIPE;
        output DFI_DBI_BYTE_DISABLE_PIPE[8];
        output DFI_DW_RDDATA_DBI_PIPE[21];
        output DFI_DW_RDDATA_DERR_PIPE[8];
        output DFI_DW_RDDATA_PAR_VALID_PIPE[2];
        output DFI_DW_RDDATA_VALID_PIPE[2];
        output DFI_INIT_COMPLETE_PIPE;
        input DFI_LP_PWR_X_REQ;
        output DFI_PHYUPD_REQ_PIPE;
        output DFI_PHYUPD_TYPE_PIPE;
        output DFI_PHY_LP_STATE_PIPE;

        input BSCAN_CK;
        input BLI_SCAN_ENABLE;
        input BLI_SCAN_IN[8];
        output BLI_SCAN_OUT[8];
        input DBG_IN[24];
        output DBG_OUT_PIPE[24];

        input HBM_MBIST_EN;
        input HBM_SCAN_CK;
        input HBM_SCAN_ENABLE;
        input HBM_SCAN_IN[2];
        input HBM_SCAN_MODE;
        output HBM_SCAN_OUT_BUF[2];
        input HBM_SCAN_RST_N;

        // TODO: attributes
    }

    bel_class FE {
        input CORE_CLK;
        input RESET_N;

        input DFTRAM_BYPASS_N;
        output INTERRUPT;

        input S_AXI_ACLK;
        output S_AXI_ARREADY;
        input S_AXI_ARVALID;
        input S_AXI_ARADDR[18];
        output S_AXI_AWREADY;
        input S_AXI_AWVALID;
        input S_AXI_AWADDR[18];
        input S_AXI_BREADY;
        output S_AXI_BVALID;
        input S_AXI_RREADY;
        output S_AXI_RVALID;
        output S_AXI_RDATA[32];
        output S_AXI_WREADY;
        input S_AXI_WVALID;
        input S_AXI_WDATA[32];

        input S_AXIS_CTRL_ACLK;
        output S_AXIS_CTRL_TREADY;
        input S_AXIS_CTRL_TVALID;
        input S_AXIS_CTRL_TDATA[32];

        input M_AXIS_STATUS_ACLK;
        input M_AXIS_STATUS_TREADY;
        output M_AXIS_STATUS_TVALID;
        output M_AXIS_STATUS_TDATA[32];

        input S_AXIS_DIN_ACLK;
        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[512];
        input S_AXIS_DIN_TLAST;

        input S_AXIS_DIN_WORDS_ACLK;
        output S_AXIS_DIN_WORDS_TREADY;
        input S_AXIS_DIN_WORDS_TVALID;
        input S_AXIS_DIN_WORDS_TDATA[32];
        input S_AXIS_DIN_WORDS_TLAST;

        input S_AXIS_DOUT_WORDS_ACLK;
        output S_AXIS_DOUT_WORDS_TREADY;
        input S_AXIS_DOUT_WORDS_TVALID;
        input S_AXIS_DOUT_WORDS_TDATA[32];
        input S_AXIS_DOUT_WORDS_TLAST;

        input M_AXIS_DOUT_ACLK;
        input M_AXIS_DOUT_TREADY;
        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[512];
        output M_AXIS_DOUT_TLAST;

        input DEBUG_CLK_EN;
        output DEBUG_DOUT[400];
        input DEBUG_EN;
        output DEBUG_PHASE;
        input DEBUG_SEL_IN[4];

        input SPARE_IN[16];
        output SPARE_OUT[16];

        output MBIST_COMPSTAT;
        input MBIST_TCK;
        input MBIST_TDI;
        output MBIST_TDO;
        input MBIST_TMS;
        input MBIST_TRST;

        input TEST_MODE_PIN_CHAR_N;

        input SCAN_CLK;
        input SCANENABLE_N;
        input SCANMODE_N;
        input SCANIN[200];
        output SCANOUT[200];

        output CSSD_CLK_STOP_DONE;
        input CSSD_CLK_STOP_EVENT[4];
        input CSSD_RST_N;
        input CTL_CSSD_EN_N;
        input CTL_CSSD_MRKR_START_INIT[16];
        input CTL_CSSD_MRKR_STOP_INIT[16];
        input CTL_CSSD_ROOT_CLK_DIS[8];
        input CTL_CSSD_ROOT_CLK_SEL[3];
        input CTL_CSSD_SNGL_CHAIN_MD_N;
        input CTL_CSSD_STOP_COUNT[48];
    }

    // The DFE bels.
    //
    // The blocks making up the DFE are distributed among 7 tile classes, named DFE_[A-G], basically
    // on the "wherever it fits" basis.  The whole DFE is made of the following tiles and bels:
    //
    // - 1× `DFE_A`: 2×`DFE_FFT`, 1×`DFE_PRACH`
    // - 5× `DFE_B`: 2×`DFE_FFT`, 2×`DFE_FIR`
    // - 5× `DFE_C`: 2×`DFE_MATRIX_FIR`, 2×`DFE_DUC_DDC`
    // - 4× `DFE_D`: 1×`DFE_FFT`, 2×`DFE_CFR`, 1×`DFE_PQ_INTERP`
    // - 5× `DFE_E`: 3×`DFE_NL_FIR`
    // - 1× `DFE_F`: 1×`DFE_NL_FIR`, 4×`DFE_PQ_INTERP`
    // - 2× `DFE_G`: 5×`DFE_PQ_DECIM`
    //
    // The grand total is thus:
    //
    // - 16×`DFE_FFT`
    // - 16×`DFE_NL_FIR`
    // - 10×`DFE_FIR`
    // - 10×`DFE_MATRIX_FIR`,
    // - 10×`DFE_DUC_DDC`
    // - 10×`DFE_PQ_DECIM`
    // - 8×`DFE_CFR`
    // - 8×`DFE_PQ_INTERP`
    // - 1×`DFE_PRACH`
    //
    // Each DFE tile vertically occupies one region. The `DFE_A` and `DFE_G` tiles are located
    // in hard IP columns. `DFE_[BCE]` each have their own dedicated column type.
    // `DFE_D` and `DFE_F` are mixed within another dedicated column type.

    bel_class DFE_FFT {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[16];
        output STATUS[23];
        input SHUTDOWN_IN;
        output SHUTDOWN_OUT;

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[32];
        input S_AXIS_DIN_TLAST;
        input S_AXIS_DIN_TUSER[25];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[58];
        output M_AXIS_DOUT_TLAST;
        output M_AXIS_DOUT_TUSER[33];

        input JTAG_CLK;
        input TEST_IN[16];
        output TEST_OUT[32];
        input MBIST_IN[8];
        output MBIST_OUT[2];
        input SCAN_DATA_IN[40];
        output SCAN_DATA_OUT[40];
    }

    bel_class DFE_PRACH {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[311];
        output STATUS[581];

        input S_AXI_PARAM_ACLK;
        input S_AXI_PARAM_ARESETN;
        output S_AXI_PARAM_ARREADY;
        input S_AXI_PARAM_ARVALID;
        input S_AXI_PARAM_ARADDR[18];
        output S_AXI_PARAM_AWREADY;
        input S_AXI_PARAM_AWVALID;
        input S_AXI_PARAM_AWADDR[18];
        input S_AXI_PARAM_BREADY;
        output S_AXI_PARAM_BVALID;
        input S_AXI_PARAM_RREADY;
        output S_AXI_PARAM_RVALID;
        output S_AXI_PARAM_RDATA[32];
        output S_AXI_PARAM_WREADY;
        input S_AXI_PARAM_WVALID;
        input S_AXI_PARAM_WDATA[32];

        output S_AXIS_DIN_TREADY[6];
        input S_AXIS_DIN_TVALID[6];
        for i in 0..6 {
           input "S_AXIS_DIN_{i}_TDATA"[32];
           input "S_AXIS_DIN_{i}_TID"[4];
        }

        output M_AXIS_DOUT_TVALID[2];
        for i in 0..2 {
           output "M_AXIS_DOUT_{i}_TDATA"[32];
           output "M_AXIS_DOUT_{i}_TID"[7];
        }

        input TEST_IN[16];
        output TEST_OUT[8];
        input SCAN_DATA_IN[850];
        output SCAN_DATA_OUT[850];
    }

    bel_class DFE_FIR {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[16];
        output STATUS[8];

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[36];
        input S_AXIS_DIN_TID[4];
        input S_AXIS_DIN_TUSER[6];

        output S_AXIS_COEFF_TREADY;
        input S_AXIS_COEFF_TVALID;
        input S_AXIS_COEFF_TDATA[16];
        input S_AXIS_COEFF_TLAST;
        input S_AXIS_COEFF_TUSER[5];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[36];
        output M_AXIS_DOUT_TID[4];
        output M_AXIS_DOUT_TUSER[4];

        input TEST_IN[16];
        output TEST_OUT;
        input SCAN_DATA_IN[401];
        output SCAN_DATA_OUT[401];
    }

    bel_class DFE_MATRIX_FIR {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[11];
        output STATUS[8];

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[72];
        input S_AXIS_DIN_TUSER[4];

        output S_AXIS_COEFF_TREADY;
        input S_AXIS_COEFF_TVALID;
        input S_AXIS_COEFF_TDATA[16];
        input S_AXIS_COEFF_TLAST;
        input S_AXIS_COEFF_TUSER[2];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[72];
        output M_AXIS_DOUT_TUSER[4];

        input TEST_IN[16];
        output TEST_OUT;
        input SCAN_DATA_IN[9];
        output SCAN_DATA_OUT[9];
    }

    bel_class DFE_DUC_DDC {
        input CLK;
        input IF_ENABLE;

        input DATA_NCO_0_BYPASS[32];
        input DATA_NCO_1_BYPASS[32];
        input DATA_NCO_2_BYPASS[32];
        input DATA_NCO_3_BYPASS[32];
        input DATA_NCO_4_BYPASS[32];

        input FIR_RESET_N;
        input FIR_ENABLE;
        input FIR_BYPASS_DN;
        input FIR_CONTROL[47];
        output FIR_STATUS[28];

        output S_AXIS_FIR_DN_TREADY;
        input S_AXIS_FIR_DN_TVALID;
        input S_AXIS_FIR_DN_TDATA[36];
        input S_AXIS_FIR_DN_TID[4];
        input S_AXIS_FIR_DN_TUSER[3];

        output S_AXIS_FIR_UP_TREADY[16];
        input S_AXIS_FIR_UP_TVALID[16];
        for i in 0..16 {
            input "S_AXIS_FIR_UP_{i}_TDATA"[36];
        }

        output M_AXIS_FIR_DN_TVALID[16];
        for i in 0..16 {
            output "M_AXIS_FIR_DN_{i}_TDATA"[36];
        }

        output M_AXIS_FIR_UP_TVALID;
        output M_AXIS_FIR_UP_TDATA[36];
        output M_AXIS_FIR_UP_TID[4];

        input FIR_TEST_IN[16];
        input FIR_SCAN_DATA_IN[170];

        input MIX_RESET_N;
        input MIX_ENABLE;
        input MIX_BYPASS_UP;
        input MIX_CONTROL[158];
        output MIX_STATUS[60];

        input S_AXI_MIX_ACLK;
        input S_AXI_MIX_ARESET_N;
        output S_AXI_MIX_ARREADY;
        input S_AXI_MIX_ARVALID;
        input S_AXI_MIX_ARADDR[18];
        output S_AXI_MIX_AWREADY;
        input S_AXI_MIX_AWVALID;
        input S_AXI_MIX_AWADDR[18];
        input S_AXI_MIX_BREADY;
        output S_AXI_MIX_BVALID;
        input S_AXI_MIX_RREADY;
        output S_AXI_MIX_RVALID;
        output S_AXI_MIX_RDATA[32];
        output S_AXI_MIX_WREADY;
        input S_AXI_MIX_WVALID;
        input S_AXI_MIX_WDATA[32];

        output S_AXIS_DIN_MIX_TREADY[2];
        input S_AXIS_DIN_MIX_TVALID[2];
        input S_AXIS_DIN_MIX_0_TDATA[42];
        input S_AXIS_DIN_MIX_1_TDATA[42];

        output S_AXIS_MIX_DN_TREADY[16];
        input S_AXIS_MIX_DN_TVALID[16];
        for i in 0..16 {
            input "S_AXIS_MIX_DN_{i}_TDATA"[36];
        }

        output S_AXIS_MIX_FAB_DN_TREADY[4];
        input S_AXIS_MIX_FAB_DN_TVALID[4];
        for i in 0..4 {
            input "S_AXIS_MIX_FAB_DN_{i}_TDATA"[36];
        }

        output S_AXIS_MIX_UP_TREADY[4];
        input S_AXIS_MIX_UP_TVALID[4];
        for i in 0..4 {
            input "S_AXIS_MIX_UP_{i}_TDATA"[36];
        }

        output M_AXIS_MIX_DN_TVALID[4];
        for i in 0..4 {
            output "M_AXIS_MIX_DN_{i}_TDATA"[42];
        }

        output M_AXIS_MIX_FAB_UP_TVALID[4];
        for i in 0..4 {
            output "M_AXIS_MIX_FAB_UP_{i}_TDATA"[36];
        }

        output M_AXIS_MIX_UP_TVALID[16];
        for i in 0..16 {
            output "M_AXIS_MIX_UP_{i}_TDATA"[36];
        }

        input MIX_TEST_IN[16];
        input MIX_SCAN_DATA_IN[30];
    }

    bel_class DFE_CFR {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[16];
        output STATUS[60];
        input SHUTDOWN_IN;
        output SHUTDOWN_OUT;

        input S_AXI_PARAM_ACLK;
        input S_AXI_PARAM_ARESETN;
        output S_AXI_PARAM_ARREADY;
        input S_AXI_PARAM_ARVALID;
        input S_AXI_PARAM_ARADDR[18];
        output S_AXI_PARAM_AWREADY;
        input S_AXI_PARAM_AWVALID;
        input S_AXI_PARAM_AWADDR[18];
        input S_AXI_PARAM_BREADY;
        output S_AXI_PARAM_BVALID;
        input S_AXI_PARAM_RREADY;
        output S_AXI_PARAM_RVALID;
        output S_AXI_PARAM_RDATA[32];
        output S_AXI_PARAM_WREADY;
        input S_AXI_PARAM_WVALID;
        input S_AXI_PARAM_WDATA[32];

        for i in 0..3 {
            output "S_AXIS_IT{i}_DIN_TREADY";
            input "S_AXIS_IT{i}_DIN_TVALID";
            input "S_AXIS_IT{i}_DIN_TDATA"[32];
            input "S_AXIS_IT{i}_DIN_TLAST";
            input "S_AXIS_IT{i}_DIN_TUSER"[4];
        }

        output S_AXIS_WIN_DIN_TREADY;
        input S_AXIS_WIN_DIN_TVALID;
        input S_AXIS_WIN_DIN_TDATA[32];
        input S_AXIS_WIN_DIN_TLAST;
        input S_AXIS_WIN_DIN_TUSER[4];

        input S_AXIS_COEFF_TVALID;
        input S_AXIS_COEFF_TDATA[32];
        input S_AXIS_COEFF_TLAST;

        output M_AXIS_COEFF_BANK_TVALID;
        output M_AXIS_COEFF_BANK_TDATA;

        for i in 0..3 {
            output "M_AXIS_IT{i}_DOUT_TVALID";
            output "M_AXIS_IT{i}_DOUT_TDATA"[32];
            output "M_AXIS_IT{i}_DOUT_TLAST";
            output "M_AXIS_IT{i}_DOUT_TUSER"[4];
        }

        output M_AXIS_WIN_DOUT_TVALID;
        output M_AXIS_WIN_DOUT_TDATA[32];
        output M_AXIS_WIN_DOUT_TLAST;
        output M_AXIS_WIN_DOUT_TUSER[4];

        input JTAG_CLK;
        input TEST_IN[16];
        output TEST_OUT[100];
        input MBIST_IN[8];
        output MBIST_OUT[2];
        input SCAN_DATA_IN[140];
        output SCAN_DATA_OUT[140];
    }

    bel_class DFE_PQ_INTERP {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[43];
        output STATUS[16];

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[32];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[224];
        output M_AXIS_DOUT_TKEEP[7];

        input TEST_IN[16];
        output TEST_OUT[8];
        input SCAN_DATA_IN[100];
        output SCAN_DATA_OUT[100];
    }

    bel_class DFE_NL_FIR {
        input CLK;
        input D_CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[117];
        output STATUS[48];
        input INDEX_BYPASS[20];
        input SIGNAL_BYPASS[64];
        input SHUTDOWN_IN;
        output SHUTDOWN_OUT;

        for i in 0..4 {
            input "XO_{i}_INDEX_BYPASS"[20];
            input "XO_{i}_SIGNAL_BYPASS"[64];
        }

        input S_AXI_PARAM_ACLK;
        input S_AXI_PARAM_ARESETN;
        output S_AXI_PARAM_ARREADY;
        input S_AXI_PARAM_ARVALID;
        input S_AXI_PARAM_ARADDR[18];
        output S_AXI_PARAM_AWREADY;
        input S_AXI_PARAM_AWVALID;
        input S_AXI_PARAM_AWADDR[18];
        input S_AXI_PARAM_BREADY;
        output S_AXI_PARAM_BVALID;
        input S_AXI_PARAM_RREADY;
        output S_AXI_PARAM_RVALID;
        output S_AXI_PARAM_RDATA[32];
        output S_AXI_PARAM_WREADY;
        input S_AXI_PARAM_WVALID;
        input S_AXI_PARAM_WDATA[32];

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[128];
        input S_AXIS_DIN_TUSER[48];

        output S_AXIS_MEM_CONFIG_TREADY;
        input S_AXIS_MEM_CONFIG_TVALID;
        input S_AXIS_MEM_CONFIG_TDATA[49];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[100];
        output M_AXIS_DOUT_TUSER[8];

        output M_AXIS_MEM_TVALID;
        output M_AXIS_MEM_TDATA[33];

        output M_AXIS_INDEX_DL_TVALID;
        output M_AXIS_INDEX_DL_TDATA[20];

        output M_AXIS_SIGNAL_DL_TVALID;
        output M_AXIS_SIGNAL_DL_TDATA[64];

        output M_AXIS_XO_FW_TVALID;
        for i in 0..4 {
            output "M_AXIS_XO_{i}_FW_TDATA"[136];
        }

        input JTAG_CLK;
        input TEST_IN[16];
        output TEST_OUT[12];
        input MBIST_IN[8];
        output MBIST_OUT[2];
        input SCAN_CHOP_CLK;
        input SCAN_DATA_IN[150];
        output SCAN_DATA_OUT[150];
    }

    bel_class DFE_PQ_DECIM {
        input CLK;
        input RESET_N;
        input ENABLE;
        input CONTROL[43];
        output STATUS[16];

        output S_AXIS_DIN_TREADY;
        input S_AXIS_DIN_TVALID;
        input S_AXIS_DIN_TDATA[108];

        output M_AXIS_DOUT_TVALID;
        output M_AXIS_DOUT_TDATA[180];
        output M_AXIS_DOUT_TKEEP[5];

        input TEST_IN[16];
        output TEST_OUT[8];
        input SCAN_DATA_IN[100];
        output SCAN_DATA_OUT[100];
    }

    bel_class PS {
        // AXI — to PS

        for i in 0..7 {
               output "AXDS{i}_ARREADY";
               output "AXDS{i}_RACOUNT"[4];
               input "AXDS{i}_ARVALID";
               input "AXDS{i}_ARADDR"[49];
               input "AXDS{i}_ARBURST"[2];
               input "AXDS{i}_ARCACHE"[4];
               input "AXDS{i}_ARID"[6];
               input "AXDS{i}_ARLEN"[8];
               input "AXDS{i}_ARLOCK";
               input "AXDS{i}_ARPROT"[3];
               input "AXDS{i}_ARQOS"[4];
               input "AXDS{i}_ARSIZE"[3];
               input "AXDS{i}_ARUSER";

               output "AXDS{i}_AWREADY";
               output "AXDS{i}_WACOUNT"[4];
               input "AXDS{i}_AWVALID";
               input "AXDS{i}_AWADDR"[49];
               input "AXDS{i}_AWBURST"[2];
               input "AXDS{i}_AWCACHE"[4];
               input "AXDS{i}_AWID"[6];
               input "AXDS{i}_AWLEN"[8];
               input "AXDS{i}_AWLOCK";
               input "AXDS{i}_AWPROT"[3];
               input "AXDS{i}_AWQOS"[4];
               input "AXDS{i}_AWSIZE"[3];
               input "AXDS{i}_AWUSER";

               input "AXDS{i}_BREADY";
               output "AXDS{i}_BVALID";
               output "AXDS{i}_BID"[6];
               output "AXDS{i}_BRESP"[2];

               input "AXDS{i}_RCLK";
               input "AXDS{i}_RREADY";
               output "AXDS{i}_RCOUNT"[8];
               output "AXDS{i}_RVALID";
               output "AXDS{i}_RDATA"[128];
               output "AXDS{i}_RID"[6];
               output "AXDS{i}_RLAST";
               output "AXDS{i}_RRESP"[2];

               input "AXDS{i}_WCLK";
               output "AXDS{i}_WREADY";
               output "AXDS{i}_WCOUNT"[8];
               input "AXDS{i}_WVALID";
               input "AXDS{i}_WDATA"[128];
               input "AXDS{i}_WSTRB"[16];
               input "AXDS{i}_WLAST";
        }

        // AXI — ACE to PS

        input PL_ACE_CLK;

        input ACE_PL_INTFPD_ACREADY;
        output ACE_PL_INTFPD_ACVALID;
        output ACE_PL_INTFPD_ACADDR[44];
        output ACE_PL_INTFPD_ACPROT[3];
        output ACE_PL_INTFPD_ACSNOOP[4];

        output ACE_PL_INTFPD_ARREADY;
        input ACE_PL_INTFPD_ARVALID;
        input ACE_PL_INTFPD_ARADDR[44];
        input ACE_PL_INTFPD_ARBAR[2];
        input ACE_PL_INTFPD_ARBURST[2];
        input ACE_PL_INTFPD_ARCACHE[4];
        input ACE_PL_INTFPD_ARDOMAIN[2];
        input ACE_PL_INTFPD_ARID[6];
        input ACE_PL_INTFPD_ARLEN[8];
        input ACE_PL_INTFPD_ARLOCK;
        input ACE_PL_INTFPD_ARPROT[3];
        input ACE_PL_INTFPD_ARQOS[4];
        input ACE_PL_INTFPD_ARREGION[4];
        input ACE_PL_INTFPD_ARSIZE[3];
        input ACE_PL_INTFPD_ARSNOOP[4];
        input ACE_PL_INTFPD_ARUSER[16];

        output ACE_PL_INTFPD_AWREADY;
        input ACE_PL_INTFPD_AWVALID;
        input ACE_PL_INTFPD_AWADDR[44];
        input ACE_PL_INTFPD_AWBAR[2];
        input ACE_PL_INTFPD_AWBURST[2];
        input ACE_PL_INTFPD_AWCACHE[4];
        input ACE_PL_INTFPD_AWDOMAIN[2];
        input ACE_PL_INTFPD_AWID[6];
        input ACE_PL_INTFPD_AWLEN[8];
        input ACE_PL_INTFPD_AWLOCK;
        input ACE_PL_INTFPD_AWPROT[3];
        input ACE_PL_INTFPD_AWQOS[4];
        input ACE_PL_INTFPD_AWREGION[4];
        input ACE_PL_INTFPD_AWSIZE[3];
        input ACE_PL_INTFPD_AWSNOOP[3];
        input ACE_PL_INTFPD_AWUSER[16];

        input ACE_PL_INTFPD_BREADY;
        output ACE_PL_INTFPD_BVALID;
        output ACE_PL_INTFPD_BID[6];
        output ACE_PL_INTFPD_BRESP[2];
        output ACE_PL_INTFPD_BUSER;

        output ACE_PL_INTFPD_CDREADY;
        input ACE_PL_INTFPD_CDVALID;
        input ACE_PL_INTFPD_CDDATA[128];
        input ACE_PL_INTFPD_CDLAST;

        output ACE_PL_INTFPD_CRREADY;
        input ACE_PL_INTFPD_CRVALID;
        input ACE_PL_INTFPD_CRRESP[5];

        input ACE_PL_INTFPD_RACK;
        input ACE_PL_INTFPD_RREADY;
        output ACE_PL_INTFPD_RVALID;
        output ACE_PL_INTFPD_RDATA[128];
        output ACE_PL_INTFPD_RID[6];
        output ACE_PL_INTFPD_RLAST;
        output ACE_PL_INTFPD_RRESP[4];
        output ACE_PL_INTFPD_RUSER;

        input ACE_PL_INTFPD_WACK;
        output ACE_PL_INTFPD_WREADY;
        input ACE_PL_INTFPD_WVALID;
        input ACE_PL_INTFPD_WDATA[128];
        input ACE_PL_INTFPD_WLAST;
        input ACE_PL_INTFPD_WSTRB[16];
        input ACE_PL_INTFPD_WUSER;

        // AXI — ACP to PS

        input PL_ACPCLK;
        input PL_ACPINACT;

        output AXI_PL_ACP_ARREADY;
        input AXI_PL_ACP_ARVALID;
        input AXI_PL_ACP_ARADDR[40];
        input AXI_PL_ACP_ARBURST[2];
        input AXI_PL_ACP_ARCACHE[4];
        input AXI_PL_ACP_ARID[5];
        input AXI_PL_ACP_ARLEN[8];
        input AXI_PL_ACP_ARLOCK;
        input AXI_PL_ACP_ARPROT[3];
        input AXI_PL_ACP_ARQOS[4];
        input AXI_PL_ACP_ARSIZE[3];
        input AXI_PL_ACP_ARUSER[2];

        output AXI_PL_ACP_AWREADY;
        input AXI_PL_ACP_AWVALID;
        input AXI_PL_ACP_AWADDR[40];
        input AXI_PL_ACP_AWBURST[2];
        input AXI_PL_ACP_AWCACHE[4];
        input AXI_PL_ACP_AWID[5];
        input AXI_PL_ACP_AWLEN[8];
        input AXI_PL_ACP_AWLOCK;
        input AXI_PL_ACP_AWPROT[3];
        input AXI_PL_ACP_AWQOS[4];
        input AXI_PL_ACP_AWSIZE[3];
        input AXI_PL_ACP_AWUSER[2];

        input AXI_PL_ACP_BREADY;
        output AXI_PL_ACP_BVALID;
        output AXI_PL_ACP_BID[5];
        output AXI_PL_ACP_BRESP[2];

        input AXI_PL_ACP_RREADY;
        output AXI_PL_ACP_RVALID;
        output AXI_PL_ACP_RDATA[128];
        output AXI_PL_ACP_RID[5];
        output AXI_PL_ACP_RLAST;
        output AXI_PL_ACP_RRESP[2];

        output AXI_PL_ACP_WREADY;
        input AXI_PL_ACP_WVALID;
        input AXI_PL_ACP_WDATA[128];
        input AXI_PL_ACP_WSTRB[16];
        input AXI_PL_ACP_WLAST;

        // AXI — from PS

        for i in 0..3 {
               input "AXI_PL_PORT{i}_ARREADY";
               output "AXI_PL_PORT{i}_ARVALID";
               output "AXI_PL_PORT{i}_ARADDR"[40];
               output "AXI_PL_PORT{i}_ARBURST"[2];
               output "AXI_PL_PORT{i}_ARCACHE"[4];
               output "AXI_PL_PORT{i}_ARID"[16];
               output "AXI_PL_PORT{i}_ARLEN"[8];
               output "AXI_PL_PORT{i}_ARLOCK";
               output "AXI_PL_PORT{i}_ARPROT"[3];
               output "AXI_PL_PORT{i}_ARQOS"[4];
               output "AXI_PL_PORT{i}_ARSIZE"[3];
               output "AXI_PL_PORT{i}_ARUSER"[16];

               input "AXI_PL_PORT{i}_AWREADY";
               output "AXI_PL_PORT{i}_AWVALID";
               output "AXI_PL_PORT{i}_AWADDR"[40];
               output "AXI_PL_PORT{i}_AWBURST"[2];
               output "AXI_PL_PORT{i}_AWCACHE"[4];
               output "AXI_PL_PORT{i}_AWID"[16];
               output "AXI_PL_PORT{i}_AWLEN"[8];
               output "AXI_PL_PORT{i}_AWLOCK";
               output "AXI_PL_PORT{i}_AWPROT"[3];
               output "AXI_PL_PORT{i}_AWQOS"[4];
               output "AXI_PL_PORT{i}_AWSIZE"[3];
               output "AXI_PL_PORT{i}_AWUSER"[16];

               output "AXI_PL_PORT{i}_BREADY";
               input "AXI_PL_PORT{i}_BVALID";
               input "AXI_PL_PORT{i}_BID"[16];
               input "AXI_PL_PORT{i}_BRESP"[2];

               output "AXI_PL_PORT{i}_RREADY";
               input "AXI_PL_PORT{i}_RVALID";
               input "AXI_PL_PORT{i}_RDATA"[128];
               input "AXI_PL_PORT{i}_RID"[16];
               input "AXI_PL_PORT{i}_RLAST";
               input "AXI_PL_PORT{i}_RRESP"[2];

               input "AXI_PL_PORT{i}_WREADY";
               output "AXI_PL_PORT{i}_WVALID";
               output "AXI_PL_PORT{i}_WDATA"[128];
               output "AXI_PL_PORT{i}_WLAST";
               output "AXI_PL_PORT{i}_WSTRB"[16];
        }

        // clocks

        output PL_CLK[4];

        output PS_PL_SYSOSC_CLK;

        input PL_FPGA_STOP[4];

        input PL_GP0_CLOCKIN;
        input PL_GP1_CLOCKIN;
        input PL_GP2_CLOCKIN;

        input PLL_AUX_REFCLK_FPD[3];
        input PLL_AUX_REFCLK_LPD[2];

        output APLL_TEST_CLK_OUT[2];
        output DPLL_TEST_CLK_OUT[2];
        output RPLL_TEST_CLK_OUT[2];
        output VPLL_TEST_CLK_OUT[2];
        output IOPLL_TEST_CLK_OUT[2];

        // assorted

        input PL_PS_EVENTI;
        output PS_PL_EVENTO;

        input PL_PS_GPIO[32];
        output PS_PL_GPIO[32];

        input PL_PS_IRQ0[8];
        input PL_PS_IRQ1[8];
        input PL_PS_APUGIC_FIQ[4];
        input PL_PS_APUGIC_IRQ[4];
        output PS_PL_IRQ_FPD[64];
        output PS_PL_IRQ_LPD[100];

        output PS_PL_STANDBYWFE[4];
        output PS_PL_STANDBYWFI[4];

        input PL_PS_TRACE_CLK;
        output PS_PL_TRACECTL;
        output PS_PL_TRACEDATA[32];
        input PL_PS_TRIGGER[4];
        input PL_PS_TRIGACK[4];
        output PS_PL_TRIGGER[4];
        output PS_PL_TRIGACK[4];
        input PL_PS_STM_EVENT[60];

        // RPU

        input EVENTI0_PL_RPU;
        input EVENTI1_PL_RPU;
        output EVENTO0_RPU_PL;
        output EVENTO1_RPU_PL;
        input NFIQ0_LPD_RPU;
        input NFIQ1_LPD_RPU;
        input NIRQ0_LPD_RPU;
        input NIRQ1_LPD_RPU;

        // PMU

        input AIB_PMU_AFIFM_FPD_ACK;
        input AIB_PMU_AFIFM_LPD_ACK;
        output PMU_AIB_AFIFM_FPD_REQ;
        output PMU_AIB_AFIFM_LPD_REQ;
        input PMU_ERROR_FROM_PL[4];
        output PMU_ERROR_TO_PL[47];
        input PL_PMU_GPI[32];
        output PMU_PL_GPO[32];

        // DMA

        input ADMA_FCI_CLK[8];
        output ADMA2PL_CACK[8];
        output ADMA2PL_TVLD[8];
        input PL2ADMA_CVLD[8];
        input PL2ADMA_TACK[8];

        input GDMA_FCI_CLK[8];
        output GDMA2PL_CACK[8];
        output GDMA2PL_TVLD[8];
        input PL2GDMA_CVLD[8];
        input PL2GDMA_TACK[8];

        // EMIO

        input FMIO_GPIO_IN[96];
        output FMIO_GPIO_OUT[96];
        output FMIO_GPIO_TRI_B[96];

        for i in 0..4 {
               input "FMIO_TTC{i}_CLK_IN"[3];
               output "FMIO_TTC{i}_WAVEOUT"[3];
        }

        for i in 0..2 {
            input "FMIO_WDT{i}_CLK_IN";
            output "FMIO_WDT{i}_RST_OUT";
        }

        for i in 0..2 {
               input "FMIO_UART{i}_RXD";
               output "FMIO_UART{i}_TXD";
               input "FMIO_UART{i}_NCTS";
               output "FMIO_UART{i}_NRTS";
               input "FMIO_UART{i}_NDSR";
               output "FMIO_UART{i}_NDTR";
               input "FMIO_UART{i}_NDCD";
               input "FMIO_UART{i}_NRI";
        }

        for i in 0..2 {
               input "FMIO_SPI{i}_SCLK_IN";
               output "FMIO_SPI{i}_SCLK_OUT";
               output "FMIO_SPI{i}_SCLK_TRI_B";
               input "FMIO_SPI{i}_MI";
               output "FMIO_SPI{i}_MO";
               output "FMIO_SPI{i}_MO_TRI_B";
               input "FMIO_SPI{i}_SI";
               output "FMIO_SPI{i}_SO";
               output "FMIO_SPI{i}_SO_TRI_B";
               input "FMIO_SPI{i}_SS_IN_B";
               output "FMIO_SPI{i}_SS_OUT_B"[3];
               output "FMIO_SPI{i}_SS_TRI_B";
        }

        for i in 0..2 {
               input "FMIO_I2C{i}_SCL_INPUT";
               output "FMIO_I2C{i}_SCL_OUT";
               output "FMIO_I2C{i}_SCL_TRI_B";
               input "FMIO_I2C{i}_SDA_INPUT";
               output "FMIO_I2C{i}_SDA_OUT";
               output "FMIO_I2C{i}_SDA_TRI_B";
        }

        for i in 0..2 {
               input "FMIO_CAN{i}_PHY_RX";
               output "FMIO_CAN{i}_PHY_TX";
        }

        for i in 0..2 {
               input "FMIO_SDIO{i}_RXCLK_IN";
               output "FMIO_SD{i}_BUSPOWER";
               output "FMIO_SD{i}_BUSVOLTAGE"[3];
               output "FMIO_SD{i}_LEDCONTROL";
               input "FMIO_SD{i}_SDIF_CD_N";
               output "FMIO_SD{i}_SDIF_CLKOUT";
               input "FMIO_SD{i}_SDIF_CMDIN";
               output "FMIO_SD{i}_SDIF_CMDOUT";
               output "FMIO_SD{i}_SDIF_CMDENA";
               input "FMIO_SD{i}_SDIF_DATIN"[8];
               output "FMIO_SD{i}_SDIF_DATOUT"[8];
               output "FMIO_SD{i}_SDIF_DATENA"[8];
               input "FMIO_SD{i}_SDIF_WP";
               input "FMIO_SD{i}_DLL_TEST_IN_N"[4];
               output "FMIO_SD{i}_DLL_TEST_OUT"[8];
        }

        input EMIO_HUB_PORT_OVERCRNT_USB2[2];
        input EMIO_HUB_PORT_OVERCRNT_USB3[2];
        output EMIO_U2DSPORT_VBUS_CTRL_USB3[2];
        output EMIO_U3DSPORT_VBUS_CTRL_USB3[2];

        input FMIO_GEM_TSU_CLK;
        input FMIO_GEM_TSU_CLK_FROM_PL;
        output FMIO_GEM_TSU_CLK_TO_PL_BUFG;
        output FMIO_GEM0_TSU_TIMER_CNT[94];
        for i in 0..4 {
               output "FMIO_GEM{i}_DELAY_REQ_RX";
               output "FMIO_GEM{i}_DELAY_REQ_TX";
               output "FMIO_GEM{i}_DMA_BUS_WIDTH"[2];
               output "FMIO_GEM{i}_DMA_TX_END_TOG";
               input "FMIO_GEM{i}_DMA_TX_STATUS_TOG";
               input "FMIO_GEM{i}_EXT_INT_IN";
               input "FMIO_GEM{i}_FIFO_RX_CLK_FROM_PL";
               output "FMIO_GEM{i}_FIFO_RX_CLK_TO_PL_BUFG";
               input "FMIO_GEM{i}_FIFO_TX_CLK_FROM_PL";
               output "FMIO_GEM{i}_FIFO_TX_CLK_TO_PL_BUFG";
               input "FMIO_GEM{i}_GMII_RX_CLK";
               input "FMIO_GEM{i}_GMII_RXD"[8];
               input "FMIO_GEM{i}_GMII_RX_DV";
               input "FMIO_GEM{i}_GMII_RX_ER";
               input "FMIO_GEM{i}_GMII_COL";
               input "FMIO_GEM{i}_GMII_CRS";
               input "FMIO_GEM{i}_GMII_TX_CLK";
               output "FMIO_GEM{i}_GMII_TXD"[8];
               output "FMIO_GEM{i}_GMII_TX_EN";
               output "FMIO_GEM{i}_GMII_TX_ER";
               input "FMIO_GEM{i}_MDIO_IN";
               output "FMIO_GEM{i}_MDIO_MDC";
               output "FMIO_GEM{i}_MDIO_OUT";
               output "FMIO_GEM{i}_MDIO_TRI_B";
               output "FMIO_GEM{i}_PDELAY_REQ_RX";
               output "FMIO_GEM{i}_PDELAY_REQ_TX";
               output "FMIO_GEM{i}_PDELAY_RESP_RX";
               output "FMIO_GEM{i}_PDELAY_RESP_TX";
               output "FMIO_GEM{i}_RX_SOF";
               output "FMIO_GEM{i}_RX_W_DATA"[8];
               output "FMIO_GEM{i}_RX_W_EOP";
               output "FMIO_GEM{i}_RX_W_ERR";
               output "FMIO_GEM{i}_RX_W_FLUSH";
               input "FMIO_GEM{i}_RX_W_OVERFLOW";
               output "FMIO_GEM{i}_RX_W_SOP";
               output "FMIO_GEM{i}_RX_W_STATUS"[45];
               output "FMIO_GEM{i}_RX_W_WR";
               input "FMIO_GEM{i}_SIGNAL_DETECT";
               output "FMIO_GEM{i}_SPEED_MODE"[3];
               output "FMIO_GEM{i}_SYNC_FRAME_RX";
               output "FMIO_GEM{i}_SYNC_FRAME_TX";
               input "FMIO_GEM{i}_TSU_INC_CTRL"[2];
               output "FMIO_GEM{i}_TSU_TIMER_CMP_VAL";
               input "FMIO_GEM{i}_TX_R_CONTROL";
               input "FMIO_GEM{i}_TX_R_DATA"[8];
               input "FMIO_GEM{i}_TX_R_DATA_RDY";
               input "FMIO_GEM{i}_TX_R_EOP";
               input "FMIO_GEM{i}_TX_R_ERR";
               output "FMIO_GEM{i}_TX_R_FIXED_LAT";
               input "FMIO_GEM{i}_TX_R_FLUSHED";
               output "FMIO_GEM{i}_TX_R_RD";
               input "FMIO_GEM{i}_TX_R_SOP";
               output "FMIO_GEM{i}_TX_R_STATUS"[4];
               input "FMIO_GEM{i}_TX_R_UNDERFLOW";
               input "FMIO_GEM{i}_TX_R_VALID";
               output "FMIO_GEM{i}_TX_SOF";
        }

        input IO_CHAR_AUDIO_MUX_SEL_N;
        input IO_CHAR_AUDIO_IN_TEST_DATA;
        output IO_CHAR_AUDIO_OUT_TEST_DATA;
        input IO_CHAR_VIDEO_MUX_SEL_N;
        input IO_CHAR_VIDEO_IN_TEST_DATA;
        output IO_CHAR_VIDEO_OUT_TEST_DATA;

        input FMIO_CHAR_AFIFSFPD_TEST_INPUT;
        output FMIO_CHAR_AFIFSFPD_TEST_OUTPUT;
        input FMIO_CHAR_AFIFSFPD_TEST_SELECT_N;
        input FMIO_CHAR_AFIFSLPD_TEST_INPUT;
        output FMIO_CHAR_AFIFSLPD_TEST_OUTPUT;
        input FMIO_CHAR_AFIFSLPD_TEST_SELECT_N;
        input FMIO_CHAR_GEM_SELECTION[2];
        input FMIO_CHAR_GEM_TEST_INPUT;
        output FMIO_CHAR_GEM_TEST_OUTPUT;
        input FMIO_CHAR_GEM_TEST_SELECT_N;

        input FMIO_TEST_GEM_SCANMUX_1;
        input FMIO_TEST_GEM_SCANMUX_2;

        input FMIO_TEST_IO_CHAR_SCAN_CLOCK;
        input FMIO_TEST_IO_CHAR_SCAN_RESET_N;
        input FMIO_TEST_IO_CHAR_SCANENABLE;
        input FMIO_TEST_IO_CHAR_SCAN_IN;
        output FMIO_TEST_IO_CHAR_SCAN_OUT;

        input FMIO_TEST_QSPI_SCANMUX_1_N;
        input FMIO_TEST_SDIO_SCANMUX_1;
        input FMIO_TEST_SDIO_SCANMUX_2;

        // DisplayPort

        output DP_AUDIO_REF_CLK;
        output DP_VIDEO_REF_CLK;

        input FMIO_DP_AUX_DATA_IN;
        input FMIO_DP_HOT_PLUG_DETECT;
        output DP_AUX_DATA_ENABLE_N_PL;
        output DP_AUX_TX_OUT_CHANNEL_PL;

        input DP_EXTERNAL_CUSTOM_EVENT1;
        input DP_EXTERNAL_CUSTOM_EVENT2;
        input DP_EXTERNAL_VSYNC_EVENT;

        input DP_LIVE_VIDEO_IN_CLK;
        input DP_LIVE_VIDEO_DE_IN;
        input DP_LIVE_VIDEO_HSYNC_IN;
        input DP_LIVE_VIDEO_VSYNC_IN;
        input DP_LIVE_VIDEO_PIXEL1_IN[36];
        input DP_LIVE_GFX_ALPHA_IN[8];
        input DP_LIVE_GFX_PIXEL1_IN[36];
        output DP_LIVE_VIDEO_DE_OUT;
        output DP_LIVE_VIDEO_HSYNC_OUT;
        output DP_LIVE_VIDEO_PIXEL1_OUT[36];
        output DP_LIVE_VIDEO_VSYNC_OUT;

        input DP_M_AXIS_MIXED_AUDIO_TREADY_OUT;
        output DP_M_AXIS_MIXED_AUDIO_TVALID_OUT;
        output DP_M_AXIS_MIXED_AUDIO_TDATA_OUT[32];
        output DP_M_AXIS_MIXED_AUDIO_TID_OUT;

        input DP_S_AXIS_LIVE_AUDIO_ACLK;
        output DP_S_AXIS_LIVE_AUDIO_TREADY_IN;
        input DP_S_AXIS_LIVE_AUDIO_TVALID_IN;
        input DP_S_AXIS_LIVE_AUDIO_TDATA_IN[32];
        input DP_S_AXIS_LIVE_AUDIO_TID_IN;

        // unsorted

        output DBG_PATH_FIFO_BYPASS;

        input DDRC_EXT_REFRESH_RANK0_REQ;
        input DDRC_EXT_REFRESH_RANK1_REQ;
        input DDRC_REFRESH_PL_CLK;
        output DDR_DTO[2];

        input PL_FPD_PLL_TEST_CK_SEL_N[3];
        input PL_FPD_PLL_TEST_FRACT_CLK_SEL_N;
        input PL_FPD_PLL_TEST_FRACT_EN_N;
        input PL_FPD_PLL_TEST_MUX_SEL[2];
        input PL_FPD_PLL_TEST_SEL[4];
        output FPD_PL_PLL_TEST_OUT[32];

        input PL_FPD_SPARE_0_IN;
        input PL_FPD_SPARE_1_IN;
        input PL_FPD_SPARE_2_IN;
        input PL_FPD_SPARE_3_IN;
        input PL_FPD_SPARE_4_IN;
        output FPD_PL_SPARE_0_OUT;
        output FPD_PL_SPARE_1_OUT;
        output FPD_PL_SPARE_2_OUT;
        output FPD_PL_SPARE_3_OUT;
        output FPD_PL_SPARE_4_OUT;

        input PL_LPD_PLL_TEST_CK_SEL_N[3];
        input PL_LPD_PLL_TEST_FRACT_CLK_SEL_N;
        input PL_LPD_PLL_TEST_FRACT_EN_N;
        input PL_LPD_PLL_TEST_MUX_SEL;
        input PL_LPD_PLL_TEST_SEL[4];
        output LPD_PL_PLL_TEST_OUT[32];

        input PL_LPD_SPARE_0_IN;
        input PL_LPD_SPARE_1_IN;
        input PL_LPD_SPARE_2_IN;
        input PL_LPD_SPARE_3_IN;
        input PL_LPD_SPARE_4_IN;
        output LPD_PL_SPARE_0_OUT;
        output LPD_PL_SPARE_1_OUT;
        output LPD_PL_SPARE_2_OUT;
        output LPD_PL_SPARE_3_OUT;
        output LPD_PL_SPARE_4_OUT;

        input I_AFE_CMN_BG_ENABLE_LOW_LEAKAGE;
        input I_AFE_CMN_BG_ISO_CTRL_BAR;
        input I_AFE_CMN_BG_PD;
        input I_AFE_CMN_BG_PD_BG_OK;
        input I_AFE_CMN_BG_PD_PTAT;
        input I_AFE_CMN_CALIB_ENABLE_LOW_LEAKAGE;
        input I_AFE_CMN_CALIB_EN_ICONST;
        input I_AFE_CMN_CALIB_ISO_CTRL_BAR;
        output O_AFE_CMN_CALIB_COMP_OUT;

        input I_AFE_MODE;

        output O_AFE_PG_AVDDCR;
        output O_AFE_PG_AVDDIO;
        output O_AFE_PG_DVDDCR;
        output O_AFE_PG_STATIC_AVDDCR;
        output O_AFE_PG_STATIC_AVDDIO;

        input I_PLL_AFE_MODE;
        input I_AFE_PLL_COARSE_CODE[11];
        input I_AFE_PLL_EN_CLOCK_HS_DIV2;
        input I_AFE_PLL_FBDIV[16];
        input I_AFE_PLL_LOAD_FBDIV;
        input I_AFE_PLL_PD;
        input I_AFE_PLL_PD_HS_CLOCK_R;
        input I_AFE_PLL_PD_PFD;
        input I_AFE_PLL_RST_FDBK_DIV;
        input I_AFE_PLL_STARTLOOP;
        input I_AFE_PLL_V2I_CODE[6];
        input I_AFE_PLL_V2I_PROG[5];
        input I_AFE_PLL_VCO_CNT_WINDOW;
        output O_AFE_PLL_CLK_SYM_HS;
        output O_AFE_PLL_DCO_COUNT[13];
        output O_AFE_PLL_FBCLK_FRAC;

        input I_AFE_RX_HSRX_CLOCK_STOP_REQ;
        output O_AFE_RX_HSRX_CLOCK_STOP_ACK;
        input I_AFE_RX_ISO_HSRX_CTRL_BAR;
        input I_AFE_RX_ISO_LFPS_CTRL_BAR;
        input I_AFE_RX_ISO_SIGDET_CTRL_BAR;
        input I_AFE_RX_MPHY_GATE_SYMBOL_CLK;
        input I_AFE_RX_MPHY_MUX_HSB_LS;
        input I_AFE_RX_PIPE_RXEQTRAINING;
        input I_AFE_RX_PIPE_RX_TERM_ENABLE;
        output O_AFE_RX_PIPE_LFPSBCN_RXELECIDLE;
        output O_AFE_RX_PIPE_SIGDET;
        input I_AFE_RX_RXPMA_REFCLK_DIG;
        input I_AFE_RX_RXPMA_RSTB;
        input I_AFE_RX_SYMBOL_CLK_BY_2_PL;
        input I_AFE_RX_UPHY_BIASGEN_ICONST_CORE_MIRROR_ENABLE;
        input I_AFE_RX_UPHY_BIASGEN_ICONST_IO_MIRROR_ENABLE;
        input I_AFE_RX_UPHY_BIASGEN_IRCONST_CORE_MIRROR_ENABLE;
        input I_AFE_RX_UPHY_ENABLE_CDR;
        input I_AFE_RX_UPHY_ENABLE_LOW_LEAKAGE;
        input I_AFE_RX_UPHY_HSCLK_DIVISION_FACTOR[2];
        input I_AFE_RX_UPHY_HSRX_RSTB;
        input I_AFE_RX_UPHY_PDN_HS_DES;
        input I_AFE_RX_UPHY_PD_SAMP_C2C;
        input I_AFE_RX_UPHY_PD_SAMP_C2C_ECLK;
        input I_AFE_RX_UPHY_PSO_CLK_LANE;
        input I_AFE_RX_UPHY_PSO_EQ;
        input I_AFE_RX_UPHY_PSO_HSRXDIG;
        input I_AFE_RX_UPHY_PSO_IQPI;
        input I_AFE_RX_UPHY_PSO_LFPSBCN;
        input I_AFE_RX_UPHY_PSO_SAMP_FLOPS;
        input I_AFE_RX_UPHY_PSO_SIGDET;
        input I_AFE_RX_UPHY_RESTORE_CALCODE;
        input I_AFE_RX_UPHY_RESTORE_CALCODE_DATA[8];
        input I_AFE_RX_UPHY_RUN_CALIB;
        input I_AFE_RX_UPHY_RX_LANE_POLARITY_SWAP;
        input I_AFE_RX_UPHY_RX_PMA_OPMODE[8];
        input I_AFE_RX_UPHY_STARTLOOP_PLL;
        output O_AFE_RX_UPHY_RX_CALIB_DONE;
        output O_AFE_RX_UPHY_SAVE_CALCODE;
        output O_AFE_RX_UPHY_SAVE_CALCODE_DATA[8];
        output O_AFE_RX_UPHY_STARTLOOP_BUF;
        output O_AFE_RX_SYMBOL[20];
        output O_AFE_RX_SYMBOL_CLK_BY_2;

        input I_AFE_TX_ANA_IF_RATE[2];
        input I_AFE_TX_ENABLE_HSCLK_DIVISION[2];
        input I_AFE_TX_ENABLE_LDO;
        input I_AFE_TX_ENABLE_REF;
        input I_AFE_TX_ENABLE_SUPPLY_HSCLK;
        input I_AFE_TX_ENABLE_SUPPLY_PIPE;
        input I_AFE_TX_ENABLE_SUPPLY_SERIALIZER;
        input I_AFE_TX_ENABLE_SUPPLY_UPHY;
        input I_AFE_TX_EN_DIG_SUBLP_MODE;
        input I_AFE_TX_HS_SER_RSTB;
        input I_AFE_TX_HS_SYMBOL[20];
        input I_AFE_TX_ISO_CTRL_BAR;
        input I_AFE_TX_LFPS_CLK;
        input I_AFE_TX_LPBK_SEL[3];
        input I_AFE_TX_MPHY_TX_LS_DATA;
        input I_AFE_TX_PIPE_TX_ENABLE_IDLE_MODE[2];
        input I_AFE_TX_PIPE_TX_ENABLE_LFPS[2];
        input I_AFE_TX_PIPE_TX_ENABLE_RXDET;
        input I_AFE_TX_PIPE_TX_FAST_EST_COMMON_MODE;
        input I_AFE_TX_PLL_SYMB_CLK_2;
        input I_AFE_TX_PMADIG_DIGITAL_RESET_N;
        input I_AFE_TX_SERIALIZER_RSTB;
        input I_AFE_TX_SERIALIZER_RST_REL;
        input I_AFE_TX_SER_ISO_CTRL_BAR;
        input I_AFE_TX_UPHY_TXPMA_OPMODE[8];
        output O_AFE_TX_DIG_RESET_REL_ACK;
        output O_AFE_TX_PIPE_TX_DN_RXDET;
        output O_AFE_TX_PIPE_TX_DP_RXDET;

        input I_BGCAL_AFE_MODE;

        for i in 0..4 {
               output "O_DBG_L{i}_RSTB";
               output "O_DBG_L{i}_PHYSTATUS";
               output "O_DBG_L{i}_POWERDOWN"[2];
               output "O_DBG_L{i}_RATE"[2];

               input "I_DBG_L{i}_RXCLK";
               output "O_DBG_L{i}_RXCLK";
               output "O_DBG_L{i}_RXDATA"[20];
               output "O_DBG_L{i}_RXDATAK"[2];
               output "O_DBG_L{i}_RXELECIDLE";
               output "O_DBG_L{i}_RXPOLARITY";
               output "O_DBG_L{i}_RXSTATUS"[3];
               output "O_DBG_L{i}_RXVALID";
               output "O_DBG_L{i}_RX_SGMII_EN_CDET";

               input "I_DBG_L{i}_TXCLK";
               output "O_DBG_L{i}_TXCLK";
               output "O_DBG_L{i}_TXDATA"[20];
               output "O_DBG_L{i}_TXDATAK"[2];
               output "O_DBG_L{i}_TXDETRX_LPBACK";
               output "O_DBG_L{i}_TXELECIDLE";
               output "O_DBG_L{i}_TX_SGMII_EWRAP";

               output "O_DBG_L{i}_SATA_CORECLOCKREADY";
               output "O_DBG_L{i}_SATA_COREREADY";
               output "O_DBG_L{i}_SATA_CORERXDATA"[20];
               output "O_DBG_L{i}_SATA_CORERXDATAVALID"[2];
               output "O_DBG_L{i}_SATA_CORERXSIGNALDET";

               output "O_DBG_L{i}_SATA_PHYCTRLRESET";
               output "O_DBG_L{i}_SATA_PHYCTRLPARTIAL";
               output "O_DBG_L{i}_SATA_PHYCTRLSLUMBER";
               output "O_DBG_L{i}_SATA_PHYCTRLRXRST";
               output "O_DBG_L{i}_SATA_PHYCTRLRXRATE"[2];
               output "O_DBG_L{i}_SATA_PHYCTRLTXRST";
               output "O_DBG_L{i}_SATA_PHYCTRLTXDATA"[20];
               output "O_DBG_L{i}_SATA_PHYCTRLTXIDLE";
               output "O_DBG_L{i}_SATA_PHYCTRLTXRATE"[2];
        }

        output OSC_RTC_CLK;

        input PL_SYSMON_TEST_CONVST;
        input PL_SYSMON_TEST_ADC_CLK[4];
        input PL_SYSMON_TEST_ADC_IN[32];
        input PL_SYSMON_TEST_ADC_IN2[32];
        output PL_SYSMON_TEST_ADC_OUT[20];
        output PL_SYSMON_TEST_AMS_OSC[8];
        output PL_SYSMON_TEST_DB[16];
        output PL_SYSMON_TEST_MON_DATA[16];

        input PL_SYSMON_TEST_DCLK;
        input PL_SYSMON_TEST_DEN;
        input PL_SYSMON_TEST_DWE;
        input PL_SYSMON_TEST_DADDR[8];
        input PL_SYSMON_TEST_DI[16];
        output PL_SYSMON_TEST_DRDY;
        output PL_SYSMON_TEST_DO[16];

        input PSTP_PL_CLK[4];
        input PSTP_PL_IN[32];
        output PSTP_PL_OUT[32];
        input PSTP_PL_TS[32];

        input TEST_BSCAN_AC_MODE;
        input TEST_BSCAN_AC_TEST;
        input TEST_BSCAN_EN_N;
        input TEST_BSCAN_EXTEST;
        input TEST_BSCAN_INTEST;
        input TEST_BSCAN_INIT_MEMORY;
        input TEST_BSCAN_MISR_JTAG_LOAD;
        input TEST_BSCAN_MODE_C;
        input TEST_BSCAN_RESET_TAP_B;
        input TEST_BSCAN_SHIFTDR;
        input TEST_BSCAN_CLOCKDR;
        input TEST_BSCAN_UPDATEDR;
        input TEST_BSCAN_TDI;
        output TEST_BSCAN_TDO;

        input TEST_CHAR_MODE_FPD_N;
        input TEST_CHAR_MODE_LPD_N;
        output TEST_DDR2PL_DCD_SKEWOUT;
        input TEST_PL2DDR_DCD_SAMPLE_PULSE;
        output TEST_PL_PLL_LOCK_OUT[5];

        input TEST_PL_SCANENABLE;
        input TEST_PL_SCANENABLE_SLCR_EN;
        input TEST_PL_SCAN_CLK[2];
        input TEST_PL_SCAN_RESET_N;
        input TEST_PL_SCAN_CHOPPER_SI;
        output TEST_PL_SCAN_CHOPPER_SO;
        input TEST_PL_SCAN_CHOPPER_TRIG;
        input TEST_PL_SCAN_EDT_CLK;
        input TEST_PL_SCAN_EDT_IN_APU;
        input TEST_PL_SCAN_EDT_IN_CPU;
        input TEST_PL_SCAN_EDT_IN_DDR[4];
        input TEST_PL_SCAN_EDT_IN_FP[10];
        input TEST_PL_SCAN_EDT_IN_GPU[4];
        input TEST_PL_SCAN_EDT_IN_LP[9];
        input TEST_PL_SCAN_EDT_IN_USB3[2];
        output TEST_PL_SCAN_EDT_OUT_APU;
        output TEST_PL_SCAN_EDT_OUT_CPU[4];
        output TEST_PL_SCAN_EDT_OUT_DDR[4];
        output TEST_PL_SCAN_EDT_OUT_FP[10];
        output TEST_PL_SCAN_EDT_OUT_GPU[4];
        output TEST_PL_SCAN_EDT_OUT_LP[9];
        output TEST_PL_SCAN_EDT_OUT_USB3[2];
        input TEST_PL_SCAN_EDT_UPDATE;
        input TEST_PL_SCAN_PLL_RESET;
        input TEST_PL_SCAN_SLCR_CONFIG_CLK;
        input TEST_PL_SCAN_SLCR_CONFIG_RSTN;
        input TEST_PL_SCAN_SLCR_CONFIG_SI;
        output TEST_PL_SCAN_SLCR_CONFIG_SO;
        input TEST_PL_SCAN_SPARE_IN[3];
        output TEST_PL_SCAN_SPARE_OUT[2];
        input TEST_PL_SCAN_WRAP_CLK;
        input TEST_PL_SCAN_WRAP_ISHIFT;
        input TEST_PL_SCAN_WRAP_OSHIFT;

        input TEST_USB0_FUNCMUX_0_N;
        input TEST_USB0_SCANMUX_0_N;
        input TEST_USB1_FUNCMUX_0_N;
        input TEST_USB1_SCANMUX_0_N;

        input TST_RTC_CLK;
        input TST_RTC_CALIBREG_IN[21];
        output TST_RTC_CALIBREG_OUT[21];
        input TST_RTC_CALIBREG_WE;
        input TST_RTC_DISABLE_BAT_OP;
        output TST_RTC_OSC_CLK_OUT;
        input TST_RTC_OSC_CNTRL_IN[4];
        output TST_RTC_OSC_CNTRL_OUT[4];
        input TST_RTC_OSC_CNTRL_WE;
        output TST_RTC_SECONDS_RAW_INT;
        output TST_RTC_SEC_COUNTER_OUT[32];
        input TST_RTC_SEC_RELOAD;
        input TST_RTC_TESTCLOCK_SELECT_N;
        input TST_RTC_TESTMODE_N;
        output TST_RTC_TICK_COUNTER_OUT[16];
        input TST_RTC_TIMESETREG_IN[32];
        output TST_RTC_TIMESETREG_OUT[32];
        input TST_RTC_TIMESETREG_WE;
    }

    bel_class VCU {
        // clocks

        input PL_VCU_CORE_CLK;
        input PL_VCU_MCU_CLK;

        input PL_VCU_PLL_REF_CLK_PL;

        output VCU_PL_CORE_STATUS_CLK_PLL;
        output VCU_PL_MCU_STATUS_CLK_PLL;
        output VCU_PL_PLL_STATUS_PLL_LOCK;
        output VCU_RSTEST_PLL_LOCK;

        input VCU_PLL_TEST_CK_SEL[3];
        output VCU_PLL_TEST_CLK_OUT[2];
        input VCU_PLL_TEST_FRACT_CLK_SEL;
        input VCU_PLL_TEST_FRACT_EN;
        output VCU_PLL_TEST_OUT[32];
        input VCU_PLL_TEST_SEL[4];

        // AXI-lite

        input PL_VCU_AXI_LITE_CLK;

        output VCU_PL_ARREADY_AXI_LITE_APB;
        input PL_VCU_ARVALID_AXI_LITE_APB;
        input PL_VCU_ARADDR_AXI_LITE_APB[20];
        input PL_VCU_ARPROT_AXI_LITE_APB[3];

        output VCU_PL_AWREADY_AXI_LITE_APB;
        input PL_VCU_AWVALID_AXI_LITE_APB;
        input PL_VCU_AWADDR_AXI_LITE_APB[20];
        input PL_VCU_AWPROT_AXI_LITE_APB[3];

        input PL_VCU_BREADY_AXI_LITE_APB;
        output VCU_PL_BVALID_AXI_LITE_APB;
        output VCU_PL_BRESP_AXI_LITE_APB[2];

        input PL_VCU_RREADY_AXI_LITE_APB;
        output VCU_PL_RVALID_AXI_LITE_APB;
        output VCU_PL_RRESP_AXI_LITE_APB[2];
        output VCU_PL_RDATA_AXI_LITE_APB[32];

        output VCU_PL_WREADY_AXI_LITE_APB;
        input PL_VCU_WVALID_AXI_LITE_APB;
        input PL_VCU_WDATA_AXI_LITE_APB[32];
        input PL_VCU_WSTRB_AXI_LITE_APB[4];

        // decoder — AXI

        input PL_VCU_AXI_DEC_CLK;

        input PL_VCU_DEC_ARREADY[2];
        output VCU_PL_DEC_ARVALID[2];
        output VCU_PL_DEC_ARADDR0[44];
        output VCU_PL_DEC_ARADDR1[44];
        output VCU_PL_DEC_ARBURST0[2];
        output VCU_PL_DEC_ARBURST1[2];
        output VCU_PL_DEC_ARCACHE0[4];
        output VCU_PL_DEC_ARCACHE1[4];
        output VCU_PL_DEC_ARID0[4];
        output VCU_PL_DEC_ARID1[4];
        output VCU_PL_DEC_ARLEN0[8];
        output VCU_PL_DEC_ARLEN1[8];
        output VCU_PL_DEC_ARPROT[2];
        output VCU_PL_DEC_ARQOS0[4];
        output VCU_PL_DEC_ARQOS1[4];
        output VCU_PL_DEC_ARSIZE0[3];
        output VCU_PL_DEC_ARSIZE1[3];

        input PL_VCU_DEC_AWREADY[2];
        output VCU_PL_DEC_AWADDR0[44];
        output VCU_PL_DEC_AWADDR1[44];
        output VCU_PL_DEC_AWBURST0[2];
        output VCU_PL_DEC_AWBURST1[2];
        output VCU_PL_DEC_AWCACHE0[4];
        output VCU_PL_DEC_AWCACHE1[4];
        output VCU_PL_DEC_AWID0[4];
        output VCU_PL_DEC_AWID1[4];
        output VCU_PL_DEC_AWLEN0[8];
        output VCU_PL_DEC_AWLEN1[8];
        output VCU_PL_DEC_AWPROT[2];
        output VCU_PL_DEC_AWQOS0[4];
        output VCU_PL_DEC_AWQOS1[4];
        output VCU_PL_DEC_AWSIZE0[3];
        output VCU_PL_DEC_AWSIZE1[3];
        output VCU_PL_DEC_AWVALID[2];

        output VCU_PL_DEC_BREADY[2];
        input PL_VCU_DEC_BVALID[2];
        input PL_VCU_DEC_BID0[4];
        input PL_VCU_DEC_BID1[4];
        input PL_VCU_DEC_BRESP0[2];
        input PL_VCU_DEC_BRESP1[2];

        output VCU_PL_DEC_RREADY[2];
        input PL_VCU_DEC_RVALID[2];
        input PL_VCU_DEC_RDATA0[128];
        input PL_VCU_DEC_RDATA1[128];
        input PL_VCU_DEC_RID0[4];
        input PL_VCU_DEC_RID1[4];
        input PL_VCU_DEC_RLAST[2];
        input PL_VCU_DEC_RRESP0[2];
        input PL_VCU_DEC_RRESP1[2];

        input PL_VCU_DEC_WREADY[2];
        output VCU_PL_DEC_WVALID[2];
        output VCU_PL_DEC_WDATA0[128];
        output VCU_PL_DEC_WDATA1[128];
        output VCU_PL_DEC_WLAST[2];

        // encoder — AXI

        input PL_VCU_AXI_ENC_CLK;

        input PL_VCU_ENC_ARREADY[2];
        output VCU_PL_ENC_ARADDR0[44];
        output VCU_PL_ENC_ARADDR1[44];
        output VCU_PL_ENC_ARBURST0[2];
        output VCU_PL_ENC_ARBURST1[2];
        output VCU_PL_ENC_ARCACHE0[4];
        output VCU_PL_ENC_ARCACHE1[4];
        output VCU_PL_ENC_ARID0[4];
        output VCU_PL_ENC_ARID1[4];
        output VCU_PL_ENC_ARLEN0[8];
        output VCU_PL_ENC_ARLEN1[8];
        output VCU_PL_ENC_ARPROT[2];
        output VCU_PL_ENC_ARQOS0[4];
        output VCU_PL_ENC_ARQOS1[4];
        output VCU_PL_ENC_ARSIZE0[3];
        output VCU_PL_ENC_ARSIZE1[3];
        output VCU_PL_ENC_ARVALID[2];

        input PL_VCU_ENC_AWREADY[2];
        output VCU_PL_ENC_AWADDR0[44];
        output VCU_PL_ENC_AWADDR1[44];
        output VCU_PL_ENC_AWBURST0[2];
        output VCU_PL_ENC_AWBURST1[2];
        output VCU_PL_ENC_AWCACHE0[4];
        output VCU_PL_ENC_AWCACHE1[4];
        output VCU_PL_ENC_AWID0[4];
        output VCU_PL_ENC_AWID1[4];
        output VCU_PL_ENC_AWLEN0[8];
        output VCU_PL_ENC_AWLEN1[8];
        output VCU_PL_ENC_AWPROT[2];
        output VCU_PL_ENC_AWQOS0[4];
        output VCU_PL_ENC_AWQOS1[4];
        output VCU_PL_ENC_AWSIZE0[3];
        output VCU_PL_ENC_AWSIZE1[3];
        output VCU_PL_ENC_AWVALID[2];

        output VCU_PL_ENC_BREADY[2];
        input PL_VCU_ENC_BVALID[2];
        input PL_VCU_ENC_BID0[4];
        input PL_VCU_ENC_BID1[4];
        input PL_VCU_ENC_BRESP0[2];
        input PL_VCU_ENC_BRESP1[2];

        output VCU_PL_ENC_RREADY[2];
        input PL_VCU_ENC_RVALID[2];
        input PL_VCU_ENC_RDATA0[128];
        input PL_VCU_ENC_RDATA1[128];
        input PL_VCU_ENC_RID0[4];
        input PL_VCU_ENC_RID1[4];
        input PL_VCU_ENC_RLAST[2];
        input PL_VCU_ENC_RRESP0[2];
        input PL_VCU_ENC_RRESP1[2];

        input PL_VCU_ENC_WREADY[2];
        output VCU_PL_ENC_WVALID[2];
        output VCU_PL_ENC_WDATA0[128];
        output VCU_PL_ENC_WDATA1[128];
        output VCU_PL_ENC_WLAST[2];

        // encoder — cache

        input PL_VCU_ENC_L2C_CLK;
        output VCU_PL_ENC_AL_L2C_RVALID;
        output VCU_PL_ENC_AL_L2C_WVALID;
        output VCU_PL_ENC_AL_L2C_ADDR[17];
        output VCU_PL_ENC_AL_L2C_WDATA[320];
        input PL_VCU_ENC_AL_L2C_RREADY;
        input PL_VCU_ENC_AL_L2C_RDATA[320];

        // MCU — AXI

        input PL_VCU_AXI_MCU_CLK;

        input PL_VCU_MCU_M_AXI_IC_DC_ARREADY;
        output VCU_PL_MCU_M_AXI_IC_DC_ARVALID;
        output VCU_PL_MCU_M_AXI_IC_DC_ARADDR[44];
        output VCU_PL_MCU_M_AXI_IC_DC_ARBURST[2];
        output VCU_PL_MCU_M_AXI_IC_DC_ARCACHE[4];
        output VCU_PL_MCU_M_AXI_IC_DC_ARID[3];
        output VCU_PL_MCU_M_AXI_IC_DC_ARLEN[8];
        output VCU_PL_MCU_M_AXI_IC_DC_ARLOCK;
        output VCU_PL_MCU_M_AXI_IC_DC_ARPROT[3];
        output VCU_PL_MCU_M_AXI_IC_DC_ARQOS[4];
        output VCU_PL_MCU_M_AXI_IC_DC_ARSIZE[3];

        input PL_VCU_MCU_M_AXI_IC_DC_AWREADY;
        output VCU_PL_MCU_M_AXI_IC_DC_AWVALID;
        output VCU_PL_MCU_M_AXI_IC_DC_AWADDR[44];
        output VCU_PL_MCU_M_AXI_IC_DC_AWBURST[2];
        output VCU_PL_MCU_M_AXI_IC_DC_AWCACHE[4];
        output VCU_PL_MCU_M_AXI_IC_DC_AWID[3];
        output VCU_PL_MCU_M_AXI_IC_DC_AWLEN[8];
        output VCU_PL_MCU_M_AXI_IC_DC_AWLOCK;
        output VCU_PL_MCU_M_AXI_IC_DC_AWPROT[3];
        output VCU_PL_MCU_M_AXI_IC_DC_AWQOS[4];
        output VCU_PL_MCU_M_AXI_IC_DC_AWSIZE[3];

        output VCU_PL_MCU_M_AXI_IC_DC_BREADY;
        input PL_VCU_MCU_M_AXI_IC_DC_BVALID;
        input PL_VCU_MCU_M_AXI_IC_DC_BID[3];
        input PL_VCU_MCU_M_AXI_IC_DC_BRESP[2];

        output VCU_PL_MCU_M_AXI_IC_DC_RREADY;
        input PL_VCU_MCU_M_AXI_IC_DC_RVALID;
        input PL_VCU_MCU_M_AXI_IC_DC_RDATA[32];
        input PL_VCU_MCU_M_AXI_IC_DC_RID[3];
        input PL_VCU_MCU_M_AXI_IC_DC_RLAST;
        input PL_VCU_MCU_M_AXI_IC_DC_RRESP[2];

        input PL_VCU_MCU_M_AXI_IC_DC_WREADY;
        output VCU_PL_MCU_M_AXI_IC_DC_WVALID;
        output VCU_PL_MCU_M_AXI_IC_DC_WDATA[32];
        output VCU_PL_MCU_M_AXI_IC_DC_WSTRB[4];
        output VCU_PL_MCU_M_AXI_IC_DC_WLAST;

        // misc

        input PL_VCU_RAW_RST_N;
        output VCU_PL_PINTREQ;
        input INIT_PL_VCU_GASKET_CLAMP_CONTROL_LVLSH_VCCINTD;
        output VCU_PL_PWR_SUPPLY_STATUS_VCCAUX;
        output VCU_PL_PWR_SUPPLY_STATUS_VCUINT;

        input PL_VCU_IOCHAR_DATA_IN_SEL_N;
        input PL_VCU_IOCHAR_DEC_AXI0_DATA_IN;
        input PL_VCU_IOCHAR_DEC_AXI1_DATA_IN;
        input PL_VCU_IOCHAR_ENC_AXI0_DATA_IN;
        input PL_VCU_IOCHAR_ENC_AXI1_DATA_IN;
        input PL_VCU_IOCHAR_ENC_CACHE_DATA_IN;
        input PL_VCU_IOCHAR_MCU_AXI_DATA_IN;
        output VCU_PL_IOCHAR_DEC_AXI0_DATA_OUT;
        output VCU_PL_IOCHAR_DEC_AXI1_DATA_OUT;
        output VCU_PL_IOCHAR_ENC_AXI0_DATA_OUT;
        output VCU_PL_IOCHAR_ENC_AXI1_DATA_OUT;
        output VCU_PL_IOCHAR_ENC_CACHE_DATA_OUT;
        output VCU_PL_IOCHAR_MCU_AXI_DATA_OUT;

        // test misc

        input PL_VCU_MBIST_ENABLE_N;
        input PL_VCU_MBIST_JTAP_TRST;
        input PL_VCU_MBIST_JTAP_TCK;
        input PL_VCU_MBIST_JTAP_TMS;
        input PL_VCU_MBIST_JTAP_TDI;
        output VCU_PL_MBIST_JTAP_TDO;
        output VCU_PL_MBIST_COMPARATOR_VALUE;
        input PL_VCU_MBIST_SPARE_IN[2];
        output VCU_PL_MBIST_SPARE_OUT[2];

        input PL_VCU_MCU_VDEC_DEBUG_CLK;
        input PL_VCU_MCU_VDEC_DEBUG_RST;
        input PL_VCU_MCU_VDEC_DEBUG_SYS_RST;
        input PL_VCU_MCU_VDEC_DEBUG_REG_EN[8];
        input PL_VCU_MCU_VDEC_DEBUG_CAPTURE;
        input PL_VCU_MCU_VDEC_DEBUG_SHIFT;
        input PL_VCU_MCU_VDEC_DEBUG_UPDATE;
        input PL_VCU_MCU_VDEC_DEBUG_TDI;
        output VCU_PL_MCU_VDEC_DEBUG_TDO;

        input PL_VCU_MCU_VENC_DEBUG_CLK;
        input PL_VCU_MCU_VENC_DEBUG_RST;
        input PL_VCU_MCU_VENC_DEBUG_SYS_RST;
        input PL_VCU_MCU_VENC_DEBUG_REG_EN[8];
        input PL_VCU_MCU_VENC_DEBUG_CAPTURE;
        input PL_VCU_MCU_VENC_DEBUG_SHIFT;
        input PL_VCU_MCU_VENC_DEBUG_UPDATE;
        input PL_VCU_MCU_VENC_DEBUG_TDI;
        output VCU_PL_MCU_VENC_DEBUG_TDO;

        input PL_VCU_SCANENABLE_CLKCTRL_N;
        input PL_VCU_SCAN_CHOPP_TRIGGER_N;
        input PL_VCU_SCAN_CLK;
        input PL_VCU_SCAN_EDTLOWP_EN_N;
        input PL_VCU_SCAN_EDT_BYPASS_N;
        input PL_VCU_SCAN_EDT_CLK;
        input PL_VCU_SCAN_EDT_UPDATE_N;
        input PL_VCU_SCAN_EN_N;
        input PL_VCU_SCAN_IN_CLK_CTRL;
        input PL_VCU_SCAN_IN_DEC[3];
        input PL_VCU_SCAN_IN_ENC[3];
        input PL_VCU_SCAN_IN_TOP[3];
        output VCU_PL_SCAN_OUT_CLK_CTRL;
        output VCU_PL_SCAN_OUT_DEC0[3];
        output VCU_PL_SCAN_OUT_DEC1[3];
        output VCU_PL_SCAN_OUT_ENC0[3];
        output VCU_PL_SCAN_OUT_ENC1[3];
        output VCU_PL_SCAN_OUT_ENC2[3];
        output VCU_PL_SCAN_OUT_ENC3[3];
        output VCU_PL_SCAN_OUT_TOP[3];
        input PL_VCU_SCAN_MODE_N;
        input PL_VCU_SCAN_PART_CTRL_N[7];
        input PL_VCU_SCAN_RAM_BYPASS_N;
        input PL_VCU_SCAN_RESET_N;
        input PL_VCU_SCAN_SPARE_IN[6];
        input PL_VCU_SCAN_TEST_TYPE_N;
        input PL_VCU_SCAN_WRAP_CLK;
        input PL_VCU_SCAN_WRAP_CTRL_N[2];
        output VCU_PL_SCAN_SPARE_OUT[6];

        input PL_VCU_SPARE_PORT_IN1[6];
        input PL_VCU_SPARE_PORT_IN2[6];
        input PL_VCU_SPARE_PORT_IN3[6];
        input PL_VCU_SPARE_PORT_IN4[6];
        input PL_VCU_SPARE_PORT_IN5[6];
        input PL_VCU_SPARE_PORT_IN6[6];
        input PL_VCU_SPARE_PORT_IN7[6];
        input PL_VCU_SPARE_PORT_IN8[6];
        input PL_VCU_SPARE_PORT_IN9[6];
        input PL_VCU_SPARE_PORT_IN10[6];
        input PL_VCU_SPARE_PORT_IN11[6];
        input PL_VCU_SPARE_PORT_IN12[6];
        input PL_VCU_SPARE_PORT_IN13[6];
        output VCU_PL_SPARE_PORT_OUT1[2];
        output VCU_PL_SPARE_PORT_OUT2[2];
        output VCU_PL_SPARE_PORT_OUT3[2];
        output VCU_PL_SPARE_PORT_OUT4[2];
        output VCU_PL_SPARE_PORT_OUT5[2];
        output VCU_PL_SPARE_PORT_OUT6[2];
        output VCU_PL_SPARE_PORT_OUT7[2];
        output VCU_PL_SPARE_PORT_OUT8[2];
        output VCU_PL_SPARE_PORT_OUT9[6];
        output VCU_PL_SPARE_PORT_OUT10[6];
        output VCU_PL_SPARE_PORT_OUT11[6];
        output VCU_PL_SPARE_PORT_OUT12[6];
        output VCU_PL_SPARE_PORT_OUT13[6];

        input VCU_TEST_IN[54];
        output VCU_TEST_OUT[54];
    }

    bel_class GT3_COMMON {
        input DCLKB_FS;
        input DEN_FS;
        input DWE_FS;
        // GTH has only 9 bits of address
        input DADDR_FS[10];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

     for i in 0..2 {
            output "REFCLK2HROW{i}";

               // GTY only
               input "COREREFCLKB_FS_{i}";

            output "DMONOUT_QDCM_SF_{i}"[8];
            input "HROW_TEST_CKB_FS_{i}";
            output "MGTREFCLKFA_SF_{i}";
            input "QDCLKPINSPRDB0_FS_{i}";
            input "QDCLKPINSPRDB1_FS_{i}";
            output "QDCMRSVDOUT_SF_{i}"[8];
            input "QDCMRSVDPIN_FS_{i}"[8];
            input "QDPINSPRD_FS_{i}"[5];
            input "QDPMAPINRSVD_FS_{i}"[8];
            output "QPLFBLOSS_SF_{i}";
            output "QPLFREQLOCK_SF_{i}";
            input "QPLLDMONCLKB_FS_{i}";
            input "QPLLKDETEN_FS_{i}";
            input "QPLPWRDN_FS_{i}";
            input "QPLREFDYN_FS_{i}"[3];
            output "QPLREFLOSS_SF_{i}";
            input "QPLRESETB_FS_{i}";
            input "REFCLKPD_FS_{i}";

               // GTY only
            input "SDMDATA_FS_{i}"[25];
            output "SDMFINALOUT_SF_{i}"[4];
            input "SDMRESETB_FS_{i}";
            output "SDMTESTDATA_SF_{i}"[15];
            input "SDMWIDTH_FS_{i}"[2];
        }

     input BGBYPASS_FS_1;
        input BGRCALCTL_FS_1[5];
        input BGRCALOVRDENB_FS_1;
        input BGTESTEN_FS_1;
        input PWRDNBGB_FS_1;
        input RCALENB_FS_1;

     input QDPMASCANCLKB_FS[8];
        input QDPMASCANRSTEN_FS;
        input QDPMASCANENB_FS;
        input QDPMASCANMODEB_FS;
        input QDPMASCANIN_FS[8];
        output QDPMASCANOUT_SF[8];
    }

 bel_class GT3_CHANNEL {
        input DCLKB_FS;
        input DEN_FS;
        input DWE_FS;
        // GTH has only 9 bits of address
        input DADDR_FS[10];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

     input AFECFOKEN_FS;
        input ALT_SIGVALID_CLKB_FS;

     // GTH only
           input AGCCTRL_FS[2];

     output BUFGTCEMASK_SF[3];
        output BUFGTCE_SF[3];
        output BUFGTDIV_SF[9];
        output BUFGTRESET_SF[3];
        output BUFGTRSTMASK_SF[3];

     input CDRFRRESETB_FS;
        input CDRHOLD_FS;
        output CDRLOCK_SF;
        input CDROVREN_FS;
        input CDRPHRESETB_FS;
        input CDRRESETB_FS;

     // GTY only
        input CDRSTEPDIR_FS;
        input CDRSTEPSQ_FS;
        input CDRSTEPSX_FS;

     input CFGRESETB_FS;

     output CFOKDONE_SF;
        output CFOKFORCEDONE_SF;
        output CFOKFSTARTED_SF;
        input CFOKRESET_FS;
        output CFOKSTART_SF;

     // GTY only
        output CKOKDONE_SF;
        input CKOKRESET_FS;

     input CKPINRSRVD0B_FS;
        input CKPINRSRVD1B_FS;

     output COMFINISH_SF;
        output COMINITDET_SF;
        output COMSASDET_SF;
        output COMWAKEDET_SF;

     input COREREFCLKB_FS;

     output CPLFBLOSS_SF;
        output CPLFREQLOCK_SF;
        input CPLLDMONCLKB_FS;
        input CPLLKDETEN_FS;
        input CPLPWRDN_FS;
        output CPLREFLOSS_SF;
        input CPLREFSELDYN_FS0;
        input CPLREFSELDYN_FS1;
        input CPLREFSELDYN_FS2;
        input CPLRESETB_FS;

     input DFECFOKCFNUM_FS[4];
        input DFECFOKFEN_FS;
        input DFECFOKFPULSE_FS;
        input DFECFOKHOLD_FS;
        input DFECFOKOVREN_FS;
        input DFEDOUTMODE_FS[2];
        output DFEDOUT_SF[7];
        input DFEGCHOLD_FS;
        input DFEGCOVREN_FS;
        input DFEH2HOLD_FS;
        input DFEH2OVREN_FS;
        input DFEH3HOLD_FS;
        input DFEH3OVREN_FS;
        input DFEH4HOLD_FS;
        input DFEH4OVREN_FS;
        input DFEH5HOLD_FS;
        input DFEH5OVREN_FS;
        input DFEH6HOLD_FS;
        input DFEH6OVREN_FS;
        input DFEH7HOLD_FS;
        input DFEH7OVREN_FS;
        input DFEH8HOLD_FS;
        input DFEH8OVREN_FS;
        input DFEH9HOLD_FS;
        input DFEH9OVREN_FS;
        input DFEH10HOLD_FS;
        input DFEH10OVREN_FS;
        input DFEH11HOLD_FS;
        input DFEH11OVREN_FS;
        input DFEH12HOLD_FS;
        input DFEH12OVREN_FS;
        input DFEH13HOLD_FS;
        input DFEH13OVREN_FS;
        input DFEH14HOLD_FS;
        input DFEH14OVREN_FS;
        input DFEH15HOLD_FS;
        input DFEH15OVREN_FS;
        input DFEKLHOLD_FS;
        input DFEKLOVREN_FS;
        input DFERESETB_FS;
        input DFEUTHOLD_FS;
        input DFEUTOVREN_FS;
        input DFEVPHOLD_FS;
        input DFEVPOVREN_FS;
        input DFEVSEN_FS;
        input DFEYEN_FS;

     input DMONCLKB_FS;
        input DMONFIFORESET_FS;
        output DMONOUT_SF[17];
        input ENPPM_FS;
        input EVODDPHICALDONE_FS;
        input EVODDPHICALSTART_FS;
        input EVODDPHIDRDEN_FS;
        input EVODDPHIDWREN_FS;
        input EVODDPHIXRDEN_FS;
        input EVODDPHIXWREN_FS;
        output EYESCANDATAERROR_SF;
        input EYESCANTRIGGER_FS;
        input GATERXELECIDLE_FS[2];
        output GTPOWERGOOD_SF;
        input GTRSTSEL_FS;
        input GTRXRSTB_FS;
        input GTTXRSTB_FS;
        input ISCANRESET_FS;
        input ISERRDETEN_FS;

     input LOOPBACK_FS[3];

     // GTY only
        input LOOPRSVD_FS[16];

     input LPBKRXTXSEREN_FS;
        input LPBKTXRXSEREN_FS;

     input LPMGCHOLD_FS;
        input LPMGCOVREN_FS;
        input LPMKHHOLD_FS;
        input LPMKHOVREN_FS;
        input LPMKLHOLD_FS;
        input LPMKLOVREN_FS;
        input LPMOSHOLD_FS;
        input LPMOSOVREN_FS;

     output MGTREFCLKFA_SF;

     input OSHOLD_FS;
        input OSOVREN_FS;

     input PCIEEQRXEQADAPTDONE_FS;
        output PCIERATEGEN3_SF;
        output PCIERATEIDLE_SF;
        output PCIERATEQPLLPD_SF[2];
        output PCIERATEQPLLRESET_SF[2];
        input PCIERSTIDLE_FS;
        input PCIERSTTXSYNCSTART_FS;
        output PCIESYNCTXSYNCDONE_SF;
        output PCIEUSERGEN3RDY_SF;
        output PCIEUSERPHYSTATUSRST_SF;

     input PCS_RSVD_IN_FS[16];
        output PCS_RSVD_OUT_SF[16];
        output PHYSTATUS_SF;
        output PINRSRVDAS_SF[8];
        input PINRSRVD_FS[16];

     input PRBSCNTRST_FS;

     input RATE_DRP_DONE_EXT_FS;
        output RATE_DRP_START_SF;

     input RCODAPWDN_FS;
        input RCODARESETB_FS;
        input RCOOVREN_FS;
        output RDASOFTRSTDONE_SF;
        input RESETOVRD_FS;

     // GTY only
        output RESET_EXCEPTION_SF;

     input RSTCLKENTX_FS;

     input RXBUFRSTB_FS;
        output RXBUFSTATUS_SF[3];
        output RXBYTEISALIGNED_SF;
        output RXBYTEREALIGN_SF;

     // GTY only
        output RXCDRPHDONE_SF;

     output RXCHANBONDSEQ_SF;
        output RXCHANREALIGN_SF;
        output RXCHARISCOMMA_SF[8];
        output RXCHARISK_SF[16];
        input RXCHBONDI_FS[5];
        input RXCHBONDLEVEL_FS[3];
        input RXCHBONDMASTER_FS;
        output RXCHBONDO_SF[5];
        input RXCHBONDSLAVE_FS;
        output RXCHISALIGNED_SF;
        output RXCKCORCNT_SF[2];
        input RXCOALGNEN_FS;
        input RXCOMMADETUSE_FS;
        output RXCOMMADET_SF;
        output RXCOPHDONE_SF;
        input RXCOSETPHS_FS;
        input RXDAALGNEN_FS;
        input RXDABYPASS_FS;
        input RXDAOVREN_FS;
        input RXDASOFTRESETB_FS;
        output RXDATAEXTENDRSVD_SF[8];
        output RXDATAVALID_SF[2];
        output RXDATA_SF[128];
        input RXDEC8B10BUSE_FS;
        output RXDISPERR_SF[16];
        output RXELECIDLE_SF;
        input RXENCHANSYNC_FS;
        input RXENMCOMMAALIGN_FS;
        input RXENPCOMMAALIGN_FS;
        input RXGEARBOXSLIP_FS;
        output RXHEADERVALID_SF[2];
        output RXHEADER_SF[6];
        input RXLATCLKB_FS;
        output RXLINKSYNCDONE_SF;
        input RXLPMEN_FS;
        input RXMSTRSETPHDONE_FS;
        output RXNOTINTABLE_SF[8];
        input RXOOBRESETB_FS;
        input RXOUTCKCTL_FS[3];
        output RXPHALIGNERR_SF;
        input RXPLLCKSEL_FS[2];
        input RXPMARESETB_FS;
        output RXPMARESETDONE_SF;
        input RXPOLARITY_FS;
        output RXPRBSERR_SF;
        output RXPRBSLOCKED_SF;
        input RXPRBSPTN_FS[4];

     // GTY only
        output RXPRGDIVRSTDONE_SF;

     input RXPROGDIVRESETB_FS;
        input RXPWRDN_FS[2];

     // GTH only
        input RXQPIEN_FS;
        output RXQPISENN_SF;
        output RXQPISENP_SF;

     input RXRATEASYNCH_FS;
        output RXRATEDONE_SF;
        input RXRATE_FS[3];
        output RXRECCLKPCS_SF;
        output RXRECCLK_INT;
        output RXRECCLK_SF;
        output RXRESETDONE_SF;
        input RXRESET_FS;
        input RXSIGVALDLY_FS[5];
        output RXSLIDERDY_SF;
        input RXSLIDE_FS;
        output RXSLIPDONE_SF;
        output RXSLIPOUTCLKRDY_SF;
        input RXSLIPOUTCLK_FS;
        output RXSLIPPMARDY_SF;
        input RXSLIPPMA_FS;
        input RXSLVSYNCEN_FS;
        output RXSTARTOFSEQ_SF[2];
        output RXSTATUS_SF[3];
        output RXSYNCEN2SLV_SF;
        input RXSYNCFSMMASTER_FS;
        input RXSYSCKSEL_FS[2];
        input RXTERMPROG_FS[5];
        input RXUSRCLK2B_FS;
        input RXUSRCLKB_FS;
        input RXUSRRDY_FS;
        output RXVALID_SF;

     // GTY only
        input RX_POST_DCC_START_FS;

     input STEPSIZEPPM_FS[5];
        input TCOCLKFSMFROUTB_FS;
        input TCODABYPASS_FS;
        input TCODAOVREN_FS;
        input TCODAPWDN_FS;
        input TCODARESET_FS;
        input TCOHOLDFROUT_FS;
        output TCOINITDONE_SF;
        input TCOINITSET_FS;
        input TCOPIOVREN_FS;
        input TCOUPDNFROUT_FS;
        input TDASOFTRESET_FS;
        output TDASOFTRSTDONE_SF;

     output TXBUFSTATUS_SF[2];
        input TXBYPASS8B10B_FS[8];
        input TXCHARDISPMODE_FS[16];
        input TXCHARDISPVAL_FS[16];
        input TXCHARISK_FS[8];
        input TXCODAALGNEN_FS;
        input TXCOMINIT_FS;
        input TXCOMSAS_FS;
        input TXCOMWAKE_FS;
        input TXCOPIALGNEN_FS;
        output TXCOPIPHDONE_SF;
        input TXCOPISETPHS_FS;
        input TXDATAEXTENDRSVD_FS[8];
        input TXDATA_FS[128];

     // GTY only
        output TXDCCDONE_SF;
        input TXDCCSRESET_FS;
        input TXDCCSTART_FS;

     input TXDEEMPH_FS;
        input TXDETECTRX_FS;

     // GTH has only 4 bits
        input TXDRVAMP_FS[5];

     input TXDRVPWRDN_FS;
        input TXELECIDLE_FS;
        input TXEMPINVPO_FS;
        input TXEMPINVPR_FS;
        input TXEMPMAIN_FS[7];
        input TXEMPPOS_FS[5];
        input TXEMPPRE_FS[5];
        input TXENC8B10BUSE_FS;
        input TXHEADER_FS[6];
        input TXINHIBIT_FS;
        input TXLATCLKB_FS;
        output TXLINKSYNCDONE_SF;
        input TXMARGIN_FS[3];
        input TXMSTRSETPHDONE_FS;
        input TXOUTCKCTL_FS[3];
        output TXOUTCLKPCS_SF;
        output TXOUTCLK_INT;
        output TXOUTCLK_SF;
        input TXPDOWNASYNCH_FS;
        input TXPIPPMPWDN_FS;
        input TXPLLCKSEL_FS[2];
        input TXPMARESETB_FS;
        output TXPMARESETDONE_SF;
        input TXPOLARITY_FS;
        input TXPPMOVRDEN_FS;
        input TXPPMSEL_FS;
        input TXPRBSINERR_FS;
        input TXPRBSPTN_FS[4];
        input TXPREAMP_FS[3];

     // GTY only
        output TXPRGDIVRSTDONE_SF;

     input TXPROGDIVRESETB_FS;
        input TXPWRDN_FS[2];

     // GTH only
        input TXQPIBIASEN_FS;
        output TXQPISENN_SF;
        output TXQPISENP_SF;
        input TXQPISTRGPD_FS;
        input TXQPIWEAKPU_FS;

     input TXRATEASYNCH_FS;
        output TXRATEDONE_SF;
        input TXRATE_FS[3];
        output TXRESETDONE_SF;
        input TXRESET_FS;
        input TXSEQUENCE_FS[7];
        input TXSERPWRDN_FS;
        input TXSLVSYNCEN_FS;
        input TXSWING_FS;
        output TXSYNCEN2SLV_SF;
        input TXSYNCFSMMASTER_FS;
        input TXSYSCKSEL_FS[2];
        input TXUSRCLK2B_FS;
        input TXUSRCLKB_FS;
        input TXUSRRDY_FS;

     // GTY only
        input TX_EL_FORCE_START_FS;

     input TSTCLK0B_FS;
        input TSTCLK1B_FS;
        input TSTIN_FS[20];
        input TSTPWRDNOVRDB_FS;
        input TSTPWRDN_FS[5];

     input PMASCANCLK0B_FS;
        input PMASCANCLK1B_FS;
        input PMASCANCLK2B_FS;
        input PMASCANCLK3B_FS;
        input PMASCANCLK4B_FS;
        input PMASCANCLK5B_FS;
        input PMASCANRSTEN_FS;
        input PMASCANENB_FS;
        input PMASCANMODEB_FS;
        input PMASCANIN_FS[12];
        output PMASCANOUT_SF[12];

     input SCANCLKB_FS;
        input SCANENB_FS;
        input SCANMODEB_FS;
        input SCANIN_FS[19];
        output SCANOUT_SF[19];
    }


    bel_class GT4_COMMON {
        input DCLK_FS;
        input DEN_FS;
        input DWE_FS;
        input DADDR_FS[16];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

        for i in 0..2 {
            output "REFCLK2HROW{i}";
            input "CSSDSTOPCLKCM_FS_{i}";
            output "CSSDSTOPCLKDONECM_SF_{i}";
            output "DMONOUT_QDCM_SF_{i}"[8];
            input "HROW_TEST_CK_FS_{i}";
            output "MGTREFCLKFA_SF_{i}";
            input "QDCLKPINSPRD0_FS_{i}";
            input "QDCLKPINSPRD1_FS_{i}";
            output "QDCMRSVDOUT_SF_{i}"[8];
            input "QDCMRSVDPIN_FS_{i}"[8];
            input "QDCOREREFCLK_FS_{i}";
            input "QDPINSPRD_FS_{i}"[5];
            input "QDPMAPINRSVD_FS_{i}"[8];
            input "QPLFBDIV_FS_{i}"[8];
            output "QPLFBLOSS_SF_{i}";
            output "QPLFREQLOCK_SF_{i}";
            input "QPLLDMONCLK_FS_{i}";
            input "QPLLKDETEN_FS_{i}";
            input "QPLPWRDN_FS_{i}";
            input "QPLREFDYN_FS_{i}"[3];
            output "QPLREFLOSS_SF_{i}";
            input "QPLRESET_FS_{i}";
            input "REFCLKPD_FS_{i}";
            input "SDMDATA_FS_{i}"[25];
            output "SDMFINALOUT_SF_{i}"[4];
            input "SDMRESET_FS_{i}";
            output "SDMTESTDATA_SF_{i}"[15];
            input "SDMTOGGLE_FS_{i}";
            input "SDMWIDTH_FS_{i}"[2];
            input "TXRATE_CM_FS_{i}"[3];
        }

        input BGBYPASS_FS_1;
        input BGRCALCTL_FS_1[5];
        input BGRCALOVRDENB_FS_1;
        input BGTESTEN_FS_1;
        input PWRDNBGB_FS_1;
        input RCALENB_FS_1;

        // GTH only
        input GPI_FS[10];
        output GPO_SF[10];
        input SPD_FS[2];
        input MDMDBGIN_FS[5];
        input MDMDBGRST_FS[2];
        output MDMDBGTDO_SF;
        input UBLAZEPOWERUP_FS;
        input UBLAZERESET_FS[2];
        input UTRST_FS;
        input UTCK_FS;
        input UTMS_FS;
        input UTDI_FS;
        output UTDO_SF;
        input UTESTMBISTMODE_FS;

        // GTY only
        input UB_ENABLE_FS;
        input UB_GPI_FS[2];
        input UB_INTR_FS[2];
        input UB_IOLMBRST_FS;
        input UB_MBRST_FS;
        input UB_MDMCAPTURE_FS;
        input UB_MDMDBGRST_FS;
        input UB_MDMDBGUPDATE_FS;
        input UB_MDMREGEN_FS[4];
        input UB_MDMSHIFT_FS;
        input UB_MDMSYSRST_FS;
        input UB_MDMTCK_FS;
        input UB_MDMTDI_FS;
        output UB_MDMTDO_SF;
        output UB_OUTDEN_SF;
        output UB_OUTDWE_SF;
        output UB_OUTDADDR_SF[16];
        output UB_OUTDI_SF[16];
        input UB_OUTDO_FS[16];
        input UB_OUTDRDY_FS;
        input UB_RSVDIN_FS;
        output UB_RSVDOUT_SF;
        output UB_TXUART_SF;

        input QDPMASCANCLK_FS[14];
        input QDPMASCANRSTEN_FS;
        input QDPMASCANENB_FS;
        input QDPMASCANMODEB_FS;
        input QDPMASCANIN_FS[14];
        output QDPMASCANOUT_SF[14];
    }

    bel_class GT4_CHANNEL {
        input AFECFOKEN_FS;
        input AGCCTRL_FS[2];
        input ALT_SIGVALID_CLK_FS;
        input BSR_SERIAL_FS;

        output BUFGTCEMASK_SF[3];
        output BUFGTCE_SF;
        output BUFGTDIV_SF[9];
        output BUFGTRESET_SF;
        output BUFGTRSTMASK_SF[3];

        input CDRFRRESET_FS;
        input CDRHOLD_FS;
        output CDRLOCK_SF;
        input CDROVREN_FS;
        input CDRPHRESET_FS;
        input CDRSTEPDIR_FS;
        input CDRSTEPSQ_FS;
        input CDRSTEPSX_FS;

        input CFGRESET_FS;

        output CFOKDONE_SF;
        output CFOKFORCEDONE_SF;
        output CFOKFSTARTED_SF;
        input CFOKRESET_FS;
        output CFOKSTART_SF;

        output CKOKDONE_SF;
        input CKOKRESET_FS;

        input CKPINRSRVD0_FS;
        input CKPINRSRVD1_FS;

        output COMFINISH_SF;
        output COMINITDET_SF;
        output COMSASDET_SF;
        output COMWAKEDET_SF;

        input COREREFCLK_FS;

        output CPLFBLOSS_SF;
        output CPLFREQLOCK_SF;
        input CPLLDMONCLK_FS;
        input CPLLFREQLOCK_FS;
        input CPLLKDETEN_FS;
        input CPLPWRDN_FS;
        output CPLREFLOSS_SF;
        input CPLREFSELDYN_FS[3];
        input CPLRESET_FS;

        input CSSDRSTB_FS;
        output CSSDSTOPCLKDONE_SF;
        input CSSDSTOPCLK_FS;

        input DCLK_FS;
        input DRST_FS;
        input DEN_FS;
        input DWE_FS;
        input DADDR_FS[10];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

        input DFECFOKCFNUM_FS[4];
        input DFECFOKFEN_FS;
        input DFECFOKFPULSE_FS;
        input DFECFOKHOLD_FS;
        input DFECFOKOVREN_FS;
        input DFEDOUTMODE_FS[2];
        output DFEDOUT_SF[8];
        input DFEGCHOLD_FS;
        input DFEGCOVREN_FS;
        input DFEH2HOLD_FS;
        input DFEH2OVREN_FS;
        input DFEH3HOLD_FS;
        input DFEH3OVREN_FS;
        input DFEH4HOLD_FS;
        input DFEH4OVREN_FS;
        input DFEH5HOLD_FS;
        input DFEH5OVREN_FS;
        input DFEH6HOLD_FS;
        input DFEH6OVREN_FS;
        input DFEH7HOLD_FS;
        input DFEH7OVREN_FS;
        input DFEH8HOLD_FS;
        input DFEH8OVREN_FS;
        input DFEH9HOLD_FS;
        input DFEH9OVREN_FS;
        input DFEH10HOLD_FS;
        input DFEH10OVREN_FS;
        input DFEH11HOLD_FS;
        input DFEH11OVREN_FS;
        input DFEH12HOLD_FS;
        input DFEH12OVREN_FS;
        input DFEH13HOLD_FS;
        input DFEH13OVREN_FS;
        input DFEH14HOLD_FS;
        input DFEH14OVREN_FS;
        input DFEH15HOLD_FS;
        input DFEH15OVREN_FS;
        input DFEKLHOLD_FS;
        input DFEKLOVREN_FS;
        input DFERESET_FS;
        input DFEUTHOLD_FS;
        input DFEUTOVREN_FS;
        input DFEVPHOLD_FS;
        input DFEVPOVREN_FS;
        input DFEYEN_FS;
        input DFE_KH_EXTHOLD_FS;
        input DFE_KH_OVERWREN_FS;

        input DMONCLK_FS;
        input DMONFIFORESET_FS;
        output DMONOUTCLK_INT;
        output DMONOUT_SF[16];

        input ENPPM_FS;
        output EYESCANDATAERROR_SF;
        input EYESCANTRIGGER_FS;
        input FREQOS_FS;
        input GATERXELECIDLE_FS[2];
        output GTPOWERGOOD_SF;
        input GTRXRSTSEL_FS;
        input GTRXRST_FS;
        input GTTXRSTSEL_FS;
        input GTTXRST_FS;
        input INCPCTRL_FS;
        input ISCANRESET_FS;
        output LFPS_TRESETDET_SF;
        input LFPS_TRESET_FS;
        output LFPS_U2LPEXITDET_SF;
        input LFPS_U2LPEXIT_FS;
        output LFPS_U3WAKEDET_SF;
        input LFPS_U3WAKE_FS;
        input LOOPBACK_FS[3];
        input LPMGCHOLD_FS;
        input LPMGCOVREN_FS;
        input LPMKHHOLD_FS;
        input LPMKHOVREN_FS;
        input LPMKLHOLD_FS;
        input LPMKLOVREN_FS;
        input LPMOSHOLD_FS;
        input LPMOSOVREN_FS;
        output MGTREFCLKFA_SF;
        input OSHOLD_FS;
        input OSOVREN_FS;

        input PCIEEQRXEQADAPTDONE_FS;
        output PCIERATEGEN3_SF;
        output PCIERATEIDLE_SF;
        output PCIERATEQPLLPD_SF[2];
        output PCIERATEQPLLRESET_SF[2];
        input PCIERSTIDLE_FS;
        input PCIERSTTXSYNCSTART_FS;
        output PCIESYNCTXSYNCDONE_SF;
        output PCIEUSERGEN3RDY_SF;
        output PCIEUSERPHYSTATUSRST_SF;
        input PCS_RSVD_IN_FS[16];
        output PCS_RSVD_OUT_SF[16];
        output PHYSTATUS_SF;
        output PINRSRVDAS_SF[16];
        input PINRSRVD_FS[16];
        output POWERPRESENT_SF;
        input PRBSCNTRST_FS;
        input QPLLFREQLOCK0_FS;
        input QPLLFREQLOCK1_FS;
        input RATE_DRP_DONE_EXT_FS;
        output RATE_DRP_START_SF;
        input RCODAPWDN_FS;
        input RCODARESET_FS;
        input RCOOVREN_FS;
        output RDASOFTRSTDONE_SF;
        input RESETOVRD_FS;
        output RESET_EXCEPTION_SF;
        input RXBUFRST_FS;
        output RXBUFSTATUS_SF[3];
        output RXBYTEISALIGNED_SF;
        output RXBYTEREALIGN_SF;
        output RXCDRPHDONE_SF;
        output RXCHANBONDSEQ_SF;
        output RXCHANREALIGN_SF;
        output RXCHARISCOMMA_SF[8];
        output RXCHARISK_SF[16];
        input RXCHBONDI_FS[5];
        input RXCHBONDLEVEL_FS[3];
        input RXCHBONDMASTER_FS;
        output RXCHBONDO_SF[5];
        input RXCHBONDSLAVE_FS;
        output RXCHISALIGNED_SF;
        input RXCKCALSTART_FS[7];
        output RXCKCORCNT_SF[2];
        input RXCOALGNEN_FS;
        input RXCOMMADETUSE_FS;
        output RXCOMMADET_SF;
        output RXCOPHDONE_SF;
        input RXCOSETPHS_FS;
        input RXDAALGNEN_FS;
        input RXDABYPASS_FS;
        input RXDAOVREN_FS;
        input RXDASOFTRESET_FS;
        output RXDATAEXTENDRSVD_SF[8];
        output RXDATAVALID_SF[2];
        output RXDATA_SF[128];
        input RXDEC8B10BUSE_FS;
        output RXDISPERR_SF[16];
        output RXELECIDLE_SF;
        input RXENCHANSYNC_FS;
        input RXENMCOMMAALIGN_FS;
        input RXENPCOMMAALIGN_FS;
        input RXEQTRAINING_FS;
        input RXGEARBOXSLIP_FS;
        output RXHEADERVALID_SF[2];
        output RXHEADER_SF[6];
        input RXLATCLK_FS;
        output RXLINKSYNCDONE_SF;
        input RXLPMEN_FS;
        input RXMSTRSETPHDONE_FS;
        output RXNOTINTABLE_SF[8];
        input RXOOBRESET_FS;
        input RXOUTCKCTL_FS[3];
        output RXPHALIGNERR_SF;
        input RXPLLCKSEL_FS[2];
        output RXPMARESETDONE_SF;
        input RXPMARESET_FS;
        input RXPOLARITY_FS;
        output RXPRBSERR_SF;
        output RXPRBSLOCKED_SF;
        input RXPRBSPTN_FS[4];
        output RXPRGDIVRSTDONE_SF;
        input RXPROGDIVRESET_FS;
        input RXPWRDN_FS[2];
        input RXQPIEN_FS;
        input RXRATEASYNCH_FS;
        output RXRATEDONE_SF;
        input RXRATE_FS[3];
        output RXRECCLKPCS_SF;
        output RXRECCLK_INT;
        output RXRECCLK_SF;
        output RXRESETDONE_SF;
        input RXRESET_FS;
        output RXSLIDERDY_SF;
        input RXSLIDE_FS;
        output RXSLIPDONE_SF;
        output RXSLIPOUTCLKRDY_SF;
        input RXSLIPOUTCLK_FS;
        output RXSLIPPMARDY_SF;
        input RXSLIPPMA_FS;
        input RXSLVSYNCEN_FS;
        output RXSTARTOFSEQ_SF[2];
        output RXSTATUS_SF[3];
        output RXSYNCEN2SLV_SF;
        input RXSYNCFSMMASTER_FS;
        input RXSYSCKSEL_FS[2];
        input RXTERMINATION_FS;
        input RXUSRCLK2_FS;
        input RXUSRCLK_FS;
        input RXUSRRDY_FS;
        output RXVALID_SF;
        output RX_QPISENN_SF;
        output RX_QPISENP_SF;
        input STEPSIZEPPM_FS[5];
        input TCOCLKFSMFROUT_FS;
        input TCODABYPASS_FS;
        input TCODAOVREN_FS;
        input TCODAPWDN_FS;
        input TCODARESET_FS;
        input TCOHOLDFROUT_FS;
        output TCOINITDONE_SF;
        input TCOINITSET_FS;
        input TCOPIOVREN_FS;
        input TCOUPDNFROUT_FS;
        input TDASOFTRESET_FS;
        output TDASOFTRSTDONE_SF;
        input TSTCLK0_FS;
        input TSTCLK1_FS;
        input TSTIN_FS[20];
        input TSTPWRDNOVRDB_FS;
        input TSTPWRDN_FS[5];
        output TXBUFSTATUS_SF[2];
        input TXBYPASS8B10B_FS[8];
        input TXCHARDISPMODE_FS[16];
        input TXCHARDISPVAL_FS[16];
        input TXCHARISK_FS[8];
        input TXCODAALGNEN_FS;
        input TXCOMINIT_FS;
        input TXCOMSAS_FS;
        input TXCOMWAKE_FS;
        input TXCOPIALGNEN_FS;
        output TXCOPIPHDONE_SF;
        input TXCOPISETPHS_FS;
        input TXDATAEXTENDRSVD_FS[8];
        input TXDATA_FS[128];
        output TXDCCDONE_SF;
        input TXDCCSRESET_FS;
        input TXDCCSTART_FS;
        input TXDEEMPH_FS[2];
        input TXDETECTRX_FS;
        input TXDRVAMP_FS[5];
        input TXELECIDLE_FS;
        input TXEMPMAIN_FS[7];
        input TXEMPPOS_FS[5];
        input TXEMPPRE_FS[5];
        input TXENC8B10BUSE_FS;
        input TXHEADER_FS[6];
        input TXINHIBIT_FS;
        input TXLATCLK_FS;
        output TXLINKSYNCDONE_SF;
        input TXMARGIN_FS[3];
        input TXMSTRSETPHDONE_FS;
        input TXMUXDCDEXHOLD_FS;
        input TXMUXDCDORWREN_FS;
        input TXONESZEROS_FS;
        input TXOUTCKCTL_FS[3];
        output TXOUTCLKPCS_SF;
        output TXOUTCLK_INT;
        output TXOUTCLK_SF;
        input TXPDOWNASYNCH_FS;
        input TXPIPPMPWDN_FS;
        input TXPLLCKSEL_FS[2];
        output TXPMARESETDONE_SF;
        input TXPMARESET_FS;
        input TXPOLARITY_FS;
        input TXPPMOVRDEN_FS;
        input TXPPMSEL_FS;
        input TXPRBSINERR_FS;
        input TXPRBSPTN_FS[4];
        output TXPRGDIVRSTDONE_SF;
        input TXPROGDIVRESET_FS;
        input TXPWRDN_FS[2];
        input TXQPIBIASEN_FS;
        input TXQPIWEAKPU_FS;
        input TXRATEASYNCH_FS;
        output TXRATEDONE_SF;
        input TXRATE_FS[3];
        output TXRESETDONE_SF;
        input TXRESET_FS;
        input TXSEQUENCE_FS[7];
        input TXSERPWRDN_FS;
        input TXSLVSYNCEN_FS;
        input TXSWING_FS;
        output TXSYNCEN2SLV_SF;
        input TXSYNCFSMMASTER_FS;
        input TXSYSCKSEL_FS[2];
        input TXUSRCLK2_FS;
        input TXUSRCLK_FS;
        input TXUSRRDY_FS;
        output TX_QPISENN_SF;
        output TX_QPISENP_SF;

        input SCANCLK_FS;
        input SCANRSTB_FS;
        input SCANRSTEN_FS;
        input SCANENB_FS;
        input SCANMODEB_FS;
        input SCANIN_FS[19];
        output SCANOUT_SF[19];
        input PMASCANCLK0_FS;
        input PMASCANCLK1_FS;
        input PMASCANCLK2_FS;
        input PMASCANCLK3_FS;
        input PMASCANCLK4_FS;
        input PMASCANCLK5_FS;
        input PMASCANCLK6_FS;
        input PMASCANCLK7_FS;
        input PMASCANCLK8_FS;
        input PMASCANRSTEN_FS;
        input PMASCANENB_FS;
        input PMASCANMODEB_FS;
        input PMASCANIN_FS[18];
        output PMASCANOUT_SF[18];
    }

    bel_class GTF_COMMON {
        input DCLK_FS;
        input DEN_FS;
        input DWE_FS;
        input DADDR_FS[16];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

        for i in 0..2 {
            output "DMONOUT_QDCM_SF_{i}"[8];
            output "REFCLK2HROW{i}";
            input "HROW_TEST_CK_FS_{i}";
            output "MGTREFCLKFA_SF_{i}";
            input "QDCLKPINSPRD0_FS_{i}";
            input "QDCLKPINSPRD1_FS_{i}";
            output "QDCMRSVDOUT_SF_{i}"[8];
            input "QDCMRSVDPIN_FS_{i}"[8];
            input "QDCOREREFCLK_FS_{i}";
            input "QDPINSPRD_FS_{i}"[5];
            input "QDPMAPINRSVD_FS_{i}"[8];
            input "QPLFBDIV_FS_{i}"[8];
            output "QPLFBLOSS_SF_{i}";
            output "QPLFREQLOCK_SF_{i}";
            input "QPLLDMONCLK_FS_{i}";
            input "QPLLKDETEN_FS_{i}";
            input "QPLPWRDN_FS_{i}";
            input "QPLREFDYN_FS_{i}"[3];
            output "QPLREFLOSS_SF_{i}";
            input "QPLRESET_FS_{i}";
            input "REFCLKPD_FS_{i}";
            input "SDMDATA_FS_{i}"[25];
            output "SDMFINALOUT_SF_{i}"[4];
            input "SDMRESET_FS_{i}";
            output "SDMTESTDATA_SF_{i}"[15];
            input "SDMTOGGLE_FS_{i}";
            input "SDMWIDTH_FS_{i}"[2];
        }

        input BGBYPASS_FS_1;
        input BGRCALCTL_FS_1[5];
        input BGRCALOVRDENB_FS_1;
        input BGTESTEN_FS_1;
        input PWRDNBGB_FS_1;
        input RCALENB_FS_1;

        input QDPMASCANCLK_FS[14];
        input QDPMASCANRSTEN_FS;
        input QDPMASCANENB_FS;
        input QDPMASCANMODEB_FS;
        input QDPMASCANIN_FS[14];
        output QDPMASCANOUT_SF[14];
    }

    bel_class GTF_CHANNEL {
        input DCLK_FS;
        input DRST_FS;
        input DEN_FS;
        input DWE_FS;
        input DADDR_FS[10];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

        input AFECFOKEN_FS;
        input BSR_SERIAL_FS;

        input CDRFRRESET_FS;
        input CDRHOLD_FS;
        output CDRLOCK_SF;
        input CDROVREN_FS;
        input CDRPHRESET_FS;
        input CDRSTEPDIR_FS;
        input CDRSTEPSQ_FS;
        input CDRSTEPSX_FS;

        input CFGRESET_FS;

        output CFOKDONE_SF;
        output CFOKFORCEDONE_SF;
        output CFOKFSTARTED_SF;
        input CFOKRESET_FS;
        output CFOKSTART_SF;

        output CKOKDONE_SF;
        input CKOKRESET_FS;
        input CKPINRSRVD0_FS;
        input CKPINRSRVD1_FS;

        input COREREFCLK_FS;

        output CPLFBLOSS_SF;
        output CPLFREQLOCK_SF;
        input CPLLDMONCLK_FS;
        input CPLLFREQLOCK_FS;
        input CPLLKDETEN_FS;
        input CPLPWRDN_FS;
        output CPLREFLOSS_SF;
        input CPLREFSELDYN_FS[3];
        input CPLRESET_FS;

        input CTL_RX_PAUSE_ACK_FS[9];
        input CTL_TX_PAUSE_REQ_FS[9];
        input CTL_TX_RESEND_PAUSE_FS;
        input CTL_TX_SEND_IDLE_FS;
        input CTL_TX_SEND_LFI_FS;
        input CTL_TX_SEND_RFI_FS;

        input DFECFOKCFNUM_FS[4];
        input DFECFOKFEN_FS;
        input DFECFOKFPULSE_FS;
        input DFECFOKHOLD_FS;
        input DFECFOKOVREN_FS;
        input DFEDOUTMODE_FS[2];
        output DFEDOUT_SF[8];
        input DFEGCHOLD_FS;
        input DFEGCOVREN_FS;
        input DFEH2HOLD_FS;
        input DFEH2OVREN_FS;
        input DFEH3HOLD_FS;
        input DFEH3OVREN_FS;
        input DFEH4HOLD_FS;
        input DFEH4OVREN_FS;
        input DFEH5HOLD_FS;
        input DFEH5OVREN_FS;
        input DFEH6HOLD_FS;
        input DFEH6OVREN_FS;
        input DFEH7HOLD_FS;
        input DFEH7OVREN_FS;
        input DFEH8HOLD_FS;
        input DFEH8OVREN_FS;
        input DFEH9HOLD_FS;
        input DFEH9OVREN_FS;
        input DFEH10HOLD_FS;
        input DFEH10OVREN_FS;
        input DFEH11HOLD_FS;
        input DFEH11OVREN_FS;
        input DFEH12HOLD_FS;
        input DFEH12OVREN_FS;
        input DFEH13HOLD_FS;
        input DFEH13OVREN_FS;
        input DFEH14HOLD_FS;
        input DFEH14OVREN_FS;
        input DFEH15HOLD_FS;
        input DFEH15OVREN_FS;
        input DFEKLHOLD_FS;
        input DFEKLOVREN_FS;
        input DFERESET_FS;
        input DFEUTHOLD_FS;
        input DFEUTOVREN_FS;
        input DFEVPHOLD_FS;
        input DFEVPOVREN_FS;
        input DFEYEN_FS;
        input DFE_KH_EXTHOLD_FS;
        input DFE_KH_OVERWREN_FS;

        input DMONCLK_FS;
        input DMONFIFORESET_FS;
        output DMONOUTCLK_INT;
        output DMONOUT_SF[16];

        input ENPPM_FS;
        output EYESCANDATAERROR_SF;
        input EYESCANTRIGGER_FS;
        input FREQOS_FS;
        input GATERXELECIDLE_FS0;
        input GATERXELECIDLE_FS1;
        output GTPOWERGOOD_SF;
        input GTRXRSTSEL_FS;
        input GTRXRST_FS;
        input GTTXRSTSEL_FS;
        input GTTXRST_FS;
        input INCPCTRL_FS;
        input ISCANRESET_FS;
        input LOOPBACK_FS[3];

        input LPMGCHOLD_FS;
        input LPMGCOVREN_FS;
        input LPMKHHOLD_FS;
        input LPMKHOVREN_FS;
        input LPMKLHOLD_FS;
        input LPMKLOVREN_FS;
        input LPMOSHOLD_FS;
        input LPMOSOVREN_FS;

        output MGTREFCLKFA_SF;
        input OSHOLD_FS;
        input OSOVREN_FS;

        input PCS_RSVD_IN_FS[16];
        output PCS_RSVD_OUT_SF[16];
        output PINRSRVDAS_SF[16];
        input PINRSRVD_FS[16];

        input PMASCANCLK0_FS;
        input PMASCANCLK1_FS;
        input PMASCANCLK2_FS;
        input PMASCANCLK3_FS;
        input PMASCANCLK4_FS;
        input PMASCANCLK5_FS;
        input PMASCANCLK6_FS;
        input PMASCANCLK7_FS;
        input PMASCANCLK8_FS;
        input PMASCANRSTEN_FS;
        input PMASCANENB_FS;
        input PMASCANMODEB_FS;
        input PMASCANIN_FS[18];
        output PMASCANOUT_SF[18];

        input PRBSCNTRST_FS;
        input QPLLFREQLOCK0_FS;
        input QPLLFREQLOCK1_FS;
        input RCODAPWDN_FS;
        input RCODARESET_FS;
        output RDASOFTRSTDONE_SF;
        input RESETOVRD_FS;
        output RESET_EXCEPTION_SF;

        output RXCDRPHDONE_SF;
        input RXCKCALSTART_FS[7];
        input RXCOALGNEN_FS;
        output RXCOPHDONE_SF;
        input RXCOSETPHS_FS;
        input RXDAALGNEN_FS;
        input RXDABYPASS_FS;
        input RXDAOVREN_FS;
        input RXDASOFTRESET_FS;
        output RXELECIDLE_SF;
        output RXLINKSYNCDONE_SF;
        input RXLPMEN_FS;
        input RXMSTRSETPHDONE_FS;
        input RXOUTCKCTL_FS[3];
        output RXPHALIGNERR_SF;
        input RXPLLCKSEL_FS[2];
        output RXPMARESETDONE_SF;
        input RXPMARESET_FS;
        input RXPOLARITY_FS;
        output RXPRBSERR_SF;
        output RXPRBSLOCKED_SF;
        input RXPRBSPTN_FS[4];
        output RXPRGDIVRSTDONE_SF;
        input RXPROGDIVRESET_FS;
        input RXPWRDN_FS[2];
        output RXRAWDATA_SF[40];
        output RXRECCLKPCS_SF;
        output RXRECCLK_INT;
        output RXRECCLK_SF;
        output RXRESETDONE_SF;
        input RXRESET_FS;
        output RXSLIPDONE_SF;
        output RXSLIPOUTCLKRDY_SF;
        input RXSLIPOUTCLK_FS;
        output RXSLIPPMARDY_SF;
        input RXSLIPPMA_FS;
        input RXSLVSYNCEN_FS;
        output RXSYNCEN2SLV_SF;
        input RXSYNCFSMMASTER_FS;
        input RXSYSCKSEL_FS[2];
        input RXTERMINATION_FS;
        input RXUSRCLK2_FS;
        input RXUSRCLK_FS;
        input RXUSRRDY_FS;

        output RX_AXIS_TVALID_SF;
        output RX_AXIS_TDATA_SF[64];
        output RX_AXIS_TERR_SF;
        output RX_AXIS_TLAST_SF[8];
        output RX_AXIS_TPRE_SF[8];
        output RX_AXIS_TSOF_SF[2];
        output RX_AXIS_TTERM_SF[5];

        output RX_BITSLIP_SF;
        output RX_GB_SEQ_START_SF;
        output RX_PTP_SOP_POS_SF;
        output RX_PTP_SOP_SF;

        input SCANCLK_FS;
        input SCANRSTB_FS;
        input SCANRSTEN_FS;
        input SCANENB_FS;
        input SCANMODEB_FS;
        input SCANIN_FS[19];
        output SCANOUT_SF[19];

        output STAT_RX_BAD_CODE_SF;
        output STAT_RX_BAD_FCS_SF;
        output STAT_RX_BAD_PREAMBLE_SF;
        output STAT_RX_BAD_SFD_SF;
        output STAT_RX_BLOCK_LOCK_SF;
        output STAT_RX_BROADCAST_SF;
        output STAT_RX_BYTES_SF[4];
        output STAT_RX_FCS_ERR_SF;
        output STAT_RX_FRAMING_ERR_SF;
        output STAT_RX_GOT_SIGNAL_OS_SF;
        output STAT_RX_HI_BER_SF;
        output STAT_RX_INRANGEERR_SF;
        output STAT_RX_INTERNAL_LOCAL_FAULT_SF;
        output STAT_RX_LOCAL_FAULT_SF;
        output STAT_RX_MULTICAST_SF;
        output STAT_RX_PAUSE_QUANTA_SF[9];
        output STAT_RX_PAUSE_REQ_SF[9];
        output STAT_RX_PAUSE_VALID_SF[9];
        output STAT_RX_PKT_ERR_SF;
        output STAT_RX_PKT_SF;
        output STAT_RX_RECEIVED_LOCAL_FAULT_SF;
        output STAT_RX_REMOTE_FAULT_SF;
        output STAT_RX_STATUS_SF;
        output STAT_RX_STOMPED_FCS_SF;
        output STAT_RX_TEST_PATTERN_MISMATCH_SF;
        output STAT_RX_TRUNCATED_SF;
        output STAT_RX_UNICAST_SF;
        output STAT_RX_VALID_CTRL_CODE_SF;
        output STAT_RX_VLAN_SF;
        output STAT_TX_BAD_FCS_SF;
        output STAT_TX_BROADCAST_SF;
        output STAT_TX_BYTES_SF[4];
        output STAT_TX_FCS_ERR_SF;
        output STAT_TX_MULTICAST_SF;
        output STAT_TX_PAUSE_VALID_SF[9];
        output STAT_TX_PKT_ERR_SF;
        output STAT_TX_PKT_SF;
        output STAT_TX_UNICAST_SF;
        output STAT_TX_VLAN_SF;

        input STEPSIZEPPM_FS[5];

        input TCOCLKFSMFROUT_FS;
        input TCODABYPASS_FS;
        input TCODAOVREN_FS;
        input TCODAPWDN_FS;
        input TCODARESET_FS;
        input TCOHOLDFROUT_FS;
        output TCOINITDONE_SF;
        input TCOINITSET_FS;
        input TCOPIOVREN_FS;
        input TCOUPDNFROUT_FS;

        input TDASOFTRESET_FS;
        output TDASOFTRSTDONE_SF;

        input TSTCLK0_FS;
        input TSTCLK1_FS;
        input TSTIN_FS[20];
        input TSTPWRDNOVRDB_FS;
        input TSTPWRDN_FS[5];

        input TXCODAALGNEN_FS;
        input TXCOPIALGNEN_FS;
        output TXCOPIPHDONE_SF;
        input TXCOPISETPHS_FS;
        output TXDCCDONE_SF;
        input TXDCCSRESET_FS;
        input TXDCCSTART_FS;
        input TXDRVAMP_FS[5];
        input TXELECIDLE_FS;
        input TXEMPMAIN_FS[7];
        input TXEMPPOS_FS[5];
        input TXEMPPRE_FS[5];
        output TXLINKSYNCDONE_SF;
        input TXMSTRSETPHDONE_FS;
        input TXMUXDCDEXHOLD_FS;
        input TXMUXDCDORWREN_FS;
        input TXOUTCKCTL_FS[3];
        output TXOUTCLKPCS_SF;
        output TXOUTCLK_INT;
        output TXOUTCLK_SF;
        input TXPIPPMPWDN_FS;
        input TXPLLCKSEL_FS[2];
        output TXPMARESETDONE_SF;
        input TXPMARESET_FS;
        input TXPOLARITY_FS;
        input TXPPMOVRDEN_FS;
        input TXPPMSEL_FS;
        input TXPRBSINERR_FS;
        input TXPRBSPTN_FS[4];
        output TXPRGDIVRSTDONE_SF;
        input TXPROGDIVRESET_FS;
        input TXPWRDN_FS[2];
        input TXRAWDATA_FS[40];
        output TXRESETDONE_SF;
        input TXRESET_FS;
        input TXSERPWRDN_FS;
        input TXSLVSYNCEN_FS;
        output TXSYNCEN2SLV_SF;
        input TXSYNCFSMMASTER_FS;
        input TXSYSCKSEL_FS[2];
        input TXUSRCLK2_FS;
        input TXUSRCLK_FS;
        input TXUSRRDY_FS;

        output TX_AXIS_TREADY_SF;
        input TX_AXIS_TVALID_FS;
        input TX_AXIS_TDATA_FS[64];
        input TX_AXIS_TERR_FS;
        input TX_AXIS_TLAST_FS[8];
        input TX_AXIS_TPOISON_FS;
        input TX_AXIS_TPRE_FS[8];
        input TX_AXIS_TSOF_FS[2];
        input TX_AXIS_TTERM_FS[5];

        output TX_GB_SEQ_START_SF;
        input TX_GB_SEQ_SYNC_FS;
        output TX_PTP_SOP_POS_SF;
        output TX_PTP_SOP_SF;
        output TX_UNFOUT_SF;
    }

    bel_class GTM_DUAL {
        input COREREFCLK_FS;
        output CLK_BUFGT_CLK_IN_BOT[6];
        output CLK_BUFGT_CLK_IN_TOP[6];
        output MGTREFCLKFA_SF;
        input HROW_TEST_CK_FS;

        input DCLK_FS;
        input DRSTB_FS;
        input DEN_FS;
        input DWE_FS;
        input DADDR_FS[11];
        input DI_FS[16];
        output DRDY_SF;
        output DRPDO_SF[16];

        input BGBYPASSB_FS;
        input BGPWRDNB_FS;
        input BGRCALCTL_FS[5];
        input BGRCALOVRDENB_FS;
        input BGTESTEN_FS;
        output GTPOWERGOOD_SF;

        input PLLRESET_MASK_FS[2];
        input PLL_FBDIV_FS[8];
        output PLL_FBLOSS_SF;
        output PLL_FREQLOCK_SF;
        input PLL_MONCLK_FS;
        input PLL_PWRDN_FS;
        input PLL_REFDYN_FS[3];
        output PLL_REFLOSS_SF;
        output PLL_RESETDONE_SF;
        input PLL_RESET_FS;

        input RCALENB_FS;
        output RCAL_CMP_SF;
        output RCAL_SF[5];

        input REFCLKPD_FS;
        input RESET_BYPASS_MODE_FS;

        input SDM_DATA_FS[26];
        input SDM_TOGGLE_FS;

        for i in 0..2 {
            input "CTRL_RX{i}_BITSLIP_FS";
            output "ST_RX{i}_CORR_CW_INC_SF";
            output "ST_RX{i}_CW_INC_SF";
            output "ST_RX{i}_FEC_ALIGNED_SF";
            output "ST_RX{i}_UNCORR_CW_INC_SF";
        }
        for i in 0..4 {
            output "ST_RX_LN{i}_BIT_ERR_0TO1_INC_SF"[8];
            output "ST_RX_LN{i}_BIT_ERR_1TO0_INC_SF"[8];
            output "ST_RX_LN{i}_DELAY_SF"[15];
            output "ST_RX_LN{i}_ERR_CNT_INC_SF"[4];
            output "ST_RX_LN{i}_FEC_LOCK_SF";
            output "ST_RX_LN{i}_MAPPING_SF"[2];
        }

        for ch in 0..2 {
            input "AXIS_EYESCANRST_CH{ch}_FS";
            input "AXIS_STREAMEN_CH{ch}_FS";
            input "AXIS_TREADY_CH{ch}_FS";
            output "AXIS_TVALID_CH{ch}_SF";
            output "AXIS_TDATA_CH{ch}_SF"[28];
            output "AXIS_TLAST_CH{ch}_SF";
            input "BSR_SERIAL_CH{ch}_FS";
            input "CDR_EXHOLD_CH{ch}_FS";
            input "CDR_FREQOS_CH{ch}_FS";
            input "CDR_INCPCTRL_CH{ch}_FS";
            input "CDR_OVWREN_CH{ch}_FS";
            input "CFGRESET_CH{ch}_FS";
            input "DMONCLK_CH{ch}_FS";
            input "DMONFIFORESET_CH{ch}_FS";
            output "DMONOUT_CH{ch}_SF"[32];
            input "GTRXRESET_CH{ch}_FS";
            input "GTTXRESET_CH{ch}_FS";
            input "LOOPBACK_CH{ch}_FS"[3];
            input "RESETOVRD_CH{ch}_FS";
            output "RESET_EXCEPTION_CH{ch}_SF";
            output "RXBUFSTATUS_CH{ch}_SF"[3];
            output "RXDATA_CH{ch}_SF"[256];
            output "RXDATA_FLAGS_CH{ch}_SF"[4];
            output "RXDATA_IS_AM_CH{ch}_SF";
            output "RXDATA_START_CH{ch}_SF";
            input "RXEQTRAINING_CH{ch}_FS";
            input "RXOUTCKCTL_CH{ch}_FS"[3];
            output "RXPMARESETDONE_CH{ch}_SF";
            input "RXPOLARITY_CH{ch}_FS";
            input "RXPRBSCNTRST_CH{ch}_FS";
            output "RXPRBSERR_CH{ch}_SF";
            output "RXPRBSLOCKED_CH{ch}_SF";
            input "RXPRBSPTN_CH{ch}_FS"[4];
            input "RXPRBS_CNT_STOP_CH{ch}_FS";
            output "RXPROGDIVRESETDONE_CH{ch}_SF";
            input "RXQPRBS_ENA_CH{ch}_FS";
            output "RXRESETDONE_CH{ch}_SF";
            input "RXUSRCLK2_CH{ch}_FS";
            input "RXUSRCLK_CH{ch}_FS";
            input "RXUSRRDY_CH{ch}_FS";
            input "RX_ADAPTRESET_CH{ch}_FS";
            input "RX_ADC_CALRESET_CH{ch}_FS";
            input "RX_ADC_CLKGENRESET_CH{ch}_FS";
            input "RX_BUFRESET_CH{ch}_FS";
            input "RX_CDRFRRESET_CH{ch}_FS";
            input "RX_CDRPHRESET_CH{ch}_FS";
            input "RX_DFERESET_CH{ch}_FS";
            input "RX_DSPRESET_CH{ch}_FS";
            input "RX_FECRESET_CH{ch}_FS";
            input "RX_ISCANRESET_CH{ch}_FS";
            input "RX_PCSRESET_CH{ch}_FS";
            input "RX_PCS_RESETMASK_CH{ch}_FS"[4];
            input "RX_PCS_SEQ_ADVANCE_CH{ch}_FS";
            input "RX_PMARESET_CH{ch}_FS";
            input "RX_PMA_RESETMASK_CH{ch}_FS"[8];
            input "RX_PROGDIVRESET_CH{ch}_FS";
            input "RX_RESETMODE_CH{ch}_FS"[2];
            input "RX_USRSTART_CH{ch}_FS";
            input "RX_USRSTOP_CH{ch}_FS";
            input "TSTCLK0_CH{ch}_FS";
            input "TSTCLK1_CH{ch}_FS";
            output "TXBUFSTATUS_CH{ch}_SF"[2];
            input "TXCTLFIRDAT_CH{ch}_FS"[6];
            input "TXDATA_CH{ch}_FS"[256];
            input "TXDATA_START_CH{ch}_FS";
            input "TXDRVAMP_CH{ch}_FS"[5];
            input "TXEMPMAIN_CH{ch}_FS"[6];
            input "TXEMPPOS_CH{ch}_FS"[5];
            input "TXEMPPRE2_CH{ch}_FS"[4];
            input "TXEMPPRE_CH{ch}_FS"[5];
            input "TXINHIBIT_CH{ch}_FS";
            input "TXMUXDCDEXHOLD_CH{ch}_FS";
            input "TXMUXDCDORWREN_CH{ch}_FS";
            input "TXOUTCKCTL_CH{ch}_FS"[3];
            output "TXPMARESETDONE_CH{ch}_SF";
            input "TXPOLARITY_CH{ch}_FS";
            input "TXPRBSINERR_CH{ch}_FS";
            input "TXPRBSPTN_CH{ch}_FS"[4];
            output "TXPROGDIVRESETDONE_CH{ch}_SF";
            input "TXQPRBS_ENA_CH{ch}_FS";
            output "TXRESETDONE_CH{ch}_SF";
            input "TXUSRCLK2_CH{ch}_FS";
            input "TXUSRCLK_CH{ch}_FS";
            input "TXUSRRDY_CH{ch}_FS";
            input "TX_CKALRESET_CH{ch}_FS";
            input "TX_FECRESET_CH{ch}_FS";
            input "TX_PCSRESET_CH{ch}_FS";
            input "TX_PCS_RESETMASK_CH{ch}_FS"[2];
            input "TX_PCS_SEQ_ADVANCE_CH{ch}_FS";
            input "TX_PMARESET_CH{ch}_FS";
            input "TX_PMA_RESETMASK_CH{ch}_FS"[2];
            input "TX_PROGDIVRESET_CH{ch}_FS";
            input "TX_RESETMODE_CH{ch}_FS"[2];

            input "PCS_RSVDIN_CH{ch}_FS"[16];
            output "PCS_RSVDOUT_CH{ch}_SF"[16];
            input "PCS_SCANCLK_CH{ch}_FS"[2];
            input "PCS_SCANRSTB_CH{ch}_FS";
            input "PCS_SCANRSTEN_CH{ch}_FS";
            input "PCS_SCANENB_CH{ch}_FS";
            input "PCS_SCANMODEB_CH{ch}_FS";
            input "PCS_SCANIN_CH{ch}_FS"[15];
            output "PCS_SCANOUT_CH{ch}_SF"[15];
            input "PMA_RSVDIN_CH{ch}_FS"[16];
            output "PMA_RSVDOUT_CH{ch}_SF"[16];
            input "PMA_SCANCLK_CH{ch}_FS"[9];
            input "PMA_SCANRSTEN_CH{ch}_FS";
            input "PMA_SCANENB_CH{ch}_FS";
            input "PMA_SCANMODEB_CH{ch}_FS";
            input "PMA_SCANIN_CH{ch}_FS"[25];
            output "PMA_SCANOUT_CH{ch}_SF"[25];
        }

        input FEC_SCANCLK_FS;
        input FEC_SCANRSTB_FS;
        input FEC_SCANENB_FS;
        input FEC_SCANMODEB_FS;
        input FEC_SCANIN_FS[200];
        output FEC_SCANOUT_SF[200];

        input PLL_RSVDIN_FS[16];
        output PLL_RSVDOUT_SF[16];
        input PLL_SCANCLK_FS[4];
        input PLL_SCANRSTEN_FS;
        input PLL_SCANENB_FS;
        input PLL_SCANMODEB_FS;
        input PLL_SCANIN_FS[8];
        output PLL_SCANOUT_SF[8];
    }

    bel_class HSDAC {
        output CLK_DAC;
        output CLK_DAC_SPARE;
        input CLK_FIFO_LM;
        input FABRIC_CLK;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[12];
        input DI[16];
        output DRDY;
        output DOUT[16];

        input CONTROL_COMMON[16];
        input CONTROL_DAC0[16];
        input CONTROL_DAC1[16];
        input CONTROL_DAC2[16];
        input CONTROL_DAC3[16];
        // HSDAC has only 16 instead of 24 bits for each of these 5
        output STATUS_COMMON[24];
        output STATUS_DAC0[24];
        output STATUS_DAC1[24];
        output STATUS_DAC2[24];
        output STATUS_DAC3[24];

        input DATA_DAC0[256];
        input DATA_DAC1[256];
        input DATA_DAC2[256];
        input DATA_DAC3[256];

        output PLL_DMON_OUT;
        input PLL_MONCLK;
        input PLL_REFCLK_IN_FABRIC;
        output PLL_REFCLK_OUT;

        input PLL_SCAN_CLK_FD[2];
        input PLL_SCAN_RST_EN_FD;
        input PLL_SCAN_EN_B_FD;
        input PLL_SCAN_MODE_B_FD;
        input PLL_SCAN_IN_FD[2];
        output PLL_SCAN_OUT_B_FD[2];

        input TEST_SCAN_CLK[5];
        input TEST_SCAN_RESET;
        input TEST_SCAN_MODE_B;
        input TEST_SCAN_CTRL[16];
        input TEST_SE_B;
        input TEST_SI[300];
        output TEST_SO[300];
        output TEST_STATUS[16];
    }

    bel_class HSADC {
        output CLK_ADC;
        output CLK_ADC_SPARE;
        input CLK_FIFO_LM;
        input FABRIC_CLK;

        input DCLK;
        input DEN;
        input DWE;
        input DADDR[12];
        input DI[16];
        output DRDY;
        output DOUT[16];

        input CONTROL_COMMON[16];
        input CONTROL_ADC0[16];
        input CONTROL_ADC1[16];
        input CONTROL_ADC2[16];
        input CONTROL_ADC3[16];
        // HSADC has only 16 instead of 24 bits for each of these 5
        output STATUS_COMMON[24];
        output STATUS_ADC0[24];
        output STATUS_ADC1[24];
        output STATUS_ADC2[24];
        output STATUS_ADC3[24];

        // only 128 bits each on HSADC
        output DATA_ADC0[192];
        output DATA_ADC1[192];
        output DATA_ADC2[192];
        output DATA_ADC3[192];

        output PLL_DMON_OUT;
        input PLL_MONCLK;
        input PLL_REFCLK_IN_FABRIC;
        output PLL_REFCLK_OUT;

        input PLL_SCAN_CLK_FD[2];
        input PLL_SCAN_RST_EN_FD;
        input PLL_SCAN_EN_B_FD;
        input PLL_SCAN_MODE_B_FD;
        input PLL_SCAN_IN_FD[2];
        output PLL_SCAN_OUT_B_FD[2];

        input TEST_SCAN_CLK[5];
        input TEST_SCAN_RESET;
        input TEST_SCAN_MODE_B;
        input TEST_SCAN_CTRL[16];
        input TEST_SE_B;
        input TEST_SI[300];
        output TEST_SO[300];
        output TEST_STATUS[16];
    }

    region_slot LEAF;
    region_slot HDISTR;
    region_slot HROUTE;

    // wires common between two halves of the interconnect tile

    wire TIE_0: tie 0;
    wire TIE_1: tie 1;
    wire GCLK[16]: regional LEAF;
    wire GNODE[32]: mux;

    // X4/X5 wires
    if variant ultrascale {
        wire LX4_W0[16]: mux;
        for i in 1..=2 {
            wire "LX4_W{i}"[16]: branch LE;
        }

        wire LX4_E0[16]: mux;
        for i in 1..=2 {
            wire "LX4_E{i}"[16]: branch LW;
        }
        wire LX4_E2_S0: branch N;
        wire LX4_E2_N15: branch S;

        wire LX4_S0[8]: mux;
        for i in 1..=4 {
            wire "LX4_S{i}"[8]: branch N;
        }

        wire LX5_S0[8]: mux;
        for i in 1..=5 {
            wire "LX5_S{i}"[8]: branch N;
        }

        wire LX4_N0[8]: mux;
        for i in 1..=4 {
            wire "LX4_N{i}"[8]: branch S;
        }

        wire LX5_N0[8]: mux;
        for i in 1..=5 {
            wire "LX5_N{i}"[8]: branch S;
        }
        wire LX5_N5_N7: branch S;
    }

    // X12/X16 wires
    wire LX12_W0[8]: mux;
    for i in 1..=6 {
        wire "LX12_W{i}"[8]: branch LE;
    }
    if variant ultrascale {
        wire LX12_W6_N7: branch S;
    }

    wire LX12_E0[8]: mux;
    for i in 1..=6 {
        wire "LX12_E{i}"[8]: branch LW;
    }
    wire LX12_E6_S0: branch N;
    if variant ultrascaleplus {
        wire LX12_E6_N7: branch S;
    }

    if variant ultrascale {
        wire LX12_S0[4]: mux;
        for i in 1..=12 {
            wire "LX12_S{i}"[4]: branch N;
        }
        wire LX12_S12_S0: branch N;
        wire LX16_S0[4]: mux;
        for i in 1..=16 {
            wire "LX16_S{i}"[4]: branch N;
        }
    } else {
        wire LX12_S0[8]: mux;
        for i in 1..=12 {
            wire "LX12_S{i}"[8]: branch N;
        }
    }

    if variant ultrascale {
        wire LX12_N0[4]: mux;
        for i in 1..=12 {
            wire "LX12_N{i}"[4]: branch S;
        }
        wire LX16_N0[4]: mux;
        for i in 1..=16 {
            wire "LX16_N{i}"[4]: branch S;
        }
        wire LX16_N16_N3: branch S;
    } else {
        wire LX12_N0[8]: mux;
        for i in 1..=12 {
            wire "LX12_N{i}"[8]: branch S;
        }
    }

    // wires belonging to interconnect left/right half-tiles

    wire OUT[32]: bel;
    wire OUT_BEL[32]: bel;
    wire OUT_TEST[32]: test;
    wire TEST[4]: test;

    // intermediates for X1/X2/X4 muxes
    if variant ultrascale {
        wire SDNODE[64]: mux;
        // only a few actually exist
        wire SDNODE_S[64]: branch N;
        wire SDNODE_N[64]: branch S;
    } else {
        wire SDQNODE[96]: mux;
        // only a few actually exist
        wire SDQNODE_S[96]: branch N;
        wire SDQNODE_N[96]: branch S;
    }

    // X1 wires

    wire X1_W0[8]: mux;
    wire X1_W1[8]: branch E;
    if variant ultrascaleplus {
        wire X1_W1_W7: branch E;
        wire X1_W1_E7: branch W;
        wire X1_W1_N7: branch S;
    }

    wire X1_E0[8]: mux;
    wire X1_E1[8]: branch W;
    if variant ultrascale {
        wire X1_E1_W0: branch E;
        wire X1_E1_E0: branch W;
        wire X1_E1_S0: branch N;
    }

    wire X1_S0[8]: mux;
    wire X1_S1[8]: branch N;
    if variant ultrascale {
        wire X1_S1_S0: branch N;
    }

    wire X1_N0[8]: mux;
    wire X1_N1[8]: branch S;

    // X2 wires

    wire X2_W0[8]: mux;
    wire X2_W1[8]: branch E;
    wire X2_W2[8]: branch E;
    if variant ultrascaleplus {
        wire X2_W2_W0: branch E;
        wire X2_W2_E0: branch W;
        wire X2_W2_S0: branch N;
    }

    wire X2_E0[8]: mux;
    wire X2_E1[8]: branch W;
    wire X2_E2[8]: branch W;
    if variant ultrascale {
        wire X2_E2_W7: branch E;
        wire X2_E2_E7: branch W;
        wire X2_E2_N7: branch S;
    }

    wire X2_S0[8]: mux;
    wire X2_S1[8]: branch N;
    wire X2_S2[8]: branch N;

    wire X2_N0[8]: mux;
    wire X2_N1[8]: branch S;
    wire X2_N2[8]: branch S;
    if variant ultrascale {
        wire X2_N2_N7: branch S;
    }

    // X4 wires

    if variant ultrascaleplus {
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
        wire X4_S4_S0: branch N;

        wire X4_N0[8]: mux;
        for i in 1..=4 {
            wire "X4_N{i}"[8]: branch S;
        }
        wire X4_N4_N7: branch S;
    }

    // intermediates for X4/X12 muxes (u+ doesn't use intermediates for X12)
    if variant ultrascale {
        wire QLNODE[64]: mux;
        // only a few actually exist
        wire QLNODE_S[64]: branch N;
        wire QLNODE_N[64]: branch S;
    }

    wire INODE[64]: mux;
    // only a few actually exist
    wire INODE_S[64]: branch N;
    wire INODE_N[64]: branch S;

    wire IMUX_CTRL[10]: mux;
    wire IMUX_BYP[16]: mux;
    // only a few actually exist
    wire IMUX_BYP_S[16]: branch N;
    wire IMUX_BYP_N[16]: branch S;
    if variant ultrascaleplus {
        wire IMUX_BYP_DELAY[16]: mux;
    }
    wire IMUX_IMUX[48]: mux;
    wire IMUX_IMUX_DELAY[48]: mux;
    wire IMUX_RCLK[24]: mux;
    wire INODE_RCLK[24]: mux;
    wire RCLK_GND[24]: tie 0;

    // clock routing wires
    wire GCLK_HDISTR[24]: regional HDISTR;
    wire GCLK_HDISTR_MUX[24]: mux;
    wire GCLK_HDISTR_FAR[24]: multi_branch GCLK_H;
    wire GCLK_HDISTR_FAR_MUX[24]: mux;
    wire GCLK_HDISTR_CMT[24]: mux;
    wire GCLK_HDISTR_CMT_MUX[24]: mux;

    wire GCLK_HROUTE[24]: regional HROUTE;
    wire GCLK_HROUTE_MUX[24]: mux;
    wire GCLK_HROUTE_FAR[24]: multi_branch GCLK_H;
    wire GCLK_HROUTE_FAR_MUX[24]: mux;
    wire GCLK_HROUTE_HDIOL_VOID[24]: mux;
    wire GCLK_HROUTE_HDIOL_W_MUX[24]: mux;
    wire GCLK_HROUTE_HDIOL_E_MUX[24]: mux;

    wire GCLK_VDISTR_S[24]: multi_branch GCLK_S;
    wire GCLK_VDISTR_N[24]: multi_root;
    wire GCLK_VDISTR_S_MUX[24]: mux;
    wire GCLK_VDISTR_N_MUX[24]: mux;
    wire GCLK_VDISTR_S_BUF[24]: mux;
    wire GCLK_VDISTR_N_BUF[24]: mux;

    wire GCLK_VROUTE_S[4]: multi_branch GCLK_S;
    wire GCLK_VROUTE_N[4]: multi_root;
    wire GCLK_VROUTE_S_MUX[4]: mux;
    wire GCLK_VROUTE_N_MUX[4]: mux;
    wire GCLK_VROUTE_S_BUF[4]: mux;
    wire GCLK_VROUTE_N_BUF[4]: mux;

    wire IMUX_BUFCE_LEAF[16]: mux;

    wire OUT_IOB_CLKPAD[4]: bel;
    wire OUT_PHY_CLK[2]: bel;

    // clock routing — HDIO
    wire CCIO_HDIO[4]: mux;
    wire IMUX_BUFGCE_HDIO[4]: mux;
    wire IMUX_BUFGCE_HDIO_CCIO[4]: mux;
    wire OUT_BUFGCE_HDIO[4]: bel;
    wire OMUX_BUFG_HDIO[24]: mux;

    // clock routing — RCLK_BEL
    wire GCLK_HDISTR_RCLK[4]: multi_branch RCLK;
    wire GCLK_HROUTE_RCLK[4]: multi_branch RCLK;
    wire GCLK_HROUTE_RCLK_MUX[4]: mux;
    wire GCLK_HDISTR_TEST_RCLK[4]: bel;
    wire IMUX_BUFCE_ROW_RCLK[4]: mux;
    wire OUT_BUFCE_ROW_RCLK[4]: bel;

    // clock routing — CMT
    wire CCIO_CMT[4]: branch RCLK;
    wire OMUX_HDISTR_CMT[24]: mux;
    wire GCLK_HDISTR_TEST_CMT[24]: bel;
    wire GCLK_HDISTR_TEST_CMT_IN[24]: mux;
    wire IMUX_BUFCE_ROW_CMT[24]: mux;
    wire OUT_BUFCE_ROW_CMT[24]: bel;
    wire OUT_BUFCE_ROW_CMT_DLY[24]: bel;

    wire IMUX_BUFG_CMT[24]: mux;
    wire IMUX_BUFG_CMT_HROUTE[24]: mux;
    wire IMUX_BUFG_CMT_PLL_CKINT[24]: mux;
    wire IMUX_BUFG_CMT_TEST[24]: mux;
    wire OUT_BUFGCE_CMT[24]: bel;
    wire IMUX_BUFGCTRL_I0[8]: mux;
    wire IMUX_BUFGCTRL_I1[8]: mux;
    wire OUT_BUFGCTRL[8]: bel;
    wire OUT_BUFGCE_DIV[4]: bel;
    wire OMUX_BUFG_CMT[24]: mux;
    wire OMUX_BUFG_CMT_DUMMY0[24]: mux;
    wire OMUX_BUFG_CMT_DUMMY1[24]: mux;
    wire OMUX_BUFG_CMT_DUMMY2[24]: mux;

    wire IMUX_MMCM_CLKIN1: mux;
    wire IMUX_MMCM_CLKIN2: mux;
    wire IMUX_MMCM_CLKFB: mux;
    wire IMUX_MMCM_CLKIN1_HDISTR: mux;
    wire IMUX_MMCM_CLKIN2_HDISTR: mux;
    wire IMUX_MMCM_CLKFB_HDISTR: mux;
    wire IMUX_MMCM_CLKIN1_HDISTR_DLY: mux;
    wire IMUX_MMCM_CLKIN2_HDISTR_DLY: mux;
    wire IMUX_MMCM_CLKFB_HDISTR_DLY: mux;
    wire IMUX_MMCM_CLKIN1_HROUTE: mux;
    wire IMUX_MMCM_CLKIN2_HROUTE: mux;
    wire IMUX_MMCM_CLKIN1_DUMMY0: mux;
    wire IMUX_MMCM_CLKIN2_DUMMY0: mux;
    wire IMUX_MMCM_CLKFB_DUMMY0: mux;
    wire IMUX_MMCM_CLKFB_DUMMY1: mux;
    wire OUT_MMCM[14]: bel;

    wire IMUX_PLL_CLKIN[2]: mux;
    wire IMUX_PLL_CLKFB[2]: mux;
    wire IMUX_PLL_CLKIN_HDISTR[2]: mux;
    wire IMUX_PLL_CLKFB_HDISTR[2]: mux;
    wire IMUX_PLL_CLKIN_HDISTR_DLY[2]: mux;
    wire IMUX_PLL_CLKFB_HDISTR_DLY[2]: mux;
    wire IMUX_PLL_CLKIN_HROUTE[2]: mux;
    wire IMUX_PLL_CLKIN_MMCM[2]: mux;
    wire OUT_PLL0[6]: bel;
    wire OUT_PLL1[6]: bel;

    wire GCLK_XIPHY[6]: regional LEAF;
    wire IMUX_BITSLICE_CLK[13]: mux;
    wire IMUX_BITSLICE_RX_CLK_C[13]: mux;
    wire IMUX_BITSLICE_RX_CLK_C_B[13]: mux;
    wire IMUX_BITSLICE_TX_OCLK[13]: mux;
    wire IMUX_BITSLICE_T_CLK[2]: mux;

    // clock routing — GT
    wire OUT_GT_CLK[14]: bel;
    wire OUT_BUFG_GT_SYNC_CE[15]: bel;
    wire OUT_BUFG_GT_SYNC_RST[15]: bel;
    wire IMUX_BUFG_GT_CLK[24]: mux;
    wire IMUX_BUFG_GT_RST[24]: mux;
    wire IMUX_BUFG_GT_CE[24]: mux;
    for i in 0..11 {
        wire "IMUX_BUFG_GT_CLK_DUMMY{i}"[24]: tie 1;
    }
    for i in 0..5 {
        wire "IMUX_BUFG_GT_RST_DUMMY{i}"[24]: tie 1;
        wire "IMUX_BUFG_GT_CE_DUMMY{i}"[24]: tie 1;
    }
    wire OUT_BUFG_GT[24]: bel;

    // clock routing — PS
    wire OUT_PS_CLK[18]: bel;
    wire IMUX_BUFG_PS[24]: mux;
    wire IMUX_BUFG_PS_DUMMY[24]: tie 1;
    wire OUT_BUFG_PS[24]: bel;

    tile_slot INT {
        bel_slot INT: routing;
        tile_class INT {
            cell CELL[2];
        }
    }

    tile_slot INTF {
        bel_slot INTF_INT: routing;
        bel_slot INTF_TESTMUX: routing;
        tile_class INTF {
            cell CELL;
        }
        tile_class INTF_DELAY {
            cell CELL;
        }

        if variant ultrascaleplus {
            tile_class INTF_IO {
                cell CELL;
            }
        }
    }

    tile_slot BEL {
        bel_slot SLICE: SLICE;
        tile_class CLEL {
            cell CELL;
        }
        tile_class CLEM {
            cell CELL;
        }

        bel_slot LAGUNA[4]: LAGUNA;
        tile_class LAGUNA {
            cell CELL;
        }

        bel_slot BRAM: BRAM;
        tile_class BRAM {
            cell CELL[5];
        }

        bel_slot DSP[2]: DSP;
        tile_class DSP {
            cell CELL[5];
        }

        if variant ultrascaleplus {
            bel_slot URAM[4]: URAM;
            tile_class URAM {
                cell CELL_W[15];
                cell CELL_E[15];
            }
        }

        if variant ultrascaleplus {
            bel_slot HDIOB[42]: HDIOB;
            bel_slot HDIOB_DIFF_IN[21]: HDIOB_DIFF_IN;
            bel_slot HDIOLOGIC[42]: HDIOLOGIC;
            bel_slot HDLOGIC_CSSD[4]: HDLOGIC_CSSD;
            bel_slot HDIO_VREF[3]: HDIO_VREF;
            bel_slot HDIO_BIAS: HDIO_BIAS;
            tile_class HDIO {
                cell CELL[60];
            }
            tile_class HDIOL_S {
                cell CELL[30];
            }
            tile_class HDIOL_N {
                cell CELL[30];
            }
            tile_class HDIOS {
                cell CELL_W[60];
                cell CELL_E[60];
            }
        }

        bel_slot XIPHY_INT: routing;
        bel_slot BITSLICE[13]: BITSLICE;
        bel_slot BITSLICE_T[2]: BITSLICE_T;
        bel_slot BITSLICE_CONTROL[2]: BITSLICE_CONTROL;
        bel_slot XIPHY_BYTE: XIPHY_BYTE;
        tile_class XIPHY {
            cell CELL[15];
        }

        if variant ultrascaleplus {
            bel_slot XP5IOB[66]: XP5IOB;
            bel_slot XP5IOB_PAIR[33]: XP5IOB_PAIR;
            bel_slot XP5IO_BITSLICE[66]: XP5IO_BITSLICE;
            bel_slot X5PHY[11]: X5PHY;
            bel_slot XP5IO_VREF[11]: XP5IO_VREF;
            bel_slot LPDDRMC: LPDDRMC;
            bel_slot ABUS_SWITCH_XP5IO[2]: ABUS_SWITCH;
            tile_class XP5IO {
                cell CELL[60];
            }
        }

        bel_slot CFG: CFG;
        bel_slot ABUS_SWITCH_CFG: ABUS_SWITCH;
        tile_class CFG, CFG_CSEC, CFG_CSEC_V2 {
            if variant ultrascale {
                if tile_class [CFG_CSEC, CFG_CSEC_V2] {
                    skip;
                }
            }
            cell CELL[60];
        }

        bel_slot PMV: PMV;
        bel_slot PMV2: PMV2;
        bel_slot PMVIOB: PMVIOB;
        bel_slot MTBF3: MTBF3;
        bel_slot CFGIO: CFGIO;
        tile_class CFGIO {
            cell CELL[30];
        }

        bel_slot SYSMON: SYSMON;
        tile_class AMS {
            cell CELL[30];
        }

        if variant ultrascale {
            bel_slot PCIE3: PCIE3;
            tile_class PCIE3 {
                cell CELL_W[60];
                cell CELL_E[60];
            }
        }
        if variant ultrascaleplus {
            bel_slot PCIE4: PCIE4;
            tile_class PCIE4, PCIE4C, PCIE4CE {
                cell CELL_W[60];
                cell CELL_E[60];
            }
        }
        bel_slot CMAC: CMAC;
        bel_slot ILKN: ILKN;
        tile_class CMAC, ILKN {
            cell CELL_W[60];
            cell CELL_E[60];
        }

        if variant ultrascaleplus {
            bel_slot FE: FE;
            tile_class FE {
                cell CELL[60];
            }
        }

        if variant ultrascaleplus {
            bel_slot DFE_FFT[2]: DFE_FFT;
            bel_slot DFE_PRACH: DFE_PRACH;
            bel_slot DFE_FIR[2]: DFE_FIR;
            bel_slot DFE_MATRIX_FIR[2]: DFE_MATRIX_FIR;
            bel_slot DFE_DUC_DDC[2]: DFE_DUC_DDC;
            bel_slot DFE_CFR[2]: DFE_CFR;
            bel_slot DFE_NL_FIR[3]: DFE_NL_FIR;
            bel_slot DFE_PQ_INTERP[4]: DFE_PQ_INTERP;
            bel_slot DFE_PQ_DECIM[5]: DFE_PQ_DECIM;
            tile_class DFE_A, DFE_C, DFE_D, DFE_E, DFE_F, DFE_G {
                cell CELL_W[60];
                cell CELL_E[60];
            }
            tile_class DFE_B {
                cell CELL[60];
            }
        }

        bel_slot GT_INT: routing;
        bel_slot BUFG_GT[24]: BUFG_GT;
        bel_slot BUFG_GT_SYNC[15]: BUFG_GT_SYNC;
        bel_slot ABUS_SWITCH_GT[5]: ABUS_SWITCH;
        if variant ultrascale {
            bel_slot GT3_COMMON: GT3_COMMON;
            bel_slot GT3_CHANNEL[4]: GT3_CHANNEL;
        }
        if variant ultrascaleplus {
            bel_slot GT4_COMMON: GT4_COMMON;
            bel_slot GT4_CHANNEL[4]: GT4_CHANNEL;
            bel_slot GTF_COMMON: GTF_COMMON;
            bel_slot GTF_CHANNEL[4]: GTF_CHANNEL;
            bel_slot GTM_DUAL: GTM_DUAL;
            bel_slot HSDAC: HSDAC;
            bel_slot HSADC: HSADC;
        }
        tile_class GTH, GTY {
            cell CELL[60];
        }
        if variant ultrascaleplus {
            tile_class GTF, GTM, HSDAC, HSADC, RFDAC, RFADC {
                cell CELL[60];
            }
        }

        if variant ultrascaleplus {
            bel_slot PS: PS;
            tile_class PS {
                cell CELL[180];
            }
            bel_slot VCU: VCU;
            tile_class VCU {
                cell CELL[60];
            }
        }

        if variant ultrascaleplus {
            bel_slot BLI_HBM_APB_INTF: BLI_HBM_APB_INTF;
            bel_slot BLI_HBM_AXI_INTF: BLI_HBM_AXI_INTF;
            tile_class HBM_BLI {
                cell CELL[15];
            }
        }
    }

    tile_slot CMT {
        bel_slot CMT_INT: routing;
        bel_slot BUFCE_ROW_CMT[24]: BUFCE_ROW;
        bel_slot GCLK_TEST_BUF_CMT[24]: GCLK_TEST_BUF;
        bel_slot BUFGCE[24]: BUFGCE;
        bel_slot BUFGCTRL[8]: BUFGCTRL;
        bel_slot BUFGCE_DIV[4]: BUFGCE_DIV;
        bel_slot PLL[2]: PLL;
        bel_slot MMCM: MMCM;
        bel_slot ABUS_SWITCH_CMT: ABUS_SWITCH;
        bel_slot HBM_REF_CLK[2]: HBM_REF_CLK;
        tile_class CMT {
            cell CELL[60];
        }
        if variant ultrascaleplus {
            tile_class CMT_HBM, CMTXP {
                cell CELL[60];
            }
        }

        if variant ultrascaleplus {
            bel_slot ABUS_SWITCH_HBM[8]: ABUS_SWITCH;
            tile_class HBM_ABUS_SWITCH {
            }
        }
    }

    tile_slot IOB {
        bel_slot HPIOB[26]: HPIOB;
        bel_slot HPIOB_DIFF_IN[12]: HPIOB_DIFF_IN;
        bel_slot HPIOB_DIFF_OUT[12]: HPIOB_DIFF_OUT;
        bel_slot HPIOB_DCI[2]: HPIOB_DCI;
        bel_slot HPIO_VREF: HPIO_VREF;
        bel_slot HPIO_BIAS: HPIO_BIAS;
        tile_class HPIO {
            cell CELL[30];
        }

        if variant ultrascale {
            bel_slot HRIOB[26]: HRIOB;
            bel_slot HRIOB_DIFF_IN[12]: HRIOB_DIFF_IN;
            bel_slot HRIOB_DIFF_OUT[12]: HRIOB_DIFF_OUT;
            tile_class HRIO {
                cell CELL[30];
            }
        }
    }

    tile_slot RCLK_INT {
        bel_slot RCLK_INT: routing;
        bel_slot BUFCE_LEAF_X16_S: BUFCE_LEAF_X16;
        bel_slot BUFCE_LEAF_X16_N: BUFCE_LEAF_X16;
        bel_slot BUFCE_LEAF_S[16]: BUFCE_LEAF;
        bel_slot BUFCE_LEAF_N[16]: BUFCE_LEAF;
        tile_class RCLK_INT {
            cell NW, NE, SW, SE;
        }
    }

    tile_slot RCLK_V {
        bel_slot RCLK_V_INT: routing;
        bel_slot BUFCE_ROW_RCLK[4]: BUFCE_ROW;
        bel_slot GCLK_TEST_BUF_RCLK[4]: GCLK_TEST_BUF;
        bel_slot VBUS_SWITCH[3]: VBUS_SWITCH;
        tile_class RCLK_V_SINGLE_CLE {
            cell CELL;
        }
        if variant ultrascaleplus {
            tile_class RCLK_V_SINGLE_LAG {
                cell CELL;
            }
        }
        if variant ultrascale {
            tile_class RCLK_V_DOUBLE_BRAM {
                cell CELL;
            }
        }
        tile_class RCLK_V_DOUBLE_DSP {
            cell CELL;
        }
        if variant ultrascaleplus {
            tile_class RCLK_V_QUAD_BRAM, RCLK_V_QUAD_URAM {
                cell CELL;
            }
        }
    }

    tile_slot RCLK_SPLITTER {
        bel_slot RCLK_SPLITTER: routing;
        tile_class RCLK_HROUTE_SPLITTER_HARD, RCLK_HROUTE_SPLITTER_CLE {
            cell CELL;
        }
        tile_class RCLK_SPLITTER {
            cell CELL;
        }
    }

    tile_slot RCLK_BEL {
        bel_slot RCLK_BEL_INT: routing;

        bel_slot HARD_SYNC[4]: HARD_SYNC;
        tile_class HARD_SYNC {
            cell CELL;
        }

        if variant ultrascaleplus {
            bel_slot BUFGCE_HDIO[4]: BUFGCE;
            bel_slot ABUS_SWITCH_HDIO[12]: ABUS_SWITCH;
            tile_class RCLK_HDIO, RCLK_HDIOS, RCLK_HDIOL, RCLK_HDIOL_CMT_W, RCLK_HDIOL_CMT_E {
                cell CELL[60];
            }
        }

        if variant ultrascaleplus {
            bel_slot BUFG_PS[24]: BUFG_PS;
            tile_class RCLK_PS {
                cell CELL;
            }
        }

        if variant ultrascaleplus {
            tile_class RCLK_XIPHY {
                cell N, S;
            }
        }
    }

    tile_slot RCLK_IOB {
        bel_slot RCLK_IOB_INT: routing;
        bel_slot ABUS_SWITCH_HPIO[7]: ABUS_SWITCH;
        bel_slot HPIO_ZMATCH: HPIO_ZMATCH;
        bel_slot HPIO_PRBS: HPIO_PRBS;
        tile_class RCLK_HPIO {
            cell CELL[60];
        }

        if variant ultrascale {
            bel_slot ABUS_SWITCH_HRIO[8]: ABUS_SWITCH;
            tile_class RCLK_HRIO {
                cell CELL[60];
            }
        }
    }

    connector_slot W {
        opposite E;

        connector_class PASS_W {
            if variant ultrascale {
                pass X1_E1 = X1_E0;
                pass X2_E1 = X2_E0;
                pass X2_E2 = X2_E1;
                pass X1_E1_E0 = X1_E1[0];
                pass X2_E2_E7 = X2_E2[7];
            } else {
                pass X1_E1 = X1_E0;
                pass X2_E1 = X2_E0;
                pass X2_E2 = X2_E1;
                pass X4_E1 = X4_E0;
                pass X4_E2 = X4_E1;
                pass X4_E3 = X4_E2;
                pass X4_E4 = X4_E3;
                pass X1_W1_E7 = X1_W1[7];
                pass X2_W2_E0 = X2_W2[0];
            }
        }
        connector_class IO_W;
        connector_class TERM_W {
            if variant ultrascale {
                reflect X1_E1 = X1_W0;
                // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAaaaaaaaaaaaaaaaaaaaa
                reflect X2_E1 = X2_W1;
                reflect X2_E2 = X2_W0;
            } else {
                reflect X1_E1 = X1_W0;
                reflect X2_E1 = X2_W0;
                reflect X2_E2 = X2_W1;
                reflect X4_E1 = X4_W0;
                reflect X4_E2 = X4_W1;
                reflect X4_E3 = X4_W2;
                reflect X4_E4 = X4_W3;
            }
        }
    }

    connector_slot E {
        opposite W;

        connector_class PASS_E {
            if variant ultrascale {
                pass X1_W1 = X1_W0;
                pass X1_E1_W0 = X1_E1[0];
                pass X2_W1 = X2_W0;
                pass X2_W2 = X2_W1;
                pass X2_E2_W7 = X2_E2[7];
            } else {
                pass X1_W1 = X1_W0;
                pass X2_W1 = X2_W0;
                pass X2_W2 = X2_W1;
                pass X4_W1 = X4_W0;
                pass X4_W2 = X4_W1;
                pass X4_W3 = X4_W2;
                pass X4_W4 = X4_W3;
                pass X1_W1_W7 = X1_W1[7];
                pass X2_W2_W0 = X2_W2[0];
            }
        }
        connector_class IO_E;
        connector_class TERM_E {
            if variant ultrascale {
                reflect X1_W1 = X1_E0;
                // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAaaaaaaaaaaaaaaaaaaaa
                reflect X2_W1 = X2_E1;
                reflect X2_W2 = X2_E0;
            } else {
                reflect X1_W1 = X1_E0;
                reflect X2_W1 = X2_E0;
                reflect X2_W2 = X2_E1;
                reflect X4_W1 = X4_E0;
                reflect X4_W2 = X4_E1;
                reflect X4_W3 = X4_E2;
                reflect X4_W4 = X4_E3;
            }
        }
    }

    connector_slot LW {
        opposite LE;

        connector_class PASS_LW {
            if variant ultrascale {
                for i in 0..2 {
                    pass "LX4_E{i+1}" = "LX4_E{i}";
                }
                for i in 0..6 {
                    pass "LX12_E{i+1}" = "LX12_E{i}";
                }
            } else {
                for i in 0..6 {
                    pass "LX12_E{i+1}" = "LX12_E{i}";
                }
            }
        }
        connector_class IO_LW;
        connector_class TERM_LW {
            if variant ultrascale {
                for i in 0..2 {
                    reflect "LX4_E{i+1}" = "LX4_W{i}";
                }
                for i in 0..6 {
                    reflect "LX12_E{i+1}" = "LX12_W{i}";
                }
            } else {
                for i in 0..6 {
                    reflect "LX12_E{i+1}" = "LX12_W{i}";
                }
            }
        }
    }

    connector_slot LE {
        opposite LW;

        connector_class PASS_LE {
            if variant ultrascale {
                for i in 0..2 {
                    pass "LX4_W{i+1}" = "LX4_W{i}";
                }
                for i in 0..6 {
                    pass "LX12_W{i+1}" = "LX12_W{i}";
                }
            } else {
                for i in 0..6 {
                    pass "LX12_W{i+1}" = "LX12_W{i}";
                }
            }
        }
        connector_class IO_LE;
        connector_class TERM_LE {
            if variant ultrascale {
                for i in 0..2 {
                    reflect "LX4_W{i+1}" = "LX4_E{i}";
                }
                for i in 0..6 {
                    reflect "LX12_W{i+1}" = "LX12_E{i}";
                }
            } else {
                for i in 0..6 {
                    reflect "LX12_W{i+1}" = "LX12_E{i}";
                }
            }
        }
    }

    connector_slot S {
        opposite N;

        connector_class PASS_S {
            if variant ultrascale {
                for i in 0..4 {
                    pass "LX4_N{i+1}" = "LX4_N{i}";
                }
                for i in 0..5 {
                    pass "LX5_N{i+1}" = "LX5_N{i}";
                }
                pass LX4_E2_N15 = LX4_E2[15];
                pass LX5_N5_N7 = LX5_N5[7];

                for i in 0..12 {
                    pass "LX12_N{i+1}" = "LX12_N{i}";
                }
                for i in 0..16 {
                    pass "LX16_N{i+1}" = "LX16_N{i}";
                }
                pass LX12_W6_N7 = LX12_W6[7];
                pass LX16_N16_N3 = LX16_N16[3];
            } else {
                for i in 0..12 {
                    pass "LX12_N{i+1}" = "LX12_N{i}";
                }
                pass LX12_E6_N7 = LX12_E6[7];
            }

            if variant ultrascale {
                pass SDNODE_N = SDNODE;

                pass X1_N1 = X1_N0;
                pass X2_N1 = X2_N0;
                pass X2_N2 = X2_N1;
                pass X2_E2_N7 = X2_E2[7];
                pass X2_N2_N7 = X2_N2[7];

                pass QLNODE_N = QLNODE;
            } else {
                pass SDQNODE_N = SDQNODE;

                pass X1_N1 = X1_N0;
                pass X2_N1 = X2_N0;
                pass X2_N2 = X2_N1;
                pass X4_N1 = X4_N0;
                pass X4_N2 = X4_N1;
                pass X4_N3 = X4_N2;
                pass X4_N4 = X4_N3;
                pass X1_W1_N7 = X1_W1[7];
                pass X4_N4_N7 = X4_N4[7];
            }
            pass INODE_N = INODE;
            pass IMUX_BYP_N = IMUX_BYP;
        }
        connector_class TERM_S0;
        connector_class TERM_S1;
    }

    connector_slot N {
        opposite S;

        connector_class PASS_N {
            if variant ultrascale {
                for i in 0..4 {
                    pass "LX4_S{i+1}" = "LX4_S{i}";
                }
                for i in 0..5 {
                    pass "LX5_S{i+1}" = "LX5_S{i}";
                }
                pass LX4_E2_S0 = LX4_E2[0];

                for i in 0..12 {
                    pass "LX12_S{i+1}" = "LX12_S{i}";
                }
                for i in 0..16 {
                    pass "LX16_S{i+1}" = "LX16_S{i}";
                }
                pass LX12_E6_S0 = LX12_E6[0];
                pass LX12_S12_S0 = LX12_S12[0];
            } else {
                for i in 0..12 {
                    pass "LX12_S{i+1}" = "LX12_S{i}";
                }
                pass LX12_E6_S0 = LX12_E6[0];
            }

            if variant ultrascale {
                pass SDNODE_S = SDNODE;

                pass X1_S1 = X1_S0;
                pass X2_S1 = X2_S0;
                pass X2_S2 = X2_S1;
                pass X1_E1_S0 = X1_E1[0];
                pass X1_S1_S0 = X1_S1[0];

                pass QLNODE_S = QLNODE;
            } else {
                pass SDQNODE_S = SDQNODE;

                pass X1_S1 = X1_S0;
                pass X2_S1 = X2_S0;
                pass X2_S2 = X2_S1;
                pass X4_S1 = X4_S0;
                pass X4_S2 = X4_S1;
                pass X4_S3 = X4_S2;
                pass X4_S4 = X4_S3;
                pass X2_W2_S0 = X2_W2[0];
                pass X4_S4_S0 = X4_S4[0];
            }
            pass INODE_S = INODE;
            pass IMUX_BYP_S = IMUX_BYP;
        }
        connector_class TERM_N0;
        connector_class TERM_N1;
    }

    connector_slot GCLK_S {
        opposite GCLK_N;
        connector_class GCLK_S {
            pass GCLK_VDISTR_S = GCLK_VDISTR_N;
            pass GCLK_VROUTE_S = GCLK_VROUTE_N;
        }
        connector_class GCLK_S_SLR1 {
            pass GCLK_VDISTR_S[0] = GCLK_VDISTR_N[0];
            pass GCLK_VROUTE_S[0] = GCLK_VROUTE_N[0];
        }
        connector_class GCLK_S_SLR2 {
            for i in 0..2 {
                pass GCLK_VDISTR_S[i] = GCLK_VDISTR_N[i];
                pass GCLK_VROUTE_S[i] = GCLK_VROUTE_N[i];
            }
        }
        connector_class GCLK_S_SLR3 {
            for i in 0..3 {
                pass GCLK_VDISTR_S[i] = GCLK_VDISTR_N[i];
                pass GCLK_VROUTE_S[i] = GCLK_VROUTE_N[i];
            }
        }
        connector_class GCLK_S_SLR4 {
            for i in 0..4 {
                pass GCLK_VDISTR_S[i] = GCLK_VDISTR_N[i];
                pass GCLK_VROUTE_S[i] = GCLK_VROUTE_N[i];
            }
        }
    }
    connector_slot GCLK_N {
        opposite GCLK_S;
        connector_class GCLK_N;
    }

    connector_slot GCLK_H {
        opposite GCLK_H_REV;
        connector_class GCLK_H {
            pass GCLK_HROUTE_FAR = GCLK_HROUTE;
            pass GCLK_HDISTR_FAR = GCLK_HDISTR;
        }
        connector_class GCLK_H_IO {
            if variant ultrascale {
                pass GCLK_HROUTE_FAR = GCLK_HROUTE_FAR;
                pass GCLK_HDISTR_FAR = GCLK_HDISTR_FAR;
            }
        }
    }

    connector_slot GCLK_H_REV {
        opposite GCLK_H;
        connector_class GCLK_H_REV;
    }

    connector_slot RCLK {
        opposite RCLK;

        for i in 0..24 {
            connector_class "RCLK1_{i}" {
                reflect GCLK_HDISTR_RCLK[0] = GCLK_HDISTR[i];
                reflect GCLK_HROUTE_RCLK[0] = GCLK_HROUTE[i];
            }
        }
        for i in 0..23 {
            connector_class "RCLK2_{i}" {
                reflect GCLK_HDISTR_RCLK[0] = GCLK_HDISTR[i];
                reflect GCLK_HROUTE_RCLK[0] = GCLK_HROUTE[i];
                reflect GCLK_HDISTR_RCLK[1] = GCLK_HDISTR[i + 1];
                reflect GCLK_HROUTE_RCLK[1] = GCLK_HROUTE[i + 1];
            }
        }
        for i in 0..22 {
            connector_class "RCLK3_{i}" {
                reflect GCLK_HDISTR_RCLK[0] = GCLK_HDISTR[i];
                reflect GCLK_HROUTE_RCLK[0] = GCLK_HROUTE[i];
                reflect GCLK_HDISTR_RCLK[1] = GCLK_HDISTR[i + 1];
                reflect GCLK_HROUTE_RCLK[1] = GCLK_HROUTE[i + 1];
                reflect GCLK_HDISTR_RCLK[2] = GCLK_HDISTR[i + 2];
                reflect GCLK_HROUTE_RCLK[2] = GCLK_HROUTE[i + 2];
            }
        }
        for i in 0..21 {
            connector_class "RCLK4_{i}" {
                reflect GCLK_HDISTR_RCLK[0] = GCLK_HDISTR[i];
                reflect GCLK_HROUTE_RCLK[0] = GCLK_HROUTE[i];
                reflect GCLK_HDISTR_RCLK[1] = GCLK_HDISTR[i + 1];
                reflect GCLK_HROUTE_RCLK[1] = GCLK_HROUTE[i + 1];
                reflect GCLK_HDISTR_RCLK[2] = GCLK_HDISTR[i + 2];
                reflect GCLK_HROUTE_RCLK[2] = GCLK_HROUTE[i + 2];
                reflect GCLK_HDISTR_RCLK[3] = GCLK_HDISTR[i + 3];
                reflect GCLK_HROUTE_RCLK[3] = GCLK_HROUTE[i + 3];
            }
        }

        connector_class RCLK_HDIO_CMT {
            reflect CCIO_CMT = CCIO_HDIO;
        }
    }
}

pub mod wiredata {
    pub mod ultrascale {
        use prjcombine_entity::id::EntityStaticRange;
        use prjcombine_interconnect::db::WireSlotId;

        use crate::defs::ultrascale::wires;

        pub const LX4_W: [EntityStaticRange<WireSlotId, 16>; 3] =
            [wires::LX4_W0, wires::LX4_W1, wires::LX4_W2];
        pub const LX4_E: [EntityStaticRange<WireSlotId, 16>; 3] =
            [wires::LX4_E0, wires::LX4_E1, wires::LX4_E2];
        pub const LX4_S: [EntityStaticRange<WireSlotId, 8>; 5] = [
            wires::LX4_S0,
            wires::LX4_S1,
            wires::LX4_S2,
            wires::LX4_S3,
            wires::LX4_S4,
        ];
        pub const LX5_S: [EntityStaticRange<WireSlotId, 8>; 6] = [
            wires::LX5_S0,
            wires::LX5_S1,
            wires::LX5_S2,
            wires::LX5_S3,
            wires::LX5_S4,
            wires::LX5_S5,
        ];
        pub const LX4_N: [EntityStaticRange<WireSlotId, 8>; 5] = [
            wires::LX4_N0,
            wires::LX4_N1,
            wires::LX4_N2,
            wires::LX4_N3,
            wires::LX4_N4,
        ];
        pub const LX5_N: [EntityStaticRange<WireSlotId, 8>; 6] = [
            wires::LX5_N0,
            wires::LX5_N1,
            wires::LX5_N2,
            wires::LX5_N3,
            wires::LX5_N4,
            wires::LX5_N5,
        ];

        pub const LX12_W: [EntityStaticRange<WireSlotId, 8>; 7] = [
            wires::LX12_W0,
            wires::LX12_W1,
            wires::LX12_W2,
            wires::LX12_W3,
            wires::LX12_W4,
            wires::LX12_W5,
            wires::LX12_W6,
        ];
        pub const LX12_E: [EntityStaticRange<WireSlotId, 8>; 7] = [
            wires::LX12_E0,
            wires::LX12_E1,
            wires::LX12_E2,
            wires::LX12_E3,
            wires::LX12_E4,
            wires::LX12_E5,
            wires::LX12_E6,
        ];
        pub const LX12_S: [EntityStaticRange<WireSlotId, 4>; 13] = [
            wires::LX12_S0,
            wires::LX12_S1,
            wires::LX12_S2,
            wires::LX12_S3,
            wires::LX12_S4,
            wires::LX12_S5,
            wires::LX12_S6,
            wires::LX12_S7,
            wires::LX12_S8,
            wires::LX12_S9,
            wires::LX12_S10,
            wires::LX12_S11,
            wires::LX12_S12,
        ];
        pub const LX16_S: [EntityStaticRange<WireSlotId, 4>; 17] = [
            wires::LX16_S0,
            wires::LX16_S1,
            wires::LX16_S2,
            wires::LX16_S3,
            wires::LX16_S4,
            wires::LX16_S5,
            wires::LX16_S6,
            wires::LX16_S7,
            wires::LX16_S8,
            wires::LX16_S9,
            wires::LX16_S10,
            wires::LX16_S11,
            wires::LX16_S12,
            wires::LX16_S13,
            wires::LX16_S14,
            wires::LX16_S15,
            wires::LX16_S16,
        ];
        pub const LX12_N: [EntityStaticRange<WireSlotId, 4>; 13] = [
            wires::LX12_N0,
            wires::LX12_N1,
            wires::LX12_N2,
            wires::LX12_N3,
            wires::LX12_N4,
            wires::LX12_N5,
            wires::LX12_N6,
            wires::LX12_N7,
            wires::LX12_N8,
            wires::LX12_N9,
            wires::LX12_N10,
            wires::LX12_N11,
            wires::LX12_N12,
        ];
        pub const LX16_N: [EntityStaticRange<WireSlotId, 4>; 17] = [
            wires::LX16_N0,
            wires::LX16_N1,
            wires::LX16_N2,
            wires::LX16_N3,
            wires::LX16_N4,
            wires::LX16_N5,
            wires::LX16_N6,
            wires::LX16_N7,
            wires::LX16_N8,
            wires::LX16_N9,
            wires::LX16_N10,
            wires::LX16_N11,
            wires::LX16_N12,
            wires::LX16_N13,
            wires::LX16_N14,
            wires::LX16_N15,
            wires::LX16_N16,
        ];
    }
    pub mod ultrascaleplus {
        use prjcombine_entity::id::EntityStaticRange;
        use prjcombine_interconnect::db::WireSlotId;

        use crate::defs::ultrascaleplus::wires;

        pub const LX12_W: [EntityStaticRange<WireSlotId, 8>; 7] = [
            wires::LX12_W0,
            wires::LX12_W1,
            wires::LX12_W2,
            wires::LX12_W3,
            wires::LX12_W4,
            wires::LX12_W5,
            wires::LX12_W6,
        ];
        pub const LX12_E: [EntityStaticRange<WireSlotId, 8>; 7] = [
            wires::LX12_E0,
            wires::LX12_E1,
            wires::LX12_E2,
            wires::LX12_E3,
            wires::LX12_E4,
            wires::LX12_E5,
            wires::LX12_E6,
        ];
        pub const LX12_S: [EntityStaticRange<WireSlotId, 8>; 13] = [
            wires::LX12_S0,
            wires::LX12_S1,
            wires::LX12_S2,
            wires::LX12_S3,
            wires::LX12_S4,
            wires::LX12_S5,
            wires::LX12_S6,
            wires::LX12_S7,
            wires::LX12_S8,
            wires::LX12_S9,
            wires::LX12_S10,
            wires::LX12_S11,
            wires::LX12_S12,
        ];
        pub const LX12_N: [EntityStaticRange<WireSlotId, 8>; 13] = [
            wires::LX12_N0,
            wires::LX12_N1,
            wires::LX12_N2,
            wires::LX12_N3,
            wires::LX12_N4,
            wires::LX12_N5,
            wires::LX12_N6,
            wires::LX12_N7,
            wires::LX12_N8,
            wires::LX12_N9,
            wires::LX12_N10,
            wires::LX12_N11,
            wires::LX12_N12,
        ];

        pub const X1_W: [EntityStaticRange<WireSlotId, 8>; 2] = [wires::X1_W0, wires::X1_W1];
        pub const X1_E: [EntityStaticRange<WireSlotId, 8>; 2] = [wires::X1_E0, wires::X1_E1];
        pub const X1_S: [EntityStaticRange<WireSlotId, 8>; 2] = [wires::X1_S0, wires::X1_S1];
        pub const X1_N: [EntityStaticRange<WireSlotId, 8>; 2] = [wires::X1_N0, wires::X1_N1];

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
    }
}
