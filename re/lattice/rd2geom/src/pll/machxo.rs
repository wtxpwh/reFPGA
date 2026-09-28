use prjcombine_ecp::defs::{bcls, bslots, tcls, wires};
use prjcombine_interconnect::db::WireSlotIdExt;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_machxo(&mut self) {
        for tcid in [tcls::PLL_S, tcls::PLL_N] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let cell = tcrd.cell;
                let bcrd = cell.bel(bslots::PLL[0]);

                let clkintfb_pll = self.rc_wire(cell, "CLKINTFB_PLL");
                let clkintfb = self.rc_wire(cell, "CLKINTFB");
                self.claim_pip(clkintfb, clkintfb_pll);

                let clki_pll = self.rc_wire(cell, "CLKI_PLL");
                let clki = self.rc_wire(cell, "CLKI");
                self.claim_pip(clki_pll, clki);

                let clkfb_pll = self.rc_wire(cell, "CLKFB_PLL");
                let clkfb = self.rc_wire(cell, "CLKFB");
                self.claim_pip(clkfb_pll, clkfb);

                let wire_clki = wires::IMUX_PLL_CLKI.cell(0);
                let wire_clkfb = wires::IMUX_PLL_CLKFB.cell(0);
                let wire_clkintfb = wires::OUT_PLL_CLKINTFB.cell(0);
                let wire_clkop = wires::OUT_PLL_CLKOP.cell(0);
                let wire_clkos = wires::OUT_PLL_CLKOS.cell(0);
                let wire_clkok = wires::OUT_PLL_CLKOK.cell(0);

                self.extract_bel(bcrd, cell)
                    .name_rc("PLL3")
                    .suffix("PLL")
                    .input(bcls::PLL::CLKI, wire_clki)
                    .input(bcls::PLL::CLKFB, wire_clkfb)
                    .output(bcls::PLL::CLKINTFB, wire_clkintfb)
                    .output_extra(bcls::PLL::CLKOP, wire_clkop)
                    .output_extra(bcls::PLL::CLKOS, wire_clkos)
                    .output_extra(bcls::PLL::CLKOK, wire_clkok)
                    .add_wire("CLKINTFB_PLL", clkintfb_pll)
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
                    .input_io(clki3, wires::IO_PLL_CLKI.cell(0))
                    .output(clkfb, wire_clkfb)
                    .input_xlat(clkfb0)
                    .input_xlat(clkfb1)
                    .input(clkintfb, wire_clkintfb)
                    .input_io(clkfb3, wires::IO_PLL_CLKFB.cell(0));
            }
        }
    }
}
