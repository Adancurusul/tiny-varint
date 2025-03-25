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
//! * **Zero-Copy Iterator API**: Memory-efficient processing without buffer allocation
//! * **Basic Encoding Functions**: Low-level functions for direct use
//! * **ZigZag Support**: Efficient encoding of signed integers
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
//! ### Zero-Copy API
//! 
//! ```rust
//! use tiny_varint::{bytes_of, values_from};
//! 
//! // Encode a value into bytes without allocating
//! let value = 16384u64;
//! for byte in bytes_of(value) {
//!     // Process each byte individually
//!     println!("{:02X}", byte);
//! }
//! 
//! // Decode values from a byte buffer without allocating
//! let buffer = [0x80, 0x80, 0x01]; // Encoded value 16384
//! for result in values_from::<u64>(&buffer) {
//!     match result {
//!         Ok(value) => println!("Decoded: {}", value),
//!         Err(e) => println!("Error: {:?}", e),
//!     }
//! }
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
#[cfg(test)]
mod tests;

// Re-export all public items
pub use error::Error;
pub use traits::VarInt;
pub use encoding::{encode, decode, varint_size};
pub use zigzag::{ZigZag, encode_zigzag, decode_zigzag};
pub use batch::{VarIntEncoder, VarIntDecoder, encode_batch, decode_batch};
pub use iter::{VarIntBytesIter, VarIntValuesIter, bytes_of, values_from};
