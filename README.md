# tiny-varint
[![crates.io](https://img.shields.io/crates/v/tiny-varint.svg)](https://crates.io/crates/tiny-varint)
![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
![no_std](https://img.shields.io/badge/no__std-compatible-orange.svg)

A dependency-free, high-performance variable-length integer (VarInt) encoding/decoding Rust library, fully compatible with no_std environments.

## Features

- ✅ **Depends only on zigzag-rs**: Supports `#![no_std]` environments
- ✅ **Full Type Support**: Supports all Rust native integer types
- ✅ **High-performance Implementation**: Optimized critical paths for resource-constrained scenarios
- ✅ **Zero-allocation Interface**: No dynamic memory allocation required, suitable for embedded systems
- ✅ **Zigzag Encoding**: Efficiently handles signed integers
- ✅ **Rich Error Handling**: Detailed error types and messages

## Installation

```toml
[dependencies]
tiny-varint = "0.1.0"
```

## Feature Overview

### Core Functions

| Feature | Function Name | Description |
|---------|---------------|-------------|
| Generic Encoding | `encode<T: VarInt>()` | Encodes any integer type to varint |
| Generic Decoding | `decode<T: VarInt>()` | Decodes a varint to any integer type |
| ZigZag Encoding | `encode_zigzag()` | Encodes signed integers using zigzag |
| ZigZag Decoding | `decode_zigzag()` | Decodes zigzag-encoded signed integers |
| Batch Processing | `VarIntEncoder/VarIntDecoder` | Batch encodes/decodes integer arrays |
| Zero-Copy Encoding | `bytes_of()` | Iterator-based encoding without allocation |
| Zero-Copy Decoding | `values_from()` | Iterator-based decoding without allocation |

### Direct Usage with encode/decode

The simplest way to use tiny-varint is with the generic `encode` and `decode` functions:

```rust
use tiny_varint::{encode, decode};

// Encode an unsigned integer
let mut buffer = [0u8; 10];
let bytes_written = encode(42u64, &mut buffer).unwrap();
println!("Encoded 42 using {} bytes", bytes_written);

// Decode it back
let (decoded_value, bytes_read) = decode::<u64>(&buffer[..bytes_written]).unwrap();
assert_eq!(decoded_value, 42u64);

// Works with any integer type - even signed integers
let bytes_written = encode(-123i32, &mut buffer).unwrap();
let (decoded_value, bytes_read) = decode::<i32>(&buffer[..bytes_written]).unwrap();
assert_eq!(decoded_value, -123i32);
```

### Using Encoders/Decoders

```rust
use tiny_varint::{VarIntEncoder, VarIntDecoder};

// Encoder example
let mut buffer = [0u8; 100];
let mut encoder = VarIntEncoder::new(&mut buffer);

// Write multiple values
encoder.write(123u64)?;
encoder.write(456u32)?;
encoder.write_zigzag(-789i32)?;

// Get encoded bytes
let bytes_written = encoder.position();

// Decoder example
let mut decoder = VarIntDecoder::new(&buffer[..bytes_written]);

// Read values
let v1 = decoder.read::<u64>()?; // 123
let v2 = decoder.read::<u32>()?; // 456
let v3 = decoder.read_zigzag::<i32>()?; // -789
```

### Using Zero-Allocation Iterators

```rust
use tiny_varint::{bytes_of, values_from};

// Encoding iterator - No buffer pre-allocation needed
let value = 16384u64;
for byte in bytes_of(value) {
    // Process each byte individually
    println!("{:02X}", byte);
}

// Decoding iterator - Decode from any byte source
let buffer = [0x80, 0x80, 0x01]; // Encoded value 16384
for result in values_from::<u64>(&buffer) {
    match result {
        Ok(value) => println!("Decoded value: {}", value),
        Err(e) => println!("Decoding error: {:?}", e),
    }
}
```

## Performance Optimizations

tiny-varint has been optimized for various use cases:

- **Avoid Boundary Checks**: Pre-validate buffer sizes to reduce in-loop checks
- **Inline Critical Functions**: Critical paths marked with `#[inline(always)]`
- **Optimized Size Calculation**: Use lookup-based approach to quickly determine encoding size
- **Batch Processing Optimization**: Batch operations avoid multiple boundary checks
- **Zero-Copy Design**: Directly operate on provided buffers without copying data


## Example Code

### Basic Encoding/Decoding

```rust
use tiny_varint::{encode, decode};

// Encode a single u64 value
let mut buffer = [0u8; 10];
let bytes_written = encode(123u64, &mut buffer)?;
println!("Encoded using {} bytes", bytes_written);

// Decode a single u64 value
let (value, bytes_read) = decode::<u64>(&buffer)?;
println!("Decoded value: {}, used {} bytes", value, bytes_read);

// Works with any integer type
let bytes_written = encode(42i32, &mut buffer)?;
let (value, bytes_read) = decode::<i32>(&buffer)?;
assert_eq!(value, 42i32);
```

### ZigZag Encoding Signed Integers

```rust
use tiny_varint::{encode_zigzag, decode_zigzag};

// Encode an i32 value
let mut buffer = [0u8; 10];
let bytes_written = encode_zigzag(-123i32, &mut buffer)?;

// Decode back to i32
let (value, bytes_read) = decode_zigzag::<i32>(&buffer)?;
assert_eq!(value, -123);
```

### Batch Processing

```rust
use tiny_varint::{VarIntEncoder, VarIntDecoder};

// Encode multiple values
let values = [1u64, 10, 100, 1000, 10000];
let mut buffer = [0u8; 50];
let mut encoder = VarIntEncoder::new(&mut buffer);
let bytes_written = encoder.write_batch(&values)?;

// Decode multiple values
let mut decoded = [0u64; 5];
let mut decoder = VarIntDecoder::new(&buffer[..bytes_written]);
decoder.read_batch(&mut decoded)?;
assert_eq!(values, decoded);
```

## Complete Examples

Check the [examples](./examples) directory for more examples:

- [Basic Usage](./examples/basic_usage.rs)
- [Protocol Serialization](./examples/protocol_serialization.rs)
- [Zero-Copy](./examples/zero_copy.rs)

## License

Dual-licensed under MIT/Apache-2.0 licenses. 