use tiny_skia::Transform;

pub(super) fn transform(grip: (f32, f32), wave: f32) -> Transform {
    Transform::from_translate(grip.0, grip.1)
        .pre_rotate(wave * 1.5)
        .pre_translate(-grip.0, -grip.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Point;

    #[test]
    fn every_grab_point_stays_fixed_through_the_entire_hanging_cycle() {
        for size in [(128, 89), (192, 134), (320, 224)] {
            for fraction in [(0.2, 0.6), (0.5, 0.3), (0.8, 0.6)] {
                let grip = (size.0 as f32 * fraction.0, size.1 as f32 * fraction.1);
                for frame in 0..8 {
                    let wave = (frame as f32 / 8. * std::f32::consts::TAU).sin();
                    let mut point = Point::from_xy(grip.0, grip.1);
                    transform(grip, wave).map_point(&mut point);
                    assert!((point.x - grip.0).abs() < 0.001);
                    assert!((point.y - grip.1).abs() < 0.001);
                }
            }
        }
    }
}
