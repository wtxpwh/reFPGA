use prjcombine_entity::{EntityId, EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    db::IntDb,
    dir::{Dir, DirH, DirHV, DirV},
    grid::{CellCoord, ColId, DieId, DieIdExt, Rect, RowId, WireCoord, builder::GridBuilder},
};

use crate::{
    chip::{
        Chip, ChipKind, IoGroupKind, MachXo2Kind, PllLoc, PllPad, RowKind, SpecialIoKey,
        SpecialLocKey,
    },
    defs::{bslots, ccls, rslots, tcls, tiledata, tslots, wires},
    expanded::ExpandedDevice,
};

struct Expander<'a, 'b> {
    chip: &'b Chip,
    die: DieId,
    egrid: &'a mut GridBuilder<'b>,
    bel_holes: Vec<Rect>,
    frame_len: usize,
    frames_num: usize,
    clk_frame: usize,
    col_frame: EntityVec<ColId, usize>,
    col_term_frame: EntityPartVec<ColId, usize>,
    row_bit: EntityVec<RowId, usize>,
    row_ebr_bit: EntityPartVec<RowId, usize>,
}

impl Expander<'_, '_> {
    fn is_in_bel_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.bel_holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    fn fill_plc(&mut self) {
        for (row, rd) in &self.chip.rows {
            let tcls_plc = match rd.kind {
                RowKind::Plc => tcls::PLC,
                RowKind::Fplc => tcls::FPLC,
                _ => continue,
            };
            for cell in self.egrid.row(self.die, row) {
                if (cell.col == self.chip.col_w() || cell.col == self.chip.col_e())
                    && self.chip.kind != ChipKind::Crosslink
                {
                    continue;
                }
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_PLC);
                self.egrid.add_tile_single(cell, tcls_plc);
            }
        }
    }

    fn add_io_conn(&mut self, wire: WireCoord, key: SpecialIoKey) {
        let io = self.chip.special_io[&key];
        let ioidx = bslots::IO.index_of(io.slot).unwrap();
        self.egrid
            .extra_conns
            .insert(wire, io.wire(wires::OUT_IO_PADDI[ioidx]));
    }

    fn fill_config_scm(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut cells = cell.cells_e(12);
        cells.push(cell.with_cr(self.chip.col_clk, self.chip.row_clk - 5));
        self.egrid.add_tile(cell, tcls::CONFIG, &cells);
        for dx in 0..12 {
            self.egrid.add_tile_single(cell.delta(dx, 0), tcls::INT_EBR);
        }
        self.bel_holes.push(cell.rect(12, 1));
    }

    fn fill_config_ecp(&mut self) {
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

    fn fill_config_ecp2(&mut self) {
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

    fn fill_config_xp2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.egrid.add_tile_single(cell, tcls::OSC);
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.egrid.add_tile_single(cell, tcls::CONFIG);
    }

    fn fill_config_ecp3(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.bel_holes.push(cell.rect(12, 1));
        let mut tcells = cell.cells_e(13);
        tcells.push(cell.with_cr(self.chip.col_clk - 2, self.chip.row_clk));
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

    fn fill_pll_scm(&mut self) {
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

        let cell = self.die.cell(self.chip.col_w(), self.chip.row_n() - 12);
        self.egrid.add_tile(
            cell,
            tcls::PLL_NW,
            &[cell, cell.delta(1, 0), cell.delta(0, -1)],
        );

        let cell = self.die.cell(self.chip.col_e(), self.chip.row_n() - 12);
        self.egrid.add_tile(
            cell,
            tcls::PLL_NE,
            &[cell, cell.delta(-1, 0), cell.delta(0, -1)],
        );
    }

    fn fill_pll_ecp(&mut self) {
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

    fn fill_pll_xp(&mut self) {
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

    fn fill_pll_ecp2(&mut self) {
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

    fn fill_pll_xp2(&mut self) {
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

    fn fill_pll_ecp3(&mut self) {
        let is_a = self.chip.kind == ChipKind::Ecp3A;
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
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_clk);
            self.bel_holes.push(cell.delta(-3, 0).rect(6, 1));
        }
    }

    fn fill_pll_machxo2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            let kind = match loc.quad.h {
                DirH::W => tcls::PLL_W,
                DirH::E => tcls::PLL_E,
            };
            match loc.quad.h {
                DirH::W => {
                    self.egrid.add_tile(cell, kind, &[cell, cell.delta(1, 0)]);
                }
                DirH::E => {
                    self.egrid.add_tile(cell, kind, &[cell, cell.delta(-1, 0)]);
                }
            }
        }
    }

    fn fill_pll_ecp4(&mut self) {
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
            match loc.quad.v {
                DirV::S => {
                    self.egrid.add_tile(
                        cell,
                        kind,
                        &[
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
                    );
                }
                DirV::N => {
                    self.egrid.add_tile_sn(cell, kind, 3, 4);
                }
            }
        }
    }

    fn fill_pll_ecp5(&mut self) {
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
                        ],
                    );
                }
                DirV::N => {
                    self.egrid.add_tile(cell, kind, &[cell, cell.delta(0, -1)]);
                }
            }
        }
    }

    fn fill_serdes_scm(&mut self) {
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
        let row = self.chip.row_n() - 12;
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
                let kind = if cell.col < self.chip.col_clk {
                    tcls::SERDES_W
                } else {
                    tcls::SERDES_E
                };
                self.egrid.add_tile_e(cell, kind, 7);
            }
        }
    }

    fn fill_serdes_ecp2(&mut self) {
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

    fn fill_serdes_ecp3(&mut self) {
        let mut first = true;
        let mut last = None;
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.chip.columns[cell.col].io_s == IoGroupKind::Serdes {
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

    fn fill_serdes_ecp4(&mut self) {
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

    fn fill_ebr_scm(&mut self) {
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
                    let kind = if cell.col < self.chip.col_clk {
                        tcls::EBR_W
                    } else {
                        tcls::EBR_E
                    };
                    self.egrid.add_tile_e(cell, kind, 2);
                    if cell.row != self.chip.row_n() - 12 {
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

    fn fill_ebr_ecp(&mut self) {
        let ebr_width = match self.chip.kind {
            ChipKind::Ecp | ChipKind::Xp => 2,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3 | ChipKind::Ecp3A => {
                3
            }
            _ => unreachable!(),
        };
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    let int_kind = if ebr_width == 3
                        && (cell.col == self.chip.col_w() || cell.col == self.chip.col_e())
                    {
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
                if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                    if matches!(self.chip.kind, ChipKind::Ecp3 | ChipKind::Ecp3A) && rd.sclk_break {
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

    fn fill_dsp_ecp(&mut self) {
        let dsp_width = match self.chip.kind {
            ChipKind::Ecp => 8,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3 | ChipKind::Ecp3A => {
                9
            }
            _ => unreachable!(),
        };
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Dsp {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    let int_kind = if dsp_width == 9
                        && (cell.col == self.chip.col_w() || cell.col == self.chip.col_e())
                    {
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
                if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                    continue;
                }
                if cell.col.to_idx() % dsp_width == 1 {
                    self.egrid.add_tile_e(cell, tcls::DSP, dsp_width);
                }
            }
        }
    }

    fn fill_ebr_machxo2(&mut self) {
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

    fn fill_ebr_ecp4(&mut self) {
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

    fn fill_ebr_crosslink(&mut self) {
        let mut col_start_w = self.chip.col_w();
        let mut col_end_e = self.chip.col_e() + 1usize;
        let col_end_w = self.chip.col_clk;
        let col_start_e = self.chip.col_clk + 1usize;
        while col_start_w.to_idx() % 10 != col_end_w.to_idx() % 10 {
            col_start_w += 1;
        }
        while col_start_e.to_idx() % 10 != col_end_e.to_idx() % 10 {
            col_end_e -= 1;
        }

        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                self.egrid.add_tile_single(cell, tcls::INT_EBR);
                if (cell.col >= col_start_w
                    && cell.col < col_end_w
                    && cell.col.to_idx() % 10 == col_start_w.to_idx() % 10)
                    || (cell.col >= col_start_e
                        && cell.col < col_end_e
                        && cell.col.to_idx() % 10 == col_start_e.to_idx() % 10)
                {
                    self.egrid.add_tile_e(cell, tcls::EBR, 9);
                }
            }
        }
    }

    fn fill_int_io(&mut self, cell: CellCoord) {
        match self.chip.kind {
            ChipKind::Ecp | ChipKind::Xp => {
                if cell.row == self.chip.row_s() || cell.row == self.chip.row_n() {
                    self.egrid.add_tile_single(cell, tcls::INT_IO_SN);
                } else {
                    self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
                }
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                if cell.row == self.chip.row_n() {
                    self.egrid.add_tile_single(cell, tcls::INT_IO_N);
                } else if cell.row == self.chip.row_s() {
                    self.egrid.add_tile_single(cell, tcls::INT_IO_S);
                } else {
                    self.egrid.add_tile_single(cell, tcls::INT_IO_WE);
                }
            }
            _ => unreachable!(),
        }
    }

    fn fill_io_scm(&mut self) {
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

    fn fill_io_ecp(&mut self) {
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
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
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
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
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
            let cell = self.chip.bel_dqsdll_ecp2(DirH::W).cell;
            self.egrid.add_tile_part(
                cell,
                tcls::CLK_W,
                &[
                    Some(cell),
                    self.chip.try_bel_pll(DirHV::SW, 0).map(|bcrd| bcrd.cell),
                    self.chip.try_bel_pll(DirHV::NW, 0).map(|bcrd| bcrd.cell),
                ],
            );
            let cell = self.chip.bel_dqsdll_ecp2(DirH::E).cell;
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

    fn fill_io_ecp3(&mut self) {
        let is_a = self.chip.kind == ChipKind::Ecp3A;
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if !self.is_in_bel_hole(cell) {
                self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            }
            if self.chip.columns[cell.col].io_s == IoGroupKind::Quad {
                self.egrid.add_tile_e(cell, tcls::XSIO_S, 3);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            match (
                self.chip.columns[cell.col].io_n,
                self.chip.columns[cell.col].bank_n,
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
            match self.chip.rows[cell.row].io_w {
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
                self.chip.rows[cell.row].io_e,
                self.chip.rows[cell.row].bank_e,
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

    fn fill_io_machxo2(&mut self) {
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

    fn fill_io_ecp4(&mut self) {
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
        for col in [
            self.chip.col_w() + 2,
            self.chip.col_w() + 3,
            self.chip.col_e() - 3,
            self.chip.col_e() - 2,
        ] {
            for (row, kind) in [
                (self.chip.row_s(), tcls::DDRDLL_S),
                (self.chip.row_n(), tcls::DDRDLL_N),
            ] {
                let cell = self.die.cell(col, row);
                self.egrid.add_tile_single(cell, kind);
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
            if self.chip.columns[cell.col].io_n != IoGroupKind::None {
                self.egrid.add_tile_e(cell, tcls::IO_N, 4);
            }
            if self.chip.columns[cell.col].io_n == IoGroupKind::QuadDqs {
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
                rd.io_w,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                if rd.kind == RowKind::Dsp {
                    assert!(cell.row >= self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_W_DSP_N,
                        &[cell.delta(0, 1), cell, cell.delta(2, 0), cell.delta(1, 0)],
                    );
                    if rd.io_w == IoGroupKind::QuadDqs {
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
                    if rd.io_w == IoGroupKind::QuadDqs {
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
                    if matches!(rd.io_w, IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs) {
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
                    if matches!(rd.io_w, IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs) {
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
                rd.io_e,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                if rd.kind == RowKind::Dsp {
                    assert!(cell.row >= self.chip.row_clk);
                    self.egrid.add_tile(
                        cell,
                        tcls::IO_E_DSP_N,
                        &[cell.delta(0, 1), cell, cell.delta(-2, 0), cell.delta(-1, 0)],
                    );
                    if rd.io_e == IoGroupKind::QuadDqs {
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
                    if rd.io_e == IoGroupKind::QuadDqs {
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
                    if matches!(rd.io_e, IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs) {
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
                    if matches!(rd.io_e, IoGroupKind::EbrDqs | IoGroupKind::QuadEbrDqs) {
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

    fn fill_io_ecp5(&mut self) {
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

    fn fill_io_crosslink(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            if matches!(key, SpecialLocKey::Bc(_)) {
                self.egrid.add_tile_single(cell, tcls::BC);
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_S);
            match self.chip.columns[cell.col].io_s {
                IoGroupKind::Quad => {
                    self.egrid.add_tile_e(cell, tcls::IO_S4, 4);
                }
                IoGroupKind::Single => {
                    if cell.col.to_idx() % 2 == 0 {
                        self.egrid.add_tile_single(cell, tcls::IO_S1A);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::IO_S1B);
                    }
                }
                _ => (),
            }
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            self.egrid.add_tile_single(cell, tcls::INT_IO_N);
            if self.chip.columns[cell.col].io_n == IoGroupKind::Mipi {
                let mut tcells = cell.cells_e(24);
                tcells.extend([
                    self.chip
                        .get_io_cell(SpecialIoKey::MipiClk(if cell.col < self.chip.col_clk {
                            DirH::W
                        } else {
                            DirH::E
                        })),
                    self.chip.get_io_cell(SpecialIoKey::Clock(
                        Dir::S,
                        if cell.col < self.chip.col_clk { 4 } else { 5 },
                    )),
                ]);
                self.egrid.add_tile(
                    cell,
                    if cell.col < self.chip.col_clk {
                        tcls::MIPI_W
                    } else {
                        tcls::MIPI_E
                    },
                    &tcells,
                );
            }
        }
    }

    fn fill_region_pclk(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            if matches!(
                self.chip.kind,
                ChipKind::MachXo | ChipKind::MachXo2(..) | ChipKind::Crosslink
            ) {
                self.egrid[cell].region_root[rslots::PCLK] = self.chip.clk_root();
            } else {
                let col_pclk = if cell.col < self.chip.col_clk {
                    self.chip.col_w()
                } else {
                    self.chip.col_e()
                };
                let row_pclk = if cell.row < self.chip.row_clk {
                    self.chip.row_s()
                } else {
                    self.chip.row_n()
                };
                self.egrid[cell].region_root[rslots::PCLK] = self.die.cell(col_pclk, row_pclk);
            }
        }
    }

    fn fill_clk_scm(&mut self) {
        for (bank, kind) in [
            (7, tcls::CLK_W),
            (2, tcls::CLK_E),
            (1, tcls::CLK_N),
            (5, tcls::CLK_SW),
            (4, tcls::CLK_SE),
        ] {
            let cell = self.chip.special_loc[&SpecialLocKey::Bc(bank)];
            let cells = if matches!(bank, 2 | 7) {
                [cell.delta(0, -1), cell]
            } else {
                [cell.delta(-1, 0), cell]
            };
            self.egrid.add_tile(cell, kind, &cells);
        }
        let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
        self.egrid
            .add_tile(cell, tcls::CLK_S, &[cell.delta(-1, 0), cell]);

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_clk);
        self.egrid.add_tile(
            cell,
            tcls::CLK_ROOT,
            &[
                cell.delta(-1, -5),
                cell.delta(0, -5),
                cell.delta(-1, 4),
                cell.delta(0, 4),
            ],
        );
    }

    fn fill_clk_ecp(&mut self) {
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
        self.egrid.add_tile_part(
            self.die.cell(self.chip.col_clk, self.chip.row_clk),
            kind,
            &cells,
        );
    }

    fn fill_clk_ecp2(&mut self) {
        let mut rows_sclk = EntityVec::new();
        let mut row_src = self.chip.row_s();
        for (row, rd) in &self.chip.rows {
            if rd.sclk_break {
                row_src = row;
            }
            rows_sclk.push(row_src);
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
                let cell_src = cell_src.with_row(rows_sclk[cell.row]);
                self.egrid[cell].region_root[rslots::RSCLK[i]] = cell_src;
                if cell == cell_src {
                    let mut cell = cell;
                    if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                        cell.row += 7;
                    }
                    self.egrid.add_tile_single(cell, tiledata::SCLK_SOURCE[i]);
                }
            }
        }

        for (row, rd) in &self.chip.rows {
            if !matches!(rd.kind, RowKind::Io | RowKind::Ebr | RowKind::Dsp) {
                continue;
            }
            let mut prev = None;
            for mut cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    if cell.row == self.chip.row_s() {
                        cell.row += 7;
                    } else if cell.row == self.chip.row_n() {
                        cell.row -= 7;
                    } else {
                        unreachable!();
                    }
                }
                if self.chip.columns[cell.col].sdclk_break {
                    prev = None;
                }
                if let Some(prev) = prev {
                    self.egrid
                        .fill_conn_pair(prev, cell, ccls::PASS_SE, ccls::PASS_SW);
                }
                prev = Some(cell);
            }

            for cell in self.egrid.row(self.die, row) {
                if self.chip.columns[cell.col].sdclk_break {
                    let mut cells = [None; 8];
                    for cell in cell.delta(-4, 0).cells_e_const::<4>() {
                        cells[self.chip.col_sclk_idx(cell.col)] = Some(cell);
                    }
                    for cell in cell.cells_e_const::<4>() {
                        cells[4 + self.chip.col_sclk_idx(cell.col)] = Some(cell);
                    }
                    let mut cells = cells.map(Option::unwrap);
                    for cell in &mut cells {
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
                    let mut cell = cell;
                    if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                        if cell.row == self.chip.row_s() {
                            cell.row += 7;
                        } else if cell.row == self.chip.row_n() {
                            cell.row -= 7;
                        } else {
                            unreachable!();
                        }
                    }
                    self.egrid.add_tile(
                        cell,
                        if cell.col == self.chip.col_clk {
                            tcls::HSDCLK_ROOT
                        } else {
                            tcls::HSDCLK_SPLITTER
                        },
                        &cells,
                    );
                }
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

    fn fill_sclk_ecp3(&mut self) {
        let mut row_sclk = self.chip.row_s();
        let mut rows_sclk = EntityVec::new();
        for (row, rd) in &self.chip.rows {
            if rd.sclk_break {
                row_sclk = row;
            }
            rows_sclk.push(row_sclk);
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
                let cell_src_sclk = cell_src.with_row(rows_sclk[cell.row]);
                self.egrid[cell].region_root[rslots::RSCLK[i]] = cell_src_sclk;
            }
        }
        for (row, rd) in &self.chip.rows {
            if !rd.sclk_break && row != self.chip.row_s() {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                let i = self.chip.col_sclk_idx(cell.col);
                let kind = if cell.col == self.chip.col_w() {
                    tiledata::SCLK_SOURCE_W[i]
                } else if cell.col == self.chip.col_e() {
                    tiledata::SCLK_SOURCE_E[i]
                } else {
                    tiledata::SCLK_SOURCE[i]
                };
                self.egrid.add_tile_single(cell, kind);
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

        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Io && !rd.sclk_break {
                continue;
            }
            let mut prev = None;
            for mut cell in self.egrid.row(self.die, row) {
                if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                    if cell.row == self.chip.row_s() {
                        cell.row += 9;
                    } else {
                        unreachable!();
                    }
                }
                if self.chip.columns[cell.col].sdclk_break {
                    prev = None;
                }
                if let Some(prev) = prev {
                    self.egrid
                        .fill_conn_pair(prev, cell, ccls::PASS_SE, ccls::PASS_SW);
                }
                prev = Some(cell);
            }

            for cell in self.egrid.row(self.die, row) {
                if self.chip.columns[cell.col].sdclk_break {
                    let mut cells = [None; 8];
                    for cell in cell.delta(-4, 0).cells_e_const::<4>() {
                        cells[self.chip.col_sclk_idx(cell.col)] = Some(cell);
                    }
                    for cell in cell.cells_e_const::<4>() {
                        cells[4 + self.chip.col_sclk_idx(cell.col)] = Some(cell);
                    }
                    let mut cells = cells.map(Option::unwrap);
                    for cell in &mut cells {
                        if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                            if cell.row == self.chip.row_s() {
                                cell.row += 9;
                            } else {
                                unreachable!();
                            }
                        }
                    }
                    let mut cell = cell;
                    if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                        if cell.row == self.chip.row_s() {
                            cell.row += 9;
                        } else {
                            unreachable!();
                        }
                    }
                    self.egrid.add_tile(
                        cell,
                        if cell.col == self.chip.col_clk
                            && !matches!(self.chip.kind, ChipKind::MachXo2(_))
                        {
                            tcls::HSDCLK_SPLITTER
                        } else {
                            tcls::HSDCLK_ROOT
                        },
                        &cells,
                    );
                }
            }
        }
    }

    fn fill_pclk_ecp3(&mut self) {
        let mut row_pclk = self.chip.row_s();
        let mut rows_pclk = EntityVec::new();
        for (row, rd) in &self.chip.rows {
            if rd.pclk_break {
                row_pclk = row;
            }
            rows_pclk.push(row_pclk);
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
                    && ((cell_src.col < self.chip.col_clk) == (cell.col < self.chip.col_clk)
                        || matches!(self.chip.kind, ChipKind::MachXo2(_)))
                {
                    cell_src
                } else if let Some(cell_src) = self.egrid.cell_delta(cell, dx_alt, 0)
                    && ((cell_src.col < self.chip.col_clk) == (cell.col < self.chip.col_clk)
                        || matches!(self.chip.kind, ChipKind::MachXo2(_)))
                {
                    cell_src
                } else {
                    cell
                };
                let cell_src_pclk = cell_src.with_row(rows_pclk[cell.row]);
                self.egrid[cell].region_root[rslots::RPCLK[i]] = cell_src_pclk;
            }
        }

        let mut prev_driven = self.chip.rows[self.chip.row_s()].pclk_drive;
        for (row, rd) in &self.chip.rows {
            if !rd.pclk_drive {
                if rd.pclk_break {
                    prev_driven = false;
                }
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                let idx = self.chip.col_sclk_idx(cell.col);
                if cell.row == self.chip.row_s() {
                    let kind = if cell.col == self.chip.col_w()
                        || (cell.col == self.chip.col_clk
                            && !matches!(self.chip.kind, ChipKind::MachXo2(_)))
                    {
                        tiledata::RPCLK_DRIVER_IO_N_W[idx]
                    } else if cell.col == self.chip.col_e()
                        || (cell.col == self.chip.col_clk - 1
                            && !matches!(self.chip.kind, ChipKind::MachXo2(_)))
                    {
                        tiledata::RPCLK_DRIVER_IO_N_E[idx]
                    } else {
                        tiledata::RPCLK_DRIVER_IO_N[idx]
                    };
                    self.egrid.add_tile_single(cell, kind);
                } else {
                    let kind = if cell.col == self.chip.col_w()
                        || (cell.col == self.chip.col_clk
                            && !matches!(self.chip.kind, ChipKind::MachXo2(_)))
                    {
                        tiledata::RPCLK_DRIVER_W[idx]
                    } else if cell.col == self.chip.col_e()
                        || (cell.col == self.chip.col_clk - 1
                            && !matches!(self.chip.kind, ChipKind::MachXo2(_)))
                    {
                        tiledata::RPCLK_DRIVER_E[idx]
                    } else {
                        tiledata::RPCLK_DRIVER[idx]
                    };
                    if prev_driven {
                        self.egrid.add_tile_part(cell, kind, &[Some(cell), None]);
                    } else {
                        self.egrid.add_tile(cell, kind, &[cell, cell.delta(0, -1)]);
                    }
                }
            }
            prev_driven = true;
        }
    }

    fn fill_clk_ecp3(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if cell.col != self.chip.col_w() && cell.col != self.chip.col_e() {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_col(self.chip.col_clk - 1);
            }
            if self.chip.columns[cell.col].eclk_tap_n {
                self.egrid.add_tile_single(cell, tcls::ECLK_TAP);
            }
        }

        for col in [self.chip.col_w(), self.chip.col_e()] {
            for cell in self.egrid.column(self.die, col) {
                self.egrid[cell].region_root[rslots::ECLK] = cell.with_row(self.chip.row_clk);
            }
        }
        for (loc, bcrd) in self.chip.plls() {
            self.egrid[bcrd.cell].region_root[rslots::ECLK] = self
                .die
                .cell(self.chip.col_edge(loc.quad.h), self.chip.row_clk);
        }

        let mut tcells = vec![];
        // center
        for idx in [6, 2, 0] {
            let cell = self.chip.special_loc[&SpecialLocKey::PclkInMid(idx)];
            tcells.extend(cell.cells_e(2).into_iter().map(Some));
        }
        // edges
        for rect in [
            Chip::DIE
                .cell(self.chip.col_w(), self.chip.row_clk - 3)
                .cells_n(6),
            Chip::DIE
                .cell(self.chip.col_e(), self.chip.row_clk - 4)
                .cells_n(8),
            Chip::DIE
                .cell(self.chip.col_clk - 3, self.chip.row_s() + 9)
                .cells_e(6),
            Chip::DIE
                .cell(self.chip.col_clk - 3, self.chip.row_n())
                .cells_e(6),
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
            if cd.io_s == IoGroupKind::Serdes {
                let idx = usize::try_from(cd.bank_s.unwrap() - 50).unwrap();
                serdes_cells[idx] = Some(Chip::DIE.cell(col, self.chip.row_s() + 9));
            }
        }
        tcells.extend(serdes_cells);

        self.egrid.add_tile_part(
            self.die.cell(self.chip.col_clk, self.chip.row_clk),
            tcls::CLK_ROOT,
            &tcells,
        );
    }

    fn fill_clk_machxo2(&mut self) {
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
                self.egrid.add_tile(
                    cell,
                    tcls::CLK_ROOT_1EBR,
                    &[
                        cell.delta(-4, 0),
                        cell.delta(-2, 0),
                        cell.delta(-1, 0),
                        cell,
                        cell.with_col(self.chip.col_w()),
                        cell.with_col(self.chip.col_e()),
                        cell.with_row(self.chip.row_s()).delta(-1, 0),
                        cell.with_row(self.chip.row_n()).delta(-1, 0),
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
                    ],
                );
            }
            _ => unreachable!(),
        }
        if !ebr_rows.is_empty() {
            self.egrid
                .add_tile_single(cell.with_row(self.chip.row_s()).delta(-1, 0), tcls::CLK_S);
            self.egrid
                .add_tile_single(cell.with_row(self.chip.row_n()).delta(-1, 0), tcls::CLK_N);
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

    fn fill_clk_ecp4(&mut self) {
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
        let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
        let mut tcells = if !has_bank0 {
            vec![
                cell.delta(-4, 0),
                cell.delta(-3, 0),
                cell.delta(-2, 0),
                cell.delta(-1, 0),
                cell,
                cell.delta(1, 0),
                cell.delta(2, 0),
                cell.delta(3, 0),
            ]
        } else {
            vec![
                cell.delta(-8, 0),
                cell.delta(-7, 0),
                cell.delta(-6, 0),
                cell.delta(-5, 0),
                cell.delta(-4, 0),
                cell.delta(-3, 0),
                cell.delta(-2, 0),
                cell.delta(-1, 0),
                cell,
                cell.delta(1, 0),
                cell.delta(2, 0),
                cell.delta(3, 0),
                cell.delta(4, 0),
                cell.delta(5, 0),
                cell.delta(6, 0),
                cell.delta(7, 0),
                self.chip.special_loc[&SpecialLocKey::Bc(1)].delta(1, 0),
                self.chip.special_loc[&SpecialLocKey::Bc(2)].delta(1, 0),
            ]
        };
        for i in 0..4 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::N, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(cell, kind_n, &tcells);

        let cell = self.die.cell(self.chip.col_w(), self.chip.row_clk);
        let mut tcells = vec![
            cell.delta(0, -4),
            cell.delta(0, -3),
            cell.delta(0, -2),
            cell.delta(0, -1),
            cell,
            cell.delta(0, 1),
            cell.delta(0, 2),
            cell.delta(0, 3),
        ];
        for i in 0..4 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::W, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(cell, kind_w, &tcells);
        let cell = self.die.cell(self.chip.col_e(), self.chip.row_clk);
        let mut tcells = vec![
            cell.delta(0, -4),
            cell.delta(0, -3),
            cell.delta(0, -2),
            cell.delta(0, -1),
            cell,
            cell.delta(0, 1),
            cell.delta(0, 2),
            cell.delta(0, 3),
        ];
        for i in 0..4 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::E, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(cell, kind_e, &tcells);

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
        let mut tcells = vec![cell.delta(-1, 0), cell];
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
        let cell = self.chip.bel_clk_root().cell;
        self.egrid.add_tile(
            cell,
            tcls::CLK_ROOT,
            &[
                cell.with_cr(self.chip.col_clk - 1, row_s),
                cell.with_cr(self.chip.col_clk, row_s),
                cell.with_cr(self.chip.col_clk - 1, row_n),
                cell.with_cr(self.chip.col_clk, row_n),
            ],
        );
    }

    fn fill_clk_ecp5(&mut self) {
        let cell = self.die.cell(self.chip.col_w(), self.chip.row_clk);
        let mut tcells = vec![
            cell.delta(0, -2),
            cell.delta(0, -1),
            cell,
            cell.delta(0, 1),
            cell.delta(1, 0),
            cell.delta(2, 0),
        ];
        for i in 0..8 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::W, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(
            cell,
            match tcells.len() {
                12 => tcls::CLK_W_S,
                14 => tcls::CLK_W_L,
                _ => unreachable!(),
            },
            &tcells,
        );

        let cell = self.die.cell(self.chip.col_e(), self.chip.row_clk);
        let mut tcells = vec![
            cell.delta(0, -2),
            cell.delta(0, -1),
            cell,
            cell.delta(0, 1),
            cell.delta(-1, 0),
            cell.delta(-2, 0),
        ];
        for i in 0..8 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::E, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(
            cell,
            match tcells.len() {
                12 => tcls::CLK_E_S,
                14 => tcls::CLK_E_L,
                _ => unreachable!(),
            },
            &tcells,
        );

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
        let mut tcells = cell.delta(-2, 0).cells_e(4);
        for i in 0..8 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::N, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(
            cell,
            match tcells.len() {
                10 => tcls::CLK_N_S,
                12 => tcls::CLK_N_L,
                _ => unreachable!(),
            },
            &tcells,
        );

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
        let mut tcells = cell.delta(-1, 0).cells_e(2);
        for i in 0..8 {
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::PclkIn(Dir::S, i)) {
                tcells.push(cell);
            }
        }
        self.egrid.add_tile(
            cell,
            match tcells.len() {
                8 => tcls::CLK_S_S,
                10 => tcls::CLK_S_L,
                _ => unreachable!(),
            },
            &tcells,
        );

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
                &[cell_s.delta(-1, 0), cell_s, cell_n.delta(-1, 0), cell_n],
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
                ],
            );
        }
    }

    fn fill_clk_crosslink(&mut self) {
        let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
        let mut tcells = cell.delta(-2, 0).cells_e(4);
        for i in 0..2 {
            tcells.push(self.chip.special_loc[&SpecialLocKey::PclkIn(Dir::S, i)]);
        }
        self.egrid.add_tile(cell, tcls::CLK_S, &tcells);

        let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
        self.egrid.add_tile_we(cell, tcls::CLK_N, 1, 2);

        let cell = self.chip.bel_clk_root().cell;
        self.egrid.add_tile_single(cell, tcls::CLK_ROOT);

        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            self.egrid[cell].region_root[rslots::ECLK] =
                cell.with_col(if cell.col < self.chip.col_clk {
                    self.chip.col_clk - 1
                } else {
                    self.chip.col_clk
                });
        }
    }

    fn fill_config_xp(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        self.egrid.add_tile_single(cell, tcls::CONFIG);
    }

    fn fill_config_machxo2(&mut self) {
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

    fn fill_config_ecp4(&mut self) {
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
            cell.with_cr(self.chip.col_w(), self.chip.row_clk - 1),
            cell.with_cr(self.chip.col_w(), self.chip.row_clk),
            cell.with_cr(self.chip.col_clk - 1, self.chip.row_s()),
            cell.with_cr(self.chip.col_clk, self.chip.row_s()),
            cell.with_cr(self.chip.col_clk - 1, row_dsp),
        ]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);
    }

    fn fill_config_ecp5(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut tcells = cell.cells_e(3);
        tcells.extend([cell.with_cr(self.chip.col_clk, self.chip.row_clk)]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);
        self.egrid.add_tile_single(cell.delta(18, 0), tcls::DTR);
    }

    fn fill_config_crosslink(&mut self) {
        let cell = self.die.cell(self.chip.col_w(), self.chip.row_n());
        self.egrid.add_tile_e(cell, tcls::I2C_W, 2);
        let cell = self.die.cell(self.chip.col_e() - 1, self.chip.row_n());
        self.egrid.add_tile_e(cell, tcls::I2C_E, 2);

        let cell = self.chip.special_loc[&SpecialLocKey::Config];
        let mut tcells = cell.cells_e(2);
        tcells.extend([
            cell.delta(-2, 0),
            cell.with_cr(self.chip.col_clk, self.chip.row_clk),
        ]);
        self.egrid.add_tile(cell, tcls::CONFIG, &tcells);

        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.egrid.add_tile_single(cell, tcls::OSC);

        let cell = self.chip.special_loc[&SpecialLocKey::Pmu];
        self.egrid.add_tile_single(cell, tcls::PMU);

        let cell = self.chip.bel_pll(DirHV::SE, 0).cell;
        self.egrid.add_tile_e(cell, tcls::PLL, 2);
    }

    fn fill_io_machxo(&mut self) {
        for col in self.egrid.cols(self.die) {
            if col != self.chip.col_w() && col != self.chip.col_e() {
                let cell_s = self.die.cell(col, self.chip.row_s());
                self.egrid.fill_conn_pair(
                    cell_s,
                    cell_s.delta(0, 1),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
                let cell_n = self.die.cell(col, self.chip.row_n());
                self.egrid.fill_conn_pair(
                    cell_n,
                    cell_n.delta(0, -1),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
            }
        }
        for row in self.egrid.rows(self.die) {
            if row > self.chip.row_s() + 1 && row < self.chip.row_n() - 1 {
                let cell_w = self.die.cell(self.chip.col_w(), row);
                self.egrid.fill_conn_pair(
                    cell_w,
                    cell_w.delta(1, 0),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
                let cell_e = self.die.cell(self.chip.col_e(), row);
                self.egrid.fill_conn_pair(
                    cell_e,
                    cell_e.delta(-1, 0),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
            }
        }
        for (&io, &(src_cell, plc_lut)) in &self.chip.io_direct_plc {
            let io_idx = bslots::IO.index_of(io.slot).unwrap();
            let dst_wire = io.wire(wires::IMUX_IO_O_PLC[io_idx]);
            let src_wire = src_cell.wire(wires::PLC_IO[plc_lut]);
            self.egrid.extra_conns.insert(dst_wire, src_wire);
        }
        let has_ebr = self.chip.special_loc.contains_key(&SpecialLocKey::Ebr(0));
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            let int_kind = if has_ebr {
                tcls::INT_SIO_L_W
            } else if cell.row >= self.chip.row_clk - 3 && cell.row < self.chip.row_clk + 3 {
                tcls::INT_SIO_S_W_CLK
            } else {
                tcls::INT_SIO_S_W
            };
            self.egrid.add_tile_single(cell, int_kind);
            let kind = match (has_ebr, rd.io_w) {
                (true, IoGroupKind::Double) => tcls::SIO_L_W2,
                (true, IoGroupKind::Quad | IoGroupKind::QuadReverse) => tcls::SIO_L_W4,
                (false, IoGroupKind::Double) => tcls::SIO_S_W2,
                (false, IoGroupKind::Quad) => tcls::SIO_S_W4,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            let int_kind = if cell == self.chip.special_loc[&SpecialLocKey::Config] {
                if has_ebr {
                    tcls::INT_SIO_L_E_CFG
                } else {
                    tcls::INT_SIO_S_E_CFG
                }
            } else {
                if has_ebr {
                    tcls::INT_SIO_L_E
                } else {
                    tcls::INT_SIO_S_E
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            let kind = match (has_ebr, rd.io_e) {
                (false, IoGroupKind::Double) => tcls::SIO_S_E2,
                (false, IoGroupKind::Quad) => tcls::SIO_S_E4,
                (true, IoGroupKind::Double) => tcls::SIO_L_E2,
                (true, IoGroupKind::Quad) => tcls::SIO_L_E4,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            let cd = &self.chip.columns[cell.col];
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let (int_kind, kind) = match (has_ebr, cd.io_s) {
                (false, IoGroupKind::Quad) => (tcls::INT_SIO_S_S4, tcls::SIO_S_S4),
                (false, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_S_S6, tcls::SIO_S_S6)
                }
                (true, IoGroupKind::Quad) => (tcls::INT_SIO_L_S4, tcls::SIO_L_S4),
                (true, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_L_S6, tcls::SIO_L_S6)
                }
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            let cd = &self.chip.columns[cell.col];
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let (int_kind, kind) = match (has_ebr, cd.io_n) {
                (false, IoGroupKind::Quad) => (tcls::INT_SIO_S_N4, tcls::SIO_S_N4),
                (false, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_S_N6, tcls::SIO_S_N6)
                }
                (true, IoGroupKind::Quad) => (tcls::INT_SIO_L_N4, tcls::SIO_L_N4),
                (true, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_L_N6, tcls::SIO_L_N6)
                }
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, kind);
        }
    }

    fn fill_special_machxo(&mut self) {
        let has_ebr = self.chip.special_loc.contains_key(&SpecialLocKey::Ebr(0));
        for (&key, &cell) in &self.chip.special_loc {
            match key {
                SpecialLocKey::Pll(which) => {
                    let kind = match which.quad.v {
                        DirV::S => tcls::PLL_S,
                        DirV::N => tcls::PLL_N,
                    };
                    self.egrid.add_tile_single(cell, kind);
                    for (wire, pad) in [
                        (wires::IO_PLL_CLKI, PllPad::PllIn0),
                        (wires::IO_PLL_CLKFB, PllPad::PllFb),
                    ] {
                        self.add_io_conn(cell.wire(wire), SpecialIoKey::Pll(pad, which));
                    }
                }
                SpecialLocKey::Ebr(_) => {
                    self.egrid.add_tile_n(cell, tcls::EBR, 4);
                }
                SpecialLocKey::Config => {
                    self.egrid.add_tile_n(cell, tcls::CONFIG, 5);
                }
                SpecialLocKey::Osc => {
                    let kind = if has_ebr { tcls::OSC_X } else { tcls::OSC };
                    self.egrid.add_tile_single(cell, kind);
                }
                _ => unreachable!(),
            }
        }
        if has_ebr {
            let bel = self.chip.bel_cibtest_sel();
            self.egrid.add_tile_single(bel.cell, tcls::CIBTEST_SEL);
        }
    }

    fn fill_clk_machxo(&mut self) {
        for &key in self.chip.special_io.keys() {
            let SpecialIoKey::Clock(edge, cidx) = key else {
                continue;
            };
            let wire = match edge {
                Dir::N => wires::IO_GCLK_N[usize::from(cidx)],
                Dir::S => wires::IO_GCLK_S[usize::from(cidx)],
                _ => unreachable!(),
            };
            self.add_io_conn(
                self.chip.clk_root().with_col(self.chip.col_w()).wire(wire),
                key,
            );
        }
        let mut tcells = self
            .die
            .cell(self.chip.col_w(), self.chip.row_clk - 3)
            .cells_n(6);
        let kind = if let Some(pll_sw) = self.chip.try_bel_pll(DirHV::SW, 0) {
            if let Some(pll_nw) = self.chip.try_bel_pll(DirHV::NW, 0) {
                tcells.extend([pll_sw.cell, pll_nw.cell]);
                tcls::CLK_ROOT_2PLL
            } else {
                tcells.extend([pll_sw.cell]);
                tcls::CLK_ROOT_1PLL
            }
        } else {
            tcls::CLK_ROOT_0PLL
        };
        self.egrid.add_tile(
            self.die.cell(self.chip.col_clk, self.chip.row_clk),
            kind,
            &tcells,
        );
    }

    fn fill_conns(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            if !self.egrid[cell].tiles.contains_id(tslots::INT) {
                continue;
            }
            if cell.col != self.chip.col_w()
                && self.egrid[cell.delta(-1, 0)].tiles.contains_id(tslots::INT)
            {
                self.egrid
                    .fill_conn_pair(cell.delta(-1, 0), cell, ccls::PASS_E, ccls::PASS_W);
            } else {
                self.egrid.fill_conn_term(cell, ccls::TERM_W);
            }
            if cell.col == self.chip.col_e()
                || !self.egrid[cell.delta(1, 0)].tiles.contains_id(tslots::INT)
            {
                self.egrid.fill_conn_term(cell, ccls::TERM_E);
            }
            if cell.row != self.chip.row_s()
                && self.egrid[cell.delta(0, -1)].tiles.contains_id(tslots::INT)
            {
                self.egrid
                    .fill_conn_pair(cell.delta(0, -1), cell, ccls::PASS_N, ccls::PASS_S);
            } else {
                self.egrid.fill_conn_term(cell, ccls::TERM_S);
            }
            if cell.row == self.chip.row_n()
                || !self.egrid[cell.delta(0, 1)].tiles.contains_id(tslots::INT)
            {
                self.egrid.fill_conn_term(cell, ccls::TERM_N);
            }
        }
    }

    fn fill_bs_ecp(&mut self) {
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

impl Chip {
    pub fn expand_grid<'a>(&'a self, db: &'a IntDb) -> ExpandedDevice<'a> {
        let mut egrid = GridBuilder::new(db);
        let die = egrid.add_die(self.columns.len(), self.rows.len());
        let mut expander = Expander {
            chip: self,
            egrid: &mut egrid,
            die,
            bel_holes: vec![],
            frame_len: 0,
            frames_num: 0,
            clk_frame: 0,
            col_frame: Default::default(),
            col_term_frame: Default::default(),
            row_bit: Default::default(),
            row_ebr_bit: Default::default(),
        };

        expander.fill_region_pclk();
        match self.kind {
            ChipKind::Scm => {
                expander.fill_config_scm();
                expander.fill_pll_scm();
                expander.fill_serdes_scm();
                expander.fill_plc();
                expander.fill_ebr_scm();
                expander.fill_io_scm();
                expander.fill_clk_scm();
            }
            ChipKind::Ecp => {
                expander.fill_config_ecp();
                expander.fill_pll_ecp();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp();
                expander.fill_bs_ecp();
            }
            ChipKind::Xp => {
                expander.fill_pll_xp();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_config_xp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp();
                expander.fill_bs_ecp();
            }
            ChipKind::MachXo => {
                expander.fill_plc();
                expander.fill_io_machxo();
                expander.fill_special_machxo();
                expander.fill_clk_machxo();
                expander.fill_bs_ecp();
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M => {
                expander.fill_config_ecp2();
                expander.fill_pll_ecp2();
                expander.fill_serdes_ecp2();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp2();
            }
            ChipKind::Xp2 => {
                expander.fill_config_xp2();
                expander.fill_pll_xp2();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp2();
            }
            ChipKind::Ecp3 | ChipKind::Ecp3A => {
                expander.fill_config_ecp3();
                expander.fill_pll_ecp3();
                expander.fill_serdes_ecp3();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp3();
                expander.fill_sclk_ecp3();
                expander.fill_pclk_ecp3();
                expander.fill_clk_ecp3();
            }
            ChipKind::MachXo2(_) => {
                expander.fill_config_machxo2();
                expander.fill_pll_machxo2();
                expander.fill_plc();
                expander.fill_ebr_machxo2();
                expander.fill_io_machxo2();
                expander.fill_sclk_ecp3();
                expander.fill_pclk_ecp3();
                expander.fill_clk_machxo2();
            }
            ChipKind::Ecp4 => {
                expander.fill_config_ecp4();
                expander.fill_pll_ecp4();
                expander.fill_plc();
                expander.fill_ebr_ecp4();
                expander.fill_io_ecp4();
                expander.fill_serdes_ecp4();
                expander.fill_clk_ecp4();
            }
            ChipKind::Ecp5 => {
                expander.fill_config_ecp5();
                expander.fill_pll_ecp5();
                expander.fill_plc();
                expander.fill_ebr_ecp4();
                expander.fill_io_ecp5();
                expander.fill_clk_ecp5();
            }
            ChipKind::Crosslink => {
                expander.fill_config_crosslink();
                expander.fill_plc();
                expander.fill_ebr_crosslink();
                expander.fill_io_crosslink();
                expander.fill_clk_crosslink();
            }
        }
        expander.fill_conns();

        let bel_holes = expander.bel_holes;
        let frame_len = expander.frame_len;
        let frames_num = expander.frames_num;
        let clk_frame = expander.clk_frame;
        let col_frame = expander.col_frame;
        let col_term_frame = expander.col_term_frame;
        let row_bit = expander.row_bit;
        let row_ebr_bit = expander.row_ebr_bit;

        let egrid = egrid.finish();
        ExpandedDevice {
            chip: self,
            egrid,
            bel_holes,
            frame_len,
            frames_num,
            clk_frame,
            col_frame,
            col_term_frame,
            row_bit,
            row_ebr_bit,
        }
    }
}
