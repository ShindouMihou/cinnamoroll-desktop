mod attention;
mod catch;
mod chase;
mod companion;
mod companion_control;
mod control;
mod drag;
mod edge;
mod extra_control;
mod extra_items;
mod input;
mod play;
mod playroom;
mod signals;
mod state;
mod surprise_items;
mod toy_games;
mod window;
use drag::Drag;
use state::Pet;
use window::{apply_shape, create_texture, instance_lock};

use crate::{
    art::{self, Animation},
    behavior::{Activity, Brain},
    command::{Command, Signal},
    geometry::Bounds,
    music::Monitor,
    native::{self, Tray},
    settings::{self, Settings},
};
use sdl3::{
    pixels::{Color, PixelFormat},
    surface::Surface,
    video::{Window, WindowFlags, WindowPos},
};
use std::{
    fs::{File, OpenOptions},
    sync::mpsc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Monitors {
    music: Monitor,
    caret: crate::caret::Monitor,
}

pub fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--version") {
        println!("Cinnamoroll {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if arguments
        .iter()
        .any(|argument| argument == "--update-config")
    {
        println!(
            "{}",
            serde_json::json!({"version": env!("CARGO_PKG_VERSION"), "enabled": crate::updates::connected(), "repository": option_env!("CINNAMOROLL_UPDATE_REPOSITORY"), "public_key": option_env!("CINNAMOROLL_UPDATE_PUBLIC_KEY")})
        );
        return Ok(());
    }
    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--verify-update")
    {
        return crate::updates::verify_release(std::path::Path::new(
            arguments
                .get(index + 1)
                .ok_or("Missing signed release archive")?,
        ));
    }
    if let Some(result) = crate::updates::apply(&arguments) {
        return result;
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--play-gallery") {
        return crate::scene::play_preview::gallery(std::path::Path::new(
            arguments
                .get(index + 1)
                .ok_or("Missing play preview folder")?,
        ));
    }
    if let Some(index) = arguments.iter().position(|a| a == "--prepare-play-assets") {
        return art::prepare_play_assets(std::path::Path::new(
            arguments
                .get(index + 1)
                .ok_or("Missing play asset folder")?,
        ));
    }
    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--prepare-bed")
    {
        return art::prepare_bed(
            std::path::Path::new(arguments.get(index + 1).ok_or("Missing bed source")?),
            std::path::Path::new(arguments.get(index + 2).ok_or("Missing bed destination")?),
        );
    }
    if arguments
        .iter()
        .any(|argument| argument == "--stacking-check")
    {
        return native::stacking::smoke();
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--prepare-strips") {
        return art::prepare_strips(
            std::path::Path::new(arguments.get(index + 1).ok_or("Missing source directory")?),
            std::path::Path::new(arguments.get(index + 2).ok_or("Missing output directory")?),
        );
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--icon") {
        return art::icon(std::path::Path::new(
            arguments.get(index + 1).ok_or("Usage: --icon PNG_PATH")?,
        ));
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--pocket-gallery") {
        return crate::pocket::preview::gallery(std::path::Path::new(
            arguments
                .get(index + 1)
                .ok_or("Missing pocket preview directory")?,
        ));
    }
    if arguments
        .iter()
        .any(|argument| argument == "--startup-status")
    {
        println!(
            "Launch at login: {:?}; {}",
            native::startup::status()?,
            native::startup::diagnostic()
        );
        return Ok(());
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--settings-gallery") {
        let path = arguments
            .get(index + 1)
            .ok_or("Usage: --settings-gallery DIRECTORY")?;
        return crate::preferences::preview::gallery(std::path::Path::new(path));
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--gallery") {
        let path = arguments
            .get(index + 1)
            .ok_or("Usage: --gallery DIRECTORY")?;
        return art::gallery(std::path::Path::new(path));
    }
    if arguments.iter().any(|arg| arg == "--help") {
        println!(
            "Cinnamoroll desktop\n  --gallery DIRECTORY  Export all activity illustrations\n  --settings-gallery DIRECTORY  Export control panel previews\n  --smoke-seconds N    Quit after N seconds\n  --activity NAME      Preview one activity\n\nPet: click to pet, drag to move, right-click for surprise.\nMenu bar / system tray: settings, activities, hide, reset, quit.\nKeys while focused: Space pause, W wander, M music, D dance, R read, L listen, C curious, P petting, Esc quit."
        );
        return Ok(());
    }
    let smoke = arguments
        .iter()
        .position(|arg| arg == "--smoke-seconds")
        .map(|index| {
            arguments
                .get(index + 1)
                .ok_or("Missing smoke duration")?
                .parse::<f64>()
                .map_err(|_| "Invalid smoke duration")
        })
        .transpose()?;
    if smoke.is_some_and(|value| !value.is_finite() || value <= 0.) {
        return Err("Smoke duration must be positive".into());
    }
    let initial = arguments
        .iter()
        .position(|arg| arg == "--activity")
        .map(|index| {
            let name = arguments.get(index + 1).ok_or("Missing activity name")?;
            serde_json::from_value::<Activity>(serde_json::Value::String(name.clone()))
                .map_err(|_| "Unknown activity")
        })
        .transpose()?;
    let _instance = instance_lock()?;
    if arguments.iter().any(|arg| arg == "--trace-input") {
        sdl3::hint::set("SDL_EVENT_LOGGING", "2");
    }
    sdl3::hint::set("SDL_MAC_BACKGROUND_APP", "1");
    sdl3::hint::set("SDL_QUIT_ON_LAST_WINDOW_CLOSE", "0");
    sdl3::hint::set("SDL_WINDOW_ACTIVATE_WHEN_SHOWN", "0");
    let sdl = sdl3::init().map_err(|e| e.to_string())?;
    let video = sdl.video().map_err(|e| e.to_string())?;
    let monitors = native::displays(&video)?;
    if monitors.is_empty() {
        return Err("No connected display was found".into());
    }
    let mut warnings = Vec::new();
    let settings = settings::load().unwrap_or_else(|error| {
        warnings.push(error);
        Settings::default()
    });
    let rules = settings::load_rules().unwrap_or_else(|error| {
        warnings.push(error);
        crate::behavior::default_rules()
    });
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos() as u64;
    let mut brain = Brain::new(rules, seed);
    brain.wandering = settings.wandering;
    brain.quiet_seconds = f64::from(settings.idle_pause_seconds);
    if let Some(activity) = initial {
        brain.start(activity, 0.);
    }
    let mut pet = Pet {
        companion: crate::companion::Companion::default(),
        position: settings
            .position
            .map(|(x, y)| (x as f32, y as f32))
            .unwrap_or_default(),
        settings,
        brain,
        monitors,
        drag: None,
        hidden: false,
        paused_at: None,
        dirty: false,
        interaction: crate::interaction::Interaction::default(),
        gaze: 0,
        play: crate::play::PlayState::default(),
        next_chase: 180. + fastrand::f64() * 120.,
    };
    if pet.settings.position.is_none() || !pet.monitors.iter().any(|b| b.contains(pet.position)) {
        pet.home();
    }
    pet.position = pet.bounds().clamp(pet.position, pet.size());
    let size = pet.size();
    let mut scene = crate::scene::Scene::new(&video);
    let window = video
        .window("Cinnamoroll", size.0, size.1)
        .set_flags(
            WindowFlags::BORDERLESS
                | WindowFlags::TRANSPARENT
                | WindowFlags::ALWAYS_ON_TOP
                | WindowFlags::UTILITY
                | WindowFlags::HIGH_PIXEL_DENSITY
                | WindowFlags::HIDDEN,
        )
        .position(pet.position.0 as i32, pet.position.1 as i32)
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = sdl3::render::create_renderer(window, None).map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    let creator = canvas.texture_creator();
    let mut walk_cycle = art::walk_cycle::WalkCycle::default();
    let mut animation = Animation::new(
        pet.brain.activity,
        pet.brain.facing,
        canvas.window().size_in_pixels(),
    )?
    .with_accessory(pet.settings.extras.accessory)?;
    let mut texture = create_texture(&creator, animation.size())?;
    let (commands_tx, commands) = mpsc::channel();
    let (signals_tx, signals) = mpsc::channel();
    let tray = Tray::new(commands_tx.clone())?;
    let monitors = Monitors {
        music: Monitor::new(signals_tx.clone()),
        caret: crate::caret::Monitor::new(signals_tx),
    };
    monitors.music.enable(pet.settings.music_reactions);
    if pet.settings.follow_typing {
        tray.caret_status("Following supported typing cursors when free");
    }
    if pet.settings.music_reactions {
        tray.music_status("Watching supported music players");
    }
    tray.sync(&pet.settings, false, false);
    for warning in &warnings {
        eprintln!("Preferences warning: {warning}");
    }
    if !warnings.is_empty() {
        tray.status("Some preferences were invalid; using defaults");
    }
    apply_shape(canvas.window())?;
    canvas.window_mut().show();
    let mut events = sdl.event_pump().map_err(|e| e.to_string())?;
    let start = Instant::now();
    let mut last = start;
    let mut last_frame: Option<(Activity, usize, bool)> = None;
    let mut pickup: Option<art::Pickup> = None;
    let mut last_draw = start - Duration::from_secs(1);
    let mut last_display_check = start;
    let mut last_attention_check = start;
    let mut last_activity = pet.brain.activity;
    let mut away = crate::system::AwayDetector::default();
    let mut low_battery = false;
    let mut tray_checked = false;
    let mut quit = false;
    let mut mouse_captured = false;
    let mut panel: Option<crate::preferences::Panel> = None;
    let mut pocket: Option<crate::pocket::Panel> = None;
    let mut updates = crate::updates::Service::default();
    while !quit {
        let now = start.elapsed().as_secs_f64();
        updates.poll(pet.settings.automatic_update_checks);
        if let Some(panel) = panel.as_mut() {
            panel.update_state(&updates.state);
        }
        if !tray_checked && now >= 1. {
            tray_checked = true;
            if arguments.iter().any(|argument| argument == "--tray-check") {
                eprintln!("{}", tray.diagnostic());
                if !tray.visibility_check() {
                    return Err("Native menu-bar item is not visible".into());
                }
            }
        }
        if smoke.is_some_and(|limit| now >= limit) {
            break;
        }
        let dt = last.elapsed().as_secs_f32();
        last = Instant::now();
        if dt > 15.
            && !pet.brain.paused
            && pet.drag.is_none()
            && !pet.play.busy()
            && !pet.quiet_busy()
        {
            pet.start(Activity::Greet, now);
        }
        let event = events.wait_event_timeout(Duration::from_millis(
            if pet.drag.is_some()
                || scene.dragging_corner()
                || pet.companion.item.as_ref().is_some_and(|item| item.held)
            {
                16
            } else if pet.hidden || pet.brain.paused {
                200
            } else if pet.companion.item.as_ref().is_some_and(|item| {
                matches!(
                    item.kind,
                    crate::companion::world::Kind::Ball
                        | crate::companion::world::Kind::Plane
                        | crate::companion::world::Kind::Bubbles
                        | crate::companion::world::Kind::Breathing
                ) && item.activated
            }) || (!pet.settings.reduced_motion
                && pet
                    .companion
                    .item
                    .as_ref()
                    .and_then(|item| item.cups.as_ref())
                    .is_some_and(|game| game.animating(now)))
                || pet.brain.activity.moves()
                || pet.brain.activity == Activity::Excited
                || matches!(pet.play.mode, crate::play::Mode::ChaseCatch { .. })
            {
                33
            } else {
                100
            },
        ));
        for event in event.into_iter().chain(events.poll_iter()) {
            if let Some(menu) = pocket.as_mut()
                && menu.owns(&event)
            {
                let (command, close) = menu.handle(&event);
                if let Some(command) = command {
                    let _ = commands_tx.send(command);
                }
                if close {
                    pocket = None;
                }
                continue;
            }
            if scene.owns(&event) {
                if let sdl3::event::Event::KeyDown {
                    keycode: Some(key), ..
                } = &event
                {
                    if *key == sdl3::keyboard::Keycode::S {
                        let _ = commands_tx.send(Command::Settings);
                    } else if *key == sdl3::keyboard::Keycode::Q {
                        let _ = commands_tx.send(Command::Pocket);
                    }
                }
                let (reaction, save) = scene.handle(
                    &event,
                    &mut pet.companion,
                    &mut pet.settings,
                    now,
                    native::mouse_position(),
                    &sdl.mouse(),
                );
                if let Some(activity) = reaction
                    && (pet.brain.activity != activity || now >= pet.brain.until)
                {
                    pet.react(activity, now);
                }
                pet.dirty |= save;
                continue;
            }
            if let Some(settings_panel) = panel.as_mut()
                && settings_panel.owns(&event)
            {
                let (command, close) = settings_panel.handle(&event);
                if let Some(command) = command {
                    let _ = commands_tx.send(command);
                }
                if close {
                    panel = None;
                }
                continue;
            }
            let (command, redraw) =
                input::handle(&mut pet, event, now, canvas.window_mut(), &sdl.mouse());
            if let Some(command) = command {
                let _ = commands_tx.send(command);
            }
            if redraw {
                last_frame = None;
            }
        }
        for signal in signals.try_iter() {
            signals::handle(&mut pet, signal, now, &tray, low_battery);
        }
        for command in commands.try_iter() {
            if matches!(command, Command::Pocket) {
                if pocket.is_some() {
                    pocket = None;
                } else {
                    match crate::pocket::Panel::new(
                        &video,
                        Bounds {
                            x: pet.position.0.round() as i32,
                            y: pet.position.1.round() as i32,
                            width: pet.size().0,
                            height: pet.size().1,
                        },
                        pet.bounds(),
                        &pet.settings,
                        pet.brain.paused,
                        pet.companion.study.view(),
                    ) {
                        Ok(menu) => pocket = Some(menu),
                        Err(error) => {
                            eprintln!("Pet pocket: {error}");
                            tray.status("Press S to open controls");
                        }
                    }
                }
                continue;
            }
            if matches!(
                command,
                Command::Settings | Command::SessionPlan | Command::CreativeTools
            ) {
                let page = match command {
                    Command::SessionPlan => Some(8),
                    Command::CreativeTools => Some(9),
                    _ => None,
                };
                if let Some(panel) = panel.as_mut() {
                    panel.raise();
                } else {
                    match crate::preferences::Panel::new(&video) {
                        Ok(created) => panel = Some(created),
                        Err(error) => {
                            eprintln!("Settings: {error}");
                            tray.status("Couldn’t open settings");
                        }
                    }
                }
                if let Some(page) = page
                    && let Some(panel) = panel.as_mut()
                {
                    panel.open_page(page);
                }
                continue;
            }
            if matches!(command, Command::LaunchAtLogin | Command::OpenLoginSettings) {
                if let Some(panel) = panel.as_mut() {
                    panel.startup_command(matches!(command, Command::OpenLoginSettings));
                }
                continue;
            }
            match command {
                Command::CheckUpdates => {
                    updates.check();
                    continue;
                }
                Command::DownloadUpdate => {
                    updates.download();
                    continue;
                }
                Command::InstallUpdate => {
                    quit |= updates.install(false);
                    continue;
                }
                Command::UpdateWhileAway => {
                    pet.settings.update_while_away = !pet.settings.update_while_away;
                    pet.dirty = true;
                    continue;
                }
                Command::AutomaticUpdateChecks => {
                    pet.settings.automatic_update_checks = !pet.settings.automatic_update_checks;
                    pet.dirty = true;
                    continue;
                }
                _ => {}
            }
            quit |= control::execute(
                &mut pet,
                command,
                now,
                canvas.window_mut(),
                &monitors,
                &sdl.mouse(),
                &tray,
            )?;
            pet.dirty = true;
            tray.sync(&pet.settings, pet.brain.paused, pet.hidden);
            last_frame = None;
        }
        pet.update_study(now);
        let capture = pet.drag.is_some()
            || scene.dragging_corner()
            || pet.companion.item.as_ref().is_some_and(|item| item.held);
        if capture != mouse_captured {
            sdl.mouse().capture(capture);
            mouse_captured = capture;
        }
        if last_display_check.elapsed() > Duration::from_secs(5) {
            low_battery = crate::system::low_battery();
            let idle = crate::system::idle_seconds();
            quit |= updates.while_away(
                idle,
                pet.settings.update_while_away,
                !pet.companion.study.active()
                    && pet.drag.is_none()
                    && pocket.is_none()
                    && panel.is_none(),
            );
            let woke = pet.idle_sleep(idle, now);
            pet.attend_to_activity(idle, now);
            if away.update(idle)
                && !woke
                && !pet.brain.paused
                && pet.drag.is_none()
                && !pet.play.busy()
                && !pet.quiet_busy()
            {
                pet.start(Activity::Greet, now);
            }
            if let Ok(monitors) = native::displays(&video)
                && !monitors.is_empty()
            {
                pet.monitors = monitors;
                if pet.drag.is_none() && !matches!(pet.play.mode, crate::play::Mode::EdgePeek(_)) {
                    pet.position = pet.bounds().clamp(pet.position, pet.size());
                    pet.place(canvas.window_mut());
                }
            }
            last_display_check = Instant::now();
        }
        if last_attention_check.elapsed() >= Duration::from_millis(200) {
            if !scene.dragging_corner() && pocket.is_none() {
                pet.touch_pointer(now);
                pet.attend_to_pointer(now);
                pet.maybe_chase(now, low_battery);
                pet.maybe_item(now, low_battery);
                pet.maybe_surprise(now, low_battery);
            }
            last_attention_check = Instant::now();
        }
        if let Some(drag) = pet.drag.as_mut()
            && drag.moved
            && let Some(position) = drag.motion(native::mouse_position())
        {
            pet.position = position;
            pet.place(canvas.window_mut());
        }
        let size = pet.size();
        let bounds = pet.bounds();
        if let Some(position) = scene.drag(
            &mut pet.companion,
            &mut pet.settings,
            native::mouse_position(),
            size,
            bounds,
            now,
        ) {
            pet.position = position;
            pet.place(canvas.window_mut());
        }
        if pet.drag.is_none() && !pet.hidden && !scene.carrying_pet() {
            if pocket.is_some() {
                pet.hold_for_menu(f64::from(dt));
            } else {
                pet.advance(now, dt, canvas.window_mut(), low_battery);
            }
        }
        if pet.brain.activity != last_activity {
            tray.status(pet.brain.activity.label());
            last_activity = pet.brain.activity;
        }
        if pet.dirty && pet.drag.is_none() {
            let saved = pet.save(&tray);
            if let Some(panel) = panel.as_mut() {
                panel.saved(saved);
            }
        }
        monitors
            .music
            .enable(pet.settings.music_reactions && !pet.quiet_busy() && !pet.hidden);
        monitors.caret.enable(
            pet.settings.follow_typing
                && !pet.settings.reduced_motion
                && !pet.hidden
                && !pet.brain.paused
                && !low_battery
                && pet.play.accepts_caret()
                && !pet.quiet_busy(),
        );
        if let Some(settings_panel) = panel.as_mut()
            && let Err(error) = settings_panel.render(
                &pet.settings,
                pet.brain.paused,
                pet.brain.activity,
                pet.companion.study.view(),
            )
        {
            eprintln!("Settings drawing: {error}");
            tray.status("Couldn’t draw settings");
            panel = None;
        }
        if let Some(menu) = pocket.as_mut()
            && let Err(error) = menu.render()
        {
            eprintln!("Pet pocket: {error}");
            pocket = None;
        }
        if let Err(error) = scene.render(crate::scene::Frame {
            settings: &pet.settings,
            companion: &pet.companion,
            pet: pet.position,
            size: pet.size(),
            bounds: pet.bounds(),
            hidden: pet.hidden,
            now: pet.paused_at.unwrap_or(now),
            pet_window: canvas.window(),
        }) {
            scene.reset();
            eprintln!("Playroom: {error}");
            tray.status("Couldn’t show the playroom. The pet is still available.");
            pet.settings.cozy_corner = false;
            pet.companion.clear_play();
            sdl.mouse().capture(false);
        }

        let walk_elapsed = walk_cycle.update(
            pet.position,
            pet.size().0,
            pet.brain.activity == Activity::Walk
                && !pet.brain.paused
                && pet.drag.is_none()
                && !pet.hidden,
        );
        if pet.hidden {
            continue;
        }
        if pet.drag.is_none() {
            pickup = None;
        }
        let pixels = canvas.window().size_in_pixels();
        if let Some(drag) = pet.drag.as_ref()
            && drag.moved
            && pickup.is_none()
        {
            pickup = Some(art::Pickup::new(pixels, drag.offset, pet.size())?);
        }
        if pet.drag.as_ref().is_some_and(|drag| drag.moved)
            && let Some(held) = pickup.as_mut()
            && held.size() == pixels
        {
            held.draw(
                &mut canvas,
                &mut texture,
                pet.position,
                dt,
                pet.settings.reduced_motion,
            )?;
            last_frame = None;
            continue;
        }
        let resized = animation.size() != pixels;
        if animation.activity() != pet.brain.activity
            || !animation.matches_direction(pet.brain.facing)
            || resized
            || animation.accessory() != pet.settings.extras.accessory
        {
            let was_peeking = animation.activity() == Activity::EdgePeek;
            animation = Animation::new(pet.brain.activity, pet.brain.facing, pixels)?
                .with_accessory(pet.settings.extras.accessory)?;
            if pet.brain.activity == Activity::EdgePeek {
                window::apply_frame_shape(canvas.window(), animation.frame(0., true).clone())?;
            } else if was_peeking && !resized {
                apply_shape(canvas.window())?;
            }
            if resized {
                texture = create_texture(&creator, pixels)?;
                if pet.brain.activity == Activity::EdgePeek {
                    window::apply_frame_shape(canvas.window(), animation.frame(0., true).clone())?;
                } else {
                    apply_shape(canvas.window())?;
                }
            }
            last_frame = None;
        }
        let elapsed = if pet.brain.activity == Activity::Walk {
            walk_elapsed
        } else {
            (pet.paused_at.unwrap_or(now) - pet.brain.started)
                * if pet.brain.activity == Activity::Dance && pet.companion.dance_rate > 0. {
                    pet.companion.dance_rate
                } else {
                    1.
                }
        };
        let frame = if pet.brain.activity == Activity::Curious {
            animation.gaze_frame(pet.gaze, pet.settings.reduced_motion)
        } else {
            animation.frame(
                if low_battery && pet.brain.activity != Activity::Walk {
                    elapsed * 0.5
                } else {
                    elapsed
                },
                pet.settings.reduced_motion,
            )
        };
        let flip = false;
        let identity = (pet.brain.activity, frame.data().as_ptr() as usize, flip);
        if last_frame != Some(identity) || last_draw.elapsed() > Duration::from_secs(5) {
            texture
                .update(None, frame.data(), frame.width() as usize * 4)
                .map_err(|e| e.to_string())?;
            canvas.clear();
            canvas
                .copy_ex(&texture, None, None, 0., None, flip, false)
                .map_err(|e| e.to_string())?;
            canvas.present();
            last_frame = Some(identity);
            last_draw = Instant::now();
        }
    }
    pet.release(start.elapsed().as_secs_f64(), &sdl.mouse(), false);
    pet.save(&tray);
    Ok(())
}
