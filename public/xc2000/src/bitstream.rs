use bitvec::prelude::*;
use prjcombine_entity::EntityId;
use prjcombine_types::{
    bitrect::{RectBitId, RectFrameId},
    bitvec::BitVec,
};

use crate::{chip::ChipKind, expanded::ExpandedDevice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitstream {
    frame_len: usize,
    frame_data: BitVec,
    pub lc_alignment_done: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitPos {
    pub frame: usize,
    pub bit: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitRect {
    pub frame: usize,
    pub width: usize,
    pub bit: usize,
    pub height: usize,
}

impl prjcombine_types::bitrect::BitRect for BitRect {
    type BitPos = BitPos;

    fn xlat_pos_rev(&self, bit: BitPos) -> Option<(RectFrameId, RectBitId)> {
        if bit.frame >= self.frame
            && bit.frame < self.frame + self.width
            && bit.bit >= self.bit
            && bit.bit < self.bit + self.height
        {
            Some((
                RectFrameId::from_idx(bit.frame - self.frame),
                RectBitId::from_idx(bit.bit - self.bit),
            ))
        } else {
            None
        }
    }

    fn xlat_pos_fwd(&self, bit: (RectFrameId, RectBitId)) -> BitPos {
        let (rframe, rbit) = bit;
        let rframe = rframe.to_idx();
        let rbit = rbit.to_idx();
        assert!(rframe < self.height);
        assert!(rbit < self.width);
        BitPos {
            frame: self.frame + rframe,
            bit: self.bit + rbit,
        }
    }
}

impl Bitstream {
    pub fn empty(frame_len: usize, frames_num: usize) -> Self {
        Self {
            frame_len,
            frame_data: BitVec::repeat(true, frame_len * frames_num),
            lc_alignment_done: false,
        }
    }

    pub fn frame_len(&self) -> usize {
        self.frame_len
    }

    pub fn frames_num(&self) -> usize {
        self.frame_data.len() / self.frame_len
    }

    pub fn diff(a: &Bitstream, b: &Bitstream) -> impl Iterator<Item = (BitPos, bool)> {
        assert_eq!(a.frame_len, b.frame_len);
        BitVec::diff(&a.frame_data, &b.frame_data).map(|(pos, val)| {
            let frame = pos / a.frame_len;
            let bit = pos % a.frame_len;
            let pos = BitPos { frame, bit };
            (pos, val)
        })
    }

    pub fn get_bit(&self, pos: BitPos) -> bool {
        self.frame_data[pos.frame * self.frame_len + pos.bit]
    }

    pub fn set_bit(&mut self, pos: BitPos, val: bool) {
        self.frame_data
            .set(pos.frame * self.frame_len + pos.bit, val);
    }
}

fn parse_xc2000_bitstream(bs: &mut Bitstream, data: &[u8]) {
    let data: &BitSlice<u8, Msb0> = BitSlice::from_slice(data);
    assert_eq!(data[..12], bits![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0]);
    let mut bitlen = 0;
    for j in 0..24 {
        if data[35 - j] {
            bitlen |= 1 << j;
        }
    }
    assert_eq!(data[36..40], bits![1, 1, 1, 1]);
    let mut pos = 40;
    let frame_len = bs.frame_len();
    for frame in 0..bs.frames_num() {
        assert!(!data[pos]);
        pos += 1;
        let fdata = &data[pos..(pos + frame_len)];
        for (bit, val) in fdata.iter().enumerate() {
            bs.set_bit(BitPos { frame, bit }, *val);
        }
        pos += frame_len;
        let stop = &data[pos..(pos + 3)];
        assert_eq!(stop, &bits![1, 1, 1]);
        pos += 3;
    }
    let post = &data[pos..(pos + 4)];
    assert_eq!(post, &bits![1, 1, 1, 1]);
    pos += 4;
    while !pos.is_multiple_of(8) {
        assert!(data[pos]);
        pos += 1;
    }
    assert_eq!(bitlen, pos + 1);
    let pad = &data[pos..(pos + 8)];
    assert_eq!(pad, &bits![1, 1, 1, 1, 1, 1, 1, 1]);
    pos += 8;
    assert_eq!(pos, data.len());
}

struct Xc4000Crc {
    crc: u16,
}

impl Xc4000Crc {
    fn new() -> Self {
        Self { crc: 0 }
    }

    fn feed_bit(&mut self, b: bool) {
        if !b {
            self.crc ^= 0x8000;
        }
        if (self.crc & 0x8000) != 0 {
            self.crc <<= 1;
            self.crc ^= 0x8005;
        } else {
            self.crc <<= 1;
        }
    }
}

fn parse_xc4000_bitstream(bs: &mut Bitstream, data: &[u8], is_s40xl: bool) {
    if data.starts_with(&[0xff, 0xff, 0xf2]) {
        let bitlen = (data[3] as u32) << 16 | (data[4] as u32) << 8 | (data[5] as u32);
        assert_eq!(data[6], 0xd2);
        let mut pos = 7;
        let frame_len = bs.frame_len;
        let frames_num = bs.frames_num();
        let start = if is_s40xl { 0xff } else { 0xfe };
        let flen = frame_len.div_ceil(8);
        let pad = flen * 8 - frame_len;
        for fi in 0..frames_num {
            assert_eq!(data[pos], start);
            pos += 1;
            let fdata = &data[pos..pos + flen];
            pos += flen;
            for (i, &b) in fdata.iter().enumerate() {
                for bit in 0..8 {
                    let bv = (b >> bit & 1) != 0;
                    if is_s40xl {
                        let bi = match bit {
                            0 => {
                                if i == 0 {
                                    assert!(!bv);
                                    continue;
                                }
                                bit * flen + i - 1
                            }
                            7 => {
                                if i == 0 {
                                    assert!(bv);
                                    continue;
                                }
                                bit * flen + i - 2
                            }
                            _ => bit * flen + i - 1,
                        };
                        assert!(bi < frame_len);
                        bs.set_bit(BitPos { frame: fi, bit: bi }, bv);
                    } else {
                        if bit < (8 - pad) {
                            let bi = bit * flen + i;
                            bs.set_bit(BitPos { frame: fi, bit: bi }, bv);
                        } else {
                            if i == 0 {
                                assert!(bv);
                            } else {
                                let bi = bit * (flen - 1) + (8 - pad) + (i - 1);
                                bs.set_bit(BitPos { frame: fi, bit: bi }, bv);
                            }
                        }
                    }
                }
            }
            assert_eq!(data[pos..pos + 6], [0xd2, 0xff, 0xd2, 0xff, 0xff, 0xff]);
            pos += 6;
        }
        assert_eq!(data[pos..pos + 10], [0xff; 10]);
        pos += 10;
        assert_eq!(pos, data.len());
        assert_eq!(bitlen as usize, pos * 8 - 7);
    } else {
        let mut crc = Xc4000Crc::new();
        let data: &BitSlice<u8, Msb0> = BitSlice::from_slice(data);
        assert_eq!(data[..12], bits![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0]);
        let mut bitlen = 0;
        for j in 0..24 {
            if data[35 - j] {
                bitlen |= 1 << j;
            }
        }
        assert_eq!(data[36..40], bits![1, 1, 1, 1]);
        let mut pos = 40;
        let frame_len = bs.frame_len;
        let frames_num = bs.frames_num();
        let mut crc_enable = false;
        for fi in 0..frames_num {
            assert!(!data[pos]);
            if fi == 0 {
                crc.feed_bit(true);
            } else {
                crc.feed_bit(false);
            }
            pos += 1;
            let fdata = &data[pos..(pos + frame_len)];
            for (i, bit) in fdata.iter().enumerate() {
                bs.set_bit(BitPos { frame: fi, bit: i }, *bit);
                if fi == 0 && i < 2 {
                    // ??!?!?!?!?!??!
                    crc.feed_bit(fdata[0]);
                    if i == 1 {
                        crc_enable = !*bit;
                    }
                } else {
                    crc.feed_bit(*bit);
                }
            }
            pos += frame_len;
            let raw_crc = &data[pos..(pos + 4)];
            pos += 4;
            if crc_enable {
                for bit in raw_crc {
                    crc.feed_bit(*bit);
                }
                assert_eq!(crc.crc & 0xf, 0);
            } else {
                assert_eq!(raw_crc, bits![0, 1, 1, 0]);
            }
            if crc_enable && fi == frames_num - 1 {
                for i in (frame_len - 7)..frame_len {
                    bs.set_bit(BitPos { frame: fi, bit: i }, true);
                }
                assert_eq!(crc.crc & 0x7ff, 0);
            }
        }
        let post = &data[pos..(pos + 8)];
        assert_eq!(post, &bits![0, 1, 1, 1, 1, 1, 1, 1]);
        pos += 8;
        while !pos.is_multiple_of(8) {
            assert!(data[pos]);
            pos += 1;
        }
        assert_eq!(bitlen, pos + 1);
        let pad = &data[pos..(pos + 8)];
        assert_eq!(pad, &bits![1, 1, 1, 1, 1, 1, 1, 1]);
        pos += 8;
        assert_eq!(pos, data.len());
    }
}

struct Xc5200Crc {
    crc: u16,
}

impl Xc5200Crc {
    fn new() -> Self {
        Self { crc: 0 }
    }

    fn feed_byte(&mut self, b: u8) {
        for i in (0..8).rev() {
            if ((b >> i) & 1) == 0 {
                self.crc ^= 0x8000;
            }
            if (self.crc & 0x8000) != 0 {
                self.crc <<= 1;
                self.crc ^= 0x8005;
            } else {
                self.crc <<= 1;
            }
        }
    }
}

fn parse_xc5200_bitstream(bs: &mut Bitstream, data: &[u8]) {
    let mut crc = Xc5200Crc::new();
    assert_eq!(data[0], 0xff);
    assert_eq!(data[1], 0xf2);
    let bit_length = (data[2] as usize) << 16 | (data[3] as usize) << 8 | (data[4] as usize);
    assert_eq!(data[5], 0xff);
    if bit_length == data.len() * 8 - 7 {
        // OK
    } else if bit_length == data.len() * 8 - 3 {
        bs.lc_alignment_done = true;
    } else {
        panic!(
            "weird length {bit_length} [total {total}]",
            total = data.len() * 8
        );
    }
    let mut pos = 6;
    let frame_len = bs.frame_len;
    let mut crc_enable = false;
    let frames_num = bs.frames_num();
    let frame_bytes = frame_len.div_ceil(8);
    for fi in 0..frames_num {
        assert_eq!(data[pos], 0xfe);
        crc.feed_byte(data[pos]);
        pos += 1;
        for j in 0..(frame_bytes * 8) {
            let bit = ((data[pos + j / 8] << (j % 8)) & 0x80) != 0;
            if fi == 0 && j == 0 {
                crc_enable = bit;
            }
            if fi == frames_num - 1 && crc_enable && j >= frame_len - 12 {
                if j < frame_len {
                    bs.set_bit(BitPos { frame: fi, bit: j }, true);
                }
                if j < frame_bytes * 8 - 12 {
                    assert!(!bit);
                }
            } else if j < frame_len {
                bs.set_bit(BitPos { frame: fi, bit: j }, bit);
            } else {
                assert!(!bit);
            }
        }
        for &b in &data[pos..(pos + frame_bytes)] {
            crc.feed_byte(b);
        }
        pos += frame_bytes;
        let fcrc = data[pos] >> 4;
        if !crc_enable {
            assert_eq!(fcrc, 6);
        } else {
            assert_eq!(fcrc, (!crc.crc >> 12) as u8);
        }
        assert_eq!(data[pos] & 0xf, 0xf);
        assert_eq!(data[pos + 1], 0xff);
        assert_eq!(data[pos + 2], 0xff);
        assert_eq!(data[pos + 3], 0xff);
        for &b in &data[pos..(pos + 4)] {
            crc.feed_byte(b);
        }
        pos += 4;
    }
    if crc_enable {
        assert_eq!(crc.crc, 0);
    }
    assert_eq!(data[pos], 0xfe);
    pos += 1;
    for _ in 0..31 {
        assert_eq!(data[pos], 0xff);
        pos += 1;
    }
    assert_eq!(data.len(), pos);
}

impl ExpandedDevice<'_> {
    pub fn parse_bitstream(&self, data: &[u8]) -> Bitstream {
        let mut res = Bitstream::empty(self.frame_len, self.frames_num);
        match self.chip.kind {
            ChipKind::Xc2000 | ChipKind::Xc3000 | ChipKind::Xc3000A => {
                parse_xc2000_bitstream(&mut res, data)
            }
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => parse_xc4000_bitstream(&mut res, data, self.is_s40xl()),
            ChipKind::Xc5200 => parse_xc5200_bitstream(&mut res, data),
        }
        //
        res
    }
}
