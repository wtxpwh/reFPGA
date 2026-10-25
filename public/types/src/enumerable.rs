use core::marker::PhantomData;

use bincode::{BorrowDecode, Decode, Encode};

trait FromFn<T> {
    fn from_fn(f: impl FnMut(usize) -> T) -> Self;
}

impl<T, const N: usize> FromFn<T> for [T; N] {
    fn from_fn(f: impl FnMut(usize) -> T) -> Self {
        core::array::from_fn(f)
    }
}

pub trait Enumerable: Copy + Eq + core::hash::Hash {
    #[expect(private_bounds)]
    type Array<T>: IntoIterator<Item = T, IntoIter: DoubleEndedIterator + ExactSizeIterator>
        + FromFn<T>
        + AsRef<[T]>
        + AsMut<[T]>;
    const VALUES: Self::Array<Self>;
    fn to_enum_idx(self) -> usize;
}

#[derive(Copy, Clone)]
pub struct EnumMap<K: Enumerable, V> {
    values: K::Array<V>,
}

impl<K: Enumerable, V> EnumMap<K, V> {
    pub fn from_fn(mut f: impl FnMut(K) -> V) -> Self {
        Self {
            values: K::Array::from_fn(|i| f(K::VALUES.as_ref()[i])),
        }
    }

    pub const fn from_array(values: K::Array<V>) -> Self {
        Self { values }
    }

    pub fn iter(&self) -> EnumMapIter<'_, K, V> {
        EnumMapIter {
            inner: self.values.as_ref().iter().enumerate(),
            phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> EnumMapIterMut<'_, K, V> {
        EnumMapIterMut {
            inner: self.values.as_mut().iter_mut().enumerate(),
            phantom: PhantomData,
        }
    }

    pub fn values(&self) -> core::slice::Iter<'_, V> {
        self.values.as_ref().iter()
    }

    pub fn values_mut(&mut self) -> core::slice::IterMut<'_, V> {
        self.values.as_mut().iter_mut()
    }

    pub fn into_values(self) -> <K::Array<V> as core::iter::IntoIterator>::IntoIter {
        self.values.into_iter()
    }
}

impl<K: Enumerable, V: Default> Default for EnumMap<K, V> {
    fn default() -> Self {
        Self::from_fn(|_| Default::default())
    }
}

impl<K: Enumerable, Lhs: PartialEq<Rhs>, Rhs> PartialEq<EnumMap<K, Rhs>> for EnumMap<K, Lhs> {
    fn eq(&self, other: &EnumMap<K, Rhs>) -> bool {
        self.values.as_ref() == other.values.as_ref()
    }
}

impl<K: Enumerable, V: Eq> Eq for EnumMap<K, V> {}

impl<K: Enumerable, V: PartialOrd> PartialOrd for EnumMap<K, V> {
    fn partial_cmp(&self, other: &EnumMap<K, V>) -> Option<core::cmp::Ordering> {
        self.values.as_ref().partial_cmp(other.values.as_ref())
    }
}

impl<K: Enumerable, V: Ord> Ord for EnumMap<K, V> {
    fn cmp(&self, other: &EnumMap<K, V>) -> core::cmp::Ordering {
        self.values.as_ref().cmp(other.values.as_ref())
    }
}

impl<K: Enumerable, V: core::hash::Hash> core::hash::Hash for EnumMap<K, V> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.values.as_ref().hash(state);
    }
}

impl<K: Enumerable, V> core::ops::Index<K> for EnumMap<K, V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.values.as_ref()[index.to_enum_idx()]
    }
}

impl<K: Enumerable, V> core::ops::IndexMut<K> for EnumMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.values.as_mut()[index.to_enum_idx()]
    }
}

impl<K: Enumerable + core::fmt::Debug, V: core::fmt::Debug> core::fmt::Debug for EnumMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Enumerable, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    type IntoIter = EnumMapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        EnumMapIntoIter {
            inner: self.values.into_iter().enumerate(),
            phantom: PhantomData,
        }
    }
}

impl<'a, K: Enumerable, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = EnumMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Enumerable, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = EnumMapIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Enumerable, V: Encode> Encode for EnumMap<K, V> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        for v in self.values() {
            v.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Ctx, K: Enumerable, V: Decode<Ctx>> Decode<Ctx> for EnumMap<K, V> {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let mut res = EnumPartMap::new();
        for k in K::VALUES {
            res.insert(k, V::decode(decoder)?);
        }
        Ok(Self::from_fn(|k| res.remove(k).unwrap()))
    }
}

impl<'de, Ctx, K: Enumerable, V: BorrowDecode<'de, Ctx>> BorrowDecode<'de, Ctx> for EnumMap<K, V> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let mut res = EnumPartMap::new();
        for k in K::VALUES {
            res.insert(k, V::borrow_decode(decoder)?);
        }
        Ok(Self::from_fn(|k| res.remove(k).unwrap()))
    }
}

#[derive(Clone)]
pub struct EnumMapIter<'a, K: Enumerable, V> {
    inner: core::iter::Enumerate<core::slice::Iter<'a, V>>,
    phantom: PhantomData<K>,
}

impl<'a, K: Enumerable, V> Iterator for EnumMapIter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumMapIter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<'a, K: Enumerable, V> ExactSizeIterator for EnumMapIter<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct EnumMapIterMut<'a, K: Enumerable, V> {
    inner: core::iter::Enumerate<core::slice::IterMut<'a, V>>,
    phantom: PhantomData<K>,
}

impl<'a, K: Enumerable, V> Iterator for EnumMapIterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumMapIterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<'a, K: Enumerable, V> ExactSizeIterator for EnumMapIterMut<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct EnumMapIntoIter<K: Enumerable, V> {
    inner: core::iter::Enumerate<<K::Array<V> as IntoIterator>::IntoIter>,
    phantom: PhantomData<K>,
}

impl<K: Enumerable, V> Iterator for EnumMapIntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<K: Enumerable, V> DoubleEndedIterator for EnumMapIntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(i, v)| (K::VALUES.as_ref()[i], v))
    }
}

impl<K: Enumerable, V> ExactSizeIterator for EnumMapIntoIter<K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct EnumPartMap<K: Enumerable, V> {
    inner: EnumMap<K, Option<V>>,
}

impl<K: Enumerable, V> EnumPartMap<K, V> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.inner[key].as_ref()
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner[key].as_mut()
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.inner[key].is_some()
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.inner[key].replace(val)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        self.inner[key].take()
    }

    pub fn iter(&self) -> EnumPartMapIter<'_, K, V> {
        EnumPartMapIter {
            inner: self.inner.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> EnumPartMapIterMut<'_, K, V> {
        EnumPartMapIterMut {
            inner: self.inner.iter_mut(),
        }
    }

    pub fn keys(&self) -> EnumPartMapKeys<'_, K, V> {
        EnumPartMapKeys { inner: self.iter() }
    }

    pub fn values(&self) -> EnumPartMapValues<'_, K, V> {
        EnumPartMapValues { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> EnumPartMapValuesMut<'_, K, V> {
        EnumPartMapValuesMut {
            inner: self.iter_mut(),
        }
    }

    pub fn into_values(self) -> EnumPartMapIntoValues<K, V> {
        EnumPartMapIntoValues {
            inner: self.into_iter(),
        }
    }
}

impl<K: Enumerable, V> Default for EnumPartMap<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K: Enumerable, V> Clone for EnumPartMap<K, V>
where
    EnumMap<K, Option<V>>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K: Enumerable, V> Copy for EnumPartMap<K, V> where EnumMap<K, Option<V>>: Copy {}

impl<K: Enumerable, V> core::ops::Index<K> for EnumPartMap<K, V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.inner[index].as_ref().unwrap()
    }
}

impl<K: Enumerable, V> core::ops::IndexMut<K> for EnumPartMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.inner[index].as_mut().unwrap()
    }
}

impl<K: Enumerable, V> IntoIterator for EnumPartMap<K, V> {
    type Item = (K, V);
    type IntoIter = EnumPartMapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        EnumPartMapIntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

impl<'a, K: Enumerable, V> IntoIterator for &'a EnumPartMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = EnumPartMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Enumerable, V> IntoIterator for &'a mut EnumPartMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = EnumPartMapIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Enumerable, V> FromIterator<(K, V)> for EnumPartMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut res = EnumPartMap::new();
        for (k, v) in iter {
            res.insert(k, v);
        }
        res
    }
}

pub struct EnumPartMapIter<'a, K: Enumerable, V> {
    inner: EnumMapIter<'a, K, Option<V>>,
}

impl<'a, K: Enumerable, V> Iterator for EnumPartMapIter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next()? {
                return Some((k, v));
            }
        }
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumPartMapIter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next_back()? {
                return Some((k, v));
            }
        }
    }
}

pub struct EnumPartMapIterMut<'a, K: Enumerable, V> {
    inner: EnumMapIterMut<'a, K, Option<V>>,
}

impl<'a, K: Enumerable, V> Iterator for EnumPartMapIterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next()? {
                return Some((k, v));
            }
        }
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumPartMapIterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next_back()? {
                return Some((k, v));
            }
        }
    }
}

pub struct EnumPartMapIntoIter<K: Enumerable, V> {
    inner: EnumMapIntoIter<K, Option<V>>,
}

impl<K: Enumerable, V> Iterator for EnumPartMapIntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next()? {
                return Some((k, v));
            }
        }
    }
}

impl<K: Enumerable, V> DoubleEndedIterator for EnumPartMapIntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let (k, Some(v)) = self.inner.next_back()? {
                return Some((k, v));
            }
        }
    }
}

pub struct EnumPartMapKeys<'a, K: Enumerable, V> {
    inner: EnumPartMapIter<'a, K, V>,
}

impl<'a, K: Enumerable, V> Iterator for EnumPartMapKeys<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.0)
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumPartMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|x| x.0)
    }
}

pub struct EnumPartMapValues<'a, K: Enumerable, V> {
    inner: EnumPartMapIter<'a, K, V>,
}

impl<'a, K: Enumerable, V> Iterator for EnumPartMapValues<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumPartMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|x| x.1)
    }
}

pub struct EnumPartMapValuesMut<'a, K: Enumerable, V> {
    inner: EnumPartMapIterMut<'a, K, V>,
}

impl<'a, K: Enumerable, V> Iterator for EnumPartMapValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

impl<'a, K: Enumerable, V> DoubleEndedIterator for EnumPartMapValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|x| x.1)
    }
}

pub struct EnumPartMapIntoValues<K: Enumerable, V> {
    inner: EnumPartMapIntoIter<K, V>,
}

impl<K: Enumerable, V> Iterator for EnumPartMapIntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

impl<K: Enumerable, V> DoubleEndedIterator for EnumPartMapIntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|x| x.1)
    }
}
