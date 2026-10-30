use std::collections::{BTreeMap, BTreeSet, HashMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColId, DieIdExt, RowIdExt},
};

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_pclk_ecp5(&mut self) {
        let clk0 = self.grid.strings.get("CLK0").unwrap();
        let jclk0 = self.grid.strings.get("JCLK0").unwrap();
        let hpbx0200 = self.grid.strings.get("HPBX0200").unwrap();
        let mut drivers = BTreeMap::new();
        let vptx0200 = self
            .grid
            .strings
            .get(if self.chip.kind == ChipKind::Crosslink {
                "VPTX0000"
            } else {
                "VPTX0200"
            })
            .unwrap();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[nf];
            let wtn = self.grid.nodes[nt];
            if wtn.suffix == hpbx0200 {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_clk {
                    assert_eq!(wfn.suffix, vptx0200);
                    let col_drv = self.chip.xlat_col(wfn.c) + 1;
                    match drivers.entry(wtn.c) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(col_drv);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), col_drv);
                        }
                    }
                }
            }
        }
        let mut clks: BTreeMap<ColId, (Vec<ColId>, Vec<ColId>)> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[nf];
            let wtn = self.grid.nodes[nt];
            if wfn.suffix == hpbx0200 && (wtn.suffix == jclk0 || wtn.suffix == clk0) {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_clk {
                    let col_drv = drivers[&wfn.c];
                    let cols = clks.entry(col_drv).or_default();
                    if cell.col < col_drv {
                        cols.0.push(cell.col);
                    } else {
                        cols.1.push(cell.col);
                    }
                }
            }
        }

        for (col_drv, (mut cols_w, mut cols_e)) in clks {
            cols_w.sort();
            cols_e.sort();
            let mut cols = cols_w;
            cols.extend(cols_e);
            let col_start = cols[0];
            assert_eq!(col_start, self.chip.cols_pclk.next_id());
            for (i, &col) in cols.iter().enumerate() {
                assert_eq!(col, col_start + i);
            }
            self.chip.cols_pclk.push(cols.len(), col_drv);
        }
        assert_eq!(
            self.chip.cols_pclk.range(self.chip.col_clk).start(),
            self.chip.col_clk
        );

        if self.chip.kind == ChipKind::Ecp5 {
            let hprx0000 = self.grid.strings.get("HPRX0000").unwrap();
            for &wn in self.grid.nodes.values() {
                if wn.suffix != hprx0000 {
                    continue;
                }
                let cell = self.chip.xlat_rc_wire(wn).with_col(self.chip.col_clk);
                self.chip.special_loc.insert(
                    SpecialLocKey::ClkQuarter(cell.row.side_of(self.chip.row_clk)),
                    cell,
                );
            }
        }
    }

    pub(super) fn fill_io_ecp5(&mut self) {
        self.fill_io(&[
            ("PICL2", IoGroupKind::Quad),
            ("PICL2_DQS1", IoGroupKind::QuadDqs),
            ("MIB_CIB_LR", IoGroupKind::Quad),
            ("MIB_CIB_LRC", IoGroupKind::Quad),
            ("PICR2", IoGroupKind::Quad),
            ("PICR2_DQS1", IoGroupKind::QuadDqs),
            ("MIB_CIB_LR_A", IoGroupKind::Quad),
            ("MIB_CIB_LRC_A", IoGroupKind::Quad),
            ("PICB0", IoGroupKind::Double),
            ("EFB0_PICB0", IoGroupKind::Double),
            ("EFB2_PICB0", IoGroupKind::Double),
            ("SPICB0", IoGroupKind::Single),
            ("PICT0", IoGroupKind::Double),
        ]);
        let dqs_w = BTreeSet::from_iter(
            self.chip
                .rows
                .ids()
                .filter(|&row| self.chip.rows[row].io[DirH::W] == IoGroupKind::QuadDqs)
                .map(|row| row - 3),
        );
        let dqs_e = BTreeSet::from_iter(
            self.chip
                .rows
                .ids()
                .filter(|&row| self.chip.rows[row].io[DirH::E] == IoGroupKind::QuadDqs)
                .map(|row| row - 3),
        );
        for (row, rd) in &mut self.chip.rows {
            if rd.io[DirH::W] != IoGroupKind::None {
                rd.bank
                    .insert(DirH::W, if row < self.chip.row_clk { 6 } else { 7 });
                rd.io[DirH::W] = if dqs_w.contains(&row) {
                    IoGroupKind::QuadDqs
                } else {
                    IoGroupKind::Quad
                };
            }
            if rd.io[DirH::E] != IoGroupKind::None {
                rd.bank
                    .insert(DirH::E, if row < self.chip.row_clk { 3 } else { 2 });
                rd.io[DirH::E] = if dqs_e.contains(&row) {
                    IoGroupKind::QuadDqs
                } else {
                    IoGroupKind::Quad
                };
            }
        }
        for (col, cd) in &mut self.chip.columns {
            if cd.io[DirV::S] != IoGroupKind::None {
                cd.bank
                    .insert(DirV::S, if col < self.chip.col_clk { 8 } else { 4 });
            }
            if cd.io[DirV::N] != IoGroupKind::None {
                cd.bank
                    .insert(DirV::N, if col < self.chip.col_clk { 0 } else { 1 });
            }
        }
    }

    pub(super) fn fill_bc_ecp5(&mut self) {
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[wf];
            let wtn = self.grid.nodes[wt];
            if let Some(bank) = self.grid.strings[wtn.suffix].strip_prefix("JPVT_SRC_IN0_BREFTEST")
            {
                let bank = bank.parse().unwrap();
                let cell = self.chip.xlat_rc_wire(wfn);
                self.chip.special_loc.insert(SpecialLocKey::Bc(bank), cell);
            }
        }
    }

    pub(super) fn fill_ddrdll_ecp5(&mut self) {
        let ebr_dsp_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(_, rd)| matches!(rd.kind, RowKind::Ebr | RowKind::Dsp))
                .map(|(row, _)| row),
        );
        for hv in DirHV::DIRS {
            let row = match hv.v {
                DirV::S => ebr_dsp_rows[0],
                DirV::N => *ebr_dsp_rows.last().unwrap(),
            };
            let col = match hv.h {
                DirH::W => self.chip.col_w() + 1,
                DirH::E => self.chip.col_e() - 1,
            };
            let cell = Chip::DIE.cell(col, row);
            self.chip
                .special_loc
                .insert(SpecialLocKey::DdrDll(hv), cell);
        }
    }

    pub(super) fn fill_serdes_ecp5(&mut self) {
        let cols = match self.chip.rows.len() {
            49 => [41].as_slice(),
            70 => [41, 68].as_slice(),
            94 => [45, 70].as_slice(),
            _ => unreachable!(),
        };
        for (i, &col) in cols.iter().enumerate() {
            let col = ColId::from_idx(col);
            self.chip.columns[col].io[DirV::S] = IoGroupKind::Serdes;
            self.chip.columns[col].bank.insert(DirV::S, 50 + (i as u32));
        }
    }

    pub(super) fn fill_special_io_ecp5(&mut self) {
        for (idx, dx) in [(0, -3), (1, -5), (2, 3), (3, 1)] {
            let col = self.chip.col_clk + dx;
            let io = Chip::DIE.cell(col, self.chip.row_n()).bel(bslots::IO[0]);
            self.chip
                .special_io
                .insert(SpecialIoKey::Clock(Dir::N, idx), io);
        }
        for (idx, dy, iob) in [(0, -3, 2), (1, -3, 0), (2, 0, 2), (3, 0, 0)] {
            let io = self.chip.clk_emid(Dir::W).delta(0, dy).bel(bslots::IO[iob]);
            self.chip
                .special_io
                .insert(SpecialIoKey::Clock(Dir::W, idx), io);
            let io = self.chip.clk_emid(Dir::E).delta(0, dy).bel(bslots::IO[iob]);
            self.chip
                .special_io
                .insert(SpecialIoKey::Clock(Dir::E, idx), io);
        }

        for (key, dx, iob) in [
            (SpecialIoKey::D(7), 3, 0),
            (SpecialIoKey::D(6), 3, 1),
            (SpecialIoKey::D(5), 5, 0),
            (SpecialIoKey::D(4), 5, 1),
            (SpecialIoKey::D(3), 8, 0),
            (SpecialIoKey::D(2), 8, 1),
            (SpecialIoKey::D(1), 10, 0),
            (SpecialIoKey::D(0), 10, 1),
            (SpecialIoKey::CsN, 12, 0),
            (SpecialIoKey::Cs1N, 12, 1),
            (SpecialIoKey::Di, 14, 0),
            (SpecialIoKey::Dout, 14, 1),
            (SpecialIoKey::WriteN, 17, 0),
        ] {
            let col = self.chip.col_w() + dx;
            let io = Chip::DIE.cell(col, self.chip.row_s()).bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }

        for (key, dy, iob) in [
            (SpecialIoKey::D(8), 3, 3),
            (SpecialIoKey::D(9), 3, 2),
            (SpecialIoKey::D(10), 6, 3),
            (SpecialIoKey::D(11), 6, 2),
            (SpecialIoKey::D(12), 6, 1),
            (SpecialIoKey::D(13), 6, 0),
            (SpecialIoKey::D(14), 9, 3),
            (SpecialIoKey::D(15), 9, 2),
        ] {
            let row = self.chip.row_s() + dy;
            let io = Chip::DIE.cell(self.chip.col_w(), row).bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }

        for h in [DirH::W, DirH::E] {
            self.chip.special_io.insert(
                SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(h & DirV::S, 0)),
                Chip::DIE
                    .cell(self.chip.col_edge(h), self.chip.row_s())
                    .bel(bslots::IO[2]),
            );

            let loc = PllLoc::new(h & DirV::N, 0);
            if self.chip.special_loc.contains_key(&SpecialLocKey::Pll(loc)) {
                self.chip.special_io.insert(
                    SpecialIoKey::Pll(PllPad::PllIn0, loc),
                    Chip::DIE
                        .cell(self.chip.col_edge(h), self.chip.row_n() - 12)
                        .bel(bslots::IO[0]),
                );
                self.chip.special_io.insert(
                    SpecialIoKey::Pll(PllPad::PllIn1, loc),
                    Chip::DIE
                        .cell(
                            match h {
                                DirH::W => self.chip.col_w() + 3,
                                DirH::E => self.chip.col_e() - 4,
                            },
                            self.chip.row_n(),
                        )
                        .bel(bslots::IO[0]),
                );
            }
        }
    }

    pub(super) fn fill_fabric_clock_ecp5(&mut self) {
        let mut xlat = HashMap::new();
        for (name, key) in [
            ("JLLQPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 0)),
            ("JLLQPCLKCIB1", SpecialLocKey::PclkIn(Dir::W, 1)),
            ("JULQPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 2)),
            ("JULQPCLKCIB1", SpecialLocKey::PclkIn(Dir::W, 3)),
            ("JLLMPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 4)),
            ("JLLMPCLKCIB2", SpecialLocKey::PclkIn(Dir::W, 5)),
            ("JULMPCLKCIB0", SpecialLocKey::PclkIn(Dir::W, 6)),
            ("JULMPCLKCIB2", SpecialLocKey::PclkIn(Dir::W, 7)),
            ("JLRQPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 0)),
            ("JLRQPCLKCIB1", SpecialLocKey::PclkIn(Dir::E, 1)),
            ("JURQPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 2)),
            ("JURQPCLKCIB1", SpecialLocKey::PclkIn(Dir::E, 3)),
            ("JLRMPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 4)),
            ("JLRMPCLKCIB2", SpecialLocKey::PclkIn(Dir::E, 5)),
            ("JURMPCLKCIB0", SpecialLocKey::PclkIn(Dir::E, 6)),
            ("JURMPCLKCIB2", SpecialLocKey::PclkIn(Dir::E, 7)),
            ("JBLQPCLKCIB0", SpecialLocKey::PclkIn(Dir::S, 0)),
            ("JBLQPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 1)),
            ("JBRQPCLKCIB0", SpecialLocKey::PclkIn(Dir::S, 2)),
            ("JBRQPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 3)),
            ("JLLMPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 4)),
            ("JLLMPCLKCIB3", SpecialLocKey::PclkIn(Dir::S, 5)),
            ("JLRMPCLKCIB1", SpecialLocKey::PclkIn(Dir::S, 6)),
            ("JLRMPCLKCIB3", SpecialLocKey::PclkIn(Dir::S, 7)),
            ("JTLQPCLKCIB0", SpecialLocKey::PclkIn(Dir::N, 0)),
            ("JTLQPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 1)),
            ("JTRQPCLKCIB0", SpecialLocKey::PclkIn(Dir::N, 2)),
            ("JTRQPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 3)),
            ("JULMPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 4)),
            ("JULMPCLKCIB3", SpecialLocKey::PclkIn(Dir::N, 5)),
            ("JURMPCLKCIB1", SpecialLocKey::PclkIn(Dir::N, 6)),
            ("JURMPCLKCIB3", SpecialLocKey::PclkIn(Dir::N, 7)),
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
