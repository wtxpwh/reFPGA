use std::collections::BTreeMap;

use prjcombine_interconnect::{
    dir::{DirH, DirV},
    grid::BelPadCoord,
};
use prjcombine_types::bscan::{BScanBuilder, BScanPad};

use crate::chip::{Chip, ColumnIoKind};

#[derive(Debug)]
pub struct BScan {
    pub bits: usize,
    pub io: BTreeMap<BelPadCoord, BScanPad>,
}

impl Chip {
    pub fn get_bscan(&self) -> BScan {
        let mut io = BTreeMap::new();
        let mut builder = BScanBuilder::new();
        // N
        for (col, &cd) in self.columns.iter().rev() {
            if cd.io[DirV::N] == ColumnIoKind::None {
                continue;
            }
            for (iob, unused) in [
                // inner
                (0, cd.io[DirV::N] == ColumnIoKind::Outer),
                (1, cd.io[DirV::N] == ColumnIoKind::Outer),
                // outer
                (2, cd.io[DirV::N] == ColumnIoKind::Inner),
                (3, cd.io[DirV::N] == ColumnIoKind::Inner),
            ] {
                if !unused {
                    let crd = self.pad_io_v(DirV::N, col, iob);
                    io.insert(crd, builder.get_toi());
                }
            }
        }
        // W
        for (row, &rd) in self.rows.iter().rev() {
            if rd.io[DirH::W] {
                for iob in [0, 1] {
                    let crd = self.pad_io_h(DirH::W, row, iob);
                    io.insert(crd, builder.get_toi());
                }
            }
        }
        // SW
        io.insert(self.pad_prog_b(), builder.get_toi());
        // S
        for (col, &cd) in &self.columns {
            if cd.io[DirV::S] == ColumnIoKind::None {
                continue;
            }
            for (iob, unused) in [
                // inner
                (0, cd.io[DirV::S] == ColumnIoKind::Outer),
                (1, cd.io[DirV::S] == ColumnIoKind::Outer),
                // outer
                (2, cd.io[DirV::S] == ColumnIoKind::Inner),
                (3, cd.io[DirV::S] == ColumnIoKind::Inner),
            ] {
                if !unused {
                    let crd = self.pad_io_v(DirV::S, col, iob);
                    io.insert(crd, builder.get_toi());
                }
            }
        }
        // SE
        io.insert(self.pad_done(), builder.get_toi());
        io.insert(self.pad_tspi_cs_b(), builder.get_toi());
        io.insert(self.pad_suspend(), builder.get_toi());
        // E
        for (row, &rd) in &self.rows {
            if rd.io[DirH::E] {
                for iob in [0, 1] {
                    let crd = self.pad_io_h(DirH::E, row, iob);
                    io.insert(crd, builder.get_toi());
                }
            }
        }
        BScan {
            bits: builder.bits,
            io,
        }
    }
}
