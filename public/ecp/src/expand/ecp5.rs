use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::DieIdExt,
};

use crate::{
    chip::{Chip, IoGroupKind, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{rslots, tcls, wires},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_pll_ecp5(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let kind = match loc.quad {
                DirHV::SW => tcls::PLL_SW,
                DirHV::SE => tcls::PLL_SE,
                DirHV::NW => tcls::PLL_NW,
                DirHV::NE => tcls::PLL_NE,
            };
            match loc.quad.v {
                DirV::S => {
                    self.egrid.add_tile(
                        cell,
                        kind,
                        &[
                            cell,
                            cell.delta(
                                match loc.quad.h {
                                    DirH::W => 1,
                                    DirH::E => -1,
                                },
                                0,
                            ),
                            self.chip
                                .get_io_cell(SpecialIoKey::Clock(Dir::H(loc.quad.h), 0)),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                        ],
                    );
                }
                DirV::N => {
                    self.egrid.add_tile(
                        cell,
                        kind,
                        &[
                            cell,
                            cell.delta(0, -1),
                            self.chip
                                .get_io_cell(SpecialIoKey::Clock(Dir::H(loc.quad.h), 2)),
                            self.chip.get_io_cell(SpecialIoKey::Clock(
                                Dir::N,
                                match loc.quad.h {
                                    DirH::W => 0,
                                    DirH::E => 2,
                                },
                            )),
                            self.chip.get_io_cell(SpecialIoKey::Clock(
                                Dir::N,
                                match loc.quad.h {
                                    DirH::W => 1,
                                    DirH::E => 3,
                                },
                            )),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                            self.chip
                                .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn1, loc)),
                        ],
                    );
                }
            }
        }
    }

    pub(super) fn fill_io_ecp5(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            self.egrid.add_tile_single(
                cell,
                match bank {
                    0 => tcls::BC0,
                    1 => tcls::BC1,
                    2 => tcls::BC2,
                    3 => tcls::BC3,
                    4 => tcls::BC4,
                    6 => tcls::BC6,
                    7 => tcls::BC7,
                    8 => tcls::BC8,
                    _ => unreachable!(),
                },
            );
        }
        for hv in DirHV::DIRS {
            let cell = self.chip.special_loc[&SpecialLocKey::DdrDll(hv)];
            self.egrid.add_tile_single(cell, tcls::DDRDLL);
            self.egrid[cell].region_root[rslots::ECLK] = Chip::DIE.cell(
                self.chip.col_edge(hv.h),
                match hv.v {
                    DirV::S => self.chip.row_clk - 1,
                    DirV::N => self.chip.row_clk,
                },
            );
            self.egrid.extra_conns.insert(
                cell.wire(wires::OUT_DDRDLL_DDRDEL),
                self.chip
                    .clk_root()
                    .with_col(self.chip.col_edge(hv.h))
                    .wire(match hv.v {
                        DirV::S => wires::DDRDEL_S,
                        DirV::N => wires::DDRDEL_N,
                    }),
            );
        }

        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            match self.chip.columns[cell.col].io_s {
                IoGroupKind::None => (),
                IoGroupKind::Single => {
                    self.egrid.add_tile_single(cell, tcls::IO_S1);
                }
                IoGroupKind::Double => {
                    self.egrid.add_tile_e(cell, tcls::IO_S2, 2);
                }
                IoGroupKind::Serdes => {
                    self.egrid.add_tile_e(cell, tcls::SERDES, 12);
                }
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            match self.chip.columns[cell.col].io_n {
                IoGroupKind::None => (),
                IoGroupKind::Double => {
                    self.egrid.add_tile_e(cell, tcls::IO_N2, 2);
                }
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            match self.chip.rows[cell.row].io_w {
                IoGroupKind::None => (),
                IoGroupKind::Quad => {
                    self.egrid.add_tile_n(cell, tcls::IO_W4, 3);
                }
                IoGroupKind::QuadDqs => {
                    self.egrid.add_tile_n(cell, tcls::IO_W4, 3);
                    self.egrid.add_tile_n(cell, tcls::DQS_W, 6);
                    for dy in [-3, 0, 3, 6] {
                        self.egrid[cell.delta(0, dy)].region_root[rslots::DQS] = cell;
                    }
                }
                _ => unreachable!(),
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            match self.chip.rows[cell.row].io_e {
                IoGroupKind::None => (),
                IoGroupKind::Quad => {
                    self.egrid.add_tile_n(cell, tcls::IO_E4, 3);
                }
                IoGroupKind::QuadDqs => {
                    self.egrid.add_tile_n(cell, tcls::IO_E4, 3);
                    self.egrid.add_tile_n(cell, tcls::DQS_E, 6);
                    for dy in [-3, 0, 3, 6] {
                        self.egrid[cell.delta(0, dy)].region_root[rslots::DQS] = cell;
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub(super) fn fill_pclk_ecp5(&mut self) {
        for (cols, &col_drv) in &self.chip.cols_pclk {
            for cell in self.egrid.column(self.die, col_drv) {
                self.egrid.add_tile_part(
                    cell,
                    tcls::PCLK_DRIVER,
                    &[
                        if cols.start() == col_drv {
                            None
                        } else {
                            Some(cell.delta(-1, 0))
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

    pub(super) fn fill_clk_ecp5(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            self.egrid[cell].region_root[rslots::GLOBAL] = self.chip.clk_root();
        }
        for col in [self.chip.col_w(), self.chip.col_e()] {
            for cell in self.egrid.column(self.die, col) {
                self.egrid[cell].region_root[rslots::ECLK] =
                    cell.with_row(if cell.row < self.chip.row_clk {
                        self.chip.row_clk - 1
                    } else {
                        self.chip.row_clk
                    });
                self.egrid[cell].region_root[rslots::DDRDLL] = cell.with_row(self.chip.row_clk);
            }
        }

        for (edge, dx, tcls) in [(DirH::W, 1, tcls::CLK_W), (DirH::E, -1, tcls::CLK_E)] {
            let cell = self.die.cell(self.chip.col_edge(edge), self.chip.row_clk);
            let mut tcells = cell.delta(0, -6).cells_n(12);
            tcells.extend([cell.delta(dx, 0), cell.delta(dx * 2, 0)]);
            let mut tcells = Vec::from_iter(tcells.into_iter().map(Some));
            for i in 4..8 {
                tcells.push(
                    self.chip
                        .special_loc
                        .get(&SpecialLocKey::PclkIn(edge.into(), i))
                        .copied(),
                );
            }
            tcells.extend([
                self.chip
                    .try_bel_pll(edge & DirV::S, 0)
                    .map(|bcrd| bcrd.cell),
                self.chip
                    .try_bel_pll(edge & DirV::N, 0)
                    .map(|bcrd| bcrd.cell),
            ]);
            tcells.push(Some(self.chip.clk_root()));
            self.egrid.add_tile_part(cell, tcls, &tcells);
        }
        for (edge, tcls) in [(DirV::S, tcls::CLK_S), (DirV::N, tcls::CLK_N)] {
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_edge(edge));
            let tcells = cell.delta(-8, 0).cells_e(16);
            let mut tcells = Vec::from_iter(tcells.into_iter().map(Some));
            for i in 4..8 {
                tcells.push(
                    self.chip
                        .special_loc
                        .get(&SpecialLocKey::PclkIn(edge.into(), i))
                        .copied(),
                );
            }

            if edge == DirV::N {
                for hv in [DirHV::NW, DirHV::NE] {
                    tcells.push(Some(self.chip.special_loc[&SpecialLocKey::DdrDll(hv)]));
                }
                for hv in [DirHV::NW, DirHV::NE] {
                    tcells.push(self.chip.try_bel_pll(hv, 0).map(|bcrd| bcrd.cell));
                }
            }
            if edge == DirV::S {
                let serdes = Vec::from_iter(
                    self.egrid
                        .row(self.die, self.chip.row_s())
                        .filter(|&cell| self.chip.columns[cell.col].io_s == IoGroupKind::Serdes),
                );
                tcells.extend([
                    serdes.get(0).copied(),
                    serdes.get(1).copied(),
                    Some(self.chip.special_loc[&SpecialLocKey::Config]),
                ]);
            }

            self.egrid.add_tile_part(cell, tcls, &tcells);
        }

        let cell = self.chip.bel_clk_root().cell;
        let cell_s = self.chip.special_loc[&SpecialLocKey::ClkQuarter(DirV::S)];
        let cell_n = self.chip.special_loc[&SpecialLocKey::ClkQuarter(DirV::N)];
        let row_s = self
            .chip
            .rows
            .ids()
            .rev()
            .filter(|&row| row < self.chip.row_clk)
            .find(|&row| matches!(self.chip.rows[row].kind, RowKind::Ebr | RowKind::Dsp))
            .unwrap();
        let row_n = self
            .chip
            .rows
            .ids()
            .filter(|&row| row > self.chip.row_clk)
            .find(|&row| matches!(self.chip.rows[row].kind, RowKind::Ebr | RowKind::Dsp))
            .unwrap();

        if row_s == cell_s.row {
            self.egrid.add_tile(
                cell,
                tcls::CLK_ROOT_S,
                &[
                    cell_s.delta(-1, 0),
                    cell_s,
                    cell_n.delta(-1, 0),
                    cell_n,
                    self.chip.clk_root(),
                    self.chip.clk_root().with_col(self.chip.col_w()),
                    self.chip.clk_root().with_col(self.chip.col_e()),
                    self.chip.clk_root().with_row(self.chip.row_s()),
                    self.chip.clk_root().with_row(self.chip.row_n()),
                ],
            );
        } else {
            self.egrid.add_tile(
                cell,
                tcls::CLK_ROOT_L,
                &[
                    cell_s.delta(-1, 0),
                    cell_s,
                    cell.with_row(row_s).delta(-1, 0),
                    cell.with_row(row_s),
                    cell.with_row(row_n).delta(-1, 0),
                    cell.with_row(row_n),
                    cell_n.delta(-1, 0),
                    cell_n,
                    self.chip.clk_root(),
                    self.chip.clk_root().with_col(self.chip.col_w()),
                    self.chip.clk_root().with_col(self.chip.col_e()),
                    self.chip.clk_root().with_row(self.chip.row_s()),
                    self.chip.clk_root().with_row(self.chip.row_n()),
                ],
            );
        }
    }

    pub(super) fn fill_config_ecp5(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut tcells = cell.cells_e(3);
        tcells.extend([cell.with_cr(self.chip.col_clk, self.chip.row_clk)]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);
        self.egrid.add_tile_single(cell.delta(18, 0), tcls::DTR);
    }
}
