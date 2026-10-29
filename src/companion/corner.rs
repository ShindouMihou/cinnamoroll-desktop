#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub size: (u32, u32),
    pub offset: (f32, f32),
    pub footprint: (u32, u32),
}

impl Layout {
    pub fn new(pet: (u32, u32)) -> Self {
        let scale = pet.0 as f32 / 128.;
        let size = ((160. * scale).round() as u32, (56. * scale).round() as u32);
        let offset = (
            -(size.0.saturating_sub(pet.0) as f32) / 2.,
            pet.1 as f32 - 44. * scale,
        );
        Self {
            size,
            offset,
            footprint: (size.0, pet.1.max((offset.1 + size.1 as f32).ceil() as u32)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{art::Animation, behavior::Activity};
    #[test]
    fn tucked_blanket_rests_on_the_mattress_instead_of_floating_above_it() {
        for size in [(96, 67), (128, 89), (192, 134), (256, 179)] {
            let layout = Layout::new(size);
            let animation = Animation::new(Activity::BlanketBurrow, 1., size).unwrap();
            let image = animation.frame(5., false);
            let x = size.0 / 2;
            let bottom = (0..size.1)
                .rev()
                .find(|y| image.pixel(x, *y).is_some_and(|pixel| pixel.alpha() > 128))
                .unwrap() as f32;
            let mattress = layout.offset.1 + layout.size.1 as f32 * 60. / 112.;
            let gap = (mattress - bottom).abs();
            assert!(
                gap <= 3. * size.0 as f32 / 128.,
                "Blanket is {gap} pixels from the mattress at {size:?}"
            );
        }
    }
}
