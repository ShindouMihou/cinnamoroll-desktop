use crate::behavior::Activity;
use tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint};

struct Cell {
    image: Pixmap,
    bounds: (u32, u32, u32, u32),
}

fn frames_from_source(
    bytes: &[u8],
    activity: Activity,
    size: (u32, u32),
) -> Result<Option<Vec<Pixmap>>, String> {
    let sheet = Pixmap::decode_png(bytes).map_err(|error| error.to_string())?;
    if sheet.width() < 4 || sheet.height() < 2 {
        return Err("Animation strip is too small".into());
    }
    let mut cells = Vec::with_capacity(8);
    for index in 0..8 {
        let left = index % 4 * sheet.width() / 4;
        let right = (index % 4 + 1) * sheet.width() / 4;
        let top = index / 4 * sheet.height() / 2;
        let bottom = (index / 4 + 1) * sheet.height() / 2;
        let rect = IntRect::from_xywh(left as i32, top as i32, right - left, bottom - top)
            .ok_or("Invalid animation frame")?;
        let image = sheet.clone_rect(rect).ok_or("Missing animation frame")?;
        cells.push(isolate(image)?);
    }
    let widest = cells
        .iter()
        .map(|cell| cell.bounds.2 - cell.bounds.0 + 1)
        .max()
        .ok_or("Empty animation strip")?;
    let tallest = cells
        .iter()
        .map(|cell| cell.bounds.3 - cell.bounds.1 + 1)
        .max()
        .ok_or("Empty animation strip")?;
    let fixed = super::strip_catalog::authored_origin(activity);
    let union = cells
        .iter()
        .fold((u32::MAX, u32::MAX, 0, 0), |bounds, cell| {
            (
                bounds.0.min(cell.bounds.0),
                bounds.1.min(cell.bounds.1),
                bounds.2.max(cell.bounds.2),
                bounds.3.max(cell.bounds.3),
            )
        });
    let width = if fixed { union.2 - union.0 + 1 } else { widest };
    let height = if fixed {
        union.3 - union.1 + 1
    } else {
        tallest
    };
    let height_budget = match activity {
        Activity::Pounce | Activity::RainPlay => 192.,
        Activity::EarWiggle => 180.,
        Activity::Stalk | Activity::Stumble => 145.,
        _ => 160.,
    };
    let scale = (234. / width as f32).min(height_budget / height as f32);
    let baseline = if activity == Activity::Nap {
        212.
    } else {
        197.
    };
    cells
        .into_iter()
        .map(|cell| {
            let mut frame = Pixmap::new(size.0, size.1).ok_or("Cannot allocate animation frame")?;
            let center = if fixed {
                (union.0 + union.2) as f32 / 2.
            } else {
                (cell.bounds.0 + cell.bounds.2) as f32 / 2.
            };
            let bottom = if fixed && activity != Activity::BlanketBurrow {
                union.3
            } else {
                cell.bounds.3
            };
            let transform = super::viewport_transform(size)
                .pre_translate(160. - center * scale, baseline - bottom as f32 * scale)
                .pre_scale(scale, scale);
            frame.draw_pixmap(
                0,
                0,
                cell.image.as_ref(),
                &PixmapPaint {
                    quality: FilterQuality::Bilinear,
                    ..PixmapPaint::default()
                },
                transform,
                None,
            );
            Ok(frame)
        })
        .collect::<Result<Vec<_>, String>>()
        .map(Some)
}

fn isolate(mut image: Pixmap) -> Result<Cell, String> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut visited = vec![false; width * height];
    let mut largest = Vec::new();
    let mut component = Vec::new();
    for start in 0..visited.len() {
        if visited[start] || image.data()[start * 4 + 3] < 12 {
            continue;
        }
        component.clear();
        component.push(start);
        visited[start] = true;
        let mut next = 0;
        while next < component.len() {
            let index = component[next];
            next += 1;
            let (x, y) = (index % width, index / width);
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let neighbor = ny * width + nx;
                    if !visited[neighbor] && image.data()[neighbor * 4 + 3] >= 12 {
                        visited[neighbor] = true;
                        component.push(neighbor);
                    }
                }
            }
        }
        if component.len() > largest.len() {
            std::mem::swap(&mut largest, &mut component);
        }
    }
    if largest.len() < 1000 {
        return Err("An animation frame has no complete character".into());
    }
    let mut keep = vec![false; width * height];
    let mut bounds = (image.width(), image.height(), 0, 0);
    for index in largest {
        keep[index] = true;
        let (x, y) = ((index % width) as u32, (index / width) as u32);
        bounds = (
            bounds.0.min(x),
            bounds.1.min(y),
            bounds.2.max(x),
            bounds.3.max(y),
        );
    }
    for (index, pixel) in image
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        if !keep[index] {
            pixel.fill(0);
        }
    }
    Ok(Cell { image, bounds })
}

pub fn prepare(source: &std::path::Path, directory: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    for strip in super::strip_catalog::STRIPS {
        let (activity, name) = (strip.activity, strip.name);
        let bytes =
            std::fs::read(source.join(format!("{name}.png"))).map_err(|error| error.to_string())?;
        let frames =
            frames_from_source(&bytes, activity, (320, 224))?.ok_or("No dedicated strip")?;
        let mut sheet = Pixmap::new(1280, 448).ok_or("Cannot allocate normalized strip")?;
        for (index, frame) in frames.into_iter().enumerate() {
            sheet.draw_pixmap(
                (index % 4 * 320) as i32,
                (index / 4 * 224) as i32,
                frame.as_ref(),
                &PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }
        sheet
            .save_png(directory.join(format!("{name}.png")))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn prepare_bed(source: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    let bytes = std::fs::read(source).map_err(|error| error.to_string())?;
    let cell = isolate(Pixmap::decode_png(&bytes).map_err(|error| error.to_string())?)?;
    let width = (cell.bounds.2 - cell.bounds.0 + 1) as f32;
    let height = (cell.bounds.3 - cell.bounds.1 + 1) as f32;
    let scale = (920. / width).min(320. / height);
    let center = (cell.bounds.0 + cell.bounds.2) as f32 / 2.;
    let transform = tiny_skia::Transform::from_translate(
        480. - center * scale,
        328. - cell.bounds.3 as f32 * scale,
    )
    .pre_scale(scale, scale);
    let mut image = Pixmap::new(960, 336).ok_or("Cannot allocate prepared bed")?;
    image.draw_pixmap(
        0,
        0,
        cell.image.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        transform,
        None,
    );
    image.save_png(target).map_err(|error| error.to_string())
}

pub fn prepare_play_assets(folder: &std::path::Path) -> Result<(), String> {
    for name in ["ribbon", "wand"] {
        let bytes =
            std::fs::read(folder.join(format!("{name}-source.png"))).map_err(|e| e.to_string())?;
        let cell = isolate(Pixmap::decode_png(&bytes).map_err(|e| e.to_string())?)?;
        let scale = (448. / (cell.bounds.2 - cell.bounds.0 + 1) as f32)
            .min(288. / (cell.bounds.3 - cell.bounds.1 + 1) as f32);
        let center = (cell.bounds.0 + cell.bounds.2) as f32 / 2.;
        let mut image = Pixmap::new(480, 336).ok_or("Cannot allocate toy artwork")?;
        image.draw_pixmap(
            0,
            0,
            cell.image.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..PixmapPaint::default()
            },
            tiny_skia::Transform::from_translate(
                240. - center * scale,
                312. - cell.bounds.3 as f32 * scale,
            )
            .pre_scale(scale, scale),
            None,
        );
        image
            .save_png(folder.join(format!("{name}.png")))
            .map_err(|e| e.to_string())?;
    }
    let source = Pixmap::load_png(folder.join("cups-source.png")).map_err(|e| e.to_string())?;
    let mut cells = Vec::with_capacity(4);
    for index in 0..4 {
        let rect = IntRect::from_xywh(
            index % 2 * (source.width() / 2) as i32,
            index / 2 * (source.height() / 2) as i32,
            source.width() / 2,
            source.height() / 2,
        )
        .ok_or("Invalid cup atlas")?;
        cells.push(isolate(source.clone_rect(rect).ok_or("Missing cup cell")?)?);
    }
    let widest = cells
        .iter()
        .map(|c| c.bounds.2 - c.bounds.0 + 1)
        .max()
        .ok_or("No cups")?;
    let tallest = cells
        .iter()
        .map(|c| c.bounds.3 - c.bounds.1 + 1)
        .max()
        .ok_or("No cups")?;
    let scale = (220. / widest as f32).min(208. / tallest as f32);
    let mut atlas = Pixmap::new(512, 512).ok_or("Cannot allocate cup atlas")?;
    for (index, cell) in cells.into_iter().enumerate() {
        let center = (cell.bounds.0 + cell.bounds.2) as f32 / 2.;
        let x = (index % 2 * 256) as f32 + 128. - center * scale;
        let y = (index / 2 * 256) as f32 + 236. - cell.bounds.3 as f32 * scale;
        atlas.draw_pixmap(
            0,
            0,
            cell.image.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..PixmapPaint::default()
            },
            tiny_skia::Transform::from_translate(x, y).pre_scale(scale, scale),
            None,
        );
    }
    atlas
        .save_png(folder.join("cups.png"))
        .map_err(|e| e.to_string())
}
