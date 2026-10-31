use crate::behavior::Activity;

#[derive(Default)]
pub struct Routine {
    remaining: Vec<Activity>,
    last: Option<Activity>,
}

impl Routine {
    pub fn next(&mut self) -> Activity {
        if self.remaining.is_empty() {
            self.remaining.extend([
                Activity::TeaSip,
                Activity::PlushHug,
                Activity::EarWiggle,
                Activity::Read,
                Activity::Dance,
                Activity::Play,
                Activity::Wave,
                Activity::Stretch,
                Activity::Curious,
            ]);
            fastrand::shuffle(&mut self.remaining);
            if self.remaining.last().copied() == self.last {
                self.remaining.swap(0, 8);
            }
        }
        let activity = self.remaining.pop().unwrap_or(Activity::Wave);
        self.last = Some(activity);
        activity
    }

    pub fn delay(quiet_seconds: f64, first: bool) -> f64 {
        let minimum = if first { 45. } else { 120. };
        let spread = if first { 45. } else { 120. };
        quiet_seconds.max(minimum) + fastrand::f64() * spread
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffled_routines_exhaust_the_collection_and_avoid_boundary_repeats() {
        let mut routine = Routine::default();
        let mut last = None;
        for _ in 0..100 {
            let mut seen = std::collections::HashSet::new();
            for _ in 0..9 {
                let next = routine.next();
                assert_ne!(Some(next), last);
                assert!(seen.insert(next));
                last = Some(next);
            }
        }
    }

    #[test]
    fn routine_delays_respect_long_quiet_preferences() {
        for first in [true, false] {
            assert!(Routine::delay(600., first) >= 600.);
        }
    }
}
