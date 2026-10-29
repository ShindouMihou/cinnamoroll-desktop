use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub automatic_update_checks: bool,
    pub update_while_away: bool,
    pub extras: crate::companion::preferences::Extras,
    pub focus_minutes: u32,
    pub break_minutes: u32,
    pub study_at_corner: bool,
    pub cozy_corner: bool,
    pub corner_position: Option<(i32, i32)>,
    pub blanket_color: u8,
    pub touch_enabled: bool,
    pub wandering: bool,
    pub music_reactions: bool,
    pub size: u32,
    pub position: Option<(i32, i32)>,
    pub reduced_motion: bool,
    pub occasional_chase: bool,
    pub follow_typing: bool,
    pub idle_pause_seconds: u32,
    pub sleep_after_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            automatic_update_checks: true,
            update_while_away: true,
            extras: crate::companion::preferences::Extras::default(),
            focus_minutes: 25,
            break_minutes: 5,
            study_at_corner: true,
            cozy_corner: false,
            corner_position: None,
            blanket_color: 0,
            touch_enabled: true,
            wandering: true,
            music_reactions: false,
            size: 40,
            position: None,
            reduced_motion: false,
            occasional_chase: false,
            follow_typing: false,
            idle_pause_seconds: 60,
            sleep_after_minutes: 5,
        }
    }
}

pub fn directory() -> io::Result<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("cinnamoroll-desktop"))
        .ok_or_else(|| io::Error::other("Cannot locate the user configuration directory"))
}

pub fn load() -> Result<Settings, String> {
    let path = directory()
        .map_err(|e| e.to_string())?
        .join("settings.json");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let settings: Settings =
                serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            if ![30, 40, 55, 75, 100, 125].contains(&settings.size) {
                return Err("Unsupported pet size in settings".into());
            }
            if !(15..=300).contains(&settings.idle_pause_seconds)
                || settings.sleep_after_minutes > 60
            {
                return Err(
                    "Quiet pause must be 15–300 seconds and sleep timeout 0–60 minutes".into(),
                );
            }
            if !settings.extras.validate() {
                return Err("Invalid study preset, task, or palette in settings".into());
            }
            if !(1..=180).contains(&settings.focus_minutes)
                || !(1..=60).contains(&settings.break_minutes)
                || settings.blanket_color > 2
            {
                return Err(
                    "Focus must be 1–180 minutes, breaks 1–60 minutes, and blanket color 0–2"
                        .into(),
                );
            }
            Ok(settings)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let dir = directory().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    let temporary = dir.join("settings.pending.json");
    std::fs::write(&temporary, data).map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, dir.join("settings.json")).map_err(|e| e.to_string())
}

pub fn load_rules() -> Result<Vec<crate::behavior::IdleRule>, String> {
    let path = directory()
        .map_err(|e| e.to_string())?
        .join("behaviors.json");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let rules: Vec<crate::behavior::IdleRule> =
                serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            crate::behavior::validate_rules(&rules)?;
            Ok(rules)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(crate::behavior::default_rules())
        }
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn older_preferences_keep_their_values_and_receive_student_defaults() {
        let old: Settings =
            serde_json::from_str(r#"{"wandering":false,"size":55,"idle_pause_seconds":90}"#)
                .unwrap();
        assert!(!old.wandering);
        assert_eq!(old.size, 55);
        assert_eq!(old.idle_pause_seconds, 90);
        assert_eq!(old.focus_minutes, 25);
        assert_eq!(old.break_minutes, 5);
        assert!(!old.cozy_corner);
        assert!(old.touch_enabled);
        let encoded = serde_json::to_string(&old).unwrap();
        assert_eq!(old, serde_json::from_str::<Settings>(&encoded).unwrap());
    }
}
