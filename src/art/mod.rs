pub(crate) mod accessories;
mod atlas;
mod attachment;
mod cozy_corner;
mod decorations;
mod hint;
mod play_assets;
mod playthings;
pub(crate) mod stickers;
pub(crate) use decorations::{Decoration, render as decoration};
mod bed;
mod held;
mod mask;
mod pickup;
mod props;
pub(crate) use pickup::Pickup;
mod shapes;
mod strip_catalog;
mod strips;
pub(crate) mod walk_cycle;
pub use mask::interaction_mask;
mod prepare;
pub use prepare::{prepare as prepare_strips, prepare_bed, prepare_play_assets};

use crate::behavior::Activity;
use shapes::Painter;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 224;

pub(crate) fn tray_icon() -> Result<Pixmap, String> {
    let mut image = Pixmap::new(64, 64).ok_or("Cannot allocate tray icon")?;
    Painter {
        pixmap: &mut image,
        transform: Transform::identity(),
    }
    .ellipse(32., 32., 30., 30., [65, 143, 182, 255], false);
    let head = atlas::cell(Activity::Idle, 0., 1.)?
        .clone_rect(tiny_skia::IntRect::from_xywh(0, 90, 192, 80).ok_or("Invalid tray crop")?)
        .ok_or("Missing tray artwork")?;
    image.draw_pixmap(
        2,
        17,
        head.as_ref(),
        &PixmapPaint {
            quality: tiny_skia::FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        Transform::from_scale(0.3125, 0.3125),
        None,
    );
    Ok(image)
}

pub fn icon(path: &std::path::Path) -> Result<(), String> {
    let source = render(Activity::Idle, 0., 3.2)?;
    let mut square = Pixmap::new(1024, 1024).ok_or("Cannot allocate app icon")?;
    square.draw_pixmap(
        0,
        154,
        source.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    square.save_png(path).map_err(|e| e.to_string())
}

pub fn render(activity: Activity, phase: f32, scale: f32) -> Result<Pixmap, String> {
    render_facing(activity, phase, scale, 1.)
}

fn render_facing(
    activity: Activity,
    phase: f32,
    scale: f32,
    facing: f32,
) -> Result<Pixmap, String> {
    render_size(
        activity,
        phase,
        (
            (WIDTH as f32 * scale) as u32,
            (HEIGHT as f32 * scale) as u32,
        ),
        facing,
    )
}

fn render_size(
    activity: Activity,
    phase: f32,
    size: (u32, u32),
    facing: f32,
) -> Result<Pixmap, String> {
    if let Some(mut frames) = strips::frames(activity, size, facing)? {
        let index =
            ((phase.clamp(0., 0.9999) * frames.len() as f32) as usize).min(frames.len() - 1);
        return Ok(frames.swap_remove(index));
    }
    let mut pixmap = Pixmap::new(size.0, size.1).ok_or("Cannot allocate pet artwork")?;
    let wave = (phase * std::f32::consts::TAU).sin();
    let sway = if matches!(activity, Activity::Listen | Activity::Dance) {
        wave * 2.
    } else {
        0.
    };
    let bob = if matches!(
        activity,
        Activity::Dance | Activity::Excited | Activity::Happy | Activity::Boop
    ) {
        -wave.abs() * 3.
    } else {
        0.
    };
    let mut transform = viewport_transform(size).pre_translate(sway, bob);
    if activity == Activity::Pant {
        transform = transform
            .pre_translate(160., 192.)
            .pre_scale(1., 1. + wave * 0.015)
            .pre_translate(-160., -192.);
    }
    if activity == Activity::Stalk {
        transform = transform
            .pre_translate(0., 195.)
            .pre_scale(1., 0.92 - phase * 0.07)
            .pre_translate(0., -195.);
    }
    if activity == Activity::Pounce {
        transform = transform
            .pre_translate(0., -(phase * std::f32::consts::PI).sin() * 34.)
            .pre_rotate_at(
                -facing * 5. * (phase * std::f32::consts::PI).sin(),
                160.,
                170.,
            );
    }
    if matches!(
        activity,
        Activity::Petting | Activity::Shy | Activity::Drowsy
    ) {
        transform = transform.pre_rotate_at(wave * 2., 160., 170.);
    }
    if activity == Activity::Land {
        transform = transform
            .pre_translate(0., 195.)
            .pre_scale(1., 0.92 + phase * 0.08)
            .pre_translate(0., -195.);
    }
    props::behind(
        &mut Painter {
            pixmap: &mut pixmap,
            transform,
        },
        activity,
        phase,
    );
    let cell = atlas::cell(activity, phase, facing)?;
    pixmap.draw_pixmap(
        0,
        0,
        cell.as_ref(),
        &PixmapPaint {
            quality: tiny_skia::FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        transform.pre_translate(35.2, -48.5).pre_scale(1.3, 1.3),
        None,
    );
    props::foreground(
        &mut Painter {
            pixmap: &mut pixmap,
            transform,
        },
        activity,
        phase,
        wave,
    );
    if activity == Activity::EdgePeek {
        let middle = size.0 as usize / 2;
        for row in pixmap.data_mut().chunks_exact_mut(size.0 as usize * 4) {
            if facing > 0. {
                row[..middle * 4].fill(0);
            } else {
                row[middle * 4..].fill(0);
            }
        }
    }
    Ok(pixmap)
}

fn viewport_transform(size: (u32, u32)) -> Transform {
    let scale = (size.0 as f32 / WIDTH as f32).min(size.1 as f32 / HEIGHT as f32);
    let offset = (
        (size.0 as f32 - WIDTH as f32 * scale) / 2.,
        (size.1 as f32 - HEIGHT as f32 * scale) / 2.,
    );
    Transform::from_translate(offset.0, offset.1).pre_scale(scale, scale)
}

pub struct Animation {
    activity: Activity,
    facing: f32,
    frames: Vec<Pixmap>,
    accessory: crate::companion::preferences::Accessory,
}

impl Animation {
    pub fn new(activity: Activity, facing: f32, size: (u32, u32)) -> Result<Self, String> {
        if let Some(frames) = strips::frames(activity, size, facing)? {
            return Ok(Self {
                activity,
                facing,
                frames,
                accessory: crate::companion::preferences::Accessory::None,
            });
        }
        let count = frame_count(activity);
        let frames = (0..count)
            .map(|i| render_size(activity, i as f32 / count as f32, size, facing))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            activity,
            facing,
            frames,
            accessory: crate::companion::preferences::Accessory::None,
        })
    }
    pub fn with_accessory(
        mut self,
        accessory: crate::companion::preferences::Accessory,
    ) -> Result<Self, String> {
        for (index, frame) in self.frames.iter_mut().enumerate() {
            accessories::apply(frame, accessory, self.activity, index, self.facing)?;
        }
        self.accessory = accessory;
        Ok(self)
    }
    pub fn accessory(&self) -> crate::companion::preferences::Accessory {
        self.accessory
    }
    pub fn activity(&self) -> Activity {
        self.activity
    }
    pub fn size(&self) -> (u32, u32) {
        (self.frames[0].width(), self.frames[0].height())
    }
    pub fn matches_direction(&self, facing: f32) -> bool {
        !directional(self.activity) || self.facing.signum() == facing.signum()
    }
    pub fn gaze_frame(&self, direction: usize, reduced: bool) -> &Pixmap {
        if reduced {
            &self.frames[0]
        } else {
            &self.frames[match direction {
                0 => 0,
                1..=7 => 2,
                _ => 5,
            }]
        }
    }
    pub fn frame(&self, elapsed: f64, reduced: bool) -> &Pixmap {
        if matches!(
            self.activity,
            Activity::Idle | Activity::Watching | Activity::Waiting
        ) {
            let time = elapsed.max(0.) % 8.;
            let index = if reduced {
                0
            } else if time < 7.5 {
                time as usize % 3
            } else {
                (3 + ((time - 7.5) * 10.) as usize).min(7)
            };
            return &self.frames[index];
        }
        if self.activity == Activity::Walk {
            let index = if reduced {
                0
            } else {
                [0, 1, 2, 4, 5, 6][((elapsed.max(0.) % 1.) * 6.) as usize]
            };
            return &self.frames[index];
        }
        let fps = match self.activity {
            Activity::Study
            | Activity::CozyNap
            | Activity::RainPlay
            | Activity::Breathe
            | Activity::CupPeek => 2.,
            Activity::TeaSip
            | Activity::PlushHug
            | Activity::PlaneReturn
            | Activity::LeafPlay
            | Activity::Birthday
            | Activity::BlanketBurrow => 3.,
            Activity::EarWiggle | Activity::GiftOpen => 4.,
            Activity::PawPlay => 5.,
            Activity::Sneeze => 6.,
            Activity::HighFive
            | Activity::CheekRub
            | Activity::Tickle
            | Activity::Nuzzle
            | Activity::SniffTreat
            | Activity::TakeTreat
            | Activity::Fetch
            | Activity::BubblePop => 5.,
            Activity::Run => 12.,
            Activity::Excited => 6.,
            Activity::Pounce => 14.,
            Activity::Snack => 5.,
            Activity::Listen | Activity::Petting => 4.,
            Activity::Dance => 6.,
            Activity::Wave | Activity::Greet => 5.,
            Activity::TypingPeek => 3.,
            Activity::Stalk => 28.,
            Activity::Pant | Activity::Happy | Activity::Shy | Activity::Stretch => 4.,
            Activity::Held => 6.,
            Activity::Dizzy => 8.,
            Activity::Stumble => 14.,
            Activity::Boop => 6.,
            Activity::Blink => 10.,
            Activity::Land => 12.,
            Activity::Walk => 8.,
            Activity::LookAround => 3.,
            Activity::Play => 3.,
            Activity::Idle | Activity::Nap | Activity::Read | Activity::Drowsy => 3.,
            _ => 6.,
        };
        let index = if reduced {
            if self.activity == Activity::BlanketBurrow {
                self.frames.len() - 1
            } else {
                0
            }
        } else {
            let tick = (elapsed.max(0.) * fps) as usize;
            if matches!(
                self.activity,
                Activity::Land
                    | Activity::Pounce
                    | Activity::Stumble
                    | Activity::Stalk
                    | Activity::Boop
                    | Activity::Blink
                    | Activity::BlanketBurrow
                    | Activity::Sneeze
                    | Activity::GiftOpen
                    | Activity::CupPeek
                    | Activity::Birthday
            ) {
                tick.min(self.frames.len() - 1)
            } else {
                tick % self.frames.len()
            }
        };
        &self.frames[index]
    }
}

fn directional(activity: Activity) -> bool {
    activity.moves() || strip_catalog::directional(activity)
}

fn frame_count(activity: Activity) -> usize {
    match activity {
        Activity::Walk
        | Activity::Run
        | Activity::Pounce
        | Activity::Snack
        | Activity::Nap
        | Activity::Listen => 8,
        Activity::ChasePause | Activity::EdgePeek => 1,
        Activity::Wave | Activity::Greet => 4,
        Activity::Stretch | Activity::Land => 5,
        Activity::LookAround => 12,
        Activity::Curious
        | Activity::Petting
        | Activity::Shy
        | Activity::Excited
        | Activity::Drowsy
        | Activity::Dance
        | Activity::Happy
        | Activity::Held
        | Activity::Play
        | Activity::Boop
        | Activity::Dizzy
        | Activity::Pant => 16,
        _ => 6,
    }
}

pub fn gallery(directory: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Cinnamoroll · Activity gallery</title><style>body{font:16px system-ui;background:#f2f7fb;color:#52768b;margin:48px}h1{font-size:28px}main{display:grid;grid-template-columns:repeat(auto-fit,minmax(250px,1fr));gap:20px}figure{margin:0;padding:20px;background:white;border-radius:24px;text-align:center}img{width:100%}figcaption{margin-top:8px}</style><h1>Cinnamoroll · Activity gallery</h1><p>Your existing ChatGPT pet artwork · 56 behavior states · 44 dedicated strips</p><main>",
    );
    for activity in Activity::ALL {
        let filename = format!("{activity:?}.png");
        render(activity, 0.3, 2.)?
            .save_png(directory.join(&filename))
            .map_err(|e| e.to_string())?;
        html.push_str(&format!(
            "<figure><img src='{filename}' alt='{}'><figcaption>{}</figcaption></figure>",
            activity.label(),
            activity.label()
        ));
    }
    let wardrobe = directory.join("wardrobe");
    std::fs::create_dir_all(&wardrobe).map_err(|e| e.to_string())?;
    for accessory in [
        crate::companion::preferences::Accessory::Bow,
        crate::companion::preferences::Accessory::Scarf,
        crate::companion::preferences::Accessory::Hat,
        crate::companion::preferences::Accessory::Bag,
    ] {
        for activity in [Activity::Idle, Activity::TeaSip, Activity::PlushHug] {
            let mut frame = render(activity, 0.3, 2.)?;
            accessories::apply(&mut frame, accessory, activity, 2, 1.)?;
            frame
                .save_png(wardrobe.join(format!("{accessory:?}-{activity:?}.png")))
                .map_err(|e| e.to_string())?;
        }
    }
    accessories::gallery(&wardrobe)?;
    html.push_str("</main>");
    std::fs::write(directory.join("index.html"), html).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_match_the_physical_display_size_without_an_extra_resize() {
        for size in [(128, 89), (192, 134), (256, 178)] {
            let animation = Animation::new(Activity::Listen, 1., size).unwrap();
            assert_eq!(animation.size(), size);
            assert_eq!(animation.frame(0.3, false).width(), size.0);
            assert_eq!(animation.frame(0.3, false).height(), size.1);
        }
    }
    #[test]
    fn every_activity_has_transparency_and_visible_character() {
        for activity in Activity::ALL {
            for phase in [0., 0.25, 0.5, 0.75] {
                let image = render(activity, phase, 1.).unwrap();
                assert_eq!(image.pixel(0, 0).unwrap().alpha(), 0);
                for x in 0..image.width() {
                    assert_eq!(
                        image.pixel(x, 0).unwrap().alpha(),
                        0,
                        "{activity:?} clips at the top"
                    );
                    assert_eq!(
                        image.pixel(x, image.height() - 1).unwrap().alpha(),
                        0,
                        "{activity:?} clips at the bottom"
                    );
                }
                let opaque = image.pixels().iter().filter(|p| p.alpha() > 200).count();
                assert!(opaque > 1000, "{activity:?} has no visible character");
            }
        }
    }
    #[test]
    fn both_directional_rows_render_and_landings_do_not_loop() {
        let right = Animation::new(Activity::Walk, 1., (WIDTH, HEIGHT)).unwrap();
        let left = Animation::new(Activity::Walk, -1., (WIDTH, HEIGHT)).unwrap();
        assert_ne!(left.frame(0., false).data(), right.frame(0., false).data());
        let landing = Animation::new(Activity::Land, 1., (WIDTH, HEIGHT)).unwrap();
        assert_eq!(
            landing.frame(2., false).data(),
            landing.frame(3., false).data()
        );
    }
}
