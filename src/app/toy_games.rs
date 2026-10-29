use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn advance_cups(&mut self, now: f64) -> bool {
        let reward = self
            .companion
            .item
            .as_mut()
            .and_then(|item| item.cups.as_mut())
            .and_then(|game| game.advance(now));
        if let Some(found) = reward {
            self.react(
                if found {
                    Activity::Happy
                } else {
                    Activity::Shy
                },
                now,
            );
        } else if now >= self.brain.until && self.brain.activity != Activity::Watching {
            self.start(Activity::Watching, now);
        }
        true
    }
    pub(super) fn advance_touch_toy(&mut self, now: f64, stationary: bool) -> bool {
        let Some(item) = self.companion.item.as_ref() else {
            return false;
        };
        if now < item.reaction_until {
            return true;
        }
        let size = self.size();
        let prop = item.kind.size();
        let local = if item.kind == Kind::Ribbon {
            (20., 84.)
        } else {
            (18., 73.)
        };
        let point = (
            item.position.0 - prop.0 as f32 / 2. + local.0 / 160. * prop.0 as f32,
            item.position.1 - prop.1 as f32 / 2. + local.1 / 112. * prop.1 as f32,
        );
        let paw = (
            self.position.0
                + size.0 as f32
                    * if point.0 >= self.position.0 + size.0 as f32 * 0.5 {
                        0.65
                    } else {
                        0.35
                    },
            self.position.1 + size.1 as f32 * 0.80,
        );
        let near = (point.0 - paw.0).hypot(point.1 - paw.1) < size.0 as f32 * 0.55;
        if near && now - item.last_pop >= 3.8 && !stationary {
            self.brain.facing = (point.0 - (self.position.0 + size.0 as f32 * 0.5)).signum();
            self.react(Activity::PawPlay, now);
            if let Some(item) = self.companion.item.as_mut() {
                item.last_pop = now;
                item.reaction_until = now + 1.6;
            }
        } else if self.brain.activity != Activity::Watching {
            self.start(Activity::Watching, now);
        }
        true
    }
}
