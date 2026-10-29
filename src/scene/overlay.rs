use crate::art::{self, Decoration};
use sdl3::{
    VideoSubsystem,
    pixels::{Color, PixelFormat},
    render::{BlendMode, WindowCanvas},
    surface::Surface,
    video::{WindowFlags, WindowPos},
};

pub(super) struct Overlay {
    pub canvas: WindowCanvas,
    visible: bool,
    key: Option<(Decoration, (u32, u32), u64)>,
    position: Option<(i32, i32)>,
}

impl Overlay {
    pub fn new(video: &VideoSubsystem, title: &str) -> Result<Self, String> {
        let window = video
            .window(title, 96, 80)
            .set_flags(
                WindowFlags::BORDERLESS
                    | WindowFlags::TRANSPARENT
                    | WindowFlags::ALWAYS_ON_TOP
                    | WindowFlags::UTILITY
                    | WindowFlags::HIGH_PIXEL_DENSITY
                    | WindowFlags::HIDDEN,
            )
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = sdl3::render::create_renderer(window, None).map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        Ok(Self {
            canvas,
            visible: false,
            key: None,
            position: None,
        })
    }
    pub fn id(&self) -> u32 {
        self.canvas.window().id()
    }
    pub fn invalidate(&mut self) {
        self.key = None;
    }
    pub fn show(
        &mut self,
        kind: Decoration,
        size: (u32, u32),
        position: (f32, f32),
        tick: u64,
    ) -> Result<(), String> {
        if self.canvas.window().size() != size {
            self.canvas
                .window_mut()
                .set_size(size.0, size.1)
                .map_err(|e| e.to_string())?;
            self.key = None;
        }
        let position = (position.0.round() as i32, position.1.round() as i32);
        if self.position != Some(position) {
            self.canvas.window_mut().set_position(
                WindowPos::Positioned(position.0),
                WindowPos::Positioned(position.1),
            );
            self.position = Some(position);
        }
        let pixels = self.canvas.window().size_in_pixels();
        let key = (kind, pixels, tick);
        if self.key != Some(key) {
            let mut image = art::decoration(kind, pixels, tick as f32 / 10.)?;
            {
                let surface = Surface::from_data(
                    image.data_mut(),
                    pixels.0,
                    pixels.1,
                    pixels.0 * 4,
                    PixelFormat::RGBA32,
                )
                .map_err(|e| e.to_string())?;
                self.canvas
                    .window()
                    .set_shape(surface)
                    .map_err(|e| e.to_string())?;
            }
            let creator = self.canvas.texture_creator();
            let mut texture = creator
                .create_texture_streaming(PixelFormat::RGBA32, pixels.0, pixels.1)
                .map_err(|e| e.to_string())?;
            texture.set_blend_mode(BlendMode::None);
            texture
                .update(None, image.data(), pixels.0 as usize * 4)
                .map_err(|e| e.to_string())?;
            self.canvas.clear();
            self.canvas
                .copy(&texture, None, None)
                .map_err(|e| e.to_string())?;
            self.canvas.present();
            self.key = Some(key);
        }
        if !self.visible {
            self.canvas.window_mut().show();
            self.visible = true;
        }
        Ok(())
    }
}
