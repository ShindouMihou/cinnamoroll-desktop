#![allow(non_snake_case)]
use sdl3::sys::everything::{
    SDL_Surface, SDL_Tray, SDL_TrayCallback, SDL_TrayEntry, SDL_TrayEntryFlags, SDL_TrayMenu,
};
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    fn Cinna_CreateTray(
        pixels: *const u8,
        width: c_int,
        height: c_int,
        pitch: c_int,
        tooltip: *const c_char,
    ) -> *mut SDL_Tray;
    #[link_name = "Cinna_CreateMenu"]
    pub fn SDL_CreateTrayMenu(tray: *mut SDL_Tray) -> *mut SDL_TrayMenu;
    #[link_name = "Cinna_CreateSubmenu"]
    pub fn SDL_CreateTraySubmenu(entry: *mut SDL_TrayEntry) -> *mut SDL_TrayMenu;
    #[link_name = "Cinna_InsertEntry"]
    pub fn SDL_InsertTrayEntryAt(
        menu: *mut SDL_TrayMenu,
        position: c_int,
        label: *const c_char,
        flags: SDL_TrayEntryFlags,
    ) -> *mut SDL_TrayEntry;
    #[link_name = "Cinna_SetCallback"]
    pub fn SDL_SetTrayEntryCallback(
        entry: *mut SDL_TrayEntry,
        callback: SDL_TrayCallback,
        context: *mut c_void,
    );
    #[link_name = "Cinna_SetLabel"]
    pub fn SDL_SetTrayEntryLabel(entry: *mut SDL_TrayEntry, label: *const c_char);
    #[link_name = "Cinna_SetChecked"]
    pub fn SDL_SetTrayEntryChecked(entry: *mut SDL_TrayEntry, checked: bool);
    #[link_name = "Cinna_DestroyTray"]
    pub fn SDL_DestroyTray(tray: *mut SDL_Tray);
    pub fn Cinna_BootstrapTray();
    pub fn Cinna_TrayVisible(tray: *mut SDL_Tray) -> bool;
    pub fn Cinna_TrayDescription(tray: *mut SDL_Tray) -> *const c_char;
}

pub unsafe fn SDL_CreateTray(surface: *mut SDL_Surface, tooltip: *const c_char) -> *mut SDL_Tray {
    // The caller owns this valid surface; AppKit copies its pixels before returning.
    let surface = unsafe { &*surface };
    unsafe {
        Cinna_CreateTray(
            surface.pixels.cast(),
            surface.w,
            surface.h,
            surface.pitch,
            tooltip,
        )
    }
}
