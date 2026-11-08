use std::collections::{BTreeMap, HashMap};

use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::{EntityId, EntityIdArith};
use prjcombine_interconnect::{
    coord::{ColId, DieIdExt, RowId},
    dir::{Dir, DirH, DirHV, DirV},
};
use prjcombine_re_lattice_naming::WireName;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_clk_ecp4(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "DCS0" {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_pclk_ecp4(&mut self) {
        let hpbx = self.grid.strings.get("HPBX0000").unwrap();
        let mut clks: BTreeMap<WireName, Vec<RowId>> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[nf];
            let wtn = self.grid.nodes[nt];
            if wtn.suffix == hpbx && wtn.c == 1 {
                let cell = self.chip.xlat_rc_wire(wtn);
                clks.entry(wfn).or_default().push(cell.row);
            }
        }
        let mut clks = Vec::from_iter(clks);
        clks.sort_by_key(|(_, rows)| rows[0]);
        let mut next = RowId::from_idx(0);
        let mut breaks = vec![];
        for (_, mut rows) in clks {
            rows.sort();
            let row_start = rows[0];
            assert_eq!(row_start, next);
            for (i, &col) in rows.iter().enumerate() {
                assert_eq!(col, row_start + i);
            }
            breaks.push(row_start);
            next = row_start + rows.len();
        }
        breaks.push(next);
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            let brk = if row < self.chip.row_clk {
                row + 1
            } else {
                row
            };
            let idx = breaks.binary_search(&brk).unwrap();
            let next = breaks[idx + 1];
            let num = next - self.chip.rows_pclk.next_id();
            self.chip.rows_pclk.push(num as usize, row);
        }
    }

    pub(super) fn fill_io_ecp4(&mut self) {
        self.fill_io(&[
            ("MIB_L_PIC0A", IoGroupKind::Quad),
            ("MIB_L_PIC0B", IoGroupKind::Quad),
            ("MIB_L_PIC0B_BREF", IoGroupKind::Quad),
            ("MIB_L_PIC0B_HIQ_U", IoGroupKind::Quad),
            ("MIB_L_PIC0A_DQS1", IoGroupKind::Quad),
            ("MIB_L_PIC0B_DQS0", IoGroupKind::Quad),
            ("MIB_L_PIC0A_DQS2", IoGroupKind::Quad),
            ("MIB_L_PIC0B_DQS3", IoGroupKind::Quad),
            ("MIB_L_PIC0A_DQS3", IoGroupKind::Quad),
            ("MIB_LS_PIC0B_E", IoGroupKind::Quad),
            ("MIB_LS_PIC0B_E_DQS2", IoGroupKind::Quad),
            ("MIB_LS_PIC0B_D", IoGroupKind::Quad),
            ("MIB_LS_PIC0B_D_DQS0", IoGroupKind::Quad),
            ("MIB_LS_PIC0A_D", IoGroupKind::Quad),
            ("MIB_LS_PIC0A_E", IoGroupKind::Quad),
            ("MIB_LS_PIC0A_E_DQS1", IoGroupKind::Quad),
            ("MIB_LS_PIC0A_E_DQS2", IoGroupKind::Quad),
            ("MIB_R_PIC0A", IoGroupKind::Quad),
            ("MIB_R_PIC0B", IoGroupKind::Quad),
            ("MIB_R_PIC0B_BREF", IoGroupKind::Quad),
            ("MIB_R_PIC0B_HIQ_U", IoGroupKind::Quad),
            ("MIB_R_PIC0A_DQS1", IoGroupKind::Quad),
            ("MIB_R_PIC0B_DQS0", IoGroupKind::Quad),
            ("MIB_R_PIC0A_DQS2", IoGroupKind::Quad),
            ("MIB_R_PIC0B_DQS3", IoGroupKind::Quad),
            ("MIB_R_PIC0A_DQS3", IoGroupKind::Quad),
            ("MIB_RS_PIC0B_E", IoGroupKind::Quad),
            ("MIB_RS_PIC0B_E_DQS2", IoGroupKind::Quad),
            ("MIB_RS_PIC0B_D", IoGroupKind::Quad),
            ("MIB_RS_PIC0B_D_DQS0", IoGroupKind::Quad),
            ("MIB_RS_PIC0A_D", IoGroupKind::Quad),
            ("MIB_RS_PIC0A_E", IoGroupKind::Quad),
            ("MIB_RS_PIC0A_E_DQS1", IoGroupKind::Quad),
            ("MIB_RS_PIC0A_E_DQS2", IoGroupKind::Quad),
            ("MIB_T_PIC0A", IoGroupKind::Quad),
            ("MIB_T_PIC0B", IoGroupKind::Quad),
            ("MIB_T_PIC0B_DLLDEL_B1", IoGroupKind::Quad),
            ("MIB_T_PIC0A_DLLDEL_B0", IoGroupKind::Quad),
            ("MIB_T_PIC0B_DLLDEL_B1_BIG", IoGroupKind::Quad),
            ("MIB_T_PIC0A_DLLDEL_B1_BIG", IoGroupKind::Quad),
            ("MIB_T_PIC0A_DQS1", IoGroupKind::Quad),
            ("MIB_T_PIC0B_DQS0", IoGroupKind::Quad),
        ]);

        let mut skip_w = 0;
        let mut skip_e = 0;
        for (row, rd) in &mut self.chip.rows {
            if rd.io[DirH::W] != IoGroupKind::None {
                if skip_w != 0 {
                    rd.io[DirH::W] = IoGroupKind::None;
                    skip_w -= 1;
                } else {
                    skip_w = 3;
                }
            }
            if rd.io[DirH::E] != IoGroupKind::None {
                if skip_e != 0 {
                    rd.io[DirH::E] = IoGroupKind::None;
                    skip_e -= 1;
                } else {
                    skip_e = 3;
                }
            }
            if rd.kind == RowKind::Dsp {
                assert!(skip_w >= 2);
                assert!(skip_e >= 2);
                skip_w -= 2;
                skip_e -= 2;
            }
            if row < self.chip.row_clk {
                if rd.io[DirH::W] != IoGroupKind::None || rd.kind == RowKind::Ebr {
                    rd.bank.insert(DirH::W, 6);
                }
                if rd.io[DirH::E] != IoGroupKind::None || rd.kind == RowKind::Ebr {
                    rd.bank.insert(DirH::E, 5);
                }
            } else {
                if rd.io[DirH::W] != IoGroupKind::None || rd.kind == RowKind::Ebr {
                    rd.bank.insert(DirH::W, 7);
                }
                if rd.io[DirH::E] != IoGroupKind::None || rd.kind == RowKind::Ebr {
                    rd.bank.insert(DirH::E, 4);
                }
            }
        }
        assert_eq!(skip_w, 0);
        assert_eq!(skip_e, 0);

        let mut bank = 2;
        for col in (self.chip.col_clk + 1usize).range_until(self.chip.col_e()) {
            if self.chip.columns[col].io[DirV::N] == IoGroupKind::None {
                bank = 3;
            } else {
                self.chip.columns[col].bank.insert(DirV::N, bank);
            }
        }
        let mut bank = 1;
        for col in self
            .chip
            .col_w()
            .range_until(self.chip.col_clk - 1usize)
            .rev()
        {
            if self.chip.columns[col].io[DirV::N] == IoGroupKind::None {
                bank = 0;
            } else {
                self.chip.columns[col].bank.insert(DirV::N, bank);
            }
        }

        let mut skip = 0;
        for (_col, cd) in &mut self.chip.columns {
            if cd.io[DirV::N] != IoGroupKind::None {
                if skip != 0 {
                    cd.io[DirV::N] = IoGroupKind::None;
                    cd.bank.remove(DirV::N);
                    skip -= 1;
                } else {
                    skip = 3;
                }
            }
        }
        assert_eq!(skip, 0);
    }

    pub(super) fn fill_dqs_ecp4(&mut self) {
        let dqsi = self.grid.strings.get("JDQSI_DQS").unwrap();
        let paddi = self.grid.strings.get("JPADDI_PIO").unwrap();
        let paddiea = self.grid.strings.get("JPADDIEA_PIO").unwrap();
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[wf];
            let wtn = self.grid.nodes[wt];
            if wtn.suffix == dqsi {
                let cell = self.chip.xlat_rc_wire(wtn);
                let is_ea = if wfn.suffix == paddi {
                    false
                } else if wfn.suffix == paddiea {
                    true
                } else {
                    unreachable!()
                };
                if cell.row == self.chip.row_n() {
                    assert!(!is_ea);
                    assert_eq!(self.chip.columns[cell.col].io[DirV::N], IoGroupKind::Quad);
                    self.chip.columns[cell.col].io[DirV::N] = IoGroupKind::QuadDqs;
                } else {
                    let edge = if cell.col == self.chip.col_w() {
                        DirH::W
                    } else if cell.col == self.chip.col_e() {
                        DirH::E
                    } else {
                        unreachable!()
                    };
                    if is_ea && self.chip.rows[cell.row].kind == RowKind::Ebr {
                        let io_kind = &mut self.chip.rows[cell.row].io[edge];
                        *io_kind = match *io_kind {
                            IoGroupKind::None => IoGroupKind::EbrDqs,
                            IoGroupKind::Quad => IoGroupKind::QuadEbrDqs,
                            _ => unreachable!(),
                        };
                    } else {
                        let mut row = cell.row;
                        loop {
                            let io_kind = &mut self.chip.rows[row].io[edge];
                            if *io_kind == IoGroupKind::Quad {
                                *io_kind = IoGroupKind::QuadDqs;
                                break;
                            }
                            row -= 1;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn fill_bc_ecp4(&mut self) {
        let has_bank0 = self
            .chip
            .columns
            .values()
            .any(|cd| cd.bank.get(DirV::N) == Some(&0));
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[wf];
            let wtn = self.grid.nodes[wt];
            let wtns = self.grid.strings[wtn.suffix].as_str();
            if wtns.starts_with("JPGENI_BCPG") {
                let cell_to = self.chip.xlat_rc_wire(wtn);
                let cell_from = self.chip.xlat_rc_wire(wfn);
                let bank = match wtns {
                    "JPGENI_BCPG_L" => {
                        if cell_to.row == self.chip.row_s() {
                            6
                        } else {
                            7
                        }
                    }
                    "JPGENI_BCPG_R" => {
                        if cell_to.row == self.chip.row_s() {
                            5
                        } else {
                            4
                        }
                    }
                    "JPGENI_BCPG_T" => {
                        if cell_to.col == self.chip.col_w() {
                            if has_bank0 { 0 } else { 1 }
                        } else {
                            if has_bank0 { 3 } else { 2 }
                        }
                    }
                    "JPGENI_BCPG_TL" => 1,
                    "JPGENI_BCPG_TR" => 2,
                    _ => unreachable!(),
                };
                self.chip
                    .special_loc
                    .insert(SpecialLocKey::Bc(bank), cell_from);
            }
        }
    }

    pub(super) fn fill_serdes_ecp4(&mut self) {
        let (loc, col) = match self.chip.rows.len() {
            78 => (SpecialLocKey::SerdesSingle, 16),
            128 => (SpecialLocKey::SerdesDouble, 12),
            130 => (SpecialLocKey::SerdesTriple, 13),
            _ => unreachable!(),
        };
        self.chip
            .special_loc
            .insert(loc, Chip::DIE.cell(ColId::from_idx(col), self.chip.row_s()));
    }

    pub(super) fn fill_special_io_ecp4(&mut self) {
        let has_bank0 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(0));
        for (idx, dx, iob, cond) in [
            (0, -9, 2, has_bank0),
            (1, -9, 0, has_bank0),
            (2, -5, 2, true),
            (3, -5, 0, true),
            (4, 1, 2, true),
            (5, 1, 0, true),
            (6, 5, 2, has_bank0),
            (7, 5, 0, has_bank0),
        ] {
            if cond {
                let io = Chip::DIE
                    .cell(self.chip.col_clk + dx, self.chip.row_n())
                    .bel(bslots::IO[iob]);
                self.chip
                    .special_io
                    .insert(SpecialIoKey::Clock(Dir::N, idx), io);
            }
        }
        for (idx, dy, iob) in [(0, -5, 2), (1, -5, 0), (2, 1, 2), (3, 1, 0)] {
            let io = self.chip.clk_emid(Dir::W).delta(0, dy).bel(bslots::IO[iob]);
            self.chip
                .special_io
                .insert(SpecialIoKey::Clock(Dir::W, idx), io);
            let io = self.chip.clk_emid(Dir::E).delta(0, dy).bel(bslots::IO[iob]);
            self.chip
                .special_io
                .insert(SpecialIoKey::Clock(Dir::E, idx), io);
        }

        let mut quads_w = vec![];
        let mut quads_e = vec![];
        for (row, rd) in &self.chip.rows {
            if rd.kind == RowKind::Ebr {
                quads_w.push(Chip::DIE.cell(self.chip.col_w() + 1, row));
                quads_e.push(Chip::DIE.cell(self.chip.col_e() - 1, row));
            }
            if matches!(
                rd.io[DirH::W],
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                quads_w.push(Chip::DIE.cell(self.chip.col_w(), row));
            }
            if matches!(
                rd.io[DirH::E],
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
            ) {
                quads_e.push(Chip::DIE.cell(self.chip.col_e(), row));
            }
        }
        let mut quads_n = vec![];
        for (col, cd) in &self.chip.columns {
            if matches!(cd.io[DirV::N], IoGroupKind::Quad | IoGroupKind::QuadDqs) {
                quads_n.push(Chip::DIE.cell(col, self.chip.row_n()));
            }
        }

        let quads_w_s = Vec::from_iter(
            quads_w
                .iter()
                .copied()
                .filter(|&cell| cell.row < self.chip.row_clk),
        );

        let quad_q = match self.chip.rows.len() {
            78 => quads_w[6],
            128 | 130 => quads_w[10],
            _ => unreachable!(),
        };

        for (cell, keys) in [
            (
                quads_w[0],
                [
                    Some(SpecialIoKey::Di),
                    Some(SpecialIoKey::Dout),
                    None,
                    Some(SpecialIoKey::WriteN),
                ],
            ),
            (
                quads_w[1],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::SW, 1))),
                    None,
                    Some(SpecialIoKey::CsN),
                    Some(SpecialIoKey::Cs1N),
                ],
            ),
            (
                quads_w[2],
                [
                    Some(SpecialIoKey::D(1)),
                    Some(SpecialIoKey::D(0)),
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SW, 1))),
                    None,
                ],
            ),
            (
                quads_w[3],
                [
                    Some(SpecialIoKey::D(4)),
                    Some(SpecialIoKey::D(3)),
                    None,
                    Some(SpecialIoKey::D(2)),
                ],
            ),
            (
                quads_w[4],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::SW, 0))),
                    None,
                    Some(SpecialIoKey::D(6)),
                    Some(SpecialIoKey::D(5)),
                ],
            ),
            (
                quads_w[5],
                [
                    Some(SpecialIoKey::D(8)),
                    Some(SpecialIoKey::D(7)),
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SW, 0))),
                    None,
                ],
            ),
            (
                quad_q,
                [
                    None,
                    None,
                    Some(SpecialIoKey::D(10)),
                    Some(SpecialIoKey::D(9)),
                ],
            ),
            (
                quads_w_s[quads_w_s.len() - 3],
                [
                    Some(SpecialIoKey::D(13)),
                    Some(SpecialIoKey::D(12)),
                    None,
                    Some(SpecialIoKey::D(11)),
                ],
            ),
            (
                quads_w_s[quads_w_s.len() - 2],
                [
                    None,
                    None,
                    Some(SpecialIoKey::D(15)),
                    Some(SpecialIoKey::D(14)),
                ],
            ),
            (
                quads_w[quads_w.len() - 1],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::NW, 0))),
                    None,
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::NW, 0))),
                    None,
                ],
            ),
        ] {
            for (i, key) in keys.into_iter().enumerate() {
                if let Some(key) = key {
                    self.chip.special_io.insert(key, cell.bel(bslots::IO[i]));
                }
            }
        }

        for (cell, keys) in [
            (
                quads_e[0],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::SE, 1))),
                    None,
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 1))),
                    None,
                ],
            ),
            (
                quads_e[2],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::SE, 0))),
                    None,
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 0))),
                    None,
                ],
            ),
            (
                quads_e[quads_e.len() - 1],
                [
                    Some(SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(DirHV::NE, 0))),
                    None,
                    Some(SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::NE, 0))),
                    None,
                ],
            ),
        ] {
            for (i, key) in keys.into_iter().enumerate() {
                if let Some(key) = key {
                    self.chip.special_io.insert(key, cell.bel(bslots::IO[i]));
                }
            }
        }
        for (cell, hv) in [
            (quads_n[0], DirHV::NW),
            (quads_n[quads_n.len() - 1], DirHV::NE),
        ] {
            for (iob, key) in [
                (0, SpecialIoKey::Pll(PllPad::PllFb, PllLoc::new(hv, 1))),
                (2, SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(hv, 1))),
            ] {
                self.chip.special_io.insert(key, cell.bel(bslots::IO[iob]));
            }
        }

        for hv in DirHV::DIRS {
            self.chip.special_loc.insert(
                SpecialLocKey::Pll(PllLoc::new(hv, 0)),
                Chip::DIE.cell(self.chip.col_edge(hv.h), self.chip.row_edge(hv.v)),
            );
        }
    }

    pub(super) fn fill_fabric_clock_ecp4(&mut self) {
        let mut xlat = HashMap::new();
        for (name, key) in [
            ("JLLMPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 0)),
            ("JLLMPCLKCIB2", SpecialLocKey::PclkIn(Dir::W, 1)),
            ("JULMPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 2)),
            ("JULMPCLKCIB2", SpecialLocKey::PclkIn(Dir::W, 3)),
            ("JLRMPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 0)),
            ("JLRMPCLKCIB2", SpecialLocKey::PclkIn(Dir::E, 1)),
            ("JURMPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 2)),
            ("JURMPCLKCIB2", SpecialLocKey::PclkIn(Dir::E, 3)),
            ("JLLMPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 0)),
            ("JLLMPCLKCIB3", SpecialLocKey::PclkIn(Dir::S, 1)),
            ("JLRMPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 2)),
            ("JLRMPCLKCIB3", SpecialLocKey::PclkIn(Dir::S, 3)),
            ("JULMPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 0)),
            ("JULMPCLKCIB3", SpecialLocKey::PclkIn(Dir::N, 1)),
            ("JURMPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 2)),
            ("JURMPCLKCIB3", SpecialLocKey::PclkIn(Dir::N, 3)),
        ] {
            if let Some(s) = self.grid.strings.get(name) {
                xlat.insert(s, key);
            }
        }
        for &(wf, wt) in self.grid.pips.keys() {
            let wnt = self.grid.nodes[wt];
            let Some(&key) = xlat.get(&wnt.suffix) else {
                continue;
            };
            let wnf = self.grid.nodes[wf];
            let cell = self.chip.xlat_rc_wire(wnf);
            self.chip.special_loc.insert(key, cell);
        }
    }
}
