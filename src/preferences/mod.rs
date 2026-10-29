mod artwork;
mod chrome;
mod editor;
mod layout;
mod overview;
pub(crate) mod paint;
mod planning;
mod playroom;
pub mod preview;
mod settings_pages;
mod status;
mod study;
mod surprise_items;
mod updates;

use crate::{behavior::Activity, command::Command, settings::Settings};
use layout::{Action, Control, Page};
use paint::{Painter, Typography};
use sdl3::{
    VideoSubsystem,
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    pixels::{Color, PixelFormat},
    render::WindowCanvas,
    video::WindowFlags,
};
use tiny_skia::Pixmap;

pub struct Panel {
    canvas: WindowCanvas,
    video: VideoSubsystem,
    editor: Option<editor::Editor>,
    typography: Typography,
    artwork: Option<artwork::Artwork>,
    tab: usize,
    controls: Vec<Control>,
    focus: Option<usize>,
    hover: Option<usize>,
    pressed: Option<usize>,
    dirty: bool,
    snapshot: Option<(Settings, bool, Activity, crate::companion::study::View)>,
    status: String,
    updates: crate::updates::State,
    startup: crate::native::startup::Status,
}

impl Panel {
    pub fn new(video: &VideoSubsystem) -> Result<Self, String> {
        let window = video
            .window("Cinnamoroll · Controls", layout::WIDTH, layout::HEIGHT)
            .set_flags(WindowFlags::HIGH_PIXEL_DENSITY)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = sdl3::render::create_renderer(window, None).map_err(|e| e.to_string())?;
        let mut panel = Self {
            canvas,
            video: video.clone(),
            editor: None,
            typography: Typography::new()?,
            artwork: None,
            tab: 0,
            controls: Vec::new(),
            focus: None,
            hover: None,
            pressed: None,
            dirty: true,
            snapshot: None,
            status: "Changes save automatically.".into(),
            updates: crate::updates::State::Disconnected,
            startup: crate::native::startup::Status::Off,
        };
        panel.refresh_startup();
        panel.raise();
        Ok(panel)
    }
    pub fn update_state(&mut self, state: &crate::updates::State) {
        if &self.updates != state {
            self.updates = state.clone();
            self.dirty = true;
        }
    }
    pub fn open_page(&mut self, page: usize) {
        self.tab = page;
        self.focus = None;
        self.hover = None;
        self.dirty = true;
    }
    pub fn raise(&mut self) {
        self.canvas.window_mut().raise();
    }
    pub fn owns(&self, event: &Event) -> bool {
        event.get_window_id() == Some(self.canvas.window().id())
    }
    pub fn saved(&mut self, success: bool) {
        self.status = (if success {
            "All changes saved."
        } else {
            "Couldn’t save. Check folder access and try again."
        })
        .into();
        self.dirty = true;
    }
    fn refresh_startup(&mut self) {
        match crate::native::startup::status() {
            Ok(status) => self.startup = status,
            Err(error) => {
                self.startup = crate::native::startup::Status::Unavailable;
                self.status = format!("Couldn’t read startup status: {error}");
            }
        }
        self.dirty = true;
    }
    pub fn startup_command(&mut self, open_settings: bool) {
        if open_settings {
            if let Err(error) = crate::native::startup::open_settings() {
                self.status = format!("Couldn’t open startup settings: {error}");
            }
        } else {
            match crate::native::startup::set_enabled(!self.startup.registered()) {
                Ok(status) => {
                    self.startup = status;
                    self.status = match status {
                        crate::native::startup::Status::On => "Launch at login enabled.",
                        crate::native::startup::Status::Off => "Launch at login disabled.",
                        crate::native::startup::Status::Approval => {
                            "Enable Cinnamoroll in your system’s startup settings."
                        }
                        crate::native::startup::Status::Unavailable => {
                            "Launch at login is unavailable here."
                        }
                    }
                    .into();
                }
                Err(error) => {
                    self.status = format!("Couldn’t change startup: {error}");
                }
            }
        }
        self.dirty = true;
    }
    fn activate(&mut self, index: usize) -> Option<Command> {
        let control = self.controls.get(index)?;
        if !control.enabled {
            return None;
        }
        match control.action.clone() {
            Action::Tab(tab) => {
                self.tab = tab;
                self.focus = Some(index);
                self.hover = None;
                self.dirty = true;
                None
            }
            Action::Command(command) => Some(command),
            Action::Edit(field) => {
                if let Some((settings, ..)) = &self.snapshot {
                    self.editor = Some(editor::Editor::new(field, settings));
                    self.video.text_input().start(self.canvas.window());
                    self.dirty = true;
                }
                None
            }
        }
    }
    pub fn handle(&mut self, event: &Event) -> (Option<Command>, bool) {
        if let Some(editor) = self.editor.as_mut()
            && !matches!(
                event,
                Event::Window {
                    win_event: WindowEvent::CloseRequested,
                    ..
                }
            )
        {
            let outcome = if matches!(event,Event::KeyDown {keycode:Some(Keycode::V),keymod,..} if keymod.intersects(Mod::LCTRLMOD|Mod::RCTRLMOD|Mod::LGUIMOD|Mod::RGUIMOD))
            {
                if let Ok(text) = self.video.clipboard().clipboard_text() {
                    editor.paste(&text);
                }
                editor::Outcome::Continue
            } else {
                editor.handle(event)
            };
            self.dirty = true;
            match outcome {
                editor::Outcome::Continue => return (None, false),
                editor::Outcome::Cancel => {
                    self.editor = None;
                    self.video.text_input().stop(self.canvas.window());
                    return (None, false);
                }
                editor::Outcome::Commit(command) => {
                    self.editor = None;
                    self.video.text_input().stop(self.canvas.window());
                    return (Some(command), false);
                }
            }
        }
        match event {
            Event::Window {
                win_event: WindowEvent::CloseRequested,
                ..
            } => return (None, true),
            Event::Window {
                win_event: WindowEvent::Exposed | WindowEvent::PixelSizeChanged(_, _),
                ..
            } => self.dirty = true,
            Event::Window {
                win_event: WindowEvent::FocusGained,
                ..
            } => {
                self.refresh_startup();
            }
            Event::Window {
                win_event: WindowEvent::FocusLost,
                ..
            } => {
                self.pressed = None;
                self.hover = None;
                self.dirty = true;
            }
            Event::MouseMotion { x, y, .. } => {
                let hover = self
                    .controls
                    .iter()
                    .position(|control| control.contains(*x, *y));
                self.dirty |= hover != self.hover;
                self.hover = hover;
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.pressed = self
                    .controls
                    .iter()
                    .position(|control| control.contains(*x, *y));
                self.focus = self.pressed;
                self.dirty = true;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                if let Some(index) = self.pressed.take()
                    && self
                        .controls
                        .get(index)
                        .is_some_and(|control| control.contains(*x, *y))
                {
                    return (self.activate(index), false);
                }
            }
            Event::KeyDown {
                keycode: Some(key),
                keymod,
                repeat: false,
                ..
            } => match *key {
                Keycode::Escape => return (None, true),
                Keycode::Tab if !self.controls.is_empty() => {
                    let backwards = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
                    self.focus = Some(match self.focus {
                        Some(index) if backwards => {
                            (index + self.controls.len() - 1) % self.controls.len()
                        }
                        Some(index) => (index + 1) % self.controls.len(),
                        None if backwards => self.controls.len() - 1,
                        None => 0,
                    });
                    while self
                        .focus
                        .is_some_and(|index| !self.controls[index].enabled)
                    {
                        self.focus = self.focus.map(|index| {
                            if backwards {
                                (index + self.controls.len() - 1) % self.controls.len()
                            } else {
                                (index + 1) % self.controls.len()
                            }
                        });
                    }
                    self.dirty = true;
                }
                Keycode::Return | Keycode::Space => {
                    if let Some(index) = self.focus {
                        return (self.activate(index), false);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        (None, false)
    }
    pub fn render(
        &mut self,
        settings: &Settings,
        paused: bool,
        activity: Activity,
        study: crate::companion::study::View,
    ) -> Result<(), String> {
        if !self.dirty
            && self
                .snapshot
                .as_ref()
                .is_some_and(|(old, state, previous, clock)| {
                    old == settings
                        && *state == paused
                        && (*previous == activity || !matches!(self.tab, 0 | 3))
                        && (*clock == study || !matches!(self.tab, 0 | 1))
                })
        {
            return Ok(());
        }
        let (width, height) = self.canvas.window().size_in_pixels();
        let scale = width as f32 / layout::WIDTH as f32;
        if self
            .artwork
            .as_ref()
            .is_none_or(|artwork| artwork.scale != scale)
        {
            self.artwork = Some(artwork::Artwork::new(scale)?);
        }
        let artwork = self
            .artwork
            .as_mut()
            .ok_or("Missing settings illustrations")?;
        let mut image = Pixmap::new(width, height).ok_or("Cannot allocate settings panel")?;
        image.fill(tiny_skia::Color::from_rgba8(246, 251, 253, 255));
        self.controls = Page {
            updates: &self.updates,
            startup: self.startup,
            artwork,
            painter: &mut Painter {
                image: &mut image,
                typography: &mut self.typography,
                scale,
            },
            controls: Vec::new(),
            focus: self.focus,
            hover: self.hover,
        }
        .draw(self.tab, settings, paused, activity, study, &self.status);
        if let Some(editor) = &self.editor {
            editor.draw(&mut Painter {
                image: &mut image,
                typography: &mut self.typography,
                scale,
            });
        }
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(PixelFormat::RGBA32, width, height)
            .map_err(|e| e.to_string())?;
        texture
            .update(None, image.data(), width as usize * 4)
            .map_err(|e| e.to_string())?;
        self.canvas.set_draw_color(Color::RGB(246, 251, 253));
        self.canvas.clear();
        self.canvas
            .copy(&texture, None, None)
            .map_err(|e| e.to_string())?;
        self.canvas.present();
        self.snapshot = Some((settings.clone(), paused, activity, study));
        self.dirty = false;
        Ok(())
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        self.video.text_input().stop(self.canvas.window());
    }
}
