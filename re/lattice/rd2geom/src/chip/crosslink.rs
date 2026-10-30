use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, PllLoc, PllPad, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColId, DieIdExt},
};

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_clk_crosslink(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "DCSOUT" {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_special_loc_crosslink(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix].ends_with("_PLL") {
                let cell = self.chip.xlat_rc_wire(wn);
                let loc = SpecialLocKey::Pll(PllLoc::new(DirHV::SE, 0));
                self.chip.special_loc.insert(loc, cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_OSC") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Osc, cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_GSR") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Config, cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_PMU") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Pmu, cell);
            }
        }
    }

    pub(super) fn fill_io_crosslink(&mut self) {
        self.fill_io(&[("LVDS_0", IoGroupKind::Quad), ("GPIO", IoGroupKind::Single)]);
        for (col, cd) in &mut self.chip.columns {
            if cd.io[DirV::S] != IoGroupKind::None {
                let bank = if col < self.chip.col_clk {
                    2
                } else if cd.io[DirV::S] == IoGroupKind::Quad {
                    1
                } else {
                    0
                };
                cd.bank.insert(DirV::S, bank);
            }
        }
    }

    pub(super) fn fill_bc_crosslink(&mut self) {
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[wf];
            let wtn = self.grid.nodes[wt];
            if self.grid.strings[wtn.suffix] == "JINRDENI_BCINRD" {
                let cell = self.chip.xlat_rc_wire(wfn);
                let bank = if cell.col < self.chip.col_clk { 2 } else { 1 };
                self.chip.special_loc.insert(SpecialLocKey::Bc(bank), cell);
            }
        }
    }

    pub(super) fn fill_mipi_crosslink(&mut self) {
        let name = self.grid.strings.get("DP0_MIPIDPHY").unwrap();
        for &wn in self.grid.nodes.values() {
            if wn.suffix == name {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.columns[cell.col].io[DirV::N] = IoGroupKind::Mipi;
                self.chip.columns[cell.col]
                    .bank
                    .insert(DirV::N, if cell.col < self.chip.col_clk { 60 } else { 61 });
            }
        }
    }

    pub(super) fn fill_special_io_crosslink(&mut self) {
        for (col, iob, key) in [
            (1, 2, SpecialIoKey::MipiClk(DirH::W)),
            (
                11,
                0,
                SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 0)),
            ),
            (15, 0, SpecialIoKey::Clock(Dir::S, 0)),
            (15, 2, SpecialIoKey::Clock(Dir::S, 1)),
            (28, 0, SpecialIoKey::Clock(Dir::S, 2)),
            (28, 2, SpecialIoKey::Clock(Dir::S, 3)),
            (33, 2, SpecialIoKey::MipiClk(DirH::E)),
            (46, 0, SpecialIoKey::D(2)),
            (47, 0, SpecialIoKey::D(3)),
            (46, 0, SpecialIoKey::Clock(Dir::S, 4)),
            (47, 0, SpecialIoKey::Clock(Dir::S, 5)),
            (48, 0, SpecialIoKey::PmuWakeupN),
            (49, 0, SpecialIoKey::D(0)),
            (50, 0, SpecialIoKey::D(1)),
            (51, 0, SpecialIoKey::CsN),
            (52, 0, SpecialIoKey::Cclk),
        ] {
            self.chip.special_io.insert(
                key,
                Chip::DIE
                    .cell(ColId::from_idx(col), self.chip.row_s())
                    .bel(bslots::IO[iob]),
            );
        }
        for (col, key) in [
            (5, SpecialLocKey::PclkIn(Dir::S, 0)),
            (33, SpecialLocKey::PclkIn(Dir::S, 1)),
        ] {
            self.chip
                .special_loc
                .insert(key, Chip::DIE.cell(ColId::from_idx(col), self.chip.row_s()));
        }
    }
}
