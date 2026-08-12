//! A map where each key-value pair is assigned a contiguous range of IDs.
use crate::map::Entry;
use crate::part_map::DuplicateKeyError;
use crate::{EntityId, EntityMap, EntityPartVec, EntityRange, entity_id_u32};
#[cfg(feature = "bincode")]
use ::bincode::{Decode, Encode};

/// Indices occupied by a given bundle.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub enum EntityBundleIndices<I: EntityId> {
    /// Singular index occupied by a unit-shaped bundle.
    Single(I),
    /// Range of indices occupied by an array-shaped bundle.
    Array(EntityRange<I>),
}

impl<I: EntityId> From<I> for EntityBundleIndices<I> {
    fn from(value: I) -> Self {
        EntityBundleIndices::Single(value)
    }
}

impl<I: EntityId> From<EntityRange<I>> for EntityBundleIndices<I> {
    fn from(value: EntityRange<I>) -> Self {
        EntityBundleIndices::Array(value)
    }
}

/// An index within a particular bundle.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntityBundleItemIndex {
    /// The bundle is unit-shaped, thus we shall meow no further of indices within it.
    Single,
    /// The bundle is array-shaped.
    Array {
        /// The index within the array.
        index: usize,
        /// The total size of the array.
        total: usize,
    },
}

entity_id_u32!(BundleId, BundleTag tag, "BUNDLE");

/// A map where each key-value pair is assigned a contiguous range of IDs.
///
/// An `EntityBundleMap` is a collection of *bundles*. Each bundle consists of a key, a value,
/// and a *range* of IDs assigned to it. The amount of IDs assigned to a bundle is determined by
/// its *shape*: an array of a specified size, or just a single unit.
///
/// Note that we distinguish between a *single unit* and *an array of size one*. This is
/// intentional, as this datastructure is used to implement data models that make this distinction,
/// much like Rust makes a distinction between `u32` and `[u32; 1]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityBundleMap<I: EntityId, T> {
    ids: EntityPartVec<I, BundleId>,
    bundles: EntityMap<BundleId, String, (EntityBundleIndices<I>, T)>,
}

impl<I: EntityId, T> EntityBundleMap<I, T> {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
            bundles: Default::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn ids(&self) -> impl DoubleEndedIterator<Item = I> + '_ {
        self.ids.ids()
    }

    /// Retrieve a bundle by its key.
    pub fn get(&self, key: &str) -> Option<(EntityBundleIndices<I>, &T)> {
        let (_, (idx, val)) = self.bundles.get(key)?;
        Some((*idx, val))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<(EntityBundleIndices<I>, &mut T)> {
        let (_, (idx, val)) = self.bundles.get_mut(key)?;
        Some((*idx, val))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.bundles.contains_key(key)
    }

    /// Given an ID, returns the key of the bundle which owns that ID, as well as the particular
    /// position within the bundle that corresponds to the ID.
    pub fn key(&self, id: I) -> (&str, EntityBundleItemIndex) {
        let idx = self.ids[id];
        let key = self.bundles.key(idx);
        let (bidx, _) = self.bundles[idx];
        match bidx {
            EntityBundleIndices::Single(sid) => {
                assert_eq!(id, sid);
                (key, EntityBundleItemIndex::Single)
            }
            EntityBundleIndices::Array(range) => (
                key,
                EntityBundleItemIndex::Array {
                    index: range.index_of(id).unwrap(),
                    total: range.len(),
                },
            ),
        }
    }

    /// Insert a bundle.
    pub fn insert(
        &mut self,
        ids: impl Into<EntityBundleIndices<I>>,
        name: String,
        value: T,
    ) -> Result<(), DuplicateKeyError> {
        let ids = ids.into();
        match self.bundles.entry(name) {
            Entry::Occupied(_) => Err(DuplicateKeyError),
            Entry::Vacant(e) => {
                match ids {
                    EntityBundleIndices::Single(id) => {
                        if self.ids.contains_id(id) {
                            return Err(DuplicateKeyError);
                        }
                        self.ids.insert(id, e.index());
                    }
                    EntityBundleIndices::Array(range) => {
                        for id in range {
                            if self.ids.contains_id(id) {
                                return Err(DuplicateKeyError);
                            }
                        }
                        for id in range {
                            self.ids.insert(id, e.index());
                        }
                    }
                }
                e.insert((ids, value));
                Ok(())
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (I, &str, EntityBundleItemIndex, &T)> {
        self.ids.ids().map(|id| {
            let (idx, name) = self.key(id);
            (id, idx, name, &self[id])
        })
    }

    pub fn bundles(&self) -> impl Iterator<Item = (EntityBundleIndices<I>, &str, &T)> {
        self.bundles
            .iter()
            .map(|(_, k, (i, v))| (*i, k.as_str(), v))
    }

    pub fn bundles_mut(&mut self) -> impl Iterator<Item = (EntityBundleIndices<I>, &str, &mut T)> {
        self.bundles
            .iter_mut()
            .map(|(_, k, (i, v))| (*i, k.as_str(), v))
    }

    pub fn into_bundles(self) -> impl Iterator<Item = (EntityBundleIndices<I>, String, T)> {
        self.bundles.into_iter().map(|(_, k, (i, v))| (i, k, v))
    }
}

impl<I: EntityId, T> Default for EntityBundleMap<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: EntityId, T> core::ops::Index<I> for EntityBundleMap<I, T> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        let idx = self.ids[index];
        &self.bundles[idx].1
    }
}

impl<I: EntityId, T> core::ops::IndexMut<I> for EntityBundleMap<I, T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let idx = self.ids[index];
        &mut self.bundles[idx].1
    }
}

#[cfg(feature = "bincode")]
mod bincode;
