use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{self, ErrorKind};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub domains: Vec<Domain>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub token: String,
    pub ip: Option<Ip>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ip {
    Public,
    IPv4(String),
    IPv6(String),
}

impl Config {
    /// Load the configuration from a custom, user, or system path.
    pub async fn load(config_path: Option<String>) -> io::Result<Config> {
        let path = Self::get_config_path(config_path).await?;
        Self::check_secure_file_mode(&path).await?;
        Self::read_config_file(&path).await
    }

    /// Resolve configuration file path from custom input, user config dir, or system default.
    async fn get_config_path(custom_path: Option<String>) -> io::Result<PathBuf> {
        if let Some(custom) = custom_path {
            return Ok(PathBuf::from(custom));
        }

        let user_path = BaseDirs::new()
            .map(|d| d.home_dir().join(".config/ducksync/config.yaml"))
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

        if fs::try_exists(&user_path).await.unwrap_or(false) {
            return Ok(user_path);
        }

        let system_path = PathBuf::from("/etc/ducksync/config.yaml");
        if fs::try_exists(&system_path).await.unwrap_or(false) {
            return Ok(system_path);
        }

        Err(io::Error::new(
            ErrorKind::NotFound,
            "Config file not found in user or system path",
        ))
    }

    /// Ensure the file is mode 600 (rw-------).
    pub async fn check_secure_file_mode<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let metadata = fs::metadata(&path).await?;
        let mode = metadata.permissions().mode() & 0o777;

        if mode != 0o600 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "Config file '{}' must have mode 600, found {:o}",
                    path.as_ref().display(),
                    mode
                ),
            ));
        }
        Ok(())
    }

    /// Read and deserialize YAML configuration.
    async fn read_config_file<P: AsRef<Path>>(path: P) -> io::Result<Config> {
        let content = fs::read_to_string(&path).await?;
        serde_yaml::from_str(&content)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))
    }
}
