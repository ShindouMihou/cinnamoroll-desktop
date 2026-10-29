use super::{State, config::Config};
use std::{path::Path, time::Duration};

pub(super) fn target() -> &'static str {
    if cfg!(target_os = "windows") {
        "x86_64-pc-windows"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-apple-darwin"
    } else {
        "x86_64-apple-darwin"
    }
}

pub(super) fn asset_name(version: &str) -> Result<String, String> {
    let version = semver::Version::parse(version).map_err(|error| error.to_string())?;
    if !version.pre.is_empty()
        || version
            <= semver::Version::parse(env!("CARGO_PKG_VERSION"))
                .map_err(|error| error.to_string())?
    {
        return Err("Only newer stable releases can be installed.".into());
    }
    Ok(format!("Cinnamoroll-{version}-{}.zip", target()))
}

fn latest() -> Result<Option<self_update::Release>, String> {
    let config = Config::embedded()?;
    let result = self_update::backends::github::Update::configure()
        .repo_owner(config.owner)
        .repo_name(config.repository)
        .auth_token("")
        .bin_name("Cinnamoroll")
        .current_version(env!("CARGO_PKG_VERSION"))
        .timeout(Duration::from_secs(20))
        .show_output(false)
        .no_confirm(true)
        .build()
        .map_err(|error| error.to_string())?
        .get_latest_release();
    match result {
        Ok(releases) => Ok(releases.latest().cloned()),
        Err(error) if error.http_status() == Some(404) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn check() -> Result<State, String> {
    let Some(release) = latest()? else {
        return Ok(State::AwaitingRelease);
    };
    let current =
        semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
    let candidate = semver::Version::parse(release.version()).map_err(|error| error.to_string())?;
    if candidate <= current || !candidate.pre.is_empty() {
        return Ok(State::Current);
    }
    let name = asset_name(release.version())?;
    if !release.assets().iter().any(|asset| asset.name() == name) {
        return Err("This release has no update for your computer yet.".into());
    }
    Ok(State::Available(release.version().into()))
}

pub(super) fn download(version: &str) -> Result<State, String> {
    let name = asset_name(version)?;
    let config = Config::embedded()?;
    let directory = crate::settings::directory()
        .map_err(|error| error.to_string())?
        .join("updates");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    prune_cache(&directory);
    let temporary = tempfile::Builder::new()
        .prefix("download-")
        .tempdir_in(&directory)
        .map_err(|error| error.to_string())?;
    let archive = temporary.path().join(&name);
    let mut file = std::fs::File::create(&archive).map_err(|error| error.to_string())?;
    let url = format!(
        "https://github.com/{}/{}/releases/download/v{version}/{name}",
        config.owner, config.repository
    );
    self_update::Download::from_url(url)
        .timeout(Duration::from_secs(120))
        .max_download_size(128 * 1024 * 1024)
        .download_to(&mut file)
        .map_err(|error| error.to_string())?;
    drop(file);
    verify(&archive)?;
    let archive = temporary.keep().join(name);
    Ok(State::Ready {
        version: version.into(),
        archive,
    })
}

pub(super) fn verify(archive: &Path) -> Result<(), String> {
    let config = Config::embedded()?;
    self_update::verify_signature(archive, &[config.key])
        .map_err(|error| format!("Update signature rejected: {error}"))
}

pub(super) fn automatic_check_due() -> bool {
    let Ok(directory) = crate::settings::directory() else {
        return false;
    };
    let stamp = directory.join("update-check.stamp");
    if std::fs::metadata(&stamp)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age.as_secs() < 86400)
    {
        return false;
    }
    std::fs::write(stamp, []).is_ok()
}

fn prune_cache(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<_> = entries
        .flatten()
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("download-")
                && entry.file_type().is_ok_and(|kind| kind.is_dir())
        })
        .collect();
    entries.sort_by_key(|entry| {
        std::cmp::Reverse(
            entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok(),
        )
    });
    for entry in entries.into_iter().skip(1) {
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_asset_identity_binds_version_and_platform_and_blocks_downgrades() {
        let name = asset_name("9.1.2").unwrap();
        assert!(name.contains(target()));
        assert!(name.starts_with("Cinnamoroll-9.1.2-"));
        for version in [
            env!("CARGO_PKG_VERSION"),
            "0.0.1",
            "99.0.0-beta.1",
            "../../file",
            "nightly",
        ] {
            assert!(asset_name(version).is_err());
        }
    }
}

#[cfg(test)]
mod signature_tests {
    use std::io::Cursor;

    #[test]
    fn rejects_unsigned_tampered_wrong_key_and_renamed_archives() {
        let directory = tempfile::tempdir().unwrap();
        let name = "Cinnamoroll-9.1.2-test.zip";
        let path = directory.path().join(name);
        let mut zip = vec![0; 22];
        zip[..4].copy_from_slice(b"PK\x05\x06");
        std::fs::write(&path, &zip).unwrap();
        let key = zipsign_api::SigningKey::from_bytes(&[42; 32]);
        let public = key.verifying_key().to_bytes();
        assert!(self_update::verify_signature(&path, &[public]).is_err());
        let mut signed = Cursor::new(Vec::new());
        zipsign_api::sign::copy_and_sign_zip(
            &mut Cursor::new(zip),
            &mut signed,
            &[key],
            Some(name.as_bytes()),
        )
        .unwrap();
        std::fs::write(&path, signed.get_ref()).unwrap();
        self_update::verify_signature(&path, &[public]).unwrap();
        assert!(self_update::verify_signature(&path, &[[7; 32]]).is_err());
        let renamed = directory.path().join("different-version.zip");
        std::fs::copy(&path, &renamed).unwrap();
        assert!(self_update::verify_signature(&renamed, &[public]).is_err());
        let mut tampered = signed.into_inner();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        std::fs::write(&path, tampered).unwrap();
        assert!(self_update::verify_signature(&path, &[public]).is_err());
    }
}
