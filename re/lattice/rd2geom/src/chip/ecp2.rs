use std::collections::{BTreeMap, HashMap};

use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirV},
    grid::{ColId, ColIdExt, DieIdExt, RowId, RowIdExt},
};

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_pclk_ecp2(&mut self) {
        let clk0 = self.naming.strings.get("CLK0").unwrap();
        let jclk0 = self.naming.strings.get("JCLK0").unwrap();
        let hpbx0000 = self.naming.strings.get("HPBX0000").unwrap();
        let mut clks: BTreeMap<u8, Vec<ColId>> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wfn.suffix == hpbx0000 && (wtn.suffix == jclk0 || wtn.suffix == clk0) {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_clk {
                    clks.entry(wfn.c).or_default().push(cell.col);
                }
            }
        }
        for (_, mut cols) in clks {
            cols.sort();
            let col_start = cols[0];
            assert_eq!(col_start, self.chip.cols_pclk.next_id());
            for (i, &col) in cols.iter().enumerate() {
                assert_eq!(col, col_start + i);
            }
            // TODO: fake, fix up
            let col_drv = if cols[0] < self.chip.col_clk {
                *cols.last().unwrap()
            } else {
                *cols.first().unwrap()
            };
            self.chip.cols_pclk.push(cols.len(), col_drv);
        }
        assert_eq!(self.chip.cols_pclk[self.chip.col_clk], self.chip.col_clk);
    }

    pub(super) fn fill_sclk_ecp2(&mut self) {
        let hsbx0000 = self.naming.strings.get("HSBX0000").unwrap();
        let mut clks: BTreeMap<RowId, Vec<RowId>> = BTreeMap::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wtn.suffix == hsbx0000 {
                let cell_t = self.chip.xlat_rc_wire(wtn);
                let cell_f = self.chip.xlat_rc_wire(wfn);
                if cell_t.col == self.chip.col_clk {
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
    }

    pub(super) fn fill_eclk_tap_ecp2(&mut self) {
        let jf6 = self.naming.strings.get("JF6").unwrap();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wtn.suffix == jf6 && self.naming.strings[wfn.suffix].contains("FRC") {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_s() {
                    self.chip.columns[cell.col].eclk_tap[DirV::S] = true;
                } else if cell.row == self.chip.row_n() {
                    self.chip.columns[cell.col].eclk_tap[DirV::N] = true;
                }
            }
        }
    }

    pub(super) fn fill_io_ecp2(&mut self) {
        self.fill_io(&[
            ("PIC_L", IoGroupKind::Double),
            ("PIC_LLPCLK", IoGroupKind::Double),
            ("PIC_LUPCLK", IoGroupKind::Double),
            ("PIC_LDQS", IoGroupKind::DoubleDqs),
            ("PIC_LDQSM2", IoGroupKind::Double),
            ("PIC_LDQSM3", IoGroupKind::Double),
            ("PIC_R", IoGroupKind::Double),
            ("PIC_RLPCLK", IoGroupKind::Double),
            ("PIC_RUPCLK", IoGroupKind::Double),
            ("PIC_RDQS", IoGroupKind::DoubleDqs),
            ("PIC_RDQSM2", IoGroupKind::Double),
            ("PIC_RDQSM3", IoGroupKind::Double),
            ("PIC_RCPU", IoGroupKind::Double),
            ("PIC_B", IoGroupKind::Double),
            ("PIC_BSPL", IoGroupKind::Double),
            ("PIC_BSPR", IoGroupKind::Double),
            ("PIC_BDQS", IoGroupKind::DoubleDqs),
            ("PIC_BLPCLK", IoGroupKind::Double),
            ("PIC_BRPCLK", IoGroupKind::Double),
            ("PIC_T", IoGroupKind::Double),
            ("PIC_TSPL", IoGroupKind::Double),
            ("PIC_TSPR", IoGroupKind::Double),
            ("PIC_TLPCLK", IoGroupKind::Double),
            ("PIC_TRPCLK", IoGroupKind::Double),
        ]);
    }

    pub(super) fn fill_io_xp2(&mut self) {
        self.fill_io(&[
            ("PIC_L", IoGroupKind::Double),
            ("PIC_L_NOPIO", IoGroupKind::DoubleDummy),
            ("PIC_LDQS", IoGroupKind::DoubleDqs),
            ("PIC_LDQSM2", IoGroupKind::Double),
            ("PIC_LDQSM3", IoGroupKind::Double),
            ("PIC_R", IoGroupKind::Double),
            ("PIC_R_NOPIO", IoGroupKind::DoubleDummy),
            ("PIC_RDQS", IoGroupKind::DoubleDqs),
            ("PIC_RDQSM2", IoGroupKind::Double),
            ("PIC_RDQSM3", IoGroupKind::Double),
            ("PIC_B", IoGroupKind::Double),
            ("PIC_BSPL", IoGroupKind::Double),
            ("PIC_BSPR", IoGroupKind::Double),
            ("PIC_B5KVIQ", IoGroupKind::Double),
            ("PIC_BDQS", IoGroupKind::DoubleDqs),
            ("PIC_BLPCLK", IoGroupKind::Double),
            ("PIC_BRPCLK", IoGroupKind::Double),
            ("PIC_B_NOPIO", IoGroupKind::DoubleDummy),
            ("PIC_T", IoGroupKind::Double),
            ("PIC_TSPL", IoGroupKind::Double),
            ("PIC_TSPR", IoGroupKind::Double),
            ("PIC_T5KVIQ", IoGroupKind::Double),
            ("PIC_TDQS", IoGroupKind::DoubleDqs),
            ("PIC_TLPCLK", IoGroupKind::Double),
            ("PIC_TRPCLK", IoGroupKind::Double),
            ("PIC_T_NOPIO", IoGroupKind::DoubleDummy),
        ]);
    }

    pub(super) fn fill_io_banks_ecp2(&mut self) {
        self.fill_io_banks_8();
        for (row, rd) in &mut self.chip.rows {
            if row.to_idx() <= 8 && rd.bank.contains_key(DirH::E) {
                rd.bank[DirH::E] = 8;
            }
        }
    }

    pub(super) fn fill_pll_ecp2(&mut self) {
        let ebr_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(_, rd)| rd.kind == RowKind::Ebr)
                .map(|(row, _)| row),
        );
        let dsp_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(row, rd)| rd.kind == RowKind::Dsp && row != self.chip.row_clk)
                .map(|(row, _)| row),
        );
        for edge in [DirH::W, DirH::E] {
            self.chip.special_loc.insert(
                SpecialLocKey::Pll(PllLoc::new(edge & DirV::S, 0)),
                Chip::DIE.cell(self.chip.col_edge(edge), ebr_rows[0]),
            );
            if ebr_rows.len() == 2 {
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & DirV::N, 0)),
                    Chip::DIE.cell(self.chip.col_edge(edge), ebr_rows[1]),
                );
            }
            if !dsp_rows.is_empty() {
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & DirV::N, 1)),
                    Chip::DIE.cell(self.chip.col_edge(edge), dsp_rows[0]),
                );
            }
        }
    }

    pub(super) fn fill_pll_ecp2m(&mut self) {
        let bot_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(row, rd)| rd.kind == RowKind::Ebr && row < self.chip.row_clk)
                .map(|(row, _)| row),
        );
        let top_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(row, rd)| rd.kind == RowKind::Ebr && row >= self.chip.row_clk)
                .map(|(row, _)| row),
        );
        for edge in [DirH::W, DirH::E] {
            self.chip.special_loc.insert(
                SpecialLocKey::Pll(PllLoc::new(edge & DirV::S, 0)),
                Chip::DIE.cell(self.chip.col_edge(edge), bot_rows[0]),
            );
            if bot_rows.len() > 1 {
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & DirV::S, 1)),
                    Chip::DIE.cell(self.chip.col_edge(edge), *bot_rows.last().unwrap()),
                );
            }
            if top_rows.len() > 1 {
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & DirV::N, 1)),
                    Chip::DIE.cell(self.chip.col_edge(edge), top_rows[0]),
                );
            }
            if !top_rows.is_empty() {
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & DirV::N, 0)),
                    Chip::DIE.cell(self.chip.col_edge(edge), *top_rows.last().unwrap()),
                );
            }
        }
    }

    pub(super) fn fill_pll_xp2(&mut self) {
        for &wn in self.nodes.values() {
            if self.naming.strings[wn.suffix].ends_with("_PLL") {
                let cell = self.chip.xlat_rc_wire(wn);
                let v = cell.row.side_of(self.chip.row_clk);
                let h = cell.col.side_of(self.chip.col_clk);
                let loc = SpecialLocKey::Pll(PllLoc::new(h & v, 0));
                self.chip.special_loc.insert(loc, cell);
            }
        }
    }

    pub(super) fn fill_serdes_ecp2m(&mut self) {
        let name = self.naming.strings.get("JFF_TX_D_0_0_PCS").unwrap();
        for &wn in self.nodes.values() {
            if wn.suffix == name {
                let mut cell = self.chip.xlat_rc_wire(wn);
                if cell.col < self.chip.col_clk {
                    cell.col -= 12;
                } else {
                    cell.col -= 13;
                }
                if cell.row < self.chip.row_clk {
                    self.chip.columns[cell.col].io[DirV::S] = IoGroupKind::Serdes;
                    self.chip.columns[cell.col]
                        .bank
                        .insert(DirV::S, if cell.col < self.chip.col_clk { 14 } else { 13 });
                } else {
                    self.chip.columns[cell.col].io[DirV::N] = IoGroupKind::Serdes;
                    self.chip.columns[cell.col]
                        .bank
                        .insert(DirV::N, if cell.col < self.chip.col_clk { 11 } else { 12 });
                }
            }
        }
    }

    pub(super) fn fill_special_io_ecp2(&mut self) {
        let pll_xlat = BTreeMap::from_iter(
            self.chip
                .plls()
                .into_iter()
                .map(|(loc, bcrd)| (bcrd.cell, loc)),
        );
        for (wn, io) in self.gather_special_io() {
            let suffix = self.naming.strings[wn.suffix].as_str();
            if let Some((edge, i)) = match suffix {
                "JLPIO0" => Some((Dir::W, 0)),
                "JRPIO0" => Some((Dir::E, 0)),
                "JBPIO0" => Some((Dir::S, 0)),
                "JTPIO0" => Some((Dir::N, 0)),
                "JLPIO1" => Some((Dir::W, 1)),
                "JRPIO1" => Some((Dir::E, 1)),
                "JBPIO1" => Some((Dir::S, 1)),
                "JTPIO1" => Some((Dir::N, 1)),
                _ => None,
            } {
                self.chip
                    .special_io
                    .insert(SpecialIoKey::Clock(edge, i), io);
            } else if matches!(
                suffix,
                "JPIO0"
                    | "JPIO1"
                    | "JPLLPIO0"
                    | "JPLLPIO1"
                    | "JCLK_SSPIPIN"
                    | "JCS_SSPIPIN"
                    | "JSI_SSPIPIN"
            ) {
                // discard — redundant
            } else if let Some(pad) = match suffix {
                "JPLLCLKI0" => Some(PllPad::PllIn1),
                "JPLLCLKI3" => Some(PllPad::PllIn0),
                "JPLLCLKFB1" => Some(PllPad::PllFb),
                "JDLLCLKI0" => Some(PllPad::DllIn0),
                "JDLLCLKI3" => Some(PllPad::DllIn1),
                "JDLLCLKFB1" => Some(PllPad::DllFb),
                "JSPLLCLKI0" => Some(PllPad::PllIn1),
                "JSPLLCLKI3" => Some(PllPad::PllIn0),
                "JSPLLCLKFB1" => Some(PllPad::PllFb),
                _ => None,
            } {
                let mut cell = self.chip.xlat_rc_wire(wn);
                if cell.col == self.chip.col_w() + 2 {
                    cell.col = self.chip.col_w();
                } else if cell.col == self.chip.col_e() - 2 {
                    cell.col = self.chip.col_e();
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

    pub(super) fn fill_fabric_clock_ecp2(&mut self) {
        let mut xlat = HashMap::new();
        for (name, key) in [
            ("JCIBLLQ0", SpecialLocKey::PclkIn(Dir::W, 0)),
            ("JCIBLLQ1", SpecialLocKey::PclkIn(Dir::S, 0)),
            ("JCIBURQ0", SpecialLocKey::PclkIn(Dir::E, 2)),
            ("JCIBURQ1", SpecialLocKey::PclkIn(Dir::N, 1)),
            ("JCIBURQ2", SpecialLocKey::PclkIn(Dir::E, 3)),
            ("JCIBLRQ0", SpecialLocKey::PclkIn(Dir::E, 1)),
            ("JCIBLRQ1", SpecialLocKey::PclkIn(Dir::S, 1)),
            ("JCIBLRQ2", SpecialLocKey::PclkIn(Dir::E, 0)),
            ("JCIBULQ0", SpecialLocKey::PclkIn(Dir::W, 1)),
            ("JCIBULQ1", SpecialLocKey::PclkIn(Dir::N, 0)),
            ("JCIBL0", SpecialLocKey::SclkIn(Dir::W, 0)),
            ("JCIBL1", SpecialLocKey::SclkIn(Dir::W, 1)),
            ("JCIBL2", SpecialLocKey::SclkIn(Dir::W, 2)),
            ("JCIBL3", SpecialLocKey::SclkIn(Dir::W, 3)),
            ("JCIBR0", SpecialLocKey::SclkIn(Dir::E, 0)),
            ("JCIBR1", SpecialLocKey::SclkIn(Dir::E, 1)),
            ("JCIBR2", SpecialLocKey::SclkIn(Dir::E, 2)),
            ("JCIBR3", SpecialLocKey::SclkIn(Dir::E, 3)),
            ("JCIBB0", SpecialLocKey::SclkIn(Dir::S, 0)),
            ("JCIBB1", SpecialLocKey::SclkIn(Dir::S, 1)),
            ("JCIBB2", SpecialLocKey::SclkIn(Dir::S, 2)),
            ("JCIBB3", SpecialLocKey::SclkIn(Dir::S, 3)),
            ("JCIBT0", SpecialLocKey::SclkIn(Dir::N, 0)),
            ("JCIBT1", SpecialLocKey::SclkIn(Dir::N, 1)),
            ("JCIBT2", SpecialLocKey::SclkIn(Dir::N, 2)),
            ("JCIBT3", SpecialLocKey::SclkIn(Dir::N, 3)),
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

    pub(super) fn fill_config_io_ecp2(&mut self) {
        if self.chip.rows[self.chip.row_s() + 2].io[DirH::E] == IoGroupKind::None {
            return;
        }
        for (key, dy, iob) in [
            (SpecialIoKey::WriteN, 2, 1),
            (SpecialIoKey::Cs1N, 2, 0),
            (SpecialIoKey::CsN, 3, 1),
            (SpecialIoKey::D(0), 3, 0),
            (SpecialIoKey::D(1), 4, 1),
            (SpecialIoKey::D(2), 4, 0),
            (SpecialIoKey::D(3), 5, 1),
            (SpecialIoKey::D(4), 5, 0),
            (SpecialIoKey::D(5), 6, 1),
            (SpecialIoKey::D(6), 6, 0),
            (SpecialIoKey::D(7), 7, 1),
            (SpecialIoKey::Di, 7, 0),
            (SpecialIoKey::Dout, 8, 1),
            (SpecialIoKey::Busy, 8, 0),
        ] {
            let io = Chip::DIE
                .cell(self.chip.col_e(), self.chip.row_s() + dy)
                .bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
    }

    pub(super) fn fill_config_io_xp2(&mut self) {
        for (key, dy, iob) in [
            (SpecialIoKey::InitB, 2, 1),
            (SpecialIoKey::D(0), 2, 0),
            (SpecialIoKey::D(1), 3, 1),
            (SpecialIoKey::Cclk, 3, 0),
            (SpecialIoKey::SpiCCsB, 5, 1),
            (SpecialIoKey::SpiPCsB, 5, 0),
            (SpecialIoKey::M1, 6, 0),
            (SpecialIoKey::Done, 7, 1),
            (SpecialIoKey::ProgB, 7, 0),
        ] {
            let io = Chip::DIE
                .cell(self.chip.col_w(), self.chip.row_clk + dy)
                .bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
    }

    pub(super) fn fill_config_loc_xp2(&mut self) {
        self.chip.special_loc.insert(
            SpecialLocKey::Osc,
            Chip::DIE.cell(self.chip.col_e(), self.chip.row_n() - 1),
        );
        let row = self
            .chip
            .rows
            .ids()
            .into_iter()
            .find(|&row| self.chip.rows[row].kind == RowKind::Dsp)
            .unwrap();
        self.chip.special_loc.insert(
            SpecialLocKey::Config,
            Chip::DIE.cell(self.chip.col_e(), row),
        );
    }
}
