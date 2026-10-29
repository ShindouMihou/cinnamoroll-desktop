use super::shapes::{BLUE, OUTLINE, PINK, Painter, WHITE};
use crate::behavior::Activity;
use tiny_skia::PathBuilder;

pub fn behind(p: &mut Painter<'_>, activity: Activity, phase: f32) {
    if activity == Activity::Nap {
        super::bed::behind(p);
    }
    if activity == Activity::Listen {
        p.curve(
            (110., 128.),
            [104., 25., 216., 25., 210., 128.],
            [167, 198, 230, 255],
            9.,
        );
        p.curve(
            (110., 128.),
            [104., 25., 216., 25., 210., 128.],
            [224, 238, 254, 255],
            4.,
        );
    }
    if matches!(
        activity,
        Activity::Happy | Activity::Greet | Activity::Petting | Activity::Excited | Activity::Shy
    ) {
        p.heart(
            78.,
            55. - phase * 18.,
            7.,
            ((1. - phase * 0.5) * 255.) as u8,
        );
        p.heart(239., 44. - phase * 15., 5., 220);
    }
}

pub fn foreground(p: &mut Painter<'_>, activity: Activity, phase: f32, wave: f32) {
    match activity {
        Activity::Pant => {
            let breath = (phase * 2.) % 1.;
            let x = 205. + breath * 17.;
            let y = 144. - breath * 4.;
            let color = [192, 222, 235, ((1. - breath) * 180.) as u8];
            p.ellipse(x, y, 5. + breath * 4., 3. + breath * 2., color, false);
            p.ellipse(
                x + 5.,
                y - 3.,
                4. + breath * 3.,
                3. + breath * 2.,
                color,
                false,
            );
            p.ellipse(212., 102., 3., 5., [149, 206, 230, 235], false);
        }
        Activity::Dizzy => {
            for index in 0..3 {
                let angle =
                    phase * std::f32::consts::TAU + index as f32 * std::f32::consts::TAU / 3.;
                let x = 160. + angle.cos() * 32.;
                let y = 79. + angle.sin() * 8.;
                p.line(&[(x - 4., y), (x + 4., y)], [239, 191, 103, 255], 2.5);
                p.line(&[(x, y - 4.), (x, y + 4.)], [239, 191, 103, 255], 2.5);
            }
        }
        Activity::TypingPeek => {
            p.cloud(238., 77., 15.);
            p.curve((232., 70.), [232., 60., 249., 62., 243., 72.], BLUE, 2.5);
            p.curve((243., 72.), [241., 75., 237., 74., 237., 79.], BLUE, 2.5);
            p.ellipse(237., 84., 1.5, 1.5, BLUE, false);
        }
        Activity::Boop => p.heart(
            160.,
            65. - phase * 12.,
            6.,
            ((1. - phase * 0.6) * 255.) as u8,
        ),
        Activity::Listen => {
            for x in [110., 210.] {
                p.ellipse(x, 128., 9., 17., [157, 191, 226, 255], true);
                p.ellipse(x, 128., 5., 12., [224, 238, 254, 255], false);
            }
            note(p, 247., 79. - wave * 5., 1.);
            note(p, 74., 48. + wave * 5., 0.7);
        }
        Activity::Dance => {
            note(p, 247., 62. - wave * 7., 1.);
            note(p, 74., 51. + wave * 4., 0.7);
        }
        Activity::Nap | Activity::Drowsy => {
            if activity == Activity::Nap {
                super::bed::foreground(p);
            }
            let x = 223. + phase * 12.;
            let y = 59. - phase * 17.;
            p.line(
                &[(x, y), (x + 11., y), (x, y + 12.), (x + 11., y + 12.)],
                BLUE,
                2.5,
            );
            p.line(
                &[
                    (x + 17., y - 14.),
                    (x + 25., y - 14.),
                    (x + 17., y - 5.),
                    (x + 25., y - 5.),
                ],
                OUTLINE,
                2.,
            );
        }
        Activity::Snack => {
            p.ellipse(160., 169., 25., 17., [240, 203, 157, 255], true);
            let mut spiral = Vec::with_capacity(40);
            for i in 0..40 {
                let t = i as f32 / 39. * std::f32::consts::TAU * 1.8;
                let radius = 2. + i as f32 * 0.3;
                spiral.push((160. + t.cos() * radius, 168. + t.sin() * radius * 0.65));
            }
            p.line(&spiral, [186, 134, 99, 255], 2.2);
        }
        Activity::Read => {
            let mut book = PathBuilder::new();
            book.move_to(126., 151.);
            book.line_to(160., 158.);
            book.line_to(194., 151.);
            book.line_to(194., 184.);
            book.line_to(160., 191.);
            book.line_to(126., 184.);
            book.close();
            if let Some(book) = book.finish() {
                p.path(&book, Some([183, 208, 240, 255]), Some((BLUE, 1.5)));
            }
            p.line(&[(160., 159.), (160., 189.)], WHITE, 2.);
            p.line(&[(133., 162.), (152., 166.)], WHITE, 1.5);
            p.line(&[(168., 166.), (187., 162.)], WHITE, 1.5);
        }
        Activity::Play => {
            let y = 146. + wave * 2.;
            p.cloud(210., y, 14.);
            p.ellipse(207., y - 3., 1., 1.7, BLUE, false);
            p.ellipse(216., y - 3., 1., 1.7, BLUE, false);
        }
        Activity::Happy | Activity::Petting => {
            p.heart(160., 164., 11. + wave, 255);
        }
        Activity::Stretch => {
            p.line(&[(74., 62.), (69., 55.)], PINK, 2.);
            p.line(&[(246., 62.), (251., 55.)], PINK, 2.);
        }
        _ => {}
    }
}

fn note(p: &mut Painter<'_>, x: f32, y: f32, scale: f32) {
    p.ellipse(x, y, 4.5 * scale, 3.3 * scale, BLUE, false);
    p.line(
        &[
            (x + 3.5 * scale, y),
            (x + 3.5 * scale, y - 17. * scale),
            (x + 11. * scale, y - 14. * scale),
        ],
        BLUE,
        2.5 * scale,
    );
}
