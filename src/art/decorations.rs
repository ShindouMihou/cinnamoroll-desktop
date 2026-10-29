use super::shapes::Painter;
use tiny_skia::{PathBuilder, Pixmap, Rect, Transform};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Decoration {
    Corner(u8),
    Dream(u8),
    Cups(crate::companion::games::CupVisual),
    Hint(&'static str),
    Ball,
    Treat,
    Bubbles(u8),
    Prop(crate::companion::world::Kind, u8, [u32; 3]),
}

pub(crate) fn render(kind: Decoration, size: (u32, u32), phase: f32) -> Result<Pixmap, String> {
    if let Decoration::Hint(text) = kind {
        return super::hint::render(text, size);
    }
    let mut image = Pixmap::new(size.0, size.1).ok_or("Cannot allocate playroom artwork")?;
    let logical = match kind {
        Decoration::Dream(_) => (160., 112.),
        Decoration::Corner(_) => (320., 112.),
        Decoration::Cups(_) => (160., 112.),
        Decoration::Bubbles(_) => (192., 160.),
        Decoration::Prop(..) => (160., 112.),
        _ => (80., 80.),
    };
    let mut p = Painter {
        pixmap: &mut image,
        transform: Transform::from_scale(size.0 as f32 / logical.0, size.1 as f32 / logical.1),
    };
    match kind {
        Decoration::Dream(kind) => super::moments::dream(&mut p, kind),
        Decoration::Prop(kind, stage, palette) => {
            if super::moment_assets::draw(&mut p, kind, stage)? {
                super::moments::effects(&mut p, kind, stage, phase);
            } else if matches!(
                kind,
                crate::companion::world::Kind::Wand
                    | crate::companion::world::Kind::Ribbon
                    | crate::companion::world::Kind::HideCups
            ) {
                super::play_assets::draw(&mut p, kind, stage, phase)?;
            } else {
                if kind == crate::companion::world::Kind::Plane && stage == 1 {
                    p.transform = p.transform.pre_translate(160., 0.).pre_scale(-1., 1.);
                }
                super::playthings::draw(&mut p, kind, stage, palette, phase)
            }
        }
        Decoration::Cups(visual) => super::play_assets::draw_cups(&mut p, visual)?,
        Decoration::Hint(_) => {}
        Decoration::Corner(theme) => super::cozy_corner::draw(&mut p, theme)?,
        Decoration::Ball => {
            oval(&mut p, 40., 40., 26., 26., [176, 218, 245, 255]);
            p.curve(
                (25., 24.),
                [53., 27. + phase.sin() * 4., 22., 52., 55., 57.],
                [238, 248, 255, 255],
                4.,
            );
            p.ellipse(29., 26., 6., 4., [255, 255, 255, 230], false);
        }
        Decoration::Treat => {
            oval(&mut p, 40., 44., 27., 20., [242, 201, 149, 255]);
            oval(&mut p, 40., 38., 26., 18., [255, 228, 180, 255]);
            let spiral: Vec<_> = (0..60)
                .map(|i| {
                    let t = i as f32 / 59. * std::f32::consts::TAU * 1.8;
                    let r = 2. + i as f32 * 0.28;
                    (40. + t.cos() * r, 38. + t.sin() * r * 0.65)
                })
                .collect();
            p.line(&spiral, [174, 112, 56, 255], 3.);
        }
        Decoration::Bubbles(mask) => {
            for (i, (x, y, r)) in [(45., 95., 25.), (105., 49., 29.), (151., 107., 22.)]
                .into_iter()
                .enumerate()
            {
                if mask & (1 << i) == 0 {
                    continue;
                }
                oval(&mut p, x, y, r, r, [210, 235, 254, 150]);
                p.curve(
                    (x - r * 0.5, y - r * 0.2),
                    [
                        x - r * 0.55,
                        y - r * 0.65,
                        x - r * 0.15,
                        y - r * 0.75,
                        x + r * 0.15,
                        y - r * 0.65,
                    ],
                    [255, 255, 255, 240],
                    3.,
                );
            }
            let popped = (mask >> 3).saturating_sub(1) as usize;
            if mask >> 3 > 0 && popped < 3 {
                let (x, y, r) = [(45., 95., 25.), (105., 49., 29.), (151., 107., 22.)][popped];
                let t = (phase / 1.2).clamp(0., 1.);
                if let Some(path) = PathBuilder::from_circle(x, y, r * (1. + t * 0.3)) {
                    p.path(
                        &path,
                        None,
                        Some(([165, 213, 239, ((1. - t) * 220.) as u8], 2.)),
                    );
                }
            }
        }
    }
    Ok(image)
}

pub(super) fn oval(p: &mut Painter<'_>, x: f32, y: f32, rx: f32, ry: f32, color: [u8; 4]) {
    if let Some(rect) = Rect::from_xywh(x - rx, y - ry, rx * 2., ry * 2.)
        && let Some(path) = PathBuilder::from_oval(rect)
    {
        p.path(&path, Some(color), Some(([174, 112, 61, 255], 2.2)));
    }
}
