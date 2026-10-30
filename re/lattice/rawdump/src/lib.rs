use bincode::{Decode, Encode, impl_borrow_decode};
use ndarray::Array2;
use prjcombine_entity::{EntitySet, EntityVec, entity_id_u32};
use prjcombine_re_lattice_naming::{StringId, WireName};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

entity_id_u32!(pub NodeId, NodeTag tag, "NODE");
entity_id_u32!(pub GridId, Grid, "GRID");
entity_id_u32!(pub BufferId, BufferTag tag, "BUF");

#[derive(Clone, Debug, Encode, Decode)]
pub struct Db {
    pub family: String,
    pub grids: EntityVec<GridId, Grid>,
    pub parts: Vec<Part>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Grid {
    pub nph: String,
    pub tiles: Array2<Tile>,
    pub strings: EntitySet<StringId, String>,
    pub nodes: EntitySet<NodeId, WireName>,
    pub pips: BTreeMap<(NodeId, NodeId), Pip>,
}

impl Encode for Grid {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.nph.encode(encoder)?;
        let (w, h) = self.tiles.dim();
        w.encode(encoder)?;
        h.encode(encoder)?;
        for tile in &self.tiles {
            tile.encode(encoder)?;
        }
        self.strings.encode(encoder)?;
        self.nodes.encode(encoder)?;
        self.pips.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Grid {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let nph = Decode::decode(decoder)?;
        let w = usize::decode(decoder)?;
        let h = usize::decode(decoder)?;
        let mut v = Vec::with_capacity(w * h);
        for _ in 0..(w * h) {
            v.push(Tile::decode(decoder)?);
        }
        let tiles = Array2::from_shape_vec((w, h), v).unwrap();
        let strings = Decode::decode(decoder)?;
        let nodes = Decode::decode(decoder)?;
        let pips = Decode::decode(decoder)?;
        Ok(Grid {
            nph,
            tiles,
            strings,
            nodes,
            pips,
        })
    }
}

impl_borrow_decode!(Grid);

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct Pip {
    pub is_pseudo: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct Tile {
    pub name: String,
    pub kind: String,
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub sites: Vec<TileSite>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct TileSite {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct Site {
    pub name: String,
    pub typ: Option<String>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Part {
    pub arch: String,
    pub name: String,
    pub package: String,
    pub speeds: Vec<String>,
    pub grid: GridId,
    pub sites: Vec<Site>,
}

impl Db {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let f = File::open(path)?;
        let mut cf = zstd::stream::Decoder::new(f)?;
        let config = bincode::config::standard();
        Ok(bincode::decode_from_std_read(&mut cf, config)?)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let f = File::create(path)?;
        let mut cf = zstd::stream::Encoder::new(f, 9)?;
        let config = bincode::config::standard();
        bincode::encode_into_std_write(self, &mut cf, config)?;
        cf.finish()?;
        Ok(())
    }
}
