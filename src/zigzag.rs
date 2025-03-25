use crate::error::Error;
use crate::traits::VarInt;
use crate::encoding::{encode, decode};

/// Trait for ZigZag encoding
pub trait ZigZag: Copy {
    /// Associated unsigned type
    type Unsigned: VarInt;
    
    /// Encode to unsigned integer using ZigZag
    fn zigzag_encode(self) -> Self::Unsigned;
    
    /// Decode from unsigned integer encoded with ZigZag
    fn zigzag_decode(value: Self::Unsigned) -> Self;
}

// Implement ZigZag for all signed integer types
macro_rules! impl_zigzag {
    ($signed:ty, $unsigned:ty, $bits:expr) => {
        impl ZigZag for $signed {
            type Unsigned = $unsigned;
            
            #[inline]
            fn zigzag_encode(self) -> Self::Unsigned {
                ((self << 1) ^ (self >> ($bits - 1))) as $unsigned
            }
            
            #[inline]
            fn zigzag_decode(value: Self::Unsigned) -> Self {
                ((value >> 1) as Self) ^ (-((value & 1) as Self))
            }
        }
    };
}

// Implement for all signed integer types
impl_zigzag!(i8, u8, 8);
impl_zigzag!(i16, u16, 16);
impl_zigzag!(i32, u32, 32);
impl_zigzag!(i64, u64, 64);
impl_zigzag!(i128, u128, 128);

/// Encode a signed integer using ZigZag, then encode it as a varint
///
/// Returns the number of bytes written
///
/// # Arguments
/// * `value` - The signed integer to encode
/// * `buf` - The output buffer to write to
///
/// # Errors
/// * Returns `Error::BufferTooSmall` if the buffer is too small
pub fn encode_zigzag<T: ZigZag>(value: T, buf: &mut [u8]) -> Result<usize, Error> {
    let zigzag = value.zigzag_encode();
    encode(zigzag, buf)
}

/// Decode a signed integer from a varint-encoded zigzag value
///
/// Returns the decoded value and the number of bytes read
///
/// # Arguments
/// * `buf` - The input buffer containing the zigzag varint encoded value
///
/// # Errors
/// * Returns `Error::InputTooShort` if the input buffer is insufficient
/// * Returns `Error::InvalidEncoding` if the varint encoding is invalid
/// * Returns `Error::Overflow` if an overflow occurs during decoding
pub fn decode_zigzag<T: ZigZag>(buf: &[u8]) -> Result<(T, usize), Error> {
    let (unsigned, bytes_read) = decode::<T::Unsigned>(buf)?;
    Ok((T::zigzag_decode(unsigned), bytes_read))
} 