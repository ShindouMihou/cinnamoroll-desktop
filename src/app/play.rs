use super::*;
use crate::play::{Mode, approach, caret_destination};
use chrono::Timelike;

impl Pet {
    pub(super) fn maybe_chase(&mut self, now: f64, low_battery: bool) {
        if !self.quiet_busy()
            && now >= self.next_chase
            && self.settings.occasional_chase
            && !self.settings.reduced_motion
            && !low_battery
            && !self.hidden
            && !self.brain.paused
            && self.drag.is_none()
            && !self.play.busy()
            && self.brain.activity == Activity::Idle
            && now - self.brain.started >= self.brain.quiet_seconds * 0.75
            && self.interaction.mouse_recent(now)
        {
            self.start_chase(now, 10.);
            self.next_chase = now + 300. + fastrand::f64() * 300.;
        }
    }

    pub(super) fn follow_caret(&mut self, target: (f32, f32), now: f64) {
        if self.quiet_busy()
            || self.hidden
            || self.brain.paused
            || self.drag.is_some()
            || self.settings.reduced_motion
            || !self.play.accepts_caret()
            || !matches!(
                self.brain.activity,
                Activity::Idle
                    | Activity::Blink
                    | Activity::LookAround
                    | Activity::Walk
                    | Activity::Curious
                    | Activity::Watching
                    | Activity::Waiting
                    | Activity::Drowsy
                    | Activity::TypingPeek
            )
            || !self.bounds().contains(target)
        {
            return;
        }
        if let Mode::CaretWatch {
            target: previous, ..
        } = self.play.mode
            && (now < self.play.retarget_at
                || (target.0 - previous.0).hypot(target.1 - previous.1) < 80.)
        {
            self.play.mode = Mode::CaretWatch {
                until: now + 8.,
                target: previous,
            };
            return;
        }
        self.play.retarget_at = now + 2.5;
        self.play.mode = Mode::CaretWatch {
            until: now + 8.,
            target,
        };
    }
    pub(super) fn idle_sleep(&mut self, idle: Option<f64>, now: f64) -> bool {
        if self
            .companion
            .recovery_until
            .is_some_and(|until| now >= until)
            && !self.brain.paused
            && !self.hidden
            && self.drag.is_none()
        {
            let computer_asleep = self.settings.sleep_after_minutes > 0
                && idle.is_some_and(|seconds| {
                    seconds >= f64::from(self.settings.sleep_after_minutes) * 60.
                });
            if computer_asleep {
                self.companion.recovery_until = None;
                self.play.auto_sleep = true;
            } else {
                self.interact(Activity::Greet, now);
                return true;
            }
        }
        if self.quiet_busy() && !self.play.auto_sleep {
            return false;
        }
        let Some(idle) = idle else {
            return false;
        };
        if self.play.auto_sleep && idle < 10. && !self.brain.paused {
            self.interact(Activity::Greet, now);
            return true;
        }
        if self.settings.sleep_after_minutes > 0
            && idle >= f64::from(self.settings.sleep_after_minutes) * 60.
            && !self.hidden
            && !self.brain.paused
            && self.drag.is_none()
            && !self.play.busy()
            && matches!(
                self.brain.activity,
                Activity::Idle
                    | Activity::Drowsy
                    | Activity::Waiting
                    | Activity::Watching
                    | Activity::Curious
                    | Activity::Nap
            )
        {
            self.toggle_nap(now);
            self.play.auto_sleep = true;
        }
        false
    }

    pub(super) fn interact(&mut self, activity: Activity, now: f64) {
        self.companion.clear_play();
        self.play.cancel();
        self.interaction.reset_stroke();
        self.start(activity, now);
    }

    pub(super) fn feed(&mut self, now: f64) {
        self.interact(Activity::Snack, now);
        self.brain.until = now + 5.;
        self.play.feed_reward = true;
    }

    pub(super) fn toggle_nap(&mut self, now: f64) {
        if self.play.mode == Mode::Sleeping {
            self.interact(Activity::Greet, now);
        } else if self.settings.cozy_corner {
            self.interact(Activity::CozyNap, now);
            self.companion.resting = true;
            self.companion.study.paused = self.companion.study.active();
        } else {
            self.interact(Activity::Nap, now);
            self.play.mode = Mode::Sleeping;
            self.brain.until = f64::INFINITY;
        }
    }

    pub(super) fn toggle_chase(&mut self, now: f64) {
        if self.play.mode.is_chasing() {
            self.interact(Activity::Happy, now);
        } else if self.settings.reduced_motion {
            self.interact(Activity::Play, now);
        } else {
            self.start_chase(now, 30.);
        }
    }

    pub(super) fn peek(&mut self, now: f64) {
        self.interact(Activity::Waiting, now);
        self.play.mode = Mode::Peek {
            capture_at: now + 3.,
            until: now + 9.,
            target: None,
        };
    }

    pub(super) fn advance(&mut self, now: f64, dt: f32, window: &mut Window, low_battery: bool) {
        if self.brain.paused {
            return;
        }
        if self.play.feed_reward && now >= self.brain.until {
            self.react(Activity::Happy, now);
            return;
        }
        if self.advance_companion(now, dt, window, low_battery) {
            return;
        }
        match self.play.mode {
            Mode::Sleeping => return,
            Mode::EdgePeek(_) => {
                if low_battery {
                    self.interact(Activity::Idle, now);
                    self.position = self.bounds().clamp(self.position, self.size());
                    self.place(window);
                } else {
                    self.advance_edge_peek(now, dt, window);
                }
                return;
            }
            Mode::ChaseCatch { .. } => {
                self.advance_catch(now, window);
                return;
            }
            Mode::CaretWatch { until, target } => {
                if now >= until || low_battery || self.settings.reduced_motion {
                    self.interact(Activity::Idle, now);
                    return;
                }
                let destination = caret_destination(target, self.size(), self.bounds());
                let distance =
                    (destination.0 - self.position.0).hypot(destination.1 - self.position.1);
                let moving = distance
                    > if self.brain.activity == Activity::Walk {
                        6.
                    } else {
                        32.
                    };
                let next = if moving {
                    approach(self.position, destination, dt)
                } else {
                    self.position
                };
                let activity = if moving {
                    Activity::Walk
                } else {
                    Activity::TypingPeek
                };
                let direction = next.0 - self.position.0;
                let arrived = self.brain.activity != activity;
                if arrived {
                    self.start(activity, now);
                }
                if direction.abs() > 0.1 {
                    self.brain.facing = direction.signum();
                }
                self.position = self.bounds().clamp(next, self.size());
                self.place(window);
                let size = self.size();
                let head = (
                    self.position.0 + size.0 as f32 * 0.5,
                    self.position.1 + size.1 as f32 * 0.55,
                );
                if arrived && let Some(direction) = crate::interaction::look_direction(head, target)
                {
                    self.gaze = direction;
                }
                return;
            }
            Mode::Chase { .. } | Mode::ChaseRest { .. } => {
                self.advance_chase(now, dt, window, low_battery);
                return;
            }
            Mode::Peek {
                capture_at,
                until,
                target,
            } => {
                if now >= until {
                    self.interact(Activity::Idle, now);
                } else if now >= capture_at {
                    let target = target.unwrap_or_else(native::mouse_position);
                    self.play.mode = Mode::Peek {
                        capture_at,
                        until,
                        target: Some(target),
                    };
                    if self.brain.activity != Activity::Curious {
                        self.start(Activity::Curious, now);
                    }
                    let size = self.size();
                    let head = (
                        self.position.0 + size.0 as f32 * 0.5,
                        self.position.1 + size.1 as f32 * 0.55,
                    );
                    if let Some(direction) = crate::interaction::look_direction(head, target) {
                        self.gaze = direction;
                    }
                }
                return;
            }
            Mode::Normal => {}
        }
        if self.play.feed_reward && now >= self.brain.until {
            self.interact(Activity::Happy, now);
            return;
        }
        let clamped = self.bounds().clamp(self.position, self.size());
        if clamped != self.position {
            self.position = clamped;
            self.place(window);
        }
        self.brain.tick(now, chrono::Local::now().hour());
        if self.brain.activity == Activity::EdgePeek {
            if low_battery {
                self.start(Activity::Idle, now);
            } else {
                self.begin_edge_peek(now, false);
            }
            return;
        }
        let speed = if self.settings.reduced_motion || low_battery {
            0.
        } else {
            self.brain.speed()
        };
        if speed != 0. {
            let next = (self.position.0 + speed * dt.min(0.1), self.position.1);
            self.position = self.bounds().clamp(next, self.size());
            if (self.position.0 - next.0).abs() > 0.1 {
                self.brain.facing *= -1.;
            }
            self.place(window);
        }
    }
}

#[cfg(test)]
mod tests;
