#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    pub fn clamp(self, position: (f32, f32), size: (u32, u32)) -> (f32, f32) {
        let max_x = self.x as f32 + self.width.saturating_sub(size.0) as f32;
        let max_y = self.y as f32 + self.height.saturating_sub(size.1) as f32;
        (
            position.0.clamp(self.x as f32, max_x),
            position.1.clamp(self.y as f32, max_y),
        )
    }

    pub fn contains(self, point: (f32, f32)) -> bool {
        point.0 >= self.x as f32
            && point.1 >= self.y as f32
            && point.0 < self.x as f32 + self.width as f32
            && point.1 < self.y as f32 + self.height as f32
    }
}

pub fn place_next_to(anchor: Bounds, size: (u32, u32), display: Bounds, gap: f32) -> (f32, f32) {
    let x = anchor.x as f32;
    let y = anchor.y as f32;
    let right = x + anchor.width as f32;
    let bottom = y + anchor.height as f32;
    let width = size.0 as f32;
    let height = size.1 as f32;
    let candidates = [
        (right + gap, y + (anchor.height as f32 - height) / 2.),
        (x - width - gap, y + (anchor.height as f32 - height) / 2.),
        (x + (anchor.width as f32 - width) / 2., bottom + gap),
        (x + (anchor.width as f32 - width) / 2., y - height - gap),
    ]
    .map(|point| display.clamp(point, size));
    if let Some(point) = candidates.iter().copied().find(|p| {
        p.0 >= right + gap
            || p.0 + width <= x - gap
            || p.1 >= bottom + gap
            || p.1 + height <= y - gap
    }) {
        return point;
    }
    let overlap = |p: (f32, f32)| {
        ((p.0 + width).min(right) - p.0.max(x)).max(0.)
            * ((p.1 + height).min(bottom) - p.1.max(y)).max(0.)
    };
    candidates
        .into_iter()
        .min_by(|a, b| overlap(*a).total_cmp(&overlap(*b)))
        .unwrap_or((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_monitor_coordinates_and_small_displays_are_safe() {
        let bounds = Bounds {
            x: -1920,
            y: -500,
            width: 1920,
            height: 1080,
        };
        assert_eq!(bounds.clamp((20., 1000.), (320, 240)), (-320., 340.));
        assert_eq!(bounds.clamp((-2500., -900.), (320, 240)), (-1920., -500.));
        assert_eq!(bounds.clamp((0., 0.), (3000, 3000)), (-1920., -500.));
    }
}
