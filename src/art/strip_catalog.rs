use crate::behavior::Activity;

pub(super) struct Strip {
    pub activity: Activity,
    pub name: &'static str,
    pub bytes: &'static [u8],
}

macro_rules! strip {
    ($activity:ident, $name:literal) => {
        Strip {
            activity: Activity::$activity,
            name: $name,
            bytes: include_bytes!(concat!("../../assets/strips/runtime/", $name, ".png")),
        }
    };
}

pub(super) const STRIPS: &[Strip] = &[
    strip!(PlaneReturn, "plane_return"),
    strip!(TeaSip, "tea_sip"),
    strip!(PlushHug, "plush_hug"),
    strip!(EarWiggle, "ear_wiggle"),
    strip!(BlanketBurrow, "blanket_burrow"),
    strip!(PawPlay, "paw_play"),
    strip!(Sneeze, "sneeze"),
    strip!(GiftOpen, "gift_open"),
    strip!(CupPeek, "cup_peek"),
    strip!(RainPlay, "rain_play"),
    strip!(LeafPlay, "leaf_play"),
    strip!(Birthday, "birthday"),
    strip!(HighFive, "high_five"),
    strip!(CheekRub, "cheek_rub"),
    strip!(Tickle, "tickle"),
    strip!(Nuzzle, "nuzzle"),
    strip!(SniffTreat, "sniff_treat"),
    strip!(TakeTreat, "take_treat"),
    strip!(Fetch, "fetch"),
    strip!(BubblePop, "bubble_pop"),
    strip!(CozyNap, "cozy_nap"),
    strip!(Idle, "idle"),
    strip!(Held, "hanging"),
    strip!(Walk, "walk"),
    strip!(Snack, "snack"),
    strip!(Nap, "nap"),
    strip!(Listen, "listen"),
    strip!(Petting, "petting"),
    strip!(Dance, "dance"),
    strip!(Wave, "wave"),
    strip!(Read, "read"),
    strip!(Curious, "curious"),
    strip!(Happy, "happy"),
    strip!(Shy, "shy"),
    strip!(Excited, "excited"),
    strip!(Drowsy, "drowsy"),
    strip!(Boop, "boop"),
    strip!(Play, "play"),
    strip!(Stretch, "stretch"),
    strip!(Pant, "pant"),
    strip!(Stalk, "stalk"),
    strip!(Pounce, "pounce"),
    strip!(Stumble, "stumble"),
    strip!(Dizzy, "dizzy"),
];

pub(super) fn find(activity: Activity) -> Option<&'static Strip> {
    let source = match activity {
        Activity::Blink | Activity::Watching | Activity::Waiting | Activity::Land => Activity::Idle,
        Activity::Study => Activity::Read,
        Activity::Breathe => Activity::Listen,
        Activity::LookAround => Activity::Curious,
        Activity::Greet => Activity::Wave,
        Activity::TypingPeek => Activity::Curious,
        Activity::ChasePause => Activity::Stalk,
        Activity::Dizzy => Activity::Stumble,
        _ => activity,
    };
    STRIPS.iter().find(|strip| strip.activity == source)
}

pub(super) fn directional(activity: Activity) -> bool {
    matches!(
        activity,
        Activity::Walk
            | Activity::PawPlay
            | Activity::ChasePause
            | Activity::Stalk
            | Activity::Pounce
            | Activity::Stumble
            | Activity::Dizzy
    )
}

pub(super) fn authored_origin(activity: Activity) -> bool {
    if matches!(activity, Activity::BlanketBurrow | Activity::CupPeek) {
        return true;
    }
    matches!(
        activity,
        Activity::Excited | Activity::Pounce | Activity::Held
    )
}
