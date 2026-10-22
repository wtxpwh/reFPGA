use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::DieIdExt,
};

use crate::{
    chip::{ChipKind, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, ccls, rslots, tcls},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_config_ecp2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.bel_holes.push(cell.rect(6, 1));
        self.egrid.add_tile(
            cell,
            tcls::CONFIG,
            &[
                cell,
                cell.delta(1, 0),
                cell.delta(2, 0).with_row(self.chip.row_clk),
            ],
        );
    }

    pub(super) fn fill_config_xp2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.egrid.add_tile_single(cell, tcls::OSC);
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.egrid.add_tile_single(cell, tcls::CONFIG);
    }

    pub(super) fn fill_pll_ecp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            if loc.quad.v == DirV::S && loc.idx == 0 {
                match loc.quad.h {
                    DirH::W => {
                        self.bel_holes.push(cell.rect(7, 1));
                        let tcells = [
                            cell,
                            cell.delta(1, 0),
                            cell.delta(2, 0),
                            cell.delta(3, 0),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn1, loc)),
                            self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
                            self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::DllFb, loc)),
                        ];
                        self.egrid.add_tile(cell, tcls::PLL_W, &tcells);
                    }
                    DirH::E => {
                        self.bel_holes.push(cell.delta(-6, 0).rect(7, 1));
                        let tcells = [
                            cell,
                            cell.delta(-1, 0),
                            cell.delta(-2, 0),
                            cell.delta(-3, 0),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn1, loc)),
                            self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
                            self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::DllFb, loc)),
                        ];
                        self.egrid.add_tile(cell, tcls::PLL_E, &tcells);
                    }
                }
            } else {
                self.bel_holes.push(cell.rect(1, 1));
                let tcells = [
                    cell,
                    cell.delta(0, 1),
                    self.chip
                        .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                    self.chip
                        .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn1, loc)),
                    self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
                ];
                self.egrid.add_tile(
                    cell,
                    match loc.quad.h {
                        DirH::W => tcls::SPLL_W,
                        DirH::E => tcls::SPLL_E,
                    },
                    &tcells,
                );
            }
        }
    }

    pub(super) fn fill_pll_xp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let kind = match loc.quad.v {
                DirV::S => tcls::PLL_S,
                DirV::N => tcls::PLL_N,
            };
            let dx = match loc.quad.h {
                DirH::W => 1,
                DirH::E => -1,
            };
            let tcells = [
                cell,
                cell.delta(dx, 0),
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
            ];
            self.egrid.add_tile(cell, kind, &tcells);
        }
    }

    pub(super) fn fill_serdes_ecp2(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.chip.columns[cell.col].io_s == IoGroupKind::Serdes {
                if cell.col < self.chip.col_clk {
                    self.bel_holes.push(cell.delta(-1, 0).rect(28, 8));
                    self.egrid
                        .add_tile_single(cell.delta(-1, 7), tcls::INT_IO_S);
                } else {
                    self.bel_holes.push(cell.rect(28, 8));
                    self.egrid
                        .add_tile_single(cell.delta(27, 7), tcls::INT_IO_S);
                }
                for dx in 0..27 {
                    self.egrid
                        .add_tile_single(cell.delta(dx, 7), tcls::INT_IO_S);
                }
                self.egrid.add_tile_e(cell.delta(0, 7), tcls::SERDES_S, 27);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if self.chip.columns[cell.col].io_n == IoGroupKind::Serdes {
                if cell.col < self.chip.col_clk {
                    self.bel_holes.push(cell.delta(-1, -7).rect(28, 8));
                    self.egrid
                        .add_tile_single(cell.delta(-1, -7), tcls::INT_SERDES_N);
                } else {
                    self.bel_holes.push(cell.delta(0, -7).rect(28, 8));
                    self.egrid
                        .add_tile_single(cell.delta(27, -7), tcls::INT_SERDES_N);
                }
                for dx in 0..27 {
                    self.egrid
                        .add_tile_single(cell.delta(dx, -7), tcls::INT_SERDES_N);
                }
                self.egrid.add_tile_e(cell.delta(0, -7), tcls::SERDES_N, 27);
            }
        }
    }

    pub(super) fn fill_clk_ecp2(&mut self) {
        for &col_drv in self.chip.cols_pclk.values() {
            for cell in self.egrid.column(self.die, col_drv) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::PCLK_DRIVER);
            }
        }

        for cell in self.egrid.die_cells(self.die) {
            let idx = self.chip.col_sclk_idx(cell.col);
            self.egrid.fill_conn_term(
                cell,
                [ccls::RCLK0, ccls::RCLK1, ccls::RCLK2, ccls::RCLK3][idx],
            );
            if self.egrid.has_bel(cell.bel(bslots::INT)) {
                self.egrid.add_tile_single(cell, tcls::RSCLK_DRIVER);
            }
        }

        for i in 0..4 {
            for cell in self.egrid.die_cells(self.die) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    continue;
                }
                let (dx, dx_alt) = match (cell.col.to_idx() + i) % 4 {
                    0 => (0, 0),
                    1 => (-1, 3),
                    2 => (-2, 2),
                    3 => (1, -3),
                    _ => unreachable!(),
                };
                let cell_src = if let Some(cell_src) = self.egrid.cell_delta(cell, dx, 0)
                    && self.egrid.has_bel(cell_src.bel(bslots::INT))
                {
                    cell_src
                } else {
                    cell.delta(dx_alt, 0)
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

        for edge in Dir::DIRS {
            let bcrd = self.chip.bel_eclk_root(edge);
            match edge {
                Dir::H(h) => {
                    let tcid = match h {
                        DirH::W => tcls::ECLK_ROOT_W,
                        DirH::E => tcls::ECLK_ROOT_E,
                    };
                    if self.chip.kind != ChipKind::Xp2 {
                        let tcells = [
                            bcrd.cell,
                            self.chip.get_io_cell(SpecialIoKey::Clock(edge, 0)),
                            self.chip.get_io_cell(SpecialIoKey::Clock(edge, 1)),
                            self.chip.bel_pll(h & DirV::S, 0).cell,
                        ];
                        self.egrid.add_tile(bcrd.cell, tcid, &tcells);
                    } else {
                        let pll_s = PllLoc::new(h & DirV::S, 0);
                        let pll_n = PllLoc::new(h & DirV::N, 0);
                        let tcells = [
                            Some(bcrd.cell),
                            Some(self.chip.get_io_cell(SpecialIoKey::Clock(edge, 0))),
                            Some(self.chip.get_io_cell(SpecialIoKey::Clock(edge, 1))),
                            self.chip.try_bel_pll(h & DirV::S, 0).map(|b| b.cell),
                            self.chip.try_bel_pll(h & DirV::N, 0).map(|b| b.cell),
                            self.chip
                                .try_get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, pll_s)),
                            self.chip
                                .try_get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, pll_n)),
                        ];
                        self.egrid.add_tile_part(bcrd.cell, tcid, &tcells);
                    }
                }
                Dir::V(v) => {
                    let tcid = match v {
                        DirV::S => tcls::ECLK_ROOT_S,
                        DirV::N => tcls::ECLK_ROOT_N,
                    };
                    let tcells = [
                        bcrd.cell,
                        bcrd.cell.delta(-1, 0),
                        self.chip.get_io_cell(SpecialIoKey::Clock(edge, 0)),
                        self.chip.get_io_cell(SpecialIoKey::Clock(edge, 1)),
                    ];
                    self.egrid.add_tile(bcrd.cell, tcid, &tcells);
                }
            }
        }

        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if cell.col != self.chip.col_w() && cell.col != self.chip.col_e() {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk);
            }
            if self.chip.columns[cell.col].eclk_tap_s {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if cell.col != self.chip.col_w() && cell.col != self.chip.col_e() {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk);
            }
            if self.chip.columns[cell.col].eclk_tap_n {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            self.egrid[cell].region_root[rslots::ECLK] = cell.with_row(self.chip.row_clk);
            if matches!(self.chip.rows[cell.row].kind, RowKind::Ebr | RowKind::Dsp)
                && cell.row != self.chip.row_clk
                && !(self.chip.kind == ChipKind::Ecp2
                    && self.chip.rows[cell.row].kind == RowKind::Ebr
                    && cell.row > self.chip.row_clk)
            {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            self.egrid[cell].region_root[rslots::ECLK] = cell.with_row(self.chip.row_clk);
            if matches!(self.chip.rows[cell.row].kind, RowKind::Ebr | RowKind::Dsp)
                && cell.row != self.chip.row_clk
                && !(self.chip.kind == ChipKind::Ecp2
                    && self.chip.rows[cell.row].kind == RowKind::Ebr
                    && cell.row > self.chip.row_clk)
            {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }

        let mut cells = vec![
            // actual clock root cells
            Some(self.die.cell(self.chip.col_w(), self.chip.row_s())),
            Some(self.die.cell(self.chip.col_e(), self.chip.row_s())),
            Some(self.die.cell(self.chip.col_w(), self.chip.row_n())),
            Some(self.die.cell(self.chip.col_e(), self.chip.row_n())),
        ];
        for cell in &mut cells {
            let cell = cell.as_mut().unwrap();
            if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                if cell.row == self.chip.row_s() {
                    cell.row += 7;
                } else if cell.row == self.chip.row_n() {
                    cell.row -= 7;
                } else {
                    unreachable!();
                }
            }
        }
        // fabric clock inputs
        for edge in Dir::DIRS {
            let num = if edge == Dir::E { 4 } else { 2 };
            for i in 0..num {
                cells.push(self.chip.try_get_spec_cell(SpecialLocKey::PclkIn(edge, i)));
            }
        }
        for edge in Dir::DIRS {
            for i in 0..4 {
                cells.push(self.chip.try_get_spec_cell(SpecialLocKey::SclkIn(edge, i)));
            }
        }
        // IO inputs
        for edge in Dir::DIRS {
            for i in 0..2 {
                cells.push(self.chip.try_get_io_cell(SpecialIoKey::Clock(edge, i)));
            }
        }
        // PLL inputs
        for hv in [DirHV::SW, DirHV::SE, DirHV::NW, DirHV::NE] {
            let num = if self.chip.kind == ChipKind::Xp2
                || (self.chip.kind == ChipKind::Ecp2 && hv.v == DirV::S)
            {
                1
            } else {
                2
            };
            for i in 0..num {
                cells.push(self.chip.try_bel_pll(hv, i).map(|bcrd| bcrd.cell));
            }
        }
        // CLKDIV inputs
        if self.chip.kind == ChipKind::Xp2 {
            for edge in [DirH::W, DirH::E] {
                cells.push(Some(
                    self.die.cell(self.chip.col_edge(edge), self.chip.row_clk),
                ));
            }
        }
        // SERDES inputs
        if self.chip.kind == ChipKind::Ecp2M {
            for hv in [DirHV::SW, DirHV::SE, DirHV::NW, DirHV::NE] {
                cells.push(self.chip.try_bel_serdes_ecp2(hv).map(|bcrd| bcrd.cell));
            }
        }

        self.egrid.add_tile_part(
            self.die.cell(self.chip.col_clk, self.chip.row_clk),
            tcls::CLK_ROOT,
            &cells,
        );
    }
}
