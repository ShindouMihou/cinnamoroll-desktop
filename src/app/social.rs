use super::*;
use crate::companion::{
    social::{CozyReturn, PawOffer, Response},
    world::Kind,
};

impl Pet {
    pub(super) fn offer_paw(&mut self, now: f64) {
        self.interact(Activity::HighFive, now);
        self.companion.touch.reset();
        self.companion.paw_offer = Some(PawOffer::new(now));
        self.brain.until = now + 10.;
    }

    pub(super) fn respond_to_paw(&mut self, pointer: (f32, f32), now: f64, low_battery: bool) {
        let size = self.size();
        let point = (
            (pointer.0 - self.position.0) * art::WIDTH as f32 / size.0 as f32,
            (pointer.1 - self.position.1) * art::HEIGHT as f32 / size.1 as f32,
        );
        let Some(offer) = self.companion.paw_offer.as_mut() else {
            return;
        };
        if self.hidden || self.brain.paused || self.drag.is_some() {
            return;
        }
        let response =
            if !self.settings.touch_enabled || self.settings.reduced_motion || low_battery {
                Response::Expired
            } else {
                offer.sample(point, now)
            };
        match response {
            Response::Accepted => self.interact(Activity::HighFive, now),
            Response::Expired => self.interact(Activity::Idle, now),
            Response::Waiting => {}
        }
    }

    pub(super) fn tidy_plush(&mut self, now: f64) -> bool {
        if !self.settings.cozy_corner || !self.settings.wandering {
            return false;
        }
        let home = crate::companion::corner_home(
            self.settings.corner_position,
            self.position,
            self.size(),
            self.bounds(),
        );
        if (home.0 - self.position.0).hypot(home.1 - self.position.1) > self.size().0 as f32 {
            return false;
        }
        self.spawn_item(Kind::Plush, now);
        let Some(item) = self.companion.item.as_mut() else {
            return false;
        };
        item.activated = true;
        self.companion.cozy_return = Some(CozyReturn {
            due: f64::INFINITY,
            home,
        });
        true
    }

    pub(super) fn welcome_back(&mut self, now: f64, low_battery: bool) {
        self.interact(Activity::Greet, now);
        if self.settings.extras.ambient_surprises && !self.settings.reduced_motion && !low_battery {
            self.companion.after_action = Some((now + 4., Activity::Stretch));
        }
    }
}
