//! Basic Usage Example (no-alloc version)
//! 
//! This example demonstrates the basic encoding and decoding functionality of the tiny-varint library,
//! using APIs that don't require dynamic memory allocation
//! Run with: cargo run --example basic_usage

extern crate tiny_varint;

use tiny_varint::{
    encode, decode,
    encode_zigzag, decode_zigzag,
    VarIntEncoder, VarIntDecoder,
    Error
};

fn main() {
    println!("=== tiny-varint Basic Example (no-alloc version) ===\n");
    
    // 1. Single unsigned integer encoding/decoding
    println!("1. Encoding and decoding single unsigned integers:");
    let values = [0u64, 127, 128, 16383, 16384, 2097151];
    
    for &value in &values {
        let mut buf = [0u8; 10];
        let bytes_written = encode(value, &mut buf).unwrap();
        
        print!("  Value {} encoded as {} bytes: [ ", value, bytes_written);
        for i in 0..bytes_written {
            print!("{:#04x} ", buf[i]);
        }
        println!("]");
        
        let (decoded, bytes_read) = decode::<u64>(&buf).unwrap();
        println!("  Decoded: {} (read {} bytes)", decoded, bytes_read);
        println!();
    }
    
    // 2. Signed integer zigzag encoding/decoding
    println!("\n2. ZigZag encoding and decoding of signed integers:");
    let signed_values = [0i32, 1, -1, 2, -2, 127, -127, 128, -128];
    
    for &value in &signed_values {
        let mut buf = [0u8; 10];
        let bytes_written = encode_zigzag(value, &mut buf).unwrap();
        
        print!("  Value {} encoded as {} bytes: [ ", value, bytes_written);
        for i in 0..bytes_written {
            print!("{:#04x} ", buf[i]);
        }
        println!("]");
        
        let (decoded, bytes_read) = decode_zigzag::<i32>(&buf).unwrap();
        println!("  Decoded: {} (read {} bytes)", decoded, bytes_read);
        println!();
    }
    
    // 3. Batch processing using VarIntEncoder/VarIntDecoder
    println!("\n3. Batch processing using VarIntEncoder/VarIntDecoder:");
    let batch_values = [1u64, 127, 128, 16383, 16384, 2097151];
    let mut buffer = [0u8; 100];
    
    // Using encoder
    let mut encoder = VarIntEncoder::new(&mut buffer);
    let bytes_written = encoder.write_batch(&batch_values).unwrap();
    
    println!("  Encoded {} values using {} bytes", batch_values.len(), bytes_written);
    
    print!("  Encoded bytes: [ ");
    for i in 0..bytes_written {
        print!("{:#04x} ", buffer[i]);
    }
    println!("]");
    
    // Using decoder
    let mut decoder = VarIntDecoder::new(&buffer[..bytes_written]);
    let mut decoded = [0u64; 6];
    let count = decoder.read_batch(&mut decoded).unwrap();
    
    print!("  Decoded values: [ ");
    for i in 0..count {
        print!("{} ", decoded[i]);
    }
    println!("]");
    println!("  Read {} bytes in total", decoder.position());
    
    // 4. Using different integer types
    println!("\n4. Using different integer types:");
    
    // Using u64 type
    let value = 16384u64;
    let mut buf = [0u8; 10];
    let size = encode(value, &mut buf).unwrap();
    
    print!("  u64 value {} encoded as {} bytes: [ ", value, size);
    for i in 0..size {
        print!("{:#04x} ", buf[i]);
    }
    println!("]");
    
    let (decoded, bytes_read) = decode::<u64>(&buf).unwrap();
    println!("  Decoded: {} (read {} bytes)", decoded, bytes_read);
    
    // Using u32 type
    let u32_value = 42u32;
    let mut buf = [0u8; 10];
    let size = encode(u32_value, &mut buf).unwrap();
    
    print!("  u32 value {} encoded as {} bytes: [ ", u32_value, size);
    for i in 0..size {
        print!("{:#04x} ", buf[i]);
    }
    println!("]");
    
    let (decoded, bytes_read) = decode::<u32>(&buf).unwrap();
    println!("  Decoded: {} (read {} bytes)", decoded, bytes_read);
    
    // Using i16 type
    let i16_value = -256i16;
    let size = encode(i16_value, &mut buf).unwrap();
    
    print!("  i16 value {} encoded as {} bytes: [ ", i16_value, size);
    for i in 0..size {
        print!("{:#04x} ", buf[i]);
    }
    println!("]");
    
    let (decoded, bytes_read) = decode::<i16>(&buf).unwrap();
    println!("  Decoded: {} (read {} bytes)", decoded, bytes_read);
    
    // 5. Error handling
    println!("\n5. Error handling:");
    let large_value: u64 = 0xFFFFFFFFFFFFFFFF; // Requires 10 bytes
    let mut small_buf = [0u8; 5];
    
    match encode(large_value, &mut small_buf) {
        Ok(_) => println!("  Encoding successful (should not reach here)"),
        Err(Error::BufferTooSmall { needed, actual }) => {
            println!("  Buffer too small: needed {} bytes, but only had {} bytes", needed, actual);
        }
        Err(e) => println!("  Error occurred: {:?}", e),
    }
    
    // Create an incomplete varint - flag bit is 1 but no following bytes
    let incomplete_buf = [0x80];
    
    match decode::<u64>(&incomplete_buf) {
        Ok(_) => println!("  Decoding successful (should not reach here)"),
        Err(Error::InputTooShort) => {
            println!("  Input too short: high bit is 1 indicating more bytes follow, but data ended");
        }
        Err(e) => println!("  Error occurred: {:?}", e),
    }
    
    println!("\n=== Example Complete ===");
} 