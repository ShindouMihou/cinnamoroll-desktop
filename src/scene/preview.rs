use crate::{
    art::{self, Animation, Decoration},
    behavior::Activity,
    companion::corner::Layout,
};
use std::path::Path;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

pub(crate) fn cozy(folder: &Path) -> Result<(), String> {
    art::decoration(Decoration::Corner(0), (640, 224), 0.)?
        .save_png(folder.join("cozy-bed.png"))
        .map_err(|error| error.to_string())?;
    let pet_size = (256, 179);
    let layout = Layout::new(pet_size);
    let bed = art::decoration(Decoration::Corner(0), layout.size, 0.)?;
    let mut image = Pixmap::new(layout.footprint.0 + 40, layout.footprint.1 + 40)
        .ok_or("Cannot allocate cozy preview")?;
    image.fill(tiny_skia::Color::from_rgba8(248, 252, 255, 255));
    image.draw_pixmap(
        20,
        20 + layout.offset.1.round() as i32,
        bed.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    let animation = Animation::new(Activity::BlanketBurrow, 1., pet_size)?;
    image.draw_pixmap(
        20 - layout.offset.0.round() as i32,
        20,
        animation.frame(5., false).as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    image
        .save_png(folder.join("tucked-in.png"))
        .map_err(|error| error.to_string())?;
    dreams(folder)
}

fn dreams(folder: &Path) -> Result<(), String> {
    for width in [96, 128, 176] {
        let size = (width, width * 224 / 320);
        let pet = (150., 70.);
        let mut image = Pixmap::new(880, 520).ok_or("Cannot allocate dream preview")?;
        image.fill(tiny_skia::Color::from_rgba8(248, 252, 255, 255));
        let mut stamp = |source: &Pixmap, point: (f32, f32)| {
            image.draw_pixmap(
                (point.0 * 2.).round() as i32,
                (point.1 * 2.).round() as i32,
                source.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            )
        };
        let layout = Layout::new(size);
        let bed = art::decoration(
            Decoration::Corner(0),
            (layout.size.0 * 2, layout.size.1 * 2),
            0.,
        )?;
        stamp(&bed, (pet.0 + layout.offset.0, pet.1 + layout.offset.1));
        let animation = Animation::new(Activity::CozyNap, 1., (size.0 * 2, size.1 * 2))?;
        stamp(animation.frame(0., false), pet);
        let (bubble, point) = crate::companion::placement::dream_layout(
            pet,
            size,
            crate::geometry::Bounds {
                x: 0,
                y: 0,
                width: 440,
                height: 260,
            },
        );
        let dream = art::decoration(Decoration::Dream(0), (bubble.0 * 2, bubble.1 * 2), 0.)?;
        stamp(&dream, point);
        image
            .save_png(folder.join(format!("dream-{width}.png")))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn social(folder: &Path) -> Result<(), String> {
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    for (name, activity, fps) in [
        ("paw", Activity::HighFive, 5.),
        ("wave", Activity::Wave, 5.),
        ("plush", Activity::PlushHug, 3.),
        ("walk", Activity::Walk, 6.),
        ("stretch", Activity::Stretch, 4.),
        ("greet", Activity::Greet, 5.),
        ("idle", Activity::Idle, 1.),
    ] {
        let animation = Animation::new(activity, 1., (320, 224))?;
        for frame in 0..8 {
            animation
                .frame(f64::from(frame) / fps, false)
                .save_png(folder.join(format!("{name}-{frame}.png")))
                .map_err(|e| e.to_string())?;
        }
    }
    for (name, decoration, size) in [
        ("ball", Decoration::Ball, (72, 72)),
        (
            "plush-toy",
            Decoration::Prop(
                crate::companion::world::Kind::Plush,
                0,
                crate::companion::preferences::Extras::default().palette,
            ),
            (96, 96),
        ),
    ] {
        art::decoration(decoration, size, 0.)?
            .save_png(folder.join(format!("{name}.png")))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
