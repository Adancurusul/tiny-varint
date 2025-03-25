//! Iterator-based Example
//! 
//! This example demonstrates how to use tiny-varint's iterator-based API
//! This is particularly useful in embedded or resource-constrained environments
//! Run with: cargo run --example zero_copy

extern crate tiny_varint;

use tiny_varint::{
    bytes_of, values_from,
    VarIntEncoder,
    encode_zigzag
};

fn main() {
    println!("=== Iterator-based API Example ===\n");
    
    // 1. Encoding a single value using VarIntBytesIter iterator
    println!("1. Encoding a single value using VarIntBytesIter iterator:");
    let value = 16384u64;
    
    println!("  Encoding value: {}", value);
    println!("  Expected bytes: {}", tiny_varint::varint_size(value));
    
    print!("  Encoded bytes: [ ");
    for byte in bytes_of(value) {
        print!("{:#04x} ", byte);
    }
    println!("]");
    
    // Manually verify the result
    let mut manual_buf = [0u8; 10];
    let manual_size = tiny_varint::encode(value, &mut manual_buf).unwrap();
    
    print!("  Verification bytes: [ ");
    for i in 0..manual_size {
        print!("{:#04x} ", manual_buf[i]);
    }
    println!("]\n");
    
    // 2. Decoding using VarIntValuesIter iterator
    println!("2. Decoding using VarIntValuesIter iterator:");
    
    // Create a buffer containing multiple varints
    let values = [0u64, 1, 127, 128, 16383, 16384, 2097151];
    let mut buffer = [0u8; 100];
    
    let mut encoder = VarIntEncoder::new(&mut buffer);
    let bytes_written = encoder.write_batch(&values).unwrap();
    
    print!("  Encoded bytes: [ ");
    for i in 0..bytes_written {
        print!("{:#04x} ", buffer[i]);
    }
    println!("]");
    
    println!("  Decoded values:");
    for result in values_from::<u64>(&buffer[..bytes_written]) {
        match result {
            Ok(value) => println!("    - {}", value),
            Err(e) => println!("    - Error: {:?}", e),
        }
    }
    
    // 3. Processing zigzag-encoded values without intermediate buffers
    println!("\n3. Processing zigzag-encoded signed values:");
    
    let signed_values = [-100i32, -50, -10, -1, 0, 1, 10, 50, 100];
    println!("  Original values: {:?}", signed_values);
    
    // Create a zigzag-encoded byte iterator for each value
    for &value in &signed_values {
        // First apply zigzag encoding
        let mut buf = [0u8; 10];
        let size = encode_zigzag(value, &mut buf).unwrap();
        
        print!("  ZigZag encoding for {}: [ ", value);
        for i in 0..size {
            print!("{:#04x} ", buf[i]);
        }
        println!("]");
    }
    
    // 4. Combining iterators with encoders/decoders
    println!("\n4. Combining iterators with encoders/decoders in stream processing:");
    
    // Simulate a simple data processing pipeline
    println!("  Data processing pipeline:");
    println!("    1. Generate raw data");
    println!("    2. Encode using iterator-based methods");
    println!("    3. Process encoded bytes");
    println!("    4. Decode using iterator-based methods");
    
    // Raw data
    let source_data = [42u64, 314, 2718, 31415];
    println!("\n  Raw data: {:?}", source_data);
    
    // Encode and collect
    let mut encoded_bytes = Vec::new();
    
    for &value in &source_data {
        let encoder = bytes_of(value);
        println!("  Encoding {} -> {} bytes", value, encoder.size());
        
        // Add bytes to our buffer
        for byte in encoder {
            encoded_bytes.push(byte);
        }
    }
    
    print!("  All encoded bytes: [ ");
    for byte in &encoded_bytes {
        print!("{:#04x} ", byte);
    }
    println!("]");
    
    // Decode bytes
    println!("\n  Decoding results:");
    let decoder = values_from::<u64>(&encoded_bytes);
    
    let mut i = 0;
    for result in decoder {
        match result {
            Ok(value) => {
                println!("    Original: {}, Decoded: {}", source_data[i], value);
                i += 1;
            }
            Err(e) => println!("    Error: {:?}", e),
        }
    }
    
    println!("\n=== Example Complete ===");
} 