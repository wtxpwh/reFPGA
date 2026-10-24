use prjcombine_entity::{EntityId, EntityIdArith};
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColId, DieIdExt, RowId},
};

use crate::{
    chip::{ChipKind, IoGroupKind, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, rslots, tcls},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_config_ecp(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.bel_holes.push(cell.rect(4, 1));
        if cell.row == self.chip.row_clk {
            self.egrid.add_tile_e(cell, tcls::CONFIG_S, 4);
        } else {
            let tcells: [_; 5] = core::array::from_fn(|i| {
                if i < 4 {
                    cell.delta(i as i32, 0)
                } else {
                    cell.with_row(self.chip.row_clk)
                }
            });
            self.egrid.add_tile(cell, tcls::CONFIG_L, &tcells);
        }
    }

    pub(super) fn fill_config_xp(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.egrid.add_tile_single(cell, tcls::CONFIG);
    }

    pub(super) fn fill_pll_ecp(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let tcells = [
                cell,
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
            ];
            match loc.quad.h {
                DirH::W => {
                    self.egrid.add_tile(cell, tcls::PLL_W, &tcells);
                    self.bel_holes.push(cell.rect(5, 1));
                    for i in 1..5 {
                        self.egrid.add_tile_single(cell.delta(i, 0), tcls::INT_PLL);
                    }
                }
                DirH::E => {
                    self.egrid.add_tile(cell, tcls::PLL_E, &tcells);
                    self.bel_holes.push(cell.delta(-4, 0).rect(5, 1));
                    for i in 1..5 {
                        self.egrid.add_tile_single(cell.delta(-i, 0), tcls::INT_PLL);
                    }
                }
            }
        }
    }

    pub(super) fn fill_pll_xp(&mut self) {
        let hole_width: i32 = if self.chip.col_clk.to_idx().is_multiple_of(2) {
            8
        } else {
            7
        };
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let tcells = [
                cell,
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
            ];
            match loc.quad.h {
                DirH::W => {
                    self.egrid.add_tile(cell, tcls::PLL_W, &tcells);
                    self.bel_holes.push(cell.rect(hole_width as usize, 1));
                    for i in 2..hole_width {
                        self.egrid.add_tile_single(cell.delta(i, 0), tcls::INT_PLL);
                    }
                }
                DirH::E => {
                    self.egrid.add_tile(cell, tcls::PLL_E, &tcells);
                    self.bel_holes.push(
                        cell.delta(-(hole_width - 1), 0)
                            .rect(hole_width as usize, 1),
                    );
                    for i in 2..hole_width {
                        self.egrid.add_tile_single(cell.delta(-i, 0), tcls::INT_PLL);
                    }
                }
            }
        }
    }

    pub(super) fn fill_ebr_ecp(&mut self) {
        let ebr_width = match self.chip.kind {
            ChipKind::Ecp | ChipKind::Xp => 2,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_) => 3,
            _ => unreachable!(),
        };
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    let int_kind = if ebr_width == 3 && self.chip.is_col_io(cell.col) {
                        if self.chip.kind.has_distributed_sclk_ecp3() && rd.sclk_break {
                            tcls::INT_EBR_IO_SCLK
                        } else {
                            tcls::INT_EBR_IO
                        }
                    } else {
                        if self.chip.kind.has_distributed_sclk_ecp3() && rd.sclk_break {
                            tcls::INT_EBR_SCLK
                        } else {
                            tcls::INT_EBR
                        }
                    };
                    self.egrid.add_tile_single(cell, int_kind);
                }
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                if self.chip.is_col_io(cell.col) {
                    if matches!(self.chip.kind, ChipKind::Ecp3(_)) && rd.sclk_break {
                        self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
                    }
                    continue;
                }
                if cell.col.to_idx() % ebr_width == self.chip.col_clk.to_idx() % ebr_width {
                    self.egrid.add_tile_e(cell, tcls::EBR, ebr_width);
                }
            }
        }
    }

    pub(super) fn fill_dsp_ecp(&mut self) {
        let dsp_width = match self.chip.kind {
            ChipKind::Ecp => 8,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_) => 9,
            _ => unreachable!(),
        };
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Dsp {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    let int_kind = if dsp_width == 9 && self.chip.is_col_io(cell.col) {
                        if self.chip.kind.has_distributed_sclk_ecp3() && rd.sclk_break {
                            tcls::INT_EBR_IO_SCLK
                        } else {
                            tcls::INT_EBR_IO
                        }
                    } else {
                        if self.chip.kind.has_distributed_sclk_ecp3() && rd.sclk_break {
                            tcls::INT_EBR_SCLK
                        } else {
                            tcls::INT_EBR
                        }
                    };
                    self.egrid.add_tile_single(cell, int_kind);
                }
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                if self.chip.is_col_io(cell.col) {
                    continue;
                }
                if cell.col.to_idx() % dsp_width == 1 {
                    self.egrid.add_tile_e(cell, tcls::DSP, dsp_width);
                }
            }
        }
    }

    pub(super) fn fill_io_ecp(&mut self) {
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }

            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.fill_int_io(cell);
            self.egrid.add_tile_single(cell, tcls::IO_W);
            self.egrid[cell].region_root[rslots::DQSDLL] = self.chip.bel_dqsdll(cell).cell;

            if rd.io_w == IoGroupKind::DoubleDqs {
                self.egrid.add_tile_single(cell, tcls::DQS_W);
                let row_base: RowId = match self.chip.kind {
                    ChipKind::Ecp | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                        cell.row - 3
                    }
                    ChipKind::Xp => cell.row - 2,
                    _ => unreachable!(),
                };
                for row_io in row_base.range(row_base + 8) {
                    self.egrid[cell.with_row(row_io)].region_root[rslots::DQS] = cell;
                }
            }
        }

        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }

            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.fill_int_io(cell);
            self.egrid.add_tile_single(cell, tcls::IO_E);
            self.egrid[cell].region_root[rslots::DQSDLL] = self.chip.bel_dqsdll(cell).cell;

            if rd.io_e == IoGroupKind::DoubleDqs {
                self.egrid.add_tile_single(cell, tcls::DQS_E);
                let row_base: RowId = match self.chip.kind {
                    ChipKind::Ecp | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                        cell.row - 3
                    }
                    ChipKind::Xp => cell.row - 2,
                    _ => unreachable!(),
                };
                for row_io in row_base.range(row_base + 8) {
                    self.egrid[cell.with_row(row_io)].region_root[rslots::DQS] = cell;
                }
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }

            let cd = &self.chip.columns[cell.col];
            self.fill_int_io(cell);
            if self.chip.is_col_io(cell.col) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::IO_S);
            self.egrid[cell].region_root[rslots::DQSDLL] = self.chip.bel_dqsdll(cell).cell;

            if cd.io_s == IoGroupKind::DoubleDqs {
                self.egrid.add_tile_single(cell, tcls::DQS_S);
                let (col_base, num): (ColId, usize) = match self.chip.kind {
                    ChipKind::Ecp => (cell.col - 4, 8),
                    ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => (cell.col - 4, 9),
                    ChipKind::Xp => (cell.col - 5, 8),
                    _ => unreachable!(),
                };
                for col_io in col_base.range(col_base + num) {
                    self.egrid[cell.with_col(col_io)].region_root[rslots::DQS] = cell;
                }
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if self.is_in_bel_hole(cell) {
                continue;
            }

            let cd = &self.chip.columns[cell.col];
            self.fill_int_io(cell);
            if self.chip.is_col_io(cell.col) {
                continue;
            }
            self.egrid.add_tile_single(cell, tcls::IO_N);
            self.egrid[cell].region_root[rslots::DQSDLL] = self.chip.bel_dqsdll(cell).cell;

            if cd.io_n == IoGroupKind::DoubleDqs {
                self.egrid.add_tile_single(cell, tcls::DQS_N);
                let (col_base, num): (ColId, usize) = match self.chip.kind {
                    ChipKind::Ecp => (cell.col - 4, 8),
                    ChipKind::Xp2 => (cell.col - 4, 9),
                    ChipKind::Xp => (cell.col - 5, 8),
                    _ => unreachable!(),
                };
                for col_io in col_base.range(col_base + num) {
                    self.egrid[cell.with_col(col_io)].region_root[rslots::DQS] = cell;
                }
            }
        }
        if matches!(self.chip.kind, ChipKind::Ecp | ChipKind::Xp) {
            let cell = self.chip.bel_dqsdll_ecp(DirV::S).cell;
            self.egrid.add_tile_single(cell, tcls::DQSDLL_S);
            let cell = self.chip.bel_dqsdll_ecp(DirV::N).cell;
            self.egrid.add_tile_single(cell, tcls::DQSDLL_N);
        }
        if self.chip.kind == ChipKind::Xp2 {
            let cell = self.chip.clk_emid(DirH::W);
            self.egrid.add_tile_part(
                cell,
                tcls::CLK_W,
                &[
                    Some(cell),
                    self.chip.try_bel_pll(DirHV::SW, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                ],
            );
            let cell = self.chip.clk_emid(DirH::E);
            self.egrid.add_tile_part(
                cell,
                tcls::CLK_E,
                &[
                    Some(cell),
                    self.chip.try_bel_pll(DirHV::SE, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_bel_pll(DirHV::NE, 0).map(|bcrd| bcrd.cell),
                    Some(cell.with_col(self.chip.col_clk + 2)),
                ],
            );
        }
    }

    pub(super) fn fill_clk_ecp(&mut self) {
        let ebr_rows = Vec::from_iter(
            self.chip
                .rows
                .ids()
                .filter(|&row| self.chip.rows[row].kind == RowKind::Ebr),
        );
        let mut cells = vec![
            // actual clock root cells
            Some(self.die.cell(self.chip.col_w(), self.chip.row_s())),
            Some(self.die.cell(self.chip.col_e(), self.chip.row_s())),
            Some(self.die.cell(self.chip.col_w(), self.chip.row_n())),
            Some(self.die.cell(self.chip.col_e(), self.chip.row_n())),
        ];
        // fabric clock inputs
        for edge in Dir::DIRS {
            cells.push(self.chip.try_get_spec_cell(SpecialLocKey::PclkIn(edge, 0)));
        }
        for edge in Dir::DIRS {
            for i in 0..4 {
                cells.push(self.chip.try_get_spec_cell(SpecialLocKey::SclkIn(edge, i)));
            }
        }
        // IO inputs
        for edge in Dir::DIRS {
            cells.push(self.chip.try_get_io_cell(SpecialIoKey::Clock(edge, 0)));
        }
        // PLLs & DCS select inputs
        match self.chip.kind {
            ChipKind::Ecp if ebr_rows.len() == 1 => {
                cells.extend([
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[0])),
                ]);
            }
            ChipKind::Ecp if ebr_rows.len() == 2 => {
                cells.extend([
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_w() + 5, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w() + 6, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e() - 6, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e() - 5, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w() + 5, ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_w() + 6, ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_e() - 6, ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_e() - 5, ebr_rows[1])),
                ]);
            }
            ChipKind::Xp if ebr_rows.len() == 1 => {
                cells.extend([
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w() + 1, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e() - 1, ebr_rows[0])),
                ]);
            }
            ChipKind::Xp if ebr_rows.len() == 2 => {
                cells.extend([
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w() + 1, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_e() - 1, ebr_rows[0])),
                    Some(self.die.cell(self.chip.col_w(), ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_w() + 1, ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_e(), ebr_rows[1])),
                    Some(self.die.cell(self.chip.col_e() - 1, ebr_rows[1])),
                ]);
            }
            _ => unreachable!(),
        }
        let kind = match (self.chip.kind, ebr_rows.len()) {
            (ChipKind::Ecp, 1) => tcls::CLK_ROOT_2PLL,
            (ChipKind::Xp, 1) => {
                if self.chip.special_loc[&SpecialLocKey::SclkIn(Dir::E, 2)].col == self.chip.col_clk
                {
                    tcls::CLK_ROOT_2PLL_A
                } else {
                    tcls::CLK_ROOT_2PLL_B
                }
            }
            (_, 2) => tcls::CLK_ROOT_4PLL,
            _ => unreachable!(),
        };
        self.egrid.add_tile_part(self.chip.clk_root(), kind, &cells);
    }

    pub(super) fn fill_bs_ecp(&mut self) {
        self.row_bit = self.chip.rows.map_values(|_| 0);
        for row in self.chip.rows.ids().rev() {
            self.row_bit[row] = self.frame_len;
            self.frame_len += self.chip.btile_height(row);
            if self.chip.rows[row].kind == RowKind::Ebr {
                self.row_ebr_bit.insert(row, self.frame_len);
                self.frame_len += 144;
            }
        }
        self.frames_num += self.chip.extra_frames_w;
        for col in self.chip.columns.ids() {
            if col == self.chip.col_w() {
                self.col_term_frame.insert(col, self.frames_num);
                self.frames_num += self.chip.btile_term_width(col);
            }
            if col == self.chip.col_clk {
                self.clk_frame = self.frames_num;
                self.frames_num += self.chip.btile_clk_width();
            }
            self.col_frame.push(self.frames_num);
            self.frames_num += self.chip.btile_width(col);
            if col == self.chip.col_e() {
                self.col_term_frame.insert(col, self.frames_num);
                self.frames_num += self.chip.btile_term_width(col);
            }
        }
        self.frames_num += self.chip.extra_frames_e;
    }
}
