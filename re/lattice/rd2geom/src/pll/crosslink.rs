use prjcombine_ecp::{
    chip::Chip,
    defs::{bcls, bslots, wires},
};
use prjcombine_interconnect::{db::WireSlotIdExt, dir::DirHV, grid::DieIdExt};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_crosslink(&mut self) {
        let cell_tile = Chip::DIE.cell(self.chip.col_clk, self.chip.row_s());
        for idx in 0..4 {
            let cell = cell_tile.delta(-2 + idx, 0);
            self.bind_wire(self.rc_io_wire(cell, "JINCK"), cell.wire(wires::INCK[0]));
        }

        let bcrd = self.chip.bel_pll(DirHV::SE, 0);
        self.hybrid_out_rc(bcrd, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[0].cell(0));
        self.hybrid_out_rc(bcrd, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[0].cell(0));
        self.hybrid_out_rc(bcrd, "JCLKOS2_PLL", wires::OUT_PLL_CLKOS2[0].cell(0));
        self.hybrid_out_rc(bcrd, "JCLKOS3_PLL", wires::OUT_PLL_CLKOS3[0].cell(0));
    }

    pub(super) fn process_pll_crosslink(&mut self) {
        let bcrd = self.chip.bel_pll(DirHV::SE, 0);

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            ex.output_bind_rc("REFCLK0", wires::IMUX_PLL_CLKI[0].cell(0))
                .output_bind_rc("REFCLK1", wires::IMUX_PLL_CLKI[1].cell(0))
                .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                .input_xlat_rc("JREFCLK0_1")
                .input_xlat_rc("JREFCLK0_2")
                .input_xlat_rc("JREFCLK0_3")
                .input_xlat_rc("JREFCLK0_4")
                .input_xlat_rc("JREFCLK0_5")
                .input_xlat_rc("JREFCLK1_1")
                .input_xlat_rc("JREFCLK1_2")
                .input_xlat_rc("JREFCLK1_3")
                .input_xlat_rc("JREFCLK1_4")
                .input_xlat_rc("JCLKFB1")
                .input_xlat_rc("JCLKFB2")
                .input_bind_rc("CLKINTFB", wires::OUT_PLL_CLKINTFB[0].cell(0));
        }

        self.extract_bel(bcrd)
            .names(["PLL", "PLLREFCS"])
            .suffix("PLL")
            .suffix("PLLREFCS")
            .input_xlat_rc(bcls::PLL::CLK[0], "CLK0_PLLREFCS")
            .input_xlat_rc(bcls::PLL::CLK[1], "CLK1_PLLREFCS")
            .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
            .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL")
            .add_wire_rc("PLLCSOUT", "PLLCSOUT_PLLREFCS")
            .add_wire_rc("CLKI", "CLKI_PLL")
            .claim_pip("PLLCSOUT", "CLK0")
            .claim_pip("PLLCSOUT", "CLK1")
            .claim_pip("CLKI", "PLLCSOUT")
            .claim_pip("CLKOP", "CLKI")
            .claim_pip("CLKOS", "CLKI")
            .claim_pip("CLKOS2", "CLKI")
            .claim_pip("CLKOS3", "CLKI")
            .claim_pip("REFCLK", "CLKI");
    }
}
