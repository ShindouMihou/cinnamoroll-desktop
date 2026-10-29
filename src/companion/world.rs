use crate::geometry::Bounds;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Ball,
    Treat,
    Bubbles,
    Wand,
    Ribbon,
    Plane,
    Blanket,
    Tea,
    HideCups,
    Parcel,
    Plush,
    Pillow,
    Palette,
    Breathing,
}

pub struct Item {
    pub kind: Kind,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub held: bool,
    pub carried: bool,
    pub activated: bool,
    pub spontaneous: bool,
    pub bubbles: u8,
    pub expires: f64,
    pub last_pop: f64,
    pub stage: u8,
    pub cups: Option<super::games::CupRound>,
    pub flight_since: Option<f64>,
    pub landed: bool,
    pub reaction_until: f64,
    pub popped: u8,
    sample: Option<(f64, (f32, f32))>,
}

impl Item {
    pub fn new(kind: Kind, position: (f32, f32), now: f64) -> Self {
        Self {
            kind,
            position,
            velocity: (0., 0.),
            held: false,
            carried: false,
            activated: kind == Kind::Bubbles,
            spontaneous: false,
            bubbles: 7,
            expires: now + 60.,
            last_pop: now,
            stage: 0,
            cups: (kind == Kind::HideCups)
                .then(|| super::games::CupRound::new(now, fastrand::u8(..3))),
            flight_since: None,
            landed: false,
            reaction_until: 0.,
            popped: 0,
            sample: None,
        }
    }
    pub fn grab(&mut self, now: f64) {
        if matches!(self.kind, Kind::Wand | Kind::Ribbon) && !self.activated {
            self.last_pop = now - 3.8;
        }
        self.held = true;
        self.landed = false;
        self.flight_since = None;
        if self.kind != Kind::Ball && self.kind != Kind::Plane {
            self.activated = true;
        }
        self.velocity = (0., 0.);
        self.sample = Some((now, self.position));
        self.expires = now + 120.;
    }
    pub fn drag(&mut self, position: (f32, f32), now: f64) {
        if let Some((last, previous)) = self.sample {
            let dt = (now - last) as f32;
            if dt > 0.004 {
                self.velocity = (
                    (position.0 - previous.0) / dt,
                    (position.1 - previous.1) / dt,
                );
            }
        }
        self.position = position;
        self.sample = Some((now, position));
        self.expires = now + 120.;
    }
    pub fn release(&mut self) {
        if !self.held {
            return;
        }
        self.held = false;
        self.activated = true;
        self.flight_since = self.sample.map(|sample| sample.0);
        if self.kind == Kind::Plane && self.velocity.0.hypot(self.velocity.1) < 60. {
            self.velocity = (140., -45.);
        }
        if matches!(self.kind, Kind::Ball | Kind::Plane) {
            self.velocity.0 = self.velocity.0.clamp(-320., 320.);
            self.velocity.1 = self.velocity.1.clamp(-320., 320.);
        } else {
            self.velocity = (0., 0.);
        }
    }
    pub fn release_at(&mut self, now: f64) {
        let held = self.held;
        self.release();
        if held && matches!(self.kind, Kind::Ball | Kind::Plane) {
            self.flight_since = Some(now);
        }
    }
    pub fn ready_for_fetch(&self, now: f64) -> bool {
        self.landed
            || (!self.held
                && self.activated
                && self.flight_since.is_some_and(|since| {
                    let elapsed = now - since;
                    if self.kind == Kind::Plane {
                        elapsed >= 1.5
                    } else {
                        elapsed >= 4.
                            || (elapsed >= 0.4 && self.velocity.0.hypot(self.velocity.1) < 30.)
                    }
                }))
    }
    pub fn hint(&self) -> Option<&'static str> {
        if self.held || self.carried {
            return None;
        }
        if let Some(cups) = self.cups.as_ref() {
            return Some(cups.hint());
        }
        match self.kind {
            Kind::Bubbles => Some("Click a bubble to pop it"),
            Kind::Ball if !self.activated => Some("Drag and release to toss"),
            Kind::Plane if !self.activated => Some("Drag and release to launch"),
            Kind::Wand if !self.activated => Some("Bring the pompom near a paw"),
            Kind::Ribbon if !self.activated => Some("Drag the bow; offer the end"),
            _ => None,
        }
    }
    pub fn pop_at(&mut self, point: (f32, f32)) -> bool {
        let circles = [(22.5, 47.5, 12.5), (52.5, 24.5, 14.5), (75.5, 53.5, 11.)];
        for (index, (x, y, radius)) in circles.into_iter().enumerate() {
            if self.bubbles & (1 << index) != 0 && (point.0 - x).hypot(point.1 - y) <= radius + 2. {
                self.bubbles &= !(1 << index);
                self.popped = index as u8 + 1;
                return true;
            }
        }
        false
    }
    pub fn step(&mut self, dt: f32, bounds: Bounds, reduced: bool) {
        if self.held
            || self.carried
            || (matches!(self.kind, Kind::Ball | Kind::Plane) && !self.activated)
        {
            return;
        }
        let dt = dt.clamp(0., 0.05);
        let margin = self.kind.size();
        let left = bounds.x as f32 + margin.0 as f32 / 2.;
        let right = (bounds.x as f32 + bounds.width as f32 - margin.0 as f32 / 2.).max(left);
        let top = bounds.y as f32 + margin.1 as f32 / 2.;
        let floor = (bounds.y as f32 + bounds.height as f32 - margin.1 as f32 / 2.).max(top);
        if !reduced && !self.landed {
            match self.kind {
                Kind::Ball => {
                    self.velocity.1 += 360. * dt;
                    self.position.0 += self.velocity.0 * dt;
                    self.position.1 += self.velocity.1 * dt;
                    self.velocity.0 *= (-0.8 * dt).exp();
                    if self.position.1 >= floor {
                        self.position.1 = floor;
                        self.velocity.1 = if self.velocity.1.abs() < 35. {
                            0.
                        } else {
                            -self.velocity.1.abs() * 0.55
                        };
                        self.velocity.0 *= 0.94;
                    }
                    if self.position.0 <= left || self.position.0 >= right {
                        self.velocity.0 *= -0.65;
                    }
                }
                Kind::Bubbles => {}
                Kind::Plane => {
                    if self.velocity.0.abs() > 1. {
                        self.stage = u8::from(self.velocity.0 < 0.);
                    }
                    self.velocity.1 += 25. * dt;
                    self.position.0 += self.velocity.0 * dt;
                    self.position.1 += self.velocity.1 * dt;
                    self.velocity.0 *= (-0.2 * dt).exp();
                    if self.position.0 <= left || self.position.0 >= right {
                        self.velocity.0 *= -0.5;
                    }
                    if self.position.1 >= floor {
                        self.velocity = (0., 0.);
                    }
                }
                _ => {}
            }
        }
        self.position.0 = self.position.0.clamp(left, right);
        self.position.1 = self.position.1.clamp(top, floor);
    }
}

impl Kind {
    pub fn size(self) -> (u32, u32) {
        match self {
            Self::Ball | Self::Treat => (36, 36),
            Self::Bubbles => (96, 80),
            Self::HideCups => (180, 96),
            Self::Palette => (120, 72),
            Self::Pillow => (84, 40),
            _ => (80, 64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thrown_balls_are_bounded_settle_and_never_teleport_after_a_stall() {
        let b = Bounds {
            x: -100,
            y: -50,
            width: 500,
            height: 400,
        };
        let mut item = Item::new(Kind::Ball, (0., 0.), 0.);
        item.grab(0.);
        item.drag((40., -10.), 0.1);
        item.release();
        item.step(10., b, false);
        assert!((item.position.0 - 40.).abs() <= 30.);
        for _ in 0..4000 {
            item.step(0.016, b, false);
            assert!(b.contains(item.position));
        }
        assert!(item.velocity.0.abs() < 0.1);
        assert!(item.velocity.1.abs() < 35.);
        item.grab(100.);
        let old = item.position;
        item.step(0.016, b, false);
        assert_eq!(item.position, old);
    }
    #[test]
    fn paper_planes_and_large_props_stay_inside_their_full_display_footprint() {
        let b = Bounds {
            x: -600,
            y: -200,
            width: 600,
            height: 600,
        };
        let mut plane = Item::new(Kind::Plane, (-300., 0.), 0.);
        plane.grab(0.);
        plane.drag((-240., -40.), 0.01);
        plane.release();
        assert!(plane.velocity.0.abs() <= 600. && plane.velocity.1.abs() <= 600.);
        for _ in 0..1000 {
            plane.step(0.016, b, false);
            assert!(plane.position.0 >= -560. && plane.position.0 <= -40.);
            assert!(plane.position.1 >= -168. && plane.position.1 <= 368.);
        }
        for kind in [
            Kind::Wand,
            Kind::Ribbon,
            Kind::Blanket,
            Kind::Tea,
            Kind::HideCups,
            Kind::Parcel,
            Kind::Plush,
            Kind::Pillow,
            Kind::Palette,
        ] {
            let mut item = Item::new(kind, (-300., 0.), 0.);
            item.grab(0.);
            item.step(10., b, false);
            assert_eq!(item.position, (-300., 0.));
            item.release();
            item.position = (10000., 10000.);
            item.step(0.05, b, true);
            let size = kind.size();
            assert!(item.position.0 + size.0 as f32 / 2. <= 0.);
            assert!(item.position.1 + size.1 as f32 / 2. <= 400.);
        }
    }
    #[test]
    fn three_bubbles_pop_and_reduced_motion_freezes_toys() {
        let mut i = Item::new(Kind::Bubbles, (60., 60.), 0.);
        assert!(i.pop_at((22.5, 47.5)));
        assert!(i.pop_at((52.5, 24.5)));
        assert!(i.pop_at((75.5, 53.5)));
        assert!(!i.pop_at((22.5, 47.5)));
        i.step(
            0.1,
            Bounds {
                x: 0,
                y: 0,
                width: 300,
                height: 300,
            },
            true,
        );
        assert_eq!(i.position, (60., 60.));
    }
    #[test]
    fn toys_wait_for_a_real_throw_and_bubbles_pop_under_the_pointer() {
        let mut ball = Item::new(Kind::Ball, (40., 40.), 0.);
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 400,
            height: 400,
        };
        ball.step(0.05, bounds, false);
        assert_eq!(ball.position, (40., 40.));
        ball.release();
        assert!(!ball.activated);
        ball.grab(1.);
        ball.drag((60., 30.), 1.1);
        ball.release();
        assert!(ball.activated);
        let mut bubbles = Item::new(Kind::Bubbles, (0., 0.), 0.);
        assert!(bubbles.pop_at((75.5, 53.5)));
        assert_eq!(bubbles.bubbles, 3);
        assert!(!bubbles.pop_at((75.5, 53.5)));
        assert!(!bubbles.pop_at((0., 0.)));
    }
}

#[cfg(test)]
mod flight_tests {
    use super::*;
    #[test]
    fn holding_a_plane_does_not_consume_its_flight_and_bubbles_do_not_drift_or_pop_by_themselves() {
        let mut plane = Item::new(Kind::Plane, (80., 80.), 0.);
        plane.grab(1.);
        plane.drag((100., 70.), 1.1);
        plane.release_at(20.);
        assert!(!plane.ready_for_fetch(20.5));
        assert!(plane.ready_for_fetch(21.5));
        let mut bubbles = Item::new(Kind::Bubbles, (120., 120.), 0.);
        for _ in 0..100 {
            bubbles.step(
                0.033,
                Bounds {
                    x: 0,
                    y: 0,
                    width: 600,
                    height: 400,
                },
                false,
            );
        }
        assert_eq!(bubbles.position, (120., 120.));
        assert_eq!(bubbles.bubbles, 7);
    }
}

#[cfg(test)]
mod offering_tests {
    use super::*;
    #[test]
    fn the_first_paw_offer_is_ready_immediately_and_landed_toys_stay_on_the_current_display() {
        let mut wand = Item::new(Kind::Wand, (80., 80.), 100.);
        wand.grab(100.2);
        assert!(100.2 - wand.last_pop >= 3.79);
        assert_eq!(wand.hint(), None);
        let mut plane = Item::new(Kind::Plane, (2000., 2000.), 0.);
        plane.activated = true;
        plane.landed = true;
        plane.step(
            0.05,
            Bounds {
                x: 0,
                y: 0,
                width: 500,
                height: 400,
            },
            false,
        );
        assert!(plane.position.0 + 40. <= 500. && plane.position.1 + 32. <= 400.);
    }
}
