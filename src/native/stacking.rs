use sdl3::{sys::everything::*, video::Window};
use std::ffi::{c_char, c_void};

fn handle(window: &Window, name: *const c_char) -> Result<*mut c_void, String> {
    let value = unsafe {
        SDL_GetPointerProperty(
            SDL_GetWindowProperties(window.raw()),
            name,
            std::ptr::null_mut(),
        )
    };
    if value.is_null() {
        Err("Cannot get a native window for cozy-corner ordering".into())
    } else {
        Ok(value)
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn CinnaWindowStackState(lower: *mut c_void, upper: *mut c_void) -> i32;
    fn CinnaWindowOrderBelow(lower: *mut c_void, upper: *mut c_void);
}

#[cfg(target_os = "macos")]
pub(crate) fn is_above(lower: &Window, upper: &Window) -> Result<bool, String> {
    let lower = handle(lower, SDL_PROP_WINDOW_COCOA_WINDOW_POINTER)?;
    let upper = handle(upper, SDL_PROP_WINDOW_COCOA_WINDOW_POINTER)?;
    Ok(unsafe { CinnaWindowStackState(lower, upper) } == 1)
}

#[cfg(target_os = "macos")]
fn reorder(lower: &Window, upper: &Window) -> Result<(), String> {
    let lower = handle(lower, SDL_PROP_WINDOW_COCOA_WINDOW_POINTER)?;
    let upper = handle(upper, SDL_PROP_WINDOW_COCOA_WINDOW_POINTER)?;
    unsafe { CinnaWindowOrderBelow(lower, upper) };
    Ok(())
}

#[cfg(target_os = "windows")]
pub(crate) fn is_above(lower: &Window, upper: &Window) -> Result<bool, String> {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{GW_HWNDPREV, GetWindow},
    };
    let lower = HWND(handle(lower, SDL_PROP_WINDOW_WIN32_HWND_POINTER)?);
    let upper = HWND(handle(upper, SDL_PROP_WINDOW_WIN32_HWND_POINTER)?);
    let mut cursor = lower;
    while let Ok(previous) = unsafe { GetWindow(cursor, GW_HWNDPREV) } {
        if previous == upper {
            return Ok(false);
        }
        cursor = previous;
    }
    Ok(true)
}

#[cfg(target_os = "windows")]
fn reorder(lower: &Window, upper: &Window) -> Result<(), String> {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
        },
    };
    let lower = HWND(handle(lower, SDL_PROP_WINDOW_WIN32_HWND_POINTER)?);
    let upper = HWND(handle(upper, SDL_PROP_WINDOW_WIN32_HWND_POINTER)?);
    unsafe {
        SetWindowPos(
            lower,
            Some(upper),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER,
        )
    }
    .map_err(|error| error.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn is_above(_: &Window, _: &Window) -> Result<bool, String> {
    Ok(false)
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reorder(_: &Window, _: &Window) -> Result<(), String> {
    Ok(())
}

pub(crate) fn below(lower: &Window, upper: &Window) -> Result<(), String> {
    if is_above(lower, upper)? {
        reorder(lower, upper)?;
    }
    Ok(())
}

pub(crate) fn smoke() -> Result<(), String> {
    let sdl = sdl3::init().map_err(|error| error.to_string())?;
    let video = sdl.video().map_err(|error| error.to_string())?;
    let flags = sdl3::video::WindowFlags::BORDERLESS
        | sdl3::video::WindowFlags::ALWAYS_ON_TOP
        | sdl3::video::WindowFlags::UTILITY;
    let upper = video
        .window("Cinnamoroll · Stacking check pet", 64, 64)
        .set_flags(flags)
        .position(20, 80)
        .build()
        .map_err(|error| error.to_string())?;
    let mut lower = video
        .window("Cinnamoroll · Stacking check bed", 64, 64)
        .set_flags(flags)
        .position(40, 100)
        .build()
        .map_err(|error| error.to_string())?;
    for _ in 0..3 {
        lower.raise();
        unsafe { SDL_PumpEvents() };
        if !is_above(&lower, &upper)? {
            return Err("Stacking check did not reproduce the raised-bed case".into());
        }
        let position = lower.position();
        let size = lower.size();
        below(&lower, &upper)?;
        if is_above(&lower, &upper)? {
            return Err("Cozy-corner window is still above Cinnamoroll".into());
        }
        if position != lower.position() || size != lower.size() {
            return Err("Stack ordering changed the bed geometry".into());
        }
        below(&lower, &upper)?;
    }
    println!(
        "Native stacking check passed: three raised-bed repairs, stable geometry, repeat calls unchanged."
    );
    Ok(())
}
