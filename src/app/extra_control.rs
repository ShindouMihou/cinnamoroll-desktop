use super::*;
use crate::companion::{
    preferences::{BreakChoice, Season, StudyPreset},
    world::Kind,
};

impl Pet {
    pub(super) fn extra_command(&mut self, command: &Command, now: f64, tray: &Tray) -> bool {
        match command {
            Command::UsePreset(index) => {
                if let Some(preset) = self.settings.extras.presets.get(*index).cloned() {
                    self.settings.focus_minutes = preset.focus;
                    self.settings.break_minutes = preset.rest;
                    self.settings.study_at_corner = preset.corner;
                    self.companion_command(Command::StudyStart, now);
                }
            }
            Command::SavePreset(index) => {
                if let Some(preset) = self.settings.extras.presets.get_mut(*index) {
                    *preset = StudyPreset {
                        name: preset.name.clone(),
                        focus: self.settings.focus_minutes,
                        rest: self.settings.break_minutes,
                        corner: self.settings.study_at_corner,
                    };
                }
            }
            Command::PresetText(index, text) => {
                let text = text.trim();
                if !text.is_empty()
                    && text.chars().count() <= 24
                    && !text.chars().any(char::is_control)
                    && let Some(preset) = self.settings.extras.presets.get_mut(*index)
                {
                    preset.name = text.into();
                }
            }
            Command::TaskText(text) => {
                let text = text.trim();
                if text.chars().count() <= 80 && !text.chars().any(char::is_control) {
                    self.settings.extras.task = text.into();
                    self.settings.extras.task_done = false;
                }
            }
            Command::TaskDone => {
                if !self.settings.extras.task.is_empty() && !self.settings.extras.task_done {
                    self.settings.extras.task_done = true;
                    self.interact(Activity::Happy, now);
                }
            }
            Command::BreakRoutine(choice) => self.settings.extras.break_choice = *choice,
            Command::ChooseBreak(choice) => {
                self.settings.extras.break_choice = *choice;
                if self.companion.study.phase == crate::companion::study::Phase::Focus {
                    self.companion.study.skip(now, self.settings.break_minutes);
                }
                self.break_moment(now);
            }
            Command::PaletteText(text) => {
                if let Some(colors) = crate::companion::extras::palette(text) {
                    self.settings.extras.palette = colors;
                }
            }
            Command::Accessory(accessory) => {
                self.settings.extras.accessory = *accessory;
                self.interact(Activity::Happy, now);
            }
            Command::Season(season) => {
                self.settings.extras.season = *season;
                self.interact(
                    match season {
                        Season::Rain => Activity::RainPlay,
                        Season::Autumn => Activity::LeafPlay,
                        Season::Birthday => Activity::Birthday,
                        Season::None => Activity::Idle,
                    },
                    now,
                );
            }
            Command::Surprises => {
                self.settings.extras.ambient_surprises = !self.settings.extras.ambient_surprises
            }
            Command::SurpriseItems => {
                self.settings.extras.surprise_items = !self.settings.extras.surprise_items;
                self.companion.next_item = 0.;
                if !self.settings.extras.surprise_items
                    && self
                        .companion
                        .item
                        .as_ref()
                        .is_some_and(|item| item.spontaneous && !item.activated && !item.held)
                {
                    self.companion.item = None;
                }
            }
            Command::AdjustItemInterval(delta) => {
                self.settings.extras.item_interval_minutes = self
                    .settings
                    .extras
                    .item_interval_minutes
                    .saturating_add_signed(*delta)
                    .clamp(1, 60);
                self.companion.next_item = 0.;
            }
            Command::Prop(kind) => self.spawn_item(*kind, now),
            Command::TapDance => {
                if self
                    .companion
                    .dance_taps
                    .last()
                    .is_none_or(|last| now - last > 3.)
                {
                    self.companion.dance_taps.clear();
                }
                self.companion.dance_taps.push(now);
                if self.companion.dance_taps.len() > 5 {
                    self.companion.dance_taps.remove(0);
                }
                let taps = &self.companion.dance_taps;
                self.companion.dance_rate = if taps.len() > 1 {
                    let interval = (taps[taps.len() - 1] - taps[0]) / (taps.len() - 1) as f64;
                    (0.6 / interval.max(0.25)).clamp(0.5, 2.2)
                } else {
                    1.
                };
                if self.brain.activity != Activity::Dance {
                    self.interact(Activity::Dance, now);
                }
                self.brain.until = now + 12.;
                tray.status("Tap K to set the dance rhythm");
            }
            Command::Sticker => {
                match crate::art::stickers::save(
                    self.brain.activity,
                    self.settings.extras.accessory,
                ) {
                    Ok(path) => tray.status(&format!("Sticker saved: {}", path.display())),
                    Err(error) => {
                        eprintln!("Sticker export: {error}");
                        tray.status("Couldn’t export sticker; check Pictures folder access");
                    }
                }
            }
            _ => return false,
        }
        true
    }
    pub(super) fn break_moment(&mut self, now: f64) {
        match self.settings.extras.break_choice {
            BreakChoice::Stretch => self.interact(Activity::Stretch, now),
            BreakChoice::Water => self.interact(Activity::TeaSip, now),
            BreakChoice::Breathe => {
                self.interact(Activity::Breathe, now);
                let size = self.size();
                let mut item = crate::companion::world::Item::new(
                    Kind::Breathing,
                    (
                        self.position.0 + size.0 as f32 + 36.,
                        self.position.1 + size.1 as f32 * 0.5,
                    ),
                    now,
                );
                item.expires = now + 24.;
                item.activated = true;
                self.companion.item = Some(item);
            }
            BreakChoice::Play => {
                self.spawn_item(Kind::Bubbles, now);
                self.companion.study.paused = false;
            }
        }
    }
    pub(super) fn maybe_surprise(&mut self, now: f64, low_battery: bool) {
        if self.companion.next_surprise == 0. {
            self.companion.next_surprise = now + 360. + fastrand::f64() * 420.;
        }
        if now < self.companion.next_surprise
            || !self.settings.extras.ambient_surprises
            || self.hidden
            || self.brain.paused
            || self.settings.reduced_motion
            || low_battery
            || self.drag.is_some()
            || self.quiet_busy()
            || self.companion.item.is_some()
            || self.play.busy()
            || self.brain.activity != Activity::Idle
            || now - self.brain.started < 15.
        {
            return;
        }
        self.companion.next_surprise = now + 480. + fastrand::f64() * 420.;
        if self.settings.extras.last_gift_day != Some(crate::companion::extras::day())
            && fastrand::u8(..8) == 0
        {
            self.spawn_item(Kind::Parcel, now);
            return;
        }
        let seasonal = match self.settings.extras.season {
            Season::Rain => Some(Activity::RainPlay),
            Season::Autumn => Some(Activity::LeafPlay),
            Season::Birthday => Some(Activity::Birthday),
            Season::None => None,
        };
        let choice = if fastrand::bool() { seasonal } else { None }.unwrap_or_else(|| {
            [
                Activity::EarWiggle,
                Activity::Sneeze,
                Activity::PlushHug,
                Activity::TeaSip,
            ][fastrand::usize(..4)]
        });
        self.interact(choice, now);
    }
}
