use crate::bitstream::{KeyData, KeySeq};
use arrayref::array_ref;
use cbc::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceKind {
    Virtex,
    Virtex2,
}

#[derive(Debug, Clone)]
pub struct PacketParser<'a> {
    kind: DeviceKind,
    key: Option<KeyData>,
    start_key: Option<usize>,
    iv: Option<[u8; 8]>,
    mask: u32,
    ctl0: u32,
    data: &'a [u8],
    pos: usize,
    sync: bool,
    last_reg: Option<u32>,
    crc: Crc,
    bypass_crc: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Packet {
    // unsynced
    DummyWord,
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
    Crc,
    Flr(u32),
    Mask(u32),
    Far(u32),
    Mfwr(usize),
    LoutDebug(u32),
    Key(u32),
    Cbc([u8; 8]),
    Cor0(u32),
    Ctl0(u32),
    Idcode(u32),
    Fdri(Vec<u8>),
    EncFdri(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Crc {
    kind: DeviceKind,
    crc: u32,
}

impl Crc {
    pub fn new(kind: DeviceKind) -> Self {
        Self { kind, crc: 0 }
    }

    pub fn reset(&mut self) {
        self.crc = 0;
    }

    pub fn update(&mut self, reg: u32, val: u32) {
        let poly = 0xa001;
        for i in 0..32 {
            let bit = (val >> i & 1) ^ (self.crc & 1);
            self.crc >>= 1;
            if bit != 0 {
                self.crc ^= poly;
            }
        }
        let rw = if self.kind == DeviceKind::Virtex {
            4
        } else {
            5
        };
        for i in 0..rw {
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
    pub fn new(kind: DeviceKind, data: &'a [u8], key: Option<KeyData>) -> Self {
        Self {
            kind,
            key,
            start_key: None,
            iv: None,
            data,
            pos: 0,
            sync: false,
            last_reg: None,
            crc: Crc::new(kind),
            bypass_crc: false,
            mask: 0,
            ctl0: 0,
        }
    }

    pub fn peek(&self) -> Option<Packet> {
        self.clone().next()
    }
}

impl Iterator for PacketParser<'_> {
    type Item = Packet;

    fn next(&mut self) -> Option<Packet> {
        if self.pos + 4 > self.data.len() {
            None
        } else {
            loop {
                let ph = u32::from_be_bytes(*array_ref!(self.data, self.pos, 4));
                self.pos += 4;
                break if self.sync {
                    let prev_crc = self.crc.get();
                    if ph == 0x00000000 && self.kind == DeviceKind::Virtex {
                        Some(Packet::Nop)
                    } else if ph == 0xffffffff {
                        // welp.
                        self.sync = false;
                        Some(Packet::DummyWord)
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
                            |i: usize| u32::from_be_bytes(*array_ref!(self.data, dpos + i * 4, 4));
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
                                if self.kind == DeviceKind::Virtex2 {
                                    let crc = u32::from_be_bytes(*array_ref!(self.data, epos, 4));
                                    let ecrc = if self.bypass_crc {
                                        0xdefc
                                    } else {
                                        self.crc.get()
                                    };
                                    if crc != ecrc {
                                        println!("AUTOCRC MISMATCH {crc:08x} {ecrc:08x}",);
                                    }
                                    self.pos += 4;
                                    self.crc.reset();
                                }
                                Some(Packet::Fdri(self.data[dpos..epos].to_vec()))
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
                                if self.kind == DeviceKind::Virtex2 {
                                    self.bypass_crc = (val & 1 << 29) != 0;
                                }
                                Some(Packet::Cor0(val))
                            }
                            (0xa, _) => {
                                assert!(self.data[dpos..epos].iter().all(|&x| x == 0));
                                Some(Packet::Mfwr(num))
                            }
                            (0xb, 1) => Some(Packet::Flr(get_val(0))),
                            (0xc, 1) => {
                                let val = get_val(0);
                                self.start_key = Some(val as usize);
                                Some(Packet::Key(val))
                            }
                            (0xd, 2) => {
                                let iv = core::array::from_fn(|i| {
                                    if i < 4 {
                                        self.data[dpos + i + 4]
                                    } else {
                                        self.data[dpos + i - 4]
                                    }
                                });
                                self.iv = Some(iv);
                                Some(Packet::Cbc(iv))
                            }
                            (0xe, 1) => Some(Packet::Idcode(get_val(0))),
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
                            |i: usize| u32::from_be_bytes(*array_ref!(self.data, dpos + i * 4, 4));
                        for i in 0..num {
                            self.crc.update(reg, get_val(i));
                        }
                        match reg {
                            2 => {
                                if self.kind == DeviceKind::Virtex2 {
                                    let crc = u32::from_be_bytes(*array_ref!(self.data, epos, 4));
                                    let ecrc = if self.bypass_crc {
                                        0xdefc
                                    } else {
                                        self.crc.get()
                                    };
                                    if crc != ecrc {
                                        println!("AUTOCRC MISMATCH {crc:08x} {ecrc:08x}");
                                    }
                                    self.pos += 4;
                                    self.crc.reset();
                                }
                                Some(Packet::Fdri(self.data[dpos..epos].to_vec()))
                            }
                            _ => panic!("unk long write: {reg} times {num}"),
                        }
                    } else if (ph >> 27) == 0xb {
                        let reg = self.last_reg.unwrap();
                        let num = (ph & 0x7ffffff) as usize;
                        let dpos = self.pos;
                        self.pos += num * 4;
                        let epos = self.pos;
                        let ct = &self.data[dpos..epos];
                        assert_eq!(num % 2, 0);
                        let mut pt = vec![];

                        let key = self.key.unwrap();
                        let start_key = self.start_key.unwrap();
                        let iv = self.iv.as_ref().unwrap();
                        let mut chain: [u8; 8] = *array_ref!(iv, 0, 8);
                        // what in the name of fuck.
                        chain[5] &= 0xfc;
                        chain[6] = 0;
                        chain[7] = 0;
                        let cipher = key.key.map(|k| {
                            let mut val: u64 = 0;
                            for bi in 0..7 {
                                val |= (k[6 - bi] as u64) << (8 * bi);
                            }
                            let mut ek: [u8; 8] = [0; 8];
                            for bi in 0..8 {
                                ek[7 - bi] = (((val >> (7 * bi)) & 0x7f) as u8) << 1;
                            }
                            des::Des::new(&ek.into())
                        });
                        let mut last_key = start_key;
                        loop {
                            if last_key == start_key {
                                assert!(matches!(
                                    key.keyseq[last_key],
                                    KeySeq::First | KeySeq::Single
                                ));
                            } else {
                                assert!(matches!(
                                    key.keyseq[last_key],
                                    KeySeq::Middle | KeySeq::Last
                                ));
                            }
                            if matches!(key.keyseq[last_key], KeySeq::Last | KeySeq::Single) {
                                break;
                            }
                            last_key += 1;
                        }
                        for i in 0..(num / 2) {
                            let ctb: [u8; 8] = core::array::from_fn(|j| ct[i * 8 + (j ^ 4)]);
                            let mut ptb = ctb;
                            for kidx in start_key..=last_key {
                                if (kidx - start_key).is_multiple_of(2) {
                                    cipher[kidx].decrypt_block((&mut ptb).into());
                                } else {
                                    cipher[kidx].encrypt_block((&mut ptb).into());
                                }
                            }
                            for j in 0..8 {
                                pt.push(ptb[j ^ 4] ^ chain[j ^ 4]);
                            }
                            chain = ctb;
                        }
                        let get_val = |i: usize| u32::from_be_bytes(*array_ref!(pt, i * 4, 4));
                        for i in 0..num {
                            self.crc.update(reg, get_val(i));
                        }
                        match reg {
                            2 => {
                                if self.kind == DeviceKind::Virtex2 {
                                    let crc = u32::from_be_bytes(*array_ref!(self.data, epos, 4));
                                    let ecrc = if self.bypass_crc {
                                        0xdefc
                                    } else {
                                        self.crc.get()
                                    };
                                    if crc != ecrc {
                                        println!("AUTOCRC MISMATCH {crc:08x} {ecrc:08x}");
                                    }
                                    self.pos += 4;
                                    self.crc.reset();
                                }
                                Some(Packet::EncFdri(pt))
                            }
                            _ => panic!("unk encrypted long write: {reg} times {num}"),
                        }
                    } else {
                        panic!("unk word: {ph:08x}")
                    }
                } else {
                    match ph {
                        0xffffffff => Some(Packet::DummyWord),
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
