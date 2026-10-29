use super::catalog::{Action, Category, Entry};
use crate::{
    art::{self, Decoration},
    behavior::Activity,
    command::Command,
    companion::{
        preferences::{BreakChoice, Season},
        world::Kind,
    },
};
use tiny_skia::Pixmap;

pub(super) fn entry(entry: &Entry, scale: f32) -> Result<Pixmap, String> {
    use Activity::*;
    match &entry.action {
        Action::Page(category) => character(
            match category {
                Category::Home => Wave,
                Category::Play => PawPlay,
                Category::Cozy => TeaSip,
                Category::Study | Category::Presets => Read,
                Category::Creative => Curious,
                Category::Style => EarWiggle,
                Category::Seasons => RainPlay,
            },
            scale,
        ),
        Action::Do(command) => match command {
            Command::ToyBall => decoration(Decoration::Ball, scale),
            Command::ToyBubbles => decoration(Decoration::Bubbles(7), scale),
            Command::OfferTreat => decoration(Decoration::Treat, scale),
            Command::Prop(kind) => decoration(
                Decoration::Prop(*kind, 0, [0xb0daf5, 0xf8cddc, 0xdbd1f6]),
                scale,
            ),
            Command::Accessory(accessory) => {
                let mut image = character(Idle, scale)?;
                art::accessories::apply(&mut image, *accessory, Idle, 0, 1.)?;
                Ok(image)
            }
            Command::Season(season) => character(
                match season {
                    Season::Rain => RainPlay,
                    Season::Autumn => LeafPlay,
                    Season::Birthday => Birthday,
                    Season::None => Idle,
                },
                scale,
            ),
            Command::ChooseBreak(choice) => character(
                match choice {
                    BreakChoice::Stretch => Stretch,
                    BreakChoice::Water => TeaSip,
                    BreakChoice::Breathe => Listen,
                    BreakChoice::Play => PawPlay,
                },
                scale,
            ),
            Command::TapDance => character(Dance, scale),
            Command::Nap => character(CozyNap, scale),
            Command::TaskDone => character(Happy, scale),
            Command::Sticker => character(Wave, scale),
            Command::CreativeTools => decoration(
                Decoration::Prop(Kind::Palette, 0, [0xb0daf5, 0xf8cddc, 0xdbd1f6]),
                scale,
            ),
            Command::Pause => character(Drowsy, scale),
            _ => character(Read, scale),
        },
    }
}

pub(super) fn character(activity: Activity, scale: f32) -> Result<Pixmap, String> {
    art::render(activity, 0., 44. * scale / art::WIDTH as f32)
}

fn decoration(kind: Decoration, scale: f32) -> Result<Pixmap, String> {
    art::decoration(
        kind,
        ((32. * scale).round() as u32, (28. * scale).round() as u32),
        0.,
    )
}

pub(super) fn selected(entry: &Entry, settings: &crate::settings::Settings, paused: bool) -> bool {
    match &entry.action {
        Action::Do(Command::Accessory(accessory)) => *accessory == settings.extras.accessory,
        Action::Do(Command::Season(season)) => *season == settings.extras.season,
        Action::Do(Command::Pause) => paused,
        _ => false,
    }
}
