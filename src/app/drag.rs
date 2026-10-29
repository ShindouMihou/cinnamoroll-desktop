pub(super) struct Drag {
    pub(super) offset: (f32, f32),
    origin: (f32, f32),
    pub(super) moved: bool,
}

impl Drag {
    pub(super) fn new(offset: (f32, f32), origin: (f32, f32)) -> Self {
        Self {
            offset,
            origin,
            moved: false,
        }
    }

    pub(super) fn motion(&mut self, pointer: (f32, f32)) -> Option<(f32, f32)> {
        let position = (pointer.0 - self.offset.0, pointer.1 - self.offset.1);
        if !self.moved && (position.0 - self.origin.0).hypot(position.1 - self.origin.1) <= 4. {
            return None;
        }
        self.moved = true;
        Some(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grab_stays_at_its_original_cursor_offset_without_repositioning() {
        for offset in [(30., 70.), (100., 50.), (160., 74.)] {
            let mut drag = Drag::new(offset, (500., 250.));
            assert_eq!(drag.motion((502. + offset.0, 251. + offset.1)), None);
            assert_eq!(
                drag.motion((510. + offset.0, 255. + offset.1)),
                Some((510., 255.))
            );
            for pointer in [(700., 400.), (700., 400.), (699., 401.)] {
                let position = drag.motion(pointer).unwrap();
                assert_eq!((position.0 + offset.0, position.1 + offset.1), pointer);
            }
        }
    }
}
