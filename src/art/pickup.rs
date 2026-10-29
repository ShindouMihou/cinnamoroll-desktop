use crate::behavior::Activity;
use sdl3::{
    render::FPoint,
    render::{Texture, WindowCanvas},
};
use tiny_skia::{Pixmap, PixmapPaint};

pub(crate) struct Pickup {
    frames: Vec<Pixmap>,
    elapsed: f64,
    pivot: FPoint,
    uploaded: Option<usize>,
    tilt: Tilt,
}

#[derive(Default)]
struct Tilt {
    previous: Option<(f32, f32)>,
    angle: f32,
}

impl Tilt {
    fn update(&mut self, position: (f32, f32), dt: f32, reduced: bool) -> f32 {
        let previous = self.previous.replace(position).unwrap_or(position);
        let dt = dt.clamp(0.001, 0.05);
        let target = if reduced {
            0.
        } else {
            (-(position.0 - previous.0) / dt * 0.012).clamp(-3., 3.)
        };
        self.angle = if reduced {
            0.
        } else {
            self.angle + (target - self.angle) * (1. - (-12. * dt).exp())
        };
        self.angle
    }
}

impl Pickup {
    pub fn new(
        size: (u32, u32),
        grip: (f32, f32),
        window_size: (u32, u32),
    ) -> Result<Self, String> {
        let pivot = FPoint::new(
            grip.0 * size.0 as f32 / window_size.0 as f32,
            grip.1 * size.1 as f32 / window_size.1 as f32,
        );
        let source = super::strips::frames(Activity::Held, size, 1.)?
            .ok_or("Missing dedicated hanging animation")?;
        let count = source.len();
        let mut frames = Vec::with_capacity(count);
        for (index, cell) in source.into_iter().enumerate() {
            let wave = (index as f32 / count as f32 * std::f32::consts::TAU).sin();
            let mut image =
                Pixmap::new(size.0, size.1).ok_or("Cannot allocate hanging animation")?;
            let transform = super::held::transform((pivot.x, pivot.y), wave);
            image.draw_pixmap(
                0,
                0,
                cell.as_ref(),
                &PixmapPaint {
                    quality: tiny_skia::FilterQuality::Bilinear,
                    ..PixmapPaint::default()
                },
                transform,
                None,
            );
            frames.push(image);
        }
        Ok(Self {
            frames,
            elapsed: 0.,
            pivot,
            uploaded: None,
            tilt: Tilt::default(),
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.frames[0].width(), self.frames[0].height())
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture: &mut Texture<'_>,
        position: (f32, f32),
        dt: f32,
        reduced: bool,
    ) -> Result<(), String> {
        let index = if reduced {
            0
        } else {
            (self.elapsed * 6.) as usize % self.frames.len()
        };
        self.elapsed += f64::from(dt.clamp(0., 0.1));
        if self.uploaded != Some(index) {
            let image = &self.frames[index];
            texture
                .update(None, image.data(), image.width() as usize * 4)
                .map_err(|e| e.to_string())?;
            self.uploaded = Some(index);
        }
        let angle = self.tilt.update(position, dt, reduced);
        canvas.clear();
        canvas
            .copy_ex(
                texture,
                None,
                None,
                f64::from(angle),
                Some(self.pivot),
                false,
                false,
            )
            .map_err(|e| e.to_string())?;
        canvas.present();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_tilt_starts_without_a_jump_is_limited_and_settles_when_motion_stops() {
        let mut tilt = Tilt::default();
        assert_eq!(tilt.update((500., 250.), 0.016, false), 0.);
        let angle = tilt.update((550., 250.), 0.016, false);
        assert!(angle < 0. && angle > -1.);
        for step in 1..100 {
            assert!(
                tilt.update((550. + step as f32 * 50., 250.), 0.016, false)
                    .abs()
                    <= 3.
            );
        }
        for _ in 0..100 {
            tilt.update((5500., 250.), 0.016, false);
        }
        assert!(tilt.angle.abs() < 0.001);
        assert_eq!(tilt.update((6000., 250.), 0.016, true), 0.);
    }
    #[test]
    fn dragging_uses_eight_dedicated_frames_at_the_physical_display_size() {
        for (size, logical) in [((128, 89), (128, 89)), ((256, 178), (128, 89))] {
            for grip in [(30., 55.), (64., 25.), (100., 55.)] {
                let pickup = Pickup::new(size, grip, logical).unwrap();
                assert_eq!(pickup.frames.len(), 8);
                assert_eq!(pickup.size(), size);
                assert_ne!(pickup.frames[0].data(), pickup.frames[4].data());
                assert!((pickup.pivot.x - grip.0 * size.0 as f32 / logical.0 as f32).abs() < 0.001);
                assert!((pickup.pivot.y - grip.1 * size.1 as f32 / logical.1 as f32).abs() < 0.001);
            }
        }
    }
}
