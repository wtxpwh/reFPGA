use prjcombine_tablegen::target_defs;

target_defs! {
    variant scm;
    variant ecp;
    variant xp;
    variant machxo;
    variant ecp2;
    variant xp2;
    variant ecp3;
    variant machxo2;
    variant ecp4;
    variant ecp5;
    variant crosslink;

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

    if variant [ecp2, xp2, ecp3] {
        // only [1] and [4] are actually used
        // X1_W0 and X1_E0 are aliased to X1_H where applicable; X1_S0/X1_N0 are aliased to X1_V
        wire X1_H[8]: mux;
        wire X1_V[8]: mux;
    }

    if variant [scm, ecp4] {
        // indices 0, 1, 4, 5, 6, 7 are valid
        const NUM_X1 = 8;
    } else if variant [ecp2, xp2, ecp3] {
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

    // Clocking

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
    } else if variant [ecp2, xp2] {
        // per quadrant
        region_slot PCLK;
        wire PCLK[8]: regional PCLK;
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

    if variant [ecp2, xp2, ecp3, machxo2] {
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
        // MachXO only (purpose unclear)
        wire OUT_TI[12]: bel;
    }

    if variant scm {
        // SCM special: EBR interconnect
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

        // SCM special: IO interconnect
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

    tile_slot INT {
        bel_slot INT: routing;
    }

    tile_slot IO {
        bel_slot IO[12]: legacy;
        bel_slot PICTEST[3]: legacy;
    }

    tile_slot BEL {
        bel_slot SLICE[4]: legacy;

        bel_slot IO_INT: routing;
        // patched to CLK for XO2
        bel_slot DQS[2]: legacy;
        bel_slot DQSTEST: legacy;
        bel_slot DQSDLL: legacy;
        bel_slot DQSDLLTEST: legacy;

        bel_slot SERDES: legacy;
        bel_slot SERDES_CENTER: legacy;
        bel_slot SERDES_CORNER: legacy;

        bel_slot MACO: legacy;
        bel_slot MACO_INT: routing;

        bel_slot MIPI: legacy;
        bel_slot CLKTEST_MIPI: legacy;

        bel_slot CIBTEST_SEL: legacy;

        bel_slot EBR_INT: routing;
        bel_slot EBR[4]: legacy;

        bel_slot DSP[2]: legacy;

        bel_slot PLL[2]: legacy;
        bel_slot PLL_SMI: legacy;
        bel_slot PLLREFCS[2]: legacy;
        bel_slot DLL[4]: legacy;
        bel_slot DLL_DCNTL[2]: legacy;
        bel_slot PROMON: legacy;
        bel_slot RNET: legacy;
        bel_slot DDRDLL: legacy;
        bel_slot DTR: legacy;

        // patched to CLK for XO2, ECP4, ECP5
        bel_slot DLLDEL[8]: legacy;
        // patched to CLK for XO2, ECP4, ECP5, SCM
        bel_slot CLKDIV[4]: legacy;
        bel_slot ECLK_ALT_ROOT: legacy;
        bel_slot SPLL: legacy;
        bel_slot SYSBUS: legacy;
        bel_slot START: legacy;
        bel_slot OSC: legacy;
        bel_slot JTAG: legacy;
        bel_slot RDBK: legacy;
        bel_slot GSR: legacy;
        bel_slot TSALL: legacy;
        bel_slot SED: legacy;
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
        bel_slot SPIM: legacy;
        bel_slot SSPI: legacy;
        bel_slot WAKEUP: legacy;
        bel_slot STF: legacy;
        bel_slot AMBOOT: legacy;
        bel_slot PERREG: legacy;
        bel_slot PCNTR: legacy;
        bel_slot EFB: legacy;
        bel_slot ESB: legacy;
        bel_slot I2C: legacy;
        bel_slot NVCMTEST: legacy;
        bel_slot PMU: legacy;
        bel_slot PMUTEST: legacy;
        bel_slot CFGTEST: legacy;
        bel_slot PVTTEST: legacy;
        bel_slot PVTCAL: legacy;

        bel_slot TESTIN: legacy;
        bel_slot TESTOUT: legacy;
        bel_slot DTS: legacy;
    }

    tile_slot BC {
        bel_slot BCPG: legacy;
        bel_slot BCINRD: legacy;
        bel_slot BCLVDSO: legacy;
        bel_slot BCSLEWRATE: legacy;
        bel_slot BCPUSL: legacy;
        bel_slot BREFTEST: legacy;
    }

    tile_slot CLK {
        bel_slot PCSCLKDIV[4]: legacy;
        bel_slot DCC_SW[6]: legacy;
        bel_slot DCC_SE[6]: legacy;
        bel_slot DCC_NW[6]: legacy;
        bel_slot DCC_NE[6]: legacy;
        bel_slot DCS_SW[2]: legacy;
        bel_slot DCS_SE[2]: legacy;
        bel_slot DCS_NW[2]: legacy;
        bel_slot DCS_NE[2]: legacy;
        bel_slot DCC[16]: legacy;
        bel_slot DCM[2]: legacy;
        bel_slot DCS[2]: legacy;
        bel_slot ECLKBRIDGECS[2]: legacy;
        bel_slot BRGECLKSYNC[2]: legacy;
        bel_slot CLKFBBUF[2]: legacy;
        bel_slot CLK_ROOT: legacy;
        bel_slot CLK_EDGE: legacy;
        bel_slot CLKTEST: legacy;
        bel_slot ECLK_ROOT: legacy;
        bel_slot ECLKSYNC[16]: legacy;
        bel_slot CLKTEST_ECLK: legacy;
    }

    tile_slot PCLK_SOURCE {
        bel_slot PCLK_SOURCE_W: legacy;
        bel_slot PCLK_SOURCE_E: legacy;
        bel_slot PCLK_DCC[2]: legacy;
    }

    tile_slot SCLK_SOURCE {
        bel_slot SCLK_SOURCE: legacy;
    }

    tile_slot ECLK_TAP {
        bel_slot ECLK_TAP: legacy;
    }

    tile_slot HSDCLK_SPLITTER {
        bel_slot HSDCLK_ROOT: legacy;
        bel_slot HSDCLK_SPLITTER: legacy;
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
            pass OUT_OFX_W = OUT_OFX;
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

            if variant [ecp2, xp2, ecp3, machxo2] {
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

    if variant [ecp2, xp2, ecp3, machxo2] {
        connector_slot RSCLK_W {
            opposite RSCLK_E;
            connector_class PASS_SW {
                if variant [ecp2, xp2] {
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
