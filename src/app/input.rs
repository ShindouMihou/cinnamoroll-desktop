use super::*;
use sdl3::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::{MouseButton, MouseUtil},
};

pub(super) fn handle(
    pet: &mut Pet,
    event: Event,
    now: f64,
    window: &mut Window,
    mouse: &MouseUtil,
) -> (Option<Command>, bool) {
    match event {
        Event::Window {
            win_event: WindowEvent::CloseRequested,
            ..
        }
        | Event::Quit { .. } => return (Some(Command::Quit), false),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if pet.drag.is_none() => {
            pet.interaction.reset_stroke();
            pet.drag = Some(Drag::new((x, y), pet.position));
            mouse.capture(true);
        }
        Event::MouseMotion { x, y, .. } => {
            if let Some(position) = pet
                .drag
                .as_mut()
                .and_then(|drag| drag.motion(native::mouse_position()))
            {
                pet.position = position;
                if pet.brain.activity != Activity::Held {
                    pet.interact(Activity::Held, now);
                }
                pet.place(window);
            } else if pet.drag.is_none()
                && !pet.brain.paused
                && !pet.hidden
                && pet.play.mode != crate::play::Mode::Sleeping
            {
                let size = pet.size();
                let point = (
                    x * art::WIDTH as f32 / size.0 as f32,
                    y * art::HEIGHT as f32 / size.1 as f32,
                );
                let reaction = if pet.settings.touch_enabled
                    && !pet.play.feed_reward
                    && pet.companion.item.is_none()
                {
                    pet.companion.touch.sample(point, now)
                } else {
                    None
                };
                if let Some(activity) = reaction {
                    pet.interact(activity, now);
                } else if !(pet.settings.touch_enabled
                    && crate::companion::touch::Touch::handles(point))
                    && pet.interaction.pet(point, now)
                {
                    pet.interact(Activity::Petting, now);
                }
            }
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => {
            pet.release(now, mouse, true);
            pet.place(window);
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            ..
        } => {
            return (Some(Command::Pocket), false);
        }
        Event::KeyDown {
            keycode: Some(key),
            repeat: false,
            ..
        } => {
            let command = match key {
                Keycode::Escape => Some(Command::Quit),
                Keycode::Space => Some(Command::Pause),
                Keycode::W => Some(Command::Wander),
                Keycode::M => Some(Command::Music),
                Keycode::Y => Some(Command::ToyBall),
                Keycode::O => Some(Command::OfferTreat),
                Keycode::I => Some(Command::Activity(Activity::HighFive)),
                Keycode::U => Some(Command::Activity(Activity::Nuzzle)),
                Keycode::D => Some(Command::Activity(Activity::Dance)),
                Keycode::R => Some(Command::Activity(Activity::Read)),
                Keycode::L => Some(Command::Activity(Activity::Listen)),
                Keycode::C => Some(Command::Activity(Activity::Curious)),
                Keycode::P => Some(Command::Activity(Activity::Petting)),
                Keycode::B => Some(Command::Boop),
                Keycode::F => Some(Command::Feed),
                Keycode::T => Some(Command::Chase),
                Keycode::H => Some(Command::Peek),
                Keycode::E => Some(Command::EdgePeek),
                Keycode::N => Some(Command::Nap),
                Keycode::S => Some(Command::Settings),
                Keycode::K => Some(Command::TapDance),
                Keycode::Q => Some(Command::Pocket),
                _ => None,
            };
            return (command, false);
        }
        Event::Window {
            win_event: WindowEvent::FocusLost,
            ..
        } => {
            pet.interaction.reset_stroke();
            pet.release(now, mouse, false);
            pet.place(window);
        }
        Event::Window {
            win_event: WindowEvent::MouseLeave,
            ..
        } => pet.interaction.reset_stroke(),
        Event::Window {
            win_event: WindowEvent::Exposed,
            ..
        } => return (None, true),
        _ => {}
    }
    (None, false)
}
