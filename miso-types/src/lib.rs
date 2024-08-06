mod hint;
pub mod u16_prefix_vec;
pub mod u8_prefix_vec;

pub use u16_prefix_vec::U16PrefixVec;
pub use u8_prefix_vec::U8PrefixVec;

use borsh::BorshSerialize;
use std::io::{Result, Write};

/// Helper method that is used to serialize a slice of data (without the length marker).
#[inline]
fn serialize_slice<T: BorshSerialize, W: Write>(data: &[T], writer: &mut W) -> Result<()> {
    if let Some(u8_slice) = T::u8_slice(data) {
        writer.write_all(u8_slice)?;
    } else {
        for item in data {
            item.serialize(writer)?;
        }
    }
    Ok(())
}
