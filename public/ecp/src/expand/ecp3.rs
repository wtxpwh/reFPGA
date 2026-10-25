use prjcombine_entity::EntityIdArith;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColIdExt, DieIdExt},
};

use crate::{
    chip::{Chip, ChipKind, Ecp3Kind, IoGroupKind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, ccls, rslots, tcls},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_config_ecp3(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.bel_holes.push(cell.rect(12, 1));
        let mut tcells = cell.cells_e(13);
        tcells.push(self.chip.clk_root().step(DirH::W, 2));
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);

        let cell = self.die.cell(self.chip.col_w(), self.chip.row_s());
        self.egrid.add_tile_e(cell, tcls::TEST_SW, 3);
        let cell = self.die.cell(self.chip.col_e(), self.chip.row_s());
        self.egrid.add_tile_we(cell, tcls::TEST_SE, 2, 3);
        let cell = self.die.cell(self.chip.col_w(), self.chip.row_n());
        self.egrid.add_tile_e(cell, tcls::TEST_NW, 2);
        let cell = self.die.cell(self.chip.col_e(), self.chip.row_n());
        self.egrid.add_tile_we(cell, tcls::TEST_NE, 1, 2);
    }

    pub(super) fn fill_pll_ecp3(&mut self) {
        let is_a = self.chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A);
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let cell_io_n = self.chip.get_io_cell(SpecialIoKey::Clock(
                Dir::N,
                match loc.quad.h {
                    DirH::W => 0,
                    DirH::E => 1,
                },
            ));
            match loc.quad.h {
                DirH::W => {
                    if cell.row == self.chip.row_clk {
                        let mut tcells = Vec::from_iter((0..15).map(|i| Some(cell.delta(i, 0))));
                        tcells.push(Some(cell.delta(-1, -3)));
                        tcells.push(Some(cell.delta(-1, 1)));
                        tcells.push(Some(cell.delta(-1, 3)));
                        tcells.push(Some(cell.delta(-1, 6)));
                        tcells.push(Some(cell_io_n));
                        if self.chip.try_bel_pll(loc.quad, 1).is_none() {
                            tcells.push(Some(cell_io_n));
                        } else {
                            tcells.push(None);
                        }
                        self.egrid.add_tile_part(
                            cell,
                            if is_a {
                                tcls::PLL_DLL_A_W
                            } else {
                                tcls::PLL_DLL_W
                            },
                            &tcells,
                        );
                        self.bel_holes.push(cell.rect(15, 1));
                    } else {
                        let mut tcells = Vec::from_iter((0..12).map(|i| Some(cell.delta(i, 0))));
                        tcells.push(Some(cell.delta(-1, -3)));
                        if loc.quad.v == DirV::N {
                            tcells.push(Some(cell_io_n));
                        } else {
                            tcells.push(None);
                        }
                        self.egrid.add_tile_part(
                            cell,
                            if is_a { tcls::PLL_A_W } else { tcls::PLL_W },
                            &tcells,
                        );
                        self.bel_holes.push(cell.rect(12, 1));
                    }
                    self.egrid.add_tile_e(
                        cell,
                        if is_a {
                            tcls::IO_A_PLL_W
                        } else {
                            tcls::IO_PLL_W
                        },
                        2,
                    );
                }
                DirH::E => {
                    if cell.row == self.chip.row_clk {
                        let mut tcells = Vec::from_iter((0..15).map(|i| Some(cell.delta(-i, 0))));
                        tcells.push(Some(cell.delta(1, -3)));
                        tcells.push(Some(cell.delta(1, 1)));
                        tcells.push(Some(cell.delta(1, 3)));
                        tcells.push(Some(cell.delta(1, 6)));
                        tcells.push(Some(cell_io_n));
                        if self.chip.try_bel_pll(loc.quad, 1).is_none() {
                            tcells.push(Some(cell_io_n));
                        } else {
                            tcells.push(None);
                        }
                        self.egrid.add_tile_part(
                            cell,
                            if is_a {
                                tcls::PLL_DLL_A_E
                            } else {
                                tcls::PLL_DLL_E
                            },
                            &tcells,
                        );
                        self.bel_holes.push(cell.delta(-14, 0).rect(15, 1));
                    } else {
                        let mut tcells = Vec::from_iter((0..12).map(|i| Some(cell.delta(-i, 0))));
                        tcells.push(Some(cell.delta(1, -3)));
                        if loc.quad.v == DirV::N {
                            tcells.push(Some(cell_io_n));
                        } else {
                            tcells.push(None);
                        }
                        self.egrid.add_tile_part(
                            cell,
                            if is_a { tcls::PLL_A_E } else { tcls::PLL_E },
                            &tcells,
                        );
                        self.bel_holes.push(cell.delta(-11, 0).rect(12, 1));
                    }
                    self.egrid.add_tile(
                        cell,
                        if is_a {
                            tcls::IO_A_PLL_E
                        } else {
                            tcls::IO_PLL_E
                        },
                        &[cell, cell.delta(-1, 0)],
                    );
                }
            }
        }
        {
            self.bel_holes
                .push(self.chip.clk_root().delta(-3, 0).rect(6, 1));
        }
    }

    pub(super) fn fill_serdes_ecp3(&mut self) {
        let mut first = true;
        let mut last = None;
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.chip.columns[cell.col].io[DirV::S] == IoGroupKind::Serdes {
                if first {
                    self.bel_holes.push(cell.delta(-3, 0).rect(3, 10));
                    for dx in 0..3 {
                        self.egrid
                            .add_tile_single(cell.delta(-3 + dx, 9), tcls::INT_IO_S);
                    }
                    first = false;
                }
                self.bel_holes.push(cell.rect(36, 10));
                for dx in 0..36 {
                    self.egrid
                        .add_tile_single(cell.delta(dx, 9), tcls::INT_IO_S);
                }
                self.egrid.add_tile_e(cell.delta(0, 9), tcls::SERDES, 36);
                last = Some(cell);
            }
        }
        let cell = last.unwrap().delta(36, 0);
        self.bel_holes.push(cell.rect(3, 10));
        for dx in 0..3 {
            self.egrid
                .add_tile_single(cell.delta(dx, 9), tcls::INT_IO_S);
        }
    }

    pub(super) fn fill_io_ecp3(&mut self) {
        let is_a = self.chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A);
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if !self.is_in_bel_hole(cell) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            }
            if self.chip.columns[cell.col].io[DirV::S] == IoGroupKind::Quad {
                self.egrid.add_tile_e(cell, tcls::XSIO_S, 3);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            match (
                self.chip.columns[cell.col].io[DirV::N],
                self.chip.columns[cell.col].bank.get(DirV::N).copied(),
            ) {
                (IoGroupKind::Quad, Some(8)) => {
                    self.egrid.add_tile_e(cell, tcls::XSIO_N, 3);
                }
                (IoGroupKind::Quad, _) => {
                    self.egrid
                        .add_tile_e(cell, if is_a { tcls::SIO_A_N } else { tcls::SIO_N }, 3);
                }
                (IoGroupKind::QuadDqs, _) => {
                    self.egrid.add_tile_e(
                        cell,
                        if is_a {
                            tcls::SIO_A_DQS_N
                        } else {
                            tcls::SIO_DQS_N
                        },
                        3,
                    );
                    self.egrid.add_tile_e(cell, tcls::DQS_N, 6);
                    for col_io in cell.delta(-3, 0).col.range(cell.col + 6) {
                        self.egrid[cell.with_col(col_io)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.with_col(col_io)].region_root[rslots::DQSDLL] =
                            self.chip.bel_dqsdll(cell).cell;
                    }
                }
                _ => (),
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            if matches!(self.chip.rows[cell.row].kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            match self.chip.rows[cell.row].io[DirH::W] {
                IoGroupKind::Quad => {
                    self.egrid
                        .add_tile_n(cell, if is_a { tcls::IO_A_W } else { tcls::IO_W }, 3);
                }
                IoGroupKind::QuadDqs => {
                    self.egrid.add_tile_n(
                        cell,
                        if is_a {
                            tcls::IO_A_DQS_W
                        } else {
                            tcls::IO_DQS_W
                        },
                        3,
                    );
                    self.egrid
                        .add_tile_n(cell, if is_a { tcls::DQS_A_W } else { tcls::DQS_W }, 3);
                    for row_io in cell.delta(0, -3).row.range(cell.row + 6) {
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQSDLL] =
                            self.chip.bel_dqsdll(cell).cell;
                    }
                    self.egrid[cell.delta(1, -3)].region_root[rslots::DQS] = cell;
                }
                IoGroupKind::QuadDqsDummy => {
                    self.egrid.add_tile_n(
                        cell,
                        if is_a {
                            tcls::IO_A_DQS_DUMMY_W
                        } else {
                            tcls::IO_DQS_DUMMY_W
                        },
                        3,
                    );
                    self.egrid
                        .add_tile_n(cell, if is_a { tcls::DQS_A_W } else { tcls::DQS_W }, 3);
                    for row_io in cell.delta(0, -3).row.range(cell.row + 3) {
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQSDLL] =
                            self.chip.bel_dqsdll(cell).cell;
                    }
                }
                _ => (),
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if matches!(self.chip.rows[cell.row].kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            match (
                self.chip.rows[cell.row].io[DirH::E],
                self.chip.rows[cell.row].bank.get(DirH::E).copied(),
            ) {
                (IoGroupKind::Quad, Some(8)) => {
                    self.egrid.add_tile_n(cell, tcls::XSIO_E, 3);
                }
                (IoGroupKind::Quad, _) => {
                    self.egrid
                        .add_tile_n(cell, if is_a { tcls::IO_A_E } else { tcls::IO_E }, 3);
                }
                (IoGroupKind::QuadDqs, _) => {
                    self.egrid.add_tile_n(
                        cell,
                        if is_a {
                            tcls::IO_A_DQS_E
                        } else {
                            tcls::IO_DQS_E
                        },
                        3,
                    );
                    self.egrid
                        .add_tile_n(cell, if is_a { tcls::DQS_A_E } else { tcls::DQS_E }, 3);
                    for row_io in cell.delta(0, -3).row.range(cell.row + 6) {
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.with_row(row_io)].region_root[rslots::DQSDLL] =
                            self.chip.bel_dqsdll(cell).cell;
                    }
                    self.egrid[cell.delta(-1, -3)].region_root[rslots::DQS] = cell;
                }
                _ => (),
            }
        }
        for (edge, tcid) in [
            (
                Dir::W,
                if is_a {
                    tcls::ECLK_ROOT_W_A
                } else {
                    tcls::ECLK_ROOT_W
                },
            ),
            (
                Dir::E,
                if is_a {
                    tcls::ECLK_ROOT_E_A
                } else {
                    tcls::ECLK_ROOT_E
                },
            ),
            (
                Dir::N,
                if is_a {
                    tcls::ECLK_ROOT_N_A
                } else {
                    tcls::ECLK_ROOT_N
                },
            ),
        ] {
            let cell = self.chip.bel_eclksync(edge, 0).cell;
            let mut tcells = vec![
                Some(cell),
                Some(self.chip.clk_root()),
                Some(self.chip.get_io_cell(SpecialIoKey::Clock(edge, 0))),
                Some(self.chip.get_io_cell(SpecialIoKey::Clock(edge, 1))),
            ];
            if let Dir::H(h) = edge {
                tcells.push(Some(self.chip.bel_pll(DirV::N & h, 0).cell));
                if is_a {
                    tcells.push(self.chip.try_bel_pll(DirV::S & h, 0).map(|bcrd| bcrd.cell));
                }
            } else {
                tcells.extend([
                    Some(
                        self.chip
                            .try_bel_pll(DirHV::NW, 1)
                            .unwrap_or(self.chip.bel_pll(DirHV::NW, 0))
                            .cell,
                    ),
                    Some(
                        self.chip
                            .try_bel_pll(DirHV::NE, 1)
                            .unwrap_or(self.chip.bel_pll(DirHV::NE, 0))
                            .cell,
                    ),
                    Some(self.chip.bel_pll(DirHV::NW, 0).cell),
                    Some(self.chip.bel_pll(DirHV::NE, 0).cell),
                ]);
            }
            self.egrid.add_tile_part(cell, tcid, &tcells);
        }
    }

    pub(super) fn fill_sclk_ecp3(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            let tcid = if cell.col == self.chip.col_w() {
                tcls::RSCLK_DRIVER_W
            } else if cell.col == self.chip.col_e() {
                tcls::RSCLK_DRIVER_E
            } else {
                tcls::RSCLK_DRIVER
            };
            if self.egrid.bel_delta(cell, 0, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, -1, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, 1, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, 2, 0, bslots::INT).is_none()
            {
                continue;
            }
            self.egrid.add_tile_single(cell, tcid);
        }

        for i in 0..4 {
            for cell in self.egrid.die_cells(self.die) {
                let (dx, dx_alt) = match (i + 4 - self.chip.col_sclk_idx(cell.col)) % 4 {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (-2, -1),
                    3 => (-1, 0),
                    _ => unreachable!(),
                };
                let cell_src = if let Some(cell_src) = self.egrid.cell_delta(cell, dx, 0) {
                    cell_src
                } else if let Some(cell_src) = self.egrid.cell_delta(cell, dx_alt, 0) {
                    cell_src
                } else {
                    cell
                };
                self.egrid[cell].region_root[rslots::RSCLK[i]] = cell_src;
            }
        }

        for col in self.chip.columns.ids() {
            let mut root = self.die.cell(col, self.chip.row_s());
            for cell in self.egrid.column(self.die, col) {
                if self.chip.rows[cell.row].sclk_break {
                    root = cell;
                }
                self.egrid[cell].region_root[rslots::RSCLK_V] = root;
            }
        }
    }

    pub(super) fn fill_pclk_ecp3(&mut self) {
        let is_xo2 = matches!(self.chip.kind, ChipKind::MachXo2(_));
        for cell in self.egrid.die_cells(self.die) {
            let idx = self.chip.col_sclk_idx(cell.col);
            self.egrid.fill_conn_term(
                cell,
                [ccls::RCLK0, ccls::RCLK1, ccls::RCLK2, ccls::RCLK3][idx],
            );
            let row_pclk = self.chip.rows_pclk[cell.row];
            self.egrid[cell].region_root[rslots::PCLK_COL] =
                cell.with_row(if cell.row < row_pclk {
                    row_pclk - 1
                } else {
                    row_pclk
                });
            let col_pclk = self.chip.cols_pclk[cell.col];
            self.egrid[cell].region_root[rslots::PCLK_ROW] =
                cell.with_col(if cell.col < col_pclk {
                    col_pclk - 1
                } else {
                    col_pclk
                });
            let tcid =
                if cell.col == self.chip.col_w() || (!is_xo2 && cell.col == self.chip.col_clk) {
                    tcls::RPCLK_DRIVER_W
                } else if cell.col == self.chip.col_e()
                    || (!is_xo2 && cell.col == self.chip.col_clk - 1)
                {
                    tcls::RPCLK_DRIVER_E
                } else {
                    tcls::RPCLK_DRIVER
                };
            if self.egrid.bel_delta(cell, 0, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, -1, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, 1, 0, bslots::INT).is_none()
                && self.egrid.bel_delta(cell, 2, 0, bslots::INT).is_none()
            {
                continue;
            }
            self.egrid.add_tile_single(cell, tcid);
        }

        for i in 0..4 {
            for cell in self.egrid.die_cells(self.die) {
                let (dx, dx_alt) = match (i + 4 - self.chip.col_sclk_idx(cell.col)) % 4 {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (-2, -1),
                    3 => (-1, 0),
                    _ => unreachable!(),
                };
                let cell_src = if let Some(cell_src) = self.egrid.cell_delta(cell, dx, 0)
                    && (cell_src.col.side_of(self.chip.col_clk)
                        == cell.col.side_of(self.chip.col_clk)
                        || is_xo2)
                {
                    cell_src
                } else if let Some(cell_src) = self.egrid.cell_delta(cell, dx_alt, 0)
                    && (cell_src.col.side_of(self.chip.col_clk)
                        == cell.col.side_of(self.chip.col_clk)
                        || is_xo2)
                {
                    cell_src
                } else {
                    cell
                };
                self.egrid[cell].region_root[rslots::RPCLK[i]] = cell_src;
            }
        }

        for (rows, &row_drv) in &self.chip.rows_pclk {
            for cell in self.egrid.row(self.die, row_drv) {
                if row_drv == self.chip.row_s() {
                    let kind = if cell.col == self.chip.col_w()
                        || (cell.col == self.chip.col_clk && !is_xo2)
                    {
                        tcls::PCLK_COL_DRIVER_IO_N_W
                    } else if cell.col == self.chip.col_e()
                        || (cell.col == self.chip.col_clk - 1 && !is_xo2)
                    {
                        tcls::PCLK_COL_DRIVER_IO_N_E
                    } else {
                        tcls::PCLK_COL_DRIVER_IO_N
                    };
                    self.egrid.add_tile_single(cell, kind);
                } else {
                    let kind = if cell.col == self.chip.col_w()
                        || (cell.col == self.chip.col_clk && !is_xo2)
                    {
                        tcls::PCLK_COL_DRIVER_W
                    } else if cell.col == self.chip.col_e()
                        || (cell.col == self.chip.col_clk - 1 && !is_xo2)
                    {
                        tcls::PCLK_COL_DRIVER_E
                    } else {
                        tcls::PCLK_COL_DRIVER
                    };
                    if rows.start() == row_drv {
                        self.egrid.add_tile_part(cell, kind, &[Some(cell), None]);
                    } else {
                        self.egrid.add_tile(cell, kind, &[cell, cell.delta(0, -1)]);
                    }
                }
            }
            for (cols, &col_drv) in &self.chip.cols_pclk {
                let cell = self.die.cell(col_drv, row_drv);
                self.egrid.add_tile_part(
                    cell,
                    tcls::PCLK_ROW_DRIVER,
                    &[
                        Some(cell),
                        if col_drv != cols.start() {
                            Some(cell.delta(-1, 0))
                        } else {
                            None
                        },
                        if col_drv != cols.end() {
                            Some(cell)
                        } else {
                            None
                        },
                    ],
                );
            }
        }
    }

    pub(super) fn fill_clk_ecp3(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if !self.chip.is_col_io(cell.col) {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk - 1);
            }
            if self.chip.columns[cell.col].eclk_tap[DirV::N] {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }

        for col in [self.chip.col_w(), self.chip.col_e()] {
            for cell in self.egrid.column(self.die, col) {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_row(self.chip.row_clk);
            }
        }
        for (loc, bcrd) in self.chip.plls() {
            self.egrid[bcrd.cell].region_root[rslots::ECLK] = self.chip.clk_emid(loc.quad.h);
        }

        let mut tcells = vec![];
        // center
        for idx in [6, 2, 0] {
            let cell = self.chip.special_loc[&SpecialLocKey::PclkInMid(idx)];
            tcells.extend(cell.cells_e(2).into_iter().map(Some));
        }
        // edges
        for rect in [
            self.chip.clk_emid(Dir::W).step(DirV::S, 3).cells_n(6),
            self.chip.clk_emid(Dir::E).step(DirV::S, 4).cells_n(8),
            self.chip.clk_emid(Dir::S).step(DirH::W, 3).cells_e(6),
            self.chip.clk_emid(Dir::N).step(DirH::W, 3).cells_e(6),
        ] {
            tcells.extend(rect.into_iter().map(Some));
        }
        // PLLs
        for hv in DirHV::DIRS {
            let num = if hv.v == DirV::S { 3 } else { 2 };
            tcells.extend((0..num).map(|i| self.chip.try_bel_pll(hv, i).map(|bcrd| bcrd.cell)));
        }
        // SERDES
        let mut serdes_cells = [None; 4];
        for (col, cd) in &self.chip.columns {
            if cd.io[DirV::S] == IoGroupKind::Serdes {
                let idx = usize::try_from(cd.bank[DirV::S] - 50).unwrap();
                serdes_cells[idx] = Some(Chip::DIE.cell(col, self.chip.row_serdes(DirV::S)));
            }
        }
        tcells.extend(serdes_cells);

        self.egrid
            .add_tile_part(self.chip.clk_root(), tcls::CLK_ROOT, &tcells);
    }
}
