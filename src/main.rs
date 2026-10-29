#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    if let Err(error) = cinnamoroll_desktop::app::run() {
        eprintln!("Cinnamoroll: {error}");
        if std::env::args_os().len() == 1 {
            let _ = sdl3::messagebox::show_simple_message_box(
                sdl3::messagebox::MessageBoxFlag::ERROR,
                "Cinnamoroll couldn't start",
                &error,
                None,
            );
        }
        std::process::exit(1);
    }
}
