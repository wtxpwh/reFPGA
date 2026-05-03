use aes::cipher::{KeyIvInit, inout::InOutBuf};
use arrayref::{array_mut_ref, array_ref};
use cbc::cipher::BlockDecryptMut;
use sha2::Digest;

use crate::chip::ChipKind;

#[derive(Debug, Clone)]
pub struct PacketParser<'a> {
    kind: ChipKind,
    key: Option<[u8; 32]>,
    iv: Option<[u8; 16]>,
    mask: u32,
    ctl0: u32,
    data: &'a [u8],
    pos: usize,
    sync: bool,
    last_reg: Option<u32>,
    crc: Crc,
    bypass_crc: bool,
    decrypted: Option<Vec<u8>>,
    orig_pos: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Packet {
    // unsynced
    DummyWord,
    WidthDetect,
    SyncWord,
    // synced
    Nop,
    CmdNull,
    CmdWcfg,
    CmdMfwr,
    CmdDGHigh,
    CmdAGHigh,
    CmdStart,
    CmdShutdown,
    CmdRcrc,
    CmdSwitch,
    CmdGRestore,
    CmdDesynch,
    CmdBspiRead,
    CmdFallEdge,
    Crc,
    Mask(u32),
    RbCrcSw(u32),
    Far(u32),
    Mfwr(usize),
    LoutDebug(u32),
    Cbc([u8; 16]),
    Cor0(u32),
    Cor1(u32),
    Ctl0(u32),
    Ctl1(u32),
    Trim1(u32),
    Bspi(u32),
    Idcode(u32),
    Timer(u32),
    WBStar(u32),
    Testmode(u32),
    Trim(u32),
    Dwc(u32),
    Axss(Vec<u32>),
    Fdri(Vec<u8>),
    Bout(Vec<u8>),
    EncFdri(Vec<u8>),
    BugFdri(Vec<u8>),
}

fn bitswap32(data: &mut [u8]) {
    for i in (0..data.len()).step_by(4) {
        let a = data[i].reverse_bits();
        let b = data[i + 1].reverse_bits();
        let c = data[i + 2].reverse_bits();
        let d = data[i + 3].reverse_bits();
        data[i + 3] = a;
        data[i + 2] = b;
        data[i + 1] = c;
        data[i] = d;
    }
}

#[derive(Debug, Clone)]
pub struct Crc {
    crc: u32,
}

impl Crc {
    pub fn new() -> Self {
        Self { crc: 0 }
    }

    pub fn reset(&mut self) {
        self.crc = 0;
    }

    pub fn update(&mut self, reg: u32, val: u32) {
        let poly = 0x82f63b78;
        for i in 0..32 {
            let bit = (val >> i & 1) ^ (self.crc & 1);
            self.crc >>= 1;
            if bit != 0 {
                self.crc ^= poly;
            }
        }
        for i in 0..5 {
            let bit = (reg >> i & 1) ^ (self.crc & 1);
            self.crc >>= 1;
            if bit != 0 {
                self.crc ^= poly;
            }
        }
    }

    pub fn get(&self) -> u32 {
        self.crc
    }

    pub fn set(&mut self, val: u32) {
        self.crc = val;
    }
}

impl<'a> PacketParser<'a> {
    pub fn new(kind: ChipKind, data: &'a [u8], key: Option<[u8; 32]>) -> Self {
        Self {
            kind,
            key,
            iv: None,
            data,
            pos: 0,
            sync: false,
            last_reg: None,
            crc: Crc::new(),
            bypass_crc: false,
            mask: 0,
            ctl0: 0,
            decrypted: None,
            orig_pos: 0,
        }
    }

    fn decrypt_v4(&self, data: &[u8]) -> Vec<u8> {
        let mut data = data.to_vec();
        assert_eq!(data.len() % 16, 0);
        bitswap32(&mut data);
        let mut iv = self.iv.unwrap();
        bitswap32(&mut iv);
        let key = self.key.unwrap();
        let mut cipher: cbc::Decryptor<aes::Aes256> =
            cbc::Decryptor::new((&key).into(), (&iv).into());
        let iodata: InOutBuf<_> = (&mut data[..]).into();
        let (mut blocks, tail) = iodata.into_chunks();
        assert!(tail.is_empty());
        cipher.decrypt_blocks_inout_mut(blocks.reborrow());
        bitswap32(&mut data);
        data
    }

    pub fn peek(&self) -> Option<Packet> {
        self.clone().next()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn desync(&mut self) {
        self.sync = false;
    }
}

impl Iterator for PacketParser<'_> {
    type Item = Packet;

    fn next(&mut self) -> Option<Packet> {
        let src_data = match self.decrypted {
            Some(ref data) => {
                if self.pos == data.len() {
                    self.pos = self.orig_pos;
                    self.decrypted = None;
                    self.data
                } else {
                    &data[..]
                }
            }
            None => self.data,
        };
        let is_v4 = true;
        if self.pos + 4 > src_data.len() {
            None
        } else {
            loop {
                let ph = u32::from_be_bytes(*array_ref!(src_data, self.pos, 4));
                self.pos += 4;
                break if self.sync {
                    let prev_crc = self.crc.get();
                    if ph == 0xffffffff {
                        // welp.
                        self.sync = false;
                        Some(Packet::DummyWord)
                    } else if ph == 0x00000000 {
                        // Not a valid packet, but a manifestation of a bitgen bug
                        // that emits broken debug bitstreams for Virtex4+.
                        // The packet header is missing, should be a long write to FDRI.
                        self.pos -= 4;
                        let dpos = self.pos;
                        let get_val =
                            |i: usize| u32::from_be_bytes(*array_ref!(src_data, dpos + i * 4, 4));
                        let mut i = 0;
                        while get_val(i) == 0 {
                            i += 1;
                        }
                        let num = i;
                        self.pos += num * 4;
                        let epos = self.pos;
                        for i in 0..num {
                            self.crc.update(2, get_val(i));
                        }
                        Some(Packet::BugFdri(src_data[dpos..epos].to_vec()))
                    } else if ph == 0x20000000 {
                        Some(Packet::Nop)
                    } else if (ph >> 27) == 6 {
                        let reg = ph >> 13 & 0x3fff;
                        let num = (ph & 0x1fff) as usize;
                        let dpos = self.pos;
                        self.last_reg = Some(reg);
                        self.pos += num * 4;
                        let epos = self.pos;
                        let get_val =
                            |i: usize| u32::from_be_bytes(*array_ref!(src_data, dpos + i * 4, 4));
                        if !matches!(reg, 8 | 0xf | 0x1e) {
                            for i in 0..num {
                                self.crc.update(reg, get_val(i));
                            }
                        }
                        match (reg, num) {
                            (0, 1) => {
                                let val = get_val(0);
                                let ecrc = if self.bypass_crc { 0xdefc } else { prev_crc };
                                if val != ecrc {
                                    println!("CRC MISMATCH {val:08x} {ecrc:08x}");
                                }
                                Some(Packet::Crc)
                            }
                            (1, 1) => Some(Packet::Far(get_val(0))),
                            (2, 0) => continue,
                            (2, _) => {
                                if matches!(self.kind, ChipKind::Virtex4 | ChipKind::Virtex5)
                                    && (self.ctl0 & 0x40) != 0
                                {
                                    // rollback CRC changes.
                                    self.crc.set(prev_crc);
                                    let data = self.decrypt_v4(&src_data[dpos..epos]);
                                    for i in 0..num {
                                        self.crc.update(
                                            reg,
                                            u32::from_be_bytes(*array_ref!(data, i * 4, 4)),
                                        );
                                    }
                                    Some(Packet::EncFdri(data))
                                } else {
                                    Some(Packet::Fdri(src_data[dpos..epos].to_vec()))
                                }
                            }
                            (4, 1) => match get_val(0) {
                                0 => Some(Packet::CmdNull),
                                1 => Some(Packet::CmdWcfg),
                                2 => Some(Packet::CmdMfwr),
                                3 => Some(Packet::CmdDGHigh),
                                5 => Some(Packet::CmdStart),
                                7 => {
                                    self.crc.reset();
                                    Some(Packet::CmdRcrc)
                                }
                                8 => Some(Packet::CmdAGHigh),
                                9 => Some(Packet::CmdSwitch),
                                10 => Some(Packet::CmdGRestore),
                                11 => Some(Packet::CmdShutdown),
                                13 => Some(Packet::CmdDesynch),
                                18 => Some(Packet::CmdBspiRead),
                                19 => Some(Packet::CmdFallEdge),
                                val => panic!("unk cmd: {val}"),
                            },
                            (5, 1) => {
                                let val = get_val(0);
                                self.ctl0 = (self.ctl0 & !self.mask) | (val & self.mask);
                                Some(Packet::Ctl0(val))
                            }
                            (6, 1) => {
                                let val = get_val(0);
                                self.mask = val;
                                Some(Packet::Mask(val))
                            }
                            (8, 1) => Some(Packet::LoutDebug(get_val(0))),
                            (9, 1) => {
                                let val = get_val(0);
                                match self.kind {
                                    ChipKind::Virtex4 | ChipKind::Virtex5 => {
                                        self.bypass_crc = (val & 1 << 28) != 0;
                                    }
                                    _ => (),
                                }
                                Some(Packet::Cor0(val))
                            }
                            (0xa, _) => {
                                assert!(src_data[dpos..epos].iter().all(|&x| x == 0));
                                Some(Packet::Mfwr(num))
                            }
                            (0xb, 4) => {
                                let data = *array_ref![src_data, dpos, 16];
                                self.iv = Some(data);
                                Some(Packet::Cbc(data))
                            }
                            (0xc, 1) => Some(Packet::Idcode(get_val(0))),
                            (0xd, 0) => continue,
                            (0xe, 1) => Some(Packet::Cor1(get_val(0))),
                            (0x10, 1) => Some(Packet::WBStar(get_val(0))),
                            (0x11, 1) => Some(Packet::Timer(get_val(0))),
                            (0x13, 1) => Some(Packet::RbCrcSw(get_val(0))),
                            (0x17, 1) => Some(Packet::Testmode(get_val(0))),
                            (0x18, 1) => Some(Packet::Ctl1(get_val(0))),
                            (0x1a, 1) => {
                                let val = get_val(0);
                                assert!(self.decrypted.is_none());
                                assert_eq!(val % 4, 0);
                                let cspos = self.pos;
                                self.pos += (val as usize) * 4;
                                let cepos = self.pos;
                                let decrypted = self.decrypt_v4(&src_data[cspos..cepos]);
                                let ipad = *array_ref!(decrypted, 0, 0x40);
                                let trailer_pos = decrypted.len() - 0x1e0;
                                let trailer = *array_ref!(decrypted, trailer_pos, 0x1e0);
                                assert_eq!(ipad[..0x20], [0x36; 0x20]);
                                let hmac_key: [_; 0x20] =
                                    core::array::from_fn(|i| ipad[0x20 + i] ^ 0x36);
                                let mut sha_pad = [0; 0x40];
                                sha_pad[0] = 0x80;
                                *array_mut_ref!(sha_pad, 0x38, 8) =
                                    ((trailer_pos as u64) * 8).to_be_bytes();
                                assert_eq!(trailer[..0x40], sha_pad);
                                assert_eq!(trailer[0x40..0x140], [0; 0x100]);
                                let opad = *array_ref!(trailer, 0x140, 0x40);
                                assert_eq!(opad[..0x20], [0x5c; 0x20]);
                                let hmac_key_tail: [_; 0x20] =
                                    core::array::from_fn(|i| opad[0x20 + i] ^ 0x5c);
                                assert_eq!(hmac_key, hmac_key_tail);
                                assert_eq!(*array_ref!(trailer, 0x180, 0x20), [0; 0x20]);
                                let mut outer_sha_pad = [0; 0x20];
                                outer_sha_pad[0] = 0x80;
                                outer_sha_pad[0x1e] = 0x3;
                                assert_eq!(*array_ref!(trailer, 0x1a0, 0x20), outer_sha_pad);
                                let mut hasher_inner = sha2::Sha256::new();
                                hasher_inner.update(&decrypted[..trailer_pos]);
                                let sha_inner: [u8; 0x20] = hasher_inner.finalize().into();
                                let mut hasher_outer = sha2::Sha256::new();
                                hasher_outer.update(opad);
                                hasher_outer.update(sha_inner);
                                let sha_outer: [u8; 0x20] = hasher_outer.finalize().into();
                                assert_eq!(*array_ref!(trailer, 0x1c0, 0x20), sha_outer);
                                self.decrypted = Some(decrypted[0x40..trailer_pos].to_vec());
                                self.orig_pos = self.pos;
                                self.pos = 0;
                                Some(Packet::Dwc(val))
                            }
                            (0x1b, 1) => Some(Packet::Trim(get_val(0))),
                            (0x1c, 1) => Some(Packet::Trim1(get_val(0))),
                            (0x1e, 0) => continue,
                            (0x1f, 1) => Some(Packet::Bspi(get_val(0))),
                            _ => panic!("unk write: {reg} times {num}"),
                        }
                    } else if (ph >> 27) == 7 {
                        let reg = ph >> 13 & 0x3fff;
                        let num = (ph & 0x1fff) as usize;
                        self.last_reg = Some(reg);
                        assert_eq!(reg, 2);
                        assert_eq!(num, 0);
                        continue;
                    } else if (ph >> 27) == 0xa {
                        let reg = self.last_reg.unwrap();
                        let num = (ph & 0x7ffffff) as usize;
                        let dpos = self.pos;
                        self.pos += num * 4;
                        let epos = self.pos;
                        let get_val =
                            |i: usize| u32::from_be_bytes(*array_ref!(src_data, dpos + i * 4, 4));
                        for i in 0..num {
                            self.crc.update(reg, get_val(i));
                        }
                        match reg {
                            2 => {
                                if matches!(self.kind, ChipKind::Virtex4 | ChipKind::Virtex5)
                                    && (self.ctl0 & 0x40) != 0
                                {
                                    // rollback CRC changes.
                                    self.crc.set(prev_crc);
                                    let data = self.decrypt_v4(&src_data[dpos..epos]);
                                    for i in 0..num {
                                        self.crc.update(
                                            reg,
                                            u32::from_be_bytes(*array_ref!(data, i * 4, 4)),
                                        );
                                    }
                                    Some(Packet::EncFdri(data))
                                } else {
                                    Some(Packet::Fdri(src_data[dpos..epos].to_vec()))
                                }
                            }
                            0xd if is_v4 => Some(Packet::Axss((0..num).map(get_val).collect())),
                            0x1e => {
                                self.crc.set(prev_crc);
                                Some(Packet::Bout(src_data[dpos..epos].to_vec()))
                            }
                            _ => panic!("unk long write: {reg} times {num}"),
                        }
                    } else {
                        panic!("unk word: {ph:08x}")
                    }
                } else {
                    match ph {
                        0xffffffff => Some(Packet::DummyWord),
                        0x000000bb => {
                            let w2 = u32::from_be_bytes(*array_ref!(src_data, self.pos, 4));
                            assert_eq!(w2, 0x11220044);
                            self.pos += 4;
                            Some(Packet::WidthDetect)
                        }
                        0xaa995566 => {
                            self.sync = true;
                            self.crc.reset();
                            Some(Packet::SyncWord)
                        }
                        _ => panic!("unk word while desyncd: {ph:08x}"),
                    }
                };
            }
        }
    }
}
