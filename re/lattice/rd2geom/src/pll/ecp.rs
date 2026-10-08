use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_interconnect::db::WireSlotIdExt;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_ecp(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;

            let clkop_pll = self.rc_wire(cell, "CLKOP_PLL3");
            let clkop = self.rc_wire(cell, "JCLKOP");
            self.claim_pip(clkop, clkop_pll);

            let clki_pll = self.rc_wire(cell, "CLKI_PLL3");
            let clki = self.rc_wire(cell, "CLKI");
            self.claim_pip(clki_pll, clki);

            let clkfb_pll = self.rc_wire(cell, "CLKFB_PLL3");
            let clkfb = self.rc_wire(cell, "CLKFB");
            self.claim_pip(clkfb_pll, clkfb);

            let wire_clki = wires::IMUX_PLL_CLKI.cell(0);
            let wire_clkfb = wires::IMUX_PLL_CLKFB.cell(0);
            let wire_clkop = wires::OUT_PLL_CLKOP.cell(0);
            let wire_clkos = wires::OUT_PLL_CLKOS.cell(0);
            let wire_clkok = wires::OUT_PLL_CLKOK.cell(0);
            self.extract_bel(bcrd, cell)
                .name_rc("PLL3")
                .suffix("PLL3")
                .add_wire("CLKOP", clkop_pll)
                .input(bcls::PLL::CLKI, wire_clki)
                .input(bcls::PLL::CLKFB, wire_clkfb)
                .xlat_output(bcls::PLL::CLKOP, clkop)
                .output_extra(bcls::PLL::CLKOP, wire_clkop)
                .output_extra(bcls::PLL::CLKOS, wire_clkos)
                .output_extra(bcls::PLL::CLKOK, wire_clkok)
                .add_wire("CLKI_PLL", clki_pll)
                .add_wire("CLKFB_PLL", clkfb_pll);

            let clki0 = self.rc_wire(cell, "JCLKI0");
            let clki1 = self.rc_wire(cell, "JCLKI1");
            let clki2 = self.rc_wire(cell, "JCLKI2");
            let clki3 = self.rc_wire(cell, "JCLKI3");
            let clkfb0 = self.rc_wire(cell, "JCLKFB0");
            let clkfb1 = self.rc_wire(cell, "JCLKFB1");
            let clkfb3 = self.rc_wire(cell, "JCLKFB3");

            self.extract_switchbox(cell.bel(bslots::BEL_INT))
                .output(clki, wire_clki)
                .input_xlat(clki0)
                .input_xlat(clki1)
                .input_xlat(clki2)
                .input_io(clki3, wires::OUT_IO_PADDI[0].cell(1))
                .output(clkfb, wire_clkfb)
                .input_xlat(clkfb0)
                .input_xlat(clkfb1)
                .input(clkop, wires::OUT_PLL_CLKOP.cell(0))
                .input_io(clkfb3, wires::OUT_IO_PADDI[0].cell(2));
        }
    }
}
