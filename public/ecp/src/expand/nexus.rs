use prjcombine_entity::EntityId;
use prjcombine_interconnect::dir::DirH;

use crate::{
    chip::RowKind,
    defs::{rslots, tcls},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_ebr_nexus(&mut self) {
        for (row, rd) in &self.chip.rows {
            if !matches!(rd.kind, RowKind::Ebr | RowKind::Dsp) {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if cell.col != self.chip.col_w() && cell.col != self.chip.col_e() {
                    self.egrid.add_tile_single(cell, tcls::INT_EBR);
                }
                if rd.kind == RowKind::Ebr {
                    if (cell.col < self.chip.col_clk && cell.col.to_idx() % 12 == 1)
                        || (cell.col >= self.chip.col_clk && cell.col.to_idx() % 12 == 2)
                    {
                        self.egrid.add_tile_e(cell, tcls::EBR, 11);
                    }
                } else {
                    if cell.col.to_idx() % 12 == 1 && cell.col != self.chip.col_e() {
                        self.egrid.add_tile_e(
                            cell,
                            if cell.col < self.chip.col_clk {
                                tcls::DSP_W
                            } else {
                                tcls::DSP_E
                            },
                            12,
                        );
                    }
                }
            }
        }
    }

    pub(super) fn fill_io_nexus(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
        }
        for edge in DirH::DIRS {
            for cell in self.egrid.column(self.die, self.chip.col_edge(edge)) {
                if cell.row == self.chip.row_s() || cell.row == self.chip.row_n() {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
        }
    }

    pub(super) fn fill_pclk_nexus(&mut self) {
        for (cols, &col_drv) in &self.chip.cols_pclk {
            let col_split = if col_drv < self.chip.col_clk {
                col_drv + 1
            } else {
                col_drv
            };
            for cell in self.egrid.column(self.die, col_drv) {
                self.egrid.add_tile_part(
                    cell,
                    tcls::PCLK_DRIVER,
                    &[
                        if cols.start() == col_split {
                            None
                        } else {
                            Some(cell.with_col(col_split - 1))
                        },
                        if cols.end() == col_split {
                            None
                        } else {
                            Some(cell.with_col(col_split))
                        },
                        Some(cell),
                    ],
                );
                if self.egrid[cell].region_root[rslots::PCLK_COL] == cell {
                    self.egrid.add_tile_single(cell, tcls::PCLK_COL_DRIVER);
                }
            }
        }
    }
}
