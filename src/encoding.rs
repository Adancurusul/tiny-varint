
use crate::error::Error;
use crate::traits::{VarInt, VarIntOps};

/// Encodes arbitrary VarInt type to varint format
///
/// Returns the number of bytes written
///
/// # Parameters
/// * `value` - The value to encode
/// * `buf` - Output buffer
///
/// # Errors
/// Returns `Error::BufferTooSmall` if the buffer is too small
pub fn encode<T: VarInt>(value: T, buf: &mut [u8]) -> Result<usize, Error> {
    let mut val = value.to_unsigned();
    let needed_size = value.varint_size();
    
    if buf.len() < needed_size {
        return Err(Error::BufferTooSmall {
            needed: needed_size,
            actual: buf.len(),
        });
    }
    
    let mut i = 0;
    while val.needs_another_byte() && i < buf.len() - 1 {
        buf[i] = val.get_byte_with_continuation();
        val = val.shift_right_7();
        i += 1;
    }
    
    if i < buf.len() {
        buf[i] = val.get_final_byte();
        Ok(i + 1)
    } else {
        Err(Error::BufferTooSmall {
            needed: i + 1,
            actual: buf.len(),
        })
    }
}

/// Decodes arbitrary VarInt type from varint format
///
/// Returns the decoded value and the number of bytes read
///
/// # Parameters
/// * `buf` - Input buffer containing varint encoding
///
/// # Errors
/// * Returns `Error::InputTooShort` if the input buffer is insufficient
/// * Returns `Error::InvalidEncoding` if the varint encoding is invalid
/// * Returns `Error::Overflow` if overflow occurs during decoding
pub fn decode<T: VarInt>(buf: &[u8]) -> Result<(T, usize), Error> {
    let mut result = T::Unsigned::from_byte(0, 0);
    let mut shift = 0;
    let mut i = 0;
    
    loop {
        if i >= buf.len() {
            return Err(Error::InputTooShort);
        }
        
        let byte = buf[i];
        i += 1;
        
        result = result.bitor(T::Unsigned::from_byte(byte & 0x7F, shift));
        
        // Check if done
        if byte & 0x80 == 0 {
            break;
        }
        
        shift += 1;
        
        // Prevent too large varint
        if shift >= T::Unsigned::BITS / 7 + 1 {
            return Err(Error::Overflow);
        }
    }
    
    Ok((T::from_unsigned(result), i))
}

/// Calculates the number of bytes needed to encode a VarInt value
///
/// # Parameters
/// * `value` - The value to calculate the size for
#[inline(always)]
pub fn varint_size<T: VarInt>(value: T) -> usize {
    value.varint_size()
} 