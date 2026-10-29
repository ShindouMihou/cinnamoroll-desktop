use std::{
    ffi::{c_char, c_void},
    ptr::NonNull,
};

#[repr(C)]
#[derive(Default)]
struct Range {
    location: isize,
    length: isize,
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: *const c_void);
    fn CFGetTypeID(value: *const c_void) -> usize;
    fn CFStringCreateWithCString(
        allocator: *const c_void,
        value: *const c_char,
        encoding: u32,
    ) -> *mut c_void;
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateSystemWide() -> *mut c_void;
    fn AXUIElementGetTypeID() -> usize;
    fn AXValueGetTypeID() -> usize;
    fn AXUIElementSetMessagingTimeout(element: *const c_void, timeout: f32) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: *const c_void,
        attribute: *const c_void,
        result: *mut *mut c_void,
    ) -> i32;
    fn AXUIElementCopyParameterizedAttributeValue(
        element: *const c_void,
        attribute: *const c_void,
        parameter: *const c_void,
        result: *mut *mut c_void,
    ) -> i32;
    fn AXValueGetType(value: *const c_void) -> u32;
    fn AXValueGetValue(value: *const c_void, kind: u32, result: *mut c_void) -> u8;
    fn AXValueCreate(kind: u32, value: *const c_void) -> *mut c_void;
}

struct Owned(NonNull<c_void>);
impl Owned {
    fn new(pointer: *mut c_void) -> Option<Self> {
        NonNull::new(pointer).map(Self)
    }
    fn raw(&self) -> *const c_void {
        self.0.as_ptr()
    }
    fn string(value: &'static std::ffi::CStr) -> Result<Self, String> {
        Self::new(unsafe {
            CFStringCreateWithCString(std::ptr::null(), value.as_ptr(), 0x08000100)
        })
        .ok_or_else(|| "Cannot allocate Accessibility metadata".into())
    }
    fn attribute(&self, name: &Self) -> Option<Self> {
        let mut result = std::ptr::null_mut();
        let error = unsafe { AXUIElementCopyAttributeValue(self.raw(), name.raw(), &mut result) };
        if error == 0 { Self::new(result) } else { None }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.raw());
        }
    }
}

pub struct Provider {
    system: Owned,
    focus: Owned,
    selection: Owned,
    bounds: Owned,
}
impl Provider {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            system: Owned::new(unsafe { AXUIElementCreateSystemWide() })
                .ok_or("Cannot create Accessibility client")?,
            focus: Owned::string(c"AXFocusedUIElement")?,
            selection: Owned::string(c"AXSelectedTextRange")?,
            bounds: Owned::string(c"AXBoundsForRange")?,
        })
    }

    pub fn position(&mut self) -> Result<Option<(f32, f32)>, String> {
        if unsafe { AXIsProcessTrusted() } == 0 {
            return Err(
                "Typing follow needs System Settings → Privacy & Security → Accessibility".into(),
            );
        }
        unsafe {
            AXUIElementSetMessagingTimeout(self.system.raw(), 0.15);
        }
        Ok(self.read_position())
    }

    fn read_position(&self) -> Option<(f32, f32)> {
        let element = self.system.attribute(&self.focus)?;
        if unsafe { CFGetTypeID(element.raw()) != AXUIElementGetTypeID() } {
            return None;
        }
        let selected = element.attribute(&self.selection)?;
        if unsafe {
            CFGetTypeID(selected.raw()) != AXValueGetTypeID() || AXValueGetType(selected.raw()) != 4
        } {
            return None;
        }
        let mut range = Range::default();
        // AXValue types are checked before exposing their typed, C-layout storage.
        if unsafe { AXValueGetValue(selected.raw(), 4, (&mut range as *mut Range).cast()) } == 0 {
            return None;
        }
        if range.location < 0 || range.length < 0 {
            return None;
        }
        range.location = range.location.checked_add(range.length)?;
        range.length = 0;
        let caret = Owned::new(unsafe { AXValueCreate(4, (&range as *const Range).cast()) })?;
        let mut result = std::ptr::null_mut();
        if unsafe {
            AXUIElementCopyParameterizedAttributeValue(
                element.raw(),
                self.bounds.raw(),
                caret.raw(),
                &mut result,
            )
        } != 0
        {
            return None;
        }
        let result = Owned::new(result)?;
        if unsafe {
            CFGetTypeID(result.raw()) != AXValueGetTypeID() || AXValueGetType(result.raw()) != 3
        } {
            return None;
        }
        let mut rect = Rect::default();
        if unsafe { AXValueGetValue(result.raw(), 3, (&mut rect as *mut Rect).cast()) } == 0 {
            return None;
        }
        if !rect.x.is_finite()
            || !rect.y.is_finite()
            || !(0.0..=200.0).contains(&rect.height)
            || rect.height == 0.
        {
            return None;
        }
        Some((rect.x as f32, (rect.y + rect.height * 0.5) as f32))
    }
}
