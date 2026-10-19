use bincode::{BorrowDecode, Decode, Encode};

use crate::{
    EntityBundleIndices, EntityBundleMap, EntityId, EntityMap, EntityPartVec, bundle_map::BundleId,
};

impl<I: EntityId + Encode, T: Encode> Encode for EntityBundleMap<I, T> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.bundles.encode(encoder)
    }
}

fn build_map<I: EntityId, T>(
    bundles: EntityMap<BundleId, String, (EntityBundleIndices<I>, T)>,
) -> Result<EntityBundleMap<I, T>, bincode::error::DecodeError> {
    let mut ids = EntityPartVec::new();
    for (bid, _, &(idx, _)) in &bundles {
        match idx {
            EntityBundleIndices::Single(idx) => {
                if ids.insert(idx, bid).is_some() {
                    return Err(bincode::error::DecodeError::Other("duplicate key"));
                }
            }
            EntityBundleIndices::Array(range) => {
                for idx in range {
                    if ids.insert(idx, bid).is_some() {
                        return Err(bincode::error::DecodeError::Other("duplicate key"));
                    }
                }
            }
        }
    }
    Ok(EntityBundleMap { bundles, ids })
}

impl<I: EntityId + Decode<Context>, Context, T: Decode<Context>> Decode<Context>
    for EntityBundleMap<I, T>
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        build_map(Decode::decode(decoder)?)
    }
}

impl<'de, I: EntityId + BorrowDecode<'de, Context>, Context, T: BorrowDecode<'de, Context>>
    BorrowDecode<'de, Context> for EntityBundleMap<I, T>
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        build_map(BorrowDecode::borrow_decode(decoder)?)
    }
}
