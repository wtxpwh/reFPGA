use std::collections::BTreeMap;

use prjcombine_interconnect::{
    coord::BelPadCoord,
    dir::{DirH, DirHV, DirV},
};
use prjcombine_types::bscan::{BScanBuilder, BScanPad};

use crate::chip::Chip;

#[derive(Debug)]
pub struct BScan {
    pub bits: usize,
    pub io: BTreeMap<BelPadCoord, BScanPad>,
}

impl Chip {
    pub fn get_bscan(&self) -> BScan {
        let mut builder = BScanBuilder::new();
        let mut io = BTreeMap::new();
        for col in self.columns().rev() {
            if self.cols_bram.contains(&col) {
                continue;
            }
            if col == self.col_w() || col == self.col_e() {
                continue;
            }
            for iob in [1, 2] {
                let pad = self.pad_io_v(DirV::N, col, iob);
                io.insert(pad, builder.get_toi());
            }
            if col == self.col_clk() {
                io.insert(self.pad_clk(DirHV::NE), builder.get_i());
                io.insert(self.pad_clk(DirHV::NW), builder.get_i());
            }
        }
        for row in self.rows().rev() {
            if row == self.row_s() || row == self.row_n() {
                continue;
            }
            for iob in [1, 2, 3] {
                let pad = self.pad_io_h(DirH::W, row, iob);
                io.insert(pad, builder.get_toi());
            }
        }
        for i in [1, 0, 2] {
            io.insert(self.pads_mode()[i], builder.get_i());
        }
        for col in self.columns() {
            if self.cols_bram.contains(&col) {
                continue;
            }
            if col == self.col_w() || col == self.col_e() {
                continue;
            }
            if col == self.col_clk() {
                io.insert(self.pad_clk(DirHV::SW), builder.get_i());
                io.insert(self.pad_clk(DirHV::SE), builder.get_i());
            }
            for iob in [2, 1] {
                let pad = self.pad_io_v(DirV::S, col, iob);
                io.insert(pad, builder.get_toi());
            }
        }
        io.insert(self.pad_done(), builder.get_toi());
        io.insert(self.pad_prog_b(), builder.get_i());
        for row in self.rows() {
            if row == self.row_s() || row == self.row_n() {
                continue;
            }
            for iob in [3, 2, 1] {
                let pad = self.pad_io_h(DirH::E, row, iob);
                io.insert(pad, builder.get_toi());
            }
        }
        io.insert(self.pad_cclk(), builder.get_toi());
        BScan {
            bits: builder.bits,
            io,
        }
    }
}
