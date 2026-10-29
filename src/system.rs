pub fn low_battery() -> bool {
    let mut percent = -1;
    let state = unsafe { sdl3::sys::power::SDL_GetPowerInfo(std::ptr::null_mut(), &mut percent) };
    state == sdl3::sys::power::SDL_POWERSTATE_ON_BATTERY && (0..=20).contains(&percent)
}

#[cfg(target_os = "macos")]
pub fn idle_seconds() -> Option<f64> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
    }
    let seconds = unsafe { CGEventSourceSecondsSinceLastEventType(0, u32::MAX) };
    seconds.is_finite().then_some(seconds)
}

#[cfg(target_os = "windows")]
pub fn idle_seconds() -> Option<f64> {
    use windows::Win32::{
        System::SystemInformation::GetTickCount64,
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    };
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        return None;
    }
    let elapsed = (unsafe { GetTickCount64() } as u32).wrapping_sub(info.dwTime);
    Some(f64::from(elapsed) / 1000.)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn idle_seconds() -> Option<f64> {
    None
}

#[derive(Default)]
pub struct AwayDetector {
    away: bool,
}

impl AwayDetector {
    pub fn update(&mut self, idle_seconds: Option<f64>) -> bool {
        let Some(idle) = idle_seconds else {
            return false;
        };
        if idle >= 120. {
            self.away = true;
        } else if idle < 5. && self.away {
            self.away = false;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn returning_greets_once_and_missing_data_is_harmless() {
        let mut detector = AwayDetector::default();
        assert!(!detector.update(Some(1.)));
        assert!(!detector.update(Some(125.)));
        assert!(!detector.update(None));
        assert!(detector.update(Some(0.)));
        assert!(!detector.update(Some(0.)));
    }
}
