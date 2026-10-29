use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn restore_nook(&mut self) {
        if !self.settings.cozy_corner {
            return;
        }
        let settled = self.settings.corner_position.is_some_and(|p| {
            (self.position.0 - p.0 as f32).hypot(self.position.1 - p.1 as f32) < 12.
        });
        let display = if settled {
            self.settings
                .extras
                .comfort
                .preferred_display
                .filter(|d| self.monitors.contains(d))
                .unwrap_or_else(|| self.bounds())
        } else {
            self.bounds()
        };
        let desired = self
            .settings
            .extras
            .comfort
            .nook(display)
            .or(self.settings.corner_position)
            .map(|p| (p.0 as f32, p.1 as f32))
            .unwrap_or(self.position);
        let home = crate::companion::corner_home(None, desired, self.size(), display);
        self.settings.corner_position = Some((home.0.round() as i32, home.1.round() as i32));
        if settled {
            self.position = home;
        }
        self.dirty = true;
    }
    pub(super) fn comfort_command(
        &mut self,
        command: &Command,
        now: f64,
        window: &mut Window,
        tray: &Tray,
    ) -> bool {
        match command {
            Command::UndoMove => {
                if let Some(old) = self.companion.move_undo.take() {
                    self.interact(Activity::Land, now);
                    let display = self
                        .monitors
                        .iter()
                        .find(|d| d.contains(old.pet))
                        .copied()
                        .unwrap_or(self.monitors[0]);
                    self.position = display.clamp(old.pet, self.size());
                    self.settings.corner_position = old.corner.map(|p| {
                        let point = (p.0 as f32, p.1 as f32);
                        let d = self
                            .monitors
                            .iter()
                            .find(|d| d.contains(point))
                            .copied()
                            .unwrap_or(display);
                        let home = crate::companion::corner_home(None, point, self.size(), d);
                        self.settings.extras.comfort.preferred_display = Some(d);
                        (home.0.round() as i32, home.1.round() as i32)
                    });
                    self.place(window);
                } else {
                    tray.status("Nothing to move back yet");
                }
            }
            Command::ClearZones => self.settings.extras.comfort.zones.clear(),
            Command::DreamBubbles => {
                self.settings.extras.dream_bubbles = !self.settings.extras.dream_bubbles
            }
            Command::StudyStopwatch => {
                self.interact(Activity::Study, now);
                self.companion.study.start_stopwatch(now);
            }
            Command::StudyResume => {
                if self.companion.study.active() {
                    self.companion.clear_play();
                    self.companion.study.paused = false;
                    self.react(Activity::Study, now);
                }
            }
            Command::StudyDiscard => {
                self.companion.study.stop();
                self.settings.extras.study_bookmark = None;
                self.interact(Activity::Idle, now);
            }
            Command::Prop(Kind::ClearZone) => {
                if self.settings.extras.comfort.zones.len() >= 8 {
                    tray.status("Up to eight keep-clear zones; clear them from Desktop comfort");
                } else {
                    self.spawn_item(Kind::ClearZone, now);
                }
            }
            _ => return false,
        }
        true
    }
}
