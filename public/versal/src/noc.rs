use std::collections::{HashMap, hash_map};

use prjcombine_entity::{
    EntityId,
    id::{EntityIdU8, EntityTag},
};
use prjcombine_interconnect::{db::IntDb, grid::BelCoord};

pub struct NocPortTag;

impl EntityTag for NocPortTag {
    const PREFIX: &'static str = "P";
}

pub type NocPortId = EntityIdU8<NocPortTag>;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct NocPortCoord {
    pub bel: BelCoord,
    pub port: NocPortId,
}

impl core::ops::Deref for NocPortCoord {
    type Target = BelCoord;

    fn deref(&self) -> &Self::Target {
        &self.bel
    }
}

impl NocPortCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        format!(
            "{cell}.{slot}.{port}",
            cell = self.cell,
            slot = db.bel_slots.key(self.slot),
            port = self.port,
        )
    }
}

pub trait BelCoordExt {
    fn noc_port(self, port: usize) -> NocPortCoord;
}

impl BelCoordExt for BelCoord {
    fn noc_port(self, port: usize) -> NocPortCoord {
        NocPortCoord {
            bel: self,
            port: NocPortId::from_idx(port),
        }
    }
}

pub(crate) struct NocBuilder<'a> {
    pub db: &'a IntDb,
    pub conns: HashMap<NocPortCoord, NocPortCoord>,
}

impl<'a> NocBuilder<'a> {
    pub(crate) fn new(db: &'a IntDb) -> Self {
        Self {
            db,
            conns: Default::default(),
        }
    }

    pub fn connect_uni(&mut self, port_a: NocPortCoord, port_b: NocPortCoord) {
        match self.conns.entry(port_a) {
            hash_map::Entry::Occupied(e) => panic!(
                "connect {a} -> {b} failed: already connected to {c}",
                a = port_a.to_string(self.db),
                b = port_b.to_string(self.db),
                c = e.get().to_string(self.db),
            ),
            hash_map::Entry::Vacant(e) => {
                e.insert(port_b);
            }
        }
    }

    pub fn connect(&mut self, port_a: NocPortCoord, port_b: NocPortCoord) {
        self.connect_uni(port_a, port_b);
        self.connect_uni(port_b, port_a);
    }

    pub fn finish(self) -> HashMap<NocPortCoord, NocPortCoord> {
        self.conns
    }
}
