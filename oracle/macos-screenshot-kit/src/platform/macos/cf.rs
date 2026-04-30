use super::ffi::*;
use crate::{Rect, ScreenshotError};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr;

/// A retained CoreFoundation-style object released with `CFRelease`.
pub(super) struct OwnedCf {
    ptr: CFTypeRef,
}

impl OwnedCf {
    pub fn new_const(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn new_mut(ptr: *mut c_void) -> Option<Self> {
        Self::new_const(ptr as CFTypeRef)
    }

    pub fn as_ptr(&self) -> CFTypeRef {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.ptr as *mut c_void
    }
}

impl Drop for OwnedCf {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { CFRelease(self.ptr) };
        }
    }
}

pub(super) fn cf_string(s: &str) -> Result<OwnedCf, ScreenshotError> {
    let c = CString::new(s).map_err(|_| ScreenshotError::InvalidInput(format!("string contains NUL byte: {s:?}")))?;
    let ptr = unsafe { CFStringCreateWithCString(null_allocator(), c.as_ptr(), kCFStringEncodingUTF8) };
    OwnedCf::new_const(ptr).ok_or_else(|| ScreenshotError::CoreGraphics(format!("CFStringCreateWithCString failed for {s:?}")))
}

pub(super) fn cf_number_i64(value: i64) -> Result<OwnedCf, ScreenshotError> {
    let ptr = unsafe {
        CFNumberCreate(
            null_allocator(),
            kCFNumberSInt64Type,
            &value as *const i64 as *const c_void,
        )
    };
    OwnedCf::new_const(ptr).ok_or_else(|| ScreenshotError::CoreGraphics("CFNumberCreate(i64) failed".into()))
}

pub(super) fn cf_number_f64(value: f64) -> Result<OwnedCf, ScreenshotError> {
    let ptr = unsafe {
        CFNumberCreate(
            null_allocator(),
            kCFNumberDoubleType,
            &value as *const f64 as *const c_void,
        )
    };
    OwnedCf::new_const(ptr).ok_or_else(|| ScreenshotError::CoreGraphics("CFNumberCreate(f64) failed".into()))
}

pub(super) fn dict_get(dict: CFDictionaryRef, key: &str) -> Option<CFTypeRef> {
    if dict.is_null() {
        return None;
    }
    let key = cf_string(key).ok()?;
    let mut value: *const c_void = ptr::null();
    let ok = unsafe { CFDictionaryGetValueIfPresent(dict, key.as_ptr(), &mut value) };
    if ok != 0 && !value.is_null() {
        Some(value as CFTypeRef)
    } else {
        None
    }
}

pub(super) fn dict_string(dict: CFDictionaryRef, key: &str) -> Option<String> {
    let value = dict_get(dict, key)?;
    cf_to_string(value as CFStringRef)
}

pub(super) fn dict_i64(dict: CFDictionaryRef, key: &str) -> Option<i64> {
    let value = dict_get(dict, key)?;
    cf_number_to_i64(value as CFNumberRef)
}

pub(super) fn dict_f64(dict: CFDictionaryRef, key: &str) -> Option<f64> {
    let value = dict_get(dict, key)?;
    cf_number_to_f64(value as CFNumberRef)
}

pub(super) fn dict_bool(dict: CFDictionaryRef, key: &str) -> Option<bool> {
    let value = dict_get(dict, key)?;
    Some(unsafe { CFBooleanGetValue(value) != 0 })
}

pub(super) fn dict_rect(dict: CFDictionaryRef, key: &str) -> Option<Rect> {
    let value = dict_get(dict, key)? as CFDictionaryRef;
    if value.is_null() {
        return None;
    }
    let mut rect = CGRect::default();
    let ok = unsafe { CGRectMakeWithDictionaryRepresentation(value, &mut rect) };
    if ok {
        Some(Rect::new(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height))
    } else {
        None
    }
}

pub(super) fn cf_to_string(value: CFStringRef) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let len = unsafe { CFStringGetLength(value) };
    if len <= 0 {
        return Some(String::new());
    }
    let max = unsafe { CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) } + 1;
    if max <= 0 {
        return None;
    }
    let mut buffer = vec![0_i8; max as usize];
    let ok = unsafe { CFStringGetCString(value, buffer.as_mut_ptr(), max, kCFStringEncodingUTF8) };
    if ok == 0 {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    Some(cstr.to_string_lossy().into_owned())
}

pub(super) fn cf_number_to_i64(value: CFNumberRef) -> Option<i64> {
    if value.is_null() {
        return None;
    }
    let mut out = 0_i64;
    let ok = unsafe { CFNumberGetValue(value, kCFNumberSInt64Type, &mut out as *mut i64 as *mut c_void) };
    if ok != 0 { Some(out) } else { None }
}

pub(super) fn cf_number_to_f64(value: CFNumberRef) -> Option<f64> {
    if value.is_null() {
        return None;
    }
    let mut out = 0_f64;
    let ok = unsafe { CFNumberGetValue(value, kCFNumberDoubleType, &mut out as *mut f64 as *mut c_void) };
    if ok != 0 { Some(out) } else { None }
}

pub(super) fn data_to_vec(data: CFDataRef) -> Vec<u8> {
    if data.is_null() {
        return Vec::new();
    }
    let len = unsafe { CFDataGetLength(data) };
    if len <= 0 {
        return Vec::new();
    }
    let ptr = unsafe { CFDataGetBytePtr(data) };
    if ptr.is_null() {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
}
