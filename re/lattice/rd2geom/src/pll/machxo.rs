use prjcombine_ecp::{
    chip::{PllLoc, PllPad, SpecialIoKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{db::WireSlotIdExt, dir::DirHV};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_machxo(&mut self) {
        for tcid in [tcls::PLL_S, tcls::PLL_N] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let cell = tcrd.cell;
                let bcrd = cell.bel(bslots::PLL[0]);
                let bcrd_int = cell.bel(bslots::BEL_INT);
                let (r, c) = self.rc(tcrd.cell);
                self.name_bel(bcrd, [format!("PLL3_R{r}C{c}")]);

                let pll_loc = if tcid == tcls::PLL_S {
                    PllLoc::new(DirHV::SW, 0)
                } else {
                    PllLoc::new(DirHV::NW, 0)
                };

                let wire_clki = wires::IMUX_PLL_CLKI.cell(0);
                let wire_clkfb = wires::IMUX_PLL_CLKFB.cell(0);
                let wire_clkintfb = wires::OUT_PLL_CLKINTFB.cell(0);
                let wire_clkop = wires::OUT_PLL_CLKOP.cell(0);
                let wire_clkos = wires::OUT_PLL_CLKOS.cell(0);
                let wire_clkok = wires::OUT_PLL_CLKOK.cell(0);
                self.extract_bel(bcrd, cell)
                    .suffix("PLL")
                    .input(bcls::PLL::CLKI, wire_clki)
                    .input(bcls::PLL::CLKFB, wire_clkfb)
                    .output(bcls::PLL::CLKINTFB, wire_clkintfb)
                    .output_extra(bcls::PLL::CLKOP, wire_clkop)
                    .output_extra(bcls::PLL::CLKOS, wire_clkos)
                    .output_extra(bcls::PLL::CLKOK, wire_clkok);

                let clki = self.rc_wire(cell, "CLKI");
                let clki0 = self.rc_wire(cell, "JCLKI0");
                let clki1 = self.rc_wire(cell, "JCLKI1");
                let clki2 = self.rc_wire(cell, "JCLKI2");
                let clki3 = self.rc_wire(cell, "JCLKI3");
                let clki_pll = self.rc_wire(cell, "CLKI_PLL");
                self.add_bel_wire(bcrd, "CLKI_PLL", clki_pll);
                let wire_io =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::PllIn0, pll_loc));
                self.claim_pip(clki3, wire_io);
                self.claim_pip(clki_pll, clki);

                let clkintfb_pll = self.rc_wire(cell, "CLKINTFB_PLL");
                let clkintfb = self.rc_wire(cell, "CLKINTFB");
                let clkfb = self.rc_wire(cell, "CLKFB");
                let clkfb0 = self.rc_wire(cell, "JCLKFB0");
                let clkfb1 = self.rc_wire(cell, "JCLKFB1");
                let clkfb3 = self.rc_wire(cell, "JCLKFB3");
                let clkfb_pll = self.rc_wire(cell, "CLKFB_PLL");
                self.add_bel_wire(bcrd, "CLKINTFB_PLL", clkintfb_pll);
                self.add_bel_wire(bcrd, "CLKFB_PLL", clkfb_pll);
                let wire_io =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::PllFb, pll_loc));
                self.claim_pip(clkfb3, wire_io);
                self.claim_pip(clkintfb, clkintfb_pll);
                self.claim_pip(clkfb_pll, clkfb);

                self.extract_switchbox(bcrd_int)
                    .output(clki, wire_clki)
                    .input_xlat(clki0)
                    .input_xlat(clki1)
                    .input_xlat(clki2)
                    .input(clki3, wires::IO_PLL_CLKI.cell(0))
                    .output(clkfb, wire_clkfb)
                    .input_xlat(clkfb0)
                    .input_xlat(clkfb1)
                    .input(clkintfb, wire_clkintfb)
                    .input(clkfb3, wires::IO_PLL_CLKFB.cell(0));
            }
        }
    }
}
