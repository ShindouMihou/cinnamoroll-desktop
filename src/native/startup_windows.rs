use super::{Status, windows_command};
use windows::{
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ, RegDeleteKeyValueW,
            RegGetValueW, RegSetKeyValueW,
        },
    },
    core::w,
};

pub(super) fn status() -> Result<Status, String> {
    let mut size = 0;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            w!("Cinnamoroll"),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
    };
    if matches!(result, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) {
        return Ok(Status::Off);
    }
    result.ok().map_err(|error| error.to_string())?;
    let mut approval = [0u8; 32];
    let mut size = approval.len() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run"),
            w!("Cinnamoroll"),
            RRF_RT_REG_BINARY,
            None,
            Some(approval.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    Ok(
        if result == ERROR_SUCCESS && size > 0 && matches!(approval[0], 3 | 7) {
            Status::Approval
        } else {
            Status::On
        },
    )
}

pub(super) fn set_enabled(enabled: bool) -> Result<Status, String> {
    let result = if enabled {
        let command =
            windows_command(&std::env::current_exe().map_err(|error| error.to_string())?)?;
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                w!("Cinnamoroll"),
                REG_SZ.0,
                Some(command.as_ptr().cast()),
                (command.len() * 2) as u32,
            )
        }
    } else {
        unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                w!("Cinnamoroll"),
            )
        }
    };
    if !(matches!(result, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) && !enabled) {
        result.ok().map_err(|error| error.to_string())?;
    }
    status()
}
