use super::paint::Painter;
use crate::{art, behavior::Activity};
use std::collections::HashMap;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

pub(super) struct Artwork {
    pub scale: f32,
    sprites: HashMap<Activity, Pixmap>,
}

impl Artwork {
    pub fn new(scale: f32) -> Result<Self, String> {
        let mut sprites = HashMap::new();
        for activity in [
            Activity::Wave,
            Activity::Read,
            Activity::PawPlay,
            Activity::Dance,
            Activity::CozyNap,
            Activity::EarWiggle,
            Activity::Drowsy,
            Activity::Listen,
        ] {
            sprites.insert(activity, Self::render(activity, scale)?);
        }
        Ok(Self { scale, sprites })
    }
    fn render(activity: Activity, scale: f32) -> Result<Pixmap, String> {
        art::render(activity, 0., 96. * scale / art::WIDTH as f32)
    }
    pub fn draw(&self, p: &mut Painter<'_>, activity: Activity, rect: [f32; 4]) {
        let Some(image) = self.sprites.get(&activity) else {
            return;
        };
        let ratio = (rect[2] * p.scale / image.width() as f32)
            .min(rect[3] * p.scale / image.height() as f32);
        let x = (rect[0] + rect[2] / 2.) * p.scale - image.width() as f32 * ratio / 2.;
        let y = (rect[1] + rect[3] / 2.) * p.scale - image.height() as f32 * ratio / 2.;
        p.image.draw_pixmap(
            0,
            0,
            image.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..PixmapPaint::default()
            },
            Transform::from_translate(x, y).pre_scale(ratio, ratio),
            None,
        );
    }
}

pub(super) fn section_pose(tab: usize) -> Activity {
    match tab {
        1 | 8 => Activity::Read,
        2 | 10 => Activity::PawPlay,
        3 => Activity::Dance,
        4 => Activity::CozyNap,
        5 | 9 => Activity::EarWiggle,
        6 => Activity::Drowsy,
        7 => Activity::Listen,
        _ => Activity::Wave,
    }
}
