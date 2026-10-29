use crate::geometry::Bounds;

pub const READY_AT: f64 = 0.6;
pub const READY_TRANSITION: f64 = 0.25;
pub const POUNCE_AT: f64 = READY_AT + READY_TRANSITION + 2.;
pub const STUMBLE_AT: f64 = POUNCE_AT + 0.6;
pub const DIZZY_AT: f64 = STUMBLE_AT + 0.6;
pub const FINISH_AT: f64 = DIZZY_AT + 2.4;

pub fn catch_activity(elapsed: f64) -> Option<crate::behavior::Activity> {
    use crate::behavior::Activity;
    if elapsed < READY_AT {
        Some(Activity::ChasePause)
    } else if elapsed < POUNCE_AT {
        Some(Activity::Stalk)
    } else if elapsed < STUMBLE_AT {
        Some(Activity::Pounce)
    } else if elapsed < DIZZY_AT {
        Some(Activity::Stumble)
    } else if elapsed < FINISH_AT {
        Some(Activity::Dizzy)
    } else {
        None
    }
}

pub fn pounce_progress(elapsed: f64) -> f32 {
    ((elapsed - POUNCE_AT) / 0.6).clamp(0., 1.) as f32
}

pub fn pounce_target(
    position: (f32, f32),
    size: (u32, u32),
    pointer: (f32, f32),
    bounds: Bounds,
) -> (f32, f32) {
    let center = (
        position.0 + size.0 as f32 * 0.5,
        position.1 + size.1 as f32 * 0.6,
    );
    let delta = (pointer.0 - center.0, pointer.1 - center.1);
    let distance = delta.0.hypot(delta.1);
    if distance < 0.1 {
        return position;
    }
    let travel = (distance - size.0 as f32 * 0.36 - 12.).max(0.);
    bounds.clamp(
        (
            position.0 + delta.0 / distance * travel,
            position.1 + delta.1 / distance * travel,
        ),
        size,
    )
}

#[derive(Default)]
pub struct ChaseMotion {
    pub velocity: (f32, f32),
}

impl ChaseMotion {
    pub fn step(
        &mut self,
        position: (f32, f32),
        size: (u32, u32),
        pointer: (f32, f32),
        bounds: Bounds,
        dt: f32,
    ) -> (f32, f32) {
        let dt = dt.clamp(0.001, 0.1);
        let ideal = chase_step(position, size, pointer, bounds, dt);
        let blend = 1. - (-6. * dt).exp();
        self.velocity.0 += ((ideal.0 - position.0) / dt - self.velocity.0) * blend;
        self.velocity.1 += ((ideal.1 - position.1) / dt - self.velocity.1) * blend;
        if self.velocity.0.hypot(self.velocity.1) < 1. {
            self.velocity = (0., 0.);
        }
        bounds.clamp(
            (
                position.0 + self.velocity.0 * dt,
                position.1 + self.velocity.1 * dt,
            ),
            size,
        )
    }
}

pub fn chase_step(
    position: (f32, f32),
    size: (u32, u32),
    pointer: (f32, f32),
    bounds: Bounds,
    dt: f32,
) -> (f32, f32) {
    let center = (
        position.0 + size.0 as f32 * 0.5,
        position.1 + size.1 as f32 * 0.6,
    );
    let delta = (pointer.0 - center.0, pointer.1 - center.1);
    let distance = delta.0.hypot(delta.1);
    let gap = size.0 as f32 * 0.4 + 18.;
    if distance <= gap {
        return position;
    }
    let step = (distance - gap).min((70_f32.min((distance - gap) * 2.)) * dt.clamp(0., 0.1));
    bounds.clamp(
        (
            position.0 + delta.0 / distance * step,
            position.1 + delta.1 / distance * step,
        ),
        size,
    )
}
