#[cfg(target_os = "windows")]
#[path = "startup_windows.rs"]
mod platform;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Status {
    #[default]
    Off,
    On,
    Approval,
    Unavailable,
}

impl Status {
    pub fn registered(self) -> bool {
        matches!(self, Self::On | Self::Approval)
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn Cinna_StartupStatus() -> i32;
    fn Cinna_SetStartup(enabled: bool, error: *mut std::ffi::c_char, capacity: usize) -> bool;
    fn Cinna_OpenLoginSettings();
    fn Cinna_StartupDescription() -> *const std::ffi::c_char;
}

pub(crate) fn status() -> Result<Status, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(match unsafe { Cinna_StartupStatus() } {
            0 | 3 => Status::Off,
            1 => Status::On,
            2 => Status::Approval,
            _ => Status::Unavailable,
        })
    }
    #[cfg(target_os = "windows")]
    {
        platform::status()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(Status::Unavailable)
    }
}

pub(crate) fn set_enabled(enabled: bool) -> Result<Status, String> {
    #[cfg(target_os = "macos")]
    {
        let mut error = [0 as std::ffi::c_char; 512];
        if !unsafe { Cinna_SetStartup(enabled, error.as_mut_ptr(), error.len()) } {
            return Err(unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
                .to_string_lossy()
                .into_owned());
        }
        status()
    }
    #[cfg(target_os = "windows")]
    {
        platform::set_enabled(enabled)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = enabled;
        Err("Launch at login is unavailable on this platform.".into())
    }
}

pub(crate) fn open_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        unsafe { Cinna_OpenLoginSettings() };
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        sdl3::url::open_url("ms-settings:startupapps").map_err(|error| error.to_string())
    }
}

#[cfg(any(target_os = "windows", test))]
fn windows_command(path: &std::path::Path) -> Result<Vec<u16>, String> {
    let path = path.to_str().ok_or("The app path is not valid Unicode.")?;
    if path.contains(['"', '\0']) {
        return Err("The app path contains unsupported characters.".into());
    }
    let command: Vec<u16> = format!("\"{path}\"").encode_utf16().chain([0]).collect();
    if command.len() > 261 {
        return Err("Move the app to a shorter path before enabling launch at login.".into());
    }
    Ok(command)
}

pub(crate) fn diagnostic() -> String {
    #[cfg(target_os = "macos")]
    {
        unsafe { std::ffi::CStr::from_ptr(Cinna_StartupDescription()) }
            .to_string_lossy()
            .into_owned()
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!("Launch at login: {:?}", status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_command_quotes_spaces_and_preserves_unicode() {
        let command = windows_command(std::path::Path::new("C:\\Cute pets\\シナモン.exe")).unwrap();
        assert_eq!(command.last(), Some(&0));
        assert_eq!(
            String::from_utf16(&command[..command.len() - 1]).unwrap(),
            "\"C:\\Cute pets\\シナモン.exe\""
        );
    }

    #[test]
    fn startup_rejects_invalid_or_oversized_commands() {
        for path in [
            "bad\"path.exe".into(),
            "bad\0path.exe".into(),
            "a".repeat(260),
        ] {
            assert!(windows_command(std::path::Path::new(&path)).is_err());
        }
        assert!(Status::Approval.registered());
        assert!(!Status::Unavailable.registered());
    }
}
