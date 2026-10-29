use super::*;
use sdl3::{
    render::{BlendMode, ScaleMode, Texture, TextureCreator},
    video::WindowContext,
};

pub(super) fn create_texture(
    creator: &TextureCreator<WindowContext>,
    size: (u32, u32),
) -> Result<Texture<'_>, String> {
    let mut texture = creator
        .create_texture_streaming(PixelFormat::RGBA32, size.0, size.1)
        .map_err(|e| e.to_string())?;
    texture.set_blend_mode(BlendMode::None);
    texture.set_scale_mode(ScaleMode::Nearest);
    Ok(texture)
}

pub(super) fn instance_lock() -> Result<File, String> {
    let directory = settings::directory().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("instance.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|_| {
        "Another copy of Cinnamoroll is already running, possibly from a different folder. Click the existing pet and press Escape to quit it before opening this copy."
            .to_string()
    })?;
    Ok(file)
}

pub(super) fn apply_shape(window: &Window) -> Result<(), String> {
    let (width, height) = window.size_in_pixels();
    let mask = art::interaction_mask(width, height)?;
    apply_frame_shape(window, mask)
}

pub(super) fn apply_frame_shape(
    window: &Window,
    mut mask: tiny_skia::Pixmap,
) -> Result<(), String> {
    let (width, height) = (mask.width(), mask.height());
    let surface = Surface::from_data(
        mask.data_mut(),
        width,
        height,
        width * 4,
        PixelFormat::RGBA32,
    )
    .map_err(|e| e.to_string())?;
    window.set_shape(surface).map_err(|e| e.to_string())
}
