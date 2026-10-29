use super::shapes::Painter;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint};

pub(super) fn draw(p: &mut Painter<'_>, theme: u8) -> Result<(), String> {
    let mut image = Pixmap::decode_png(include_bytes!("../../assets/decorations/cozy-bed.png"))
        .map_err(|error| error.to_string())?;
    if matches!(theme, 1 | 2) {
        for pixel in image.data_mut().as_chunks_mut::<4>().0 {
            let [r, g, b, _] = *pixel;
            if b.saturating_sub(r) > 4 && g > r && b >= g {
                let range = f32::from(b - r);
                let channel = |amount: f32| r.saturating_add((range * amount).round() as u8);
                let color = if theme == 1 {
                    [b, channel(0.15), channel(0.42)]
                } else {
                    [channel(0.6), channel(0.32), b]
                };
                pixel[..3].copy_from_slice(&color);
            }
        }
    }
    p.pixmap.draw_pixmap(
        0,
        0,
        image.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        p.transform
            .pre_scale(320. / image.width() as f32, 112. / image.height() as f32),
        None,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::art::{Decoration, decoration};
    #[test]
    fn fabric_colors_preserve_the_soft_bed_silhouette_and_transparent_margin() {
        for size in [(160, 56), (320, 112), (640, 224)] {
            let blue = decoration(Decoration::Corner(0), size, 0.).unwrap();
            assert_eq!(blue.pixel(0, 0).unwrap().alpha(), 0);
            for theme in [1, 2] {
                let tinted = decoration(Decoration::Corner(theme), size, 0.).unwrap();
                let changed = blue
                    .data()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(tinted.data().as_chunks::<4>().0.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                assert!(changed > (size.0 * size.1 / 10) as usize);
                assert!(
                    blue.data()
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(tinted.data().as_chunks::<4>().0.iter())
                        .all(|(a, b)| a[3] == b[3])
                );
            }
        }
    }
}
