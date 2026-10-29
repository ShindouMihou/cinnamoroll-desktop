use super::*;

pub(super) fn handle(pet: &mut Pet, signal: Signal, now: f64, tray: &Tray, low_battery: bool) {
    match signal {
        Signal::MusicStarted
            if pet.settings.music_reactions && !pet.play.busy() && !pet.quiet_busy() =>
        {
            pet.brain.music_started(now);
        }
        Signal::MusicStatus(message) => {
            eprintln!("{message}");
            tray.music_status(&message);
        }
        Signal::CaretMoved(point) if pet.settings.follow_typing && !low_battery => {
            pet.follow_caret(point, now)
        }
        Signal::CaretStatus(message) if pet.settings.follow_typing => tray.caret_status(&message),
        _ => {}
    }
}
