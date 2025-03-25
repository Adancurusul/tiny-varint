//! Protocol Serialization Example
//! 
//! This example demonstrates how to combine tiny-varint with custom protocol serialization
//! Using APIs that don't require dynamic memory allocation
//! Run with: cargo run --example protocol_serialization

use tiny_varint::{
    encode_zigzag,
    VarIntEncoder, VarIntDecoder, Error, varint_size
};

/// A simple message structure that can be serialized and deserialized
#[derive(Debug, PartialEq)]
struct SimpleMessage {
    // Message ID encoded as varint (typically small numbers)
    message_id: u64,
    
    // Temperature value encoded using zigzag (can be positive or negative)
    temperature: i16,
    
    // Humidity value encoded using zigzag (0-100 positive, but using zigzag for consistency)
    humidity: i8,
    
    // Payload data
    payload: [u8; 16],
}

impl SimpleMessage {
    /// Calculate the serialized size
    fn serialized_size(&self) -> usize {
        // Number of bytes needed
        let id_size = varint_size(self.message_id);
        
        // Create temporary buffers for temperature and humidity to calculate size
        let mut temp_buf = [0u8; 10];
        let temp_size = encode_zigzag(self.temperature, &mut temp_buf).unwrap();
        
        let mut hum_buf = [0u8; 10];
        let humidity_size = encode_zigzag(self.humidity, &mut hum_buf).unwrap();
        
        // Payload size needs a length field + the payload itself
        let payload_len_size = varint_size(self.payload.len() as u64);
        
        id_size + temp_size + humidity_size + payload_len_size + self.payload.len()
    }
    
    /// Serialize message to byte array
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        // Use VarIntEncoder for serialization
        let mut encoder = VarIntEncoder::<u32>::new(buffer);
        
        // Serialize message ID (varint) - using u32 implementation instead of u64
        encoder.write(self.message_id as u32)?;
        
        // Serialize temperature (zigzag varint)
        encoder.write_zigzag(self.temperature)?;
        
        // Serialize humidity (zigzag varint)
        encoder.write_zigzag(self.humidity)?;
        
        // Serialize payload length
        encoder.write(self.payload.len() as u32)?;
        
        // Write payload bytes
        let current_offset = encoder.position();
        if current_offset + self.payload.len() > buffer.len() {
            return Err(Error::BufferTooSmall {
                needed: current_offset + self.payload.len(),
                actual: buffer.len(),
            });
        }
        
        buffer[current_offset..current_offset + self.payload.len()]
            .copy_from_slice(&self.payload);
            
        Ok(current_offset + self.payload.len())
    }
    
    /// Deserialize message from byte array
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Error> {
        let mut decoder = VarIntDecoder::<u32>::new(bytes);
        
        // Decode message ID
        let message_id = decoder.read()? as u64;
        
        // Decode temperature
        let temperature = decoder.read_zigzag::<i16>()?;
        
        // Decode humidity
        let humidity = decoder.read_zigzag::<i8>()?;
        
        // Decode payload length
        let payload_len = decoder.read()? as usize;
        if payload_len != 16 {
            return Err(Error::InvalidEncoding);
        }
        
        // Decode payload
        let current_offset = decoder.position();
        if current_offset + payload_len > bytes.len() {
            return Err(Error::InputTooShort);
        }
        
        let mut payload = [0u8; 16];
        payload.copy_from_slice(&bytes[current_offset..current_offset + payload_len]);
        
        let final_offset = current_offset + payload_len;
        
        let msg = SimpleMessage {
            message_id,
            temperature,
            humidity,
            payload,
        };
        
        Ok((msg, final_offset))
    }
}

fn main() {
    println!("=== Protocol Serialization Example ===\n");
    
    // Create an example message
    let mut payload = [0u8; 16];
    for i in 0..payload.len() {
        payload[i] = (i * 10) as u8;
    }
    
    let original_msg = SimpleMessage {
        message_id: 42,
        temperature: -15,
        humidity: 85,
        payload,
    };
    
    println!("Original message: {:#?}", original_msg);
    
    // Calculate serialization size
    let expected_size = original_msg.serialized_size();
    println!("Expected serialized size: {} bytes", expected_size);
    
    // Serialize message
    let mut buffer = [0u8; 100];
    let serialize_result = original_msg.serialize(&mut buffer);
    
    match serialize_result {
        Ok(size) => {
            println!("Serialization successful, used {} bytes", size);
            
            print!("Serialized bytes: [ ");
            for i in 0..size {
                print!("{:#04x} ", buffer[i]);
            }
            println!("]");
            
            // Deserialize message
            match SimpleMessage::deserialize(&buffer[0..size]) {
                Ok((decoded_msg, decoded_size)) => {
                    println!("\nDecoded message: {:#?}", decoded_msg);
                    println!("Decoded byte size: {} bytes", decoded_size);
                    
                    println!("\nOriginal message and decoded message match: {}", 
                        original_msg == decoded_msg);
                }
                Err(e) => {
                    println!("Decoding failed: {:?}", e);
                }
            }
            
            // Space efficiency analysis
            println!("\nSpace efficiency analysis:");
            let fixed_size = 8 + 2 + 1 + 16; // u64 + i16 + i8 + payload
            println!("  Using fixed-size encoding: {} bytes", fixed_size);
            println!("  Using varint encoding: {} bytes", size);
            println!("  Savings: {} bytes ({:.1}%)", 
                fixed_size - size, 
                ((fixed_size - size) as f32 / fixed_size as f32) * 100.0);
        }
        Err(e) => {
            println!("Serialization failed: {:?}", e);
        }
    }
    
    println!("\n=== Example Complete ===");
} 