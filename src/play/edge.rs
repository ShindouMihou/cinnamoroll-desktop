use crate::geometry::Bounds;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeStage {
    Entering,
    Watching { until: f64 },
    Leaving,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeVisit {
    pub origin: (f32, f32),
    pub target: (f32, f32),
    pub bounds: Bounds,
    pub facing: f32,
    pub stage: EdgeStage,
}

pub fn plan(
    position: (f32, f32),
    size: (u32, u32),
    bounds: Bounds,
    near_only: bool,
) -> Option<EdgeVisit> {
    if bounds.width < size.0 * 2 {
        return None;
    }
    let left = position.0 - bounds.x as f32;
    let right = bounds.x as f32 + bounds.width as f32 - size.0 as f32 - position.0;
    if near_only && left.min(right) > size.0 as f32 * 0.75 {
        return None;
    }
    let facing = if left <= right { 1. } else { -1. };
    let x = if facing > 0. {
        bounds.x as f32 - size.0 as f32 * 0.5
    } else {
        bounds.x as f32 + bounds.width as f32 - size.0 as f32 * 0.5
    };
    Some(EdgeVisit {
        origin: position,
        target: (x, position.1),
        bounds,
        facing,
        stage: EdgeStage::Entering,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edge_peeks_hide_half_the_window_and_automatic_peeks_stay_nearby() {
        let bounds = Bounds {
            x: -1440,
            y: -100,
            width: 1440,
            height: 900,
        };
        let left = plan((-1400., 200.), (128, 89), bounds, true).unwrap();
        assert_eq!(left.target, (-1504., 200.));
        assert_eq!(left.facing, 1.);
        let right = plan((-180., 200.), (128, 89), bounds, true).unwrap();
        assert_eq!(right.target, (-64., 200.));
        assert_eq!(right.facing, -1.);
        assert!(plan((-720., 200.), (128, 89), bounds, true).is_none());
        assert!(plan((-720., 200.), (128, 89), bounds, false).is_some());
    }
}
