use crate::chip::{Chip, ChipKind, DisabledPart, Interposer};
use crate::expanded::ExpandedDevice;
use prjcombine_entity::EntityVec;
use prjcombine_interconnect::coord::DieId;
use prjcombine_interconnect::db::IntDb;
use std::collections::BTreeSet;

mod virtex4;
mod virtex5;
mod virtex6;
mod virtex7;

pub fn expand_grid<'a>(
    chips: &EntityVec<DieId, &'a Chip>,
    interposer: Option<&'a Interposer>,
    disabled: &BTreeSet<DisabledPart>,
    db: &'a IntDb,
) -> ExpandedDevice<'a> {
    match chips.first().unwrap().kind {
        ChipKind::Virtex4 => virtex4::expand_grid(chips, disabled, db),
        ChipKind::Virtex5 => virtex5::expand_grid(chips, disabled, db),
        ChipKind::Virtex6 => virtex6::expand_grid(chips, disabled, db),
        ChipKind::Virtex7 => virtex7::expand_grid(chips, interposer.unwrap(), disabled, db),
    }
}
