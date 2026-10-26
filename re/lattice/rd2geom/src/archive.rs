use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

use arrayref::array_ref;
use flate2::read::ZlibDecoder;

pub struct Reader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn new_from(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    pub fn get_u8(&mut self) -> u8 {
        let res = self.data[self.pos];
        self.pos += 1;
        res
    }

    pub fn get_u16(&mut self) -> u16 {
        let res = u16::from_be_bytes(*array_ref![self.data, self.pos, 2]);
        self.pos += 2;
        res
    }

    pub fn get_u32(&mut self) -> u32 {
        let res = u32::from_be_bytes(*array_ref![self.data, self.pos, 4]);
        self.pos += 4;
        res
    }

    pub fn get_i32(&mut self) -> i32 {
        let res = i32::from_be_bytes(*array_ref![self.data, self.pos, 4]);
        self.pos += 4;
        res
    }

    pub fn get_f64(&mut self) -> f64 {
        let res = f64::from_be_bytes(*array_ref![self.data, self.pos, 8]);
        self.pos += 8;
        res
    }

    pub fn get_zstring(&mut self) -> String {
        let mut epos = self.pos;
        while self.data[epos] != 0 {
            epos += 1;
        }
        let res = String::from_utf8(self.data[self.pos..epos].to_vec()).unwrap();
        self.pos = epos + 1;
        res
    }

    pub fn get_nlstring(&mut self) -> String {
        let mut epos = self.pos;
        while self.data[epos] != b'\n' {
            epos += 1;
        }
        let res = String::from_utf8(self.data[self.pos..epos].to_vec()).unwrap();
        self.pos = epos + 1;
        res
    }
}

pub struct Archive {
    #[allow(unused)]
    pub typ: u16,
    pub entries: BTreeMap<String, ArchiveEntry>,
}

pub struct ArchiveEntry {
    pub data: Vec<u8>,
    #[allow(unused)]
    pub hidden: bool,
    #[allow(unused)]
    pub flags: u32,
}

fn decode(raw_data: &[u8]) -> Vec<u8> {
    let Some(raw_data) = raw_data.strip_prefix(b"\x5b\x5b\xa6\xa6\xc4\xc4\x72") else {
        return raw_data.to_vec();
    };
    let mut pos = 0;
    let mut result = vec![];
    while pos != raw_data.len() {
        let chunk_len = u32::from_be_bytes(*array_ref![raw_data, pos, 4]) as usize;
        pos += 4;
        let raw_chunk = &raw_data[pos..(pos + chunk_len)];
        let mut dec = ZlibDecoder::new(raw_chunk);
        let mut buf = vec![];
        dec.read_to_end(&mut buf).unwrap();
        result.extend(buf);
        pos += chunk_len;
    }
    result
}

pub fn read_archive(path: &Path) -> Archive {
    let data = std::fs::read(path).unwrap();
    let data = decode(&data);
    let mut reader = Reader::new(&data);
    let typ = reader.get_u16();
    let num_entries = reader.get_u32() as usize;
    let dir_offset = reader.get_u32() as usize;
    let mut reader = Reader::new_from(&data, dir_offset);
    let mut entries = BTreeMap::new();
    for _ in 0..num_entries {
        let name = reader.get_zstring();
        let entry_off = reader.get_u32() as usize;
        let entry_size = reader.get_u32() as usize;
        let hidden = reader.get_u32();
        let flags = reader.get_u32();
        let hidden = match hidden {
            0 => false,
            1 => true,
            _ => unreachable!(),
        };
        entries.insert(
            name,
            ArchiveEntry {
                data: data[entry_off..(entry_off + entry_size)].to_vec(),
                hidden,
                flags,
            },
        );
    }
    Archive { typ, entries }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    #[allow(unused)]
    Float(f64),
    Int(i32),
}

pub struct PkgData {
    pub pkgs: BTreeSet<String>,
    pub pins: Vec<BTreeMap<String, Value>>,
}

pub fn parse_pkg(archive: &Archive) -> PkgData {
    let mut reader = Reader::new(&archive.entries["pininfo_table"].data);
    let version = reader.get_u32();
    let _ver = reader.get_nlstring();
    let num_lines = reader.get_u32();
    for _ in 0..num_lines {
        let _line = reader.get_nlstring();
    }
    let num_pkgs = reader.get_u32();
    let mut pkgs = BTreeSet::new();
    for _ in 0..num_pkgs {
        let _unk0 = reader.get_u32();
        let pkg = reader.get_zstring();
        let _ = reader.get_zstring();
        let _ = reader.get_zstring();
        let _ = reader.get_zstring();
        pkgs.insert(pkg);
    }
    let num_unk = reader.get_u32();
    for _ in 0..num_unk {
        let _ = reader.get_zstring();
        let _ = reader.get_zstring();
        let _ = reader.get_zstring();
    }
    if version != 0x3e8 {
        let num_die = reader.get_u32();
        for _ in 0..num_die {
            let _ = reader.get_u32();
            let _ = reader.get_zstring();
            let _ = reader.get_zstring();
        }
    }
    let num_kv = reader.get_u32() as usize;
    let num_pins = reader.get_u32() as usize;
    let mut pins = vec![];
    for _ in 0..num_pins {
        pins.push(BTreeMap::new());
    }
    for _ in 0..num_kv {
        let kind = reader.get_u8();
        let key = reader.get_zstring();
        for i in 0..num_pins {
            let val = match kind {
                1 => Value::String(reader.get_zstring()),
                2 => Value::Int(reader.get_i32()),
                3 => Value::Float(reader.get_f64()),
                _ => panic!("weird kind {kind}"),
            };
            pins[i].insert(key.clone(), val);
        }
    }
    PkgData { pkgs, pins }
}
