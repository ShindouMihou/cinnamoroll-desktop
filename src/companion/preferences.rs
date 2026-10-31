use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Accessory {
    #[default]
    None,
    Bow,
    Scarf,
    Hat,
    Bag,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    #[default]
    None,
    Rain,
    Autumn,
    Birthday,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakChoice {
    #[default]
    Stretch,
    Water,
    Breathe,
    Play,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StudyPreset {
    pub name: String,
    pub focus: u32,
    pub rest: u32,
    pub corner: bool,
}

impl Default for StudyPreset {
    fn default() -> Self {
        Self {
            name: "Study".into(),
            focus: 25,
            rest: 5,
            corner: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Extras {
    pub comfort: super::comfort::Preferences,
    pub keepsakes: u8,
    pub dream_bubbles: bool,
    pub study_bookmark: Option<super::study::Bookmark>,
    pub presets: [StudyPreset; 3],
    pub task: String,
    pub task_done: bool,
    pub break_choice: BreakChoice,
    pub palette: [u32; 3],
    pub accessory: Accessory,
    pub season: Season,
    pub ambient_surprises: bool,
    pub surprise_items: bool,
    pub item_interval_minutes: u32,
    pub last_gift_day: Option<u64>,
}

impl Default for Extras {
    fn default() -> Self {
        Self {
            comfort: super::comfort::Preferences::default(),
            keepsakes: 0,
            dream_bubbles: true,
            study_bookmark: None,
            presets: [
                StudyPreset::default(),
                StudyPreset {
                    name: "Quick start".into(),
                    focus: 10,
                    rest: 3,
                    corner: true,
                },
                StudyPreset {
                    name: "Deep focus".into(),
                    focus: 45,
                    rest: 10,
                    corner: true,
                },
            ],
            task: String::new(),
            task_done: false,
            break_choice: BreakChoice::Stretch,
            palette: [0xb0daf5, 0xf8cddc, 0xdbd1f6],
            accessory: Accessory::None,
            season: Season::None,
            ambient_surprises: true,
            surprise_items: true,
            item_interval_minutes: 8,
            last_gift_day: None,
        }
    }
}

impl Extras {
    pub fn validate(&self) -> bool {
        self.comfort.valid()
            && self
                .study_bookmark
                .as_ref()
                .is_none_or(super::study::Bookmark::valid)
            && self.keepsakes <= 15
            && (1..=60).contains(&self.item_interval_minutes)
            && self.task.chars().count() <= 80
            && !self.task.chars().any(char::is_control)
            && self.palette.iter().all(|c| *c <= 0xffffff)
            && self.presets.iter().all(|p| {
                (1..=180).contains(&p.focus)
                    && (1..=60).contains(&p.rest)
                    && !p.name.is_empty()
                    && p.name.chars().count() <= 24
                    && !p.name.chars().any(char::is_control)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extras_defaults_round_trip_and_reject_unbounded_or_invalid_user_data() {
        let mut s = Extras::default();
        assert!(s.validate());
        let text = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Extras>(&text).unwrap(), s);
        s.task = "é".repeat(80);
        assert!(s.validate());
        s.task.push('x');
        assert!(!s.validate());
        s.task.clear();
        s.presets[0].name.clear();
        assert!(!s.validate());
        s.presets[0].name = "Biology".into();
        s.palette[0] = 0x1000000;
        assert!(!s.validate());
    }
}

#[cfg(test)]
mod surprise_defaults_tests {
    use super::*;
    #[test]
    fn surprises_default_on_without_overriding_a_saved_opt_out() {
        assert!(Extras::default().surprise_items);
        let missing: Extras = serde_json::from_str("{}").unwrap();
        assert!(missing.surprise_items);
        let opted_out: Extras = serde_json::from_str(r#"{"surprise_items":false}"#).unwrap();
        assert!(!opted_out.surprise_items);
        assert!(
            !serde_json::from_str::<Extras>(&serde_json::to_string(&opted_out).unwrap())
                .unwrap()
                .surprise_items
        );
    }
}
