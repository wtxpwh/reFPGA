use prjcombine_ecp::{
    chip::{SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotId, WireSlotIdExt},
    dir::{Dir, DirHV, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_ecp4(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell_tile = bcrd.cell;
            let idx = usize::from(loc.idx);
            let cell = match loc.quad.v {
                DirV::S => cell_tile.delta(0, 2 - idx as i32),
                DirV::N => cell_tile.delta(0, -1 - idx as i32),
            };
            let ci = match loc.quad.v {
                DirV::S => 1,
                DirV::N => 3,
            };
            self.hybrid_out_rc_cell(bcrd, cell, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[idx].cell(ci));
            self.hybrid_out_rc_cell(bcrd, cell, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[idx].cell(ci));
            self.hybrid_out_rc_cell(
                bcrd,
                cell,
                "JCLKOS2_PLL",
                wires::OUT_PLL_CLKOS2[idx].cell(ci),
            );
            self.hybrid_out_rc_cell(
                bcrd,
                cell,
                "JCLKOS3_PLL",
                wires::OUT_PLL_CLKOS3[idx].cell(ci),
            );
        }
        {
            for (edge, num) in [(Dir::N, 8), (Dir::W, 4), (Dir::E, 4)] {
                for idx in 0..num {
                    if !self
                        .chip
                        .special_io
                        .contains_key(&SpecialIoKey::Clock(edge, idx as u8))
                    {
                        continue;
                    }
                    let cell = match edge {
                        Dir::N => self.chip.clk_emid(edge).delta(-4 + idx, 0),
                        Dir::W => self.chip.clk_emid(edge).delta(0, -2 + idx),
                        Dir::E => self.chip.clk_emid(edge).delta(0, -2 + (idx ^ 1)),
                        Dir::S => unreachable!(),
                    };

                    let wire = self.rc_io_wire(cell, "JINCK");
                    self.bind_wire(wire, cell.wire(wires::INCK[0]));
                }
            }
        }
        {
            for i in 0..4 {
                self.bind_wire(
                    self.rc_wire(self.chip.clk_root().delta(-1, 0), &format!("JBRGECLK{i}")),
                    self.chip.clk_root().wire(wires::BRGECLK[i]),
                );
            }
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::Config];

            self.hybrid_out_rc(
                cell.bel(bslots::OSC),
                "JOSC_OSC",
                wires::OUT_OSC_OSC.cell(0),
            );
        }
        {
            let num_quads = if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesSingle)
            {
                1
            } else if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesDouble)
            {
                2
            } else if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesTriple)
            {
                3
            } else {
                unreachable!()
            };
            let bcrd = self
                .chip
                .special_loc
                .get(&SpecialLocKey::SerdesSingle)
                .or(self.chip.special_loc.get(&SpecialLocKey::SerdesDouble))
                .or(self.chip.special_loc.get(&SpecialLocKey::SerdesTriple))
                .unwrap()
                .bel(bslots::SERDES);
            let cell = bcrd.with_col(self.chip.col_w());
            let wire = self.rc_io_sn_wire(cell, "JQ0P_CORECLK_ASB");
            self.bind_wire(wire, bcrd.wire(wires::OUT_SERDES_Q0P_CORECLK));
            for i in 0..num_quads {
                let w_tx = [
                    wires::OUT_SERDES_Q0_FOPCLKA,
                    wires::OUT_SERDES_Q1_FOPCLKA,
                    wires::OUT_SERDES_Q2_FOPCLKA,
                    wires::OUT_SERDES_Q3_FOPCLKA,
                ][i];
                let w_rx = [
                    wires::OUT_SERDES_Q0_FOPCLKB,
                    wires::OUT_SERDES_Q1_FOPCLKB,
                    wires::OUT_SERDES_Q2_FOPCLKB,
                    wires::OUT_SERDES_Q3_FOPCLKB,
                ][i];
                for j in 0..4 {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{i}CH{j}_FOPCLKA_ASB"));
                    self.bind_wire(wire, bcrd.wire(w_tx[j]));
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{i}CH{j}_FOPCLKB_ASB"));
                    self.bind_wire(wire, bcrd.wire(w_rx[j]));
                }
            }
        }
    }

    pub(super) fn process_pll_ecp4(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let hv = loc.quad;
            let cell_tile = bcrd.cell;
            let idx = usize::from(loc.idx);
            let corner = match hv {
                DirHV::SW => "BL",
                DirHV::SE => "BR",
                DirHV::NW => "TL",
                DirHV::NE => "TR",
            };
            let cell = match hv.v {
                DirV::S => cell_tile.delta(0, 2 - idx as i32),
                DirV::N => cell_tile.delta(0, -1 - idx as i32),
            };

            let ci_base = match loc.quad.v {
                DirV::S => 1,
                DirV::N => 3,
            };
            {
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::BEL_INT));
                ex.cell(cell)
                    .output_bind_rc("REFCLK0", wires::IMUX_PLL_CLKI[idx * 2].cell(ci_base))
                    .output_bind_rc("REFCLK1", wires::IMUX_PLL_CLKI[idx * 2 + 1].cell(ci_base))
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[idx].cell(ci_base))
                    .input_xlat_rc("JREFCLK0_0")
                    .input_xlat_rc("JREFCLK1_0")
                    .input_xlat_rc("JREFCLK0_1")
                    .input_xlat_rc("JREFCLK1_1")
                    .input_xlat_rc("JREFCLK0_2")
                    .input_xlat_rc("JREFCLK1_2")
                    .input_xlat_rc("JREFCLK0_3")
                    .input_xlat_rc("JREFCLK1_3")
                    .input_xlat_rc("JREFCLK0_4")
                    .input_xlat_rc("JREFCLK1_4")
                    .input_xlat_rc("JREFCLK0_5")
                    .input_xlat_rc("JREFCLK1_5")
                    .input_xlat_rc("JCLKFB0")
                    .input_xlat_rc("JCLKFB1")
                    .input_xlat_rc("JCLKFB2")
                    .input_xlat_rc("JCLKFB3")
                    .input_xlat_rc("JCLKFB4")
                    .input_xlat_rc("JCLKFB5")
                    .input_xlat_rc("JCLKFB6")
                    .input_bind_rc("CLKINTFB", wires::OUT_PLL_CLKINTFB[idx].cell(ci_base));
                if hv.v == DirV::N {
                    ex.input_xlat_rc_nudge("JREFCLK0_6", CellSlotId::from_idx(7))
                        .input_xlat_rc_nudge("JREFCLK1_6", CellSlotId::from_idx(8))
                        .input_xlat_rc_nudge("JREFCLK0_7", CellSlotId::from_idx(7))
                        .input_xlat_rc_nudge("JREFCLK1_7", CellSlotId::from_idx(8))
                        .input_xlat_rc_nudge("JCLKFB7", CellSlotId::from_idx(7))
                        .input_xlat_rc_nudge("JCLKFB8", CellSlotId::from_idx(7))
                        .input_xlat_rc_nudge("JCLKFB9", CellSlotId::from_idx(7))
                        .input_xlat_rc_nudge("JCLKFB10", CellSlotId::from_idx(7));
                }
                if hv == DirHV::SW {
                    ex.input_xlat_rc("JREFCLK0_6")
                        .input_xlat_rc("JREFCLK1_6")
                        .input_xlat_rc("JREFCLK0_7")
                        .input_xlat_rc("JREFCLK1_7");
                }
            }

            self.extract_bel_cell(bcrd, cell)
                .names([
                    format!("PLL_{corner}{idx}"),
                    format!("PLLREFCS_{corner}{idx}"),
                ])
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
