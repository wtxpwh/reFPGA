use core::fmt::{self, Debug};
use std::{
    hash::{BuildHasher, Hash, RandomState},
    ops::{Index, IndexMut},
};

use hashbrown::HashTable;
use indexmap::Equivalent;

use crate::{EntityId, EntityPartVec};

pub trait EntityKey {
    type KeyType: core::hash::Hash + Eq + ?Sized;
    fn key(&self) -> &Self::KeyType;
}

impl EntityKey for String {
    type KeyType = String;
    fn key(&self) -> &Self::KeyType {
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct DuplicateKeyError;

impl core::fmt::Display for DuplicateKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "duplicate key error")
    }
}

impl core::error::Error for DuplicateKeyError {}

#[derive(Clone)]
pub struct EntityPartMap<I: EntityId, V: EntityKey, RS: BuildHasher = RandomState> {
    items: EntityPartVec<I, V>,
    table: HashTable<I>,
    h: RS,
}

impl<I: EntityId, V: EntityKey> EntityPartMap<I, V, RandomState> {
    pub fn new() -> Self {
        Self::with_hasher(Default::default())
    }

    pub fn with_capacity(n: usize) -> Self {
        Self::with_capacity_and_hasher(n, Default::default())
    }
}

impl<I: EntityId, V: EntityKey, RS: BuildHasher> EntityPartMap<I, V, RS> {
    pub fn with_hasher(h: RS) -> Self {
        Self {
            items: Default::default(),
            table: Default::default(),
            h,
        }
    }

    pub fn with_capacity_and_hasher(n: usize, h: RS) -> Self {
        Self {
            items: EntityPartVec::with_capacity(n),
            table: HashTable::with_capacity(n),
            h,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.table.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<(I, &V)>
    where
        Q: ?Sized + Hash + Equivalent<V::KeyType>,
    {
        self.table
            .find(self.h.hash_one(key), |&id| {
                key.equivalent(self.items[id].key())
            })
            .map(|&id| (id, &self.items[id]))
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<(I, &mut V)>
    where
        Q: ?Sized + Hash + Equivalent<V::KeyType>,
    {
        self.table
            .find(self.h.hash_one(key), |&id| {
                key.equivalent(self.items[id].key())
            })
            .map(|&id| (id, &mut self.items[id]))
    }

    pub fn get_id(&self, id: I) -> Option<&V> {
        self.items.get(id)
    }

    pub fn get_mut_id(&mut self, id: I) -> Option<&mut V> {
        self.items.get_mut(id)
    }

    pub fn contains_id(&self, id: I) -> bool {
        self.items.contains_id(id)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: ?Sized + Hash + Equivalent<V::KeyType>,
    {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, id: I, v: V) -> Result<(), DuplicateKeyError> {
        let h = self.h.hash_one(v.key());
        if let Some(&cur_id) = self
            .table
            .find(h, |&cur_id| v.key() == self.items[cur_id].key())
        {
            if id == cur_id {
                self.items.insert(id, v);
                Ok(())
            } else {
                Err(DuplicateKeyError)
            }
        } else if let Some(cur_item) = self.items.get(id) {
            if cur_item.key() == v.key() {
                self.items.insert(id, v);
            } else {
                // must be in the table already
                let entry = self
                    .table
                    .find_entry(self.h.hash_one(cur_item.key()), |&cur_id| cur_id == id)
                    .unwrap();
                entry.remove();
                self.items.insert(id, v);
                self.table
                    .insert_unique(h, id, |&id| self.h.hash_one(self.items[id].key()));
            }
            Ok(())
        } else {
            self.items.insert(id, v);
            self.table
                .insert_unique(h, id, |&id| self.h.hash_one(self.items[id].key()));
            Ok(())
        }
    }

    pub fn remove(&mut self, id: I) -> Option<V> {
        let res = self.items.remove(id)?;
        let entry = self
            .table
            .find_entry(self.h.hash_one(res.key()), |&cur_id| cur_id == id)
            .unwrap();
        entry.remove();
        Some(res)
    }

    pub fn iter(&self) -> crate::part::Iter<'_, I, V> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> crate::part::IterMut<'_, I, V> {
        self.items.iter_mut()
    }

    pub fn ids(&self) -> crate::part::Ids<'_, I, V> {
        self.items.ids()
    }

    pub fn into_ids(self) -> crate::part::IntoIds<I, V> {
        self.items.into_ids()
    }

    pub fn values(&self) -> crate::part::Values<'_, I, V> {
        self.items.values()
    }

    pub fn values_mut(&mut self) -> crate::part::ValuesMut<'_, I, V> {
        self.items.values_mut()
    }

    pub fn into_values(self) -> crate::part::IntoValues<I, V> {
        self.items.into_values()
    }

    pub fn map<O>(&self, f: impl FnMut(I, &V) -> O) -> EntityPartVec<I, O> {
        self.items.map(f)
    }

    pub fn map_values<O>(&self, f: impl FnMut(&V) -> O) -> EntityPartVec<I, O> {
        self.items.map_values(f)
    }
}

impl<I: EntityId, V: EntityKey> Default for EntityPartMap<I, V, RandomState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, V, RS> Debug for EntityPartMap<I, V, RS>
where
    I: EntityId,
    V: EntityKey + Debug,
    RS: BuildHasher,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_map().entries(self.iter()).finish()
    }
}

impl<I, V, RS, RS2> PartialEq<EntityPartMap<I, V, RS2>> for EntityPartMap<I, V, RS>
where
    I: EntityId,
    V: EntityKey + PartialEq,
    RS: BuildHasher,
    RS2: BuildHasher,
{
    fn eq(&self, other: &EntityPartMap<I, V, RS2>) -> bool {
        self.items.eq(&other.items)
    }
}

impl<I, V, RS> Eq for EntityPartMap<I, V, RS>
where
    I: EntityId,
    V: EntityKey + Eq,
    RS: BuildHasher,
{
}

impl<I: EntityId, V: EntityKey, RS: BuildHasher> IntoIterator for EntityPartMap<I, V, RS> {
    type Item = (I, V);
    type IntoIter = crate::part::IntoIter<I, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, I: EntityId, V: EntityKey, RS: BuildHasher> IntoIterator for &'a EntityPartMap<I, V, RS> {
    type Item = (I, &'a V);
    type IntoIter = crate::part::Iter<'a, I, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, I: EntityId, V: EntityKey, RS: BuildHasher> IntoIterator
    for &'a mut EntityPartMap<I, V, RS>
{
    type Item = (I, &'a mut V);
    type IntoIter = crate::part::IterMut<'a, I, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<I: EntityId, V: EntityKey, RS: BuildHasher> Index<I> for EntityPartMap<I, V, RS> {
    type Output = V;
    fn index(&self, index: I) -> &V {
        &self.items[index]
    }
}

impl<I: EntityId, V: EntityKey, RS: BuildHasher> IndexMut<I> for EntityPartMap<I, V, RS> {
    fn index_mut(&mut self, index: I) -> &mut V {
        &mut self.items[index]
    }
}

#[cfg(feature = "bincode")]
mod bincode;
