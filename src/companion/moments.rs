#[derive(Default)]
pub struct Moments {
    pub next_dream: f64,
    pub dream: Option<(f64, u8)>,
    brush_last: Option<((f32, f32), f64)>,
    brush_distance: f32,
}

impl Moments {
    pub fn brush(&mut self, point: (f32, f32), now: f64, near: bool) -> bool {
        if !near {
            self.brush_last = None;
            self.brush_distance = 0.;
            return false;
        }
        if let Some((previous, last)) = self.brush_last
            && now - last < 0.5
        {
            self.brush_distance += (point.0 - previous.0).hypot(point.1 - previous.1).min(20.);
        }
        self.brush_last = Some((point, now));
        if self.brush_distance >= 35. {
            self.brush_distance = 0.;
            return true;
        }
        false
    }
    pub fn dream(&mut self, now: f64, eligible: bool) {
        if !eligible {
            self.dream = None;
            self.next_dream = now + 90.;
            return;
        }
        if self.next_dream == 0. {
            self.next_dream = now + 90.;
        }
        if self.dream.is_some_and(|(end, _)| now >= end) {
            self.dream = None;
        }
        if now >= self.next_dream {
            self.dream = Some((now + 8., fastrand::u8(..3)));
            self.next_dream = now + 150. + fastrand::f64() * 150.;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brushing_requires_real_strokes_and_dreams_clear_outside_naps() {
        let mut m = Moments::default();
        for t in 0..20 {
            assert!(!m.brush((10., 10.), t as f64 * 0.1, true));
        }
        assert!(!m.brush((25., 10.), 2., true));
        assert!(m.brush((45., 10.), 2.1, true));
        m.dream(0., true);
        m.dream(91., true);
        assert!(m.dream.is_some());
        m.dream(92., false);
        assert!(m.dream.is_none());
    }
}
