#[derive(Default)]
pub struct Homecoming {
    quiet_since: Option<f64>,
    walking: bool,
}

impl Homecoming {
    pub fn walking(&self) -> bool {
        self.walking
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn hold(&mut self, elapsed: f64) {
        if let Some(since) = self.quiet_since.as_mut() {
            *since += elapsed;
        }
    }
    pub fn tick(
        &mut self,
        now: f64,
        distance: f32,
        far: f32,
        available: bool,
        can_start: bool,
    ) -> bool {
        if !available || distance < 6. {
            self.reset();
            return false;
        }
        if self.walking {
            return true;
        }
        if distance < far {
            self.reset();
            return false;
        }
        let since = *self.quiet_since.get_or_insert(now);
        if now - since >= 120. && can_start {
            self.walking = true;
        }
        self.walking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_for_a_quiet_interval_and_finishes_the_walk() {
        let mut home = Homecoming::default();
        assert!(!home.tick(0., 400., 150., true, true));
        assert!(!home.tick(119., 400., 150., true, true));
        assert!(!home.tick(121., 400., 150., true, false));
        assert!(home.tick(122., 400., 150., true, true));
        assert!(home.tick(124., 80., 150., true, false));
        assert!(!home.tick(126., 4., 150., true, true));
    }

    #[test]
    fn interactions_busy_modes_and_menu_time_restart_or_hold_the_wait() {
        let mut home = Homecoming::default();
        home.tick(0., 400., 150., true, true);
        home.hold(60.);
        assert!(!home.tick(130., 400., 150., true, true));
        assert!(!home.tick(181., 400., 150., false, true));
        assert!(!home.tick(200., 400., 150., true, true));
        home.reset();
        assert!(!home.tick(340., 400., 150., true, true));
        assert!(!home.tick(500., 80., 150., true, true));
        assert!(!home.tick(600., 400., 150., true, true));
    }
}
