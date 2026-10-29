use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn companion_command(&mut self, command: Command, now: f64) -> bool {
        match command {
            Command::StudyStart => {
                self.interact(Activity::Study, now);
                self.companion.study.start(now, self.settings.focus_minutes);
                if self.settings.cozy_corner && self.settings.corner_position.is_none() {
                    self.corner_here();
                }
            }
            Command::StudyPause => {
                self.companion.study.toggle_pause(now);
                self.companion.clear_play();
                self.react(
                    if self.companion.study.paused {
                        Activity::Idle
                    } else {
                        Activity::Study
                    },
                    now,
                );
            }
            Command::StudySkip => {
                if let Some(phase) = self.companion.study.skip(now, self.settings.break_minutes) {
                    if phase == crate::companion::study::Phase::Break {
                        self.break_moment(now);
                    } else {
                        self.react(crate::companion::Companion::transition_activity(phase), now);
                    }
                }
            }
            Command::StudyExtend => self.companion.study.extend(),
            Command::StudyStop => {
                self.companion.study.stop();
                self.interact(Activity::Idle, now);
            }
            Command::AdjustFocus(delta) => {
                self.settings.focus_minutes =
                    (self.settings.focus_minutes as i32 + delta).clamp(1, 180) as u32
            }
            Command::AdjustBreak(delta) => {
                self.settings.break_minutes =
                    (self.settings.break_minutes as i32 + delta).clamp(1, 60) as u32
            }
            Command::StudyAtCorner => {
                self.settings.study_at_corner = !self.settings.study_at_corner
            }
            Command::CozyCorner => {
                self.settings.cozy_corner = !self.settings.cozy_corner;
                if self.settings.cozy_corner && self.settings.corner_position.is_none() {
                    self.corner_here();
                }
                if !self.settings.cozy_corner && self.companion.resting {
                    self.interact(Activity::Idle, now);
                }
            }
            Command::CornerHere => self.corner_here(),
            Command::CornerRest => {
                if !self.settings.cozy_corner {
                    self.corner_here();
                }
                self.interact(Activity::CozyNap, now);
                self.companion.resting = true;
                self.companion.study.paused = self.companion.study.active();
            }
            Command::Blanket(color) => self.settings.blanket_color = color.min(2),
            Command::TouchEnabled => {
                self.settings.touch_enabled = !self.settings.touch_enabled;
                self.companion.touch.reset();
            }
            Command::OfferTreat => self.spawn_item(Kind::Treat, now),
            Command::ToyBall => self.spawn_item(Kind::Ball, now),
            Command::ToyBubbles => self.spawn_item(Kind::Bubbles, now),
            Command::PutAway => {
                self.companion.clear_play();
                self.interact(Activity::Idle, now);
            }
            _ => return false,
        }
        true
    }
}
