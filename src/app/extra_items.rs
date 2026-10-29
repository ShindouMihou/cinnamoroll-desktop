use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn advance_extra_item(
        &mut self,
        now: f64,
        dt: f32,
        window: &mut Window,
        stationary: bool,
        bounds: Bounds,
        size: (u32, u32),
    ) -> bool {
        let Some(item) = self.companion.item.as_ref() else {
            return false;
        };
        let kind = item.kind;
        if kind == Kind::Pillow && item.carried {
            if let Some(item) = self.companion.item.as_mut() {
                item.position = (
                    self.position.0 + size.0 as f32 * 0.5,
                    self.position.1 + size.1 as f32 * 0.88,
                );
            }
            if self.brain.activity != Activity::CozyNap {
                self.start(Activity::CozyNap, now);
            }
            self.play.mode = crate::play::Mode::Sleeping;
            return true;
        }
        if kind == Kind::Breathing {
            if self.brain.activity != Activity::Breathe {
                self.start(Activity::Breathe, now);
            }
            return true;
        }
        let point = item.position;
        let held = item.held;
        let stage = item.stage;
        let activated = item.activated;
        let last = item.last_pop;
        let carried = item.carried;
        let target = bounds.clamp(
            (
                point.0 - size.0 as f32 * 0.5,
                point.1 - size.1 as f32 * 0.65,
            ),
            size,
        );
        let distance = (self.position.0 - target.0).hypot(self.position.1 - target.1);
        if kind == Kind::Plane && carried {
            let home = self.companion.return_to.unwrap_or(self.position);
            if stationary || self.move_toward(home, now, dt, window, false) {
                self.companion.item = None;
                self.react(Activity::PlaneReturn, now);
            } else if let Some(item) = self.companion.item.as_mut() {
                item.position = (
                    self.position.0 + size.0 as f32 * 0.5,
                    self.position.1 + size.1 as f32 * 0.65,
                );
                item.stage = u8::from(self.brain.facing < 0.);
            }
            return true;
        }
        if kind == Kind::HideCups {
            return self.advance_cups(now);
        }
        if kind == Kind::Parcel {
            if activated && stage > 0 && now - last > 0.1 {
                self.companion.item = None;
                self.react(Activity::GiftOpen, now);
                {
                    use crate::companion::preferences::Accessory;
                    self.settings.extras.accessory = [
                        Accessory::Bow,
                        Accessory::Scarf,
                        Accessory::Hat,
                        Accessory::Bag,
                    ][fastrand::usize(..4)];
                    self.settings.extras.last_gift_day = Some(crate::companion::extras::day());
                    self.dirty = true;
                }
            } else if !activated && self.brain.activity != Activity::Curious {
                self.start(Activity::Curious, now);
            }
            return true;
        }
        if !activated {
            if self.brain.activity != Activity::Curious {
                self.start(Activity::Curious, now);
            }
            return true;
        }
        if matches!(kind, Kind::Wand | Kind::Ribbon) {
            return self.advance_touch_toy(now, stationary);
        }
        if kind == Kind::Plane
            && !self
                .companion
                .item
                .as_ref()
                .is_some_and(|item| item.ready_for_fetch(now))
        {
            if self.brain.activity != Activity::Watching {
                self.start(Activity::Watching, now);
            }
            return true;
        }
        if distance > 28. {
            self.move_toward(target, now, dt, window, stationary);
            return true;
        }
        if now < self.brain.until
            && matches!(
                self.brain.activity,
                Activity::PawPlay | Activity::TeaSip | Activity::PlushHug | Activity::Curious
            )
            && now - last < 2.5
        {
            return true;
        }
        match kind {
            Kind::Plane if !held => {
                if let Some(item) = self.companion.item.as_mut() {
                    item.carried = true;
                    item.velocity = (0., 0.);
                }
            }
            Kind::Pillow if !held => {
                self.react(Activity::CozyNap, now);
                self.companion.resting = true;
                if let Some(item) = self.companion.item.as_mut() {
                    item.carried = true;
                    item.expires = f64::INFINITY;
                }
            }
            Kind::Blanket => {
                self.companion.item = None;
                self.react(Activity::BlanketBurrow, now);
                self.companion.resting = true;
                self.companion.tucked = true;
            }
            Kind::Tea | Kind::Plush | Kind::Palette if !held => {
                self.companion.item = None;
                self.react(
                    match kind {
                        Kind::Tea => Activity::TeaSip,
                        Kind::Plush => Activity::PlushHug,
                        _ => Activity::Curious,
                    },
                    now,
                );
            }
            _ => {}
        }
        true
    }
}
