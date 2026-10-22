use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV},
    grid::DieIdExt,
};

use crate::{
    chip::{IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{rslots, tcls},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_ebr_crosslink(&mut self) {
        let mut col_start_w = self.chip.col_w();
        let mut col_end_e = self.chip.col_e() + 1usize;
        let col_end_w = self.chip.col_clk;
        let col_start_e = self.chip.col_clk + 1usize;
        while col_start_w.to_idx() % 10 != col_end_w.to_idx() % 10 {
            col_start_w += 1;
        }
        while col_start_e.to_idx() % 10 != col_end_e.to_idx() % 10 {
            col_end_e -= 1;
        }

        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
                if (cell.col >= col_start_w
                    && cell.col < col_end_w
                    && cell.col.to_idx() % 10 == col_start_w.to_idx() % 10)
                    || (cell.col >= col_start_e
                        && cell.col < col_end_e
                        && cell.col.to_idx() % 10 == col_start_e.to_idx() % 10)
                {
                    self.egrid.add_tile_e(cell, tcls::EBR, 9);
                }
            }
        }
    }

    pub(super) fn fill_io_crosslink(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            if matches!(key, SpecialLocKey::Bc(_)) {
                self.egrid.add_tile_single(cell, tcls::BC);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            match self.chip.columns[cell.col].io_s {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_e(cell, tcls::IO_S4, 4);
                }
                IoGroupKind::Single => {
                    if cell.col.to_idx() % 2 == 0 {
                        self.egrid.add_tile_single(cell, tcls::IO_S1A);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::IO_S1B);
                    }
                }
                _ => (),
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            if self.chip.columns[cell.col].io_n == IoGroupKind::Mipi {
                let mut tcells = cell.cells_e(24);
                tcells.extend([
                    self.chip
                        .get_io_cell(SpecialIoKey::MipiClk(if cell.col < self.chip.col_clk {
                            DirH::W
                        } else {
                            DirH::E
                        })),
                    self.chip.get_io_cell(SpecialIoKey::Clock(
                        Dir::S,
                        if cell.col < self.chip.col_clk { 4 } else { 5 },
                    )),
                ]);
                self.egrid.add_tile(
                    cell,
                    if cell.col < self.chip.col_clk {
                        tcls::MIPI_W
                    } else {
                        tcls::MIPI_E
                    },
                    &tcells,
                );
            }
        }
    }

    pub(super) fn fill_clk_crosslink(&mut self) {
        let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
        let mut tcells = cell.delta(-2, 0).cells_e(4);
        for i in 0..2 {
            tcells.push(self.chip.special_loc[&SpecialLocKey::PclkIn(Dir::S, i)]);
        }
        for i in [0, 2] {
            tcells.push(self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, i)));
        }
        tcells.push(self.chip.bel_pll(DirHV::SE, 0).cell);
        tcells.push(self.chip.special_loc[&SpecialLocKey::Osc]);
        self.egrid.add_tile(cell, tcls::CLK_S, &tcells);

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
        let mut tcells = cell.delta(-1, 0).cells_e(2);
        tcells.extend([
            cell.with_row(self.chip.row_s()),
            self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 4)),
            self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 5)),
        ]);
        for (col, cd) in &self.chip.columns {
            if cd.io_n == IoGroupKind::Mipi {
                tcells.push(cell.with_col(col));
            }
        }
        self.egrid.add_tile(cell, tcls::CLK_N, &tcells);

        let cell = self.chip.bel_clk_root().cell;
        self.egrid.add_tile(
            cell,
            tcls::CLK_ROOT,
            &[
                cell.delta(-1, 0),
                cell,
                cell.with_row(self.chip.row_s()),
                cell.with_row(self.chip.row_n()),
            ],
        );

        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid[cell].region_root[rslots::ECLK] =
                cell.with_col(if cell.col < self.chip.col_clk {
                    self.chip.col_clk - 1
                } else {
                    self.chip.col_clk
                });
        }
    }

    pub(super) fn fill_config_crosslink(&mut self) {
        let cell = self.die.cell(self.chip.col_w(), self.chip.row_n());
        self.egrid.add_tile_e(cell, tcls::I2C_W, 2);
        let cell = self.die.cell(self.chip.col_e() - 1, self.chip.row_n());
        self.egrid.add_tile_e(cell, tcls::I2C_E, 2);

        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut tcells = cell.cells_e(2);
        tcells.extend([
            cell.delta(-2, 0),
            cell.with_cr(self.chip.col_clk, self.chip.row_clk),
        ]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);

        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.egrid.add_tile_single(cell, tcls::OSC);

        let cell = self.chip.special_loc[&SpecialLocKey::Pmu];
        self.egrid.add_tile_single(cell, tcls::PMU);

        let cell = self.chip.bel_pll(DirHV::SE, 0).cell;
        self.egrid.add_tile(
            cell,
            tcls::PLL,
            &[
                cell,
                cell.delta(1, 0),
                self.chip.clk_root().with_row(self.chip.row_s()),
                self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 0)),
                self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 2)),
                self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 4)),
                self.chip.get_io_cell(SpecialIoKey::Clock(Dir::S, 5)),
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 0))),
            ],
        );
    }
}
