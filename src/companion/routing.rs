use super::comfort::Preferences;
use crate::geometry::Bounds;

#[derive(Default)]
pub struct Route {
    key: Option<Key>,
    waypoint: (f32, f32),
    blocked: bool,
}

#[derive(PartialEq)]
struct Key {
    target: (f32, f32),
    size: (u32, u32),
    display: Bounds,
    zones: Vec<Bounds>,
}

impl Route {
    pub fn next(
        &mut self,
        prefs: &Preferences,
        from: (f32, f32),
        target: (f32, f32),
        size: (u32, u32),
        display: Bounds,
    ) -> (f32, f32) {
        if prefs.zones.is_empty() {
            return target;
        }
        let valid = self.key.as_ref().is_some_and(|k| {
            k.target == target && k.size == size && k.display == display && k.zones == prefs.zones
        });
        if !valid
            || (!self.blocked && (from.0 - self.waypoint.0).hypot(from.1 - self.waypoint.1) < 5.)
        {
            self.waypoint = prefs.waypoint(from, target, size, display);
            self.blocked = self.waypoint == from && from != target;
            self.key = Some(Key {
                target,
                size,
                display,
                zones: prefs.zones.clone(),
            });
        }
        self.waypoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_routes_advance_at_waypoints_and_react_to_zone_changes() {
        let mut prefs = Preferences::default();
        prefs.zones.push(Bounds {
            x: 200,
            y: 100,
            width: 100,
            height: 200,
        });
        let b = Bounds {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let mut route = Route::default();
        let first = route.next(&prefs, (100., 180.), (400., 180.), (40, 40), b);
        assert_ne!(first, (400., 180.));
        assert_eq!(
            route.next(&prefs, (101., 180.), (400., 180.), (40, 40), b),
            first
        );
        prefs.zones.clear();
        assert_eq!(
            route.next(&prefs, first, (400., 180.), (40, 40), b),
            (400., 180.)
        );
    }
}
