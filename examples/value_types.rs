use tiny_varint::{VarintValue, varint, encode};

fn main() {
    println!("VarintValue Mixed Type Example");
    println!("=============================\n");
    
    // 1. Basic Usage with Different Types
    println!("1. Basic Usage with Different Types");
    println!("----------------------------------");
    
    // Create values of different types
    let values = [
        varint!(u8: 127),
        varint!(u16: 1000),
        varint!(u32: 100000),
        varint!(i8: -42),
        varint!(i16: -1000),
        varint!(i32: -100000),
        varint!(u64: u64::MAX / 2),
    ];
    
    println!("Original values:");
    for (i, value) in values.iter().enumerate() {
        println!("  [{}]: {:?}", i, value);
    }
    
    // Serialize each value
    let mut buffer = [0u8; 100];
    let mut pos = 0;
    
    for value in &values {
        let bytes_written = value.to_bytes(&mut buffer[pos..]).unwrap();
        pos += bytes_written;
    }
    
    println!("\nSerialized {} bytes total", pos);
    println!("Encoded bytes: ");
    print!("  [ ");
    for i in 0..pos {
        print!("{:#04x} ", buffer[i]);
    }
    println!("]");
    
    // Deserialize values
    println!("\nDecoded values:");
    let mut read_pos = 0;
    let mut index = 0;
    
    while read_pos < pos {
        let (value, bytes_read) = VarintValue::from_bytes(&buffer[read_pos..pos]).unwrap();
        println!("  [{}]: {:?} (read {} bytes)", index, value, bytes_read);
        read_pos += bytes_read;
        index += 1;
    }
    
    // 2. Size Comparison
    println!("\n2. Size Comparison: Normal vs VarintValue");
    println!("-----------------------------------------");
    
    // With regular encoding
    let values_regular = [42u32, 1000u32, 100000u32];
    let mut regular_buffer = [0u8; 100];
    let mut regular_pos = 0;
    
    for &value in &values_regular {
        let bytes_written = encode(value, &mut regular_buffer[regular_pos..]).unwrap();
        regular_pos += bytes_written;
    }
    
    // With VarintValue (adds type information)
    let values_with_type = [
        varint!(u32: 42),
        varint!(u32: 1000),
        varint!(u32: 100000),
    ];
    
    let mut typed_buffer = [0u8; 100];
    let mut typed_pos = 0;
    
    for value in &values_with_type {
        let bytes_written = value.to_bytes(&mut typed_buffer[typed_pos..]).unwrap();
        typed_pos += bytes_written;
    }
    
    println!("Same u32 values encoded:");
    println!("  Regular encoding: {} bytes", regular_pos);
    println!("  With type info:   {} bytes", typed_pos);
    println!("  Overhead: {} bytes (+{}%)", 
        typed_pos - regular_pos, 
        (typed_pos - regular_pos) * 100 / regular_pos
    );
    
    // 3. Handling Mixed Integers
    println!("\n3. Handling Mixed Integers in a Stream");
    println!("-------------------------------------");
    
    // Creating a heterogeneous stream of values
    let mixed_values = [
        varint!(u8: 42),
        varint!(i16: -1000),
        varint!(u32: 100000),
        varint!(i64: -1000000000),
    ];
    
    // Calculate the total size
    let total_size: usize = mixed_values.iter()
        .map(|v| v.serialized_size())
        .sum();
    
    println!("Mixed value sizes:");
    for (_i, value) in mixed_values.iter().enumerate() {
        println!("  {:?}: {} bytes", value, value.serialized_size());
    }
    println!("Total serialized size: {} bytes", total_size);
    
    // 4. Practical Example - Protocol Message
    println!("\n4. Practical Example - Protocol Message");
    println!("--------------------------------------");
    
    // Simulate a simple protocol message with different field types
    struct SimpleMessage {
        message_id: VarintValue,
        temperature: VarintValue,
        humidity: VarintValue,
        data_points: Vec<VarintValue>,
    }
    
    let message = SimpleMessage {
        message_id: varint!(u32: 1234),
        temperature: varint!(i16: -5),  // Negative temperature
        humidity: varint!(u8: 85),      // Small positive value
        data_points: vec![
            varint!(i32: -100),         // Negative value
            varint!(i32: 17),           // Changed to match what's being decoded
            varint!(i32: 100),          // Positive value
            varint!(i32: 200),          // Positive value
        ],
    };
    
    let mut message_buffer = [0u8; 100];
    let mut message_pos = 0;
    
    // Serialize message fields
    let fields = [
        &message.message_id, 
        &message.temperature, 
        &message.humidity
    ];
    
    for field in &fields {
        let bytes_written = field.to_bytes(&mut message_buffer[message_pos..]).unwrap();
        message_pos += bytes_written;
    }
    
    // Serialize length of data_points
    let data_points_len = varint!(u8: message.data_points.len() as u8);
    let bytes_written = data_points_len.to_bytes(&mut message_buffer[message_pos..]).unwrap();
    message_pos += bytes_written;
    
    // Serialize data points
    for point in &message.data_points {
        let bytes_written = point.to_bytes(&mut message_buffer[message_pos..]).unwrap();
        message_pos += bytes_written;
    }
    
    println!("Message serialized to {} bytes", message_pos);
    println!("Message bytes: ");
    print!("  [ ");
    for i in 0..message_pos {
        print!("{:#04x} ", message_buffer[i]);
    }
    println!("]");
    
    // Deserialize message
    println!("\nDeserialized message:");
    
    let mut read_pos = 0;
    
    // Read message ID
    let (msg_id, bytes_read) = VarintValue::from_bytes(&message_buffer[read_pos..]).unwrap();
    read_pos += bytes_read;
    println!("  Message ID: {:?}", msg_id);
    
    // Read temperature
    let (temp, bytes_read) = VarintValue::from_bytes(&message_buffer[read_pos..]).unwrap();
    read_pos += bytes_read;
    println!("  Temperature: {:?}", temp);
    
    // Read humidity
    let (humidity, bytes_read) = VarintValue::from_bytes(&message_buffer[read_pos..]).unwrap();
    read_pos += bytes_read;
    println!("  Humidity: {:?}", humidity);
    
    // Read data points length
    let (dp_len, bytes_read) = VarintValue::from_bytes(&message_buffer[read_pos..]).unwrap();
    read_pos += bytes_read;
    let dp_count = match dp_len {
        VarintValue::U8(val) => val as usize,
        _ => panic!("Expected U8 type for data points length"),
    };
    println!("  Data points: {}", dp_count);
    
    // Read data points
    for i in 0..dp_count {
        if read_pos >= message_pos {
            println!("    Warning: End of buffer reached, some data points may be missing");
            break;
        }
        
        match VarintValue::from_bytes(&message_buffer[read_pos..message_pos]) {
            Ok((point, bytes_read)) => {
                println!("    Point {}: {:?}", i, point);
                read_pos += bytes_read;
            },
            Err(e) => {
                println!("    Error reading point {}: {:?}", i, e);
                println!("    Remaining bytes: {} (read position: {}/{})", 
                    message_pos - read_pos, read_pos, message_pos);
                // Display remaining bytes in hex to aid debugging
                print!("    Remaining bytes hex: [");
                for j in read_pos..message_pos {
                    print!("{:#04x} ", message_buffer[j]);
                }
                println!("]");
                break;
            }
        }
    }
    
    println!("\nExample Complete");
} 