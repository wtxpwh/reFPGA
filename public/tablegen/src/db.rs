use std::collections::{HashMap, HashSet};

use prjcombine_entity::{
    EntityBundleIndices, EntityBundleMap, EntityId, EntityMap, EntityPartVec, EntityVec,
    entity_id_u32,
};
use prjcombine_interconnect::db::{
    BelAttributeId, BelBidirId, BelClassId, BelInputId, BelOutputId, BelPadId, BelPinIndexing,
    BelSlotId, CellSlotId, ConnectorClassId, ConnectorSlotId, DeviceDataId, EnumClassId,
    EnumValueId, IntDb, RegionSlotId, TableFieldId, TableId, TableRowId, TileClassId, TileSlotId,
    WireSlotId,
};
use prjcombine_types::bsdata::{BitRectGeometry, BitRectId};
use proc_macro::Ident;

#[derive(Default)]
pub struct AnnotatedBelClass {
    pub input_id: EntityBundleMap<BelInputId, (Ident, BelPinIndexing)>,
    pub output_id: EntityBundleMap<BelOutputId, (Ident, BelPinIndexing)>,
    pub bidir_id: EntityBundleMap<BelBidirId, (Ident, BelPinIndexing)>,
    pub pad_id: EntityBundleMap<BelPadId, Ident>,
    pub attr_id: EntityMap<BelAttributeId, String, Ident>,
}

#[derive(Default)]
pub struct AnnotatedTable {
    pub field_id: EntityMap<TableFieldId, String, Ident>,
    pub row_id: EntityMap<TableRowId, String, Ident>,
}

pub struct AnnotatedDb {
    pub name: Option<Ident>,
    pub db: IntDb,
    pub eval_id: EntityPartVec<EnumClassId, EntityMap<EnumValueId, String, Ident>>,
    pub tcls_cell_id: EntityPartVec<TileClassId, EntityBundleMap<CellSlotId, Ident>>,
    pub tcls_bitrect_id: EntityPartVec<TileClassId, EntityBundleMap<BitRectId, Ident>>,
    pub bcls: EntityPartVec<BelClassId, AnnotatedBelClass>,
    pub table: EntityPartVec<TableId, AnnotatedTable>,
    pub bitrect_geoms: HashMap<String, BitRectGeometry>,
    pub skip_tcls: HashSet<TileClassId>,
    pub skip_ccls: HashSet<ConnectorClassId>,
    pub consts: EntityMap<ConstId, String, Const>,
}

entity_id_u32!(pub VariantId, AnnotatedDb, "VAR");

pub struct Const {
    #[allow(unused)]
    pub ident: Ident,
    pub value: usize,
}

entity_id_u32!(pub ConstId, Const, "CONST");

pub struct IdentRegistry<I: EntityId> {
    pub ids: EntityMap<I, String, Vec<Ident>>,
}

impl<I: EntityId> IdentRegistry<I> {
    pub fn insert(&mut self, ident: &Ident) -> (I, String) {
        let name = ident.to_string();
        if let Some((id, idents)) = self.ids.get_mut(&name) {
            idents.push(ident.clone());
            (id, name)
        } else {
            let (id, _) = self.ids.insert(name.clone(), vec![ident.clone()]);
            (id, name)
        }
    }

    pub fn get(&self, ident: &Ident) -> I {
        self.ids.get(&ident.to_string()).unwrap().0
    }

    pub fn get_ref(&mut self, ident: &Ident) -> Option<I> {
        let (id, idents) = self.ids.get_mut(&ident.to_string())?;
        idents.push(ident.clone());
        Some(id)
    }
}

impl<I: EntityId> Default for IdentRegistry<I> {
    fn default() -> Self {
        Self {
            ids: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BundleIdentData {
    pub array_size: Option<usize>,
    pub idents: Vec<Ident>,
    pub variants: EntityPartVec<VariantId, Option<usize>>,
}

pub struct BundleIdentRegistry<I: EntityId> {
    pub ids: EntityBundleMap<I, BundleIdentData>,
}

impl<I: EntityId> BundleIdentRegistry<I> {
    pub fn get(&self, ident: &Ident, i: Option<usize>) -> I {
        let idx = self.ids.get(&ident.to_string()).unwrap().0;
        match idx {
            EntityBundleIndices::Single(id) => {
                assert!(i.is_none());
                id
            }
            EntityBundleIndices::Array(range) => range.index(i.unwrap()),
        }
    }

    pub fn get_ref(&mut self, ident: &Ident) -> Option<EntityBundleIndices<I>> {
        let (id, data) = self.ids.get_mut(&ident.to_string())?;
        data.idents.push(ident.clone());
        Some(id)
    }
}

impl<I: EntityId> Default for BundleIdentRegistry<I> {
    fn default() -> Self {
        Self {
            ids: Default::default(),
        }
    }
}

pub struct AnnotatedDbSet {
    pub variants: EntityVec<VariantId, AnnotatedDb>,
    pub enum_id: IdentRegistry<EnumClassId>,
    pub tslot_id: IdentRegistry<TileSlotId>,
    pub cslot_id: IdentRegistry<ConnectorSlotId>,
    pub bslot_id: BundleIdentRegistry<BelSlotId>,
    pub rslot_id: BundleIdentRegistry<RegionSlotId>,
    pub wire_id: BundleIdentRegistry<WireSlotId>,
    pub tcls_id: IdentRegistry<TileClassId>,
    pub ccls_id: IdentRegistry<ConnectorClassId>,
    pub bcls_id: IdentRegistry<BelClassId>,
    pub table_id: IdentRegistry<TableId>,
    pub devdata_id: IdentRegistry<DeviceDataId>,
}
