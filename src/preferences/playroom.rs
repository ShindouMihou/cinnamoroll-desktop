use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{BORDER, INK, MUTED, SOFT},
    status::activity_name,
};
use crate::{behavior::Activity, command::Command, settings::Settings};

impl Page<'_, '_> {
    pub(super) fn playroom(&mut self, s: &Settings) {
        self.painter
            .pill([LEFT, 112., 244., 250.], 12., [255; 3], Some(BORDER));
        self.painter
            .pill([448., 112., 248., 250.], 12., [255; 3], Some(BORDER));
        self.painter.text("Toys & treats", 208., 128., 15., INK);
        self.painter
            .text("Bring a little something to him.", 208., 152., 11., MUTED);
        for (index, (label, command)) in [
            ("Offer a treat", Command::OfferTreat),
            ("Take out a ball", Command::ToyBall),
            ("Pop bubbles", Command::ToyBubbles),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                [208., 180. + index as f32 * 42., 212., 34.],
                label,
                Action::Command(command),
                false,
                true,
            );
        }
        self.painter.pill([208., 314., 212., 1.], 0., BORDER, None);
        self.button(
            [208., 325., 212., 24.],
            "Tidy toys & treats",
            Action::Command(Command::PutAway),
            false,
            true,
        );

        self.painter.text("Touch & affection", 464., 128., 15., INK);
        self.painter
            .text("A little affection, anytime.", 464., 152., 11., MUTED);
        for (index, (label, activity)) in [
            ("Head pats", Activity::Petting),
            ("Cheek rub", Activity::CheekRub),
            ("High five", Activity::HighFive),
            ("Nose boop", Activity::Boop),
            ("Tickle", Activity::Tickle),
            ("Nuzzle", Activity::Nuzzle),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                [
                    464. + (index % 2) as f32 * 114.,
                    180. + (index / 2) as f32 * 42.,
                    102.,
                    34.,
                ],
                label,
                Action::Command(Command::Activity(activity)),
                false,
                true,
            );
        }
        self.painter
            .text("Or pet him directly on screen.", 464., 331., 11., MUTED);

        self.painter.pill([LEFT, 382., 504., 66.], 12., SOFT, None);
        self.painter.text("Surprise items", 208., 396., 14., INK);
        self.painter.text(
            &if s.extras.surprise_items {
                format!("On · About every {} min", s.extras.item_interval_minutes)
            } else {
                "Off · Only take out toys yourself".into()
            },
            208.,
            420.,
            11.,
            MUTED,
        );
        self.button(
            [564., 399., 116., 32.],
            "Manage",
            Action::Tab(10),
            false,
            true,
        );

        self.painter
            .pill([LEFT, 462., 504., 64.], 12., [255; 3], Some(BORDER));
        self.painter.text("Touch reactions", 208., 476., 14., INK);
        self.painter.text(
            "React when you stroke his head, cheeks, or tummy.",
            208.,
            500.,
            11.,
            MUTED,
        );
        self.button(
            [616., 478., 64., 32.],
            if s.touch_enabled { "On" } else { "Off" },
            Action::Command(Command::TouchEnabled),
            s.touch_enabled,
            true,
        );
    }
    pub(super) fn animations(&mut self, s: &Settings, paused: bool, activity: Activity) {
        self.painter.pill([LEFT, 112., 504., 54.], 9., SOFT, None);
        self.painter.text(
            if paused { "Paused" } else { "Playing" },
            LEFT + 14.,
            122.,
            10.,
            MUTED,
        );
        self.painter
            .text(activity_name(activity), LEFT + 14., 138., 13., INK);
        self.button(
            [RIGHT - 122., 122., 108., 34.],
            "Rest quietly",
            Action::Command(Command::Activity(Activity::Idle)),
            false,
            true,
        );
        self.section(190., "Little moments");
        self.note(
            217.,
            if paused {
                "Pet paused. Resume from Companion to see movement."
            } else if s.reduced_motion {
                "Reduced motion is on. Previews use still poses."
            } else {
                "Choose a pose. He returns to his usual routine afterward."
            },
        );
        for (i, (label, choice)) in [
            ("Dance", Activity::Dance),
            ("Wave", Activity::Wave),
            ("Read a book", Activity::Read),
            ("Listen", Activity::Listen),
            ("Cloud play", Activity::Play),
            ("Feel loved", Activity::Happy),
            ("Act shy", Activity::Shy),
            ("Be curious", Activity::Curious),
            ("Get excited", Activity::Excited),
            ("Get sleepy", Activity::Drowsy),
            ("Stretch", Activity::Stretch),
            ("Head pats", Activity::Petting),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 245., 3),
                label,
                Action::Command(Command::Activity(choice)),
                activity == choice,
                true,
            );
        }
        self.divider(427.);
        self.section(449., "Little adventures");
        for (i, (label, command)) in [
            ("Chase / stop", Command::Chase),
            ("Peek at pointer", Command::Peek),
            ("Hide at an edge", Command::EdgePeek),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 481., 3),
                label,
                Action::Command(command),
                false,
                true,
            );
        }
    }
}
