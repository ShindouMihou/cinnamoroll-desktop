use super::*;
use crate::play::{Mode, catch_activity, pounce_progress, pounce_target};

impl Pet {
    pub(super) fn catch_cursor(&mut self, pointer: (f32, f32), now: f64) -> bool {
        let size = self.size();
        let center = (
            self.position.0 + size.0 as f32 * 0.5,
            self.position.1 + size.1 as f32 * 0.6,
        );
        if (pointer.0 - center.0).hypot(pointer.1 - center.1) > size.0 as f32 * 0.4 + 26. {
            return false;
        }
        let target = pounce_target(self.position, size, pointer, self.bounds());
        if (pointer.0 - center.0).abs() > 2. {
            self.brain.facing = (pointer.0 - center.0).signum();
        }
        self.play.mode = Mode::ChaseCatch {
            started: now,
            origin: self.position,
            target,
        };
        self.play.motion = Default::default();
        self.start(Activity::ChasePause, now);
        true
    }

    pub(super) fn advance_catch(&mut self, now: f64, window: &mut Window) {
        let Mode::ChaseCatch {
            started,
            origin,
            target,
        } = self.play.mode
        else {
            return;
        };
        let elapsed = (now - started).max(0.);
        let Some(activity) = catch_activity(elapsed) else {
            self.finish_chase(now);
            return;
        };
        if self.settings.reduced_motion {
            self.interact(Activity::Happy, now);
            return;
        }
        if self.brain.activity != activity {
            self.start(activity, now);
        }
        let progress = pounce_progress(elapsed);
        let eased = progress * progress * (3. - 2. * progress);
        self.position = self.bounds().clamp(
            (
                origin.0 + (target.0 - origin.0) * eased,
                origin.1 + (target.1 - origin.1) * eased,
            ),
            self.size(),
        );
        self.place(window);
    }
}
