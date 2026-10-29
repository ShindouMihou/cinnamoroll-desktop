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
