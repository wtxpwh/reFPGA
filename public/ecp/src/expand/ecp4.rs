use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{ColId, ColIdExt, DieIdExt, RowIdExt},
    dir::{Dir, DirH, DirHV, DirV},
};

use crate::{
    chip::{ChipKind, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{rslots, tcls, wires},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_pll_ecp4(&mut self) {
        let has_bank0 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(0));
        for (loc, bcrd) in self.chip.plls() {
            if loc.idx != 0 {
                continue;
            }
            let cell = bcrd.cell;
            let kind = match loc.quad {
                DirHV::SW => tcls::PLL_SW,
                DirHV::SE => tcls::PLL_SE,
                DirHV::NW => tcls::PLL_NW,
                DirHV::NE => tcls::PLL_NE,
            };
            let mut tcells = match loc.quad.v {
                DirV::S => vec![
                    cell.delta(
                        match loc.quad.h {
                            DirH::W => 1,
                            DirH::E => -1,
                        },
                        0,
                    ),
                    cell,
                    cell.delta(0, 1),
                    cell.delta(0, 2),
                ],
                DirV::N => cell.delta(0, -3).cells_n(4),
            };
            tcells.extend([
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, loc)),
                self.chip
                    .get_io_cell(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc { idx: 1, ..loc })),
                self.chip.get_io_cell(SpecialIoKey::Clock(
                    loc.quad.h.into(),
                    match loc.quad.v {
                        DirV::S => 0,
                        DirV::N => 2,
                    },
                )),
            ]);
            if loc.quad.v == DirV::N {
                tcells.extend([
                    self.chip.get_io_cell(SpecialIoKey::Clock(
                        Dir::N,
                        match loc.quad.h {
                            DirH::W => {
                                if has_bank0 {
                                    0
                                } else {
                                    2
                                }
                            }
                            DirH::E => {
                                if has_bank0 {
                                    6
                                } else {
                                    4
                                }
                            }
                        },
                    )),
                    self.chip.get_io_cell(SpecialIoKey::Clock(
                        Dir::N,
                        match loc.quad.h {
                            DirH::W => 2,
                            DirH::E => 4,
                        },
                    )),
                ]);
            }
            if kind == tcls::PLL_SW {
                tcells.extend([
                    self.chip.get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, loc)),
                    self.chip
                        .get_io_cell(SpecialIoKey::Pll(PllPad::PllFb, PllLoc { idx: 1, ..loc })),
                    self.chip
                        .special_loc
                        .get(&SpecialLocKey::SerdesSingle)
                        .or(self.chip.special_loc.get(&SpecialLocKey::SerdesDouble))
                        .or(self.chip.special_loc.get(&SpecialLocKey::SerdesTriple))
                        .copied()
                        .unwrap(),
                ]);
            }
            self.egrid.add_tile(cell, kind, &tcells);
        }
    }

    pub(super) fn fill_serdes_ecp4(&mut self) {
        for (key, kind, num_cells) in [
            (SpecialLocKey::SerdesSingle, tcls::SERDES1, 62),
            (SpecialLocKey::SerdesDouble, tcls::SERDES2, 120),
            (SpecialLocKey::SerdesTriple, tcls::SERDES3, 177),
        ] {
            if let Some(&cell) = self.chip.special_loc.get(&key) {
                self.egrid.add_tile_e(cell, kind, num_cells);
            }
        }
    }

    pub(super) fn fill_ebr_ecp4(&mut self) {
        let (mut col_start_w, mut col_end_e) = match self.chip.kind {
            ChipKind::Ecp4 => (self.chip.col_w() + 5usize, self.chip.col_e() - 4usize),
            ChipKind::Ecp5 => (self.chip.col_w() + 3usize, self.chip.col_e() - 2usize),
            _ => unreachable!(),
        };
        let col_end_w = self.chip.col_clk - 1usize;
        let col_start_e = self.chip.col_clk + 1usize;
        while col_start_w.to_idx() % 9 != col_end_w.to_idx() % 9 {
            col_start_w += 1;
        }
        while col_start_e.to_idx() % 9 != col_end_e.to_idx() % 9 {
            col_end_e -= 1;
        }
        for (row, rd) in &self.chip.rows {
            let kind = match rd.kind {
                RowKind::Ebr => tcls::EBR,
                RowKind::Dsp => tcls::DSP,
                _ => continue,
            };
            for cell in self.egrid.row(self.die, row) {
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
                if (cell.col >= col_start_w
                    && cell.col < col_end_w
                    && cell.col.to_idx() % 9 == col_start_w.to_idx() % 9)
                    || (cell.col >= col_start_e
                        && cell.col < col_end_e
                        && cell.col.to_idx() % 9 == col_start_e.to_idx() % 9)
                {
                    self.egrid.add_tile_e(cell, kind, 9);
                }
            }
        }
    }

    pub(super) fn fill_io_ecp4(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let kind = match bank {
                0..4 => tcls::BC_N,
                4..6 => tcls::BC_E,
                6..8 => tcls::BC_W,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, kind);
        }
        {
            let cell = self
                .chip
                .special_loc
                .get(&SpecialLocKey::Bc(3))
                .copied()
                .unwrap_or(self.chip.special_loc[&SpecialLocKey::Bc(2)]);
            self.egrid.add_tile_single(cell, tcls::PVTTEST);
        }
        for (col, wire_s, wire_n) in [
            (
                self.chip.col_w() + 2,
                wires::DDRDEL_SW[0],
                wires::DDRDEL_NW[0],
            ),
            (
                self.chip.col_w() + 3,
                wires::DDRDEL_SW[1],
                wires::DDRDEL_NW[1],
            ),
            (
                self.chip.col_e() - 3,
                wires::DDRDEL_SE[0],
                wires::DDRDEL_NE[0],
            ),
            (
                self.chip.col_e() - 2,
                wires::DDRDEL_SE[1],
                wires::DDRDEL_NE[1],
            ),
        ] {
            for (row, kind, wire) in [
                (self.chip.row_s(), tcls::DDRDLL_S, wire_s),
                (self.chip.row_n(), tcls::DDRDLL_N, wire_n),
            ] {
                let cell = self.die.cell(col, row);
                self.egrid.add_tile(
                    cell,
                    kind,
                    &[
                        cell,
                        self.die.cell(
                            self.chip.col_edge(cell.col.side_of(self.chip.col_clk)),
                            cell.row.snap_to_side_of(self.chip.row_clk),
                        ),
                    ],
                );
                self.egrid.extra_conns.insert(
                    cell.wire(wires::OUT_DDRDLL_DDRDEL),
                    self.chip.clk_root().wire(wire),
                );
            }
        }
        for (col, row, kind) in [
            (self.chip.col_w() + 1, self.chip.row_n(), tcls::DTR_N),
            (self.chip.col_e() - 4, self.chip.row_s(), tcls::DTR_S),
        ] {
            let cell = self.die.cell(col, row);
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            if self.chip.columns[cell.col].io[DirV::N] != IoGroupKind::None {
                self.egrid.add_tile_e(cell, tcls::IO_N, 4);
            }
            if self.chip.columns[cell.col].io[DirV::N] == IoGroupKind::QuadDqs {
                self.egrid.add_tile_we(cell, tcls::DQS_N, 1, 5);
                self.egrid[cell].region_root[rslots::DQS] = cell;
                self.egrid[cell.delta(-4, 0)].region_root[rslots::DQS] = cell;
                self.egrid[cell.delta(4, 0)].region_root[rslots::DQS] = cell;
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            if matches!(
                rd.io[DirH::W],
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                if rd.kind == RowKind::Dsp {
                    assert!(cell.row >= self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_W_DSP_N,
                        &[cell.delta(0, 1), cell, cell.delta(2, 0), cell.delta(1, 0)],
                    );
                    if rd.io[DirH::W] == IoGroupKind::QuadDqs {
                        self.egrid.add_tile(
                            cell,
                            tcls::DQS_W_DSP_N,
                            &[
                                cell.delta(0, 2),
                                cell.delta(0, 1),
                                cell,
                                cell.delta(2, 0),
                                cell.delta(1, 0),
                            ],
                        );
                        self.egrid[cell].region_root[rslots::DQS] = cell;
                        self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.delta(0, 2)].region_root[rslots::DQS] = cell;
                    }
                } else if self.chip.rows[cell.row + 1].kind == RowKind::Dsp {
                    assert!(cell.row < self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_W_DSP_S,
                        &[cell.delta(1, 1), cell.delta(2, 1), cell.delta(0, 1), cell],
                    );
                } else {
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_W,
                        &[
                            cell.delta(0, 3),
                            cell.delta(0, 2),
                            cell.delta(0, 1),
                            cell.delta(0, 0),
                        ],
                    );
                    if rd.io[DirH::W] == IoGroupKind::QuadDqs {
                        if self.chip.rows[cell.row + 3].kind == RowKind::Ebr {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_W_BELOW_EBR_S,
                                &[
                                    cell.delta(4, 3),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(1, 3)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row + 4].kind == RowKind::Dsp {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_W_BELOW_DSP_N,
                                &[
                                    cell.delta(1, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(0, 4)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row + 4].kind == RowKind::Ebr {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_W_BELOW_EBR_N,
                                &[
                                    cell.delta(1, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(1, 4)].region_root[rslots::DQS] = cell;
                        } else {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_W,
                                &[
                                    cell.delta(0, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(0, 4)].region_root[rslots::DQS] = cell;
                        }
                        self.egrid[cell].region_root[rslots::DQS] = cell;
                        if self.chip.rows[cell.row].kind == RowKind::Ebr {
                            self.egrid[cell.delta(1, 0)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row - 1].kind == RowKind::Ebr {
                            self.egrid[cell.delta(1, -1)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row - 1].kind == RowKind::Dsp
                            || self.chip.rows[cell.row - 2].kind == RowKind::Dsp
                        {
                            self.egrid[cell.delta(0, -2)].region_root[rslots::DQS] = cell;
                        } else {
                            self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell;
                        }
                    }
                }
            }
            if rd.kind == RowKind::Ebr {
                if cell.row < self.chip.row_clk {
                    self.egrid.add_tile(
                        cell.delta(1, 0),
                        tcls::IO_W_EBR_S,
                        &[
                            cell.delta(1, 0),
                            cell.delta(2, 0),
                            cell.delta(3, 0),
                            cell.delta(4, 0),
                        ],
                    );
                    if matches!(
                        rd.io[DirH::W],
                        IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        self.egrid.add_tile(
                            cell.delta(1, 0),
                            tcls::DQS_W_EBR_S,
                            &[
                                cell.delta(0, 1),
                                cell.delta(1, 0),
                                cell.delta(2, 0),
                                cell.delta(3, 0),
                                cell.delta(4, 0),
                            ],
                        );
                        self.egrid[cell.delta(1, 0)].region_root[rslots::DQS] = cell.delta(1, 0);
                        self.egrid[cell.delta(0, -3)].region_root[rslots::DQS] = cell.delta(1, 0);
                        self.egrid[cell.delta(0, 1)].region_root[rslots::DQS] = cell.delta(1, 0);
                    }
                } else {
                    self.egrid.add_tile(
                        cell.delta(1, 0),
                        tcls::IO_W_EBR_N,
                        &[
                            cell.delta(4, 0),
                            cell.delta(3, 0),
                            cell.delta(2, 0),
                            cell.delta(1, 0),
                        ],
                    );
                    if matches!(
                        rd.io[DirH::W],
                        IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        self.egrid.add_tile(
                            cell.delta(1, 0),
                            tcls::DQS_W_EBR_N,
                            &[
                                cell,
                                cell.delta(4, 0),
                                cell.delta(3, 0),
                                cell.delta(2, 0),
                                cell.delta(1, 0),
                            ],
                        );
                        self.egrid[cell.delta(1, 0)].region_root[rslots::DQS] = cell.delta(1, 0);
                        self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell.delta(1, 0);
                        self.egrid[cell].region_root[rslots::DQS] = cell.delta(1, 0);
                    }
                }
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
            if matches!(
                rd.io[DirH::E],
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                if rd.kind == RowKind::Dsp {
                    assert!(cell.row >= self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_E_DSP_N,
                        &[cell.delta(0, 1), cell, cell.delta(-2, 0), cell.delta(-1, 0)],
                    );
                    if rd.io[DirH::E] == IoGroupKind::QuadDqs {
                        self.egrid.add_tile(
                            cell,
                            tcls::DQS_E_DSP_N,
                            &[
                                cell.delta(0, 2),
                                cell.delta(0, 1),
                                cell,
                                cell.delta(-2, 0),
                                cell.delta(-1, 0),
                            ],
                        );
                        self.egrid[cell].region_root[rslots::DQS] = cell;
                        self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell;
                        self.egrid[cell.delta(0, 2)].region_root[rslots::DQS] = cell;
                    }
                } else if self.chip.rows[cell.row + 1].kind == RowKind::Dsp {
                    assert!(cell.row < self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_E_DSP_S,
                        &[cell.delta(-1, 1), cell.delta(-2, 1), cell.delta(0, 1), cell],
                    );
                } else {
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_E,
                        &[
                            cell.delta(0, 3),
                            cell.delta(0, 2),
                            cell.delta(0, 1),
                            cell.delta(0, 0),
                        ],
                    );
                    if rd.io[DirH::E] == IoGroupKind::QuadDqs {
                        if self.chip.rows[cell.row + 3].kind == RowKind::Ebr {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_E_BELOW_EBR_S,
                                &[
                                    cell.delta(-4, 3),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(-1, 3)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row + 4].kind == RowKind::Dsp {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_E_BELOW_DSP_N,
                                &[
                                    cell.delta(-1, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(0, 4)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row + 4].kind == RowKind::Ebr {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_E_BELOW_EBR_N,
                                &[
                                    cell.delta(-1, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(-1, 4)].region_root[rslots::DQS] = cell;
                        } else {
                            self.egrid.add_tile(
                                cell,
                                tcls::DQS_E,
                                &[
                                    cell.delta(0, 4),
                                    cell.delta(0, 3),
                                    cell.delta(0, 2),
                                    cell.delta(0, 1),
                                    cell.delta(0, 0),
                                ],
                            );
                            self.egrid[cell.delta(0, 4)].region_root[rslots::DQS] = cell;
                        }
                        self.egrid[cell].region_root[rslots::DQS] = cell;
                        if self.chip.rows[cell.row].kind == RowKind::Ebr {
                            self.egrid[cell.delta(-1, 0)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row - 1].kind == RowKind::Ebr {
                            self.egrid[cell.delta(-1, -1)].region_root[rslots::DQS] = cell;
                        } else if self.chip.rows[cell.row - 1].kind == RowKind::Dsp
                            || self.chip.rows[cell.row - 2].kind == RowKind::Dsp
                        {
                            self.egrid[cell.delta(0, -2)].region_root[rslots::DQS] = cell;
                        } else {
                            self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell;
                        }
                    }
                }
            }
            if rd.kind == RowKind::Ebr {
                if cell.row < self.chip.row_clk {
                    self.egrid.add_tile(
                        cell.delta(-1, 0),
                        tcls::IO_E_EBR_S,
                        &[
                            cell.delta(-1, 0),
                            cell.delta(-2, 0),
                            cell.delta(-3, 0),
                            cell.delta(-4, 0),
                        ],
                    );
                    if matches!(
                        rd.io[DirH::E],
                        IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        self.egrid.add_tile(
                            cell.delta(-1, 0),
                            tcls::DQS_E_EBR_S,
                            &[
                                cell.delta(0, 1),
                                cell.delta(-1, 0),
                                cell.delta(-2, 0),
                                cell.delta(-3, 0),
                                cell.delta(-4, 0),
                            ],
                        );
                        self.egrid[cell.delta(-1, 0)].region_root[rslots::DQS] = cell.delta(-1, 0);
                        self.egrid[cell.delta(0, -3)].region_root[rslots::DQS] = cell.delta(-1, 0);
                        self.egrid[cell.delta(0, 1)].region_root[rslots::DQS] = cell.delta(-1, 0);
                    }
                } else {
                    self.egrid.add_tile(
                        cell.delta(-1, 0),
                        tcls::IO_E_EBR_N,
                        &[
                            cell.delta(-4, 0),
                            cell.delta(-3, 0),
                            cell.delta(-2, 0),
                            cell.delta(-1, 0),
                        ],
                    );
                    if matches!(
                        rd.io[DirH::E],
                        IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        self.egrid.add_tile(
                            cell.delta(-1, 0),
                            tcls::DQS_E_EBR_N,
                            &[
                                cell,
                                cell.delta(-4, 0),
                                cell.delta(-3, 0),
                                cell.delta(-2, 0),
                                cell.delta(-1, 0),
                            ],
                        );
                        self.egrid[cell.delta(-1, 0)].region_root[rslots::DQS] = cell.delta(-1, 0);
                        self.egrid[cell.delta(0, -4)].region_root[rslots::DQS] = cell.delta(-1, 0);
                        self.egrid[cell].region_root[rslots::DQS] = cell.delta(-1, 0);
                    }
                }
            }
        }
    }

    pub(super) fn fill_clk_ecp4(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            self.egrid[cell].region_root[rslots::GLOBAL] = self.chip.clk_root();
            let row_pclk = self.chip.rows_pclk[cell.row];
            let row_split = if row_pclk < self.chip.row_clk {
                row_pclk + 1
            } else {
                row_pclk
            };
            self.egrid[cell].region_root[rslots::PCLK_LEAF] =
                cell.with_row(cell.row.snap_to_side_of(row_split));
            self.egrid[cell].region_root[rslots::PCLK_ROW] = cell
                .with_row(row_pclk)
                .with_col(cell.col.snap_to_side_of(self.chip.col_clk));
        }
        for &row in self.chip.rows_pclk.values() {
            let row_split = if row < self.chip.row_clk {
                row + 1
            } else {
                row
            };
            for cell in self.egrid.row(self.die, row) {
                self.egrid.add_tile(
                    cell,
                    tcls::PCLK_DRIVER,
                    &[cell, cell.with_row(row_split - 1), cell.with_row(row_split)],
                );
            }
            self.egrid.add_tile_single(
                self.die.cell(self.chip.col_clk - 1, row),
                tcls::PCLK_ROW_DRIVER_W,
            );
            self.egrid.add_tile_single(
                self.die.cell(self.chip.col_clk, row),
                tcls::PCLK_ROW_DRIVER_E,
            );
        }

        let cell_serdes = self
            .chip
            .special_loc
            .get(&SpecialLocKey::SerdesSingle)
            .or(self.chip.special_loc.get(&SpecialLocKey::SerdesDouble))
            .or(self.chip.special_loc.get(&SpecialLocKey::SerdesTriple))
            .copied()
            .unwrap();

        let num_quads = if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesSingle)
        {
            1
        } else if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesDouble)
        {
            2
        } else if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesTriple)
        {
            3
        } else {
            unreachable!()
        };

        let (kind_w, kind_e, kind_s, kind_n) = match num_quads {
            1 => (tcls::CLK_W_S, tcls::CLK_E_S, tcls::CLK_S_S, tcls::CLK_N_S),
            2 => (tcls::CLK_W_M, tcls::CLK_E_M, tcls::CLK_S_M, tcls::CLK_N_M),
            3 => (tcls::CLK_W_L, tcls::CLK_E_L, tcls::CLK_S_L, tcls::CLK_N_L),
            _ => unreachable!(),
        };

        let has_bank0 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(0));
        let cell = self.chip.clk_emid(Dir::N);
        let mut tcells = if !has_bank0 {
            cell.delta(-5, 0).cells_e(10)
        } else {
            cell.delta(-9, 0).cells_e(18)
        };
        if has_bank0 {
            tcells.extend([
                self.chip.special_loc[&SpecialLocKey::Bc(1)].delta(1, 0),
                self.chip.special_loc[&SpecialLocKey::Bc(2)].delta(1, 0),
            ]);
        }
        tcells.extend([
            self.chip.clk_root(),
            self.chip.bel_pll(DirHV::NW, 0).cell,
            self.chip.bel_pll(DirHV::NE, 0).cell,
        ]);
        for i in 0..4 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::N, i)) {
                tcells.push(cell);
            }
        }
        {
            let mut quads_w = vec![];
            let mut quads_e = vec![];
            for (col, cd) in &self.chip.columns {
                let quad = cell.with_cr(col, self.chip.row_n());
                if cd.io[DirV::N] == IoGroupKind::QuadDqs {
                    if col < self.chip.col_clk {
                        quads_w.push(quad);
                    } else {
                        quads_e.push(quad);
                    }
                }
            }
            quads_e.reverse();
            tcells.extend(quads_w);
            tcells.extend(quads_e);
        }
        self.egrid.add_tile(cell, kind_n, &tcells);
        if has_bank0 {
            let col0: ColId = self.chip.col_w() + 5;
            let col1: ColId = self.chip.special_loc[&SpecialLocKey::Bc(1)].col + 3;
            let col3: ColId = self.chip.special_loc[&SpecialLocKey::Bc(2)].col - 1;
            let col4: ColId = self.chip.col_e() - 5;
            let root0 = self.chip.special_loc[&SpecialLocKey::Bc(1)].step(DirH::E, 1);
            let root1 = self.chip.clk_emid(Dir::N).step(DirH::W, 1);
            let root2 = self.chip.clk_emid(Dir::N);
            let root3 = self.chip.special_loc[&SpecialLocKey::Bc(2)].step(DirH::E, 1);
            for cell in self.egrid.row(self.die, self.chip.row_n()) {
                if self.chip.is_col_io(cell.col) {
                    continue;
                }
                let root = if cell.col < col0 {
                    // for PLL
                    root1
                } else if cell.col < col1 {
                    root0
                } else if cell.col < self.chip.col_clk {
                    root1
                } else if cell.col < col3 {
                    root2
                } else if cell.col < col4 {
                    root3
                } else {
                    // for PLL
                    root2
                };
                self.egrid[cell].region_root[rslots::ECLK] = root;
            }
        } else {
            let root1 = self.chip.clk_emid(Dir::N).step(DirH::W, 1);
            let root2 = self.chip.clk_emid(Dir::N);
            for cell in self.egrid.row(self.die, self.chip.row_n()) {
                if self.chip.is_col_io(cell.col) {
                    continue;
                }
                let root = if cell.col < self.chip.col_clk {
                    root1
                } else {
                    root2
                };
                self.egrid[cell].region_root[rslots::ECLK] = root;
            }
        }

        for (edge, kind) in [(DirH::W, kind_w), (DirH::E, kind_e)] {
            let cell = self.chip.clk_emid(edge);
            let mut tcells = cell.delta(0, -5).cells_n(10);
            tcells.extend([
                self.chip.clk_root(),
                self.chip.bel_pll(DirV::S & edge, 0).cell,
                self.chip.bel_pll(DirV::N & edge, 0).cell,
            ]);
            for i in 0..4 {
                if let Some(&cell) = self
                    .chip
                    .special_loc
                    .get(&SpecialLocKey::PclkIn(edge.into(), i))
                {
                    tcells.push(cell);
                }
            }
            let mut quads_s = vec![];
            let mut quads_n = vec![];
            for (row, rd) in &self.chip.rows {
                let quad = match rd.io[edge] {
                    IoGroupKind::QuadDqs => Some(cell.with_cr(self.chip.col_edge(edge), row)),
                    IoGroupKind::QuadEbrDqs | IoGroupKind::EbrDqs => Some(cell.with_cr(
                        match edge {
                            DirH::W => self.chip.col_w() + 1,
                            DirH::E => self.chip.col_e() - 1,
                        },
                        row,
                    )),
                    _ => None,
                };
                if let Some(quad) = quad {
                    if row < self.chip.row_clk {
                        quads_s.push(quad);
                    } else {
                        quads_n.push(quad);
                    }
                }
            }
            quads_n.reverse();
            tcells.extend(quads_s);
            tcells.extend(quads_n);
            if kind == tcls::CLK_E_L {
                tcells.push(cell_serdes);
            }
            self.egrid.add_tile(cell, kind, &tcells);
        }

        for (col, dx) in [(self.chip.col_w(), 1), (self.chip.col_e(), -1)] {
            for cell in self.egrid.column(self.die, col) {
                let root = cell.with_row(cell.row.snap_to_side_of(self.chip.row_clk));
                self.egrid[cell].region_root[rslots::ECLK] = root;
                match self.chip.rows[cell.row].kind {
                    RowKind::Ebr => {
                        for i in 1..5 {
                            self.egrid[cell.delta(dx * i, 0)].region_root[rslots::ECLK] = root;
                        }
                    }
                    RowKind::Dsp => {
                        for i in 1..3 {
                            self.egrid[cell.delta(dx * i, 0)].region_root[rslots::ECLK] = root;
                        }
                    }
                    _ => (),
                }
            }
        }

        let cell = self.chip.clk_emid(Dir::S);
        let mut tcells = vec![cell.delta(-1, 0), cell, cell_serdes];
        for i in 0..4 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::S, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(cell, kind_s, &tcells);

        let row_s = self
            .chip
            .rows
            .ids()
            .rev()
            .filter(|&row| row < self.chip.row_clk)
            .find(|&row| self.chip.rows[row].kind == RowKind::Ebr)
            .unwrap();
        let row_n = self
            .chip
            .rows
            .ids()
            .filter(|&row| row >= self.chip.row_clk)
            .find(|&row| self.chip.rows[row].kind == RowKind::Ebr)
            .unwrap();
        let cell = self.chip.clk_root();
        self.egrid.add_tile(
            cell,
            tcls::CLK_ROOT,
            &[
                cell.with_cr(self.chip.col_clk - 1, row_s),
                cell.with_cr(self.chip.col_clk, row_s),
                cell.with_cr(self.chip.col_clk - 1, row_n),
                cell.with_cr(self.chip.col_clk, row_n),
                cell,
                self.chip.clk_emid(Dir::W),
                self.chip.clk_emid(Dir::E),
                self.chip.clk_emid(Dir::S),
                self.chip.clk_emid(Dir::N),
            ],
        );
    }

    pub(super) fn fill_config_ecp4(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut tcells = cell.cells_e(6);
        let row_dsp = self
            .chip
            .rows
            .ids()
            .rev()
            .filter(|&row| row < self.chip.row_clk)
            .find(|&row| self.chip.rows[row].kind == RowKind::Dsp)
            .unwrap();
        tcells.extend([
            self.chip.clk_emid(Dir::W).step(Dir::S, 1),
            self.chip.clk_emid(Dir::W),
            self.chip.clk_emid(Dir::S).step(Dir::W, 1),
            self.chip.clk_emid(Dir::S),
            self.die.cell(self.chip.col_clk - 1, row_dsp),
        ]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);
    }
}
