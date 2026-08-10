use core::fmt::Debug;
use std::collections::{BTreeMap, btree_map};

use bincode::{Decode, Encode};
use itertools::*;

use crate::{
    bitrect::{PolTileBit, TileBit},
    bitvec::BitVec,
};

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, Default)]
pub struct Tile {
    pub items: BTreeMap<String, TileItem>,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub fn merge(&mut self, other: &Tile, neutral: impl Fn(TileBit) -> bool) {
        if self == other {
            return;
        }
        for (k, v) in &other.items {
            match self.items.entry(k.clone()) {
                btree_map::Entry::Vacant(e) => {
                    e.insert(v.clone());
                }
                btree_map::Entry::Occupied(mut e) => {
                    e.get_mut().merge(v, &neutral);
                }
            }
        }
    }

    pub fn insert(
        &mut self,
        name: impl Into<String>,
        item: TileItem,
        neutral: impl Fn(TileBit) -> bool,
    ) {
        match self.items.entry(name.into()) {
            btree_map::Entry::Vacant(e) => {
                e.insert(item);
            }
            btree_map::Entry::Occupied(mut e) => {
                e.get_mut().merge(&item, neutral);
            }
        }
    }

    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (key, item) in &self.items {
            write!(o, "\t{key}:")?;
            for bit in item.bits.iter().rev() {
                write!(o, " {bit:?}")?;
            }
            match item.kind {
                TileItemKind::Enum { ref values } => {
                    writeln!(o,)?;
                    for (key, val) in values {
                        write!(o, "\t\t")?;
                        for bit in val.iter().rev() {
                            write!(o, "{}", usize::from(bit))?;
                        }
                        writeln!(o, ": {key}")?;
                    }
                }
                TileItemKind::BitVec { ref invert } => {
                    write!(o, " inv ")?;
                    for bit in invert.iter().rev() {
                        write!(o, "{}", usize::from(bit))?;
                    }
                    writeln!(o)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct TileItem {
    pub bits: Vec<TileBit>,
    pub kind: TileItemKind,
}

impl TileItem {
    pub fn merge(&mut self, other: &TileItem, neutral: impl Fn(TileBit) -> bool) {
        if self == other {
            return;
        }
        let TileItemKind::Enum { values: av } = &mut self.kind else {
            panic!("weird merge: {self:?} {other:?}");
        };
        let TileItemKind::Enum { values: bv } = &other.kind else {
            unreachable!()
        };
        let mut bits = self.bits.clone();
        for &bit in &other.bits {
            if !bits.contains(&bit) {
                bits.push(bit);
            }
        }
        bits.sort();
        let bit_map_a: Vec<_> = bits
            .iter()
            .map(|&x| self.bits.iter().find_position(|&&y| x == y).map(|x| x.0))
            .collect();
        let bit_map_b: Vec<_> = bits
            .iter()
            .map(|&x| other.bits.iter().find_position(|&&y| x == y).map(|x| x.0))
            .collect();
        self.bits = bits;
        for val in av.values_mut() {
            *val = bit_map_a
                .iter()
                .enumerate()
                .map(|(i, &x)| match x {
                    Some(idx) => val[idx],
                    None => neutral(self.bits[i]),
                })
                .collect();
        }
        for (key, val) in bv {
            let val: BitVec = bit_map_b
                .iter()
                .enumerate()
                .map(|(i, &x)| match x {
                    Some(idx) => val[idx],
                    None => neutral(self.bits[i]),
                })
                .collect();

            match av.entry(key.clone()) {
                btree_map::Entry::Vacant(e) => {
                    e.insert(val);
                }
                btree_map::Entry::Occupied(e) => {
                    if *e.get() != val {
                        panic!("tile merge failed at {key}: {cv} vs {val:?}", cv = e.get());
                    }
                }
            }
        }
    }

    pub fn from_bit_inv(bit: TileBit, invert: bool) -> Self {
        Self {
            bits: vec![bit],
            kind: TileItemKind::BitVec {
                invert: BitVec::from_iter([invert]),
            },
        }
    }

    pub fn from_bitvec_inv(bits: Vec<TileBit>, invert: bool) -> Self {
        let invert = BitVec::repeat(invert, bits.len());
        Self {
            bits,
            kind: TileItemKind::BitVec { invert },
        }
    }

    pub fn as_bitvec(&self) -> Vec<PolTileBit> {
        let TileItemKind::BitVec { ref invert } = self.kind else {
            unreachable!()
        };
        self.bits
            .iter()
            .zip(invert)
            .map(|(&bit, inv)| PolTileBit { bit, inv })
            .collect()
    }

    pub fn as_bit(&self) -> PolTileBit {
        assert_eq!(self.bits.len(), 1);
        let TileItemKind::BitVec { ref invert } = self.kind else {
            unreachable!()
        };
        PolTileBit {
            bit: self.bits[0],
            inv: invert[0],
        }
    }
}

impl From<PolTileBit> for TileItem {
    fn from(value: PolTileBit) -> Self {
        Self {
            bits: vec![value.bit],
            kind: TileItemKind::BitVec {
                invert: BitVec::from_iter([value.inv]),
            },
        }
    }
}

impl From<&[PolTileBit]> for TileItem {
    fn from(value: &[PolTileBit]) -> Self {
        Self {
            bits: value.iter().map(|pbit| pbit.bit).collect(),
            kind: TileItemKind::BitVec {
                invert: value.iter().map(|pbit| pbit.inv).collect(),
            },
        }
    }
}

impl From<Vec<PolTileBit>> for TileItem {
    fn from(value: Vec<PolTileBit>) -> Self {
        TileItem::from(value.as_slice())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Encode, Decode)]
pub enum TileItemKind {
    Enum { values: BTreeMap<String, BitVec> },
    BitVec { invert: BitVec },
}
