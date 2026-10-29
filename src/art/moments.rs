use super::{decorations::oval, shapes::Painter};
use crate::companion::world::Kind;
use tiny_skia::{PathBuilder, Rect};

pub(super) fn draw(p: &mut Painter<'_>, kind: Kind, _stage: u8, _phase: f32) {
    if kind == Kind::ClearZone {
        if let Some(r) = Rect::from_xywh(2., 2., 156., 108.) {
            p.path(
                &PathBuilder::from_rect(r),
                Some([213, 236, 249, 60]),
                Some(([118, 172, 200, 200], 1.5)),
            );
        }
        for (x, y) in [(5., 5.), (155., 5.), (5., 107.), (155., 107.)] {
            p.ellipse(x, y, 3., 3., [195, 226, 245, 255], false);
        }
        p.heart(80., 56., 10., 170);
    }
}

pub(super) fn effects(p: &mut Painter<'_>, kind: Kind, stage: u8, phase: f32) {
    match kind {
        Kind::Dandelion if stage > 0 => {
            let t = (phase / 5.).clamp(0., 1.);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.;
                let x = 72. + a.cos() * 14. + t * (i as f32 * 6. - 20.);
                let y = 35. + a.sin() * 14. - t * 20.;
                p.line(
                    &[(x - 3., y - 2.), (x, y + 1.), (x + 3., y - 2.)],
                    [255, 252, 240, ((1. - t) * 240.) as u8],
                    1.,
                );
            }
        }
        Kind::SnowGlobe if stage > 0 => {
            let t = (phase / 6.).clamp(0., 1.);
            for i in 0..12 {
                let x = 62. + (i * 17 % 36) as f32;
                let y = if t >= 1. {
                    76. + (i % 2) as f32
                } else {
                    28. + (i * 13 % 30) as f32 + t * 18.
                };
                p.ellipse(x, y, 1.1, 1.1, [255, 255, 255, 235], false);
            }
        }
        Kind::Pond => {
            let x = 40. + stage.min(100) as f32 * 0.8;
            let y = 47. + phase.sin();
            let mut b = PathBuilder::new();
            b.move_to(x - 14., y);
            b.line_to(x + 14., y);
            b.line_to(x + 8., y + 8.);
            b.line_to(x - 7., y + 8.);
            b.close();
            if let Some(path) = b.finish() {
                p.path(
                    &path,
                    Some([248, 205, 220, 255]),
                    Some(([174, 112, 61, 255], 1.2)),
                );
            }
            p.line(
                &[(x - 5., y), (x, y - 13.), (x + 6., y)],
                [174, 112, 61, 255],
                1.2,
            );
            p.curve(
                (x - 19., y + 10.),
                [x - 7., y + 13., x + 8., y + 13., x + 18., y + 10.],
                [240, 252, 255, 200],
                1.,
            );
        }
        Kind::KeepsakeBox if stage & 128 > 0 => {
            for i in 0..4 {
                if stage & (1 << i) != 0 {
                    let x = 53. + i as f32 * 18.;
                    match i {
                        0 => {
                            p.line(&[(x, 62.), (x + 5., 69.)], [165, 132, 99, 255], 2.);
                            oval(p, x - 2., 60., 4., 3., [195, 226, 245, 255]);
                        }
                        1 => {
                            p.line(&[(x, 63.), (x, 69.)], [133, 177, 142, 255], 1.);
                            p.ellipse(x, 59., 4., 4., [255, 253, 237, 255], false);
                        }
                        2 => p.heart(x, 63., 4., 235),
                        _ => p.line(
                            &[(x - 5., 61.), (x, 65.), (x + 5., 61.), (x - 5., 61.)],
                            [207, 150, 160, 255],
                            2.,
                        ),
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn dream(p: &mut Painter<'_>, kind: u8) {
    oval(p, 24., 101., 4., 3., [247, 251, 255, 235]);
    oval(p, 35., 88., 7., 5., [247, 251, 255, 235]);
    oval(p, 82., 48., 58., 38., [247, 251, 255, 245]);
    match kind {
        0 => {
            oval(p, 82., 49., 24., 15., [255, 228, 180, 255]);
            p.curve(
                (69., 49.),
                [72., 31., 104., 51., 79., 57.],
                [174, 112, 61, 255],
                2.,
            );
        }
        1 => {
            for (x, y, r) in [(64., 51., 14.), (80., 43., 21.), (99., 53., 14.)] {
                p.ellipse(x, y, r, 14., [191, 225, 245, 255], false);
            }
        }
        2 => {
            oval(p, 82., 48., 21., 21., [219, 209, 246, 255]);
            p.heart(82., 48., 9., 230);
        }
        _ => p.heart(82., 48., 18., 245),
    }
}
