const WATCH_SECONDS: f64 = 1.6;
const SWAP_SECONDS: f64 = 0.75;
const MOVE_SECONDS: f64 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CupPhase {
    Watch,
    Shuffle,
    Guess,
    Reveal(u8),
    Again(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CupVisual {
    pub order: [u8; 3],
    pub target: u8,
    pub chosen: u8,
    pub lift: u8,
    pub swap: Option<(u8, u8, u8)>,
}
impl CupVisual {
    pub fn poses(self) -> [(f32, f32); 3] {
        let mut positions = [(28., 42.), (80., 42.), (132., 42.)];
        if let Some((a, b, progress)) = self.swap {
            let t = f32::from(progress) / 24.;
            let eased = t * t * (3. - 2. * t);
            let ax = positions[a as usize].0;
            let bx = positions[b as usize].0;
            let arc = (t * std::f32::consts::PI).sin() * 6.;
            positions[a as usize] = (ax + (bx - ax) * eased, 42. - arc);
            positions[b as usize] = (bx + (ax - bx) * eased, 42. + arc);
        }
        positions
    }
}

pub struct CupRound {
    pub phase: CupPhase,
    pub target: u8,
    pub started: f64,
    pub order: [u8; 3],
    pub swaps: [(u8, u8); 3],
    swap_index: usize,
}
impl CupRound {
    pub fn new(now: f64, target: u8) -> Self {
        let target = target.min(2);
        let pairs = [(0, 1), (0, 2), (1, 2)];
        let mut swaps = [(target, (target + 1 + fastrand::u8(..2)) % 3); 3];
        for index in 1..3 {
            let previous = swaps[index - 1];
            let candidates: Vec<_> = pairs
                .into_iter()
                .filter(|pair| {
                    !((pair.0 == previous.0 && pair.1 == previous.1)
                        || (pair.0 == previous.1 && pair.1 == previous.0))
                })
                .collect();
            swaps[index] = candidates[fastrand::usize(..candidates.len())];
        }
        Self {
            phase: CupPhase::Watch,
            target,
            started: now,
            order: [0, 1, 2],
            swaps,
            swap_index: 0,
        }
    }
    pub fn hit(&self, point: (f32, f32), size: (u32, u32)) -> Option<u8> {
        let x = point.0 * 160. / size.0 as f32;
        let y = point.1 * 112. / size.1 as f32;
        if !(24. ..102.).contains(&y) {
            return None;
        }
        [28., 80., 132.]
            .iter()
            .position(|center| (x - center).abs() < 22.)
            .map(|index| index as u8)
    }
    pub fn winning_slot(&self) -> u8 {
        self.order
            .iter()
            .position(|cup| *cup == self.target)
            .unwrap_or(0) as u8
    }
    pub fn click(&mut self, index: u8, now: f64) -> bool {
        match self.phase {
            CupPhase::Guess if index < 3 => {
                self.phase = CupPhase::Reveal(index);
                self.started = now;
                true
            }
            CupPhase::Again(_) => {
                *self = Self::new(now, fastrand::u8(..3));
                false
            }
            _ => false,
        }
    }
    pub fn advance(&mut self, now: f64) -> Option<bool> {
        loop {
            match self.phase {
                CupPhase::Watch if now - self.started >= WATCH_SECONDS => {
                    self.phase = CupPhase::Shuffle;
                    self.started += WATCH_SECONDS;
                }
                CupPhase::Shuffle if now - self.started >= SWAP_SECONDS => {
                    let (a, b) = self.swaps[self.swap_index];
                    self.order.swap(a as usize, b as usize);
                    self.swap_index += 1;
                    self.started += SWAP_SECONDS;
                    if self.swap_index == self.swaps.len() {
                        self.phase = CupPhase::Guess;
                    }
                }
                CupPhase::Reveal(index) if now - self.started >= 4. => {
                    self.phase = CupPhase::Again(index);
                    return Some(index == self.winning_slot());
                }
                _ => return None,
            }
        }
    }
    pub fn animating(&self, now: f64) -> bool {
        match self.phase {
            CupPhase::Watch => now - self.started > 1.2,
            CupPhase::Shuffle => true,
            CupPhase::Reveal(_) => now - self.started < 0.25,
            _ => false,
        }
    }
    pub fn visual(&self, now: f64, reduced: bool) -> CupVisual {
        let (chosen, lift) = match self.phase {
            CupPhase::Watch => (
                3,
                if reduced {
                    10.
                } else {
                    (1. - ((now - self.started - 1.2) / 0.4).clamp(0., 1.)) * 10.
                },
            ),
            CupPhase::Reveal(index) => (
                index,
                if reduced {
                    10.
                } else {
                    ((now - self.started) / 0.25).clamp(0., 1.) * 10.
                },
            ),
            CupPhase::Again(index) => (index, 10.),
            _ => (3, 0.),
        };
        let swap = if self.phase == CupPhase::Shuffle && !reduced {
            let (a, b) = self.swaps[self.swap_index];
            Some((
                a,
                b,
                (((now - self.started) / MOVE_SECONDS).clamp(0., 1.) * 24.).round() as u8,
            ))
        } else {
            None
        };
        CupVisual {
            order: self.order,
            target: self.winning_slot(),
            chosen,
            lift: lift.round() as u8,
            swap,
        }
    }
    pub fn hint(&self) -> &'static str {
        match self.phase {
            CupPhase::Watch => "Watch the little star",
            CupPhase::Shuffle => "Keep an eye on the cups",
            CupPhase::Guess => "Which cup? Tap to choose",
            CupPhase::Reveal(index) if index == self.winning_slot() => "You found the little star!",
            CupPhase::Reveal(_) => "Here was the little star",
            CupPhase::Again(_) => "Tap a cup to play again",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_star_follows_its_cup_through_three_visible_swaps_before_guessing() {
        let mut game = CupRound::new(0., 0);
        game.swaps = [(0, 1), (1, 2), (0, 1)];
        assert!(!game.click(0, 0.5));
        game.advance(1.6);
        assert_eq!(game.phase, CupPhase::Shuffle);
        assert!(!game.click(0, 2.));
        let mid = game.visual(1.9, false);
        assert!(mid.poses()[0].0 > 28. && mid.poses()[0].0 < 80.);
        game.advance(3.9);
        assert_eq!(game.phase, CupPhase::Guess);
        assert_eq!(game.order, [2, 1, 0]);
        assert_eq!(game.winning_slot(), 2);
        assert!(game.click(2, 4.));
        assert_eq!(game.advance(7.), None);
        assert_eq!(game.advance(8.), Some(true));
    }
    #[test]
    fn stalled_frames_commit_each_swap_once_and_wrong_guesses_reveal_the_actual_star() {
        let mut game = CupRound::new(0., 1);
        game.swaps = [(0, 1), (0, 2), (1, 2)];
        game.advance(30.);
        let order = game.order;
        assert_eq!(game.phase, CupPhase::Guess);
        game.advance(31.);
        assert_eq!(game.order, order);
        let wrong = (game.winning_slot() + 1) % 3;
        assert!(game.click(wrong, 32.));
        assert_eq!(game.advance(36.), Some(false));
        assert_eq!(game.visual(36., true).swap, None);
        assert_eq!(game.visual(36., true).target, game.winning_slot());
    }
    #[test]
    fn empty_space_between_cups_is_not_a_guess() {
        let game = CupRound::new(0., 1);
        assert_eq!(game.hit((31.5, 60.), (180, 96)), Some(0));
        assert_eq!(game.hit((60., 60.), (180, 96)), None);
        assert_eq!(game.hit((90., 4.), (180, 96)), None);
    }
}
