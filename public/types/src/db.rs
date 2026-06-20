use bincode::{Decode, Encode};
use prjcombine_entity::{entity_id_u8, entity_id_u16};

entity_id_u16!(pub ChipId, ChipTag tag, "CHIP");
entity_id_u16!(pub BondId, BondTag tag, "BOND");
entity_id_u16!(pub InterposerId, InterposerTag tag, "INTERPOSER");
entity_id_u16!(pub SpeedId, SpeedTag tag, "SPEED");
entity_id_u8!(pub DevBondId, DevBondTag tag, "DEVBOND");
entity_id_u8!(pub DevSpeedId, DevSpeedTag tag, "DEVSPEED");

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct DeviceCombo {
    pub devbond: DevBondId,
    pub speed: DevSpeedId,
}

#[derive(Copy, Clone, Debug)]
pub struct DumpFlags {
    pub intdb: bool,
    pub chip: bool,
    pub bond: bool,
    pub speed: bool,
    pub device: bool,
    pub bsdata: bool,
}

impl DumpFlags {
    pub fn all() -> Self {
        DumpFlags {
            intdb: true,
            chip: true,
            bond: true,
            device: true,
            speed: true,
            bsdata: true,
        }
    }
}
