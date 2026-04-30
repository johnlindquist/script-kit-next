use super::cf::{cf_number_f64, cf_string, data_to_vec, OwnedCf};
use super::ffi::*;
use crate::{ImageFormat, Result, RgbaImage, ScreenshotError};
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

pub(crate) struct NativeImage {
    image: CGImageRef,
}

impl NativeImage {
    pub(super) fn from_create_rule(image: CGImageRef) -> Result<Self> {
        if image.is_null() {
            Err(ScreenshotError::CoreGraphics("capture returned a null CGImage".into()))
        } else {
            Ok(Self { image })
        }
    }

    pub(super) fn from_encoded_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(ScreenshotError::Image("encoded image was empty".into()));
        }
        let cf_data = unsafe { CFDataCreate(null_allocator(), bytes.as_ptr(), bytes.len() as CFIndex) };
        let cf_data = OwnedCf::new_const(cf_data).ok_or_else(|| ScreenshotError::Image("CFDataCreate failed".into()))?;
        let source = unsafe { CGImageSourceCreateWithData(cf_data.as_ptr() as CFDataRef, null_dict()) };
        let source = OwnedCf::new_mut(source).ok_or_else(|| ScreenshotError::Image("CGImageSourceCreateWithData failed".into()))?;
        let image = unsafe { CGImageSourceCreateImageAtIndex(source.as_mut_ptr() as CGImageSourceRef, 0, null_dict()) };
        Self::from_create_rule(image)
    }

    pub fn width(&self) -> usize {
        unsafe { CGImageGetWidth(self.image) }
    }

    pub fn height(&self) -> usize {
        unsafe { CGImageGetHeight(self.image) }
    }

    pub fn to_bytes(&self, format: ImageFormat) -> Result<Vec<u8>> {
        let data = unsafe { CFDataCreateMutable(null_allocator(), 0) };
        let data = OwnedCf::new_mut(data).ok_or_else(|| ScreenshotError::Image("CFDataCreateMutable failed".into()))?;
        let uti = cf_string(format.uti())?;
        let properties = image_properties(format)?;
        let props_ptr = properties.as_ref().map(|p| p.as_ptr()).unwrap_or_else(null_dict);
        let dest = unsafe { CGImageDestinationCreateWithData(data.as_mut_ptr() as CFMutableDataRef, uti.as_ptr() as CFStringRef, 1, null_dict()) };
        let dest = OwnedCf::new_mut(dest)
            .ok_or_else(|| ScreenshotError::Image(format!("CGImageDestinationCreateWithData failed for {}", format.uti())))?;
        unsafe {
            CGImageDestinationAddImage(dest.as_mut_ptr() as CGImageDestinationRef, self.image, props_ptr);
        }
        let ok = unsafe { CGImageDestinationFinalize(dest.as_mut_ptr() as CGImageDestinationRef) };
        if !ok {
            return Err(ScreenshotError::Image(format!("CGImageDestinationFinalize failed for {}", format.uti())));
        }
        Ok(data_to_vec(data.as_ptr() as CFDataRef))
    }

    pub fn save_as(&self, path: &Path, format: ImageFormat) -> Result<()> {
        let bytes = self.to_bytes(format)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn to_rgba8(&self) -> Result<RgbaImage> {
        let width = self.width();
        let height = self.height();
        if width == 0 || height == 0 {
            return Err(ScreenshotError::Image("image has zero width or height".into()));
        }

        let bytes_per_row = width
            .checked_mul(4)
            .ok_or_else(|| ScreenshotError::Image("image row size overflow".into()))?;
        let len = bytes_per_row
            .checked_mul(height)
            .ok_or_else(|| ScreenshotError::Image("image buffer size overflow".into()))?;
        let mut data = vec![0_u8; len];

        let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
        if color_space.is_null() {
            return Err(ScreenshotError::Image("CGColorSpaceCreateDeviceRGB failed".into()));
        }

        let bitmap_info = kCGBitmapByteOrder32Big | kCGImageAlphaPremultipliedLast;
        let ctx = unsafe {
            CGBitmapContextCreate(
                data.as_mut_ptr() as *mut c_void,
                width,
                height,
                8,
                bytes_per_row,
                color_space,
                bitmap_info,
            )
        };
        if ctx.is_null() {
            unsafe { CGColorSpaceRelease(color_space) };
            return Err(ScreenshotError::Image("CGBitmapContextCreate failed".into()));
        }

        let rect = cg_rect(0.0, 0.0, width as f64, height as f64);
        unsafe {
            // Normalize the memory layout so row 0 is the top row for typical image processing.
            CGContextTranslateCTM(ctx, 0.0, height as f64);
            CGContextScaleCTM(ctx, 1.0, -1.0);
            CGContextDrawImage(ctx, rect, self.image);
            CGContextRelease(ctx);
            CGColorSpaceRelease(color_space);
        }

        Ok(RgbaImage { width, height, bytes_per_row, data })
    }
}

impl Drop for NativeImage {
    fn drop(&mut self) {
        if !self.image.is_null() {
            unsafe { CGImageRelease(self.image) };
        }
    }
}

impl std::fmt::Debug for NativeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeImage")
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

struct ImageProperties {
    dict: OwnedCf,
    // The dictionary is created without retain callbacks, so keep this value alive as long as the dictionary.
    _quality: OwnedCf,
}

impl ImageProperties {
    fn as_ptr(&self) -> CFDictionaryRef {
        self.dict.as_ptr() as CFDictionaryRef
    }
}

fn image_properties(format: ImageFormat) -> Result<Option<ImageProperties>> {
    let Some(quality) = format.quality() else {
        return Ok(None);
    };
    let quality = cf_number_f64(quality as f64)?;
    let keys = [unsafe { kCGImageDestinationLossyCompressionQuality } as *const c_void];
    let values = [quality.as_ptr() as *const c_void];
    let dict = unsafe {
        CFDictionaryCreate(
            null_allocator(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        )
    };
    let dict = OwnedCf::new_const(dict)
        .ok_or_else(|| ScreenshotError::Image("CFDictionaryCreate for image properties failed".into()))?;
    Ok(Some(ImageProperties { dict, _quality: quality }))
}
