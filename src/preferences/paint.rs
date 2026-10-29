use fontdue::{Font, FontSettings, Metrics};
use std::collections::HashMap;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub const INK: [u8; 3] = [42, 74, 91];
pub const MUTED: [u8; 3] = [108, 130, 145];
pub const BLUE: [u8; 3] = [70, 145, 181];
pub const SOFT: [u8; 3] = [229, 244, 250];
pub const BORDER: [u8; 3] = [220, 232, 238];

pub struct Typography {
    font: Font,
    glyphs: HashMap<(char, u16), (Metrics, Vec<u8>)>,
}

impl Typography {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            font: Font::from_bytes(
                include_bytes!("../../assets/Geist.ttf") as &[u8],
                FontSettings::default(),
            )
            .map_err(str::to_owned)?,
            glyphs: HashMap::new(),
        })
    }
}

pub struct Painter<'a> {
    pub image: &'a mut Pixmap,
    pub typography: &'a mut Typography,
    pub scale: f32,
}

impl Painter<'_> {
    pub fn pill(&mut self, rect: [f32; 4], radius: f32, color: [u8; 3], border: Option<[u8; 3]>) {
        let [x, y, w, h] = rect;
        let r = radius.min(w / 2.).min(h / 2.);
        let mut path = PathBuilder::new();
        path.move_to(x + r, y);
        path.line_to(x + w - r, y);
        path.quad_to(x + w, y, x + w, y + r);
        path.line_to(x + w, y + h - r);
        path.quad_to(x + w, y + h, x + w - r, y + h);
        path.line_to(x + r, y + h);
        path.quad_to(x, y + h, x, y + h - r);
        path.line_to(x, y + r);
        path.quad_to(x, y, x + r, y);
        path.close();
        if let Some(path) = path.finish() {
            let paint = |c: [u8; 3]| Paint {
                shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(c[0], c[1], c[2], 255)),
                anti_alias: true,
                ..Paint::default()
            };
            let transform = Transform::from_scale(self.scale, self.scale);
            self.image
                .fill_path(&path, &paint(color), FillRule::Winding, transform, None);
            if let Some(color) = border {
                self.image.stroke_path(
                    &path,
                    &paint(color),
                    &Stroke {
                        width: 1.,
                        ..Stroke::default()
                    },
                    transform,
                    None,
                );
            }
        }
    }

    pub fn centered(&mut self, text: &str, rect: [f32; 4], size: f32, color: [u8; 3]) {
        let pixels = (size * self.scale).round().max(1.) as u16;
        let width: f32 = text
            .chars()
            .map(|character| {
                self.typography
                    .glyphs
                    .entry((character, pixels))
                    .or_insert_with(|| self.typography.font.rasterize(character, f32::from(pixels)))
                    .0
                    .advance_width
            })
            .sum();
        let line = self
            .typography
            .font
            .horizontal_line_metrics(f32::from(pixels));
        let baseline = line.map_or(size * 0.35, |line| {
            (line.ascent + line.descent) / (2. * self.scale)
        });
        if width / self.scale > rect[2] - 20. {
            self.fitted(
                text,
                [
                    rect[0] + 10.,
                    rect[1] + rect[3] / 2. + baseline - size,
                    rect[2] - 20.,
                    rect[3],
                ],
                size,
                color,
            );
            return;
        }
        self.text(
            text,
            rect[0] + (rect[2] - width / self.scale) / 2.,
            rect[1] + rect[3] / 2. + baseline - size,
            size,
            color,
        );
    }

    pub fn fitted(&mut self, text: &str, rect: [f32; 4], size: f32, color: [u8; 3]) {
        let pixels = (size * self.scale).round().max(1.) as u16;
        let mut value = String::new();
        let mut width = 0.;
        for c in text.chars() {
            let advance = self
                .typography
                .glyphs
                .entry((c, pixels))
                .or_insert_with(|| self.typography.font.rasterize(c, f32::from(pixels)))
                .0
                .advance_width
                / self.scale;
            if width + advance > rect[2] - size {
                value.push('…');
                break;
            }
            width += advance;
            value.push(c);
        }
        self.text(&value, rect[0], rect[1], size, color);
    }
    pub fn text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [u8; 3]) {
        let pixels = (size * self.scale).round().max(1.) as u16;
        let baseline = ((y + size) * self.scale).round() as i32;
        let mut cursor = x * self.scale;
        let width = self.image.width() as i32;
        let height = self.image.height() as i32;
        for character in text.chars() {
            let (metrics, bitmap) = self
                .typography
                .glyphs
                .entry((character, pixels))
                .or_insert_with(|| self.typography.font.rasterize(character, f32::from(pixels)));
            let left = cursor.round() as i32 + metrics.xmin;
            let top = baseline - metrics.height as i32 - metrics.ymin;
            let data = self.image.data_mut();
            for row in 0..metrics.height {
                let py = top + row as i32;
                if py < 0 || py >= height {
                    continue;
                }
                for column in 0..metrics.width {
                    let px = left + column as i32;
                    if px < 0 || px >= width {
                        continue;
                    }
                    let alpha = u32::from(bitmap[row * metrics.width + column]);
                    let offset = (py * width + px) as usize * 4;
                    for channel in 0..3 {
                        data[offset + channel] = ((u32::from(color[channel]) * alpha
                            + u32::from(data[offset + channel]) * (255 - alpha))
                            / 255) as u8;
                    }
                }
            }
            cursor += metrics.advance_width;
        }
    }
}
