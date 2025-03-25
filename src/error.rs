/// Error type for varint encoding/decoding operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Output buffer is too small to hold the encoded data
    BufferTooSmall {
        /// Number of bytes needed
        needed: usize,
        /// Actual buffer size
        actual: usize,
    },
    /// Overflow error encountered during decoding
    Overflow,
    /// Input data is insufficient during decoding
    InputTooShort,
    /// Invalid varint encoding encountered during decoding
    InvalidEncoding,
}

// Helper methods for the Error error type
impl Error {
    /// Get the required buffer size
    pub fn needed(&self) -> Option<usize> {
        match self {
            Error::BufferTooSmall { needed, .. } => Some(*needed),
            _ => None,
        }
    }
    
    /// Get the actual buffer size
    pub fn actual(&self) -> Option<usize> {
        match self {
            Error::BufferTooSmall { actual, .. } => Some(*actual),
            _ => None,
        }
    }
} 