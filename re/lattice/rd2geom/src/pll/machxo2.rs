use prjcombine_ecp::{
    chip::{ChipKind, MachXo2Kind, RowKind, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_machxo2(&mut self) {
        self.hybrid_out_rc(
            self.chip.special_loc[&SpecialLocKey::Config].bel(bslots::OSC),
            "JOSC_OSC",
            wires::OUT_OSC_OSC.cell(0),
        );

        for quad in [DirHV::NW, DirHV::NE] {
            let Some(bcrd) = self.chip.try_bel_pll(quad, 0) else {
                continue;
            };
            self.hybrid_out_rc(bcrd, "JCLKOP_PLL", wires::OUT_PLL_CLKOP[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS_PLL", wires::OUT_PLL_CLKOS[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS2_PLL", wires::OUT_PLL_CLKOS2[0].cell(0));
            self.hybrid_out_rc(bcrd, "JCLKOS3_PLL", wires::OUT_PLL_CLKOS3[0].cell(0));
        }
        if self.chip.rows[self.chip.row_clk].kind == RowKind::Ebr {
            for idx in 0..2 {
                let wire = self.rc_wire(
                    self.chip.clk_root().delta(-1, 0),
                    &format!("JECSOUT{idx}_ECLKBRIDGECS"),
                );
                self.claim_node(wire);
                self.bind_wire(wire, self.chip.clk_root().wire(wires::BRGECLK[idx]));
            }

            for (edge, idx) in [
                (Dir::W, 0),
                (Dir::W, 1),
                (Dir::W, 2),
                (Dir::E, 0),
                (Dir::S, 0),
                (Dir::S, 1),
                (Dir::N, 0),
                (Dir::N, 1),
            ] {
                let cell = match edge {
                    Dir::H(edge) => self.chip.clk_emid(edge),
                    Dir::V(edge) => self.chip.clk_emid(edge).step(Dir::W, 1),
                };
                self.bind_wire(
                    self.rc_io_wire(cell, &format!("JINCK{idx}")),
                    cell.wire(wires::INCK[idx]),
                );
            }
        }
    }

    pub(super) fn process_pll_machxo2(&mut self) {
        for (lr, idx, quad) in [('L', 0, DirHV::NW), ('R', 1, DirHV::NE)] {
            let Some(bcrd) = self.chip.try_bel_pll(quad, 0) else {
                continue;
            };

            {
                let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                ex.output_bind_rc("REFCLK0", wires::IMUX_PLL_CLKI[0].cell(0))
                    .output_bind_rc("REFCLK1", wires::IMUX_PLL_CLKI[1].cell(0))
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB[0].cell(0))
                    .input_xlat_rc("JREFCLK0")
                    .input_xlat_rc("JREFCLK1_0")
                    .input_xlat_rc("JREFCLK1_1")
                    .input_xlat_rc("JREFCLK2_0")
                    .input_xlat_rc("JREFCLK2_1")
                    .input_io_rc("JREFCLK3", wires::IO_PLL_IN.cell(0))
                    .input_io_rc("JREFCLK4", wires::IO_GCLK_S[1].cell(0))
                    .input_io_rc("JREFCLK5", wires::IO_GCLK_N[1].cell(0))
                    .input_io_rc("JREFCLK6", wires::IO_GCLK_S[0].cell(0))
                    .input_io_rc("JREFCLK7", wires::IO_GCLK_N[0].cell(0))
                    .input_io_rc("JCLKFB0", wires::IO_PLL_FB.cell(0))
                    .input_xlat_rc("JCLKFB1")
                    .input_xlat_rc("JCLKFB2")
                    .input_bind_rc("CLKINTFB", wires::OUT_PLL_CLKINTFB[0].cell(0))
                    .input_xlat_rc("JCLKFB4");
                for i in 0..2 {
                    let bcrd_eclksync = ex.ctx.chip.bel_eclksync(Dir::N, i);
                    let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
                    ex.input_xlat(wire_eclk);
                }
            }

            let mut bel = self.extract_bel(bcrd);
            bel.names([format!("{lr}PLL"), format!("{lr}PLLREFCS")])
                .suffix("PLL")
                .suffix("PLLREFCS")
                .input_xlat_rc(bcls::PLL::CLK[0], "CLK0_PLLREFCS")
                .input_xlat_rc(bcls::PLL::CLK[1], "CLK1_PLLREFCS")
                .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
                .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL")
                .add_wire_rc("PLLCSOUT", "PLLCSOUT_PLLREFCS")
                .claim_pip("PLLCSOUT", "CLK0")
                .add_wire_rc("CLKI", "CLKI_PLL")
                .claim_pip("CLKI", "PLLCSOUT")
                .claim_pip("CLKOP", "CLKI")
                .claim_pip("CLKOS", "CLKI")
                .claim_pip("CLKOS2", "CLKI")
                .claim_pip("CLKOS3", "CLKI")
                .claim_pip("REFCLK", "CLKI");

            let cell_config = bel.ctx.chip.special_loc[&SpecialLocKey::Config];
            for (pin, pin_efb) in [
                ("PLLADDR0", "PLLADRO0"),
                ("PLLADDR1", "PLLADRO1"),
                ("PLLADDR2", "PLLADRO2"),
                ("PLLADDR3", "PLLADRO3"),
                ("PLLADDR4", "PLLADRO4"),
                ("PLLDATI0", "PLLDATO0"),
                ("PLLDATI1", "PLLDATO1"),
                ("PLLDATI2", "PLLDATO2"),
                ("PLLDATI3", "PLLDATO3"),
                ("PLLDATI4", "PLLDATO4"),
                ("PLLDATI5", "PLLDATO5"),
                ("PLLDATI6", "PLLDATO6"),
                ("PLLDATI7", "PLLDATO7"),
                ("PLLWE", "PLLWEO"),
                ("PLLRST", "PLLRSTO"),
                ("PLLCLK", "PLLCLKO"),
            ] {
                let wire_efb = bel.ctx.rc_wire(cell_config, &format!("J{pin_efb}_EFB"));
                bel.add_wire_rc(pin, format!("J{pin}_PLL"))
                    .claim_pip_from(pin, wire_efb);
            }
            let has_mux = matches!(
                bel.ctx.chip.kind,
                ChipKind::MachXo2(MachXo2Kind::MachXo2 | MachXo2Kind::MachXo3L)
            ) && bel.ctx.chip.try_bel_pll(DirHV::NE, 0).is_some();
            let wire_efb = bel.ctx.rc_wire(
                cell_config,
                &if has_mux {
                    format!("JPLL{idx}STBOMUX")
                } else {
                    format!("JPLL{idx}STBO_EFB")
                },
            );
            bel.add_wire_rc("PLLSTB", "JPLLSTB_PLL")
                .claim_pip_from("PLLSTB", wire_efb);
            for pin in [
                "PLLDATO0", "PLLDATO1", "PLLDATO2", "PLLDATO3", "PLLDATO4", "PLLDATO5", "PLLDATO6",
                "PLLDATO7", "PLLACK",
            ] {
                bel.add_wire_rc(pin, format!("J{pin}_PLL"));
            }
        }
    }
}
