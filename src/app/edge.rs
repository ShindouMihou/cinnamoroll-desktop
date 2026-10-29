use super::*;
use crate::play::{EdgeStage, Mode, approach, edge_plan};

impl Pet {
    pub(super) fn begin_edge_peek(&mut self, now: f64, manual: bool) {
        if let Mode::EdgePeek(mut visit) = self.play.mode {
            visit.stage = EdgeStage::Leaving;
            self.play.mode = Mode::EdgePeek(visit);
            self.start(Activity::Walk, now);
            self.brain.facing = visit.facing;
            return;
        }
        if self.settings.reduced_motion || (!manual && !self.settings.wandering) {
            self.interact(Activity::Idle, now);
            return;
        }
        let Some(visit) = edge_plan(self.position, self.size(), self.bounds(), !manual) else {
            self.interact(Activity::Idle, now);
            return;
        };
        self.interact(Activity::Walk, now);
        self.play.mode = Mode::EdgePeek(visit);
        self.brain.facing = -visit.facing;
    }

    pub(super) fn advance_edge_peek(&mut self, now: f64, dt: f32, window: &mut Window) {
        let Mode::EdgePeek(mut visit) = self.play.mode else {
            return;
        };
        if !self.monitors.contains(&visit.bounds) || self.settings.reduced_motion {
            self.interact(Activity::Idle, now);
            self.position = self.bounds().clamp(visit.origin, self.size());
            self.place(window);
            return;
        }
        let target = match visit.stage {
            EdgeStage::Entering => visit.target,
            EdgeStage::Watching { until } => {
                if now >= until {
                    visit.stage = EdgeStage::Leaving;
                    self.start(Activity::Walk, now);
                    self.brain.facing = visit.facing;
                }
                self.play.mode = Mode::EdgePeek(visit);
                return;
            }
            EdgeStage::Leaving => visit.origin,
        };
        self.brain.facing = if target.0 < self.position.0 { -1. } else { 1. };
        self.position = approach(self.position, target, dt);
        if (self.position.0 - target.0).abs() < 1. {
            self.position = target;
            if visit.stage == EdgeStage::Entering {
                visit.stage = EdgeStage::Watching { until: now + 6. };
                self.start(Activity::EdgePeek, now);
                self.brain.facing = visit.facing;
            } else {
                self.interact(Activity::Idle, now);
                self.dirty = true;
                self.place(window);
                return;
            }
        }
        self.play.mode = Mode::EdgePeek(visit);
        self.place(window);
    }
}
