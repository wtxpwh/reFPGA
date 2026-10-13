use prjcombine_ecp::defs::{bcls, bslots, tcls, wires};
use prjcombine_interconnect::db::WireSlotIdExt;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_machxo(&mut self) {
        for tcid in [tcls::PLL_S, tcls::PLL_N] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::PLL[0]);

                self.hybrid_out_rc(bcrd, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[0].cell(0));
                self.hybrid_out_rc(bcrd, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[0].cell(0));
                self.hybrid_out_rc(bcrd, "JCLKOK_PLL", wires::OUT_PLL_CLKOK.cell(0));

                self.extract_switchbox(tcrd.bel(bslots::BEL_INT))
                    .output_bind_rc("CLKI", wires::IMUX_PLL_CLKI[0].cell(0))
                    .input_xlat_rc("JCLKI0")
                    .input_xlat_rc("JCLKI1")
                    .input_xlat_rc("JCLKI2")
                    .input_io_rc("JCLKI3", wires::IO_PLL_CLKI.cell(0))
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                    .input_xlat_rc("JCLKFB0")
                    .input_xlat_rc("JCLKFB1")
                    .input_bind_rc("CLKINTFB", wires::OUT_PLL_CLKINTFB[0].cell(0))
                    .input_io_rc("JCLKFB3", wires::IO_PLL_CLKFB.cell(0));

                self.extract_bel(bcrd)
                    .name_rc("PLL3")
                    .suffix("PLL")
                    .input_xlat_rc(bcls::PLL::CLKI, "CLKI_PLL")
                    .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
                    .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL");
            }
        }
    }
}
