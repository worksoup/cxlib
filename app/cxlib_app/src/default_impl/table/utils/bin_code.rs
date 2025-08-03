use bincode::{decode_from_slice, encode_to_vec};
use redb::{TypeName, Value};
use std::{cmp::Ordering, fmt::Debug, ops::Deref};

#[derive(Debug)]
pub struct BinCode<Content>(Content);

impl<T> Deref for BinCode<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<T> for BinCode<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self(data)
    }
}

impl<T> BinCode<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Value for BinCode<T>
where
    T: Debug + bincode::Decode<()> + bincode::Encode,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    #[inline]
    fn fixed_width() -> Option<usize> {
        None
    }

    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    #[inline]
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    #[inline]
    fn type_name() -> TypeName {
        TypeName::new(&format!("BinCode<{}>", std::any::type_name::<T>()))
    }
}

impl<K> redb::Key for BinCode<K>
where
    K: Debug + bincode::Decode<()> + bincode::Encode + Ord,
{
    #[inline]
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
