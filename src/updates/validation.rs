use super::{backend, install::executable};
use std::{path::Path, process::Command};

pub(super) fn validate(root: &Path, version: &str) -> Result<(), String> {
    if !executable(root).is_file() {
        return Err("The update is missing Cinnamoroll’s executable.".into());
    }
    if cfg!(target_os = "macos") {
        if !Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(root)
            .status()
            .map_err(|error| error.to_string())?
            .success()
        {
            return Err("The updated Mac bundle failed its code-signature check.".into());
        }
    } else if !root.join("SDL3.dll").is_file() {
        return Err("The update is missing SDL3.dll.".into());
    }
    let mut command = Command::new(executable(root));
    command
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            let mut output = String::new();
            if let Some(stdout) = child.stdout.take() {
                use std::io::Read;
                stdout
                    .take(4096)
                    .read_to_string(&mut output)
                    .map_err(|error| error.to_string())?;
            }
            if !status.success() || output.trim() != format!("Cinnamoroll {version}") {
                return Err(
                    "The signed update’s executable does not match its release version.".into(),
                );
            }
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("The updated executable did not pass its launch check.".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}

pub(super) fn verify_release(archive: &Path) -> Result<(), String> {
    let expected = format!(
        "Cinnamoroll-{}-{}.zip",
        env!("CARGO_PKG_VERSION"),
        backend::target()
    );
    if archive
        .file_name()
        .is_none_or(|name| name != expected.as_str())
    {
        return Err(
            "The release filename does not match this build’s version and platform.".into(),
        );
    }
    backend::verify(archive)?;
    let stage = tempfile::tempdir().map_err(|error| error.to_string())?;
    self_update::Extract::from_source(archive)
        .archive(self_update::ArchiveKind::Zip)
        .extract_into(stage.path())
        .map_err(|error| error.to_string())?;
    validate(
        &stage.path().join(if cfg!(target_os = "macos") {
            "Cinnamoroll.app"
        } else {
            "Cinnamoroll-Windows"
        }),
        env!("CARGO_PKG_VERSION"),
    )
}
