use super::*;
use crate::companion::world::Kind;

impl Pet {
    pub(super) fn advance_item(
        &mut self,
        now: f64,
        dt: f32,
        window: &mut Window,
        stationary: bool,
        bounds: Bounds,
        size: (u32, u32),
    ) -> bool {
        if self
            .companion
            .item
            .as_ref()
            .is_some_and(|item| now >= item.expires)
        {
            self.companion.item = None;
            self.companion.sniff_since = None;
            self.companion.return_to = None;
        }
        if let Some(item) = self.companion.item.as_mut() {
            if matches!(item.kind, Kind::Ball | Kind::Plane)
                && item.ready_for_fetch(now)
                && !item.carried
            {
                item.landed = true;
                item.velocity = (0., 0.);
            }
            item.step(dt, bounds, stationary);
        }
        if let Some(item) = self.companion.item.as_ref() {
            if item.spontaneous && !item.activated && !item.held {
                return false;
            }
            let kind = item.kind;
            if !matches!(kind, Kind::Ball | Kind::Treat | Kind::Bubbles) {
                return self.advance_extra_item(now, dt, window, stationary, bounds, size);
            }
            let point = item.position;
            let held = item.held;
            let carried = item.carried;
            if kind == Kind::Ball && carried {
                let target = self.companion.return_to.unwrap_or(self.position);
                if self.move_toward(target, now, dt, window, stationary) || stationary {
                    self.companion.item = None;
                    self.companion.return_to = None;
                    self.react(Activity::Fetch, now);
                } else if let Some(item) = self.companion.item.as_mut() {
                    item.position = (
                        self.position.0 + size.0 as f32 * 0.5,
                        self.position.1 + size.1 as f32 * 0.68,
                    );
                }
                return true;
            }
            let destination = bounds.clamp(
                (
                    point.0 - size.0 as f32 * 0.5,
                    point.1 - size.1 as f32 * 0.54,
                ),
                size,
            );
            let mouth = (
                self.position.0 + size.0 as f32 * 0.5,
                self.position.1 + size.1 as f32 * 0.54,
            );
            let distance = (mouth.0 - point.0).hypot(mouth.1 - point.1);
            match kind {
                Kind::Treat => {
                    if !item.activated {
                        if self.brain.activity != Activity::Curious {
                            self.start(Activity::Curious, now);
                        }
                        return true;
                    }
                    if distance > 30. {
                        self.companion.offer_ready(distance, now);
                        if !self.move_toward(destination, now, dt, window, stationary)
                            && stationary
                            && self.brain.activity != Activity::Idle
                        {
                            self.start(Activity::Idle, now);
                        }
                    } else {
                        let ready = self.companion.offer_ready(distance, now);
                        if self.brain.activity != Activity::SniffTreat {
                            self.start(Activity::SniffTreat, now);
                        }
                        if ready {
                            self.companion.item = None;
                            self.companion.sniff_since = None;
                            self.react(Activity::TakeTreat, now);
                            self.play.feed_reward = true;
                        }
                    }
                }
                Kind::Ball => {
                    if !held && item.activated && item.ready_for_fetch(now) {
                        if distance <= 32.
                            || (destination.0 - self.position.0)
                                .hypot(destination.1 - self.position.1)
                                < 6.
                        {
                            if let Some(item) = self.companion.item.as_mut() {
                                item.carried = true;
                                item.velocity = (0., 0.);
                            }
                        } else {
                            self.move_toward(destination, now, dt, window, stationary);
                        }
                    } else if self.brain.activity != Activity::Curious {
                        self.start(Activity::Curious, now);
                    }
                }
                _ => {
                    if self.brain.activity == Activity::BubblePop && now < self.brain.until {
                        return true;
                    }
                    if self.brain.activity != Activity::Watching {
                        self.start(Activity::Watching, now);
                    }
                }
            }
            return true;
        }
        false
    }
}
