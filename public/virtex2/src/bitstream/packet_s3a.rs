use aes::cipher::{BlockModeDecrypt, KeyIvInit, inout::InOutBuf};
use arrayref::array_ref;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DeviceKind {
    Spartan3A,
    Spartan6,
}

#[derive(Debug, Clone)]
pub struct PacketParser<'a> {
    kind: DeviceKind,
    key: Option<[u8; 32]>,
    iv: Option<[u8; 16]>,
    mask: u16,
    ctl0: u16,
    data: &'a [u8],
    pos: usize,
    sync: bool,
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
    Flr(u16),
    Mask(u16),
    RbCrcSw(u32),
    Far(u32),
    Mfwr(usize),
    LoutDebug(u32),
    Cbc([u8; 16]),
    Cor1(u16),
    Cor2(u16),
    Ctl0(u16),
    Idcode(u32),
    Timer(u16),
    Powerdown(u16),
    HcOpt(u16),
    Mode(u16),
    PuGwe(u16),
    PuGts(u16),
    CclkFrequency(u16),
    SeuOpt(u16),
    General1(u16),
    General2(u16),
    General3(u16),
    General4(u16),
    General5(u16),
    EyeMask(u16),
    Testmode(u16),
    Fdri(Vec<u8>),
    EncFdri(Vec<u8>),
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

    pub fn update(&mut self, reg: u16, val: u16) {
        // Yup, rotates once per 22 bits. Really.
        let data = (val as u32) | (reg as u32) << 16;
        self.crc <<= 1;
        if (self.crc & 0x400000) != 0 {
            self.crc ^= 0x409081;
        }
        self.crc ^= data;
    }

    pub fn get(&self) -> u32 {
        self.crc
    }

    pub fn set(&mut self, val: u32) {
        self.crc = val;
    }
}

impl Default for Crc {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PacketParser<'a> {
    pub fn new(kind: DeviceKind, data: &'a [u8], key: Option<[u8; 32]>) -> Self {
        Self {
            kind,
            key,
            iv: None,
            data,
            pos: 0,
            sync: false,
            crc: Crc::new(),
            bypass_crc: false,
            mask: 0,
            ctl0: 0,
        }
    }

    fn decrypt_v4(&self, data: &[u8]) -> Vec<u8> {
        let mut data = data.to_vec();
        assert_eq!(data.len() % 16, 0);
        bitswap32(&mut data);
        let mut iv: [u8; 16] = *array_ref!(self.iv.as_ref().unwrap(), 0, 16);
        bitswap32(&mut iv);
        let key = self.key.unwrap();
        let mut cipher: cbc::Decryptor<aes::Aes256> =
            cbc::Decryptor::new((&key).into(), (&iv).into());
        let iodata: InOutBuf<_> = (&mut data[..]).into();
        let (mut blocks, tail) = iodata.into_chunks();
        assert!(tail.is_empty());
        cipher.decrypt_blocks_inout(blocks.reborrow());
        bitswap32(&mut data);
        data
    }

    pub fn peek(&self) -> Option<Packet> {
        self.clone().next()
    }
}

impl Iterator for PacketParser<'_> {
    type Item = Packet;

    fn next(&mut self) -> Option<Packet> {
        let is_s6 = self.kind == DeviceKind::Spartan6;
        if self.pos + 2 > self.data.len() {
            None
        } else {
            let ph = u16::from_be_bytes(*array_ref!(self.data, self.pos, 2));
            self.pos += 2;
            if self.sync {
                let prev_crc = self.crc.get();
                if ph == 0x2000 {
                    Some(Packet::Nop)
                } else if (ph >> 11) == 6 {
                    let reg = ph >> 5 & 0x3f;
                    let num = (ph & 0x1f) as usize;
                    let dpos = self.pos;
                    self.pos += num * 2;
                    let epos = self.pos;
                    let get_val =
                        |i: usize| u16::from_be_bytes(*array_ref!(self.data, dpos + i * 2, 2));
                    let get_val32 =
                        |i: usize| u32::from_be_bytes(*array_ref!(self.data, dpos + i * 2, 4));
                    if !matches!(reg, 0 | 9 | 0x12) {
                        for i in 0..num {
                            self.crc.update(reg, get_val(i));
                        }
                    }
                    match (reg, num) {
                        (0, 2) => {
                            let val = get_val32(0);
                            let ecrc = if self.bypass_crc {
                                0x9876defc
                            } else {
                                self.crc.get()
                            };
                            if val != ecrc {
                                println!("CRC MISMATCH {val:08x} {ecrc:08x}");
                            }
                            Some(Packet::Crc)
                        }
                        (1, 2) => Some(Packet::Far(get_val32(0))),
                        (5, 1) => match get_val(0) {
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
                        (6, 1) => {
                            let val = get_val(0);
                            self.ctl0 = (self.ctl0 & !self.mask) | (val & self.mask);
                            Some(Packet::Ctl0(val))
                        }
                        (7, 1) => {
                            let val = get_val(0);
                            self.mask = val;
                            Some(Packet::Mask(val))
                        }
                        (9, 2) => Some(Packet::LoutDebug(get_val32(0))),
                        (0xa, 1) => {
                            let val = get_val(0);
                            self.bypass_crc = (val & 0x10) != 0;
                            Some(Packet::Cor1(val))
                        }
                        (0xb, 1) => Some(Packet::Cor2(get_val(0))),
                        (0xc, 1) => Some(Packet::Powerdown(get_val(0))),
                        (0xd, 1) => Some(Packet::Flr(get_val(0))),
                        (0xe, 2) => Some(Packet::Idcode(get_val32(0))),
                        (0xf, 1) if is_s6 => Some(Packet::Timer(get_val(0))),
                        (0x10, 1) => Some(Packet::HcOpt(get_val(0))),
                        (0x11, 1) => Some(Packet::Testmode(get_val(0))),
                        (0x13, 1) => Some(Packet::General1(get_val(0))),
                        (0x14, 1) => Some(Packet::General2(get_val(0))),
                        (0x15, 1) if !is_s6 => Some(Packet::Mode(get_val(0))),
                        (0x16, 1) if !is_s6 => Some(Packet::PuGwe(get_val(0))),
                        (0x17, 1) if !is_s6 => Some(Packet::PuGts(get_val(0))),
                        (0x18, _) if !is_s6 => {
                            assert!(self.data[dpos..epos].iter().all(|&x| x == 0));
                            Some(Packet::Mfwr(num))
                        }
                        (0x19, 1) if !is_s6 => Some(Packet::CclkFrequency(get_val(0))),
                        (0x1a, 1) if !is_s6 => Some(Packet::SeuOpt(get_val(0))),
                        (0x1b, 2) if !is_s6 => Some(Packet::RbCrcSw(get_val32(0))),
                        (0x15, 1) if is_s6 => Some(Packet::General3(get_val(0))),
                        (0x16, 1) if is_s6 => Some(Packet::General4(get_val(0))),
                        (0x17, 1) if is_s6 => Some(Packet::General5(get_val(0))),
                        (0x18, 1) if is_s6 => Some(Packet::Mode(get_val(0))),
                        (0x19, 1) if is_s6 => Some(Packet::PuGwe(get_val(0))),
                        (0x1a, 1) if is_s6 => Some(Packet::PuGts(get_val(0))),
                        (0x1b, _) if is_s6 => {
                            assert!(self.data[dpos..epos].iter().all(|&x| x == 0));
                            Some(Packet::Mfwr(num))
                        }
                        (0x1c, 1) if is_s6 => Some(Packet::CclkFrequency(get_val(0))),
                        (0x1d, 1) if is_s6 => Some(Packet::SeuOpt(get_val(0))),
                        (0x1e, 2) if is_s6 => Some(Packet::RbCrcSw(get_val32(0))),
                        (0x21, 1) if is_s6 => Some(Packet::EyeMask(get_val(0))),
                        (0x22, 8) if is_s6 => {
                            let data = *array_ref![self.data, dpos, 16];
                            self.iv = Some(data);
                            Some(Packet::Cbc(data))
                        }
                        _ => panic!("unk write: {reg} times {num}"),
                    }
                } else if (ph >> 11) == 0xa {
                    let reg = ph >> 5 & 0x3f;
                    let num = u32::from_be_bytes(*array_ref!(self.data, self.pos, 4)) as usize;
                    self.pos += 4;
                    let dpos = self.pos;
                    self.pos += num * 2;
                    let epos = self.pos;
                    let get_val =
                        |i: usize| u16::from_be_bytes(*array_ref!(self.data, dpos + i * 2, 2));
                    for i in 0..num {
                        self.crc.update(reg, get_val(i));
                    }
                    match reg {
                        3 => {
                            if self.kind == DeviceKind::Spartan6 && (self.ctl0 & 0x40) != 0 {
                                // rollback CRC changes.
                                self.crc.set(prev_crc);
                                let mut real_num = num + 2;
                                while !real_num.is_multiple_of(8) {
                                    real_num += 1;
                                }
                                let epos = dpos + real_num * 2;
                                self.pos = epos;
                                let data = self.decrypt_v4(&self.data[dpos..epos]);
                                for i in 0..num {
                                    self.crc.update(
                                        reg,
                                        u16::from_be_bytes(*array_ref!(data, i * 2, 2)),
                                    );
                                }
                                let crc = u32::from_be_bytes(*array_ref!(data, num * 2, 4));
                                let ecrc = if self.bypass_crc {
                                    0x9876defc
                                } else {
                                    self.crc.get()
                                };
                                if crc != ecrc {
                                    println!("AUTOCRC MISMATCH {crc:08x} {ecrc:08x}");
                                }
                                for i in (num + 2)..real_num {
                                    assert_eq!(
                                        u16::from_be_bytes(*array_ref!(data, i * 2, 2)),
                                        0x2000
                                    );
                                }
                                Some(Packet::EncFdri(data[..num * 2].to_vec()))
                            } else {
                                if self.kind == DeviceKind::Spartan6 {
                                    let crc = u32::from_be_bytes(*array_ref!(self.data, epos, 4));
                                    let ecrc = if self.bypass_crc {
                                        0x9876defc
                                    } else {
                                        self.crc.get()
                                    };
                                    if crc != ecrc {
                                        println!("AUTOCRC MISMATCH {crc:08x} {ecrc:08x}");
                                    }
                                    self.pos += 4;
                                }
                                Some(Packet::Fdri(self.data[dpos..epos].to_vec()))
                            }
                        }
                        _ => panic!("unk long write: {reg} times {num}"),
                    }
                } else {
                    panic!("unk word: {ph:04x}")
                }
            } else {
                match ph {
                    0xffff => Some(Packet::DummyWord),
                    0xaa99 => {
                        if self.kind == DeviceKind::Spartan6 {
                            assert_eq!(
                                u32::from_be_bytes(*array_ref!(self.data, self.pos - 2, 4)),
                                0xaa995566
                            );
                            self.pos += 2;
                        }
                        self.sync = true;
                        Some(Packet::SyncWord)
                    }
                    _ => panic!("unk word while desyncd: {ph:04x}"),
                }
            }
        }
    }
}
