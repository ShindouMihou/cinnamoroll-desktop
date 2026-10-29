use super::*;

pub(super) struct Pet {
    pub(super) companion: crate::companion::Companion,
    pub(super) settings: Settings,
    pub(super) brain: Brain,
    pub(super) position: (f32, f32),
    pub(super) monitors: Vec<Bounds>,
    pub(super) drag: Option<Drag>,
    pub(super) hidden: bool,
    pub(super) paused_at: Option<f64>,
    pub(super) dirty: bool,
    pub(super) interaction: crate::interaction::Interaction,
    pub(super) gaze: usize,
    pub(super) play: crate::play::PlayState,
    pub(super) next_chase: f64,
}

impl Pet {
    pub(super) fn start(&mut self, activity: Activity, now: f64) {
        self.brain.start(activity, now);
        if self.brain.paused {
            self.paused_at = Some(now);
        }
    }
    pub(super) fn size(&self) -> (u32, u32) {
        (
            art::WIDTH * self.settings.size / 100,
            art::HEIGHT * self.settings.size / 100,
        )
    }

    pub(super) fn bounds(&self) -> Bounds {
        let center = (
            self.position.0 + self.size().0 as f32 / 2.,
            self.position.1 + self.size().1 as f32 / 2.,
        );
        self.monitors
            .iter()
            .find(|bounds| bounds.contains(center))
            .copied()
            .unwrap_or(self.monitors[0])
    }

    pub(super) fn home(&mut self) {
        let bounds = self.monitors[0];
        self.position = bounds.clamp(
            (
                bounds.x as f32 + bounds.width as f32 - self.size().0 as f32 - 40.,
                bounds.y as f32 + bounds.height as f32 - self.size().1 as f32 - 12.,
            ),
            self.size(),
        );
    }

    pub(super) fn place(&mut self, window: &mut Window) {
        window.set_position(
            WindowPos::Positioned(self.position.0.round() as i32),
            WindowPos::Positioned(self.position.1.round() as i32),
        );
    }

    pub(super) fn release(&mut self, now: f64, mouse: &sdl3::mouse::MouseUtil, clicked: bool) {
        if let Some(drag) = self.drag.take() {
            self.companion.homecoming.reset();
            mouse.capture(false);
            if !drag.moved && clicked && matches!(self.play.mode, crate::play::Mode::EdgePeek(_)) {
                self.begin_edge_peek(now, true);
                return;
            }
            self.position = self.bounds().clamp(self.position, self.size());
            if drag.moved {
                self.companion.move_undo = Some(crate::companion::comfort::MoveSnapshot {
                    pet: drag.origin,
                    corner: self.settings.corner_position,
                });
                self.start(Activity::Land, now);
            } else if clicked && !self.brain.paused {
                let size = self.size();
                let point = (
                    drag.offset.0 * art::WIDTH as f32 / size.0 as f32,
                    drag.offset.1 * art::HEIGHT as f32 / size.1 as f32,
                );
                let reaction = if self.play.mode == crate::play::Mode::Sleeping {
                    Activity::Greet
                } else if crate::companion::touch::is_ear(point) {
                    Activity::EarWiggle
                } else if crate::interaction::is_nose(point) {
                    if matches!(
                        self.interaction.click(now),
                        Activity::Petting | Activity::Excited
                    ) {
                        Activity::Sneeze
                    } else {
                        Activity::Boop
                    }
                } else {
                    self.interaction.click(now)
                };
                self.interact(reaction, now);
            }
            self.dirty = true;
        }
    }

    pub(super) fn hold_for_menu(&mut self, elapsed: f64) {
        if self.brain.paused {
            return;
        }
        self.companion.homecoming.hold(elapsed);
        self.brain.started += elapsed;
        self.brain.until += elapsed;
        self.play.resume(elapsed);
        self.companion.next_surprise += elapsed;
        self.companion.next_item += elapsed;
        if let Some(due) = self.companion.chase_return.as_mut() {
            *due += elapsed;
        }
        if let Some(since) = self.companion.sniff_since.as_mut() {
            *since += elapsed;
        }
        if let Some((due, _)) = self.companion.after_action.as_mut() {
            *due += elapsed;
        }
        if let Some(item) = self.companion.item.as_mut() {
            item.expires += elapsed;
            item.last_pop += elapsed;
            item.reaction_until += elapsed;
            if let Some(since) = item.flight_since.as_mut() {
                *since += elapsed;
            }
            if let Some(cups) = item.cups.as_mut() {
                cups.started += elapsed;
            }
        }
    }

    pub(super) fn save(&mut self, tray: &Tray) -> bool {
        let position = if let crate::play::Mode::EdgePeek(visit) = self.play.mode {
            visit.origin
        } else {
            self.position
        };
        self.settings.extras.study_bookmark = self.companion.study.bookmark();
        self.settings.position = Some((position.0.round() as i32, position.1.round() as i32));
        if let Some(home) = self.settings.corner_position
            && let Some(display) = self
                .monitors
                .iter()
                .find(|d| d.contains((home.0 as f32, home.1 as f32)))
                .copied()
        {
            self.settings.extras.comfort.remember(display, home);
        }
        let result = settings::save(&self.settings);
        if let Err(error) = &result {
            eprintln!("Could not save preferences: {error}");
            tray.status("Couldn't save preferences; see terminal log");
        }
        self.dirty = false;
        result.is_ok()
    }
}
