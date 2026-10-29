use super::*;
fn pet() -> Pet {
    Pet {
        companion: crate::companion::Companion::default(),
        settings: Settings::default(),
        brain: Brain::new(crate::behavior::default_rules(), 1),
        position: (0., 0.),
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
    }
}
#[test]
fn automatic_sleep_wakes_on_activity_but_manual_naps_stay_tucked_in() {
    let mut pet = pet();
    pet.interact(Activity::Idle, 0.);
    assert!(!pet.idle_sleep(Some(299.), 299.));
    assert!(!pet.play.auto_sleep);
    pet.idle_sleep(Some(300.), 300.);
    assert_eq!(pet.play.mode, Mode::Sleeping);
    assert!(pet.play.auto_sleep);
    assert!(pet.idle_sleep(Some(4.), 310.));
    assert_eq!(pet.brain.activity, Activity::Greet);
    assert!(!pet.play.auto_sleep);
    pet.toggle_nap(320.);
    assert!(!pet.idle_sleep(Some(0.), 325.));
    assert_eq!(pet.play.mode, Mode::Sleeping);
    pet.interact(Activity::Idle, 330.);
    pet.settings.sleep_after_minutes = 0;
    pet.idle_sleep(Some(900.), 900.);
    assert!(!pet.play.busy());
}
#[test]
fn automatic_chase_requires_recent_mouse_movement_and_respects_quiet_breaks() {
    let mut pet = pet();
    pet.settings.occasional_chase = true;
    pet.interact(Activity::Idle, 260.);
    pet.maybe_chase(310., false);
    assert!(!pet.play.busy());
    pet.interaction.notice_pointer((90., 100.), (0., 0.), 310.);
    pet.interaction.notice_pointer((100., 100.), (0., 0.), 311.);
    pet.maybe_chase(311., true);
    assert!(!pet.play.busy());
    pet.maybe_chase(311., false);
    assert_eq!(
        pet.play.mode,
        Mode::Chase {
            until: 321.,
            tired_at: 316.
        }
    );
    assert!((611. ..=911.).contains(&pet.next_chase));
    pet.interact(Activity::Idle, 312.);
    pet.maybe_chase(350., false);
    assert!(!pet.play.busy());
}
#[test]
fn direct_actions_cancel_pending_actions_and_a_tucked_in_nap_waits_for_wakeup() {
    let mut pet = pet();
    pet.feed(10.);
    assert!(pet.play.feed_reward);
    assert_eq!(pet.brain.until, 15.);
    pet.toggle_nap(11.);
    assert_eq!(pet.play.mode, Mode::Sleeping);
    assert!(pet.brain.until.is_infinite());
    assert!(!pet.play.feed_reward);
    pet.toggle_nap(12.);
    assert_eq!(pet.play.mode, Mode::Normal);
    assert_eq!(pet.brain.activity, Activity::Greet);
    pet.peek(20.);
    assert_eq!(
        pet.play.mode,
        Mode::Peek {
            capture_at: 23.,
            until: 29.,
            target: None
        }
    );
    pet.interact(Activity::Held, 21.);
    assert!(!pet.play.busy());
    pet.toggle_chase(30.);
    assert_eq!(
        pet.play.mode,
        Mode::Chase {
            until: 60.,
            tired_at: 35.
        }
    );
    pet.toggle_chase(31.);
    assert!(!pet.play.busy());
    pet.settings.reduced_motion = true;
    pet.toggle_chase(40.);
    assert!(!pet.play.busy());
    assert_eq!(pet.brain.activity, Activity::Play);
}

#[test]
fn a_nearby_cursor_starts_a_pounce_and_direct_interaction_cancels_it() {
    let mut pet = pet();
    assert!(!pet.catch_cursor((900., 700.), 10.));
    assert!(pet.catch_cursor((104., 53.), 10.));
    assert_eq!(pet.brain.activity, Activity::ChasePause);
    assert!(matches!(pet.play.mode, Mode::ChaseCatch { .. }));
    pet.interact(Activity::Held, 10.2);
    assert_eq!(pet.play.mode, Mode::Normal);
    assert_eq!(pet.brain.activity, Activity::Held);
}

#[test]
fn an_unreachable_cursor_gets_a_full_five_second_breathing_break_then_chasing_resumes() {
    let mut pet = pet();
    let pointer = (900., 700.);
    pet.start_chase(10., 10.);
    assert!(pet.chase_phase(pointer, 14.99, false));
    assert!(!pet.chase_phase(pointer, 15., false));
    assert_eq!(pet.brain.activity, Activity::Pant);
    assert_eq!(
        pet.play.mode,
        Mode::ChaseRest {
            until: 20.,
            chase_until: 25.
        }
    );
    let position = pet.position;
    assert!(!pet.chase_phase((80., 53.), 19.99, false));
    assert_eq!(pet.position, position);
    assert_eq!(pet.brain.activity, Activity::Pant);
    assert!(!pet.chase_phase(pointer, 20., false));
    assert_eq!(
        pet.play.mode,
        Mode::Chase {
            until: 25.,
            tired_at: 25.
        }
    );
    assert!(pet.chase_phase(pointer, 20.01, false));
    assert_eq!(pet.play.motion.velocity, (0., 0.));
}

#[test]
fn rest_timers_respect_pause_and_explicit_stop_and_reaching_the_pointer_beats_exhaustion() {
    let mut pet = pet();
    pet.start_chase(0., 30.);
    pet.chase_phase((900., 700.), 5., false);
    pet.play.resume(12.);
    assert_eq!(
        pet.play.mode,
        Mode::ChaseRest {
            until: 22.,
            chase_until: 47.
        }
    );
    pet.toggle_chase(18.);
    assert!(!pet.play.busy());
    pet.start_chase(20., 30.);
    pet.chase_phase((104., 53.), 25., false);
    assert_eq!(pet.brain.activity, Activity::ChasePause);
    assert!(matches!(pet.play.mode, Mode::ChaseCatch { .. }));
}

#[test]
fn study_controls_pause_for_toys_and_resume_without_losing_the_session() {
    let mut p = pet();
    assert!(p.companion_command(Command::StudyStart, 0.));
    assert!(p.quiet_busy());
    assert_eq!(p.brain.activity, Activity::Study);
    p.companion.study.tick(10., false, 5);
    let remaining = p.companion.study.view().seconds;
    p.companion_command(Command::ToyBall, 10.);
    assert!(p.companion.study.paused);
    assert!(p.companion.item.is_some());
    p.companion.study.tick(100., false, 5);
    assert_eq!(p.companion.study.view().seconds, remaining);
    p.companion_command(Command::StudyPause, 100.);
    assert!(!p.companion.study.paused);
    assert!(p.companion.item.is_none());
    p.companion_command(Command::StudyStop, 101.);
    assert!(!p.quiet_busy());
}

#[test]
fn study_transitions_and_cozy_rest_are_explicit_and_cancellable() {
    let mut p = pet();
    p.settings.focus_minutes = 1;
    p.settings.break_minutes = 1;
    p.companion_command(Command::StudyStart, 0.);
    p.update_study(60., false);
    assert_eq!(
        p.companion.study.phase,
        crate::companion::study::Phase::Break
    );
    assert_eq!(p.brain.activity, Activity::Stretch);
    p.update_study(120., false);
    assert_eq!(
        p.companion.study.phase,
        crate::companion::study::Phase::Ready
    );
    p.companion_command(Command::CornerRest, 121.);
    assert!(p.settings.cozy_corner);
    assert!(p.settings.corner_position.is_some());
    assert!(p.companion.resting);
    p.interact(Activity::HighFive, 122.);
    assert!(!p.companion.resting);
    p.companion_command(Command::OfferTreat, 123.);
    assert!(p.companion.item.is_some());
    p.companion_command(Command::PutAway, 124.);
    assert!(p.companion.item.is_none());
}

#[test]
fn ambient_moments_wait_for_quiet_idle_and_never_interrupt_focus_or_manual_naps() {
    let mut pet = pet();
    pet.companion.next_surprise = 1.;
    pet.settings.extras.ambient_surprises = true;
    pet.companion.study.start(0., 25);
    pet.maybe_surprise(30., false);
    assert_eq!(pet.companion.next_surprise, 1.);
    pet.companion.study.stop();
    pet.toggle_nap(30.);
    pet.maybe_surprise(60., false);
    assert_eq!(pet.companion.next_surprise, 1.);
    pet.interact(Activity::Idle, 70.);
    pet.settings.reduced_motion = true;
    pet.maybe_surprise(100., false);
    assert_eq!(pet.companion.next_surprise, 1.);
    pet.settings.reduced_motion = false;
    pet.maybe_surprise(100., false);
    assert!(pet.companion.next_surprise >= 580.);
    let deadline = pet.companion.next_surprise;
    pet.maybe_surprise(101., false);
    assert_eq!(pet.companion.next_surprise, deadline);
}

#[test]
fn the_pet_menu_holds_games_and_props_without_freezing_study_time() {
    let mut p = pet();
    p.start(Activity::Walk, 0.);
    p.brain.until = 10.;
    p.play.mode = Mode::Chase {
        until: 30.,
        tired_at: 5.,
    };
    p.companion.study.start(0., 25);
    let mut toy =
        crate::companion::world::Item::new(crate::companion::world::Kind::Wand, (100., 100.), 0.);
    toy.last_pop = 3.;
    toy.reaction_until = 4.6;
    p.companion.item = Some(toy);
    p.companion.after_action = Some((4., Activity::Happy));
    p.hold_for_menu(2.);
    assert_eq!(p.brain.started, 2.);
    assert_eq!(p.brain.until, 12.);
    assert_eq!(
        p.play.mode,
        Mode::Chase {
            until: 32.,
            tired_at: 7.
        }
    );
    assert_eq!(p.companion.item.as_ref().unwrap().last_pop, 5.);
    assert_eq!(p.companion.item.as_ref().unwrap().expires, 62.);
    assert_eq!(p.companion.after_action, Some((6., Activity::Happy)));
    p.companion.study.tick(2., false, 5);
    assert_eq!(p.companion.study.view().seconds, 1498);
}

#[test]
fn surprise_items_wait_for_an_offer_and_respect_focus_and_existing_toys() {
    let mut pet = pet();
    pet.settings.extras.surprise_items = true;
    pet.interact(Activity::Idle, 0.);
    pet.companion.next_item = 1.;
    pet.maybe_item(30., true);
    assert!(pet.companion.item.is_none());
    pet.companion.study.start(31., 25);
    pet.maybe_item(32., false);
    assert!(pet.companion.item.is_none());
    pet.companion.study.stop();
    pet.maybe_item(33., false);
    let item = pet.companion.item.as_ref().unwrap();
    assert!(item.spontaneous);
    assert!(!item.activated);
    assert!(!pet.quiet_busy());
    assert_eq!(pet.brain.activity, Activity::Idle);
    let expires = item.expires;
    pet.companion.next_item = 1.;
    pet.maybe_item(40., false);
    assert_eq!(pet.companion.item.as_ref().unwrap().expires, expires);
    pet.companion.item.as_mut().unwrap().grab(41.);
    assert!(pet.quiet_busy());
}

#[test]
fn chase_timeout_schedules_a_tired_return_and_picking_up_cancels_it() {
    let mut pet = pet();
    pet.settings.cozy_corner = true;
    pet.settings.corner_position = Some((100, 200));
    pet.start_chase(0., 10.);
    assert!(!pet.chase_phase((900., 700.), 11., false));
    assert_eq!(pet.brain.activity, Activity::Pant);
    assert_eq!(pet.companion.chase_return, Some(14.));
    assert!(pet.quiet_busy());
    pet.hold_for_menu(5.);
    assert_eq!(pet.companion.chase_return, Some(19.));
    pet.interact(Activity::Held, 17.);
    assert_eq!(pet.companion.chase_return, None);
    assert!(!pet.quiet_busy());
}

#[test]
fn cup_tray_starts_clear_of_pet_and_bed_even_at_the_right_display_edge() {
    let mut pet = pet();
    pet.position = (860., 600.);
    pet.settings.cozy_corner = true;
    pet.settings.corner_position = Some((860, 600));
    pet.spawn_item(crate::companion::world::Kind::HideCups, 1.);
    let item = pet.companion.item.as_ref().unwrap();
    let size = item.kind.size();
    let tray = Bounds {
        x: (item.position.0 - size.0 as f32 / 2.).round() as i32,
        y: (item.position.1 - size.1 as f32 / 2.).round() as i32,
        width: size.0,
        height: size.1,
    };
    assert!(tray.x >= 0 && tray.x + tray.width as i32 <= 1000);
    let pet_size = pet.size();
    let home = crate::companion::corner_home(
        pet.settings.corner_position,
        pet.position,
        pet_size,
        pet.bounds(),
    );
    let layout = crate::companion::corner::Layout::new(pet_size);
    let pet_rect = Bounds {
        x: pet.position.0 as i32,
        y: pet.position.1 as i32,
        width: pet_size.0,
        height: pet_size.1,
    };
    let bed_rect = Bounds {
        x: (home.0 + layout.offset.0) as i32,
        y: (home.1 + layout.offset.1) as i32,
        width: layout.size.0,
        height: layout.size.1,
    };
    for obstacle in [pet_rect, bed_rect] {
        let overlap = tray.x < obstacle.x + obstacle.width as i32
            && tray.x + tray.width as i32 > obstacle.x
            && tray.y < obstacle.y + obstacle.height as i32
            && tray.y + tray.height as i32 > obstacle.y;
        assert!(!overlap, "Cup tray overlaps the pet or its bed");
    }
}

#[test]
fn ribbon_and_wand_taps_finish_without_moving_or_restarting_when_the_toy_is_pulled_away() {
    use crate::companion::world::{Item, Kind};
    for kind in [Kind::Ribbon, Kind::Wand] {
        let mut pet = pet();
        let size = pet.size();
        let prop = kind.size();
        let tip = if kind == Kind::Ribbon {
            (20., 84.)
        } else {
            (18., 73.)
        };
        let position = (
            size.0 as f32 * 0.65 + prop.0 as f32 / 2. - tip.0 / 160. * prop.0 as f32,
            size.1 as f32 * 0.8 + prop.1 as f32 / 2. - tip.1 / 112. * prop.1 as f32,
        );
        let mut item = Item::new(kind, position, 0.);
        item.activated = true;
        item.held = true;
        pet.companion.item = Some(item);
        pet.advance_touch_toy(4., false);
        assert_eq!(pet.brain.activity, Activity::PawPlay);
        assert_eq!(pet.position, (0., 0.));
        pet.companion.item.as_mut().unwrap().position = (900., 700.);
        pet.advance_touch_toy(4.8, false);
        assert_eq!(pet.brain.activity, Activity::PawPlay);
        assert_eq!(pet.brain.started, 4.);
        pet.advance_touch_toy(5.7, false);
        assert_eq!(pet.brain.activity, Activity::Watching);
        assert_eq!(pet.position, (0., 0.));
    }
}

#[test]
fn new_moments_require_the_correct_gestures_and_keep_the_pet_stationary() {
    use crate::companion::world::Kind;
    let mut p = pet();
    p.spawn_item(Kind::Dandelion, 0.);
    p.advance_moment(1.);
    assert_eq!(p.settings.extras.keepsakes, 0);
    let head = (
        p.position.0 + p.size().0 as f32 * 0.5,
        p.position.1 + p.size().1 as f32 * 0.38,
    );
    let i = p.companion.item.as_mut().unwrap();
    i.position = head;
    i.held = true;
    let before = p.position;
    p.advance_moment(2.);
    assert_eq!(p.brain.activity, Activity::Sneeze);
    assert_eq!(p.settings.extras.keepsakes, 2);
    assert_eq!(p.position, before);
    p.spawn_item(Kind::Brush, 10.);
    for (n, x) in [head.0 - 20., head.0, head.0 + 20.].into_iter().enumerate() {
        let i = p.companion.item.as_mut().unwrap();
        i.position = (x, head.1);
        i.held = true;
        p.advance_moment(10. + n as f64 * 0.1);
    }
    assert_eq!(p.brain.activity, Activity::Groom);
    assert_eq!(p.settings.extras.keepsakes, 3);
}

#[test]
fn dream_bubbles_obey_study_reduced_motion_and_low_battery() {
    let mut p = pet();
    p.play.mode = Mode::Sleeping;
    p.update_moments(0., false);
    p.update_moments(91., false);
    assert!(p.companion.moments.dream.is_some());
    p.update_moments(92., true);
    assert!(p.companion.moments.dream.is_none());
    p.companion.study.start(93., 25);
    p.update_moments(999., false);
    assert!(p.companion.moments.dream.is_none());
    p.companion.study.stop();
    p.settings.reduced_motion = true;
    p.update_moments(1999., false);
    assert!(p.companion.moments.dream.is_none());
}

#[test]
fn sleep_pauses_study_and_reconnecting_a_display_restores_its_nook() {
    let mut p = pet();
    p.companion.study.start(0., 25);
    p.update_study(5., false);
    p.update_study(5000., true);
    assert!(p.companion.study.paused);
    assert_eq!(p.companion.study.view().seconds, 1495);
    let display = Bounds {
        x: -1200,
        y: 0,
        width: 1200,
        height: 900,
    };
    p.settings.cozy_corner = true;
    p.settings.extras.comfort.preferred_display = Some(display);
    p.settings.extras.comfort.remember(display, (-900, 500));
    p.restore_nook();
    assert!(p.settings.corner_position.unwrap().0 >= 0);
    let home = p.settings.corner_position.unwrap();
    p.position = (home.0 as f32, home.1 as f32);
    p.monitors.push(display);
    p.restore_nook();
    assert_eq!(p.settings.corner_position, Some((-900, 500)));
    assert_eq!(p.position, (-900., 500.));
}
