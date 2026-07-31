use std::hash::BuildHasher;

use bincode::{BorrowDecode, Decode, Encode, error::DecodeError};
use hashbrown::HashTable;

use crate::{EntityId, EntityPartMap, EntityPartVec, part_map::EntityKey};

impl<I, V, RS> Encode for EntityPartMap<I, V, RS>
where
    I: Encode + EntityId,
    V: Encode + EntityKey,
    RS: BuildHasher,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.items.encode(encoder)
    }
}

impl<I, V, RS, Context> Decode<Context> for EntityPartMap<I, V, RS>
where
    I: Decode<Context> + EntityId,
    V: Decode<Context> + EntityKey,
    RS: Default + BuildHasher,
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let items: EntityPartVec<I, V> = Decode::decode(decoder)?;
        let mut table = HashTable::new();
        let h = RS::default();
        for (id, item) in &items {
            let hash = h.hash_one(item.key());
            if table
                .find_entry(hash, |&id| item.key() == items[id].key())
                .is_ok()
            {
                return Err(DecodeError::Other("duplicate key"));
            }
            table.insert_unique(hash, id, |&id| h.hash_one(items[id].key()));
        }
        Ok(Self { items, table, h })
    }
}

impl<'de, I, V, RS, Context> BorrowDecode<'de, Context> for EntityPartMap<I, V, RS>
where
    I: BorrowDecode<'de, Context> + EntityId,
    V: BorrowDecode<'de, Context> + EntityKey,
    RS: Default + BuildHasher,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let items: EntityPartVec<I, V> = BorrowDecode::borrow_decode(decoder)?;
        let mut table = HashTable::new();
        let h = RS::default();
        for (id, item) in &items {
            let hash = h.hash_one(item.key());
            if table
                .find_entry(hash, |&id| item.key() == items[id].key())
                .is_ok()
            {
                return Err(DecodeError::Other("duplicate key"));
            }
            table.insert_unique(hash, id, |&id| h.hash_one(items[id].key()));
        }
        Ok(Self { items, table, h })
    }
}
