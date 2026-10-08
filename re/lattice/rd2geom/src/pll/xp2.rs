use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirHV, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_xp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;

            let clkintfb_pll = self.rc_wire(cell, "CLKINTFB_PLL");
            let clkintfb = self.rc_wire(cell, "PLLCLKINTFB");
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
            let wire_clkok2 = wires::OUT_PLL_CLKOK2.cell(0);

            let name = match loc.quad {
                DirHV::SW => "LLPLL",
                DirHV::SE => "LRPLL",
                DirHV::NW => "ULPLL",
                DirHV::NE => "URPLL",
            };
            self.extract_bel(cell.bel(bslots::PLL[0]), cell)
                .name(name)
                .suffix("PLL")
                .input(bcls::PLL::CLKI, wire_clki)
                .input(bcls::PLL::CLKFB, wire_clkfb)
                .output(bcls::PLL::CLKINTFB, wire_clkintfb)
                .output_extra(bcls::PLL::CLKOP, wire_clkop)
                .output_extra(bcls::PLL::CLKOS, wire_clkos)
                .output_extra(bcls::PLL::CLKOK, wire_clkok)
                .output_extra(bcls::PLL::CLKOK2, wire_clkok2)
                .add_wire("CLKINTFB_PLL", clkintfb_pll)
                .add_wire("CLKI_PLL", clki_pll)
                .add_wire("CLKFB_PLL", clkfb_pll);

            let clki0 = self.rc_wire(cell, "JPLLCLKI0");
            let clki1 = self.rc_wire(cell, "JPLLCLKI1");
            let clki2 = self.rc_wire(cell, "JPLLCLKI2");
            let clki3 = self.rc_wire(cell, "JPLLCLKI3");
            let clkfb0 = self.rc_wire(cell, "JPLLCLKFB0");
            let clkfb1 = self.rc_wire(cell, "JPLLCLKFB1");
            let clkfb3 = self.rc_wire(cell, "JPLLCLKFB3");

            let bel_eclk_root = self.chip.bel_eclk_root(Dir::H(loc.quad.h));
            let eclk0_in = self.naming.bel_wire(bel_eclk_root, "ECLK0_IN");
            self.claim_pip(clkfb3, eclk0_in);

            self.extract_switchbox(cell.bel(bslots::BEL_INT))
                .output(clki, wire_clki)
                .input_xlat(clki0)
                .input_xlat(clki1)
                .input_xlat(clki2)
                .input_io(clki3, wires::OUT_IO_PADDI[0].cell(2))
                .output(clkfb, wire_clkfb)
                .input_xlat(clkfb0)
                .input_io(clkfb1, wires::OUT_IO_PADDI[0].cell(3))
                .input(clkintfb, wire_clkintfb)
                .input(clkfb3, wires::ECLK[0].cell(0));
        }
    }

    pub(super) fn process_clkdiv_xp2(&mut self) {
        for edge in [DirH::W, DirH::E] {
            let bcrd = self.chip.bel_dqsdll_ecp2(edge).bel(bslots::CLKDIV[0]);
            let cell = bcrd.cell;

            let clki = self.rc_wire(cell, "CLKI_CLKDIV");
            let clki_in = self.rc_wire(cell, "CLKDIVCLKI");
            self.claim_pip(clki, clki_in);

            let wire_clki = wires::IMUX_CLKDIV_CLKI.cell(0);
            let wire_cdiv1 = wires::OUT_CLKDIV_CDIV1.cell(0);
            let wire_cdiv2 = wires::OUT_CLKDIV_CDIV2.cell(0);
            let wire_cdiv4 = wires::OUT_CLKDIV_CDIV4.cell(0);
            let wire_cdiv8 = wires::OUT_CLKDIV_CDIV8.cell(0);

            self.extract_bel(bcrd, cell)
                .name(match edge {
                    DirH::W => "LCLKDIV",
                    DirH::E => "RCLKDIV",
                })
                .suffix("CLKDIV")
                .input(bcls::CLKDIV::CLKI, wire_clki)
                .add_wire("CLKI", clki)
                .output_extra(bcls::CLKDIV::CDIV1, wire_cdiv1)
                .output_extra(bcls::CLKDIV::CDIV2, wire_cdiv2)
                .output_extra(bcls::CLKDIV::CDIV4, wire_cdiv4)
                .output_extra(bcls::CLKDIV::CDIV8, wire_cdiv8);

            let bel_eclk_root = self.chip.bel_eclk_root(Dir::H(edge));
            let eclk0_in = self.naming.bel_wire(bel_eclk_root, "ECLK0_IN");
            let eclk1_in = self.naming.bel_wire(bel_eclk_root, "ECLK1_IN");
            let eclk0 = self.rc_wire(cell, "JFRC0");
            let eclk1 = self.rc_wire(cell, "JFRC1");
            self.claim_pip(eclk0, eclk0_in);
            self.claim_pip(eclk1, eclk1_in);

            let mut ex = self.extract_switchbox(cell.bel(bslots::BEL_INT));
            ex.output(clki_in, wire_clki)
                .input(eclk0, wires::ECLK[0].cell(0))
                .input(eclk1, wires::ECLK[1].cell(0));

            for (v, ci) in [(DirV::S, 1), (DirV::N, 2)] {
                let wire_clkop = wires::OUT_PLL_CLKOP.cell(ci);
                if let Some(bel_pll) = ex.ctx.chip.try_bel_pll(edge & v, 0) {
                    let cell_pll = bel_pll.cell;
                    let clkop_in = ex.ctx.rc_wire(cell_pll, "JCLKOP_PLL");
                    let idx = match edge & v {
                        DirHV::SW => 1,
                        DirHV::SE => 0,
                        DirHV::NW => 0,
                        DirHV::NE => 1,
                    };
                    let clkop = ex.ctx.rc_wire(cell, &format!("JCLKOP{idx}"));
                    ex.ctx.claim_pip(clkop, clkop_in);
                    ex.input(clkop, wire_clkop);
                } else {
                    ex.edge(wire_clki, wire_clkop);
                }
            }
        }
    }
}
