use super::{Animation, HEIGHT, WIDTH, viewport_transform};
use crate::behavior::Activity;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint};

pub fn interaction_mask(width: u32, height: u32) -> Result<Pixmap, String> {
    let mut base = Pixmap::new(WIDTH, HEIGHT).ok_or("Cannot allocate the interaction mask")?;
    for activity in Activity::ALL {
        let directions: &[f32] = if super::directional(activity) {
            &[-1., 1.]
        } else {
            &[1.]
        };
        for &facing in directions {
            let animation = Animation::new(activity, facing, (WIDTH, HEIGHT))?;
            for frame in animation.frames {
                for (destination, source) in base
                    .data_mut()
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(frame.data().as_chunks::<4>().0.iter())
                {
                    if source[3] > 0 {
                        *destination = [255; 4];
                    }
                }
            }
        }
    }
    let original = base.clone();
    for y in 1..HEIGHT - 1 {
        for x in 1..WIDTH - 1 {
            if original.pixel(x, y).is_some_and(|pixel| pixel.alpha() > 0) {
                for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    let offset = ((ny * WIDTH + nx) * 4) as usize;
                    base.data_mut()[offset..offset + 4].fill(255);
                }
            }
        }
    }
    let mut scaled = Pixmap::new(width, height).ok_or("Cannot scale the interaction mask")?;
    scaled.draw_pixmap(
        0,
        0,
        base.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Nearest,
            ..PixmapPaint::default()
        },
        viewport_transform((width, height)),
        None,
    );
    Ok(scaled)
}
