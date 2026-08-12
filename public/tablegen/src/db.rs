use std::collections::{HashMap, HashSet};

use prjcombine_entity::{
    EntityBundleIndices, EntityBundleMap, EntityId, EntityMap, EntityPartVec, EntityRange,
    EntityVec, entity_id_u32,
};
use prjcombine_interconnect::db::{
    BelAttributeId, BelBidirId, BelClassBidir, BelClassId, BelClassInput, BelClassOutput,
    BelClassPad, BelInputId, BelOutputId, BelPadId, BelSlotId, CellSlotId, ConnectorClassId,
    ConnectorSlotId, DeviceDataId, EnumClassId, EnumValueId, IntDb, RegionSlotId, TableFieldId,
    TableId, TableRowId, TileClassId, TileSlotId, WireSlotId,
};
use prjcombine_types::bitrect::{BitRectGeometry, BitRectId};
use proc_macro::Ident;

pub struct AnnotatedDb {
    pub name: Option<Ident>,
    pub db: IntDb,
    pub tcls_cell_id: EntityPartVec<TileClassId, EntityBundleMap<CellSlotId, Ident>>,
    pub tcls_bitrect_id: EntityPartVec<TileClassId, EntityBundleMap<BitRectId, Ident>>,
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
pub struct BundleIdentData<V> {
    pub array_size: Option<usize>,
    pub idents: Vec<Ident>,
    pub variants: EntityPartVec<VariantId, (Option<usize>, V)>,
}

pub struct BundleIdentRegistry<I: EntityId, V> {
    pub ids: EntityBundleMap<I, BundleIdentData<V>>,
}

impl<I: EntityId, V> BundleIdentRegistry<I, V> {
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

    pub fn get_ref(
        &mut self,
        vid: VariantId,
        ident: &Ident,
    ) -> Option<(EntityBundleIndices<I>, &V)> {
        let (id, data) = self.ids.get_mut(&ident.to_string())?;
        data.idents.push(ident.clone());
        let (asize, vdata) = data.variants.get(vid)?;
        let id = match id {
            EntityBundleIndices::Single(_) => id,
            EntityBundleIndices::Array(range) => EntityBundleIndices::Array(EntityRange::new(
                range.start().to_idx(),
                range.start().to_idx() + asize.unwrap(),
            )),
        };
        Some((id, vdata))
    }
}

impl<I: EntityId, V> Default for BundleIdentRegistry<I, V> {
    fn default() -> Self {
        Self {
            ids: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct BelClassRegistry {
    pub input_id: BundleIdentRegistry<BelInputId, BelClassInput>,
    pub output_id: BundleIdentRegistry<BelOutputId, BelClassOutput>,
    pub bidir_id: BundleIdentRegistry<BelBidirId, BelClassBidir>,
    pub pad_id: BundleIdentRegistry<BelPadId, BelClassPad>,
    pub attr_id: IdentRegistry<BelAttributeId>,
}

#[derive(Default)]
pub struct TableRegistry {
    pub field_id: IdentRegistry<TableFieldId>,
    pub row_id: IdentRegistry<TableRowId>,
}

pub struct AnnotatedDbSet {
    pub variants: EntityVec<VariantId, AnnotatedDb>,
    pub enum_id: IdentRegistry<EnumClassId>,
    pub tslot_id: IdentRegistry<TileSlotId>,
    pub cslot_id: IdentRegistry<ConnectorSlotId>,
    pub bslot_id: BundleIdentRegistry<BelSlotId, TileSlotId>,
    pub rslot_id: BundleIdentRegistry<RegionSlotId, ()>,
    pub wire_id: BundleIdentRegistry<WireSlotId, ()>,
    pub tcls_id: IdentRegistry<TileClassId>,
    pub ccls_id: IdentRegistry<ConnectorClassId>,
    pub bcls_id: IdentRegistry<BelClassId>,
    pub table_id: IdentRegistry<TableId>,
    pub devdata_id: IdentRegistry<DeviceDataId>,
    pub ecls: EntityPartVec<EnumClassId, IdentRegistry<EnumValueId>>,
    pub bcls: EntityPartVec<BelClassId, BelClassRegistry>,
    pub table: EntityPartVec<TableId, TableRegistry>,
}
