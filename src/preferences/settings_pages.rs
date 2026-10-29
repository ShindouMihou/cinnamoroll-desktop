use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{INK, MUTED, SOFT},
};
use crate::{
    command::Command,
    companion::preferences::{Accessory, Season},
    settings::Settings,
};

impl Page<'_, '_> {
    pub(super) fn appearance(&mut self, s: &Settings) {
        self.section(118., "Pet size");
        self.note(145., "Small keeps the artwork crisp and unobtrusive.");
        for (i, (label, size)) in [("Tiny", 30), ("Small", 40), ("Medium", 55), ("Large", 75)]
            .into_iter()
            .enumerate()
        {
            self.button(
                Self::cell(i, 178., 4),
                label,
                Action::Command(Command::Size(size)),
                s.size == size,
                true,
            );
        }
        self.divider(230.);
        self.section(250., "Accessory");
        for (i, (label, choice)) in [
            ("None", Accessory::None),
            ("Bow", Accessory::Bow),
            ("Scarf", Accessory::Scarf),
            ("Hat", Accessory::Hat),
            ("Bag", Accessory::Bag),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 285., 5),
                label,
                Action::Command(Command::Accessory(choice)),
                s.extras.accessory == choice,
                true,
            );
        }
        self.note(
            331.,
            "Put aside for naps, hanging, and certain active poses.",
        );
        self.divider(365.);
        self.toggle(
            387.,
            "Reduced motion",
            "Still poses and no automatic movement.",
            s.reduced_motion,
            Command::ReducedMotion,
        );
        self.divider(446.);
        self.painter.text("Creative tools", LEFT, 468., 14., INK);
        self.note(491., "Play with colors and keep a little moment.");
        self.button(
            [RIGHT - 130., 463., 130., 34.],
            "Open tools",
            Action::Tab(9),
            false,
            true,
        );
    }
    pub(super) fn behavior(&mut self, s: &Settings) {
        self.section(118., "Activity & rest");
        self.painter.text("Quiet time", LEFT, 158., 14., INK);
        self.stepper(
            153.,
            &format!("{} sec", s.idle_pause_seconds),
            (Command::AdjustQuiet(-15), s.idle_pause_seconds > 15),
            (Command::AdjustQuiet(15), s.idle_pause_seconds < 300),
        );
        self.note(
            195.,
            &format!(
                "About {}–{} seconds between random activities.",
                s.idle_pause_seconds * 3 / 4,
                s.idle_pause_seconds * 3 / 2
            ),
        );
        self.divider(229.);
        self.painter.text("Sleep when idle", LEFT, 253., 14., INK);
        self.stepper(
            248.,
            &if s.sleep_after_minutes == 0 {
                "Off".into()
            } else {
                format!("{} min", s.sleep_after_minutes)
            },
            (Command::AdjustSleep(-1), s.sleep_after_minutes > 0),
            (Command::AdjustSleep(1), s.sleep_after_minutes < 60),
        );
        self.note(290., "Wake when you return. Set to Off to disable.");
        self.divider(326.);
        self.toggle(
            351.,
            "Wandering",
            "Allow occasional little walks.",
            s.wandering,
            Command::Wander,
        );
        self.toggle(
            410.,
            "Occasional mouse chase",
            "Short chases with long breaks. Off by default.",
            s.occasional_chase,
            Command::OccasionalChase,
        );
        self.toggle(
            476.,
            "Quiet surprises",
            "Tiny moments with several minutes of quiet between.",
            s.extras.ambient_surprises,
            Command::Surprises,
        );
    }
    pub(super) fn connections(&mut self, s: &Settings) {
        self.toggle(
            120.,
            "Music reactions",
            "Listen to supported players and browser audio.",
            s.music_reactions,
            Command::Music,
        );
        self.note(180., "Videos and podcasts can trigger listening too.");
        self.divider(219.);
        self.toggle(
            245.,
            "Follow typing cursor",
            "A quiet visit beside a supported text cursor.",
            s.follow_typing,
            Command::FollowTyping,
        );
        self.painter.pill([LEFT, 319., 504., 140.], 10., SOFT, None);
        self.painter.text(
            if cfg!(target_os = "macos") {
                "Accessibility access"
            } else {
                "App compatibility"
            },
            LEFT + 16.,
            335.,
            13.,
            INK,
        );
        if cfg!(target_os = "macos") {
            self.painter.text(
                "Add Cinnamoroll and enable its switch in macOS.",
                LEFT + 16.,
                361.,
                12.,
                MUTED,
            );
            self.button(
                [LEFT + 16., 401., 254., 34.],
                "Open Accessibility settings",
                Action::Command(Command::OpenAccessibility),
                false,
                true,
            );
        } else {
            self.painter.text(
                "Works with apps that expose a typing cursor.",
                LEFT + 16.,
                367.,
                12.,
                MUTED,
            );
            self.painter.text(
                "No extra permission is needed. Support varies by app.",
                LEFT + 16.,
                395.,
                12.,
                MUTED,
            );
        }
        self.note(
            488.,
            "Cursor position only. No typed text is read or saved.",
        );
    }
    pub(super) fn corner(&mut self, s: &Settings) {
        self.toggle(
            118.,
            "Cozy corner",
            "Drag the blanket to move it; click for a nap.",
            s.cozy_corner,
            Command::CozyCorner,
        );
        self.divider(195.);
        self.section(220., "Blanket color");
        for (i, label) in ["Cloud blue", "Rose pink", "Soft lilac"]
            .into_iter()
            .enumerate()
        {
            self.button(
                Self::cell(i, 257., 3),
                label,
                Action::Command(Command::Blanket(i as u8)),
                s.blanket_color == i as u8,
                s.cozy_corner,
            );
        }
        self.section(328., "Make yourself at home");
        self.button(
            [LEFT, 365., 246., 34.],
            "Place bed by Cinnamoroll",
            Action::Command(Command::CornerHere),
            false,
            true,
        );
        self.button(
            [RIGHT - 246., 365., 246., 34.],
            "Go rest",
            Action::Command(Command::CornerRest),
            false,
            true,
        );
        self.section(426., "Seasonal moments");
        for (i, (label, choice)) in [
            ("None", Season::None),
            ("Rain", Season::Rain),
            ("Autumn", Season::Autumn),
            ("Birthday", Season::Birthday),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 462., 4),
                label,
                Action::Command(Command::Season(choice)),
                s.extras.season == choice,
                true,
            );
        }
        self.note(
            511.,
            "Drag the blanket to carry him when he’s settled on it.",
        );
    }
}
