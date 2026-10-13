use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{DirH, DirHV, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_xp2(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            self.hybrid_out_rc(bcrd, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOK_PLL", wires::OUT_PLL_CLKOK.cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOK2_PLL", wires::OUT_PLL_CLKOK2.cell(0));
        }
        for edge in DirH::DIRS {
            let bcrd = self.chip.bel_dqsdll_ecp2(edge).bel(bslots::CLKDIV[0]);
            self.hybrid_out_rc(bcrd, "JCDIV1_CLKDIV", wires::OUT_CLKDIV_CDIV1[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCDIV2_CLKDIV", wires::OUT_CLKDIV_CDIV2[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCDIV4_CLKDIV", wires::OUT_CLKDIV_CDIV4[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCDIV8_CLKDIV", wires::OUT_CLKDIV_CDIV8[0].cell(0));
        }
    }

    pub(super) fn process_pll_xp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;

            self.extract_switchbox(cell.bel(bslots::BEL_INT))
                .output_bind_rc("CLKI", wires::IMUX_PLL_CLKI[0].cell(0))
                .input_xlat_rc("JPLLCLKI0")
                .input_xlat_rc("JPLLCLKI1")
                .input_xlat_rc("JPLLCLKI2")
                .input_xlat_rc("JPLLCLKI3")
                .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                .input_xlat_rc("JPLLCLKFB0")
                .input_xlat_rc("JPLLCLKFB1")
                .input_bind_rc("PLLCLKINTFB", wires::OUT_PLL_CLKINTFB.cell(0))
                .input_xlat_rc("JPLLCLKFB3");

            let name = match loc.quad {
                DirHV::SW => "LLPLL",
                DirHV::SE => "LRPLL",
                DirHV::NW => "ULPLL",
                DirHV::NE => "URPLL",
            };
            self.extract_bel(cell.bel(bslots::PLL[0]))
                .name(name)
                .suffix("PLL")
                .input_xlat_rc(bcls::PLL::CLKI, "CLKI_PLL")
                .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
                .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL");
        }
    }

    pub(super) fn process_clkdiv_xp2(&mut self) {
        for edge in DirH::DIRS {
            let bcrd = self.chip.bel_dqsdll_ecp2(edge).bel(bslots::CLKDIV[0]);
            let cell = bcrd.cell;

            let wire_clki = wires::IMUX_CLKDIV_CLKI[0].cell(0);

            {
                let mut ex = self.extract_switchbox(cell.bel(bslots::BEL_INT));
                ex.output_bind_rc("CLKDIVCLKI", wire_clki)
                    .input_xlat_rc("JFRC0")
                    .input_xlat_rc("JFRC1");

                for (v, ci) in [(DirV::S, 1), (DirV::N, 2)] {
                    let wire_clkop = wires::OUT_PLL_CLKOP[0].cell(ci);
                    if ex.ctx.chip.try_bel_pll(edge & v, 0).is_some() {
                        let idx = match edge & v {
                            DirHV::SW => 1,
                            DirHV::SE => 0,
                            DirHV::NW => 0,
                            DirHV::NE => 1,
                        };
                        ex.input_xlat_rc(format!("JCLKOP{idx}"));
                    } else {
                        ex.edge(wire_clki, wire_clkop);
                    }
                }
            }

            self.extract_bel(bcrd)
                .name(match edge {
                    DirH::W => "LCLKDIV",
                    DirH::E => "RCLKDIV",
                })
                .suffix("CLKDIV")
                .input_xlat_rc(bcls::CLKDIV::CLKI, "CLKI_CLKDIV");
        }
    }
}
