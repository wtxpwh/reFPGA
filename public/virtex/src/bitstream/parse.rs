use arrayref::array_ref;
use prjcombine_types::bitvec::BitVec;
use std::collections::HashMap;

use crate::{
    bitstream::{
        Bitstream, BitstreamMode, FrameAddr, Reg,
        packet::{DeviceKind, Packet, PacketParser},
    },
    expanded::ExpandedDevice,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum State {
    None,
    Wcfg,
    Mfwr,
}

fn virtex_far(addr: FrameAddr) -> u32 {
    addr.minor << 9 | addr.major << 17 | addr.typ << 25
}

fn insert_frame(bs: &mut Bitstream, fi: usize, data: &[u8]) {
    let frame_len = bs.frame_len;
    let frame_words = frame_len.div_ceil(32);
    assert_eq!(data.len(), (frame_words + 1) * 4);
    for i in 0..frame_words {
        let word = u32::from_be_bytes(*array_ref!(data, i * 4, 4));
        if i == frame_words - 1 {
            let pad = frame_words * 32 - frame_len;
            for j in pad..32 {
                bs.frame_data
                    .set(fi * frame_len + j - pad, (word >> j & 1) != 0);
            }
        } else {
            let tgt = frame_len - (i + 1) * 32;
            for j in 0..32 {
                bs.frame_data
                    .set(fi * frame_len + tgt + j, (word >> j & 1) != 0);
            }
        }
    }
    if bs.frame_present[fi] {
        panic!("FRAME {fi} SET TWICE");
    }
    bs.frame_present.set(fi, true);
}

impl ExpandedDevice<'_> {
    fn empty_bitstream(&self) -> Bitstream {
        Bitstream {
            regs: Default::default(),
            mode: BitstreamMode::Plain,
            frame_len: self.frame_len,
            frame_data: BitVec::repeat(false, self.frame_len * self.frame_info.len()),
            frame_info: self.frame_info.clone(),
            frame_present: BitVec::repeat(false, self.frame_info.len()),
        }
    }

    pub fn parse_bitstream(&self, data: &[u8]) -> Bitstream {
        let mut res = self.empty_bitstream();
        let mut packets = PacketParser::new(DeviceKind::Virtex, data, None);
        let far_dict: HashMap<_, _> = self
            .frame_info
            .iter()
            .enumerate()
            .map(|(i, &addr)| (virtex_far(addr), i))
            .collect();

        assert_eq!(packets.next(), Some(Packet::DummyWord));
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        if packets.peek() == Some(Packet::LoutDebug(0)) {
            packets.next();
            res.mode = BitstreamMode::Debug;
        }
        assert_eq!(packets.next(), Some(Packet::CmdRcrc));
        let frame_words = self.frame_len.div_ceil(32) + 1;
        let frame_bytes = frame_words * 4;
        let flr = (frame_words - 1) as u32;
        let mut early_dghigh = false;
        if packets.peek() == Some(Packet::CmdDGHigh) {
            packets.next();
            early_dghigh = true;
            res.regs.insert(Reg::FakeEarlyGhigh, 1);
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
                res.regs.insert(Reg::Cor0, val);
            }
            p => panic!("expected cor0 got {p:?}"),
        }
        // TODO: validate?
        let _mask = match packets.next() {
            Some(Packet::Mask(val)) => val,
            p => panic!("expected mask got {p:?}"),
        };
        match packets.next() {
            Some(Packet::CmdSwitch) => {
                res.regs.insert(Reg::FakeHasSwitch, 1);
            }
            Some(Packet::CmdNull) => (),
            p => panic!("expected switch or null got {p:?}"),
        }

        // main loop
        let mut fi = 0;
        let mut last_frame = None;
        if res.mode == BitstreamMode::Debug {
            assert_eq!(packets.next(), Some(Packet::Far(0)));
            assert_eq!(packets.next(), Some(Packet::CmdWcfg));
            loop {
                let val = match packets.next() {
                    Some(Packet::Fdri(val)) => {
                        assert_eq!(val.len(), frame_bytes);
                        val
                    }
                    Some(Packet::Crc) => {
                        fi -= 1;
                        break;
                    }
                    p => panic!("expected fdri got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Crc) => (),
                    Some(Packet::Far(far)) => {
                        assert_eq!(far_dict[&far], fi);
                        continue;
                    }
                    p => panic!("expected crc/far got {p:?}"),
                }
                if let Some(Packet::LoutDebug(far)) = packets.peek() {
                    assert_eq!(far_dict[&far], fi);
                    packets.next();
                    insert_frame(&mut res, fi, &val);
                    fi += 1;
                } else {
                    panic!("expected loutdebug got {p:?}", p = packets.next());
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
                        res.mode = BitstreamMode::Compress;
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
                            insert_frame(&mut res, fi, &val[pos..pos + frame_bytes]);
                            fi += 1;
                        }
                        let last = val[(frames - 1) * frame_bytes..].to_vec();
                        last_frame = Some(last);
                    }
                    (Some(Packet::Mfwr(2)), State::Mfwr) => {
                        packets.next();
                        insert_frame(&mut res, fi, last_frame.as_ref().unwrap());
                    }
                    (Some(Packet::Far(_)), _) => continue,
                    (p, _) => {
                        panic!("UNEXPECTED PACKET IN STATE {state:?}: {p:#x?}");
                    }
                }
            }
            assert_eq!(packets.next(), Some(Packet::Crc));
        }

        if !early_dghigh {
            assert_eq!(packets.next(), Some(Packet::CmdDGHigh));
        }

        match packets.next() {
            Some(Packet::Fdri(val)) => {
                assert_eq!(val.len(), frame_bytes);
                assert!(val.iter().all(|&x| x == 0));
            }
            p => panic!("expected fdri got {p:?}"),
        }
        if !res.frame_present[fi] {
            insert_frame(&mut res, fi, &last_frame.unwrap());
        }
        assert!(res.frame_present.all());

        assert_eq!(packets.next(), Some(Packet::CmdStart));
        match packets.next() {
            Some(Packet::Ctl0(val)) => {
                res.regs.insert(Reg::Ctl0, val);
            }
            p => panic!("expected ctl0 got {p:?}"),
        }
        assert_eq!(packets.next(), Some(Packet::Crc));
        for _ in 0..4 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), None);
        res
    }
}
