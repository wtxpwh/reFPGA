use prjcombine_tablegen::target_defs;

target_defs! {
    variant scm;
    variant ecp;
    variant xp;
    variant machxo;
    variant ecp2;
    variant ecp2m;
    variant xp2;
    variant ecp3;
    variant machxo2;
    variant ecp4;
    variant ecp5;
    variant crosslink;

    bel_class SLICE {
        input A[2], B[2], C[2], D[2];
        input M[2];
        input FXA, FXB;
        input CLK, LSR, CE;
        if variant [ecp5, crosslink] {
            input WCK, WRE;
        }
        output F[2], Q[2];
        if variant [ecp5, crosslink] {
            // skip
        } else {
            output OFX[2];
        }
    }

    if variant machxo {
        // ???
        bel_class CIBTEST_SEL {
            input TSEL[2];
        }
    }

    bel_class EBR {
        input CLKA, CLKB;
        input RSTA, RSTB;
        input CEA, CEB;
        input WEA, WEB;
        if variant [ecp, xp, machxo, ecp2, ecp2m, xp2] {
            input MORCLKA, MORCLKB;
        }
        if variant ecp3 {
            input OCLKA, OCLKB;
        }
        if variant [ecp3, machxo2, ecp4, ecp5, crosslink] {
            input OCEA, OCEB;
        }
        input CSA[3], CSB[3];
        if variant [ecp, xp, machxo, machxo2, crosslink] {
            input ADA[13], ADB[13];
        } else {
            input ADA[14], ADB[14];
        }
        if variant [ecp2, ecp2m, xp2, ecp3] {
            input ADB3W; // ?!?
            input BLENABLE, BHENABLE;
        }
        if variant [machxo2, crosslink] {
            input DIA[9], DIB[9];
            output DOA[9], DOB[9];
        } else {
            input DIA[18], DIB[18];
            output DOA[18], DOB[18];
        }
        if variant [machxo, machxo2, scm, ecp4] {
            output EF, FF, AE, AF;
        }
    }

    if variant [ecp, ecp2, ecp2m, xp2] {
        if variant ecp {
            enum DSP_MODE {
                MULT9X9,
                MULT9X9MAC,
                MULT9X9ADDSUB,
                MULT9X9ADDSUBSUM,
                MULT18X18,
                MULT18X18MAC,
                MULT18X18ADDSUB,
                MULT18X18ADDSUBSUM,
                MULT36X36,
            }
        } else {
            // missing the MULT9X9MAC mode (which was wholly redundant with MULT18X18MAC anyway)
            enum DSP_MODE {
                MULT9X9,
                MULT9X9ADDSUB,
                MULT9X9ADDSUBSUM,
                MULT18X18,
                MULT18X18MAC,
                MULT18X18ADDSUB,
                MULT18X18ADDSUBSUM,
                MULT36X36,
            }
        }

        if variant ecp {
        } else {
            enum DSP_MUX_P3 {
                P,
                SRO,
            }
        }

        bel_class DSP {
            // CLK, CE, ADDNSUB, ACCUMSLOAD, SOURCE* invertible via INT tile
            // RST, SIGNED* invertible within bel
            input CLK[4];
            input RST[4];
            input CE[4];
            input A0[18];
            input A1[18];
            input A2[18];
            input A3[18];
            input B0[18];
            input B1[18];
            input B2[18];
            input B3[18];
            input ACCUMSLOAD1, ACCUMSLOAD3;
            input ADDNSUB1, ADDNSUB3;
            if variant ecp {
                input SIGNEDAB[4];
            } else {
                input SIGNEDA[4];
                input SIGNEDB[4];
                input SOURCEA[4];
                input SOURCEB[4];
                input LD1[16];
                input LD3[16];
            }
            if variant ecp {
                // on ecp, these are independent outputs
                // on ecp2, {SROB,SROA} is multiplexed onto P3 instead
                output SROA[18];
                output SROB[18];
            }
            output P0[36];
            output P1[36];
            output P2[36];
            output P3[36];

            attribute MODE: DSP_MODE;
            attribute GSR: bool;

            if variant ecp {
            } else {
                attribute MUX_P3: DSP_MUX_P3;
            }

            for i in 0..8 {
                if variant ecp {
                    // replaced by SOURCE[AB] on ecp2
                    attribute "SHIFT_IN_A{i}": bool;
                    attribute "SHIFT_IN_B{i}": bool;
                }

                attribute "MUX_CLK_A{i}": bitvec[2];
                attribute "MUX_RST_A{i}": bitvec[2];
                attribute "MUX_CE_A{i}": bitvec[2];
                attribute "REG_A{i}": bool;
                attribute "MUX_CLK_B{i}": bitvec[2];
                attribute "MUX_RST_B{i}": bitvec[2];
                attribute "MUX_CE_B{i}": bitvec[2];
                attribute "REG_B{i}": bool;

                attribute "MUX_CLK_M{i}": bitvec[2];
                attribute "MUX_RST_M{i}": bitvec[2];
                attribute "MUX_CE_M{i}": bitvec[2];
                attribute "REG_M{i}": bool;

                attribute "MUX_CLK_P{i}": bitvec[2];
                attribute "MUX_RST_P{i}": bitvec[2];
                attribute "MUX_CE_P{i}": bitvec[2];
                attribute "REG_P{i}": bool;
            }

            for j in 0..2 {
                for i in 0..4 {
                    if variant ecp {
                        attribute "MUX_CLK_SIGNEDAB_S{j}_{i}": bitvec[2];
                        attribute "MUX_RST_SIGNEDAB_S{j}_{i}": bitvec[2];
                        attribute "MUX_CE_SIGNEDAB_S{j}_{i}": bitvec[2];
                        attribute "REG_SIGNEDAB_S{j}_{i}": bool;
                    } else {
                        attribute "MUX_CLK_SIGNEDA_S{j}_{i}": bitvec[2];
                        attribute "MUX_RST_SIGNEDA_S{j}_{i}": bitvec[2];
                        attribute "MUX_CE_SIGNEDA_S{j}_{i}": bitvec[2];
                        attribute "REG_SIGNEDA_S{j}_{i}": bool;

                        attribute "MUX_CLK_SIGNEDB_S{j}_{i}": bitvec[2];
                        attribute "MUX_RST_SIGNEDB_S{j}_{i}": bitvec[2];
                        attribute "MUX_CE_SIGNEDB_S{j}_{i}": bitvec[2];
                        attribute "REG_SIGNEDB_S{j}_{i}": bool;
                    }
                }
            }

            for j in 0..2 {
                for i in 0..2 {
                    attribute "MUX_CLK_ADDNSUB_S{j}_{i}": bitvec[2];
                    attribute "MUX_RST_ADDNSUB_S{j}_{i}": bitvec[2];
                    attribute "MUX_CE_ADDNSUB_S{j}_{i}": bitvec[2];
                    attribute "REG_ADDNSUB_S{j}_{i}": bool;

                    attribute "MUX_CLK_ACCUMSLOAD_S{j}_{i}": bitvec[2];
                    attribute "MUX_RST_ACCUMSLOAD_S{j}_{i}": bitvec[2];
                    attribute "MUX_CE_ACCUMSLOAD_S{j}_{i}": bitvec[2];
                    attribute "REG_ACCUMSLOAD_S{j}_{i}": bool;
                }
            }
        }
    } else if variant [ecp3, ecp4, ecp5] {
        bel_class DSP {
            input CLK[4];
            input RST[4];
            input CE[4];
            input A0[18], A1[18];
            input B0[18], B1[18];
            input C[54];
            input SIGNEDA[2], SIGNEDB[2];
            input SOURCEA[2], SOURCEB[2];
            input OP[11];
            if variant [ecp4, ecp5] {
                input OPPRE[2];
            }
            // in mult mode:
            // - ecp3: R[35:0] is P0, R[71:36] is P1
            // - ecp[45]:
            //   - R[17:0] == P0[17:0]
            //   - R[26:18] == P1[8:0]
            //   - R[44:27] == P0[35:18]
            //   - R[71:45] == P1[35:9]
            // SROA, when used, is mapped to bits 71:54
            // in alu mode, low 54 bits are ALU out, some high bits are flags:
            // - 54: OVERUNDER
            // - 55: UNDER
            // - 56: OVER
            // - 57: EQPATB
            // - 58: EQPAT
            // - 59: EQOM
            // - 60: EQZM
            // - 61: EQZ
            output R[72];
        }
    }

    if variant machxo {
        bel_class IO {
            input PADDO, PADDT;
            output PADDI;
        }
        bel_class PICTEST {
            input C[2], D[2];
            output Q[2];
        }
    } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
        bel_class IO {
            input CLK, LSR, CE;
            // on ecp2+, IO[0] also has OPOS[2] and ONEG[2]; however, they are borrowed from
            // IO[1]'s OPOS[0] and ONEG[0] respectively
            input OPOS[2], ONEG[2];
            input PADDT;
            output IPOS[2];
            output DI, PADDI;
            if variant [ecp2, ecp2m, xp2] {
                input ECLKI, ECLKO;
                input DEL[4];
                output QPOS[2];
                // IO[0] also has QNEG[0..2]; however, it is multiplexed onto IO[1]'s QPOS
                // and not an independent output
            }
        }

        bel_class DQS {
            input CLK;
            input READ;
            output DDRCLKPOL;
            output PRMBDET;
            if variant [ecp2, ecp2m, xp2] {
                output DATAVALID;
            }
        }

        bel_class DQSDLL {
            input CLK, RST;
            input UDDCNTL;
            output LOCK;
        }
    } else if variant ecp3 {
        bel_class IO {
            input CLK, LSR, CE;
            input OPOSA, OPOSB, ONEGA, ONEGB;
            input TS;
            input DEL[4];
            input ECLK, ECLKDQSR;
            output IPA, IPB, INA, INB;
            output INDD, PADDI;
        }

        bel_class DQS {
            input SCLK, RST;
            input ECLK, ECLKW;
            input READ;
            input DCA[7];
            input DYNDELAY[8];
            output DQSW, ECLKDQSR, DDRCLKPOL;
            output PRMBDET;
            output DDRLAT;
            output DATAVALID;
            output TEST_DQSCNT[2];
            output TEST_ECLKCNT;
        }

        bel_class DQSDLL {
            input CLK, RST;
            input UDDCNTLN;
            output LOCK;
            output TEST_SDOUT[7];
        }
    }

    if variant ecp3 {
        bel_class ECLKSYNC {
            input ECLKI;
            input STOP;
            output ECLKO;
        }
    }

    bel_class DCS {
        input CLK[2];
        input SEL;
        output DCSOUT;
    }

    if variant [machxo, ecp, xp, ecp2, ecp2m, xp2] {
        bel_class PLL {
            input CLKI, CLKFB;
            output CLKOP, CLKOS, CLKOK;
            if variant xp2 {
                output CLKOK2;
            }
            if variant [ecp, xp] {
                // hm.
            } else {
                output CLKINTFB;
            }
            input RESETM, RESETK;
            output LOCK, DNLOCK, UPLOCK;
            input CNTRST, PWD;
            if variant xp2 {
            } else {
                // on ecp2, only present on plain PLL, not SPLL
                input DDAMODE, DDAIZR, DDAILAG, DDAIDEL[3];
                output DDAOZR, DDAOLAG, DDAODEL[3];
            }
            if variant [ecp2, ecp2m, xp2] {
                input DPAMODE, DRPAI[4], DFPAI[4];
                output DRPAO[4], DFPAO[4];

            }
            input TCLKI;
            output TESTOUT;
        }
    }

    if variant [ecp2, ecp2m, xp2] {
        bel_class CLKDIV {
            input CLKI;
            output CDIV1, CDIV2, CDIV4, CDIV8;
            input RST, RELEASE;
        }
    }

    if variant [ecp2, ecp2m] {
        bel_class DLL {
            input CLKI, CLKFB;
            output CLKOP, CLKOS;
            output DCNTL[9];
            input DTCCST[2];
            input RSTN;
            input UDDCNTL;
            output LOCK;

            input SMICLK, SMIRSTN;
            input SMIRD, SMIWR, SMIADDR[10], SMIWDATA;
            output SMIRDATA;
        }

        bel_class DLLDEL {
            input CLKI;
            output BYPASS;
            output CLKO;
        }
    }

    bel_class GSR {
        if variant machxo {
        } else {
            input CLK;
        }
        if variant scm {
            input USR;
        } else {
            input GSR;
        }
    }

    if variant crosslink {
    } else {
        bel_class JTAG {
            output JTDI, JTCK, JSHIFT, JUPDATE;
            if variant machxo {
                output JRST;
            } else {
                output JRSTN;
            }
            if variant scm {
                for i in 1..=8 {
                    input "JTDO{i}";
                    output "JCE{i}", "JRTI{i}";
                }
                output PSRCAP, PSRSFTN;
                for i in 1..=3 {
                    input "PSROUT{i}";
                    output "PSRENABLE{i}";
                }
                output SCANI;
                for i in 1..=8 {
                    output "SCANENABLE{i}";
                }
                output TRESET, TCK, TMS, TDI;
                input TDO;
            } else {
                for i in 1..=2 {
                    input "JTDO{i}";
                    output "JCE{i}", "JRTI{i}";
                }
            }
            if variant [ecp2, ecp2m] {
                input IJTAGEN, ITCK, ITMS, ITDI;
                output ITDO;
            }
        }
    }

    if variant [machxo, crosslink] {
    } else {
        bel_class START {
            input STARTCLK;
        }
    }

    if variant [scm, machxo, ecp, ecp2, ecp2m, ecp3, xp2] {
        bel_class OSC {
            output CFGCLK;
        }
    } else if variant [machxo2, ecp4, ecp5] {
        bel_class OSC {
            input STDBY;
            output OSC;
        }
    } else if variant crosslink {
        bel_class OSC {
            input HFOUTEN;
            input HFCLKDIV[2];
            output LFCLKOUT, HFCLKOUT;
        }
    }

    if variant [machxo, machxo2] {
        bel_class TSALL {
            if variant machxo2 {
                input TSALLI;
            }
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2, ecp4, ecp5] {
        bel_class SED {
            input SEDENABLE, SEDSTART, SEDFRCERR;
            output SEDCLKOUT, SEDINPROG, SEDDONE, SEDERR;
            if variant [ecp2, ecp2m, ecp3] {
                input SEDMODE;
            }
            if variant [machxo2, ecp4, ecp5] {
                input SEDEXCLK;
            }
            if variant [ecp3, machxo2, ecp4, ecp5] {
                output AUTODONE;
            }
        }
    }

    if variant [ecp2, ecp2m] {
        bel_class SPIM {
            input A[8];
            input SEL;
        }
    }

    if variant xp2 {
        bel_class SSPI {
            input CLK, CS, SI;
            output SO;
        }

        bel_class STF {
            input STOREN;
            output UFMBUSYN, UFMFAIL;
        }

        bel_class WAKEUP {
            input USRGOE;
        }
    }

    if variant ecp3 {
        // wtf is all of this
        bel_class AMBOOT {
            input CSI;
            input AUTOREBOOTI;
            input A[8];
            output CSO;
            output O[8];
        }

        bel_class PERREG {
            input D[16];
            output Q[16];
        }

        bel_class TESTIN {
            input CLK, CE;
            input SCANIN;
            output SCANOUT;
            output OUT[16];
        }

        bel_class TESTOUT {
            input CLK, CE;
            input LOADIN[32];
            output SCANOUT;
        }

        bel_class DTS {
            input DTSI[4];
            output DTSO;
        }
    }

    if variant ecp5 {
        bel_class CCLK {
            input PADDO, PADDT;
            output PADDI;
        }
    }

    if variant [ecp2m, ecp3] {
        bel_class SERDES {
            input CIN[12];
            output COUT[20];
            input CYAWSTN;

            if variant ecp2m {
                input FFC_CK_CORE_RX;
            }
            input FFC_CK_CORE_TX;
            input FFC_MACRO_RST;
            input FFC_QUAD_RST;
            input FFC_TRST;
            output FFS_PLOL;
            if variant ecp2m {
                output FF_TX_F_CLK;
                output FF_TX_H_CLK;
                output FF_TX_Q_CLK;
            }
            output REFCK2CORE;
            if variant ecp3 {
                input FFC_SYNC_TOGGLE;
            }

            for ch in 0..4 {
                input "FFC_EI_EN_{ch}";
                input "FFC_ENABLE_CGALIGN_{ch}";
                input "FFC_FB_LOOPBACK_{ch}";
                input "FFC_LANE_RX_RST_{ch}";
                input "FFC_LANE_TX_RST_{ch}";
                input "FFC_PCIE_CT_{ch}";
                input "FFC_PCI_DET_EN_{ch}";
                input "FFC_PFIFO_CLR_{ch}";
                input "FFC_RRST_{ch}";
                input "FFC_RXPWDNB_{ch}";
                input "FFC_SB_INV_RX_{ch}";
                input "FFC_SB_PFIFO_LP_{ch}";
                input "FFC_SIGNAL_DETECT_{ch}";
                input "FFC_TXPWDNB_{ch}";
                if variant ecp3 {
                    input "FFC_CK_CORE_RX_{ch}";
                    input "FFC_DIV11_MODE_RX_{ch}";
                    input "FFC_DIV11_MODE_TX_{ch}";
                    input "FFC_LDR_CORE2TX_EN_{ch}";
                    input "FFC_RATE_MODE_RX_{ch}";
                    input "FFC_RATE_MODE_TX_{ch}";
                }

                output "FFS_CC_OVERRUN_{ch}";
                output "FFS_CC_UNDERRUN_{ch}";
                output "FFS_LS_SYNC_STATUS_{ch}";
                output "FFS_PCIE_CON_{ch}";
                output "FFS_PCIE_DONE_{ch}";
                output "FFS_RLOL_{ch}";
                output "FFS_RLOS_LO_{ch}";
                output "FFS_RXFBFIFO_ERROR_{ch}";
                output "FFS_TXFBFIFO_ERROR_{ch}";
                if variant ecp3 {
                    output "FFS_CDR_TRAIN_DONE_{ch}";
                    output "FFS_RLOS_HI_{ch}";
                    output "FFS_SKP_ADDED_{ch}";
                    output "FFS_SKP_DELETED_{ch}";
                }

                input "FF_EBRD_CLK_{ch}";
                input "FF_RXI_CLK_{ch}";
                input "FF_TX_D_{ch}"[24];
                output "FF_RX_D_{ch}"[24];
                output "FF_RX_F_CLK_{ch}";
                output "FF_RX_H_CLK_{ch}";
                if variant ecp2m {
                    output "FF_RX_Q_CLK_{ch}";
                } else {
                    output "FF_TX_F_CLK_{ch}";
                    output "FF_TX_H_CLK_{ch}";
                }
                input "FF_TXI_CLK_{ch}";

                if variant ecp2m {
                    output "OOB_OUT_{ch}";
                } else {
                    input "LDR_CORE2TX_{ch}";
                    output "LDR_RX2CORE_{ch}";
                    output "PCIE_PHYSTATUS_{ch}";
                    input "PCIE_POWERDOWN_{ch}"[2];
                    input "PCIE_RXPOLARITY_{ch}";
                    output "PCIE_RXVALID_{ch}";
                    input "PCIE_TXCOMPLIANCE_{ch}";
                    input "PCIE_TXDETRX_PR2TLB_{ch}";
                }
            }
            if variant ecp3 {
                output FFS_BIST_RPT[16];
            }

            input SCIADDR[6];
            input SCISELAUX;
            input SCISELCH[4];
            input SCIENAUX;
            input SCIENCH[4];
            input SCIRD;
            input SCIWSTN;
            input SCIWDATA[8];
            output SCIRDATA[8];
            output SCIINT;
        }
    }

    // TODO: convert to routing
    if variant [ecp3, machxo2] {
        bel_class HSDCLK_ROOT {
            output OUT_W[8];
            output OUT_E[8];
        }
    }

    wire TIE0: tie 0;
    wire TIE1: tie 1;

    if variant [ecp, xp, machxo, machxo2] {
        const NUM_X0 = 4;
    } else {
        const NUM_X0 = 2;
    }

    wire X0_W[NUM_X0]: mux;
    wire X0_E[NUM_X0]: mux;
    wire X0_S[NUM_X0]: mux;
    wire X0_N[NUM_X0]: mux;

    if variant [ecp2, ecp2m, xp2, ecp3] {
        // only [1] and [4] are actually used
        // X1_W0 and X1_E0 are aliased to X1_H where applicable; X1_S0/X1_N0 are aliased to X1_V
        wire X1_H[8]: mux;
        wire X1_V[8]: mux;
    }

    if variant [scm, ecp4] {
        // indices 0, 1, 4, 5, 6, 7 are valid
        const NUM_X1 = 8;
    } else if variant [ecp2, ecp2m, xp2, ecp3] {
        // indices 0, 1, 4, 5 are valid (and 1, 4 are aliased to X1_[HV])
        const NUM_X1 = 8;
    } else {
        // indices 0, 1 are valid
        const NUM_X1 = 2;
    }
    wire X1_W0[NUM_X1]: mux;
    wire X1_W1[NUM_X1]: branch E;
    wire X1_E0[NUM_X1]: mux;
    wire X1_E1[NUM_X1]: branch W;
    wire X1_S0[NUM_X1]: mux;
    wire X1_S1[NUM_X1]: branch N;
    wire X1_N0[NUM_X1]: mux;
    wire X1_N1[NUM_X1]: branch S;

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

    if variant [scm, ecp4] {
        const NUM_X6 = 8;
    } else {
        const NUM_X6 = 4;
    }
    wire X6_W0[NUM_X6]: mux;
    for i in 1..7 {
        wire "X6_W{i}"[NUM_X6]: branch E;
    }
    wire X6_E0[NUM_X6]: mux;
    for i in 1..7 {
        wire "X6_E{i}"[NUM_X6]: branch W;
    }
    wire X6_S0[NUM_X6]: mux;
    for i in 1..7 {
        wire "X6_S{i}"[NUM_X6]: branch N;
    }
    wire X6_N0[NUM_X6]: mux;
    for i in 1..7 {
        wire "X6_N{i}"[NUM_X6]: branch S;
    }

    // The primary clocks, connected to all cells.
    if variant scm {
        // per quadrant
        region_slot PCLK;
        wire PCLK[12]: regional PCLK;
    } else if variant [ecp, xp, machxo] {
        // nominally 4 primary and 4 secondary clocks, but they're close enough in behavior
        // per quadrant on ECP, XP; global on MachXO
        region_slot PCLK;
        wire PCLK[8]: regional PCLK;
    } else if variant [ecp2, ecp2m, xp2] {
        // per quadrant
        region_slot PCLK;
        wire PCLK[8]: regional PCLK;
        wire SCLK_ROOT[8]: regional PCLK;
    } else if variant [ecp3, machxo2] {
        // Rotating primary clocks.
        region_slot RPCLK[4];
        wire RPCLK[8] {
            0 => regional RPCLK[0],
            1 => regional RPCLK[1],
            2 => regional RPCLK[2],
            3 => regional RPCLK[3],
            4 => regional RPCLK[0],
            5 => regional RPCLK[1],
            6 => regional RPCLK[2],
            7 => regional RPCLK[3],
        }
    } else if variant ecp4 {
        // per quadrant
        region_slot PCLK;
        wire PCLK[20]: regional PCLK;
    } else if variant ecp5 {
        // per quadrant
        region_slot PCLK;
        wire PCLK[16]: regional PCLK;
    } else if variant crosslink {
        // global
        region_slot PCLK;
        wire PCLK[8]: regional PCLK;
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        region_slot RSCLK[4];
        region_slot RSCLK_V;
        // Rotating secondary clocks.
        wire RSCLK[8] {
            0 => regional RSCLK[0],
            1 => regional RSCLK[1],
            2 => regional RSCLK[2],
            3 => regional RSCLK[3],
            4 => regional RSCLK[0],
            5 => regional RSCLK[1],
            6 => regional RSCLK[2],
            7 => regional RSCLK[3],
        }

        // Distribution wires supplying RSCLK.
        wire RSCLK_H[8]: branch RSCLK_W;
        wire RSCLK_V[8]: regional RSCLK_V;
        wire RSCLK_V_N[8]: branch S;
    }

    if variant scm {
        // Distributed secondary clocks.
        wire DSCLK[2]: mux;
        wire DSCLK_W[2]: branch E;
        wire DSCLK_E[2]: branch W;

        // Distribution wires supplying the above
        wire DSCLK_V[7] {
            0 => multi_root,
            1..7 => multi_branch S,
        }
        wire DSCLK_H[7] {
            0..3 => multi_branch E,
            3 => multi_root,
            4..7 => multi_branch W,
        }
    }

    if variant [ecp, xp, ecp2, ecp2m, xp2] {
        wire IMUX_DCS_CLKA[2]: mux;
        wire IMUX_DCS_CLKB[2]: mux;
    }

    // Edge clocks.
    if variant [ecp2, ecp2m, xp2, ecp3] {
        region_slot ECLK;
        wire ECLK[2]: regional ECLK;

        if variant [ecp2, ecp2m, xp2] {
            wire IMUX_IO_ECLKI[2]: mux;
            wire IMUX_IO_ECLKO[2]: mux;
        } else {
            wire IMUX_IO_ECLK[4]: mux;
            wire IMUX_IO_ECLKDQSR[4]: mux;
        }
    }

    wire IMUX_A[8]: mux;
    wire IMUX_B[8]: mux;
    wire IMUX_C[8]: mux;
    wire IMUX_D[8]: mux;
    wire IMUX_M[8]: mux;

    // ECP4. ECP5, Crosslink: only 2 indices used, but has IMUX_MUX*
    // others: all 4 indices used
    if variant [ecp4, ecp5, crosslink] {
        wire IMUX_CLK[2]: mux;
        wire IMUX_LSR[2]: mux;
        wire IMUX_MUXCLK[4]: mux;
        wire IMUX_MUXLSR[4]: mux;
    } else {
        wire IMUX_CLK[4]: mux;
        wire IMUX_LSR[4]: mux;
    }
    if variant ecp4 {
        wire IMUX_CLK_DELAY[2]: mux;
    }
    wire IMUX_CE[4]: mux;

    wire OUT_F[8]: bel;
    wire OUT_F_W[8]: branch E;
    wire OUT_F_E[8]: branch W;
    wire OUT_Q[8]: bel;
    if variant [ecp5, crosslink] {
    } else {
        wire OUT_OFX[8]: bel;
        wire OUT_OFX_W[8]: branch E;
    }

    if variant machxo {
        // test wires (purpose unclear)
        wire OUT_TI[12]: bel;
    }

    if variant scm {
        // EBR interconnect
        wire IMUX_EBR_DIA[18]: mux;
        wire IMUX_EBR_DIB[18]: mux;
        wire IMUX_EBR_ADA[11]: mux;
        wire IMUX_EBR_ADB[11]: mux;
        wire IMUX_EBR_CLKA: mux;
        wire IMUX_EBR_CLKB: mux;
        wire IMUX_EBR_RSTA: mux;
        wire IMUX_EBR_RSTB: mux;
        wire IMUX_EBR_CEA: mux;
        wire IMUX_EBR_CEB: mux;
        wire IMUX_EBR_WEA: mux;
        wire IMUX_EBR_WEB: mux;
        wire OUT_EBR_DOA[18]: bel;
        wire OUT_EBR_DOB[18]: bel;
        wire OUT_EBR_FF: bel;
        wire OUT_EBR_AF: bel;
        wire OUT_EBR_EF: bel;
        wire OUT_EBR_AE: bel;

        wire EBR_W0[64]: mux;
        wire EBR_W1[64]: branch EBR_E;
        wire EBR_E0[64]: mux;
        wire EBR_E1[64]: branch EBR_W;

        // IO interconnect
        wire IMUX_IO[32]: mux;
        wire OUT_IO[24]: bel;
        for i in 0..8 {
            wire "OUT_MACO_IO_S{i}"[32]: bel;
        }
        for i in 0..8 {
            wire "OUT_MACO_IO_N{i}"[32]: bel;
        }
        wire IO_W0[32]: mux;
        for i in 1..9 {
            wire "IO_W{i}"[32]: branch IO_E;
        }
        wire IO_E0[32]: mux;
        for i in 1..9 {
            wire "IO_E{i}"[32]: branch IO_W;
        }
        wire IO_T_W: branch IO_E;
        wire IO_T_E: branch IO_W;
    }

    if variant machxo {
        wire OUT_F_PLC_IO[6]: branch IO_FROM_PLC;
        wire OUT_Q_PLC_IO[6]: branch IO_FROM_PLC;
        // mux of the above two
        wire PLC_IO[6]: mux;

        // alias of PLC_IO, but with complex mapping
        wire IMUX_IO_O_PLC[6]: mux;
        wire IMUX_IO_O[6]: mux;
        wire IMUX_IO_T[6]: mux;
        wire OUT_IO_PADDI[6]: bel;
    } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
        region_slot DQS;
        region_slot DQSDLL;

        wire OUT_IO_PADDI[2]: bel;
    } else if variant ecp3 {
        region_slot DQS;
        region_slot DQSDLL;

        wire OUT_IO_PADDI[4]: bel;
        wire OUT_DQS_ECLKDQSR: regional DQS;
        wire IMUX_DQS_ECLK: mux;
        wire IMUX_DQS_ECLKW: mux;
    } else if variant ecp4 {
        region_slot DQS;
    } else if variant ecp5 {
        region_slot DQS;
    }

    if variant machxo {
        // alias of OUT_IO_PADDI
        wire IO_GCLK_S[2]: mux;
        wire IO_GCLK_N[2]: mux;
    }

    if variant [machxo, ecp, xp, ecp2, ecp2m, xp2, ecp3] {
        wire IMUX_PLL_CLKI: mux;
        wire IMUX_PLL_CLKFB: mux;
        wire OUT_PLL_CLKINTFB: bel;
        wire OUT_PLL_CLKOP: bel;
        wire OUT_PLL_CLKOS: bel;
        wire OUT_PLL_CLKOK: bel;
        if variant [xp2, ecp3] {
            wire OUT_PLL_CLKOK2: bel;
        }
    }

    if variant machxo {
        // aliases of OUT_IO_PADDI
        wire IO_PLL_CLKI: mux;
        wire IO_PLL_CLKFB: mux;
    }

    if variant [ecp2, ecp2m, ecp3] {
        wire PLL_TO_ECLK[2]: mux;
        wire IMUX_DLL_CLKI: mux;
        wire IMUX_DLL_CLKFB: mux;
        wire OUT_DLL_CLKOP: bel;
        wire OUT_DLL_CLKOS: bel;
        wire IMUX_DLLDEL_CLKI: mux;
        wire OUT_DLLDEL_CLKO: bel;
        if variant [ecp2, ecp2m] {
            wire OUT_DLLDEL_BYPASS: bel;
        }
    }
    if variant [ecp2, ecp2m, xp2] {
        wire IMUX_CLKDIV_CLKI: mux;
        wire OUT_CLKDIV_CDIV1: bel;
        wire OUT_CLKDIV_CDIV2: bel;
        wire OUT_CLKDIV_CDIV4: bel;
        wire OUT_CLKDIV_CDIV8: bel;
    }
    if variant ecp3 {
        wire IMUX_DQSDLL_CLK: mux;
    }

    if variant [ecp2m, ecp3] {
        wire OUT_SERDES_FF_TX_F_CLK: bel;
        wire OUT_SERDES_FF_TX_H_CLK: bel;
    }

    tile_slot INT {
        bel_slot INT: routing;

        tile_class INT_PLC {
            cell CELL;
            switchbox INT;
        }

        if variant machxo {
            tile_class
                INT_SIO_S_W, INT_SIO_S_W_CLK, INT_SIO_S_E, INT_SIO_S_E_CFG,
                INT_SIO_S_S4, INT_SIO_S_S6, INT_SIO_S_N4, INT_SIO_S_N6,
                INT_SIO_L_W, INT_SIO_L_E, INT_SIO_L_E_CFG,
                INT_SIO_L_S4, INT_SIO_L_S6, INT_SIO_L_N4, INT_SIO_L_N6
            {
                cell CELL;
                switchbox INT;
            }
        }
        if variant scm {
            tile_class INT_IO {
                cell CELL;
                switchbox INT;
            }
        }
        if variant [ecp, xp] {
            tile_class INT_IO_WE, INT_IO_SN {
                cell CELL;
                switchbox INT;
            }
        }
        if variant [ecp2, ecp2m, xp2, ecp3, machxo2, ecp4, ecp5] {
            tile_class INT_IO_WE, INT_IO_S, INT_IO_N {
                cell CELL;
                switchbox INT;
            }
        }
        if variant crosslink {
            tile_class INT_IO_S, INT_IO_N {
                cell CELL;
                switchbox INT;
            }
        }
        if variant ecp2m {
            tile_class INT_SERDES_N {
                cell CELL;
                switchbox INT;
            }
        }

        if variant [scm, ecp, xp, machxo2, ecp4, ecp5, crosslink] {
            tile_class INT_EBR {
                cell CELL;
                switchbox INT;
            }
        } else if variant [ecp2, ecp2m, xp2] {
            tile_class INT_EBR, INT_EBR_IO {
                cell CELL;
                switchbox INT;
            }
        } else if variant ecp3 {
            tile_class INT_EBR, INT_EBR_IO, INT_EBR_SCLK, INT_EBR_IO_SCLK {
                cell CELL;
                switchbox INT;
            }
        }

        if variant [ecp, xp] {
            tile_class INT_PLL {
                cell CELL;
                switchbox INT;
            }
        }
    }

    tile_slot IO {
        if variant scm {
            bel_slot IO[12]: legacy;
            bel_slot PICTEST[3]: legacy;
            tile_class IO_W4, IO_E4, IO_S4, IO_N4 {
                cell CELL[2];
                for i in 0..4 {
                    bel IO[i];
                }
                bel PICTEST[0];
            }
            tile_class IO_N8 {
                cell CELL[3];
                for i in 0..8 {
                    bel IO[i];
                }
                for i in 0..2 {
                    bel PICTEST[i];
                }
            }
            tile_class IO_W12, IO_E12, IO_S12, IO_N12 {
                cell CELL[4];
                for i in 0..12 {
                    bel IO[i];
                }
                for i in 0..3 {
                    bel PICTEST[i];
                }
            }
        } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
            bel_slot IO[2]: IO;
            bel_slot IO_INT: routing;
            tile_class IO_W, IO_E, IO_S, IO_N {
                cell CELL;
                for i in 0..2 {
                    bel IO[i];
                }
                if variant [ecp2, ecp2m, xp2] {
                    switchbox IO_INT;
                }
            }
        } else if variant machxo {
            bel_slot IO[6]: IO;
            bel_slot PICTEST[1]: PICTEST;
            bel_slot IO_INT: routing;
            tile_class
                SIO_S_W2, SIO_S_W4, SIO_S_E2, SIO_S_E4,
                SIO_S_S4, SIO_S_S6, SIO_S_N4, SIO_S_N6,
                SIO_L_W2, SIO_L_W4, SIO_L_E2, SIO_L_E4,
                SIO_L_S4, SIO_L_S6, SIO_L_N4, SIO_L_N6
            {
                cell CELL;
                switchbox IO_INT;
                if tile_class [SIO_S_S6, SIO_S_N6, SIO_L_S6, SIO_L_N6] {
                    for i in 0..6 {
                        bel IO[i];
                    }
                } else if tile_class [SIO_S_W2, SIO_S_E2, SIO_L_W2, SIO_L_E2] {
                    for i in 0..2 {
                        bel IO[i];
                    }
                    bel PICTEST[0];
                } else {
                    for i in 0..4 {
                        bel IO[i];
                    }
                }
            }
        } else if variant ecp3 {
            bel_slot IO[4]: IO;
            bel_slot IO_INT: routing;
            tile_class
                IO_W, IO_A_W, IO_DQS_W, IO_A_DQS_W, IO_DQS_DUMMY_W, IO_A_DQS_DUMMY_W,
                IO_E, IO_A_E, IO_DQS_E, IO_A_DQS_E,
                SIO_N, SIO_A_N, SIO_DQS_N, SIO_A_DQS_N,
                XSIO_E, XSIO_S, XSIO_N
            {
                cell CELL[3];
                for i in 0..4 {
                    bel IO[i];
                }
                if tile_class [XSIO_E, XSIO_S, XSIO_N] {
                } else {
                    switchbox IO_INT;
                }
            }
            tile_class IO_PLL_W, IO_PLL_E, IO_A_PLL_W, IO_A_PLL_E {
                cell CELL[2];
                for i in 0..4 {
                    bel IO[i];
                }
                switchbox IO_INT;
            }
        } else if variant machxo2 {
            bel_slot IO[4]: legacy;
            tile_class
                IO_W2, IO_W4, IO_W4_I3C,
                IO_E2, IO_E4,
                IO_S4, SIO_S2, SIO_S4,
                IO_N4, SIO_N2, SIO_N4
            {
                cell CELL;
                if tile_class [IO_W2, IO_E2, SIO_S2, SIO_N2] {
                    for i in 0..2 {
                        bel IO[i];
                    }
                } else {
                    for i in 0..4 {
                        bel IO[i];
                    }
                }
            }
        } else if variant ecp4 {
            bel_slot IO[4]: legacy;
            tile_class
                IO_W, IO_W_EBR_S, IO_W_EBR_N, IO_W_DSP_S, IO_W_DSP_N,
                IO_E, IO_E_EBR_S, IO_E_EBR_N, IO_E_DSP_S, IO_E_DSP_N,
                IO_N
            {
                cell CELL[4];
                for i in 0..4 {
                    bel IO[i];
                }
            }
        } else if variant ecp5 {
            bel_slot IO[4]: legacy;
            tile_class IO_W4, IO_E4 {
                cell CELL[3];
                for i in 0..4 {
                    bel IO[i];
                }
            }
            tile_class IO_S1 {
                cell CELL;
                bel IO[0];
            }
            tile_class IO_S2, IO_N2 {
                cell CELL[2];
                for i in 0..2 {
                    bel IO[i];
                }
            }
        } else if variant crosslink {
            bel_slot IO[4]: legacy;
            tile_class IO_S4 {
                cell CELL[4];
                for i in 0..4 {
                    bel IO[i];
                }
            }
            tile_class IO_S1A, IO_S1B {
                cell CELL;
                bel IO[0];
            }
        }
    }

    tile_slot BEL {
        bel_slot BEL_INT: routing;

        bel_slot SLICE[4]: SLICE;

        tile_class PLC, FPLC {
            if tile_class FPLC {
                if variant [scm, machxo2, ecp4, ecp5, crosslink] {
                    skip;
                }
            }
            cell CELL;

            for i in 0..4 {
                bel SLICE[i] {
                    for j in 0..2 {
                        input A[j] = IMUX_A[2 * i + j];
                        input B[j] = IMUX_B[2 * i + j];
                        input C[j] = IMUX_C[2 * i + j];
                        input D[j] = IMUX_D[2 * i + j];
                        input M[j] = IMUX_M[2 * i + j];
                        output F[j] = OUT_F[2 * i + j];
                    }

                    if variant [scm, ecp, xp, machxo] {
                        if bel_slot SLICE[3] {
                            input FXB = OUT_OFX_W[3];
                        }
                    }
                    if variant [machxo2, ecp4] {
                        if bel_slot SLICE[3] {
                            input FXA = OUT_OFX_W[3];
                        }
                    }
                    if variant [ecp5, crosslink] {
                        if bel_slot SLICE[3] {
                            input FXA = OUT_F_W[3];
                        }
                    }

                    if variant [ecp5, crosslink] {
                        // skip
                    } else {
                        for j in 0..2 {
                            output OFX[j] = OUT_OFX[2 * i + j];
                        }
                    }

                    if variant [ecp2, ecp2m, xp2, ecp3] {
                        if bel_slot SLICE[3] {
                            // skip
                        } else {
                            input CLK = IMUX_CLK[i];
                            input LSR = IMUX_LSR[i];
                            input CE = IMUX_CE[i];
                            for j in 0..2 {
                                output Q[j] = OUT_Q[2 * i + j];
                            }
                        }
                    } else {
                        if variant [ecp4, ecp5, crosslink] {
                            input CLK = IMUX_MUXCLK[i];
                            input LSR = IMUX_MUXLSR[i];
                        } else {
                            input CLK = IMUX_CLK[i];
                            input LSR = IMUX_LSR[i];
                        }
                        input CE = IMUX_CE[i];
                        for j in 0..2 {
                            output Q[j] = OUT_Q[2 * i + j];
                        }
                    }

                    if variant [ecp5, crosslink] {
                        if bel_slot SLICE[2] {
                            input WCK = IMUX_CLK[1];
                            input WRE = IMUX_LSR[1];
                        }
                    }
                }
            }
        }

        if variant scm {
            bel_slot IO_INT: routing;
            tile_class IO_INT_W, IO_INT_E, IO_INT_S {
                cell CELL;
                switchbox IO_INT;
            }
        }

        if variant machxo {
            bel_slot CIBTEST_SEL: CIBTEST_SEL;
            tile_class CIBTEST_SEL {
                cell CELL;
                bel CIBTEST_SEL;
            }
        }

        // EBR & DSP
        if variant scm {
            bel_slot EBR[1]: EBR;
            tile_class EBR_W, EBR_E {
                cell CELL[2];
                switchbox BEL_INT;
                bel EBR[0];
            }
        } else if variant [ecp, xp] {
            bel_slot EBR[1]: EBR;
            tile_class EBR {
                cell CELL[2];
                bel EBR[0];
            }
        } else if variant machxo {
            bel_slot EBR[1]: EBR;
            tile_class EBR {
                cell CELL[4];
                bel EBR[0];
            }
        } else if variant [ecp2, ecp2m, xp2, ecp3] {
            bel_slot EBR[1]: EBR;
            tile_class EBR {
                cell CELL[3];
                bel EBR[0];
            }
        } else if variant machxo2 {
            bel_slot EBR[1]: EBR;
            tile_class EBR, EBR_N {
                cell CELL[3];
                bel EBR[0];
            }
        } else if variant [ecp4, ecp5, crosslink] {
            bel_slot EBR[4]: EBR;
            tile_class EBR {
                cell CELL[9];
                for i in 0..4 {
                    bel EBR[i];
                }
            }
        }
        if variant ecp {
            bel_slot DSP[1]: DSP;
            tile_class DSP {
                cell CELL[8];
                bel DSP[0];
            }
        } else if variant [ecp2, ecp2m, xp2] {
            bel_slot DSP[1]: DSP;
            tile_class DSP {
                cell CELL[9];
                bel DSP[0];
            }
        } else if variant [ecp3, ecp4, ecp5] {
            bel_slot DSP[2]: DSP;
            tile_class DSP {
                cell CELL[9];
                for i in 0..2 {
                    bel DSP[i];
                }
            }
        }

        // MACO, SERDES
        if variant scm {
            bel_slot MACO: legacy;
            tile_class MACO_W, MACO_E {
                cell CELL[4];
                cell CELL_IO[2];
                cell CELL_EBR[10];
                switchbox BEL_INT;
                bel MACO;
            }

            bel_slot SERDES: legacy;
            tile_class SERDES_W, SERDES_E {
                cell CELL[7];
                bel SERDES;
            }
        } else if variant ecp2m {
            bel_slot SERDES: SERDES;
            tile_class SERDES_S, SERDES_N {
                cell CELL[27];
                bel SERDES;
            }
        } else if variant ecp3 {
            bel_slot SERDES: SERDES;
            tile_class SERDES {
                cell CELL[36];
                bel SERDES;
            }
        } else if variant ecp4 {
            bel_slot SERDES: legacy;
            tile_class SERDES1 {
                cell CELL[62];
                bel SERDES;
            }
            tile_class SERDES2 {
                cell CELL[120];
                bel SERDES;
            }
            tile_class SERDES3 {
                cell CELL[177];
                bel SERDES;
            }
        } else if variant ecp5 {
            bel_slot SERDES: legacy;
            tile_class SERDES {
                cell CELL[12];
                bel SERDES;
            }
        } else if variant crosslink {
            bel_slot MIPI: legacy;
            bel_slot CLKTEST_MIPI: legacy;
            tile_class MIPI_W, MIPI_E {
                cell CELL[24];
                bel MIPI;
                bel CLKTEST_MIPI;
            }
        }

        // PLLs etc
        if variant scm {
            bel_slot PLL[2]: legacy;
            bel_slot PLL_SMI: legacy;
            bel_slot DLL[4]: legacy;
            bel_slot DLL_DCNTL[2]: legacy;
            bel_slot PROMON: legacy;
            bel_slot RNET: legacy;
            bel_slot SERDES_CORNER: legacy;
            bel_slot M0: legacy;
            bel_slot M1: legacy;
            bel_slot M2: legacy;
            bel_slot M3: legacy;
            bel_slot RESETN: legacy;
            bel_slot RDCFGN: legacy;
            bel_slot CCLK: legacy;
            bel_slot TCK: legacy;
            bel_slot TMS: legacy;
            bel_slot TDI: legacy;
            tile_class PLL_SW, PLL_SE {
                cell CELL_S[4];
                cell CELL_WE[2];
                for i in 0..2 {
                    bel PLL[i];
                }
                for i in 0..4 {
                    bel DLL[i];
                }
                for i in 0..2 {
                    bel DLL_DCNTL[i];
                }
                bel PLL_SMI;
                bel PROMON;
                if tile_class PLL_SW {
                    bel RNET;
                }
            }
            tile_class PLL_NW, PLL_NE {
                cell CELL_N[2];
                cell CELL_WE;
                for i in 0..2 {
                    bel PLL[i];
                }
                for i in 0..2 {
                    bel DLL[i];
                }
                for i in 0..2 {
                    bel DLL_DCNTL[i];
                }
                bel PLL_SMI;
                bel SERDES_CORNER;
                if tile_class PLL_NW {
                    bel M0;
                    bel M1;
                    bel M2;
                    bel M3;
                    bel RESETN;
                    bel RDCFGN;
                }
                if tile_class PLL_NE {
                    bel CCLK;
                    bel TCK;
                    bel TMS;
                    bel TDI;
                }
            }
        } else if variant [ecp, xp] {
            bel_slot PLL[1]: PLL;
            tile_class PLL_W, PLL_E {
                cell CELL, IO_CLKI, IO_CLKFB;
                switchbox BEL_INT;
                bel PLL[0];
            }

            bel_slot DQSDLL: DQSDLL;
            tile_class DQSDLL_S, DQSDLL_N {
                cell CELL;
                bel DQSDLL;
            }
        } else if variant machxo {
            bel_slot PLL[1]: PLL;
            tile_class PLL_S, PLL_N {
                cell CELL;
                switchbox BEL_INT;
                bel PLL[0];
            }
        } else if variant [ecp2, ecp2m] {
            bel_slot PLL[1]: PLL;
            bel_slot DLL[1]: DLL;
            bel_slot DLLDEL[1]: DLLDEL;
            bel_slot CLKDIV[1]: CLKDIV;
            bel_slot DQSDLL: DQSDLL;
            tile_class PLL_W, PLL_E {
                cell CELL[4], IO_CLKI[2], IO_CLKFB[2];
                switchbox BEL_INT;
                bel PLL[0];
                bel DLL[0];
                bel DLLDEL[0];
                bel CLKDIV[0];
                bel DQSDLL;
            }

            tile_class SPLL_W, SPLL_E {
                cell CELL[2], IO_CLKI[2], IO_CLKFB;
                switchbox BEL_INT;
                bel PLL[0];
            }
        } else if variant xp2 {
            bel_slot PLL[1]: PLL;
            tile_class PLL_S, PLL_N {
                cell CELL[2], IO_CLKI, IO_CLKFB;
                switchbox BEL_INT;
                bel PLL[0];
            }

            bel_slot DQSDLL: DQSDLL;
            bel_slot CLKDIV[1]: CLKDIV;
            bel_slot START: START;
            bel_slot GSR: GSR;
            bel_slot SSPI: SSPI;
            bel_slot STF: STF;
            bel_slot WAKEUP: WAKEUP;
            tile_class CLK_W, CLK_E {
                cell CELL, CELL_PLL_S, CELL_PLL_N;
                switchbox BEL_INT;
                bel DQSDLL;
                bel CLKDIV[0];
                if tile_class CLK_E {
                    cell CELL_CLK;
                    bel SSPI;
                    bel STF;
                    bel WAKEUP;
                    bel START;
                    bel GSR;
                }
            }
        } else if variant ecp3 {
            bel_slot PLL[1]: legacy;
            bel_slot DLL[1]: legacy;
            bel_slot DLLDEL[1]: legacy;
            bel_slot DQSDLL: legacy;
            bel_slot DQSDLLTEST: legacy;
            bel_slot ECLK_ALT_ROOT: legacy;
            bel_slot CLKDIV[1]: legacy;
            tile_class PLL_DLL_W, PLL_DLL_E, PLL_DLL_A_W, PLL_DLL_A_E {
                cell CELL[15];
                cell CELL_WE[3];
                bel PLL[0];
                bel DLL[0];
                bel DLLDEL[0];
                bel DQSDLL;
                bel DQSDLLTEST;
                bel ECLK_ALT_ROOT;
                bel CLKDIV[0];
            }
            tile_class PLL_W, PLL_E, PLL_A_W, PLL_A_E {
                cell CELL[12];
                cell CELL_WE;
                bel PLL[0];
            }
        } else if variant machxo2 {
            bel_slot PLL[1]: legacy;
            bel_slot PLLREFCS[1]: legacy;
            tile_class PLL_W, PLL_E {
                cell CELL[2];
                bel PLL[0];
                bel PLLREFCS[0];
            }

            bel_slot DQSDLL: legacy;
            bel_slot DQSDLLTEST: legacy;
            tile_class DQSDLL_S, DQSDLL_N {
                cell CELL;
                bel DQSDLL;
                bel DQSDLLTEST;
            }
        } else if variant ecp4 {
            bel_slot PLL[2]: legacy;
            bel_slot PLLREFCS[2]: legacy;
            tile_class PLL_SW, PLL_SE, PLL_NW, PLL_NE {
                cell CELL[4];
                for i in 0..2 {
                    bel PLL[i];
                    bel PLLREFCS[i];
                }
            }

            bel_slot DDRDLL: legacy;
            tile_class DDRDLL_S, DDRDLL_N {
                cell CELL;
                bel DDRDLL;
            }

            bel_slot DTR: legacy;
            tile_class DTR_S, DTR_N {
                cell CELL;
                bel DTR;
            }
        } else if variant ecp5 {
            bel_slot PLL[1]: legacy;
            bel_slot PLLREFCS[1]: legacy;
            tile_class PLL_SW, PLL_SE, PLL_NW, PLL_NE {
                cell CELL[2];
                bel PLL[0];
                bel PLLREFCS[0];
            }

            bel_slot DDRDLL: legacy;
            tile_class DDRDLL {
                cell CELL;
                bel DDRDLL;
            }

            bel_slot DTR: legacy;
            tile_class DTR {
                cell CELL;
                bel DTR;
            }
        } else if variant crosslink {
            bel_slot PLL[1]: legacy;
            bel_slot PLLREFCS[1]: legacy;
            tile_class PLL {
                cell CELL[2];
                bel PLL[0];
                bel PLLREFCS[0];
            }

            bel_slot BCINRD: legacy;
            bel_slot BCLVDSO: legacy;
            bel_slot DDRDLL: legacy;
            tile_class BC {
                cell CELL;
                bel BCINRD;
                bel BCLVDSO;
                bel DDRDLL;
            }
        }

        if variant [ecp, xp, ecp2, ecp2m, xp2] {
            bel_slot DQS[1]: DQS;
            tile_class DQS_W, DQS_E, DQS_S, DQS_N {
                if tile_class DQS_N {
                    if variant [ecp2, ecp2m] {
                        skip;
                    }
                }
                cell CELL;
                bel DQS[0];
            }
        } else if variant ecp3 {
            bel_slot DQS[1]: DQS;
            tile_class DQS_N, DQS_W, DQS_E, DQS_A_W, DQS_A_E {
                if tile_class DQS_N {
                    cell CELL[6];
                } else {
                    cell CELL[3];
                }
                switchbox BEL_INT;
                bel DQS[0];
            }
        } else if variant ecp4 {
            bel_slot DQS[1]: legacy;
            tile_class
                DQS_W,
                DQS_W_BELOW_DSP_N,
                DQS_W_BELOW_EBR_N,
                DQS_W_BELOW_EBR_S,
                DQS_W_EBR_S,
                DQS_W_EBR_N,
                DQS_W_DSP_N,
                DQS_E,
                DQS_E_BELOW_DSP_N,
                DQS_E_BELOW_EBR_N,
                DQS_E_BELOW_EBR_S,
                DQS_E_EBR_S,
                DQS_E_EBR_N,
                DQS_E_DSP_N,
                DQS_N
            {
                cell CELL[5];
                bel DQS[0];
            }
        } else if variant ecp5 {
            bel_slot DQS[1]: legacy;
            tile_class DQS_W, DQS_E {
                cell CELL[6];
                bel DQS[0];
            }
        }

        // CONFIG etc.
        if variant scm {
            bel_slot JTAG: JTAG;
            bel_slot RDBK: legacy;
            bel_slot GSR: GSR;
            bel_slot OSC: OSC;
            bel_slot START: START;
            bel_slot SYSBUS: legacy;
            bel_slot SERDES_CENTER: legacy;
            tile_class CONFIG {
                cell CELL[12];
                cell CELL_CLK;
                bel JTAG;
                bel RDBK;
                bel GSR;
                bel OSC;
                bel START;
                bel SYSBUS;
                bel SERDES_CENTER;
            }
        } else if variant ecp {
            bel_slot START: START;
            bel_slot OSC: OSC;
            bel_slot JTAG: JTAG;
            bel_slot GSR: GSR;
            tile_class CONFIG_S, CONFIG_L {
                cell CELL[4];
                if tile_class CONFIG_L {
                    cell CELL_CLK;
                }
                bel START;
                bel OSC;
                bel JTAG;
                bel GSR;
                // RDBK exists as stub?
            }
        } else if variant xp {
            bel_slot START: START;
            bel_slot JTAG: JTAG;
            bel_slot GSR: GSR;
            tile_class CONFIG {
                cell CELL;
                bel START;
                bel JTAG;
                bel GSR;
                // RDBK and OSC exist as stubs?
            }
        } else if variant machxo {
            bel_slot OSC: OSC;
            tile_class OSC, OSC_X {
                cell CELL;
                bel OSC;
            }

            bel_slot JTAG: JTAG;
            bel_slot GSR: GSR;
            bel_slot TSALL: TSALL;
            tile_class CONFIG {
                cell CELL[5];
                bel JTAG;
                bel GSR;
                bel TSALL;
            }
        } else if variant [ecp2, ecp2m] {
            bel_slot START: START;
            bel_slot OSC: OSC;
            bel_slot JTAG: JTAG;
            bel_slot GSR: GSR;
            bel_slot SED: SED;
            bel_slot SPIM: SPIM;
            tile_class CONFIG {
                cell CELL[2];
                cell CELL_CLK;
                bel START;
                bel OSC;
                bel JTAG;
                bel GSR;
                bel SED;
                bel SPIM;
            }
        } else if variant xp2 {
            bel_slot JTAG: JTAG;
            bel_slot SED: SED;
            tile_class CONFIG {
                cell CELL;
                bel JTAG;
                bel SED;
            }

            bel_slot OSC: OSC;
            tile_class OSC {
                cell CELL;
                bel OSC;
            }

        } else if variant ecp3 {
            bel_slot START: START;
            bel_slot JTAG: JTAG;
            bel_slot OSC: OSC;
            bel_slot GSR: GSR;
            bel_slot SED: SED;
            bel_slot AMBOOT: AMBOOT;
            bel_slot PERREG: PERREG;
            tile_class CONFIG {
                cell CELL[13];
                cell CELL_CLK;
                bel START;
                bel JTAG;
                bel OSC;
                bel GSR;
                bel SED;
                bel AMBOOT;
                bel PERREG;
            }

            bel_slot TESTIN: TESTIN;
            bel_slot TESTOUT: TESTOUT;
            bel_slot DTS: DTS;
            tile_class TEST_SW, TEST_SE {
                cell CELL[3];
                bel TESTIN;
                bel TESTOUT;
                if tile_class TEST_SE {
                    bel DTS;
                }
            }

            tile_class TEST_NW, TEST_NE {
                cell CELL[2];
                bel TESTIN;
                bel TESTOUT;
            }
        } else if variant machxo2 {
            bel_slot START: START;
            bel_slot JTAG: JTAG;
            bel_slot OSC: OSC;
            bel_slot GSR: GSR;
            bel_slot SED: SED;
            bel_slot PCNTR: legacy;
            bel_slot TSALL: TSALL;
            bel_slot EFB: legacy;
            bel_slot ESB: legacy;
            tile_class CONFIG, CONFIG_XO3D {
                if tile_class CONFIG {
                    cell CELL[4];
                } else {
                    cell CELL[10];
                }
                bel START;
                bel JTAG;
                bel OSC;
                bel GSR;
                bel SED;
                bel PCNTR;
                bel TSALL;
                bel EFB;
                if tile_class CONFIG_XO3D {
                    bel ESB;
                }
            }
        } else if variant ecp4 {
            bel_slot START: START;
            bel_slot JTAG: JTAG;
            bel_slot OSC: OSC;
            bel_slot GSR: GSR;
            bel_slot SED: SED;
            bel_slot PCNTR: legacy;
            bel_slot EFB: legacy;
            tile_class CONFIG {
                cell CELL[6];
                cell CELL_CLK[5];
                bel START;
                bel JTAG;
                bel OSC;
                bel GSR;
                bel SED;
                bel PCNTR;
                bel EFB;
            }
        } else if variant ecp5 {
            bel_slot START: START;
            bel_slot JTAG: JTAG;
            bel_slot OSC: OSC;
            bel_slot GSR: GSR;
            bel_slot SED: SED;
            bel_slot CCLK: CCLK;
            tile_class CONFIG {
                cell CELL[3];
                cell CELL_CLK;
                bel START;
                bel JTAG;
                bel OSC;
                bel GSR;
                bel SED;
                bel CCLK;
            }
        } else if variant crosslink {
            bel_slot GSR: GSR;
            bel_slot CFGTEST: legacy;
            tile_class CONFIG {
                cell CELL[2];
                cell CELL_W;
                cell CELL_CLK;
                bel GSR;
                bel CFGTEST;
            }

            bel_slot OSC: OSC;
            tile_class OSC {
                cell CELL;
                bel OSC;
            }

            bel_slot PMU: legacy;
            bel_slot PMUTEST: legacy;
            tile_class PMU {
                cell CELL;
                bel PMU;
                bel PMUTEST;
            }

            bel_slot I2C: legacy;
            bel_slot NVCMTEST: legacy;
            tile_class I2C_W, I2C_E {
                cell CELL[2];
                bel I2C;
                if tile_class I2C_E {
                    bel NVCMTEST;
                }
            }
        }

        if variant ecp4 {
            bel_slot PVTTEST: legacy;
            bel_slot PVTCAL: legacy;
            tile_class PVTTEST {
                cell CELL;
                bel PVTTEST;
                bel PVTCAL;
            }
        }
    }

    if variant machxo2 {
        tile_slot BC {
            bel_slot BCPG: legacy;
            bel_slot BCINRD: legacy;
            bel_slot BCLVDSO: legacy;
            bel_slot BCSLEWRATE: legacy;
            tile_class BC, BC_N, BCSR_W, BCSR_E, BCSR_S, BCSR_N {
                if tile_class BCSR_E {
                    cell CELL[2];
                } else {
                    cell CELL;
                }
                bel BCPG;
                bel BCINRD;
                if tile_class [BC_N, BCSR_N] {
                    bel BCLVDSO;
                }
                if tile_class [BCSR_W, BCSR_E, BCSR_S, BCSR_N] {
                    bel BCSLEWRATE;
                }
            }
        }
    } else if variant ecp4 {
        tile_slot BC {
            bel_slot BCINRD: legacy;
            bel_slot BCPG: legacy;
            bel_slot BCLVDSO: legacy;
            bel_slot BCPUSL: legacy;
            bel_slot BREFTEST: legacy;
            tile_class BC_W, BC_E, BC_N {
                cell CELL;
                bel BCINRD;
                bel BCPG;
                bel BCLVDSO;
                bel BCPUSL;
                bel BREFTEST;
            }
        }
    } else if variant ecp5 {
        tile_slot BC {
            bel_slot BCINRD: legacy;
            bel_slot BCLVDSO: legacy;
            bel_slot BREFTEST: legacy;
            // yup, no bank 5
            tile_class BC0, BC1, BC2, BC3, BC4, BC6, BC7, BC8 {
                cell CELL;
                if tile_class [BC2, BC3, BC6, BC7] {
                    bel BCINRD;
                    bel BCLVDSO;
                }
                bel BREFTEST;
            }
        }
    }

    tile_slot CLK {
        bel_slot CLK_INT: routing;
        if variant machxo {
            tile_class CLK_ROOT_0PLL, CLK_ROOT_1PLL, CLK_ROOT_2PLL {
                cell CELL[6];
                if tile_class CLK_ROOT_1PLL {
                    cell PLL[1];
                } else if tile_class CLK_ROOT_2PLL {
                    cell PLL[2];
                }
                switchbox CLK_INT;
            }
        } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
            bel_slot DCS_SW[2]: DCS;
            bel_slot DCS_SE[2]: DCS;
            bel_slot DCS_NW[2]: DCS;
            bel_slot DCS_NE[2]: DCS;
            if variant [ecp, xp] {
                tile_class CLK_ROOT_2PLL, CLK_ROOT_2PLL_A, CLK_ROOT_2PLL_B, CLK_ROOT_4PLL {
                    if variant ecp {
                        if tile_class [CLK_ROOT_2PLL_A, CLK_ROOT_2PLL_B] {
                            skip;
                        }
                    } else {
                        if tile_class CLK_ROOT_2PLL {
                            skip;
                        }
                    }

                    // actual clock root cells
                    cell ROOT_SW, ROOT_SE, ROOT_NW, ROOT_NE;
                    // fabric clock inputs
                    cell PCLK_IN_W, PCLK_IN_E, PCLK_IN_S, PCLK_IN_N;
                    cell SCLK_IN_W[4], SCLK_IN_E[4], SCLK_IN_S[4], SCLK_IN_N[4];
                    // I/O inputs
                    cell IO_IN_W, IO_IN_E, IO_IN_S, IO_IN_N;
                    if variant ecp {
                        if tile_class CLK_ROOT_2PLL {
                            cell PLL_SW, PLL_SE;
                        } else {
                            cell PLL_SW, PLL_SE, PLL_NW, PLL_NE;
                            cell DCS_SEL[8];
                        }
                    } else {
                        if tile_class [CLK_ROOT_2PLL_A, CLK_ROOT_2PLL_B] {
                            cell PLL_SW[2], PLL_SE[2];
                        } else {
                            cell PLL_SW[2], PLL_SE[2], PLL_NW[2], PLL_NE[2];
                        }
                    }

                    switchbox CLK_INT;
                    for i in 0..2 {
                        bel DCS_SW[i];
                        bel DCS_SE[i];
                        bel DCS_NW[i];
                        bel DCS_NE[i];
                    }
                }
            } else {
                tile_class CLK_ROOT {
                    // actual clock root cells
                    cell ROOT_SW, ROOT_SE, ROOT_NW, ROOT_NE;
                    // fabric clock inputs
                    cell PCLK_IN_W[2], PCLK_IN_E[4], PCLK_IN_S[2], PCLK_IN_N[2];
                    cell SCLK_IN_W[4], SCLK_IN_E[4], SCLK_IN_S[4], SCLK_IN_N[4];
                    // I/O inputs
                    cell IO_W[2], IO_E[2], IO_S[2], IO_N[2];

                    if variant xp2 {
                        cell PLL_SW, PLL_SE, PLL_NW, PLL_NE;
                        cell CLK_W, CLK_E;
                    } else if variant ecp2 {
                        cell PLL_SW, PLL_SE, PLL_NW[2], PLL_NE[2];
                    } else if variant ecp2m {
                        cell PLL_SW[2], PLL_SE[2], PLL_NW[2], PLL_NE[2];
                        cell SERDES_SW, SERDES_SE, SERDES_NW, SERDES_NE;
                    }

                    switchbox CLK_INT;
                    for i in 0..2 {
                        bel DCS_SW[i];
                        bel DCS_SE[i];
                        bel DCS_NW[i];
                        bel DCS_NE[i];
                    }
                }
            }
        } else if variant ecp3 {
            bel_slot CLK_ROOT: legacy;
            bel_slot DCC_SW[6]: legacy;
            bel_slot DCC_SE[6]: legacy;
            bel_slot DCC_NW[6]: legacy;
            bel_slot DCC_NE[6]: legacy;
            bel_slot DCS_SW[2]: legacy;
            bel_slot DCS_SE[2]: legacy;
            bel_slot DCS_NW[2]: legacy;
            bel_slot DCS_NE[2]: legacy;
            tile_class CLK_ROOT {
                cell CELL[30];
                bel CLK_ROOT;
                for i in 0..6 {
                    bel DCC_SW[i];
                    bel DCC_SE[i];
                    bel DCC_NW[i];
                    bel DCC_NE[i];
                }
                for i in 0..2 {
                    bel DCS_SW[i];
                    bel DCS_SE[i];
                    bel DCS_NW[i];
                    bel DCS_NE[i];
                }
            }
        } else if variant machxo2 {
            bel_slot CLK_ROOT: legacy;
            bel_slot CLKTEST: legacy;
            bel_slot DCC[8]: legacy;
            bel_slot DCM[2]: legacy;
            bel_slot ECLKBRIDGECS[2]: legacy;
            tile_class CLK_ROOT_0EBR, CLK_ROOT_1EBR, CLK_ROOT_2EBR, CLK_ROOT_3EBR {
                if tile_class CLK_ROOT_0EBR {
                    cell EDGE_W[2];
                    cell EDGE_E[2];
                    cell EDGE_S[2];
                    cell EDGE_N[2];
                } else if tile_class [CLK_ROOT_1EBR, CLK_ROOT_2EBR] {
                    cell CELL[4];
                    cell EDGE_W, EDGE_E, EDGE_S, EDGE_N;
                    if tile_class CLK_ROOT_2EBR {
                        cell EBR[3];
                        cell EBR_W, EBR_E;
                    }
                } else {
                    cell CELL[2];
                    cell EDGE_W, EDGE_E, EDGE_S, EDGE_N;
                    cell EBR_S[3];
                    cell EBR_N[3];
                    cell EBR_SW, EBR_SE;
                }
                bel CLK_ROOT;
                bel CLKTEST;
                for i in 0..8 {
                    bel DCC[i];
                }
                for i in 0..2 {
                    bel DCM[i];
                }
                if tile_class CLK_ROOT_0EBR {
                    // nothing
                } else {
                    for i in 0..2 {
                        bel ECLKBRIDGECS[i];
                    }
                }
            }

            bel_slot DLLDEL[3]: legacy;
            bel_slot CLKDIV[4]: legacy;
            bel_slot DQS[2]: legacy;
            bel_slot ECLKSYNC[2]: legacy;
            bel_slot CLKFBBUF[2]: legacy;
            tile_class CLK_S, CLK_N {
                cell CELL;
                for i in 0..2 {
                    bel ECLKSYNC[i];
                    bel DLLDEL[i];
                    bel CLKDIV[i];
                    if tile_class CLK_S {
                        bel CLKFBBUF[i];
                    }
                }
            }
            tile_class CLK_W {
                cell CELL;
                for i in 0..3 {
                    bel DLLDEL[i];
                }
            }
            tile_class CLK_E {
                cell CELL;
                bel DLLDEL[0];
            }
            tile_class CLK_E_DQS {
                cell CELL[5];
                bel DLLDEL[0];
                for i in 0..2 {
                    bel DQS[i];
                }
            }
        } else if variant scm {
            bel_slot ECLK_ROOT: legacy;
            bel_slot CLKDIV[4]: legacy;
            bel_slot DCS[2]: legacy;
            bel_slot CLK_EDGE: legacy;
            bel_slot CLK_ROOT: legacy;
            bel_slot CLKTEST: legacy;
            tile_class CLK_W, CLK_E, CLK_N {
                cell CELL[2];
                bel ECLK_ROOT;
                for i in 0..4 {
                    bel CLKDIV[i];
                }
                for i in 0..2 {
                    bel DCS[i];
                }
                bel CLK_EDGE;
            }
            tile_class CLK_S {
                cell CELL[2];
                for i in 0..2 {
                    bel DCS[i];
                }
                bel CLK_EDGE;
            }
            tile_class CLK_SW, CLK_SE {
                cell CELL[2];
                bel ECLK_ROOT;
                for i in 0..4 {
                    bel CLKDIV[i];
                }
            }
            tile_class CLK_ROOT {
                cell CELL_S[2];
                cell CELL_N[2];
                bel CLK_ROOT;
                bel CLKTEST;
            }
        } else if variant ecp4 {
            bel_slot CLK_ROOT: legacy;
            bel_slot CLKTEST: legacy;
            bel_slot DCC_SW[1]: legacy;
            bel_slot DCC_SE[1]: legacy;
            bel_slot DCC_NW[1]: legacy;
            bel_slot DCC_NE[1]: legacy;
            bel_slot DCS[2]: legacy;
            tile_class CLK_ROOT {
                cell ROOT_SW, ROOT_SE, ROOT_NW, ROOT_NE;
                bel CLK_ROOT;
                bel CLKTEST;
                bel DCC_SW[0];
                bel DCC_SE[0];
                bel DCC_NW[0];
                bel DCC_NE[0];
                for i in 0..2 {
                    bel DCS[i];
                }
            }

            bel_slot CLK_EDGE: legacy;
            bel_slot CLKTEST_ECLK: legacy;
            bel_slot DCC[16]: legacy;
            bel_slot DLLDEL[8]: legacy;
            bel_slot CLKDIV[4]: legacy;
            bel_slot ECLKSYNC[16]: legacy;
            bel_slot BRGECLKSYNC[2]: legacy;
            bel_slot ECLKBRIDGECS[2]: legacy;
            tile_class
                CLK_N_S, CLK_N_M, CLK_N_L,
                CLK_W_S, CLK_W_M, CLK_W_L,
                CLK_E_S, CLK_E_M, CLK_E_L
            {
                if tile_class CLK_N_L {
                    cell CELL[16];
                    cell BC1, BC2;
                } else {
                    cell CELL[8];
                }
                if tile_class [CLK_N_M, CLK_W_M, CLK_E_M] {
                    cell FABRIC_IN[2];
                } else if tile_class [CLK_N_L, CLK_W_L, CLK_E_L] {
                    cell FABRIC_IN[4];
                }
                bel CLKTEST;
                bel CLK_EDGE;
                bel CLKTEST_ECLK;
                if tile_class [CLK_N_S, CLK_N_M, CLK_N_L] {
                    for i in 0..16 {
                        bel DCC[i];
                    }
                    if tile_class CLK_N_L {
                        for i in 0..16 {
                            bel ECLKSYNC[i];
                        }
                        for i in 0..8 {
                            bel DLLDEL[i];
                        }
                    } else {
                        for i in 4..12 {
                            bel ECLKSYNC[i];
                        }
                        for i in 2..6 {
                            bel DLLDEL[i];
                        }
                    }
                } else {
                    for i in 0..14 {
                        bel DCC[i];
                    }
                    for i in 0..2 {
                        bel BRGECLKSYNC[i];
                        bel ECLKBRIDGECS[i];
                    }
                    for i in 0..8 {
                        bel ECLKSYNC[i];
                    }
                    for i in 0..4 {
                        bel DLLDEL[i];
                    }
                }
                for i in 0..4 {
                    bel CLKDIV[i];
                }
            }

            bel_slot PCSCLKDIV[4]: legacy;
            tile_class CLK_S_S, CLK_S_M, CLK_S_L {
                cell CELL[2];
                if tile_class CLK_S_S {
                    bel PCSCLKDIV[0];
                } else if tile_class CLK_S_M {
                    cell FABRIC_IN[2];
                    for i in 0..2 {
                        bel PCSCLKDIV[i];
                    }
                } else if tile_class CLK_S_L {
                    cell FABRIC_IN[4];
                    for i in 0..3 {
                        bel PCSCLKDIV[i];
                    }
                }
                bel CLK_EDGE;
                bel CLKTEST;
                for i in 0..16 {
                    bel DCC[i];
                }
            }
        } else if variant ecp5 {
            bel_slot CLK_ROOT: legacy;
            bel_slot CLKTEST: legacy;
            bel_slot DCC_SW[1]: legacy;
            bel_slot DCC_SE[1]: legacy;
            bel_slot DCC_NW[1]: legacy;
            bel_slot DCC_NE[1]: legacy;
            bel_slot DCS[2]: legacy;
            tile_class CLK_ROOT_S, CLK_ROOT_L {
                if tile_class CLK_ROOT_S {
                    cell SW, SE, NW, NE;
                } else {
                    cell QSW, QSE, SW, SE, NW, NE, QNW, QNE;
                }
                bel CLK_ROOT;
                bel CLKTEST;
                bel DCC_SW[0];
                bel DCC_SE[0];
                bel DCC_NW[0];
                bel DCC_NE[0];
                for i in 0..2 {
                    bel DCS[i];
                }
            }

            bel_slot DCC[16]: legacy;
            bel_slot CLK_EDGE: legacy;
            bel_slot CLKTEST_ECLK: legacy;
            bel_slot ECLKSYNC[4]: legacy;
            bel_slot DLLDEL[4]: legacy;
            bel_slot CLKDIV[2]: legacy;
            bel_slot PCSCLKDIV[2]: legacy;
            bel_slot BRGECLKSYNC[1]: legacy;
            bel_slot ECLKBRIDGECS[1]: legacy;
            tile_class CLK_W_S, CLK_W_L, CLK_E_S, CLK_E_L {
                cell CELL_EDGE[4];
                cell CELL_EBR[2];
                if tile_class [CLK_W_S, CLK_E_S] {
                    cell FABRIC_IN[6];
                } else {
                    cell FABRIC_IN[8];
                }
                for i in 0..14 {
                    bel DCC[i];
                }
                bel CLKTEST;
                bel CLK_EDGE;
                bel CLKTEST_ECLK;
                for i in 0..4 {
                    bel ECLKSYNC[i];
                    bel DLLDEL[i];
                }
                for i in 0..2 {
                    bel CLKDIV[i];
                }
                bel BRGECLKSYNC[0];
                bel ECLKBRIDGECS[0];
            }
            tile_class CLK_N_S, CLK_N_L {
                cell CELL[4];
                if tile_class CLK_N_S {
                    cell FABRIC_IN[6];
                } else {
                    cell FABRIC_IN[8];
                }
                for i in 0..12 {
                    bel DCC[i];
                }
                bel CLKTEST;
                bel CLK_EDGE;
                for i in 0..4 {
                    bel DLLDEL[i];
                }
            }
            tile_class CLK_S_S, CLK_S_L {
                cell CELL[2];
                if tile_class CLK_S_S {
                    cell FABRIC_IN[6];
                } else {
                    cell FABRIC_IN[8];
                }
                for i in 0..16 {
                    bel DCC[i];
                }
                bel CLKTEST;
                bel CLK_EDGE;
                for i in 0..2 {
                    bel PCSCLKDIV[i];
                }
            }
        } else if variant crosslink {
            bel_slot CLK_ROOT: legacy;
            bel_slot CLKTEST: legacy;
            bel_slot DCS[1]: legacy;
            tile_class CLK_ROOT {
                cell CELL;
                bel CLK_ROOT;
                bel CLKTEST;
                bel DCS[0];
            }

            bel_slot DCC[8]: legacy;
            bel_slot CLK_EDGE: legacy;
            bel_slot ECLKSYNC[4]: legacy;
            bel_slot DLLDEL[4]: legacy;
            bel_slot CLKDIV[4]: legacy;
            bel_slot CLKTEST_ECLK: legacy;
            tile_class CLK_S {
                cell CELL[4];
                cell FABRIC_IN[2];
                for i in 0..8 {
                    bel DCC[i];
                }
                bel CLKTEST;
                bel CLK_EDGE;
                for i in 0..4 {
                    bel ECLKSYNC[i];
                    bel DLLDEL[i];
                    bel CLKDIV[i];
                }
                bel CLKTEST_ECLK;
            }
            tile_class CLK_N {
                cell CELL[2];
                for i in 0..6 {
                    bel DCC[i];
                }
                bel CLKTEST;
                bel CLK_EDGE;
            }
        }


        if variant [ecp2, ecp2m, xp2] {
            tile_class ECLK_ROOT_W, ECLK_ROOT_E {
                cell CELL, IO[2];
                if variant [ecp2, ecp2m] {
                    cell PLL;
                } else {
                    cell PLL_S, PLL_N, PLL_IO_S, PLL_IO_N;
                }
                switchbox CLK_INT;
            }
            tile_class ECLK_ROOT_S, ECLK_ROOT_N {
                cell CELL[2], IO[2];
                switchbox CLK_INT;
            }
        }

        if variant ecp3 {
            bel_slot ECLKSYNC[2]: legacy;
            tile_class ECLK_ROOT_W, ECLK_ROOT_E, ECLK_ROOT_N {
                cell CELL;
                for i in 0..2 {
                    bel ECLKSYNC[i];
                }
            }
        }
    }

    if variant [ecp3, machxo2] {
        tile_slot PCLK_SOURCE {
            bel_slot PCLK_SOURCE_W: legacy;
            bel_slot PCLK_SOURCE_E: legacy;
            bel_slot PCLK_DCC[2]: legacy;

            for i in 0..4 {
                if variant machxo2 {
                    tile_class "PCLK{i}_SOURCE_IO_N", "PCLK{i}_SOURCE_N" {
                        cell CELL;
                        for j in 0..2 {
                            bel PCLK_DCC[j];
                        }
                    }
                    tile_class "PCLK{i}_SOURCE_IO_N_W", "PCLK{i}_SOURCE_N_W" {
                        if tile_class [PCLK0_SOURCE_IO_N_W, PCLK3_SOURCE_IO_N_W, PCLK1_SOURCE_N_W] {
                            // keep
                        } else {
                            skip;
                        }
                        cell CELL;
                        bel PCLK_SOURCE_W;
                        for j in 0..2 {
                            bel PCLK_DCC[j];
                        }
                    }
                    tile_class "PCLK{i}_SOURCE_IO_N_E", "PCLK{i}_SOURCE_N_E" {
                        if tile_class [PCLK0_SOURCE_IO_N_E, PCLK1_SOURCE_IO_N_E, PCLK1_SOURCE_N_E] {
                            // keep
                        } else {
                            skip;
                        }
                        cell CELL;
                        bel PCLK_SOURCE_E;
                        for j in 0..2 {
                            bel PCLK_DCC[j];
                        }
                    }
                }

                tile_class "PCLK{i}_SOURCE" {
                    cell N, S;
                    for j in 0..2 {
                        bel PCLK_DCC[j];
                    }
                }
                tile_class "PCLK{i}_SOURCE_W" {
                    if variant ecp3 {
                        if tile_class PCLK2_SOURCE_W {
                            skip;
                        }
                    }
                    cell N, S;
                    bel PCLK_SOURCE_W;
                    for j in 0..2 {
                        bel PCLK_DCC[j];
                    }
                }
                tile_class "PCLK{i}_SOURCE_E" {
                    if variant ecp3 {
                        if tile_class PCLK1_SOURCE_E {
                            skip;
                        }
                    } else {
                        if tile_class PCLK2_SOURCE_E {
                            skip;
                        }
                    }
                    cell N, S;
                    bel PCLK_SOURCE_E;
                    for j in 0..2 {
                        bel PCLK_DCC[j];
                    }
                }
            }
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        tile_slot SCLK_SOURCE {
            bel_slot SCLK_SOURCE: routing;
            for i in 0..4 {
                tile_class "SCLK{i}_SOURCE" {
                    cell CELL;
                    switchbox SCLK_SOURCE {
                        permabuf RSCLK[i] = RSCLK_V[0];
                        permabuf RSCLK[i + 4] = RSCLK_V[1];
                    }
                }
                if variant [ecp3, machxo2] {
                    tile_class "SCLK{i}_SOURCE_W" {
                        if variant ecp3 {
                            if tile_class SCLK2_SOURCE_W {
                                skip;
                            }
                        }
                        cell CELL;
                        switchbox SCLK_SOURCE {
                            permabuf RSCLK[i] = RSCLK_V[0];
                            permabuf RSCLK[i + 4] = RSCLK_V[1];
                            permabuf RSCLK[(i + 3) % 4] = RSCLK_V[6];
                            permabuf RSCLK[(i + 3) % 4 + 4] = RSCLK_V[7];
                            permabuf RSCLK[(i + 2) % 4] = RSCLK_V[4];
                            permabuf RSCLK[(i + 2) % 4 + 4] = RSCLK_V[5];
                        }
                    }
                    tile_class "SCLK{i}_SOURCE_E" {
                        if variant ecp3 {
                            if tile_class SCLK1_SOURCE_E {
                                skip;
                            }
                        }
                        cell CELL;
                        switchbox SCLK_SOURCE {
                            permabuf RSCLK[i] = RSCLK_V[0];
                            permabuf RSCLK[i + 4] = RSCLK_V[1];
                            permabuf RSCLK[(i + 1) % 4] = RSCLK_V[2];
                            permabuf RSCLK[(i + 1) % 4 + 4] = RSCLK_V[3];
                        }
                    }
                }
            }
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3] {
        tile_slot ECLK_TAP {
            bel_slot ECLK_TAP: routing;
            tile_class ECLK_TAP {
                cell CELL;
                switchbox ECLK_TAP {
                    permabuf OUT_F[6] = ECLK[0];
                    permabuf OUT_F[7] = ECLK[1];
                }
            }
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        tile_slot HSDCLK_SPLITTER {
            if variant [ecp3, machxo2] {
                bel_slot HSDCLK_ROOT: HSDCLK_ROOT;
            }
            bel_slot HSDCLK_SPLITTER: routing;

            tile_class HSDCLK_SPLITTER, HSDCLK_ROOT {
                if variant machxo2 {
                    if tile_class HSDCLK_SPLITTER {
                        skip;
                    }
                }
                cell W[4], E[4];
                switchbox HSDCLK_SPLITTER {
                    for i in 0..2 {
                        for j in 0..4 {
                            progbuf W[j].RSCLK_H[i * 4] = E[j].RSCLK_H[i * 4];
                            progbuf E[j].RSCLK_H[i * 4] = W[j].RSCLK_H[i * 4];
                        }
                    }
                    if tile_class HSDCLK_ROOT {
                        if variant [ecp2, ecp2m, xp2] {
                            for i in 0..8 {
                                progbuf W[i % 4].RSCLK_H[i / 4 * 4] = W[0].SCLK_ROOT[i];
                                progbuf E[i % 4].RSCLK_H[i / 4 * 4] = E[0].SCLK_ROOT[i];
                            }
                        }
                    }
                }
                if tile_class HSDCLK_ROOT {
                    if variant [ecp3, machxo2] {
                        bel HSDCLK_ROOT {
                            for i in 0..8 {
                                output OUT_W[i] = W[i % 4].RSCLK_H[i / 4 * 4];
                                output OUT_E[i] = E[i % 4].RSCLK_H[i / 4 * 4];
                            }
                        }
                    }
                }
            }
        }
    }

    connector_slot W {
        opposite E;
        connector_class PASS_W {
            pass X1_E1 = X1_E0;
            pass X2_E1 = X2_E0;
            pass X2_E2 = X2_E1;
            for i in 0..6 {
                pass "X6_E{i+1}" = "X6_E{i}";
            }

            if variant scm {
                pass DSCLK_E = DSCLK;
                pass DSCLK_H[6] = DSCLK_H[5];
                pass DSCLK_H[5] = DSCLK_H[4];
                pass DSCLK_H[4] = DSCLK_H[3];
            }

            pass OUT_F_E = OUT_F;
        }
        connector_class TERM_W {
            if variant [ecp5, crosslink] {
                reflect X1_E1 = X1_W0;
            }
            reflect X2_E1 = X2_W0;
            reflect X2_E2 = X2_W1;
            for i in 0..6 {
                reflect "X6_E{i+1}" = "X6_W{i}";
            }

            if variant scm {
                reflect DSCLK_H[6] = DSCLK_H[1];
                reflect DSCLK_H[5] = DSCLK_H[2];
                reflect DSCLK_H[4] = DSCLK_H[3];
            }
        }
    }
    connector_slot E {
        opposite W;
        connector_class PASS_E {
            pass X1_W1 = X1_W0;
            pass X2_W1 = X2_W0;
            pass X2_W2 = X2_W1;
            for i in 0..6 {
                pass "X6_W{i+1}" = "X6_W{i}";
            }

            if variant scm {
                pass DSCLK_W = DSCLK;
                pass DSCLK_H[0] = DSCLK_H[1];
                pass DSCLK_H[1] = DSCLK_H[2];
                pass DSCLK_H[2] = DSCLK_H[3];
            }

            pass OUT_F_W = OUT_F;
            if variant [ecp5, crosslink] {
            } else {
                pass OUT_OFX_W = OUT_OFX;
            }
        }
        connector_class TERM_E {
            if variant [ecp5, crosslink] {
                reflect X1_W1 = X1_E0;
            }
            reflect X2_W1 = X2_E0;
            reflect X2_W2 = X2_E1;
            for i in 0..6 {
                reflect "X6_W{i+1}" = "X6_E{i}";
            }

            if variant scm {
                reflect DSCLK_H[0] = DSCLK_H[5];
                reflect DSCLK_H[1] = DSCLK_H[4];
                reflect DSCLK_H[2] = DSCLK_H[3];
            }
        }
    }
    connector_slot S {
        opposite N;
        connector_class PASS_S {
            pass X1_N1 = X1_N0;
            pass X2_N1 = X2_N0;
            pass X2_N2 = X2_N1;
            for i in 0..6 {
                pass "X6_N{i+1}" = "X6_N{i}";
            }

            if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
                pass RSCLK_V_N = RSCLK_V;
            }

            if variant scm {
                for i in 0..6 {
                    pass DSCLK_V[i + 1] = DSCLK_V[i];
                }
            }
        }
        connector_class TERM_S {
            if variant [ecp5, crosslink] {
                reflect X1_N1 = X1_S0;
            }
            reflect X2_N1 = X2_S0;
            reflect X2_N2 = X2_S1;
            for i in 0..6 {
                reflect "X6_N{i+1}" = "X6_S{i}";
            }
        }
    }
    connector_slot N {
        opposite S;
        connector_class PASS_N {
            pass X1_S1 = X1_S0;
            pass X2_S1 = X2_S0;
            pass X2_S2 = X2_S1;
            for i in 0..6 {
                pass "X6_S{i+1}" = "X6_S{i}";
            }
        }
        connector_class TERM_N {
            if variant [ecp5, crosslink] {
                reflect X1_S1 = X1_N0;
            }
            reflect X2_S1 = X2_N0;
            reflect X2_S2 = X2_N1;
            for i in 0..6 {
                reflect "X6_S{i+1}" = "X6_N{i}";
            }
        }
    }

    if variant machxo {
        connector_slot IO_FROM_PLC {
            opposite PLC_TO_IO;
            connector_class IO_FROM_PLC {
                for i in 0..6 {
                    pass OUT_F_PLC_IO[i] = OUT_F[i];
                    pass OUT_Q_PLC_IO[i] = OUT_Q[i];
                }
            }
        }

        connector_slot PLC_TO_IO {
            opposite IO_FROM_PLC;
            connector_class PLC_TO_IO;
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        connector_slot RSCLK_W {
            opposite RSCLK_E;
            connector_class PASS_SW {
                if variant [ecp2, ecp2m, xp2] {
                    pass RSCLK_H[0] = RSCLK_H[1];
                    pass RSCLK_H[1] = RSCLK_H[2];
                    pass RSCLK_H[2] = RSCLK_H[3];
                    pass RSCLK_H[3] = RSCLK_H[0];
                    pass RSCLK_H[4] = RSCLK_H[5];
                    pass RSCLK_H[5] = RSCLK_H[6];
                    pass RSCLK_H[6] = RSCLK_H[7];
                    pass RSCLK_H[7] = RSCLK_H[4];
                } else if variant [ecp3, machxo2] {
                    pass RSCLK_H[0] = RSCLK_H[3];
                    pass RSCLK_H[1] = RSCLK_H[0];
                    pass RSCLK_H[2] = RSCLK_H[1];
                    pass RSCLK_H[3] = RSCLK_H[2];
                    pass RSCLK_H[4] = RSCLK_H[7];
                    pass RSCLK_H[5] = RSCLK_H[4];
                    pass RSCLK_H[6] = RSCLK_H[5];
                    pass RSCLK_H[7] = RSCLK_H[6];
                }
            }
        }
        connector_slot RSCLK_E {
            opposite RSCLK_W;
            connector_class PASS_SE {
            }
        }
    }

    if variant scm {
        connector_slot EBR_W {
            opposite EBR_E;
            connector_class PASS_EBR_W_W {
                pass EBR_E1 = EBR_E0;
            }
            connector_class PASS_EBR_M_W {
                for i in 0..24 {
                    pass EBR_E1[i] = EBR_E0[i];
                    pass EBR_E1[i + 24] = EBR_E0[i + 40];
                }
            }
            connector_class PASS_EBR_E_W {
                for i in 0..48 {
                    pass EBR_E1[i] = EBR_E0[i];
                }
            }
        }
        connector_slot EBR_E {
            opposite EBR_W;
            connector_class PASS_EBR_W_E {
                for i in 0..48 {
                    pass EBR_W1[i] = EBR_W0[i];
                }
            }
            connector_class PASS_EBR_M_E {
                for i in 0..24 {
                    pass EBR_W1[i] = EBR_W0[i];
                    pass EBR_W1[i + 24] = EBR_W0[i + 40];
                }
            }
            connector_class PASS_EBR_E_E {
                pass EBR_W1 = EBR_W0;
            }
        }

        connector_slot IO_W {
            opposite IO_E;
            connector_class PASS_IO_W {
                for i in 0..8 {
                    pass "IO_E{i+1}" = "IO_E{i}";
                }
                pass IO_T_E = IO_T_E;
            }
        }
        connector_slot IO_E {
            opposite IO_W;
            connector_class PASS_IO_E {
                for i in 0..8 {
                    pass "IO_W{i+1}" = "IO_W{i}";
                }
                pass IO_T_W = IO_T_W;
            }
        }
    }
}

pub mod wiredata {
    use prjcombine_entity::id::EntityStaticRange;
    use prjcombine_interconnect::{
        db::{ConnectorSlotId, WireSlotId},
        dir::DirMap,
    };

    use crate::defs::{cslots, wires};

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
    pub const X6_W: [EntityStaticRange<WireSlotId, 8>; 7] = [
        wires::X6_W0,
        wires::X6_W1,
        wires::X6_W2,
        wires::X6_W3,
        wires::X6_W4,
        wires::X6_W5,
        wires::X6_W6,
    ];
    pub const X6_E: [EntityStaticRange<WireSlotId, 8>; 7] = [
        wires::X6_E0,
        wires::X6_E1,
        wires::X6_E2,
        wires::X6_E3,
        wires::X6_E4,
        wires::X6_E5,
        wires::X6_E6,
    ];
    pub const X6_S: [EntityStaticRange<WireSlotId, 8>; 7] = [
        wires::X6_S0,
        wires::X6_S1,
        wires::X6_S2,
        wires::X6_S3,
        wires::X6_S4,
        wires::X6_S5,
        wires::X6_S6,
    ];
    pub const X6_N: [EntityStaticRange<WireSlotId, 8>; 7] = [
        wires::X6_N0,
        wires::X6_N1,
        wires::X6_N2,
        wires::X6_N3,
        wires::X6_N4,
        wires::X6_N5,
        wires::X6_N6,
    ];

    pub const OUT_MACO_IO_S: [EntityStaticRange<WireSlotId, 32>; 8] = [
        wires::OUT_MACO_IO_S0,
        wires::OUT_MACO_IO_S1,
        wires::OUT_MACO_IO_S2,
        wires::OUT_MACO_IO_S3,
        wires::OUT_MACO_IO_S4,
        wires::OUT_MACO_IO_S5,
        wires::OUT_MACO_IO_S6,
        wires::OUT_MACO_IO_S7,
    ];
    pub const OUT_MACO_IO_N: [EntityStaticRange<WireSlotId, 32>; 8] = [
        wires::OUT_MACO_IO_N0,
        wires::OUT_MACO_IO_N1,
        wires::OUT_MACO_IO_N2,
        wires::OUT_MACO_IO_N3,
        wires::OUT_MACO_IO_N4,
        wires::OUT_MACO_IO_N5,
        wires::OUT_MACO_IO_N6,
        wires::OUT_MACO_IO_N7,
    ];
    pub const IO_W: [EntityStaticRange<WireSlotId, 32>; 9] = [
        wires::IO_W0,
        wires::IO_W1,
        wires::IO_W2,
        wires::IO_W3,
        wires::IO_W4,
        wires::IO_W5,
        wires::IO_W6,
        wires::IO_W7,
        wires::IO_W8,
    ];
    pub const IO_E: [EntityStaticRange<WireSlotId, 32>; 9] = [
        wires::IO_E0,
        wires::IO_E1,
        wires::IO_E2,
        wires::IO_E3,
        wires::IO_E4,
        wires::IO_E5,
        wires::IO_E6,
        wires::IO_E7,
        wires::IO_E8,
    ];

    pub const CONNS: DirMap<ConnectorSlotId> = DirMap {
        w: cslots::W,
        e: cslots::E,
        s: cslots::S,
        n: cslots::N,
    };
}

pub mod tiledata {
    use prjcombine_interconnect::db::TileClassId;

    use crate::defs::tcls;

    pub const PCLK_SOURCE: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE,
        tcls::PCLK1_SOURCE,
        tcls::PCLK2_SOURCE,
        tcls::PCLK3_SOURCE,
    ];
    pub const PCLK_SOURCE_W: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_W,
        tcls::PCLK1_SOURCE_W,
        tcls::PCLK2_SOURCE_W,
        tcls::PCLK3_SOURCE_W,
    ];
    pub const PCLK_SOURCE_E: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_E,
        tcls::PCLK1_SOURCE_E,
        tcls::PCLK2_SOURCE_E,
        tcls::PCLK3_SOURCE_E,
    ];

    pub const PCLK_SOURCE_N: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_N,
        tcls::PCLK1_SOURCE_N,
        tcls::PCLK2_SOURCE_N,
        tcls::PCLK3_SOURCE_N,
    ];
    pub const PCLK_SOURCE_N_W: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_N_W,
        tcls::PCLK1_SOURCE_N_W,
        tcls::PCLK2_SOURCE_N_W,
        tcls::PCLK3_SOURCE_N_W,
    ];
    pub const PCLK_SOURCE_N_E: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_N_E,
        tcls::PCLK1_SOURCE_N_E,
        tcls::PCLK2_SOURCE_N_E,
        tcls::PCLK3_SOURCE_N_E,
    ];

    pub const PCLK_SOURCE_IO_N: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_IO_N,
        tcls::PCLK1_SOURCE_IO_N,
        tcls::PCLK2_SOURCE_IO_N,
        tcls::PCLK3_SOURCE_IO_N,
    ];
    pub const PCLK_SOURCE_IO_N_W: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_IO_N_W,
        tcls::PCLK1_SOURCE_IO_N_W,
        tcls::PCLK2_SOURCE_IO_N_W,
        tcls::PCLK3_SOURCE_IO_N_W,
    ];
    pub const PCLK_SOURCE_IO_N_E: [TileClassId; 4] = [
        tcls::PCLK0_SOURCE_IO_N_E,
        tcls::PCLK1_SOURCE_IO_N_E,
        tcls::PCLK2_SOURCE_IO_N_E,
        tcls::PCLK3_SOURCE_IO_N_E,
    ];

    pub const SCLK_SOURCE: [TileClassId; 4] = [
        tcls::SCLK0_SOURCE,
        tcls::SCLK1_SOURCE,
        tcls::SCLK2_SOURCE,
        tcls::SCLK3_SOURCE,
    ];
    pub const SCLK_SOURCE_W: [TileClassId; 4] = [
        tcls::SCLK0_SOURCE_W,
        tcls::SCLK1_SOURCE_W,
        tcls::SCLK2_SOURCE_W,
        tcls::SCLK3_SOURCE_W,
    ];
    pub const SCLK_SOURCE_E: [TileClassId; 4] = [
        tcls::SCLK0_SOURCE_E,
        tcls::SCLK1_SOURCE_E,
        tcls::SCLK2_SOURCE_E,
        tcls::SCLK3_SOURCE_E,
    ];
}
