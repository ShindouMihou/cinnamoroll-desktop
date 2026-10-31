pub mod comfort;
pub mod corner;
pub mod extras;
pub mod games;
pub mod homecoming;
pub mod moments;
pub mod placement;
pub mod preferences;
pub mod routine;
pub mod routing;
pub mod social;
pub mod study;
pub mod touch;
pub mod world;

use crate::{behavior::Activity, geometry::Bounds};
use study::Study;
use touch::Touch;
use world::Item;

#[derive(Default)]
pub struct Companion {
    pub paw_offer: Option<social::PawOffer>,
    pub cozy_return: Option<social::CozyReturn>,
    pub routine: routine::Routine,
    pub route: routing::Route,
    pub moments: moments::Moments,
    pub move_undo: Option<comfort::MoveSnapshot>,
    pub homecoming: homecoming::Homecoming,
    pub study: Study,
    pub touch: Touch,
    pub item: Option<Item>,
    pub resting: bool,
    pub recovery_until: Option<f64>,
    pub tucked: bool,
    pub dance_taps: Vec<f64>,
    pub dance_rate: f64,
    pub next_surprise: f64,
    pub next_item: f64,
    pub chase_return: Option<f64>,
    pub after_action: Option<(f64, Activity)>,
    pub sniff_since: Option<f64>,
    pub return_to: Option<(f32, f32)>,
}

impl Companion {
    pub fn active_item(&self) -> bool {
        self.item
            .as_ref()
            .is_some_and(|item| !item.spontaneous || item.activated || item.held)
    }
    pub fn busy(&self) -> bool {
        self.paw_offer.is_some()
            || self.cozy_return.is_some()
            || self.study.active()
            || self.active_item()
            || self.resting
            || self.after_action.is_some()
            || self.chase_return.is_some()
    }
    pub fn clear_play(&mut self) {
        self.paw_offer = None;
        self.cozy_return = None;
        self.route = Default::default();
        self.homecoming.reset();
        self.item = None;
        self.resting = false;
        self.recovery_until = None;
        self.tucked = false;
        self.sniff_since = None;
        self.return_to = None;
        self.after_action = None;
        self.chase_return = None;
    }
    pub fn offer_ready(&mut self, distance: f32, now: f64) -> bool {
        if distance > 30. {
            self.sniff_since = None;
            return false;
        }
        let since = *self.sniff_since.get_or_insert(now);
        now - since >= 1.2
    }
    pub fn transition_activity(phase: study::Phase) -> Activity {
        if phase == study::Phase::Break {
            Activity::Stretch
        } else {
            Activity::Excited
        }
    }
}

pub fn corner_home(
    saved: Option<(i32, i32)>,
    fallback: (f32, f32),
    size: (u32, u32),
    bounds: Bounds,
) -> (f32, f32) {
    let layout = corner::Layout::new(size);
    let desired = saved.map_or(fallback, |(x, y)| (x as f32, y as f32));
    let group = bounds.clamp((desired.0 + layout.offset.0, desired.1), layout.footprint);
    (group.0 - layout.offset.0, group.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corner_keeps_both_bed_and_pet_inside_negative_display_coordinates() {
        let b = Bounds {
            x: -1200,
            y: -200,
            width: 1200,
            height: 900,
        };
        let home = corner_home(Some((-50, 680)), (-800., 300.), (128, 89), b);
        assert!(home.0 - 16. >= -1200.);
        assert!(home.0 + 144. <= 0.);
        assert!(home.1 >= -200.);
        assert!(home.1 + corner::Layout::new((128, 89)).footprint.1 as f32 <= 700.);
    }
    #[test]
    fn withdrawing_a_treat_restarts_the_sniff_before_acceptance() {
        let mut c = Companion::default();
        assert!(!c.offer_ready(10., 0.));
        assert!(!c.offer_ready(10., 0.8));
        assert!(!c.offer_ready(80., 0.9));
        assert!(!c.offer_ready(10., 1.));
        assert!(!c.offer_ready(10., 1.8));
        assert!(c.offer_ready(10., 2.3));
    }
}
