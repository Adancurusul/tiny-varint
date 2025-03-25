/// Generic trait for variable-length integer encoding
pub trait VarInt: Copy + Sized {
    /// The corresponding unsigned type used for internal encoding operations
    type Unsigned: Copy + VarIntOps;
    
    /// Convert to the corresponding unsigned type
    fn to_unsigned(self) -> Self::Unsigned;
    
    /// Convert from the corresponding unsigned type
    fn from_unsigned(value: Self::Unsigned) -> Self;
    
    /// Determine how many bytes are needed to encode this value
    fn varint_size(self) -> usize;
}

/// Operations trait for unsigned types
pub trait VarIntOps: Copy + Sized {
    /// Maximum number of bits for this type
    const BITS: usize;
    
    /// Check if another byte is needed for encoding (value >= 0x80)
    fn needs_another_byte(self) -> bool;
    
    /// Get a byte from the value and set the continuation bit
    fn get_byte_with_continuation(self) -> u8;
    
    /// Get the final byte from the value
    fn get_final_byte(self) -> u8;
    
    /// Shift right by 7 bits
    fn shift_right_7(self) -> Self;
    
    /// Build a value from a byte
    fn from_byte(byte: u8, shift: usize) -> Self;
    
    /// Get the number of leading zeros
    fn leading_zeros(self) -> usize;
    
    /// Bitwise OR operation
    fn bitor(self, other: Self) -> Self;
}

// Implement VarIntOps for unsigned types
macro_rules! impl_varint_ops {
    ($type:ty, $bits:expr) => {
        impl VarIntOps for $type {
            const BITS: usize = $bits;
            
            #[inline]
            fn needs_another_byte(self) -> bool {
                self >= 0x80
            }
            
            #[inline]
            fn get_byte_with_continuation(self) -> u8 {
                (self as u8) | 0x80
            }
            
            #[inline]
            fn get_final_byte(self) -> u8 {
                self as u8
            }
            
            #[inline]
            fn shift_right_7(self) -> Self {
                self >> 7
            }
            
            #[inline]
            fn from_byte(byte: u8, shift: usize) -> Self {
                ((byte & 0x7F) as Self) << ((shift * 7) as u32)
            }
            
            #[inline]
            fn leading_zeros(self) -> usize {
                self.leading_zeros() as usize
            }
            
            #[inline]
            fn bitor(self, other: Self) -> Self {
                self | other
            }
        }
    };
}

// Implement VarInt for unsigned types
macro_rules! impl_unsigned_varint {
    ($type:ty, $bits:expr) => {
        impl VarInt for $type {
            type Unsigned = Self;
            
            #[inline]
            fn to_unsigned(self) -> Self::Unsigned {
                self
            }
            
            #[inline]
            fn from_unsigned(value: Self::Unsigned) -> Self {
                value
            }
            
            #[inline]
            fn varint_size(self) -> usize {
                if self == 0 {
                    return 1;
                }
                let bits_needed = Self::Unsigned::BITS - self.leading_zeros();
                ((bits_needed + 6) / 7) as usize // 7 bits per byte, round up
            }
        }
    };
}

// Implement VarInt for signed types
macro_rules! impl_signed_varint {
    ($type:ty, $unsigned:ty) => {
        impl VarInt for $type {
            type Unsigned = $unsigned;
            
            #[inline]
            fn to_unsigned(self) -> Self::Unsigned {
                self as $unsigned
            }
            
            #[inline]
            fn from_unsigned(value: Self::Unsigned) -> Self {
                value as Self
            }
            
            #[inline]
            fn varint_size(self) -> usize {
                // For signed types, calculate size based on actual bit pattern
                let value = self.to_unsigned();
                
                if value == 0 {
                    return 1;
                }
                let bits_needed = Self::Unsigned::BITS - value.leading_zeros();
                ((bits_needed + 6) / 7) as usize
            }
        }
    };
}

// Implement for all integer types
impl_varint_ops!(u8, 8);
impl_varint_ops!(u16, 16);
impl_varint_ops!(u32, 32);
impl_varint_ops!(u64, 64);
impl_varint_ops!(u128, 128);

impl_unsigned_varint!(u8, 8);
impl_unsigned_varint!(u16, 16);
impl_unsigned_varint!(u32, 32);
impl_unsigned_varint!(u64, 64);
impl_unsigned_varint!(u128, 128);

impl_signed_varint!(i8, u8);
impl_signed_varint!(i16, u16);
impl_signed_varint!(i32, u32);
impl_signed_varint!(i64, u64);
impl_signed_varint!(i128, u128); 