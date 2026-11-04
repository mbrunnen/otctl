use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Router {
    pub url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub routers: BTreeMap<String, Router>,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("otctl").join("config.toml"))
    }

    pub fn load_from(path: &Path) -> Result<Config> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => return Err(Error::Config(e.to_string())),
        };
        let table: toml::Table =
            toml::from_str(&contents).map_err(|e| Error::Config(e.to_string()))?;
        if table.contains_key("base_url") {
            return Err(Error::Config(format!(
                "{} uses the old base_url format; delete it and run 'otctl config add <name> <url>'",
                path.display()
            )));
        }
        table
            .try_into()
            .map_err(|e: toml::de::Error| Error::Config(e.to_string()))
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

    pub fn add(&mut self, name: &str, url: &str) -> Result<()> {
        if self.routers.contains_key(name) {
            return Err(Error::Config(format!(
                "router '{name}' already exists; remove it first"
            )));
        }
        self.routers.insert(
            name.to_string(),
            Router {
                url: url.to_string(),
            },
        );
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.router(name)?;
        self.routers.remove(name);
        if self.default.as_deref() == Some(name) {
            self.default = None;
        }
        Ok(())
    }

    pub fn set_default(&mut self, name: &str) -> Result<()> {
        self.router(name)?;
        self.default = Some(name.to_string());
        Ok(())
    }

    pub fn resolve_url(&self, router: Option<String>, builtin: &str) -> Result<String> {
        let name = match router.or_else(|| self.default.clone()) {
            Some(name) => name,
            None if self.routers.is_empty() => return Ok(builtin.to_string()),
            None => {
                return Err(Error::Config(
                    "no default router set; run 'otctl config default <name>'".into(),
                ))
            }
        };
        Ok(self.router(&name)?.url.clone())
    }

    fn router(&self, name: &str) -> Result<&Router> {
        self.routers.get(name).ok_or_else(|| {
            let saved = if self.routers.is_empty() {
                "no routers saved".to_string()
            } else {
                let names: Vec<&str> = self.routers.keys().map(String::as_str).collect();
                format!("saved routers: {}", names.join(", "))
            };
            Error::Config(format!("unknown router '{name}'; {saved}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn two_routers() -> Config {
        let mut c = Config::default();
        c.add("lab", "http://lab").unwrap();
        c.add("home", "http://home").unwrap();
        c
    }

    #[test]
    fn first_add_becomes_default() {
        let c = two_routers();
        assert_eq!(c.default.as_deref(), Some("lab"));
        assert_eq!(c.routers["home"].url, "http://home");
    }

    #[test]
    fn add_existing_name_fails() {
        let mut c = two_routers();
        assert!(c.add("lab", "http://other").is_err());
        assert_eq!(c.routers["lab"].url, "http://lab");
    }

    #[test]
    fn remove_default_clears_default() {
        let mut c = two_routers();
        c.remove("lab").unwrap();
        assert_eq!(c.default, None);
        assert!(!c.routers.contains_key("lab"));
    }

    #[test]
    fn remove_and_set_default_reject_unknown_name() {
        let mut c = two_routers();
        assert!(c.remove("nope").is_err());
        assert!(c.set_default("nope").is_err());
        c.set_default("home").unwrap();
        assert_eq!(c.default.as_deref(), Some("home"));
    }

    #[test]
    fn resolve_prefers_router_then_default() {
        let c = two_routers();
        assert_eq!(
            c.resolve_url(Some("home".into()), "http://builtin")
                .unwrap(),
            "http://home"
        );
        assert_eq!(c.resolve_url(None, "http://builtin").unwrap(), "http://lab");
    }

    #[test]
    fn resolve_uses_builtin_only_when_no_router_saved() {
        let c = Config::default();
        assert_eq!(
            c.resolve_url(None, "http://builtin").unwrap(),
            "http://builtin"
        );
    }

    #[test]
    fn resolve_errors_without_default_when_routers_saved() {
        let mut c = two_routers();
        c.default = None;
        let err = c
            .resolve_url(None, "http://builtin")
            .unwrap_err()
            .to_string();
        assert!(err.contains("otctl config default <name>"), "{err}");
    }

    #[test]
    fn resolve_unknown_router_lists_saved_names() {
        let c = two_routers();
        let err = c
            .resolve_url(Some("lba".into()), "http://builtin")
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown router 'lba'"), "{err}");
        assert!(err.contains("home, lab"), "{err}");
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn save_then_load_round_trips_routers() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let c = two_routers();
        c.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path).unwrap(), c);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("default = \"lab\""), "{text}");
        assert!(text.contains("[routers.lab]"), "{text}");
    }

    #[test]
    fn load_from_old_format_returns_hint() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "base_url = \"http://old\"\n").unwrap();
        let err = Config::load_from(&path).unwrap_err().to_string();
        assert!(err.contains("old base_url format"), "{err}");
        assert!(err.contains("otctl config add <name> <url>"), "{err}");
    }

    #[test]
    fn load_from_invalid_toml_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not = [valid").unwrap();
        assert!(Config::load_from(&path).is_err());
    }
}
