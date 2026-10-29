use std::path::Path;
use tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint, Transform};

pub fn prepare(source: &Path, folder: &Path) -> Result<(), String> {
    let sheet = Pixmap::load_png(source).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let mut cells = Vec::with_capacity(6);
    for (i, name) in [
        "brush",
        "dandelion",
        "globe",
        "pond",
        "box-closed",
        "box-open",
    ]
    .into_iter()
    .enumerate()
    {
        let left = i as u32 % 3 * sheet.width() / 3;
        let right = (i as u32 % 3 + 1) * sheet.width() / 3;
        let top = i as u32 / 3 * sheet.height() / 2;
        let bottom = (i as u32 / 3 + 1) * sheet.height() / 2;
        let cell = sheet
            .clone_rect(
                IntRect::from_xywh(left as i32, top as i32, right - left, bottom - top)
                    .ok_or("Invalid prop cell")?,
            )
            .ok_or("Missing prop cell")?;
        let border = (0..cell.width()).any(|x| {
            [0, cell.height() - 1]
                .into_iter()
                .any(|y| cell.pixel(x, y).is_some_and(|p| p.alpha() > 32))
        }) || (0..cell.height()).any(|y| {
            [0, cell.width() - 1]
                .into_iter()
                .any(|x| cell.pixel(x, y).is_some_and(|p| p.alpha() > 32))
        });
        if border {
            return Err(format!(
                "Prop {name} crosses a cell boundary; regenerate with larger transparent gutters"
            ));
        }
        let mut bounds = (cell.width(), cell.height(), 0, 0);
        for y in 0..cell.height() {
            for x in 0..cell.width() {
                if cell.pixel(x, y).is_some_and(|p| p.alpha() > 20) {
                    bounds = (
                        bounds.0.min(x),
                        bounds.1.min(y),
                        bounds.2.max(x),
                        bounds.3.max(y),
                    );
                }
            }
        }
        if bounds.2 <= bounds.0 || bounds.3 <= bounds.1 {
            return Err(format!("Empty prop: {name}"));
        }
        let cell = cell
            .clone_rect(
                IntRect::from_xywh(
                    bounds.0 as i32,
                    bounds.1 as i32,
                    bounds.2 - bounds.0 + 1,
                    bounds.3 - bounds.1 + 1,
                )
                .ok_or("Invalid prop bounds")?,
            )
            .ok_or("Missing cropped prop")?;
        cells.push((name, cell));
    }
    let box_scale = cells[4..]
        .iter()
        .map(|(_, c)| (432. / c.width() as f32).min(294. / c.height() as f32))
        .fold(f32::INFINITY, f32::min);
    for (index, (name, cell)) in cells.into_iter().enumerate() {
        let scale = if index >= 4 {
            box_scale
        } else {
            (432. / cell.width() as f32).min(294. / cell.height() as f32)
        };
        let mut image = Pixmap::new(480, 336).ok_or("Cannot allocate toy artwork")?;
        image.draw_pixmap(
            0,
            0,
            cell.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..Default::default()
            },
            Transform::from_translate(
                240. - (cell.width() - 1) as f32 / 2. * scale,
                318. - (cell.height() - 1) as f32 * scale,
            )
            .pre_scale(scale, scale),
            None,
        );
        image
            .save_png(folder.join(format!("{name}.png")))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
