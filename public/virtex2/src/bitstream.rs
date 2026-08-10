pub mod packet_s3a;
use prjcombine_types::bitvec::BitVec;
pub use prjcombine_virtex::bitstream::packet as packet_v2;
pub use prjcombine_virtex::bitstream::{KeyData, KeySeq};

use prjcombine_entity::EntityId;
use prjcombine_types::bitrect::{RectBitId, RectFrameId};
use std::collections::{BTreeMap, HashMap};

mod parse;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Reg {
    Idcode,
    Ctl0,
    Ctl1,
    Cor0,
    Cor1,
    Cor2,
    Key,
    Timer,
    Testmode,
    RbCrcSw,
    CclkFrequency,
    Powerdown,
    HcOpt,
    PuGwe,
    PuGts,
    SeuOpt,
    Mode,
    General1,
    General2,
    FakeEarlyGhigh,
    FakeDoubleGrestore,
    FakeFreezeDciNops,
    FakeHasSwitch,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitstreamMode {
    Plain,
    Debug,
    Compress,
    Encrypt,
}

impl Bitstream {
    pub fn diff(a: &Bitstream, b: &Bitstream) -> HashMap<BitPos, bool> {
        let mut res = HashMap::new();
        for (&reg, &va) in &a.regs {
            if matches!(reg, Reg::RbCrcSw | Reg::Key) {
                continue;
            }
            let vb = b.regs.get(&reg);
            if vb.is_none() && reg != Reg::Testmode {
                res.insert(BitPos::RegPresent(reg), false);
            }
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
            if matches!(reg, Reg::RbCrcSw | Reg::Key) {
                continue;
            }
            if !a.regs.contains_key(&reg) {
                let va = 0;
                if reg != Reg::Testmode {
                    res.insert(BitPos::RegPresent(reg), true);
                }
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
        for k in a.frame_fixups.keys() {
            if !b.frame_fixups.contains_key(k) {
                res.insert(BitPos::Fixup(k.0, k.1), false);
            }
        }
        for k in b.frame_fixups.keys() {
            if !a.frame_fixups.contains_key(k) {
                res.insert(BitPos::Fixup(k.0, k.1), true);
            }
        }
        res
    }

    pub fn get_bit(&self, bit: BitPos) -> bool {
        match bit {
            BitPos::Reg(reg, bit) => match self.regs.get(&reg) {
                Some(val) => (val >> bit & 1) != 0,
                None => false,
            },
            BitPos::RegPresent(reg) => self.regs.contains_key(&reg),
            BitPos::Main(frame, bit) => self.frame_data[frame * self.frame_len + bit],
            BitPos::Fixup(frame, bit) => self.frame_fixups.contains_key(&(frame, bit)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bitstream {
    pub regs: BTreeMap<Reg, u32>,
    pub mode: BitstreamMode,
    pub iv: Option<[u8; 8]>,
    pub frame_len: usize,
    pub frame_data: BitVec,
    pub frame_info: Vec<FrameAddr>,
    pub frame_present: BitVec,
    // frame idx, bit idx
    pub frame_fixups: HashMap<(usize, usize), bool>,
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
    RegPresent(Reg),
    // frame, bit
    Main(usize, usize),
    Fixup(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitRect {
    Null,
    Reg(Reg),
    RegPresent(Reg),
    // frame, width, bit, height, flip
    Main(usize, usize, usize, usize, bool),
    Fixup(usize, usize, usize, usize, bool),
}

impl prjcombine_types::bitrect::BitRect for BitRect {
    type BitPos = BitPos;

    fn xlat_pos_rev(&self, bit: BitPos) -> Option<(RectFrameId, RectBitId)> {
        let (rframe, rbit) = match (*self, bit) {
            (BitRect::Reg(reg), BitPos::Reg(breg, pos)) if breg == reg => (0, pos),
            (BitRect::RegPresent(reg), BitPos::RegPresent(breg)) if breg == reg => (0, 0),
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
            (BitRect::Fixup(frame, width, bit, height, flip), BitPos::Fixup(bframe, bbit))
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
            BitRect::RegPresent(reg) => {
                assert_eq!(rframe, 0);
                assert_eq!(rbit, 0);
                BitPos::RegPresent(reg)
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
            BitRect::Fixup(frame, width, bit, height, flip) => {
                assert!(rframe < width);
                assert!(rbit < height);
                BitPos::Fixup(
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

impl BitRect {
    pub fn to_fixup(self) -> BitRect {
        match self {
            BitRect::Main(frame, width, bit, height, flip) => {
                BitRect::Fixup(frame, width, bit, height, flip)
            }
            _ => unreachable!(),
        }
    }
}
