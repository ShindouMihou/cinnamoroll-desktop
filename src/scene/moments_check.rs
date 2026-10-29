use super::*;
use crate::companion::world::Item;

pub(crate) fn run() -> Result<(), String> {
    let sdl = sdl3::init().map_err(|e| e.to_string())?;
    let video = sdl.video().map_err(|e| e.to_string())?;
    let window = video
        .window("Cinnamoroll · Native moment check", 128, 89)
        .hidden()
        .build()
        .map_err(|e| e.to_string())?;
    let bounds = crate::native::displays(&video)?
        .into_iter()
        .next()
        .ok_or("No display for native check")?;
    let pet = (bounds.x as f32 + 120., bounds.y as f32 + 120.);
    let mut c = Companion::default();
    let mut s = Settings::default();
    let mut scene = Scene::new(&video);
    for kind in [
        Kind::Brush,
        Kind::Dandelion,
        Kind::SnowGlobe,
        Kind::Pond,
        Kind::KeepsakeBox,
        Kind::ClearZone,
    ] {
        c.item = Some(Item::new(kind, (pet.0 + 200., pet.1 + 60.), 0.));
        for stage in [0, 1, 50, 143] {
            c.item.as_mut().ok_or("Missing check item")?.stage = stage;
            scene.render(Frame {
                settings: &s,
                companion: &c,
                pet,
                size: (128, 89),
                bounds,
                hidden: false,
                now: 1.,
                pet_window: &window,
            })?;
        }
    }
    let item = scene.item.as_ref().ok_or("Missing zone overlay")?.id();
    let event = Event::MouseButtonDown {
        timestamp: 0,
        window_id: item,
        which: 0,
        mouse_btn: MouseButton::Right,
        clicks: 1,
        x: 20.,
        y: 20.,
    };
    let (_, saved) = scene.handle(
        &event,
        &mut c,
        &mut s,
        2.,
        InputPosition { pointer: pet, pet },
        &sdl.mouse(),
    );
    if !saved || s.extras.comfort.zones.len() != 1 {
        return Err("Native zone placement did not save".into());
    }
    c.moments.dream = Some((10., 0));
    scene.render(Frame {
        settings: &s,
        companion: &c,
        pet,
        size: (128, 89),
        bounds,
        hidden: false,
        now: 3.,
        pet_window: &window,
    })?;
    let dream = scene.dream.as_ref().ok_or("Missing dream overlay")?.id();
    let event = Event::MouseButtonDown {
        timestamp: 0,
        window_id: dream,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x: 40.,
        y: 25.,
    };
    scene.handle(
        &event,
        &mut c,
        &mut s,
        4.,
        InputPosition { pointer: pet, pet },
        &sdl.mouse(),
    );
    if c.moments.dream != Some((6., 3)) {
        return Err("Dream tap did not respond".into());
    }
    scene.reset();
    println!("Native moment windows, hit shapes, keep-clear placement and dream taps passed");
    Ok(())
}
