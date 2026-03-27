use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::YukiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub default_admin: String,
    pub administrations: BTreeMap<String, String>,
}

impl Config {
    pub fn default_path() -> PathBuf {
        #[cfg(unix)]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config/yuki/config.toml")
        }
        #[cfg(not(unix))]
        {
            directories::ProjectDirs::from("nl", "yukiworks", "yuki")
                .map(|d| d.config_dir().join("config.toml"))
                .unwrap_or_else(|| PathBuf::from("config.toml"))
        }
    }

    pub fn load() -> Result<Self, YukiError> {
        Self::load_from(&Self::default_path())
    }

    pub fn load_from(path: &Path) -> Result<Self, YukiError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| YukiError::Config(format!("failed to read {}: {e}", path.display())))?;
        toml::from_str(&content)
            .map_err(|e| YukiError::Config(format!("invalid config: {e}")))
    }

    pub fn save_to(&self, path: &Path) -> Result<(), YukiError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| YukiError::Config(format!("cannot create config dir: {e}")))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| YukiError::Config(format!("serialize error: {e}")))?;
        std::fs::write(path, content)
            .map_err(|e| YukiError::Config(format!("failed to write {}: {e}", path.display())))
    }

    pub fn resolve_admin(&self, override_name: Option<&str>) -> Result<String, YukiError> {
        let name = override_name.unwrap_or(&self.default_admin);
        self.administrations
            .get(name)
            .cloned()
            .ok_or_else(|| YukiError::Config(format!("unknown administration: {name}")))
    }
}
