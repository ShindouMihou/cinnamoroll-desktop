use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{INK, MUTED, SOFT},
    status::activity_name,
};
use crate::{behavior::Activity, command::Command, settings::Settings};

impl Page<'_, '_> {
    pub(super) fn playroom(&mut self, s: &Settings) {
        self.section(118., "Toys & treats");
        self.button(
            [548., 109., 148., 30.],
            "Surprise items",
            Action::Tab(10),
            false,
            true,
        );
        self.note(
            146.,
            "Drag a treat near him, toss a ball, or click the bubbles.",
        );
        for (i, (label, command)) in [
            ("Offer a treat", Command::OfferTreat),
            ("Take out a ball", Command::ToyBall),
            ("Bubbles", Command::ToyBubbles),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 179., 3),
                label,
                Action::Command(command),
                false,
                true,
            );
        }
        self.button(
            [192., 225., 234., 30.],
            "Put toys & treats away",
            Action::Command(Command::PutAway),
            false,
            true,
        );
        self.divider(281.);
        self.section(305., "Touch & affection");
        for (i, (label, activity)) in [
            ("High five", Activity::HighFive),
            ("Tickle", Activity::Tickle),
            ("Nuzzle", Activity::Nuzzle),
            ("Cheek rub", Activity::CheekRub),
            ("Nose boop", Activity::Boop),
            ("Head pats", Activity::Petting),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 340., 3),
                label,
                Action::Command(Command::Activity(activity)),
                false,
                true,
            );
        }
        self.toggle(
            442.,
            "Touch reactions",
            "React to gentle pointer gestures.",
            s.touch_enabled,
            Command::TouchEnabled,
        );
        self.painter.text(
            "Rub cheeks or tummy; hold at a paw or nose.",
            192.,
            507.,
            11.,
            MUTED,
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
