use crate::error::Error;
use crate::traits::{VarInt, VarIntOps};
use crate::encoding::decode;
use core::marker::PhantomData;

/// Iterator representing varint encoded bytes
pub struct VarIntBytesIter<T: VarInt = u64> 
where T::Unsigned: VarIntOps {
    value: T::Unsigned,  
    index: usize,
    size: usize,
    finished: bool,
    _marker: PhantomData<T>,
}

impl<T: VarInt> VarIntBytesIter<T> 
where T::Unsigned: VarIntOps {
    /// Creates a new varint bytes iterator
    pub fn new(value: T) -> Self {
        let unsigned = value.to_unsigned();
        let size = value.varint_size();
        
        VarIntBytesIter {
            value: unsigned,
            index: 0,
            size,
            finished: false,
            _marker: PhantomData,
        }
    }
    
    /// Gets the current index
    pub fn index(&self) -> usize {
        self.index
    }
    
    /// Gets the size of the encoded value without iterating
    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T: VarInt> Iterator for VarIntBytesIter<T> 
where T::Unsigned: VarIntOps {
    type Item = u8;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        
        let byte = if self.value.needs_another_byte() {
            self.value.get_byte_with_continuation()
        } else {
            self.finished = true;
            self.value.get_final_byte()
        };
        
        self.value = self.value.shift_right_7();
        self.index += 1;
        
        Some(byte)
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            (0, Some(0))
        } else {
            let remaining = self.size.saturating_sub(self.index);
            (remaining.min(1), Some(remaining))
        }
    }
}

/// Iterator for decoding varint values from a byte buffer
pub struct VarIntValuesIter<'a, T: VarInt = u64> {
    buf: &'a [u8],
    pos: usize,
    finished: bool,
    _marker: PhantomData<T>,
}

impl<'a, T: VarInt> VarIntValuesIter<'a, T> {
    /// Creates a new varint decoder iterator
    pub fn new(buf: &'a [u8]) -> Self {
        VarIntValuesIter {
            buf,
            pos: 0,
            finished: false,
            _marker: PhantomData,
        }
    }
    
    /// Gets the current position
    pub fn position(&self) -> usize {
        self.pos
    }
    
    /// Gets the remaining buffer
    pub fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }
}

impl<'a, T: VarInt> Iterator for VarIntValuesIter<'a, T> {
    type Item = Result<T, Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.pos >= self.buf.len() {
            return None;
        }
        
        match decode::<T>(&self.buf[self.pos..]) {
            Ok((value, bytes_read)) => {
                self.pos += bytes_read;
                Some(Ok(value))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Helper function to create a bytes encoder for a value
pub fn bytes_of<T: VarInt>(value: T) -> VarIntBytesIter<T> {
    VarIntBytesIter::new(value)
}

/// Helper function to create a values decoder from a buffer
pub fn values_from<'a, T: VarInt>(buf: &'a [u8]) -> VarIntValuesIter<'a, T> {
    VarIntValuesIter::new(buf)
} 