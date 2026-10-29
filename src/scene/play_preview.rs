use crate::{
    art::{self, Animation, Decoration},
    behavior::Activity,
    companion::{corner::Layout, placement::item_center, world::Kind},
    geometry::Bounds,
};
use std::path::Path;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

fn stamp(canvas: &mut Pixmap, image: &Pixmap, point: (f32, f32)) {
    canvas.draw_pixmap(
        (point.0 * 2.).round() as i32,
        (point.1 * 2.).round() as i32,
        image.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}
fn scene(
    folder: &Path,
    name: &str,
    kind: Kind,
    code: u8,
    phase: f32,
    label: &'static str,
) -> Result<(), String> {
    let pet = (150., 65.);
    let size = (128, 89);
    let layout = Layout::new(size);
    let mut image = Pixmap::new(1200, 520).ok_or("Cannot allocate play preview")?;
    image.fill(tiny_skia::Color::from_rgba8(246, 251, 253, 255));
    let bed = art::decoration(
        Decoration::Corner(0),
        (layout.size.0 * 2, layout.size.1 * 2),
        0.,
    )?;
    stamp(
        &mut image,
        &bed,
        (pet.0 + layout.offset.0, pet.1 + layout.offset.1),
    );
    let prop_size = kind.size();
    let center = if matches!(kind, Kind::Ribbon | Kind::Wand) {
        let tip = if kind == Kind::Ribbon {
            (20., 84.)
        } else {
            (18., 73.)
        };
        (
            pet.0 + size.0 as f32 * 0.65 + prop_size.0 as f32 / 2.
                - tip.0 / 160. * prop_size.0 as f32,
            pet.1 + size.1 as f32 * 0.8 + prop_size.1 as f32 / 2.
                - tip.1 / 112. * prop_size.1 as f32,
        )
    } else {
        item_center(
            kind,
            pet,
            size,
            Bounds {
                x: 0,
                y: 0,
                width: 600,
                height: 260,
            },
            Some(pet),
        )
    };
    let decoration = match kind {
        Kind::Ball => Decoration::Ball,
        Kind::Bubbles => Decoration::Bubbles(7),
        _ => Decoration::Prop(kind, code, [0xb0daf5, 0xf8cddc, 0xdbd1f6]),
    };
    let toy = art::decoration(decoration, (prop_size.0 * 2, prop_size.1 * 2), phase)?;
    stamp(
        &mut image,
        &toy,
        (
            center.0 - prop_size.0 as f32 / 2.,
            center.1 - prop_size.1 as f32 / 2.,
        ),
    );
    let activity = if kind == Kind::Brush {
        Activity::Groom
    } else if matches!(kind, Kind::Ribbon | Kind::Wand) {
        Activity::PawPlay
    } else {
        Activity::Watching
    };
    let pet_image = Animation::new(activity, 1., (size.0 * 2, size.1 * 2))?;
    stamp(&mut image, pet_image.frame(0.65, false), pet);
    if kind == Kind::Wand {
        stamp(
            &mut image,
            &toy,
            (
                center.0 - prop_size.0 as f32 / 2.,
                center.1 - prop_size.1 as f32 / 2.,
            ),
        );
    }
    let hint = art::decoration(Decoration::Hint(label), (360, 48), 0.)?;
    stamp(
        &mut image,
        &hint,
        (center.0 - 90., center.1 + prop_size.1 as f32 / 2. + 8.),
    );
    image
        .save_png(folder.join(format!("{name}.png")))
        .map_err(|error| error.to_string())
}
pub(crate) fn gallery(folder: &Path) -> Result<(), String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    for (name, kind, code, phase, label) in [
        ("brush", Kind::Brush, 0, 0., "Brush his head or ears"),
        ("dandelion", Kind::Dandelion, 0, 0., "Offer near his nose"),
        ("globe", Kind::SnowGlobe, 1, 2., "Drag to shake the globe"),
        ("pond", Kind::Pond, 50, 0.5, "Drag the little boat"),
        (
            "keepsakes",
            Kind::KeepsakeBox,
            143,
            0.,
            "Click to open / close",
        ),
        ("cups-watch", Kind::HideCups, 4, 1., "Watch the little star"),
        (
            "cups-guess",
            Kind::HideCups,
            5,
            0.,
            "Which cup? Tap to choose",
        ),
        (
            "cups-reveal",
            Kind::HideCups,
            22,
            1.,
            "You found the little star!",
        ),
        ("ribbon", Kind::Ribbon, 0, 0., "Drag the bow; offer the end"),
        ("wand", Kind::Wand, 0, 0., "Bring the pompom near a paw"),
        ("bubbles", Kind::Bubbles, 0, 0., "Click a bubble to pop it"),
        ("plane", Kind::Plane, 0, 0., "Drag and release to launch"),
        ("ball", Kind::Ball, 0, 0., "Drag and release to toss"),
    ] {
        scene(folder, name, kind, code, phase, label)?;
    }
    shuffle_frames(folder)?;
    super::preview::cozy(folder)
}

fn shuffle_frames(folder: &Path) -> Result<(), String> {
    use crate::companion::games::CupRound;
    let path = folder.join("cup-shuffle-frames");
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    let mut round = CupRound::new(0., 0);
    round.swaps = [(0, 1), (1, 2), (0, 1)];
    for index in 0..100 {
        let now = f64::from(index) * 0.05;
        round.advance(now);
        let cups = art::decoration(Decoration::Cups(round.visual(now, false)), (540, 288), 0.)?;
        let hint = art::decoration(Decoration::Hint(round.hint()), (540, 72), 0.)?;
        let mut image = Pixmap::new(600, 390).ok_or("Cannot allocate shuffle preview")?;
        image.fill(tiny_skia::Color::from_rgba8(246, 251, 253, 255));
        image.draw_pixmap(
            30,
            0,
            cups.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        image.draw_pixmap(
            30,
            300,
            hint.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        image
            .save_png(path.join(format!("frame-{index:03}.png")))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
