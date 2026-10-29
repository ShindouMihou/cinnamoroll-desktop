use crate::{companion::corner_home, geometry::Bounds};

type Point = (f32, f32);

pub(super) struct CornerDrag {
    offset: Point,
    origin: Point,
    passenger: Option<Point>,
    pub moved: bool,
}

impl CornerDrag {
    pub fn new(pointer: Point, window: (i32, i32), passenger: Option<Point>) -> Self {
        Self {
            offset: (pointer.0 - window.0 as f32, pointer.1 - window.1 as f32),
            origin: pointer,
            passenger,
            moved: false,
        }
    }
    pub fn carries_pet(&self) -> bool {
        self.passenger.is_some()
    }
    pub fn motion(
        &mut self,
        pointer: Point,
        size: (u32, u32),
        bounds: Bounds,
    ) -> Option<(Point, Option<Point>)> {
        if !self.moved && (pointer.0 - self.origin.0).hypot(pointer.1 - self.origin.1) <= 3. {
            return None;
        }
        self.moved = true;
        let layout = crate::companion::corner::Layout::new(size);
        let margin = -layout.offset.0;
        let desired = (
            pointer.0 - self.offset.0 + margin,
            pointer.1 - self.offset.1 - layout.offset.1,
        );
        let home = if let Some(offset) = self.passenger {
            let left = (-margin).min(offset.0);
            let top = 0_f32.min(offset.1);
            let right = (size.0 as f32 + margin).max(offset.0 + size.0 as f32);
            let bottom = (layout.footprint.1 as f32).max(offset.1 + size.1 as f32);
            let group = bounds.clamp(
                (desired.0 + left, desired.1 + top),
                ((right - left).ceil() as u32, (bottom - top).ceil() as u32),
            );
            (group.0 - left, group.1 - top)
        } else {
            corner_home(None, desired, size, bounds)
        };
        Some((
            home,
            self.passenger
                .map(|offset| (home.0 + offset.0, home.1 + offset.1)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bounds() -> Bounds {
        Bounds {
            x: -1200,
            y: -200,
            width: 1200,
            height: 900,
        }
    }
    #[test]
    fn blanket_drag_moves_both_by_the_same_amount_without_a_pickup_jump() {
        let mut drag = CornerDrag::new((-880., 455.), (-916, 445), Some((2., -3.)));
        assert_eq!(drag.motion((-878., 455.), (128, 89), bounds()), None);
        let (home, pet) = drag.motion((-830., 480.), (128, 89), bounds()).unwrap();
        assert_eq!(home, (-850., 425.));
        assert_eq!(pet, Some((-848., 422.)));
        let (home, pet) = drag.motion((-800., 500.), (128, 89), bounds()).unwrap();
        assert_eq!(pet, Some((home.0 + 2., home.1 - 3.)));
    }
    #[test]
    fn the_full_group_stays_on_screen_and_an_empty_bed_does_not_collect_a_pet() {
        for size in [(96, 67), (128, 89), (176, 123), (240, 168)] {
            let mut drag = CornerDrag::new((-880., 455.), (-916, 445), Some((2., -3.)));
            for pointer in [(-5000., -5000.), (5000., 5000.)] {
                let (home, pet) = drag.motion(pointer, size, bounds()).unwrap();
                let pet = pet.unwrap();
                let scale = size.0 as f32 / 128.;
                assert!(home.0 - 16. * scale >= -1200.);
                assert!(home.0 + 144. * scale <= 0.);
                assert!(home.1 >= -200.);
                assert!(
                    home.1 + crate::companion::corner::Layout::new(size).footprint.1 as f32 <= 700.
                );
                assert!(pet.1 >= -200.);
                assert_eq!((pet.0 - home.0, pet.1 - home.1), (2., -3.));
            }
        }
        let mut empty = CornerDrag::new((-880., 475.), (-916, 465), None);
        assert!(!empty.carries_pet());
        assert!(
            empty
                .motion((-830., 500.), (128, 89), bounds())
                .unwrap()
                .1
                .is_none()
        );
    }
}
