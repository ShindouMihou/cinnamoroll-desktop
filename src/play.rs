mod edge;
pub use edge::{EdgeStage, EdgeVisit, plan as edge_plan};
mod chase;
use crate::geometry::Bounds;
#[cfg(test)]
use chase::chase_step;
pub use chase::{ChaseMotion, catch_activity, pounce_progress, pounce_target};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Sleeping,
    EdgePeek(EdgeVisit),
    CaretWatch {
        until: f64,
        target: (f32, f32),
    },
    Chase {
        until: f64,
        tired_at: f64,
    },
    ChaseRest {
        until: f64,
        chase_until: f64,
    },
    ChaseCatch {
        started: f64,
        origin: (f32, f32),
        target: (f32, f32),
    },
    Peek {
        capture_at: f64,
        until: f64,
        target: Option<(f32, f32)>,
    },
}

impl Mode {
    pub fn is_chasing(self) -> bool {
        matches!(
            self,
            Self::Chase { .. } | Self::ChaseRest { .. } | Self::ChaseCatch { .. }
        )
    }
}

#[derive(Default)]
pub struct PlayState {
    pub mode: Mode,
    pub feed_reward: bool,
    pub auto_sleep: bool,
    pub motion: ChaseMotion,
    pub retarget_at: f64,
}

impl PlayState {
    pub fn cancel(&mut self) {
        *self = Self::default();
    }
    pub fn busy(&self) -> bool {
        self.mode != Mode::Normal || self.feed_reward
    }
    pub fn accepts_caret(&self) -> bool {
        !self.feed_reward && matches!(self.mode, Mode::Normal | Mode::CaretWatch { .. })
    }
    pub fn resume(&mut self, duration: f64) {
        match &mut self.mode {
            Mode::Chase { until, tired_at } => {
                *until += duration;
                *tired_at += duration;
            }
            Mode::ChaseRest { until, chase_until } => {
                *until += duration;
                *chase_until += duration;
            }
            Mode::CaretWatch { until, .. } => *until += duration,
            Mode::ChaseCatch { started, .. } => *started += duration,
            Mode::EdgePeek(visit) => {
                if let EdgeStage::Watching { until } = &mut visit.stage {
                    *until += duration;
                }
            }
            Mode::Peek {
                capture_at, until, ..
            } => {
                *capture_at += duration;
                *until += duration;
            }
            _ => {}
        }
    }
}

pub fn caret_destination(point: (f32, f32), size: (u32, u32), bounds: Bounds) -> (f32, f32) {
    let right = bounds.x as f32 + bounds.width as f32;
    let bottom = bounds.y as f32 + bounds.height as f32;
    let x = if point.0 + 28. + size.0 as f32 <= right {
        point.0 + 28.
    } else {
        point.0 - size.0 as f32 - 28.
    };
    let y = if point.1 - size.1 as f32 - 24. >= bounds.y as f32 {
        point.1 - size.1 as f32 - 24.
    } else {
        (point.1 + 24.).min(bottom - size.1 as f32)
    };
    bounds.clamp((x, y), size)
}

pub fn approach(position: (f32, f32), target: (f32, f32), dt: f32) -> (f32, f32) {
    let delta = (target.0 - position.0, target.1 - position.1);
    let distance = delta.0.hypot(delta.1);
    if distance < 0.1 {
        return target;
    }
    let step = distance.min(55_f32.min(distance * 2.) * dt.clamp(0., 0.1));
    (
        position.0 + delta.0 / distance * step,
        position.1 + delta.1 / distance * step,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_pounce_lands_short_of_the_pointer_without_retreating_if_already_close() {
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 1000,
            height: 800,
        };
        let size = (128, 89);
        let pointer = (150., 153.4);
        let landed = pounce_target((0., 100.), size, pointer, bounds);
        assert!(landed.0 > 0.);
        assert!((pointer.0 - (landed.0 + 64.) - 58.08).abs() < 0.01);
        assert_eq!(
            pounce_target((0., 100.), size, (104., 153.4), bounds),
            (0., 100.)
        );
    }
    #[test]
    fn a_catch_runs_one_pounce_stumble_and_dizzy_sequence_and_pause_shifts_its_start() {
        use crate::behavior::Activity;
        assert_eq!(catch_activity(0.), Some(Activity::ChasePause));
        assert_eq!(catch_activity(chase::READY_AT), Some(Activity::Stalk));
        assert_eq!(catch_activity(chase::POUNCE_AT), Some(Activity::Pounce));
        assert_eq!(catch_activity(chase::STUMBLE_AT), Some(Activity::Stumble));
        assert_eq!(catch_activity(chase::DIZZY_AT), Some(Activity::Dizzy));
        assert_eq!(catch_activity(chase::FINISH_AT), None);
        assert_eq!(
            chase::POUNCE_AT - chase::READY_AT - chase::READY_TRANSITION,
            2.
        );
        assert_eq!(pounce_progress(chase::POUNCE_AT - 0.01), 0.);
        assert!(pounce_progress(chase::POUNCE_AT + 0.3) > 0.49);
        let mut play = PlayState {
            mode: Mode::ChaseCatch {
                started: 10.,
                origin: (0., 0.),
                target: (40., 0.),
            },
            ..PlayState::default()
        };
        play.resume(5.);
        assert_eq!(
            play.mode,
            Mode::ChaseCatch {
                started: 15.,
                origin: (0., 0.),
                target: (40., 0.)
            }
        );
        assert!(!play.accepts_caret());
    }
    #[test]
    fn chase_keeps_a_gap_and_cannot_teleport_after_a_long_frame() {
        let bounds = Bounds {
            x: -500,
            y: 0,
            width: 1500,
            height: 900,
        };
        let position = (100., 100.);
        assert_eq!(
            chase_step(position, (128, 89), (170., 160.), bounds, 1.),
            position
        );
        let moved = chase_step(position, (128, 89), (900., 500.), bounds, 20.);
        assert!((moved.0 - position.0).hypot(moved.1 - position.1) <= 10.51);
        let edge = chase_step((872., 800.), (128, 89), (5000., 5000.), bounds, 0.1);
        assert!(edge.0 <= 872. && edge.1 <= 811.);
    }
    #[test]
    fn chasing_eases_in_and_reverses_gradually_instead_of_snapping() {
        let bounds = Bounds {
            x: -1000,
            y: -1000,
            width: 3000,
            height: 3000,
        };
        let mut motion = ChaseMotion::default();
        let first = motion.step((100., 100.), (128, 89), (900., 150.), bounds, 0.033);
        assert!(first.0 > 100. && first.0 < 101.);
        for _ in 0..30 {
            motion.step((100., 100.), (128, 89), (900., 150.), bounds, 0.033);
        }
        let previous = motion.velocity.0;
        motion.step((100., 100.), (128, 89), (-900., 150.), bounds, 0.033);
        assert!(motion.velocity.0 > 0. && motion.velocity.0 < previous);
    }
    #[test]
    fn caret_visits_keep_clear_of_the_text_and_stay_inside_negative_coordinate_displays() {
        let bounds = Bounds {
            x: -1440,
            y: -200,
            width: 1440,
            height: 900,
        };
        let size = (128, 89);
        let point = (-700., 100.);
        let target = caret_destination(point, size, bounds);
        assert_eq!(target, (-672., -13.));
        let edge = caret_destination((-2., 699.), size, bounds);
        assert!(edge.0 + (size.0 as f32) < 0. && edge.1 + (size.1 as f32) < 699.);
        let next = approach((0., 0.), (1., 0.), 0.1);
        assert!(next.0 > 0. && next.0 < 1.);
    }
    #[test]
    fn pausing_shifts_play_deadlines_and_cancel_resets_all_pending_actions() {
        let mut play = PlayState {
            mode: Mode::Chase {
                until: 30.,
                tired_at: 5.,
            },
            feed_reward: true,
            auto_sleep: false,
            ..PlayState::default()
        };
        play.resume(15.);
        assert_eq!(
            play.mode,
            Mode::Chase {
                until: 45.,
                tired_at: 20.
            }
        );
        play.cancel();
        assert!(!play.busy());
        play.mode = Mode::Peek {
            capture_at: 3.,
            until: 9.,
            target: None,
        };
        play.resume(5.);
        assert_eq!(
            play.mode,
            Mode::Peek {
                capture_at: 8.,
                until: 14.,
                target: None
            }
        );
    }
}
