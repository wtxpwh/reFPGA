use std::collections::{BTreeMap, HashMap};

use prjcombine_entity::EntityId;
use prjcombine_types::{
    bitrect::{RectBitId, RectFrameId},
    bitvec::BitVec,
};

pub mod packet;
mod parse;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum KeySeq {
    First,
    Middle,
    Last,
    Single,
}

#[derive(Copy, Clone, Debug)]
pub struct KeyData {
    pub key: [[u8; 7]; 6],
    pub keyseq: [KeySeq; 6],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Reg {
    Ctl0,
    Cor0,
    FakeEarlyGhigh,
    FakeDoubleGrestore,
    FakeHasSwitch,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitstreamMode {
    Plain,
    Debug,
    Compress,
}

impl Bitstream {
    pub fn diff(a: &Bitstream, b: &Bitstream) -> HashMap<BitPos, bool> {
        let mut res = HashMap::new();
        for (&reg, &va) in &a.regs {
            let vb = b.regs.get(&reg);
            let vb = vb.copied().unwrap_or(0);
            if va != vb {
                for j in 0..32 {
                    if (va >> j & 1) != (vb >> j & 1) {
                        res.insert(BitPos::Reg(reg, j), (vb >> j & 1) != 0);
                    }
                }
            }
        }
        for (&reg, &vb) in &b.regs {
            if !a.regs.contains_key(&reg) {
                let va = 0;
                if va != vb {
                    for j in 0..32 {
                        if (va >> j & 1) != (vb >> j & 1) {
                            res.insert(BitPos::Reg(reg, j), (vb >> j & 1) != 0);
                        }
                    }
                }
            }
        }
        assert_eq!(a.frame_len, b.frame_len);
        assert_eq!(a.frame_info, b.frame_info);
        for (pos, val) in BitVec::diff(&a.frame_data, &b.frame_data) {
            let frame = pos / a.frame_len;
            let bit = pos % a.frame_len;
            res.insert(BitPos::Main(frame, bit), val);
        }
        res
    }

    pub fn get_bit(&self, bit: BitPos) -> bool {
        match bit {
            BitPos::Reg(reg, bit) => match self.regs.get(&reg) {
                Some(val) => (val >> bit & 1) != 0,
                None => false,
            },
            BitPos::Main(frame, bit) => self.frame_data[frame * self.frame_len + bit],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bitstream {
    pub regs: BTreeMap<Reg, u32>,
    pub mode: BitstreamMode,
    pub frame_len: usize,
    pub frame_data: BitVec,
    pub frame_info: Vec<FrameAddr>,
    pub frame_present: BitVec,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct FrameAddr {
    pub typ: u32,
    pub major: u32,
    pub minor: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitPos {
    Reg(Reg, usize),
    // frame, bit
    Main(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitRect {
    Null,
    Reg(Reg),
    // frame, width, bit, height, flip
    Main(usize, usize, usize, usize, bool),
}

impl prjcombine_types::bitrect::BitRect for BitRect {
    type BitPos = BitPos;

    fn xlat_pos_rev(&self, bit: BitPos) -> Option<(RectFrameId, RectBitId)> {
        let (rframe, rbit) = match (*self, bit) {
            (BitRect::Reg(reg), BitPos::Reg(breg, pos)) if breg == reg => (0, pos),
            (BitRect::Main(frame, width, bit, height, flip), BitPos::Main(bframe, bbit))
                if bframe >= frame
                    && bframe < frame + width
                    && bbit >= bit
                    && bbit < bit + height =>
            {
                (
                    bframe - frame,
                    if flip {
                        height - 1 - (bbit - bit)
                    } else {
                        bbit - bit
                    },
                )
            }
            _ => return None,
        };
        Some((RectFrameId::from_idx(rframe), RectBitId::from_idx(rbit)))
    }

    fn xlat_pos_fwd(&self, bit: (RectFrameId, RectBitId)) -> BitPos {
        let (rframe, rbit) = bit;
        let rframe = rframe.to_idx();
        let rbit = rbit.to_idx();
        match *self {
            BitRect::Null => unreachable!(),
            BitRect::Reg(reg) => {
                assert_eq!(rframe, 0);
                BitPos::Reg(reg, rbit)
            }
            BitRect::Main(frame, width, bit, height, flip) => {
                assert!(rframe < width);
                assert!(rbit < height);
                BitPos::Main(
                    frame + rframe,
                    if flip {
                        bit + height - 1 - rbit
                    } else {
                        bit + rbit
                    },
                )
            }
        }
    }
}
