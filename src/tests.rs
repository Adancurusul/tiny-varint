#[cfg(test)]
mod tests {
    // Enable standard library in tests
    extern crate std;
    use self::std::vec::Vec;
    
    use crate::encoding::{encode, decode, varint_size};
    use crate::zigzag::{encode_zigzag, decode_zigzag};
    use crate::batch::{VarIntEncoder, VarIntDecoder};
    use crate::iter::{bytes_of, values_from};
    use crate::Error;

    #[test]
    fn test_encode_decode_u64() {
        // Test specific values
        let test_cases = [
            0u64, 1, 127, 128, 16383, 16384, 2097151, 2097152, 
            268435455, 268435456, 0xFFFFFFFF, 0xFFFFFFFFFFFFFFFF,
        ];
        
        for &value in &test_cases {
            let mut buf = [0u8; 10];
            let bytes_written = encode(value, &mut buf).unwrap();
            
            let (decoded, bytes_read) = decode::<u64>(&buf).unwrap();
            
            assert_eq!(value, decoded, "Value mismatch for {}", value);
            assert_eq!(bytes_written, bytes_read, "Bytes count mismatch for {}", value);
        }
    }
    
    #[test]
    fn test_generic_encode_decode() {
        // Test u8
        let value_u8 = 127u8;
        let mut buf = [0u8; 20]; // Increase buffer size
        let bytes_written = encode(value_u8, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<u8>(&buf).unwrap();
        assert_eq!(value_u8, decoded);
        assert_eq!(bytes_written, bytes_read);
        
        // Test u16
        let value_u16 = 16383u16;
        let bytes_written = encode(value_u16, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<u16>(&buf).unwrap();
        assert_eq!(value_u16, decoded);
        assert_eq!(bytes_written, bytes_read);
        
        // Test u32
        let value_u32 = 2097151u32;
        let bytes_written = encode(value_u32, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<u32>(&buf).unwrap();
        assert_eq!(value_u32, decoded);
        assert_eq!(bytes_written, bytes_read);
        
        // Test i8
        let value_i8 = -42i8;
        let bytes_written = encode(value_i8, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<i8>(&buf).unwrap();
        assert_eq!(value_i8, decoded);
        assert_eq!(bytes_written, bytes_read);
        
        // Test i16
        let value_i16 = -16383i16;
        let bytes_written = encode(value_i16, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<i16>(&buf).unwrap();
        assert_eq!(value_i16, decoded);
        assert_eq!(bytes_written, bytes_read);
        
        // Test i32
        let value_i32 = -2097151i32;
        let bytes_written = encode(value_i32, &mut buf).unwrap();
        let (decoded, bytes_read) = decode::<i32>(&buf).unwrap();
        assert_eq!(value_i32, decoded);
        assert_eq!(bytes_written, bytes_read);
    }
    
    #[test]
    fn test_encode_decode_zigzag() {
        // Test specific signed values
        let test_cases = [
            0i32, 1, -1, 2, -2, 127, -127, 128, -128, 
            16383, -16383, 16384, -16384, i32::MAX, i32::MIN,
        ];
        
        for &value in &test_cases {
            let mut buf = [0u8; 10];
            let bytes_written = encode_zigzag(value, &mut buf).unwrap();
            
            let (decoded, bytes_read) = decode_zigzag::<i32>(&buf).unwrap();
            
            assert_eq!(value, decoded, "Value mismatch for {}", value);
            assert_eq!(bytes_written, bytes_read, "Bytes count mismatch for {}", value);
        }
    }
    
    #[test]
    fn test_varint_size() {
        assert_eq!(varint_size(0u64), 1);
        assert_eq!(varint_size(127u64), 1);
        assert_eq!(varint_size(128u64), 2);
        assert_eq!(varint_size(16383u64), 2);
        assert_eq!(varint_size(16384u64), 3);
        assert_eq!(varint_size(2097151u64), 3);
        assert_eq!(varint_size(2097152u64), 4);
        assert_eq!(varint_size(268435455u64), 4);
        assert_eq!(varint_size(268435456u64), 5);
        assert_eq!(varint_size(0xFFFFFFFFu64), 5);
        assert_eq!(varint_size(0xFFFFFFFFFFFFFFFFu64), 10);
        
        // Other types
        assert_eq!(varint_size(127u8), 1);
        assert_eq!(varint_size(127u16), 1);
        assert_eq!(varint_size(127u32), 1);
        assert_eq!(varint_size(127i8), 1);
        assert_eq!(varint_size(127i16), 1);
        assert_eq!(varint_size(127i32), 1);
        assert_eq!(varint_size(127i64), 1);
    }
    
    #[test]
    fn test_encode_decode_batch() {
        let values = [0u64, 1, 127, 128, 16383, 16384];
        let mut buf = [0u8; 100];
        let mut decoded = [0u64; 6];
        
        // Using VarIntEncoder/VarIntDecoder
        let mut encoder = VarIntEncoder::new(&mut buf);
        let bytes_written = encoder.write_batch(&values).unwrap();
        
        let mut decoder = VarIntDecoder::new(&buf[..bytes_written]);
        let count = decoder.read_batch(&mut decoded).unwrap();
        
        assert_eq!(count, values.len());
        assert_eq!(values.to_vec(), decoded.to_vec());
        
        // Test with u32
        let values_u32 = [0u32, 1, 127, 128, 16383, 16384];
        let mut decoded_u32 = [0u32; 6];
        
        let mut encoder = VarIntEncoder::<u32>::new(&mut buf);
        let bytes_written = encoder.write_batch(&values_u32).unwrap();
        
        let mut decoder = VarIntDecoder::<u32>::new(&buf[..bytes_written]);
        let count = decoder.read_batch(&mut decoded_u32).unwrap();
        
        assert_eq!(count, values_u32.len());
        assert_eq!(values_u32.to_vec(), decoded_u32.to_vec());
    }
    
    #[test]
    fn test_buffer_too_small_error() {
        let value = 0xFFFFFFFFFFFFFFFFu64; // Requires 10 bytes
        let mut small_buf = [0u8; 5];
        
        let result = encode(value, &mut small_buf);
        assert!(result.is_err());
        
        if let Err(Error::BufferTooSmall { needed, actual }) = result {
            assert_eq!(needed, 10);
            assert_eq!(actual, 5);
        } else {
            panic!("Expected BufferTooSmall error");
        }
    }
    
    #[test]
    fn test_decode_input_too_short() {
        // Create an incomplete varint - highest bit is 1 indicating more bytes follow, but none provided
        let buf = [0x80];
        
        let result = decode::<u64>(&buf);
        assert!(result.is_err());
        
        if let Err(Error::InputTooShort) = result {
            // Test passed
        } else {
            panic!("Expected InputTooShort error");
        }
    }
    
    #[test]
    fn test_u128_encode_decode() {
        let test_cases = [
            0u128, 1, 127, 128, 16383, 16384, 2097151, 2097152, 
            268435455, 268435456, 0xFFFFFFFF, 0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        ];
        
        for &value in &test_cases {
            let mut buf = [0u8; 20]; // u128 requires at most 19 bytes
            let bytes_written = encode(value, &mut buf).unwrap();
            
            let (decoded, bytes_read) = decode::<u128>(&buf).unwrap();
            
            assert_eq!(value, decoded, "Value mismatch for {}", value);
            assert_eq!(bytes_written, bytes_read, "Bytes count mismatch for {}", value);
        }
    }
    
    #[test]
    fn test_bytes_encoder_iterator() {
        let test_cases = [0u64, 1, 127, 128, 16383, 16384];
        
        for &value in &test_cases {
            // Get size
            let expected_size = varint_size(value);
            let encoder = bytes_of(value);
            assert_eq!(encoder.size(), expected_size);
            
            // Collect bytes
            let bytes: Vec<u8> = encoder.collect();
            assert_eq!(bytes.len(), expected_size);
            
            // Verify by decoding
            let (decoded, _) = decode::<u64>(&bytes).unwrap();
            assert_eq!(decoded, value);
        }
        
        // Test with other integer types
        let value_u32 = 16384u32;
        let encoder = bytes_of(value_u32);
        let bytes: Vec<u8> = encoder.collect();
        let (decoded, _) = decode::<u32>(&bytes).unwrap();
        assert_eq!(decoded, value_u32);
        
        let value_i16 = -42i16;
        let encoder = bytes_of(value_i16);
        let bytes: Vec<u8> = encoder.collect();
        let (decoded, _) = decode::<i16>(&bytes).unwrap();
        assert_eq!(decoded, value_i16);
    }
    
    #[test]
    fn test_values_decoder_iterator() {
        let values = [0u64, 1, 127, 128, 16383, 16384];
        let mut buf = [0u8; 100];
        
        // Encode values
        let mut encoder = VarIntEncoder::new(&mut buf);
        let bytes_written = encoder.write_batch(&values).unwrap();
        
        // Decode using iterator
        let decoder = values_from::<u64>(&buf[..bytes_written]);
        let decoded: Result<Vec<u64>, _> = decoder.collect();
        let decoded = decoded.unwrap();
        
        assert_eq!(values.to_vec(), decoded);
        
        // Test with other integer types
        let values_u32 = [0u32, 1, 127, 128, 16383, 16384];
        
        let mut encoder = VarIntEncoder::<u32>::new(&mut buf);
        let bytes_written = encoder.write_batch(&values_u32).unwrap();
        
        let decoder = values_from::<u32>(&buf[..bytes_written]);
        let decoded: Result<Vec<u32>, _> = decoder.collect();
        let decoded = decoded.unwrap();
        
        assert_eq!(values_u32.to_vec(), decoded);
    }
    
    #[test]
    fn test_encoder_decoder_batch() {
        // Test u64 values
        let values_u64 = [1u64, 127, 128, 16383, 16384, 2097151];
        let mut buffer = [0u8; 100];
        
        // Test encoder
        let mut encoder = VarIntEncoder::new(&mut buffer);
        let bytes_written = encoder.write_batch(&values_u64).unwrap();
        
        // Test decoder
        let mut decoder = VarIntDecoder::new(&buffer[..bytes_written]);
        let mut decoded = [0u64; 6];
        decoder.read_batch(&mut decoded).unwrap();
        
        assert_eq!(values_u64, decoded);
        
        // Test zigzag values
        let values_i32 = [0i32, 1, -1, 2, -2, 127, -127, 128, -128];
        let mut buffer = [0u8; 100];
        
        // Test encoder
        let mut encoder = VarIntEncoder::<u32>::new(&mut buffer);
        let bytes_written = encoder.write_zigzag_batch(&values_i32).unwrap();
        
        // Test decoder
        let mut decoder = VarIntDecoder::<u32>::new(&buffer[..bytes_written]);
        let mut decoded = [0i32; 9];
        decoder.read_zigzag_batch(&mut decoded).unwrap();
        
        assert_eq!(values_i32, decoded);
    }
} 