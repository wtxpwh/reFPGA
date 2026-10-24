use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColIdExt, DieIdExt},
};

use crate::{
    chip::{IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, ccls, rslots, tcls, wires},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_config_scm(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut cells = cell.cells_e(12);
        cells.push(self.chip.clk_root().step(Dir::S, 5));
        self.egrid.add_tile(cell, tcls::CONFIG, &cells);
        for dx in 0..12 {
            self.egrid.add_tile_single(cell.delta(dx, 0), tcls::INT_EBR);
        }
        self.bel_holes.push(cell.rect(12, 1));
    }

    pub(super) fn fill_pll_scm(&mut self) {
        let cell = self.die.cell(self.chip.col_w(), self.chip.row_s());
        self.egrid.add_tile(
            cell,
            tcls::PLL_SW,
            &[
                cell,
                cell.delta(1, 0),
                cell.delta(2, 0),
                cell.delta(3, 0),
                cell.delta(0, 1),
                cell.delta(0, 2),
            ],
        );

        let cell = self.die.cell(self.chip.col_e(), self.chip.row_s());
        self.egrid.add_tile(
            cell,
            tcls::PLL_SE,
            &[
                cell,
                cell.delta(-1, 0),
                cell.delta(-2, 0),
                cell.delta(-3, 0),
                cell.delta(0, 1),
                cell.delta(0, 2),
            ],
        );

        let cell = self
            .die
            .cell(self.chip.col_w(), self.chip.row_serdes(DirV::N));
        let mut tcells = vec![Some(cell), Some(cell.delta(1, 0)), Some(cell.delta(0, -1))];
        for (col, cd) in &self.chip.columns {
            if cd.io_n == IoGroupKind::Serdes && col < self.chip.col_clk {
                tcells.push(Some(cell.with_col(col)));
            }
        }
        while tcells.len() < 7 {
            tcells.push(None);
        }
        self.egrid.add_tile_part(cell, tcls::PLL_NW, &tcells);

        let cell = self
            .die
            .cell(self.chip.col_e(), self.chip.row_serdes(DirV::N));
        let mut tcells = vec![Some(cell), Some(cell.delta(-1, 0)), Some(cell.delta(0, -1))];
        for (col, cd) in self.chip.columns.iter().rev() {
            if cd.io_n == IoGroupKind::Serdes && col >= self.chip.col_clk {
                tcells.push(Some(cell.with_col(col)));
            }
        }
        while tcells.len() < 7 {
            tcells.push(None);
        }
        self.egrid.add_tile_part(cell, tcls::PLL_NE, &tcells);

        for hv in DirHV::DIRS {
            let cell = self.chip.bel_pll(hv, 0).cell;
            for (i, pad) in [
                PllPad::PllIn0,
                PllPad::PllIn1,
                PllPad::DllIn0,
                PllPad::DllIn1,
                PllPad::DllIn2,
                PllPad::DllIn3,
            ]
            .into_iter()
            .enumerate()
            {
                if i >= 4 && hv.v == DirV::N {
                    continue;
                }
                self.add_io_conn(
                    cell.wire(wires::IO_PLL[i]),
                    SpecialIoKey::Pll(pad, PllLoc::new(hv, 0)),
                );
            }
        }
    }

    pub(super) fn fill_serdes_scm(&mut self) {
        let mut col_end_w = self.chip.col_w();
        let mut col_end_e = self.chip.col_e();
        for (col, cd) in &self.chip.columns {
            if col < self.chip.col_clk && cd.io_n == IoGroupKind::Serdes {
                col_end_w = col + 7;
            }
        }
        for (col, cd) in self.chip.columns.iter().rev() {
            if col >= self.chip.col_clk && cd.io_n == IoGroupKind::Serdes {
                col_end_e = col;
            }
        }
        let row = self.chip.row_serdes(DirV::N);
        self.bel_holes.push(
            self.die
                .cell(self.chip.col_w(), row)
                .rect((col_end_w - self.chip.col_w()) as usize, 13),
        );
        self.bel_holes.push(
            self.die
                .cell(col_end_e, row)
                .rect((self.chip.col_e() - col_end_e) as usize + 1, 13),
        );
        for cell in self.egrid.row(self.die, row) {
            if cell.col < col_end_w || cell.col >= col_end_e {
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
            }
            if self.chip.columns[cell.col].io_n == IoGroupKind::Serdes {
                let kind = match cell.col.side_of(self.chip.col_clk) {
                    DirH::W => tcls::SERDES_W,
                    DirH::E => tcls::SERDES_E,
                };
                let mut tcells = cell.cells_e(7);
                tcells.push(cell.with_col(self.chip.col_clk - 1));
                self.egrid.add_tile(cell, kind, &tcells);
            }
        }
    }

    pub(super) fn fill_ebr_scm(&mut self) {
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
                if self.is_in_bel_hole(cell.delta(1, 0)) {
                    continue;
                }
                if cell.col == self.chip.col_w() {
                    self.egrid.add_tile(
                        cell,
                        tcls::MACO_W,
                        &[
                            cell,
                            cell.delta(1, 0),
                            cell.delta(2, 0),
                            cell.delta(3, 0),
                            // IO
                            cell.delta(0, -1),
                            cell.delta(0, 1),
                            // EBR
                            cell.delta(4, 0),
                            cell.delta(6, 0),
                            cell.delta(8, 0),
                            cell.delta(10, 0),
                            cell.delta(12, 0),
                            cell.delta(14, 0),
                            cell.delta(16, 0),
                            cell.delta(18, 0),
                            cell.delta(20, 0),
                            cell.delta(22, 0),
                        ],
                    );
                }
                if cell.col == self.chip.col_e() {
                    self.egrid.add_tile(
                        cell,
                        tcls::MACO_E,
                        &[
                            cell,
                            cell.delta(-1, 0),
                            cell.delta(-2, 0),
                            cell.delta(-3, 0),
                            // IO
                            cell.delta(0, -1),
                            cell.delta(0, 1),
                            // EBR
                            cell.delta(-5, 0),
                            cell.delta(-7, 0),
                            cell.delta(-9, 0),
                            cell.delta(-11, 0),
                            cell.delta(-13, 0),
                            cell.delta(-15, 0),
                            cell.delta(-17, 0),
                            cell.delta(-19, 0),
                            cell.delta(-21, 0),
                            cell.delta(-23, 0),
                        ],
                    );
                }
                if cell.col < self.chip.col_w() + 4 || cell.col >= self.chip.col_e() - 3 {
                    continue;
                }
                if cell.col.to_idx() % 2 == self.chip.col_clk.to_idx() % 2 {
                    let kind = match cell.col.side_of(self.chip.col_clk) {
                        DirH::W => tcls::EBR_W,
                        DirH::E => tcls::EBR_E,
                    };
                    self.egrid.add_tile_e(cell, kind, 2);
                    if cell.row != self.chip.row_serdes(DirV::N) {
                        if cell.col < self.chip.col_w() + 14 {
                            // nothing
                        } else if cell.col < self.chip.col_clk - 10 {
                            let cell_next = cell.delta(10, 0);
                            self.egrid.fill_conn_pair(
                                cell,
                                cell_next,
                                ccls::PASS_EBR_W_E,
                                ccls::PASS_EBR_W_W,
                            );
                        } else if cell.col < self.chip.col_clk {
                            let cell_next = cell.delta(10, 0);
                            self.egrid.fill_conn_pair(
                                cell,
                                cell_next,
                                ccls::PASS_EBR_M_E,
                                ccls::PASS_EBR_M_W,
                            );
                        } else if cell.col < self.chip.col_e() - 23 {
                            let cell_next = cell.delta(10, 0);
                            self.egrid.fill_conn_pair(
                                cell,
                                cell_next,
                                ccls::PASS_EBR_E_E,
                                ccls::PASS_EBR_E_W,
                            );
                        }
                    }
                }
            }
        }
    }

    pub(super) fn fill_io_scm(&mut self) {
        let mut prev = None;
        for cell in self.egrid.column(self.die, self.chip.col_w()).rev() {
            if self.is_in_bel_hole(cell) {
                continue;
            }
            let rd = &self.chip.rows[cell.row];
            if rd.kind == RowKind::Ebr {
                prev = None;
            }
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::INT_IO);
            self.egrid.add_tile_single(cell, tcls::IO_INT_W);
            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::PASS_IO_E, ccls::PASS_IO_W);
            }
            prev = Some(cell);
            match rd.io_w {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_n(cell, tcls::IO_W4, 2);
                }
                IoGroupKind::Dozen => {
                    self.egrid.add_tile_n(cell, tcls::IO_W12, 4);
                }
                IoGroupKind::None => (),
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::INT_IO);
            if cell.col >= self.chip.col_w() + 2 && cell.col < self.chip.col_e() - 1 {
                self.egrid.add_tile_single(cell, tcls::IO_INT_S);
                if let Some(prev) = prev {
                    self.egrid
                        .fill_conn_pair(prev, cell, ccls::PASS_IO_E, ccls::PASS_IO_W);
                }
                prev = Some(cell);
            }
            match self.chip.columns[cell.col].io_s {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_e(cell, tcls::IO_S4, 2);
                }
                IoGroupKind::Dozen => {
                    self.egrid.add_tile_e(cell, tcls::IO_S12, 4);
                }
                IoGroupKind::None => (),
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }
            let rd = &self.chip.rows[cell.row];
            if rd.kind == RowKind::Ebr {
                prev = None;
            }
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::INT_IO);
            self.egrid.add_tile_single(cell, tcls::IO_INT_E);
            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::PASS_IO_E, ccls::PASS_IO_W);
            }
            prev = Some(cell);
            match rd.io_e {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_n(cell, tcls::IO_E4, 2);
                }
                IoGroupKind::Dozen => {
                    self.egrid.add_tile_n(cell, tcls::IO_E12, 4);
                }
                IoGroupKind::None => (),
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::INT_IO);
            match self.chip.columns[cell.col].io_n {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_e(cell, tcls::IO_N4, 2);
                }
                IoGroupKind::Octal => {
                    self.egrid.add_tile_e(cell, tcls::IO_N8, 3);
                }
                IoGroupKind::Dozen => {
                    self.egrid.add_tile_e(cell, tcls::IO_N12, 4);
                }
                IoGroupKind::None | IoGroupKind::Serdes => (),
                _ => unreachable!(),
            }
        }
    }

    pub(super) fn fill_clk_scm(&mut self) {
        for &col_drv in self.chip.cols_pclk.values() {
            for cell in self.egrid.column(self.die, col_drv) {
                if !self.egrid.has_bel(cell.bel(bslots::INT))
                    && !self.egrid.has_bel(cell.delta(-1, 0).bel(bslots::INT))
                {
                    continue;
                }
                let tcid = if self.egrid[cell].region_root[rslots::PCLK_COL] == cell {
                    tcls::PCLK_DRIVER_EBR_ROOT
                } else if self.chip.rows[cell.row].kind == RowKind::Ebr {
                    tcls::PCLK_DRIVER_EBR
                } else {
                    tcls::PCLK_DRIVER
                };
                self.egrid.add_tile_single(cell, tcid);
            }
        }

        for col in [self.chip.col_w(), self.chip.col_e()] {
            for cell in self.egrid.column(self.die, col) {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_row(self.chip.row_clk);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid[cell].region_root[rslots::ECLK] = self.chip.special_loc
                [&SpecialLocKey::Bc(if cell.col < self.chip.col_clk { 5 } else { 4 })];
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk);
        }

        for (bank, kind, plls) in [
            (7, tcls::CLK_W, [DirHV::SW, DirHV::NW].as_slice()),
            (2, tcls::CLK_E, [DirHV::SE, DirHV::NE].as_slice()),
            (1, tcls::CLK_N, [].as_slice()),
            (5, tcls::CLK_SW, [DirHV::SW].as_slice()),
            (4, tcls::CLK_SE, [DirHV::SE].as_slice()),
        ] {
            let cell = self.chip.special_loc[&SpecialLocKey::Bc(bank)];
            let mut tcells = if matches!(bank, 2 | 7) {
                vec![Some(cell.delta(0, -1)), Some(cell)]
            } else {
                vec![Some(cell.delta(-1, 0)), Some(cell)]
            };
            for &hv in plls {
                tcells.push(Some(self.chip.bel_pll(hv, 0).cell));
            }
            let row_serdes = self.chip.row_serdes(DirV::N);
            match kind {
                tcls::CLK_W => {
                    for (col, cd) in &self.chip.columns {
                        if cd.io_n == IoGroupKind::Serdes && col < self.chip.col_clk {
                            tcells.push(Some(self.die.cell(col, row_serdes)));
                        }
                    }
                    while tcells.len() < 8 {
                        tcells.push(None);
                    }
                }
                tcls::CLK_E => {
                    for (col, cd) in self.chip.columns.iter().rev() {
                        if cd.io_n == IoGroupKind::Serdes && col >= self.chip.col_clk {
                            tcells.push(Some(self.die.cell(col, row_serdes)));
                        }
                    }
                    while tcells.len() < 8 {
                        tcells.push(None);
                    }
                }
                _ => (),
            }
            self.egrid.add_tile_part(cell, kind, &tcells);
        }
        let cell = self.chip.clk_emid(Dir::S);
        self.egrid.add_tile(
            cell,
            tcls::CLK_S,
            &[
                cell.delta(-1, 0),
                cell,
                self.chip.special_loc[&SpecialLocKey::Bc(5)],
                self.chip.special_loc[&SpecialLocKey::Bc(4)],
            ],
        );

        for (edge, cell, base) in [
            (Dir::W, self.chip.clk_emid(Dir::W), 0),
            (Dir::E, self.chip.clk_emid(Dir::E), 0),
            (Dir::S, self.chip.special_loc[&SpecialLocKey::Bc(5)], 0),
            (Dir::S, self.chip.special_loc[&SpecialLocKey::Bc(4)], 8),
            (Dir::N, self.chip.clk_emid(Dir::N), 0),
        ] {
            for i in 0..8 {
                self.add_io_conn(
                    cell.wire(wires::IO_GCLK[i]),
                    SpecialIoKey::Clock(edge, base + i as u8),
                );
            }
        }

        let cell = self.chip.clk_root();
        self.egrid.add_tile(
            cell,
            tcls::CLK_ROOT,
            &[
                cell.delta(-1, -5),
                cell.delta(0, -5),
                cell.delta(-1, 4),
                cell.delta(0, 4),
                self.chip.clk_emid(Dir::W),
                self.chip.clk_emid(Dir::E),
                self.chip.clk_emid(Dir::S),
                self.chip.clk_emid(Dir::N),
            ],
        );
    }
}
