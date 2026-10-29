use crate::preferences::paint::Painter;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Transform};

pub(crate) fn cloud(p: &mut Painter<'_>, x: f32, y: f32, color: [u8; 3]) {
    p.pill([x, y + 5., 22., 8.], 4., color, None);
    p.pill([x + 4., y + 1., 10., 11.], 5., color, None);
    p.pill([x + 11., y + 3., 8., 9.], 4., color, None);
}

pub(crate) fn heart(p: &mut Painter<'_>, x: f32, y: f32, color: [u8; 3]) {
    let mut path = PathBuilder::new();
    path.move_to(x, y + 2.);
    path.cubic_to(x - 7., y - 5., x - 10., y + 5., x, y + 10.);
    path.cubic_to(x + 10., y + 5., x + 7., y - 5., x, y + 2.);
    if let Some(path) = path.finish() {
        p.image.fill_path(
            &path,
            &Paint {
                shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(
                    color[0], color[1], color[2], 255,
                )),
                anti_alias: true,
                ..Paint::default()
            },
            FillRule::Winding,
            Transform::from_scale(p.scale, p.scale),
            None,
        );
    }
}

pub(crate) fn chevron(p: &mut Painter<'_>, x: f32, y: f32, back: bool) {
    let direction = if back { -1. } else { 1. };
    let mut path = PathBuilder::new();
    path.move_to(x - direction * 2., y - 4.);
    path.line_to(x + direction * 2., y);
    path.line_to(x - direction * 2., y + 4.);
    if let Some(path) = path.finish() {
        p.image.stroke_path(
            &path,
            &Paint {
                shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(104, 150, 173, 255)),
                anti_alias: true,
                ..Paint::default()
            },
            &tiny_skia::Stroke {
                width: 1.5,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..tiny_skia::Stroke::default()
            },
            Transform::from_scale(p.scale, p.scale),
            None,
        );
    }
}
