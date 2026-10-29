use super::backend;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn bundle() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    if cfg!(target_os = "macos") {
        executable
            .ancestors()
            .find(|path| path.extension().is_some_and(|extension| extension == "app"))
            .map(Path::to_path_buf)
            .ok_or_else(|| "Run the bundled app before installing updates.".into())
    } else {
        executable
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Cannot find the app folder.".into())
    }
}

pub(super) fn executable(root: &Path) -> PathBuf {
    root.join(if cfg!(target_os = "macos") {
        "Contents/MacOS/Cinnamoroll"
    } else {
        "Cinnamoroll.exe"
    })
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), destination.join(entry.file_name()))
                .map_err(|error| error.to_string())?;
        } else {
            return Err(
                "The portable app contains unsupported links. Use a fresh portable copy.".into(),
            );
        }
    }
    Ok(())
}

pub(super) fn launch(archive: &Path, version: &str, automatic: bool) -> Result<bool, String> {
    let expected = backend::asset_name(version)?;
    if archive
        .file_name()
        .is_none_or(|name| name != expected.as_str())
    {
        return Err("The staged update has an unexpected name.".into());
    }
    backend::verify(archive)?;
    let original = bundle()?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let _probe = tempfile::tempdir_in(
        original
            .parent()
            .ok_or("The installation has no parent folder.")?,
    )
    .map_err(|error| format!("The app folder is not writable: {error}"))?;
    let cache = archive.parent().ok_or("Cannot locate update staging.")?;
    let helper = cache.join(if cfg!(target_os = "macos") {
        "Helper.app"
    } else {
        "helper"
    });
    if helper.exists() {
        std::fs::remove_dir_all(&helper).map_err(|error| error.to_string())?;
    }
    if cfg!(target_os = "macos") {
        let status = Command::new("/usr/bin/ditto")
            .arg(&original)
            .arg(&helper)
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("Couldn’t prepare the update helper.".into());
        }
    } else {
        copy_tree(&original, &helper)?;
    }
    if automatic && !super::away::long_idle(crate::system::idle_seconds()) {
        return Ok(false);
    }
    let mut command = Command::new(executable(&helper));
    command
        .arg("--apply-signed-update")
        .arg(&original)
        .arg(archive)
        .arg(version)
        .arg(std::process::id().to_string())
        .arg(automatic.to_string());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command.spawn().map_err(|error| error.to_string())?;
    Ok(true)
}

fn wait_for_parent(pid: u32) -> Result<(), String> {
    if pid == 0 || pid == std::process::id() {
        return Err("Invalid updater parent process.".into());
    }
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::{
            Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0},
            System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
        };
        let process = match unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) } {
            Ok(process) => process,
            Err(error) if error.code() == ERROR_INVALID_PARAMETER.to_hresult() => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let result = unsafe { WaitForSingleObject(process, 60000) };
        let _ = unsafe { CloseHandle(process) };
        if result != WAIT_OBJECT_0 {
            return Err("Cinnamoroll is still running. The update was left untouched.".into());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        let start = std::time::Instant::now();
        loop {
            if unsafe { kill(pid as i32, 0) } != 0 {
                break;
            }
            if start.elapsed() > std::time::Duration::from_secs(60) {
                return Err("Cinnamoroll is still running. The update was left untouched.".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    Ok(())
}

fn replace(original: &Path, archive: &Path, version: &str, automatic: bool) -> Result<(), String> {
    backend::verify(archive)?;
    let parent = original
        .parent()
        .ok_or("The installation has no parent folder.")?;
    let stage = tempfile::tempdir_in(parent).map_err(|error| error.to_string())?;
    self_update::Extract::from_source(archive)
        .archive(self_update::ArchiveKind::Zip)
        .extract_into(stage.path())
        .map_err(|error| error.to_string())?;
    let candidate = stage.path().join(if cfg!(target_os = "macos") {
        "Cinnamoroll.app"
    } else {
        "Cinnamoroll-Windows"
    });
    super::validation::validate(&candidate, version)?;
    if automatic && !super::away::long_idle(crate::system::idle_seconds()) {
        return Ok(());
    }
    commit(&candidate, original)
}

fn commit(candidate: &Path, original: &Path) -> Result<(), String> {
    let parent = original
        .parent()
        .ok_or("The installation has no parent folder.")?;
    let stash = tempfile::tempdir_in(parent)
        .map_err(|error| error.to_string())?
        .keep();
    let backup = stash.join("previous");
    if let Err(error) = std::fs::rename(original, &backup) {
        let _ = std::fs::remove_dir(&stash);
        return Err(error.to_string());
    }
    if let Err(error) = std::fs::rename(candidate, original) {
        if let Err(restore) = std::fs::rename(&backup, original) {
            return Err(format!(
                "Update failed: {error}. Restore failed: {restore}. Your previous app is preserved at {}",
                backup.display()
            ));
        }
        let _ = std::fs::remove_dir(&stash);
        return Err(error.to_string());
    }
    let _ = std::fs::remove_dir_all(stash);
    Ok(())
}

pub(super) fn apply(arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 5 {
        return Err("Invalid updater arguments.".into());
    }
    let original = Path::new(&arguments[0])
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if bundle()?
        .canonicalize()
        .map_err(|error| error.to_string())?
        == original
    {
        return Err("Run the updater from its staging copy.".into());
    }
    let archive = Path::new(&arguments[1]);
    let expected = backend::asset_name(&arguments[2])?;
    if archive
        .file_name()
        .is_none_or(|name| name != expected.as_str())
    {
        return Err("The update’s version and archive do not match.".into());
    }
    let pid = arguments[3]
        .parse()
        .map_err(|_| "Invalid parent process.")?;
    wait_for_parent(pid)?;
    let automatic = arguments[4]
        .parse()
        .map_err(|_| "Invalid restart policy.")?;
    let result = replace(&original, archive, &arguments[2], automatic);
    if let Err(error) = &result {
        let _ = std::fs::write(archive.with_extension("error.txt"), error);
        let _ = sdl3::messagebox::show_simple_message_box(
            sdl3::messagebox::MessageBoxFlag::ERROR,
            "Cinnamoroll couldn’t update",
            error,
            None,
        );
    }
    Command::new(executable(&original))
        .spawn()
        .map_err(|error| error.to_string())?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_folder_replacement_preserves_the_original_on_failure() {
        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("installed");
        std::fs::create_dir(&original).unwrap();
        std::fs::write(original.join("pet"), "old").unwrap();
        assert!(commit(&directory.path().join("missing"), &original).is_err());
        assert_eq!(
            std::fs::read_to_string(original.join("pet")).unwrap(),
            "old"
        );
        let candidate = directory.path().join("staged");
        std::fs::create_dir(&candidate).unwrap();
        std::fs::write(candidate.join("pet"), "new").unwrap();
        std::fs::write(candidate.join("SDL3.dll"), "new-library").unwrap();
        commit(&candidate, &original).unwrap();
        assert_eq!(
            std::fs::read_to_string(original.join("pet")).unwrap(),
            "new"
        );
        assert!(original.join("SDL3.dll").is_file());
    }
}
