#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    #[default]
    Idle,
    Focus,
    Break,
    Ready,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct View {
    pub stopwatch: bool,
    pub phase: Phase,
    pub seconds: u32,
    pub paused: bool,
}

#[derive(Default)]
pub struct Study {
    pub stopwatch: bool,
    pub phase: Phase,
    pub paused: bool,
    remaining: f64,
    last: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bookmark {
    pub phase: Phase,
    pub seconds: u32,
    pub stopwatch: bool,
}
impl Bookmark {
    pub fn valid(&self) -> bool {
        matches!(self.phase, Phase::Focus | Phase::Break) && self.seconds <= 604800
    }
}

impl Study {
    pub fn start_stopwatch(&mut self, now: f64) {
        self.start(now, 1);
        self.stopwatch = true;
        self.remaining = 0.;
    }
    pub fn bookmark(&self) -> Option<Bookmark> {
        self.active().then(|| Bookmark {
            phase: self.phase,
            seconds: self.view().seconds,
            stopwatch: self.stopwatch,
        })
    }
    pub fn restore(&mut self, bookmark: &Bookmark, now: f64) {
        if bookmark.valid() {
            self.phase = bookmark.phase;
            self.remaining = f64::from(bookmark.seconds);
            self.stopwatch = bookmark.stopwatch;
            self.paused = true;
            self.last = Some(now);
        }
    }

    pub fn active(&self) -> bool {
        matches!(self.phase, Phase::Focus | Phase::Break)
    }
    pub fn view(&self) -> View {
        View {
            stopwatch: self.stopwatch,
            phase: self.phase,
            seconds: self.remaining.ceil().max(0.) as u32,
            paused: self.paused,
        }
    }
    pub fn start(&mut self, now: f64, minutes: u32) {
        self.stopwatch = false;
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
        if self.active() && !self.stopwatch {
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
        if self.stopwatch && self.phase == Phase::Focus {
            self.remaining = (self.remaining + elapsed).min(604800.);
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

#[cfg(test)]
mod bookmark_tests {
    use super::*;
    #[test]
    fn stopwatch_counts_only_active_time_and_break_is_a_countdown() {
        let mut s = Study::default();
        s.start_stopwatch(10.);
        s.tick(20., false, 1);
        assert_eq!(s.view().seconds, 10);
        s.toggle_pause(20.);
        s.tick(50., false, 1);
        assert_eq!(s.view().seconds, 10);
        s.toggle_pause(50.);
        s.tick(55., false, 1);
        assert_eq!(s.view().seconds, 15);
        s.skip(55., 1);
        s.tick(65., false, 1);
        assert_eq!(s.view().phase, Phase::Break);
        assert_eq!(s.view().seconds, 50);
    }
    #[test]
    fn bookmarks_restore_paused_without_counting_closed_or_sleeping_time() {
        let mut first = Study::default();
        first.start(0., 25);
        first.tick(10., false, 5);
        let bookmark = first.bookmark().unwrap();
        let mut restored = Study::default();
        restored.restore(&bookmark, 10000.);
        restored.tick(20000., false, 5);
        assert_eq!(restored.view().seconds, 1490);
        assert!(restored.paused);
        restored.toggle_pause(20000.);
        restored.tick(20005., false, 5);
        assert_eq!(restored.view().seconds, 1485);
        restored.tick(30000., true, 5);
        restored.tick(30001., false, 5);
        assert_eq!(restored.view().seconds, 1484);
        restored.stop();
        assert!(restored.bookmark().is_none());
    }
}
