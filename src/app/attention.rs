use super::*;

impl Pet {
    pub(super) fn attend_to_pointer(&mut self, now: f64) {
        if self.quiet_busy()
            || self.hidden
            || self.brain.paused
            || self.drag.is_some()
            || self.settings.reduced_motion
            || self.play.busy()
        {
            return;
        }
        let pointer = native::mouse_position();
        let size = self.size();
        let head = (
            self.position.0 + size.0 as f32 * 0.5,
            self.position.1 + size.1 as f32 * 0.55,
        );
        if self.interaction.notice_pointer(pointer, head, now)
            && self.brain.attention(Activity::Curious, now)
            && let Some(direction) = crate::interaction::look_direction(head, pointer)
        {
            self.gaze = direction;
        }
    }

    pub(super) fn attend_to_activity(&mut self, idle: Option<f64>, now: f64) {
        if self.quiet_busy()
            || self.hidden
            || self.brain.paused
            || self.drag.is_some()
            || self.settings.reduced_motion
            || self.play.busy()
        {
            return;
        }
        if let Some(activity) = self.interaction.activity(idle, now) {
            self.brain.attention(activity, now);
        }
    }
}
