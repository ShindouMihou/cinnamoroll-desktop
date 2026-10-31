mod corner_drag;
mod input;
pub(crate) use input::InputPosition;
pub(crate) mod moments_check;
mod overlay;
pub(crate) mod play_preview;
pub(crate) mod preview;
use crate::{
    art::Decoration,
    behavior::Activity,
    companion::{Companion, corner_home, world::Kind},
    geometry::Bounds,
    settings::Settings,
};
use corner_drag::CornerDrag;
use overlay::Overlay;
use sdl3::{
    VideoSubsystem,
    event::{Event, WindowEvent},
    mouse::{MouseButton, MouseUtil},
};

pub(crate) struct Frame<'a> {
    pub settings: &'a Settings,
    pub companion: &'a Companion,
    pub pet: (f32, f32),
    pub size: (u32, u32),
    pub bounds: Bounds,
    pub hidden: bool,
    pub now: f64,
    pub pet_window: &'a sdl3::video::Window,
}

pub(crate) struct Scene {
    video: VideoSubsystem,
    corner: Option<Overlay>,
    item: Option<Overlay>,
    hint: Option<Overlay>,
    dream: Option<Overlay>,
    corner_drag: Option<CornerDrag>,
    item_offset: (f32, f32),
    pending_move: Option<crate::companion::comfort::MoveSnapshot>,
    settled_offset: Option<(f32, f32)>,
}

impl Scene {
    pub fn new(video: &VideoSubsystem) -> Self {
        Self {
            video: video.clone(),
            corner: None,
            item: None,
            hint: None,
            dream: None,
            corner_drag: None,
            item_offset: (0., 0.),
            pending_move: None,
            settled_offset: None,
        }
    }
    pub fn reset(&mut self) {
        self.corner = None;
        self.item = None;
        self.hint = None;
        self.dream = None;
        self.corner_drag = None;
        self.settled_offset = None;
    }
    pub fn dragging_corner(&self) -> bool {
        self.corner_drag.is_some()
    }
    pub fn carrying_pet(&self) -> bool {
        self.corner_drag
            .as_ref()
            .is_some_and(CornerDrag::carries_pet)
    }
    pub fn owns(&self, event: &Event) -> bool {
        event.get_window_id().is_some_and(|id| {
            self.corner.as_ref().is_some_and(|w| id == w.id())
                || self.item.as_ref().is_some_and(|w| id == w.id())
                || self.dream.as_ref().is_some_and(|w| id == w.id())
                || self.hint.as_ref().is_some_and(|w| id == w.id())
        })
    }
    pub fn drag(
        &mut self,
        c: &mut Companion,
        s: &mut Settings,
        pointer: (f32, f32),
        size: (u32, u32),
        bounds: Bounds,
        now: f64,
    ) -> Option<(f32, f32)> {
        let mut passenger = None;
        if let Some(drag) = self.corner_drag.as_mut()
            && let Some((home, pet)) = drag.motion(pointer, size, bounds)
        {
            if let Some(snapshot) = self.pending_move.take() {
                c.move_undo = Some(snapshot);
            }
            s.corner_position = Some((home.0.round() as i32, home.1.round() as i32));
            s.extras.comfort.preferred_display = Some(bounds);
            passenger = pet;
            if let Some(pet) = pet
                && let Some(item) = c.item.as_mut()
                && item.kind == Kind::Pillow
                && item.carried
            {
                item.position = (pet.0 + size.0 as f32 * 0.5, pet.1 + size.1 as f32 * 0.88);
            }
        }
        if let Some(item) = c.item.as_mut()
            && item.held
        {
            if item.kind == Kind::Pond {
                let stage = (((pointer.0 - item.position.0) / (item.size().0 as f32 * 0.5) + 0.5)
                    * 100.)
                    .clamp(0., 100.) as u8;
                item.velocity = (f32::from(stage) - f32::from(item.stage), 0.);
                item.stage = stage;
            } else {
                item.drag(
                    (
                        pointer.0 - self.item_offset.0,
                        pointer.1 - self.item_offset.1,
                    ),
                    now,
                );
            }
        }
        passenger
    }
    pub fn render(&mut self, frame: Frame<'_>) -> Result<(), String> {
        let Frame {
            settings: s,
            companion: c,
            pet,
            size,
            bounds,
            hidden,
            now,
            pet_window,
        } = frame;
        if hidden {
            self.corner = None;
            self.item = None;
            self.hint = None;
            self.dream = None;
            self.corner_drag = None;
            self.settled_offset = None;
            return Ok(());
        }
        if let Some((_, kind)) = c.moments.dream {
            if self.dream.is_none() {
                self.dream = Some(Overlay::new(&self.video, "Cinnamoroll · A little dream")?);
            }
            let (bubble_size, position) =
                crate::companion::placement::dream_layout(pet, size, bounds);
            self.dream.as_mut().ok_or("Missing dream window")?.show(
                Decoration::Dream(kind),
                bubble_size,
                position,
                0,
            )?;
        } else {
            self.dream = None;
        }
        if s.cozy_corner {
            let home = corner_home(s.corner_position, pet, size, bounds);
            self.settled_offset = if c
                .item
                .as_ref()
                .is_none_or(|item| item.kind == Kind::Pillow && item.carried)
                && (pet.0 - home.0).hypot(pet.1 - home.1) <= 8.
            {
                Some((pet.0 - home.0, pet.1 - home.1))
            } else {
                None
            };
            let layout = crate::companion::corner::Layout::new(size);
            if self.corner.is_none() {
                self.corner = Some(Overlay::new(&self.video, "Cinnamoroll · Cozy corner")?);
            }
            self.corner
                .as_mut()
                .ok_or("Missing cozy corner window")?
                .show(
                    Decoration::Corner(s.blanket_color),
                    layout.size,
                    (home.0 + layout.offset.0, home.1 + layout.offset.1),
                    0,
                )?;
            if let Some(corner) = self.corner.as_ref() {
                crate::native::stacking::below(corner.canvas.window(), pet_window)?;
            }
        } else {
            self.corner = None;
            self.corner_drag = None;
            self.settled_offset = None;
        }
        if let Some(item) = c.item.as_ref().filter(|item| {
            !(item.kind == Kind::Plush
                && item.carried
                && c.cozy_return.is_some_and(|visit| now < visit.due))
        }) {
            let (kind, logical, tick) = match item.kind {
                Kind::Ball => (
                    Decoration::Ball,
                    (36, 36),
                    if !s.reduced_motion && item.velocity.0.abs() > 15. {
                        (now * 10.) as u64
                    } else {
                        0
                    },
                ),
                Kind::Treat => (Decoration::Treat, (36, 36), 0),
                Kind::Bubbles => (
                    Decoration::Bubbles(
                        item.bubbles
                            | if now - item.last_pop < 0.4 {
                                item.popped << 3
                            } else {
                                0
                            },
                    ),
                    (96, 80),
                    if !s.reduced_motion && now - item.last_pop < 0.4 {
                        (now - item.last_pop).max(0.) as f32 * 30.
                    } else {
                        0.
                    } as u64,
                ),
                Kind::HideCups => {
                    let visual = item.cups.as_ref().map_or_else(
                        || {
                            crate::companion::games::CupRound::new(now, 0)
                                .visual(now, s.reduced_motion)
                        },
                        |game| game.visual(now, s.reduced_motion),
                    );
                    (Decoration::Cups(visual), item.kind.size(), 0)
                }
                kind => (
                    Decoration::Prop(
                        kind,
                        if kind == Kind::KeepsakeBox {
                            item.stage | s.extras.keepsakes
                        } else {
                            item.stage
                        },
                        s.extras.palette,
                    ),
                    item.size(),
                    if ((item.held && matches!(kind, Kind::Wand | Kind::Ribbon | Kind::Pond))
                        || kind == Kind::Breathing
                        || (matches!(kind, Kind::Dandelion | Kind::SnowGlobe)
                            && item.stage > 0
                            && now - item.last_pop < 7.))
                        && !s.reduced_motion
                    {
                        ((now - item.last_pop)
                            .max(0.)
                            .min(if kind == Kind::SnowGlobe { 6. } else { 120. })
                            * 10.) as u64
                    } else if kind == Kind::SnowGlobe && item.stage > 0 {
                        60
                    } else {
                        0
                    },
                ),
            };
            if self.item.is_none() {
                self.item = Some(Overlay::new(&self.video, "Cinnamoroll · Toy or treat")?);
            }
            self.item.as_mut().ok_or("Missing toy window")?.show(
                kind,
                logical,
                (
                    item.position.0 - logical.0 as f32 / 2.,
                    item.position.1 - logical.1 as f32 / 2.,
                ),
                tick,
            )?;
            if item.kind == Kind::Ribbon
                && let Some(toy) = self.item.as_ref()
            {
                crate::native::stacking::below(toy.canvas.window(), pet_window)?;
            }
            if let Some(text) = item.hint() {
                let position = bounds.clamp(
                    (
                        item.position.0 - 90.,
                        item.position.1 + logical.1 as f32 / 2. + 4.,
                    ),
                    (180, 24),
                );
                if self.hint.is_none() {
                    self.hint = Some(Overlay::new(&self.video, "Cinnamoroll · Play hint")?);
                }
                self.hint.as_mut().ok_or("Missing play hint")?.show(
                    Decoration::Hint(text),
                    (180, 24),
                    position,
                    0,
                )?;
            } else {
                self.hint = None;
            }
        } else {
            self.item = None;
            if c.paw_offer.is_some() {
                let position = bounds.clamp(
                    (
                        pet.0 + size.0 as f32 * 0.5 - 90.,
                        pet.1 + size.1 as f32 + 4.,
                    ),
                    (180, 24),
                );
                if self.hint.is_none() {
                    self.hint = Some(Overlay::new(&self.video, "Cinnamoroll · Paw invitation")?);
                }
                self.hint.as_mut().ok_or("Missing paw hint")?.show(
                    Decoration::Hint("Hold near my raised paw"),
                    (180, 24),
                    position,
                    0,
                )?;
            } else {
                self.hint = None;
            }
        }
        Ok(())
    }
}
