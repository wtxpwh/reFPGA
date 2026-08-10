use std::fmt::Debug;
use std::hash::Hash;

use bincode::{Decode, Encode};

use prjcombine_entity::entity_id_u16;

entity_id_u16!(pub BitRectId, BitRectTag tag, "R");
entity_id_u16!(pub RectFrameId arith, RectFrameTag tag, "F");
entity_id_u16!(pub RectBitId arith, RectBitTag tag, "B");

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct TileBit {
    pub rect: BitRectId,
    pub frame: RectFrameId,
    pub bit: RectBitId,
}

impl TileBit {
    pub const DUMMY: TileBit = TileBit::new(0xdead, 0xdead, 0xdead);

    pub const fn new(rect: usize, frame: usize, bit: usize) -> Self {
        Self {
            rect: BitRectId::from_idx_const(rect),
            frame: RectFrameId::from_idx_const(frame),
            bit: RectBitId::from_idx_const(bit),
        }
    }

    pub const fn pos(self) -> PolTileBit {
        PolTileBit {
            bit: self,
            inv: false,
        }
    }

    pub const fn neg(self) -> PolTileBit {
        PolTileBit {
            bit: self,
            inv: true,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct PolTileBit {
    pub bit: TileBit,
    pub inv: bool,
}

impl PolTileBit {
    pub const DUMMY: PolTileBit = TileBit::DUMMY.pos();
}

impl core::fmt::Debug for TileBit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.rect, self.frame, self.bit)
    }
}

impl core::fmt::Debug for PolTileBit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.inv {
            write!(f, "~{:?}", self.bit)
        } else {
            write!(f, "{:?}", self.bit)
        }
    }
}

impl core::ops::Not for PolTileBit {
    type Output = PolTileBit;

    fn not(self) -> Self::Output {
        PolTileBit {
            bit: self.bit,
            inv: !self.inv,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode)]
pub struct BitRectGeometry {
    pub frames: usize,
    pub bits: usize,
    pub orientation: FrameOrientation,
    pub rev_frames: bool,
    pub rev_bits: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode)]
pub enum FrameOrientation {
    Horizontal,
    Vertical,
}

pub trait BitRect: Copy + Debug + Hash + Eq + Sync + Send {
    type BitPos: Copy + Debug + Hash + Eq + Sync + Send;

    fn xlat_pos_rev(&self, bit: Self::BitPos) -> Option<(RectFrameId, RectBitId)>;
    fn xlat_pos_fwd(&self, bit: (RectFrameId, RectBitId)) -> Self::BitPos;
}
