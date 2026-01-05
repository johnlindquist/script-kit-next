//! Clipboard image encoding and decoding
//!
//! Handles base64 encoding/decoding of clipboard images, including
//! PNG compression and legacy RGBA format support.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use gpui::RenderImage;
use smallvec::smallvec;
use std::sync::Arc;
use tracing::{debug, warn};

use super::blob_store::{is_blob_content, load_blob, store_blob};

/// Encode image data as a blob file (PNG stored on disk)
///
/// Format: "blob:{hash}" where hash is SHA-256 of PNG bytes
/// The PNG file is stored at ~/.scriptkit/clipboard/blobs/<hash>.png
///
/// This is the most efficient format:
/// - No base64 overhead (saves 33%)
/// - No SQLite WAL churn for large images
/// - Content-addressed deduplication
pub fn encode_image_as_blob(image: &arboard::ImageData) -> Result<String> {
    let png_bytes = encode_image_to_png_bytes(image)?;
    store_blob(&png_bytes)
}

/// Encode image data as base64 PNG string (compressed, ~90% smaller than raw RGBA)
///
/// Format: "png:{base64_encoded_png_data}"
/// The PNG format is detected by the "png:" prefix for decoding.
///
/// NOTE: For new images, prefer encode_image_as_blob() which avoids base64 overhead.
/// This function is kept for backwards compatibility.
#[allow(dead_code)] // Kept for backwards compatibility with existing clipboard entries
pub fn encode_image_as_png(image: &arboard::ImageData) -> Result<String> {
    let png_bytes = encode_image_to_png_bytes(image)?;
    let base64_data = BASE64.encode(&png_bytes);
    Ok(format!("png:{}", base64_data))
}

/// Internal helper to encode image to PNG bytes
fn encode_image_to_png_bytes(image: &arboard::ImageData) -> Result<Vec<u8>> {
    use std::io::Cursor;

    // Create an RgbaImage from the raw bytes
    let rgba_image = image::RgbaImage::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
    .context("Failed to create RGBA image from clipboard data")?;

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    rgba_image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode image as PNG")?;

    Ok(png_data)
}

/// Encode image data as base64 raw RGBA string (legacy format, kept for compatibility)
///
/// Format: "rgba:{width}:{height}:{base64_data}"
/// This is the old format - new code should use encode_image_as_png() instead.
#[allow(dead_code)] // Kept for backward compatibility if needed
pub fn encode_image_as_base64(image: &arboard::ImageData) -> Result<String> {
    let base64_data = BASE64.encode(&image.bytes);
    Ok(format!(
        "rgba:{}:{}:{}",
        image.width, image.height, base64_data
    ))
}

/// Decode a base64 image string back to ImageData
///
/// Supports both formats:
/// - New PNG format: "png:{base64_encoded_png_data}"
/// - Legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
#[allow(dead_code)]
pub fn decode_base64_image(content: &str) -> Option<arboard::ImageData<'static>> {
    if content.starts_with("png:") {
        decode_png_to_image_data(content)
    } else if content.starts_with("rgba:") {
        decode_legacy_rgba(content)
    } else {
        warn!("Unknown clipboard image format prefix");
        None
    }
}

/// Decode PNG format: "png:{base64_encoded_png_data}"
fn decode_png_to_image_data(content: &str) -> Option<arboard::ImageData<'static>> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();

    Some(arboard::ImageData {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        bytes: rgba.into_raw().into(),
    })
}

/// Decode legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
fn decode_legacy_rgba(content: &str) -> Option<arboard::ImageData<'static>> {
    let parts: Vec<&str> = content.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "rgba" {
        return None;
    }

    let width: usize = parts[1].parse().ok()?;
    let height: usize = parts[2].parse().ok()?;
    let bytes = BASE64.decode(parts[3]).ok()?;

    Some(arboard::ImageData {
        width,
        height,
        bytes: bytes.into(),
    })
}

/// Decode a clipboard image content string to GPUI RenderImage
///
/// Supports three formats:
/// - Blob format: "blob:{hash}" (file-based, most efficient)
/// - PNG format: "png:{base64_encoded_png_data}"
/// - Legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
///
/// Returns an Arc<RenderImage> for efficient caching.
///
/// **IMPORTANT**: Call this ONCE per entry and cache the result. Do NOT
/// decode during rendering as this is expensive.
pub fn decode_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    if is_blob_content(content) {
        decode_blob_to_render_image(content)
    } else if content.starts_with("png:") {
        decode_png_to_render_image(content)
    } else if content.starts_with("rgba:") {
        decode_rgba_to_render_image(content)
    } else {
        warn!("Invalid clipboard image format, expected blob:, png: or rgba: prefix");
        None
    }
}

/// Decode blob format to RenderImage
fn decode_blob_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    let png_bytes = load_blob(content)?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let img_width = rgba.width();
    let img_height = rgba.height();

    let frame = image::Frame::new(rgba);
    // Use smallvec! macro to avoid cloning the frame buffer
    // (SmallVec::from_elem clones the element, which would duplicate megabytes of pixel data)
    let render_image = RenderImage::new(smallvec![frame]);

    debug!(
        width = img_width,
        height = img_height,
        format = "blob",
        "Decoded blob clipboard image to RenderImage"
    );
    Some(Arc::new(render_image))
}

/// Decode PNG format to RenderImage
fn decode_png_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let img_width = rgba.width();
    let img_height = rgba.height();

    let frame = image::Frame::new(rgba);
    // Use smallvec! macro to avoid cloning the frame buffer
    // (SmallVec::from_elem clones the element, which would duplicate megabytes of pixel data)
    let render_image = RenderImage::new(smallvec![frame]);

    debug!(
        width = img_width,
        height = img_height,
        format = "png",
        "Decoded clipboard image to RenderImage"
    );
    Some(Arc::new(render_image))
}

/// Decode legacy RGBA format to RenderImage
fn decode_rgba_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    let parts: Vec<&str> = content.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "rgba" {
        warn!("Invalid clipboard image format, expected rgba:W:H:data");
        return None;
    }

    let width: u32 = parts[1].parse().ok()?;
    let height: u32 = parts[2].parse().ok()?;
    let rgba_bytes = BASE64.decode(parts[3]).ok()?;

    let expected_bytes = (width as usize) * (height as usize) * 4;
    if rgba_bytes.len() != expected_bytes {
        warn!(
            "Clipboard image byte count mismatch: expected {}, got {}",
            expected_bytes,
            rgba_bytes.len()
        );
        return None;
    }

    let rgba_image = image::RgbaImage::from_raw(width, height, rgba_bytes)?;
    let frame = image::Frame::new(rgba_image);
    // Use smallvec! macro to avoid cloning the frame buffer
    // (SmallVec::from_elem clones the element, which would duplicate megabytes of pixel data)
    let render_image = RenderImage::new(smallvec![frame]);

    debug!(
        width,
        height,
        format = "rgba",
        "Decoded clipboard image to RenderImage"
    );
    Some(Arc::new(render_image))
}

/// Get image dimensions from content string without fully decoding
///
/// Returns (width, height) if the content is a valid image format.
/// For blob format, reads PNG header from file to extract dimensions.
/// For PNG format, reads PNG header to extract dimensions (fast, no full decode).
/// For legacy RGBA format, parses dimensions from metadata prefix.
pub fn get_image_dimensions(content: &str) -> Option<(u32, u32)> {
    if is_blob_content(content) {
        get_blob_dimensions(content)
    } else if content.starts_with("png:") {
        get_png_dimensions(content)
    } else if content.starts_with("rgba:") {
        let parts: Vec<&str> = content.splitn(4, ':').collect();
        if parts.len() >= 3 {
            let width: u32 = parts[1].parse().ok()?;
            let height: u32 = parts[2].parse().ok()?;
            Some((width, height))
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract dimensions from blob file without full decode
fn get_blob_dimensions(content: &str) -> Option<(u32, u32)> {
    let png_bytes = load_blob(content)?;

    let cursor = std::io::Cursor::new(&png_bytes);
    let reader = image::ImageReader::with_format(cursor, image::ImageFormat::Png);
    let (width, height) = reader.into_dimensions().ok()?;

    Some((width, height))
}

/// Extract dimensions from PNG header without full decode
fn get_png_dimensions(content: &str) -> Option<(u32, u32)> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let cursor = std::io::Cursor::new(&png_bytes);
    let reader = image::ImageReader::with_format(cursor, image::ImageFormat::Png);
    let (width, height) = reader.into_dimensions().ok()?;

    Some((width, height))
}

/// Compute a simple hash of image data for change detection
pub fn compute_image_hash(image: &arboard::ImageData) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image.width.hash(&mut hasher);
    image.height.hash(&mut hasher);

    // Hash first 1KB of pixels for quick comparison
    let sample_size = 1024.min(image.bytes.len());
    image.bytes[..sample_size].hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_hash_deterministic() {
        let image = arboard::ImageData {
            width: 100,
            height: 100,
            bytes: vec![0u8; 40000].into(),
        };

        let hash1 = compute_image_hash(&image);
        let hash2 = compute_image_hash(&image);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_base64_image_roundtrip_legacy() {
        let original = arboard::ImageData {
            width: 2,
            height: 2,
            bytes: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ]
            .into(),
        };

        let encoded = encode_image_as_base64(&original).expect("Should encode");
        assert!(
            encoded.starts_with("rgba:"),
            "Legacy format should have rgba: prefix"
        );
        let decoded = decode_base64_image(&encoded).expect("Should decode");

        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
        assert_eq!(original.bytes.as_ref(), decoded.bytes.as_ref());
    }

    #[test]
    fn test_png_image_roundtrip() {
        let original = arboard::ImageData {
            width: 2,
            height: 2,
            bytes: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ]
            .into(),
        };

        let encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        assert!(
            encoded.starts_with("png:"),
            "PNG format should have png: prefix"
        );

        let decoded = decode_base64_image(&encoded).expect("Should decode");

        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
        assert_eq!(original.bytes.as_ref(), decoded.bytes.as_ref());
    }

    #[test]
    fn test_png_compression_saves_space() {
        let original = arboard::ImageData {
            width: 100,
            height: 100,
            bytes: vec![128u8; 100 * 100 * 4].into(),
        };

        let png_encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        let rgba_encoded = encode_image_as_base64(&original).expect("Should encode as RGBA");

        assert!(
            png_encoded.len() < rgba_encoded.len(),
            "PNG should be smaller for 100x100 image: PNG={} vs RGBA={}",
            png_encoded.len(),
            rgba_encoded.len()
        );

        let decoded = decode_base64_image(&png_encoded).expect("Should decode");
        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
    }

    #[test]
    fn test_get_image_dimensions_both_formats() {
        let original = arboard::ImageData {
            width: 100,
            height: 50,
            bytes: vec![0u8; 100 * 50 * 4].into(),
        };

        let rgba_encoded = encode_image_as_base64(&original).expect("Should encode");
        let dims = get_image_dimensions(&rgba_encoded).expect("Should get dimensions");
        assert_eq!(dims, (100, 50));

        let png_encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        let dims = get_image_dimensions(&png_encoded).expect("Should get PNG dimensions");
        assert_eq!(dims, (100, 50));
    }
}
