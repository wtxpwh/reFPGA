use std::collections::HashMap;

use arrayref::array_ref;
use prjcombine_types::bitvec::BitVec;
use prjcombine_virtex2::bitstream::packet_s3a::{DeviceKind, Packet, PacketParser};

use crate::{
    bitstream::{BRAM_FRAME_LEN, Bitstream, BitstreamMode, FRAME_LEN, FrameAddr, Reg},
    expanded::ExpandedDevice,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum State {
    None,
    Wcfg,
    Mfwr,
}

fn far(addr: FrameAddr) -> u32 {
    if addr.typ == 1 {
        // BRAM
        addr.minor << 14 | addr.major << 16 | addr.region << 24 | addr.typ << 28
    } else {
        addr.minor | addr.major << 16 | addr.region << 24 | addr.typ << 28
    }
}

fn insert_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    let frame_words = FRAME_LEN / 16;
    assert_eq!(data.len(), frame_words * 2);
    for i in 0..frame_words {
        let word = u16::from_be_bytes(*array_ref!(data, i * 2, 2));
        let tgt = FRAME_LEN - (i + 1) * 16;
        for j in 0..16 {
            bs.frame_data
                .set(fi * FRAME_LEN + tgt + j, (word >> j & 1) != 0);
        }
    }
    if bs.frame_present[fi] {
        panic!("FRAME {fi} SET TWICE");
    }
    bs.frame_present.set(fi, true);
}

fn insert_bram_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    let frame_words = BRAM_FRAME_LEN / 16;
    assert_eq!(data.len(), frame_words * 2);
    for i in 0..frame_words {
        let word = u16::from_be_bytes(*array_ref!(data, i * 2, 2));
        let tgt = BRAM_FRAME_LEN - (i + 1) * 16;
        for j in 0..16 {
            bs.bram_data
                .set(fi * BRAM_FRAME_LEN + tgt + j, (word >> j & 1) != 0);
        }
    }
    if bs.bram_frame_present[fi] {
        panic!("BRAM FRAME {fi} SET TWICE");
    }
    bs.bram_frame_present.set(fi, true);
}

fn insert_iob_frame(bs: &mut Bitstream, data: &[u8]) {
    let frame_len = bs.iob.len();
    assert_eq!(frame_len % 16, 0);
    let frame_words = frame_len / 16;
    assert_eq!(data.len(), frame_words * 2);
    for i in 0..frame_words {
        let word = u16::from_be_bytes(*array_ref!(data, i * 2, 2));
        let tgt = frame_len - (i + 1) * 16;
        for j in 0..16 {
            bs.iob.set(tgt + j, (word >> j & 1) != 0);
        }
    }
    if bs.iob_present {
        panic!("IOB FRAME SET TWICE");
    }
    bs.iob_present = true;
}

fn fixup_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    if !bs.frame_present[fi] {
        panic!("FIXUP ON NOT PRESENT FRAME {fi}");
    }
    let frame_words = FRAME_LEN / 16;
    assert_eq!(data.len(), frame_words * 2);
    for i in 0..frame_words {
        let word = u16::from_be_bytes(*array_ref!(data, i * 2, 2));
        let tgt = FRAME_LEN - (i + 1) * 16;
        for j in 0..16 {
            let bit = (word >> j & 1) != 0;
            if bs.frame_data[fi * FRAME_LEN + tgt + j] != bit {
                bs.frame_fixups.insert((fi, tgt + j), bit);
            }
        }
    }
}

impl ExpandedDevice<'_> {
    pub fn empty_bitstream(&self) -> Bitstream {
        Bitstream {
            regs: Default::default(),
            mode: BitstreamMode::Plain,
            iv: None,
            frame_data: BitVec::repeat(false, FRAME_LEN * self.frame_info.len()),
            frame_info: self.frame_info.clone(),
            frame_present: BitVec::repeat(false, self.frame_info.len()),
            bram_data: BitVec::repeat(false, BRAM_FRAME_LEN * self.bram_frame_info.len()),
            bram_frame_present: BitVec::repeat(false, self.bram_frame_info.len()),
            bram_frame_info: self.bram_frame_info.clone(),
            iob: BitVec::repeat(false, self.iob_frame_len),
            iob_present: false,
            frame_fixups: HashMap::new(),
        }
    }

    pub fn parse_bitstream(&self, data: &[u8], key: Option<[u8; 32]>) -> Bitstream {
        let mut bs = self.empty_bitstream();
        let mut packets = PacketParser::new(DeviceKind::Spartan6, data, key);
        let far_dict: HashMap<_, _> = bs
            .frame_info
            .iter()
            .enumerate()
            .map(|(i, &addr)| (far(addr), i))
            .collect();
        let bram_far_dict: HashMap<_, _> = bs
            .bram_frame_info
            .iter()
            .enumerate()
            .map(|(i, &addr)| (far(addr), i))
            .collect();

        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        assert_eq!(packets.next(), Some(Packet::CmdRcrc));
        assert_eq!(packets.next(), Some(Packet::Nop));
        let flr = (bs.iob.len() / 16) as u16;
        assert_eq!(packets.next(), Some(Packet::Flr(flr)));
        match packets.next() {
            Some(Packet::Cor1(val)) => {
                bs.regs.insert(Reg::Cor1, val.into());
            }
            p => panic!("expected cor1 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::Cor2(val)) => {
                bs.regs.insert(Reg::Cor2, val.into());
            }
            p => panic!("expected cor2 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::Idcode(val)) => {
                bs.regs.insert(Reg::Idcode, val);
            }
            p => panic!("expected idcode got {p:?}"),
        }
        let _mask = match packets.next() {
            Some(Packet::Mask(val)) => val,
            p => panic!("expected mask got {p:?}"),
        };
        match packets.next() {
            Some(Packet::Ctl0(val)) => {
                bs.regs.insert(Reg::Ctl0, val.into());
            }
            p => panic!("expected ctl0 got {p:?}"),
        }
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        match packets.next() {
            Some(Packet::Cbc(_)) => {
                assert_eq!(bs.regs[&Reg::Ctl0] & 0x40, 0x40);
                bs.mode = BitstreamMode::Encrypt;
            }
            Some(Packet::Nop) => {
                assert_eq!(bs.regs[&Reg::Ctl0] & 0x40, 0);
                for _ in 0..8 {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
            }
            p => panic!("expected cbc or nop got {p:?}"),
        }
        match packets.next() {
            Some(Packet::CclkFrequency(val)) => {
                bs.regs.insert(Reg::CclkFrequency, val.into());
            }
            p => panic!("expected cclk frequency got {p:?}"),
        }
        if let Some(Packet::CclkFrequency(val)) = packets.peek() {
            packets.next();
            bs.regs.insert(Reg::CclkFrequency, val.into());
            bs.regs.insert(Reg::FakeDoubleCclkFrequency, 1);
        }
        match packets.next() {
            Some(Packet::Powerdown(val)) => {
                bs.regs.insert(Reg::Powerdown, val.into());
            }
            p => panic!("expected powerdown got {p:?}"),
        }
        match packets.next() {
            Some(Packet::EyeMask(val)) => {
                bs.regs.insert(Reg::EyeMask, val.into());
            }
            p => panic!("expected eyemask got {p:?}"),
        }
        match packets.next() {
            Some(Packet::HcOpt(val)) => {
                bs.regs.insert(Reg::HcOpt, val.into());
            }
            p => panic!("expected hc opt got {p:?}"),
        }
        match packets.next() {
            Some(Packet::Timer(val)) => {
                bs.regs.insert(Reg::Timer, val.into());
            }
            p => panic!("expected timer got {p:?}"),
        }
        match packets.next() {
            Some(Packet::PuGwe(val)) => {
                bs.regs.insert(Reg::PuGwe, val.into());
            }
            p => panic!("expected pu gwe got {p:?}"),
        }
        match packets.next() {
            Some(Packet::PuGts(val)) => {
                bs.regs.insert(Reg::PuGts, val.into());
            }
            p => panic!("expected pu gts got {p:?}"),
        }
        match packets.next() {
            Some(Packet::Mode(val)) => {
                bs.regs.insert(Reg::Mode, val.into());
            }
            p => panic!("expected mode got {p:?}"),
        }
        match packets.next() {
            Some(Packet::General1(val)) => {
                bs.regs.insert(Reg::General1, val.into());
            }
            p => panic!("expected general1 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::General2(val)) => {
                bs.regs.insert(Reg::General2, val.into());
            }
            p => panic!("expected general2 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::General3(val)) => {
                bs.regs.insert(Reg::General3, val.into());
            }
            p => panic!("expected general3 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::General4(val)) => {
                bs.regs.insert(Reg::General4, val.into());
            }
            p => panic!("expected general4 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::General5(val)) => {
                bs.regs.insert(Reg::General5, val.into());
            }
            p => panic!("expected general5 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::SeuOpt(val)) => {
                bs.regs.insert(Reg::SeuOpt, val.into());
            }
            p => panic!("expected seuopt got {p:?}"),
        }
        match packets.next() {
            Some(Packet::RbCrcSw(val)) => {
                bs.regs.insert(Reg::RbCrcSw, val);
            }
            p => panic!("expected rbcrcsw got {p:?}"),
        }
        if let Some(Packet::Testmode(val)) = packets.peek() {
            packets.next();
            bs.regs.insert(Reg::Testmode, val.into());
        } else if !bs.regs.contains_key(&Reg::FakeDoubleCclkFrequency) {
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
        }

        // main loop
        #[derive(Debug, Copy, Clone)]
        enum Frame {
            None,
            Main(usize),
            Bram(usize),
            Iob,
        }
        let frame_bytes = FRAME_LEN / 8;
        let bram_frame_bytes = BRAM_FRAME_LEN / 8;
        let iob_frame_bytes = bs.iob.len() / 8;
        let mut frame = Frame::None;
        let mut skip = 0;
        let mut last_frame: Option<Vec<u8>> = None;
        let mut state = State::None;
        if bs.mode != BitstreamMode::Encrypt {
            loop {
                match packets.peek() {
                    Some(Packet::Far(far)) => {
                        frame = if let Some(&fi) = far_dict.get(&far) {
                            if bs.frame_present[fi] {
                                break;
                            }
                            Frame::Main(fi)
                        } else if let Some(&fi) = bram_far_dict.get(&far) {
                            Frame::Bram(fi)
                        } else if far == 0x20000000 {
                            Frame::Iob
                        } else {
                            panic!("weird FAR {far:08x}")
                        };
                        packets.next();
                        skip = 1;
                    }
                    Some(Packet::CmdWcfg) => {
                        packets.next();
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                    }
                    Some(Packet::CmdMfwr) => {
                        packets.next();
                        assert_ne!(state, State::Mfwr);
                        state = State::Mfwr;
                        for _ in 0..8 {
                            assert_eq!(packets.next(), Some(Packet::Nop));
                        }
                    }
                    Some(Packet::Mfwr(4)) => {
                        packets.next();
                        bs.mode = BitstreamMode::Compress;
                        assert_eq!(state, State::Mfwr);
                        let Frame::Main(fi) = frame else {
                            panic!("mfwr in weird frame {frame:?}");
                        };
                        insert_frame(&mut bs, fi, last_frame.as_ref().unwrap());
                    }
                    Some(Packet::Fdri(orig_data)) => {
                        let mut data = &orig_data[..];
                        packets.next();
                        while !data.is_empty() {
                            match frame {
                                Frame::Main(fi) => {
                                    let cur = &data[..frame_bytes];
                                    data = &data[frame_bytes..];
                                    if skip != 0 {
                                        skip -= 1;
                                        if fi == bs.frame_info.len() {
                                            frame = Frame::Bram(0);
                                        }
                                    } else {
                                        insert_frame(&mut bs, fi, last_frame.as_ref().unwrap());
                                        frame = Frame::Main(fi + 1);
                                        if fi == bs.frame_info.len() - 1
                                            || bs.frame_info[fi + 1].region
                                                != bs.frame_info[fi].region
                                        {
                                            skip = 2;
                                        }
                                    }
                                    last_frame = Some(cur.to_vec());
                                }
                                Frame::Bram(fi) => {
                                    let cur = &data[..bram_frame_bytes];
                                    data = &data[bram_frame_bytes..];
                                    insert_bram_frame(&mut bs, fi, cur);
                                    if fi == bs.bram_frame_info.len() - 1 {
                                        frame = Frame::Iob;
                                    } else {
                                        frame = Frame::Bram(fi + 1)
                                    }
                                }
                                Frame::Iob => {
                                    assert_eq!(data.len(), iob_frame_bytes + 2);
                                    insert_iob_frame(&mut bs, &data[..iob_frame_bytes]);
                                    data = &[];
                                    frame = Frame::None;
                                }
                                _ => panic!("fdri {l} with no frame", l = data.len()),
                            }
                        }
                    }
                    _ => break,
                }
            }
            if bs.mode != BitstreamMode::Compress {
                for _ in 0..24 {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
            }
        } else {
            assert_eq!(packets.next(), Some(Packet::Far(0)));
            assert_eq!(packets.next(), Some(Packet::CmdWcfg));
            match packets.next() {
                Some(Packet::EncFdri(orig_data)) => {
                    let num = orig_data.len() / 2;
                    let mut data = &orig_data[..];
                    frame = Frame::Main(0);
                    skip = 1;
                    while !data.is_empty() {
                        match frame {
                            Frame::Main(fi) => {
                                let cur = &data[..frame_bytes];
                                data = &data[frame_bytes..];
                                if skip != 0 {
                                    skip -= 1;
                                    if fi == bs.frame_info.len() {
                                        frame = Frame::Bram(0);
                                    }
                                } else {
                                    insert_frame(&mut bs, fi, last_frame.as_ref().unwrap());
                                    frame = Frame::Main(fi + 1);
                                    if fi == bs.frame_info.len() - 1
                                        || bs.frame_info[fi + 1].region != bs.frame_info[fi].region
                                    {
                                        skip = 2;
                                    }
                                }
                                last_frame = Some(cur.to_vec());
                            }
                            Frame::Bram(fi) => {
                                let cur = &data[..bram_frame_bytes];
                                data = &data[bram_frame_bytes..];
                                insert_bram_frame(&mut bs, fi, cur);
                                if fi == bs.bram_frame_info.len() - 1 {
                                    frame = Frame::Iob;
                                } else {
                                    frame = Frame::Bram(fi + 1)
                                }
                            }
                            Frame::Iob => {
                                assert_eq!(data.len(), iob_frame_bytes + 2);
                                insert_iob_frame(&mut bs, &data[..iob_frame_bytes]);
                                data = &[];
                                frame = Frame::None;
                            }
                            _ => panic!("fdri {l} with no frame", l = data.len()),
                        }
                    }
                    let mut real_num = num + 2;
                    while !real_num.is_multiple_of(8) {
                        real_num += 1;
                    }
                    let exp_nops = 24 - (real_num - num - 2);
                    for _ in 0..exp_nops {
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                }
                p => panic!("expected enc fdri got {p:?}"),
            }
        }

        let mut first = true;
        while let Some(Packet::Far(far)) = packets.peek() {
            packets.next();
            let Some(&fi) = far_dict.get(&far) else {
                panic!("weird fixup FAR {far:08x}");
            };
            if first {
                assert_eq!(packets.next(), Some(Packet::CmdWcfg));
                first = false;
            }
            let data = match packets.next() {
                Some(Packet::Fdri(val)) => val,
                p => panic!("expected fdri got {p:?}"),
            };
            let mut fi = fi;
            let frames = data.len() / frame_bytes;
            assert_eq!(data.len() % frame_bytes, 0);
            for i in 0..(frames - 1) {
                let pos = i * frame_bytes;
                fixup_frame(&mut bs, fi, &data[pos..pos + frame_bytes]);
                fi += 1;
            }
        }

        assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        assert_eq!(packets.next(), Some(Packet::CmdDGHigh));
        for _ in 0..4 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        assert_eq!(packets.next(), Some(Packet::CmdStart));
        let _mask = match packets.next() {
            Some(Packet::Mask(val)) => val,
            p => panic!("expected mask got {p:?}"),
        };
        match packets.next() {
            Some(Packet::Ctl0(val)) => {
                bs.regs.insert(Reg::Ctl0, val.into());
            }
            p => panic!("expected ctl0 got {p:?}"),
        }
        assert_eq!(packets.next(), Some(Packet::Crc));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..14 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), None);

        bs
    }
}
