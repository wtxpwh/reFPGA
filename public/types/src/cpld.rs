use core::fmt::Debug;

use bincode::{Decode, Encode};
use prjcombine_entity::{EntityId, entity_id_u8};

pub mod bsdata;

entity_id_u8!(pub ClusterId, ClusterTag tag, "C");
entity_id_u8!(pub BlockId, BlockTag tag, "B");
entity_id_u8!(pub MacrocellId, MacrocellTag tag, "MC");
entity_id_u8!(pub ProductTermId, ProductTermTag tag, "PT");
entity_id_u8!(pub IpadId, IpadTag tag, "IPAD");

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct MacrocellCoord {
    pub cluster: ClusterId,
    pub block: BlockId,
    pub macrocell: MacrocellId,
}

impl MacrocellCoord {
    pub fn simple(block: BlockId, macrocell: MacrocellId) -> Self {
        Self {
            cluster: ClusterId::from_idx(0),
            block,
            macrocell,
        }
    }
}

impl Debug for MacrocellCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for MacrocellCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.cluster, self.block, self.macrocell)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum IoCoord {
    Ipad(IpadId),
    Macrocell(MacrocellCoord),
}

impl Debug for IoCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for IoCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoCoord::Ipad(ipad) => write!(f, "{ipad}"),
            IoCoord::Macrocell(mc) => write!(f, "IOB_{mc}"),
        }
    }
}
