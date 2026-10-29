#[derive(Clone)]
pub(super) struct Config {
    pub owner: String,
    pub repository: String,
    pub key: [u8; 32],
}

impl Config {
    pub fn parse(repository: &str, public_key: &str) -> Result<Self, String> {
        let (owner, repository) = repository
            .split_once('/')
            .ok_or("Use owner/repository for the update source.")?;
        for component in [owner, repository] {
            if component.is_empty()
                || matches!(component, "." | "..")
                || !component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
            {
                return Err("Invalid update repository.".into());
            }
        }
        if public_key.len() != 64 || !public_key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("The update public key must be 64 hexadecimal characters.".into());
        }
        let mut key = [0; 32];
        for (index, byte) in key.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&public_key[index * 2..index * 2 + 2], 16)
                .map_err(|error| error.to_string())?;
        }
        if key == [0; 32] {
            return Err("A real update signing key is required.".into());
        }
        Ok(Self {
            owner: owner.into(),
            repository: repository.into(),
            key,
        })
    }
    #[cfg(feature = "updater")]
    pub fn embedded() -> Result<Self, String> {
        Self::parse(
            option_env!("CINNAMOROLL_UPDATE_REPOSITORY").unwrap_or(""),
            option_env!("CINNAMOROLL_UPDATE_PUBLIC_KEY").unwrap_or(""),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_a_repository_and_non_placeholder_signing_key() {
        let key = "ab".repeat(32);
        let config = Config::parse("someone/cinnamoroll-desktop", &key).unwrap();
        assert_eq!(config.key, [0xab; 32]);
        assert_eq!(config.owner, "someone");
        assert_eq!(config.repository, "cinnamoroll-desktop");
        for repo in [
            "",
            "owner/repo/extra",
            "../repo",
            "https://github.com/owner/repo",
            "owner/repo?x=1",
        ] {
            assert!(Config::parse(repo, &key).is_err());
        }
        for key in ["".into(), "00".repeat(32), "gg".repeat(32), "é".repeat(32)] {
            assert!(Config::parse("owner/repo", &key).is_err());
        }
    }
}
