use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind},
    defs::{bslots, tcls, wires},
};
use prjcombine_interconnect::{db::WireSlotIdExt, grid::DieIdExt};

use crate::ChipContext;

mod crosslink;
mod ecp;
mod ecp2;
mod ecp3;
mod ecp4;
mod ecp5;
mod machxo;
mod machxo2;
mod nexus;
mod scm;

impl ChipContext<'_> {
    fn process_sclk_h_splitter(&mut self) {
        for tcid in [tcls::SCLK_H_SPLITTER, tcls::SCLK_H_ROOT] {
            if !self.intdb.tile_classes.contains_id(tcid) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::SCLK_H_SPLITTER);
                let cell = tcrd.cell.delta(-1, 0);
                let mut ex = self.extract_switchbox(bcrd);
                ex.cell(cell);
                for i in 0..8 {
                    ex.progbuf_output_int(wires::SCLK_H[i].cell(0))
                        .progbuf_output_int(wires::SCLK_H[i].cell(1))
                        .input_xlat_rc(format!("HSSX0{i}00_L2R"))
                        .input_xlat_rc(format!("HSSX0{i}00_R2L"));
                }
            }
        }
    }

    fn process_sclk_h_vcc(&mut self) {
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Io && !rd.sclk_break {
                continue;
            }
            for (col, cd) in &self.chip.columns {
                if col != self.chip.col_w() && !cd.sclk_break {
                    continue;
                }
                let mut cell = Chip::DIE.cell(col, row);
                if !self.edev.has_bel(cell.bel(bslots::INT)) {
                    cell.row += 9;
                }
                for i in 0..8 {
                    let wire = cell.wire(wires::SCLK_H[i]);
                    let vcc = cell.wire(wires::TIE1);
                    let vcc = self.naming.interconnect[&vcc];
                    self.claim_pip_int_out(wire, vcc);
                }
            }
        }
    }

    pub fn process_clk(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => {
                self.process_pclk_scm();
                self.process_clk_edge_scm();
                self.process_clk_root_scm();
            }
            ChipKind::Ecp | ChipKind::Xp => self.process_clk_ecp(),
            ChipKind::MachXo => self.process_clk_machxo(),
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                self.process_sclk_h_splitter();
                self.process_pclk_ecp2();
                let sclk_root = self.process_sclk_root_ecp2();
                self.process_clk_ecp2(sclk_root);
            }
            ChipKind::Ecp3(_) => {
                self.process_sclk_h_splitter();
                self.process_sclk_h_vcc();
                self.process_hsdclk_root_ecp3();
                self.process_clk_ecp3();
                self.process_pclk_ecp3();
            }
            ChipKind::MachXo2(_) => {
                self.process_sclk_h_splitter();
                self.process_hsdclk_root_machxo2();
                self.process_pclk_ecp3();
                self.process_dlldel_machxo2();
                self.process_clk_machxo2();
            }
            ChipKind::Ecp4 => {
                self.process_pclk_ecp4();
                self.process_clk_edge_ecp4();
                self.process_clk_root_ecp4();
            }
            ChipKind::Ecp5 => {
                self.process_pclk_ecp5();
                self.process_clk_edge_ecp5();
                self.process_clk_root_ecp5();
            }
            ChipKind::Crosslink => {
                self.process_pclk_ecp5();
                self.process_clk_edge_crosslink();
                self.process_clk_root_crosslink();
            }
            ChipKind::Nexus(_) => {
                self.process_pclk_nexus();
                // TODO
            }
        }
    }
}
