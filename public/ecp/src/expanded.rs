use prjcombine_entity::{EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, ColId, Rect, RowId, RowIdExt},
    dir::{Dir, DirV},
    grid::ExpandedGrid,
};

use crate::{
    chip::{Chip, ChipKind, MachXo2Kind, RowKind},
    defs::{bslots, rslots},
};

pub struct ExpandedDevice<'a> {
    pub chip: &'a Chip,
    pub egrid: ExpandedGrid<'a>,
    pub bel_holes: Vec<Rect>,
    pub frame_len: usize,
    pub frames_num: usize,
    pub clk_frame: usize,
    pub col_frame: EntityVec<ColId, usize>,
    pub col_term_frame: EntityPartVec<ColId, usize>,
    pub row_bit: EntityVec<RowId, usize>,
    pub row_ebr_bit: EntityPartVec<RowId, usize>,
}

impl ExpandedDevice<'_> {
    pub fn is_in_int_hole(&self, cell: CellCoord) -> bool {
        !self.has_bel(cell.bel(bslots::INT))
    }

    pub fn is_in_bel_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.bel_holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    pub fn bel_io_dqs(&self, io: BelCoord) -> Option<BelCoord> {
        match self.chip.kind {
            ChipKind::Ecp
            | ChipKind::Xp
            | ChipKind::Ecp2
            | ChipKind::Ecp2M
            | ChipKind::Xp2
            | ChipKind::Ecp3(_)
            | ChipKind::Ecp4
            | ChipKind::Ecp5
            | ChipKind::Nexus(_) => {
                let dqs = self[io.cell].region_root[rslots::DQS].bel(bslots::DQS[0]);
                if self.has_bel(dqs) { Some(dqs) } else { None }
            }
            ChipKind::MachXo2(kind) => {
                if self.chip.rows[self.chip.row_clk].kind == RowKind::Ebr
                    && kind != MachXo2Kind::MachXo3L
                    && io.col == self.chip.col_e()
                {
                    Some(
                        self.chip
                            .clk_emid(Dir::E)
                            .bel(match io.row.side_of(self.chip.row_clk) {
                                DirV::S => bslots::DQS[1],
                                DirV::N => bslots::DQS[0],
                            }),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> std::ops::Deref for ExpandedDevice<'a> {
    type Target = ExpandedGrid<'a>;

    fn deref(&self) -> &Self::Target {
        &self.egrid
    }
}
