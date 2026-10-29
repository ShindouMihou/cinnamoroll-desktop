use super::*;
use crate::companion::{
    corner_home,
    study::Phase,
    world::{Item, Kind},
};

impl Pet {
    pub(super) fn react(&mut self, activity: Activity, now: f64) {
        self.play.cancel();
        self.interaction.reset_stroke();
        self.start(activity, now);
    }
    pub(super) fn quiet_busy(&self) -> bool {
        self.companion.busy()
    }
    pub(super) fn update_study(&mut self, now: f64) {
        if let Some(phase) =
            self.companion
                .study
                .tick(now, self.brain.paused, self.settings.break_minutes)
            && self.drag.is_none()
            && !self.hidden
        {
            if phase == Phase::Break {
                self.break_moment(now);
            } else {
                self.react(crate::companion::Companion::transition_activity(phase), now);
            }
        }
    }
    pub(super) fn touch_pointer(&mut self, now: f64) {
        if self.companion.active_item()
            || self.play.feed_reward
            || !self.settings.touch_enabled
            || self.drag.is_some()
            || self.hidden
            || self.brain.paused
            || self.play.mode == crate::play::Mode::Sleeping
        {
            self.companion.touch.reset();
            return;
        }
        let pointer = native::mouse_position();
        let size = self.size();
        let point = (
            (pointer.0 - self.position.0) * art::WIDTH as f32 / size.0 as f32,
            (pointer.1 - self.position.1) * art::HEIGHT as f32 / size.1 as f32,
        );
        if let Some(activity) = self.companion.touch.sample(point, now) {
            self.interact(activity, now);
        }
    }
    pub(super) fn spawn_item(&mut self, kind: Kind, now: f64) {
        self.interact(Activity::Idle, now);
        if kind != Kind::Treat && self.companion.study.active() {
            self.companion.study.paused = true;
        }
        let size = self.size();
        let bounds = self.bounds();
        let point = crate::companion::placement::item_center(
            kind,
            self.position,
            size,
            bounds,
            if self.settings.cozy_corner {
                Some(corner_home(
                    self.settings.corner_position,
                    self.position,
                    size,
                    bounds,
                ))
            } else {
                None
            },
        );
        self.companion.item = Some(Item::new(kind, point, now));
        self.companion.return_to = Some(if self.settings.cozy_corner {
            corner_home(self.settings.corner_position, self.position, size, bounds)
        } else {
            self.position
        });
    }
    pub(super) fn corner_here(&mut self) {
        self.settings.cozy_corner = true;
        let home = corner_home(None, self.position, self.size(), self.bounds());
        self.settings.corner_position = Some((home.0.round() as i32, home.1.round() as i32));
    }
    pub(super) fn move_toward(
        &mut self,
        target: (f32, f32),
        now: f64,
        dt: f32,
        window: &mut Window,
        stationary: bool,
    ) -> bool {
        let distance = (target.0 - self.position.0).hypot(target.1 - self.position.1);
        if distance < 6. {
            return true;
        }
        if stationary {
            return false;
        }
        let next = crate::play::approach(self.position, target, dt);
        if (next.0 - self.position.0).abs() > 0.1 {
            self.brain.facing = (next.0 - self.position.0).signum();
        }
        if self.brain.activity != Activity::Walk {
            self.start(Activity::Walk, now);
        }
        self.position = self.bounds().clamp(next, self.size());
        self.place(window);
        false
    }
    pub(super) fn advance_companion(
        &mut self,
        now: f64,
        dt: f32,
        window: &mut Window,
        low_battery: bool,
    ) -> bool {
        let stationary = self.settings.reduced_motion || low_battery;
        let bounds = self.bounds();
        let size = self.size();
        let target = corner_home(self.settings.corner_position, self.position, size, bounds);
        let distance = (target.0 - self.position.0).hypot(target.1 - self.position.1);
        let was_going_home = self.companion.homecoming.walking();
        let going_home = self.companion.homecoming.tick(
            now,
            distance,
            (size.0 as f32 * 1.25).max(120.),
            self.settings.cozy_corner
                && self.settings.corner_position.is_some()
                && self.settings.wandering
                && !stationary
                && !self.brain.paused
                && !self.play.busy()
                && !self.quiet_busy(),
            self.brain.activity == Activity::Idle,
        );
        if was_going_home && !going_home && distance < 6. {
            self.start(Activity::Idle, now);
            return true;
        }
        if going_home {
            if self.move_toward(target, now, dt * 0.65, window, false) {
                self.companion.homecoming.reset();
                self.start(Activity::Idle, now);
            }
            return true;
        }
        if self.advance_item(now, dt, window, stationary, bounds, size) {
            return true;
        }
        if let Some((due, activity)) = self.companion.after_action {
            if now >= due {
                self.companion.after_action = None;
                self.react(activity, now);
            }
            return true;
        }
        if let Some(due) = self.companion.chase_return {
            if now < due {
                return true;
            }
            if stationary || !self.settings.cozy_corner {
                self.companion.chase_return = None;
                self.react(Activity::Idle, now);
                return true;
            }
            let target = corner_home(self.settings.corner_position, self.position, size, bounds);
            if self.move_toward(target, now, dt * 0.6, window, false) {
                self.companion.chase_return = None;
                self.companion.resting = true;
                self.start(Activity::CozyNap, now);
                self.play.mode = crate::play::Mode::Sleeping;
            }
            return true;
        }
        let studying = self.companion.study.active();
        if self.companion.resting || studying {
            if !self.companion.resting
                && now < self.brain.until
                && !matches!(
                    self.brain.activity,
                    Activity::Study | Activity::Idle | Activity::Walk | Activity::CozyNap
                )
            {
                return true;
            }
            let at_corner = self.settings.cozy_corner
                && (self.companion.resting || self.settings.study_at_corner);
            if at_corner {
                let target =
                    corner_home(self.settings.corner_position, self.position, size, bounds);
                if !self.move_toward(target, now, dt, window, stationary) && !stationary {
                    return true;
                }
            }
            let activity = if self.companion.resting {
                if self.companion.tucked {
                    Activity::BlanketBurrow
                } else {
                    Activity::CozyNap
                }
            } else if self.companion.study.phase == Phase::Focus && !self.companion.study.paused {
                Activity::Study
            } else {
                Activity::Idle
            };
            if self.brain.activity != activity {
                self.start(activity, now);
            }
            if self.companion.resting {
                self.play.mode = crate::play::Mode::Sleeping;
            }
            return true;
        }
        false
    }
}
