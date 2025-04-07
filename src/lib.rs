#![no_std]

//! # tiny-varint
//!
//! A high-performance variable-length integer (VarInt) encoding/decoding library, 
//! fully compatible with no_std environments.
//!
//! VarInt is an efficient way to encode integers into byte sequences, using fewer 
//! bytes for smaller values. It's widely used in protocol buffers, data compression, 
//! and network transmission scenarios.
//!
//! ## Features
//!
//! * **Generic Integer Support**: Works with all integer types (u8-u128, i8-i128)
//! * **Batch Processing API**: Efficiently handle multiple values with state management
//! * **Iterator-based API**: Memory-efficient processing using iterator methods
//! * **Basic Encoding Functions**: Low-level functions for direct use
//! * **ZigZag Support**: Efficient encoding of signed integers
//! * **Unified Value Type**: VarintValue enum for type-aware encoding/decoding
//! * **No-std Compatible**: Works in embedded environments
//!
//! ## Usage Examples
//!
//! ### Generic API with different integer types
//! 
//! ```rust
//! use tiny_varint::{encode, decode};
//! 
//! // Works with any integer type
//! let mut buf = [0u8; 10];
//! 
//! // Use with u32
//! let bytes_written = encode(42u32, &mut buf).unwrap();
//! let (value, bytes_read) = decode::<u32>(&buf).unwrap();
//! assert_eq!(value, 42u32);
//! 
//! // Use with i16
//! let bytes_written = encode(-42i16, &mut buf).unwrap();
//! let (value, bytes_read) = decode::<i16>(&buf).unwrap();
//! assert_eq!(value, -42i16);
//! ```
//!
//! ### Batch API
//! 
//! ```rust
//! use tiny_varint::{VarIntEncoder, VarIntDecoder};
//! 
//! // Encode multiple values
//! let values = [1u64, 127, 128, 16383, 16384];
//! let mut buffer = [0u8; 100];
//! 
//! let mut encoder = VarIntEncoder::new(&mut buffer);
//! let bytes_written = encoder.write_batch(&values).unwrap();
//! 
//! // Decode multiple values
//! let mut decoded = [0u64; 5];
//! let mut decoder = VarIntDecoder::new(&buffer[..bytes_written]);
//! decoder.read_batch(&mut decoded).unwrap();
//! ```
//! 
//! ### Iterator-based API
//! 
//! ```rust
//! use tiny_varint::{bytes_of, values_from};
//! 
//! // Encode a value into bytes using iterator methods
//! let value = 16384u64;
//! for byte in bytes_of(value) {
//!     // Process each byte individually
//!     println!("{:02X}", byte);
//! }
//! 
//! // Decode values from a byte buffer using iterator methods
//! let buffer = [0x80, 0x80, 0x01]; // Encoded value 16384
//! for result in values_from::<u64>(&buffer) {
//!     match result {
//!         Ok(value) => println!("Decoded: {}", value),
//!         Err(e) => println!("Error: {:?}", e),
//!     }
//! }
//! ```
//!
//! ### VarintValue for mixed type data
//!
//! ```rust
//! use tiny_varint::{VarintValue, varint};
//!
//! // Create values of different types
//! let values = [
//!     varint!(u32: 42),
//!     varint!(i16: -100),
//!     varint!(u64: 1000000)
//! ];
//!
//! // Serialize each value
//! let mut buffer = [0u8; 100];
//! let mut pos = 0;
//!
//! for value in &values {
//!     let bytes_written = value.to_bytes(&mut buffer[pos..]).unwrap();
//!     pos += bytes_written;
//! }
//!
//! // Deserialize values
//! let mut read_pos = 0;
//! let mut results = Vec::new();
//!
//! while read_pos < pos {
//!     let (value, bytes_read) = VarintValue::from_bytes(&buffer[read_pos..pos]).unwrap();
//!     results.push(value);
//!     read_pos += bytes_read;
//! }
//!
//! // Values are preserved with their original types
//! assert_eq!(results[0], varint!(u32: 42));
//! assert_eq!(results[1], varint!(i16: -100));
//! assert_eq!(results[2], varint!(u64: 1000000));
//! ```

// Import zigzag-rs for ZigZag encoding/decoding
extern crate zigzag_rs;

// Define modules
mod error;
mod traits;
mod encoding;
mod batch;
mod iter;
mod zigzag;
mod value;
#[cfg(test)]
mod tests;

// Re-export all public items
pub use error::Error;
pub use traits::VarInt;
pub use encoding::{encode, decode, varint_size};
pub use zigzag::{ZigZag, encode_zigzag, decode_zigzag};
pub use batch::{VarIntEncoder, VarIntDecoder, encode_batch, decode_batch};
pub use iter::{VarIntBytesIter, VarIntValuesIter, bytes_of, values_from};
pub use value::VarintValue;
// varint! macro is re-exported via #[macro_export]
