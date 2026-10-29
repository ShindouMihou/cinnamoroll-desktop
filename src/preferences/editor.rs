use super::{
    layout::LEFT,
    paint::{BLUE, BORDER, INK, MUTED, Painter},
};
use crate::{command::Command, settings::Settings};
use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};

#[derive(Clone, Copy)]
pub(super) enum Field {
    Task,
    Preset(usize),
    Palette,
}
pub(super) struct Editor {
    kind: Field,
    draft: String,
    replace: bool,
    error: Option<&'static str>,
}
pub(super) enum Outcome {
    Continue,
    Cancel,
    Commit(Command),
}

impl Editor {
    pub fn new(kind: Field, s: &Settings) -> Self {
        let draft = match kind {
            Field::Task => s.extras.task.clone(),
            Field::Preset(i) => s
                .extras
                .presets
                .get(i)
                .map_or(String::new(), |p| p.name.clone()),
            Field::Palette => s
                .extras
                .palette
                .iter()
                .map(|c| format!("#{c:06X}"))
                .collect::<Vec<_>>()
                .join(" "),
        };
        Self {
            kind,
            draft,
            replace: true,
            error: None,
        }
    }
    fn commit(&mut self) -> Outcome {
        let value = self.draft.trim().to_owned();
        let command = match self.kind {
            Field::Task => Command::TaskText(value),
            Field::Preset(i) => {
                if value.is_empty() {
                    self.error = Some("Give this preset a name.");
                    return Outcome::Continue;
                } else {
                    Command::PresetText(i, value)
                }
            }
            Field::Palette => {
                if crate::companion::extras::palette(&value).is_none() {
                    self.error =
                        Some("Enter three six-digit colors, such as #B0DAF5 #F8CDDC #DBD1F6.");
                    return Outcome::Continue;
                } else {
                    Command::PaletteText(value)
                }
            }
        };
        Outcome::Commit(command)
    }
    pub fn paste(&mut self, text: &str) {
        if self.replace {
            self.draft.clear();
            self.replace = false;
        }
        let max = match self.kind {
            Field::Task => 80,
            Field::Preset(_) => 24,
            Field::Palette => 48,
        };
        for c in text.chars().filter(|c| !c.is_control()) {
            if self.draft.chars().count() < max {
                self.draft.push(c);
            }
        }
        self.error = None;
    }
    pub fn handle(&mut self, event: &Event) -> Outcome {
        match event {
            Event::TextInput { text, .. } => self.paste(text),
            Event::KeyDown {
                keycode: Some(key),
                keymod,
                ..
            } => match key {
                Keycode::Escape => return Outcome::Cancel,
                Keycode::Return => return self.commit(),
                Keycode::Backspace => {
                    if self.replace {
                        self.draft.clear();
                        self.replace = false;
                    } else {
                        self.draft.pop();
                    }
                }
                Keycode::A
                    if keymod.intersects(
                        Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD,
                    ) =>
                {
                    self.replace = true
                }
                _ => {}
            },
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } if (276. ..310.).contains(y) => {
                if (LEFT + 16. ..LEFT + 156.).contains(x) {
                    return self.commit();
                }
                if (LEFT + 168. ..LEFT + 308.).contains(x) {
                    return Outcome::Cancel;
                }
            }
            _ => {}
        }
        Outcome::Continue
    }
    pub fn draw(&self, p: &mut Painter<'_>) {
        p.pill([LEFT - 4., 108., 512., 224.], 12., [255; 3], Some(BORDER));
        p.text(
            match self.kind {
                Field::Task => "One small goal",
                Field::Preset(_) => "Name this subject preset",
                Field::Palette => "Your palette colors",
            },
            LEFT + 16.,
            127.,
            16.,
            INK,
        );
        p.text(
            "Type to replace · Enter to save · Esc to cancel",
            LEFT + 16.,
            158.,
            11.,
            MUTED,
        );
        p.pill(
            [LEFT + 16., 187., 472., 42.],
            7.,
            [246, 251, 253],
            Some(BLUE),
        );
        p.fitted(&self.draft, [LEFT + 27., 198., 450., 20.], 13., INK);
        if let Some(error) = self.error {
            p.fitted(error, [LEFT + 16., 244., 472., 20.], 11., [164, 84, 74]);
        }
        for (x, label) in [(LEFT + 16., "Save"), (LEFT + 168., "Cancel")] {
            p.pill([x, 276., 140., 34.], 8., [246, 251, 253], Some(BORDER));
            p.centered(label, [x, 276., 140., 34.], 13., INK);
        }
    }
}
