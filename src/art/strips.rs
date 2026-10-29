use crate::behavior::Activity;
use tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint};

pub(super) fn source_index(activity: Activity, index: usize) -> usize {
    match activity {
        Activity::ChasePause | Activity::Land => 0,
        Activity::Dizzy => [5, 6, 7, 6, 5, 6, 7, 6][index],
        Activity::Blink => [0, 0, 3, 4, 5, 6, 7, 7][index],
        _ => index,
    }
}

pub(super) fn frames(
    activity: Activity,
    size: (u32, u32),
    facing: f32,
) -> Result<Option<Vec<Pixmap>>, String> {
    let Some(strip) = super::strip_catalog::find(activity) else {
        return Ok(None);
    };
    let bytes = strip.bytes;
    let sheet = Pixmap::decode_png(bytes).map_err(|error| error.to_string())?;
    if sheet.width() != 1280 || sheet.height() != 448 {
        return Err("Dedicated strips must have eight 320 by 224 frames".into());
    }
    let mut frames = Vec::with_capacity(8);
    for index in 0..8 {
        let source_index = source_index(activity, index as usize) as i32;
        let cell = sheet
            .clone_rect(
                IntRect::from_xywh(source_index % 4 * 320, source_index / 4 * 224, 320, 224)
                    .ok_or("Invalid strip frame")?,
            )
            .ok_or("Missing strip frame")?;
        let mut frame = Pixmap::new(size.0, size.1).ok_or("Cannot allocate strip frame")?;
        let mut transform = super::viewport_transform(size);
        if activity == Activity::Land {
            transform = transform
                .pre_translate(160., 197.)
                .pre_scale(1., 0.94 + index as f32 / 7. * 0.06)
                .pre_translate(-160., -197.);
        }
        if facing < 0. && super::strip_catalog::directional(activity) {
            transform = transform.pre_translate(320., 0.).pre_scale(-1., 1.);
        }
        frame.draw_pixmap(
            0,
            0,
            cell.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..PixmapPaint::default()
            },
            transform,
            None,
        );
        if matches!(activity, Activity::TypingPeek | Activity::Boop) {
            super::props::foreground(
                &mut super::shapes::Painter {
                    pixmap: &mut frame,
                    transform: super::viewport_transform(size),
                },
                activity,
                index as f32 / 8.,
                0.,
            );
        }
        frames.push(frame);
    }
    Ok(Some(frames))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chase_recovery_keeps_the_seated_dazed_landing_artwork() {
        use super::super::Animation;
        for facing in [-1., 1.] {
            let landing = frames(Activity::Stumble, (320, 224), facing)
                .unwrap()
                .unwrap();
            let recovery = Animation::new(Activity::Dizzy, facing, (320, 224)).unwrap();
            for (index, source) in [5, 6, 7, 6, 5, 6, 7, 6].into_iter().enumerate() {
                assert!(
                    recovery.frame(index as f64 / 8., false).data() == landing[source].data(),
                    "Recovery frame {index} switched away from the dazed landing strip"
                );
            }
            assert!(recovery.frame(0., true).data() == landing[5].data());
        }
    }

    #[test]
    fn dedicated_strips_are_aligned_complete_transparent_and_have_distinct_frames() {
        for activity in Activity::ALL
            .into_iter()
            .filter(|activity| super::super::strip_catalog::find(*activity).is_some())
        {
            let frames = frames(activity, (320, 224), 1.).unwrap().unwrap();
            assert_eq!(frames.len(), 8);
            let mut baselines = Vec::new();
            for frame in &frames {
                assert_eq!(frame.pixel(0, 0).unwrap().alpha(), 0);
                let bottom = (0..frame.height())
                    .rev()
                    .find(|&y| (0..frame.width()).any(|x| frame.pixel(x, y).unwrap().alpha() > 128))
                    .unwrap();
                baselines.push(bottom);
            }
            if !super::super::strip_catalog::authored_origin(activity) {
                assert!(baselines.iter().max().unwrap() - baselines.iter().min().unwrap() <= 1);
            }
            if activity != Activity::ChasePause {
                assert!(
                    frames[0].data() != frames[3].data(),
                    "{} has duplicate first and fourth frames",
                    activity.label()
                );
            }
        }
    }
    #[test]
    fn chase_strips_face_both_directions_and_ready_and_failure_poses_do_not_loop() {
        use super::super::{Animation, HEIGHT, WIDTH};
        for activity in [
            Activity::ChasePause,
            Activity::Stalk,
            Activity::Pounce,
            Activity::Stumble,
        ] {
            let right = Animation::new(activity, 1., (WIDTH, HEIGHT)).unwrap();
            let left = Animation::new(activity, -1., (WIDTH, HEIGHT)).unwrap();
            assert!(!right.matches_direction(-1.));
            assert_ne!(right.frame(0., false).data(), left.frame(0., false).data());
            if activity == Activity::ChasePause {
                assert_eq!(
                    right.frame(0., false).data(),
                    right.frame(0.4, false).data()
                );
            }
            if activity == Activity::Stalk {
                assert_eq!(
                    right.frame(0.25, false).data(),
                    right.frame(2.249, false).data()
                );
            }
            assert_eq!(
                right.frame(8., false).data(),
                right.frame(12., false).data()
            );
        }
        let boop = Animation::new(Activity::Boop, 1., (WIDTH, HEIGHT)).unwrap();
        assert_eq!(boop.frame(1.3, false).data(), boop.frame(5., false).data());
    }

    #[test]
    fn airborne_strips_preserve_their_vertical_travel_and_reduced_motion_is_static() {
        use super::super::{Animation, HEIGHT, WIDTH};
        for activity in [Activity::Excited, Activity::Pounce] {
            let animation = Animation::new(activity, 1., (WIDTH, HEIGHT)).unwrap();
            let baselines: Vec<_> = animation
                .frames
                .iter()
                .map(|frame| {
                    (0..HEIGHT)
                        .rev()
                        .find(|&y| (0..WIDTH).any(|x| frame.pixel(x, y).unwrap().alpha() > 128))
                        .unwrap()
                })
                .collect();
            assert!(
                baselines.iter().max().unwrap() - baselines.iter().min().unwrap() >= 4,
                "{activity:?} lost its airborne motion"
            );
            assert_eq!(
                animation.frame(0., true).data(),
                animation.frame(5., true).data()
            );
        }
    }
    #[test]
    fn resting_states_share_artwork_size_and_ground_and_only_blink_briefly() {
        use super::super::{Animation, HEIGHT, WIDTH};
        let idle = Animation::new(Activity::Idle, 1., (WIDTH, HEIGHT)).unwrap();
        for activity in [Activity::Watching, Activity::Waiting] {
            let quiet = Animation::new(activity, 1., (WIDTH, HEIGHT)).unwrap();
            assert_eq!(idle.frame(0., false).data(), quiet.frame(0., false).data());
        }
        for elapsed in [0., 0.5, 1.5, 2.5, 4.5, 7.49] {
            assert!(
                idle.frames[..3]
                    .iter()
                    .any(|frame| frame.data() == idle.frame(elapsed, false).data())
            );
        }
        assert!(
            !idle.frames[..3]
                .iter()
                .any(|frame| frame.data() == idle.frame(7.65, false).data())
        );
        assert_eq!(idle.frame(0., false).data(), idle.frame(8., false).data());
        let landing = Animation::new(Activity::Land, 1., (WIDTH, HEIGHT)).unwrap();
        assert_eq!(
            idle.frame(0., false).data(),
            landing.frame(1., false).data()
        );
        let bounds = |frame: &Pixmap| {
            let mut left = WIDTH;
            let mut right = 0;
            let mut bottom = 0;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if frame.pixel(x, y).unwrap().alpha() > 128 {
                        left = left.min(x);
                        right = right.max(x);
                        bottom = bottom.max(y);
                    }
                }
            }
            (right - left + 1, bottom)
        };
        let wave = Animation::new(Activity::Wave, 1., (WIDTH, HEIGHT)).unwrap();
        let rest = bounds(idle.frame(0., false));
        let reference = bounds(wave.frame(0., false));
        assert!(rest.0.abs_diff(reference.0) <= 8);
        assert!(rest.1.abs_diff(reference.1) <= 1);
    }
}
