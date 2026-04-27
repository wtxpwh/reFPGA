use prjcombine_tablegen::target_defs;

target_defs! {
    region_slot PCLK[4];
    region_slot SCLK[4];
    region_slot VSDCLK;

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
    }
    connector_slot E {
        opposite W;
    }
    connector_slot S {
        opposite N;
    }
    connector_slot N {
        opposite S;
    }

    connector_slot SW {
        opposite SE;
    }
    connector_slot SE {
        opposite SW;
    }

    connector_slot EBR_W {
        opposite EBR_E;
    }
    connector_slot EBR_E {
        opposite EBR_W;
    }
    connector_slot IO_W {
        opposite IO_E;
    }
    connector_slot IO_E {
        opposite IO_W;
    }
}
