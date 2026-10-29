use crate::preferences::paint::{Painter, Typography};
use tiny_skia::Pixmap;
pub(super) fn render(text: &str, size: (u32, u32)) -> Result<Pixmap, String> {
    let mut image = Pixmap::new(size.0, size.1).ok_or("Cannot allocate toy hint")?;
    let mut typography = Typography::new()?;
    let mut painter = Painter {
        image: &mut image,
        typography: &mut typography,
        scale: size.0 as f32 / 180.,
    };
    painter.pill(
        [0.5, 0.5, 179., 23.],
        9.,
        [245, 251, 254],
        Some([209, 229, 239]),
    );
    painter.centered(text, [1., 1., 178., 22.], 10., [79, 113, 133]);
    Ok(image)
}
