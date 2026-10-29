use crate::behavior::Activity;

#[derive(Default)]
pub struct Interaction {
    last_click: Option<f64>,
    click_count: u8,
    reaction_count: u32,
    stroke: Stroke,
    last_pet: Option<f64>,
    pointer: Option<(f32, f32)>,
    active_since: Option<f64>,
    last_watching: Option<f64>,
    away_stage: u8,
    last_pointer_motion: Option<f64>,
}

impl Interaction {
    pub fn click(&mut self, now: f64) -> Activity {
        self.click_count = if self.last_click.is_some_and(|last| now - last < 1.5) {
            self.click_count.saturating_add(1)
        } else {
            1
        };
        self.last_click = Some(now);
        if self.click_count >= 3 {
            return Activity::Excited;
        }
        if self.click_count == 2 {
            return Activity::Petting;
        }
        let choices = [Activity::Happy, Activity::Wave, Activity::Shy];
        let reaction = choices[self.reaction_count as usize % choices.len()];
        self.reaction_count = self.reaction_count.wrapping_add(1);
        reaction
    }

    pub fn pet(&mut self, point: (f32, f32), now: f64) -> bool {
        if !(105.0..=215.0).contains(&point.0) || !(78.0..=150.0).contains(&point.1) {
            self.stroke = Stroke::default();
            return false;
        }
        if self.last_pet.is_some_and(|last| now - last < 4.) {
            return false;
        }
        if self.stroke.update(point.0, now) {
            self.last_pet = Some(now);
            return true;
        }
        false
    }

    pub fn reset_stroke(&mut self) {
        self.stroke = Stroke::default();
    }

    pub fn notice_pointer(&mut self, pointer: (f32, f32), center: (f32, f32), now: f64) -> bool {
        let previous = self.pointer.replace(pointer);
        if previous.is_some_and(|old| (old.0 - pointer.0).hypot(old.1 - pointer.1) >= 3.) {
            self.last_pointer_motion = Some(now);
        }
        let distance = (pointer.0 - center.0).hypot(pointer.1 - center.1);
        (45.0..=200.0).contains(&distance)
            && previous.is_some_and(|old| (old.0 - pointer.0).hypot(old.1 - pointer.1) >= 16.)
    }
    pub fn mouse_recent(&self, now: f64) -> bool {
        self.last_pointer_motion
            .is_some_and(|last| now - last < 1.5)
    }

    pub fn activity(&mut self, idle: Option<f64>, now: f64) -> Option<Activity> {
        let idle = idle?;
        if idle >= 120. {
            self.active_since = None;
            if self.away_stage < 2 {
                self.away_stage = 2;
                return Some(Activity::Waiting);
            }
        } else if idle >= 60. {
            self.active_since = None;
            if self.away_stage < 1 {
                self.away_stage = 1;
                return Some(Activity::Drowsy);
            }
        } else if idle < 10. {
            self.away_stage = 0;
            let started = *self.active_since.get_or_insert(now);
            if now - started >= 45. && self.last_watching.is_none_or(|last| now - last >= 180.) {
                self.last_watching = Some(now);
                return Some(Activity::Watching);
            }
        } else {
            self.active_since = None;
        }
        None
    }
}

#[derive(Default)]
struct Stroke {
    anchor: Option<f32>,
    direction: i8,
    turns: u8,
    last_motion: f64,
}

impl Stroke {
    fn update(&mut self, x: f32, now: f64) -> bool {
        if now - self.last_motion > 1.5 {
            *self = Self::default();
        }
        self.last_motion = now;
        let anchor = self.anchor.get_or_insert(x);
        let delta = x - *anchor;
        if delta.abs() < 12. {
            return false;
        }
        let direction = if delta > 0. { 1 } else { -1 };
        if self.direction != 0 && direction != self.direction {
            self.turns += 1;
        }
        self.direction = direction;
        self.anchor = Some(x);
        if self.turns >= 2 {
            *self = Self::default();
            return true;
        }
        false
    }
}

pub fn is_nose(point: (f32, f32)) -> bool {
    ((point.0 - 160.) / 16.).powi(2) + ((point.1 - 117.) / 10.).powi(2) <= 1.
}

pub fn look_direction(origin: (f32, f32), pointer: (f32, f32)) -> Option<usize> {
    let (x, y) = (pointer.0 - origin.0, pointer.1 - origin.1);
    if x.hypot(y) < 8. {
        return None;
    }
    Some(
        ((x.atan2(-y).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * 16.).round()
            as usize)
            % 16,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nose_boops_do_not_match_the_forehead_or_body() {
        assert!(is_nose((160., 117.)));
        assert!(!is_nose((160., 100.)));
        assert!(!is_nose((160., 180.)));
        assert!(!is_nose((120., 117.)));
    }
    #[test]
    fn clicks_escalate_without_permanently_escalating_single_clicks() {
        let mut input = Interaction::default();
        assert_eq!(input.click(0.), Activity::Happy);
        assert_eq!(input.click(0.4), Activity::Petting);
        assert_eq!(input.click(0.8), Activity::Excited);
        assert_eq!(input.click(5.), Activity::Wave);
        assert_eq!(input.click(10.), Activity::Shy);
    }
    #[test]
    fn petting_requires_back_and_forth_motion_over_the_head_and_has_a_cooldown() {
        let mut input = Interaction::default();
        assert!(!input.pet((140., 100.), 0.));
        assert!(!input.pet((170., 100.), 0.2));
        assert!(!input.pet((140., 100.), 0.4));
        assert!(input.pet((170., 100.), 0.6));
        assert!(!input.pet((140., 100.), 0.8));
        assert!(!input.pet((170., 180.), 6.));
        assert!(!input.pet((170., 100.), 7.));
        assert!(!input.pet((171., 100.), 7.1));
    }
    #[test]
    fn attention_is_spaced_and_idle_stages_do_not_repeat() {
        let mut input = Interaction::default();
        assert_eq!(input.activity(Some(0.), 0.), None);
        assert_eq!(input.activity(Some(1.), 46.), Some(Activity::Watching));
        assert_eq!(input.activity(Some(1.), 47.), None);
        assert_eq!(input.activity(Some(65.), 120.), Some(Activity::Drowsy));
        assert_eq!(input.activity(Some(125.), 180.), Some(Activity::Waiting));
        assert_eq!(input.activity(Some(135.), 190.), None);
        assert_eq!(input.activity(None, 100.), None);
    }
    #[test]
    fn gaze_matches_all_four_cardinals_in_the_supplied_sheet() {
        assert_eq!(look_direction((0., 0.), (0., -100.)), Some(0));
        assert_eq!(look_direction((0., 0.), (100., 0.)), Some(4));
        assert_eq!(look_direction((0., 0.), (0., 100.)), Some(8));
        assert_eq!(look_direction((0., 0.), (-100., 0.)), Some(12));
        assert_eq!(look_direction((0., 0.), (0., 0.)), None);
    }
}
