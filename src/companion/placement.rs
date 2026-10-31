use super::{corner::Layout, world::Kind};
use crate::geometry::{Bounds, place_next_to};

pub fn item_center(
    kind: Kind,
    pet: (f32, f32),
    size: (u32, u32),
    display: Bounds,
    home: Option<(f32, f32)>,
) -> (f32, f32) {
    let mut left = pet.0;
    let mut top = pet.1;
    let mut right = pet.0 + size.0 as f32;
    let mut bottom = pet.1 + size.1 as f32;
    if let Some(home) = home
        && (home.0 - pet.0).hypot(home.1 - pet.1) < size.0 as f32 * 2.
    {
        let layout = Layout::new(size);
        left = left.min(home.0 + layout.offset.0);
        top = top.min(home.1 + layout.offset.1);
        right = right.max(home.0 + layout.offset.0 + layout.size.0 as f32);
        bottom = bottom.max(home.1 + layout.offset.1 + layout.size.1 as f32);
    }
    let anchor = Bounds {
        x: left.floor() as i32,
        y: top.floor() as i32,
        width: (right - left).ceil() as u32,
        height: (bottom - top).ceil() as u32,
    };
    let item = kind.size();
    let envelope = (item.0.max(180), item.1 + 28);
    let point = place_next_to(anchor, envelope, display, 18.);
    (
        point.0 + envelope.0 as f32 / 2.,
        point.1 + item.1 as f32 / 2.,
    )
}

pub fn dream_layout(
    pet: (f32, f32),
    size: (u32, u32),
    display: Bounds,
) -> ((u32, u32), (f32, f32)) {
    let width = (size.0 as f32 * 0.5).round().clamp(48., 100.) as u32;
    let bubble = (width, (width as f32 * 0.7).round() as u32);
    let head = (pet.0 + size.0 as f32 * 0.64, pet.1 + size.1 as f32 * 0.46);
    let position = display.clamp(
        (
            head.0 - bubble.0 as f32 * 0.15,
            head.1 - bubble.1 as f32 * 0.90 - 4.,
        ),
        bubble,
    );
    (bubble, position)
}

#[cfg(test)]
mod dream_tests {
    use super::*;
    #[test]
    fn dream_layout_keeps_its_tail_close_to_the_sleeping_head_at_every_pet_size() {
        let display = Bounds {
            x: 0,
            y: 0,
            width: 1800,
            height: 1400,
        };
        for width in [96, 128, 176, 240, 320, 400] {
            let size = (width, width * 224 / 320);
            let pet = (400., 500.);
            let (bubble, point) = dream_layout(pet, size, display);
            let tail = (
                point.0 + bubble.0 as f32 * 0.15,
                point.1 + bubble.1 as f32 * 0.90,
            );
            let head = (pet.0 + size.0 as f32 * 0.64, pet.1 + size.1 as f32 * 0.46);
            assert!(
                (tail.0 - head.0).hypot(tail.1 - head.1) <= 6.,
                "Dream is detached at pet width {width}: tail {tail:?}, head {head:?}"
            );
        }
    }
    #[test]
    fn dreams_remain_visible_on_negative_displays_and_near_screen_edges() {
        let display = Bounds {
            x: -1200,
            y: -100,
            width: 1200,
            height: 900,
        };
        for pet in [
            (-1200., -100.),
            (-128., -100.),
            (-1200., 700.),
            (-128., 700.),
        ] {
            let (size, p) = dream_layout(pet, (128, 89), display);
            assert!(p.0 >= -1200. && p.1 >= -100.);
            assert!(p.0 + size.0 as f32 <= 0. && p.1 + size.1 as f32 <= 800.);
        }
    }
}
