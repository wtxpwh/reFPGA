use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_interconnect::db::WireSlotIdExt;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_ecp(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            self.hybrid_out_rc(bcrd, "JCLKOP", wires::OUT_PLL_CLKOP[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS_PLL3", wires::OUT_PLL_CLKOS[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOK_PLL3", wires::OUT_PLL_CLKOK.cell(0));
        }
    }

    pub(super) fn process_pll_ecp(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            self.extract_switchbox(bcrd.bel(bslots::BEL_INT))
                .output_bind_rc("CLKI", wires::IMUX_PLL_CLKI[0].cell(0))
                .input_xlat_rc("JCLKI0")
                .input_xlat_rc("JCLKI1")
                .input_xlat_rc("JCLKI2")
                .input_xlat_rc("JCLKI3")
                .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                .input_xlat_rc("JCLKFB0")
                .input_xlat_rc("JCLKFB1")
                .input_xlat_rc("JCLKOP")
                .input_xlat_rc("JCLKFB3");

            self.extract_bel(bcrd)
                .name_rc("PLL3")
                .suffix("PLL3")
                .input_xlat_rc(bcls::PLL::CLKI, "CLKI_PLL3")
                .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL3")
                .output_xlat_rc(bcls::PLL::CLKOP, "CLKOP_PLL3")
                .add_wire_rc("JCLKOP", "JCLKOP");
        }
    }
}
