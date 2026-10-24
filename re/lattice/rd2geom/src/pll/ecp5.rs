use prjcombine_ecp::defs::{bcls, bslots, tcls, wires};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_ecp5(&mut self) {
        if self.naming.strings.get("JCH0_FF_TX_D_0_DCU").is_none() {
            self.skip_serdes = true;
        }
        if !self.skip_serdes {
            for &tcrd in &self.edev.tile_index[tcls::SERDES] {
                for i in 0..2 {
                    let wname = self.rc_io_wire(tcrd.cell, &format!("JCH{i}_FF_TX_PCLK_DCU"));
                    self.bind_wire(wname, tcrd.wire(wires::OUT_SERDES_FF_TX_PCLK[i]));
                    let wname = self.rc_io_wire(tcrd.cell, &format!("JCH{i}_FF_RX_PCLK_DCU"));
                    self.bind_wire(wname, tcrd.wire(wires::OUT_SERDES_FF_RX_PCLK[i]));
                }
                let wname = self.rc_io_wire(tcrd.cell, "JREFCLKO_EXTREF");
                self.claim_node(wname);
                self.bind_wire(wname, tcrd.wire(wires::OUT_SERDES_REFCLKO));
            }
        }
        {
            for i in 0..2 {
                self.bind_wire(
                    self.rc_wire(self.chip.clk_root().delta(-1, 0), &format!("JBRGECLK{i}")),
                    self.chip.clk_root().wire(wires::BRGECLK[i]),
                );
            }
        }
        {
            for edge in [Dir::N, Dir::W, Dir::E] {
                for idx in 0..4 {
                    let cell = match edge {
                        Dir::N => self.chip.clk_emid(edge).delta(-2 + idx, 0),
                        Dir::W => self.chip.clk_emid(edge).delta(0, -2 + (idx ^ 1)),
                        Dir::E => self.chip.clk_emid(edge).delta(0, -2 + (idx ^ 1)),
                        Dir::S => unreachable!(),
                    };
                    let wire = self.rc_io_wire(cell, "JINCK");
                    self.bind_wire(wire, cell.wire(wires::INCK[0]));
                }
            }
        }
        for (_loc, bcrd) in self.chip.plls() {
            self.hybrid_out_rc(bcrd, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS2_PLL", wires::OUT_PLL_CLKOS2[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS3_PLL", wires::OUT_PLL_CLKOS3[0].cell(0));
        }
    }

    pub(super) fn process_pll_ecp5(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let hv = loc.quad;
            let corner = match hv {
                DirHV::SW => "BL",
                DirHV::SE => "BR",
                DirHV::NW => "TL",
                DirHV::NE => "TR",
            };

            {
                let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                ex.output_bind_rc("REFCLK0", wires::IMUX_PLL_CLKI[0].cell(0))
                    .output_bind_rc("REFCLK1", wires::IMUX_PLL_CLKI[1].cell(0))
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                    .input_xlat_rc("JREFCLK0_0")
                    .input_xlat_rc("JREFCLK1_0")
                    .input_xlat_rc("JREFCLK0_1")
                    .input_xlat_rc("JREFCLK1_1")
                    .input_xlat_rc("JREFCLK0_2")
                    .input_xlat_rc("JREFCLK1_2")
                    .input_xlat_rc("JREFCLK0_3")
                    .input_xlat_rc("JREFCLK1_3")
                    .input_xlat_rc("JCLKFB1")
                    .input_xlat_rc("JCLKFB2")
                    .input_xlat_rc("JCLKFB3")
                    .input_bind_rc("CLKINTFB", wires::OUT_PLL_CLKINTFB[0].cell(0));
                if hv.v == DirV::N {
                    ex.input_xlat_rc("JREFCLK0_4")
                        .input_xlat_rc("JREFCLK1_4")
                        .input_xlat_rc("JREFCLK0_5")
                        .input_xlat_rc("JREFCLK1_5")
                        .input_xlat_rc("JREFCLK0_6")
                        .input_xlat_rc("JREFCLK1_6");
                }
            }

            self.extract_bel(bcrd)
                .names([format!("PLL_{corner}0"), format!("PLLREFCS_{corner}0")])
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
}
