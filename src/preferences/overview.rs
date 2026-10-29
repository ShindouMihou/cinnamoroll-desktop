use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{INK, MUTED},
    status::activity_name,
};
use crate::{
    behavior::Activity,
    command::Command,
    companion::study::{Phase, View},
};

impl Page<'_, '_> {
    pub(super) fn overview(&mut self, paused: bool, activity: Activity, study: View) {
        self.note(116., "Current activity");
        self.painter.text(
            if paused {
                "Paused"
            } else {
                activity_name(activity)
            },
            LEFT,
            139.,
            20.,
            INK,
        );
        self.button(
            [RIGHT - 132., 132., 132., 34.],
            if paused { "Resume pet" } else { "Pause pet" },
            Action::Command(Command::Pause),
            false,
            true,
        );
        if matches!(study.phase, Phase::Focus | Phase::Break) {
            self.painter.text(
                &format!(
                    "{} · {:02}:{:02} remaining{}",
                    if study.phase == Phase::Focus {
                        "Focus"
                    } else {
                        "Break"
                    },
                    study.seconds / 60,
                    study.seconds % 60,
                    if study.paused { " · Paused" } else { "" }
                ),
                LEFT,
                174.,
                11.,
                MUTED,
            );
        }
        self.divider(204.);
        self.section(229., "Quick interactions");
        let asleep = matches!(activity, Activity::Nap | Activity::CozyNap);
        for (i, (title, help, label, command)) in [
            (
                "Head pats",
                "Give him a little affection.",
                "Pet him",
                Command::Activity(Activity::Petting),
            ),
            (
                "A little treat",
                "Offer a snack for him to sniff and enjoy.",
                "Offer a treat",
                Command::OfferTreat,
            ),
            (
                "Nap time",
                if asleep {
                    "Bring him back from his cozy nap."
                } else {
                    "Tuck him in until you wake him."
                },
                if asleep { "Wake him up" } else { "Tuck him in" },
                Command::Nap,
            ),
            (
                "Quiet company",
                "End a game and settle down together.",
                "Rest quietly",
                Command::Activity(Activity::Idle),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 269. + i as f32 * 65.;
            self.painter.text(title, LEFT, y, 14., INK);
            self.painter.text(help, LEFT, y + 22., 12., MUTED);
            self.button(
                [RIGHT - 132., y + 3., 132., 34.],
                label,
                Action::Command(command),
                false,
                true,
            );
            if i < 3 {
                self.divider(y + 53.);
            }
        }
    }
}
