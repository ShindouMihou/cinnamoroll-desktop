use super::*;
use crate::companion::world::{Item, Kind};

impl Pet {
    pub(super) fn maybe_item(&mut self, now: f64, low_battery: bool) {
        if !self.settings.extras.surprise_items {
            return;
        }
        if self.companion.next_item == 0. {
            self.schedule_item(now);
        }
        if now < self.companion.next_item
            || self.hidden
            || self.brain.paused
            || self.settings.reduced_motion
            || low_battery
            || self.drag.is_some()
            || self.quiet_busy()
            || self.companion.item.is_some()
            || self.play.busy()
            || self.brain.activity != Activity::Idle
            || now - self.brain.started < 15.
        {
            return;
        }
        self.schedule_item(now);
        let kinds = [
            Kind::Treat,
            Kind::Wand,
            Kind::Ribbon,
            Kind::Tea,
            Kind::Plush,
            Kind::Blanket,
        ];
        let kind = kinds[fastrand::usize(..kinds.len())];
        let pet = self.size();
        let home = self.settings.cozy_corner.then(|| {
            crate::companion::corner_home(
                self.settings.corner_position,
                self.position,
                pet,
                self.bounds(),
            )
        });
        let point =
            crate::companion::placement::item_center(kind, self.position, pet, self.bounds(), home);
        let mut item = Item::new(kind, point, now);
        item.spontaneous = true;
        item.expires = now + 120.;
        self.companion.item = Some(item);
    }

    fn schedule_item(&mut self, now: f64) {
        let interval = f64::from(self.settings.extras.item_interval_minutes) * 60.;
        self.companion.next_item = now + interval * (0.8 + fastrand::f64() * 0.4);
    }
}
