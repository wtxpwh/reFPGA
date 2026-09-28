use prjcombine_entity::{EntityPartVec, EntityVec};
use prjcombine_interconnect::grid::{BelCoord, CellCoord, ColId, ExpandedGrid, Rect, RowId};

use crate::{
    chip::{Chip, ChipKind},
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
            | ChipKind::Ecp3
            | ChipKind::Ecp3A
            | ChipKind::Ecp4
            | ChipKind::Ecp5 => {
                let dqs = self[io.cell].region_root[rslots::DQS].bel(bslots::DQS[0]);
                if self.has_bel(dqs) { Some(dqs) } else { None }
            }
            ChipKind::MachXo2(_) => {
                // TODO?
                None
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
