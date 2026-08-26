use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub base_url: Option<String>,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("otctl").join("config.toml"))
    }

    pub fn load_from(path: &Path) -> Result<Config> {
        match std::fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).map_err(|e| Error::Config(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(Error::Config(e.to_string())),
        }
    }

    pub fn load() -> Result<Config> {
        match Self::path() {
            Some(path) => Self::load_from(&path),
            None => Ok(Config::default()),
        }
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Config(e.to_string()))?;
        }
        let contents = toml::to_string_pretty(self).map_err(|e| Error::Config(e.to_string()))?;
        std::fs::write(path, contents).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        match Self::path() {
            Some(path) => self.save_to(&path),
            None => Err(Error::Config("could not determine config directory".into())),
        }
    }
}

pub fn resolve_base_url(
    cli_value: Option<String>,
    config_value: Option<String>,
    default: &str,
) -> String {
    cli_value
        .or(config_value)
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_base_url_prefers_cli_over_config_and_default() {
        let resolved = resolve_base_url(
            Some("http://cli".into()),
            Some("http://config".into()),
            "http://default",
        );
        assert_eq!(resolved, "http://cli");
    }

    #[test]
    fn resolve_base_url_falls_back_to_config_when_no_cli() {
        let resolved = resolve_base_url(None, Some("http://config".into()), "http://default");
        assert_eq!(resolved, "http://config");
    }

    #[test]
    fn resolve_base_url_falls_back_to_default_when_nothing_set() {
        let resolved = resolve_base_url(None, None, "http://default");
        assert_eq!(resolved, "http://default");
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn save_then_load_round_trips_base_url() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let cfg = Config {
            base_url: Some("http://example.test".into()),
        };
        cfg.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded, cfg);
    }

    #[test]
    fn load_from_invalid_toml_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not = [valid").unwrap();
        assert!(Config::load_from(&path).is_err());
    }
}
