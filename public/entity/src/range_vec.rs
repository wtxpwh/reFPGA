use crate::{EntityIdArith, EntityRange, EntityVec, entity_id_u32};

entity_id_u32!(RangeId, RangeTag tag, "RANGE");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityRangeVec<I: EntityIdArith, V> {
    ranges: EntityVec<RangeId, (EntityRange<I>, V)>,
    items: EntityVec<I, RangeId>,
}

impl<I: EntityIdArith, V> EntityRangeVec<I, V> {
    pub fn new() -> Self {
        Self {
            ranges: Default::default(),
            items: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn len_ranges(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn range(&self, idx: I) -> EntityRange<I> {
        self.ranges[self.items[idx]].0
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
        self.items.clear();
    }

    pub fn ids(&self) -> EntityRange<I> {
        self.items.ids()
    }

    pub fn next_id(&self) -> I {
        self.items.next_id()
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &V> + ExactSizeIterator {
        self.ranges.values().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> + ExactSizeIterator {
        self.ranges.values_mut().map(|(_, v)| v)
    }

    pub fn into_values(self) -> impl DoubleEndedIterator<Item = V> + ExactSizeIterator {
        self.ranges.into_values().map(|(_, v)| v)
    }

    pub fn iter(&self) -> Iter<'_, I, V> {
        Iter {
            inner: self.ranges.values(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, I, V> {
        IterMut {
            inner: self.ranges.values_mut(),
        }
    }

    pub fn inner_separators(&self) -> impl DoubleEndedIterator<Item = I> + ExactSizeIterator {
        self.ranges.values().skip(1).map(|&(r, _)| r.start())
    }

    pub fn push(&mut self, num: usize, val: V) -> EntityRange<I> {
        let start = self.items.next_id();
        let end = start + num;
        let range = start.range(end);
        if num != 0 {
            let rid = self.ranges.push((range, val));
            for _ in 0..num {
                self.items.push(rid);
            }
        }
        assert_eq!(self.items.next_id(), end);
        range
    }
}

impl<I: EntityIdArith, V> Default for EntityRangeVec<I, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: EntityIdArith, V> core::ops::Index<I> for EntityRangeVec<I, V> {
    type Output = V;

    fn index(&self, index: I) -> &Self::Output {
        &self.ranges[self.items[index]].1
    }
}

impl<I: EntityIdArith, V> core::ops::IndexMut<I> for EntityRangeVec<I, V> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.ranges[self.items[index]].1
    }
}

impl<I: EntityIdArith, V> IntoIterator for EntityRangeVec<I, V> {
    type Item = (EntityRange<I>, V);

    type IntoIter = std::vec::IntoIter<(EntityRange<I>, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_values()
    }
}

impl<'a, I: EntityIdArith, V> IntoIterator for &'a EntityRangeVec<I, V> {
    type Item = (EntityRange<I>, &'a V);

    type IntoIter = Iter<'a, I, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, I: EntityIdArith, V> IntoIterator for &'a mut EntityRangeVec<I, V> {
    type Item = (EntityRange<I>, &'a mut V);

    type IntoIter = IterMut<'a, I, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterator of `(range, &value)` pairs.
#[derive(Clone, Debug)]
pub struct Iter<'a, I: EntityIdArith, V> {
    inner: core::slice::Iter<'a, (EntityRange<I>, V)>,
}

impl<'a, I: EntityIdArith, V> Iterator for Iter<'a, I, V> {
    type Item = (EntityRange<I>, &'a V);
    fn next(&mut self) -> Option<(EntityRange<I>, &'a V)> {
        let &(ids, ref val) = self.inner.next()?;
        Some((ids, val))
    }
}

impl<'a, I: EntityIdArith, V> DoubleEndedIterator for Iter<'a, I, V> {
    fn next_back(&mut self) -> Option<(EntityRange<I>, &'a V)> {
        let &(ids, ref val) = self.inner.next_back()?;
        Some((ids, val))
    }
}

impl<'a, I: EntityIdArith, V> ExactSizeIterator for Iter<'a, I, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Iterator of `(id, &mut value)` pairs.
#[derive(Debug)]
pub struct IterMut<'a, I: EntityIdArith, V> {
    inner: core::slice::IterMut<'a, (EntityRange<I>, V)>,
}

impl<'a, I: EntityIdArith, V> Iterator for IterMut<'a, I, V> {
    type Item = (EntityRange<I>, &'a mut V);
    fn next(&mut self) -> Option<(EntityRange<I>, &'a mut V)> {
        let &mut (ids, ref mut val) = self.inner.next()?;
        Some((ids, val))
    }
}

impl<'a, I: EntityIdArith, V> DoubleEndedIterator for IterMut<'a, I, V> {
    fn next_back(&mut self) -> Option<(EntityRange<I>, &'a mut V)> {
        let &mut (ids, ref mut val) = self.inner.next_back()?;
        Some((ids, val))
    }
}

impl<'a, I: EntityIdArith, V> ExactSizeIterator for IterMut<'a, I, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(feature = "bincode")]
mod bincode;
