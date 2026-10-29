use super::{decorations::oval, shapes::Painter};
use crate::{behavior::Activity, companion::preferences::Accessory};
use tiny_skia::{PathBuilder, Pixmap};

pub(crate) fn apply(
    image: &mut Pixmap,
    accessory: Accessory,
    activity: Activity,
    frame: usize,
    facing: f32,
) -> Result<(), String> {
    if accessory == Accessory::None
        || matches!(
            activity,
            Activity::Held
                | Activity::Pounce
                | Activity::Stumble
                | Activity::Stalk
                | Activity::Nap
                | Activity::CozyNap
                | Activity::BlanketBurrow
                | Activity::EdgePeek
                | Activity::RainPlay
        )
    {
        return Ok(());
    }
    if matches!(accessory, Accessory::Scarf | Accessory::Bag)
        && matches!(
            activity,
            Activity::TeaSip
                | Activity::PlushHug
                | Activity::Snack
                | Activity::TakeTreat
                | Activity::Fetch
                | Activity::PlaneReturn
                | Activity::GiftOpen
                | Activity::CupPeek
                | Activity::LeafPlay
                | Activity::Birthday
                | Activity::Read
                | Activity::Study
        )
    {
        return Ok(());
    }
    let Some(pose) = super::attachment::pose(activity, frame, facing)? else {
        return Ok(());
    };
    let [x, y, angle, scale] = match accessory {
        Accessory::Hat => pose.hat,
        Accessory::Bow => pose.bow,
        Accessory::Scarf => pose.scarf,
        Accessory::Bag => pose.bag,
        Accessory::None => return Ok(()),
    };
    let mirror = if facing < 0. && super::strip_catalog::directional(activity) {
        -1.
    } else {
        1.
    };
    let transform = super::viewport_transform((image.width(), image.height()))
        .pre_translate(x, y)
        .pre_rotate(angle)
        .pre_scale(scale * mirror, scale);
    let mut p = Painter {
        pixmap: image,
        transform,
    };
    let blue = [176, 218, 245, 255];
    let pink = [248, 205, 220, 255];
    let brown = [174, 112, 61, 255];
    match accessory {
        Accessory::Bow => {
            oval(&mut p, -9., 0., 10., 7., pink);
            oval(&mut p, 9., 0., 10., 7., pink);
            oval(&mut p, 0., 1., 4., 4., blue);
        }
        Accessory::Scarf => {
            p.line(&[(-22., 0.), (22., 0.)], brown, 12.);
            p.line(&[(-22., 0.), (22., 0.)], blue, 8.);
            p.line(&[(16., 3.), (23., 23.)], brown, 11.);
            p.line(&[(16., 3.), (23., 23.)], blue, 7.);
        }
        Accessory::Hat => {
            oval(&mut p, 0., 0., 26., 6., blue);
            oval(&mut p, 0., -8., 19., 10., blue);
            p.line(&[(-18., -5.), (18., -5.)], pink, 4.);
        }
        Accessory::Bag => {
            p.curve((-16., -25.), [-5., -36., 4., -27., 4., -9.], brown, 2.);
            let mut b = PathBuilder::new();
            b.move_to(-14., -11.);
            b.line_to(15., -11.);
            b.line_to(12., 13.);
            b.line_to(-11., 13.);
            b.close();
            if let Some(path) = b.finish() {
                p.path(&path, Some(pink), Some((brown, 2.)));
            }
            p.heart(0., 0., 5., 245);
        }
        Accessory::None => {}
    }
    Ok(())
}

pub(super) fn gallery(folder: &std::path::Path) -> Result<(), String> {
    use tiny_skia::{PixmapPaint, Transform};
    for activity in [
        Activity::Idle,
        Activity::Walk,
        Activity::Dance,
        Activity::EarWiggle,
        Activity::Petting,
        Activity::Curious,
        Activity::Excited,
        Activity::Drowsy,
        Activity::Wave,
        Activity::HighFive,
        Activity::Stretch,
        Activity::Pant,
        Activity::Dizzy,
    ] {
        let frames = super::strips::frames(activity, (320, 224), 1.)?
            .ok_or("Missing outfit preview strip")?;
        for accessory in [
            Accessory::Bow,
            Accessory::Hat,
            Accessory::Scarf,
            Accessory::Bag,
        ] {
            let mut sheet = Pixmap::new(1280, 448).ok_or("Cannot allocate outfit preview")?;
            for (index, frame) in frames.iter().enumerate() {
                let mut image = frame.clone();
                apply(&mut image, accessory, activity, index, 1.)?;
                sheet.draw_pixmap(
                    (index % 4 * 320) as i32,
                    (index / 4 * 224) as i32,
                    image.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
            sheet
                .save_png(folder.join(format!("{accessory:?}-{activity:?}-strip.png")))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_outfits_follow_closed_eye_frames_and_mirror_at_small_and_large_sizes() {
        for activity in [
            Activity::Idle,
            Activity::Walk,
            Activity::EarWiggle,
            Activity::Dizzy,
        ] {
            for accessory in [
                Accessory::Hat,
                Accessory::Bow,
                Accessory::Scarf,
                Accessory::Bag,
            ] {
                for size in [(128, 89), (320, 224)] {
                    for facing in [-1., 1.] {
                        let plain = super::super::Animation::new(activity, facing, size).unwrap();
                        let dressed = super::super::Animation::new(activity, facing, size)
                            .unwrap()
                            .with_accessory(accessory)
                            .unwrap();
                        assert_eq!(dressed.accessory(), accessory);
                        for elapsed in [0., 0.5, 0.875] {
                            assert!(
                                plain.frame(elapsed, false).data()
                                    != dressed.frame(elapsed, false).data(),
                                "Outfit missing from {activity:?} {accessory:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
