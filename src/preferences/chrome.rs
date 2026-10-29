use super::{
    artwork::section_pose,
    layout::{Action, Control, HEIGHT, LEFT, Page, RIGHT, TITLES},
    paint::{BLUE, BORDER, INK, MUTED, SOFT},
};
use crate::{behavior::Activity, ui::motifs};

impl Page<'_, '_> {
    pub(super) fn navigation(&mut self, tab: usize) {
        self.painter
            .pill([0., 0., 168., HEIGHT as f32], 0., [237, 247, 252], None);
        self.painter
            .pill([167., 0., 1., HEIGHT as f32], 0., BORDER, None);
        self.painter
            .pill([8., 8., 152., 105.], 17., [228, 244, 251], None);
        motifs::cloud(self.painter, 16., 23., [245, 252, 255]);
        motifs::heart(self.painter, 139., 47., [245, 213, 226]);
        self.artwork
            .draw(self.painter, Activity::Wave, [28., 7., 110., 76.]);
        self.painter
            .centered("Cinnamoroll", [12., 84., 144., 24.], 16., INK);
        self.painter.text("Together", 18., 130., 11., MUTED);
        for (index, title) in TITLES.iter().copied().chain(["App settings"]).enumerate() {
            let page = if index == 8 { 11 } else { index };
            let y = if index < 4 {
                153. + index as f32 * 39.
            } else {
                354. + (index - 4) as f32 * 39.
            };
            let rect = [12., y, 144., 33.];
            let focused = self.focus == Some(index);
            if page == tab || focused || self.hover == Some(index) {
                self.painter.pill(
                    rect,
                    11.,
                    if page == tab { [216, 237, 248] } else { SOFT },
                    if focused { Some(BLUE) } else { None },
                );
            }
            self.painter.text(
                title,
                24.,
                y + 8.,
                12.5,
                if page == tab { BLUE } else { INK },
            );
            self.controls.push(Control {
                rect,
                action: Action::Tab(page),
                enabled: true,
            });
        }
        self.painter.text("Preferences", 18., 331., 11., MUTED);
        self.painter
            .text("Tab / Enter · Esc to close", 18., 559., 9., MUTED);
    }
    pub(super) fn header(&mut self, tab: usize) {
        self.painter
            .pill([184., 12., 520., 74.], 15., [231, 245, 252], None);
        motifs::cloud(self.painter, 198., 57., [246, 252, 255]);
        self.artwork
            .draw(self.painter, section_pose(tab), [190., 14., 72., 60.]);
        let title = match tab {
            8 => "Session plan",
            9 => "Creative tools",
            10 => "Surprise items",
            11 => "App settings",
            _ => TITLES[tab],
        };
        self.painter.text(title, 270., 25., 22., INK);
        let help = match tab {
            0 => "A little company, whenever you need it.",
            1 => "Settle in for a quiet focus session.",
            2 => "A few small ways to play together.",
            3 => "Choose a little moment to enjoy.",
            4 => "A soft place to call home.",
            5 => "Keep him comfortable on your desktop.",
            6 => "Give everyday activities a gentle pace.",
            7 => "Choose how he reacts to your computer.",
            8 => "One small goal and your favorite study rhythms.",
            9 => "Colors to play with, moments to keep.",
            11 => "Startup and updates, kept together.",
            _ => "Little things to discover and offer together.",
        };
        self.painter
            .fitted(help, [270., 59., RIGHT - 282., 18.], 11., [91, 124, 144]);
        for x in (198..694).step_by(10) {
            self.painter
                .pill([x as f32, 83., 4., 1.], 0.5, [195, 222, 236], None);
        }
        for y in [126., 262., 399.] {
            motifs::cloud(self.painter, 706., y, [238, 247, 252]);
            motifs::heart(self.painter, 713., y + 62., [251, 237, 244]);
        }
    }
    pub(super) fn footer(&mut self, status: &str) {
        self.painter
            .pill([184., 541., 520., 31.], 10., [236, 247, 252], None);
        motifs::heart(self.painter, 197., 550., [237, 199, 215]);
        motifs::cloud(self.painter, 667., 548., [247, 252, 255]);
        self.painter
            .fitted(status, [LEFT + 18., 551., 446., 15.], 11., MUTED);
    }
}
