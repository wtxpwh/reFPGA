use std::collections::BTreeSet;

use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotId, CellSlotIdExt, WireSlotIdExt},
    dir::{DirHV, DirV},
};

use crate::{ChipContext, sb::EdgeKind};

impl ChipContext<'_> {
    pub(super) fn prep_pll_nexus(&mut self) {
        for (_loc, bcrd) in self.chip.plls() {
            for (pin, wire) in [
                ("CLKOP", wires::OUT_PLL_CLKOP[0]),
                ("CLKOS", wires::OUT_PLL_CLKOS[0]),
                ("CLKOS2", wires::OUT_PLL_CLKOS2[0]),
                ("CLKOS3", wires::OUT_PLL_CLKOS3[0]),
                ("CLKOS4", wires::OUT_PLL_CLKOS4[0]),
                ("CLKOS5", wires::OUT_PLL_CLKOS5[0]),
            ] {
                self.hybrid_out_rc(bcrd, format!("J{pin}_PLL_CORE_PLL_LMMI"), wire.cell(0));
            }
        }
    }

    pub(super) fn process_pll_nexus(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let corner = match loc.quad {
                DirHV::SW => "LLC",
                DirHV::SE => "LRC",
                DirHV::NW => "ULC",
                DirHV::NE => "URC",
            };
            let tcrd = self.edev.bel_tile(bcrd);
            let refck = self.rc_wire(bcrd.cell, "JREFCK_REFMUX_CORE_REFMUX");
            for i in 0..2 {
                let ii = i + 1;
                let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                ex.output_raw_no_claim(refck, wires::IMUX_PLL_CLKI[i].cell(0), EdgeKind::Mux)
                    .input_xlat_rc(format!("JREFCLK{ii}1_REFMUX_CORE_REFMUX"));
                let mut handled = BTreeSet::new();
                handled.insert(1);

                if i == 0 {
                    ex.input_io_rc(
                        "JREFCLK14_REFMUX_CORE_REFMUX",
                        wires::OUT_IO_INCK[0].cell(3),
                    );
                    handled.insert(4);
                }

                match loc.quad.v {
                    DirV::S => {
                        for (j, ci) in [(3, 4 + i), (5, 6 + i), (6, 8 + i)] {
                            let cslot = CellSlotId::from_idx(ci);
                            let wire = cslot.wire(wires::OUT_IO_INCK[0]);
                            if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                                ex.input_io_rc(format!("JREFCLK{ii}{j}_REFMUX_CORE_REFMUX"), wire);
                            } else {
                                ex.input_raw_rc(format!("JREFCLK{ii}{j}_REFMUX_CORE_REFMUX"), wire);
                            }
                            handled.insert(j);
                        }
                    }
                    DirV::N => {
                        for (j, ci) in [(3, 4 + i), (6, 6 + i)] {
                            let cslot = CellSlotId::from_idx(ci);
                            let wire = cslot.wire(wires::OUT_IO_INCK[0]);
                            if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                                ex.input_io_rc(format!("JREFCLK{ii}{j}_REFMUX_CORE_REFMUX"), wire);
                            } else {
                                ex.input_raw_rc(format!("JREFCLK{ii}{j}_REFMUX_CORE_REFMUX"), wire);
                            }
                            handled.insert(j);
                        }
                        if i == 0 {
                            let cslot = CellSlotId::from_idx(8);
                            if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                                ex.input_xlat_rc_nudge(
                                    format!("JREFCLK{ii}5_REFMUX_CORE_REFMUX"),
                                    cslot,
                                )
                                .input_xlat_rc_nudge(
                                    format!("JREFCLK{ii}7_REFMUX_CORE_REFMUX"),
                                    cslot,
                                );
                            } else {
                                ex.input_raw_rc(
                                    format!("JREFCLK{ii}5_REFMUX_CORE_REFMUX"),
                                    cslot.wire(wires::OUT_PCS_CLKTX[3]),
                                )
                                .input_raw_rc(
                                    format!("JREFCLK{ii}7_REFMUX_CORE_REFMUX"),
                                    cslot.wire(wires::OUT_PCS_CLKTX[2]),
                                );
                            }
                            handled.insert(5);
                            handled.insert(7);
                        }
                    }
                }

                for j in 0..8 {
                    if !handled.contains(&j) {
                        ex.input_bind_rc(
                            format!("JREFCLK{ii}{j}_REFMUX_CORE_REFMUX"),
                            wires::IMUX_PLL_CLKI_DUMMY[i * 8 + j].cell(0),
                        );
                    }
                }
            }
            {
                let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                ex.output_bind_rc("JFBKCK_FBMUX_CORE_FBKMUX", wires::IMUX_PLL_CLKFB[0].cell(0))
                    .input_bind_rc(
                        "JFBKCLK6_FBMUX_CORE_FBKMUX",
                        wires::OUT_PLL_CLKOUTDL.cell(0),
                    )
                    .input_xlat_rc("JFBKCLK11_FBMUX_CORE_FBKMUX");
                if loc.quad.v == DirV::S {
                    ex.input_xlat_rc("JFBKCLK10_FBMUX_CORE_FBKMUX");
                }
                for i in 0..6 {
                    ex.input_bind_rc(
                        format!("JFBKCLK{i}_FBMUX_CORE_FBKMUX"),
                        wires::OUT_PLL_CLKINTFB[i].cell(0),
                    );
                }
                for i in 7..16 {
                    if i == 11 || (i == 10 && loc.quad.v == DirV::S) {
                        continue;
                    }
                    ex.input_bind_rc(
                        format!("JFBKCLK{i}_FBMUX_CORE_FBKMUX"),
                        wires::IMUX_PLL_CLKFB_DUMMY[i].cell(0),
                    );
                }
            }
            let (r, c) = self.rc(bcrd.cell);
            let mut bel = self.extract_bel(bcrd);
            bel.names([
                format!("PLL_{corner}"),
                if loc.quad == DirHV::NE {
                    format!("REFMUX_CORE_R{r}C{c}")
                } else {
                    format!("REFMUX_{corner}")
                },
                format!("FBMUX_{corner}"),
            ])
            .suffix("PLL_CORE_PLL_LMMI")
            .input(bcls::PLL::CLK[0], wires::IMUX_PLL_CLKI[0].cell(0))
            .input(bcls::PLL::CLK[1], wires::IMUX_PLL_CLKI[1].cell(0))
            .add_wire_rc("REFCK", "JREFCK_PLL_CORE_PLL_LMMI")
            .add_wire("REFCK_MUX", refck)
            .claim_pip("REFCK", "REFCK_MUX")
            .input_xlat_rc(bcls::PLL::CLKFB, "JFBKCK_PLL_CORE_PLL_LMMI")
            .input_xlat_rc(bcls::PLL::WKUPSYNC, "JWKUPSYNC_FBMUX_CORE_FBKMUX")
            .input_xlat_rc(bcls::PLL::REFSEL, "JREFSEL_REFMUX_CORE_REFMUX")
            .add_wire_rc("LEGRDYN", "JLEGRDYN_PLL_CORE_PLL_LMMI")
            .add_wire_rc("ENEXT", "JENEXT_PLL_CORE_PLL_LMMI")
            .add_wire_rc("ZRSEL3", "JZRSEL3_PLL_CORE_PLL_LMMI")
            .add_wire_rc("INTLOCK_FBMUX", "JINTLOCK_FBMUX_CORE_FBKMUX")
            .add_wire_rc("LEGRDYN_FBMUX", "JLGYRDYN_FBMUX_CORE_FBKMUX")
            .add_wire_rc("ENEXT_FBMUX", "JENEXT_FBMUX_CORE_FBKMUX")
            .add_wire_rc("ZRSEL3_REFMUX", "JZRSEL3_REFMUX_CORE_REFMUX")
            .claim_pip("LEGRDYN_FBMUX", "LEGRDYN")
            .claim_pip("INTLOCK_FBMUX", "INTLOCK")
            .claim_pip("ENEXT", "ENEXT_FBMUX")
            .claim_pip("ZRSEL3", "ZRSEL3_REFMUX");
        }
    }
}
