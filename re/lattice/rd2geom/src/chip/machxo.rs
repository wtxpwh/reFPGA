use std::collections::{BTreeMap, BTreeSet};

use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, PllLoc, PllPad, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{ColId, ColIdExt, DieIdExt, RowId, RowIdExt},
    dir::{Dir, DirH, DirHV, DirV},
};

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_clk_machxo(&mut self) {
        for ((y, x), tile) in self.grid.tiles.indexed_iter() {
            if tile.kind.starts_with("CLK3") {
                self.chip.col_clk = ColId::from_idx(x);
                self.chip.row_clk = RowId::from_idx(self.chip.rows.len() - 1 - y);
                return;
            }
        }
        unreachable!()
    }

    pub(super) fn fill_special_loc_machxo(&mut self) {
        let mut ebr_locs = BTreeSet::new();
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix].ends_with("_EBR") {
                let cell = self.chip.xlat_rc_wire(wn).delta(0, -3);
                ebr_locs.insert(cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_PLL") {
                let cell = self.chip.xlat_rc_wire(wn);
                let loc = SpecialLocKey::Pll(PllLoc::new(
                    cell.row.side_of(self.chip.row_clk) & DirH::W,
                    0,
                ));
                self.chip.special_loc.insert(loc, cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_OSC") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Osc, cell);
            }
            if self.grid.strings[wn.suffix].ends_with("_JTAG") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Config, cell);
            }
        }
        for (i, cell) in ebr_locs.into_iter().enumerate() {
            self.chip
                .special_loc
                .insert(SpecialLocKey::Ebr(i.try_into().unwrap()), cell);
        }
    }

    pub(super) fn fill_io_machxo(&mut self) {
        self.fill_io(&[
            ("PIC_L", IoGroupKind::Quad),
            ("PIC2_L", IoGroupKind::Double),
            ("PIC4_L", IoGroupKind::Quad),
            ("PIC_L_GSR", IoGroupKind::Quad),
            ("PIC_L_OSC", IoGroupKind::Quad),
            ("PIC_L_ISP", IoGroupKind::Quad),
            ("PIC2_L_GSR", IoGroupKind::Double),
            ("PIC2_L_OSC", IoGroupKind::Double),
            ("PIC2_L_ISP", IoGroupKind::Double),
            ("PIC2_L_EBR1K_0", IoGroupKind::Double),
            ("PIC4_L_EBR1K_1", IoGroupKind::Quad),
            ("PIC4_L_EBR1K_2", IoGroupKind::Quad),
            ("PIC4_L_EBR1K_3", IoGroupKind::Quad),
            ("PIC4_L_EBR1K_4", IoGroupKind::Quad),
            ("PIC4_L_EBR1K_5", IoGroupKind::Quad),
            ("PIC4_L_EBR1K_6", IoGroupKind::Quad),
            ("PIC2_L_EBR2K_1", IoGroupKind::Double),
            ("PIC2_L_EBR2K_2", IoGroupKind::Double),
            ("PIC2_L_EBR2K_3", IoGroupKind::Double),
            ("PIC4_L_EBR2K_4", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_5", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_6", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_7", IoGroupKind::QuadReverse),
            ("PIC4_L_EBR2K_8", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_9", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_10", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_11", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_12", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_13", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_14", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_15", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_16", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_17", IoGroupKind::Quad),
            ("PIC4_L_EBR2K_18", IoGroupKind::Quad),
            ("PIC2_L_EBR2K_19", IoGroupKind::Double),
            ("PIC2_L_PLL1K", IoGroupKind::Double),
            ("PIC_R", IoGroupKind::Quad),
            ("PIC2_R", IoGroupKind::Double),
            ("PIC2_R_LVDS", IoGroupKind::Double),
            ("PIC4_R", IoGroupKind::Quad),
            ("PIC4_B", IoGroupKind::Quad),
            ("PIC6_B", IoGroupKind::Hex),
            ("PIC4_T", IoGroupKind::Quad),
            ("PIC6_T", IoGroupKind::Hex),
        ]);
        if self.chip.rows.len() == 21 {
            self.chip.columns[ColId::from_idx(3)].io[DirV::N] = IoGroupKind::HexReverse;
            self.chip.columns[ColId::from_idx(5)].io[DirV::N] = IoGroupKind::HexReverse;
            self.chip.columns[ColId::from_idx(9)].io[DirV::S] = IoGroupKind::HexReverse;
        }
    }

    pub(super) fn fill_io_banks_machxo(&mut self) {
        let num_rows = self.chip.rows.len();
        for (row, rd) in &mut self.chip.rows {
            if rd.io[DirH::W] == IoGroupKind::None {
                continue;
            }
            let (bank_w, bank_e) = match num_rows {
                10 => (1, 0),
                12 => (3, 1),
                17 => {
                    if row < self.chip.row_clk {
                        (6, 3)
                    } else {
                        (7, 2)
                    }
                }
                21 => {
                    if row < self.chip.row_clk + 1 {
                        (6, 3)
                    } else {
                        (7, 2)
                    }
                }
                _ => unreachable!(),
            };
            rd.bank.insert(DirH::W, bank_w);
            rd.bank.insert(DirH::E, bank_e);
        }
        for (col, cd) in &mut self.chip.columns {
            if cd.io[DirV::S] == IoGroupKind::None {
                continue;
            }
            let (bank_s, bank_n) = match num_rows {
                10 => (1, 0),
                12 => (2, 0),
                17 | 21 => {
                    if col < self.chip.col_clk {
                        (5, 0)
                    } else {
                        (4, 1)
                    }
                }
                _ => unreachable!(),
            };
            cd.bank.insert(DirV::S, bank_s);
            cd.bank.insert(DirV::N, bank_n);
        }
    }

    pub(super) fn fill_special_io_machxo(&mut self) {
        for (wn, io) in self.gather_special_io() {
            let suffix = self.grid.strings[wn.suffix].as_str();
            if let Some(key) = match suffix {
                "JGCLK0" => Some(SpecialIoKey::Clock(Dir::N, 0)),
                "JGCLK1" => Some(SpecialIoKey::Clock(Dir::N, 1)),
                "JGCLK2" => Some(SpecialIoKey::Clock(Dir::S, 0)),
                "JGCLK3" => Some(SpecialIoKey::Clock(Dir::S, 1)),
                "JTSALLI_TSALL" => Some(SpecialIoKey::TsAll),
                "JGSRPADN_GSR" => Some(SpecialIoKey::Gsr),
                _ => None,
            } {
                self.chip.special_io.insert(key, io);
            } else if let Some(pad) = match suffix {
                "JCLKI3" => Some(PllPad::PllIn0),
                "JCLKFB3" => Some(PllPad::PllFb),
                _ => None,
            } {
                let cell = self.chip.xlat_rc_wire(wn);
                let h = cell.col.side_of(self.chip.col_clk);
                let v = cell.row.side_of(self.chip.row_clk);
                let hv = DirHV { h, v };
                let key = SpecialIoKey::Pll(pad, PllLoc::new(hv, 0));
                self.chip.special_io.insert(key, io);
            } else {
                panic!(
                    "WEIRD SPECIO: R{r}C{c}_{suffix} {io}",
                    r = wn.r,
                    c = wn.c,
                    io = io.to_string(self.intdb)
                );
            }
        }
        let (col, iob) = match self.chip.rows.len() {
            10 => (4, 1),
            12 => (8, 4),
            17 => (9, 4),
            21 => (14, 2),
            _ => unreachable!(),
        };
        let col = ColId::from_idx(col);
        let io = Chip::DIE.cell(col, self.chip.row_s()).bel(bslots::IO[iob]);
        self.chip.special_io.insert(SpecialIoKey::SleepN, io);
    }

    pub(super) fn fill_direct_io_machxo(&mut self) {
        let xlat = BTreeMap::from_iter(
            ['A', 'B', 'C', 'D', 'E', 'F']
                .into_iter()
                .enumerate()
                .map(|(i, l)| (self.grid.strings.get(&format!("JDD2{l}")).unwrap(), i)),
        );
        for &(wf, wt) in self.grid.pips.keys() {
            let wtn = self.grid.nodes[wt];
            if let Some(&iob) = xlat.get(&wtn.suffix) {
                let io_cell = self.chip.xlat_rc_wire(wtn);
                let io = io_cell.bel(bslots::IO[iob]);
                let wfn = self.grid.nodes[wf];
                let src_cell = self.chip.xlat_rc_wire(wfn);
                let lut = self.grid.strings[wfn.suffix]
                    .strip_prefix("JDD")
                    .unwrap()
                    .parse()
                    .unwrap();
                self.chip.io_direct_plc.insert(io, (src_cell, lut));
            }
        }
    }
}
