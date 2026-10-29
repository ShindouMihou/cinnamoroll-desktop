use super::*;
use crate::play::Mode;

impl Pet {
    pub(super) fn finish_chase(&mut self, now: f64) {
        self.interact(Activity::Pant, now);
        if self.settings.cozy_corner && !self.settings.reduced_motion {
            self.companion.chase_return = Some(now + 3.);
        }
    }

    pub(super) fn start_chase(&mut self, now: f64, duration: f64) {
        if self.settings.cozy_corner && self.settings.corner_position.is_none() {
            self.corner_here();
            self.dirty = true;
        }
        self.interact(Activity::Walk, now);
        self.play.mode = Mode::Chase {
            until: now + duration,
            tired_at: now + 5.,
        };
    }

    pub(super) fn chase_phase(&mut self, pointer: (f32, f32), now: f64, low_battery: bool) -> bool {
        if low_battery || self.settings.reduced_motion {
            self.interact(Activity::Happy, now);
            return false;
        }
        match self.play.mode {
            Mode::Chase { until, tired_at } => {
                if now >= until {
                    self.finish_chase(now);
                    return false;
                }
                if self.catch_cursor(pointer, now) {
                    return false;
                }
                if now >= tired_at {
                    self.play.mode = Mode::ChaseRest {
                        until: now + 5.,
                        chase_until: until + 5.,
                    };
                    self.play.motion = Default::default();
                    self.start(Activity::Pant, now);
                    return false;
                }
                true
            }
            Mode::ChaseRest { until, chase_until } => {
                if now >= until {
                    self.play.mode = Mode::Chase {
                        until: chase_until,
                        tired_at: now + 5.,
                    };
                    self.play.motion = Default::default();
                    self.start(Activity::Walk, now);
                }
                false
            }
            _ => false,
        }
    }

    pub(super) fn advance_chase(
        &mut self,
        now: f64,
        dt: f32,
        window: &mut Window,
        low_battery: bool,
    ) {
        let pointer = native::mouse_position();
        if !self.chase_phase(pointer, now, low_battery) {
            return;
        }
        let next = self
            .play
            .motion
            .step(self.position, self.size(), pointer, self.bounds(), dt);
        let speed = self
            .play
            .motion
            .velocity
            .0
            .hypot(self.play.motion.velocity.1);
        let moving = speed
            > if self.brain.activity == Activity::Walk {
                2.
            } else {
                6.
            };
        let activity = if moving {
            Activity::Walk
        } else {
            Activity::Idle
        };
        let direction = next.0 - self.position.0;
        if self.brain.activity != activity {
            self.start(activity, now);
        }
        if self.play.motion.velocity.0.abs() > 12. {
            self.brain.facing = direction.signum();
        }
        if moving {
            self.position = next;
        } else {
            self.play.motion.velocity = (0., 0.);
        }
        self.place(window);
    }
}
