use super::{
    catalog::{self, Category},
    layout, paint,
};
use crate::{companion::study::View, preferences::paint::Typography, settings::Settings};
use std::path::Path;

pub fn gallery(folder: &Path) -> Result<(), String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let settings = Settings::default();
    let mut typography = Typography::new()?;
    for (name, category) in [
        ("home", Category::Home),
        ("play", Category::Play),
        ("cozy", Category::Cozy),
        ("study", Category::Study),
        ("presets", Category::Presets),
        ("creative", Category::Creative),
        ("style", Category::Style),
        ("seasons", Category::Seasons),
    ] {
        let entries = catalog::entries(category, &settings, false, View::default());
        let artwork = paint::Artwork::new(&entries, 2.)?;
        let image = paint::draw(
            (layout::WIDTH * 2, layout::height(entries.len()) * 2),
            &mut typography,
            paint::View {
                category,
                entries: &entries,
                focus: 0,
                settings: &settings,
                paused: false,
                artwork: &artwork,
            },
        )?;
        image
            .save_png(folder.join(format!("{name}.png")))
            .map_err(|error| error.to_string())?;
    }
    crate::scene::preview::cozy(folder)?;
    Ok(())
}
