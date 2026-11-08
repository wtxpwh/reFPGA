use prjcombine_ecp::{
    chip::{Chip, ChipKind, Ecp3Kind},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::DieIdExt,
    db::{CellSlotId, WireSlotIdExt},
    dir::DirH,
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_ecp3(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let is_main = bcrd.row == self.chip.row_clk;
            let cell_dqsdll = match loc.quad.h {
                DirH::W => bcrd.delta(2, 0),
                DirH::E => bcrd.delta(-2, 0),
            };
            let cell_pll = match loc.quad.h {
                DirH::W => bcrd.delta(3, 0),
                DirH::E => bcrd.delta(-3, 0),
            };
            let cell_dll = match loc.quad.h {
                DirH::W => bcrd.delta(13, 0),
                DirH::E => bcrd.delta(-13, 0),
            };

            self.hybrid_out_rc_cell(
                bcrd,
                cell_pll,
                "JCLKOP_PLL",
                wires::OUT_PLL_CLKOP[0].cell(0),
            );
            self.hybrid_out_rc_cell(
                bcrd,
                cell_pll,
                "JCLKOS_PLL",
                wires::OUT_PLL_CLKOS[0].cell(0),
            );
            self.hybrid_out_rc_cell(bcrd, cell_pll, "JCLKOK_PLL", wires::OUT_PLL_CLKOK.cell(0));
            self.hybrid_out_rc_cell(bcrd, cell_pll, "JCLKOK2_PLL", wires::OUT_PLL_CLKOK2.cell(0));

            if is_main {
                let bcrd = bcrd.bel(bslots::DLL[0]);
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCLKOP_DLL",
                    wires::OUT_DLL_CLKOP[0].cell(0),
                );
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCLKOS_DLL",
                    wires::OUT_DLL_CLKOS[0].cell(0),
                );

                let bcrd = bcrd.bel(bslots::CLKDIV[0]);
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCDIV1_CLKDIV",
                    wires::OUT_CLKDIV_CDIV1[0].cell(0),
                );
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCDIV2_CLKDIV",
                    wires::OUT_CLKDIV_CDIV2[0].cell(0),
                );
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCDIV4_CLKDIV",
                    wires::OUT_CLKDIV_CDIV4[0].cell(0),
                );
                self.hybrid_out_rc_cell(
                    bcrd,
                    cell_dll,
                    "JCDIV8_CLKDIV",
                    wires::OUT_CLKDIV_CDIV8[0].cell(0),
                );

                if self.chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A) {
                    let dqsdllclk = self.rc_wire(cell_dqsdll, "DQSDLLCLK");
                    self.bind_wire(dqsdllclk, bcrd.cell.wire(wires::IMUX_DDRDLL_CLK));
                }

                self.hybrid_out_rc_cell(
                    bcrd.bel(bslots::DLLDEL[0]),
                    cell_dll,
                    "JCLKO_DLLDEL",
                    wires::OUT_DLLDEL_CLKO[0].cell(0),
                );

                let eclk1 = self.rc_wire(cell_dll, "DLLECLKI1");
                self.claim_node(eclk1);
                self.bind_wire(eclk1, bcrd.cell.wire(wires::PLL_TO_ECLK_TEST[0]));
                let eclk2 = self.rc_wire(cell_dll, "DLLECLKI2");
                self.claim_node(eclk2);
                self.bind_wire(eclk2, bcrd.cell.wire(wires::PLL_TO_ECLK_TEST[1]));

                let eclk1 = self.rc_wire(cell_dll, "JDLLECLK1");
                self.bind_wire(eclk1, bcrd.cell.wire(wires::PLL_TO_ECLK[0]));
                let eclk2 = self.rc_wire(cell_dll, "JDLLECLK2");
                self.bind_wire(eclk2, bcrd.cell.wire(wires::PLL_TO_ECLK[1]));
            }
        }

        for &tcrd in &self.edev.tile_index[tcls::SERDES] {
            let bcrd = tcrd.bel(bslots::SERDES);
            let cell = bcrd.cell.delta(0, -1);
            self.hybrid_out_rc_cell(
                bcrd,
                cell,
                "JFF_TX_F_CLK_0_PCS",
                wires::OUT_SERDES_FF_TX_F_CLK.cell(0),
            );
            self.hybrid_out_rc_cell(
                bcrd,
                cell,
                "JFF_TX_H_CLK_0_PCS",
                wires::OUT_SERDES_FF_TX_H_CLK.cell(0),
            );
        }
    }

    pub(super) fn process_pll_int_ecp3(&mut self) {
        let chip = self.chip;
        for (loc, bcrd) in chip.plls() {
            let is_main = bcrd.row == chip.row_clk;
            let cell_dqsdll = match loc.quad.h {
                DirH::W => bcrd.delta(2, 0),
                DirH::E => bcrd.delta(-2, 0),
            };
            let cell_pll = match loc.quad.h {
                DirH::W => bcrd.delta(3, 0),
                DirH::E => bcrd.delta(-3, 0),
            };
            let cell_dll = match loc.quad.h {
                DirH::W => bcrd.delta(13, 0),
                DirH::E => bcrd.delta(-13, 0),
            };

            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            // north clock input
            let cslot_n = if is_main {
                CellSlotId::from_idx(20)
            } else {
                CellSlotId::from_idx(13)
            };
            // PLL
            let wire_clki = wires::IMUX_PLL_CLKI[0].cell(0);
            let wire_clkfb = wires::IMUX_PLL_CLKFB[0].cell(0);
            ex.cell(cell_pll)
                .output_bind_rc("CLKI", wire_clki)
                .output_bind_rc("CLKFB", wire_clkfb)
                .input_xlat_rc("JPLLCLKI1")
                .input_xlat_rc("JPLLCLKI2")
                .input_xlat_rc("JPLLCLKI3")
                .input_xlat_rc("JPLLCLKFB0")
                .input_xlat_rc("JPLLCLKFB1")
                .input_bind_rc("PLLCLKINTFB", wires::OUT_PLL_CLKINTFB[0].cell(0))
                .input_xlat_rc("JPLLCLKFB3");
            if is_main {
                // DLL's input
                ex.input_xlat_rc("JPLLCLKI0");
            }
            if ex.ctx.edev[ex.ctx.edev.bel_tile(bcrd)]
                .cells
                .contains_id(cslot_n)
            {
                ex.input_xlat_rc_nudge("JPLLCLKI4", cslot_n);
                ex.input_xlat_rc_nudge("JPLLCLKFB4", cslot_n);
                ex.input_xlat_rc_nudge("JPLLCLKFB5", cslot_n);
            }
            if chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A) {
                ex.input_xlat_rc("JPLLCLKI5").input_xlat_rc("JPLLCLKFB6");
            }
            if is_main {
                ex.cell(cell_dll)
                    .output_bind_rc("DLLCLKI", wires::IMUX_DLL_CLKI[0].cell(0))
                    .output_bind_rc("DLLCLKFB", wires::IMUX_DLL_CLKFB[0].cell(0))
                    .input_xlat_rc("JDLLCLKI0")
                    .input_xlat_rc("JDLLCLKI1")
                    .input_xlat_rc("JDLLCLKI2")
                    .input_xlat_rc("JDLLCLKI3")
                    .input_xlat_rc("JDLLCLKI4")
                    .input_xlat_rc("JDLLCLKFB0")
                    .input_xlat_rc("JDLLCLKFB1")
                    .input_xlat_rc("JDLLCLKFB2")
                    .input_xlat_rc("JDLLCLKFB3")
                    .input_xlat_rc("JDLLCLKFB4")
                    .output_xlat_rc("JDLLECLK1")
                    .output_xlat_rc("JDLLECLK2")
                    .input_xlat_rc("JCIBCLK1")
                    .input_xlat_rc("JCIBCLK2")
                    .input_xlat_rc("JPLLPIO1")
                    .input_xlat_rc("JPLLPIO2")
                    .input_xlat_rc("JPLLCLKOP")
                    .input_xlat_rc("JPLLCLKOS")
                    .input_xlat_rc("DLLCLKOP")
                    .input_xlat_rc("DLLCLKOS")
                    .input_xlat_rc("DLLDEL")
                    .output_bind_rc("CLKDIVCLKI", wires::IMUX_CLKDIV_CLKI[0].cell(0))
                    .input_xlat_rc("JECLK1")
                    .input_xlat_rc("JECLK2")
                    .input_xlat_rc("DLLECLKI1")
                    .input_xlat_rc("DLLECLKI2");
                if chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A) {
                    ex.input_xlat_rc("JDLLCLKI5")
                        .input_xlat_rc("JDLLCLKI6")
                        .input_xlat_rc("JDLLCLKFB5");

                    ex.cell(cell_dqsdll)
                        .output_xlat_rc("DQSDLLCLK")
                        .input_xlat_rc("JCIBCLK")
                        .input_xlat_rc("JECLK");
                }
            }
        }
    }

    pub(super) fn process_pll_ecp3(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = match loc.quad.h {
                DirH::W => bcrd.delta(3, 0),
                DirH::E => bcrd.delta(-3, 0),
            };

            self.extract_bel_cell(bcrd, cell)
                .name_rc("PLL")
                .suffix("PLL")
                .input_xlat_rc(bcls::PLL::CLKI, "CLKI_PLL")
                .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
                .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL")
                .claim_pip("CLKOP", "CLKI")
                .claim_pip("CLKOS", "CLKI")
                .claim_pip("CLKOK", "CLKI");
        }
    }

    pub(super) fn process_dll_ecp3(&mut self) {
        for edge in [DirH::W, DirH::E] {
            let bcrd_dll = Chip::DIE
                .cell(
                    match edge {
                        DirH::W => self.chip.col_w() + 1,
                        DirH::E => self.chip.col_e() - 1,
                    },
                    self.chip.row_clk,
                )
                .bel(bslots::DLL[0]);
            let bcrd_dlldel = bcrd_dll.bel(bslots::DLLDEL[0]);
            let cell = match edge {
                DirH::W => bcrd_dll.cell.delta(13, 0),
                DirH::E => bcrd_dll.cell.delta(-13, 0),
            };

            self.extract_bel_cell(bcrd_dll, cell)
                .name_rc("DLL")
                .suffix("DLL")
                .input_xlat_rc(bcls::DLL::CLKI, "CLKI_DLL")
                .input_xlat_rc(bcls::DLL::CLKFB, "CLKFB_DLL");

            let mut bel = self.extract_bel_cell(bcrd_dlldel, cell);
            bel.name_rc("DLLDEL")
                .suffix("DLLDEL")
                .input_xlat_rc(bcls::DLLDEL::CLKI, "CLKI_DLLDEL")
                .output_xlat_rc(bcls::DLLDEL::BYPASS_OUT, "JBYPASS_DLLDEL");
            for i in 0..6 {
                let wire_dll = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLL"));
                let wire_dlldel = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLLDEL"));
                bel.add_wire(format!("DCNTL{i}"), wire_dlldel);
                bel.ctx.claim_pip(wire_dlldel, wire_dll);
            }
        }
    }

    pub(super) fn process_clkdiv_ecp3(&mut self) {
        for edge in [DirH::W, DirH::E] {
            let bcrd = Chip::DIE
                .cell(
                    match edge {
                        DirH::W => self.chip.col_w() + 1,
                        DirH::E => self.chip.col_e() - 1,
                    },
                    self.chip.row_clk,
                )
                .bel(bslots::CLKDIV[0]);
            let cell = match edge {
                DirH::W => bcrd.cell.delta(13, 0),
                DirH::E => bcrd.cell.delta(-13, 0),
            };
            let (r, c) = self.rc(cell);
            self.extract_bel_cell(bcrd, cell)
                .names([format!("CLKDIV_R{r}C{c}"), format!("CLKDIVTEST_R{r}C{c}")])
                .suffix("CLKDIV")
                .input_xlat_rc(bcls::CLKDIV::CLKI, "CLKI_CLKDIV")
                .input_xlat_rc(bcls::CLKDIV::TEST_CLKI1, "CLKI1_CLKDIVTEST")
                .input_xlat_rc(bcls::CLKDIV::TEST_CLKI2, "CLKI2_CLKDIVTEST")
                .output_xlat_rc(bcls::CLKDIV::TEST_CLKO1, "CLKO1_CLKDIVTEST")
                .output_xlat_rc(bcls::CLKDIV::TEST_CLKO2, "CLKO2_CLKDIVTEST");
        }
    }
}
