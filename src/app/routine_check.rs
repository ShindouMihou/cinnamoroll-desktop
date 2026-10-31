use super::*;

pub(super) fn run() -> Result<(), String> {
    let sdl = sdl3::init().map_err(|e| e.to_string())?;
    let video = sdl.video().map_err(|e| e.to_string())?;
    let mut window = video
        .window("Cinnamoroll · Routine check", 128, 89)
        .hidden()
        .build()
        .map_err(|e| e.to_string())?;
    let mut pet = Pet {
        companion: crate::companion::Companion::default(),
        settings: Settings {
            cozy_corner: true,
            corner_position: Some((100, 100)),
            ..Settings::default()
        },
        brain: Brain::new(crate::behavior::default_rules(), 1),
        position: (100., 100.),
        monitors: vec![Bounds {
            x: 0,
            y: 0,
            width: 1000,
            height: 800,
        }],
        drag: None,
        hidden: false,
        paused_at: None,
        dirty: false,
        interaction: crate::interaction::Interaction::default(),
        gaze: 0,
        play: crate::play::PlayState::default(),
        next_chase: 300.,
    };
    pet.finish_chase(0.);
    pet.advance_companion(4., 0.016, &mut window, false);
    if pet.play.mode != crate::play::Mode::Sleeping {
        return Err("Chase recovery did not settle at home".into());
    }
    for second in 5..=70 {
        let now = f64::from(second);
        pet.idle_sleep(Some(0.), now);
        pet.advance_companion(now, 0.016, &mut window, false);
    }
    if pet.companion.resting || pet.play.mode == crate::play::Mode::Sleeping {
        return Err("An automatic post-chase nap never ends while the computer is active; surprise activities remain blocked".into());
    }
    pet.start(Activity::Idle, 100.);
    pet.companion.next_surprise = 1.;
    pet.settings.extras.last_gift_day = Some(crate::companion::extras::day());
    pet.maybe_surprise(120., false);
    if pet.companion.next_surprise == 1. {
        return Err("Surprise activities remain blocked after automatic rest".into());
    }
    pet.companion_command(Command::CornerRest, 130.);
    for second in 131..=220 {
        let now = f64::from(second);
        pet.idle_sleep(Some(0.), now);
        pet.advance_companion(now, 0.016, &mut window, false);
    }
    if pet.play.mode != crate::play::Mode::Sleeping {
        return Err("A deliberate tuck-in was unexpectedly interrupted".into());
    }
    pet.interact(Activity::Idle, 300.);
    if !pet.tidy_plush(300.) {
        return Err("Cozy plush routine did not start".into());
    }
    for frame in 1..=1800 {
        let now = 300. + f64::from(frame) * 0.05;
        pet.advance_companion(now, 0.05, &mut window, false);
    }
    if pet.companion.cozy_return.is_some()
        || pet.quiet_busy()
        || pet.play.mode == crate::play::Mode::Sleeping
        || (pet.position.0 - 100.).hypot(pet.position.1 - 100.) > 6.
    {
        return Err("Plush routine did not return home awake and release control".into());
    }
    pet.offer_paw(400.);
    pet.interact(Activity::Held, 401.);
    if pet.companion.paw_offer.is_some() {
        return Err("Picking up the pet did not cancel its paw invitation".into());
    }
    println!(
        "Automatic rest ends, surprises resume, manual naps stay tucked in, plush routines return home awake, and invitations yield to direct interaction"
    );
    Ok(())
}
