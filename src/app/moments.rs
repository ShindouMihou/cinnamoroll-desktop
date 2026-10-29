use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn update_moments(&mut self, now: f64, low_battery: bool) {
        let eligible = self.play.mode == crate::play::Mode::Sleeping
            && self.settings.extras.dream_bubbles
            && !self.settings.reduced_motion
            && !low_battery
            && !self.hidden
            && !self.brain.paused
            && self.drag.is_none()
            && !self.companion.study.active()
            && self.companion.item.is_none();
        self.companion.moments.dream(now, eligible);
    }
    pub(super) fn advance_moment(&mut self, now: f64) -> bool {
        let Some(item) = self.companion.item.as_ref() else {
            return false;
        };
        if !matches!(
            item.kind,
            Kind::Brush
                | Kind::Dandelion
                | Kind::SnowGlobe
                | Kind::Pond
                | Kind::KeepsakeBox
                | Kind::ClearZone
        ) {
            return false;
        }
        let kind = item.kind;
        let point = item.position;
        let held = item.held;
        let last = item.last_pop;
        let stage = item.stage;
        let size = self.size();
        let head = (
            self.position.0 + size.0 as f32 * 0.5,
            self.position.1 + size.1 as f32 * 0.38,
        );
        let near = (point.0 - head.0).hypot(point.1 - head.1) < size.0 as f32 * 0.25;
        let brush_contact = (
            (point.0 - self.position.0 - 12.) / size.0 as f32,
            (point.1 - self.position.1 - 10.) / size.1 as f32,
        );
        let brush_near =
            (0.1..0.9).contains(&brush_contact.0) && (0.15..0.65).contains(&brush_contact.1);
        if kind == Kind::ClearZone {
            return true;
        }
        if kind == Kind::Brush {
            let stroke = self.companion.moments.brush(point, now, held && brush_near);
            if stroke {
                if self.brain.activity != Activity::Groom || now >= self.brain.until {
                    self.react(Activity::Groom, now);
                }
                self.settings.extras.keepsakes |= 1;
                self.dirty = true;
                if let Some(item) = self.companion.item.as_mut() {
                    item.last_pop = now;
                }
            } else if now - last > 4.
                && now >= self.brain.until
                && self.brain.activity != Activity::Watching
            {
                self.start(Activity::Watching, now);
            }
        } else if kind == Kind::Dandelion && held && near && stage == 0 {
            self.react(Activity::Sneeze, now);
            if let Some(item) = self.companion.item.as_mut() {
                item.stage = 1;
                item.last_pop = now;
                item.expires = now + 8.;
            }
            self.settings.extras.keepsakes |= 2;
            self.dirty = true;
        } else if kind == Kind::SnowGlobe && held && item.velocity.0.hypot(item.velocity.1) > 65. {
            if let Some(item) = self.companion.item.as_mut() {
                item.last_pop = now;
                item.stage = 1;
            }
            if now >= self.brain.until {
                self.react(Activity::Curious, now);
            }
            self.dirty |= self.settings.extras.keepsakes & 4 == 0;
            self.settings.extras.keepsakes |= 4;
        } else if kind == Kind::Pond && held && item.velocity.0.abs() > 0.5 && now - last > 3. {
            self.react(Activity::PawPlay, now);
            if let Some(item) = self.companion.item.as_mut() {
                item.last_pop = now;
            }
            self.settings.extras.keepsakes |= 8;
            self.dirty = true;
        } else if kind == Kind::KeepsakeBox {
            if now >= self.brain.until && self.brain.activity != Activity::Happy {
                self.start(Activity::Happy, now);
            }
        } else if now >= self.brain.until && self.brain.activity != Activity::Watching {
            self.start(Activity::Watching, now);
        }
        true
    }
}
