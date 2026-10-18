use prjcombine_ecp::{
    chip::{ChipKind, SpecialIoKey},
    defs::{bslots, wires},
};
use prjcombine_interconnect::{
    dir::Dir,
    grid::{BelCoord, WireCoord},
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

mod crosslink;
mod ecp;
mod ecp3;
mod ecp4;
mod ecp5;
mod machxo;
mod machxo2;
mod scm;

impl ChipContext<'_> {
    pub fn process_io(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => {
                self.prep_io_scm();
                self.process_eclk_scm();
                self.process_pll_scm();
                self.process_io_scm();
            }
            ChipKind::Ecp | ChipKind::Xp => {
                self.prep_io_ecp();
                self.process_dqsdll_ecp();
                self.process_io_ecp();
            }
            ChipKind::MachXo => self.process_io_machxo(),
            ChipKind::Ecp2 | ChipKind::Ecp2M => {
                self.prep_io_ecp();
                self.process_eclk_ecp2();
                self.process_eclk_tap_ecp2();
                self.process_dqsdll_ecp2();
                self.process_io_ecp();
            }
            ChipKind::Xp2 => {
                self.prep_io_ecp();
                self.process_eclk_xp2();
                self.process_eclk_tap_ecp2();
                self.process_dqsdll_ecp2();
                self.process_io_ecp();
            }
            ChipKind::Ecp3(_) => {
                self.prep_io_ecp3();
                self.process_eclk_ecp3();
                self.process_eclk_tap_ecp3();
                self.process_dqsdll_ecp3();
                self.process_io_ecp3();
            }
            ChipKind::MachXo2(_) => {
                self.prep_io_machxo2();
                self.process_bc_machxo2();
                self.process_eclk_machxo2();
                self.process_dqsdll_machxo2();
                self.process_dqs_machxo2();
                self.process_io_machxo2();
                self.process_clkdiv_machxo2();
                self.process_icc_machxo2();
            }
            ChipKind::Ecp4 => {
                self.prep_io_ecp4();
                self.process_bc_ecp4();
                self.process_eclk_ecp4();
                self.process_ddrdll_ecp4();
                self.process_dlldel_ecp4();
                self.process_dtr_ecp4();
                self.process_io_ecp4();
                self.process_clkdiv_ecp4();
                self.process_pcsclkdiv_ecp4();
            }
            ChipKind::Ecp5 => {
                self.prep_io_ecp5();
                self.process_bc_ecp5();
                self.process_eclk_ecp5();
                self.process_ddrdll_ecp5();
                self.process_dlldel_ecp5();
                self.process_dtr_ecp5();
                self.process_io_ecp5();
                self.process_clkdiv_ecp5();
                self.process_pcsclkdiv_ecp5();
            }
            ChipKind::Crosslink => {
                self.prep_io_crosslink();
                self.process_bc_crosslink();
                self.process_eclk_crosslink();
                self.process_ddrdll_crosslink();
                self.process_dlldel_crosslink();
                self.process_io_crosslink();
                self.process_clkdiv_crosslink();
            }
        }
    }

    pub fn get_io_wire_in(&self, bcrd: BelCoord) -> WireName {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        match self.chip.kind {
            ChipKind::Ecp3(_) => {
                match self.chip.get_io_edge(bcrd) {
                    Dir::H(h) => {
                        if bcrd.col != self.chip.col_edge(h) {
                            // PLL special
                            let abcd = ['A', 'B', 'C', 'D'][idx];
                            self.rc_io_wire(
                                bcrd.with_col(self.chip.col_edge(h)),
                                &format!("JPADDIE{abcd}_PIO"),
                            )
                        } else {
                            let abcd = ['A', 'B'][idx % 2];
                            let cell = if idx >= 2 {
                                bcrd.cell
                            } else {
                                bcrd.cell.delta(0, 2)
                            };
                            self.rc_io_wire(cell, &format!("JPADDI{abcd}_PIO"))
                        }
                    }
                    Dir::V(_) => {
                        let abcd = ['A', 'B'][idx % 2];
                        let cell = if idx >= 2 {
                            bcrd.cell.delta(2, 0)
                        } else {
                            bcrd.cell
                        };
                        self.rc_io_wire(cell, &format!("JPADDI{abcd}_PIO"))
                    }
                }
            }
            _ => {
                let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][idx];
                self.rc_io_wire(bcrd.cell, &format!("JPADDI{abcd}_PIO"))
            }
        }
    }

    pub fn get_special_io_wire_in(&self, key: SpecialIoKey) -> WireName {
        self.get_io_wire_in(self.chip.special_io[&key])
    }

    pub fn get_io_wire_out(&self, bcrd: BelCoord) -> WireName {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][idx];
        self.rc_io_wire(bcrd.cell, &format!("JPADDO{abcd}_PIO"))
    }

    pub fn get_special_io_wire_out(&self, key: SpecialIoKey) -> WireName {
        self.get_io_wire_out(self.chip.special_io[&key])
    }

    pub fn get_paddi_wire(&self, wire: WireCoord) -> WireName {
        let wire = self.edev.resolve_wire(wire).unwrap();
        let idx = wires::OUT_IO_PADDI.index_of(wire.slot).unwrap();
        match self.chip.kind {
            ChipKind::Ecp3(_) => {
                if wire.row == self.chip.row_s() || wire.row == self.chip.row_n() {
                    let cell = if idx >= 2 {
                        wire.cell.delta(2, 0)
                    } else {
                        wire.cell
                    };
                    let abcd = ['A', 'B'][idx % 2];
                    self.rc_io_wire(cell, &format!("JPADDI{abcd}_PIO"))
                } else if wire.col == self.chip.col_w() || wire.col == self.chip.col_e() {
                    let cell = if idx >= 2 {
                        wire.cell
                    } else {
                        wire.cell.delta(0, 2)
                    };
                    let abcd = ['A', 'B'][idx % 2];
                    self.rc_io_wire(cell, &format!("JPADDI{abcd}_PIO"))
                } else {
                    let cell = if wire.col < self.chip.col_clk {
                        wire.cell.delta(-1, 0)
                    } else {
                        wire.cell.delta(1, 0)
                    };
                    let abcd = ['A', 'B', 'C', 'D'][idx];
                    self.rc_io_wire(cell, &format!("JPADDIE{abcd}_PIO"))
                }
            }
            ChipKind::MachXo2(_) => {
                let abcd = ['A', 'B', 'C', 'D'][idx];
                self.rc_io_wire(wire.cell, &format!("JDI{abcd}"))
            }
            ChipKind::Scm => {
                let (cell, abcd) = self.xlat_io_loc_scm(wire.bel(bslots::IO[idx]));
                self.rc_io_wire(cell, &format!("JINDDCK{abcd}"))
            }
            _ => {
                let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][idx];
                self.rc_io_wire(wire.cell, &format!("JPADDI{abcd}_PIO"))
            }
        }
    }
}
