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
    variant nexus;

    bel_class SLICE {
        input A[2], B[2], C[2], D[2];
        input M[2];
        input FXA, FXB;
        input CLK, LSR, CE;
        if variant [ecp5, crosslink] {
            input WCK, WRE;
        }
        output F[2], Q[2];
        if variant [ecp5, crosslink, nexus] {
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
        if variant nexus {
            input DWS[5];
            output AFULL, AEMPTY, FULLF, EMPTYF;
            output ONEERR, TWOERR;
        }
    }

    if variant nexus {
        bel_class LRAM {
            input CLK;
            input RSTA, RSTB;
            input CEA, CEB;
            input WEA, WEB;
            input OCEA, OCEB;
            input CSA, CSB;
            input ADA[14], ADB[14];
            input BENA_N[4], BENB_N[4];
            input DIA[32], DIB[32];
            output DOA[32], DOB[32];
            input OPCGLOADCLK;
            input DPS, IGN, INITN, STDBYN;
            input ERRDECA[2], ERRDECB[2];
            output OEA, OEB;
            output ERRDET;
            output LRAMREADY;

            input SCANCLK;
            input SCANRST;
            input TBISTN;
        }
    }

    if variant scm {
        bel_class MACO {
            input ECLK[4];
            for i in 0..3 {
                input "CLK_{i}"[4];
            }
            input LSR_0[2];
            input LSR_1[4];
            input LSR_2[4];
            for i in 0..3 {
                input "CE_{i}"[4];
            }
            for i in 0..3 {
                input "CIBIN_{i}"[32];
            }
            output CIBOUT_0[22];
            output CIBOUT_1[24];
            output CIBOUT_2[24];
            for i in 0..10 {
                input "EBRI{i}"[48];
                output "EBRO{i}"[64];
            }
            for i in 0..8 {
                input "SIPI{i}"[32];
                output "SIPO{i}"[32];
            }
            for i in 0..8 {
                input "NIPI{i}"[32];
                output "NIPO{i}"[32];
            }
            output TIP[2];
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
    } else if variant nexus {
        bel_class DSP {
            input MULT_CLK[2];
            input RSTA[2], RSTB[2], RSTCL[2], RSTP[2];
            input CEA[2], CEB[2], CECL[2], CEP[2];
            input A0[18], A1[18], B0[18], B1[18];
            input SIGNEDA[4], SIGNEDB[4];
            input MULT_ROUNDEN[2];
            input MULT18X36_ROUNDEN;
            input MULT0_SFTCTRL[4], MULT1_SFTCTRL[4];
            input MULT18X36_SFTCTRL[4];

            input ACC_CLK;
            input RSTC, RSTO, RSTCIN, RSTCTRL;
            input CEC, CEO, CECIN, CECTRL;
            input C[54];
            input ACC_SIGNEDI;
            input CIN;
            input ADDSUB[2];
            input M9ADDSUB[2];
            input LOAD;
            input ACC_ROUNDEN;
            input ACC_SFTCTRL[4];

            output SUM0[36], SUM1[36];
        }
    }

    if variant scm {
        bel_class IO {
            input CLK, LSR, CE;
            input ECLK, ELSR;
            input OPOS[4];
            input ONEG[4];
            input TD;
            output IPOS[4];
            output INEG[4];
            output INFF;
            output INDDCK;
            output UP;
            input RUNAIL;
            output LOCK;
        }

        bel_class PICTEST {
            input CLK;
            input SCANENABLE;
            input SCANSEL[2];
            output SCANOUT;
        }
    } else if variant machxo {
        bel_class IO {
            input PADDO, PADDT;
            output PADDI;
            output INCK;

            pad PAD: inout;
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
            output DI, INCK;
            if variant [ecp2, ecp2m, xp2] {
                input ECLKI, ECLKO;
                input DEL[4];
                output QPOS[2];
                // IO[0] also has QNEG[0..2]; however, it is multiplexed onto IO[1]'s QPOS
                // and not an independent output
            }
            pad PAD: inout;
        }

        bel_class DQS {
            input CLK;
            input READ[1];
            output DDRCLKPOL;
            output PRMBDET;
            if variant [ecp2, ecp2m, xp2] {
                output DATAVALID;
            }
        }
    } else if variant ecp3 {
        bel_class IO {
            input CLK, LSR, CE;
            input OPOSA, OPOSB, ONEGA, ONEGB;
            input TS;
            input DEL[4];
            input ECLK, ECLKDQSR;
            output IPA, IPB, INA, INB;
            output INDD, INCK;

            pad PAD: inout;
        }

        bel_class DQS {
            input SCLK, RST;
            input ECLK, ECLKW;
            input READ[1];
            input DCA[7];
            input DYNDELAY[8];
            output DQSW, ECLKDQSR, DDRCLKPOL;
            output PRMBDET;
            output DDRLAT;
            output DATAVALID;
            output TEST_DQSCNT[2];
            output TEST_ECLKCNT;
        }
    } else if variant machxo2 {
        bel_class IO {
            input CLK, LSR, CE;
            input OPOS[1], ONEG[1], TS;
            output IP, IN, DI;

            // south + north only
            input ECLK;

            // south only
            input DEL[5];
            input SLIP;
            output RXD[4];
            output RXDA[8];

            // north only
            input TXD[8];

            // special I3C only
            input RESEN, PULLUPEN;

            pad PAD: inout;
        }

        bel_class DQS {
            input SCLK, RST;
            input READ[1];
            input READCLKSEL[2];
            output BURSTDET, DATAVALID, DDRCLKPOL;
            output DQSR90, DQSW90;
        }
    } else if variant ecp4 {
        bel_class IO {
            input CLK, LSR, CE;
            input ECLK;
            input TXDATA[10];
            input TSDATA[4];
            output RXDATA[10];
            output DI;
            output INCK;
            input DIRECTION;
            input LOADN;
            input MOVE;
            input SLIP;
            input ACK;
            input WINDOWSIZE[2];
            output CFLAG;
            output PLUS, MINUS;

            pad PAD: inout;
        }

        bel_class DQS {
            input SCLK, RST;
            input ECLK, ECLK90;
            input DDRDEL;
            input READ[4], READCLKSEL[3];
            input ALIGNWD, PAUSE, SWITCH;
            input SELA[11], SELB[11];
            input RDDIRECTION, RDLOADN, RDMOVE;
            output RDCFLAG;
            input WRDIRECTION, WRLOADN, WRMOVE;
            output WRCFLAG;
            output BURSTDET, DATAVALID;
            output CRUCLK, NEXTCLK, DIVCLK;
            output DQSR90, DQSW, DQSW270;
        }
    } else if variant ecp5 {
        bel_class IO {
            input CLK, LSR, CE;
            input ECLK;
            input TXDATA[7];
            input TSDATA[2];
            output RXDATA[7];
            output INFF, DI;
            output INCK;
            input DIRECTION;
            input LOADN;
            input MOVE;
            input SLIP;
            output CFLAG;

            pad PAD: inout;
        }

        bel_class DQS {
            input SCLK, RST;
            input ECLK;
            input DDRDEL;
            input READ[2], READCLKSEL[3];
            input PAUSE;
            input DYNDELAY[8];
            input RDDIRECTION, RDLOADN, RDMOVE;
            output RDCFLAG;
            input WRDIRECTION, WRLOADN, WRMOVE;
            output WRCFLAG;
            output BURSTDET, DATAVALID;
            output DQSR90, DQSW, DQSW270;
        }
    } else if variant crosslink {
        bel_class IO {
            input CLK, LSR, CE;
            input ECLK;
            input TSDATA[1];
            input TXDATA[16];
            output RXDATA[16];
            output INFF;
            output DI;
            output INCK;
            input DIRECTION;
            input HSSEL;
            input LOADN;
            input MOVE;
            input SLIP;
            output CFLAG;

            pad PAD: inout;
        }
    } else if variant nexus {
        bel_class IO {
            output INCK;

            input SCLK, ECLK, LSR, CE;
            // non-HPIO have only 2 TXDATA/RXDATA and 1 TSDATA
            input TXDATA[10];
            input TSDATA[4];
            output RXDATA[10];
            output INFF, INDD;

            // HPIO-only; HS[RT]XEN only present in IO[0]
            input TDOLP, HSTXEN, HSRXEN;
            input WORDALIGN, MOVE, LOAD_N, DIR;
            input RANKENABLE, RANKSELECT, RANK0UPDATE, RANK1UPDATE;
            input CIBTSTSG, CIBCRS[2];
            output COUT, EDETERR;

            // LPDDR4 version only
            input MOVE_LPDDR4_WT, LOAD_N_LPDDR4_WT, DIRECTION_LPDDR4_WT;
            output COUT_LPDDR4_WT;

            // non-HPIO only
            input I3CRESEN, I3CWKPU;

            pad PAD: inout;
        }

        bel_class DQS {
            input SCLK, ECLKIN, RST, RSTSMCNT, SELCLK;
            input PAUSE;
            input READ[4], RDCLKSEL[4];
            input READMOVE, RDLOADN, RDDIR;
            input WRMOVE, WRLOAD_N, WRDIR;
            input WRLVMOVE, WRLVLOAD_N, WRLVDIR;
            output BURSTDETECT, BTDETECT, DATAVALID;
            output DQSWCIB, SW270CIB, SR90CIB, DQSWRD;
            output READCOUT, WRCOUT, WRLVCOUT;
        }
    }

    if variant [ecp, xp, ecp2, ecp2m, xp2, ecp3, machxo2, ecp4, ecp5, crosslink] {
        // named DQSDLL on ecp thru ecp3 and machxo2; named DDRDLL on ecp4, ecp5, crosslink
        bel_class DDRDLL {
            input CLK, RST;
            if variant [ecp, xp, ecp2, ecp2m, xp2] {
                input UDDCNTL;
            } else {
                input UDDCNTLN;
            }
            output LOCK;
            if variant [machxo2, ecp4, ecp5, crosslink] {
                input FREEZE;
            }
            if variant machxo2 {
                output TEST_DIVOSC;
            }
            if variant [machxo2, ecp3] {
                output TEST_SDOUT[7];
            }
            if variant [ecp4, ecp5, crosslink] {
                output DCNTL[8];
                output DIVOSC;
                output DDRDEL;
            }
        }
    } else if variant nexus {
        bel_class DDRDLL {
            input CLKIN, RST;
            input UDDCNTL_N, FREEZE;
            output LOCK, DIVOSCIB, DCNTL[9];
        }
    }
    if variant machxo2 {
        bel_class DLLDEL {
            input CLKI;
            output CLKO;
        }
    } else if variant [ecp4, ecp5, crosslink] {
        bel_class DLLDEL {
            input A, DDRDEL;
            output Z;
            input DIRECTION, LOADN, MOVE;
            output CFLAG;
        }
    } else if variant nexus {
        bel_class DLLDEL {
            input CLKIN;
            output CLKOUT;
            input BYPASS;
            input DIR, LOAD_N, MOVE;
            output COUT;
        }
    }


    if variant [machxo2, ecp4, ecp5, crosslink] {
        bel_class BCINRD {
            input INRDENI;
        }
        bel_class BCLVDSO {
            input LVDSENI;
        }
    }
    if variant [machxo2, ecp4] {
        bel_class BCPG {
            input PGENI;
        }
    }
    if variant machxo2 {
        bel_class BCSLEWRATE {
            input SLEWRATEENI;
        }
    }
    if variant ecp4 {
        bel_class BCPUSL {
            input PUSLENI;
        }
        bel_class BREFTEST {
            input TESTIN[6];
            output TESTOUT[6];
        }
        bel_class PVTTEST {
            input TESTIN[7];
            output TESTOUT[7];
        }
        bel_class PVTCAL {
            input STARTCAL;
        }
    }
    if variant ecp5 {
        bel_class BREFTEST {
            input PVT_SRC_IN[6], PVT_SNK_IN[6];
            output PVT_SRC_OUT[6], PVT_SNK_OUT[6];
        }
    }

    if variant [ecp3, machxo2, ecp4, ecp5, crosslink, nexus] {
        bel_class ECLKSYNC {
            input ECLKI;
            input STOP;
            output ECLKO;
        }
    }

    if variant [machxo2, ecp4, ecp5] {
        bel_class ECLKBRIDGECS {
            input CLK[2];
            input SEL;
            output ECSOUT;
        }
    }

    if variant machxo2 {
        bel_class CLKFBBUF {
            input A;
            output Z;
        }
    }

    if variant nexus {
        bel_class ECLKDIV {
            input ECLKIN;
            input DIVRST, SLIP;
            output DIVOUT;
            input TESTINP[4];
        }

        bel_class ECLKBANK_TEST {
            input TESTINP[4];
        }

        bel_class ECLKCASMUX_TEST {
            input TESTINP[2];
        }

        bel_class ECLKPLLFB_TEST {
            input TESTINP[4];
        }

        bel_class ECLK_TEST;
    }

    bel_class DCC {
        input CLKI;
        input CE;
        output CLKO;
    }

    if variant [scm, ecp, xp, ecp2, ecp2m, xp2, ecp3] {
        bel_class DCS {
            input CLK[2];
            input SEL[1];
            output DCSOUT;
        }
    } else if variant ecp4 {
        bel_class DCS {
            input CLK[4];
            input SEL[4];
            input MODESEL;
            output DCSOUT;
        }
    } else if variant [ecp5, crosslink] {
        bel_class DCS {
            input CLK[2];
            input SEL[2];
            input MODESEL;
            output DCSOUT;
        }
    }

    if variant machxo2 {
        bel_class DCM {
            input CLK[2];
            input SEL;
            output DCMOUT;
        }
    }

    if variant scm {
        bel_class CLKTEST {
            input CIBTESTB[6];
            input CIBTESTT[6];
        }
    } else if variant [machxo2, ecp4, ecp5, crosslink] {
        bel_class CLKTEST {
            input TESTIN[24];
        }
    } else if variant nexus {
        bel_class EMIDMUX_TEST {
            input TESTINP[4];
        }
    }

    if variant scm {
        bel_class PLL {
            input CLKI, CLKFB;
            output CLKOP, CLKOS;
            output CLKINTFB;
            input RST;
            output LOCK;
            output TPFB;
            output TPREF;
            output SMIRDATA;
        }

        bel_class PLL_SMI {
            input SMICLK;
            input SMIRSTN;
            input SMIRD;
            input SMIWR;
            input SMIADDR[10];
            input SMIWDATA;
        }
    } else if variant [machxo, ecp, xp, ecp2, ecp2m, xp2, ecp3] {
        bel_class PLL {
            input CLKI, CLKFB;
            output CLKOP, CLKOS, CLKOK;
            if variant [xp2, ecp3] {
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
            if variant [xp2, ecp3] {
            } else {
                // on ecp2, only present on plain PLL, not SPLL
                input DDAMODE, DDAIZR, DDAILAG, DDAIDEL[3];
                output DDAOZR, DDAOLAG, DDAODEL[3];
            }
            if variant [ecp2, ecp2m, xp2] {
                input DPAMODE;
            }
            if variant [ecp2, ecp2m, xp2, ecp3] {
                input DRPAI[4], DFPAI[4];
                output DRPAO[4], DFPAO[4];

            }
            if variant ecp3 {
                input FDA[4];
                input WRDEL;
            }
            input TCLKI;
            output TESTOUT;
        }
    } else if variant [machxo2, ecp4, ecp5, crosslink] {
        bel_class PLL {
            input CLK[2];
            input SEL;
            input CLKFB;
            input RST;
            input PHASEDIR;
            input PHASESEL[2];
            input PHASESTEP;
            if variant machxo2 {
                input LOADREG;
            } else {
                input PHASELOADREG;
            }
            if variant ecp4 {
                output PHASESRCSTAT;
            }
            input PLLWAKESYNC;
            if variant machxo2 {
                input RESETC;
                input RESETD;
                input RESETM;
                output DPHSRC;
            }
            if variant crosslink {
                input USRSTDBY;
            } else {
                input STDBY;
            }
            input ENCLKOP, ENCLKOS, ENCLKOS2, ENCLKOS3;
            output CLKOP, CLKOS, CLKOS2, CLKOS3;
            output CLKINTFB, REFCLK;
            output LOCK;
            output INTLOCK;
            if variant crosslink {
                output PFDDN, PFDUP;
            }
            if variant ecp4 {
                input SMICLK, SMIRSTN;
                input SMIRD, SMIWR, SMIAD, SMIWDATA;
                output SMIRDATA;
            }
        }
    } else if variant nexus {
        bel_class PLL {
            //
        }
    }

    if variant scm {
        bel_class CLKDIV {
            input CLKI;
            input LSR;
            output CLKO;
            output ELSR;
        }
    } else if variant [ecp2, ecp2m, xp2, ecp3] {
        bel_class CLKDIV {
            input CLKI;
            output CDIV1, CDIV2, CDIV4, CDIV8;
            input RST, RELEASE;
            if variant ecp3 {
                // hrm. is this just passthru?
                input TEST_CLKI1, TEST_CLKI2;
                output TEST_CLKO1, TEST_CLKO2;
            }
        }
    } else if variant [machxo2, ecp4, ecp5, crosslink] {
        bel_class CLKDIV {
            input CLKI;
            input RST, ALIGNWD;
            output CDIVX;
            if variant machxo2 {
                output CDIV1;
            }
        }
    }

    if variant [ecp4, ecp5] {
        bel_class PCSCLKDIV {
            input CLKI;
            input RST, SEL[3];
            output CDIV1, CDIVX;
        }
    }

    if variant scm {
        bel_class DLL {
            input CLKI, CLKFB;
            input CLKIDUTY;
            output CLKOP, CLKOS;
            input RST;
            output LOCK;
            input UDDCNTL;
            input DTCCST[2];
            output SMIRDATA;
        }

        bel_class DLL_DCNTL {
            input DCNTL_IN[9];
            output DCNTL_OUT[9];
        }
    } else if variant [ecp2, ecp2m] {
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

    }

    if variant ecp3 {
        bel_class DLL {
            input CLKI, CLKFB;
            output CLKOP, CLKOS;
            output DCNTL[6];
            input RSTN;
            input UDDCNTL;
            output LOCK;
            input GRAYI[6];
            output GRAYO[6];
            input INCI;
            output INCO;
            input ALUHOLD;
            input CIB_DCPS[6];
            output DIFF;
        }
    }

    if variant [ecp2, ecp2m, ecp3] {
        bel_class DLLDEL {
            input CLKI;
            output BYPASS_OUT;
            output CLKO;
        }
    }

    if variant nexus {
        bel_class CDR {
            // direct connection to PLL
            input SREFCLK;
            // direct connection to clock tree
            output SRCLK;

            // normal routable stuff
            input SPCLK;
            input RRST;
            input RSTBRXF, RSTBFBW;
            input DFACQRST, DCALIRST;
            input OPCGLOADCLK;
            output SRXD[10];
            output CDRLOL;
            input LMMICLK, LMMIRESETN;
            input LMMIREQUEST, LMMIWRRDN;
            input LMMIOFFSET[4];
            input LMMIWDATA[8];
            output LMMIRDATA[8];
            output LMMIREADY, LMMIRDATAVALID;
            input SCANCLK, SCANRST;
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

    if variant scm {
        bel_class IPAD {
            output PADDI;
            pad PAD: input;
        }

        bel_class RDBK {
            input FFRDCFGCLK, FFRDCFG;
            output RDDATA;
        }

        bel_class RDCFGN {
            input TSALLN;
            output RDCFGN;
        }

        bel_class RNET {
            input RNETUPD;
            input RNETI[6];
            output RNETO[6];
        }

        bel_class PROMON {
            input PROCHK;
            output PROMON;
        }

        bel_class SYSBUS {
            output HCLKCIB;
            input SYSRSTN;
            input MPIRSTN;
            input USRCLK;
            input USERIRQI;
            output USERIRQO;
            output SYSBDEBUG[24];

            input DMARDDATA[32];
            input DMARDPARITY[4];
            input DMARETRY;
            input DMATA;
            input DMATEA;
            input DMATRICTL;
            input DMATRIDATA;

            input FMIRQ;
            input FMCLK;
            input FMRSTN;
            input FMLOCK;
            input FMRDY;
            input FMWRN;
            input FMBURST;
            input FMSIZE[2];
            input FMADDR[18];
            input FMWDATA[36];
            output FMRDATA[36];
            output FMACK;
            output FMERR;
            output FMRETRY;

            input FSIRQ;
            input FSCLK;
            input FSRSTN;
            output FSRDY;
            output FSWRN;
            output FSSIZE[2];
            output FSADDR[18];
            output FSWDATA[36];
            input FSRDATA[36];
            input FSACK;
            input FSERR;
            input FSRETRY;

            output SMICLK;
            output SMIRSTN;
            output SMIRD;
            output SMIWR;
            output SMIADDR[10];
            output SMIWDATA;
            input SMIRDATA[64];
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

    if variant [machxo2, ecp4] {
        bel_class PCNTR {
            input CLK;
            input CLRFLAG;
            input USERSTDBY;
            input USERTIMEOUT;
            output SFLAG;
            output STDBY;
            output STOP;
        }

        bel_class EFB {
            input WBCLKI;
            input WBRSTI;
            input WBCYCI;
            input WBSTBI;
            input WBWEI;
            if variant ecp4 {
                input WBADRI[16];
            } else {
                input WBADRI[8];
            }
            input WBDATI[8];
            output WBDATO[8];
            output WBACKO;
            if variant ecp4 {
                output WBERRO;
                output WBRTYO;
                output WBCFGINT;
            } else {
                output WBCUFMIRQ;
            }

            output I2C1IRQO;
            output I2C2IRQO;
            input I2C2SCLI;
            output I2C2SCLO;
            output I2C2SCLOEN;
            input I2C2SDAI;
            output I2C2SDAO;
            output I2C2SDAOEN;

            if variant ecp4 {
                output PCS0INT;
                output PCS1INT;
                output PCS2INT;
                output PCS3INT;
                output PCS4INT;

                output SMICLKO;
                output SMIADO;
                output SMIRDO;
                output SMIWRO;
                output SMIRSTNO;
                output SMIWDATAO;
                input SMIRDATAI;
            }

            output SPIIRQO;
            output SPIMCSN1;
            output SPIMCSN2;
            output SPIMCSN3;
            output SPIMCSN4;
            output SPIMCSN5;
            output SPIMCSN6;
            output SPIMCSN7;
            input SPISCSN;

            input TCCLKI;
            input TCRSTN;
            input TCIC;
            output TCOC;
            output TCINT;

            if variant machxo2 {
                // machxo3d only
                input TAMPERDETCLK;
                input TAMPERDETEN;
                input TAMPERLOCKSRC;
                output TAMPERDET;
                output TAMPERSRC[2];
                output TAMPERTYPE[2];
            }
        }
    }

    if variant machxo2 {
        bel_class ESB {
            input WBCLKI;
            input WBRSTI;
            input WBCYCI;
            input WBSTBI;
            input WBWEI;
            input WBADRI[18];
            input WBDATI[32];
            output WBDATO[32];
            output WBACKO;
            input ASFCLKI;
            input ASFRESETI;
            input ASFRDI;
            input ASFWRI;
            output ASFEMPTYO;
            output ASFFULLO;
        }
    }

    if variant [ecp4, ecp5] {
        bel_class DTR {
            input START_PULSE;
            output DTR_OUT[8];
        }
    }

    if variant ecp5 {
        bel_class CCLK {
            input PADDO, PADDT;
            output PADDI;
        }
    }

    if variant scm {
        bel_class SERDES {
            // special clock input shared across all SERDES
            input FFC_CK_CORE_TX;
            // dedicated clock outs
            output FF_RXCLK_P1;
            output FF_RXCLK_P2;
            output FF_SYSCLK_P1;
            // output to fabric, or to IO interconnect ring
            output BS4PAD[4];

            output FB_RXD_0[24];
            output FB_RXD_1[24];
            output FB_RXD_2[24];
            output FB_RXD_3[24];
            input FFC_AB_RESET;
            input FFC_ALIGN_EN[4];
            input FFC_CD_RESET;
            input FFC_CK_CORE_RX;
            input FFC_EN_CGA[4];
            input FFC_FB_LB[4];
            input FFC_LANE_RX_RST[4];
            input FFC_LANE_TX_RST[4];
            input FFC_MACRO_RST;
            input FFC_PCIE_CT[4];
            input FFC_PCIE_EI_EN[4];
            input FFC_PCIE_RX[4];
            input FFC_PCIE_TX[4];
            input FFC_QUAD_RST;
            input FFC_SB_INV_RX[4];
            input FFC_SD[4];
            output FFS_AB_ALIGNED;
            output FFS_AB_FAILED;
            output FFS_AB_STATUS;
            output FFS_CC_ORUN[4];
            output FFS_CC_URUN[4];
            output FFS_CD_ALIGNED;
            output FFS_CD_FAILED;
            output FFS_CD_STATUS;
            output FFS_LS_STATUS[4];
            output FFS_PCIE_CON[4];
            output FFS_PCIE_DONE[4];
            output FFS_RLOS_LO[4];
            input FF_RCLK[4];
            output FF_RXCLK[4];
            output FF_SYSCLK[4];
            input FF_TCLK[4];
            input FF_TXD_0[24];
            input FF_TXD_1[24];
            input FF_TXD_2[24];
            input FF_TXD_3[24];
        }

        bel_class SERDES_CORNER {
            input CIN[13];
            output COUT[22];
            output TCK_FMAC_PCS;
            input TESTCLK_MACO;
        }
    } else if variant [ecp2m, ecp3] {
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
    } else if variant ecp4 {
        bel_class SERDES {
            input CORNER_FCSCANMODE;
            input CORNER_FCSYNCTOGGLE;
            output CORNER_FOREFCK2CORE;
            output CORNER_LSPLLLOL;
            input CORNER_LSPLLPWRUP;
            input CORNER_LSPLLREFCLKI;
            input CORNER_LSPLLRST;

            for q in 0..4 {
                for ch in 0..4 {
                    // dedicated clock out
                    output "Q{q}CH{ch}_FOPCLKA";
                    output "Q{q}CH{ch}_FOPCLKB";

                    input "Q{q}CH{ch}_FCALIGNEN";
                    input "Q{q}CH{ch}_FCCDRFORCEDLOCK";
                    input "Q{q}CH{ch}_FCDFERDEN";
                    input "Q{q}CH{ch}_FCDFEUPD";
                    input "Q{q}CH{ch}_FCLDRTXEN";
                    input "Q{q}CH{ch}_FCLSMEN";
                    input "Q{q}CH{ch}_FCPCIEDETEN";
                    input "Q{q}CH{ch}_FCPCSRXRST";
                    input "Q{q}CH{ch}_FCPCSTXRST";
                    input "Q{q}CH{ch}_FCPIPEPHYRESETN";
                    input "Q{q}CH{ch}_FCPLLLOL";
                    input "Q{q}CH{ch}_FCRATE"[3];
                    input "Q{q}CH{ch}_FCRRST";
                    input "Q{q}CH{ch}_FCRXPOLARITY";
                    input "Q{q}CH{ch}_FCRXPWRUP";
                    input "Q{q}CH{ch}_FCTMRSTART";
                    input "Q{q}CH{ch}_FCTMRSTOP";
                    input "Q{q}CH{ch}_FCTRST";
                    input "Q{q}CH{ch}_FCTXMARGIN"[3];
                    input "Q{q}CH{ch}_FCTXPWRUP";
                    input "Q{q}CH{ch}_FCWORDALGNEN";
                    output "Q{q}CH{ch}_FDLDRRX";
                    input "Q{q}CH{ch}_FDLDRTX";
                    output "Q{q}CH{ch}_FDRX"[48];
                    input "Q{q}CH{ch}_FDTX"[50];
                    input "Q{q}CH{ch}_FIRCLK";
                    input "Q{q}CH{ch}_FIREFRXCLK";
                    input "Q{q}CH{ch}_FITCLK";
                    input "Q{q}CH{ch}_FITMRSTARTCLK";
                    input "Q{q}CH{ch}_FITMRSTOPCLK";
                    output "Q{q}CH{ch}_FSCCOVERRUN";
                    output "Q{q}CH{ch}_FSCCUNDERRUN";
                    output "Q{q}CH{ch}_FSDFEVLD";
                    output "Q{q}CH{ch}_FSLSM";
                    output "Q{q}CH{ch}_FSPCIECON";
                    output "Q{q}CH{ch}_FSPCIEDONE";
                    output "Q{q}CH{ch}_FSRCDONE";
                    output "Q{q}CH{ch}_FSRLOL";
                    output "Q{q}CH{ch}_FSRLOS";
                    output "Q{q}CH{ch}_FSSKPADDED";
                    output "Q{q}CH{ch}_FSSKPDELETED";
                }
                for d in 0..2 {
                    input "Q{q}D{d}_FCDERST";
                    output "Q{q}D{d}_FSDE";
                    output "Q{q}D{d}_FSDM";
                }
                input "Q{q}_FCDFECOEFF0"[8];
                input "Q{q}_FCDFECOEFF1"[8];
                input "Q{q}_FCDFECOEFF2"[8];
                input "Q{q}_FCDFECOEFF3"[8];
                input "Q{q}_FCDFECOEFF4"[8];
                input "Q{q}_FCDFECOEFF5"[8];
                input "Q{q}_FCDFESIGN1";
                input "Q{q}_FCDFESIGN2";
                input "Q{q}_FCDFESIGN3";
                input "Q{q}_FCDFESIGN4";
                input "Q{q}_FCDFESIGN5";
                input "Q{q}_FCMPWRUP";
                input "Q{q}_FCMRST";
                input "Q{q}_FCSCANMODE";
                input "Q{q}_FDDFECHSEL"[2];
                output "Q{q}_FDDFEDATA"[10];
                output "Q{q}_FDDFEERR"[10];
                input "Q{q}_FIGRPFBRRCLK"[2];
                input "Q{q}_FIGRPFBTWCLK"[2];
                input "Q{q}_FIRXTESTCLK";
                input "Q{q}_FISYNCCLK";
                input "Q{q}_FITMRCLK";
                input "Q{q}_FITXTESTCLK";
                output "Q{q}_FOREFCLK2FPGA";
                output "Q{q}_HSPLLLOL";
                input "Q{q}_HSPLLPWRUP";
                input "Q{q}_HSPLLREFCLKI";
                input "Q{q}_HSPLLRST";
                output "Q{q}_LSPLLLOL";
                input "Q{q}_LSPLLPWRUP";
                input "Q{q}_LSPLLREFCLKI";
                input "Q{q}_LSPLLRST";
            }

            // direct to PLL
            output Q0P_CORECLK;

            input Q0P_CORERRIN;
            output Q0P_CORERROUT;
            output Q0P_DBGDATOUT[16];
            output Q0P_FTLERROUT;
            output Q0P_FUNCPWRSTATE[3];
            output Q0P_FUNCSTS[4];
            output Q0P_HALMSTACK;
            output Q0P_HALMSTCOMPDES[128];
            output Q0P_HALMSTCOMPEOP;
            input Q0P_HALMSTCOMPRDY;
            output Q0P_HALMSTCOMPSOP;
            output Q0P_HALMSTCOMPVLD;
            output Q0P_HALMSTRBVLD[8];
            output Q0P_HALMSTREJ;
            input Q0P_HALMSTREQ;
            input Q0P_HALMSTREQATTR[3];
            input Q0P_HALMSTREQDES[128];
            input Q0P_HALMSTREQTPHPRESENT;
            input Q0P_HALMSTREQTPHSTTAG[8];
            input Q0P_HALMSTREQTPHTYPE[2];
            output Q0P_HALMSTRERR;
            output Q0P_HALMSTTAG[5];
            input Q0P_HALMSTWBVLD[8];
            input Q0P_HALMSTWDATVLD;
            input Q0P_HALMSTWEOP;
            input Q0P_HALMSTWERR;
            output Q0P_HALMSTWRDY;
            input Q0P_HALTGTACK;
            input Q0P_HALTGTCAREQDES[128];
            input Q0P_HALTGTCLNTCOMPID[16];
            input Q0P_HALTGTCLNTCOMPIDEN;
            input Q0P_HALTGTCOMPBVLD[8];
            input Q0P_HALTGTCOMPDES[128];
            input Q0P_HALTGTCOMPEOP;
            input Q0P_HALTGTCOMPERR;
            output Q0P_HALTGTCOMPRDY;
            input Q0P_HALTGTCOMPSOP;
            input Q0P_HALTGTCOMPVLD;
            input Q0P_HALTGTNPREJ;
            output Q0P_HALTGTREQ;
            output Q0P_HALTGTREQDES[128];
            output Q0P_HALTGTREQMBA[6];
            output Q0P_HALTGTREQTPHPRESENT;
            output Q0P_HALTGTREQTPHSTTAG[8];
            output Q0P_HALTGTREQTPHTYPE[2];
            output Q0P_HALTGTWBVLD[8];
            output Q0P_HALTGTWDATVLD;
            output Q0P_HALTGTWEOP;
            input Q0P_HALTGTWRDY;
            output Q0P_HALTSTREQATTR[3];
            input Q0P_HOTRSTIN;
            output Q0P_HOTRSTOUT;
            output Q0P_INTACK;
            input Q0P_INTAI;
            output Q0P_INTAO;
            input Q0P_INTBI;
            output Q0P_INTBO;
            input Q0P_INTCI;
            output Q0P_INTCO;
            input Q0P_INTDI;
            output Q0P_INTDO;
            input Q0P_INTPENDSTS;
            output Q0P_LCLINT;
            output Q0P_LNKDWNRSTOUT;
            output Q0P_LNKPWRSTATE[4];
            output Q0P_LNKSTS[2];
            output Q0P_LTSSMSTATE[6];
            output Q0P_MAXPYLDSIZE[3];
            output Q0P_MAXREADREQSIZE[3];
            input Q0P_MBISTCLK;
            input Q0P_MBISTMODE;
            input Q0P_MBISTRSTN;
            input Q0P_MSIASRTINT[32];
            input Q0P_MSIATTRIN[3];
            output Q0P_MSIEN;
            output Q0P_MSIMSGABRT;
            output Q0P_MSIMSGSENT;
            input Q0P_MSIREQTPHPRESENT;
            input Q0P_MSIREQTPHSTTAG[8];
            input Q0P_MSIREQTPHTYPE[2];
            output Q0P_MSIVECCNT[3];
            input Q0P_MSIXATTRIN[3];
            output Q0P_MSIXEN;
            output Q0P_MSIXMASK;
            input Q0P_MSIXMSGADDR[64];
            input Q0P_MSIXMSGDAT[32];
            output Q0P_MSIXMSGSENT;
            input Q0P_MSIXMSGVLD;
            input Q0P_MSIXREQTPHPRESENT;
            input Q0P_MSIXREQTPHSTTAG[8];
            input Q0P_MSIXREQTPHTYPE[2];
            output Q0P_NEGLNKWIDTH[2];
            output Q0P_NEGSPEED;
            output Q0P_NFTLERROUT;
            input Q0P_PWRSTATECHNGACK;
            output Q0P_PWRSTATECHNGINT;
            output Q0P_RCBSTS;
            input Q0P_RSTN;
            input Q0P_SCANCLK;
            input Q0P_SCANENA;
            input Q0P_SCANI[27];
            input Q0P_SCANMODE;
            output Q0P_SCANO[25];
            input Q0P_SCANRSTN;
            output Q0P_TPHREQENABLE;
            output Q0P_TPHSTMODE[3];
            input Q0P_UNCORRERRIN;

            for q in 1..4 {
                for ch in 0..4 {
                    input "Q{q}EA{ch}_CIRXFULL";
                    input "Q{q}EA{ch}_CIRXIGNOREPKT";
                    input "Q{q}EA{ch}_CITXDATA"[16];
                    input "Q{q}EA{ch}_CITXDATAAVAIL";
                    input "Q{q}EA{ch}_CITXEMPTY";
                    input "Q{q}EA{ch}_CITXEOF";
                    input "Q{q}EA{ch}_CITXFIFOCTRL";
                    input "Q{q}EA{ch}_CITXFORCEERR";
                    input "Q{q}EA{ch}_CITXLASTBYTEVLD";
                    input "Q{q}EA{ch}_CITXPAUSREQ";
                    input "Q{q}EA{ch}_CITXPAUSTIM"[16];
                    output "Q{q}EA{ch}_CORXDATA"[16];
                    output "Q{q}EA{ch}_CORXEOF";
                    output "Q{q}EA{ch}_CORXERROR";
                    output "Q{q}EA{ch}_CORXFIFOFULLERROR";
                    output "Q{q}EA{ch}_CORXLASTBYTEVLD";
                    output "Q{q}EA{ch}_CORXSTATEN";
                    output "Q{q}EA{ch}_CORXSTATVEC"[8];
                    output "Q{q}EA{ch}_CORXWRITE";
                    output "Q{q}EA{ch}_COTXDISCFRM";
                    output "Q{q}EA{ch}_COTXDONE";
                    output "Q{q}EA{ch}_COTXREAD";
                    output "Q{q}EA{ch}_COTXSTATEN";
                    output "Q{q}EA{ch}_COTXSTATVEC"[8];
                    input "Q{q}EA{ch}_GIIPGSHRINK";
                    input "Q{q}EA{ch}_GINONPADRXDV";
                    input "Q{q}EA{ch}_GISYNCCOL";
                    input "Q{q}EA{ch}_GISYNCCRS";
                    input "Q{q}EA{ch}_GISYNCNIBDRIB";
                    input "Q{q}EA{ch}_GISYNCRXD"[8];
                    input "Q{q}EA{ch}_GISYNCRXDV";
                    input "Q{q}EA{ch}_GISYNCRXER";
                    output "Q{q}EA{ch}_GODISCARDFCS";
                    output "Q{q}EA{ch}_GOTXMACDATA"[8];
                    output "Q{q}EA{ch}_GOTXMACERR";
                    output "Q{q}EA{ch}_GOTXMACWR";
                    input "Q{q}EA{ch}_KIRSTN";
                    input "Q{q}EA{ch}_KIRXMACCLK";
                    input "Q{q}EA{ch}_KIRXMACCLKENEXT";
                    input "Q{q}EA{ch}_KIRXTXFECLK";
                    input "Q{q}EA{ch}_KITXGMIILPBK";
                    input "Q{q}EA{ch}_KITXMACCLK";
                    input "Q{q}EA{ch}_KITXMACCLKENEXT";
                    output "Q{q}EA{ch}_KOGBITEN";
                    output "Q{q}EA{ch}_KORXMACCLKEN";
                }
            }
            for q in 2..4 {
                for ch in 0..4 {
                    input "Q{q}EB{ch}_CIRXFULL";
                    input "Q{q}EB{ch}_CIRXIGNOREPKT";
                    input "Q{q}EB{ch}_CITXDATA"[8];
                    input "Q{q}EB{ch}_CITXDATAAVAIL";
                    input "Q{q}EB{ch}_CITXEMPTY";
                    input "Q{q}EB{ch}_CITXEOF";
                    input "Q{q}EB{ch}_CITXFIFOCTRL";
                    input "Q{q}EB{ch}_CITXFORCEERR";
                    input "Q{q}EB{ch}_CITXPAUSEREQ";
                    input "Q{q}EB{ch}_CITXPAUSTIM"[16];
                    input "Q{q}EB{ch}_GIIPGSHRINK";
                    input "Q{q}EB{ch}_GINONPADRXDV";
                    input "Q{q}EB{ch}_GISYNCCOL";
                    input "Q{q}EB{ch}_GISYNCCRS";
                    input "Q{q}EB{ch}_GISYNCNIBDRIB";
                    input "Q{q}EB{ch}_GISYNCRXD"[8];
                    input "Q{q}EB{ch}_GISYNCRXDV";
                    input "Q{q}EB{ch}_GISYNCRXER";
                    input "Q{q}EB{ch}_KIRSTN";
                    input "Q{q}EB{ch}_KIRXMACCLK";
                    input "Q{q}EB{ch}_KIRXMACCLKENEXT";
                    input "Q{q}EB{ch}_KIRXTXFECLK";
                    input "Q{q}EB{ch}_KITXGMIILPBK";
                    input "Q{q}EB{ch}_KITXMACCLK";
                    input "Q{q}EB{ch}_KITXMACCLKENEXT";
                }
                output "Q{q}X_CSO"[141];
                output "Q{q}X_GSO"[44];
                input "Q{q}X_IIGNOREPKT";
                input "Q{q}X_IRESETN";
                input "Q{q}X_IRXMACCLK";
                input "Q{q}X_ITSMSGAVAIL";
                input "Q{q}X_ITXBYTEN"[3];
                input "Q{q}X_ITXDATA"[64];
                input "Q{q}X_ITXDATAAVAIL";
                input "Q{q}X_ITXEMPTY";
                input "Q{q}X_ITXEOF";
                input "Q{q}X_ITXFORCEERR";
                input "Q{q}X_ITXMACCLK";
                input "Q{q}X_ITXMSG"[64];
                input "Q{q}X_ITXPAUSREQ";
                input "Q{q}X_ITXPAUSTIM"[16];
                output "Q{q}X_KSO"[20];
                input "Q{q}X_TIBISTBANKSEL"[4];
                input "Q{q}X_TIBISTRA"[8];
                input "Q{q}X_TIBISTREN";
                input "Q{q}X_TIBISTTDI"[36];
                input "Q{q}X_TIBISTTESTMODE";
                input "Q{q}X_TIBISTWA"[8];
                input "Q{q}X_TIBISTWEN";
                input "Q{q}X_TISCANEN";
                input "Q{q}X_TISCANIN"[30];
                input "Q{q}X_TISCANMODE";
                input "Q{q}X_TISCANRSTN";
                output "Q{q}X_TOSCANOUT"[29];
            }

            input Q1S_BAUDSUPPORT[5];
            output Q1S_BUFFDEQACKADVPTRN;
            output Q1S_DECRBUFCNTVECTOR[10];
            input Q1S_ENABLETXFLOWCONTROLN;
            output Q1S_FLOWFIFOACKON[4];
            input Q1S_GEAR;
            input Q1S_LNKCLK;
            input Q1S_LNKCLKDIV2;
            output Q1S_LNKCLKDIV2RSTN;
            output Q1S_LNKCLKRSTN;
            input Q1S_LNKDIV2RSTN;
            input Q1S_LNKMCERXACKN;
            output Q1S_LNKMCERXREQN;
            output Q1S_LNKMCETXACKN;
            input Q1S_LNKMCETXREQN;
            input Q1S_LNKRSTN;
            output Q1S_LNKTOUTPUTPORTENABLE;
            input Q1S_LOOPBACK[3];
            output Q1S_MASTERENABLE;
            input Q1S_MGTA[22];
            input Q1S_MGTCLK;
            output Q1S_MGTCLKRSTN;
            input Q1S_MGTDI[32];
            input Q1S_MGTRDN;
            input Q1S_MGTRSTN;
            input Q1S_MGTWRN[4];
            input Q1S_OLLMEFPTR[16];
            output Q1S_OLLMMGTDI[32];
            output Q1S_OLLMMGTINTN;
            output Q1S_OLLMMGTRDYN;
            input Q1S_OUTPUTUNRECOVERREVENTACKN;
            output Q1S_OUTPUTUNRECOVERREVENTREQN;
            input Q1S_PHYCLK;
            input Q1S_PHYEMEVENTACKN;
            output Q1S_PHYEMEVENTREQN;
            input Q1S_PHYMSTB;
            input Q1S_PHYRINITN;
            input Q1S_PHYRSTN;
            input Q1S_PHYTINITN;
            input Q1S_PHYUSTB;
            output Q1S_PORTDISABLE;
            output Q1S_PORTNERRORDETECT[32];
            input Q1S_RCVCLKSEL[4];
            input Q1S_RECOVERRSTN;
            output Q1S_RESPTIMEOUT[24];
            input Q1S_RIOCLK;
            input Q1S_RIORSTN;
            output Q1S_RLNKBEATS[8];
            output Q1S_RLNKD[64];
            input Q1S_RLNKDSTDSCN;
            input Q1S_RLNKDSTRDYN;
            output Q1S_RLNKEOFN;
            output Q1S_RLNKREM[3];
            output Q1S_RLNKSOFN;
            output Q1S_RLNKSRCRDYN;
            output Q1S_RXINITN;
            input Q1S_RXPFORCERETRYN;
            input Q1S_RXSYNCCTRL[2];
            input Q1S_SOFTRSTN;
            output Q1S_STATUS[22];
            input Q1S_SYSRSTIN;
            output Q1S_SYSRSTON;
            input Q1S_TIMBISTBANKSEL[4];
            input Q1S_TIMBISTMODE;
            input Q1S_TIMBISTRA[8];
            input Q1S_TIMBISTRE;
            input Q1S_TIMBISTSDI[36];
            input Q1S_TIMBISTWA[8];
            input Q1S_TIMBISTWE;
            input Q1S_TISCANCLK;
            input Q1S_TISCANENA;
            input Q1S_TISCANI[30];
            input Q1S_TISCANMODE;
            output Q1S_TISCANO[30];
            input Q1S_TISCANRSTN;
            input Q1S_TLNKD[64];
            output Q1S_TLNKDSTRDYN;
            input Q1S_TLNKEOFN;
            input Q1S_TLNKREM0;
            input Q1S_TLNKREM1;
            input Q1S_TLNKREM2;
            input Q1S_TLNKSOFN;
            input Q1S_TLNKSRCDSCN;
            input Q1S_TLNKSRCRDYN;
            output Q1S_TOMBISTDO[36];
            input Q1S_TPORTCHARISK[8];
            input Q1S_TPORTENABLEN;
            input Q1S_TPORTTDI[64];
            output Q1S_TXENQUEUEFLOWN[4];
            output Q1S_TXFLOWCTRLSTATE[5];
            output Q1S_TXINITN;
            output Q1S_TXLANESILENCECH[4];
            input Q1S_TXSYNCCTRL[2];
            input Q1S_WM0[4];
            input Q1S_WM1[4];
            input Q1S_WM2[4];
        }
    } else if variant ecp5 {
        bel_class SERDES {
            for ch in 0..2 {
                // dedicated clock outs
                output "CH{ch}_FF_TX_PCLK";
                output "CH{ch}_FF_RX_PCLK";

                input "CH{ch}_FFC_CDR_EN_BITSLIP";
                input "CH{ch}_FFC_DIV11_MODE_RX";
                input "CH{ch}_FFC_DIV11_MODE_TX";
                input "CH{ch}_FFC_EI_EN";
                input "CH{ch}_FFC_ENABLE_CGALIGN";
                input "CH{ch}_FFC_FB_LOOPBACK";
                input "CH{ch}_FFC_LANE_RX_RST";
                input "CH{ch}_FFC_LANE_TX_RST";
                input "CH{ch}_FFC_LDR_CORE2TX_EN";
                input "CH{ch}_FFC_PCIE_CT";
                input "CH{ch}_FFC_PCIE_DET_EN";
                input "CH{ch}_FFC_PFIFO_CLR";
                input "CH{ch}_FFC_RATE_MODE_RX";
                input "CH{ch}_FFC_RATE_MODE_TX";
                input "CH{ch}_FFC_RRST";
                input "CH{ch}_FFC_RXPWDNB";
                input "CH{ch}_FFC_RX_GEAR_MODE";
                input "CH{ch}_FFC_SB_INV_RX";
                input "CH{ch}_FFC_SB_PFIFO_LP";
                input "CH{ch}_FFC_SIGNAL_DETECT";
                input "CH{ch}_FFC_TXPWDNB";
                input "CH{ch}_FFC_TX_GEAR_MODE";
                output "CH{ch}_FFS_CC_OVERRUN";
                output "CH{ch}_FFS_CC_UNDERRUN";
                output "CH{ch}_FFS_LS_SYNC_STATUS";
                output "CH{ch}_FFS_PCIE_CON";
                output "CH{ch}_FFS_PCIE_DONE";
                output "CH{ch}_FFS_RLOL";
                output "CH{ch}_FFS_RLOS";
                output "CH{ch}_FFS_RXFBFIFO_ERROR";
                output "CH{ch}_FFS_SKP_ADDED";
                output "CH{ch}_FFS_SKP_DELETED";
                output "CH{ch}_FFS_TXFBFIFO_ERROR";
                input "CH{ch}_FF_EBRD_CLK";
                input "CH{ch}_FF_RXI_CLK";
                output "CH{ch}_FF_RX_D"[24];
                output "CH{ch}_FF_RX_F_CLK";
                output "CH{ch}_FF_RX_H_CLK";
                input "CH{ch}_FF_TXI_CLK";
                input "CH{ch}_FF_TX_D"[24];
                output "CH{ch}_FF_TX_F_CLK";
                output "CH{ch}_FF_TX_H_CLK";
                input "CH{ch}_LDR_CORE2TX";
                output "CH{ch}_LDR_RX2CORE";
                input "CH{ch}_RX_REFCLK";
                input "CH{ch}_SCIEN";
                input "CH{ch}_SCISEL";
            }

            input D_CIN[12];
            output D_COUT[20];
            input D_CYAWSTN;
            input D_FFC_DUAL_RST;
            input D_FFC_MACROPDB;
            input D_FFC_MACRO_RST;
            input D_FFC_SYNC_TOGGLE;
            input D_FFC_TRST;
            output D_FFS_PLOL;
            input D_REFCLKI;
            input D_SCAN_ENABLE;
            input D_SCAN_IN[8];
            input D_SCAN_MODE;
            output D_SCAN_OUT[8];
            input D_SCAN_RESET;
            input D_SCIADDR[6];
            input D_SCIENAUX;
            output D_SCIINT;
            input D_SCIRD;
            output D_SCIRDATA[8];
            input D_SCISELAUX;
            input D_SCIWDATA[8];
            input D_SCIWSTN;

            output REFCLKO;
        }
    }

    if variant crosslink {
        bel_class PMU {
            input SLEEP;
            input USRCLK, USRCS, USRSTB;
            input USRADR[4];
            input USRDATA[8];
            output USRWDTDONE, USRWDTINT;

            input PMUSCANCLK, PMUSCANRESET, PMUSCANSHIFTEN, PMUSCANSDI;
            output PMUSCANSDO;
        }

        bel_class I2C {
            input CLKI, CSI, STBI, WEI;
            input ADRI[4];
            input DATI[10];
            output DATO[10];
            output ACKO;

            input FIFORST;
            output RXFIFOAF;
            output RXFIFOE;
            output RXFIFOF;
            output TXFIFOAE;
            output TXFIFOE;
            output TXFIFOF;
            output MRDCMPL;
            output SRDWR;

            output I2CIRQ;
            output I2CWKUP;

            input SCLI;
            output SCLO;
            output SCLOEN;
            input SDAI;
            output SDAO;
            output SDAOEN;
        }

        bel_class NVCMTEST {
            input NVSCANCLK, NVSCANRESET, NVSCANSHIFTEN, NVSCANSDI;
            output NVSCANSDO;
            output SCANTESTMODE;
        }

        bel_class CFGTEST {
            input CFGSCANCLK, CFGSCANRESET, CFGSCANSDI, CFGSCANSHIFTEN;
            output CFGSCANSDO;
        }

        bel_class MIPI {
            input CLKREF;
            output CLKHSBYTE, HSBYTECLKS, HSBYTECLKD;

            input CLKCDEN;
            output CLKDCDN;
            output CLKDRXHS;
            output CLKDRXLPN;
            output CLKDRXLPP;
            input CLKDTXLPN;
            input CLKDTXLPP;
            input CLKRXHSEN;
            input CLKRXLPEN;
            input CLKTXHSEN;
            input CLKTXHSGATE;
            input CLKTXHSPD;
            input CLKTXLPEN;

            input CM[8];
            input CN[5];
            input CO[2];

            for ch in 0..4 {
                input "D{ch}CDEN";
                output "D{ch}DCDN";
                output "D{ch}DCDP";
                output "D{ch}DRXHS";
                output "D{ch}DRXLPN";
                output "D{ch}DRXLPP";
                input "D{ch}DTXLPN";
                input "D{ch}DTXLPP";
                output "D{ch}ERRSYNC";
                input "D{ch}HSDESEREN";
                output "D{ch}HSRXDATA"[16];
                input "D{ch}HSSEREN";
                input "D{ch}HSTXDATA"[16];
                output "D{ch}NOSYNC";
                input "D{ch}RXHSEN";
                input "D{ch}RXLPEN";
                output "D{ch}SYNC";
                input "D{ch}TXHSEN";
                input "D{ch}TXHSPD";
                input "D{ch}TXLPEN";
            }

            input LBEN;
            output LOCK;
            input PDBIAS;
            input PDCKG;
            input PDDPHY;
            input PDPLL;
            input TST[4];

            input CLKTEST_TESTIN[2];
        }
    }

    if variant nexus {
        bel_class MIPI {
            // TODO
        }

        bel_class ADC {
            // dedicated input from PLL CLKOS4
            input CLKDCLK;

            input CLKFAB, RSTN, RESETN;
            input ADCEN;
            input SOC, CONVSTOP;
            input CHAEN, CHBEN;
            input CHASEL[4], CHBSEL[4];
            input CAL;
            input RSVDL, RSVDH;
            output DA[12], DB[12];
            output EOC;
            output COG;
            output CALRDY;
            output COMP1O, COMP2O, COMP3O;
            output COMP1OL, COMP2OL, COMP3OL;

            pad DP[2]: analog;
            pad DN[2]: analog;
            pad REFP[2]: analog;
            pad VSS: power;
            pad VCC18: power;
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
    } else if variant nexus {
        wire X1_H[4]: mux;
        wire X1_V[4]: mux;
    }

    if variant [scm, ecp4] {
        // indices 0, 1, 4, 5, 6, 7 are valid
        const NUM_X1 = 8;
    } else if variant [ecp2, ecp2m, xp2, ecp3] {
        // indices 0, 1, 4, 5 are valid (and 1, 4 are aliased to X1_[HV])
        const NUM_X1 = 8;
    } else if variant nexus {
        // indices 0..=3 are valid
        const NUM_X1 = 4;
    } else {
        // indices 0, 1 are valid
        const NUM_X1 = 2;
    }
    if variant nexus {
    } else {
        wire X1_W0[NUM_X1]: mux;
    }
    wire X1_W1[NUM_X1]: branch E;
    if variant nexus {
    } else {
        wire X1_E0[NUM_X1]: mux;
    }
    wire X1_E1[NUM_X1]: branch W;
    if variant nexus {
    } else {
        wire X1_S0[NUM_X1]: mux;
    }
    wire X1_S1[NUM_X1]: branch N;
    if variant nexus {
    } else {
        wire X1_N0[NUM_X1]: mux;
    }
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
        // subdivided per clock distribution column
        region_slot PCLK_COL;
        // further subdivided by row
        region_slot PCLK_LEAF;
        wire PCLK[12]: regional PCLK_LEAF;
        wire PCLK_COL[12]: regional PCLK_COL;
        wire PCLK_ROOT[12]: regional PCLK;
    } else if variant [ecp, xp, machxo] {
        // nominally 4 primary and 4 secondary clocks, but they're close enough in behavior
        // per quadrant on ECP, XP; global on MachXO
        region_slot PCLK;
        wire PCLK[8]: regional PCLK;
    } else if variant [ecp2, ecp2m, xp2] {
        // per quadrant
        region_slot PCLK;
        // subdivided per row and per column range
        region_slot PCLK_LEAF;
        wire PCLK[8]: regional PCLK_LEAF;
        wire PCLK_ROOT[8]: regional PCLK;
        wire SCLK_ROOT[8]: regional PCLK;
    } else if variant [ecp3, machxo2] {
        // ecp3: per quadrant; machxo2: global
        region_slot PCLK;
        // subdivided per distribution row and per cols_pclk half-range
        region_slot PCLK_ROW;
        wire PCLK_ROOT[8]: regional PCLK;
        wire SCLK_ROOT[8]: regional PCLK;
        wire PCLK_ROW[8]: regional PCLK_ROW;
        // Rotating primary clocks.
        // per distribution row side
        region_slot PCLK_COL;
        wire RPCLK_COL[8]: regional PCLK_COL;
        region_slot RPCLK[4];
        wire PCLK[8] {
            0 => regional RPCLK[0],
            1 => regional RPCLK[1],
            2 => regional RPCLK[2],
            3 => regional RPCLK[3],
            4 => regional RPCLK[0],
            5 => regional RPCLK[1],
            6 => regional RPCLK[2],
            7 => regional RPCLK[3],
        }
        // rotated aliases of PCLK
        wire RPCLK[8]: multi_branch RCLK;
        wire RPCLK_ROW[8]: branch RCLK;
    } else if variant ecp4 {
        // per quadrant
        region_slot PCLK;
        // subdivided per clock distribution row
        region_slot PCLK_ROW;
        // further subdivided by column and side of the distribution row
        region_slot PCLK_LEAF;
        wire PCLK[20]: regional PCLK_LEAF;
        wire PCLK_ROW[20]: regional PCLK_ROW;
        wire PCLK_ROOT[20]: regional PCLK;
    } else if variant ecp5 {
        // per quadrant
        region_slot PCLK;
        // subdivided per clock distribution column
        region_slot PCLK_COL;
        // subdivided per row and per clock distribution column side
        region_slot PCLK_LEAF;
        wire PCLK[16]: regional PCLK_LEAF;
        wire PCLK_COL[16]: regional PCLK_COL;
        wire PCLK_ROOT[16]: regional PCLK;
    } else if variant crosslink {
        // west/east halves
        region_slot PCLK;
        // subdivided per clock distribution column
        region_slot PCLK_COL;
        // subdivided per row and per clock distribution column side
        region_slot PCLK_LEAF;
        wire PCLK[8]: regional PCLK_LEAF;
        wire PCLK_COL[8]: regional PCLK_COL;
        wire PCLK_HALF[8]: regional PCLK;
        wire PCLK_ROOT[8]: mux;
    } else if variant nexus {
        region_slot PCLK;
        region_slot PCLK_COL;
        region_slot PCLK_LEAF;
        wire PCLK[8]: regional PCLK_LEAF;
        wire PCLK_MUX[16]: mux;
        wire PCLK_COL[16]: regional PCLK_COL;
        wire PCLK_ROOT[16]: regional PCLK;
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        region_slot RSCLK[4];
        region_slot RSCLK_V;
        region_slot SCLK_H;
        // Rotating secondary clocks.
        wire SCLK[8] {
            0 => regional RSCLK[0],
            1 => regional RSCLK[1],
            2 => regional RSCLK[2],
            3 => regional RSCLK[3],
            4 => regional RSCLK[0],
            5 => regional RSCLK[1],
            6 => regional RSCLK[2],
            7 => regional RSCLK[3],
        }
        // rotated aliases of SCLK
        wire RSCLK[8]: multi_branch RCLK;

        // Distribution wires supplying RSCLK.
        wire RSCLK_V[8]: regional RSCLK_V;
        wire RSCLK_V_N[8]: branch S;
        wire SCLK_H[8]: regional SCLK_H;
        wire RSCLK_H[8]: multi_branch RCLK;
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

    wire IMUX_A[8]: mux;
    wire IMUX_B[8]: mux;
    wire IMUX_C[8]: mux;
    wire IMUX_D[8]: mux;
    if variant nexus {
        wire IMUX_MUXA[8]: mux;
        wire IMUX_MUXB[8]: mux;
        wire IMUX_MUXC[8]: mux;
        wire IMUX_MUXD[8]: mux;
    }
    wire IMUX_M[8]: mux;

    // ECP4. ECP5, Crosslink: only 2 indices used, but has IMUX_MUX*
    // others: all 4 indices used
    if variant [ecp4, ecp5, crosslink] {
        wire IMUX_CLK[2]: mux;
        wire IMUX_LSR[2]: mux;
        wire IMUX_CE[4]: mux;
        wire IMUX_MUXCLK[4]: mux;
        wire IMUX_MUXLSR[4]: mux;
        if variant ecp4 {
            wire IMUX_CLK_DELAY[2]: mux;
        }
    } else if variant nexus {
        wire IMUX_CLK[2]: mux;
        wire IMUX_LSR[2]: mux;
        wire IMUX_CE[2]: mux;
    } else {
        wire IMUX_CLK[4]: mux;
        wire IMUX_LSR[4]: mux;
        wire IMUX_CE[4]: mux;
    }

    wire OUT_F[8]: bel;
    wire OUT_F_W[8]: branch E;
    wire OUT_F_E[8]: branch W;
    wire OUT_Q[8]: bel;
    if variant [ecp5, crosslink, nexus] {
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
        wire IMUX_MACO_ECLK[4]: mux;
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

    if variant scm {
        wire IMUX_PCLK[24]: mux;
    } else if variant [ecp3, machxo2] {
        wire IMUX_DCC_CLKI[6]: mux;
    } else if variant [ecp4, ecp5] {
        wire IMUX_DCC_CLKI[16]: mux;
        wire OUT_DCC_CLKO[16]: mux;
    } else if variant crosslink {
        wire IMUX_DCC_CLKI[8]: mux;
        wire OUT_DCC_CLKO[8]: mux;
    }
    if variant scm {
        wire OUT_DCS_DCSOUT[2]: bel;
    } else if variant [ecp, xp, ecp2, ecp2m, xp2, ecp3] {
        wire IMUX_DCS_CLKA[2]: mux;
        wire IMUX_DCS_CLKB[2]: mux;
    } else if variant ecp4 {
        wire IMUX_DCS_CLKA[2]: mux;
        wire IMUX_DCS_CLKB[2]: mux;
        wire IMUX_DCS_CLKC[2]: mux;
        wire IMUX_DCS_CLKD[2]: mux;
        wire OUT_DCS_DCSOUT[2]: bel;
    } else if variant ecp5 {
        wire IMUX_DCS_CLKA[2]: mux;
        wire IMUX_DCS_CLKB[2]: mux;
        wire OUT_DCS_DCSOUT[2]: bel;
    } else if variant crosslink {
        wire IMUX_DCS_CLKA[1]: mux;
        wire IMUX_DCS_CLKB[1]: mux;
        wire OUT_DCS_DCSOUT[1]: bel;
    }
    if variant machxo2 {
        wire OUT_DCM_DCMOUT[2]: bel;
        wire IMUX_DCM_CLK0[2]: mux;
        wire IMUX_DCM_CLK1[2]: mux;
        wire IMUX_ECLKBRIDGECS_CLK0[2]: mux;
        wire IMUX_ECLKBRIDGECS_CLK1[2]: mux;
    }

    // Edge clocks.
    if variant scm {
        region_slot ECLK;
        wire ECLK[8]: regional ECLK;
    } else if variant [ecp2, ecp2m, xp2, ecp3, machxo2, ecp5, crosslink] {
        region_slot ECLK;
        wire ECLK[2]: regional ECLK;

        if variant [ecp3, machxo2] {
            wire IMUX_ECLKSYNC_ECLKI[2]: mux;
        }
        if variant [ecp5, crosslink] {
            wire IMUX_ECLKSYNC_ECLKI[4]: mux;
            wire OUT_ECLKSYNC_ECLKO[4]: bel;
        }
        if variant [ecp3, machxo2, ecp5] {
            wire BRGECLK[2]: mux;
        }
        if variant ecp5 {
            wire OUT_ECLKBRIDGECS_ECSOUT[1]: bel;
            wire OUT_BRGECLKSYNC_ECLKO[1]: bel;
        }
        if variant crosslink {
            wire ECLKFB: mux;
        }
    } else if variant ecp4 {
        region_slot ECLK;
        wire ECLK[4]: regional ECLK;
        wire IMUX_ECLKSYNC_ECLKI[16]: mux;
        wire OUT_ECLKSYNC_ECLKO[16]: bel;
        wire OUT_ECLKBRIDGECS_ECSOUT[2]: bel;
        wire OUT_BRGECLKSYNC_ECLKO[2]: bel;
        wire BRGECLK[4]: mux;
    } else if variant nexus {
        region_slot ECLK;
        wire ECLK[4]: regional ECLK;
        wire IMUX_ECLKSYNC_ECLKI[12]: mux;
        wire OUT_ECLKSYNC_ECLKO[12]: bel;
        wire OUT_ECLKDIV_DIVOUT[12]: bel;
        wire ECLK_TO_DDRDLL[4]: mux;
        wire ECLKFB: mux;
    }

    if variant scm {
        wire OUT_IO_INCK[12]: bel;
        wire IMUX_IO_ECLK[12]: mux;
        wire IMUX_IO_ELSR[12]: mux;
        wire IMUX_IO_TD[12]: mux;
    } else if variant machxo {
        wire OUT_F_PLC_IO[6]: branch IO_FROM_PLC;
        wire OUT_Q_PLC_IO[6]: branch IO_FROM_PLC;
        // mux of the above two
        wire PLC_IO[6]: mux;

        // alias of PLC_IO, but with complex mapping
        wire IMUX_IO_O_PLC[6]: mux;
        wire IMUX_IO_O[6]: mux;
        wire IMUX_IO_T[6]: mux;
        wire OUT_IO_INCK[6]: bel;

        // alias of OUT_IO_INCK
        wire IO_GCLK_S[2]: mux;
        wire IO_GCLK_N[2]: mux;
        wire IO_PLL_CLKI: mux;
        wire IO_PLL_CLKFB: mux;
    } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
        region_slot DQS;
        region_slot DQSDLL;

        wire OUT_IO_INCK[2]: bel;
        if variant [ecp2, ecp2m, xp2] {
            wire IMUX_IO_ECLKI[2]: mux;
            wire IMUX_IO_ECLKO[2]: mux;
        }
    } else if variant ecp3 {
        region_slot DQS;
        region_slot DQSDLL;

        wire OUT_IO_INCK[4]: bel;
        wire IMUX_IO_ECLK[4]: mux;
        wire IMUX_IO_ECLKDQSR[4]: mux;

        wire OUT_DQS_ECLKDQSR: regional DQS;
        wire IMUX_DQS_ECLK: mux;
        wire IMUX_DQS_ECLKW: mux;
    } else if variant machxo2 {
        wire OUT_IO_INCK[4]: bel;
        wire IMUX_IO_ECLK[2]: mux;

        wire OUT_CLKFBBUF_Z[2]: bel;
        wire PLLCLKFB[2]: mux;
    } else if variant ecp4 {
        region_slot DQS;
        wire OUT_IO_INCK[4]: bel;
        wire IMUX_IO_ECLK[4]: mux;
        wire IMUX_DQS_ECLK: mux;
        wire IMUX_DQS_ECLK90: mux;
        wire IMUX_DQS_DDRDEL: mux;
        wire OUT_DQS_CRUCLK: regional DQS;
        wire OUT_DQS_NEXTCLK: regional DQS;
        wire OUT_DQS_DIVCLK: bel;

        // aliased to one of the below global wires
        wire OUT_DDRDLL_DDRDEL: bel;
        wire DDRDEL_SW[2]: regional GLOBAL;
        wire DDRDEL_SE[2]: regional GLOBAL;
        wire DDRDEL_NW[2]: regional GLOBAL;
        wire DDRDEL_NE[2]: regional GLOBAL;
    } else if variant ecp5 {
        region_slot DQS;
        wire OUT_IO_INCK[4]: bel;
        wire IMUX_IO_ECLK[4]: mux;
        wire IMUX_DQS_ECLK: mux;
        wire IMUX_DQS_DDRDEL: mux;

        // aliased to one of the below regional wires
        wire OUT_DDRDLL_DDRDEL: bel;
        region_slot DDRDLL;
        wire DDRDEL_S: regional DDRDLL;
        wire DDRDEL_N: regional DDRDLL;
    } else if variant crosslink {
        wire OUT_IO_INCK[4]: bel;
        wire IMUX_IO_ECLK[4]: mux;
        wire OUT_DDRDLL_DDRDEL: regional ECLK;
    } else if variant nexus {
        region_slot DQS;
        wire OUT_IO_INCK[2]: bel;
        wire IMUX_IO_ECLK[2]: mux;
        wire IMUX_DQS_ECLK: mux;
        wire IMUX_DDRDLL_CLK: mux;
        wire IMUX_DDRDLL_ECLK[4]: mux;
    }

    if variant scm {
        // aliased to OUT_IO_INCK
        wire IO_GCLK[8]: mux;
        wire IO_PLL[6]: mux;
    } else if variant machxo2 {
        // aliased to OUT_IO_INCK
        wire IO_GCLK_W[3]: regional GLOBAL;
        wire IO_GCLK_E[1]: regional GLOBAL;
        wire IO_GCLK_S[2]: regional GLOBAL;
        wire IO_GCLK_N[2]: regional GLOBAL;
        wire IO_PLL_IN: mux;
        wire IO_PLL_FB: mux;
        wire OUT_DLLDEL_CLKO[3]: bel;
        wire INCK[3]: mux;
    } else if variant [ecp4, ecp5, crosslink] {
        wire IMUX_DLLDEL_DDRDEL: mux;
        wire OUT_DLLDEL_Z: bel;
        wire INCK[1]: mux;
    } else if variant nexus {
        // aliased to OUT_IO_INCK
        wire IO_DLLDEL: mux;
        wire OUT_DLLDEL_CLKO[1]: bel;
    }

    if variant scm {
        wire IMUX_PLL_CLKI[2]: mux;
        wire IMUX_PLL_CLKFB[2]: mux;
        wire OUT_PLL_CLKINTFB[2]: bel;
        wire OUT_PLL_CLKOP[2]: bel;
        wire OUT_PLL_CLKOS[2]: bel;
    } else if variant [machxo, ecp, xp, ecp2, ecp2m, xp2, ecp3] {
        wire IMUX_PLL_CLKI[1]: mux;
        wire IMUX_PLL_CLKFB[1]: mux;
        wire OUT_PLL_CLKINTFB[1]: bel;
        wire OUT_PLL_CLKOP[1]: bel;
        wire OUT_PLL_CLKOS[1]: bel;
        wire OUT_PLL_CLKOK: bel;
        if variant [xp2, ecp3] {
            wire OUT_PLL_CLKOK2: bel;
        }
    } else if variant [machxo2, ecp5, crosslink] {
        wire IMUX_PLL_CLKI[2]: mux;
        wire IMUX_PLL_CLKFB[1]: mux;
        wire OUT_PLL_CLKINTFB[1]: bel;
        wire OUT_PLL_CLKOP[1]: bel;
        wire OUT_PLL_CLKOS[1]: bel;
        wire OUT_PLL_CLKOS2[1]: bel;
        wire OUT_PLL_CLKOS3[1]: bel;
    } else if variant ecp4 {
        wire IMUX_PLL_CLKI[4]: mux;
        wire IMUX_PLL_CLKFB[2]: mux;
        wire OUT_PLL_CLKINTFB[2]: bel;
        wire OUT_PLL_CLKOP[2]: bel;
        wire OUT_PLL_CLKOS[2]: bel;
        wire OUT_PLL_CLKOS2[2]: bel;
        wire OUT_PLL_CLKOS3[2]: bel;
    } else if variant nexus {
        wire OUT_PLL_CLKOP[1]: bel;
        wire OUT_PLL_CLKOS[1]: bel;
        wire OUT_PLL_CLKOS2[1]: bel;
        wire OUT_PLL_CLKOS3[1]: bel;
        wire OUT_PLL_CLKOS4[1]: bel;
        wire OUT_PLL_CLKOS5[1]: bel;
    }

    if variant scm {
        wire IMUX_DLL_CLKI[4]: mux;
        wire IMUX_DLL_CLKFB[4]: mux;
        wire IMUX_DLL_CLKIDUTY[4]: mux;
        wire OUT_DLL_CLKOP[4]: bel;
        wire OUT_DLL_CLKOS[4]: bel;
    } else if variant [ecp2, ecp2m, ecp3] {
        wire PLL_TO_ECLK[2]: mux;
        if variant ecp3 {
            wire PLL_TO_ECLK_TEST[2]: bel;
        }
        wire IMUX_DLL_CLKI[1]: mux;
        wire IMUX_DLL_CLKFB[1]: mux;
        wire OUT_DLL_CLKOP[1]: bel;
        wire OUT_DLL_CLKOS[1]: bel;
        wire IMUX_DLLDEL_CLKI: mux;
        wire OUT_DLLDEL_CLKO[1]: bel;
        if variant [ecp2, ecp2m] {
            wire OUT_DLLDEL_BYPASS: bel;
        }
    }
    if variant scm {
        wire IMUX_CLKDIV_CLKI[4]: mux;
        wire OUT_CLKDIV_CLKO[4]: bel;
        wire OUT_CLKDIV_ELSR[4]: bel;
    } else if variant [ecp2, ecp2m, xp2, ecp3] {
        wire IMUX_CLKDIV_CLKI[1]: mux;
        wire OUT_CLKDIV_CDIV1[1]: bel;
        wire OUT_CLKDIV_CDIV2[1]: bel;
        wire OUT_CLKDIV_CDIV4[1]: bel;
        wire OUT_CLKDIV_CDIV8[1]: bel;
    } else if variant machxo2 {
        wire OUT_CLKDIV_CDIVX[2]: bel;
        wire OUT_CLKDIV_CDIV1[2]: bel;
    } else if variant ecp4 {
        wire IMUX_CLKDIV_CLKI[4]: mux;
        wire OUT_CLKDIV_CDIVX[4]: bel;
        wire OUT_CLKDIV_CDIV1[4]: bel;
    } else if variant ecp5 {
        wire IMUX_CLKDIV_CLKI[2]: mux;
        wire OUT_CLKDIV_CDIVX[2]: bel;
        wire OUT_CLKDIV_CDIV1[2]: bel;
    } else if variant crosslink {
        wire OUT_CLKDIV_CDIVX[4]: bel;
    }
    if variant [ecp3, machxo2, ecp4, ecp5, crosslink] {
        wire IMUX_DDRDLL_CLK: mux;
    }

    if variant scm {
        wire OUT_SERDES_FF_RXCLK_P1: bel;
        wire OUT_SERDES_FF_RXCLK_P2: bel;
        wire OUT_SERDES_FF_SYSCLK_P1: bel;
        wire OUT_SERDES_BS4PAD[4]: bel;
    } else if variant [ecp2m, ecp3] {
        wire OUT_SERDES_FF_TX_F_CLK: bel;
        wire OUT_SERDES_FF_TX_H_CLK: bel;
    } else if variant ecp4 {
        wire OUT_SERDES_Q0_FOPCLKA[4]: bel;
        wire OUT_SERDES_Q1_FOPCLKA[4]: bel;
        wire OUT_SERDES_Q2_FOPCLKA[4]: bel;
        wire OUT_SERDES_Q3_FOPCLKA[4]: bel;
        wire OUT_SERDES_Q0_FOPCLKB[4]: bel;
        wire OUT_SERDES_Q1_FOPCLKB[4]: bel;
        wire OUT_SERDES_Q2_FOPCLKB[4]: bel;
        wire OUT_SERDES_Q3_FOPCLKB[4]: bel;
        wire OUT_SERDES_Q0P_CORECLK: bel;
    } else if variant ecp5 {
        wire OUT_SERDES_FF_TX_PCLK[2]: bel;
        wire OUT_SERDES_FF_RX_PCLK[2]: bel;
        wire OUT_SERDES_REFCLKO: bel;
    } else if variant nexus {
        wire OUT_CDR_SRCLK[2]: bel;
    }

    if variant [machxo2, ecp4, ecp5] {
        region_slot GLOBAL;
        wire OUT_OSC_OSC: regional GLOBAL;
    }
    if variant ecp5 {
        wire OUT_SED_SEDCLKOUT: bel;
        wire OUT_JTAG_JTCK: bel;
    }
    if variant [machxo2, ecp4] {
        wire IMUX_PCNTR_CLK: mux;
    }

    if variant crosslink {
        wire IMUX_MIPI_CLKREF: mux;
        wire OUT_MIPI_CLKHSBYTE: bel;
        wire OUT_MIPI_HSBYTECLKS: bel;
        wire OUT_MIPI_HSBYTECLKD: bel;
        wire OUT_OSC_LFCLKOUT: bel;
        wire OUT_OSC_HFCLKOUT: bel;
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
        if variant [ecp2, ecp2m, xp2, ecp3, machxo2, ecp4, ecp5, nexus] {
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

        if variant [scm, ecp, xp, machxo2, ecp4, ecp5, crosslink, nexus] {
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
        bel_slot IO_INT: routing;
        if variant scm {
            bel_slot IO[12]: IO;
            bel_slot PICTEST[3]: PICTEST;
            tile_class IO_W4, IO_E4, IO_S4, IO_N4 {
                cell CELL[2];
                switchbox IO_INT;
                for i in 0..4 {
                    bel IO[i];
                }
                bel PICTEST[0];
            }
            tile_class IO_N8 {
                cell CELL[3];
                switchbox IO_INT;
                for i in 0..8 {
                    bel IO[i];
                }
                for i in 0..2 {
                    bel PICTEST[i];
                }
            }
            tile_class IO_W12, IO_E12, IO_S12, IO_N12 {
                cell CELL[4];
                switchbox IO_INT;
                for i in 0..12 {
                    bel IO[i];
                }
                for i in 0..3 {
                    bel PICTEST[i];
                }
            }
        } else if variant [ecp, xp, ecp2, ecp2m, xp2] {
            bel_slot IO[2]: IO;
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
            bel_slot IO[4]: IO;
            tile_class
                IO_W2, IO_W4, IO_W4_I3C,
                IO_E2, IO_E4,
                IO_S4, SIO_S2, SIO_S4,
                IO_N4, SIO_N2, SIO_N4
            {
                cell CELL;
                if tile_class [IO_S4, IO_N4] {
                    switchbox IO_INT;
                }
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
            bel_slot IO[4]: IO;
            tile_class
                IO_W, IO_W_EBR_S, IO_W_EBR_N, IO_W_DSP_S, IO_W_DSP_N,
                IO_E, IO_E_EBR_S, IO_E_EBR_N, IO_E_DSP_S, IO_E_DSP_N,
                IO_N
            {
                cell CELL[4];
                switchbox IO_INT;
                for i in 0..4 {
                    bel IO[i];
                }
            }
        } else if variant ecp5 {
            bel_slot IO[4]: IO;
            tile_class IO_W4, IO_E4 {
                cell CELL[3];
                switchbox IO_INT;
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
            bel_slot IO[4]: IO;
            tile_class IO_S4 {
                cell CELL[4];
                switchbox IO_INT;
                for i in 0..4 {
                    bel IO[i];
                }
            }
            tile_class IO_S1A, IO_S1B {
                cell CELL;
                bel IO[0];
            }
        } else if variant nexus {
            bel_slot IO[2]: IO;
            // tile     IO[0].TSDATA0   IO[1].TSDATA0
            // *_A      B1              B4
            // *_B      B1              C0
            // *_C      C4              A0
            tile_class IO_W_A, IO_W_B, IO_E_A, IO_E_B, IO_E_C, IO_N_A, IO_N_B {
                cell CELL;
                for i in 0..2 {
                    bel IO[i];
                }
            }
            tile_class IO_HP_S, IO_HP_S_JD5U, IO_HP_S_LPDDR4 {
                cell CELL[2];
                switchbox IO_INT;
                for i in 0..2 {
                    bel IO[i];
                }
            }
        }
    }

    tile_slot BEL {
        bel_slot BEL_INT: routing;

        bel_slot SLICE[4]: SLICE;

        tile_class PLC, FPLC {
            if tile_class FPLC {
                if variant [scm, machxo2, ecp4, ecp5, crosslink, nexus] {
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

                    if variant [ecp5, crosslink, nexus] {
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
                            input CE = IMUX_CE[i];
                        } else if variant nexus {
                            input CLK = IMUX_CLK[i / 2];
                            input LSR = IMUX_LSR[i / 2];
                            input CE = IMUX_CE[i / 2];
                        } else {
                            input CLK = IMUX_CLK[i];
                            input LSR = IMUX_LSR[i];
                            input CE = IMUX_CE[i];
                        }
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
            tile_class IO_INT_W, IO_INT_E, IO_INT_S {
                cell CELL;
                switchbox BEL_INT;
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
        } else if variant nexus {
            bel_slot EBR[4]: EBR;
            tile_class EBR {
                cell CELL[11];
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
        } else if variant nexus {
            bel_slot DSP[2]: DSP;
            tile_class DSP_W, DSP_E {
                cell CELL[12];
                for i in 0..2 {
                    bel DSP[i];
                }
            }
        }

        // MACO, SERDES
        if variant scm {
            bel_slot MACO: MACO;
            tile_class MACO_W, MACO_E {
                cell CELL[4];
                cell CELL_IO[2];
                cell CELL_EBR[10];
                switchbox BEL_INT;
                bel MACO;
            }

            bel_slot SERDES: SERDES;
            tile_class SERDES_W, SERDES_E {
                cell CELL[7];
                cell CENTER;
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
            bel_slot SERDES: SERDES;
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
            bel_slot SERDES: SERDES;
            tile_class SERDES {
                cell CELL[12];
                bel SERDES;
            }
        } else if variant crosslink {
            bel_slot MIPI: MIPI;
            tile_class MIPI_W, MIPI_E {
                cell CELL[24];
                cell IO[2];
                switchbox BEL_INT;
                bel MIPI;
            }
        } else if variant nexus {
            bel_slot MIPI: MIPI;
            // TODO

            bel_slot CDR[2]: CDR;
            tile_class CDR_S, CDR_W_JE5D30, CDR_W_JD5F {
                if tile_class CDR_W_JD5F {
                    cell CELL[4];
                } else {
                    cell CELL[3];
                }
                cell PLL;
                bel CDR[0];
                bel CDR[1];
            }

            bel_slot ADC: ADC;
            tile_class ADC_JE5D15, ADC_JE5D30, ADC_JD5D {
                cell CELL[3], PLL;
                bel ADC;
            }
        }

        // PLLs etc
        if variant scm {
            bel_slot PLL[2]: PLL;
            bel_slot PLL_SMI: PLL_SMI;
            bel_slot DLL[4]: DLL;
            bel_slot DLL_DCNTL[2]: DLL_DCNTL;
            bel_slot PROMON: PROMON;
            bel_slot RNET: RNET;
            bel_slot SERDES_CORNER: SERDES_CORNER;
            bel_slot M0: IPAD;
            bel_slot M1: IPAD;
            bel_slot M2: IPAD;
            bel_slot M3: IPAD;
            bel_slot RESETN: IPAD;
            bel_slot RDCFGN: RDCFGN;
            bel_slot CCLK: IPAD;
            bel_slot TCK: IPAD;
            bel_slot TMS: IPAD;
            bel_slot TDI: IPAD;
            tile_class PLL_SW, PLL_SE {
                cell CELL_S[4];
                cell CELL_WE[2];
                switchbox BEL_INT;
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
                cell SERDES[4];
                switchbox BEL_INT;
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

            bel_slot DQSDLL: DDRDLL;
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
            bel_slot DQSDLL: DDRDLL;
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

            bel_slot DQSDLL: DDRDLL;
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
            bel_slot PLL[1]: PLL;
            bel_slot DLL[1]: DLL;
            bel_slot DLLDEL[1]: DLLDEL;
            bel_slot DQSDLL: DDRDLL;
            bel_slot CLKDIV[1]: CLKDIV;
            tile_class PLL_DLL_W, PLL_DLL_E, PLL_DLL_A_W, PLL_DLL_A_E {
                cell CELL[15];
                cell CELL_WE[4];
                cell IO_N_DLL;
                cell IO_N_PLL;

                switchbox BEL_INT;
                bel PLL[0];
                bel DLL[0];
                bel DLLDEL[0];
                bel DQSDLL;
                bel CLKDIV[0];
            }
            tile_class PLL_W, PLL_E, PLL_A_W, PLL_A_E {
                cell CELL[12];
                cell CELL_WE;
                cell IO_N;

                switchbox BEL_INT;
                bel PLL[0];
            }
        } else if variant machxo2 {
            bel_slot PLL[1]: PLL;
            tile_class PLL_W, PLL_E {
                cell CELL[2], CLK_S;
                switchbox BEL_INT;
                bel PLL[0];
            }

            bel_slot DQSDLL: DDRDLL;
            tile_class DQSDLL_S, DQSDLL_N {
                cell CELL;
                switchbox BEL_INT;
                bel DQSDLL;
            }
        } else if variant ecp4 {
            bel_slot PLL[2]: PLL;
            tile_class PLL_SW, PLL_SE, PLL_NW, PLL_NE {
                cell CELL[4];
                cell IO_IN[2];
                cell IO_WE;
                if tile_class [PLL_NW, PLL_NE] {
                    cell IO_N[2];
                }
                if tile_class PLL_SW {
                    cell IO_FB[2];
                    cell SERDES;
                }

                switchbox BEL_INT;
                for i in 0..2 {
                    bel PLL[i];
                }
            }

            bel_slot DDRDLL: DDRDLL;
            tile_class DDRDLL_S, DDRDLL_N {
                cell CELL;
                cell WE;
                switchbox BEL_INT;
                bel DDRDLL;
            }

            bel_slot DTR: DTR;
            tile_class DTR_S, DTR_N {
                cell CELL;
                bel DTR;
            }
        } else if variant ecp5 {
            bel_slot PLL[1]: PLL;
            tile_class PLL_SW, PLL_SE, PLL_NW, PLL_NE {
                cell CELL[2];
                cell IO_WE;
                if tile_class [PLL_SW, PLL_SE] {
                    cell IO_IN;
                } else {
                    cell IO_N[2];
                    cell IO_IN[2];
                }
                switchbox BEL_INT;
                bel PLL[0];
            }

            bel_slot DDRDLL: DDRDLL;
            tile_class DDRDLL {
                cell CELL;
                switchbox BEL_INT;
                bel DDRDLL;
            }

            bel_slot DTR: DTR;
            tile_class DTR {
                cell CELL;
                bel DTR;
            }
        } else if variant crosslink {
            bel_slot PLL[1]: PLL;
            tile_class PLL {
                cell CELL[2];
                cell CLK_S;
                cell IO[5];

                switchbox BEL_INT;
                bel PLL[0];
            }

            bel_slot BCINRD: BCINRD;
            bel_slot BCLVDSO: BCLVDSO;
            bel_slot DDRDLL: DDRDLL;
            tile_class BC {
                cell CELL;
                switchbox BEL_INT;
                bel BCINRD;
                bel BCLVDSO;
                bel DDRDLL;
            }
        } else if variant nexus {
            bel_slot PLL[1]: PLL;
            tile_class PLL_SW_JE5D15 {
                cell CELL[3];
                bel PLL[0];
            }
            tile_class PLL_SE_JE5D15 {
                cell CELL[3];
                bel PLL[0];
            }
            tile_class PLL_SW_JE5D30 {
                cell CELL, CELL_N, CELL_E;
                bel PLL[0];
            }
            tile_class PLL_SE_JE5D30 {
                cell CELL, CELL_N, CELL_S;
                bel PLL[0];
            }
            tile_class PLL_NW_JE5D30 {
                cell CELL, CELL_S, CELL_E;
                bel PLL[0];
            }
            tile_class PLL_SW_JD5D {
                cell CELL, CELL_S, CELL_SE;
                bel PLL[0];
            }
            tile_class PLL_NE_JD5D {
                cell CELL, CELL_S, CELL_W;
                bel PLL[0];
            }
            tile_class PLL_NW_JD5F {
                cell CELL[3];
                bel PLL[0];
            }

            bel_slot DDRDLL: DDRDLL;
            tile_class DDRDLL_S, DDRDLL_W, DDRDLL_E, DDRDLL_W_JD5U, DDRDLL_E_JD5U {
                cell CELL;
                switchbox BEL_INT;
                bel DDRDLL;
            }

            bel_slot DLLDEL[1]: DLLDEL;
            tile_class
                DLLDEL_W_A, DLLDEL_W_B,
                DLLDEL_E_A, DLLDEL_E_B,
                DLLDEL_S_A, DLLDEL_S_B,
                DLLDEL_N_B
            {
                cell CELL;
                bel DLLDEL[0];
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
            bel_slot DQS[1]: DQS;
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
                switchbox BEL_INT;
                bel DQS[0];
            }
        } else if variant ecp5 {
            bel_slot DQS[1]: DQS;
            tile_class DQS_W, DQS_E {
                cell CELL[6];
                switchbox BEL_INT;
                bel DQS[0];
            }
        } else if variant nexus {
            bel_slot DQS[1]: DQS;
            tile_class DQS_S, DQS_L, DQS_JD5D, DQS_JD5D_HOLE, DQS_JD5F, DQS_JD5F_HOLE {
                if tile_class [DQS_S, DQS_L] {
                    cell CELL[16];
                } else if tile_class [DQS_JD5D_HOLE, DQS_JD5F_HOLE] {
                    cell CELL[24];
                } else {
                    cell CELL[12];
                }
                switchbox BEL_INT;
                bel DQS[0];
            }
        }

        // CONFIG etc.
        if variant scm {
            bel_slot JTAG: JTAG;
            bel_slot RDBK: RDBK;
            bel_slot GSR: GSR;
            bel_slot OSC: OSC;
            bel_slot START: START;
            bel_slot SYSBUS: SYSBUS;
            tile_class CONFIG {
                cell CELL[12];
                cell CELL_CLK;
                bel JTAG;
                bel RDBK;
                bel GSR;
                bel OSC;
                bel START;
                bel SYSBUS;
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
            bel_slot PCNTR: PCNTR;
            bel_slot TSALL: TSALL;
            bel_slot EFB: EFB;
            bel_slot ESB: ESB;
            tile_class CONFIG, CONFIG_XO3D {
                if tile_class CONFIG {
                    cell CELL[4];
                } else {
                    cell CELL[10];
                }
                switchbox BEL_INT;
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
            bel_slot PCNTR: PCNTR;
            bel_slot EFB: EFB;
            tile_class CONFIG {
                cell CELL[6];
                cell CELL_CLK[5];
                switchbox BEL_INT;
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
            bel_slot CFGTEST: CFGTEST;
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

            bel_slot PMU: PMU;
            tile_class PMU {
                cell CELL;
                bel PMU;
            }

            bel_slot I2C: I2C;
            bel_slot NVCMTEST: NVCMTEST;
            tile_class I2C_W, I2C_E {
                cell CELL[2];
                bel I2C;
                if tile_class I2C_E {
                    bel NVCMTEST;
                }
            }
        }

        if variant ecp4 {
            bel_slot PVTTEST: PVTTEST;
            bel_slot PVTCAL: PVTCAL;
            tile_class PVTTEST {
                cell CELL;
                bel PVTTEST;
                bel PVTCAL;
            }
        }
    }

    if variant machxo2 {
        tile_slot BC {
            bel_slot BCPG: BCPG;
            bel_slot BCINRD: BCINRD;
            bel_slot BCLVDSO: BCLVDSO;
            bel_slot BCSLEWRATE: BCSLEWRATE;
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
            bel_slot BCINRD: BCINRD;
            bel_slot BCPG: BCPG;
            bel_slot BCLVDSO: BCLVDSO;
            bel_slot BCPUSL: BCPUSL;
            bel_slot BREFTEST: BREFTEST;
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
            bel_slot BCINRD: BCINRD;
            bel_slot BCLVDSO: BCLVDSO;
            bel_slot BREFTEST: BREFTEST;
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
            bel_slot DCC_SW[6]: DCC;
            bel_slot DCC_SE[6]: DCC;
            bel_slot DCC_NW[6]: DCC;
            bel_slot DCC_NE[6]: DCC;
            bel_slot DCS_SW[2]: DCS;
            bel_slot DCS_SE[2]: DCS;
            bel_slot DCS_NW[2]: DCS;
            bel_slot DCS_NE[2]: DCS;
            tile_class CLK_ROOT {
                cell CENTER_S[2], CENTER_M[2], CENTER_N[2];
                cell EDGE_W[6], EDGE_E[8], EDGE_S[6], EDGE_N[6];
                cell PLL_SW[3], PLL_SE[3], PLL_NW[2], PLL_NE[2];
                cell SERDES[4];

                switchbox CLK_INT;
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
            bel_slot CLKTEST: CLKTEST;
            bel_slot DCC[8]: DCC;
            bel_slot DCM[2]: DCM;
            bel_slot ECLKBRIDGECS[2]: ECLKBRIDGECS;
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
                    cell PLL_NW, PLL_NE;
                } else {
                    cell CELL[3];
                    cell EDGE_W, EDGE_E, EDGE_S, EDGE_N;
                    cell EBR_S[3];
                    cell EBR_N[3];
                    cell EBR_SW, EBR_SE;
                    cell PLL_NW, PLL_NE;
                }
                switchbox CLK_INT;
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

            bel_slot DLLDEL[3]: DLLDEL;
            bel_slot CLKDIV[4]: CLKDIV;
            bel_slot DQS[2]: DQS;
            bel_slot ECLKSYNC[2]: ECLKSYNC;
            bel_slot CLKFBBUF[2]: CLKFBBUF;
            tile_class CLK_S, CLK_N {
                cell CELL, ROOT, PLL_NW, PLL_NE;

                switchbox CLK_INT;
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
                switchbox CLK_INT;
                for i in 0..3 {
                    bel DLLDEL[i];
                }
            }
            tile_class CLK_E {
                cell CELL;
                switchbox CLK_INT;
                bel DLLDEL[0];
            }
            tile_class CLK_E_DQS {
                cell CELL[5];
                switchbox CLK_INT;
                bel DLLDEL[0];
                for i in 0..2 {
                    bel DQS[i];
                }
            }
        } else if variant scm {
            bel_slot CLKDIV[4]: CLKDIV;
            bel_slot DCS[2]: DCS;
            bel_slot CLKTEST: CLKTEST;
            tile_class CLK_W, CLK_E, CLK_N {
                cell CELL[2];
                if tile_class [CLK_W, CLK_E] {
                    cell PLL_S, PLL_N;
                    cell SERDES[4];
                }
                switchbox CLK_INT;
                for i in 0..4 {
                    bel CLKDIV[i];
                }
                for i in 0..2 {
                    bel DCS[i];
                }
            }
            tile_class CLK_S {
                cell CELL[2];
                cell CLK_SW, CLK_SE;
                switchbox CLK_INT;
                for i in 0..2 {
                    bel DCS[i];
                }
            }
            tile_class CLK_SW, CLK_SE {
                cell CELL[2];
                cell PLL;
                switchbox CLK_INT;
                for i in 0..4 {
                    bel CLKDIV[i];
                }
            }
            tile_class CLK_ROOT {
                cell CELL_S[2];
                cell CELL_N[2];
                cell CLK_W, CLK_E, CLK_S, CLK_N;
                switchbox CLK_INT;
                bel CLKTEST;
            }
        } else if variant ecp4 {
            bel_slot CLKTEST: CLKTEST;
            bel_slot DCC_SW[1]: DCC;
            bel_slot DCC_SE[1]: DCC;
            bel_slot DCC_NW[1]: DCC;
            bel_slot DCC_NE[1]: DCC;
            bel_slot DCS[2]: DCS;
            tile_class CLK_ROOT {
                cell ROOT_SW, ROOT_SE, ROOT_NW, ROOT_NE;
                cell CENTER;
                cell CLK_W, CLK_E, CLK_S, CLK_N;
                switchbox CLK_INT;
                bel CLKTEST;
                bel DCC_SW[0];
                bel DCC_SE[0];
                bel DCC_NW[0];
                bel DCC_NE[0];
                for i in 0..2 {
                    bel DCS[i];
                }
            }

            bel_slot CLKTEST_ECLK: CLKTEST;
            bel_slot DCC[16]: DCC;
            bel_slot DLLDEL[8]: DLLDEL;
            bel_slot CLKDIV[4]: CLKDIV;
            bel_slot ECLKSYNC[16]: ECLKSYNC;
            bel_slot BRGECLKSYNC[2]: ECLKSYNC;
            bel_slot ECLKBRIDGECS[2]: ECLKBRIDGECS;
            tile_class
                CLK_N_S, CLK_N_M, CLK_N_L,
                CLK_W_S, CLK_W_M, CLK_W_L,
                CLK_E_S, CLK_E_M, CLK_E_L
            {
                if tile_class CLK_N_L {
                    cell CELL[18];
                    cell BC1, BC2;
                } else {
                    cell CELL[10];
                }
                cell ROOT;
                cell PLL[2];
                if tile_class [CLK_N_M, CLK_W_M, CLK_E_M] {
                    cell FABRIC_IN[2];
                } else if tile_class [CLK_N_L, CLK_W_L, CLK_E_L] {
                    cell FABRIC_IN[4];
                }
                if tile_class [CLK_W_S, CLK_E_S] {
                    cell DQS_S[3], DQS_N[3];
                } else if tile_class [CLK_W_M, CLK_E_M, CLK_W_L, CLK_E_L] {
                    cell DQS_S[6], DQS_N[5];
                } else if tile_class CLK_N_S {
                    cell DQS_W[3], DQS_E[3];
                } else if tile_class CLK_N_M {
                    cell DQS_W[5], DQS_E[5];
                } else if tile_class CLK_N_L {
                    cell DQS_W[7], DQS_E[7];
                }
                if tile_class CLK_E_L {
                    cell SERDES;
                }

                switchbox CLK_INT;
                bel CLKTEST;
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

            bel_slot PCSCLKDIV[4]: PCSCLKDIV;
            tile_class CLK_S_S, CLK_S_M, CLK_S_L {
                cell CELL[2];
                cell SERDES;
                switchbox CLK_INT;
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
                bel CLKTEST;
                for i in 0..16 {
                    bel DCC[i];
                }
            }
        } else if variant ecp5 {
            bel_slot CLKTEST: CLKTEST;
            bel_slot DCC_SW[1]: DCC;
            bel_slot DCC_SE[1]: DCC;
            bel_slot DCC_NW[1]: DCC;
            bel_slot DCC_NE[1]: DCC;
            bel_slot DCS[2]: DCS;
            tile_class CLK_ROOT_S, CLK_ROOT_L {
                if tile_class CLK_ROOT_S {
                    cell SW, SE, NW, NE;
                } else {
                    cell QSW, QSE, SW, SE, NW, NE, QNW, QNE;
                }
                cell CENTER;
                cell CLK_W, CLK_E, CLK_S, CLK_N;

                switchbox CLK_INT;
                bel CLKTEST;
                bel DCC_SW[0];
                bel DCC_SE[0];
                bel DCC_NW[0];
                bel DCC_NE[0];
                for i in 0..2 {
                    bel DCS[i];
                }
            }

            bel_slot DCC[16]: DCC;
            bel_slot CLKTEST_ECLK: CLKTEST;
            bel_slot ECLKSYNC[4]: ECLKSYNC;
            bel_slot DLLDEL[4]: DLLDEL;
            bel_slot CLKDIV[2]: CLKDIV;
            bel_slot PCSCLKDIV[2]: PCSCLKDIV;
            bel_slot BRGECLKSYNC[1]: ECLKSYNC;
            bel_slot ECLKBRIDGECS[1]: ECLKBRIDGECS;
            tile_class CLK_W, CLK_E {
                cell CELL_EDGE[12];
                cell CELL_EBR[2];
                cell FABRIC_IN_M[4];
                cell PLL[2];
                cell ROOT;

                switchbox CLK_INT;
                for i in 0..14 {
                    bel DCC[i];
                }
                bel CLKTEST;
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
            tile_class CLK_N {
                cell CELL[16];
                cell FABRIC_IN_M[4];
                cell DDRDLL_NW, DDRDLL_NE;
                cell PLL[2];

                switchbox CLK_INT;
                for i in 0..12 {
                    bel DCC[i];
                }
                bel CLKTEST;
                for i in 0..4 {
                    bel DLLDEL[i];
                }
            }
            tile_class CLK_S {
                cell CELL[16];
                cell FABRIC_IN_M[4];
                cell SERDES[2];
                cell CONFIG;

                switchbox CLK_INT;
                for i in 0..16 {
                    bel DCC[i];
                }
                bel CLKTEST;
                for i in 0..2 {
                    bel PCSCLKDIV[i];
                }
            }
        } else if variant crosslink {
            bel_slot CLKTEST: CLKTEST;
            bel_slot DCS[1]: DCS;
            tile_class CLK_ROOT {
                cell CELL[2], CLK_S, CLK_N;

                switchbox CLK_INT;
                bel CLKTEST;
                bel DCS[0];
            }

            bel_slot DCC[8]: DCC;
            bel_slot ECLKSYNC[4]: ECLKSYNC;
            bel_slot DLLDEL[4]: DLLDEL;
            bel_slot CLKDIV[4]: CLKDIV;
            bel_slot CLKTEST_ECLK: CLKTEST;
            tile_class CLK_S {
                cell CELL[4];
                cell FABRIC_IN[2];
                cell IO_IN[2];
                cell PLL;
                cell OSC;

                switchbox CLK_INT;
                for i in 0..8 {
                    bel DCC[i];
                }
                bel CLKTEST;
                for i in 0..4 {
                    bel ECLKSYNC[i];
                    bel DLLDEL[i];
                    bel CLKDIV[i];
                }
                bel CLKTEST_ECLK;
            }
            tile_class CLK_N {
                cell CELL[2];
                cell CLK_S;
                cell IO_IN[2];
                cell MIPI[2];

                switchbox CLK_INT;
                for i in 0..6 {
                    bel DCC[i];
                }
                bel CLKTEST;
            }
        } else if variant nexus {
            bel_slot DCC[18]: DCC;
            bel_slot EMIDMUX_TEST: EMIDMUX_TEST;
            bel_slot ECLKSYNC[12]: ECLKSYNC;
            bel_slot ECLKDIV[12]: ECLKDIV;
            bel_slot ECLKBANK_TEST[3]: ECLKBANK_TEST;
            bel_slot ECLKCASMUX_TEST[3]: ECLKCASMUX_TEST;
            bel_slot ECLKPLLFB_TEST[2]: ECLKPLLFB_TEST;
            bel_slot ECLK_TEST: ECLK_TEST;
            tile_class CLK_S_JE5D15, CLK_S_JE5D30, CLK_S_JD5U, CLK_S_JD5F {
                cell CELL[12];
                cell PLL[2];
                cell DDRDLL[2];
                cell IO[6], DLLDEL[6];
                switchbox CLK_INT;
                bel EMIDMUX_TEST;
                for i in 0..18 {
                    bel DCC[i];
                }
                if tile_class [CLK_S_JE5D15, CLK_S_JE5D30, CLK_S_JD5U] {
                    for i in 0..12 {
                        bel ECLKSYNC[i];
                        bel ECLKDIV[i];
                    }
                    for i in 0..3 {
                        bel ECLKBANK_TEST[i];
                        bel ECLKCASMUX_TEST[i];
                    }
                } else {
                    for i in 0..8 {
                        bel ECLKSYNC[i];
                        bel ECLKDIV[i];
                    }
                    for i in 0..2 {
                        bel ECLKBANK_TEST[i];
                        bel ECLKCASMUX_TEST[i];
                    }
                }
                if tile_class [CLK_S_JE5D15, CLK_S_JE5D30] {
                    bel ECLKPLLFB_TEST[0];
                    bel ECLKPLLFB_TEST[1];
                } else if tile_class CLK_S_JD5U {
                    bel ECLKPLLFB_TEST[0];
                } else if tile_class CLK_S_JD5F {
                    bel ECLKPLLFB_TEST[1];
                }
                bel ECLK_TEST;
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
            bel_slot ECLKSYNC[2]: ECLKSYNC;
            tile_class
                ECLK_ROOT_W, ECLK_ROOT_E, ECLK_ROOT_N,
                ECLK_ROOT_W_A, ECLK_ROOT_E_A, ECLK_ROOT_N_A
            {
                cell CELL, ROOT, IO[2];

                if tile_class [ECLK_ROOT_N, ECLK_ROOT_N_A] {
                    cell PLL_W, PLL_E;
                    cell DLL_W, DLL_E;
                } else {
                    cell PLL;
                    if tile_class [ECLK_ROOT_W_A, ECLK_ROOT_E_A] {
                        cell PLL_S;
                    }
                }

                switchbox CLK_INT;
                for i in 0..2 {
                    bel ECLKSYNC[i];
                }
            }
        }
    }

    if variant scm {
        tile_slot PCLK_DRIVER {
            bel_slot PCLK_DRIVER: routing;

            tile_class PCLK_DRIVER {
                cell CELL;
                // PCLK <- PCLK_COL
                switchbox PCLK_DRIVER;
            }

            tile_class PCLK_DRIVER_EBR {
                cell CELL;
                // PCLK <- PCLK_COL
                switchbox PCLK_DRIVER;
            }

            tile_class PCLK_DRIVER_EBR_ROOT {
                cell CELL;
                // PCLK <- PCLK_COL, PCLK_COL <- PCLK_ROOT
                switchbox PCLK_DRIVER;
            }
        }
    }

    if variant [ecp2, ecp2m, xp2] {
        tile_slot PCLK_DRIVER {
            bel_slot PCLK_DRIVER: routing;
            tile_class PCLK_DRIVER {
                cell CELL;
                switchbox PCLK_DRIVER;
            }
        }
    }

    if variant [ecp3, machxo2] {
        tile_slot PCLK_DRIVER {
            bel_slot PCLK_DRIVER: routing;

            tile_class RPCLK_DRIVER, RPCLK_DRIVER_W, RPCLK_DRIVER_E {
                cell CELL;
                switchbox PCLK_DRIVER;
            }
        }

        tile_slot PCLK_COL_DRIVER {
            bel_slot PCLK_COL_DRIVER: routing;
            bel_slot PCLK_DCC_S[2]: DCC;
            bel_slot PCLK_DCC_N[2]: DCC;

            tile_class PCLK_COL_DRIVER, PCLK_COL_DRIVER_W, PCLK_COL_DRIVER_E {
                cell N, S;
                if tile_class PCLK_COL_DRIVER {
                } else {
                    switchbox PCLK_COL_DRIVER;
                }
                for i in 0..2 {
                    bel PCLK_DCC_S[i];
                    bel PCLK_DCC_N[i];
                }
            }
            if variant machxo2 {
                tile_class PCLK_COL_DRIVER_IO_N, PCLK_COL_DRIVER_IO_N_W, PCLK_COL_DRIVER_IO_N_E {
                    cell CELL;
                    if tile_class PCLK_COL_DRIVER_IO_N {
                    } else {
                        switchbox PCLK_COL_DRIVER;
                    }
                    for i in 0..2 {
                        bel PCLK_DCC_N[i];
                    }
                }
            }
        }

        tile_slot PCLK_ROW_DRIVER {
            bel_slot PCLK_ROW_DRIVER: routing;
            tile_class PCLK_ROW_DRIVER {
                cell ROW, W, E;
                switchbox PCLK_ROW_DRIVER;
            }
        }
    }

    if variant ecp4 {
        tile_slot PCLK_DRIVER {
            bel_slot PCLK_DRIVER: routing;

            tile_class PCLK_DRIVER {
                cell ROW, S, N;
                switchbox PCLK_DRIVER;
            }
        }

        tile_slot PCLK_ROW_DRIVER {
            bel_slot PCLK_ROW_DRIVER: routing;

            tile_class PCLK_ROW_DRIVER_W, PCLK_ROW_DRIVER_E {
                cell CELL;
                switchbox PCLK_ROW_DRIVER;
            }
        }
    }

    if variant [ecp5, crosslink, nexus] {
        tile_slot PCLK_DRIVER {
            bel_slot PCLK_DRIVER: routing;

            tile_class PCLK_DRIVER {
                cell W, E;
                if variant nexus {
                    cell COL;
                }
                switchbox PCLK_DRIVER;
            }
        }

        tile_slot PCLK_COL_DRIVER {
            bel_slot PCLK_COL_DRIVER: routing;

            tile_class PCLK_COL_DRIVER {
                cell CELL;
                switchbox PCLK_COL_DRIVER;
            }
        }
    }

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        tile_slot SCLK_DRIVER {
            bel_slot SCLK_DRIVER: routing;
            tile_class RSCLK_DRIVER {
                cell CELL;
                switchbox SCLK_DRIVER;
            }
            if variant [ecp3, machxo2] {
                tile_class RSCLK_DRIVER_W, RSCLK_DRIVER_E {
                    cell CELL;
                    switchbox SCLK_DRIVER;
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
        tile_slot SCLK_H_SPLITTER {
            bel_slot SCLK_H_SPLITTER: routing;

            tile_class SCLK_H_SPLITTER, SCLK_H_ROOT {
                if variant machxo2 {
                    if tile_class SCLK_H_SPLITTER {
                        skip;
                    }
                }
                cell W, E;
                switchbox SCLK_H_SPLITTER;
            }
        }
    }

    connector_slot W {
        opposite E;
        connector_class PASS_W {
            if variant nexus {
                pass X1_E1 = X1_H;
            } else {
                pass X1_E1 = X1_E0;
            }
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
            if variant nexus {
                reflect X1_E1[0] = X1_H[1];
                reflect X1_E1[1] = X1_H[0];
                reflect X1_E1[2] = X1_H[3];
                reflect X1_E1[3] = X1_H[2];
            } else if variant [ecp5, crosslink] {
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
            if variant nexus {
                pass X1_W1 = X1_H;
            } else {
                pass X1_W1 = X1_W0;
            }
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
            if variant [ecp5, crosslink, nexus] {
            } else {
                pass OUT_OFX_W = OUT_OFX;
            }
        }
        connector_class TERM_E {
            if variant nexus {
                reflect X1_W1[0] = X1_H[1];
                reflect X1_W1[1] = X1_H[0];
                reflect X1_W1[2] = X1_H[3];
                reflect X1_W1[3] = X1_H[2];
            } else if variant [ecp5, crosslink] {
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
            if variant nexus {
                pass X1_N1 = X1_V;
            } else {
                pass X1_N1 = X1_N0;
            }
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
            if variant nexus {
                reflect X1_N1[0] = X1_V[1];
                reflect X1_N1[1] = X1_V[0];
                reflect X1_N1[2] = X1_V[3];
                reflect X1_N1[3] = X1_V[2];
            } else if variant [ecp5, crosslink] {
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
            if variant nexus {
                pass X1_S1 = X1_V;
            } else {
                pass X1_S1 = X1_S0;
            }
            pass X2_S1 = X2_S0;
            pass X2_S2 = X2_S1;
            for i in 0..6 {
                pass "X6_S{i+1}" = "X6_S{i}";
            }
        }
        connector_class TERM_N {
            if variant nexus {
                reflect X1_S1[0] = X1_V[1];
                reflect X1_S1[1] = X1_V[0];
                reflect X1_S1[2] = X1_V[3];
                reflect X1_S1[3] = X1_V[2];
            } else if variant [ecp5, crosslink] {
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

    if variant [ecp2, ecp2m, xp2, ecp3, machxo2] {
        connector_slot RCLK {
            opposite RCLK;
            for i in 0..4 {
                connector_class "RCLK{i}" {
                    for j in 0..4 {
                        reflect RSCLK[j] = SCLK[(i + j) % 4];
                        reflect RSCLK[4 + j] = SCLK[4 + (i + j) % 4];
                        reflect RSCLK_H[j] = SCLK_H[(i + j) % 4];
                        reflect RSCLK_H[4 + j] = SCLK_H[4 + (i + j) % 4];
                        if variant [ecp3, machxo2] {
                            reflect RPCLK[j] = PCLK[(i + j) % 4];
                            reflect RPCLK[4 + j] = PCLK[4 + (i + j) % 4];
                            reflect RPCLK_ROW[j] = PCLK_ROW[(i + j) % 4];
                            reflect RPCLK_ROW[4 + j] = PCLK_ROW[4 + (i + j) % 4];
                        }
                    }
                }
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
