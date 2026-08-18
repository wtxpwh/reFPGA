use std::collections::BTreeMap;

use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{DirH, DirHV, DirV},
    grid::{BelPadCoord, DieId, DieIdExt},
};
use prjcombine_types::bscan::{BScanBuilder, BScanPad};

use crate::{
    chip::{Chip, ChipKind, ColumnKind},
    defs::{bcls, bslots},
    iob::IobKind,
};

#[derive(Debug)]
pub struct BScan {
    pub bits: usize,
    pub io: BTreeMap<BelPadCoord, BScanPad>,
}

impl Chip {
    pub fn get_bscan(&self) -> BScan {
        let mut io = BTreeMap::new();
        let mut builder = BScanBuilder::new();
        if self.kind == ChipKind::FpgaCore {
            for col in self.columns.ids().rev() {
                if self.columns[col].kind == ColumnKind::Clb {
                    for i in 0..4 {
                        let crd = self.pad_io_v(DirV::N, col, bslots::OREG[i]);
                        io.insert(crd, builder.get_o());
                        let crd = self.pad_io_v(DirV::N, col, bslots::IREG[i]);
                        io.insert(crd, builder.get_i());
                    }
                }
            }
            io.insert(self.pad_prog_b(), builder.get_i());
            for row in self.rows.ids().rev() {
                if row != self.row_s() && row != self.row_n() {
                    for i in (0..4).rev() {
                        let crd = self.pad_io_h(DirH::W, row, bslots::OREG[i]);
                        io.insert(crd, builder.get_o());
                        let crd = self.pad_io_h(DirH::W, row, bslots::IREG[i]);
                        io.insert(crd, builder.get_i());
                    }
                }
            }
            for i in [1, 0, 2] {
                io.insert(self.pads_mode()[i], builder.get_i());
            }
            for col in self.columns.ids() {
                if self.columns[col].kind == ColumnKind::Clb {
                    for i in 0..4 {
                        let crd = self.pad_io_v(DirV::S, col, bslots::OREG[i]);
                        io.insert(crd, builder.get_o());
                        let crd = self.pad_io_v(DirV::S, col, bslots::IREG[i]);
                        io.insert(crd, builder.get_i());
                    }
                }
            }
            let bel = self.bel_corner_misc(DirHV::SE);
            for i in (0..8).rev() {
                io.insert(bel.pad(bcls::MISC_SE::DOUT[i]), builder.get_o());
            }
            io.insert(bel.pad(bcls::MISC_SE::INIT), builder.get_o());
            io.insert(bel.pad(bcls::MISC_SE::DONE_O), builder.get_o());
            io.insert(bel.pad(bcls::MISC_SE::POR_EN_B), builder.get_i());
            for i in (0..8).rev() {
                io.insert(bel.pad(bcls::MISC_SE::DIN[i]), builder.get_i());
            }
            io.insert(bel.pad(bcls::MISC_SE::WRITE_B), builder.get_i());
            io.insert(bel.pad(bcls::MISC_SE::DONE_I), builder.get_i());
            io.insert(bel.pad(bcls::MISC_SE::CS_B), builder.get_i());
            io.insert(bel.pad(bcls::MISC_SE::CCLK), builder.get_i());
            for row in self.rows.ids() {
                if row != self.row_s() && row != self.row_n() {
                    for i in 0..4 {
                        let crd = self.pad_io_h(DirH::E, row, bslots::OREG[i]);
                        io.insert(crd, builder.get_o());
                        let crd = self.pad_io_h(DirH::E, row, bslots::IREG[i]);
                        io.insert(crd, builder.get_i());
                    }
                }
            }
        } else {
            let die = DieId::from_idx(0);
            for col in self.columns.ids().rev() {
                let row = self.row_n();
                if let Some((data, tidx)) = self.get_iob_tile_data(die.cell(col, row)) {
                    for &iob in data.iobs.iter().rev() {
                        if iob.cell == tidx {
                            let crd = self.pad_io_v(DirV::N, col, iob.ioi);
                            if iob.kind == IobKind::Iob {
                                io.insert(crd, builder.get_toi());
                            } else {
                                io.insert(crd, builder.get_i());
                            };
                        }
                    }
                }
            }
            if !self.kind.is_spartan3ea() {
                io.insert(self.pad_hswap_en(), builder.get_i());
            }
            io.insert(self.pad_prog_b(), builder.get_i());
            for row in self.rows.ids().rev() {
                let col = self.col_w();
                if let Some((data, tidx)) = self.get_iob_tile_data(die.cell(col, row)) {
                    for &iob in data.iobs.iter().rev() {
                        if iob.cell == tidx {
                            let crd = self.pad_io_h(DirH::W, row, iob.ioi);
                            if iob.kind == IobKind::Iob {
                                io.insert(crd, builder.get_toi());
                            } else {
                                io.insert(crd, builder.get_i());
                            };
                        }
                    }
                }
            }
            if !self.kind.is_spartan3ea() {
                for i in [1, 0, 2] {
                    io.insert(self.pads_mode()[i], builder.get_i());
                }
            }
            for col in self.columns.ids() {
                let row = self.row_s();
                if let Some((data, tidx)) = self.get_iob_tile_data(die.cell(col, row)) {
                    for &iob in data.iobs.iter().rev() {
                        if iob.cell == tidx {
                            let crd = self.pad_io_v(DirV::S, col, iob.ioi);
                            if iob.kind == IobKind::Iob {
                                io.insert(crd, builder.get_toi());
                            } else {
                                io.insert(crd, builder.get_i());
                            };
                        }
                    }
                }
            }
            io.insert(self.pad_done(), builder.get_toi());
            if self.kind.is_virtex2() {
                io.insert(self.pad_powerdown_b(), builder.get_i());
            }
            if !self.kind.is_spartan3ea() {
                io.insert(self.pad_cclk(), builder.get_toi());
            }
            if self.kind.is_spartan3a() {
                io.insert(self.pad_suspend(), builder.get_i());
            }
            for row in self.rows.ids() {
                let col = self.col_e();
                if let Some((data, tidx)) = self.get_iob_tile_data(die.cell(col, row)) {
                    for &iob in data.iobs.iter().rev() {
                        if iob.cell == tidx {
                            let crd = self.pad_io_h(DirH::E, row, iob.ioi);
                            if iob.kind == IobKind::Iob {
                                io.insert(crd, builder.get_toi());
                            } else {
                                io.insert(crd, builder.get_i());
                            };
                        }
                    }
                }
            }
        }
        BScan {
            bits: builder.bits,
            io,
        }
    }
}
