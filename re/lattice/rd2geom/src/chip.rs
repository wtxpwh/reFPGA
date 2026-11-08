use std::collections::{BTreeMap, HashMap};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, Column, IoGroupKind, Row, RowKind},
    defs::bslots,
};
use prjcombine_entity::{EntityId, EntityRangeVec, EntityVec};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, ColId, DieIdExt, RowId},
    db::IntDb,
    dir::{Dir, DirH, DirV},
};
use prjcombine_re_lattice_naming::WireName;
use prjcombine_re_lattice_rawdump::Grid;
use prjcombine_types::enumerable::EnumMap;

mod crosslink;
mod ecp;
mod ecp2;
mod ecp3;
mod ecp4;
mod ecp5;
mod machxo;
mod machxo2;
mod nexus;
mod scm;

pub trait ChipExt {
    fn xlat_row(&self, r: u16) -> RowId;
    fn xlat_col(&self, c: u16) -> ColId;
    fn xlat_rc(&self, r: u16, c: u16) -> CellCoord;
    fn xlat_rc_wire(&self, wire: WireName) -> CellCoord;
}

impl ChipExt for Chip {
    fn xlat_row(&self, r: u16) -> RowId {
        RowId::from_idx(self.rows.len() - usize::from(r))
    }

    fn xlat_col(&self, c: u16) -> ColId {
        ColId::from_idx((c - 1).into())
    }

    fn xlat_rc(&self, r: u16, c: u16) -> CellCoord {
        let c = if c == 0 {
            1
        } else if c as usize == self.columns.len() + 1 {
            c - 1
        } else {
            c
        };
        let r = if r == 0 {
            1
        } else if r as usize == self.rows.len() + 1 {
            r - 1
        } else {
            r
        };
        let col = self.xlat_col(c);
        let row = self.xlat_row(r);
        Chip::DIE.cell(col, row)
    }

    fn xlat_rc_wire(&self, wire: WireName) -> CellCoord {
        self.xlat_rc(wire.r, wire.c)
    }
}

struct ChipBuilder<'a> {
    name: &'a str,
    chip: Chip,
    grid: &'a Grid,
    intdb: &'a IntDb,
}

impl ChipBuilder<'_> {
    fn fill_ebr_dsp_rows(&mut self) {
        for wn in self.grid.nodes.values() {
            let suffix = self.grid.strings[wn.suffix].as_str();
            if (suffix.ends_with("EBR") || suffix.ends_with("EBR_CORE"))
                && self.chip.kind != ChipKind::MachXo
            {
                let row = self.chip.xlat_row(wn.r);
                if !self.chip.is_row_io(row) {
                    self.chip.rows[row].kind = RowKind::Ebr;
                }
            }
            if suffix.ends_with("MULT9") || suffix.ends_with("MULT9_H0") {
                let row = self.chip.xlat_row(wn.r);
                self.chip.rows[row].kind = RowKind::Dsp;
            }
        }
    }

    fn fill_plc_rows(&mut self) {
        for tile in &self.grid.tiles {
            let kind = match tile.kind.as_str() {
                "PLC" | "PLC2" | "PLC_CR" => RowKind::Plc,
                "FPLC" => RowKind::Fplc,
                _ => continue,
            };
            let (r, _c) = tile
                .name
                .strip_prefix('R')
                .unwrap()
                .split_once('C')
                .unwrap();
            let row = self.chip.xlat_row(r.parse().unwrap());
            self.chip.rows[row].kind = kind;
        }
    }

    fn fill_io(&mut self, tiles: &[(&str, IoGroupKind)]) {
        let tiles = BTreeMap::from_iter(tiles.iter().copied());
        for tile in &self.grid.tiles {
            let Some(&kind) = tiles.get(&tile.kind.as_str()) else {
                continue;
            };
            if let Some(r) = tile.name.strip_prefix("PL") {
                let row = self.chip.xlat_row(r.parse().unwrap());
                self.chip.rows[row].io[DirH::W] = kind;
            } else if let Some(r) = tile.name.strip_prefix("PR") {
                let row = self.chip.xlat_row(r.parse().unwrap());
                if tile.kind == "PIC_L_NOPIO" {
                    // ...... fucking vendors
                    self.chip.rows[row].io[DirH::W] = kind;
                } else {
                    self.chip.rows[row].io[DirH::E] = kind;
                }
            } else if let Some(c) = tile.name.strip_prefix("PB") {
                let col = self.chip.xlat_col(c.parse().unwrap());
                self.chip.columns[col].io[DirV::S] = kind;
            } else if let Some(c) = tile.name.strip_prefix("PT") {
                let col = self.chip.xlat_col(c.parse().unwrap());
                if tile.kind == "PIC_B5KVIQ" {
                    // likewise.
                    self.chip.columns[col].io[DirV::S] = kind;
                } else {
                    self.chip.columns[col].io[DirV::N] = kind;
                }
            } else if let Some(rc) = tile
                .name
                .strip_prefix("EBR_R")
                .or_else(|| tile.name.strip_prefix("CIB_R"))
                .or_else(|| tile.name.strip_prefix("MIB_R"))
                .or_else(|| tile.name.strip_prefix("DUMMYEND_R"))
            {
                let (r, c) = rc.split_once('C').unwrap();
                let cell = self.chip.xlat_rc(r.parse().unwrap(), c.parse().unwrap());
                if self.chip.kind == ChipKind::Scm && kind == IoGroupKind::Serdes {
                    self.chip.columns[cell.col].io[DirV::N] = kind;
                } else {
                    match self.chip.get_io_edge(cell) {
                        Dir::W => self.chip.rows[cell.row].io[DirH::W] = kind,
                        Dir::E => self.chip.rows[cell.row].io[DirH::E] = kind,
                        Dir::S => self.chip.columns[cell.col].io[DirV::S] = kind,
                        Dir::N => self.chip.columns[cell.col].io[DirV::N] = kind,
                    }
                }
            } else {
                panic!("umm weird IO tile {}", tile.name);
            }
        }
    }

    fn fill_io_banks_8(&mut self) {
        for (row, rd) in &mut self.chip.rows {
            if row < self.chip.row_clk {
                if rd.io[DirH::W] != IoGroupKind::None {
                    rd.bank.insert(DirH::W, 6);
                }
                if rd.io[DirH::E] != IoGroupKind::None {
                    rd.bank.insert(DirH::E, 3);
                }
            } else {
                if rd.io[DirH::W] != IoGroupKind::None {
                    rd.bank.insert(DirH::W, 7);
                }
                if rd.io[DirH::E] != IoGroupKind::None {
                    rd.bank.insert(DirH::E, 2);
                }
            }
        }
        for (col, cd) in &mut self.chip.columns {
            if col < self.chip.col_clk {
                if cd.io[DirV::S] != IoGroupKind::None {
                    cd.bank.insert(DirV::S, 5);
                }
                if cd.io[DirV::N] != IoGroupKind::None {
                    cd.bank.insert(DirV::N, 0);
                }
            } else {
                if cd.io[DirV::S] != IoGroupKind::None {
                    cd.bank.insert(DirV::S, 4);
                }
                if cd.io[DirV::N] != IoGroupKind::None {
                    cd.bank.insert(DirV::N, 1);
                }
            }
        }
    }

    fn gather_special_io(&mut self) -> BTreeMap<WireName, BelCoord> {
        let jpaddia_pio = self.grid.strings.get("JPADDIA_PIO");
        let jpaddib_pio = self.grid.strings.get("JPADDIB_PIO");
        let jpaddic_pio = self.grid.strings.get("JPADDIC_PIO");
        let jpaddid_pio = self.grid.strings.get("JPADDID_PIO");
        let jpaddie_pio = self.grid.strings.get("JPADDIE_PIO");
        let jpaddif_pio = self.grid.strings.get("JPADDIF_PIO");
        let jpaddiea_pio = self.grid.strings.get("JPADDIEA_PIO");
        let jpaddieb_pio = self.grid.strings.get("JPADDIEB_PIO");
        let jpaddiec_pio = self.grid.strings.get("JPADDIEC_PIO");
        let jpaddied_pio = self.grid.strings.get("JPADDIED_PIO");
        let mut pad_nodes = HashMap::new();
        for (node, &wn) in &self.grid.nodes {
            let mut cell = self.chip.xlat_rc_wire(wn);
            let (mut idx, e) = if Some(wn.suffix) == jpaddia_pio {
                (0, false)
            } else if Some(wn.suffix) == jpaddib_pio {
                (1, false)
            } else if Some(wn.suffix) == jpaddic_pio {
                (2, false)
            } else if Some(wn.suffix) == jpaddid_pio {
                (3, false)
            } else if Some(wn.suffix) == jpaddie_pio {
                (4, false)
            } else if Some(wn.suffix) == jpaddif_pio {
                (5, false)
            } else if Some(wn.suffix) == jpaddiea_pio {
                (0, true)
            } else if Some(wn.suffix) == jpaddieb_pio {
                (1, true)
            } else if Some(wn.suffix) == jpaddiec_pio {
                (2, true)
            } else if Some(wn.suffix) == jpaddied_pio {
                (3, true)
            } else {
                continue;
            };
            if matches!(self.chip.kind, ChipKind::Ecp3(_)) {
                if e {
                    if cell.col == self.chip.col_w() {
                        cell.col += 1;
                    } else if cell.col == self.chip.col_e() {
                        cell.col -= 1;
                    } else {
                        unreachable!();
                    }
                } else if self.chip.is_col_io(cell.col) {
                    let kind = if cell.col == self.chip.col_w() {
                        self.chip.rows[cell.row].io[DirH::W]
                    } else {
                        self.chip.rows[cell.row].io[DirH::E]
                    };
                    if kind == IoGroupKind::None {
                        cell.row -= 2;
                    } else {
                        idx += 2;
                    }
                } else {
                    let kind = if cell.row == self.chip.row_s() {
                        self.chip.columns[cell.col].io[DirV::S]
                    } else {
                        self.chip.columns[cell.col].io[DirV::N]
                    };
                    if kind == IoGroupKind::None {
                        idx += 2;
                        cell.col -= 2;
                    }
                }
            } else {
                assert!(!e);
            }
            pad_nodes.insert(node, cell.bel(bslots::IO[idx]));
        }
        let mut result = BTreeMap::new();
        for &(wf, wt) in self.grid.pips.keys() {
            let Some(&io) = pad_nodes.get(&wf) else {
                continue;
            };
            let wn = self.grid.nodes[wt];
            let suffix = self.grid.strings[wn.suffix].as_str();
            if matches!(
                suffix,
                "JPADDIA"
                    | "JPADDIB"
                    | "DIA_IOLOGIC"
                    | "DIB_IOLOGIC"
                    | "DIA_SIOLOGIC"
                    | "DIB_SIOLOGIC"
                    | "DIA_XSIOLOGIC"
                    | "DIB_XSIOLOGIC"
                    | "DIEA_XSIOLOGIC"
                    | "DIEB_XSIOLOGIC"
                    | "DIEC_XSIOLOGIC"
                    | "DIED_XSIOLOGIC"
                    | "DIA_DQSIOL"
                    | "DIA_SDQSIOL"
                    | "JDQSI_DQS"
                    | "JDQSI_SDQS"
                    | "JF0"
                    | "JF1"
                    | "JF2"
                    | "JF3"
                    | "JF4"
                    | "JF5"
                    | "JF6"
                    | "JF7"
                    | "JQ0"
                    | "JQ1"
                    | "JOFX6"
                    | "JOFX7"
            ) {
                continue;
            }
            result.insert(wn, io);
        }
        result
    }
}

fn init_chip(kind: ChipKind, grid: &Grid) -> Chip {
    let ja0 = grid.strings.get("JA0").unwrap();
    let mut max_r = 0;
    let mut max_c = 0;
    for wn in grid.nodes.values() {
        if wn.suffix == ja0 {
            max_r = max_r.max(wn.r);
            max_c = max_c.max(wn.c);
        }
    }
    let columns = EntityVec::from_iter((0..max_c).map(|_| Column {
        io: EnumMap::from_fn(|_| IoGroupKind::None),
        bank: Default::default(),
        eclk_tap: EnumMap::from_fn(|_| false),
        sclk_break: false,
    }));
    let rows = EntityVec::from_iter((0..max_r).map(|_| Row {
        kind: RowKind::Io,
        io: EnumMap::from_fn(|_| IoGroupKind::None),
        bank: Default::default(),
        sclk_break: false,
    }));
    Chip {
        kind,
        columns,
        rows,
        col_clk: ColId::from_idx(0),
        row_clk: RowId::from_idx(0),
        cols_pclk: EntityRangeVec::new(),
        rows_pclk: EntityRangeVec::new(),
        special_loc: BTreeMap::new(),
        special_io: BTreeMap::new(),
        io_direct_plc: BTreeMap::new(),
        extra_frames: EnumMap::from_fn(|_| 0),
        double_frames: false,
    }
}

pub fn make_chip(name: &str, grid: &Grid, kind: ChipKind, intdb: &IntDb) -> Chip {
    let chip = init_chip(kind, grid);
    let mut builder = ChipBuilder {
        name,
        chip,
        grid,
        intdb,
    };
    builder.fill_ebr_dsp_rows();
    builder.fill_plc_rows();
    match builder.chip.kind {
        ChipKind::Scm => {
            builder.fill_clk_scm();
            builder.fill_pclk_scm();
            builder.fill_config_loc_scm();
            builder.fill_pll_scm();
            builder.fill_io_scm();
            builder.fill_special_io_scm();
        }
        ChipKind::Ecp => {
            builder.fill_clk_ecp();
            builder.fill_config_loc_ecp();
            builder.fill_pll_ecp();
            builder.fill_io_ecp();
            builder.fill_io_banks_8();
            builder.fill_special_io_ecp();
            builder.fill_fabric_clock_ecp();
            builder.fill_config_io_ecp();
        }
        ChipKind::Xp => {
            builder.fill_clk_ecp();
            builder.fill_config_loc_ecp();
            builder.fill_config_bits_loc_xp();
            builder.fill_frames_xp();
            builder.fill_pll_ecp();
            builder.fill_io_xp();
            builder.fill_io_banks_8();
            builder.fill_special_io_ecp();
            builder.fill_fabric_clock_ecp();
            builder.fill_config_io_xp();
        }
        ChipKind::MachXo => {
            builder.fill_clk_machxo();
            builder.fill_special_loc_machxo();
            builder.fill_io_machxo();
            builder.fill_io_banks_machxo();
            builder.fill_special_io_machxo();
            builder.fill_direct_io_machxo();
        }
        ChipKind::Ecp2 | ChipKind::Ecp2M => {
            builder.fill_clk_ecp();
            builder.fill_pclk_ecp2();
            builder.fill_sclk_ecp2();
            builder.fill_eclk_tap_ecp2();
            builder.fill_config_loc_ecp();
            builder.fill_io_ecp2();
            builder.fill_io_banks_ecp2();
            if builder.chip.kind == ChipKind::Ecp2 {
                builder.fill_pll_ecp2();
            } else {
                builder.fill_pll_ecp2m();
                builder.fill_serdes_ecp2m();
            }
            builder.fill_special_io_ecp2();
            builder.fill_fabric_clock_ecp2();
            builder.fill_config_io_ecp2();
        }
        ChipKind::Xp2 => {
            builder.fill_clk_ecp();
            builder.fill_pclk_ecp2();
            builder.fill_sclk_ecp2();
            builder.fill_eclk_tap_ecp2();
            builder.fill_config_loc_xp2();
            builder.fill_io_xp2();
            builder.fill_io_banks_8();
            builder.fill_pll_xp2();
            builder.fill_special_io_ecp2();
            builder.fill_fabric_clock_ecp2();
            builder.fill_config_io_xp2();
        }
        ChipKind::Ecp3(_) => {
            builder.fill_kind_ecp3();
            builder.fill_clk_ecp();
            builder.fill_pclk_ecp3();
            builder.fill_sclk_ecp3();
            builder.fill_eclk_tap_ecp3();
            builder.fill_config_loc_ecp3();
            builder.fill_io_ecp3();
            builder.fill_io_banks_ecp3();
            builder.fill_serdes_ecp3();
            builder.fill_pll_ecp3();
            builder.fill_special_io_ecp3();
            builder.fill_fabric_clock_ecp3();
            builder.fill_config_io_ecp3();
        }
        ChipKind::MachXo2(_) => {
            builder.fill_kind_machxo2();
            builder.fill_clk_machxo2();
            builder.fill_pclk_ecp3();
            builder.fill_sclk_ecp3();
            builder.fill_ebr_machxo2();
            builder.fill_config_loc_ecp();
            builder.fill_io_machxo2();
            builder.fill_io_banks_machxo2();
            builder.fill_bc_machxo2();
            builder.fill_dqsdll_machxo2();
            builder.fill_pll_xp2();
            builder.fill_special_io_machxo2();
        }
        ChipKind::Ecp4 => {
            builder.fill_clk_ecp4();
            builder.fill_pclk_ecp4();
            builder.fill_config_loc_ecp();
            builder.fill_io_ecp4();
            builder.fill_dqs_ecp4();
            builder.fill_bc_ecp4();
            builder.fill_serdes_ecp4();
            builder.fill_special_io_ecp4();
            builder.fill_fabric_clock_ecp4();
        }
        ChipKind::Ecp5 => {
            builder.fill_clk_ecp4();
            builder.fill_pclk_ecp5();
            builder.fill_config_loc_ecp();
            builder.fill_io_ecp5();
            builder.fill_bc_ecp5();
            builder.fill_ddrdll_ecp5();
            builder.fill_serdes_ecp5();
            builder.fill_pll_xp2();
            builder.fill_special_io_ecp5();
            builder.fill_fabric_clock_ecp5();
        }
        ChipKind::Crosslink => {
            builder.fill_clk_crosslink();
            builder.fill_pclk_ecp5();
            builder.fill_special_loc_crosslink();
            builder.fill_io_crosslink();
            builder.fill_bc_crosslink();
            builder.fill_mipi_crosslink();
            builder.fill_special_io_crosslink();
        }
        ChipKind::Nexus(_) => {
            builder.fill_clk_nexus();
            builder.fill_kind_nexus();
            builder.fill_pclk_nexus();
            builder.fill_io_nexus();
            builder.fill_special_loc_nexus();
            builder.fill_special_io_nexus();
        }
    };
    // builder
    // .chip
    // .dump(&mut std::io::stdout().lock(), builder.intdb);
    builder.chip
}
