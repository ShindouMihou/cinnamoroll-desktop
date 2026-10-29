use crate::{
    art::{self, Animation, Decoration},
    behavior::Activity,
    companion::corner::Layout,
};
use std::path::Path;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

pub(crate) fn cozy(folder: &Path) -> Result<(), String> {
    art::decoration(Decoration::Corner(0), (640, 224), 0.)?
        .save_png(folder.join("cozy-bed.png"))
        .map_err(|error| error.to_string())?;
    let pet_size = (256, 179);
    let layout = Layout::new(pet_size);
    let bed = art::decoration(Decoration::Corner(0), layout.size, 0.)?;
    let mut image = Pixmap::new(layout.footprint.0 + 40, layout.footprint.1 + 40)
        .ok_or("Cannot allocate cozy preview")?;
    image.fill(tiny_skia::Color::from_rgba8(248, 252, 255, 255));
    image.draw_pixmap(
        20,
        20 + layout.offset.1.round() as i32,
        bed.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    let animation = Animation::new(Activity::BlanketBurrow, 1., pet_size)?;
    image.draw_pixmap(
        20 - layout.offset.0.round() as i32,
        20,
        animation.frame(5., false).as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    image
        .save_png(folder.join("tucked-in.png"))
        .map_err(|error| error.to_string())
}
