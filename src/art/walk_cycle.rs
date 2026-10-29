#[derive(Default)]
pub(crate) struct WalkCycle {
    previous: Option<(f32, f32)>,
    active: bool,
    phase: f64,
}

impl WalkCycle {
    pub fn update(&mut self, position: (f32, f32), width: u32, active: bool) -> f64 {
        if let Some(previous) = self.previous
            && active
            && self.active
        {
            let distance = (position.0 - previous.0).hypot(position.1 - previous.1);
            let width = width.max(1) as f32;
            if distance.is_finite() && distance <= width * 0.5 {
                self.phase = (self.phase + f64::from(distance / (width * 0.24))) % 1.;
            }
        }
        self.previous = Some(position);
        self.active = active;
        self.phase
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feet_advance_with_travel_and_do_not_step_while_stopped_or_being_dragged() {
        let mut cycle = WalkCycle::default();
        cycle.update((0., 0.), 128, true);
        let first = cycle.update((4., 0.), 128, true);
        let second = cycle.update((8., 0.), 128, true);
        assert!(first > 0. && second > first);
        assert_eq!(cycle.update((8., 0.), 128, true), second);
        assert_eq!(cycle.update((900., 0.), 128, false), second);
        assert_eq!(cycle.update((900., 0.), 128, true), second);
        assert!(cycle.update((904., 0.), 128, true) > second);
    }
    #[test]
    fn slow_and_fast_routes_have_the_same_steps_for_the_same_distance() {
        let mut slow = WalkCycle::default();
        let mut fast = WalkCycle::default();
        slow.update((0., 0.), 128, true);
        fast.update((0., 0.), 128, true);
        for x in 1..=24 {
            slow.update((x as f32, 0.), 128, true);
        }
        for x in [8., 16., 24.] {
            fast.update((x, 0.), 128, true);
        }
        assert!((slow.phase - fast.phase).abs() < 0.00001);
    }
}
