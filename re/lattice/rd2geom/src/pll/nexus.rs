use prjcombine_ecp::defs::wires;
use prjcombine_interconnect::db::WireSlotIdExt;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_nexus(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            for (pin, wire) in [
                ("CLKOP", wires::OUT_PLL_CLKOP[0]),
                ("CLKOS", wires::OUT_PLL_CLKOS[0]),
                ("CLKOS2", wires::OUT_PLL_CLKOS2[0]),
                ("CLKOS3", wires::OUT_PLL_CLKOS3[0]),
                ("CLKOS4", wires::OUT_PLL_CLKOS4[0]),
                ("CLKOS5", wires::OUT_PLL_CLKOS5[0]),
            ] {
                self.hybrid_out_rc(bcrd, format!("J{pin}_PLL_CORE_PLL_LMMI"), wire.cell(0));
            }
        }
    }

    pub(super) fn process_pll_nexus(&mut self) {
        // TODO
    }
}
