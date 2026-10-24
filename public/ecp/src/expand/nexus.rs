use std::collections::BTreeMap;

use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::DieIdExt,
};

use crate::{
    chip::{ChipKind, IoGroupKind, NexusKind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{rslots, tcls, wires},
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

    pub(super) fn fill_pll_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            match (loc.quad, kind) {
                (DirHV::SW, NexusKind::SmallMipi) => {
                    self.egrid.add_tile_e(cell, tcls::PLL_SW_JE5D15, 3);
                }
                (DirHV::SE, NexusKind::SmallMipi) => {
                    self.egrid.add_tile_n(cell, tcls::PLL_SE_JE5D15, 3);
                }
                (DirHV::SW, NexusKind::SmallMipiPcie) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::PLL_SW_JE5D30,
                        &[cell, cell.delta(0, 1), cell.delta(1, 0)],
                    );
                }
                (
                    DirHV::SE,
                    NexusKind::SmallMipiPcie | NexusKind::LargePcs | NexusKind::SmallDenseIo,
                ) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::PLL_SE_JE5D30,
                        &[cell, cell.delta(0, 1), cell.delta(0, -1)],
                    );
                }
                (DirHV::NW, NexusKind::SmallMipiPcie | NexusKind::LargePcs) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::PLL_NW_JE5D30,
                        &[cell, cell.delta(0, -1), cell.delta(1, 0)],
                    );
                }
                (DirHV::SW, NexusKind::LargePcs | NexusKind::SmallHalfIo) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::PLL_SW_JD5D,
                        &[cell, cell.delta(0, -1), cell.delta(1, -1)],
                    );
                }
                (DirHV::NE, NexusKind::LargePcs) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::PLL_NE_JD5D,
                        &[cell, cell.delta(0, -1), cell.delta(-1, 0)],
                    );
                }
                (DirHV::NW, NexusKind::SmallDenseIo) => {
                    self.egrid.add_tile_e(cell, tcls::PLL_NW_JD5F, 3);
                }
                _ => unreachable!(),
            }
        }
    }

    pub(super) fn fill_io_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let mut bank_cells = BTreeMap::new();
        let mut io_cells = vec![];
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            if let Some(bank) = self.chip.columns[cell.col].bank_s {
                self.egrid[cell].region_root[rslots::ECLK] =
                    *bank_cells.entry(bank).or_insert(cell);
                io_cells.push(cell);
                self.egrid.add_tile_e(
                    cell,
                    match kind {
                        NexusKind::SmallHalfIo => tcls::IO_HP_S_JD5U,
                        NexusKind::LargePcs => tcls::IO_HP_S_LPDDR4,
                        _ => tcls::IO_HP_S,
                    },
                    2,
                );
            }
            if self.chip.columns[cell.col].io_s == IoGroupKind::DoubleDqs {
                let (cells, tcid) = match kind {
                    NexusKind::SmallMipi => (cell.delta(-6, 0).cells_e(16), tcls::DQS_S),
                    NexusKind::SmallMipiPcie => (cell.delta(-6, 0).cells_e(16), tcls::DQS_L),
                    NexusKind::LargePcs => {
                        if cell.col == self.chip.col_clk - 8 {
                            (cell.delta(-4, 0).cells_e(24), tcls::DQS_JD5D_HOLE)
                        } else {
                            (cell.delta(-4, 0).cells_e(12), tcls::DQS_JD5D)
                        }
                    }
                    NexusKind::SmallDenseIo => {
                        if cell.col == self.chip.col_clk - 8 {
                            (cell.delta(-4, 0).cells_e(24), tcls::DQS_JD5F_HOLE)
                        } else {
                            (cell.delta(-4, 0).cells_e(12), tcls::DQS_JD5F)
                        }
                    }
                    _ => unreachable!(),
                };
                for &cell_io in &cells {
                    self.egrid[cell_io].region_root[rslots::DQS] = cell;
                }
                self.egrid.add_tile(cell, tcid, &cells);
            }
        }
        for (i, &cell_dqs) in io_cells.iter().enumerate() {
            if self.chip.columns[cell_dqs.col].io_s != IoGroupKind::DoubleDqs {
                continue;
            }
            let cells = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie => &io_cells[(i - 3)..(i + 5)],
                NexusKind::LargePcs | NexusKind::SmallDenseIo => &io_cells[(i - 2)..(i + 4)],
                NexusKind::SmallHalfIo => unreachable!(),
            };
            for &cell in cells {
                self.egrid[cell].region_root[rslots::DQS] = cell_dqs;
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            let io_kind = self.chip.columns[cell.col].io_n;
            let tcid = match io_kind {
                IoGroupKind::None | IoGroupKind::Mipi | IoGroupKind::Serdes => continue,
                IoGroupKind::DoubleA => tcls::IO_N_A,
                IoGroupKind::DoubleB => tcls::IO_N_B,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, tcid);
        }
        for edge in DirH::DIRS {
            for cell in self.egrid.column(self.die, self.chip.col_edge(edge)) {
                if cell.row == self.chip.row_s() || cell.row == self.chip.row_n() {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
                let io_kind = match edge {
                    DirH::W => self.chip.rows[cell.row].io_w,
                    DirH::E => self.chip.rows[cell.row].io_e,
                };
                let tcid = match (edge, io_kind) {
                    (_, IoGroupKind::None) => continue,
                    (DirH::W, IoGroupKind::DoubleA) => tcls::IO_W_A,
                    (DirH::W, IoGroupKind::DoubleB) => tcls::IO_W_B,
                    (DirH::E, IoGroupKind::DoubleA) => tcls::IO_E_A,
                    (DirH::E, IoGroupKind::DoubleB) => tcls::IO_E_B,
                    (DirH::E, IoGroupKind::DoubleC) => tcls::IO_E_C,
                    _ => unreachable!(),
                };
                self.egrid.add_tile_single(cell, tcid);
            }
        }
        for edge in DirH::DIRS {
            let cell = self.chip.special_loc[&SpecialLocKey::DdrDll(edge & DirV::S)];
            let tcid = if cell.col != self.chip.col_edge(edge) {
                tcls::DDRDLL_S
            } else {
                match kind {
                    NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs => {
                        match edge {
                            DirH::W => tcls::DDRDLL_W,
                            DirH::E => tcls::DDRDLL_E,
                        }
                    }
                    NexusKind::SmallHalfIo | NexusKind::SmallDenseIo => match edge {
                        DirH::W => tcls::DDRDLL_W_JD5U,
                        DirH::E => tcls::DDRDLL_E_JD5U,
                    },
                }
            };
            self.egrid.add_tile_single(cell, tcid);
        }
        for (&loc, &cell) in &self.chip.special_loc {
            let SpecialLocKey::DllDel(edge, idx) = loc else {
                continue;
            };
            let tcid = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie => match edge {
                    Dir::W => tcls::DLLDEL_W_A,
                    Dir::E => tcls::DLLDEL_E_A,
                    Dir::S => tcls::DLLDEL_S_A,
                    Dir::N => unreachable!(),
                },
                NexusKind::LargePcs | NexusKind::SmallHalfIo | NexusKind::SmallDenseIo => {
                    match edge {
                        Dir::W => tcls::DLLDEL_W_B,
                        Dir::E => tcls::DLLDEL_E_B,
                        Dir::S => tcls::DLLDEL_S_B,
                        Dir::N => tcls::DLLDEL_N_B,
                    }
                }
            };
            self.egrid.add_tile_single(cell, tcid);
            self.add_io_conn(cell.wire(wires::IO_DLLDEL), SpecialIoKey::Clock(edge, idx));
        }
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::Cdr) {
            match kind {
                NexusKind::SmallMipi => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_S,
                        &[
                            cell,
                            cell.delta(1, 0),
                            cell.delta(2, 0),
                            self.chip.bel_pll(DirHV::SW, 0).cell,
                        ],
                    );
                }
                NexusKind::SmallMipiPcie | NexusKind::LargePcs => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JE5D30,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            self.chip.bel_pll(DirHV::SW, 0).cell,
                        ],
                    );
                }
                NexusKind::SmallHalfIo => unreachable!(),
                NexusKind::SmallDenseIo => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JD5F,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            cell.delta(0, 3),
                            self.chip.bel_pll(DirHV::NW, 0).cell,
                        ],
                    );
                }
            }
        }
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::Adc) {
            let (tcid, cell_comp) = match kind {
                NexusKind::SmallMipi => (tcls::ADC_JE5D15, cell.delta(0, -2)),
                NexusKind::SmallMipiPcie => (tcls::ADC_JE5D30, cell.delta(0, 2)),
                NexusKind::LargePcs | NexusKind::SmallDenseIo => {
                    (tcls::ADC_JD5D, cell.delta(0, -2))
                }
                NexusKind::SmallHalfIo => unreachable!(),
            };
            self.egrid.add_tile(
                cell,
                tcid,
                &[
                    cell.delta(0, -1),
                    cell,
                    cell_comp,
                    self.chip.bel_pll(DirHV::SE, 0).cell,
                ],
            );
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

    pub(super) fn fill_clk_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let cell_s = self.die.cell(self.chip.col_clk, self.chip.row_s());
        let mut tcells = vec![];
        tcells.extend(cell_s.delta(-6, 0).cells_e(12).into_iter().map(Some));
        tcells.extend([
            self.chip.try_bel_pll(DirHV::SW, 0).map(|bcrd| bcrd.cell),
            self.chip.try_bel_pll(DirHV::SE, 0).map(|bcrd| bcrd.cell),
            Some(self.chip.special_loc[&SpecialLocKey::DdrDll(DirHV::SW)]),
            Some(self.chip.special_loc[&SpecialLocKey::DdrDll(DirHV::SE)]),
        ]);
        for i in 0..6 {
            tcells.push(
                self.chip
                    .try_get_io_cell(SpecialIoKey::Clock(Dir::S, 2 * i + 1)),
            );
        }
        for i in 0..6 {
            tcells.push(
                self.chip
                    .special_loc
                    .get(&SpecialLocKey::DllDel(Dir::S, 2 * i))
                    .copied(),
            );
        }
        let tcid = match kind {
            NexusKind::SmallMipi => tcls::CLK_S_JE5D15,
            NexusKind::SmallMipiPcie | NexusKind::LargePcs => tcls::CLK_S_JE5D30,
            NexusKind::SmallHalfIo => tcls::CLK_S_JD5U,
            NexusKind::SmallDenseIo => tcls::CLK_S_JD5F,
        };
        self.egrid.add_tile_part(cell_s, tcid, &tcells);
    }
}
