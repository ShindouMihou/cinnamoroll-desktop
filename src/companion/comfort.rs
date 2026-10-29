use crate::geometry::Bounds;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub zones: Vec<Bounds>,
    pub preferred_display: Option<Bounds>,
    pub nooks: Vec<Nook>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nook {
    pub display: Bounds,
    pub position: (i32, i32),
}

#[derive(Clone, Copy)]
pub struct MoveSnapshot {
    pub pet: (f32, f32),
    pub corner: Option<(i32, i32)>,
}

impl Preferences {
    pub fn valid(&self) -> bool {
        self.zones.len() <= 8
            && self.nooks.len() <= 16
            && self
                .zones
                .iter()
                .all(|b| (80..=800).contains(&b.width) && (60..=600).contains(&b.height))
            && self
                .nooks
                .iter()
                .all(|n| n.display.width > 0 && n.display.height > 0)
    }
    pub fn remember(&mut self, display: Bounds, position: (i32, i32)) {
        self.nooks.retain(|n| n.display != display);
        if self.nooks.len() == 16 {
            self.nooks.remove(0);
        }
        self.nooks.push(Nook { display, position });
    }
    pub fn nook(&self, display: Bounds) -> Option<(i32, i32)> {
        self.nooks
            .iter()
            .find(|n| n.display == display)
            .map(|n| n.position)
    }
    pub fn allows(&self, point: (f32, f32), size: (u32, u32)) -> bool {
        !self.zones.iter().any(|zone| overlaps(point, size, *zone))
    }
    pub fn segment_clear(&self, from: (f32, f32), to: (f32, f32), size: (u32, u32)) -> bool {
        let distance = (to.0 - from.0).abs().max((to.1 - from.1).abs());
        let steps = (distance / 8.).ceil().max(1.) as u32;
        (0..=steps).all(|i| {
            let t = i as f32 / steps as f32;
            self.allows(
                (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t),
                size,
            )
        })
    }
    pub fn step(&self, from: (f32, f32), to: (f32, f32), size: (u32, u32)) -> (f32, f32) {
        if self.segment_clear(from, to, size) {
            return to;
        }
        if !self.allows(from, size) {
            let overlap = |p: (f32, f32)| {
                self.zones
                    .iter()
                    .map(|z| {
                        ((p.0 + size.0 as f32).min(z.x as f32 + z.width as f32)
                            - p.0.max(z.x as f32))
                        .max(0.)
                            * ((p.1 + size.1 as f32).min(z.y as f32 + z.height as f32)
                                - p.1.max(z.y as f32))
                            .max(0.)
                    })
                    .sum::<f32>()
            };
            return if overlap(to) < overlap(from) {
                to
            } else {
                from
            };
        }
        for point in [(to.0, from.1), (from.0, to.1)] {
            if point != from && self.segment_clear(from, point, size) {
                return point;
            }
        }
        from
    }
    pub fn waypoint(
        &self,
        from: (f32, f32),
        target: (f32, f32),
        size: (u32, u32),
        display: Bounds,
    ) -> (f32, f32) {
        if self.segment_clear(from, target, size) || !self.allows(from, size) {
            return target;
        }
        let mut nodes = vec![from, target];
        for z in &self.zones {
            for x in [
                z.x as f32 - size.0 as f32 - 8.,
                z.x as f32 + z.width as f32 + 8.,
            ] {
                for y in [
                    z.y as f32 - size.1 as f32 - 8.,
                    z.y as f32 + z.height as f32 + 8.,
                ] {
                    let p = display.clamp((x, y), size);
                    if self.allows(p, size) {
                        nodes.push(p);
                    }
                }
            }
        }
        let mut distances = vec![f32::INFINITY; nodes.len()];
        let mut previous = vec![None; nodes.len()];
        let mut visited = vec![false; nodes.len()];
        distances[0] = 0.;
        for _ in 0..nodes.len() {
            let Some(i) = (0..nodes.len())
                .filter(|i| !visited[*i])
                .min_by(|a, b| distances[*a].total_cmp(&distances[*b]))
            else {
                break;
            };
            if !distances[i].is_finite() || i == 1 {
                break;
            }
            visited[i] = true;
            for j in 0..nodes.len() {
                if visited[j] {
                    continue;
                }
                let d = distances[i] + (nodes[j].0 - nodes[i].0).hypot(nodes[j].1 - nodes[i].1);
                if d < distances[j] && self.segment_clear(nodes[i], nodes[j], size) {
                    distances[j] = d;
                    previous[j] = Some(i);
                }
            }
        }
        let mut node = 1;
        while let Some(parent) = previous[node] {
            if parent == 0 {
                return nodes[node];
            }
            node = parent;
        }
        from
    }
    pub fn placement(
        &self,
        desired: (f32, f32),
        size: (u32, u32),
        display: Bounds,
    ) -> Option<(f32, f32)> {
        let desired = display.clamp(desired, size);
        if self.allows(desired, size) {
            return Some(desired);
        }
        let mut candidates = Vec::with_capacity(self.zones.len() * 4);
        for z in &self.zones {
            candidates.extend(
                [
                    (z.x as f32 - size.0 as f32 - 8., desired.1),
                    (z.x as f32 + z.width as f32 + 8., desired.1),
                    (desired.0, z.y as f32 - size.1 as f32 - 8.),
                    (desired.0, z.y as f32 + z.height as f32 + 8.),
                ]
                .map(|p| display.clamp(p, size)),
            );
        }
        candidates
            .into_iter()
            .filter(|p| self.allows(*p, size))
            .min_by(|a, b| {
                (a.0 - desired.0)
                    .hypot(a.1 - desired.1)
                    .total_cmp(&(b.0 - desired.0).hypot(b.1 - desired.1))
            })
    }
}

fn overlaps(p: (f32, f32), s: (u32, u32), z: Bounds) -> bool {
    p.0 < z.x as f32 + z.width as f32
        && p.0 + s.0 as f32 > z.x as f32
        && p.1 < z.y as f32 + z.height as f32
        && p.1 + s.1 as f32 > z.y as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zones_block_whole_pet_and_props_and_support_negative_displays() {
        let p = Preferences {
            zones: vec![Bounds {
                x: -300,
                y: 0,
                width: 200,
                height: 150,
            }],
            ..Default::default()
        };
        assert!(!p.allows((-310., 10.), (40, 40)));
        assert_eq!(p.step((-350., 10.), (-320., 10.), (40, 40)), (-350., 10.));
        let display = Bounds {
            x: -600,
            y: -200,
            width: 600,
            height: 600,
        };
        let point = p.placement((-250., 20.), (80, 64), display).unwrap();
        assert!(p.allows(point, (80, 64)));
        assert!(display.contains(point));
    }
    #[test]
    fn nooks_remember_each_display_without_unbounded_growth() {
        let mut p = Preferences::default();
        for x in 0..24 {
            p.remember(
                Bounds {
                    x: x * 100,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                (x * 100, 20),
            );
        }
        assert_eq!(p.nooks.len(), 16);
        let d = Bounds {
            x: 2300,
            y: 0,
            width: 100,
            height: 100,
        };
        p.remember(d, (2310, 30));
        assert_eq!(p.nook(d), Some((2310, 30)));
        assert!(p.valid());
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;
    #[test]
    fn walking_routes_around_zones_and_cannot_jump_through_them() {
        let p = Preferences {
            zones: vec![Bounds {
                x: 200,
                y: 100,
                width: 100,
                height: 200,
            }],
            ..Default::default()
        };
        let b = Bounds {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let from = (100., 180.);
        let target = (400., 180.);
        let size = (40, 40);
        assert!(!p.segment_clear(from, target, size));
        assert_ne!(p.step(from, target, size), target);
        let mut current = from;
        for _ in 0..8 {
            let next = p.waypoint(current, target, size, b);
            assert!(p.segment_clear(current, next, size));
            current = next;
            if current == target {
                break;
            }
        }
        assert_eq!(current, target);
        assert_eq!(p.step((210., 110.), (190., 110.), size), (190., 110.));
    }
}
