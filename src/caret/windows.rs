use std::{ffi::c_void, ptr};
use windows::{
    Win32::{
        Foundation::POINT,
        Graphics::Gdi::ClientToScreen,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, SAFEARRAY,
            },
            Ole::{
                SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetLBound,
                SafeArrayGetUBound, SafeArrayGetVartype, SafeArrayUnaccessData,
            },
            Variant::VT_R8,
        },
        UI::{
            Accessibility::{
                CUIAutomation8, IUIAutomation2, IUIAutomationTextPattern,
                IUIAutomationTextPattern2, TextPatternRangeEndpoint_End,
                TextPatternRangeEndpoint_Start, TextUnit_Character, UIA_TextPattern2Id,
                UIA_TextPatternId,
            },
            WindowsAndMessaging::{GUITHREADINFO, GetGUIThreadInfo},
        },
    },
    core::BOOL,
};

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
struct Array {
    raw: *mut SAFEARRAY,
    locked: bool,
}
impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            if self.locked {
                let _ = SafeArrayUnaccessData(self.raw);
            }
            let _ = SafeArrayDestroy(self.raw);
        }
    }
}

pub struct Provider {
    automation: IUIAutomation2,
    _apartment: Apartment,
}
impl Provider {
    pub fn new() -> Result<Self, String> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|e| e.to_string())?;
        let apartment = Apartment;
        let automation: IUIAutomation2 =
            unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| e.to_string())?;
        unsafe {
            automation
                .SetConnectionTimeout(150)
                .and_then(|_| automation.SetTransactionTimeout(150))
        }
        .map_err(|e| e.to_string())?;
        Ok(Self {
            automation,
            _apartment: apartment,
        })
    }
    pub fn position(&mut self) -> Result<Option<(f32, f32)>, String> {
        let element = match unsafe { self.automation.GetFocusedElement() } {
            Ok(value) => value,
            Err(_) => return Ok(native_caret()),
        };
        if unsafe { element.CurrentIsPassword() }.is_ok_and(|value| value.as_bool()) {
            return Ok(None);
        }
        if let Some(point) = native_caret() {
            return Ok(Some(point));
        }
        let range = unsafe {
            if let Ok(pattern) =
                element.GetCurrentPatternAs::<IUIAutomationTextPattern2>(UIA_TextPattern2Id)
            {
                let mut active = BOOL(0);
                match pattern.GetCaretRange(&mut active) {
                    Ok(value) if active.as_bool() => Some(value),
                    _ => None,
                }
            } else if let Ok(pattern) =
                element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
            {
                pattern
                    .GetSelection()
                    .ok()
                    .and_then(|ranges| ranges.GetElement(0).ok())
            } else {
                None
            }
        };
        let Some(range) = range else {
            return Ok(None);
        };
        unsafe {
            let _ = range.MoveEndpointByRange(
                TextPatternRangeEndpoint_Start,
                &range,
                TextPatternRangeEndpoint_End,
            );
            let _ = range.ExpandToEnclosingUnit(TextUnit_Character);
        }
        let raw = match unsafe { range.GetBoundingRectangles() } {
            Ok(value) if !value.is_null() => value,
            _ => return Ok(None),
        };
        let mut array = Array { raw, locked: false };
        if unsafe { SafeArrayGetDim(raw) } != 1
            || unsafe { SafeArrayGetVartype(raw) }.ok() != Some(VT_R8)
        {
            return Ok(None);
        }
        let lower = unsafe { SafeArrayGetLBound(raw, 1) }.map_err(|e| e.to_string())?;
        let upper = unsafe { SafeArrayGetUBound(raw, 1) }.map_err(|e| e.to_string())?;
        if upper.checked_sub(lower).is_none_or(|length| length < 3) {
            return Ok(None);
        }
        let mut data: *mut c_void = ptr::null_mut();
        unsafe { SafeArrayAccessData(raw, &mut data) }.map_err(|e| e.to_string())?;
        array.locked = true;
        if data.is_null() {
            return Ok(None);
        }
        // UIA returns a VT_R8 array of rectangle quadruples; the owner keeps it locked during the copy.
        let rect = unsafe { std::slice::from_raw_parts(data.cast::<f64>(), 4) };
        if !rect.iter().all(|value| value.is_finite()) || rect[3] <= 0. || rect[3] > 200. {
            return Ok(None);
        }
        Ok(Some((rect[0] as f32, (rect[1] + rect[3] * 0.5) as f32)))
    }
}

fn native_caret() -> Option<(f32, f32)> {
    let mut info = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetGUIThreadInfo(0, &mut info) }.ok()?;
    if info.hwndCaret.is_invalid() || info.rcCaret.bottom <= info.rcCaret.top {
        return None;
    }
    let mut point = POINT {
        x: info.rcCaret.left,
        y: ((i64::from(info.rcCaret.top) + i64::from(info.rcCaret.bottom)) / 2) as i32,
    };
    if !unsafe { ClientToScreen(info.hwndCaret, &mut point) }.as_bool() {
        return None;
    }
    Some((point.x as f32, point.y as f32))
}
