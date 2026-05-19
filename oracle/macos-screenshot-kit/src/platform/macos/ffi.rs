#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use std::os::raw::{c_char, c_double, c_int, c_void};

pub type Boolean = u8;
pub type CFIndex = isize;
pub type CFAllocatorRef = *const c_void;
pub type CFTypeRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFMutableDataRef = *mut c_void;
pub type CFDataRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFNumberRef = *const c_void;
pub type CGImageRef = *mut c_void;
pub type CGImageDestinationRef = *mut c_void;
pub type CGImageSourceRef = *mut c_void;
pub type CGColorSpaceRef = *mut c_void;
pub type CGContextRef = *mut c_void;
pub type CGEventRef = *mut c_void;
pub type CGEventSourceRef = *const c_void;
pub type CGDataProviderRef = *const c_void;
pub type CGDirectDisplayID = u32;
pub type CGWindowID = u32;
pub type CGError = i32;
pub type CGWindowListOption = u32;
pub type CGWindowImageOption = u32;
pub type CGBitmapInfo = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGPoint {
    pub x: c_double,
    pub y: c_double,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGSize {
    pub width: c_double,
    pub height: c_double,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

pub const kCGErrorSuccess: CGError = 0;

pub const kCGWindowListOptionAll: CGWindowListOption = 0;
pub const kCGWindowListOptionOnScreenOnly: CGWindowListOption = 1 << 0;
pub const kCGWindowListOptionOnScreenAboveWindow: CGWindowListOption = 1 << 1;
pub const kCGWindowListOptionOnScreenBelowWindow: CGWindowListOption = 1 << 2;
pub const kCGWindowListOptionIncludingWindow: CGWindowListOption = 1 << 3;
pub const kCGWindowListExcludeDesktopElements: CGWindowListOption = 1 << 4;

pub const kCGWindowImageDefault: CGWindowImageOption = 0;
pub const kCGWindowImageBoundsIgnoreFraming: CGWindowImageOption = 1 << 0;
pub const kCGWindowImageShouldBeOpaque: CGWindowImageOption = 1 << 1;
pub const kCGWindowImageOnlyShadows: CGWindowImageOption = 1 << 2;
pub const kCGWindowImageBestResolution: CGWindowImageOption = 1 << 3;
pub const kCGWindowImageNominalResolution: CGWindowImageOption = 1 << 4;

pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;
pub const kCFNumberSInt64Type: c_int = 4;
pub const kCFNumberDoubleType: c_int = 13;

pub const kCGImageAlphaPremultipliedLast: CGBitmapInfo = 1;
pub const kCGBitmapByteOrder32Big: CGBitmapInfo = 0x0000_4000;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub fn CFRelease(cf: CFTypeRef);

    pub fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> *const c_void;
    pub fn CFArrayCreate(
        allocator: CFAllocatorRef,
        values: *const *const c_void,
        numValues: CFIndex,
        callBacks: *const c_void,
    ) -> CFArrayRef;

    pub fn CFDictionaryGetValueIfPresent(
        theDict: CFDictionaryRef,
        key: *const c_void,
        value: *mut *const c_void,
    ) -> Boolean;
    pub fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        numValues: CFIndex,
        keyCallBacks: *const c_void,
        valueCallBacks: *const c_void,
    ) -> CFDictionaryRef;

    pub fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cStr: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    pub fn CFStringGetLength(theString: CFStringRef) -> CFIndex;
    pub fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    pub fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut c_char,
        bufferSize: CFIndex,
        encoding: u32,
    ) -> Boolean;

    pub fn CFNumberCreate(
        allocator: CFAllocatorRef,
        theType: c_int,
        valuePtr: *const c_void,
    ) -> CFNumberRef;
    pub fn CFNumberGetValue(number: CFNumberRef, theType: c_int, valuePtr: *mut c_void) -> Boolean;
    pub fn CFBooleanGetValue(boolean: CFTypeRef) -> Boolean;

    pub fn CFDataCreate(allocator: CFAllocatorRef, bytes: *const u8, length: CFIndex) -> CFDataRef;
    pub fn CFDataCreateMutable(allocator: CFAllocatorRef, capacity: CFIndex) -> CFMutableDataRef;
    pub fn CFDataGetLength(theData: CFDataRef) -> CFIndex;
    pub fn CFDataGetBytePtr(theData: CFDataRef) -> *const u8;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub static CGRectInfinite: CGRect;
    pub static CGRectNull: CGRect;

    pub fn CGPreflightScreenCaptureAccess() -> bool;
    pub fn CGRequestScreenCaptureAccess() -> bool;

    pub fn CGMainDisplayID() -> CGDirectDisplayID;
    pub fn CGGetActiveDisplayList(
        maxDisplays: u32,
        activeDisplays: *mut CGDirectDisplayID,
        displayCount: *mut u32,
    ) -> CGError;
    pub fn CGDisplayBounds(display: CGDirectDisplayID) -> CGRect;
    pub fn CGDisplayPixelsWide(display: CGDirectDisplayID) -> usize;
    pub fn CGDisplayPixelsHigh(display: CGDirectDisplayID) -> usize;
    pub fn CGDisplayIsBuiltin(display: CGDirectDisplayID) -> u32;

    pub fn CGDisplayCreateImage(displayID: CGDirectDisplayID) -> CGImageRef;
    pub fn CGDisplayCreateImageForRect(display: CGDirectDisplayID, rect: CGRect) -> CGImageRef;

    pub fn CGWindowListCopyWindowInfo(
        option: CGWindowListOption,
        relativeToWindow: CGWindowID,
    ) -> CFArrayRef;
    pub fn CGWindowListCreateImage(
        screenBounds: CGRect,
        listOption: CGWindowListOption,
        windowID: CGWindowID,
        imageOption: CGWindowImageOption,
    ) -> CGImageRef;
    pub fn CGWindowListCreateImageFromArray(
        screenBounds: CGRect,
        windowArray: CFArrayRef,
        imageOption: CGWindowImageOption,
    ) -> CGImageRef;

    pub fn CGRectMakeWithDictionaryRepresentation(dict: CFDictionaryRef, rect: *mut CGRect)
        -> bool;

    pub fn CGImageGetWidth(image: CGImageRef) -> usize;
    pub fn CGImageGetHeight(image: CGImageRef) -> usize;
    pub fn CGImageRelease(image: CGImageRef);

    pub fn CGColorSpaceCreateDeviceRGB() -> CGColorSpaceRef;
    pub fn CGColorSpaceRelease(space: CGColorSpaceRef);

    pub fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bitsPerComponent: usize,
        bytesPerRow: usize,
        space: CGColorSpaceRef,
        bitmapInfo: CGBitmapInfo,
    ) -> CGContextRef;
    pub fn CGContextRelease(context: CGContextRef);
    pub fn CGContextDrawImage(context: CGContextRef, rect: CGRect, image: CGImageRef);
    pub fn CGContextTranslateCTM(context: CGContextRef, tx: c_double, ty: c_double);
    pub fn CGContextScaleCTM(context: CGContextRef, sx: c_double, sy: c_double);

    pub fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    pub fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
}

#[link(name = "ImageIO", kind = "framework")]
extern "C" {
    pub static kCGImageDestinationLossyCompressionQuality: CFStringRef;

    pub fn CGImageDestinationCreateWithData(
        data: CFMutableDataRef,
        type_: CFStringRef,
        count: usize,
        options: CFDictionaryRef,
    ) -> CGImageDestinationRef;
    pub fn CGImageDestinationAddImage(
        idst: CGImageDestinationRef,
        image: CGImageRef,
        properties: CFDictionaryRef,
    );
    pub fn CGImageDestinationFinalize(idst: CGImageDestinationRef) -> bool;

    pub fn CGImageSourceCreateWithData(
        data: CFDataRef,
        options: CFDictionaryRef,
    ) -> CGImageSourceRef;
    pub fn CGImageSourceCreateImageAtIndex(
        isrc: CGImageSourceRef,
        index: usize,
        options: CFDictionaryRef,
    ) -> CGImageRef;
}

pub fn null_allocator() -> CFAllocatorRef {
    std::ptr::null()
}

pub fn null_dict() -> CFDictionaryRef {
    std::ptr::null()
}

pub fn cg_rect(x: f64, y: f64, width: f64, height: f64) -> CGRect {
    CGRect {
        origin: CGPoint { x, y },
        size: CGSize { width, height },
    }
}
