use std::{
    fmt::{Display, Formatter},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::infra::KEYRING_SERVICE;

#[derive(Debug, Serialize, Deserialize)]
pub enum Encryption {
    SslTls,
    StartTls,
}

impl Display for Encryption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Encryption::SslTls => "ssltls",
            Encryption::StartTls => "starttls",
        })
    }
}

impl FromStr for Encryption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "ssltls" => Ok(Encryption::SslTls),
            "starttls" => Ok(Encryption::StartTls),

            _ => Err(Self::Err::msg(format!(
                "Unrecognized encryption method: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    pub imap_server_host: String,
    pub imap_server_port: u16,
    pub imap_encryption: Encryption,

    pub smtp_server_host: String,
    pub smtp_server_port: u16,
    pub smtp_encryption: Encryption,

    pub email: String,
}

impl SyncConfig {
    pub fn get_password(&self) -> Result<Option<String>> {
        let password = keyring::Entry::new(KEYRING_SERVICE, &self.email).get_password();
        if let Err(keyring::Error::NoEntry) = password {
            return Ok(None);
        }

        Ok(Some(password.unwrap()))
    }

    pub fn set_password(&self, password: &str) -> Result<()> {
        keyring::Entry::new(KEYRING_SERVICE, &self.email).set_password(password)?;
        Ok(())
    }

    pub fn clear_password(&self) -> Result<()> {
        keyring::Entry::new(KEYRING_SERVICE, &self.email).delete_password()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub sync: Option<SyncConfig>,
}

impl AppConfig {
    pub fn get_default_conf_dir() -> PathBuf {
        let mut conf_dir = PathBuf::new();
        conf_dir.push(dirs::config_dir().unwrap());
        conf_dir.push("wordmem");

        conf_dir
    }

    pub fn get_default_conf_path() -> PathBuf {
        let mut conf_path = PathBuf::new();
        conf_path.push(Self::get_default_conf_dir());
        conf_path.push("wordmem.conf");

        conf_path
    }

    pub fn load_from_file<P>(file: &P) -> Result<Option<AppConfig>>
    where
        P: AsRef<Path>,
    {
        if !file.as_ref().exists() {
            return Ok(None);
        }

        let json_text = fs::read_to_string(file)?;
        let app_config: AppConfig = serde_json::from_str(&json_text)?;
        Ok(Some(app_config))
    }

    pub fn save_to_file<P>(&self, file: &P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let dir = file.as_ref().parent();
        if dir.is_some() {
            let dir = dir.unwrap();
            if !dir.exists() {
                fs::create_dir(dir)?;
            }
        }

        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)?;
        serde_json::to_writer(file, self)?;

        Ok(())
    }
}
