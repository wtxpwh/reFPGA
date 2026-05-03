use prjcombine_entity::EntityId;
use prjcombine_types::{
    bitvec::BitVec,
    bsdata::{RectBitId, RectFrameId},
};
use std::collections::{BTreeMap, HashMap};

mod parse;

pub const FRAME_LEN: usize = 16 * 64 + 16;
pub const BRAM_FRAME_LEN: usize = FRAME_LEN * 18;

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
    EyeMask,
    HcOpt,
    PuGwe,
    PuGts,
    SeuOpt,
    Mode,
    General1,
    General2,
    General3,
    General4,
    General5,
    FakeIgnoreCrc,
    FakeDoubleCclkFrequency,
    FakeHasSwitch,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitstreamMode {
    Plain,
    Debug,
    Compress,
    Encrypt,
}

#[derive(Clone, Debug)]
pub struct Bitstream {
    pub regs: BTreeMap<Reg, u32>,
    pub mode: BitstreamMode,
    pub iv: Option<[u8; 16]>,
    pub frame_data: BitVec,
    pub frame_info: Vec<FrameAddr>,
    pub frame_present: BitVec,
    pub bram_data: BitVec,
    pub bram_frame_info: Vec<FrameAddr>,
    pub bram_frame_present: BitVec,
    pub iob: BitVec,
    pub iob_present: bool,
    // frame idx, bit idx
    pub frame_fixups: HashMap<(usize, usize), bool>,
}

impl Bitstream {
    pub fn diff(a: &Bitstream, b: &Bitstream) -> HashMap<BitPos, bool> {
        let mut res = HashMap::new();
        for (&reg, &va) in &a.regs {
            if matches!(reg, Reg::RbCrcSw | Reg::Key | Reg::FakeDoubleCclkFrequency) {
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
            if matches!(reg, Reg::RbCrcSw | Reg::Key | Reg::FakeDoubleCclkFrequency) {
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
        assert_eq!(a.frame_info, b.frame_info);
        for (pos, val) in BitVec::diff(&a.frame_data, &b.frame_data) {
            let frame = pos / FRAME_LEN;
            let bit = pos % FRAME_LEN;
            res.insert(BitPos::Main(frame, bit), val);
        }
        assert_eq!(a.bram_frame_info, b.bram_frame_info);
        for (pos, val) in BitVec::diff(&a.bram_data, &b.bram_data) {
            let frame = pos / BRAM_FRAME_LEN;
            let bit = pos % BRAM_FRAME_LEN;
            res.insert(BitPos::Bram(frame, bit), val);
        }
        if a.iob != b.iob {
            assert_eq!(a.iob.len(), b.iob.len());
            for j in 0..a.iob.len() {
                if a.iob[j] != b.iob[j] {
                    res.insert(BitPos::Iob(j), b.iob[j]);
                }
            }
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
            BitPos::Main(frame, bit) => self.frame_data[frame * FRAME_LEN + bit],
            BitPos::Fixup(frame, bit) => self.frame_fixups.contains_key(&(frame, bit)),
            BitPos::Bram(frame, bit) => self.bram_data[frame * BRAM_FRAME_LEN + bit],
            BitPos::Iob(bit) => self.iob[bit],
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct FrameAddr {
    pub typ: u32,
    pub region: u32,
    pub major: u32,
    pub minor: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FrameMaskMode {
    // no masked bits; everything is read back
    None,
    // bits 1-16 of each 20-bit group masked iff bit 18 of the group is set
    DrpV4,
    // INIT_[AB] bits masked
    BramV4,
    // all bits masked
    All,
    // whole frame masked iff given (frame, bit) bit in HCLK tile is set
    DrpHclk(usize, usize),
    // likewise, but the mask bit is 3 major columns to the left
    PcieLeftDrpHclk(usize, usize),
    // whole frame except two tiles nearest to HCLK masked iff given (frame, bit) bit in HCLK tile is set
    CmtDrpHclk(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitPos {
    Reg(Reg, usize),
    RegPresent(Reg),
    // frame, bit
    Main(usize, usize),
    Fixup(usize, usize),
    Bram(usize, usize),
    Iob(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitRect {
    Null,
    Reg(Reg),
    RegPresent(Reg),
    // frame, width, bit, height, flip
    Main(usize, usize, usize, usize, bool),
    Fixup(usize, usize, usize, usize, bool),
    // Spartan 6 horrible; single whole frame
    Bram(usize),
    // Spartan 6 horrible; bit, width
    Iob(usize, usize),
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
            (BitRect::Bram(frame), BitPos::Bram(bframe, pos)) if bframe == frame => (0, pos),
            (BitRect::Iob(bit, height), BitPos::Iob(bbit))
                if bbit >= bit && bbit < bit + height =>
            {
                (0, bbit - bit)
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
            BitRect::Bram(frame) => {
                assert_eq!(rframe, 0);
                BitPos::Bram(frame, rbit)
            }
            BitRect::Iob(bit, height) => {
                assert!(rbit < height);
                BitPos::Iob(bit + rbit)
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
