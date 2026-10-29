use super::layout::{Action, Page};
use crate::{command::Command, settings::Settings};

impl Page<'_, '_> {
    pub(super) fn surprise_items(&mut self, s: &Settings) {
        self.toggle(
            121.,
            "Leave little surprises",
            "Occasionally place an item nearby.",
            s.extras.surprise_items,
            Command::SurpriseItems,
        );
        self.divider(196.);
        self.section(220., "Time between discoveries");
        self.note(250., "A gentle interval, with a little randomness.");
        let minutes = s.extras.item_interval_minutes;
        self.painter
            .text("About every", 192., 296., 14., super::paint::INK);
        self.stepper(
            286.,
            &format!("{minutes} min"),
            (Command::AdjustItemInterval(-1), minutes > 1),
            (Command::AdjustItemInterval(1), minutes < 60),
        );
        self.divider(350.);
        self.section(375., "You make the moment");
        self.note(407., "Treats, tea, ribbons, toys, and a little blanket.");
        self.note(434., "Drag an item close to him to see his reaction.");
        self.note(461., "Unoffered items disappear after two minutes.");
        self.note(488., "Surprises wait during study, naps, and play.");
        self.button(
            [192., 112. + 397., 130., 24.],
            "Back to Playroom",
            Action::Tab(2),
            false,
            true,
        );
    }
}
