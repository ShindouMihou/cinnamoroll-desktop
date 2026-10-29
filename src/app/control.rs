use super::*;

pub(super) fn execute(
    pet: &mut Pet,
    command: Command,
    now: f64,
    window: &mut Window,
    monitors: &Monitors,
    mouse: &sdl3::mouse::MouseUtil,
    tray: &Tray,
) -> Result<bool, String> {
    if pet.drag.is_none() && pet.comfort_command(&command, now, window, tray) {
        return Ok(false);
    }
    if pet.drag.is_none() && pet.extra_command(&command, now, tray) {
        return Ok(false);
    }
    if pet.drag.is_none() && pet.companion_command(command.clone(), now) {
        return Ok(false);
    }
    let mut quit = false;
    match command {
        Command::Quit => quit = true,
        Command::OpenAccessibility => {
            let url = if cfg!(target_os = "macos") {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            } else {
                "ms-settings:easeofaccess"
            };
            if let Err(error) = sdl3::url::open_url(url) {
                eprintln!("Could not open accessibility settings: {error}");
                tray.status("Open your system’s Accessibility settings manually");
            }
        }
        Command::Activity(activity) if pet.drag.is_none() => pet.interact(activity, now),
        Command::Feed if pet.drag.is_none() => pet.feed(now),
        Command::Chase if pet.drag.is_none() => pet.toggle_chase(now),
        Command::Peek if pet.drag.is_none() => pet.peek(now),
        Command::EdgePeek if pet.drag.is_none() => pet.begin_edge_peek(now, true),
        Command::Nap if pet.drag.is_none() => pet.toggle_nap(now),
        Command::Boop if pet.drag.is_none() => pet.interact(Activity::Boop, now),
        Command::AdjustQuiet(delta) => {
            pet.settings.idle_pause_seconds = ((pet.settings.idle_pause_seconds as i32)
                .saturating_add(delta))
            .clamp(15, 300) as u32;
            pet.brain.quiet_seconds = f64::from(pet.settings.idle_pause_seconds);
            if pet.brain.activity == Activity::Idle {
                pet.start(Activity::Idle, now);
            }
        }
        Command::AdjustSleep(delta) => {
            pet.settings.sleep_after_minutes = ((pet.settings.sleep_after_minutes as i32)
                .saturating_add(delta))
            .clamp(0, 60) as u32;
            if pet.settings.sleep_after_minutes == 0 && pet.play.auto_sleep {
                pet.interact(Activity::Idle, now);
            }
        }
        Command::OccasionalChase => {
            pet.settings.occasional_chase = !pet.settings.occasional_chase;
            pet.next_chase = now + 300. + fastrand::f64() * 300.;
        }
        Command::FollowTyping => {
            pet.settings.follow_typing = !pet.settings.follow_typing;
            if !pet.settings.follow_typing
                && matches!(pet.play.mode, crate::play::Mode::CaretWatch { .. })
            {
                pet.interact(Activity::Idle, now);
            }
            tray.caret_status(if pet.settings.follow_typing {
                "Following supported typing cursors when free"
            } else {
                "Typing cursor following off"
            });
        }
        Command::Pause => {
            pet.brain.paused = !pet.brain.paused;
            if pet.brain.paused {
                pet.paused_at = Some(now);
            } else if let Some(paused) = pet.paused_at.take() {
                pet.hold_for_menu(now - paused);
            }
        }
        Command::Wander => {
            if pet.play.mode.is_chasing() {
                pet.interact(Activity::Idle, now);
            }
            pet.settings.wandering = !pet.settings.wandering;
            pet.brain.wandering = pet.settings.wandering;
            if !pet.settings.wandering && pet.brain.activity.moves() {
                pet.start(Activity::Idle, now);
            }
        }
        Command::Music => {
            pet.settings.music_reactions = !pet.settings.music_reactions;
            monitors.music.enable(pet.settings.music_reactions);
            tray.music_status(if pet.settings.music_reactions {
                "Music reactions on · permission may be requested"
            } else {
                "Music reactions off"
            });
        }
        Command::ReducedMotion => {
            pet.settings.reduced_motion = !pet.settings.reduced_motion;
            if pet.settings.reduced_motion && pet.play.mode.is_chasing() {
                pet.interact(Activity::Play, now);
            }
        }
        Command::Size(size) => {
            pet.settings.size = size;
            let (width, height) = pet.size();
            window.set_size(width, height).map_err(|e| e.to_string())?;
            pet.position = pet.bounds().clamp(pet.position, pet.size());
            pet.place(window);
        }
        Command::Reset => {
            pet.interact(Activity::Idle, now);
            pet.home();
            pet.place(window);
        }
        Command::Hide => {
            pet.release(now, mouse, false);
            if matches!(
                pet.play.mode,
                crate::play::Mode::Chase { .. }
                    | crate::play::Mode::ChaseRest { .. }
                    | crate::play::Mode::ChaseCatch { .. }
                    | crate::play::Mode::Peek { .. }
                    | crate::play::Mode::CaretWatch { .. }
                    | crate::play::Mode::EdgePeek(_)
            ) {
                pet.interact(Activity::Idle, now);
            }
            pet.hidden = !pet.hidden;
            if pet.hidden {
                pet.companion.clear_play();
                mouse.capture(false);
                window.hide();
            } else {
                window.show();
            }
        }
        _ => {}
    }

    Ok(quit)
}
