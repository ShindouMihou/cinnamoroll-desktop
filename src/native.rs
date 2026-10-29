#[cfg(target_os = "macos")]
mod tray_macos;
use crate::{art, behavior::Activity, command::Command, geometry::Bounds, settings::Settings};
use sdl3::{pixels::PixelFormat, surface::Surface, sys::everything::*};
use std::{
    ffi::{CString, c_void},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
    sync::mpsc::Sender,
};
#[cfg(target_os = "macos")]
use tray_macos::{
    SDL_CreateTray, SDL_CreateTrayMenu, SDL_CreateTraySubmenu, SDL_DestroyTray,
    SDL_InsertTrayEntryAt, SDL_SetTrayEntryCallback, SDL_SetTrayEntryChecked,
    SDL_SetTrayEntryLabel,
};

struct Callback {
    sender: Sender<Command>,
    command: Command,
}

pub struct Tray {
    raw: NonNull<SDL_Tray>,
    #[allow(
        clippy::vec_box,
        reason = "SDL retains each callback pointer, so Vec reallocation must not move the callback"
    )]
    callbacks: Vec<Box<Callback>>,
    status: *mut SDL_TrayEntry,
    pause: *mut SDL_TrayEntry,
    wander: *mut SDL_TrayEntry,
    music: *mut SDL_TrayEntry,
    music_status: *mut SDL_TrayEntry,
    occasional_chase: *mut SDL_TrayEntry,
    follow_typing: *mut SDL_TrayEntry,
    caret_status: *mut SDL_TrayEntry,
    reduced: *mut SDL_TrayEntry,
    hide: *mut SDL_TrayEntry,
    main_thread: PhantomData<Rc<()>>,
}

impl Tray {
    pub fn new(sender: Sender<Command>) -> Result<Self, String> {
        let mut icon = art::tray_icon()?;
        let surface = Surface::from_data(icon.data_mut(), 64, 64, 64 * 4, PixelFormat::RGBA32)
            .map_err(|e| e.to_string())?;
        // SDL copies the icon; the tray is main-thread-only and outlives every callback allocation.
        let raw = unsafe { SDL_CreateTray(surface.raw(), c"Cinnamoroll".as_ptr()) };
        let raw = NonNull::new(raw).ok_or_else(|| sdl3::get_error().to_string())?;
        let mut tray = Self {
            raw,
            callbacks: Vec::new(),
            status: std::ptr::null_mut(),
            pause: std::ptr::null_mut(),
            wander: std::ptr::null_mut(),
            music: std::ptr::null_mut(),
            music_status: std::ptr::null_mut(),
            occasional_chase: std::ptr::null_mut(),
            follow_typing: std::ptr::null_mut(),
            caret_status: std::ptr::null_mut(),
            reduced: std::ptr::null_mut(),
            hide: std::ptr::null_mut(),
            main_thread: PhantomData,
        };
        let menu = unsafe { SDL_CreateTrayMenu(raw.as_ptr()) };
        if menu.is_null() {
            return Err(sdl3::get_error().to_string());
        }
        tray.status = tray.entry(
            menu,
            "Cinnamoroll · your little cloud",
            SDL_TRAYENTRY_BUTTON | SDL_TRAYENTRY_DISABLED,
            None,
            &sender,
        )?;
        tray.entry(
            menu,
            "Drag to pick up · click to pet · right-click for a surprise",
            SDL_TRAYENTRY_BUTTON | SDL_TRAYENTRY_DISABLED,
            None,
            &sender,
        )?;
        tray.separator(menu)?;
        tray.entry(
            menu,
            "Settings… · S",
            SDL_TRAYENTRY_BUTTON,
            Some(Command::Settings),
            &sender,
        )?;
        let interactions = tray.entry(
            menu,
            "Play with Cinnamoroll",
            SDL_TRAYENTRY_SUBMENU,
            None,
            &sender,
        )?;
        let interaction_menu = unsafe { SDL_CreateTraySubmenu(interactions) };
        if interaction_menu.is_null() {
            return Err(sdl3::get_error().to_string());
        }
        for (label, command) in [
            ("Offer a cinnamon roll · F", Command::Feed),
            ("Boop his nose · B", Command::Boop),
            ("Play chase / stop · T", Command::Chase),
            ("Peek where I point (3-second delay) · H", Command::Peek),
            ("Tuck in / wake up · N", Command::Nap),
            ("Hide and peek from an edge · E", Command::EdgePeek),
        ] {
            tray.entry(
                interaction_menu,
                label,
                SDL_TRAYENTRY_BUTTON,
                Some(command),
                &sender,
            )?;
        }
        let activities = tray.entry(
            menu,
            "Let's do something…",
            SDL_TRAYENTRY_SUBMENU,
            None,
            &sender,
        )?;
        let submenu = unsafe { SDL_CreateTraySubmenu(activities) };
        if submenu.is_null() {
            return Err(sdl3::get_error().to_string());
        }
        for activity in Activity::ALL {
            if matches!(activity, Activity::Held | Activity::Land) {
                continue;
            }
            tray.entry(
                submenu,
                activity.label(),
                SDL_TRAYENTRY_BUTTON,
                Some(Command::Activity(activity)),
                &sender,
            )?;
        }
        tray.pause = tray.entry(
            menu,
            "Pause animations",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::Pause),
            &sender,
        )?;
        tray.occasional_chase = tray.entry(
            menu,
            "Occasionally chase the mouse",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::OccasionalChase),
            &sender,
        )?;
        tray.follow_typing = tray.entry(
            menu,
            "Follow typing cursor (optional)",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::FollowTyping),
            &sender,
        )?;
        tray.caret_status = tray.entry(
            menu,
            "Typing cursor following off",
            SDL_TRAYENTRY_BUTTON | SDL_TRAYENTRY_DISABLED,
            None,
            &sender,
        )?;
        tray.wander = tray.entry(
            menu,
            "Wander around",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::Wander),
            &sender,
        )?;
        tray.music = tray.entry(
            menu,
            "React to music",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::Music),
            &sender,
        )?;
        tray.reduced = tray.entry(
            menu,
            "Reduced motion",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::ReducedMotion),
            &sender,
        )?;
        let sizes = tray.entry(menu, "Pet size", SDL_TRAYENTRY_SUBMENU, None, &sender)?;
        tray.music_status = tray.entry(
            menu,
            "Music reactions off",
            SDL_TRAYENTRY_BUTTON | SDL_TRAYENTRY_DISABLED,
            None,
            &sender,
        )?;
        let size_menu = unsafe { SDL_CreateTraySubmenu(sizes) };
        if size_menu.is_null() {
            return Err(sdl3::get_error().to_string());
        }
        for (label, size) in [
            ("Tiny", 30),
            ("Small · recommended", 40),
            ("Medium", 55),
            ("Large", 75),
        ] {
            tray.entry(
                size_menu,
                label,
                SDL_TRAYENTRY_BUTTON,
                Some(Command::Size(size)),
                &sender,
            )?;
        }
        tray.separator(menu)?;
        tray.entry(
            menu,
            "Bring Cinnamoroll home",
            SDL_TRAYENTRY_BUTTON,
            Some(Command::Reset),
            &sender,
        )?;
        tray.hide = tray.entry(
            menu,
            "Hide pet",
            SDL_TRAYENTRY_CHECKBOX,
            Some(Command::Hide),
            &sender,
        )?;
        tray.entry(
            menu,
            "Quit",
            SDL_TRAYENTRY_BUTTON,
            Some(Command::Quit),
            &sender,
        )?;
        #[cfg(target_os = "macos")]
        unsafe {
            // AppKit must enter its run loop once to register the menu-bar scene before SDL takes over.
            tray_macos::Cinna_BootstrapTray();
        }
        Ok(tray)
    }

    fn entry(
        &mut self,
        menu: *mut SDL_TrayMenu,
        label: &str,
        flags: SDL_TrayEntryFlags,
        command: Option<Command>,
        sender: &Sender<Command>,
    ) -> Result<*mut SDL_TrayEntry, String> {
        let label = CString::new(label).map_err(|e| e.to_string())?;
        let entry = unsafe { SDL_InsertTrayEntryAt(menu, -1, label.as_ptr(), flags) };
        if entry.is_null() {
            return Err(sdl3::get_error().to_string());
        }
        if let Some(command) = command {
            let mut callback = Box::new(Callback {
                sender: sender.clone(),
                command,
            });
            unsafe {
                SDL_SetTrayEntryCallback(
                    entry,
                    Some(invoke),
                    (&mut *callback as *mut Callback).cast(),
                );
            }
            self.callbacks.push(callback);
        }
        Ok(entry)
    }

    fn separator(&self, menu: *mut SDL_TrayMenu) -> Result<(), String> {
        let entry =
            unsafe { SDL_InsertTrayEntryAt(menu, -1, std::ptr::null(), SDL_TRAYENTRY_BUTTON) };
        if entry.is_null() {
            Err(sdl3::get_error().to_string())
        } else {
            Ok(())
        }
    }

    pub fn diagnostic(&self) -> String {
        #[cfg(target_os = "macos")]
        {
            unsafe {
                std::ffi::CStr::from_ptr(tray_macos::Cinna_TrayDescription(self.raw.as_ptr()))
            }
            .to_string_lossy()
            .into_owned()
        }
        #[cfg(not(target_os = "macos"))]
        {
            "SDL system tray created".into()
        }
    }

    pub fn visibility_check(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            unsafe { tray_macos::Cinna_TrayVisible(self.raw.as_ptr()) }
        }
        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    pub fn sync(&self, settings: &Settings, paused: bool, hidden: bool) {
        unsafe {
            SDL_SetTrayEntryChecked(self.pause, paused);
            SDL_SetTrayEntryChecked(self.wander, settings.wandering);
            SDL_SetTrayEntryChecked(self.music, settings.music_reactions);
            SDL_SetTrayEntryChecked(self.reduced, settings.reduced_motion);
            SDL_SetTrayEntryChecked(self.hide, hidden);
            SDL_SetTrayEntryChecked(self.occasional_chase, settings.occasional_chase);
            SDL_SetTrayEntryChecked(self.follow_typing, settings.follow_typing);
        }
    }

    pub fn caret_status(&self, text: &str) {
        if let Ok(label) = CString::new(text) {
            unsafe {
                SDL_SetTrayEntryLabel(self.caret_status, label.as_ptr());
            }
        }
    }

    pub fn music_status(&self, text: &str) {
        if let Ok(label) = CString::new(text) {
            unsafe {
                SDL_SetTrayEntryLabel(self.music_status, label.as_ptr());
            }
        }
    }

    pub fn status(&self, text: &str) {
        if let Ok(label) = CString::new(text) {
            unsafe {
                SDL_SetTrayEntryLabel(self.status, label.as_ptr());
            }
        }
    }
}

unsafe extern "C" fn invoke(data: *mut c_void, _: *mut SDL_TrayEntry) {
    // SDL invokes this only while Tray owns the stable Box; Drop destroys SDL callbacks first.
    let callback = unsafe { &*data.cast::<Callback>() };
    let _ = callback.sender.send(callback.command.clone());
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTray(self.raw.as_ptr());
        }
    }
}

pub fn mouse_position() -> (f32, f32) {
    let (mut x, mut y) = (0., 0.);
    unsafe {
        SDL_GetGlobalMouseState(&mut x, &mut y);
    }
    (x, y)
}

pub fn displays(video: &sdl3::VideoSubsystem) -> Result<Vec<Bounds>, String> {
    video
        .displays()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|display| {
            let rect = display.get_usable_bounds().map_err(|e| e.to_string())?;
            Ok(Bounds {
                x: rect.x(),
                y: rect.y(),
                width: rect.width(),
                height: rect.height(),
            })
        })
        .collect()
}

pub(crate) mod stacking;

pub(crate) mod startup;
