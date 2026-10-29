use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{BLUE, INK, SOFT},
};
use crate::{
    command::Command,
    companion::{
        preferences::BreakChoice,
        study::{Phase, View},
    },
    settings::Settings,
};

impl Page<'_, '_> {
    pub(super) fn study(&mut self, s: &Settings, view: View, paused: bool) {
        self.button(
            [RIGHT - 140., 28., 140., 34.],
            "Session plan",
            Action::Tab(8),
            false,
            true,
        );
        let active = matches!(view.phase, Phase::Focus | Phase::Break);
        let label = if paused {
            "Pet paused"
        } else if view.paused {
            "Session paused"
        } else {
            match view.phase {
                Phase::Focus => {
                    if view.stopwatch {
                        "Open-ended study · elapsed"
                    } else {
                        "Focus time"
                    }
                }
                Phase::Break => "A little break",
                Phase::Ready => "Ready for another session?",
                Phase::Idle => "Ready when you are",
            }
        };
        self.painter.pill([LEFT, 114., 504., 128.], 12., SOFT, None);
        self.painter.text(label, LEFT + 18., 130., 13., INK);
        let seconds = if active {
            view.seconds
        } else {
            s.focus_minutes * 60
        };
        self.painter.text(
            &format!("{:02}:{:02}", seconds / 60, seconds % 60),
            LEFT + 18.,
            157.,
            38.,
            INK,
        );
        self.button(
            [RIGHT - 170., 157., 152., 36.],
            if !active {
                "Start focus"
            } else if view.paused {
                "Resume session"
            } else {
                "Pause session"
            },
            Action::Command(if active {
                Command::StudyPause
            } else {
                Command::StudyStart
            }),
            true,
            !paused,
        );
        self.painter.fitted(
            if paused {
                "Resume the pet from Companion to continue."
            } else if !s.extras.task.is_empty() {
                &s.extras.task
            } else {
                "A gentle stretch follows each focus session."
            },
            [LEFT + 18., 215., 468., 18.],
            11.,
            BLUE,
        );
        for (i, (label, command, enabled)) in [
            (
                "Add five minutes",
                Command::StudyExtend,
                active && !view.stopwatch,
            ),
            ("Skip this phase", Command::StudySkip, active),
            ("End session", Command::StudyStop, view.phase != Phase::Idle),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 256., 3),
                label,
                Action::Command(command),
                false,
                enabled,
            );
        }
        self.section(310., "Session lengths");
        self.painter.text("Focus", LEFT, 349., 14., INK);
        self.stepper(
            344.,
            &format!("{} min", s.focus_minutes),
            (Command::AdjustFocus(-5), s.focus_minutes > 1),
            (Command::AdjustFocus(5), s.focus_minutes < 180),
        );
        self.painter.text("Break", LEFT, 394., 14., INK);
        self.stepper(
            389.,
            &format!("{} min", s.break_minutes),
            (Command::AdjustBreak(-1), s.break_minutes > 1),
            (Command::AdjustBreak(1), s.break_minutes < 60),
        );
        self.note(435., "New lengths apply to the next focus or break.");
        self.section(464., "Break activity");
        for (i, (label, choice)) in [
            ("Stretch", BreakChoice::Stretch),
            ("Water", BreakChoice::Water),
            ("Breathe", BreakChoice::Breathe),
            ("Play", BreakChoice::Play),
        ]
        .into_iter()
        .enumerate()
        {
            self.button(
                Self::cell(i, 494., 4),
                label,
                Action::Command(Command::BreakRoutine(choice)),
                s.extras.break_choice == choice,
                true,
            );
        }
    }
}
