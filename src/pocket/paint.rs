use super::{
    catalog::{Action, Category, Entry},
    icons, layout,
};
use crate::{
    behavior::Activity,
    preferences::paint::{INK, Painter, Typography},
    settings::Settings,
    ui::motifs,
};
use tiny_skia::{Pixmap, PixmapPaint, Transform};

pub(super) struct Artwork {
    pub scale: f32,
    entries: Vec<Pixmap>,
    hero: Pixmap,
}

impl Artwork {
    pub fn new(entries: &[Entry], scale: f32) -> Result<Self, String> {
        Ok(Self {
            scale,
            entries: entries
                .iter()
                .map(|entry| icons::entry(entry, scale))
                .collect::<Result<_, _>>()?,
            hero: icons::character(Activity::Wave, scale * 2.)?,
        })
    }
}

pub(super) struct View<'a> {
    pub category: Category,
    pub entries: &'a [Entry],
    pub focus: usize,
    pub settings: &'a Settings,
    pub paused: bool,
    pub artwork: &'a Artwork,
}

fn stamp(p: &mut Painter<'_>, image: &Pixmap, x: f32, y: f32) {
    p.image.draw_pixmap(
        (x * p.scale).round() as i32,
        (y * p.scale).round() as i32,
        image.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

pub(super) fn draw(
    size: (u32, u32),
    typography: &mut Typography,
    view: View<'_>,
) -> Result<Pixmap, String> {
    let mut image = Pixmap::new(size.0, size.1).ok_or("Cannot allocate little pocket")?;
    let scale = size.0 as f32 / layout::WIDTH as f32;
    let height = size.1 as f32 / scale;
    let mut p = Painter {
        image: &mut image,
        typography,
        scale,
    };
    p.pill(
        [0.5, 0.5, 283., height - 1.],
        20.,
        [248, 252, 255],
        Some([203, 224, 235]),
    );
    for y in (28..(height as usize).saturating_sub(70)).step_by(56) {
        motifs::cloud(&mut p, 2., y as f32, [236, 246, 251]);
        motifs::heart(&mut p, 271., y as f32 + 22., [249, 234, 241]);
    }
    p.pill([8., 8., 268., 76.], 16., [228, 244, 251], None);
    motifs::cloud(&mut p, 239., 56., [249, 253, 255]);
    let (title, subtitle) = match view.category {
        Category::Home => ("Little pocket", "What shall we do today?"),
        Category::Play => ("Play", "Tiny games, happy paws."),
        Category::Cozy => ("Cozy time", "Something soft, something sweet."),
        Category::Study => ("Study together", "One little step at a time."),
        Category::Presets => ("Subject presets", "Pick your study rhythm."),
        Category::Creative => ("Creative corner", "A little spark of inspiration."),
        Category::Style => ("Dress up", "A little something to wear."),
        Category::Seasons => ("Seasonal moments", "Small joys for today."),
    };
    if view.category == Category::Home {
        p.text(title, 20., 23., 17., INK);
        p.text(subtitle, 20., 49., 10.5, [88, 124, 145]);
        motifs::heart(&mut p, 163., 24., [239, 190, 208]);
        stamp(&mut p, &view.artwork.hero, 190., 10.);
    } else {
        p.pill(
            [16., 17., 26., 26.],
            9.,
            [247, 252, 255],
            Some([207, 228, 239]),
        );
        motifs::chevron(&mut p, 29., 30., true);
        p.fitted(title, [50., 21., 210., 23.], 16., INK);
        p.text(subtitle, 20., 56., 10.5, [88, 124, 145]);
    }
    for x in (22..266).step_by(9) {
        p.pill([x as f32, 78., 4., 1.], 0.5, [189, 217, 232], None);
    }
    for (index, entry) in view.entries.iter().enumerate() {
        let rect = layout::row(index);
        let y = rect[1];
        let selected = icons::selected(entry, view.settings, view.paused);
        let focused = index == view.focus;
        p.pill(
            rect,
            12.,
            if focused {
                [230, 244, 251]
            } else {
                [255, 255, 255]
            },
            Some(if focused {
                [119, 177, 204]
            } else {
                [231, 239, 245]
            }),
        );
        let tint = match index % 3 {
            0 => [231, 242, 253],
            1 => [252, 236, 243],
            _ => [241, 236, 252],
        };
        p.pill([20., y + 5., 28., 28.], 10., tint, None);
        let icon = &view.artwork.entries[index];
        stamp(
            &mut p,
            icon,
            34. - icon.width() as f32 / scale / 2.,
            y + 19. - icon.height() as f32 / scale / 2.,
        );
        p.fitted(&entry.label, [60., y + 12., 182., 21.], 12., INK);
        if selected {
            motifs::heart(&mut p, 253., y + 13., [222, 151, 178]);
        } else if matches!(entry.action, Action::Page(_)) {
            motifs::chevron(&mut p, 253., y + 19., false);
        }
    }
    let footer = height - layout::FOOTER as f32;
    motifs::cloud(&mut p, 132., footer + 2., [231, 243, 250]);
    p.pill(
        [12., footer + 16., 124., 30.],
        10.,
        [231, 244, 251],
        Some([214, 232, 242]),
    );
    p.pill(
        [148., footer + 16., 124., 30.],
        10.,
        [253, 239, 245],
        Some([239, 220, 230]),
    );
    p.centered(
        "Settings",
        [12., footer + 16., 124., 30.],
        11.,
        [67, 111, 136],
    );
    p.centered(
        "Tidy toys",
        [148., footer + 16., 124., 30.],
        11.,
        [131, 89, 111],
    );
    Ok(image)
}
