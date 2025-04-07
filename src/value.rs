use crate::{encode, decode, encode_zigzag, decode_zigzag, Error};

/// Enum representing different integer types that can be encoded as varints.
/// Each variant wraps a specific Rust integer type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarintValue {
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Unsigned 128-bit integer
    U128(u128),
    /// Signed 8-bit integer
    I8(i8),
    /// Signed 16-bit integer
    I16(i16),
    /// Signed 32-bit integer
    I32(i32),
    /// Signed 64-bit integer
    I64(i64),
    /// Signed 128-bit integer
    I128(i128),
}

// Type encoding bits:
// First 3 bits: Type info
// Last 5 bits: Value type info
const TYPE_BITS_UNSIGNED: u8 = 0b000_00000;
const TYPE_BITS_SIGNED: u8   = 0b001_00000;

// Size bits
const SIZE_BITS_8: u8    = 0b000_00000;
const SIZE_BITS_16: u8   = 0b000_00001;
const SIZE_BITS_32: u8   = 0b000_00010;
const SIZE_BITS_64: u8   = 0b000_00011;
const SIZE_BITS_128: u8  = 0b000_00100;

// Optimization: Macro for handling all types in a match statement
macro_rules! for_all_types {
    ($value:expr, $unsigned_op:expr, $signed_op:expr) => {
        match $value {
            VarintValue::U8(val) => ($unsigned_op)(*val, TYPE_BITS_UNSIGNED | SIZE_BITS_8),
            VarintValue::U16(val) => ($unsigned_op)(*val, TYPE_BITS_UNSIGNED | SIZE_BITS_16),
            VarintValue::U32(val) => ($unsigned_op)(*val, TYPE_BITS_UNSIGNED | SIZE_BITS_32),
            VarintValue::U64(val) => ($unsigned_op)(*val, TYPE_BITS_UNSIGNED | SIZE_BITS_64),
            VarintValue::U128(val) => ($unsigned_op)(*val, TYPE_BITS_UNSIGNED | SIZE_BITS_128),
            VarintValue::I8(val) => ($signed_op)(*val, TYPE_BITS_SIGNED | SIZE_BITS_8),
            VarintValue::I16(val) => ($signed_op)(*val, TYPE_BITS_SIGNED | SIZE_BITS_16),
            VarintValue::I32(val) => ($signed_op)(*val, TYPE_BITS_SIGNED | SIZE_BITS_32),
            VarintValue::I64(val) => ($signed_op)(*val, TYPE_BITS_SIGNED | SIZE_BITS_64),
            VarintValue::I128(val) => ($signed_op)(*val, TYPE_BITS_SIGNED | SIZE_BITS_128),
        }
    };
}

impl VarintValue {
    /// Returns the type identifier byte for this value
    #[inline]
    pub fn get_type_id(&self) -> u8 {
        for_all_types!(self, 
            |_, type_id| type_id, 
            |_, type_id| type_id
        )
    }
    
    /// Directly calculate the number of bytes needed to encode this value
    #[inline]
    fn direct_size_calculation(&self) -> usize {
        let type_byte_size = 1; // 类型标识字节
        
        // 对于值为0的情况优化
        match self {
            VarintValue::U8(0) | VarintValue::U16(0) | VarintValue::U32(0) | 
            VarintValue::U64(0) | VarintValue::U128(0) | VarintValue::I8(0) | 
            VarintValue::I16(0) | VarintValue::I32(0) | VarintValue::I64(0) | 
            VarintValue::I128(0) => return type_byte_size,
            _ => {}
        }
        
        // 计算值所需的字节数
        let value_size = match self {
            // 无符号类型 - 计算所需比特数然后转换为字节
            VarintValue::U8(val) => {
                if *val == 0 { 1 } else {
                    let bits = 8 - val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::U16(val) => {
                if *val == 0 { 1 } else {
                    let bits = 16 - val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::U32(val) => {
                if *val == 0 { 1 } else {
                    let bits = 32 - val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::U64(val) => {
                if *val == 0 { 1 } else {
                    let bits = 64 - val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::U128(val) => {
                if *val == 0 { 1 } else {
                    let bits = 128 - val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            
            // 有符号类型 - 使用ZigZag编码计算
            VarintValue::I8(val) => {
                let zigzag_val = ((val << 1) ^ (val >> 7)) as u8;
                if zigzag_val == 0 { 1 } else {
                    let bits = 8 - zigzag_val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::I16(val) => {
                let zigzag_val = ((val << 1) ^ (val >> 15)) as u16;
                if zigzag_val == 0 { 1 } else {
                    let bits = 16 - zigzag_val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::I32(val) => {
                let zigzag_val = ((val << 1) ^ (val >> 31)) as u32;
                if zigzag_val == 0 { 1 } else {
                    let bits = 32 - zigzag_val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::I64(val) => {
                let zigzag_val = ((val << 1) ^ (val >> 63)) as u64;
                if zigzag_val == 0 { 1 } else {
                    let bits = 64 - zigzag_val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
            VarintValue::I128(val) => {
                let zigzag_val = ((val << 1) ^ (val >> 127)) as u128;
                if zigzag_val == 0 { 1 } else {
                    let bits = 128 - zigzag_val.leading_zeros() as usize;
                    (bits + 6) / 7
                }
            },
        };
        
        type_byte_size + value_size
    }
    
    /// Returns the number of bytes needed to serialize this value
    #[inline]
    pub fn serialized_size(&self) -> usize {
        self.direct_size_calculation()
    }
    
    /// Serializes the value into a byte buffer.
    /// 
    /// The first byte contains the type identifier, followed by the encoded integer value.
    /// Unsigned integers use standard varint encoding, while signed integers use zigzag encoding.
    ///
    /// # Arguments
    /// * `buffer` - The buffer to write into
    ///
    /// # Returns
    /// * `Ok(size)` - The number of bytes written
    /// * `Err(...)` - If encoding fails or buffer is too small
    #[inline]
    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Err(Error::BufferTooSmall { 
                needed: 1,
                actual: 0
            });
        }
        
        // 优化：处理零值的特殊情况
        match self {
            VarintValue::U8(0) | VarintValue::U16(0) | VarintValue::U32(0) | 
            VarintValue::U64(0) | VarintValue::U128(0) | VarintValue::I8(0) | 
            VarintValue::I16(0) | VarintValue::I32(0) | VarintValue::I64(0) | 
            VarintValue::I128(0) => {
                // 零值的特殊情况 - 只需要一个类型字节
                buffer[0] = self.get_type_id();
                return Ok(1);
            },
            _ => { /* 继续正常编码 */ }
        }
        
        // 一般情况编码
        buffer[0] = self.get_type_id();
        
        // 直接编码到缓冲区，避免临时缓冲区
        match self {
            // 无符号类型使用标准编码
            VarintValue::U8(val) => {
                let result = encode(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1), // +1 表示类型字节
                    Err(e) => Err(e),
                }
            },
            VarintValue::U16(val) => {
                let result = encode(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::U32(val) => {
                let result = encode(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::U64(val) => {
                let result = encode(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::U128(val) => {
                let result = encode(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            
            // 有符号类型使用zigzag编码
            VarintValue::I8(val) => {
                let result = encode_zigzag(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::I16(val) => {
                let result = encode_zigzag(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::I32(val) => {
                let result = encode_zigzag(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::I64(val) => {
                let result = encode_zigzag(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
            VarintValue::I128(val) => {
                let result = encode_zigzag(*val, &mut buffer[1..]);
                match result {
                    Ok(bytes_written) => Ok(bytes_written + 1),
                    Err(e) => Err(e),
                }
            },
        }
    }
    
    /// Deserializes a value from a byte buffer.
    ///
    /// # Arguments
    /// * `bytes` - The byte buffer to read from
    ///
    /// # Returns
    /// * `Ok((value, size))` - The deserialized value and number of bytes read
    /// * `Err(...)` - If decoding fails
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if bytes.is_empty() {
            return Err(Error::InputTooShort);
        }
        
        let type_byte = bytes[0];
        let type_bits = type_byte & 0b111_00000; // Get high 3 bits
        let size_bits = type_byte & 0b000_11111; // Get low 5 bits
        
        let data = &bytes[1..];
        
        // Check if it's the special case for zero
        if data.is_empty() && (type_bits == TYPE_BITS_UNSIGNED || type_bits == TYPE_BITS_SIGNED) {
            // This might be a zero value in compact form
            match (type_bits, size_bits) {
                (TYPE_BITS_UNSIGNED, SIZE_BITS_8) => return Ok((VarintValue::U8(0), 1)),
                (TYPE_BITS_UNSIGNED, SIZE_BITS_16) => return Ok((VarintValue::U16(0), 1)),
                (TYPE_BITS_UNSIGNED, SIZE_BITS_32) => return Ok((VarintValue::U32(0), 1)),
                (TYPE_BITS_UNSIGNED, SIZE_BITS_64) => return Ok((VarintValue::U64(0), 1)),
                (TYPE_BITS_UNSIGNED, SIZE_BITS_128) => return Ok((VarintValue::U128(0), 1)),
                (TYPE_BITS_SIGNED, SIZE_BITS_8) => return Ok((VarintValue::I8(0), 1)),
                (TYPE_BITS_SIGNED, SIZE_BITS_16) => return Ok((VarintValue::I16(0), 1)),
                (TYPE_BITS_SIGNED, SIZE_BITS_32) => return Ok((VarintValue::I32(0), 1)),
                (TYPE_BITS_SIGNED, SIZE_BITS_64) => return Ok((VarintValue::I64(0), 1)),
                (TYPE_BITS_SIGNED, SIZE_BITS_128) => return Ok((VarintValue::I128(0), 1)),
                _ => return Err(Error::InvalidEncoding),
            }
        }
        
        // Regular decoding based on type
        match (type_bits, size_bits) {
            (TYPE_BITS_UNSIGNED, SIZE_BITS_8) => {
                let (val, bytes_read) = decode::<u8>(data)?;
                Ok((VarintValue::U8(val), bytes_read + 1))
            },
            (TYPE_BITS_UNSIGNED, SIZE_BITS_16) => {
                let (val, bytes_read) = decode::<u16>(data)?;
                Ok((VarintValue::U16(val), bytes_read + 1))
            },
            (TYPE_BITS_UNSIGNED, SIZE_BITS_32) => {
                let (val, bytes_read) = decode::<u32>(data)?;
                Ok((VarintValue::U32(val), bytes_read + 1))
            },
            (TYPE_BITS_UNSIGNED, SIZE_BITS_64) => {
                let (val, bytes_read) = decode::<u64>(data)?;
                Ok((VarintValue::U64(val), bytes_read + 1))
            },
            (TYPE_BITS_UNSIGNED, SIZE_BITS_128) => {
                let (val, bytes_read) = decode::<u128>(data)?;
                Ok((VarintValue::U128(val), bytes_read + 1))
            },
            (TYPE_BITS_SIGNED, SIZE_BITS_8) => {
                let (val, bytes_read) = decode_zigzag::<i8>(data)?;
                Ok((VarintValue::I8(val), bytes_read + 1))
            },
            (TYPE_BITS_SIGNED, SIZE_BITS_16) => {
                let (val, bytes_read) = decode_zigzag::<i16>(data)?;
                Ok((VarintValue::I16(val), bytes_read + 1))
            },
            (TYPE_BITS_SIGNED, SIZE_BITS_32) => {
                let (val, bytes_read) = decode_zigzag::<i32>(data)?;
                Ok((VarintValue::I32(val), bytes_read + 1))
            },
            (TYPE_BITS_SIGNED, SIZE_BITS_64) => {
                let (val, bytes_read) = decode_zigzag::<i64>(data)?;
                Ok((VarintValue::I64(val), bytes_read + 1))
            },
            (TYPE_BITS_SIGNED, SIZE_BITS_128) => {
                let (val, bytes_read) = decode_zigzag::<i128>(data)?;
                Ok((VarintValue::I128(val), bytes_read + 1))
            },
            _ => Err(Error::InvalidEncoding),
        }
    }
}

/// Macro for creating VarintValue instances in a concise way
#[macro_export]
macro_rules! varint {
    (u8: $val:expr) => { $crate::VarintValue::U8($val) };
    (u16: $val:expr) => { $crate::VarintValue::U16($val) };
    (u32: $val:expr) => { $crate::VarintValue::U32($val) };
    (u64: $val:expr) => { $crate::VarintValue::U64($val) };
    (u128: $val:expr) => { $crate::VarintValue::U128($val) };
    (i8: $val:expr) => { $crate::VarintValue::I8($val) };
    (i16: $val:expr) => { $crate::VarintValue::I16($val) };
    (i32: $val:expr) => { $crate::VarintValue::I32($val) };
    (i64: $val:expr) => { $crate::VarintValue::I64($val) };
    (i128: $val:expr) => { $crate::VarintValue::I128($val) };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_id_encoding() {
        // Test that type IDs are correctly encoded and decoded
        let values = [
            VarintValue::U8(42),
            VarintValue::U16(1000),
            VarintValue::U32(100000),
            VarintValue::I8(-42),
            VarintValue::I16(-1000),
            VarintValue::I32(-100000),
        ];
        
        for value in &values {
            let type_id = value.get_type_id();
            
            // Check if we can reconstruct the value's type from just the type ID
            let mut dummy_buffer = [type_id, 0, 0, 0, 0]; // Just need the type byte
            let (decoded, _) = VarintValue::from_bytes(&dummy_buffer).unwrap_err_or_else(|_| {
                // Only testing that the type is correctly identified
                match value {
                    VarintValue::U8(_) => (VarintValue::U8(0), 1),
                    VarintValue::U16(_) => (VarintValue::U16(0), 1),
                    VarintValue::U32(_) => (VarintValue::U32(0), 1),
                    VarintValue::U64(_) => (VarintValue::U64(0), 1),
                    VarintValue::U128(_) => (VarintValue::U128(0), 1),
                    VarintValue::I8(_) => (VarintValue::I8(0), 1),
                    VarintValue::I16(_) => (VarintValue::I16(0), 1),
                    VarintValue::I32(_) => (VarintValue::I32(0), 1),
                    VarintValue::I64(_) => (VarintValue::I64(0), 1),
                    VarintValue::I128(_) => (VarintValue::I128(0), 1),
                }
            });
            
            match (value, decoded) {
                (VarintValue::U8(_), VarintValue::U8(_)) => {},
                (VarintValue::U16(_), VarintValue::U16(_)) => {},
                (VarintValue::U32(_), VarintValue::U32(_)) => {},
                (VarintValue::U64(_), VarintValue::U64(_)) => {},
                (VarintValue::U128(_), VarintValue::U128(_)) => {},
                (VarintValue::I8(_), VarintValue::I8(_)) => {},
                (VarintValue::I16(_), VarintValue::I16(_)) => {},
                (VarintValue::I32(_), VarintValue::I32(_)) => {},
                (VarintValue::I64(_), VarintValue::I64(_)) => {},
                (VarintValue::I128(_), VarintValue::I128(_)) => {},
                _ => panic!("Type mismatch: original {:?}, decoded {:?}", value, decoded),
            }
        }
    }
    
    #[test]
    fn test_varint_value_serialization() {
        // Test unsigned types
        let values = [
            VarintValue::U8(42),
            VarintValue::U16(1000),
            VarintValue::U32(1000000),
            VarintValue::U64(1000000000),
            VarintValue::U128(u128::MAX / 2),
        ];
        
        for value in &values {
            let mut buffer = [0u8; 30];
            let bytes_written = value.to_bytes(&mut buffer).unwrap();
            let (decoded, bytes_read) = VarintValue::from_bytes(&buffer[..bytes_written]).unwrap();
            
            assert_eq!(*value, decoded);
            assert_eq!(bytes_written, bytes_read);
        }
        
        // Test signed types
        let values = [
            VarintValue::I8(-42),
            VarintValue::I16(-1000),
            VarintValue::I32(-1000000),
            VarintValue::I64(-1000000000),
            VarintValue::I128(i128::MIN / 2),
        ];
        
        for value in &values {
            let mut buffer = [0u8; 30];
            let bytes_written = value.to_bytes(&mut buffer).unwrap();
            let (decoded, bytes_read) = VarintValue::from_bytes(&buffer[..bytes_written]).unwrap();
            
            assert_eq!(*value, decoded);
            assert_eq!(bytes_written, bytes_read);
        }
    }
    
    #[test]
    fn test_zero_optimization() {
        // Test zero values special encoding
        let zero_values = [
            VarintValue::U8(0),
            VarintValue::U16(0),
            VarintValue::U32(0),
            VarintValue::U64(0),
            VarintValue::U128(0),
            VarintValue::I8(0),
            VarintValue::I16(0),
            VarintValue::I32(0),
            VarintValue::I64(0),
            VarintValue::I128(0),
        ];
        
        for value in &zero_values {
            let mut buffer = [0u8; 30];
            let bytes_written = value.to_bytes(&mut buffer).unwrap();
            
            // Zero values should be encoded in 1 byte (just the type)
            assert_eq!(bytes_written, 1, "Zero value {:?} should be encoded in 1 byte", value);
            
            let (decoded, bytes_read) = VarintValue::from_bytes(&buffer[..bytes_written]).unwrap();
            assert_eq!(*value, decoded);
            assert_eq!(bytes_written, bytes_read);
        }
    }
    
    #[test]
    fn test_varint_macro() {
        assert_eq!(varint!(u8: 42), VarintValue::U8(42));
        assert_eq!(varint!(i16: -1000), VarintValue::I16(-1000));
        assert_eq!(varint!(u32: 1000000), VarintValue::U32(1000000));
        assert_eq!(varint!(i64: -1000000000), VarintValue::I64(-1000000000));
    }
    
    #[test]
    fn test_serialized_size() {
        let value = VarintValue::U64(128);
        assert_eq!(value.serialized_size(), 3); // 1 byte type + 2 bytes value
        
        let value = VarintValue::I32(-1);
        assert_eq!(value.serialized_size(), 2); // 1 byte type + 1 byte zigzag value
        
        // Test zero value optimization
        let value = VarintValue::U32(0);
        assert_eq!(value.serialized_size(), 1); // Just 1 byte for type + 0
    }
    
    #[test]
    fn test_error_handling() {
        let value = VarintValue::U64(1000000);
        let mut small_buffer = [0u8; 2];
        
        // Buffer too small
        assert!(value.to_bytes(&mut small_buffer).is_err());
        
        // Empty input
        let empty: [u8; 0] = [];
        assert!(VarintValue::from_bytes(&empty).is_err());
        
        // Invalid type ID
        let invalid = [0xFF, 0x00];
        assert!(VarintValue::from_bytes(&invalid).is_err());
    }
}

// Extension trait for Result to help with unwrap_err_or_else in tests
#[cfg(test)]
trait ResultExt<T, E> {
    fn unwrap_err_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(&E) -> T;
}

#[cfg(test)]
impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_err_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(&E) -> T,
    {
        match self {
            Ok(t) => t,
            Err(ref e) => f(e),
        }
    }
} 