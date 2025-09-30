use crate::error::{Result, StorageError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    LZ4,
    Zstd,
}

pub fn compress(data: &[u8], compression_type: &CompressionType) -> Result<Vec<u8>> {
    match compression_type {
        CompressionType::None => Ok(data.to_vec()),
        CompressionType::LZ4 => {
            Ok(lz4_flex::compress_prepend_size(data))
        }
        CompressionType::Zstd => {
            zstd::bulk::compress(data, 3)
                .map_err(|e| StorageError::Compression(format!("ZSTD compression failed: {}", e)))
        }
    }
}

pub fn decompress(data: &[u8], compression_type: &CompressionType) -> Result<Vec<u8>> {
    match compression_type {
        CompressionType::None => Ok(data.to_vec()),
        CompressionType::LZ4 => {
            lz4_flex::decompress_size_prepended(data)
                .map_err(|e| StorageError::Compression(format!("LZ4 decompression failed: {}", e)))
        }
        CompressionType::Zstd => {
            zstd::bulk::decompress(data, 1024 * 1024) // 1MB limit
                .map_err(|e| StorageError::Compression(format!("ZSTD decompression failed: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_compression() {
        let data = b"test data for compression";
        let compressed = compress(data, &CompressionType::None).unwrap();
        let decompressed = decompress(&compressed, &CompressionType::None).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
    }
    
    #[test]
    fn test_lz4_compression() {
        let data = b"test data for compression that should compress well with repeated patterns patterns patterns";
        let compressed = compress(data, &CompressionType::LZ4).unwrap();
        let decompressed = decompress(&compressed, &CompressionType::LZ4).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
        // LZ4 should achieve some compression on repeated data
        assert!(compressed.len() < data.len());
    }
    
    #[test]
    fn test_zstd_compression() {
        let data = b"test data for compression that should compress well with repeated patterns patterns patterns";
        let compressed = compress(data, &CompressionType::Zstd).unwrap();
        let decompressed = decompress(&compressed, &CompressionType::Zstd).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
        // ZSTD should achieve some compression on repeated data
        assert!(compressed.len() < data.len());
    }
}