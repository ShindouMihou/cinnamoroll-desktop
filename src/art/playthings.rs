use super::{decorations::oval, shapes::Painter};
use crate::companion::world::Kind;
use tiny_skia::PathBuilder;

fn panel(p: &mut Painter<'_>, points: &[(f32, f32)], color: [u8; 4]) {
    let mut b = PathBuilder::new();
    if let Some(first) = points.first() {
        b.move_to(first.0, first.1);
        for point in &points[1..] {
            b.line_to(point.0, point.1);
        }
        b.close();
        if let Some(path) = b.finish() {
            p.path(&path, Some(color), Some(([174, 112, 61, 255], 2.2)));
        }
    }
}

pub(super) fn draw(p: &mut Painter<'_>, kind: Kind, stage: u8, palette: [u32; 3], phase: f32) {
    let blue = [195, 226, 245, 255];
    let pink = [248, 205, 220, 255];
    let purple = [219, 209, 246, 255];
    let brown = [174, 112, 61, 255];
    match kind {
        Kind::Breathing => {
            let radius = 24. + (phase * std::f32::consts::TAU / 6.4).sin() * 10.;
            oval(p, 80., 56., radius, radius, blue);
            p.heart(80., 56., 12., 220);
        }
        Kind::Wand => {
            p.line(&[(28., 94.), (83., 37.)], brown, 6.);
            p.curve((83., 37.), [110., 14., 114., 63., 133., 49.], pink, 3.);
            oval(p, 134., 48., 17., 17., purple);
            p.heart(134., 48., 7., 245);
        }
        Kind::Ribbon => {
            p.curve(
                (17., 78.),
                [
                    75.,
                    8. + phase.sin() * 4.,
                    72.,
                    112. - phase.sin() * 4.,
                    142.,
                    38.,
                ],
                pink,
                11.,
            );
            p.curve(
                (17., 78.),
                [
                    75.,
                    8. + phase.sin() * 4.,
                    72.,
                    112. - phase.sin() * 4.,
                    142.,
                    38.,
                ],
                brown,
                1.8,
            );
            oval(p, 19., 78., 10., 7., purple);
        }
        Kind::Plane => {
            panel(p, &[(14., 45.), (147., 22.), (94., 91.), (76., 59.)], blue);
            p.line(&[(147., 22.), (76., 59.), (14., 45.)], brown, 2.);
            p.line(&[(76., 59.), (64., 80.), (94., 70.)], brown, 2.);
        }
        Kind::Blanket => {
            panel(p, &[(20., 31.), (130., 22.), (145., 84.), (31., 96.)], blue);
            for x in [40., 60., 80., 100., 120.] {
                p.line(&[(x, 39.), (x + 7., 77.)], [245, 251, 255, 255], 2.);
            }
            p.heart(80., 59., 13., 240);
        }
        Kind::Tea => {
            oval(p, 80., 86., 61., 13., pink);
            oval(p, 120., 57., 17., 18., blue);
            panel(p, &[(42., 38.), (112., 38.), (103., 84.), (53., 84.)], blue);
            oval(p, 77., 39., 35., 10., [255, 239, 206, 255]);
            p.heart(78., 63., 9., 245);
            oval(p, 30., 79., 16., 10., [242, 201, 149, 255]);
            oval(p, 30., 75., 15., 9., [255, 228, 180, 255]);
            let spiral: Vec<_> = (0..24)
                .map(|i| {
                    let t = i as f32 / 23. * std::f32::consts::TAU * 1.5;
                    let r = 1. + i as f32 * 0.4;
                    (30. + t.cos() * r, 75. + t.sin() * r * 0.5)
                })
                .collect();
            p.line(&spiral, brown, 2.);
        }
        Kind::HideCups => {
            for i in 0..3 {
                let x = 29. + i as f32 * 51.;
                let lift = if stage & 3 == i + 1 { 25. } else { 0. };
                if stage & 3 == i + 1 && stage >> 2 == i {
                    oval(p, x, 86., 10., 10., pink);
                }
                panel(
                    p,
                    &[
                        (x - 14., 32. - lift),
                        (x + 14., 32. - lift),
                        (x + 20., 83. - lift),
                        (x - 20., 83. - lift),
                    ],
                    [blue, pink, purple][i as usize],
                );
                oval(p, x, 32. - lift, 14., 5., [255, 250, 244, 255]);
            }
        }
        Kind::Parcel => {
            panel(p, &[(36., 36.), (124., 36.), (124., 94.), (36., 94.)], pink);
            p.line(&[(80., 37.), (80., 94.)], blue, 13.);
            p.line(&[(36., 60.), (124., 60.)], blue, 10.);
            oval(p, 64., 25., 18., 10., purple);
            oval(p, 94., 25., 18., 10., purple);
            oval(p, 80., 31., 8., 7., blue);
        }
        Kind::Pillow => {
            oval(p, 80., 64., 65., 34., pink);
            oval(p, 80., 59., 59., 28., [255, 247, 251, 255]);
            p.curve((27., 65.), [58., 82., 103., 45., 133., 64.], purple, 3.);
            p.heart(80., 59., 12., 245);
        }
        Kind::Plush => {
            oval(p, 80., 75., 35., 27., purple);
            oval(p, 51., 27., 16., 16., purple);
            oval(p, 109., 27., 16., 16., purple);
            oval(p, 80., 48., 35., 29., purple);
            oval(p, 80., 57., 21., 15., [255, 250, 244, 255]);
            for x in [67., 93.] {
                oval(p, x, 43., 3., 4., brown);
            }
            oval(p, 80., 54., 4., 3., brown);
            for x in [57., 103.] {
                oval(p, x, 94., 14., 12., purple);
            }
        }
        Kind::Palette => {
            oval(p, 80., 65., 67., 34., [255, 250, 244, 255]);
            for (i, c) in palette.into_iter().enumerate() {
                let rgba = [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255];
                oval(p, 40. + i as f32 * 40., 55., 16., 16., rgba);
            }
            oval(p, 126., 78., 10., 7., [246, 251, 253, 255]);
        }
        _ => {}
    }
}
