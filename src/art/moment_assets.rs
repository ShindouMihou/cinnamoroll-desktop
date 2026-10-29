use super::shapes::Painter;
use crate::companion::world::Kind;
use std::sync::OnceLock;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint};

pub(super) fn draw(p: &mut Painter<'_>, kind: Kind, stage: u8) -> Result<bool, String> {
    let index = match kind {
        Kind::Brush => 0,
        Kind::Dandelion => 1,
        Kind::SnowGlobe => 2,
        Kind::Pond => 3,
        Kind::KeepsakeBox => {
            if stage & 128 == 0 {
                4
            } else {
                5
            }
        }
        _ => return Ok(false),
    };
    static CACHE: [OnceLock<Result<Pixmap, String>>; 6] = [const { OnceLock::new() }; 6];
    const BYTES: [&[u8]; 6] = [
        include_bytes!("../../assets/moments/runtime/brush.png"),
        include_bytes!("../../assets/moments/runtime/dandelion.png"),
        include_bytes!("../../assets/moments/runtime/globe.png"),
        include_bytes!("../../assets/moments/runtime/pond.png"),
        include_bytes!("../../assets/moments/runtime/box-closed.png"),
        include_bytes!("../../assets/moments/runtime/box-open.png"),
    ];
    let image = CACHE[index]
        .get_or_init(|| Pixmap::decode_png(BYTES[index]).map_err(|e| e.to_string()))
        .as_ref()
        .map_err(Clone::clone)?;
    p.pixmap.draw_pixmap(
        0,
        0,
        image.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..Default::default()
        },
        p.transform.pre_scale(1. / 3., 1. / 3.),
        None,
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn props_are_visible_with_transparent_margins_and_open_box_changes() {
        for kind in [
            Kind::Brush,
            Kind::Dandelion,
            Kind::SnowGlobe,
            Kind::Pond,
            Kind::KeepsakeBox,
        ] {
            let image = crate::art::decoration(
                crate::art::Decoration::Prop(kind, 0, [0; 3]),
                (320, 224),
                0.,
            )
            .unwrap();
            assert!(image.data().as_chunks::<4>().0.iter().any(|p| p[3] > 128));
            for x in 0..320 {
                assert_eq!(image.pixel(x, 0).unwrap().alpha(), 0);
                assert_eq!(image.pixel(x, 223).unwrap().alpha(), 0);
            }
            for y in 0..224 {
                assert_eq!(image.pixel(0, y).unwrap().alpha(), 0);
                assert_eq!(image.pixel(319, y).unwrap().alpha(), 0);
            }
        }
        let closed = crate::art::decoration(
            crate::art::Decoration::Prop(Kind::KeepsakeBox, 0, [0; 3]),
            (160, 112),
            0.,
        )
        .unwrap();
        let open = crate::art::decoration(
            crate::art::Decoration::Prop(Kind::KeepsakeBox, 143, [0; 3]),
            (160, 112),
            0.,
        )
        .unwrap();
        assert_ne!(closed.data(), open.data());
    }
}
