mod catalog;
mod icons;
mod layout;
mod paint;
mod placement;
pub mod preview;
use crate::preferences::paint::Typography;
use crate::{command::Command, companion::study::View, geometry::Bounds, settings::Settings};
use catalog::{Action, Category, Entry};
use sdl3::{
    VideoSubsystem,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::PixelFormat,
    render::WindowCanvas,
    video::WindowFlags,
};

pub struct Panel {
    canvas: WindowCanvas,
    typography: Typography,
    category: Category,
    entries: Vec<Entry>,
    focus: usize,
    pressed: Option<usize>,
    settings: Settings,
    paused: bool,
    study: View,
    dirty: bool,
    bounds: Bounds,
    anchor: Bounds,
    artwork: Option<paint::Artwork>,
}

impl Panel {
    pub fn new(
        video: &VideoSubsystem,
        mut anchor: Bounds,
        bounds: Bounds,
        settings: &Settings,
        paused: bool,
        study: View,
    ) -> Result<Self, String> {
        if settings.cozy_corner {
            let home = crate::companion::corner_home(
                settings.corner_position,
                (anchor.x as f32, anchor.y as f32),
                (anchor.width, anchor.height),
                bounds,
            );
            if (home.0 - anchor.x as f32).hypot(home.1 - anchor.y as f32) <= 8. {
                let layout = crate::companion::corner::Layout::new((anchor.width, anchor.height));
                anchor.x += layout.offset.0.round() as i32;
                (anchor.width, anchor.height) = layout.footprint;
            }
        }
        let point = placement::position(anchor, (layout::WIDTH, layout::height(7)), bounds);
        let window = video
            .window(
                "Cinnamoroll · Little pocket",
                layout::WIDTH,
                layout::height(7),
            )
            .set_flags(
                WindowFlags::BORDERLESS
                    | WindowFlags::TRANSPARENT
                    | WindowFlags::UTILITY
                    | WindowFlags::ALWAYS_ON_TOP
                    | WindowFlags::HIGH_PIXEL_DENSITY
                    | WindowFlags::HIDDEN,
            )
            .position(point.0 as i32, point.1 as i32)
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = sdl3::render::create_renderer(window, None).map_err(|e| e.to_string())?;
        let mut result = Self {
            canvas,
            typography: Typography::new()?,
            category: Category::Home,
            entries: Vec::new(),
            focus: 0,
            pressed: None,
            settings: settings.clone(),
            paused,
            study,
            dirty: true,
            bounds,
            anchor,
            artwork: None,
        };
        result.populate();
        result.render()?;
        result.canvas.window_mut().show();
        result.canvas.window_mut().raise();
        Ok(result)
    }
    pub fn owns(&self, event: &Event) -> bool {
        event.get_window_id() == Some(self.canvas.window().id())
    }
    fn populate(&mut self) {
        self.entries = catalog::entries(self.category, &self.settings, self.paused, self.study);
        let height = layout::height(self.entries.len());
        self.artwork = None;
        if let Err(error) = self.canvas.window_mut().set_size(layout::WIDTH, height) {
            eprintln!("Pocket resize: {error}");
        }
        let point = placement::position(self.anchor, self.canvas.window().size(), self.bounds);
        self.canvas.window_mut().set_position(
            sdl3::video::WindowPos::Positioned(point.0 as i32),
            sdl3::video::WindowPos::Positioned(point.1 as i32),
        );
        self.focus = 0;
        self.pressed = None;
        self.dirty = true;
    }
    fn hit(&self, x: f32, y: f32) -> Option<usize> {
        layout::hit(x, y, self.entries.len())
    }
    fn activate(&mut self, index: usize) -> (Option<Command>, bool) {
        match self.entries.get(index).map(|e| e.action.clone()) {
            Some(Action::Do(command)) => (Some(command), true),
            Some(Action::Page(page)) => {
                self.category = page;
                self.populate();
                (None, false)
            }
            None => (None, false),
        }
    }
    pub fn handle(&mut self, event: &Event) -> (Option<Command>, bool) {
        match event {
            Event::Window {
                win_event: WindowEvent::FocusLost | WindowEvent::CloseRequested,
                ..
            } => (None, true),
            Event::Window {
                win_event: WindowEvent::Exposed | WindowEvent::PixelSizeChanged(..),
                ..
            } => {
                self.dirty = true;
                (None, false)
            }
            Event::MouseMotion { x, y, .. } => {
                if let Some(i) = self.hit(*x, *y) {
                    self.dirty |= i != self.focus;
                    self.focus = i;
                }
                (None, false)
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.pressed = self.hit(*x, *y);
                (None, false)
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                if (16. ..42.).contains(x)
                    && (17. ..43.).contains(y)
                    && self.category != Category::Home
                {
                    self.category = self.category.parent();
                    self.populate();
                    return (None, false);
                }
                if let Some(settings) =
                    layout::footer_hit(*x, *y, self.canvas.window().size().1 as f32)
                {
                    return (
                        Some(if settings {
                            Command::Settings
                        } else {
                            Command::PutAway
                        }),
                        true,
                    );
                }
                if let Some(i) = self.pressed.take()
                    && self.hit(*x, *y) == Some(i)
                {
                    return self.activate(i);
                }
                (None, false)
            }
            Event::KeyDown {
                keycode: Some(key),
                repeat: false,
                ..
            } => match key {
                Keycode::Escape => (None, true),
                Keycode::Left | Keycode::Backspace => {
                    self.category = self.category.parent();
                    self.populate();
                    (None, false)
                }
                Keycode::Down | Keycode::Tab => {
                    self.focus = (self.focus + 1) % self.entries.len();
                    self.dirty = true;
                    (None, false)
                }
                Keycode::Up => {
                    self.focus = (self.focus + self.entries.len() - 1) % self.entries.len();
                    self.dirty = true;
                    (None, false)
                }
                Keycode::Return | Keycode::Space => self.activate(self.focus),
                _ => (None, false),
            },
            _ => (None, false),
        }
    }
    pub fn render(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let size = self.canvas.window().size_in_pixels();
        let scale = size.0 as f32 / layout::WIDTH as f32;
        if self.artwork.as_ref().is_none_or(|art| art.scale != scale) {
            self.artwork = Some(paint::Artwork::new(&self.entries, scale)?);
        }
        let image = paint::draw(
            size,
            &mut self.typography,
            paint::View {
                category: self.category,
                entries: &self.entries,
                focus: self.focus,
                settings: &self.settings,
                paused: self.paused,
                artwork: self.artwork.as_ref().ok_or("Missing pocket artwork")?,
            },
        )?;
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(PixelFormat::RGBA32, size.0, size.1)
            .map_err(|e| e.to_string())?;
        texture
            .update(None, image.data(), size.0 as usize * 4)
            .map_err(|e| e.to_string())?;
        self.canvas
            .set_draw_color(sdl3::pixels::Color::RGBA(0, 0, 0, 0));
        self.canvas.clear();
        self.canvas
            .copy(&texture, None, None)
            .map_err(|e| e.to_string())?;
        self.canvas.present();
        self.dirty = false;
        Ok(())
    }
}
