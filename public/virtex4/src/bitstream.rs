use std::collections::{BTreeMap, HashMap};

use prjcombine_entity::{EntityId, EntityVec};
use prjcombine_interconnect::{dir::DirV, grid::DieId};
use prjcombine_types::{
    bitvec::BitVec,
    bsdata::{RectBitId, RectFrameId},
};

use crate::{chip::ChipKind, expanded::FrameInfo};

mod packet;
mod parse;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Reg {
    Idcode,
    Ctl0,
    Ctl1,
    Cor0,
    Cor1,
    WbStar,
    Timer,
    Trim0,
    Trim1,
    Trim2,
    Testmode,
    Bspi,
    Axss,
    RbCrcSw,
    FakeIgnoreCrc,
    FakeEncrypted,
    FakeFallEdge,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitstreamMode {
    Plain,
    Debug,
    Compress,
    PerFrameCrc,
    Encrypt,
}

#[derive(Clone, Debug)]
pub struct DieBitstream {
    pub regs: BTreeMap<Reg, u32>,
    pub mode: BitstreamMode,
    pub iv: Option<[u8; 16]>,
    pub frame_len: usize,
    pub frame_data: BitVec,
    pub frame_info: Vec<FrameInfo>,
    pub frame_present: BitVec,
}

#[derive(Clone, Debug)]
pub struct GtzBitstream {
    pub idcode: u32,
    pub data: Vec<u32>,
    pub code: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct Bitstream {
    pub kind: ChipKind,
    pub die: EntityVec<DieId, DieBitstream>,
    pub gtz: BTreeMap<DirV, GtzBitstream>,
    pub gtz_loader: Option<Box<Bitstream>>,
}

impl Bitstream {
    pub fn diff(a: &Bitstream, b: &Bitstream) -> HashMap<BitPos, bool> {
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.die.len(), b.die.len());
        let mut res = HashMap::new();
        for ((die, da), db) in a.die.iter().zip(b.die.values()) {
            for (&reg, &va) in &da.regs {
                if matches!(reg, Reg::RbCrcSw | Reg::FakeEncrypted) {
                    continue;
                }
                let vb = db.regs.get(&reg);
                if vb.is_none() && reg != Reg::Testmode {
                    res.insert(BitPos::RegPresent(die, reg), false);
                }
                let vb = vb.copied().unwrap_or(0);
                if va != vb {
                    for j in 0..32 {
                        if (va >> j & 1) != (vb >> j & 1) {
                            res.insert(BitPos::Reg(die, reg, j), (vb >> j & 1) != 0);
                        }
                    }
                }
            }
            for (&reg, &vb) in &db.regs {
                if matches!(reg, Reg::RbCrcSw | Reg::FakeEncrypted) {
                    continue;
                }
                if !da.regs.contains_key(&reg) {
                    let va = 0;
                    if reg != Reg::Testmode {
                        res.insert(BitPos::RegPresent(die, reg), true);
                    }
                    if va != vb {
                        for j in 0..32 {
                            if (va >> j & 1) != (vb >> j & 1) {
                                res.insert(BitPos::Reg(die, reg, j), (vb >> j & 1) != 0);
                            }
                        }
                    }
                }
            }
            assert_eq!(da.frame_len, db.frame_len);
            assert_eq!(da.frame_info, db.frame_info);
            for (pos, val) in BitVec::diff(&da.frame_data, &db.frame_data) {
                let frame = pos / da.frame_len;
                let bit = pos % da.frame_len;
                let pos = BitPos::Main(die, frame, bit);
                let is_ecc = match a.kind {
                    ChipKind::Virtex4 | ChipKind::Virtex5 => (640..652).contains(&bit),
                    ChipKind::Virtex6 => (1280..1293).contains(&bit),
                    ChipKind::Virtex7 => (1600..1613).contains(&bit),
                };
                if !is_ecc {
                    res.insert(pos, val);
                }
            }
        }
        for (&dir, ba) in &a.gtz {
            let bb = &b.gtz[&dir];
            for (i, (&wa, &wb)) in ba.data.iter().zip(bb.data.iter()).enumerate() {
                if wa != wb {
                    for j in 0..32 {
                        if (wa >> j & 1) != (wb >> j & 1) {
                            res.insert(BitPos::Gtz(dir, i, j), (wb >> j & 1) != 0);
                        }
                    }
                }
            }
        }
        res
    }

    pub fn get_bit(&self, bit: BitPos) -> bool {
        match bit {
            BitPos::Reg(die, reg, bit) => match self.die[die].regs.get(&reg) {
                Some(val) => (val >> bit & 1) != 0,
                None => false,
            },
            BitPos::RegPresent(die, reg) => self.die[die].regs.contains_key(&reg),
            BitPos::Main(die, frame, bit) => {
                let dbs = &self.die[die];
                dbs.frame_data[frame * dbs.frame_len + bit]
            }
            BitPos::Gtz(dir, frame, bit) => (self.gtz[&dir].data[frame] >> bit & 1) != 0,
        }
    }
}

impl DieBitstream {
    pub fn get_bit(&self, frame: usize, bit: usize) -> bool {
        self.frame_data[frame * self.frame_len + bit]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitPos {
    Reg(DieId, Reg, usize),
    RegPresent(DieId, Reg),
    // die, frame, bit
    Main(DieId, usize, usize),
    Gtz(DirV, usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BitRect {
    Null,
    Reg(DieId, Reg),
    RegPresent(DieId, Reg),
    // die, frame, width, bit, height, flip
    Main(DieId, usize, usize, usize, usize, bool),
    // start word, num words
    Gtz(DirV, usize, usize),
}

impl prjcombine_types::bitrect::BitRect for BitRect {
    type BitPos = BitPos;

    fn xlat_pos_rev(&self, bit: BitPos) -> Option<(RectFrameId, RectBitId)> {
        let (rframe, rbit) = match (*self, bit) {
            (BitRect::Reg(die, reg), BitPos::Reg(bdie, breg, pos))
                if bdie == die && breg == reg =>
            {
                (0, pos)
            }
            (BitRect::RegPresent(die, reg), BitPos::RegPresent(bdie, breg))
                if bdie == die && breg == reg =>
            {
                (0, 0)
            }
            (
                BitRect::Main(die, frame, width, bit, height, flip),
                BitPos::Main(bdie, bframe, bbit),
            ) if die == bdie
                && bframe >= frame
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

            (BitRect::Gtz(dir, start, len), BitPos::Gtz(bdir, frame, bit))
                if dir == bdir && frame >= start && frame < start + len =>
            {
                (frame - start, bit)
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
            BitRect::Reg(die, reg) => {
                assert_eq!(rframe, 0);
                BitPos::Reg(die, reg, rbit)
            }
            BitRect::RegPresent(die, reg) => {
                assert_eq!(rframe, 0);
                assert_eq!(rbit, 0);
                BitPos::RegPresent(die, reg)
            }
            BitRect::Main(die, frame, width, bit, height, flip) => {
                assert!(rframe < width);
                assert!(rbit < height);
                BitPos::Main(
                    die,
                    frame + rframe,
                    if flip {
                        bit + height - 1 - rbit
                    } else {
                        bit + rbit
                    },
                )
            }
            BitRect::Gtz(dir, start, len) => {
                assert!(rframe < len);
                BitPos::Gtz(dir, start + rframe, rbit)
            }
        }
    }
}
