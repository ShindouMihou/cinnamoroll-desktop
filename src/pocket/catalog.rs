use crate::{
    command::Command,
    companion::{
        preferences::{Accessory, BreakChoice, Season},
        study::View,
        world::Kind,
    },
    settings::Settings,
};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Category {
    Home,
    Play,
    Cozy,
    Study,
    Presets,
    Creative,
    Style,
    Seasons,
    Moments,
    Comfort,
    StudyTools,
}
impl Category {
    pub fn parent(self) -> Self {
        match self {
            Self::Presets | Self::StudyTools => Self::Study,
            Self::Seasons => Self::Moments,
            _ => Self::Home,
        }
    }
}
#[derive(Clone)]
pub(super) enum Action {
    Page(Category),
    Do(Command),
}
pub(super) struct Entry {
    pub label: String,
    pub action: Action,
}

pub(super) fn entries(
    category: Category,
    settings: &Settings,
    paused: bool,
    study: View,
) -> Vec<Entry> {
    use Action::{Do, Page};
    let commands = match category {
        Category::Home => vec![
            ("Play", Page(Category::Play)),
            ("Cozy time", Page(Category::Cozy)),
            ("Study together", Page(Category::Study)),
            ("Creative corner", Page(Category::Creative)),
            ("Dress up", Page(Category::Style)),
            ("Little moments", Page(Category::Moments)),
            ("Desktop comfort", Page(Category::Comfort)),
            (
                if paused { "Resume pet" } else { "Pause pet" },
                Do(Command::Pause),
            ),
        ],
        Category::Moments => vec![
            ("Gentle grooming", Do(Command::Prop(Kind::Brush))),
            ("Dandelion puff", Do(Command::Prop(Kind::Dandelion))),
            ("Snow globe", Do(Command::Prop(Kind::SnowGlobe))),
            ("Paper-boat pond", Do(Command::Prop(Kind::Pond))),
            ("Keepsake box", Do(Command::Prop(Kind::KeepsakeBox))),
            ("Seasonal moments", Page(Category::Seasons)),
        ],
        Category::Comfort => vec![
            ("Undo last move", Do(Command::UndoMove)),
            (
                "Place a keep-clear zone",
                Do(Command::Prop(Kind::ClearZone)),
            ),
            ("Clear all keep-clear zones", Do(Command::ClearZones)),
            (
                if settings.extras.dream_bubbles {
                    "Dream bubbles · on"
                } else {
                    "Dream bubbles · off"
                },
                Do(Command::DreamBubbles),
            ),
            ("Cozy corner here", Do(Command::CornerHere)),
        ],
        Category::StudyTools => vec![
            ("Play break", Do(Command::ChooseBreak(BreakChoice::Play))),
            ("Start open-ended study", Do(Command::StudyStopwatch)),
            (
                if study.paused {
                    "Resume saved session"
                } else {
                    "Pause session"
                },
                Do(if study.paused {
                    Command::StudyResume
                } else {
                    Command::StudyPause
                }),
            ),
            ("End / discard session", Do(Command::StudyDiscard)),
        ],
        Category::Play => vec![
            ("Ball", Do(Command::ToyBall)),
            ("Bubbles", Do(Command::ToyBubbles)),
            ("Toy wand", Do(Command::Prop(Kind::Wand))),
            ("Ribbon", Do(Command::Prop(Kind::Ribbon))),
            ("Paper plane", Do(Command::Prop(Kind::Plane))),
            ("Find the little star", Do(Command::Prop(Kind::HideCups))),
            ("Tap-along dance · K", Do(Command::TapDance)),
        ],
        Category::Cozy => vec![
            ("Offer a treat", Do(Command::OfferTreat)),
            ("Tea time", Do(Command::Prop(Kind::Tea))),
            ("Tuck-in blanket", Do(Command::Prop(Kind::Blanket))),
            ("Plush hug", Do(Command::Prop(Kind::Plush))),
            ("Pillow nap", Do(Command::Prop(Kind::Pillow))),
            ("A little parcel", Do(Command::Prop(Kind::Parcel))),
            ("Nap / wake", Do(Command::Nap)),
        ],
        Category::Study => vec![
            (
                if !matches!(
                    study.phase,
                    crate::companion::study::Phase::Focus | crate::companion::study::Phase::Break
                ) {
                    "Start focus"
                } else if study.paused {
                    "Resume session"
                } else {
                    "Pause session"
                },
                Do(
                    if !matches!(
                        study.phase,
                        crate::companion::study::Phase::Focus
                            | crate::companion::study::Phase::Break
                    ) {
                        Command::StudyStart
                    } else {
                        Command::StudyPause
                    },
                ),
            ),
            ("Complete my task", Do(Command::TaskDone)),
            ("Choose session & goal", Do(Command::SessionPlan)),
            ("Stopwatch & saved session", Page(Category::StudyTools)),
            (
                "Stretch together",
                Do(Command::ChooseBreak(BreakChoice::Stretch)),
            ),
            ("Water break", Do(Command::ChooseBreak(BreakChoice::Water))),
            (
                "Breathe together",
                Do(Command::ChooseBreak(BreakChoice::Breathe)),
            ),
        ],
        Category::Creative => vec![
            ("Palette play", Do(Command::Prop(Kind::Palette))),
            ("Choose palette colors", Do(Command::CreativeTools)),
            ("Export a sticker", Do(Command::Sticker)),
        ],
        Category::Presets => Vec::new(),
        Category::Style => vec![
            ("No accessory", Do(Command::Accessory(Accessory::None))),
            ("Little bow", Do(Command::Accessory(Accessory::Bow))),
            ("Cozy scarf", Do(Command::Accessory(Accessory::Scarf))),
            ("Tiny hat", Do(Command::Accessory(Accessory::Hat))),
            ("Little bag", Do(Command::Accessory(Accessory::Bag))),
        ],
        Category::Seasons => vec![
            ("Umbrella peek", Do(Command::Season(Season::Rain))),
            ("Autumn leaf play", Do(Command::Season(Season::Autumn))),
            ("Birthday cupcake", Do(Command::Season(Season::Birthday))),
            ("No seasonal routine", Do(Command::Season(Season::None))),
        ],
    };
    let mut entries: Vec<Entry> = commands
        .into_iter()
        .map(|(label, action)| Entry {
            label: label.into(),
            action,
        })
        .collect();
    entries.retain(|entry| match entry.action {
        Action::Do(Command::StudyResume | Command::StudyPause | Command::StudyDiscard)
            if category == Category::StudyTools =>
        {
            matches!(
                study.phase,
                crate::companion::study::Phase::Focus | crate::companion::study::Phase::Break
            )
        }
        Action::Do(Command::ClearZones) => !settings.extras.comfort.zones.is_empty(),
        _ => true,
    });
    if category == Category::Study {
        entries.insert(
            1,
            Entry {
                label: "Subject presets".into(),
                action: Action::Page(Category::Presets),
            },
        );
    }
    if category == Category::Presets {
        entries = settings
            .extras
            .presets
            .iter()
            .enumerate()
            .map(|(i, preset)| Entry {
                label: format!("{} · {} min", preset.name, preset.focus),
                action: Action::Do(Command::UsePreset(i)),
            })
            .collect();
    }
    entries
}
