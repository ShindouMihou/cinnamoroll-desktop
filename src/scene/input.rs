use super::*;

impl Scene {
    pub fn handle(
        &mut self,
        event: &Event,
        c: &mut Companion,
        s: &mut Settings,
        now: f64,
        pointer: (f32, f32),
        mouse: &MouseUtil,
    ) -> (Option<Activity>, bool) {
        if self
            .hint
            .as_ref()
            .is_some_and(|hint| event.get_window_id() == Some(hint.id()))
        {
            return (None, false);
        }
        let corner = self
            .corner
            .as_ref()
            .is_some_and(|w| event.get_window_id() == Some(w.id()));
        match event {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                if corner {
                    let Some(window) = self.corner.as_ref() else {
                        return (None, false);
                    };
                    self.corner_drag = Some(CornerDrag::new(
                        pointer,
                        window.canvas.window().position(),
                        self.settled_offset,
                    ));
                    mouse.capture(true);
                } else if let Some(item) = c.item.as_mut() {
                    if item.kind == Kind::Breathing {
                        return (None, false);
                    }
                    if item.kind == Kind::HideCups {
                        item.expires = now + 120.;
                        let size = item.kind.size();
                        let chosen = item.cups.as_mut().is_some_and(|game| {
                            game.hit((*x, *y), size)
                                .is_some_and(|index| game.click(index, now))
                        });
                        return (chosen.then_some(Activity::CupPeek), false);
                    } else if item.kind == Kind::Parcel {
                        item.stage = 1;
                        item.activated = true;
                        item.last_pop = now;
                    } else if item.kind == Kind::Palette {
                        s.extras.palette.rotate_left(1);
                        item.activated = true;
                        return (Some(Activity::Curious), true);
                    } else if item.kind == Kind::Bubbles {
                        if !item.pop_at((*x, *y)) {
                            return (None, false);
                        }
                        let empty = item.bubbles == 0;
                        item.last_pop = now;
                        if empty {
                            c.item = None;
                            c.after_action = Some((now + 2., Activity::Happy));
                        }
                        return (Some(Activity::BubblePop), false);
                    } else if !item.carried {
                        self.item_offset =
                            (pointer.0 - item.position.0, pointer.1 - item.position.1);
                        item.grab(now);
                        mouse.capture(true);
                    }
                }
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Right,
                ..
            } => {
                if corner {
                    c.resting = true;
                    c.study.paused = c.study.active();
                    return (Some(Activity::CozyNap), false);
                }
                c.item = None;
                c.sniff_since = None;
                mouse.capture(false);
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                mouse.capture(false);
                if let Some(drag) = self.corner_drag.take() {
                    let moved = drag.moved;
                    if !moved {
                        c.resting = true;
                        c.study.paused = c.study.active();
                    }
                    return (if moved { None } else { Some(Activity::CozyNap) }, moved);
                }
                if let Some(item) = c.item.as_mut() {
                    item.release_at(now);
                }
            }
            Event::Window {
                win_event: WindowEvent::FocusLost,
                ..
            } => {
                let save = self.corner_drag.take().is_some();
                if let Some(item) = c.item.as_mut() {
                    item.release_at(now);
                }
                mouse.capture(false);
                return (None, save);
            }
            Event::Window {
                win_event: WindowEvent::Exposed | WindowEvent::PixelSizeChanged(_, _),
                ..
            } => {
                if corner {
                    if let Some(w) = self.corner.as_mut() {
                        w.invalidate();
                    }
                } else if let Some(w) = self.item.as_mut() {
                    w.invalidate();
                }
            }
            Event::Window {
                win_event: WindowEvent::CloseRequested,
                ..
            } => {
                if corner {
                    s.cozy_corner = false;
                    c.resting = false;
                    return (None, true);
                } else {
                    c.item = None;
                }
            }
            Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                ..
            } => {
                c.clear_play();
                return (Some(Activity::Idle), false);
            }
            _ => {}
        }
        (None, false)
    }
}
