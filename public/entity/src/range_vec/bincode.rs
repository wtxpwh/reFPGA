use bincode::{BorrowDecode, Decode, Encode};

use crate::{EntityIdArith, EntityRangeVec};

impl<I: EntityIdArith + Encode, T: Encode> Encode for EntityRangeVec<I, T> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.ranges.len().encode(encoder)?;
        for (r, val) in self {
            r.len().encode(encoder)?;
            val.encode(encoder)?;
        }
        Ok(())
    }
}

fn build_vec<I: EntityIdArith, T>(
    data: Vec<(usize, T)>,
) -> Result<EntityRangeVec<I, T>, bincode::error::DecodeError> {
    let mut res = EntityRangeVec::new();
    for (num, val) in data {
        res.push(num, val);
    }
    Ok(res)
}

impl<I: EntityIdArith + Decode<Context>, Context, T: Decode<Context>> Decode<Context>
    for EntityRangeVec<I, T>
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        build_vec(Decode::decode(decoder)?)
    }
}

impl<'de, I: EntityIdArith + BorrowDecode<'de, Context>, Context, T: BorrowDecode<'de, Context>>
    BorrowDecode<'de, Context> for EntityRangeVec<I, T>
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        build_vec(BorrowDecode::borrow_decode(decoder)?)
    }
}
