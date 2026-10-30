use std::{assert_matches, collections::BTreeMap, path::PathBuf};

use prjcombine_ecp::{
    bond::{AdcPad, AscPad, Bond, BondKind, BondPad, CfgPad, MipiPad, PfrPad, PllSet, SerdesPad},
    chip::{
        Chip, ChipKind, IoGroupKind, NexusKind, PllLoc, PllPad, RowKind, SpecialIoKey,
        SpecialLocKey,
    },
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{CellCoord, ColId, ColIdExt, DieIdExt, EdgeIoCoord, RowId, RowIdExt, TileIobId},
};
use prjcombine_re_lattice_rawdump::Part;
use prjcombine_types::{Polarity, bscan::BScanPad, enumerable::EnumMap};

use crate::{
    archive::{PkgData, Value, parse_pkg, read_archive},
    chip::ChipExt,
};

struct BondContext<'a> {
    datadirs: &'a [PathBuf],
    part: &'a Part,
    chip: &'a Chip,
    nph: &'a str,
    pkg: PkgData,
    bond: Bond,
    special_io: BTreeMap<SpecialIoKey, EdgeIoCoord>,
    serdes_xlat: BTreeMap<u32, ColId>,
    mipi_xlat: BTreeMap<u32, ColId>,
    pll_xlat: BTreeMap<CellCoord, PllLoc>,
    pll_rows: EnumMap<DirV, Vec<RowId>>,
    virt_io: BTreeMap<String, Vec<BondPad>>,
    bscan: Vec<Option<(BondPad, String)>>,
    cdone_shared: Option<EdgeIoCoord>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SpecIoResult {
    Store(SpecialIoKey, Polarity),
    Ignore,
}

impl<'a> BondContext<'a> {
    fn get_filename(&self) -> PathBuf {
        for datadir in self.datadirs {
            for sub in datadir.read_dir().unwrap() {
                let sub = sub.unwrap();
                let fname = sub
                    .path()
                    .join("data")
                    .join(format!("{nph}.pkg", nph = self.nph));
                if fname.exists() {
                    return fname;
                }
            }
        }
        panic!("cannot find pkg for {}", self.nph)
    }

    fn load_pkg(&mut self) {
        let fname = self.get_filename();
        let archive = read_archive(&fname);
        self.pkg = parse_pkg(&archive);
        // println!("{} {}", part.name, part.package);
        assert!(self.pkg.pkgs.contains(&self.part.package));
    }

    fn set_kind(&mut self) {
        self.bond.kind = if self.part.name.starts_with("LPTM21") {
            BondKind::Asc
        } else if self.part.name == "LFMNX-50" {
            BondKind::MachNx
        } else {
            BondKind::Single
        };
    }

    fn fill_xlat(&mut self) {
        for (col, cd) in &self.chip.columns {
            if cd.io[DirV::S] == IoGroupKind::Serdes {
                self.serdes_xlat.insert(cd.bank[DirV::S], col);
            }
            if cd.io[DirV::N] == IoGroupKind::Serdes {
                self.serdes_xlat.insert(cd.bank[DirV::N], col);
            }
        }
        for (col, cd) in &self.chip.columns {
            if cd.io[DirV::N] == IoGroupKind::Mipi {
                self.mipi_xlat.insert(cd.bank[DirV::N], col);
            }
        }
        for (loc, bel_pll) in self.chip.plls() {
            self.pll_xlat.insert(bel_pll.cell, loc);
        }
        for (row, rd) in &self.chip.rows {
            if matches!(rd.kind, RowKind::Ebr | RowKind::Dsp) {
                let side = row.side_of(self.chip.row_clk);
                self.pll_rows[side].push(row);
            }
        }
        if matches!(self.chip.kind, ChipKind::Ecp3(_)) {
            // indexing inside out
            self.pll_rows[DirV::S].reverse();
        } else {
            // indexing outside in
            self.pll_rows[DirV::N].reverse();
        }
    }

    fn parse_io_scm(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, mut iob_idx) = if let Some(f) = func.strip_suffix('A') {
            (f, 0)
        } else if let Some(f) = func.strip_suffix('B') {
            (f, 1)
        } else if let Some(f) = func.strip_suffix('C') {
            (f, 2)
        } else if let Some(f) = func.strip_suffix('D') {
            (f, 3)
        } else {
            (func, 0)
        };
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            let mut ridx = 0;
            while self.chip.rows[row].io[DirH::W] == IoGroupKind::None {
                row -= 1;
                ridx += 1;
            }
            match self.chip.rows[row].io[DirH::W] {
                IoGroupKind::Quad => (),
                IoGroupKind::Octal => iob_idx += (1 - ridx) * 4,
                IoGroupKind::Dozen => iob_idx += (2 - ridx) * 4,
                _ => unreachable!(),
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::W(row, iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            let mut ridx = 0;
            while self.chip.rows[row].io[DirH::E] == IoGroupKind::None {
                row -= 1;
                ridx += 1;
            }
            match self.chip.rows[row].io[DirH::E] {
                IoGroupKind::Quad => (),
                IoGroupKind::Octal => iob_idx += (1 - ridx) * 4,
                IoGroupKind::Dozen => iob_idx += (2 - ridx) * 4,
                _ => unreachable!(),
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::E(row, iob))
        } else if let Some(c) = func.strip_prefix("PB")
            && let Ok(c) = c.parse()
        {
            let mut col = self.chip.xlat_col(c);
            while self.chip.columns[col].io[DirV::S] == IoGroupKind::None {
                col -= 1;
                iob_idx += 4;
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::S(col, iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse()
        {
            let mut col = self.chip.xlat_col(c);
            while self.chip.columns[col].io[DirV::N] == IoGroupKind::None {
                col -= 1;
                iob_idx += 4;
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::N(col, iob))
        } else {
            None
        }
    }

    fn parse_io_ecp(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, iob_idx) = if let Some(f) = func.strip_suffix('A') {
            (f, 0)
        } else if let Some(f) = func.strip_suffix('B') {
            (f, 1)
        } else if let Some(f) = func.strip_suffix('C') {
            (f, 2)
        } else if let Some(f) = func.strip_suffix('D') {
            (f, 3)
        } else if let Some(f) = func.strip_suffix('E') {
            (f, 4)
        } else if let Some(f) = func.strip_suffix('F') {
            (f, 5)
        } else {
            (func, 0)
        };
        let iob = TileIobId::from_idx(iob_idx);
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse()
        {
            Some(EdgeIoCoord::W(self.chip.xlat_row(r), iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse()
        {
            Some(EdgeIoCoord::E(self.chip.xlat_row(r), iob))
        } else if let Some(c) = func.strip_prefix("PB")
            && let Ok(c) = c.parse()
        {
            Some(EdgeIoCoord::S(self.chip.xlat_col(c), iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse()
        {
            Some(EdgeIoCoord::N(self.chip.xlat_col(c), iob))
        } else {
            None
        }
    }

    fn parse_io_ecp3(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, mut iob_idx) = if let Some(f) = func.strip_suffix("E_A") {
            (f, 4)
        } else if let Some(f) = func.strip_suffix("E_B") {
            (f, 5)
        } else if let Some(f) = func.strip_suffix("E_C") {
            (f, 6)
        } else if let Some(f) = func.strip_suffix("E_D") {
            (f, 7)
        } else if let Some(f) = func.strip_suffix('A') {
            (f, 0)
        } else if let Some(f) = func.strip_suffix('B') {
            (f, 1)
        } else {
            (func, 0)
        };
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            if iob_idx < 4 {
                if self.chip.rows[row].io[DirH::W] == IoGroupKind::None {
                    row -= 2;
                } else {
                    iob_idx += 2;
                }
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::W(row, iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            if iob_idx < 4 {
                if self.chip.rows[row].io[DirH::E] == IoGroupKind::None {
                    row -= 2;
                } else {
                    iob_idx += 2;
                }
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::E(row, iob))
        } else if let Some(c) = func.strip_prefix("PB")
            && let Ok(c) = c.parse()
        {
            let mut col = self.chip.xlat_col(c);
            if iob_idx < 4 && self.chip.columns[col].io[DirV::S] == IoGroupKind::None {
                col -= 2;
                iob_idx += 2;
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::S(col, iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse()
        {
            let mut col = self.chip.xlat_col(c);
            if iob_idx < 4 && self.chip.columns[col].io[DirV::N] == IoGroupKind::None {
                col -= 2;
                iob_idx += 2;
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::N(col, iob))
        } else {
            None
        }
    }

    fn parse_io_ecp4(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, mut iob_idx) = if let Some(f) = func.strip_suffix("EA") {
            (f, 4)
        } else if let Some(f) = func.strip_suffix("EB") {
            (f, 5)
        } else if let Some(f) = func.strip_suffix("EC") {
            (f, 6)
        } else if let Some(f) = func.strip_suffix("ED") {
            (f, 7)
        } else {
            (func, 0)
        };
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            if iob_idx >= 4 {
                match (self.chip.rows[row].kind, row >= self.chip.row_clk) {
                    (RowKind::Ebr, _) => (),
                    (RowKind::Dsp, false) => {
                        row -= 1;
                        iob_idx -= 4;
                    }
                    (RowKind::Dsp, true) => {
                        iob_idx -= 4;
                        iob_idx += 2;
                    }
                    _ => unreachable!(),
                }
            } else {
                iob_idx += 3;
                while !matches!(
                    self.chip.rows[row].io[DirH::W],
                    IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
                ) {
                    iob_idx -= 1;
                    row -= 1;
                }
                if self.chip.rows[row].kind == RowKind::Dsp {
                    iob_idx -= 2;
                }
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::W(row, iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse()
        {
            let mut row = self.chip.xlat_row(r);
            if iob_idx >= 4 {
                match (self.chip.rows[row].kind, row >= self.chip.row_clk) {
                    (RowKind::Ebr, _) => (),
                    (RowKind::Dsp, false) => {
                        row -= 1;
                        iob_idx -= 4;
                    }
                    (RowKind::Dsp, true) => {
                        iob_idx -= 4;
                        iob_idx += 2;
                    }
                    _ => unreachable!(),
                }
            } else {
                iob_idx += 3;
                while !matches!(
                    self.chip.rows[row].io[DirH::E],
                    IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
                ) {
                    iob_idx -= 1;
                    row -= 1;
                }
                if self.chip.rows[row].kind == RowKind::Dsp {
                    iob_idx -= 2;
                }
            }
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::E(row, iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse()
        {
            assert_eq!(iob_idx, 0);
            let mut col = self.chip.xlat_col(c);
            while self.chip.columns[col].io[DirV::N] == IoGroupKind::None {
                iob_idx += 1;
                col -= 1;
            }
            assert!(iob_idx < 4);
            let iob = TileIobId::from_idx(iob_idx);
            Some(EdgeIoCoord::N(col, iob))
        } else {
            None
        }
    }

    fn parse_io_ecp5(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, iob_idx) = if let Some(f) = func.strip_suffix('A') {
            (f, 0)
        } else if let Some(f) = func.strip_suffix('B') {
            (f, 1)
        } else if let Some(f) = func.strip_suffix('C') {
            (f, 2)
        } else if let Some(f) = func.strip_suffix('D') {
            (f, 3)
        } else {
            (func, 0)
        };
        let iob = TileIobId::from_idx(iob_idx);
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse()
        {
            Some(EdgeIoCoord::W(self.chip.xlat_row(r) - 2, iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse()
        {
            Some(EdgeIoCoord::E(self.chip.xlat_row(r) - 2, iob))
        } else if let Some(c) = func.strip_prefix("PB")
            && let Ok(c) = c.parse()
        {
            Some(EdgeIoCoord::S(self.chip.xlat_col(c), iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse()
        {
            Some(EdgeIoCoord::N(self.chip.xlat_col(c), iob))
        } else {
            None
        }
    }

    fn parse_io(&self, func: &str) -> Option<EdgeIoCoord> {
        match self.chip.kind {
            ChipKind::Scm => self.parse_io_scm(func),
            ChipKind::Ecp3(_) => self.parse_io_ecp3(func),
            ChipKind::Ecp4 => self.parse_io_ecp4(func),
            ChipKind::Ecp5 => self.parse_io_ecp5(func),
            _ => self.parse_io_ecp(func),
        }
    }

    fn parse_pfr_io(&self, func: &str) -> Option<EdgeIoCoord> {
        let (func, iob_idx) = if let Some(f) = func.strip_suffix('A') {
            (f, 0)
        } else if let Some(f) = func.strip_suffix('B') {
            (f, 1)
        } else {
            (func, 0)
        };
        let iob = TileIobId::from_idx(iob_idx);
        if let Some(r) = func.strip_prefix("PL")
            && let Ok(r) = r.parse::<usize>()
        {
            let row = RowId::from_idx(55 - r);
            Some(EdgeIoCoord::W(row, iob))
        } else if let Some(r) = func.strip_prefix("PR")
            && let Ok(r) = r.parse::<usize>()
        {
            let row = RowId::from_idx(55 - r);
            Some(EdgeIoCoord::E(row, iob))
        } else if let Some(c) = func.strip_prefix("PB")
            && let Ok(c) = c.parse::<usize>()
        {
            let col = ColId::from_idx(c - 1);
            Some(EdgeIoCoord::S(col, iob))
        } else if let Some(c) = func.strip_prefix("PT")
            && let Ok(c) = c.parse::<usize>()
        {
            let col = ColId::from_idx(c - 1);
            Some(EdgeIoCoord::N(col, iob))
        } else {
            None
        }
    }

    fn uncompl_io(&self, io: EdgeIoCoord) -> EdgeIoCoord {
        assert_eq!(io.iob().to_idx() % 2, 1);
        io.with_iob(TileIobId::from_idx(io.iob().to_idx() - 1))
    }

    fn parse_polarity_tc(&self, p: &str) -> Option<Polarity> {
        match p {
            "T" => Some(Polarity::P),
            "C" => Some(Polarity::N),
            _ => None,
        }
    }

    fn parse_polarity_pn(&self, p: &str) -> Option<Polarity> {
        match p {
            "P" => Some(Polarity::P),
            "N" => Some(Polarity::N),
            _ => None,
        }
    }

    fn parse_io_clock(&self, io: EdgeIoCoord, bank: u32, func: &str) -> Option<SpecIoResult> {
        let pclk = func.strip_prefix("PCLK")?;
        let (which, idx) = pclk.split_once('_')?;
        let (mut pclk_bank, pol): (u32, Polarity) = if let Some(pclk_bank) = which.strip_prefix('T')
        {
            (pclk_bank.parse().ok()?, Polarity::P)
        } else if let Some(pclk_bank) = which.strip_prefix('C') {
            (pclk_bank.parse().ok()?, Polarity::N)
        } else {
            return None;
        };
        let mut idx = idx.parse().unwrap();
        if self.chip.kind == ChipKind::Scm {
            match pclk_bank {
                1 => (),
                5 => (),
                4 => idx += 8,
                6 | 3 => idx += 4,
                7 | 2 => (),
                _ => unreachable!(),
            }
        }
        if self.chip.kind == ChipKind::Ecp4 {
            match pclk_bank {
                0 => {
                    pclk_bank = 1;
                }
                1 => {
                    idx += 2;
                }
                2 => {
                    idx += 4;
                }
                3 => {
                    pclk_bank = 2;
                    idx += 6;
                }
                4 | 7 => {
                    idx += 2;
                }
                5 | 6 => (),
                _ => unreachable!(),
            }
        }
        if self.chip.kind == ChipKind::Ecp5 {
            match pclk_bank {
                0 => (),
                1 => {
                    idx += 2;
                }
                2 | 7 => {
                    idx += 2;
                }
                3 | 6 => (),
                _ => unreachable!(),
            }
        }
        if self.chip.kind == ChipKind::Crosslink {
            match pclk_bank {
                2 => (),
                1 => {
                    idx += 2;
                }
                0 => {
                    idx += 4;
                }
                _ => unreachable!(),
            }
        }
        if self.chip.kind != ChipKind::MachXo {
            assert_eq!(bank, pclk_bank);
        }
        if matches!(
            self.chip.kind,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_)
        ) {
            assert_eq!(idx, 0);
            idx = match bank {
                0 | 2 | 5 | 7 => 0,
                1 | 3 | 4 | 6 => 1,
                _ => unreachable!(),
            };
        }
        if matches!(self.chip.kind, ChipKind::MachXo2(_)) && bank >= 3 && self.chip.rows.len() >= 14
        {
            idx = (bank - 3) as u8;
        }
        if let ChipKind::Nexus(kind) = self.chip.kind {
            match io.edge() {
                Dir::S => {
                    let base = match kind {
                        NexusKind::SmallMipi
                        | NexusKind::SmallMipiPcie
                        | NexusKind::LargePcs(_) => 3,
                        NexusKind::SmallHalfIo => 2,
                        NexusKind::SmallDenseIo => 5,
                    };
                    let bidx = bank - base;
                    idx += (bidx * 4) as u8;
                }
                Dir::W => {
                    match kind {
                        NexusKind::SmallMipi
                        | NexusKind::SmallMipiPcie
                        | NexusKind::LargePcs(_) => {
                            let bidx = bank - 6;
                            idx += (bidx * 3) as u8;
                        }
                        NexusKind::SmallHalfIo => unreachable!(),
                        NexusKind::SmallDenseIo => {
                            idx += match bank {
                                7 => 0,
                                8 => 2,
                                9 => 4,
                                _ => unreachable!(),
                            };
                        }
                    };
                }
                Dir::E => {
                    match kind {
                        NexusKind::SmallMipi
                        | NexusKind::SmallMipiPcie
                        | NexusKind::LargePcs(_) => {
                            let bidx = 2 - bank;
                            idx += (bidx * 3) as u8;
                        }
                        NexusKind::SmallHalfIo => unreachable!(),
                        NexusKind::SmallDenseIo => {
                            idx += match bank {
                                4 => 0,
                                3 => 2,
                                2 => 4,
                                _ => unreachable!(),
                            };
                        }
                    };
                }
                Dir::N => match kind {
                    NexusKind::SmallMipi
                    | NexusKind::SmallMipiPcie
                    | NexusKind::LargePcs(_)
                    | NexusKind::SmallHalfIo => {}
                    NexusKind::SmallDenseIo => {
                        idx += (bank * 2) as u8;
                    }
                },
            }
        }
        Some(SpecIoResult::Store(
            SpecialIoKey::Clock(io.edge(), idx),
            pol,
        ))
    }

    fn parse_io_vref(&self, bank: u32, func: &str) -> Option<SpecIoResult> {
        let rest = func.strip_prefix("VREF")?;
        let (a, b) = rest.split_once('_')?;
        let a = a.parse::<u32>().ok()?;
        let b = b.parse::<u32>().ok()?;
        let (which, vref_bank) = if matches!(self.chip.kind, ChipKind::Nexus(_)) {
            (b, a)
        } else {
            (a, b)
        };
        assert_eq!(bank, vref_bank);
        assert_matches!(which, 1 | 2);
        Some(SpecIoResult::Store(
            if which == 1 {
                SpecialIoKey::Vref1(vref_bank)
            } else {
                SpecialIoKey::Vref2(vref_bank)
            },
            Polarity::P,
        ))
    }

    fn parse_loc_corner(&self, loc: &str) -> Option<DirHV> {
        match loc {
            "LLC" => Some(DirHV::SW),
            "ULC" => Some(DirHV::NW),
            "LRC" => Some(DirHV::SE),
            "URC" => Some(DirHV::NE),
            _ => None,
        }
    }

    fn parse_loc_pll_mid(&self, loc: &str) -> Option<PllLoc> {
        if loc.len() < 4 {
            return None;
        }
        let (hv, idx) = loc.split_at(3);
        let hv = match hv {
            "LLM" => DirHV::SW,
            "LUM" => DirHV::NW,
            "RLM" => DirHV::SE,
            "RUM" => DirHV::NE,
            _ => return None,
        };
        let idx: usize = idx.parse().ok()?;
        match self.chip.kind {
            ChipKind::Ecp2 | ChipKind::Ecp2M => {
                let cell = Chip::DIE.cell(self.chip.col_edge(hv.h), self.pll_rows[hv.v][idx]);
                Some(self.pll_xlat[&cell])
            }
            ChipKind::Ecp3(_) => {
                let cell = Chip::DIE.cell(
                    self.chip.col_edge(hv.h).step(!hv.h, 1),
                    self.pll_rows[hv.v][idx],
                );
                Some(self.pll_xlat[&cell])
            }
            _ => {
                assert_eq!(idx, 0);
                Some(PllLoc::new(hv, 0))
            }
        }
    }

    fn parse_io_pll_scm(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once('_')?;
        let (pad, pol) = match sig {
            "PLLT_IN_A" | "PLLT_FB_B" => (PllPad::PllIn0, Polarity::P),
            "PLLC_IN_A" | "PLLC_FB_B" => (PllPad::PllIn0, Polarity::N),
            "PLLT_IN_B" | "PLLT_FB_A" => (PllPad::PllIn1, Polarity::P),
            "PLLC_IN_B" | "PLLC_FB_A" => (PllPad::PllIn1, Polarity::N),
            "DLLT_IN_C" | "DLLT_FB_D" => (PllPad::DllIn0, Polarity::P),
            "DLLC_IN_C" | "DLLC_FB_D" => (PllPad::DllIn0, Polarity::N),
            "DLLT_IN_D" | "DLLT_FB_C" => (PllPad::DllIn1, Polarity::P),
            "DLLC_IN_D" | "DLLC_FB_C" => (PllPad::DllIn1, Polarity::N),
            "DLLT_IN_E" | "DLLT_FB_F" => (PllPad::DllIn2, Polarity::P),
            "DLLC_IN_E" | "DLLC_FB_F" => (PllPad::DllIn2, Polarity::N),
            "DLLT_IN_F" | "DLLT_FB_E" => (PllPad::DllIn3, Polarity::P),
            "DLLC_IN_F" | "DLLC_FB_E" => (PllPad::DllIn3, Polarity::N),
            _ => return None,
        };
        let hv = self.parse_loc_corner(loc)?;
        Some(SpecIoResult::Store(
            SpecialIoKey::Pll(pad, PllLoc::new(hv, 0)),
            pol,
        ))
    }

    fn parse_io_pll_ecp(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once("_PLL")?;
        let (pol, sig) = sig.split_once('_')?;
        let pol = self.parse_polarity_tc(pol)?;
        let loc = self.parse_loc_pll_mid(loc)?;
        let pad = match sig {
            "IN_A" => PllPad::PllIn0,
            "FB_A" => PllPad::PllFb,
            _ => return None,
        };
        Some(SpecIoResult::Store(SpecialIoKey::Pll(pad, loc), pol))
    }

    fn parse_io_pll_ecp2(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once('_')?;
        let (kind, sig) = sig.split_at_checked(4)?;
        let (pol, sig) = sig.split_once('_')?;
        let pad = match (kind, sig) {
            ("GPLL" | "SPLL", "IN_A") => PllPad::PllIn0,
            ("GPLL" | "SPLL", "FB_A") => PllPad::PllFb,
            ("GDLL", "IN_A") => PllPad::DllIn0,
            ("GDLL", "FB_A" | "FB_D") => PllPad::DllFb,
            _ => return None,
        };
        let pol = self.parse_polarity_tc(pol)?;
        let loc = self.parse_loc_pll_mid(loc)?;
        Some(SpecIoResult::Store(SpecialIoKey::Pll(pad, loc), pol))
    }

    fn parse_io_pll_xp2(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once("_GPLL")?;
        let (pol, sig) = sig.split_once('_')?;
        let pol = self.parse_polarity_tc(pol)?;
        let hv = self.parse_loc_corner(loc)?;
        let pad = match sig {
            "IN_A" => PllPad::PllIn0,
            "FB_A" => PllPad::PllFb,
            _ => return None,
        };
        Some(SpecIoResult::Store(
            SpecialIoKey::Pll(pad, PllLoc::new(hv, 0)),
            pol,
        ))
    }

    fn parse_io_pll_ecp3(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once('_')?;
        let (kind, sig) = sig.split_once('_')?;
        let (sig, pol) = sig.split_once('_')?;
        let pol = match pol {
            "A" => Polarity::P,
            "B" => Polarity::N,
            _ => return None,
        };
        let pad = match (kind, sig) {
            ("GPLLT", "IN") => PllPad::PllIn0,
            ("GPLLT", "FB") => PllPad::PllFb,
            ("GDLLT", "IN") => PllPad::DllIn0,
            ("GDLLT", "FB") => PllPad::DllFb,
            _ => return None,
        };
        let loc = self.parse_loc_pll_mid(loc)?;
        Some(SpecIoResult::Store(SpecialIoKey::Pll(pad, loc), pol))
    }

    fn parse_io_pll_machxo2(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once("_GPLL")?;
        let (pol, sig) = sig.split_once('_')?;
        let pol = self.parse_polarity_tc(pol)?;
        let hv = match loc {
            "L" => DirHV::NW,
            "R" => DirHV::NE,
            _ => return None,
        };
        let pad = match sig {
            "IN" => PllPad::PllIn0,
            "FB" => PllPad::PllFb,
            "MFGOUT1" | "MFGOUT2" => return Some(SpecIoResult::Ignore),
            _ => return None,
        };
        Some(SpecIoResult::Store(
            SpecialIoKey::Pll(pad, PllLoc::new(hv, 0)),
            pol,
        ))
    }

    fn parse_io_pll_ecp4(&self, func: &str) -> Option<SpecIoResult> {
        let (loc, sig) = func.split_once("_GPLL")?;
        let (idxpol, sig) = sig.split_once('_')?;
        let (idx, pol) = idxpol.split_at(1);
        let pol = self.parse_polarity_tc(pol)?;
        let idx: u8 = idx.parse().ok()?;
        let hv = self.parse_loc_corner(loc)?;
        let (idx, pad) = match sig {
            "IN" => {
                if self.chip.kind == ChipKind::Ecp4 {
                    (idx, PllPad::PllIn0)
                } else {
                    (0, [PllPad::PllIn0, PllPad::PllIn1][usize::from(idx)])
                }
            }
            "FB" => (idx, PllPad::PllFb),
            "MFGOUT1" | "MFGOUT2" => return Some(SpecIoResult::Ignore),
            _ => return None,
        };
        Some(SpecIoResult::Store(
            SpecialIoKey::Pll(pad, PllLoc::new(hv, idx)),
            pol,
        ))
    }

    fn parse_io_pll_crosslink(&self, func: &str) -> Option<SpecIoResult> {
        match func {
            "GPLL_MFGOUT_1" | "GPLL_MFGOUT_2" => Some(SpecIoResult::Ignore),
            "GPLLT2_0" if self.chip.kind == ChipKind::Crosslink => Some(SpecIoResult::Store(
                SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 0)),
                Polarity::P,
            )),
            "GPLLC2_0" if self.chip.kind == ChipKind::Crosslink => Some(SpecIoResult::Store(
                SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(DirHV::SE, 0)),
                Polarity::N,
            )),
            _ => None,
        }
    }

    fn parse_io_pll(&self, func: &str) -> Option<SpecIoResult> {
        match self.chip.kind {
            ChipKind::Scm => self.parse_io_pll_scm(func),
            ChipKind::Ecp | ChipKind::Xp | ChipKind::MachXo => self.parse_io_pll_ecp(func),
            ChipKind::Ecp2 | ChipKind::Ecp2M => self.parse_io_pll_ecp2(func),
            ChipKind::Xp2 => self.parse_io_pll_xp2(func),
            ChipKind::Ecp3(_) => self.parse_io_pll_ecp3(func),
            ChipKind::MachXo2(_) => self.parse_io_pll_machxo2(func),
            ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Nexus(_) => self.parse_io_pll_ecp4(func),
            ChipKind::Crosslink => self.parse_io_pll_crosslink(func),
        }
    }

    fn parse_io_adc(&self, func: &str) -> Option<SpecIoResult> {
        if !matches!(self.chip.kind, ChipKind::Nexus(_)) {
            return None;
        }
        if let Some(rest) = func.strip_prefix("ADC_C")
            && let Some((pol, idx)) = rest.split_at_checked(1)
            && let Some(pol) = self.parse_polarity_pn(pol)
            && let Ok(idx) = idx.parse()
        {
            Some(SpecIoResult::Store(SpecialIoKey::AdcCh(idx), pol))
        } else if let Some(idx_pol) = func.strip_prefix("COMP")
            && let Ok(idx) = idx_pol[0..1].parse()
            && matches!(idx, 1..=3)
            && matches!(&idx_pol[1..], "P" | "N" | "IP" | "IN")
        {
            let pol = if idx_pol.ends_with("N") {
                Polarity::N
            } else {
                Polarity::P
            };
            if &idx_pol[1..2] == "I" {
                Some(SpecIoResult::Store(SpecialIoKey::AdcCompI(idx), pol))
            } else {
                Some(SpecIoResult::Store(SpecialIoKey::AdcCh(idx - 1), pol))
            }
        } else {
            None
        }
    }

    fn parse_io_cdr(&self, func: &str) -> Option<SpecIoResult> {
        if !matches!(self.chip.kind, ChipKind::Nexus(_)) {
            return None;
        }
        match func {
            "CDR_RXP0" => Some(SpecIoResult::Store(SpecialIoKey::CdrRx(0), Polarity::P)),
            "CDR_RXN0" => Some(SpecIoResult::Store(SpecialIoKey::CdrRx(0), Polarity::N)),
            "CDR_RXP1" => Some(SpecIoResult::Store(SpecialIoKey::CdrRx(1), Polarity::P)),
            "CDR_RXN1" => Some(SpecIoResult::Store(SpecialIoKey::CdrRx(1), Polarity::N)),
            "CDR0_TESTP" | "CDR0_TESTN" | "CDR1_TESTP" | "CDR1_TESTN" => Some(SpecIoResult::Ignore),
            _ => None,
        }
    }

    fn parse_io_mipi_crosslink(&self, func: &str) -> Option<SpecIoResult> {
        let (side, pol) = match func {
            "MIPI_CLKT2_0" => (DirH::W, Polarity::P),
            "MIPI_CLKT1_0" => (DirH::E, Polarity::P),
            "MIPI_CLKC2_0" => (DirH::W, Polarity::N),
            "MIPI_CLKC1_0" => (DirH::E, Polarity::N),
            _ => return None,
        };

        Some(SpecIoResult::Store(SpecialIoKey::MipiClk(side), pol))
    }

    fn parse_io_mipi_nexus(&self, func: &str) -> Option<SpecIoResult> {
        let func = func.strip_prefix("DPHY_REFCLK")?;
        let (pol, func) = func.split_at_checked(1)?;
        let pol = self.parse_polarity_tc(pol)?;
        let (phy, clk) = func.split_once('_')?;
        let phy = phy.parse::<u8>().ok()?;
        let clk = clk.parse::<u8>().ok()?;
        let (edge, idx) = match clk {
            1 => (Dir::S, 7),
            2 => (Dir::S, 3),
            3 => (Dir::S, 1),
            4 => (Dir::N, phy),
            _ => return None,
        };
        Some(SpecIoResult::Store(SpecialIoKey::Clock(edge, idx), pol))
    }

    fn parse_io_mipi(&self, func: &str) -> Option<SpecIoResult> {
        match self.chip.kind {
            ChipKind::Crosslink => self.parse_io_mipi_crosslink(func),
            ChipKind::Nexus(_) => self.parse_io_mipi_nexus(func),
            _ => None,
        }
    }

    fn parse_io_cfg_scm(&self, func: &str) -> Option<SpecIoResult> {
        let key = if let Some(idx) = func.strip_prefix('D')
            && let Ok(idx) = idx.parse()
            && idx < 32
        {
            SpecialIoKey::D(idx)
        } else if let Some(idx) = func.strip_prefix("DP")
            && let Ok(idx) = idx.parse()
            && idx < 4
        {
            SpecialIoKey::DP(idx)
        } else if let Some(idx) = func.strip_prefix('A')
            && let Ok(idx) = idx.parse()
            && idx < 22
        {
            SpecialIoKey::A(idx)
        } else if let Some(idx) = func.strip_prefix("DEBUG_BUS")
            && let Ok(idx) = idx.parse::<u8>()
            && idx < 16
        {
            return Some(SpecIoResult::Ignore);
        } else {
            match func {
                "RDN" => SpecialIoKey::ReadN,
                "WRN" => SpecialIoKey::WriteN,
                "MPICLK" => SpecialIoKey::MpiClk,
                "QOUT" => SpecialIoKey::Qout,
                "DOUT" => SpecialIoKey::Dout,
                "CS1" => SpecialIoKey::Cs1,
                "CS0N" => SpecialIoKey::CsN,
                "RDY" => SpecialIoKey::Busy,
                "MPIACKN" => SpecialIoKey::MpiAckN,
                "MPIRTRYN" => SpecialIoKey::MpiRetryN,
                "MPITEAN" => SpecialIoKey::MpiTeaN,
                "LDCN" => SpecialIoKey::Ldc,
                "HDC" => SpecialIoKey::Hdc,
                "EXTDONEI" => SpecialIoKey::ExtDoneI,
                "EXTDONEO" => SpecialIoKey::ExtDoneO,
                "EXTCLKP1I" => SpecialIoKey::ExtClkI(1),
                "EXTCLKP2I" => SpecialIoKey::ExtClkI(2),
                "EXTCLKP1O" => SpecialIoKey::ExtClkO(1),
                "EXTCLKP2O" => SpecialIoKey::ExtClkO(2),
                "DIFFR_2" => SpecialIoKey::DiffR(2),
                "DIFFR_3" => SpecialIoKey::DiffR(3),
                "DIFFR_6" => SpecialIoKey::DiffR(6),
                "DIFFR_7" => SpecialIoKey::DiffR(7),
                "TESTCFGN" => return Some(SpecIoResult::Ignore),
                _ => return None,
            }
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_ecp(&self, func: &str) -> Option<SpecIoResult> {
        let key = if let Some(idx) = func.strip_prefix('D')
            && let Ok(idx) = idx.parse()
            && idx < 8
        {
            SpecialIoKey::D(idx)
        } else {
            match func {
                "WRITEN" => SpecialIoKey::WriteN,
                "CS1N" => SpecialIoKey::Cs1N,
                "CSN" => SpecialIoKey::CsN,
                "DI" => SpecialIoKey::Di,
                "DOUT" | "DOUT,CSON" | "DOUT_CSON" => SpecialIoKey::Dout,
                "BUSY" => SpecialIoKey::Busy,
                "DQS" => return Some(SpecIoResult::Ignore),
                _ => return None,
            }
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_machxo(&self, func: &str) -> Option<SpecIoResult> {
        let key = if let Some(idx) = func.strip_prefix('D')
            && let Ok(idx) = idx.parse::<u8>()
            && idx < 8
        {
            return Some(SpecIoResult::Ignore);
        } else {
            match func {
                "TSALLPAD" => SpecialIoKey::TsAll,
                "GSR_PADN" => SpecialIoKey::Gsr,
                _ => return None,
            }
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_xp2(&self, func: &str) -> Option<SpecIoResult> {
        let key = match func {
            "INITN" => SpecialIoKey::InitB,
            "SI" => SpecialIoKey::D(0),
            "SO" => SpecialIoKey::D(1),
            "CCLK" => SpecialIoKey::Cclk,
            "CSSPIN" => SpecialIoKey::SpiCCsB,
            "CSSPISN" => SpecialIoKey::SpiPCsB,
            "CFG1" => SpecialIoKey::M1,
            "DONE" => SpecialIoKey::Done,
            "PROGRAMN" => SpecialIoKey::ProgB,
            "MFG_EXT_CLK" | "FL_EXT_PULSE_D" | "FL_EXT_PULSE_G" => {
                return Some(SpecIoResult::Ignore);
            }
            _ => return None,
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_ecp3(&self, func: &str) -> Option<SpecIoResult> {
        let key = match func {
            "WRITEN" => SpecialIoKey::WriteN,
            "CS1N" | "HOLDN" | "CONT2N" | "RDY" => SpecialIoKey::Cs1N,
            "CSN" | "SN" | "CONT1N" | "OEN" => SpecialIoKey::CsN,
            "DI" | "CSSPI0N" | "CEN" | "CSSPIN" => SpecialIoKey::Di,
            "DOUT" | "CSON" | "CSSPI1N" => SpecialIoKey::Dout,
            "BUSY" | "SISPI" | "AVDN" => SpecialIoKey::Busy,
            "D0" | "SPIFASTN" => SpecialIoKey::D(0),
            "D1" => SpecialIoKey::D(1),
            "D2" => SpecialIoKey::D(2),
            "D3" | "SI" => SpecialIoKey::D(3),
            "D4" | "SO" => SpecialIoKey::D(4),
            "D5" => SpecialIoKey::D(5),
            "D6" | "SPID1" => SpecialIoKey::D(6),
            "D7" | "SPID0" => SpecialIoKey::D(7),
            "MCLK" => SpecialIoKey::MClk,
            _ => return None,
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_machxo2(&self, func: &str) -> Option<SpecIoResult> {
        let key = match func {
            "SI" | "SISPI" | "IO0" | "MD0" => SpecialIoKey::D(0),
            "SO" | "SPISO" | "IO1" | "MD1" => SpecialIoKey::D(1),
            "SCL" | "IO2" | "MD2" => SpecialIoKey::D(2),
            "SDA" | "IO3" | "MD3" => SpecialIoKey::D(3),
            "MCLK" | "CCLK" => SpecialIoKey::Cclk,
            "CSSPIN" | "MD4" => SpecialIoKey::SpiCCsB,
            "SN" | "MD5" => SpecialIoKey::SpiPCsB,
            "JTAGENB" | "MD6" => SpecialIoKey::JtagEn,
            "TDI" | "MD7" => SpecialIoKey::Tdi,
            "TDO" => SpecialIoKey::Tdo,
            "TMS" => SpecialIoKey::Tms,
            "TCK" | "TEST_CLK" => SpecialIoKey::Tck,
            "DONE" => SpecialIoKey::Done,
            "PROGRAMN" => SpecialIoKey::ProgB,
            "INITN" => SpecialIoKey::InitB,
            "TDIL" | "TDIB" | "TDOB" | "TDIR" | "TDOR" | "TDOT" | "SCAN_SHFT_ENB"
            | "INTEST_OVER" | "ATB_SENSE" | "ATB_FORCE" | "ATB_SENSE_1" | "ATB_FORCE_1"
            | "ATB_SENSE_2" | "ATB_FORCE_2" => {
                return Some(SpecIoResult::Ignore);
            }
            _ => return None,
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_ecp4(&self, func: &str) -> Option<SpecIoResult> {
        let key = if let Some(idx) = func.strip_prefix('D')
            && let Ok(idx) = idx.parse()
            && idx < 16
        {
            SpecialIoKey::D(idx)
        } else if let Some(idx) = func.strip_prefix("IO")
            && let Ok(idx) = idx.parse()
            && idx < 16
        {
            SpecialIoKey::D(idx)
        } else if let Some(idx) = func
            .strip_prefix("TDI")
            .or(func.strip_prefix("TDO"))
            .or(func.strip_prefix("GR_PCLK"))
            && let Ok(idx) = idx.parse::<u8>()
            && idx < 8
        {
            return Some(SpecIoResult::Ignore);
        } else if let Some(rest) = func.strip_prefix("S")
            && let Some((idx, io)) = rest.split_once('_')
            && matches!(io, "IN" | "OUT")
            && let Ok(idx) = idx.parse::<u8>()
            && idx < 8
        {
            return Some(SpecIoResult::Ignore);
        } else if let Some(rest) = func.strip_prefix("GR_PCLK")
            && let Some((a, b)) = rest.split_once('_')
            && let Ok(a) = a.parse::<u8>()
            && let Ok(b) = b.parse::<u8>()
            && a < 8
            && b < 2
        {
            return Some(SpecIoResult::Ignore);
        } else {
            match func {
                "SISPI1" | "SI" | "MOSI" => SpecialIoKey::D(0),
                "SPISO1" | "SO" | "MISO" => SpecialIoKey::D(1),
                "SISPI2" | "MOSI2" => SpecialIoKey::D(4),
                "SPISO2" | "MISO2" => SpecialIoKey::D(5),
                "SCL" | "WRITEN" => SpecialIoKey::WriteN,
                "SDA" | "CS1N" => SpecialIoKey::Cs1N,
                "SN" | "CSN" => SpecialIoKey::CsN,
                "HOLDN" | "DI" | "BUSY" | "CSSPIN" | "CEN" => SpecialIoKey::Di,
                "DOUT" | "CSON" => SpecialIoKey::Dout,
                "INTEST_OVER" | "TDIB" | "ATB_FORCE" | "ATB_SENSE" | "SCAN_SHFT_EN" => {
                    return Some(SpecIoResult::Ignore);
                }
                _ => return None,
            }
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_crosslink(&self, func: &str) -> Option<SpecIoResult> {
        let key = match func {
            "PMU_WKUPN" => SpecialIoKey::PmuWakeupN,
            "MOSI_D0" => SpecialIoKey::D(0),
            "MISO_D1" => SpecialIoKey::D(1),
            "D2" | "USER_SDA" => SpecialIoKey::D(2),
            "D3" | "USER_SCL" => SpecialIoKey::D(3),
            "CSN" | "SPI_SS" | "SCL" => SpecialIoKey::CsN,
            "SPI_SCK" | "MCK" | "SDA" => SpecialIoKey::Cclk,
            "GR_PCLK1_0" | "GR_PCLK2_0" | "ATB_SENSE" | "ATB_FORCE" => {
                return Some(SpecIoResult::Ignore);
            }
            _ => return None,
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg_nexus(&self, func: &str) -> Option<SpecIoResult> {
        let key = if let Some(idx) = func.strip_prefix("SD")
            && let Ok(idx) = idx.parse()
            && idx < 16
        {
            SpecialIoKey::D(idx)
        } else if let Some(rest) = func.strip_prefix("S")
            && let Some((idx, io)) = rest.split_once('_')
            && matches!(io, "IN" | "OUT")
            && let Ok(idx) = idx.parse::<u8>()
            && idx < 12
        {
            return Some(SpecIoResult::Ignore);
        } else {
            match func {
                "TCK" | "SCLK" | "PMU_EXT_CLK" => SpecialIoKey::Cclk,
                "TMS" | "SCSN" => SpecialIoKey::SpiPCsB,
                "PMU_WAKEUPN" | "PMU_WAKEUP" => SpecialIoKey::PmuWakeupN,
                "TDI" | "SSI" => SpecialIoKey::D(0),
                "TDO" | "SSO" => SpecialIoKey::D(1),
                "SCL" | "USER_SCL" => SpecialIoKey::D(2),
                "SDA" | "USER_SDA" => SpecialIoKey::D(3),
                "MD0" | "MOSI" => SpecialIoKey::SpiCD(0),
                "MD1" | "MISO" => SpecialIoKey::SpiCD(1),
                "MD2" => SpecialIoKey::SpiCD(2),
                "MD3" => SpecialIoKey::SpiCD(3),
                "MCLK" => SpecialIoKey::MClk,
                "MCSN" => SpecialIoKey::SpiCCsB,
                "MCSNO" | "MSDO" => SpecialIoKey::Dout,
                "DONE" => SpecialIoKey::Done,
                "PROGRAMN" => SpecialIoKey::ProgB,
                "INITN" => SpecialIoKey::InitB,
                "EIO" | "ATB_SENSE" | "ATB_FORCE" | "OSC_HI" | "OSC_BURST" => {
                    return Some(SpecIoResult::Ignore);
                }
                _ => return None,
            }
        };
        Some(SpecIoResult::Store(key, Polarity::P))
    }

    fn parse_io_cfg(&self, func: &str) -> Option<SpecIoResult> {
        match self.chip.kind {
            ChipKind::Scm => self.parse_io_cfg_scm(func),
            ChipKind::Ecp | ChipKind::Xp | ChipKind::Ecp2 | ChipKind::Ecp2M => {
                self.parse_io_cfg_ecp(func)
            }
            ChipKind::MachXo => self.parse_io_cfg_machxo(func),
            ChipKind::Xp2 => self.parse_io_cfg_xp2(func),
            ChipKind::Ecp3(_) => self.parse_io_cfg_ecp3(func),
            ChipKind::MachXo2(_) => self.parse_io_cfg_machxo2(func),
            ChipKind::Ecp4 | ChipKind::Ecp5 => self.parse_io_cfg_ecp4(func),
            ChipKind::Crosslink => self.parse_io_cfg_crosslink(func),
            ChipKind::Nexus(_) => self.parse_io_cfg_nexus(func),
        }
    }

    fn parse_io_special(
        &self,
        pin: &str,
        io: EdgeIoCoord,
        bank: u32,
        func: &str,
    ) -> Option<(SpecialIoKey, bool)> {
        if func == "Unused" {
            None
        } else if let Some(res) = self
            .parse_io_clock(io, bank, func)
            .or_else(|| self.parse_io_vref(bank, func))
            .or_else(|| self.parse_io_pll(func))
            .or_else(|| self.parse_io_adc(func))
            .or_else(|| self.parse_io_cdr(func))
            .or_else(|| self.parse_io_mipi(func))
            .or_else(|| self.parse_io_cfg(func))
        {
            match res {
                SpecIoResult::Store(key, pol) => Some((key, pol == Polarity::N)),
                SpecIoResult::Ignore => None,
            }
        } else {
            println!(
                "\t{name}: unknown spec {pin:5} {func} {bank} {io}",
                name = self.part.name
            );
            None
        }
    }

    fn parse_pad_serdes_scm(&self, func: &str, bank: i32) -> Option<BondPad> {
        let (abcd, rest) = func.split_once('_')?;
        let (pad, lr) = rest.rsplit_once('_')?;
        let exp_bank = match (abcd, lr) {
            ("A", "L") => 9,
            ("B", "L") => 11,
            ("C", "L") => 13,
            ("D", "L") => 15,
            ("A", "R") => 10,
            ("B", "R") => 12,
            ("C", "R") => 14,
            ("D", "R") => 16,
            _ => unreachable!(),
        };
        let pad = match pad {
            "REFCLKP" => SerdesPad::ClkP,
            "REFCLKN" => SerdesPad::ClkN,
            "RXREFCLKP" => SerdesPad::RxClkP,
            "RXREFCLKN" => SerdesPad::RxClkN,
            "VDDAX25" => SerdesPad::VccAux25,
            "VDDRX01" => SerdesPad::VccRxPair(0),
            "VDDRX23" => SerdesPad::VccRxPair(2),
            _ => {
                if pad.is_empty() {
                    return None;
                }
                let (pad, ch) = pad.split_at(pad.len() - 1);
                let ch = ch.parse::<u8>().ok()?;
                match pad {
                    "HDINP" => SerdesPad::InP(ch),
                    "HDINN" => SerdesPad::InN(ch),
                    "HDOUTP" => SerdesPad::OutP(ch),
                    "HDOUTN" => SerdesPad::OutN(ch),
                    "VDDRX" => SerdesPad::VccRx(ch),
                    "VDDIB" => SerdesPad::VccIB(ch),
                    "VDDOB" => SerdesPad::VccOB(ch),
                    _ => return None,
                }
            }
        };
        if !matches!(pad, SerdesPad::VccRxPair(_)) {
            assert_eq!(bank, exp_bank as i32);
        }
        let col = self.serdes_xlat[&exp_bank];
        Some(BondPad::Serdes(DirV::N, col, pad))
    }

    fn parse_pad_serdes_common_scm(&self, func: &str) -> Option<BondPad> {
        let (bank, pad) = match func {
            "VDDAX25_L" => (9, SerdesPad::VccAux25),
            "VDDAX25_R" => (10, SerdesPad::VccAux25),
            "RESP_ULC" => (9, SerdesPad::ResP),
            "RESP_URC" => (10, SerdesPad::ResP),
            "RESPN_ULC" => (9, SerdesPad::ResPN),
            "RESPN_URC" => (10, SerdesPad::ResPN),
            _ => return None,
        };
        Some(BondPad::Serdes(DirV::N, self.serdes_xlat[&bank], pad))
    }

    fn parse_pad_serdes_ecp2m(&self, func: &str, bank: i32) -> Option<BondPad> {
        let (corner, pad) = func.split_once("_SQ_")?;
        let hv = self.parse_loc_corner(corner)?;
        let col = match hv.h {
            DirH::W => self.chip.col_w() + 1,
            DirH::E => self.chip.col_e() - 27,
        };
        let pad = match pad {
            "REFCLKP" => SerdesPad::ClkP,
            "REFCLKN" => SerdesPad::ClkN,
            "VCCAUX33" => SerdesPad::VccAux33,
            "VCCP" => SerdesPad::VccP,
            _ => {
                if pad.is_empty() {
                    return None;
                }
                let (pad, ch) = pad.split_at(pad.len() - 1);
                let ch = ch.parse::<u8>().ok()?;
                match pad {
                    "HDINP" => SerdesPad::InP(ch),
                    "HDINN" => SerdesPad::InN(ch),
                    "HDOUTP" => SerdesPad::OutP(ch),
                    "HDOUTN" => SerdesPad::OutN(ch),
                    "VCCTX" => SerdesPad::VccTx(ch),
                    "VCCRX" => SerdesPad::VccRx(ch),
                    "VCCIB" => SerdesPad::VccIB(ch),
                    "VCCOB" => SerdesPad::VccOB(ch),
                    _ => return None,
                }
            }
        };
        let exp_bank = self.chip.columns[col].bank[hv.v];
        assert_eq!(bank, exp_bank as i32);
        Some(BondPad::Serdes(hv.v, col, pad))
    }

    fn parse_pad_serdes_ecp3(&self, func: &str, bank: i32) -> Option<BondPad> {
        let (loc, pad) = func.split_once('_')?;
        let exp_bank = match loc {
            "PCSA" => 50,
            "PCSB" => 51,
            "PCSC" => 52,
            "PCSD" => 53,
            _ => return None,
        };
        let pad = match pad {
            "REFCLKP" => SerdesPad::ClkP,
            "REFCLKN" => SerdesPad::ClkN,
            "VCCTX01" => SerdesPad::VccTx(0),
            "VCCTX23" => SerdesPad::VccTx(2),
            "VCCP" => SerdesPad::VccP,
            _ => {
                if pad.is_empty() {
                    return None;
                }
                let (pad, ch) = pad.split_at(pad.len() - 1);
                let ch = ch.parse::<u8>().ok()?;
                match pad {
                    "HDINP" => SerdesPad::InP(ch),
                    "HDINN" => SerdesPad::InN(ch),
                    "HDOUTP" => SerdesPad::OutP(ch),
                    "HDOUTN" => SerdesPad::OutN(ch),
                    "VCCRX" => SerdesPad::VccRx(ch),
                    "VCCIB" => SerdesPad::VccIB(ch),
                    "VCCOB" => SerdesPad::VccOB(ch),
                    _ => return None,
                }
            }
        };
        let col = self.serdes_xlat[&exp_bank];
        assert_eq!(bank, exp_bank as i32);
        Some(BondPad::Serdes(DirV::S, col, pad))
    }

    fn parse_pad_serdes_ecp4(&self, func: &str) -> Option<BondPad> {
        let cell = self
            .chip
            .special_loc
            .get(&SpecialLocKey::SerdesSingle)
            .or_else(|| self.chip.special_loc.get(&SpecialLocKey::SerdesDouble))
            .or_else(|| self.chip.special_loc.get(&SpecialLocKey::SerdesTriple))
            .copied()
            .unwrap();
        let (sig, which) = func.split_once("_Q")?;
        let (quad, pad) = if let Some((quad, ch)) = which.split_once("CH") {
            let quad: usize = quad.parse().ok()?;
            let ch = ch.parse().ok()?;
            let pad = match sig {
                "HDTXP" => SerdesPad::OutP(ch),
                "HDTXN" => SerdesPad::OutN(ch),
                "HDRXP" => SerdesPad::InP(ch),
                "HDRXN" => SerdesPad::InN(ch),
                _ => return None,
            };
            (quad, pad)
        } else {
            let quad: usize = which.parse().ok()?;
            let pad = match sig {
                "REFCLKP" => SerdesPad::ClkP,
                "REFCLKN" => SerdesPad::ClkN,
                _ => return None,
            };
            (quad, pad)
        };
        Some(BondPad::Serdes(DirV::S, cell.col + quad, pad))
    }

    fn parse_pad_serdes_ecp4_corner(&self, func: &str) -> Option<BondPad> {
        let pad = match func {
            "REFCLKP_CORNER" => SerdesPad::ClkP,
            "REFCLKN_CORNER" => SerdesPad::ClkN,
            "RXANTOUTP" => SerdesPad::RxantOutP,
            "RXANTOUTN" => SerdesPad::RxantOutN,
            "AUXTSTPADOUTP" => SerdesPad::AuxTstPadOutP,
            "AUXTSTPADOUTN" => SerdesPad::AuxTstPadOutN,
            _ => return None,
        };
        Some(BondPad::SerdesCorner(pad))
    }

    fn parse_pad_serdes_ecp5(&self, func: &str, bank: i32) -> Option<BondPad> {
        let (sig, which) = func.split_once("_D")?;
        let (dual, pad) = if let Some((dual, ch)) = which.split_once("CH") {
            let dual: usize = dual.parse().ok()?;
            let ch = ch.parse().ok()?;
            let pad = match sig {
                "HDTXP0" => SerdesPad::OutP(ch),
                "HDTXN0" => SerdesPad::OutN(ch),
                "HDRXP0" => SerdesPad::InP(ch),
                "HDRXN0" => SerdesPad::InN(ch),
                "VCCHRX0" if ch == 0 => SerdesPad::VccRx(0),
                "VCCHRX1" if ch == 1 => SerdesPad::VccRx(1),
                "VCCHTX0" if ch == 0 => SerdesPad::VccTx(0),
                "VCCHTX1" if ch == 1 => SerdesPad::VccTx(1),
                _ => return None,
            };
            (dual, pad)
        } else {
            let dual: usize = which.parse().ok()?;
            let pad = match sig {
                "REFCLKP" => SerdesPad::ClkP,
                "REFCLKN" => SerdesPad::ClkN,
                "ATSTP" => SerdesPad::AuxTstPadOutP,
                "ATSTN" => SerdesPad::AuxTstPadOutN,
                _ => return None,
            };
            (dual, pad)
        };
        let exp_bank = match dual {
            0 => 50,
            1 => 51,
            _ => unreachable!(),
        };
        if matches!(
            pad,
            SerdesPad::OutP(_)
                | SerdesPad::OutN(_)
                | SerdesPad::InP(_)
                | SerdesPad::InN(_)
                | SerdesPad::ClkP
                | SerdesPad::ClkN
        ) {
            assert_eq!(bank, exp_bank as i32);
        }

        let col = self.serdes_xlat[&exp_bank];
        Some(BondPad::Serdes(DirV::S, col, pad))
    }

    fn parse_pad_serdes_ecp5_common(&self, func: &str) -> Option<BondPad> {
        let (bank, pad) = match func {
            "VCCA0" => (50, SerdesPad::VccA),
            "VCCA1" => (51, SerdesPad::VccA),
            "VCCAUXA0" => (50, SerdesPad::VccAuxA),
            "VCCAUXA1" => (51, SerdesPad::VccAuxA),
            "VCCHRX" => (50, SerdesPad::VccTxCommon),
            _ => return None,
        };
        Some(BondPad::Serdes(DirV::S, self.serdes_xlat[&bank], pad))
    }

    fn parse_pad_serdes_nexus(&self, func: &str, bank: i32) -> Option<BondPad> {
        // TODO
        let _ = func;
        let _ = bank;
        None
    }

    fn parse_pad_serdes(&self, func: &str, bank: i32) -> Option<BondPad> {
        match self.chip.kind {
            ChipKind::Scm => self
                .parse_pad_serdes_scm(func, bank)
                .or(self.parse_pad_serdes_common_scm(func)),
            ChipKind::Ecp2M => self.parse_pad_serdes_ecp2m(func, bank),
            ChipKind::Ecp3(_) => self.parse_pad_serdes_ecp3(func, bank),
            ChipKind::Ecp4 => self
                .parse_pad_serdes_ecp4(func)
                .or_else(|| self.parse_pad_serdes_ecp4_corner(func)),
            ChipKind::Ecp5 => self
                .parse_pad_serdes_ecp5(func, bank)
                .or_else(|| self.parse_pad_serdes_ecp5_common(func)),
            ChipKind::Nexus(_) => self.parse_pad_serdes_nexus(func, bank),
            _ => None,
        }
    }

    fn parse_pad_mipi_crosslink(&self, func: &str, bank: i32) -> Option<BondPad> {
        if let Some(suffix) = func.strip_prefix("DPHY") {
            let (idx, sig) = suffix.split_once('_')?;
            let idx: u32 = idx.parse().ok()?;
            let pad = match sig {
                "DP0" => MipiPad::DataP(0),
                "DN0" => MipiPad::DataN(0),
                "DP1" => MipiPad::DataP(1),
                "DN1" => MipiPad::DataN(1),
                "DP2" => MipiPad::DataP(2),
                "DN2" => MipiPad::DataN(2),
                "DP3" => MipiPad::DataP(3),
                "DN3" => MipiPad::DataN(3),
                "CKP" => MipiPad::ClkP,
                "CKN" => MipiPad::ClkN,
                _ => return None,
            };
            let exp_bank = 60 + idx;
            let col = self.mipi_xlat[&exp_bank];
            assert_eq!(bank, exp_bank as i32);
            Some(BondPad::Mipi(col, pad))
        } else if let Some((sig, which)) = func.split_once("_DPHY") {
            let pad = match sig {
                "VCCA" => MipiPad::Vcc,
                "VSSA" => MipiPad::Gnd,
                "VCCPLL" => MipiPad::VccPll,
                "VSSPLL" => MipiPad::GndPll,
                "VCCMU" => MipiPad::VccMu,
                "VSSMU" => MipiPad::GndMu,
                _ => return None,
            };
            if which == "X" {
                Some(BondPad::MipiCommon(pad))
            } else {
                let idx: u32 = which.parse().unwrap();
                let exp_bank = 60 + idx;
                let col = self.mipi_xlat[&exp_bank];
                Some(BondPad::Mipi(col, pad))
            }
        } else {
            None
        }
    }

    fn parse_pad_mipi_nexus(&self, func: &str, bank: i32) -> Option<BondPad> {
        if let Some(suffix) = func.strip_prefix("DPHY") {
            let (idx, sig) = suffix.split_once('_')?;
            let idx: u32 = idx.parse().ok()?;
            let pad = match sig {
                "DP0" => MipiPad::DataP(0),
                "DN0" => MipiPad::DataN(0),
                "DP1" => MipiPad::DataP(1),
                "DN1" => MipiPad::DataN(1),
                "DP2" => MipiPad::DataP(2),
                "DN2" => MipiPad::DataN(2),
                "DP3" => MipiPad::DataP(3),
                "DN3" => MipiPad::DataN(3),
                "CKP" => MipiPad::ClkP,
                "CKN" => MipiPad::ClkN,
                _ => return None,
            };
            let exp_bank = 60 + idx;
            assert_eq!(bank, exp_bank as i32);
            let col = self.chip.special_loc[&SpecialLocKey::Mipi].col + (idx as usize);
            Some(BondPad::Mipi(col, pad))
        } else {
            None
        }
    }

    fn parse_pad_mipi(&self, func: &str, bank: i32) -> Option<BondPad> {
        match self.chip.kind {
            ChipKind::Crosslink => self.parse_pad_mipi_crosslink(func, bank),
            ChipKind::Nexus(_) => self.parse_pad_mipi_nexus(func, bank),
            _ => None,
        }
    }

    fn parse_pad_adc(&self, func: &str) -> Option<BondPad> {
        if !matches!(self.chip.kind, ChipKind::Nexus(_)) {
            return None;
        }
        let pad = match func {
            "ADC_DP0" => AdcPad::D(Polarity::P, 0),
            "ADC_DP1" => AdcPad::D(Polarity::P, 1),
            "ADC_DN0" => AdcPad::D(Polarity::N, 0),
            "ADC_DN1" => AdcPad::D(Polarity::N, 1),
            "ADC_REFP0" => AdcPad::RefP(0),
            "ADC_REFP1" => AdcPad::RefP(1),
            "VSSADC" => AdcPad::Vss,
            "VCCADC" => AdcPad::Vcc,
            "VCCADC18" => AdcPad::Vcc18,
            _ => return None,
        };
        Some(BondPad::Adc(pad))
    }

    fn parse_pad_pll_ecp(&self, func: &str) -> Option<BondPad> {
        if func == "VCCPLL" {
            Some(BondPad::VccPll(PllSet::All))
        } else {
            None
        }
    }

    fn parse_pad_pll_xp(&self, func: &str) -> Option<BondPad> {
        let pad = match func {
            "VCCP0" => BondPad::VccPll(PllSet::Side(DirH::W)),
            "VCCP1" => BondPad::VccPll(PllSet::Side(DirH::E)),
            "GNDP0" => BondPad::GndPll(PllSet::Side(DirH::W)),
            "GNDP1" => BondPad::GndPll(PllSet::Side(DirH::E)),
            _ => return None,
        };
        Some(pad)
    }

    fn parse_pad_pll_ecp2(&self, func: &str) -> Option<BondPad> {
        match func {
            "L_VCCPLL" => return Some(BondPad::VccPll(PllSet::Side(DirH::W))),
            "R_VCCPLL" => return Some(BondPad::VccPll(PllSet::Side(DirH::E))),
            "VCCPLL" => return Some(BondPad::VccPll(PllSet::All)),
            _ => (),
        }
        let (loc, pad) = func.split_once('_')?;
        let loc = self.parse_loc_pll_mid(loc)?;
        let pad = match pad {
            "PLLCAP" => BondPad::PllCap(PllSet::Side(loc.quad.h)),
            "VCCPLL" => BondPad::VccPll(PllSet::Quad(loc.quad)),
            _ => return None,
        };
        Some(pad)
    }

    fn parse_pad_pll_xp2(&self, func: &str) -> Option<BondPad> {
        let (loc, pad) = func.split_once('_')?;
        let loc = self.parse_loc_corner(loc)?;
        let pad = match pad {
            "GNDPLL" => BondPad::GndPll(PllSet::Quad(loc)),
            "VCCPLL" => BondPad::VccPll(PllSet::Quad(loc)),
            _ => return None,
        };
        Some(pad)
    }

    fn parse_pad_pll_ecp3(&self, func: &str) -> Option<BondPad> {
        match func {
            "VCCPLL_L" => return Some(BondPad::VccPll(PllSet::Side(DirH::W))),
            "VCCPLL_R" => return Some(BondPad::VccPll(PllSet::Side(DirH::E))),
            _ => (),
        }
        let loc = func.strip_suffix("_VCCPLL")?;
        let loc = self.parse_loc_pll_mid(loc)?;
        Some(BondPad::VccPll(PllSet::Side(loc.quad.h)))
    }

    fn parse_pad_pll_crosslink(&self, func: &str) -> Option<BondPad> {
        let pad = match func {
            "VCCGPLL" => BondPad::VccPll(PllSet::All),
            "VSSGPLL" => BondPad::GndPll(PllSet::All),
            _ => return None,
        };
        Some(pad)
    }

    fn parse_pad_pll_nexus(&self, func: &str) -> Option<BondPad> {
        let pad = match func {
            "VCCPLL_LLC" => BondPad::VccPll(PllSet::Quad(DirHV::SW)),
            "VCCPLL_LRC" => BondPad::VccPll(PllSet::Quad(DirHV::SE)),
            "VCCPLL_ULC" => BondPad::VccPll(PllSet::Quad(DirHV::NW)),
            "VCCPLL_URC" => BondPad::VccPll(PllSet::Quad(DirHV::NE)),
            _ => return None,
        };
        Some(pad)
    }

    fn parse_pad_pll(&self, func: &str) -> Option<BondPad> {
        match self.chip.kind {
            ChipKind::Scm => None,
            ChipKind::Ecp => self.parse_pad_pll_ecp(func),
            ChipKind::Xp => self.parse_pad_pll_xp(func),
            ChipKind::MachXo => None,
            ChipKind::Ecp2 | ChipKind::Ecp2M => self.parse_pad_pll_ecp2(func),
            ChipKind::Xp2 => self.parse_pad_pll_xp2(func),
            ChipKind::Ecp3(_) => self.parse_pad_pll_ecp3(func),
            ChipKind::MachXo2(_) => None,
            ChipKind::Ecp4 => None,
            ChipKind::Ecp5 => None,
            ChipKind::Crosslink => self.parse_pad_pll_crosslink(func),
            ChipKind::Nexus(_) => self.parse_pad_pll_nexus(func),
        }
    }

    fn parse_pad_misc_scm(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else if let Some(bank) = func.strip_prefix("VTT_")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::Vtt(bank))
        } else {
            let pad = match func {
                "RESETN" => BondPad::Cfg(CfgPad::ResetB),
                "RDCFGN" => BondPad::Cfg(CfgPad::RdCfgB),
                "DONE" => BondPad::Cfg(CfgPad::Done),
                "INITN" => BondPad::Cfg(CfgPad::InitB),
                "M0" => BondPad::Cfg(CfgPad::M0),
                "M1" => BondPad::Cfg(CfgPad::M1),
                "M2" => BondPad::Cfg(CfgPad::M2),
                "M3" => BondPad::Cfg(CfgPad::M3),
                "MPIIRQN" => BondPad::Cfg(CfgPad::MpiIrqB),
                "PROGRAMN" => BondPad::Cfg(CfgPad::ProgB),
                "CCLK" => BondPad::Cfg(CfgPad::Cclk),
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "TEMP" => BondPad::TempSense,
                "PROBE_VCC" => BondPad::ProbeVcc,
                "PROBE_GND" => BondPad::ProbeGnd,
                "XRES" => BondPad::XRes,
                "VCCJ" => BondPad::VccJtag,
                // TODO?
                "VCCAUX1" | "VCCAUX2" | "VCCAUX3" | "VCCAUX4" | "VCCAUX5" | "VCCAUX6"
                | "VCCAUX7" | "VCCAUX23" | "VCCAUX67" => BondPad::VccAux,
                "VCCAUX" => BondPad::VccAux,
                "VCC12" => BondPad::Vcc12,
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_ecp(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func
            .strip_prefix("GNDIO")
            .or(func.strip_prefix("GNDO"))
            .or(func.strip_prefix("GND0_"))
            && let Ok(bank) = bank.parse()
        {
            // XXX
            let _: u32 = bank;
            Some(BondPad::Gnd)
        } else if let Some(bank) = func.strip_prefix("VCCIO").or(func.strip_prefix("VCCO"))
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else if matches!(self.chip.kind, ChipKind::Ecp3(_))
            && let Some(bank) = func.strip_prefix("VTT")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::Vtt(bank))
        } else {
            let pad = match func {
                "CFG0" => BondPad::Cfg(CfgPad::M0),
                "CFG1" => BondPad::Cfg(CfgPad::M1),
                "CFG2" => BondPad::Cfg(CfgPad::M2),
                "PROGRAMN" => BondPad::Cfg(CfgPad::ProgB),
                "INITN" => BondPad::Cfg(CfgPad::InitB),
                "DONE" => BondPad::Cfg(CfgPad::Done),
                "CCLK" => BondPad::Cfg(CfgPad::Cclk),
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "SLEEPN/TOE" if self.chip.kind == ChipKind::Xp => BondPad::Cfg(CfgPad::SleepB),
                "HFP" if matches!(self.chip.kind, ChipKind::Ecp2 | ChipKind::Ecp2M) => {
                    BondPad::Cfg(CfgPad::Hfp)
                }
                "XRES" => BondPad::XRes,
                "VCCJ" => BondPad::VccJtag,
                "VCCA" if matches!(self.chip.kind, ChipKind::Ecp3(_)) => BondPad::VccA,
                "TEMPSENSE" if matches!(self.chip.kind, ChipKind::Ecp3(_)) => BondPad::TempSense,
                "TEMPVSS" if matches!(self.chip.kind, ChipKind::Ecp3(_)) => BondPad::TempVss,
                "VCCAUX" => BondPad::VccAux,
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                // hm.
                "GNDIO" | "GNDAUX" if self.chip.kind == ChipKind::Ecp2 => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_ecp2m_extra(&self, func: &str) -> Option<BondPad> {
        let pad = match func {
            "WRITEN" => CfgPad::WriteN,
            "CS1N" => CfgPad::Cs1N,
            "CSN" => CfgPad::CsN,
            "D0" => CfgPad::D(0),
            "D1" => CfgPad::D(1),
            "D2" => CfgPad::D(2),
            "D3" => CfgPad::D(3),
            "D4" => CfgPad::D(4),
            "D5" => CfgPad::D(5),
            "D6" => CfgPad::D(6),
            "D7" => CfgPad::D(7),
            "DI" => CfgPad::Di,
            "DOUT_CSON" => CfgPad::Dout,
            "BUSY" => CfgPad::Busy,
            _ => return None,
        };
        Some(BondPad::Cfg(pad))
    }

    fn parse_pad_misc_machxo(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("GNDIO")
            && let Ok(bank) = bank.parse()
        {
            // XXX
            let _: u32 = bank;
            Some(BondPad::Gnd)
        } else if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "SLEEPN/NC" => {
                    // AAAAAAAAAAAAAAAAAAAAAAAa
                    let io = self.chip.special_io[&SpecialIoKey::SleepN];
                    BondPad::Io(self.chip.get_io_crd(io))
                }
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "VCCAUX" => BondPad::VccAux,
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_xp2(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("GNDIO")
            && let Ok(bank) = bank.parse()
        {
            // XXX
            let _: u32 = bank;
            Some(BondPad::Gnd)
        } else if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "CFG0" => BondPad::Cfg(CfgPad::M0),
                "TOE" => BondPad::Cfg(CfgPad::Toe),
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "VCCJ" => BondPad::VccJtag,
                "VCCAUX" => BondPad::VccAux,
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_machxo2(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_ecp4(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "CFGMDN0" => BondPad::Cfg(CfgPad::M0),
                "CFGMDN1" => BondPad::Cfg(CfgPad::M1),
                "CFGMDN2" => BondPad::Cfg(CfgPad::M2),
                "PROGRAMN" => BondPad::Cfg(CfgPad::ProgB),
                "INITN" => BondPad::Cfg(CfgPad::InitB),
                "DONE" => BondPad::Cfg(CfgPad::Done),
                "CCLK" => BondPad::Cfg(CfgPad::Cclk),
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "VCCJ" => BondPad::VccJtag,
                "VCCA" => BondPad::VccA,
                "GNDA" => BondPad::GndA,
                "VCCAUX" => BondPad::VccAux,
                "VCCAUXA" => BondPad::VccAuxA,
                "VCC" => BondPad::VccInt,
                "GND" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_ecp5(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VSSIO")
            && let Ok(bank) = bank.parse()
        {
            // XXX
            let _: u32 = bank;
            Some(BondPad::Gnd)
        } else if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "CFG_0" => BondPad::Cfg(CfgPad::M0),
                "CFG_1" => BondPad::Cfg(CfgPad::M1),
                "CFG_2" => BondPad::Cfg(CfgPad::M2),
                "PROGRAMN" => BondPad::Cfg(CfgPad::ProgB),
                "INITN" => BondPad::Cfg(CfgPad::InitB),
                "DONE" => BondPad::Cfg(CfgPad::Done),
                "CCLK" => BondPad::Cfg(CfgPad::Cclk),
                "TCK" => BondPad::Cfg(CfgPad::Tck),
                "TMS" => BondPad::Cfg(CfgPad::Tms),
                "TDI" => BondPad::Cfg(CfgPad::Tdi),
                "TDO" => BondPad::Cfg(CfgPad::Tdo),
                "GNDA" => BondPad::GndA,
                "VCCAUX" => BondPad::VccAux,
                "VCC" => BondPad::VccInt,
                "GND" | "VSS" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_crosslink(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else {
            let pad = match func {
                "CDONE" => BondPad::Cfg(CfgPad::Done),
                "CRESET_B" => BondPad::Cfg(CfgPad::ProgB),
                "VCCAUX25VPP" => BondPad::Vpp,
                "VCC" => BondPad::VccInt,
                "VSS" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc_nexus(&self, func: &str) -> Option<BondPad> {
        if let Some(bank) = func.strip_prefix("VSSIO")
            && let Ok(bank) = bank.parse()
        {
            // XXX
            let _: u32 = bank;
            Some(BondPad::Gnd)
        } else if let Some(bank) = func.strip_prefix("VCCIO")
            && let Ok(bank) = bank.parse()
        {
            Some(BondPad::VccIo(bank))
        } else if let Some(vccauxh_bank) = func.strip_prefix("VCCAUXH")
            && let Ok(vccauxh_bank) = vccauxh_bank.parse()
        {
            Some(BondPad::VccAuxH(vccauxh_bank))
        } else {
            let pad = match func {
                "JTAG_EN" => BondPad::Cfg(CfgPad::JtagEn),
                "VCCCDR" => BondPad::VccCdr,
                "VCCECLK" => BondPad::VccEclk,
                "VCCAUX" => BondPad::VccAux,
                "VCCAUXA" => BondPad::VccAuxA,
                "VCC" => BondPad::VccInt,
                "VSS" => BondPad::Gnd,
                _ => return None,
            };
            Some(pad)
        }
    }

    fn parse_pad_misc(&self, func: &str) -> Option<BondPad> {
        match self.chip.kind {
            ChipKind::Scm => self.parse_pad_misc_scm(func),
            ChipKind::Ecp | ChipKind::Xp | ChipKind::Ecp2 | ChipKind::Ecp3(_) => {
                self.parse_pad_misc_ecp(func)
            }
            ChipKind::Ecp2M => self
                .parse_pad_misc_ecp(func)
                .or_else(|| self.parse_pad_misc_ecp2m_extra(func)),
            ChipKind::MachXo => self.parse_pad_misc_machxo(func),
            ChipKind::Xp2 => self.parse_pad_misc_xp2(func),
            ChipKind::MachXo2(_) => self.parse_pad_misc_machxo2(func),
            ChipKind::Ecp4 => self.parse_pad_misc_ecp4(func),
            ChipKind::Ecp5 => self.parse_pad_misc_ecp5(func),
            ChipKind::Crosslink => self.parse_pad_misc_crosslink(func),
            ChipKind::Nexus(_) => self.parse_pad_misc_nexus(func),
        }
    }

    fn parse_pad_asc(&self, func: &str) -> Option<BondPad> {
        if self.bond.kind != BondKind::Asc {
            return None;
        }
        let pad = if let Some(idx) = func.strip_prefix("GPIO")
            && let Ok(idx) = idx.parse()
        {
            AscPad::Gpio(idx)
        } else if let Some(idx) = func.strip_prefix("TRIM")
            && let Ok(idx) = idx.parse()
        {
            AscPad::Trim(idx)
        } else if let Some(idx) = func.strip_prefix("HVOUT")
            && let Ok(idx) = idx.parse()
        {
            AscPad::HvOut(idx)
        } else {
            match func {
                "WDAT" => AscPad::WDat,
                "RDAT" => AscPad::RDat,
                "WRCLK" => AscPad::WrClk,
                "ASCCLK" => AscPad::AscClk,
                "SCL" => AscPad::Scl,
                "SDA" => AscPad::Sda,
                "I2C_ADDR" => AscPad::I2cAddr,
                "RESETb" => AscPad::ResetB,
                "HVIMONP" => AscPad::HviMonP,
                "HIMONN" => AscPad::HiMonN,
                "IMON1P" => AscPad::IMonP(1),
                "IMON1N" => AscPad::IMonN(1),
                "TMON1P" => AscPad::TMonP(1),
                "TMON1N" => AscPad::TMonN(1),
                "TMON2P" => AscPad::TMonP(2),
                "TMON2N" => AscPad::TMonN(2),
                "VMON1" => AscPad::VMon(1),
                "VMON2" => AscPad::VMon(2),
                "VMON3" => AscPad::VMon(3),
                "VMON4" => AscPad::VMon(4),
                "VMON5" => AscPad::VMon(5),
                "VMON6" => AscPad::VMon(6),
                "VMON7" => AscPad::VMon(7),
                "VMON8" => AscPad::VMon(8),
                "VMON9" => AscPad::VMon(9),
                "VMON1GS" => AscPad::VMonGs(1),
                "VMON2GS" => AscPad::VMonGs(2),
                "VMON3GS" => AscPad::VMonGs(3),
                "VMON4GS" => AscPad::VMonGs(4),
                "LDRV" => AscPad::Ldrv,
                "HDRV" => AscPad::Hdrv,
                "VDC" => AscPad::Vdc,
                "VSSA" => AscPad::VssA,
                "VDDA" => AscPad::VddA,
                "VDDD" => AscPad::VddD,
                "VSSD_GND" => return Some(BondPad::Gnd),
                _ => return None,
            }
        };
        Some(BondPad::Asc(pad))
    }

    fn parse_pad_pfr(&self, func: &str) -> Option<BondPad> {
        if self.bond.kind != BondKind::MachNx {
            return None;
        }
        let pad = if let Some(pfr_func) = func.strip_prefix("PFR_")
            && let Some(io) = self.parse_pfr_io(pfr_func)
        {
            PfrPad::Io(io)
        } else if let Some(vccio_bank) = func.strip_prefix("VCCXDIO")
            && let Ok(vccio_bank) = vccio_bank.parse()
        {
            return Some(BondPad::VccIo(vccio_bank));
        } else if let Some(vccio_bank) = func.strip_prefix("VCCNXIO")
            && let Ok(vccio_bank) = vccio_bank.parse()
        {
            PfrPad::VccIo(vccio_bank)
        } else {
            match func {
                "JTAG_EN" => PfrPad::JtagEn,
                "ADC_REFP0" => PfrPad::AdcRefP0,
                "ADC_REFP1" => PfrPad::AdcRefP1,
                "ADC_DP0" => PfrPad::AdcDp0,
                "ADC_DP1" => PfrPad::AdcDp1,
                "VSSADC" => PfrPad::VssAdc,
                "VCCXDNXIO0XDIO4" | "VCCXDXDIO4" => return Some(BondPad::VccInt),
                "VCCXDIO0NXIO0NXIO1NXIO2" => return Some(BondPad::VccIo(0)),
                "VCCNX" => PfrPad::VccInt,
                "VCCNXECLK" => PfrPad::VccEclk,
                "VCCNXAUX" => PfrPad::VccAux,
                "VCCNXAUXA" => PfrPad::VccAuxA,
                "VCCNXAUXH3" => PfrPad::VccAuxH(3),
                "VCCNXAUXH4" => PfrPad::VccAuxH(4),
                "VCCNXAUXH5" => PfrPad::VccAuxH(5),
                "VCCNXADC18" => PfrPad::VccAdc18,
                "VSS" => return Some(BondPad::Gnd),
                _ => return None,
            }
        };
        Some(BondPad::Pfr(pad))
    }

    fn process_pins(&mut self) {
        for pin_info in &self.pkg.pins {
            let Value::String(ref func) = pin_info["FNC"] else {
                unreachable!()
            };
            let Value::String(ref cfg) = pin_info["CFG"] else {
                unreachable!()
            };
            let Value::Int(bank) = pin_info["BANK"] else {
                unreachable!()
            };
            let Value::String(ref pin) = pin_info[&self.part.package] else {
                unreachable!()
            };
            let Value::String(ref bs_type) = pin_info["BS_TYPE"] else {
                unreachable!()
            };
            let Value::Int(bs_order) = pin_info["BS_ORDER"] else {
                unreachable!()
            };
            let mut func = func;
            if func == "UNUSED" && cfg.contains("RXREFCLK") && self.chip.kind == ChipKind::Scm {
                func = cfg;
            }
            if (bs_order == 0 || bs_type == "L")
                && (pin.starts_with("Unused")
                    || pin == "UNUSED"
                    || pin == "GND"
                    || pin == "GNDA"
                    || pin == "DB"
                    || pin.starts_with("VCC")
                    || pin.starts_with("VDD")
                    || pin.starts_with("VTT")
                    || pin.starts_with("VSS"))
            {
                continue;
            }
            if pin.ends_with("PAD") && func.starts_with("DU_") {
                continue;
            }
            if let Some(suffix) = func.strip_prefix("NC_")
                && pin == suffix
            {
                continue;
            }
            if func == "CDONE"
                && let Some(pin) = pin.strip_suffix("_CDONE")
                && let Some(io) = self.parse_io(pin)
            {
                self.cdone_shared = Some(io);
                continue;
            }
            // I hate you. I hate you so fucking much.
            let pin = match pin.as_str() {
                "NXBOOT_MCLK" => "NXBOOTMCLK",
                "NXBOOT_MOSI" => "NXBOOTMOSI",
                "NXBOOT_MISO" => "NXBOOTMISO",
                "NXBOOT_MCSN" => "NXBOOTMCSN",
                "NX_PROGRAMN" => "NXPROGRAMN",
                "NX_JTAGEN" => "NXJTAGEN",
                "FLASH_CSN" => "FLASHCSN",
                _ => pin,
            };
            let pin = pin.strip_suffix("_DB").unwrap_or(pin);
            let pin = pin.to_string();
            if let Some(n) = func.strip_suffix("_DB")
                && n == pin
            {
                continue;
            }
            let pad = if let Some(pads) = self.virt_io.get_mut(func) {
                pads.sort();
                match *pads.as_slice() {
                    [BondPad::Io(io), BondPad::Asc(asc)] => BondPad::IoAsc(io, asc),
                    [BondPad::Io(io), BondPad::Pfr(pfr)] => BondPad::IoPfr(io, pfr),
                    [BondPad::Pfr(pad)] => BondPad::Pfr(pad),
                    [BondPad::Io(pad)] if func == "FLASHCSN" => BondPad::Io(pad),
                    _ => {
                        print!("WEIRD VIRT {func}:");
                        for pad in pads {
                            print!(" {pad}");
                        }
                        println!();
                        continue;
                    }
                }
            } else if let Some(io) = self.parse_io(func) {
                for func in cfg.split('/') {
                    let Some((spec, uncompl)) = self.parse_io_special(&pin, io, bank as u32, func)
                    else {
                        continue;
                    };
                    let spec_io = if uncompl { self.uncompl_io(io) } else { io };
                    // well.
                    if let Some(prev_io) = self.special_io.insert(spec, spec_io) {
                        assert_eq!(prev_io, spec_io, "fail on {spec}");
                    }
                }
                let bcrd = self.chip.get_io_loc(io);
                let io_bank = self.chip.get_io_bank(bcrd.cell);
                assert_eq!(io_bank as i32, bank);
                BondPad::Io(io)
            } else if func.starts_with("J_UNUSED") && self.chip.kind == ChipKind::Xp2 {
                assert!(pin.starts_with("Unused"));
                BondPad::Other
            } else if matches!(func.as_str(), "Unused" | "UNUSED" | "NC") {
                BondPad::Nc
            } else if func == "RESERVE"
                && self.part.name.starts_with("LPTM")
                && self.chip.kind == ChipKind::MachXo
            {
                // TODO LPTM special crap
                BondPad::Other
            } else if let Some(pad) = self
                .parse_pad_serdes(func, bank)
                .or_else(|| self.parse_pad_mipi(func, bank))
                .or_else(|| self.parse_pad_adc(func))
                .or_else(|| self.parse_pad_pll(func))
                .or_else(|| self.parse_pad_misc(func))
                .or_else(|| self.parse_pad_asc(func))
                .or_else(|| self.parse_pad_pfr(func))
            {
                pad
            } else {
                println!("\tUNK SPEC {pin:5} {func:8} {cfg:20} {bank}");
                continue;
            };
            if !pin.starts_with("Unused")
                && pin != "GND"
                && pin != "GNDA"
                && pin != "VCCA"
                && pin != "VSS"
            {
                if pin.starts_with("ICC")
                    || pin.starts_with("NX")
                    || pin == "FLASHCSN"
                    || pin.starts_with("WDAT")
                    || pin.starts_with("RDAT")
                    || pin.starts_with("WRCLK")
                    || pin.starts_with("ASCCLK")
                    || pin.starts_with("SCL")
                    || pin.starts_with("SDA")
                    || pin.starts_with("RESET")
                {
                    self.virt_io.entry(pin).or_default().push(pad);
                } else {
                    self.bond.pins.insert(pin, pad);
                }
            }
            if bs_order != 0 && !matches!(pad, BondPad::Pfr(_)) {
                let idx: usize = (bs_order - 1).try_into().unwrap();
                while idx >= self.bscan.len() {
                    self.bscan.push(None);
                }
                assert_eq!(self.bscan[idx], None);
                self.bscan[idx] = Some((pad, bs_type.clone()));
            }
        }
    }

    fn process_bscan(&mut self) {
        // for (i, x) in self.bscan.iter().enumerate() {
        //     if let &Some((x, kind)) = x {
        //         println!("{i}: {x} {kind}");
        //     } else {
        //         println!("{i}: ----");
        //     }
        // }
        let pkg_bscan = Vec::from_iter(self.bscan.iter().rev().cloned().map(Option::unwrap));
        let bscan = self.chip.get_bscan();
        let mut bit = 0;
        for (mut pad, bs_type) in pkg_bscan {
            let bspad = match bs_type.as_str() {
                "I:1" | "OO:4" => {
                    bit += 1;
                    BScanPad::Input(bit - 1)
                }
                "O2:1" => {
                    bit += 1;
                    BScanPad::Output(bit - 1)
                }
                "BD:7:1" | "BD:7:2" | "BD:7-0:1" => {
                    bit += 2;
                    BScanPad::BiTristate(bit - 1, bit - 2)
                }
                "O3:1:1" => {
                    bit += 2;
                    BScanPad::OutputTristate(bit - 1, bit - 2)
                }
                "BDC:1" => {
                    bit += 1;
                    if matches!(pad, BondPad::Serdes(_, _, SerdesPad::OutP(_))) {
                        BScanPad::Output(bit - 1)
                    } else {
                        BScanPad::Input(bit - 1)
                    }
                }
                _ => panic!("unk BS_TYPE {bs_type}"),
            };
            if matches!(pad, BondPad::Other | BondPad::Nc) {
                continue;
            }
            if pad == BondPad::Nc && self.chip.kind == ChipKind::MachXo {
                // sigh. sigh. sigh.
                let io = self.chip.special_io[&SpecialIoKey::SleepN];
                pad = BondPad::Io(self.chip.get_io_crd(io));
            }
            if self.part.name == "LFE2M70E" {
                // toolchain bug I guess? I don't believe the bscan register actually does this shit
                if let BondPad::Serdes(DirV::S, col, spad) = pad {
                    let ncol = if col == self.chip.col_w() + 1 {
                        self.chip.col_e() - 27
                    } else {
                        self.chip.col_w() + 1
                    };
                    pad = BondPad::Serdes(DirV::S, ncol, spad);
                }
            }
            if bscan.pads.get(&pad) != Some(&bspad) {
                println!(
                    "BSCAN MISMATCH for {pad}: {l:?} != {bspad:?}",
                    l = bscan.pads.get(&pad)
                );
            }
        }
        assert_eq!(bit, bscan.bits);
    }

    fn finish_virt(&mut self) {
        for pads in self.virt_io.values_mut() {
            pads.sort();
            if let [BondPad::Io(io), BondPad::Pfr(pfr)] = *pads.as_slice() {
                self.bond.pfr_io.insert(pfr, io);
            }
        }
    }

    fn finish_cdone(&mut self) {
        if let Some(cdone_shared) = self.cdone_shared {
            for pad in self.bond.pins.values_mut() {
                if *pad == BondPad::Io(cdone_shared) {
                    *pad = BondPad::IoCdone(cdone_shared);
                }
            }
        }
    }
}

pub struct BondResult {
    pub bond: Bond,
    pub special_io: BTreeMap<SpecialIoKey, EdgeIoCoord>,
}

pub fn process_bond(datadirs: &[PathBuf], part: &Part, chip: &Chip, nph: &str) -> BondResult {
    let mut ctx = BondContext {
        datadirs,
        part,
        chip,
        nph,
        pkg: PkgData {
            pkgs: Default::default(),
            pins: Default::default(),
        },
        bond: Bond {
            kind: BondKind::Single,
            pins: Default::default(),
            pfr_io: Default::default(),
        },
        special_io: Default::default(),
        serdes_xlat: Default::default(),
        mipi_xlat: Default::default(),
        pll_xlat: Default::default(),
        pll_rows: Default::default(),
        virt_io: Default::default(),
        bscan: vec![],
        cdone_shared: None,
    };
    ctx.load_pkg();
    ctx.set_kind();
    ctx.fill_xlat();
    ctx.process_pins();
    ctx.process_bscan();
    ctx.finish_virt();
    ctx.finish_cdone();

    let bond = ctx.bond;
    let special_io = ctx.special_io;
    BondResult { bond, special_io }
}
