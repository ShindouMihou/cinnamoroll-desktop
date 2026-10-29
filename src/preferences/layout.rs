use super::paint::{BLUE, BORDER, INK, MUTED, Painter, SOFT};
use crate::{behavior::Activity, command::Command, settings::Settings};

pub const WIDTH: u32 = 720;
pub const HEIGHT: u32 = 580;
pub const LEFT: f32 = 192.;
pub const RIGHT: f32 = 696.;
pub const CONTENT: f32 = RIGHT - LEFT;
pub const TITLES: [&str; 8] = [
    "Companion",
    "Study",
    "Playroom",
    "Animations",
    "Cozy corner",
    "Appearance",
    "Behavior",
    "Connections",
];
#[derive(Clone)]
pub enum Action {
    Tab(usize),
    Command(Command),
    Edit(super::editor::Field),
}

pub struct Control {
    pub rect: [f32; 4],
    pub action: Action,
    pub enabled: bool,
}

impl Control {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let [left, top, width, height] = self.rect;
        self.enabled && x >= left && x < left + width && y >= top && y < top + height
    }
}

pub struct Page<'a, 'b> {
    pub updates: &'a crate::updates::State,
    pub startup: crate::native::startup::Status,
    pub artwork: &'a super::artwork::Artwork,
    pub painter: &'a mut Painter<'b>,
    pub controls: Vec<Control>,
    pub focus: Option<usize>,
    pub hover: Option<usize>,
}

impl Page<'_, '_> {
    pub(super) fn button(
        &mut self,
        rect: [f32; 4],
        label: &str,
        action: Action,
        selected: bool,
        enabled: bool,
    ) {
        let index = self.controls.len();
        let focused = self.focus == Some(index) || self.hover == Some(index);
        self.painter.pill(
            rect,
            10.,
            if selected {
                SOFT
            } else if enabled {
                [255; 3]
            } else {
                [241, 246, 249]
            },
            Some(if focused { BLUE } else { BORDER }),
        );
        self.painter.centered(
            label,
            rect,
            13.,
            if !enabled {
                [142, 159, 169]
            } else if selected {
                BLUE
            } else {
                INK
            },
        );
        self.controls.push(Control {
            rect,
            action,
            enabled,
        });
    }
    pub(super) fn section(&mut self, y: f32, title: &str) {
        self.painter.text(title, LEFT, y, 15., INK);
    }
    pub(super) fn note(&mut self, y: f32, text: &str) {
        self.painter.text(text, LEFT, y, 12., MUTED);
    }
    pub(super) fn divider(&mut self, y: f32) {
        self.painter.pill([LEFT, y, CONTENT, 1.], 0., BORDER, None);
    }
    pub(super) fn toggle(&mut self, y: f32, title: &str, help: &str, on: bool, command: Command) {
        self.painter.text(title, LEFT, y, 14., INK);
        self.painter.text(help, LEFT, y + 23., 12., MUTED);
        self.button(
            [RIGHT - 64., y + 4., 64., 32.],
            if on { "On" } else { "Off" },
            Action::Command(command),
            on,
            true,
        );
    }
    pub(super) fn stepper(
        &mut self,
        y: f32,
        value: &str,
        minus: (Command, bool),
        plus: (Command, bool),
    ) {
        let x = RIGHT - 164.;
        self.painter
            .pill([x, y, 164., 34.], 8., [255; 3], Some(BORDER));
        self.painter
            .centered(value, [x + 34., y, 96., 34.], 13., INK);
        self.button(
            [x, y, 34., 34.],
            "−",
            Action::Command(minus.0),
            false,
            minus.1,
        );
        self.button(
            [RIGHT - 34., y, 34., 34.],
            "+",
            Action::Command(plus.0),
            false,
            plus.1,
        );
    }
    pub(super) fn cell(index: usize, top: f32, columns: usize) -> [f32; 4] {
        let width = (CONTENT - (columns - 1) as f32 * 12.) / columns as f32;
        [
            LEFT + index as f32 % columns as f32 * (width + 12.),
            top + (index / columns) as f32 * 44.,
            width,
            34.,
        ]
    }
    pub fn draw(
        mut self,
        tab: usize,
        s: &Settings,
        paused: bool,
        activity: Activity,
        study: crate::companion::study::View,
        status: &str,
    ) -> Vec<Control> {
        let parent = match tab {
            8 => 1,
            9 => 5,
            10 => 2,
            _ => tab,
        };
        self.navigation(parent);
        self.header(tab);
        match tab {
            0 => self.overview(paused, activity, study),
            1 => self.study(s, study, paused),
            2 => self.playroom(s),
            3 => self.animations(s, paused, activity),
            4 => self.corner(s),
            5 => self.appearance(s),
            6 => self.behavior(s),
            8 => self.planning(s),
            9 => self.creative(s),
            10 => self.surprise_items(s),
            11 => self.updates_page(s),
            _ => self.connections(s),
        }
        self.footer(status);
        self.controls
    }
}
