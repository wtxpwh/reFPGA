use std::collections::BTreeMap;

use prjcombine_interconnect::{
    coord::BelPadCoord,
    dir::{DirH, DirV},
};
use prjcombine_types::bscan::{BScanBuilder, BScanPad};

use crate::chip::{Chip, ChipKind};

#[derive(Debug)]
pub struct BScan {
    pub bits: usize,
    pub io: BTreeMap<BelPadCoord, BScanPad>,
    pub upd: usize,
}

impl Chip {
    pub fn get_bscan(&self) -> BScan {
        if self.kind.is_xc3000() || self.kind == ChipKind::Xc2000 {
            panic!("no boundary scan on XC2000/XC3000");
        }
        let iobs: &[_] = if self.kind == ChipKind::Xc5200 {
            &[3, 2, 1, 0]
        } else if self.kind == ChipKind::Xc4000H {
            &[0, 1, 2, 3]
        } else {
            &[0, 1]
        };
        let mut io = BTreeMap::new();
        let mut builder = BScanBuilder::new();
        if self.kind.is_xc4000() {
            io.insert(self.pad_tdo().unwrap(), builder.get_to());
        }
        // top edge, right-to-left
        for col in self.columns() {
            if col == self.col_w() || col == self.col_e() {
                continue;
            }
            for iob in iobs.iter().copied() {
                let pad = self.pad_io_v(DirV::N, col, iob);
                if self.unbonded_io.contains(&pad.bel) {
                    continue;
                }
                io.insert(pad, builder.get_toi());
            }
        }
        // left edge, top-to-bottom
        for row in self.rows().rev() {
            if row == self.row_s() || row == self.row_n() {
                continue;
            }
            for iob in iobs.iter().copied() {
                let pad = self.pad_io_h(DirH::E, row, iob);
                if self.unbonded_io.contains(&pad.bel) {
                    continue;
                }
                io.insert(pad, builder.get_toi());
            }
        }
        if self.kind.is_xc4000() {
            io.insert(self.pad_mode(1).unwrap(), builder.get_toi());
            io.insert(self.pad_mode(0).unwrap(), builder.get_i());
            if self.kind != ChipKind::SpartanXl {
                io.insert(self.pad_mode(2).unwrap(), builder.get_i());
            }
        }
        // bottom edge, left-to-right
        for col in self.columns().rev() {
            if col == self.col_w() || col == self.col_e() {
                continue;
            }
            for iob in iobs.iter().copied().rev() {
                let pad = self.pad_io_v(DirV::S, col, iob);
                if self.unbonded_io.contains(&pad.bel) {
                    continue;
                }
                io.insert(pad, builder.get_toi());
            }
        }
        // right edge, bottom-to-top
        for row in self.rows() {
            if row == self.row_s() || row == self.row_n() {
                continue;
            }
            for iob in iobs.iter().copied().rev() {
                let pad = self.pad_io_h(DirH::W, row, iob);
                if self.unbonded_io.contains(&pad.bel) {
                    continue;
                }
                io.insert(pad, builder.get_toi());
            }
        }
        let upd = builder.bits;
        builder.bits += 1;
        BScan {
            bits: builder.bits,
            io,
            upd,
        }
    }
}
