use anyhow::{bail, Context, Result};

use super::ffi::{
    kCFStringEncodingUTF8, CFRelease, CFRetain, CFStringCreateWithCString, CFStringGetCString,
    CFStringGetLength, CFStringRef, CFTypeRef,
};

/// Create a CFString from a Rust string.
pub(super) fn try_create_cf_string(s: &str) -> Result<CFStringRef> {
    let c_str = std::ffi::CString::new(s)
        .with_context(|| format!("CFString input contains interior NUL: {:?}", s))?;
    let cf_string = unsafe {
        CFStringCreateWithCString(std::ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8)
    };
    if cf_string.is_null() {
        bail!("CFStringCreateWithCString returned null for input: {:?}", s);
    }
    Ok(cf_string)
}

/// Convert a CFString to a Rust String
pub(super) fn cf_string_to_string(cf_string: CFStringRef) -> Option<String> {
    if cf_string.is_null() {
        return None;
    }

    unsafe {
        let length = CFStringGetLength(cf_string);
        if length <= 0 {
            return Some(String::new());
        }

        // Allocate buffer with extra space for UTF-8 expansion
        let buffer_size = (length * 4 + 1) as usize;
        let mut buffer: Vec<i8> = vec![0; buffer_size];

        if CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr(),
            buffer_size as i64,
            kCFStringEncodingUTF8,
        ) {
            let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
            c_str.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// Release a CoreFoundation object
pub(super) fn cf_release(cf: CFTypeRef) {
    if !cf.is_null() {
        unsafe {
            CFRelease(cf);
        }
    }
}

/// Retain a CoreFoundation object (increment reference count)
/// Returns the same pointer for convenience
pub(super) fn cf_retain(cf: CFTypeRef) -> CFTypeRef {
    if !cf.is_null() {
        unsafe { CFRetain(cf) }
    } else {
        cf
    }
}

#[cfg(test)]
mod tests {
    use super::try_create_cf_string;

    #[test]
    fn test_try_create_cf_string_rejects_interior_nul() {
        let error = try_create_cf_string("AX\0Title").expect_err("interior NUL should fail");
        assert!(
            error.to_string().contains("interior NUL"),
            "error should describe invalid CFString input: {error}"
        );
    }
}
