use std::collections::{BTreeMap, BTreeSet, HashMap};

use prjcombine_ecp::{
    chip::{
        Chip, ChipKind, Ecp3Kind, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey,
    },
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{DieIdExt, RowId},
};
use prjcombine_re_lattice_naming::WireName;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_kind_ecp3(&mut self) {
        for &wn in self.nodes.values() {
            if self.naming.strings[wn.suffix] == "JECLKCNT_DQSTEST" {
                self.chip.kind = ChipKind::Ecp3(Ecp3Kind::Ecp3A);
            }
        }
    }

    pub(super) fn fill_pclk_ecp3(&mut self) {
        let idx = (self.chip.col_sclk_idx(self.chip.col_w()) + 2) % 4;
        let hpbx = self.naming.strings.get(&format!("HPBX0{idx}00")).unwrap();
        let mut clks: BTreeMap<WireName, Vec<RowId>> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wtn.suffix == hpbx && wtn.c == 1 {
                let cell = self.chip.xlat_rc_wire(wtn);
                clks.entry(wfn).or_default().push(cell.row);
            }
        }
        let mut breaks = vec![];
        let mut clks = Vec::from_iter(clks);
        clks.sort_by_key(|(_, rows)| rows[0]);
        let mut next = RowId::from_idx(0);
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
        let clko0b = self.naming.strings.get("CLKO0T_DCC").unwrap();
        let mut drivers = BTreeSet::new();
        for &wn in self.nodes.values() {
            if wn.suffix == clko0b {
                let mut cell = self.chip.xlat_rc_wire(wn);
                if matches!(self.chip.kind, ChipKind::MachXo2(_))
                    && self.chip.rows[cell.row].kind == RowKind::Plc
                {
                    cell.row = self.chip.row_s();
                }
                drivers.insert(cell.row);
            }
        }
        for row in drivers {
            let idx = breaks.binary_search(&row).unwrap();
            let next = breaks[idx + 1];
            let num = next - self.chip.rows_pclk.next_id();
            self.chip.rows_pclk.push(num as usize, row);
        }
    }

    pub(super) fn fill_sclk_ecp3(&mut self) {
        let hsbx0000 = self.naming.strings.get("HSBX0000").unwrap();
        let mut clks: BTreeMap<RowId, Vec<RowId>> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wtn.suffix == hsbx0000 {
                let cell_t = self.chip.xlat_rc_wire(wtn);
                let cell_f = self.chip.xlat_rc_wire(wfn);
                if cell_t.col == self.chip.col_w() {
                    clks.entry(cell_f.row).or_default().push(cell_t.row);
                }
            }
        }
        let mut next = RowId::from_idx(0);
        for (_, mut rows) in clks {
            rows.sort();
            let row_start = rows[0];
            assert_eq!(row_start, next);
            for (i, &row) in rows.iter().enumerate() {
                assert_eq!(row, row_start + i);
            }
            if row_start.to_idx() != 0 {
                self.chip.rows[row_start].sclk_break = true;
            }
            next = row_start + rows.len();
        }

        let hssx_l2r = self.naming.strings.get("HSSX0000_L2R").unwrap();
        for &wn in self.nodes.values() {
            if wn.suffix == hssx_l2r {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.columns[cell.col + 1].sclk_break = true;
            }
        }

        if matches!(self.chip.kind, ChipKind::Ecp3(_)) {
            let mut breaks = vec![self.chip.col_w()];
            for (col, cd) in &self.chip.columns {
                if cd.sclk_break {
                    breaks.push(col);
                }
            }
            breaks.push(self.chip.columns.next_id());
            for (i, &col) in breaks.iter().enumerate() {
                if col == self.chip.col_w()
                    || col == self.chip.columns.next_id()
                    || col == self.chip.col_clk
                {
                    continue;
                }
                let (col_w, col_e) = if col < self.chip.col_clk {
                    (
                        breaks[i - 1],
                        if breaks[i + 1] == self.chip.col_clk {
                            self.chip.col_clk
                        } else {
                            col
                        },
                    )
                } else {
                    (
                        if breaks[i - 1] == self.chip.col_clk {
                            self.chip.col_clk
                        } else {
                            col
                        },
                        breaks[i + 1],
                    )
                };
                assert_eq!(self.chip.cols_pclk.next_id(), col_w);
                self.chip.cols_pclk.push((col_e - col_w) as usize, col);
            }
            assert_eq!(self.chip.cols_pclk.len(), self.chip.columns.len());
        } else {
            self.chip
                .cols_pclk
                .push(self.chip.columns.len(), self.chip.col_clk);
        }
    }

    pub(super) fn fill_eclk_tap_ecp3(&mut self) {
        let jf6 = self.naming.strings.get("JF6").unwrap();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wtn.suffix == jf6 && self.naming.strings[wfn.suffix].contains("TECLK") {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_s() {
                    self.chip.columns[cell.col].eclk_tap_s = true;
                } else if cell.row == self.chip.row_n() {
                    self.chip.columns[cell.col].eclk_tap_n = true;
                }
            }
        }
    }

    pub(super) fn fill_config_loc_ecp3(&mut self) {
        for &wn in self.nodes.values() {
            if self.naming.strings[wn.suffix].ends_with("_START") {
                let cell = self.chip.xlat_rc_wire(wn).delta(-3, 0);
                self.chip.special_loc.insert(SpecialLocKey::Config, cell);
            }
        }
    }

    pub(super) fn fill_io_ecp3(&mut self) {
        self.fill_io(&[
            // west
            ("PIC_L2", IoGroupKind::Quad),
            ("PIC_L2A", IoGroupKind::Quad),
            ("PIC_L2B", IoGroupKind::Quad),
            ("PIC_L2E", IoGroupKind::Quad),
            ("PIC_LDQS2A", IoGroupKind::QuadDqs),
            ("PIC_LDQS2AS", IoGroupKind::QuadDqsDummy),
            ("PIC_LDQS2B", IoGroupKind::QuadDqs),
            ("PIC_LDQS2C", IoGroupKind::QuadDqs),
            ("PIC_LDQS2D", IoGroupKind::QuadDqsDummy),
            ("PIC_LDQS2E", IoGroupKind::QuadDqsDummy),
            ("PIC_LDQS2F", IoGroupKind::QuadDqs),
            ("PICATEMB_L2EVREF", IoGroupKind::Quad),
            ("PICATEMB_L2EPT", IoGroupKind::Quad),
            ("PICATVREFL_L2EPT", IoGroupKind::Quad),
            ("LLC2", IoGroupKind::Quad),
            ("PICATPLL_L2E", IoGroupKind::Quad),
            ("PICATEMB_L2APT", IoGroupKind::Quad),
            ("PICATEMB_L2A", IoGroupKind::Quad),
            ("PICATVREFU_L2APT", IoGroupKind::Quad),
            ("PICATVREFL_L2APT", IoGroupKind::Quad),
            ("LLC0", IoGroupKind::Quad),
            ("PICATPLL_L2A", IoGroupKind::Quad),
            ("PICATPLL_L2APT", IoGroupKind::Quad),
            ("PICATEMB_L2BPT", IoGroupKind::Quad),
            ("PICATEMB_L2B", IoGroupKind::Quad),
            ("PICATVREFU_L2BPT", IoGroupKind::Quad),
            ("PICATVREFL_L2BPT", IoGroupKind::Quad),
            ("PICATDSP_L2B", IoGroupKind::Quad),
            ("LLC1", IoGroupKind::Quad),
            ("PICATPLL_L2B", IoGroupKind::Quad),
            ("PICATPLL_L2BPT", IoGroupKind::Quad),
            // east
            ("PIC_R2", IoGroupKind::Quad),
            ("PIC_RCPU2", IoGroupKind::Quad),
            ("PIC_RCPU2C", IoGroupKind::Quad),
            ("PICATEMB_RCPU2VREF", IoGroupKind::Quad),
            ("PICATEMB_RCPU2PT", IoGroupKind::Quad),
            ("PICATEMB_RCPU2", IoGroupKind::Quad),
            ("PIC_RDQS2", IoGroupKind::QuadDqs),
            ("PIC_R3DQS2", IoGroupKind::QuadDqs),
            ("PIC_RDQS2C", IoGroupKind::QuadDqs),
            ("PICATVREFL_R2PT", IoGroupKind::Quad),
            ("PICATVREFU_R2PT", IoGroupKind::Quad),
            ("PICATDSP_R2", IoGroupKind::Quad),
            ("PICATEMB_R2", IoGroupKind::Quad),
            ("LRC", IoGroupKind::Quad),
            ("PICATPLL_R2", IoGroupKind::Quad),
            ("PICATPLL_R2PT", IoGroupKind::Quad),
            // south
            ("PIC_B0", IoGroupKind::Quad),
            // north
            ("PIC_T0", IoGroupKind::Quad),
            ("PIC_TSPR0", IoGroupKind::Quad),
            ("PIC_TVIQSPR0", IoGroupKind::Quad),
            ("PIC_TCPU0", IoGroupKind::Quad),
            ("PIC_TDQS0", IoGroupKind::QuadDqs),
        ]);
    }

    pub(super) fn fill_io_banks_ecp3(&mut self) {
        let row_cfg = self.chip.special_loc[&SpecialLocKey::Config].row;
        for (row, rd) in &mut self.chip.rows {
            if row < self.chip.row_clk {
                if rd.io_w != IoGroupKind::None {
                    rd.bank_w = Some(6);
                }
                if rd.io_e != IoGroupKind::None {
                    rd.bank_e = Some(3);
                }
            } else {
                if rd.io_w != IoGroupKind::None {
                    rd.bank_w = Some(7);
                }
                if rd.io_e != IoGroupKind::None {
                    if row < row_cfg {
                        rd.bank_e = Some(2);
                    } else {
                        rd.bank_e = Some(8);
                    }
                }
            }
        }
        let col_e = self.chip.col_e();
        for (col, cd) in &mut self.chip.columns {
            if col < self.chip.col_clk {
                if cd.io_s != IoGroupKind::None {
                    cd.bank_s = Some(6);
                }
                if cd.io_n != IoGroupKind::None {
                    cd.bank_n = Some(0);
                }
            } else {
                if cd.io_s != IoGroupKind::None {
                    cd.bank_s = Some(3);
                }
                if cd.io_n != IoGroupKind::None {
                    if col < col_e - 6 {
                        cd.bank_n = Some(1);
                    } else {
                        cd.bank_n = Some(8);
                    }
                }
            }
        }
    }

    pub(super) fn fill_pll_ecp3(&mut self) {
        let mut plls: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for &wn in self.nodes.values() {
            if self.naming.strings[wn.suffix].ends_with("_PLL") {
                let cell = self.chip.xlat_rc_wire(wn);
                let v = if cell.row < self.chip.row_clk {
                    DirV::S
                } else {
                    DirV::N
                };
                let h = if cell.col < self.chip.col_clk {
                    DirH::W
                } else {
                    DirH::E
                };
                let cell = cell.delta(
                    match h {
                        DirH::W => -3,
                        DirH::E => 3,
                    },
                    0,
                );
                plls.entry(DirHV { h, v }).or_default().insert(cell);
            }
        }
        for (quad, cells) in plls {
            let mut cells = Vec::from_iter(cells);
            match quad.v {
                DirV::S => {
                    cells.sort_by_key(|cell| std::cmp::Reverse(cell.row));
                }
                DirV::N => {
                    cells.sort();
                }
            }
            for (i, cell) in cells.into_iter().enumerate() {
                let loc = SpecialLocKey::Pll(PllLoc::new(quad, i as u8));
                self.chip.special_loc.insert(loc, cell);
            }
        }
    }

    pub(super) fn fill_serdes_ecp3(&mut self) {
        let name = self.naming.strings.get("JFF_TX_D_0_0_PCS").unwrap();
        for &wn in self.nodes.values() {
            if wn.suffix == name {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.columns[cell.col].io_s = IoGroupKind::Serdes;
                self.chip.columns[cell.col].bank_s = if cell.col < self.chip.col_clk {
                    Some(14)
                } else {
                    Some(13)
                };
            }
        }
        let mut bank = 50;
        for (col, cd) in &mut self.chip.columns {
            if col < self.chip.col_clk - 18 {
                continue;
            }
            if cd.io_s == IoGroupKind::Serdes {
                cd.bank_s = Some(bank);
                bank += 2;
            }
        }
        let mut bank = 51;
        for (col, cd) in self.chip.columns.iter_mut().rev() {
            if col >= self.chip.col_clk - 18 {
                continue;
            }
            if cd.io_s == IoGroupKind::Serdes {
                cd.bank_s = Some(bank);
                bank += 2;
            }
        }
    }

    pub(super) fn fill_special_io_ecp3(&mut self) {
        let pll_xlat =
            BTreeMap::from_iter(self.chip.special_loc.iter().filter_map(|(&key, &cell)| {
                if let SpecialLocKey::Pll(loc) = key {
                    Some((cell, loc))
                } else {
                    None
                }
            }));
        for (wn, io) in self.gather_special_io() {
            let suffix = self.naming.strings[wn.suffix].as_str();
            if let Some((edge, i)) = match suffix {
                "JLPIO0" => Some((Dir::W, 0)),
                "JRPIO0" => Some((Dir::E, 0)),
                "JTPIO0" => Some((Dir::N, 0)),
                "JLPIO1" => Some((Dir::W, 1)),
                "JRPIO1" => Some((Dir::E, 1)),
                "JTPIO1" => Some((Dir::N, 1)),
                _ => None,
            } {
                self.chip
                    .special_io
                    .insert(SpecialIoKey::Clock(edge, i), io);
            } else if matches!(
                suffix,
                "JPIO1"
                    | "JPIO2"
                    | "JPLLPIO1"
                    | "JPLLPIO2"
                    | "JPLLCLKI4"
                    | "JPLLCLKI0"
                    | "JDLLCLKI3"
                    | "JDLLCLKI4"
            ) {
                // discard — redundant
            } else if let Some(pad) = match suffix {
                "JPLLCLKI3" => Some(PllPad::PllIn0),
                "JPLLCLKFB1" => Some(PllPad::PllFb),
                "JDLLCLKI0" => Some(PllPad::DllIn0),
                "JDLLCLKFB1" => Some(PllPad::DllFb),
                _ => None,
            } {
                let mut cell = self.chip.xlat_rc_wire(wn);
                if cell.col < self.chip.col_clk {
                    cell.col = self.chip.col_w() + 1;
                } else {
                    cell.col = self.chip.col_e() - 1;
                }
                let pll = pll_xlat[&cell];
                let key = SpecialIoKey::Pll(pad, pll);
                self.chip.special_io.insert(key, io);
            } else {
                println!(
                    "{name}: WEIRD SPECIO: R{r}C{c}_{suffix} {io}",
                    name = self.name,
                    r = wn.r,
                    c = wn.c,
                    io = io.to_string(self.intdb)
                );
            }
        }
    }

    pub(super) fn fill_fabric_clock_ecp3(&mut self) {
        let mut xlat = HashMap::new();
        for (name, key) in [
            ("JPCLKCIBLLQ0", SpecialLocKey::PclkIn(Dir::W, 0)),
            ("JPCLKCIBLLQ1", SpecialLocKey::PclkIn(Dir::S, 0)),
            ("JPCLKCIBURQ0", SpecialLocKey::PclkIn(Dir::E, 2)),
            ("JPCLKCIBURQ1", SpecialLocKey::PclkIn(Dir::N, 1)),
            ("JPCLKCIBURQ2", SpecialLocKey::PclkIn(Dir::E, 3)),
            ("JPCLKCIBLRQ0", SpecialLocKey::PclkIn(Dir::E, 1)),
            ("JPCLKCIBLRQ1", SpecialLocKey::PclkIn(Dir::S, 1)),
            ("JPCLKCIBLRQ2", SpecialLocKey::PclkIn(Dir::E, 0)),
            ("JPCLKCIBULQ0", SpecialLocKey::PclkIn(Dir::W, 1)),
            ("JPCLKCIBULQ1", SpecialLocKey::PclkIn(Dir::N, 0)),
            ("JPCLKCIBMID0", SpecialLocKey::PclkInMid(0)),
            ("JPCLKCIBMID1", SpecialLocKey::PclkInMid(1)),
            ("JPCLKCIBMID2", SpecialLocKey::PclkInMid(2)),
            ("JPCLKCIBMID3", SpecialLocKey::PclkInMid(3)),
            ("JPCLKCIBMID4", SpecialLocKey::PclkInMid(4)),
            ("JPCLKCIBMID5", SpecialLocKey::PclkInMid(5)),
            ("JPCLKCIBMID6", SpecialLocKey::PclkInMid(6)),
            ("JPCLKCIBMID7", SpecialLocKey::PclkInMid(7)),
            ("JSCLKCIBL0", SpecialLocKey::SclkIn(Dir::W, 0)),
            ("JSCLKCIBL1", SpecialLocKey::SclkIn(Dir::W, 1)),
            ("JSCLKCIBL2", SpecialLocKey::SclkIn(Dir::W, 2)),
            ("JSCLKCIBL3", SpecialLocKey::SclkIn(Dir::W, 3)),
            ("JSCLKCIBR0", SpecialLocKey::SclkIn(Dir::E, 0)),
            ("JSCLKCIBR1", SpecialLocKey::SclkIn(Dir::E, 1)),
            ("JSCLKCIBR2", SpecialLocKey::SclkIn(Dir::E, 2)),
            ("JSCLKCIBR3", SpecialLocKey::SclkIn(Dir::E, 3)),
            ("JSCLKCIBB0", SpecialLocKey::SclkIn(Dir::S, 0)),
            ("JSCLKCIBB1", SpecialLocKey::SclkIn(Dir::S, 1)),
            ("JSCLKCIBB2", SpecialLocKey::SclkIn(Dir::S, 2)),
            ("JSCLKCIBB3", SpecialLocKey::SclkIn(Dir::S, 3)),
            ("JSCLKCIBT0", SpecialLocKey::SclkIn(Dir::N, 0)),
            ("JSCLKCIBT1", SpecialLocKey::SclkIn(Dir::N, 1)),
            ("JSCLKCIBT2", SpecialLocKey::SclkIn(Dir::N, 2)),
            ("JSCLKCIBT3", SpecialLocKey::SclkIn(Dir::N, 3)),
        ] {
            if let Some(s) = self.naming.strings.get(name) {
                xlat.insert(s, key);
            }
        }
        for &(wf, wt) in self.grid.pips.keys() {
            let wnt = self.nodes[wt];
            let Some(&key) = xlat.get(&wnt.suffix) else {
                continue;
            };
            let wnf = self.nodes[wf];
            let cell = self.chip.xlat_rc_wire(wnf);
            self.chip.special_loc.insert(key, cell);
        }
    }

    pub(super) fn fill_config_io_ecp3(&mut self) {
        let row_cfg = self.chip.special_loc[&SpecialLocKey::Config].row;
        for (key, dy, iob) in [
            (SpecialIoKey::Busy, 0, 3),
            (SpecialIoKey::D(7), 0, 2),
            (SpecialIoKey::D(6), 0, 1),
            (SpecialIoKey::D(5), 0, 0),
            (SpecialIoKey::D(4), 3, 3),
            (SpecialIoKey::D(3), 3, 2),
            (SpecialIoKey::D(2), 3, 1),
            (SpecialIoKey::D(1), 3, 0),
            (SpecialIoKey::D(0), 6, 3),
            (SpecialIoKey::WriteN, 6, 2),
            (SpecialIoKey::MClk, 6, 1),
            (SpecialIoKey::Dout, 6, 0),
            (SpecialIoKey::CsN, 9, 3),
            (SpecialIoKey::Cs1N, 9, 2),
            (SpecialIoKey::Di, 9, 1),
        ] {
            let io = Chip::DIE
                .cell(self.chip.col_e(), row_cfg + dy)
                .bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
    }
}
