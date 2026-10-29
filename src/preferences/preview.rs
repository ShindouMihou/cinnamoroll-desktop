use super::{
    layout::{self, Page},
    paint::{Painter, Typography},
};
use crate::{behavior::Activity, companion::study::View, settings::Settings};
use std::path::Path;
use tiny_skia::Pixmap;

pub fn gallery(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let mut typography = Typography::new()?;
    let artwork = super::artwork::Artwork::new(2.)?;
    let updates = if crate::updates::connected() {
        crate::updates::State::Idle
    } else {
        crate::updates::State::Disconnected
    };
    for (tab, title) in layout::TITLES
        .iter()
        .copied()
        .chain([
            "Session plan",
            "Creative tools",
            "Surprise items",
            "App settings",
        ])
        .enumerate()
    {
        let mut image = Pixmap::new(layout::WIDTH * 2, layout::HEIGHT * 2)
            .ok_or("Cannot allocate control panel preview")?;
        image.fill(tiny_skia::Color::from_rgba8(246, 251, 253, 255));
        Page {
            updates: &updates,
            startup: crate::native::startup::Status::On,
            artwork: &artwork,
            painter: &mut Painter {
                image: &mut image,
                typography: &mut typography,
                scale: 2.,
            },
            controls: Vec::new(),
            focus: None,
            hover: None,
        }
        .draw(
            tab,
            &Settings::default(),
            false,
            Activity::Idle,
            View::default(),
            "Changes save automatically.",
        );
        image
            .save_png(path.join(format!("{}.png", title.to_lowercase().replace(' ', "-"))))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
