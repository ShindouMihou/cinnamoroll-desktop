use crate::geometry::Bounds;

const GAP: f32 = 12.;

pub(super) fn position(anchor: Bounds, size: (u32, u32), display: Bounds) -> (f32, f32) {
    crate::geometry::place_next_to(anchor, size, display, GAP)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(anchor: Bounds, size: (u32, u32), display: Bounds) {
        let p = position(anchor, size, display);
        assert!(p.0 >= display.x as f32 && p.1 >= display.y as f32);
        assert!(p.0 + size.0 as f32 <= display.x as f32 + display.width as f32);
        assert!(p.1 + size.1 as f32 <= display.y as f32 + display.height as f32);
        assert!(
            p.0 >= anchor.x as f32 + anchor.width as f32 + GAP
                || p.0 + size.0 as f32 <= anchor.x as f32 - GAP
                || p.1 >= anchor.y as f32 + anchor.height as f32 + GAP
                || p.1 + size.1 as f32 <= anchor.y as f32 - GAP
        );
    }
    #[test]
    fn menus_keep_clear_of_the_pet_and_blanket_at_screen_edges_and_after_resizing() {
        let display = Bounds {
            x: 0,
            y: 24,
            width: 1440,
            height: 876,
        };
        for (x, y) in [(0, 24), (1280, 24), (0, 760), (1280, 760), (640, 400)] {
            for height in [233, 381, 418, 446, 488] {
                check(
                    Bounds {
                        x,
                        y,
                        width: 160,
                        height: 121,
                    },
                    (284, height),
                    display,
                );
            }
        }
        let pet = Bounds {
            x: 1280,
            y: 400,
            width: 160,
            height: 121,
        };
        assert!(position(pet, (284, 381), display).0 < pet.x as f32);
    }
    #[test]
    fn placement_uses_vertical_space_when_sides_do_not_fit_and_handles_negative_monitors() {
        check(
            Bounds {
                x: 150,
                y: 700,
                width: 160,
                height: 100,
            },
            (284, 381),
            Bounds {
                x: 0,
                y: 0,
                width: 460,
                height: 1000,
            },
        );
        check(
            Bounds {
                x: -200,
                y: -100,
                width: 160,
                height: 121,
            },
            (284, 418),
            Bounds {
                x: -1440,
                y: -200,
                width: 1440,
                height: 900,
            },
        );
    }
}
