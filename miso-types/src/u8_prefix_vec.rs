use borsh::{BorshDeserialize, BorshSerialize};
use std::{
    io::{ErrorKind, Read, Result, Write},
    mem::{forget, size_of},
};

use crate::{hint, serialize_slice};

pub struct U8PrefixVec<T>(pub Vec<T>);

impl<T> From<Vec<T>> for U8PrefixVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> BorshSerialize for U8PrefixVec<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(
            &(u8::try_from(self.0.len()).map_err(|_| ErrorKind::InvalidInput)?).to_le_bytes(),
        )?;
        serialize_slice(&self.0, writer)
    }
}

impl<T> BorshDeserialize for U8PrefixVec<T>
where
    T: BorshDeserialize,
{
    #[inline]
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let len = u8::deserialize_reader(reader)?;
        if len == 0 {
            Ok(Vec::new().into())
        } else if let Some(vec_bytes) = T::vec_from_reader(len as u32, reader)? {
            Ok(vec_bytes.into())
        } else if size_of::<T>() == 0 {
            let mut result = vec![T::deserialize_reader(reader)?];

            let p = result.as_mut_ptr();
            unsafe {
                forget(result);
                let len: usize = len.into();
                let result = Vec::from_raw_parts(p, len, len);
                Ok(result.into())
            }
        } else {
            // TODO(16): return capacity allocation when we can safely do that.
            let mut result = Vec::with_capacity(hint::cautious::<T>(len as u32));
            for _ in 0..len {
                result.push(T::deserialize_reader(reader)?);
            }
            Ok(result.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_u8s() {
        let result = U8PrefixVec::<u8>::from(vec![1, 2, 3, 4])
            .try_to_vec()
            .unwrap();
        assert_eq!(result, vec![4, 1, 2, 3, 4]);
    }

    #[test]
    fn it_serializes_u16s() {
        let result = U8PrefixVec::<u16>::from(vec![1, 2, 3, 4])
            .try_to_vec()
            .unwrap();
        assert_eq!(result, vec![4, 1, 0, 2, 0, 3, 0, 4, 0]);
    }

    #[test]
    fn it_deserializes_u8s() {
        let result = U8PrefixVec::<u8>::try_from_slice(&[4, 1, 2, 3, 4]).unwrap();
        assert_eq!(result.0, vec![1, 2, 3, 4]);
    }

    #[test]
    fn it_deserializes_u16s() {
        let result = U8PrefixVec::<u16>::try_from_slice(&[4, 1, 0, 2, 0, 3, 0, 4, 0]).unwrap();
        assert_eq!(result.0, vec![1, 2, 3, 4]);
    }
}
