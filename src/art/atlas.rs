use crate::behavior::Activity;
use std::sync::OnceLock;
use tiny_skia::{IntRect, Pixmap};

const CELL_WIDTH: u32 = 192;
const CELL_HEIGHT: u32 = 208;
static SOURCE: OnceLock<Result<Pixmap, String>> = OnceLock::new();

pub fn cell(activity: Activity, phase: f32, facing: f32) -> Result<Pixmap, String> {
    let source = SOURCE
        .get_or_init(|| {
            let image = Pixmap::decode_png(include_bytes!("../../assets/cinnamoroll.png"))
                .map_err(|e| e.to_string())?;
            if image.width() != 1536 || image.height() != 2288 {
                return Err("The bundled pet atlas must be a 1536 × 2288 v2 sheet".into());
            }
            Ok(image)
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let (row, column) = frame_location(activity, phase, facing);
    let rect = IntRect::from_xywh(
        (column * CELL_WIDTH) as i32,
        (row * CELL_HEIGHT) as i32,
        CELL_WIDTH,
        CELL_HEIGHT,
    )
    .ok_or("Invalid sprite coordinates")?;
    source
        .clone_rect(rect)
        .ok_or_else(|| "A required pet sprite is missing".into())
}

fn frame_location(activity: Activity, phase: f32, facing: f32) -> (u32, u32) {
    let frame = |count: u32| ((phase.clamp(0., 0.9999) * count as f32) as u32).min(count - 1);
    match activity {
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
        | Activity::CozyNap => (0, 0),
        Activity::Pant => (0, 2),
        Activity::EdgePeek => {
            if facing > 0. {
                (9, 3)
            } else {
                (10, 5)
            }
        }
        Activity::ChasePause | Activity::Stalk => (if facing < 0. { 2 } else { 1 }, 5),
        Activity::Pounce => (if facing < 0. { 2 } else { 1 }, frame(8)),
        Activity::Stumble => (5, [1, 2, 3, 4, 4, 4][frame(6) as usize]),
        Activity::Dizzy => (5, 4),
        Activity::TypingPeek => (8, 1),
        Activity::Idle | Activity::Listen => (0, frame(6)),
        Activity::Blink
        | Activity::Happy
        | Activity::Petting
        | Activity::Drowsy
        | Activity::Boop => (0, 2),
        Activity::Held => (6, 2),
        Activity::LookAround => match frame(12) {
            2..=4 => (9, 3),
            7..=9 => (10, 5),
            _ => (0, 0),
        },
        Activity::Curious => match frame(16) {
            2..=6 => (9, 3),
            10..=14 => (10, 5),
            _ => (0, 0),
        },
        Activity::Walk | Activity::Run => (if facing < 0. { 2 } else { 1 }, frame(8)),
        Activity::Wave | Activity::Greet => (3, frame(4)),
        Activity::Stretch | Activity::Dance | Activity::Excited => (4, frame(5)),
        Activity::Land => (4, 4),
        Activity::Nap => (5, 3),
        Activity::Snack | Activity::Play => (7, frame(6)),
        Activity::Read | Activity::Watching => (8, frame(6)),
        Activity::Waiting => (6, frame(6)),
        Activity::Shy => (5, [1, 2, 3, 3, 2, 1][frame(6) as usize]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directions_follow_the_validated_atlas_order() {
        assert_eq!(frame_location(Activity::Walk, 0., -1.), (2, 0));
        assert_eq!(frame_location(Activity::Curious, 0., 1.), (0, 0));
        assert_eq!(frame_location(Activity::Curious, 0.25, 1.), (9, 3));
        assert_eq!(frame_location(Activity::Curious, 0.75, 1.), (10, 5));
        for index in 0..16 {
            assert!([(0, 0), (9, 3), (10, 5)].contains(&frame_location(
                Activity::Curious,
                index as f32 / 16.,
                1.
            )));
        }
        assert_eq!(frame_location(Activity::LookAround, 0., 1.), (0, 0));
        assert_eq!(frame_location(Activity::LookAround, 1., 1.), (0, 0));
    }
}
