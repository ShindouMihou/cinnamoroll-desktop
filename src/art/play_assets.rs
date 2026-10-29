use super::{decorations::oval, shapes::Painter};
use crate::companion::world::Kind;
use std::sync::OnceLock;
use tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint};

fn prop(kind: Kind) -> Result<&'static Pixmap, String> {
    static RIBBON: OnceLock<Result<Pixmap, String>> = OnceLock::new();
    static WAND: OnceLock<Result<Pixmap, String>> = OnceLock::new();
    let (cache, bytes) = if kind == Kind::Ribbon {
        (
            &RIBBON,
            include_bytes!("../../assets/decorations/ribbon.png") as &[u8],
        )
    } else {
        (
            &WAND,
            include_bytes!("../../assets/decorations/wand.png") as &[u8],
        )
    };
    cache
        .get_or_init(|| Pixmap::decode_png(bytes).map_err(|e| e.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}
fn cups() -> Result<&'static [Pixmap; 4], String> {
    static CUPS: OnceLock<Result<[Pixmap; 4], String>> = OnceLock::new();
    CUPS.get_or_init(|| {
        let atlas = Pixmap::decode_png(include_bytes!("../../assets/decorations/cups.png"))
            .map_err(|e| e.to_string())?;
        let mut cells = Vec::with_capacity(4);
        for index in 0..4 {
            cells.push(
                atlas
                    .clone_rect(
                        IntRect::from_xywh(index % 2 * 256, index / 2 * 256, 256, 256)
                            .ok_or("Invalid cup crop")?,
                    )
                    .ok_or("Missing cup art")?,
            );
        }
        cells
            .try_into()
            .map_err(|_| "Incomplete cup atlas".to_owned())
    })
    .as_ref()
    .map_err(Clone::clone)
}
fn stamp(p: &mut Painter<'_>, image: &Pixmap, x: f32, y: f32, width: f32, height: f32) {
    p.pixmap.draw_pixmap(
        0,
        0,
        image.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        p.transform
            .pre_translate(x, y)
            .pre_scale(width / image.width() as f32, height / image.height() as f32),
        None,
    );
}
pub(super) fn draw(p: &mut Painter<'_>, kind: Kind, code: u8, phase: f32) -> Result<(), String> {
    if kind != Kind::HideCups {
        let image = prop(kind)?;
        let old = p.transform;
        p.transform = p.transform.pre_rotate_at(phase.sin() * 1.2, 80., 84.);
        if kind == Kind::Wand {
            p.transform = p.transform.pre_translate(160., 0.).pre_scale(-1., 1.);
        }
        stamp(p, image, 0., 0., 160., 112.);
        p.transform = old;
        return Ok(());
    }
    let state = code & 3;
    draw_cups(
        p,
        crate::companion::games::CupVisual {
            order: [0, 1, 2],
            target: (code >> 2) & 3,
            chosen: if state == 0 { 3 } else { (code >> 4) & 3 },
            lift: if state == 0 || state >= 2 {
                (phase.clamp(0., 1.) * 10.).round() as u8
            } else {
                0
            },
            swap: None,
        },
    )
}

pub(super) fn draw_cups(
    p: &mut Painter<'_>,
    visual: crate::companion::games::CupVisual,
) -> Result<(), String> {
    let images = cups()?;
    oval(p, 80., 96., 76., 11., [238, 246, 251, 255]);
    p.curve(
        (12., 96.),
        [48., 108., 112., 108., 148., 96.],
        [255, 253, 250, 255],
        2.,
    );
    let poses = visual.poses();
    let mut slots = [0usize, 1, 2];
    slots.sort_by(|a, b| poses[*a].1.total_cmp(&poses[*b].1));
    for slot in slots {
        let (x, y) = poses[slot];
        let revealed =
            visual.lift > 0 && (slot as u8 == visual.target || slot as u8 == visual.chosen);
        let lift = if revealed {
            18. * f32::from(visual.lift) / 10.
        } else {
            0.
        };
        if slot as u8 == visual.target && revealed {
            stamp(p, &images[3], x - 9., 82., 18., 18.);
        }
        stamp(
            p,
            &images[visual.order[slot] as usize],
            x - 25.,
            y - lift,
            50.,
            58.,
        );
    }
    Ok(())
}
