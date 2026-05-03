use std::collections::HashMap;

use arrayref::array_ref;
use prjcombine_interconnect::dir::DirV;
use prjcombine_types::bitvec::BitVec;

use crate::{
    bitstream::{
        Bitstream, BitstreamMode, DieBitstream, GtzBitstream, Reg,
        packet::{Crc, Packet, PacketParser},
    },
    chip::ChipKind,
    expanded::{ExpandedDevice, FrameAddr, FrameMaskMode},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum State {
    None,
    Wcfg,
    Mfwr,
}

fn virtex4_far(addr: FrameAddr) -> u32 {
    let (row, bt) = if addr.region < 0 {
        ((-1 - addr.region) as u32, 1)
    } else {
        (addr.region as u32, 0)
    };
    addr.minor | addr.major << 6 | row << 14 | addr.typ << 19 | bt << 22
}

fn virtex5_far(addr: FrameAddr) -> u32 {
    let (row, bt) = if addr.region < 0 {
        ((-1 - addr.region) as u32, 1)
    } else {
        (addr.region as u32, 0)
    };
    addr.minor | addr.major << 7 | row << 15 | bt << 20 | addr.typ << 21
}

fn virtex7_far(addr: FrameAddr) -> u32 {
    let (row, bt) = if addr.region < 0 {
        ((-1 - addr.region) as u32, 1)
    } else {
        (addr.region as u32, 0)
    };
    addr.minor | addr.major << 7 | row << 17 | bt << 22 | addr.typ << 23
}

fn insert_virtex4_frame(bs: &mut DieBitstream, fi: usize, data: &[u8]) {
    let frame_len = bs.frame_len;
    let frame_words = frame_len.div_ceil(32);
    assert_eq!(data.len(), frame_words * 4);
    for i in 0..frame_words {
        let word = u32::from_be_bytes(*array_ref!(data, i * 4, 4));
        for j in 0..32 {
            bs.frame_data
                .set(bs.frame_len * fi + i * 32 + j, (word >> j & 1) != 0);
        }
    }
    if bs.frame_present[fi] {
        panic!("FRAME {fi} SET TWICE");
    }
    bs.frame_present.set(fi, true);
}

impl ExpandedDevice<'_> {
    fn parse_subbitstream(
        &self,
        bs: &mut Bitstream,
        data: &[u8],
        key: Option<[u8; 32]>,
        die_index: usize,
    ) {
        let die = self.die_order[die_index];
        let mut packets = PacketParser::new(bs.kind, data, key);
        let kind = bs.kind;
        let diebs = &mut bs.die[die];
        let far_dict: HashMap<_, _> = diebs
            .frame_info
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    match kind {
                        ChipKind::Virtex4 => virtex4_far(f.addr),
                        ChipKind::Virtex5 | ChipKind::Virtex6 => virtex5_far(f.addr),
                        ChipKind::Virtex7 => virtex7_far(f.addr),
                    },
                    i,
                )
            })
            .collect();

        let mut ctl0 = 0;
        let mut trim_regs = 0;

        if kind == ChipKind::Virtex4 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
            assert_eq!(packets.next(), Some(Packet::SyncWord));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::CmdRcrc));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            match packets.next() {
                Some(Packet::Cor0(val)) => {
                    diebs.regs.insert(Reg::Cor0, val);
                }
                p => panic!("expected cor0 got {p:?}"),
            }
            match packets.next() {
                Some(Packet::Idcode(val)) => {
                    diebs.regs.insert(Reg::Idcode, val);
                }
                p => panic!("expected idcode got {p:?}"),
            }
            if matches!(packets.peek(), Some(Packet::Mask(_))) {
                let mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Ctl0(val)) => {
                        assert_eq!(mask & 0x40, 0x40);
                        assert_eq!(val & 0x40, 0x40);
                        ctl0 = (ctl0 & !mask) | (val & mask);
                    }
                    p => panic!("expected ctl0 got {p:?}"),
                }
                diebs.mode = BitstreamMode::Encrypt;
            }
            assert_eq!(packets.next(), Some(Packet::CmdSwitch));
            assert_eq!(packets.next(), Some(Packet::Nop));
            match packets.next() {
                Some(Packet::CmdNull) => {
                    // the following block is just missing in xqr4v?!?
                }
                Some(Packet::Mask(mask)) => {
                    match packets.next() {
                        Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                        p => panic!("expected ctl0 got {p:?}"),
                    }
                    for _ in 0..1150 {
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                    let mask = match packets.next() {
                        Some(Packet::Mask(val)) => val,
                        p => panic!("expected mask got {p:?}"),
                    };
                    match packets.next() {
                        Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                        p => panic!("expected ctl0 got {p:?}"),
                    }
                    assert_eq!(packets.next(), Some(Packet::CmdNull));
                }
                p => panic!("expected mask got {p:?}"),
            };
            assert_eq!(packets.next(), Some(Packet::Nop));
        } else {
            for _ in 0..8 {
                assert_eq!(packets.next(), Some(Packet::DummyWord));
            }
            assert_eq!(packets.next(), Some(Packet::WidthDetect));
            assert_eq!(packets.next(), Some(Packet::DummyWord));
            assert_eq!(packets.next(), Some(Packet::DummyWord));
            assert_eq!(packets.next(), Some(Packet::SyncWord));
            assert_eq!(packets.next(), Some(Packet::Nop));
            if matches!(packets.peek(), Some(Packet::Bspi(_))) {
                match packets.next() {
                    Some(Packet::Bspi(val)) => {
                        diebs.regs.insert(Reg::Bspi, val);
                        assert_eq!(packets.next(), Some(Packet::CmdBspiRead));
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                    p => panic!("expected bspi got {p:?}"),
                }
            }
            if matches!(packets.peek(), Some(Packet::Mask(_))) {
                let mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                    p => panic!("expected ctl0 got {p:?}"),
                }
                assert_eq!(ctl0 & 0x40, 0x40);
                if kind == ChipKind::Virtex7 {
                    match packets.next() {
                        Some(Packet::Cor1(val)) => {
                            diebs.regs.insert(Reg::Cor1, val);
                        }
                        p => panic!("expected cor1 got {p:?}"),
                    }
                    for _ in 0..14 {
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                } else {
                    for _ in 0..16 {
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                }
                match packets.next() {
                    Some(Packet::Cbc(val)) => diebs.iv = Some(val),
                    p => panic!("expected fdri got {p:?}"),
                };

                match packets.next() {
                    Some(Packet::Dwc(_)) => (),
                    p => panic!("expected dwc got {p:?}"),
                };

                diebs.regs.insert(Reg::FakeEncrypted, 1);
            }
            if kind == ChipKind::Virtex7 {
                match packets.next() {
                    Some(Packet::Timer(val)) => {
                        diebs.regs.insert(Reg::Timer, val);
                    }
                    p => panic!("expected timer got {p:?}"),
                }
            }
            match packets.next() {
                Some(Packet::WBStar(val)) => {
                    diebs.regs.insert(Reg::WbStar, val);
                }
                p => panic!("expected wbstar got {p:?}"),
            }
            assert_eq!(packets.next(), Some(Packet::CmdNull));
            assert_eq!(packets.next(), Some(Packet::Nop));
            while matches!(packets.peek(), Some(Packet::Mask(_))) {
                let _mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Trim1(val)) => {
                        diebs.regs.insert(Reg::Trim1, val);
                    }
                    Some(Packet::Trim(val)) => {
                        diebs.regs.insert(Reg::Trim0, val);
                    }
                    p => panic!("expected ctl2 or trim got {p:?}"),
                }
            }
            while matches!(packets.peek(), Some(Packet::Cor1(_))) {
                let cor1 = match packets.next() {
                    Some(Packet::Cor1(val)) => val,
                    p => panic!("expected cor1 got {p:?}"),
                };
                let _mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                let reg = match cor1 {
                    0x1000 => Reg::Trim0,
                    0x1400 => Reg::Trim1,
                    0x1800 => Reg::Trim2,
                    _ => panic!("unexpected cor1 val {cor1:#x}"),
                };
                match packets.next() {
                    Some(Packet::Trim(val)) => {
                        diebs.regs.insert(reg, val);
                    }
                    p => panic!("expected trim got {p:?}"),
                }
                trim_regs += 1;
            }
            if kind != ChipKind::Virtex5 && matches!(packets.peek(), Some(Packet::Testmode(_))) {
                match packets.next() {
                    Some(Packet::Testmode(val)) => {
                        diebs.regs.insert(Reg::Testmode, val);
                    }
                    p => panic!("expected testmode got {p:?}"),
                }
            }
            assert_eq!(packets.next(), Some(Packet::CmdRcrc));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            if kind != ChipKind::Virtex7 {
                match packets.next() {
                    Some(Packet::Timer(val)) => {
                        diebs.regs.insert(Reg::Timer, val);
                    }
                    p => panic!("expected timer got {p:?}"),
                }
            }
            match packets.next() {
                Some(Packet::RbCrcSw(val)) => {
                    diebs.regs.insert(Reg::RbCrcSw, val);
                }
                p => panic!("expected rbcrcsw got {p:?}"),
            }
            if kind == ChipKind::Virtex5 && matches!(packets.peek(), Some(Packet::Testmode(_))) {
                match packets.next() {
                    Some(Packet::Testmode(val)) => {
                        diebs.regs.insert(Reg::Testmode, val);
                    }
                    p => panic!("expected testmode got {p:?}"),
                }
            }
            match packets.next() {
                Some(Packet::Cor0(val)) => {
                    diebs.regs.insert(Reg::Cor0, val);
                }
                p => panic!("expected cor0 got {p:?}"),
            }
            match packets.next() {
                Some(Packet::Cor1(val)) => {
                    diebs.regs.insert(Reg::Cor1, val);
                }
                p => panic!("expected cor1 got {p:?}"),
            }
            match packets.next() {
                Some(Packet::Idcode(val)) => {
                    diebs.regs.insert(Reg::Idcode, val);
                }
                p => panic!("expected idcode got {p:?}"),
            }
            match packets.next() {
                Some(Packet::CmdSwitch) => (),
                Some(Packet::CmdFallEdge) => {
                    diebs.regs.insert(Reg::FakeFallEdge, 0);
                    assert_eq!(packets.next(), Some(Packet::CmdSwitch));
                }
                p => panic!("expected switch got {p:?}"),
            }
            assert_eq!(packets.next(), Some(Packet::Nop));
            let mask = match packets.next() {
                Some(Packet::Mask(val)) => val,
                p => panic!("expected mask got {p:?}"),
            };
            match packets.next() {
                Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                p => panic!("expected ctl0 got {p:?}"),
            }
            if kind == ChipKind::Virtex5 && (ctl0 & 0x40) != 0 {
                diebs.mode = BitstreamMode::Encrypt;
            }
            let _mask = match packets.next() {
                Some(Packet::Mask(val)) => val,
                p => panic!("expected mask got {p:?}"),
            };
            match packets.next() {
                Some(Packet::Ctl1(val)) => {
                    diebs.regs.insert(Reg::Ctl1, val);
                }
                p => panic!("expected ctl1 got {p:?}"),
            }
            for _ in 0..8 {
                assert_eq!(packets.next(), Some(Packet::Nop));
            }
        }

        // main loop
        let frame_bytes = diebs.frame_len / 8;
        if diebs.mode == BitstreamMode::Encrypt {
            let data;
            if kind == ChipKind::Virtex4 {
                assert_eq!(packets.next(), Some(Packet::Far(0)));
                assert_eq!(packets.next(), Some(Packet::CmdWcfg));
                assert_eq!(packets.next(), Some(Packet::Nop));
                let init_iv = match packets.next() {
                    Some(Packet::Cbc(val)) => val,
                    p => panic!("expected fdri got {p:?}"),
                };
                diebs.iv = Some(init_iv);
                let init_data = match packets.next() {
                    Some(Packet::EncFdri(val)) => val,
                    p => panic!("expected fdri got {p:?}"),
                };
                for _ in 0..9 {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
                assert_eq!(packets.next(), Some(Packet::Crc));
                assert_eq!(packets.next(), Some(Packet::CmdNull));
                assert_eq!(packets.next(), Some(Packet::Nop));
                assert_eq!(packets.next(), Some(Packet::Far(0)));
                assert_eq!(packets.next(), Some(Packet::CmdWcfg));
                assert_eq!(packets.next(), Some(Packet::Nop));
                assert_eq!(packets.next(), Some(Packet::Cbc(init_iv)));
                data = match packets.next() {
                    Some(Packet::EncFdri(val)) => val,
                    p => panic!("expected fdri got {p:?}"),
                };
                assert_eq!(init_data.len(), 12 * 41 * 4);
                assert!(data.starts_with(&init_data));
            } else {
                assert_eq!(packets.next(), Some(Packet::Far(0)));
                assert_eq!(packets.next(), Some(Packet::CmdWcfg));
                assert_eq!(packets.next(), Some(Packet::Nop));
                match packets.next() {
                    Some(Packet::Cbc(val)) => diebs.iv = Some(val),
                    p => panic!("expected fdri got {p:?}"),
                };
                data = match packets.next() {
                    Some(Packet::EncFdri(val)) => val,
                    p => panic!("expected fdri got {p:?}"),
                };
            }
            let frames = data.len() / frame_bytes;
            let tail = data.len() % frame_bytes;
            assert!(tail < 16);
            let tail = &data[(data.len() - tail)..];
            assert!(tail.iter().all(|&x| x == 0));
            let mut skip = 0;
            let mut fi = 0;
            for i in 0..(frames - 1) {
                if skip != 0 {
                    skip -= 1;
                    continue;
                }
                let pos = i * frame_bytes;
                insert_virtex4_frame(diebs, fi, &data[pos..pos + frame_bytes]);
                let cur_reg = diebs.frame_info[fi].addr.region;
                let cur_typ = diebs.frame_info[fi].addr.typ;
                fi += 1;
                if fi >= diebs.frame_info.len()
                    || diebs.frame_info[fi].addr.region != cur_reg
                    || diebs.frame_info[fi].addr.typ != cur_typ
                {
                    skip = 2;
                }
            }
            let last = data[(frames - 1) * frame_bytes..].to_vec();
            assert!(last.iter().all(|&x| x == 0));
            for _ in 0..9 {
                assert_eq!(packets.next(), Some(Packet::Nop));
            }
        } else {
            let mut fi = 0;
            let mut skip = 0;
            let mut state = State::None;
            let mut last_frame = None;
            let mut first_mf = false;
            loop {
                match packets.peek() {
                    Some(Packet::Far(far)) => {
                        packets.next();
                        fi = far_dict[&far];
                        skip = 0;
                    }
                    Some(Packet::CmdWcfg) => {
                        packets.next();
                        assert_eq!(packets.next(), Some(Packet::Nop));
                        if kind != ChipKind::Virtex7 {
                            assert_ne!(state, State::Wcfg);
                        }
                        state = State::Wcfg;
                        match packets.next() {
                            Some(Packet::Far(far)) => {
                                fi = far_dict[&far];
                                skip = 0;
                            }
                            p => panic!("expected far got {p:?}"),
                        }
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                    Some(Packet::CmdMfwr) => (),
                    _ => break,
                }
                match packets.peek() {
                    Some(Packet::CmdWcfg) => {
                        assert_ne!(state, State::Wcfg);
                        state = State::Wcfg;
                        packets.next();
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                    Some(Packet::CmdMfwr) => {
                        assert_ne!(state, State::Mfwr);
                        diebs.mode = BitstreamMode::Compress;
                        state = State::Mfwr;
                        packets.next();
                        let num_nops = match kind {
                            ChipKind::Virtex4 | ChipKind::Virtex5 => 1,
                            ChipKind::Virtex6 | ChipKind::Virtex7 => 12,
                        };
                        for _ in 0..num_nops {
                            assert_eq!(packets.next(), Some(Packet::Nop));
                        }
                        first_mf = true;
                    }
                    Some(Packet::Nop) => {
                        packets.next();
                    }
                    _ => (),
                }
                match (packets.peek(), state) {
                    (Some(Packet::CmdWcfg), State::Wcfg) => {
                        packets.next();
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                    (Some(Packet::Fdri(val)), State::Wcfg) => {
                        packets.next();
                        let frames = val.len() / frame_bytes;
                        assert_eq!(val.len() % frame_bytes, 0);
                        for i in 0..(frames - 1) {
                            if skip != 0 {
                                skip -= 1;
                                continue;
                            }
                            let pos = i * frame_bytes;
                            insert_virtex4_frame(diebs, fi, &val[pos..pos + frame_bytes]);
                            let cur_reg = diebs.frame_info[fi].addr.region;
                            let cur_typ = diebs.frame_info[fi].addr.typ;
                            fi += 1;
                            if fi >= diebs.frame_info.len()
                                || diebs.frame_info[fi].addr.region != cur_reg
                                || diebs.frame_info[fi].addr.typ != cur_typ
                            {
                                skip = 2;
                            }
                        }
                        let last = val[(frames - 1) * frame_bytes..].to_vec();
                        last_frame = Some(last);
                    }
                    (Some(Packet::Mfwr(mf)), State::Mfwr) => {
                        assert_eq!(
                            mf,
                            match kind {
                                ChipKind::Virtex4 => 2,
                                ChipKind::Virtex5 | ChipKind::Virtex6 =>
                                    if diebs.frame_info[fi].addr.typ == 1 {
                                        6
                                    } else {
                                        2
                                    },
                                ChipKind::Virtex7 =>
                                    if first_mf {
                                        8
                                    } else {
                                        4
                                    },
                            },
                        );
                        first_mf = false;
                        packets.next();
                        insert_virtex4_frame(diebs, fi, last_frame.as_ref().unwrap());
                        if kind == ChipKind::Virtex7 && diebs.frame_info[fi].addr.typ == 1 {
                            for _ in 0..8 {
                                assert_eq!(packets.next(), Some(Packet::Nop));
                            }
                        }
                    }
                    (p, _) => {
                        panic!("UNEXPECTED PACKET IN STATE {state:?}: {p:#x?}");
                    }
                }
            }
        }
        match packets.next() {
            Some(Packet::Crc) => (),
            Some(Packet::CmdRcrc) => {
                diebs.regs.insert(Reg::FakeIgnoreCrc, 1);
            }
            p => panic!("expected CRC or RCRC got {p:?}"),
        }
        if matches!(kind, ChipKind::Virtex6 | ChipKind::Virtex7) {
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdDGHigh));
        if kind != ChipKind::Virtex4 && diebs.mode == BitstreamMode::Compress {
            let _mask = match packets.next() {
                Some(Packet::Mask(val)) => val,
                p => panic!("expected mask got {p:?}"),
            };
            match packets.next() {
                Some(Packet::Ctl1(val)) => {
                    diebs.regs.insert(Reg::Ctl1, val);
                }
                p => panic!("expected ctl1 got {p:?}"),
            }
        }
        for _ in 0..100 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        if matches!(kind, ChipKind::Virtex4 | ChipKind::Virtex5) {
            assert_eq!(packets.next(), Some(Packet::CmdGRestore));
        }
        match kind {
            ChipKind::Virtex4 => {
                assert_eq!(packets.next(), Some(Packet::Nop));
                assert_eq!(packets.next(), Some(Packet::CmdNull));
                assert_eq!(packets.next(), Some(Packet::Nop));
                let _final_far = match packets.next() {
                    Some(Packet::Far(val)) => val,
                    p => panic!("expected far got {p:?}"),
                };
                assert_eq!(packets.next(), Some(Packet::CmdStart));
                assert_eq!(packets.next(), Some(Packet::Nop));
                let mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                    p => panic!("expected ctl0 got {p:?}"),
                }
                assert_eq!(packets.next(), Some(Packet::Crc));
                assert_eq!(packets.next(), Some(Packet::CmdDesynch));
                for _ in 0..16 {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
            }
            ChipKind::Virtex5 | ChipKind::Virtex6 | ChipKind::Virtex7 => {
                if kind == ChipKind::Virtex5 {
                    for _ in 0..30 {
                        assert_eq!(packets.next(), Some(Packet::Nop));
                    }
                }
                assert_eq!(packets.next(), Some(Packet::CmdStart));
                assert_eq!(packets.next(), Some(Packet::Nop));
                let _final_far = match packets.next() {
                    Some(Packet::Far(val)) => val,
                    p => panic!("expected far got {p:?}"),
                };
                let mask = match packets.next() {
                    Some(Packet::Mask(val)) => val,
                    p => panic!("expected mask got {p:?}"),
                };
                match packets.next() {
                    Some(Packet::Ctl0(val)) => ctl0 = (ctl0 & !mask) | (val & mask),
                    p => panic!("expected ctl0 got {p:?}"),
                }
                if diebs.regs.contains_key(&Reg::FakeIgnoreCrc) {
                    assert_eq!(packets.next(), Some(Packet::CmdRcrc));
                } else {
                    assert_eq!(packets.next(), Some(Packet::Crc));
                }
                if matches!(kind, ChipKind::Virtex6 | ChipKind::Virtex7) {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
                if !diebs.regs.contains_key(&Reg::FakeEncrypted) {
                    assert_eq!(packets.next(), Some(Packet::CmdDesynch));
                }
                let mut num_nops = match kind {
                    ChipKind::Virtex5 => 61,
                    ChipKind::Virtex6 | ChipKind::Virtex7 => 400,
                    _ => unreachable!(),
                };
                if diebs.regs.contains_key(&Reg::FakeEncrypted) {
                    num_nops += 2; // desync
                    num_nops -= 27; // mask+ctl+cbc+dwc
                    num_nops -= 0x10; // encrypted header
                    num_nops -= 0x78; // encrypted trailer
                }
                if diebs.regs.contains_key(&Reg::Trim1) {
                    num_nops -= 4;
                }
                if trim_regs == 0 && diebs.regs.contains_key(&Reg::Trim0) {
                    num_nops -= 4;
                }
                if diebs.regs.contains_key(&Reg::Testmode) {
                    num_nops -= 2;
                }
                if diebs.regs.contains_key(&Reg::Bspi) {
                    num_nops -= 5;
                }
                if diebs.regs.contains_key(&Reg::FakeFallEdge) {
                    num_nops -= 2;
                }
                num_nops -= 6 * trim_regs;
                for _ in 0..num_nops {
                    assert_eq!(packets.next(), Some(Packet::Nop));
                }
            }
        }
        diebs.regs.insert(Reg::Ctl0, ctl0);
        if die_index != self.die_order.len() - 1 {
            packets.desync();
            assert_eq!(packets.next(), Some(Packet::SyncWord));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::CmdShutdown));
            assert_eq!(packets.next(), Some(Packet::CmdRcrc));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            let subdata = match packets.next() {
                Some(Packet::Bout(data)) => data,
                p => panic!("expected bout got {p:?}"),
            };
            self.parse_subbitstream(bs, &subdata, key, die_index + 1);
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::CmdStart));
            assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        }
        loop {
            match packets.next() {
                Some(Packet::Nop) => (),
                Some(Packet::DummyWord) => break,
                None => return,
                p => panic!("expected end got {p:?}"),
            }
        }
        assert_eq!(die_index, 0);
        let interposer = self.interposer.unwrap();
        assert!(interposer.gtz_s || interposer.gtz_n);
        let loader = core::mem::replace(bs, self.empty_bitstream());
        bs.gtz_loader = Some(Box::new(loader));
        for _ in 0..7 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::WidthDetect));
        for _ in 0..2 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        assert_eq!(packets.next(), Some(Packet::Nop));
        loop {
            let pkt = packets.next().unwrap();
            if let Packet::Axss(data) = pkt {
                let gtz = parse_gtz_bitstream(&data);
                if interposer.gtz_s && !bs.gtz.contains_key(&DirV::S) {
                    bs.gtz.insert(DirV::S, gtz);
                } else if interposer.gtz_n && !bs.gtz.contains_key(&DirV::N) {
                    bs.gtz.insert(DirV::N, gtz);
                } else {
                    panic!("ummm too many GTZ?");
                }
            } else {
                assert_eq!(pkt, Packet::Crc);
                break;
            }
        }
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        self.consume_virtex4_shutdown(&mut packets, die_index);
        self.consume_virtex4_aghigh(&mut packets, die_index);
        for _ in 0..100 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        self.parse_subbitstream(bs, &data[packets.pos()..], key, die_index)
    }

    fn consume_virtex4_shutdown(&self, packets: &mut PacketParser, die_index: usize) {
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::WidthDetect));
        for _ in 0..2 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdShutdown));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdRcrc));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        packets.desync();
        if die_index != self.die_order.len() - 1 {
            assert_eq!(packets.next(), Some(Packet::SyncWord));
            assert_eq!(packets.next(), Some(Packet::Nop));
            let subdata = match packets.next() {
                Some(Packet::Bout(data)) => data,
                p => panic!("expected bout got {p:?}"),
            };
            let mut subpackets = PacketParser::new(self.kind, &subdata, None);
            self.consume_virtex4_shutdown(&mut subpackets, die_index + 1);
            assert_eq!(subpackets.next(), None);
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::CmdDesynch));
            for _ in 0..8 {
                assert_eq!(packets.next(), Some(Packet::Nop));
            }
        }
    }

    fn consume_virtex4_aghigh(&self, packets: &mut PacketParser, die_index: usize) {
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::WidthDetect));
        for _ in 0..2 {
            assert_eq!(packets.next(), Some(Packet::DummyWord));
        }
        assert_eq!(packets.next(), Some(Packet::SyncWord));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdAGHigh));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::Nop));
        assert_eq!(packets.next(), Some(Packet::CmdDesynch));
        for _ in 0..8 {
            assert_eq!(packets.next(), Some(Packet::Nop));
        }
        packets.desync();
        if die_index != self.die_order.len() - 1 {
            assert_eq!(packets.next(), Some(Packet::SyncWord));
            assert_eq!(packets.next(), Some(Packet::Nop));
            let subdata = match packets.next() {
                Some(Packet::Bout(data)) => data,
                p => panic!("expected bout got {p:?}"),
            };
            let mut subpackets = PacketParser::new(self.kind, &subdata, None);
            self.consume_virtex4_aghigh(&mut subpackets, die_index + 1);
            assert_eq!(subpackets.next(), None);
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::Nop));
            assert_eq!(packets.next(), Some(Packet::CmdDesynch));
            for _ in 0..8 {
                assert_eq!(packets.next(), Some(Packet::Nop));
            }
        }
    }
}

fn parse_gtz_bitstream(data: &[u32]) -> GtzBitstream {
    let idcode = data[0];
    assert_eq!(idcode >> 28, 0);
    assert_eq!(data[1], 0x00010001);
    let data_len: usize = data[2].try_into().unwrap();
    let data_seg = &data[3..(data_len + 3)];
    let data_seg = data_seg.strip_suffix(&[0; 0x20]).unwrap();
    let data_crc = data_seg[data_seg.len() - 1];
    let data_seg = &data_seg[..data_seg.len() - 1];
    let mut crc = Crc::new();
    for &w in &data[..3 + data_len - 0x21] {
        crc.update(0, w);
    }
    assert_eq!(data_crc, crc.get());

    assert_eq!(data[3 + data_len], idcode | 1 << 28);
    assert_eq!(data[3 + data_len + 1], 0x00010001);
    let code_len: usize = data[3 + data_len + 2].try_into().unwrap();
    let code_seg = &data[3 + data_len + 3..3 + data_len + 3 + code_len];
    assert_eq!(3 + data_len + 3 + code_len, data.len());
    let code_seg = code_seg.strip_suffix(&[0; 0x20]).unwrap();
    let code_crc = code_seg[code_seg.len() - 1];
    let code_seg = &code_seg[..code_seg.len() - 1];
    let mut crc = Crc::new();
    for &w in &data[3 + data_len..3 + data_len + 3 + code_len - 0x21] {
        crc.update(0, w);
    }
    assert_eq!(code_crc, crc.get());
    GtzBitstream {
        idcode,
        data: data_seg.to_vec(),
        code: code_seg.to_vec(),
    }
}

fn check_virtex4_ecc(bs: &Bitstream) {
    for (die, dbs) in &bs.die {
        for (fi, present) in dbs.frame_present.iter().enumerate() {
            if !present {
                continue;
            }
            let finfo = &dbs.frame_info[fi];
            let mut ecc: u32 = 0;
            let mut recc: u32 = 0;
            let flip = finfo.addr.region < 0;
            for idx in 0..dbs.frame_len {
                if !dbs.get_bit(fi, idx) {
                    continue;
                }
                let mask = match idx {
                    0..0x280 if !flip => finfo.mask_mode[idx / 0x140],
                    0..0x280 if flip => finfo.mask_mode[3 - idx / 0x140],
                    0x280..0x28c => {
                        recc ^= 1 << (idx - 0x280);
                        continue;
                    }
                    0x28c..0x2a0 => FrameMaskMode::None,
                    0x2a0..0x520 if !flip => finfo.mask_mode[(idx - 0x20) / 0x140],
                    0x2a0..0x520 if flip => finfo.mask_mode[3 - (idx - 0x20) / 0x140],
                    _ => unreachable!(),
                };
                let idx = idx as u32;
                match mask {
                    FrameMaskMode::None => (),
                    FrameMaskMode::BramV4 => {
                        let eidx = if flip { 0x520 - 1 - idx } else { idx };
                        let eidx = if eidx < 0x280 { eidx } else { eidx - 0x20 };
                        let eidx = eidx % 0x140;
                        if matches!(
                            eidx,
                            8 | 12
                                | 14
                                | 19
                                | 21
                                | 26
                                | 27
                                | 32
                                | 35
                                | 39
                                | 41
                                | 46
                                | 48
                                | 52
                                | 55
                                | 59
                                | 61
                                | 66
                                | 68
                                | 72
                                | 74
                                | 79
                                | 81
                                | 86
                                | 88
                                | 92
                                | 95
                                | 99
                                | 101
                                | 106
                                | 108
                                | 112
                                | 114
                                | 119
                                | 121
                                | 126
                                | 200
                                | 204
                                | 207
                                | 211
                                | 213
                                | 218
                                | 220
                                | 224
                                | 227
                                | 231
                                | 233
                                | 237
                                | 240
                                | 244
                                | 247
                                | 251
                                | 253
                                | 258
                                | 260
                                | 264
                                | 266
                                | 271
                                | 273
                                | 277
                                | 280
                                | 284
                                | 287
                                | 291
                                | 293
                                | 298
                                | 300
                                | 304
                                | 306
                                | 311
                                | 313
                                | 318
                        ) {
                            continue;
                        }
                    }
                    FrameMaskMode::DrpV4 => {
                        let eidx = if flip { 0x520 - 1 - idx } else { idx };
                        let eidx = if eidx < 0x280 { eidx } else { eidx - 0x20 };
                        if matches!(eidx % 20, 1..17) {
                            let midx = eidx / 20 * 20 + 18;
                            let midx = if midx < 0x280 { midx } else { midx + 0x20 };
                            let midx = if flip { 0x520 - 1 - midx } else { midx };
                            if dbs.get_bit(fi, midx as usize) {
                                continue;
                            }
                        }
                    }
                    FrameMaskMode::All => continue,
                    _ => todo!(),
                }
                let code = if idx < 0x140 {
                    0x2c0 + idx
                } else {
                    0x420 + (idx - 0x140)
                };
                ecc ^= 0x800 | code;
            }
            for i in 0..11 {
                if (ecc & (1 << i)) != 0 {
                    ecc ^= 0x800;
                }
            }
            if ecc != recc {
                eprintln!(
                    "ECC MISMATCH at frame {die}.{ft}.{fr}.{fmaj}.{fmin}: computed {ecc:04x} found {recc:04x}",
                    ft = finfo.addr.typ,
                    fr = finfo.addr.region,
                    fmaj = finfo.addr.major,
                    fmin = finfo.addr.minor
                );
            }
        }
    }
}

fn check_virtex5_ecc(bs: &Bitstream) {
    for (die, dbs) in &bs.die {
        for (fi, present) in dbs.frame_present.iter().enumerate() {
            if !present {
                continue;
            }
            let finfo = &dbs.frame_info[fi];
            let mut ecc: u32 = 0;
            let mut recc: u32 = 0;
            for idx in 0..dbs.frame_len {
                if !dbs.get_bit(fi, idx) {
                    continue;
                }
                let mask = match idx {
                    0..0x280 => finfo.mask_mode[0],
                    0x280..0x28c => {
                        recc ^= 1 << (idx - 0x280);
                        continue;
                    }
                    0x28c..0x2a0 => FrameMaskMode::None,
                    0x2a0..0x520 => finfo.mask_mode[1],
                    _ => unreachable!(),
                };
                let idx = idx as u32;
                match mask {
                    FrameMaskMode::None => (),
                    FrameMaskMode::DrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) + cframe;
                        if dbs.get_bit(cfi, 0x280 + cbit) {
                            continue;
                        }
                    }
                    FrameMaskMode::All => continue,
                    _ => todo!(),
                }
                let code = if idx < 0x140 {
                    0x2c0 + idx
                } else {
                    0x420 + (idx - 0x140)
                };
                ecc ^= 0x800 | code;
            }
            for i in 0..11 {
                if (ecc & (1 << i)) != 0 {
                    ecc ^= 0x800;
                }
            }
            if ecc != recc {
                eprintln!(
                    "ECC MISMATCH at frame {die}.{ft}.{fr}.{fmaj}.{fmin}: computed {ecc:04x} found {recc:04x}",
                    ft = finfo.addr.typ,
                    fr = finfo.addr.region,
                    fmaj = finfo.addr.major,
                    fmin = finfo.addr.minor
                );
            }
        }
    }
}

fn check_virtex6_ecc(bs: &Bitstream) {
    for (die, dbs) in &bs.die {
        for (fi, present) in dbs.frame_present.iter().enumerate() {
            if !present {
                continue;
            }
            let finfo = &dbs.frame_info[fi];
            let mut ecc: u32 = 0;
            let mut recc: u32 = 0;
            for idx in 0..dbs.frame_len {
                if !dbs.get_bit(fi, idx) {
                    continue;
                }
                let mask = match idx {
                    0..0x500 => finfo.mask_mode[0],
                    0x500..0x50d => {
                        recc ^= 1 << (idx - 0x500);
                        continue;
                    }
                    0x50d..0x520 => FrameMaskMode::None,
                    0x520..0xa20 => finfo.mask_mode[1],
                    _ => unreachable!(),
                };
                let idx = idx as u32;
                match mask {
                    FrameMaskMode::None => (),
                    FrameMaskMode::DrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) + cframe;
                        if dbs.get_bit(cfi, 0x500 + cbit) {
                            continue;
                        }
                    }
                    FrameMaskMode::CmtDrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) + cframe;
                        if dbs.get_bit(cfi, 0x500 + cbit)
                            && !matches!(idx, 0..0x80 | 0x480..0x5a0 | 0x9a0..0xa20)
                        {
                            continue;
                        }
                    }
                    FrameMaskMode::All => continue,
                    _ => todo!(),
                }
                let code = if idx < 0x240 {
                    0x5c0 + idx
                } else {
                    0x820 + (idx - 0x240)
                };
                ecc ^= 0x1000 | code;
            }
            for i in 0..12 {
                if (ecc & (1 << i)) != 0 {
                    ecc ^= 0x1000;
                }
            }
            if ecc != recc {
                eprintln!(
                    "ECC MISMATCH at frame {die}.{ft}.{fr}.{fmaj}.{fmin}: computed {ecc:04x} found {recc:04x}",
                    ft = finfo.addr.typ,
                    fr = finfo.addr.region,
                    fmaj = finfo.addr.major,
                    fmin = finfo.addr.minor
                );
            }
        }
    }
}

fn check_virtex7_ecc(bs: &Bitstream) {
    for (die, dbs) in &bs.die {
        for (fi, present) in dbs.frame_present.iter().enumerate() {
            if !present {
                continue;
            }
            let finfo = &dbs.frame_info[fi];
            let mut ecc: u32 = 0;
            let mut recc: u32 = 0;
            for idx in 0..dbs.frame_len {
                if !dbs.get_bit(fi, idx) {
                    continue;
                }
                let mask = match idx {
                    0..0x640 => finfo.mask_mode[0],
                    0x640..0x64d => {
                        recc ^= 1 << (idx - 0x640);
                        continue;
                    }
                    0x64d..0x660 => FrameMaskMode::None,
                    0x660..0xca0 => finfo.mask_mode[1],
                    _ => unreachable!(),
                };
                let idx = idx as u32;
                match mask {
                    FrameMaskMode::None => (),
                    FrameMaskMode::DrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) + cframe;
                        if dbs.get_bit(cfi, 0x640 + cbit) {
                            continue;
                        }
                    }
                    FrameMaskMode::PcieLeftDrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) - 28 - 2 * 36 + cframe;
                        assert_eq!(dbs.frame_info[cfi].addr.minor, cframe as u32);
                        if dbs.get_bit(cfi, 0x640 + cbit) {
                            continue;
                        }
                    }
                    FrameMaskMode::CmtDrpHclk(cframe, cbit) => {
                        let cfi = fi - (finfo.addr.minor as usize) + cframe;
                        if dbs.get_bit(cfi, 0x640 + cbit) && matches!(idx, 0..0x600 | 0x6a0..0xca0)
                        {
                            continue;
                        }
                    }
                    FrameMaskMode::All => continue,
                    _ => todo!(),
                }
                let code = if idx < 0xe0 {
                    0x320 + idx
                } else if idx < 0x4c0 {
                    0x420 + (idx - 0xe0)
                } else {
                    0x820 + (idx - 0x4c0)
                };
                ecc ^= 0x1000 | code;
            }
            for i in 0..12 {
                if (ecc & (1 << i)) != 0 {
                    ecc ^= 0x1000;
                }
            }
            if ecc != recc {
                eprintln!(
                    "ECC MISMATCH at frame {die}.{ft}.{fr}.{fmaj}.{fmin}: computed {ecc:04x} found {recc:04x}",
                    ft = finfo.addr.typ,
                    fr = finfo.addr.region,
                    fmaj = finfo.addr.major,
                    fmin = finfo.addr.minor
                );
            }
        }
    }
}

impl ExpandedDevice<'_> {
    fn empty_bitstream(&self) -> Bitstream {
        Bitstream {
            kind: self.kind,
            die: self.frames.map_values(|frames| DieBitstream {
                regs: Default::default(),
                mode: BitstreamMode::Plain,
                iv: None,
                frame_len: self.kind.frame_len(),
                frame_data: BitVec::repeat(false, self.kind.frame_len() * frames.frame_info.len()),
                frame_info: frames.frame_info.clone(),
                frame_present: BitVec::repeat(false, frames.frame_info.len()),
            }),
            gtz: Default::default(),
            gtz_loader: None,
        }
    }

    pub fn parse_bitstream(&self, data: &[u8], key: Option<[u8; 32]>) -> Bitstream {
        let mut res = self.empty_bitstream();
        self.parse_subbitstream(&mut res, data, key, 0);
        match self.kind {
            ChipKind::Virtex4 => {
                check_virtex4_ecc(&res);
            }
            ChipKind::Virtex5 => {
                check_virtex5_ecc(&res);
            }
            ChipKind::Virtex6 => {
                check_virtex6_ecc(&res);
            }
            ChipKind::Virtex7 => {
                check_virtex7_ecc(&res);
            }
        }
        res
    }
}
