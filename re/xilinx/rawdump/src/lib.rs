use prjcombine_entity::{
    EntityMap, EntityPartVec, EntitySet, EntityVec, entity_id_u16, entity_id_u32,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

impl Coord {
    pub fn delta(self, dx: i32, dy: i32) -> Coord {
        Coord {
            x: (self.x as i32 + dx) as u16,
            y: (self.y as i32 + dy) as u16,
        }
    }
}

entity_id_u32!(pub WireId, WireTag tag, "Wire");
entity_id_u32!(pub NodeClassId, NodeClassTag tag, "NodeClass");
entity_id_u32!(pub SpeedId, SpeedTag tag, "Speed");
entity_id_u16!(pub SlotKindId, SlotKindTag tag, "SlotKind");
entity_id_u16!(pub TileKindId, TileKind, "TileKind");
entity_id_u32!(pub TemplateId, TemplateTag tag, "Template");
entity_id_u32!(pub NodeId, TkNode, "Node");
entity_id_u32!(pub TkSiteId, TkSite, "TkSite");
entity_id_u32!(pub TkPipId, TkPip, "TkPip");
entity_id_u32!(pub TkWireId, TkWire, "TkWire");
entity_id_u32!(pub TkConnWireId, TkConnWireTag tag, "TkConnWire");

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Source {
    ISE,
    Vivado,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkSiteSlot {
    Single(SlotKindId),
    Indexed(SlotKindId, u8),
    Xy(SlotKindId, u8, u8),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkSitePinDir {
    Input,
    Output,
    Bidir,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkSitePin {
    pub dir: TkSitePinDir,
    pub wire: Option<WireId>,
    pub speed: Option<SpeedId>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TkSite {
    pub kind: String,
    pub pins: HashMap<String, TkSitePin>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkWire {
    Internal(Option<SpeedId>, Option<NodeClassId>),
    Connected(TkConnWireId),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkPipInversion {
    Never,
    Always,
    Prog,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkPipDirection {
    Uni,
    BiFwd,
    BiBwd,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkPip {
    pub is_buf: bool,
    pub is_excluded: bool,
    pub is_test: bool,
    pub inversion: TkPipInversion,
    pub direction: TkPipDirection,
    pub speed: Option<SpeedId>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TileKind {
    pub sites: EntityMap<TkSiteId, TkSiteSlot, TkSite>,
    pub wires: EntityMap<TkWireId, WireId, TkWire>,
    pub conn_wires: EntityVec<TkConnWireId, WireId>,
    pub pips: EntityMap<TkPipId, (WireId, WireId), TkPip>,
    pub tiles: Vec<Coord>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub name: String,
    pub kind: TileKindId,
    pub sites: EntityPartVec<TkSiteId, String>,
    #[serde(skip)]
    pub conn_wires: EntityPartVec<TkConnWireId, NodeId>,
    pub pip_overrides: HashMap<TkPipId, (NodeClassId, NodeClassId)>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkNode {
    pub base: Coord,
    pub template: TemplateId,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkNodeTemplateWire {
    pub delta: Coord,
    pub wire: WireId,
    pub speed: Option<SpeedId>,
    pub cls: Option<NodeClassId>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct PartCombo {
    pub name: String,
    pub device: String,
    pub package: String,
    pub speed: String,
    pub temp: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct PkgPin {
    pub pad: Option<String>,
    pub pin: String,
    pub vref_bank: Option<u32>,
    pub vcco_bank: Option<u32>,
    pub func: String,
    pub tracelen_um: Option<u32>,
    pub delay_min_fs: Option<u32>,
    pub delay_max_fs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub part: String,
    pub family: String,
    pub source: Source,
    pub width: u16,
    pub height: u16,
    pub tile_kinds: EntityMap<TileKindId, String, TileKind>,
    pub tiles: HashMap<Coord, Tile>,
    pub speeds: EntitySet<SpeedId, String>,
    pub node_classes: EntitySet<NodeClassId, String>,
    pub nodes: EntityVec<NodeId, TkNode>,
    pub templates: EntitySet<TemplateId, Vec<TkNodeTemplateWire>>,
    pub wires: EntitySet<WireId, String>,
    pub slot_kinds: EntitySet<SlotKindId, String>,
    pub packages: HashMap<String, Vec<PkgPin>>,
    pub combos: Vec<PartCombo>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeOrWire {
    Node(NodeId),
    Wire(Coord, TkWireId),
}

impl Part {
    pub fn post_deserialize(&mut self) {
        for (ni, node) in self.nodes.iter() {
            let template = &self.templates[node.template];
            for w in template {
                let coord = Coord {
                    x: node.base.x + w.delta.x,
                    y: node.base.y + w.delta.y,
                };
                let tile = self.tiles.get_mut(&coord).unwrap();
                let tk = &self.tile_kinds[tile.kind];
                let wire = tk.wires.get(&w.wire).unwrap().1;
                let idx = match *wire {
                    TkWire::Internal(_, _) => panic!("node on internal wire"),
                    TkWire::Connected(idx) => idx,
                };
                tile.conn_wires.insert(idx, ni);
            }
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let f = File::open(path)?;
        let mut cf = zstd::stream::Decoder::new(f)?;
        let config = bincode::config::standard();
        let mut res: Part = bincode::serde::decode_from_std_read(&mut cf, config).unwrap();
        res.post_deserialize();
        Ok(res)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let f = File::create(path)?;
        let mut cf = zstd::stream::Encoder::new(f, 9)?;
        let config = bincode::config::standard();
        bincode::serde::encode_into_std_write(self, &mut cf, config)?;
        cf.finish()?;
        Ok(())
    }

    pub fn tiles_by_kind_name(&self, name: &str) -> &[Coord] {
        if let Some((_, tk)) = self.tile_kinds.get(name) {
            &tk.tiles
        } else {
            &[]
        }
    }

    pub fn lookup_wire_raw(&self, crd: Coord, wire: WireId) -> Option<NodeOrWire> {
        let tile = &self.tiles[&crd];
        let tk = &self.tile_kinds[tile.kind];
        match tk.wires.get(&wire)? {
            (twi, TkWire::Internal(_, _)) => Some(NodeOrWire::Wire(crd, twi)),
            (_, &TkWire::Connected(idx)) => match tile.conn_wires.get(idx) {
                Some(&nidx) => Some(NodeOrWire::Node(nidx)),
                _ => None,
            },
        }
    }

    pub fn lookup_wire(&self, crd: Coord, wire: &str) -> Option<NodeOrWire> {
        let widx = self.wires.get(wire)?;
        self.lookup_wire_raw(crd, widx)
    }

    pub fn lookup_wire_raw_force(&self, crd: Coord, wire: WireId) -> NodeOrWire {
        let tile = &self.tiles[&crd];
        let tk = &self.tile_kinds[tile.kind];
        match tk.wires.get(&wire).unwrap() {
            (twi, TkWire::Internal(_, _)) => NodeOrWire::Wire(crd, twi),
            (twi, &TkWire::Connected(idx)) => match tile.conn_wires.get(idx) {
                Some(&nidx) => NodeOrWire::Node(nidx),
                _ => NodeOrWire::Wire(crd, twi),
            },
        }
    }

    pub fn lookup_wire_force(&self, crd: Coord, wire: &str) -> NodeOrWire {
        let widx = self.wires.get(wire).unwrap();
        self.lookup_wire_raw_force(crd, widx)
    }

    pub fn iter_node(&self, ni: NodeId) -> impl Iterator<Item = (Coord, WireId)> {
        let node = &self.nodes[ni];
        let tpl = &self.templates[node.template];
        tpl.iter().map(|w| {
            let tc = Coord {
                x: node.base.x + w.delta.x,
                y: node.base.y + w.delta.y,
            };
            (tc, w.wire)
        })
    }

    pub fn slot_xy(&self, name: &str, x: usize, y: usize) -> Option<TkSiteSlot> {
        let slot = self.slot_kinds.get(name)?;
        Some(TkSiteSlot::Xy(slot, x as _, y as _))
    }

    pub fn tk_slot_xy(&self, tk: TileKindId, name: &str, x: usize, y: usize) -> Option<TkSiteId> {
        let slot = self.slot_xy(name, x, y)?;
        Some(self.tile_kinds[tk].sites.get(&slot)?.0)
    }

    pub fn site_name_xy(&self, xy: Coord, name: &str, x: usize, y: usize) -> Option<&str> {
        let tile = &self.tiles[&xy];
        let sid = self.tk_slot_xy(tile.kind, name, x, y)?;
        tile.sites.get(sid).map(|x| x.as_str())
    }
}

impl Tile {
    pub fn has_wire(&self, tk: &TileKind, w: WireId) -> bool {
        match tk.wires.get(&w) {
            None => false,
            Some((_, &TkWire::Internal(_, _))) => true,
            Some((_, &TkWire::Connected(idx))) => self.conn_wires.get(idx).is_some(),
        }
    }
}
