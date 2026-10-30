use std::collections::BTreeMap;

use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::ColIdExt,
};

use crate::{
    chip::{ChipKind, IoGroupKind, NexusKind, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
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
                if !self.chip.is_col_io(cell.col) {
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
                            match cell.col.side_of(self.chip.col_clk) {
                                DirH::W => tcls::DSP_W,
                                DirH::E => tcls::DSP_E,
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
            let (tcid, tcells) = match (loc.quad, kind) {
                (DirHV::SW, NexusKind::SmallMipi) => (tcls::PLL_SW_JE5D15, cell.cells_e(3)),
                (DirHV::SE, NexusKind::SmallMipi) => (tcls::PLL_SE_JE5D15, cell.cells_n(3)),
                (DirHV::SW, NexusKind::SmallMipiPcie) => (
                    tcls::PLL_SW_JE5D30,
                    vec![cell, cell.delta(0, 1), cell.delta(1, 0)],
                ),
                (
                    DirHV::SE,
                    NexusKind::SmallMipiPcie
                    | NexusKind::LargePcs(_)
                    | NexusKind::SmallDenseIo
                    | NexusKind::SmallDenseIoPcie,
                ) => (
                    tcls::PLL_SE_JE5D30,
                    vec![cell, cell.delta(0, 1), cell.delta(0, -1)],
                ),
                (DirHV::NW, NexusKind::SmallMipiPcie | NexusKind::LargePcs(_)) => (
                    tcls::PLL_NW_JE5D30,
                    vec![cell, cell.delta(0, -1), cell.delta(1, 0)],
                ),
                (
                    DirHV::SW,
                    NexusKind::LargePcs(_) | NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb,
                ) => (
                    tcls::PLL_SW_JD5D,
                    vec![cell, cell.delta(0, -1), cell.delta(1, -1)],
                ),
                (DirHV::NE, NexusKind::LargePcs(_)) => (
                    tcls::PLL_NE_JD5D,
                    vec![cell, cell.delta(0, -1), cell.delta(-1, 0)],
                ),
                (DirHV::NW, NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie) => {
                    (tcls::PLL_NW_JD5F, cell.cells_e(3))
                }
                _ => unreachable!(),
            };
            let mut tcells = Vec::from_iter(tcells.into_iter().map(Some));
            // REFCLK14
            tcells.push(
                self.chip
                    .try_get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
            );
            match loc.quad.v {
                DirV::S => {
                    // REFCLK{13,23}
                    match loc.quad.h {
                        DirH::W => {
                            tcells.extend([
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::W, 0)),
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::W, 1)),
                            ]);
                        }
                        DirH::E => {
                            tcells.extend([
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 0)),
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 1)),
                            ]);
                        }
                    }
                    // REFCLK{15,25,16,26}
                    let s4 =
                        if matches!(kind, NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie) {
                            [None; 2]
                        } else {
                            [
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 4)),
                                self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 5)),
                            ]
                        };
                    let s8e0 = match loc.quad.h {
                        DirH::W => [
                            self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 8)),
                            self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::S, 9)),
                        ],
                        DirH::E => [
                            self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::E, 0)),
                            self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::E, 1)),
                        ],
                    };
                    if matches!(kind, NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb) {
                        tcells.extend(s8e0);
                        tcells.extend(s4);
                    } else {
                        tcells.extend(s4);
                        tcells.extend(s8e0);
                    }
                }
                DirV::N => {
                    // REFCLK{13,23}
                    let cidx =
                        if matches!(kind, NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie) {
                            4
                        } else {
                            3
                        };
                    tcells.extend([
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(loc.quad.h.into(), cidx)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(loc.quad.h.into(), cidx + 1)),
                    ]);
                    // REFCLK{16,26}
                    tcells.extend([
                        self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 0)),
                        self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    ]);
                    // REFCLK{15,17} in one cell
                    tcells.push(self.chip.try_get_spec_cell(SpecialLocKey::Pcs(
                        match loc.quad.h {
                            DirH::W => 0,
                            DirH::E => 1,
                        },
                    )));
                }
            }
            self.egrid.add_tile_part(cell, tcid, &tcells);
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
            if let Some(&bank) = self.chip.columns[cell.col].bank.get(DirV::S) {
                if bank == 11 {
                    self.egrid.add_tile_single(cell, tcls::IO_S_B);
                    continue;
                }
                self.egrid[cell].region_root[rslots::ECLK] =
                    *bank_cells.entry(bank).or_insert(cell);
                io_cells.push(cell);
                self.egrid.add_tile_e(
                    cell,
                    match kind {
                        NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => tcls::IO_HP_S_JD5U,
                        NexusKind::LargePcs(_) => tcls::IO_HP_S_LPDDR4,
                        _ => tcls::IO_HP_S,
                    },
                    2,
                );
            }
            if self.chip.columns[cell.col].io[DirV::S] == IoGroupKind::DoubleDqs {
                let (cells, tcid) = match kind {
                    NexusKind::SmallMipi => (cell.delta(-6, 0).cells_e(16), tcls::DQS_S),
                    NexusKind::SmallMipiPcie => (cell.delta(-6, 0).cells_e(16), tcls::DQS_L),
                    NexusKind::LargePcs(_) => {
                        if cell.col == self.chip.col_clk - 8 {
                            (cell.delta(-4, 0).cells_e(24), tcls::DQS_JD5D_HOLE)
                        } else {
                            (cell.delta(-4, 0).cells_e(12), tcls::DQS_JD5D)
                        }
                    }
                    NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => {
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
            if self.chip.columns[cell_dqs.col].io[DirV::S] != IoGroupKind::DoubleDqs {
                continue;
            }
            let cells = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie => &io_cells[(i - 3)..(i + 5)],
                NexusKind::LargePcs(_) | NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => {
                    &io_cells[(i - 2)..(i + 4)]
                }
                NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => unreachable!(),
            };
            for &cell in cells {
                self.egrid[cell].region_root[rslots::DQS] = cell_dqs;
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            let io_kind = self.chip.columns[cell.col].io[DirV::N];
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
                if self.chip.is_row_io(cell.row) {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
                let io_kind = self.chip.rows[cell.row].io[edge];
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
                    NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs(_) => {
                        match edge {
                            DirH::W => tcls::DDRDLL_W,
                            DirH::E => tcls::DDRDLL_E,
                        }
                    }
                    NexusKind::SmallHalfIo
                    | NexusKind::SmallHalfIoUsb
                    | NexusKind::SmallDenseIo
                    | NexusKind::SmallDenseIoPcie => match edge {
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
                NexusKind::LargePcs(_)
                | NexusKind::SmallHalfIo
                | NexusKind::SmallHalfIoUsb
                | NexusKind::SmallDenseIo
                | NexusKind::SmallDenseIoPcie => match edge {
                    Dir::W => tcls::DLLDEL_W_B,
                    Dir::E => tcls::DLLDEL_E_B,
                    Dir::S => tcls::DLLDEL_S_B,
                    Dir::N => tcls::DLLDEL_N_B,
                },
            };
            self.egrid.add_tile_single(cell, tcid);
            self.add_io_conn(cell.wire(wires::IO_DLLDEL), SpecialIoKey::Clock(edge, idx));
        }
    }

    pub(super) fn fill_misc_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
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
                NexusKind::SmallMipiPcie => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JE5D30,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            cell.delta(0, 3),
                            self.chip.bel_pll(DirHV::SW, 0).cell,
                        ],
                    );
                }
                NexusKind::LargePcs(_) => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JD5D,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            cell.delta(0, 3),
                            self.chip.bel_pll(DirHV::SW, 0).cell,
                        ],
                    );
                }
                NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => unreachable!(),
                NexusKind::SmallDenseIo => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JD5F,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            cell.delta(0, 3),
                            self.chip.bel_pll(DirHV::SE, 0).cell,
                        ],
                    );
                }
                NexusKind::SmallDenseIoPcie => {
                    self.egrid.add_tile(
                        cell,
                        tcls::CDR_W_JD5T,
                        &[
                            cell,
                            cell.delta(0, 1),
                            cell.delta(0, 2),
                            cell.delta(0, 3),
                            self.chip.bel_pll(DirHV::SE, 0).cell,
                        ],
                    );
                }
            }
        }
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::Adc) {
            let (tcid, cell_comp) = match kind {
                NexusKind::SmallMipi => (tcls::ADC_JE5D15, cell.delta(0, -2)),
                NexusKind::SmallMipiPcie => (tcls::ADC_JE5D30, cell.delta(0, 2)),
                NexusKind::LargePcs(_) | NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => {
                    (tcls::ADC_JD5D, cell.delta(0, -2))
                }
                NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => unreachable!(),
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
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::Mipi) {
            let (tcid, num) = match kind {
                NexusKind::SmallMipi => (tcls::MIPI_JE5D15, 55),
                NexusKind::SmallMipiPcie => (tcls::MIPI_JE5D30, 48),
                _ => unreachable!(),
            };
            let mut tcells = cell.cells_e(num);
            for key in [
                SpecialIoKey::Clock(Dir::S, 7),
                SpecialIoKey::Clock(Dir::S, 3),
                SpecialIoKey::Clock(Dir::S, 1),
                SpecialIoKey::Clock(Dir::N, 0),
                SpecialIoKey::Clock(Dir::N, 1),
            ] {
                tcells.push(self.chip.get_io_cell(key));
            }
            self.egrid.add_tile(cell, tcid, &tcells);
        }
        match kind {
            NexusKind::SmallMipi => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(cell, tcls::CONFIG_JE5D15, 22);

                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::W, 0)];
                self.egrid.add_tile(
                    cell,
                    tcls::LRAM_W_JE5D15,
                    &Vec::from_iter(
                        [
                            cell.cells_n(6),
                            cell.step(Dir::N, 9).cells_n(6),
                            cell.step(Dir::N, 18).cells_n(6),
                        ]
                        .into_iter()
                        .flatten(),
                    ),
                );
                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 0)];
                self.egrid.add_tile_n(cell, tcls::LRAM_E_JE5D15, 10);

                for (bank, tcid) in [
                    (0, tcls::BNKREF33_N_JE5D15),
                    (1, tcls::BNKREF33_E_JE5D15),
                    (3, tcls::BNKREF18_S_JE5D15),
                    (5, tcls::BNKREF18_S_JE5D15),
                ] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }
                let cell = self.chip.special_loc[&SpecialLocKey::Bc(4)];
                self.egrid.add_tile_e(cell, tcls::BNKREF18_S_JE5D15_4, 6);
            }
            NexusKind::SmallMipiPcie => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(cell, tcls::CONFIG_JE5D30, 14);

                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 0)];
                self.egrid.add_tile_sn(cell, tcls::LRAM_E_JE5D30_A, 5, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 1)];
                self.egrid.add_tile_sn(cell, tcls::LRAM_E_JE5D30_B, 7, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Alu];
                self.egrid.add_tile_sn(cell, tcls::ALU_JE5D30, 2, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Pcie];
                self.egrid.add_tile_e(cell, tcls::PCIE_JE5D30, 15);

                for (bank, tcid) in [
                    (0, tcls::BNKREF33_N_JE5D30),
                    (1, tcls::BNKREF33_E_JE5D15),
                    (2, tcls::BNKREF33_E_JE5D15),
                    (3, tcls::BNKREF18_E_JE5D30),
                    (4, tcls::BNKREF18_S_JE5D30),
                    (5, tcls::BNKREF18_S_JE5D30),
                    (6, tcls::BNKREF33_W_JE5D30),
                    (7, tcls::BNKREF33_W_JE5D30),
                ] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }
            }
            NexusKind::LargePcs(_) => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(cell, tcls::CONFIG_JD5D, 15);

                let cell = self.chip.special_loc[&SpecialLocKey::Pcs(0)];
                self.egrid.add_tile(
                    cell,
                    tcls::PCS_N_JD5D,
                    &Vec::from_iter(
                        [
                            cell.cells_e(8),
                            cell.step(Dir::E, 9).cells_e(8),
                            cell.step(Dir::E, 38).cells_e(8),
                            cell.step(Dir::E, 49).cells_e(8),
                        ]
                        .into_iter()
                        .flatten(),
                    ),
                );
                let cell = self.chip.special_loc[&SpecialLocKey::Pcs(1)];
                self.egrid.add_tile(
                    cell,
                    tcls::PCS_N_JD5D,
                    &Vec::from_iter(
                        [
                            cell.cells_e(5),
                            cell.step(Dir::E, 7).cells_e(3),
                            cell.step(Dir::E, 11).cells_e(8),
                            cell.step(Dir::E, 29).cells_e(8),
                            cell.step(Dir::E, 47).cells_e(8),
                        ]
                        .into_iter()
                        .flatten(),
                    ),
                );
                let cell = self.chip.special_loc[&SpecialLocKey::PcsRefmux];
                self.egrid.add_tile(
                    cell,
                    tcls::PCS_REFMUX_JD5D,
                    &[
                        cell,
                        self.chip.bel_pll(DirHV::NW, 0).cell,
                        self.chip.bel_pll(DirHV::NE, 0).cell,
                    ],
                );
                let cell = self.chip.special_loc[&SpecialLocKey::PcsRefclk];
                self.egrid.add_tile(cell, tcls::PCS_REFCLK, &[]);
                let cell = self.chip.special_loc[&SpecialLocKey::Pcie];
                self.egrid.add_tile(
                    cell,
                    tcls::PCIE_JD5D,
                    &Vec::from_iter(
                        [
                            cell.cells_e(20),
                            cell.step(Dir::E, 29).cells_e(2),
                            cell.step(Dir::E, 40).cells_e(5),
                        ]
                        .into_iter()
                        .flatten(),
                    ),
                );

                for i in 0..4 {
                    let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::N, i)];
                    self.egrid.add_tile_e(cell, tcls::LRAM_N_JD5D, 5);
                }
                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 0)];
                self.egrid.add_tile_sn(cell, tcls::LRAM_E_JD5D_A, 7, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 1)];
                self.egrid.add_tile_sn(cell, tcls::LRAM_E_JD5D_B, 7, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, 2)];
                self.egrid.add_tile_sn(cell, tcls::LRAM_E_JD5D_C, 7, 9);
                let cell = self.chip.special_loc[&SpecialLocKey::Alu];
                self.egrid.add_tile_sn(cell, tcls::ALU_JD5D, 9, 11);

                for (bank, tcid) in [(3, tcls::IVREF_E), (4, tcls::IVREF_S), (5, tcls::IVREF_S)] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::IVref(bank)], tcid);
                }

                for (bank, tcid) in [
                    (0, tcls::BNKREF33_N_JE5D30),
                    (1, tcls::BNKREF33_E_JE5D15),
                    (2, tcls::BNKREF33_E_JE5D15),
                    (3, tcls::BNKREF18_E_JD5D),
                    (4, tcls::BNKREF18_S_JD5D),
                    (5, tcls::BNKREF18_S_JD5D),
                    (6, tcls::BNKREF33_W_JE5D30),
                    (7, tcls::BNKREF33_W_JE5D30),
                ] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }
            }
            NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(
                    cell,
                    if kind == NexusKind::SmallHalfIoUsb {
                        tcls::CONFIG_JD5UA
                    } else {
                        tcls::CONFIG_JD5U
                    },
                    15,
                );

                for i in 0..5 {
                    let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::E, i)];
                    self.egrid.add_tile_sn(cell, tcls::LRAM_E_JD5U, 8, 9);
                }

                for (bank, tcid) in [
                    (0, tcls::BNKREF33_N_JD5U),
                    (1, tcls::BNKREF33_N_JD5U),
                    (2, tcls::BNKREF18_S_JE5D30),
                    (3, tcls::BNKREF18_S_JE5D30),
                    (4, tcls::BNKREF18_S_JE5D30),
                    (5, tcls::BNKREF33_N_JD5U),
                ] {
                    if bank == 5 && kind == NexusKind::SmallHalfIoUsb {
                        continue;
                    }
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }

                if kind == NexusKind::SmallHalfIoUsb {
                    let cell = self.chip.special_loc[&SpecialLocKey::Usb];
                    self.egrid.add_tile_e(cell, tcls::USB, 20);
                    let cell = self.chip.special_loc[&SpecialLocKey::Aon];
                    self.egrid.add_tile_e(cell, tcls::AON, 2);
                }
            }
            NexusKind::SmallDenseIo => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(cell, tcls::CONFIG_JD5F, 24);

                let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::N, 0)];
                self.egrid.add_tile_e(cell, tcls::LRAM_N_JD5D, 5);

                for (bank, tcid, dx, dy) in [
                    (0, tcls::BNKREF33_N_JD5F, 1, 0),
                    (1, tcls::BNKREF33_N_JD5F, 5, 0),
                    (2, tcls::BNKREF33_E_JD5F, 0, -1),
                    (3, tcls::BNKREF33_E_JD5F, 0, 1),
                    (4, tcls::BNKREF33_E_JD5F, 0, 2),
                    (7, tcls::BNKREF33_W_JD5F, 0, 2),
                    (8, tcls::BNKREF33_W_JD5F, 0, 1),
                    (9, tcls::BNKREF33_W_JD5F, 0, -1),
                ] {
                    let cell = self.chip.special_loc[&SpecialLocKey::Bc(bank)];
                    self.egrid.add_tile(cell, tcid, &[cell, cell.delta(dx, dy)]);
                }
                for (bank, tcid) in [(5, tcls::BNKREF18_S_JE5D30), (6, tcls::BNKREF18_S_JE5D30)] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }
            }
            NexusKind::SmallDenseIoPcie => {
                let cell = self.chip.special_loc[&SpecialLocKey::Config];
                self.egrid.add_tile_e(cell, tcls::CONFIG_JD5T, 24);

                for i in 0..2 {
                    let cell = self.chip.special_loc[&SpecialLocKey::Lram(Dir::N, i)];
                    self.egrid.add_tile_e(cell, tcls::LRAM_N_JD5D, 5);
                }

                let cell = self.chip.special_loc[&SpecialLocKey::Pcie];
                self.egrid.add_tile_e(cell, tcls::PCIE_JD5T, 20);

                for (bank, tcid, dx, dy) in [
                    (0, tcls::BNKREF33_N_JD5F, 1, 0),
                    (1, tcls::BNKREF33_N_JD5F, 5, 0),
                    (2, tcls::BNKREF33_E_JD5F, 0, -1),
                    (3, tcls::BNKREF33_E_JD5T, 0, 0),
                    (4, tcls::BNKREF33_E_JD5T, 0, 0),
                    (7, tcls::BNKREF33_W_JD5T, 0, 0),
                    (8, tcls::BNKREF33_W_JD5T, 0, 0),
                    (9, tcls::BNKREF33_N_JD5F, -1, -1),
                    (11, tcls::BNKREF33_S_JD5T, 0, 0),
                ] {
                    let cell = self.chip.special_loc[&SpecialLocKey::Bc(bank)];
                    if dx == 0 && dy == 0 {
                        self.egrid.add_tile_single(cell, tcid);
                    } else {
                        self.egrid.add_tile(cell, tcid, &[cell, cell.delta(dx, dy)]);
                    }
                }
                for (bank, tcid) in [(5, tcls::BNKREF18_S_JE5D30), (6, tcls::BNKREF18_S_JE5D30)] {
                    self.egrid
                        .add_tile_single(self.chip.special_loc[&SpecialLocKey::Bc(bank)], tcid);
                }
            }
        }
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PvtCal33) {
            let tcid = match kind {
                NexusKind::SmallMipi => tcls::PVTCAL33_JE5D15,
                NexusKind::SmallMipiPcie => tcls::PVTCAL33_JE5D30,
                NexusKind::LargePcs(_)
                | NexusKind::SmallHalfIo
                | NexusKind::SmallHalfIoUsb
                | NexusKind::SmallDenseIo
                | NexusKind::SmallDenseIoPcie => tcls::PVTCAL33_JD5D,
            };
            self.egrid.add_tile_single(cell, tcid);
        }
        if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PvtCal18) {
            let tcid = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie => tcls::PVTCAL18_JE5D15,
                NexusKind::LargePcs(_) | NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => {
                    tcls::PVTCAL18_JD5D
                }
                NexusKind::SmallDenseIo => tcls::PVTCAL18_JD5F,
                NexusKind::SmallDenseIoPcie => tcls::PVTCAL18_JD5T,
            };
            self.egrid.add_tile_single(cell, tcid);
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

        if !matches!(kind, NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb) {
            for edge in DirH::DIRS {
                let cell = self.chip.clk_emid(edge);
                let mut tcells = cell.step(Dir::S, 2).cells_n(5);
                if kind == NexusKind::SmallMipi {
                    tcells.extend([
                        cell.step(!edge, 12),
                        cell.step(!edge, 12).step(Dir::S, 9),
                        cell.step(!edge, 24),
                        cell.step(!edge, 24).step(Dir::S, 9),
                    ]);
                } else {
                    tcells.extend([
                        cell.step(!edge, 12).step(Dir::N, 18),
                        cell.step(!edge, 12).step(Dir::N, 9),
                        cell.step(!edge, 12).step(Dir::S, 9),
                        cell.step(!edge, 12).step(Dir::S, 18),
                    ]);
                }
                let mut tcells = Vec::from_iter(tcells.into_iter().map(Some));
                if matches!(kind, NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie) {
                    tcells.extend([
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 0)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 1)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 4)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 5)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 6)),
                        self.chip
                            .try_get_spec_cell(SpecialLocKey::DllDel(edge.into(), 2)),
                        self.chip
                            .try_get_spec_cell(SpecialLocKey::DllDel(edge.into(), 3)),
                    ]);
                } else {
                    tcells.extend([
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 1)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 2)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 4)),
                        self.chip
                            .try_get_io_cell(SpecialIoKey::Clock(edge.into(), 5)),
                        self.chip
                            .try_get_spec_cell(SpecialLocKey::DllDel(edge.into(), 0)),
                        self.chip
                            .try_get_spec_cell(SpecialLocKey::DllDel(edge.into(), 3)),
                    ]);
                }
                match edge {
                    DirH::W => {
                        if !matches!(kind, NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie) {
                            tcells.push(self.chip.try_bel_pll(DirHV::SW, 0).map(|bcrd| bcrd.cell));
                        }
                        tcells.push(self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell));
                        tcells.push(self.chip.try_get_spec_cell(SpecialLocKey::Cdr));
                        let tcid = match kind {
                            NexusKind::SmallMipi => tcls::CLK_W_JE5D15,
                            NexusKind::SmallMipiPcie => tcls::CLK_W_JE5D30,
                            NexusKind::LargePcs(_) => tcls::CLK_W_JD5D,
                            NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => unreachable!(),
                            NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => {
                                tcls::CLK_W_JD5F
                            }
                        };
                        self.egrid.add_tile_part(cell, tcid, &tcells);
                    }
                    DirH::E => {
                        tcells.push(self.chip.try_bel_pll(DirHV::SE, 0).map(|bcrd| bcrd.cell));
                        if matches!(kind, NexusKind::LargePcs(_)) {
                            tcells.push(self.chip.try_bel_pll(DirHV::NE, 0).map(|bcrd| bcrd.cell));
                        }
                        if matches!(
                            kind,
                            NexusKind::LargePcs(_)
                                | NexusKind::SmallDenseIo
                                | NexusKind::SmallDenseIoPcie
                        ) {
                            tcells.push(self.chip.try_get_spec_cell(SpecialLocKey::Config));
                        }
                        let tcid = match kind {
                            NexusKind::SmallMipi => tcls::CLK_E_JE5D15,
                            NexusKind::SmallMipiPcie => tcls::CLK_E_JE5D30,
                            NexusKind::LargePcs(_) => tcls::CLK_E_JD5D,
                            NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => unreachable!(),
                            NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => {
                                tcls::CLK_E_JD5F
                            }
                        };
                        self.egrid.add_tile_part(cell, tcid, &tcells);
                    }
                }
            }
        }

        let cell_s = self.chip.clk_emid(Dir::S);
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
        if kind == NexusKind::SmallHalfIoUsb {
            tcells.push(Some(
                self.chip.special_loc[&SpecialLocKey::Aon].step(Dir::E, 1),
            ));
        }
        let tcid = match kind {
            NexusKind::SmallMipi => tcls::CLK_S_JE5D15,
            NexusKind::SmallMipiPcie | NexusKind::LargePcs(_) => tcls::CLK_S_JE5D30,
            NexusKind::SmallHalfIo => tcls::CLK_S_JD5U,
            NexusKind::SmallHalfIoUsb => tcls::CLK_S_JD5UA,
            NexusKind::SmallDenseIo | NexusKind::SmallDenseIoPcie => tcls::CLK_S_JD5F,
        };
        self.egrid.add_tile_part(cell_s, tcid, &tcells);

        let cell_n = self.chip.clk_emid(Dir::N);
        let mut tcells = cell_n.step(Dir::W, 3).cells_e(6);
        tcells.push(self.chip.special_loc[&SpecialLocKey::Config]);
        let mut tcells = Vec::from_iter(tcells.into_iter().map(Some));
        match kind {
            NexusKind::SmallMipi | NexusKind::SmallMipiPcie => {
                tcells.extend([
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 0)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_get_spec_cell(SpecialLocKey::Mipi),
                    self.chip.try_get_spec_cell(SpecialLocKey::Pcie),
                ]);
            }
            NexusKind::LargePcs(_) => {
                tcells.extend([
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 0)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_bel_pll(DirHV::NE, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_get_spec_cell(SpecialLocKey::Pcs(0)),
                    self.chip.try_get_spec_cell(SpecialLocKey::Pcs(1)),
                    self.chip.try_get_spec_cell(SpecialLocKey::Pcie),
                ]);
            }
            NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => {
                tcells.extend([
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 3)),
                    self.chip
                        .try_get_spec_cell(SpecialLocKey::DllDel(Dir::N, 0)),
                    self.chip
                        .try_get_spec_cell(SpecialLocKey::DllDel(Dir::N, 2)),
                ]);
            }
            NexusKind::SmallDenseIo => {
                tcells.extend([
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 0)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 2)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 3)),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                ]);
            }
            NexusKind::SmallDenseIoPcie => {
                tcells.extend([
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 0)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 1)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 2)),
                    self.chip.try_get_io_cell(SpecialIoKey::Clock(Dir::N, 3)),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_get_spec_cell(SpecialLocKey::Pcie),
                ]);
            }
        }
        let tcid = match kind {
            NexusKind::SmallMipi => tcls::CLK_N_JE5D15,
            NexusKind::SmallMipiPcie => tcls::CLK_N_JE5D30,
            NexusKind::LargePcs(_) => tcls::CLK_N_JD5D,
            NexusKind::SmallHalfIo | NexusKind::SmallHalfIoUsb => tcls::CLK_N_JD5U,
            NexusKind::SmallDenseIo => tcls::CLK_N_JD5F,
            NexusKind::SmallDenseIoPcie => tcls::CLK_N_JD5T,
        };
        self.egrid.add_tile_part(cell_n, tcid, &tcells);

        let cell = self.chip.clk_root();
        let mut tcells = vec![
            cell.delta(-1, -9),
            cell.delta(0, -9),
            cell.delta(-1, 0),
            cell,
        ];
        if matches!(
            kind,
            NexusKind::SmallMipiPcie
                | NexusKind::SmallHalfIo
                | NexusKind::SmallHalfIoUsb
                | NexusKind::SmallDenseIo
                | NexusKind::SmallDenseIoPcie
        ) {
            tcells.extend([cell.delta(-1, 9), cell.delta(0, 9)]);
        }
        tcells.extend([
            self.chip.clk_emid(Dir::W),
            self.chip.clk_emid(Dir::E),
            self.chip.clk_emid(Dir::S),
            self.chip.clk_emid(Dir::N),
        ]);
        let tcid = match kind {
            NexusKind::SmallMipi => tcls::CLK_ROOT_JE5D15,
            NexusKind::SmallMipiPcie => tcls::CLK_ROOT_JE5D30,
            NexusKind::LargePcs(_) => tcls::CLK_ROOT_JD5D,
            NexusKind::SmallHalfIo
            | NexusKind::SmallHalfIoUsb
            | NexusKind::SmallDenseIo
            | NexusKind::SmallDenseIoPcie => tcls::CLK_ROOT_JD5U,
        };
        self.egrid.add_tile(cell, tcid, &tcells);
    }
}
