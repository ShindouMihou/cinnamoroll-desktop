#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Phase {
    #[default]
    Idle,
    Focus,
    Break,
    Ready,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct View {
    pub phase: Phase,
    pub seconds: u32,
    pub paused: bool,
}

#[derive(Default)]
pub struct Study {
    pub phase: Phase,
    pub paused: bool,
    remaining: f64,
    last: Option<f64>,
}

impl Study {
    pub fn active(&self) -> bool {
        matches!(self.phase, Phase::Focus | Phase::Break)
    }
    pub fn view(&self) -> View {
        View {
            phase: self.phase,
            seconds: self.remaining.ceil().max(0.) as u32,
            paused: self.paused,
        }
    }
    pub fn start(&mut self, now: f64, minutes: u32) {
        self.phase = Phase::Focus;
        self.remaining = f64::from(minutes) * 60.;
        self.paused = false;
        self.last = Some(now);
    }
    pub fn toggle_pause(&mut self, now: f64) {
        if self.active() {
            self.paused = !self.paused;
            self.last = Some(now);
        }
    }
    pub fn stop(&mut self) {
        *self = Self::default();
    }
    pub fn extend(&mut self) {
        if self.active() {
            self.remaining = (self.remaining + 300.).min(14400.);
        }
    }
    pub fn skip(&mut self, now: f64, break_minutes: u32) -> Option<Phase> {
        if !self.active() {
            return None;
        }
        self.phase = if self.phase == Phase::Focus {
            Phase::Break
        } else {
            Phase::Ready
        };
        self.remaining = if self.phase == Phase::Break {
            f64::from(break_minutes) * 60.
        } else {
            0.
        };
        self.paused = false;
        self.last = Some(now);
        Some(self.phase)
    }
    pub fn tick(&mut self, now: f64, suspended: bool, break_minutes: u32) -> Option<Phase> {
        let elapsed = self
            .last
            .replace(now)
            .map_or(0., |last| (now - last).max(0.));
        if !self.active() || self.paused || suspended {
            return None;
        }
        self.remaining = (self.remaining - elapsed).max(0.);
        if self.remaining == 0. {
            self.skip(now, break_minutes)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sessions_pause_extend_skip_and_require_an_explicit_next_focus() {
        let mut s = Study::default();
        s.start(0., 1);
        s.tick(20., false, 1);
        assert_eq!(s.view().seconds, 40);
        s.toggle_pause(20.);
        s.tick(120., false, 1);
        assert_eq!(s.view().seconds, 40);
        s.toggle_pause(120.);
        s.extend();
        assert_eq!(s.view().seconds, 340);
        assert_eq!(s.tick(460., false, 1), Some(Phase::Break));
        assert_eq!(s.view().seconds, 60);
        assert_eq!(s.tick(520., false, 1), Some(Phase::Ready));
        assert!(!s.active());
        assert_eq!(s.tick(9999., false, 1), None);
        s.start(10000., 1);
        assert_eq!(s.skip(10001., 1), Some(Phase::Break));
        s.stop();
        assert_eq!(s.phase, Phase::Idle);
    }
    #[test]
    fn global_pause_does_not_consume_time_or_catch_up_after_resume() {
        let mut s = Study::default();
        s.start(0., 25);
        s.tick(10., false, 5);
        s.tick(110., true, 5);
        s.tick(111., false, 5);
        assert_eq!(s.view().seconds, 1489);
    }
}
