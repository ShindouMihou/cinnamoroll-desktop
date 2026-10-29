use super::{
    editor::Field,
    layout::{Action, LEFT, Page, RIGHT},
    paint::{INK, MUTED},
};
use crate::{command::Command, settings::Settings};

impl Page<'_, '_> {
    pub(super) fn planning(&mut self, s: &Settings) {
        self.section(118., "One small goal");
        self.button(
            [LEFT, 151., 504., 36.],
            if s.extras.task.is_empty() {
                "Add a goal for this session"
            } else {
                &s.extras.task
            },
            Action::Edit(Field::Task),
            false,
            true,
        );
        self.note(
            202.,
            if s.extras.task_done {
                "Done. A little progress worth celebrating."
            } else {
                "Keep it small. This stays on your computer."
            },
        );
        self.button(
            [RIGHT - 116., 195., 116., 30.],
            "Mark done",
            Action::Command(Command::TaskDone),
            s.extras.task_done,
            !s.extras.task.is_empty() && !s.extras.task_done,
        );
        self.divider(237.);
        self.section(257., "Subject presets");
        self.note(
            284.,
            "Start with a preset, or save your current session lengths.",
        );
        for (i, preset) in s.extras.presets.iter().enumerate() {
            let y = 320. + i as f32 * 54.;
            self.painter
                .fitted(&preset.name, [LEFT, y, 244., 22.], 14., INK);
            self.painter.text(
                &format!("{} min focus · {} min break", preset.focus, preset.rest),
                LEFT,
                y + 22.,
                11.,
                MUTED,
            );
            for (x, w, label, action) in [
                (
                    RIGHT - 220.,
                    62.,
                    "Start",
                    Action::Command(Command::UsePreset(i)),
                ),
                (
                    RIGHT - 150.,
                    62.,
                    "Save",
                    Action::Command(Command::SavePreset(i)),
                ),
                (RIGHT - 80., 80., "Rename", Action::Edit(Field::Preset(i))),
            ] {
                self.button([x, y + 3., w, 32.], label, action, false, true);
            }
        }
        self.toggle(
            486.,
            "Study at the cozy corner",
            if s.cozy_corner {
                "Settle in the bed during focus time."
            } else {
                "Enable the bed in Cozy corner to use this."
            },
            s.study_at_corner,
            Command::StudyAtCorner,
        );
    }
    pub(super) fn creative(&mut self, s: &Settings) {
        self.section(118., "Palette colors");
        self.note(
            146.,
            "Choose three six-digit RGB colors for your play palette.",
        );
        let colors = s
            .extras
            .palette
            .iter()
            .map(|c| format!("#{c:06X}"))
            .collect::<Vec<_>>()
            .join("   ");
        self.button(
            [LEFT, 179., 504., 36.],
            &colors,
            Action::Edit(Field::Palette),
            false,
            true,
        );
        for (i, c) in s.extras.palette.iter().enumerate() {
            self.painter.pill(
                [LEFT + i as f32 * 172., 235., 160., 70.],
                10.,
                [(c >> 16) as u8, (c >> 8) as u8, *c as u8],
                None,
            );
        }
        self.section(345., "Make something");
        self.button(
            [LEFT, 382., 246., 34.],
            "Play with this palette",
            Action::Command(Command::Prop(crate::companion::world::Kind::Palette)),
            false,
            true,
        );
        self.button(
            [RIGHT - 246., 382., 246., 34.],
            "Export a sticker",
            Action::Command(Command::Sticker),
            false,
            true,
        );
        self.note(
            447.,
            "Exports a transparent PNG at the artwork’s native resolution.",
        );
        self.note(471., "Find your stickers in Pictures / Cinnamoroll.");
    }
}
