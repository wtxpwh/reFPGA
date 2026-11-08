use std::collections::{BTreeMap, BTreeSet, HashMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, PllLoc, PllPad, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_interconnect::{
    coord::{BelCoord, ColId, DieIdExt},
    db::PinDir,
    dir::{Dir, DirH, DirHV, DirV},
};
use prjcombine_re_lattice_naming::WireName;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_clk_scm(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "JCIBTESTB0_TESTCK" {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_pclk_scm(&mut self) {
        let hpbx0001 = self.grid.strings.get("HPBX0001").unwrap();
        let vpsx0000 = self.grid.strings.get("VPSX0000").unwrap();
        let mut clks: BTreeMap<u16, Vec<ColId>> = BTreeMap::new();
        let mut pclk_drive = BTreeSet::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[nf];
            let wtn = self.grid.nodes[nt];
            if wfn.suffix == vpsx0000 {
                let cell = self.chip.xlat_rc_wire(wfn);
                pclk_drive.insert(cell.col + 1);
            }
            if wtn.suffix == hpbx0001 {
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
            let mut col_drv = None;
            for (i, &col) in cols.iter().enumerate() {
                assert_eq!(col, col_start + i);
                if pclk_drive.contains(&col) {
                    assert_eq!(col_drv, None);
                    col_drv = Some(col);
                }
            }
            self.chip.cols_pclk.push(cols.len(), col_drv.unwrap());
        }
        assert_eq!(self.chip.cols_pclk.len(), self.chip.columns.len());
        assert_eq!(
            self.chip.cols_pclk.range(self.chip.col_clk).start(),
            self.chip.col_clk
        );
    }

    pub(super) fn fill_config_loc_scm(&mut self) {
        let cell = Chip::DIE.cell(self.chip.col_clk - 12, self.chip.row_n() - 12);
        self.chip.special_loc.insert(SpecialLocKey::Config, cell);
    }

    pub(super) fn fill_pll_scm(&mut self) {
        for hv in DirHV::DIRS {
            let col = self.chip.col_edge(hv.h);
            let row = match hv.v {
                DirV::S => self.chip.row_s(),
                DirV::N => self.chip.row_n() - 12,
            };
            let cell = Chip::DIE.cell(col, row);
            self.chip
                .special_loc
                .insert(SpecialLocKey::Pll(PllLoc::new(hv, 0)), cell);
        }
    }

    pub(super) fn fill_io_scm(&mut self) {
        self.fill_io(&[
            ("PICL1B", IoGroupKind::Quad),
            ("PICL3", IoGroupKind::Dozen),
            ("PICR1B", IoGroupKind::Quad),
            ("PICR3", IoGroupKind::Dozen),
            ("PICT1B", IoGroupKind::Quad),
            ("PICT2B", IoGroupKind::Octal),
            ("PICT3", IoGroupKind::Dozen),
            ("PIC4B1B", IoGroupKind::Quad),
            ("PIC4B3", IoGroupKind::Dozen),
            ("PIC5B1B", IoGroupKind::Quad),
            ("PIC5B3", IoGroupKind::Dozen),
            ("LPCS6", IoGroupKind::Serdes),
            ("RPCS0", IoGroupKind::Serdes),
        ]);
        let mut add_cols_n = BTreeMap::new();
        let mut add_cols_s = BTreeMap::new();
        for (col, cd) in &mut self.chip.columns {
            let dx = match cd.io[DirV::N] {
                IoGroupKind::Quad => 1,
                IoGroupKind::Octal => 2,
                IoGroupKind::Dozen => 3,
                _ => continue,
            };
            add_cols_n.insert(col - dx, cd.io[DirV::N]);
            cd.io[DirV::N] = IoGroupKind::None;
        }
        for (col, cd) in &mut self.chip.columns {
            let dx = match cd.io[DirV::S] {
                IoGroupKind::Quad => 1,
                IoGroupKind::Octal => 2,
                IoGroupKind::Dozen => 3,
                _ => continue,
            };
            add_cols_s.insert(col - dx, cd.io[DirV::S]);
            cd.io[DirV::S] = IoGroupKind::None;
        }
        for (col, kind) in add_cols_n {
            self.chip.columns[col].io[DirV::N] = kind;
        }
        for (col, kind) in add_cols_s {
            self.chip.columns[col].io[DirV::S] = kind;
        }
        let mut add_rows_w = BTreeMap::new();
        let mut add_rows_e = BTreeMap::new();
        for (row, rd) in &mut self.chip.rows {
            let dy = match rd.io[DirH::W] {
                IoGroupKind::Quad => 1,
                IoGroupKind::Octal => 2,
                IoGroupKind::Dozen => 3,
                _ => continue,
            };
            add_rows_w.insert(row - dy, rd.io[DirH::W]);
            rd.io[DirH::W] = IoGroupKind::None;
        }
        for (row, rd) in &mut self.chip.rows {
            let dy = match rd.io[DirH::E] {
                IoGroupKind::Quad => 1,
                IoGroupKind::Octal => 2,
                IoGroupKind::Dozen => 3,
                _ => continue,
            };
            add_rows_e.insert(row - dy, rd.io[DirH::E]);
            rd.io[DirH::E] = IoGroupKind::None;
        }
        for (row, kind) in add_rows_w {
            self.chip.rows[row].io[DirH::W] = kind;
        }
        for (row, kind) in add_rows_e {
            self.chip.rows[row].io[DirH::E] = kind;
        }

        self.fill_io_banks_8();
        let mut serdes_bank = 9;
        for (col, cd) in &mut self.chip.columns {
            if cd.io[DirV::N] == IoGroupKind::Serdes {
                if col < self.chip.col_clk {
                    cd.bank.insert(DirV::N, serdes_bank);
                    serdes_bank += 2;
                }
            } else if cd.io[DirV::N] != IoGroupKind::None {
                cd.bank.insert(DirV::N, 1);
            }
        }
        serdes_bank = 10;
        for (col, cd) in self.chip.columns.iter_mut().rev() {
            if cd.io[DirV::N] == IoGroupKind::Serdes && col >= self.chip.col_clk {
                cd.bank.insert(DirV::N, serdes_bank);
                serdes_bank += 2;
            }
        }
    }

    fn gather_special_io_scm(&mut self) -> BTreeMap<WireName, (BelCoord, PinDir)> {
        let jinddcka = self.grid.strings.get("JINDDCKA").unwrap();
        let jinddckb = self.grid.strings.get("JINDDCKB").unwrap();
        let jinddckc = self.grid.strings.get("JINDDCKC").unwrap();
        let jinddckd = self.grid.strings.get("JINDDCKD").unwrap();
        let jpaddoa = self.grid.strings.get("JPADDOA_PIO").unwrap();
        let jpaddob = self.grid.strings.get("JPADDOB_PIO").unwrap();
        let jpaddoc = self.grid.strings.get("JPADDOC_PIO").unwrap();
        let jpaddod = self.grid.strings.get("JPADDOD_PIO").unwrap();
        let mut pad_nodes = HashMap::new();
        for (node, &wn) in &self.grid.nodes {
            let mut cell = self.chip.xlat_rc_wire(wn);
            let (idx, dir) = if wn.suffix == jinddcka {
                (0, PinDir::Input)
            } else if wn.suffix == jinddckb {
                (1, PinDir::Input)
            } else if wn.suffix == jinddckc {
                (2, PinDir::Input)
            } else if wn.suffix == jinddckd {
                (3, PinDir::Input)
            } else if wn.suffix == jpaddoa {
                (0, PinDir::Output)
            } else if wn.suffix == jpaddob {
                (1, PinDir::Output)
            } else if wn.suffix == jpaddoc {
                (2, PinDir::Output)
            } else if wn.suffix == jpaddod {
                (3, PinDir::Output)
            } else {
                continue;
            };
            let bel = if self.chip.is_col_io(cell.col) {
                let mut ridx = 0;
                let edge = if cell.col == self.chip.col_w() {
                    DirH::W
                } else {
                    DirH::E
                };
                loop {
                    match self.chip.rows[cell.row].io[edge] {
                        IoGroupKind::None => {
                            ridx += 1;
                            cell.row -= 1;
                        }
                        IoGroupKind::Quad => break bslots::IO[idx],
                        IoGroupKind::Dozen => break bslots::IO[(2 - ridx) * 4 + idx],
                        _ => unreachable!(),
                    }
                }
            } else {
                let mut idx = idx;
                let edge = if cell.row == self.chip.row_s() {
                    DirV::S
                } else {
                    DirV::N
                };
                loop {
                    if self.chip.columns[cell.col].io[edge] == IoGroupKind::None {
                        idx += 4;
                        cell.col -= 1;
                    } else {
                        break;
                    }
                }
                bslots::IO[idx]
            };
            let io = cell.bel(bel);
            pad_nodes.insert(node, (io, dir));
        }
        let mut result = BTreeMap::new();
        for &(wf, wt) in self.grid.pips.keys() {
            if let Some(&io) = pad_nodes.get(&wf) {
                let wn = self.grid.nodes[wt];
                let suffix = self.grid.strings[wn.suffix].as_str();
                if matches!(suffix, |"JQ0"| "JQ1"
                    | "JQ2"
                    | "JQ3"
                    | "JF1"
                    | "JF4"
                    | "JF6"
                    | "JFMPIC_4"
                    | "JFMPIC_6"
                    | "JFMPIC_13")
                {
                    continue;
                }
                result.insert(wn, io);
            } else if let Some(&io) = pad_nodes.get(&wt) {
                let wn = self.grid.nodes[wf];
                let suffix = self.grid.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "JA0"
                        | "JA1"
                        | "JA2"
                        | "JA3"
                        | "JB5"
                        | "JD0"
                        | "JD3"
                        | "JTOPIC_0"
                        | "JTOPIC_1"
                        | "JTOPIC_5"
                        | "JPADDIA"
                        | "JPADDIB"
                        | "JPADDIC"
                        | "JPADDID"
                        | "JINDDA"
                        | "JINDDB"
                        | "JINDDC"
                        | "JINDDD"
                ) {
                    continue;
                }
                result.insert(wn, io);
            }
        }
        result
    }

    pub(super) fn fill_special_io_scm(&mut self) {
        for (wn, (io, _dir)) in self.gather_special_io_scm() {
            let cell = self.chip.xlat_rc_wire(wn);
            let suffix = self.grid.strings[wn.suffix].as_str();
            let key = if let Some(idx) = suffix.strip_prefix("JPIO_")
                && let Ok(idx) = idx.parse()
            {
                let mut idx: u8 = idx;
                let edge = self.chip.get_io_edge(cell);
                if edge == Dir::S && cell.col >= self.chip.col_clk {
                    idx += 8;
                }
                SpecialIoKey::Clock(edge, idx)
            } else if let Some(which) = suffix.strip_prefix("JPCK")
                && which.len() == 2
                && let Ok(bank) = which[..1].parse()
                && let Ok(idx) = which[1..].parse()
            {
                let bank: u32 = bank;
                let idx: u8 = idx;
                let (edge, idx) = match bank {
                    1 => (Dir::N, idx),
                    2 => (Dir::E, idx),
                    3 => (Dir::E, idx + 4),
                    4 => (Dir::S, idx + 8),
                    5 => (Dir::S, idx),
                    6 => (Dir::W, idx + 4),
                    7 => (Dir::W, idx),
                    _ => unreachable!(),
                };
                SpecialIoKey::Clock(edge, idx)
            } else if suffix.starts_with("JPIO_IN_") || suffix.starts_with("JPIO_FB_") {
                let pad = match suffix {
                    "JPIO_IN_A" => PllPad::PllIn0,
                    "JPIO_IN_B" => PllPad::PllIn1,
                    "JPIO_IN_C" => PllPad::DllIn0,
                    "JPIO_IN_D" => PllPad::DllIn1,
                    "JPIO_IN_E" => PllPad::DllIn2,
                    "JPIO_IN_F" => PllPad::DllIn3,
                    "JPIO_FB_A" => PllPad::PllIn1,
                    "JPIO_FB_B" => PllPad::PllIn0,
                    "JPIO_FB_C" => PllPad::DllIn1,
                    "JPIO_FB_D" => PllPad::DllIn0,
                    "JPIO_FB_E" => PllPad::DllIn3,
                    "JPIO_FB_F" => PllPad::DllIn2,
                    _ => {
                        println!(
                            "WEIRD SPECIO: R{r}C{c}_{suffix} {io}",
                            r = wn.r,
                            c = wn.c,
                            io = io.to_string(self.intdb)
                        );
                        continue;
                    }
                };
                let h = if cell.col == self.chip.col_w() {
                    DirH::W
                } else if cell.col == self.chip.col_e() {
                    DirH::E
                } else {
                    unreachable!()
                };
                let v = if cell.row == self.chip.row_s() {
                    DirV::S
                } else if cell.row == self.chip.row_n() {
                    DirV::N
                } else {
                    unreachable!()
                };
                SpecialIoKey::Pll(pad, PllLoc::new(h & v, 0))
            } else if let Some(suffix) = suffix.strip_prefix("JMPIWRDATA")
                && let Some(idx) = suffix.strip_suffix("_SYSBUS")
            {
                let idx = idx.parse().unwrap();
                SpecialIoKey::D(idx)
            } else if let Some(suffix) = suffix.strip_prefix("JMPIWRPARITY")
                && let Some(idx) = suffix.strip_suffix("_SYSBUS")
            {
                let idx = idx.parse().unwrap();
                SpecialIoKey::DP(idx)
            } else if let Some(suffix) = suffix.strip_prefix("JMPIRDDATA")
                && let Some(idx) = suffix.strip_suffix("_SYSBUS")
            {
                let idx = idx.parse().unwrap();
                SpecialIoKey::D(idx)
            } else if let Some(suffix) = suffix.strip_prefix("JMPIRDPARITY")
                && let Some(idx) = suffix.strip_suffix("_SYSBUS")
            {
                let idx = idx.parse().unwrap();
                SpecialIoKey::DP(idx)
            } else if let Some(suffix) = suffix.strip_prefix("JMPIADDR")
                && let Some(idx) = suffix.strip_suffix("_SYSBUS")
            {
                let idx: u8 = idx.parse().unwrap();
                SpecialIoKey::A(idx - 14)
            } else {
                match suffix {
                    "JCS0N_SYSBUS" => SpecialIoKey::CsN,
                    "JCS1_SYSBUS" => SpecialIoKey::Cs1,
                    "JMPITSIZ0_SYSBUS" => SpecialIoKey::A(18),
                    "JMPITSIZ1_SYSBUS" => SpecialIoKey::A(19),
                    "JMPIBDIP_SYSBUS" => SpecialIoKey::A(20),
                    "JMPIBURST_SYSBUS" => SpecialIoKey::A(21),
                    "JMPICLK_SYSBUS" => SpecialIoKey::MpiClk,
                    "JMPIRWN_SYSBUS" => SpecialIoKey::WriteN,
                    "JMPISTRBN_SYSBUS" => SpecialIoKey::ReadN,
                    "JMPITA_SYSBUS" => SpecialIoKey::MpiAckN,
                    "JMPITEA_SYSBUS" => SpecialIoKey::MpiTeaN,
                    "JMPIRETRY_SYSBUS" => SpecialIoKey::MpiRetryN,
                    "JEXTDONEI_SYSBUS" => SpecialIoKey::ExtDoneI,
                    "JEXTDONEO_SYSBUS" => SpecialIoKey::ExtDoneO,
                    "JEXTCLKP1I_SYSBUS" => SpecialIoKey::ExtClkI(1),
                    "JEXTCLKP2I_SYSBUS" => SpecialIoKey::ExtClkI(2),
                    "JEXTCLKP1O_SYSBUS" => SpecialIoKey::ExtClkO(1),
                    "JEXTCLKP2O_SYSBUS" => SpecialIoKey::ExtClkO(2),
                    _ => {
                        println!(
                            "WEIRD SPECIO: R{r}C{c}_{suffix} {io}",
                            r = wn.r,
                            c = wn.c,
                            io = io.to_string(self.intdb)
                        );
                        continue;
                    }
                }
            };
            match self.chip.special_io.entry(key) {
                btree_map::Entry::Vacant(e) => {
                    e.insert(io);
                }
                btree_map::Entry::Occupied(e) => {
                    assert_eq!(*e.get(), io);
                }
            }
        }
        let col_clk_w = self.chip.cols_pclk[self.chip.col_clk - 1];
        let col_clk_e = self.chip.cols_pclk[self.chip.col_clk];
        self.chip.special_loc.insert(
            SpecialLocKey::Bc(5),
            Chip::DIE.cell(col_clk_w, self.chip.row_s()),
        );
        self.chip.special_loc.insert(
            SpecialLocKey::Bc(4),
            Chip::DIE.cell(col_clk_e, self.chip.row_s()),
        );
        self.chip
            .special_loc
            .insert(SpecialLocKey::Bc(7), self.chip.clk_emid(Dir::W));
        self.chip
            .special_loc
            .insert(SpecialLocKey::Bc(2), self.chip.clk_emid(Dir::E));
        self.chip
            .special_loc
            .insert(SpecialLocKey::Bc(1), self.chip.clk_emid(Dir::N));
    }
}
