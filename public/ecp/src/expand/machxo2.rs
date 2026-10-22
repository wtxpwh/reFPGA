use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::DieIdExt,
};

use crate::{
    chip::{ChipKind, IoGroupKind, MachXo2Kind, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{rslots, tcls, wires},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_pll_machxo2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let kind = match loc.quad.h {
                DirH::W => tcls::PLL_W,
                DirH::E => tcls::PLL_E,
            };
            match loc.quad.h {
                DirH::W => {
                    self.egrid.add_tile(
                        cell,
                        kind,
                        &[
                            cell,
                            cell.delta(1, 0),
                            self.die.cell(self.chip.col_clk - 1, self.chip.row_s()),
                        ],
                    );
                }
                DirH::E => {
                    self.egrid.add_tile(
                        cell,
                        kind,
                        &[
                            cell,
                            cell.delta(-1, 0),
                            self.die.cell(self.chip.col_clk - 1, self.chip.row_s()),
                        ],
                    );
                }
            }
            self.add_io_conn(
                cell.wire(wires::IO_PLL_IN),
                SpecialIoKey::Pll(PllPad::PllIn0, loc),
            );
            self.add_io_conn(
                cell.wire(wires::IO_PLL_FB),
                SpecialIoKey::Pll(PllPad::PllFb, loc),
            );
        }
    }

    pub(super) fn fill_ebr_machxo2(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if self.chip.columns[cell.col].io_n == IoGroupKind::Ebr {
                self.egrid.add_tile_e(cell, tcls::EBR_N, 3);
            }
        }
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                let is_ebr_start = if cell.col < self.chip.col_clk {
                    cell.col.to_idx() % 3 == (self.chip.col_clk.to_idx() - 1) % 3
                        && cell.col != self.chip.col_clk - 1
                } else {
                    cell.col.to_idx() % 3 == self.chip.columns.len() % 3
                };
                if is_ebr_start {
                    self.egrid.add_tile_e(cell, tcls::EBR, 3);
                }
            }
        }
    }

    pub(super) fn fill_io_machxo2(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            if matches!(self.chip.rows[cell.row].kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if matches!(self.chip.rows[cell.row].kind, RowKind::Plc | RowKind::Fplc) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
            }
        }

        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        let has_slewrate = matches!(
            self.chip.kind,
            ChipKind::MachXo2(
                MachXo2Kind::MachXo3Lfp | MachXo2Kind::MachXo3D | MachXo2Kind::MachNx
            )
        );
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            if has_slewrate && bank == 1 {
                self.egrid
                    .add_tile(cell, tcls::BCSR_E, &[cell, cell.delta(0, -1)]);
            } else if has_slewrate {
                self.egrid.add_tile_single(
                    cell,
                    match bank {
                        0 => tcls::BCSR_N,
                        2 => tcls::BCSR_S,
                        3..=5 => tcls::BCSR_W,
                        _ => unreachable!(),
                    },
                );
            } else if bank == 0 && !is_smol {
                self.egrid.add_tile_single(cell, tcls::BC_N);
            } else {
                self.egrid.add_tile_single(cell, tcls::BC);
            }
        }

        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            let kind = match self.chip.rows[cell.row].io_w {
                IoGroupKind::Double => tcls::IO_W2,
                IoGroupKind::Quad => tcls::IO_W4,
                IoGroupKind::QuadI3c => tcls::IO_W4_I3C,
                _ => continue,
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            let kind = match self.chip.rows[cell.row].io_e {
                IoGroupKind::Double => tcls::IO_E2,
                IoGroupKind::Quad => tcls::IO_E4,
                _ => continue,
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            let kind = match self.chip.columns[cell.col].io_s {
                IoGroupKind::Double if is_smol => tcls::SIO_S2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse if is_smol => tcls::SIO_S4,
                IoGroupKind::Double if !is_smol => tcls::IO_S2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse if !is_smol => tcls::IO_S4,
                _ => continue,
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            let kind = match self.chip.columns[cell.col].io_n {
                IoGroupKind::Double if is_smol => tcls::SIO_N2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse if is_smol => tcls::SIO_N4,
                IoGroupKind::Double if !is_smol => tcls::IO_N2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse if !is_smol => tcls::IO_N4,
                _ => continue,
            };
            self.egrid.add_tile_single(cell, kind);
        }
    }

    pub(super) fn fill_clk_machxo2(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            self.egrid[cell].region_root[rslots::GLOBAL] = self.chip.clk_root();
        }
        for row in [self.chip.row_s(), self.chip.row_n()] {
            for cell in self.egrid.row(self.die, row) {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk - 1);
            }
        }
        let ebr_rows = Vec::from_iter(
            self.chip
                .rows
                .ids()
                .filter(|&row| self.chip.rows[row].kind == RowKind::Ebr),
        );
        let cell = self.die.cell(self.chip.col_clk, self.chip.row_clk);
        match ebr_rows.len() {
            0 => {
                self.egrid.add_tile(
                    cell,
                    tcls::CLK_ROOT_0EBR,
                    &[
                        cell.with_col(self.chip.col_w()).delta(0, -2),
                        cell.with_col(self.chip.col_w()).delta(0, -1),
                        cell.with_col(self.chip.col_e()).delta(0, -2),
                        cell.with_col(self.chip.col_e()).delta(0, -1),
                        cell.with_row(self.chip.row_s()).delta(-1, 0),
                        cell.with_row(self.chip.row_s()),
                        cell.with_row(self.chip.row_n()).delta(-1, 0),
                        cell.with_row(self.chip.row_n()),
                    ],
                );
            }
            1 => {
                self.egrid.add_tile_part(
                    cell,
                    tcls::CLK_ROOT_1EBR,
                    &[
                        Some(cell.delta(-4, 0)),
                        Some(cell.delta(-2, 0)),
                        Some(cell.delta(-1, 0)),
                        Some(cell),
                        Some(cell.with_col(self.chip.col_w())),
                        Some(cell.with_col(self.chip.col_e())),
                        Some(cell.with_row(self.chip.row_s()).delta(-1, 0)),
                        Some(cell.with_row(self.chip.row_n()).delta(-1, 0)),
                        self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                        self.chip.try_bel_pll(DirHV::NE, 0).map(|bcrd| bcrd.cell),
                    ],
                );
            }
            2 => {
                self.egrid.add_tile(
                    cell,
                    tcls::CLK_ROOT_2EBR,
                    &[
                        cell.delta(-4, 0),
                        cell.delta(-2, 0),
                        cell.delta(-1, 0),
                        cell,
                        cell.with_col(self.chip.col_w()),
                        cell.with_col(self.chip.col_e()),
                        cell.with_row(self.chip.row_s()).delta(-1, 0),
                        cell.with_row(self.chip.row_n()).delta(-1, 0),
                        cell.with_row(ebr_rows[0]).delta(-2, 0),
                        cell.with_row(ebr_rows[0]).delta(-1, 0),
                        cell.with_row(ebr_rows[0]),
                        cell.with_row(ebr_rows[0]).with_col(self.chip.col_w()),
                        cell.with_row(ebr_rows[0]).with_col(self.chip.col_e()),
                        self.chip.bel_pll(DirHV::NW, 0).cell,
                        self.chip.bel_pll(DirHV::NE, 0).cell,
                    ],
                );
            }
            3 => {
                self.egrid.add_tile(
                    cell,
                    tcls::CLK_ROOT_3EBR,
                    &[
                        cell.delta(-4, 0),
                        cell.delta(-1, 0),
                        cell,
                        cell.with_col(self.chip.col_w()),
                        cell.with_col(self.chip.col_e()),
                        cell.with_row(self.chip.row_s()).delta(-1, 0),
                        cell.with_row(self.chip.row_n()).delta(-1, 0),
                        cell.with_row(ebr_rows[0]).delta(-2, 0),
                        cell.with_row(ebr_rows[0]).delta(-1, 0),
                        cell.with_row(ebr_rows[0]),
                        cell.with_row(ebr_rows[2]).delta(-2, 0),
                        cell.with_row(ebr_rows[2]).delta(-1, 0),
                        cell.with_row(ebr_rows[2]),
                        cell.with_row(ebr_rows[0]).with_col(self.chip.col_w()),
                        cell.with_row(ebr_rows[0]).with_col(self.chip.col_e()),
                        self.chip.bel_pll(DirHV::NW, 0).cell,
                        self.chip.bel_pll(DirHV::NE, 0).cell,
                    ],
                );
            }
            _ => unreachable!(),
        }
        for i in 0..3 {
            self.add_io_conn(
                self.chip.clk_root().wire(wires::IO_GCLK_W[i]),
                SpecialIoKey::Clock(Dir::W, i as u8),
            );
        }
        self.add_io_conn(
            self.chip.clk_root().wire(wires::IO_GCLK_E[0]),
            SpecialIoKey::Clock(Dir::E, 0),
        );
        for i in 0..2 {
            self.add_io_conn(
                self.chip.clk_root().wire(wires::IO_GCLK_S[i]),
                SpecialIoKey::Clock(Dir::S, i as u8),
            );
        }
        for i in 0..2 {
            self.add_io_conn(
                self.chip.clk_root().wire(wires::IO_GCLK_N[i]),
                SpecialIoKey::Clock(Dir::N, i as u8),
            );
        }
        if !ebr_rows.is_empty() {
            for (edge, tcls) in [(DirV::S, tcls::CLK_S), (DirV::N, tcls::CLK_N)] {
                let cell = cell.with_row(self.chip.row_edge(edge)).delta(-1, 0);
                self.egrid.add_tile_part(
                    cell,
                    tcls,
                    &[
                        Some(cell),
                        Some(self.chip.clk_root()),
                        self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                        self.chip.try_bel_pll(DirHV::NE, 0).map(|bcrd| bcrd.cell),
                    ],
                );
            }
            self.egrid
                .add_tile_single(cell.with_col(self.chip.col_w()), tcls::CLK_W);
            if self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachXo2) {
                self.egrid
                    .add_tile_sn(cell.with_col(self.chip.col_e()), tcls::CLK_E_DQS, 2, 5);
            } else {
                self.egrid
                    .add_tile_single(cell.with_col(self.chip.col_e()), tcls::CLK_E);
            }
        }
        for (edge, kind) in [(DirV::S, tcls::DQSDLL_S), (DirV::N, tcls::DQSDLL_N)] {
            let Some(&cell) = self
                .chip
                .special_loc
                .get(&SpecialLocKey::DqsDll(Dir::V(edge)))
            else {
                continue;
            };
            self.egrid.add_tile_single(cell, kind);
        }
    }

    pub(super) fn fill_config_machxo2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        match self.chip.kind {
            ChipKind::MachXo2(
                MachXo2Kind::MachXo2 | MachXo2Kind::MachXo3L | MachXo2Kind::MachXo3Lfp,
            ) => {
                self.egrid.add_tile_e(cell, tcls::CONFIG, 4);
            }
            ChipKind::MachXo2(MachXo2Kind::MachXo3D | MachXo2Kind::MachNx) => {
                self.egrid.add_tile_e(cell, tcls::CONFIG_XO3D, 10);
            }
            _ => unreachable!(),
        }
    }
}
