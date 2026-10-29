use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform,
};

pub const WHITE: [u8; 4] = [255, 254, 250, 255];
pub const BLUE: [u8; 4] = [114, 173, 200, 255];
pub const OUTLINE: [u8; 4] = [164, 200, 214, 255];
pub const PINK: [u8; 4] = [248, 180, 196, 230];

pub struct Painter<'a> {
    pub pixmap: &'a mut Pixmap,
    pub transform: Transform,
}

impl Painter<'_> {
    pub fn path(&mut self, path: &Path, fill: Option<[u8; 4]>, outline: Option<([u8; 4], f32)>) {
        if let Some(color) = fill {
            self.pixmap
                .fill_path(path, &paint(color), FillRule::Winding, self.transform, None);
        }
        if let Some((color, width)) = outline {
            let stroke = Stroke {
                width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Stroke::default()
            };
            self.pixmap
                .stroke_path(path, &paint(color), &stroke, self.transform, None);
        }
    }

    pub fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32, fill: [u8; 4], outline: bool) {
        if let Some(rect) = tiny_skia::Rect::from_xywh(x - rx, y - ry, rx * 2., ry * 2.)
            && let Some(path) = PathBuilder::from_oval(rect)
        {
            self.path(&path, Some(fill), outline.then_some((OUTLINE, 1.8)));
        }
    }

    pub fn line(&mut self, points: &[(f32, f32)], color: [u8; 4], width: f32) {
        if let Some(&(x, y)) = points.first() {
            let mut path = PathBuilder::new();
            path.move_to(x, y);
            for &(x, y) in &points[1..] {
                path.line_to(x, y);
            }
            if let Some(path) = path.finish() {
                self.path(&path, None, Some((color, width)));
            }
        }
    }

    pub fn curve(&mut self, start: (f32, f32), controls: [f32; 6], color: [u8; 4], width: f32) {
        let mut path = PathBuilder::new();
        path.move_to(start.0, start.1);
        path.cubic_to(
            controls[0],
            controls[1],
            controls[2],
            controls[3],
            controls[4],
            controls[5],
        );
        if let Some(path) = path.finish() {
            self.path(&path, None, Some((color, width)));
        }
    }

    pub fn heart(&mut self, x: f32, y: f32, size: f32, opacity: u8) {
        let mut path = PathBuilder::new();
        path.move_to(x, y + size);
        path.cubic_to(
            x - size * 2.,
            y - size * 0.2,
            x - size,
            y - size,
            x,
            y - size * 0.25,
        );
        path.cubic_to(
            x + size,
            y - size,
            x + size * 2.,
            y - size * 0.2,
            x,
            y + size,
        );
        path.close();
        if let Some(path) = path.finish() {
            self.path(&path, Some([244, 153, 182, opacity]), None);
        }
    }

    pub fn cloud(&mut self, x: f32, y: f32, size: f32) {
        self.ellipse(x, y, size * 1.5, size * 0.6, [226, 244, 252, 255], false);
        self.ellipse(
            x - size * 0.7,
            y - size * 0.3,
            size * 0.65,
            size * 0.65,
            [226, 244, 252, 255],
            false,
        );
        self.ellipse(
            x + size * 0.3,
            y - size * 0.55,
            size * 0.85,
            size * 0.85,
            [226, 244, 252, 255],
            false,
        );
    }
}

fn paint(color: [u8; 4]) -> Paint<'static> {
    Paint {
        shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(
            color[0], color[1], color[2], color[3],
        )),
        anti_alias: true,
        ..Paint::default()
    }
}
