use arrayref::array_ref;
use prjcombine_types::bitvec::BitVec;
use prjcombine_virtex::bitstream::KeyData;
use std::collections::HashMap;

use crate::{
    bitstream::{Bitstream, BitstreamMode, FrameAddr, Reg},
    expanded::ExpandedDevice,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum State {
    None,
    Wcfg,
    Mfwr,
}

fn virtex2_far(addr: FrameAddr) -> u32 {
    addr.minor << 9 | addr.major << 17 | addr.typ << 25
}

fn spartan3a_far(addr: FrameAddr) -> u32 {
    addr.minor | addr.major << 16 | addr.typ << 26
}

fn insert_virtex2_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    let frame_len = bs.frame_len;
    let frame_words = frame_len.div_ceil(32);
    assert_eq!(data.len(), frame_words * 4);
    for i in 0..frame_words {
        let word = u32::from_be_bytes(*array_ref!(data, i * 4, 4));
        let tgt = frame_len - (i + 1) * 32;
        for j in 0..32 {
            bs.frame_data
                .set(fi * frame_len + tgt + j, (word >> j & 1) != 0);
        }
    }
    if bs.frame_present[fi] {
        panic!("FRAME {fi} SET TWICE");
    }
    bs.frame_present.set(fi, true);
}

fn fixup_virtex2_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    if !bs.frame_present[fi] {
        panic!("FIXUP ON NOT PRESENT FRAME {fi}");
    }
    let frame_len = bs.frame_len;
    let frame_words = frame_len / 32;
    assert_eq!(data.len(), frame_words * 4);
    for i in 0..frame_words {
        let word = u32::from_be_bytes(*array_ref!(data, i * 4, 4));
        let tgt = frame_len - (i + 1) * 32;
        for j in 0..32 {
            let bit = (word >> j & 1) != 0;
            if bs.frame_data[fi * frame_len + tgt + j] != bit {
                bs.frame_fixups.insert((fi, tgt + j), bit);
            }
        }
    }
}

fn insert_spartan3a_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    let frame_len = bs.frame_len;
    assert_eq!(frame_len % 16, 0);
    let frame_words = frame_len / 16;
    assert_eq!(data.len(), frame_words * 2);
    for i in 0..frame_words {
        let word = u16::from_be_bytes(*array_ref!(data, i * 2, 2));
        let tgt = frame_len - (i + 1) * 16;
        for j in 0..16 {
            bs.frame_data
                .set(fi * frame_len + tgt + j, (word >> j & 1) != 0);
        }
    }
    if bs.frame_present[fi] {
        panic!("FRAME {fi} SET TWICE");
    }
    bs.frame_present.set(fi, true);
}

impl ExpandedDevice<'_> {
    fn parse_virtex2_bitstream(&self, bs: &mut Bitstream, data: &[u8], key: Option<KeyData>) {
        use crate::bitstream::packet_v2::{DeviceKind, Packet, PacketParser};

        let mut packets = PacketParser::new(DeviceKind::Virtex2, data, key);
        let far_dict: HashMap<_, _> = bs
            .frame_info
            .iter()
            .enumerate()
            .map(|(i, &addr)| (virtex2_far(addr), i))
            .collect();

        assert_eq!(packets.next(), Some(Packet::DummyWord));
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        if packets.peek() == Some(Packet::LoutDebug(0)) {
            packets.next();
            bs.mode = BitstreamMode::Debug;
        }
        assert_eq!(packets.next(), Some(Packet::CmdRcrc));
        let frame_words = bs.frame_len / 32;
        let frame_bytes = frame_words * 4;
        let flr = (frame_words - 1) as u32;
        let mut early_dghigh = false;
        if packets.peek() == Some(Packet::CmdDGHigh) {
            packets.next();
            early_dghigh = true;
            bs.regs.insert(Reg::FakeEarlyGhigh, 1);
            let mut nops = 0;
            while let Some(Packet::Nop) = packets.peek() {
                packets.next();
                nops += 1;
            }
            assert_eq!(nops, flr + 1);
        }
        assert_eq!(packets.next(), Some(Packet::Flr(flr)));
        match packets.next() {
            Some(Packet::Cor0(val)) => {
                bs.regs.insert(Reg::Cor0, val);
            }
            p => panic!("expected cor0 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::Idcode(val)) => {
                bs.regs.insert(Reg::Idcode, val);
            }
            p => panic!("expected idcode got {p:?}"),
        }
        // TODO: validate?
        let _mask = match packets.next() {
            Some(Packet::Mask(val)) => val,
            p => panic!("expected mask got {p:?}"),
        };
        match packets.next() {
            Some(Packet::CmdSwitch) => {
                bs.regs.insert(Reg::FakeHasSwitch, 1);
            }
            Some(Packet::CmdNull) => (),
            p => panic!("expected switch or null got {p:?}"),
        }

        // main loop
        let mut fi = 0;
        let mut last_frame = None;
        if bs.mode == BitstreamMode::Debug {
            assert_eq!(packets.next(), Some(Packet::Far(0)));
            assert_eq!(packets.next(), Some(Packet::CmdWcfg));
            loop {
                let val = match packets.next() {
                    Some(Packet::Fdri(val)) => {
                        assert_eq!(val.len(), frame_bytes);
                        val
                    }
                    p => panic!("expected fdri got {p:?}"),
                };
                if let Some(Packet::LoutDebug(far)) = packets.peek() {
                    assert_eq!(far_dict[&far], fi);
                    packets.next();
                    insert_virtex2_frame(bs, fi, &val);
                    fi += 1;
                } else {
                    break;
                }
            }
        } else {
            let mut state = State::None;
            loop {
                match packets.peek() {
                    Some(Packet::Far(far)) => {
                        packets.next();
                        fi = far_dict[&far];
                    }
                    Some(Packet::CmdWcfg) => {
                        packets.next();
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                        match packets.next() {
                            Some(Packet::Far(far)) => {
                                fi = far_dict[&far];
                            }
                            p => panic!("expected far got {p:?}"),
                        }
                    }
                    _ => break,
                }
                match packets.peek() {
                    Some(Packet::CmdWcfg) => {
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                        packets.next();
                    }
                    Some(Packet::CmdMfwr) => {
                        assert_ne!(state, State::Mfwr);
                        bs.mode = BitstreamMode::Compress;
                        state = State::Mfwr;
                        packets.next();
                    }
                    _ => (),
                }
                match (packets.peek(), state) {
                    (Some(Packet::Fdri(val)), State::Wcfg) => {
                        packets.next();
                        let frames = val.len() / frame_bytes;
                        assert_eq!(val.len() % frame_bytes, 0);
                        for i in 0..(frames - 1) {
                            let pos = i * frame_bytes;
                            insert_virtex2_frame(bs, fi, &val[pos..pos + frame_bytes]);
                            fi += 1;
                        }
                        let last = val[(frames - 1) * frame_bytes..].to_vec();
                        last_frame = Some(last);
                    }
                    (Some(Packet::Mfwr(2)), State::Mfwr) => {
                        packets.next();
                        insert_virtex2_frame(bs, fi, last_frame.as_ref().unwrap());
                    }
                    (Some(Packet::Far(_)), _) => continue,
                    (Some(Packet::Key(key)), State::Wcfg) => {
                        bs.mode = BitstreamMode::Encrypt;
                        bs.regs.insert(Reg::Key, key);
                        packets.next();
                        assert_eq!(packets.next(), Some(Packet::Nop));
                        match packets.next() {
                            Some(Packet::Cbc(iv)) => {
                                bs.iv = Some(iv);
                            }
                            p => panic!("expected iv got {p:?}"),
                        }
                        match packets.next() {
                            Some(Packet::EncFdri(val)) => {
                                let frames = val.len() / frame_bytes;
                                assert_eq!(val.len() % frame_bytes, 0);
                                for i in 0..(frames - 1) {
                                    let pos = i * frame_bytes;
                                    insert_virtex2_frame(bs, fi, &val[pos..pos + frame_bytes]);
                                    fi += 1;
                                }
                            }
                            p => panic!("expected fdri got {p:?}"),
                        }
                        break;
                    }
                    (p, _) => {
                        panic!("UNEXPECTED PACKET IN STATE {state:?}: {p:#x?}");
                    }
                }
            }
        }

        assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        if !early_dghigh {
            assert_eq!(packets.next(), Some(Packet::CmdDGHigh));
        }

        assert!(bs.frame_present.all());

        if !early_dghigh {
            let mut nops = 0;
            while let Some(Packet::Nop) = packets.peek() {
                packets.next();
                nops += 1;
            }

            if packets.peek() == Some(Packet::CmdWcfg) {
                bs.regs.insert(Reg::FakeFreezeDciNops, nops);
                packets.next();
                while let Some(Packet::Far(far)) = packets.peek() {
                    packets.next();
                    match packets.next() {
                        Some(Packet::Fdri(val)) => {
                            let frames = val.len() / frame_bytes;
                            assert_eq!(val.len() % frame_bytes, 0);
                            fi = far_dict[&far];
                            for i in 0..(frames - 1) {
                                let pos = i * frame_bytes;
                                fixup_virtex2_frame(bs, fi, &val[pos..pos + frame_bytes]);
                                fi += 1;
                            }
                        }
                        p => panic!("expected fdri got {p:?}"),
                    }
                }
            } else {
                assert_eq!(nops, flr + 1);
            }
        }

        if packets.peek() == Some(Packet::CmdGRestore) {
            bs.regs.insert(Reg::FakeDoubleGrestore, 1);
            packets.next();
        }

        assert_eq!(packets.next(), Some(Packet::CmdStart));
        match packets.next() {
            Some(Packet::Ctl0(val)) => {
                bs.regs.insert(Reg::Ctl0, val);
            }
            p => panic!("expected ctl0 got {p:?}"),
        }
        assert_eq!(packets.next(), Some(Packet::Crc));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..4 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), None);
    }

    fn parse_spartan3a_bitstream(&self, bs: &mut Bitstream, data: &[u8]) {
        use crate::bitstream::packet_s3a::{DeviceKind, Packet, PacketParser};
        let mut packets = PacketParser::new(DeviceKind::Spartan3A, data, None);
        let far_dict: HashMap<_, _> = bs
            .frame_info
            .iter()
            .enumerate()
            .map(|(i, &addr)| (spartan3a_far(addr), i))
            .collect();

        for _ in 0..16 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        if packets.peek() == Some(Packet::LoutDebug(0)) {
            packets.next();
            bs.mode = BitstreamMode::Debug;
        }
        assert_eq!(packets.next(), Some(Packet::CmdRcrc));
        assert_eq!(packets.next(), Some(Packet::Nop));
        match packets.next() {
            Some(Packet::Cor2(val)) => {
                bs.regs.insert(Reg::Cor2, val.into());
            }
            p => panic!("expected cor2 got {p:?}"),
        }
        match packets.next() {
            Some(Packet::CclkFrequency(val)) => {
                bs.regs.insert(Reg::CclkFrequency, val.into());
            }
            p => panic!("expected cclk frequency got {p:?}"),
        }
        let frame_bytes = bs.frame_len / 8;
        let flr = (bs.frame_len / 16 - 1) as u16;
        assert_eq!(packets.next(), Some(Packet::Flr(flr)));
        match packets.next() {
            Some(Packet::Cor1(val)) => {
                bs.regs.insert(Reg::Cor1, val.into());
            }
            p => panic!("expected cor1 got {p:?}"),
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
        match packets.next() {
            Some(Packet::Powerdown(val)) => {
                bs.regs.insert(Reg::Powerdown, val.into());
            }
            p => panic!("expected powerdown got {p:?}"),
        }
        match packets.next() {
            Some(Packet::HcOpt(val)) => {
                bs.regs.insert(Reg::HcOpt, val.into());
            }
            p => panic!("expected hc opt got {p:?}"),
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
            }
            Some(Packet::Nop) => {
                for _ in 0..5 {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
            }
            p => panic!("expected mode got {p:?}"),
        }
        match packets.next() {
            Some(Packet::SeuOpt(val)) => {
                bs.regs.insert(Reg::SeuOpt, val.into());
            }
            p => panic!("expected seuopt got {p:?}"),
        }
        match packets.next() {
            Some(Packet::RbCrcSw(val)) => {
                bs.regs.insert(Reg::RbCrcSw, val.into());
            }
            p => panic!("expected rbcrcsw got {p:?}"),
        }

        // main loop
        if bs.mode == BitstreamMode::Debug {
            assert_eq!(packets.next(), Some(Packet::Far(0)));
            assert_eq!(packets.next(), Some(Packet::CmdWcfg));
            let mut fi = 0;
            loop {
                let val = match packets.next() {
                    Some(Packet::Fdri(val)) => {
                        assert_eq!(val.len(), frame_bytes);
                        val
                    }
                    p => panic!("expected fdri got {p:?}"),
                };
                if let Some(Packet::LoutDebug(far)) = packets.peek() {
                    assert_eq!(far_dict[&far], fi);
                    packets.next();
                    insert_spartan3a_frame(bs, fi, &val);
                    fi += 1;
                } else {
                    assert!(val.iter().all(|&x| x == 0));
                    break;
                }
            }
        } else {
            let mut state = State::None;
            let mut mfwr_frame = None;
            loop {
                let far = match packets.peek() {
                    Some(Packet::Far(far)) => {
                        packets.next();
                        far
                    }
                    Some(Packet::CmdWcfg) => {
                        packets.next();
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                        match packets.next() {
                            Some(Packet::Far(far)) => far,
                            p => panic!("expected far got {p:?}"),
                        }
                    }
                    _ => break,
                };
                match packets.peek() {
                    Some(Packet::CmdWcfg) => {
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                        packets.next();
                    }
                    Some(Packet::CmdMfwr) => {
                        assert_ne!(state, State::Mfwr);
                        bs.mode = BitstreamMode::Compress;
                        state = State::Mfwr;
                        packets.next();
                    }
                    _ => (),
                }
                match (packets.next(), state) {
                    (Some(Packet::Fdri(val)), State::Wcfg) => {
                        let frames = val.len() / frame_bytes;
                        assert_eq!(val.len() % frame_bytes, 0);
                        if frames > 1 {
                            let fi = far_dict[&far];
                            for i in 0..(frames - 1) {
                                let pos = i * frame_bytes;
                                insert_spartan3a_frame(bs, fi + i, &val[pos..pos + frame_bytes]);
                            }
                        }
                        mfwr_frame = Some(val[(frames - 1) * frame_bytes..].to_vec());
                    }
                    (Some(Packet::Mfwr(4)), State::Mfwr) => {
                        let fi = far_dict[&far];
                        assert_ne!(bs.frame_info[fi].typ, 1);
                        insert_spartan3a_frame(bs, fi, mfwr_frame.as_ref().unwrap());
                    }
                    (Some(Packet::Mfwr(14)), State::Mfwr) => {
                        let fi = far_dict[&far];
                        assert_eq!(bs.frame_info[fi].typ, 1);
                        insert_spartan3a_frame(bs, fi, mfwr_frame.as_ref().unwrap());
                    }
                    p => {
                        panic!("UNEXPECTED PACKET IN STATE {state:?}: {p:#x?}");
                    }
                }
            }
        }

        assert!(bs.frame_present.all());

        assert_eq!(packets.next(), Some(Packet::Crc));
        assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        assert_eq!(packets.next(), Some(Packet::CmdDGHigh));
        for _ in 0..4 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), Some(Packet::CmdStart));
        let _mask2 = match packets.next() {
            Some(Packet::Mask(val)) => val,
            p => panic!("expected mask got {p:?}"),
        };
        let _ctl0_2 = match packets.next() {
            Some(Packet::Ctl0(val)) => val,
            p => panic!("expected ctl0 got {p:?}"),
        };
        assert_eq!(packets.next(), Some(Packet::Crc));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..16 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), None);
    }

    pub fn empty_bitstream(&self) -> Bitstream {
        Bitstream {
            regs: Default::default(),
            mode: BitstreamMode::Plain,
            iv: None,
            frame_len: self.frame_len,
            frame_data: BitVec::repeat(false, self.frame_len * self.frame_info.len()),
            frame_info: self.frame_info.clone(),
            frame_present: BitVec::repeat(false, self.frame_info.len()),
            frame_fixups: HashMap::new(),
        }
    }

    pub fn parse_bitstream(&self, data: &[u8], key: Option<KeyData>) -> Bitstream {
        let mut res = self.empty_bitstream();
        if self.chip.kind.is_spartan3a() {
            self.parse_spartan3a_bitstream(&mut res, data);
        } else {
            self.parse_virtex2_bitstream(&mut res, data, key);
        }
        res
    }
}
