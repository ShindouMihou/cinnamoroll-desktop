use crate::behavior::Activity;

#[derive(Clone, Copy, PartialEq)]
enum Zone {
    Cheek,
    Tummy,
    Paw,
    Nose,
    Ear,
}

#[derive(Default)]
pub struct Touch {
    zone: Option<Zone>,
    entered: f64,
    previous: (f32, f32),
    anchor: (f32, f32),
    direction: i8,
    turns: u8,
    last_motion: f64,
    cooldown: f64,
    offered: bool,
}

pub fn is_ear((x, y): (f32, f32)) -> bool {
    ((x - 65.) / 34.).powi(2) + ((y - 114.) / 16.).powi(2) < 1.
        || ((x - 255.) / 34.).powi(2) + ((y - 114.) / 16.).powi(2) < 1.
}

fn zone((x, y): (f32, f32)) -> Option<Zone> {
    if is_ear((x, y)) {
        Some(Zone::Ear)
    } else if ((x - 160.) / 15.).powi(2) + ((y - 117.) / 9.).powi(2) < 1. {
        Some(Zone::Nose)
    } else if (105. ..=132.).contains(&x) || (188. ..=215.).contains(&x) {
        if (104. ..=130.).contains(&y) {
            Some(Zone::Cheek)
        } else if (139. ..=158.).contains(&y) {
            Some(Zone::Paw)
        } else {
            None
        }
    } else if (140. ..=182.).contains(&x) && (158. ..=184.).contains(&y) {
        Some(Zone::Tummy)
    } else {
        None
    }
}

impl Touch {
    pub fn handles(point: (f32, f32)) -> bool {
        zone(point).is_some()
    }
    pub fn reset(&mut self) {
        self.zone = None;
        self.offered = false;
        self.turns = 0;
    }
    pub fn sample(&mut self, point: (f32, f32), now: f64) -> Option<Activity> {
        let next = zone(point);
        if next != self.zone {
            self.zone = next;
            self.entered = now;
            self.anchor = point;
            self.previous = point;
            self.direction = 0;
            self.turns = 0;
            self.offered = false;
            self.last_motion = now;
        }
        let current = next?;
        let moved = (point.0 - self.previous.0).hypot(point.1 - self.previous.1);
        self.previous = point;
        if now < self.cooldown {
            return None;
        }
        let reaction = match current {
            Zone::Paw | Zone::Nose | Zone::Ear => {
                if moved > 3. {
                    self.entered = now;
                }
                if !self.offered && now - self.entered >= 1.4 {
                    self.offered = true;
                    Some(if current == Zone::Ear {
                        Activity::EarWiggle
                    } else if current == Zone::Paw {
                        Activity::HighFive
                    } else {
                        Activity::Nuzzle
                    })
                } else {
                    None
                }
            }
            Zone::Cheek | Zone::Tummy => {
                if now - self.last_motion > 1.3 {
                    self.turns = 0;
                    self.direction = 0;
                    self.anchor = point;
                }
                let dx = point.0 - self.anchor.0;
                let dy = point.1 - self.anchor.1;
                let delta = if dx.abs() >= dy.abs() { dx } else { dy };
                if delta.abs() >= 7. {
                    self.last_motion = now;
                    let direction = if delta > 0. { 1 } else { -1 };
                    if self.direction != 0 && direction != self.direction {
                        self.turns += 1;
                    }
                    self.direction = direction;
                    self.anchor = point;
                }
                if self.turns >= 2 {
                    self.turns = 0;
                    Some(if current == Zone::Cheek {
                        Activity::CheekRub
                    } else {
                        Activity::Tickle
                    })
                } else {
                    None
                }
            }
        };
        if reaction.is_some() {
            self.cooldown = now + 5.;
        }
        reaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cheek_and_tummy_rubs_need_reversals_and_have_a_shared_cooldown() {
        for (y, activity) in [(120., Activity::CheekRub), (170., Activity::Tickle)] {
            let x = if y == 120. { 108. } else { 145. };
            let mut t = Touch::default();
            for (time, point) in [(0., (x, y)), (0.1, (x + 12., y)), (0.2, (x, y))] {
                assert_eq!(t.sample(point, time), None);
            }
            assert_eq!(t.sample((x + 12., y), 0.3), Some(activity));
            assert_eq!(t.sample((x, y), 0.4), None);
        }
    }
    #[test]
    fn a_stationary_offer_triggers_once_until_the_pointer_leaves() {
        for (point, activity) in [
            ((120., 154.), Activity::HighFive),
            ((160., 117.), Activity::Nuzzle),
        ] {
            let mut t = Touch::default();
            assert_eq!(t.sample(point, 0.), None);
            assert_eq!(t.sample(point, 1.5), Some(activity));
            assert_eq!(t.sample(point, 10.), None);
            t.sample((0., 0.), 11.);
            t.sample(point, 12.);
            assert_eq!(t.sample(point, 13.5), Some(activity));
        }
    }
}
