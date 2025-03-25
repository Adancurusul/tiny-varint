use crate::error::Error;
use crate::traits::VarInt;
use crate::encoding::{encode, decode};
use crate::zigzag::{ZigZag, encode_zigzag, decode_zigzag};
use core::marker::PhantomData;

/// Batch encoder for VarInt values with state management
pub struct VarIntEncoder<'a, T: VarInt = u64> {
    buf: &'a mut [u8],
    pos: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: VarInt> VarIntEncoder<'a, T> {
    /// Creates a new encoder with the provided buffer
    pub fn new(buf: &'a mut [u8]) -> Self {
        VarIntEncoder {
            buf,
            pos: 0,
            _marker: PhantomData,
        }
    }
    
    /// Gets the current position in the buffer
    pub fn position(&self) -> usize {
        self.pos
    }
    
    /// Gets the remaining space in the buffer
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }
    
    /// Writes a VarInt value to the buffer
    ///
    /// Returns the number of bytes written
    pub fn write(&mut self, value: T) -> Result<usize, Error> {
        if self.pos >= self.buf.len() {
            return Err(Error::BufferTooSmall {
                needed: self.pos + 1,
                actual: self.buf.len(),
            });
        }
        
        let bytes_written = encode(value, &mut self.buf[self.pos..])?;
        self.pos += bytes_written;
        Ok(bytes_written)
    }
    
    /// Writes a batch of VarInt values
    ///
    /// Returns the total number of bytes written
    pub fn write_batch(&mut self, values: &[T]) -> Result<usize, Error> {
        let start_pos = self.pos;
        for &value in values {
            self.write(value)?;
        }
        Ok(self.pos - start_pos)
    }
    
    /// Writes a u64 value to the buffer (convenience method)
    ///
    /// Returns the number of bytes written
    #[inline]
    pub fn write_u64(&mut self, value: u64) -> Result<usize, Error> 
    where T: From<u64> {
        self.write(T::from(value))
    }
    
    /// Writes a u128 value to the buffer (convenience method)
    ///
    /// Returns the number of bytes written
    #[inline]
    pub fn write_u128(&mut self, value: u128) -> Result<usize, Error>
    where T: From<u128> {
        self.write(T::from(value))
    }
    
    /// Writes a signed value using zigzag encoding
    ///
    /// Returns the number of bytes written
    pub fn write_zigzag<S>(&mut self, value: S) -> Result<usize, Error>
    where 
        S: ZigZag,
        S::Unsigned: VarInt {
        if self.pos >= self.buf.len() {
            return Err(Error::BufferTooSmall {
                needed: self.pos + 1,
                actual: self.buf.len(),
            });
        }
        
        let bytes_written = encode_zigzag(value, &mut self.buf[self.pos..])?;
        self.pos += bytes_written;
        Ok(bytes_written)
    }
    
    /// Writes a batch of signed values using zigzag encoding
    ///
    /// Returns the total number of bytes written
    pub fn write_zigzag_batch<S>(&mut self, values: &[S]) -> Result<usize, Error>
    where 
        S: ZigZag,
        S::Unsigned: VarInt {
        let start_pos = self.pos;
        for &value in values {
            self.write_zigzag(value)?;
        }
        Ok(self.pos - start_pos)
    }
}

/// Batch decoder for VarInt values with state management
pub struct VarIntDecoder<'a, T: VarInt = u64> {
    buf: &'a [u8],
    pos: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: VarInt> VarIntDecoder<'a, T> {
    /// Creates a new decoder with the provided buffer
    pub fn new(buf: &'a [u8]) -> Self {
        VarIntDecoder {
            buf,
            pos: 0,
            _marker: PhantomData,
        }
    }
    
    /// Gets the current position in the buffer
    pub fn position(&self) -> usize {
        self.pos
    }
    
    /// Gets the remaining bytes in the buffer
    pub fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }
    
    /// Reads a VarInt value from the buffer
    ///
    /// Returns the decoded value
    pub fn read(&mut self) -> Result<T, Error> {
        if self.pos >= self.buf.len() {
            return Err(Error::InputTooShort);
        }
        
        let (value, bytes_read) = decode(&self.buf[self.pos..])?;
        self.pos += bytes_read;
        Ok(value)
    }
    
    /// Reads a batch of VarInt values into the provided buffer
    ///
    /// Returns the number of values read
    pub fn read_batch(&mut self, values: &mut [T]) -> Result<usize, Error> {
        let mut count = 0;
        
        while count < values.len() && self.pos < self.buf.len() {
            match self.read() {
                Ok(value) => {
                    values[count] = value;
                    count += 1;
                }
                Err(Error::InputTooShort) => break,
                Err(e) => return Err(e),
            }
        }
        
        Ok(count)
    }
    
    /// Reads a u64 value from the buffer (convenience method)
    ///
    /// Returns the decoded value
    #[inline]
    pub fn read_u64(&mut self) -> Result<u64, Error> 
    where u64: From<T> {
        Ok(u64::from(self.read()?))
    }
    
    /// Reads a u128 value from the buffer (convenience method)
    ///
    /// Returns the decoded value
    #[inline]
    pub fn read_u128(&mut self) -> Result<u128, Error>
    where u128: From<T> {
        Ok(u128::from(self.read()?))
    }
    
    /// Reads a signed value that was encoded using zigzag encoding
    ///
    /// Returns the decoded value
    pub fn read_zigzag<S>(&mut self) -> Result<S, Error>
    where 
        S: ZigZag,
        S::Unsigned: VarInt {
        if self.pos >= self.buf.len() {
            return Err(Error::InputTooShort);
        }
        
        let (value, bytes_read) = decode_zigzag(&self.buf[self.pos..])?;
        self.pos += bytes_read;
        Ok(value)
    }
    
    /// Reads a batch of signed values that were encoded using zigzag encoding
    ///
    /// Returns the number of values read
    pub fn read_zigzag_batch<S>(&mut self, values: &mut [S]) -> Result<usize, Error>
    where 
        S: ZigZag,
        S::Unsigned: VarInt {
        let mut count = 0;
        
        while count < values.len() && self.pos < self.buf.len() {
            match self.read_zigzag() {
                Ok(value) => {
                    values[count] = value;
                    count += 1;
                }
                Err(Error::InputTooShort) => break,
                Err(e) => return Err(e),
            }
        }
        
        Ok(count)
    }
}

/// Convenience function to encode a batch of u64 values
///
/// Returns the number of bytes written
#[inline]
pub fn encode_batch(values: &[u64], buf: &mut [u8]) -> Result<usize, Error> {
    let mut encoder = VarIntEncoder::new(buf);
    encoder.write_batch(values)
}

/// Convenience function to decode a batch of u64 values
///
/// Returns the number of values read
#[inline]
pub fn decode_batch(buf: &[u8], values: &mut [u64]) -> Result<usize, Error> {
    let mut decoder = VarIntDecoder::new(buf);
    decoder.read_batch(values)
} 