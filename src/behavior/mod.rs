mod activity;
pub use activity::Activity;
use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdleRule {
    pub activity: Activity,
    pub weight: u32,
    pub cooldown_seconds: f64,
}

pub fn default_rules() -> Vec<IdleRule> {
    serde_json::from_str(include_str!("../../assets/behaviors.json"))
        .expect("bundled behavior definitions must be valid")
}

pub fn validate_rules(rules: &[IdleRule]) -> Result<(), String> {
    if rules.is_empty() || rules.len() > 64 {
        return Err("Provide between 1 and 64 idle rules".into());
    }
    let mut seen = std::collections::HashSet::new();
    for rule in rules {
        if !seen.insert(rule.activity) {
            return Err("Duplicate idle activity".into());
        }
        if rule.weight == 0
            || rule.weight > 10_000
            || !rule.cooldown_seconds.is_finite()
            || !(0.0..=86_400.0).contains(&rule.cooldown_seconds)
            || matches!(
                rule.activity,
                Activity::PlaneReturn
                    | Activity::TeaSip
                    | Activity::PlushHug
                    | Activity::EarWiggle
                    | Activity::BlanketBurrow
                    | Activity::PawPlay
                    | Activity::Sneeze
                    | Activity::GiftOpen
                    | Activity::CupPeek
                    | Activity::RainPlay
                    | Activity::LeafPlay
                    | Activity::Birthday
                    | Activity::Breathe
                    | Activity::Study
                    | Activity::HighFive
                    | Activity::CheekRub
                    | Activity::Tickle
                    | Activity::Nuzzle
                    | Activity::SniffTreat
                    | Activity::TakeTreat
                    | Activity::Fetch
                    | Activity::BubblePop
                    | Activity::CozyNap
                    | Activity::Held
                    | Activity::Land
                    | Activity::Listen
                    | Activity::Greet
                    | Activity::Happy
                    | Activity::Petting
                    | Activity::Boop
                    | Activity::Stalk
                    | Activity::Pounce
                    | Activity::Stumble
                    | Activity::Dizzy
                    | Activity::TypingPeek
                    | Activity::ChasePause
                    | Activity::Pant
            )
        {
            return Err(format!("Invalid idle rule for {:?}", rule.activity));
        }
    }
    Ok(())
}

pub struct Brain {
    pub activity: Activity,
    pub started: f64,
    pub until: f64,
    pub facing: f32,
    pub paused: bool,
    pub wandering: bool,
    pub quiet_seconds: f64,
    rules: Vec<IdleRule>,
    recent: VecDeque<Activity>,
    last_played: std::collections::HashMap<Activity, f64>,
    last_music: Option<f64>,
    last_attention: Option<f64>,
    rng: fastrand::Rng,
}

impl Brain {
    pub fn new(rules: Vec<IdleRule>, seed: u64) -> Self {
        Self {
            activity: Activity::Greet,
            started: 0.,
            until: 4.,
            facing: 1.,
            paused: false,
            wandering: true,
            quiet_seconds: 60.,
            rules,
            recent: VecDeque::with_capacity(4),
            last_played: std::collections::HashMap::new(),
            last_music: None,
            last_attention: None,
            rng: fastrand::Rng::with_seed(seed),
        }
    }

    pub fn start(&mut self, activity: Activity, now: f64) {
        let (min, max) = if activity == Activity::Idle {
            (self.quiet_seconds * 0.75, self.quiet_seconds * 1.5)
        } else {
            activity.duration()
        };
        let duration = if min.is_infinite() {
            min
        } else {
            min + self.rng.f64() * (max - min)
        };
        self.activity = activity;
        self.started = now;
        self.until = now + duration;
        self.last_played.insert(activity, now);
        if activity.moves() {
            self.facing = if self.rng.bool() { 1. } else { -1. };
        }
        if self.recent.len() == 4 {
            self.recent.pop_front();
        }
        self.recent.push_back(activity);
    }

    pub fn music_started(&mut self, now: f64) -> bool {
        if self.paused
            || matches!(
                self.activity,
                Activity::Held
                    | Activity::Happy
                    | Activity::Petting
                    | Activity::Excited
                    | Activity::Boop
            )
            || self.last_music.is_some_and(|last| now - last < 90.)
        {
            return false;
        }
        self.last_music = Some(now);
        self.start(Activity::Listen, now);
        true
    }

    pub fn attention(&mut self, activity: Activity, now: f64) -> bool {
        if self.paused
            || !matches!(
                activity,
                Activity::Curious | Activity::Watching | Activity::Waiting | Activity::Drowsy
            )
            || !matches!(
                self.activity,
                Activity::Idle
                    | Activity::Blink
                    | Activity::LookAround
                    | Activity::Walk
                    | Activity::Curious
                    | Activity::Watching
                    | Activity::Drowsy
                    | Activity::Waiting
            )
            || now - self.started < self.quiet_seconds * 0.75
            || self.last_attention.is_some_and(|last| now - last < 120.)
        {
            return false;
        }
        self.last_attention = Some(now);
        self.start(activity, now);
        true
    }

    pub fn tick(&mut self, now: f64, hour: u32) -> bool {
        if self.paused || now < self.until {
            return false;
        }
        if self.activity != Activity::Idle {
            self.start(Activity::Idle, now);
            return true;
        }
        let eligible: Vec<_> = self
            .rules
            .iter()
            .filter_map(|rule| {
                if (!self.wandering && rule.activity.moves())
                    || self
                        .last_played
                        .get(&rule.activity)
                        .is_some_and(|last| now - last < rule.cooldown_seconds)
                {
                    return None;
                }
                let mut weight = rule.weight;
                if self.recent.contains(&rule.activity) {
                    weight = (weight / 5).max(1);
                }
                if matches!(rule.activity, Activity::Nap | Activity::Drowsy)
                    && !(7..22).contains(&hour)
                {
                    weight *= 4;
                }
                Some((rule.activity, weight))
            })
            .collect();
        let total: u32 = eligible.iter().map(|(_, weight)| weight).sum();
        let mut pick = if total > 0 { self.rng.u32(0..total) } else { 0 };
        let mut next = Activity::Idle;
        for (activity, weight) in eligible {
            if pick < weight {
                next = activity;
                break;
            }
            pick -= weight;
        }
        self.start(next, now);
        true
    }

    pub fn speed(&self) -> f32 {
        if self.paused || !self.wandering {
            return 0.;
        }
        match self.activity {
            Activity::Walk => 26. * self.facing,
            Activity::Run => 76. * self.facing,
            _ => 0.,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_actions_get_a_quiet_idle_gap_before_any_more_random_actions() {
        let mut brain = Brain::new(default_rules(), 29);
        brain.start(Activity::Excited, 0.);
        assert!(brain.tick(5., 12));
        assert_eq!(brain.activity, Activity::Idle);
        assert!((45.0..=90.0).contains(&(brain.until - brain.started)));
        assert!(!brain.tick(49., 12));
        assert!(!brain.attention(Activity::Curious, 20.));
    }

    #[test]
    fn curiosity_respects_direct_interaction_music_sleep_and_cooldowns() {
        let mut brain = Brain::new(default_rules(), 19);
        for activity in [
            Activity::Petting,
            Activity::Happy,
            Activity::Held,
            Activity::Listen,
            Activity::Nap,
            Activity::Excited,
        ] {
            brain.start(activity, 1.);
            assert!(!brain.attention(Activity::Curious, 2.));
        }
        brain.start(Activity::Idle, 10.);
        assert!(!brain.attention(Activity::Curious, 11.));
        assert!(brain.attention(Activity::Curious, 60.));
        assert!(!brain.attention(Activity::Watching, 20.));
        brain.start(Activity::Idle, 200.);
        assert!(brain.attention(Activity::Watching, 250.));
        brain.paused = true;
        assert!(!brain.attention(Activity::Curious, 100.));
    }

    #[test]
    fn music_is_bounded_debounced_and_cannot_interrupt_dragging() {
        let mut brain = Brain::new(default_rules(), 42);
        assert!(brain.music_started(1.));
        assert!((5.0..=10.0).contains(&(brain.until - brain.started)));
        assert!(!brain.music_started(20.));
        brain.start(Activity::Held, 100.);
        assert!(!brain.music_started(101.));
        assert!(!brain.tick(1000., 12));
        brain.start(Activity::Land, 1001.);
        assert!(brain.music_started(1002.));
    }

    #[test]
    fn cooldowns_and_stay_mode_hold_over_many_selections() {
        let mut brain = Brain::new(default_rules(), 7);
        brain.wandering = false;
        let mut previous = std::collections::HashMap::new();
        for step in 0..1000 {
            let now = 100. + f64::from(step) * 40.;
            brain.tick(now, 23);
            assert!(!brain.activity.moves());
            if let Some(rule) = brain.rules.iter().find(|r| r.activity == brain.activity)
                && let Some(last) = previous.insert(brain.activity, now)
            {
                assert!(now - last >= rule.cooldown_seconds);
            }
        }
    }

    #[test]
    fn malformed_rules_are_rejected() {
        assert!(validate_rules(&default_rules()).is_ok());
        assert!(
            validate_rules(&[IdleRule {
                activity: Activity::Held,
                weight: 1,
                cooldown_seconds: 0.
            }])
            .is_err()
        );
        assert!(validate_rules(&[]).is_err());
    }
}
