use super::shapes::{BLUE, Painter, WHITE};
use tiny_skia::PathBuilder;

pub fn behind(p: &mut Painter<'_>) {
    p.ellipse(160., 198., 77., 19., [175, 210, 226, 230], false);
    p.ellipse(160., 190., 77., 23., [200, 229, 242, 255], true);
    p.ellipse(160., 185., 68., 19., [241, 249, 253, 255], true);
    p.ellipse(201., 175., 27., 12., WHITE, true);
    p.curve(
        (183., 175.),
        [191., 170., 204., 170., 217., 175.],
        [219, 235, 244, 255],
        1.5,
    );
}

pub fn foreground(p: &mut Painter<'_>) {
    let mut blanket = PathBuilder::new();
    blanket.move_to(98., 184.);
    blanket.cubic_to(120., 193., 143., 180., 162., 186.);
    blanket.cubic_to(181., 192., 202., 181., 222., 184.);
    blanket.cubic_to(227., 202., 201., 212., 160., 212.);
    blanket.cubic_to(119., 212., 94., 202., 98., 184.);
    blanket.close();
    if let Some(path) = blanket.finish() {
        p.path(&path, Some([193, 222, 242, 255]), Some((BLUE, 1.6)));
    }
    p.curve(
        (102., 188.),
        [135., 196., 183., 186., 218., 188.],
        WHITE,
        2.5,
    );
    for x in [116., 129., 142., 155., 168., 181., 194., 207.] {
        p.line(&[(x, 204.), (x + 3., 205.)], [237, 247, 252, 255], 1.2);
    }
    p.heart(160., 198., 5., 245);
    for (x, y) in [(124., 196.), (197., 197.)] {
        p.line(&[(x - 3., y), (x + 3., y)], WHITE, 1.5);
        p.line(&[(x, y - 3.), (x, y + 3.)], WHITE, 1.5);
    }
}
