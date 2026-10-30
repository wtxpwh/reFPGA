use std::collections::HashMap;

use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, PllLoc, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_interconnect::{
    dir::{Dir, DirH, DirHV, DirV},
    grid::{ColIdExt, DieIdExt},
};
use prjcombine_types::enumerable::EnumMap;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_clk_ecp(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "LLDCSOUT0_DCS" {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_config_loc_ecp(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix].ends_with("_START") {
                let cell = self.chip.xlat_rc_wire(wn);
                self.chip.special_loc.insert(SpecialLocKey::Config, cell);
            }
        }
    }

    pub(super) fn fill_config_bits_loc_xp(&mut self) {
        for tile in &self.grid.tiles {
            match tile.kind.as_str() {
                "PIC_R_3K_CONFIG" => {
                    let row = self
                        .chip
                        .xlat_row(tile.name.strip_prefix("PR").unwrap().parse().unwrap());
                    let cell = Chip::DIE.cell(self.chip.col_e(), row);
                    self.chip
                        .special_loc
                        .insert(SpecialLocKey::ConfigBits, cell);
                }
                "PIC_L_6K_CONFIG" => {
                    let row = self
                        .chip
                        .xlat_row(tile.name.strip_prefix("PL").unwrap().parse().unwrap());
                    let cell = Chip::DIE.cell(self.chip.col_w(), row);
                    self.chip
                        .special_loc
                        .insert(SpecialLocKey::ConfigBits, cell);
                }
                _ => (),
            }
        }
    }

    pub(super) fn fill_frames_xp(&mut self) {
        match self.chip.rows.len() {
            19 => {
                self.chip.extra_frames[DirH::W] = 3;
                self.chip.extra_frames[DirH::E] = 3;
            }
            27 => {
                self.chip.extra_frames[DirH::W] = 3;
                self.chip.extra_frames[DirH::E] = 11;
            }
            36 => {
                self.chip.extra_frames[DirH::W] = 3;
                self.chip.extra_frames[DirH::E] = 3;
                self.chip.double_frames = true;
            }
            44 => {
                self.chip.extra_frames[DirH::W] = 5;
                self.chip.extra_frames[DirH::E] = 5;
                self.chip.double_frames = true;
            }
            48 => {
                self.chip.extra_frames[DirH::W] = 1;
                self.chip.extra_frames[DirH::E] = 1;
                self.chip.double_frames = true;
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn fill_pll_ecp(&mut self) {
        for edge in [DirH::W, DirH::E] {
            for (row, rd) in &self.chip.rows {
                if rd.kind != RowKind::Ebr {
                    continue;
                }
                let sn = match (row.cmp(&self.chip.row_clk), self.chip.kind) {
                    (std::cmp::Ordering::Less, _) => DirV::S,
                    (std::cmp::Ordering::Equal, ChipKind::Xp) => DirV::N,
                    (std::cmp::Ordering::Equal, ChipKind::Ecp) => DirV::S,
                    (std::cmp::Ordering::Greater, _) => DirV::N,
                    _ => unreachable!(),
                };
                self.chip.special_loc.insert(
                    SpecialLocKey::Pll(PllLoc::new(edge & sn, 0)),
                    Chip::DIE.cell(self.chip.col_edge(edge), row),
                );
            }
        }
    }

    pub(super) fn fill_io_ecp(&mut self) {
        self.fill_io(&[
            ("PIC_L", IoGroupKind::Double),
            ("PIC_LDQS", IoGroupKind::DoubleDqs),
            ("PIC_R", IoGroupKind::Double),
            ("PIC_RDQS", IoGroupKind::DoubleDqs),
            ("PIC_RA", IoGroupKind::Double),
            ("PIC_RB", IoGroupKind::Double),
            ("PIC_T", IoGroupKind::Double),
            ("PIC_TDQS", IoGroupKind::DoubleDqs),
            ("PIC_B", IoGroupKind::Double),
            ("PIC_BDQS", IoGroupKind::DoubleDqs),
            ("PIC_BAB1", IoGroupKind::Double),
            ("PIC_BAB2", IoGroupKind::Double),
            ("PIC_BB1", IoGroupKind::Double),
            ("PIC_BB2", IoGroupKind::Double),
            ("PIC_BB3", IoGroupKind::Double),
            ("PIC_BDQSB", IoGroupKind::DoubleDqs),
        ]);
    }

    pub(super) fn fill_io_xp(&mut self) {
        self.fill_io(&[
            ("PIC_L", IoGroupKind::Double),
            ("PIC_L_6K_CONFIG", IoGroupKind::DoubleA),
            ("PIC_L_A", IoGroupKind::DoubleA),
            ("PIC_L_A_20K", IoGroupKind::DoubleA),
            ("PIC_L_B", IoGroupKind::DoubleB),
            ("PIC_L_B_20K", IoGroupKind::DoubleB),
            ("PIC_LDQS", IoGroupKind::DoubleDqs),
            ("PIC_R", IoGroupKind::Double),
            ("PIC_R_3K_CONFIG", IoGroupKind::Double),
            ("PIC_R_A", IoGroupKind::DoubleA),
            ("PIC_R_A_20K", IoGroupKind::DoubleA),
            ("PIC_R_B", IoGroupKind::DoubleB),
            ("PIC_R_B_20K", IoGroupKind::DoubleB),
            ("PIC_RDQS", IoGroupKind::DoubleDqs),
            ("PIC_B_NO_IO", IoGroupKind::None),
            ("PIC_BL", IoGroupKind::Double),
            ("PIC_BL_A", IoGroupKind::DoubleA),
            ("PIC_BL_B", IoGroupKind::DoubleB),
            ("PIC_BLDQS", IoGroupKind::DoubleDqs),
            ("PIC_BR", IoGroupKind::Double),
            ("PIC_BR_A", IoGroupKind::DoubleA),
            ("PIC_BR_B", IoGroupKind::DoubleB),
            ("PIC_BRDQS", IoGroupKind::DoubleDqs),
            ("PIC_T_NO_IO", IoGroupKind::None),
            ("PIC_TL", IoGroupKind::Double),
            ("PIC_TL_A", IoGroupKind::DoubleA),
            ("PIC_TL_A_CFG", IoGroupKind::Double),
            ("PIC_TL_AB_CFG", IoGroupKind::Double),
            ("PIC_TL_A_ONLY_CFG", IoGroupKind::DoubleA),
            ("PIC_TL_B", IoGroupKind::DoubleB),
            ("PIC_TLDQS", IoGroupKind::DoubleDqs),
            ("PIC_TR", IoGroupKind::Double),
            ("PIC_TR_A", IoGroupKind::DoubleA),
            ("PIC_TR_A_CFG", IoGroupKind::Double),
            ("PIC_TR_AB_CFG", IoGroupKind::Double),
            ("PIC_TR_A_ONLY_CFG", IoGroupKind::DoubleA),
            ("PIC_TR_B", IoGroupKind::DoubleB),
            ("PIC_TR_B_CFG", IoGroupKind::Double),
            ("PIC_TRDQS", IoGroupKind::DoubleDqs),
        ]);
        if self.chip.rows.len() == 48 {
            let col_w1 = self.chip.col_w() + 1;
            let col_e1 = self.chip.col_e() - 1;
            self.chip.columns[col_w1].io = EnumMap::from_fn(|_| IoGroupKind::None);
            self.chip.columns[col_e1].io = EnumMap::from_fn(|_| IoGroupKind::None);
        }
    }

    pub(super) fn fill_special_io_ecp(&mut self) {
        for (wn, io) in self.gather_special_io() {
            let suffix = self.grid.strings[wn.suffix].as_str();
            if let Some(edge) = match suffix {
                "JLPIO" => Some(Dir::W),
                "JRPIO" => Some(Dir::E),
                "JBPIO" => Some(Dir::S),
                "JTPIO" => Some(Dir::N),
                _ => None,
            } {
                self.chip
                    .special_io
                    .insert(SpecialIoKey::Clock(edge, 0), io);
            } else if let Some(pad) = match suffix {
                "JCLKI3" => Some(PllPad::PllIn0),
                "JCLKFB3" => Some(PllPad::PllFb),
                _ => None,
            } {
                let cell = self.chip.xlat_rc_wire(wn);
                let h = cell.col.side_of(self.chip.col_clk);
                let v = if cell.row < self.chip.row_clk
                    || (cell.row == self.chip.row_clk && self.chip.kind == ChipKind::Ecp)
                {
                    DirV::S
                } else {
                    DirV::N
                };
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
    }

    pub(super) fn fill_fabric_clock_ecp(&mut self) {
        let mut xlat = HashMap::new();
        for (name, key) in [
            ("JCIBLLQ", SpecialLocKey::PclkIn(Dir::W, 0)),
            ("JCIBURQ", SpecialLocKey::PclkIn(Dir::E, 0)),
            ("JCIBLRQ", SpecialLocKey::PclkIn(Dir::S, 0)),
            ("JCIBULQ", SpecialLocKey::PclkIn(Dir::N, 0)),
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

    pub(super) fn fill_config_io_ecp(&mut self) {
        for (key, dx, iob) in [
            (SpecialIoKey::WriteN, 0, 0),
            (SpecialIoKey::Cs1N, 0, 1),
            (SpecialIoKey::CsN, 1, 1),
            (SpecialIoKey::D(0), 2, 1),
            (SpecialIoKey::D(2), 3, 0),
            (SpecialIoKey::D(1), 3, 1),
            (SpecialIoKey::D(3), 4, 1),
            (SpecialIoKey::D(4), 5, 1),
            (SpecialIoKey::D(5), 6, 1),
            (SpecialIoKey::D(6), 7, 1),
        ] {
            let io = self
                .chip
                .clk_emid(Dir::S)
                .step(Dir::E, dx)
                .bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
        let row_cfg = self
            .chip
            .rows
            .iter()
            .find(|(_, rd)| rd.kind == RowKind::Ebr)
            .unwrap()
            .0;
        for (key, dy, iob) in [
            (SpecialIoKey::Di, -2, 1),
            (SpecialIoKey::Dout, -2, 0),
            (SpecialIoKey::Busy, -1, 1),
            (SpecialIoKey::D(7), -1, 0),
        ] {
            let io = Chip::DIE
                .cell(self.chip.col_e(), row_cfg + dy)
                .bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
    }

    pub(super) fn fill_config_io_xp(&mut self) {
        for (key, dx, iob) in [
            (SpecialIoKey::CsN, -9, 0),
            (SpecialIoKey::Di, -8, 0),
            (SpecialIoKey::WriteN, -6, 0),
            (SpecialIoKey::Dout, -5, 0),
            (SpecialIoKey::Cs1N, -1, 0),
            (SpecialIoKey::Busy, -1, 1),
            (SpecialIoKey::D(7), 0, 1),
            (SpecialIoKey::D(6), 1, 1),
            (SpecialIoKey::D(5), 2, 0),
            (SpecialIoKey::D(4), 3, 0),
            (SpecialIoKey::D(3), 6, 1),
            (SpecialIoKey::D(2), 7, 0),
            (SpecialIoKey::D(1), 8, 1),
            (SpecialIoKey::D(0), 9, 0),
        ] {
            let io = self.chip.clk_emid(Dir::N).delta(dx, 0).bel(bslots::IO[iob]);
            self.chip.special_io.insert(key, io);
        }
    }
}
