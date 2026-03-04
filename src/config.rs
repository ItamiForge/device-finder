use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub platforms: PlatformConfig,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    #[serde(default)]
    pub default_filter: Option<Vec<String>>,
    #[serde(default)]
    pub exclude_devices: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default)]
    pub adb_path: Option<String>,
    #[serde(default)]
    pub xcrun_path: Option<String>,
    #[serde(default)]
    pub emulator_path: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            platforms: PlatformConfig {
                default_filter: None,
                exclude_devices: None,
            },
            tools: ToolsConfig {
                adb_path: None,
                xcrun_path: None,
                emulator_path: None,
            },
        }
    }

    pub fn config_dir() -> Result<PathBuf> {
        let base = directories::ProjectDirs::from("", "", "itamiforge")
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
        let config_dir = base.config_dir().join("device-finder");
        Ok(config_dir)
    }

    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let config_dir = config_file.parent().unwrap();

        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;

        Ok(())
    }

    pub fn example() -> String {
        r#"# Device Finder Configuration
# Save to ~/.config/itamiforge/device-finder/config.toml

[platforms]
# Filter devices by platform on startup
# default_filter = ["ios", "android"]

# Exclude specific devices
# exclude_devices = ["device-to-ignore-1", "device-to-ignore-2"]

[tools]
# Override tool paths if they're not in PATH
# adb_path = "/path/to/adb"
# xcrun_path = "/path/to/xcrun"
# emulator_path = "/path/to/emulator"
"#
        .to_string()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
