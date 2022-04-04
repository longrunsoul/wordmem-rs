use std::{
    fs,
    path::PathBuf,
};

use anyhow::Result;

pub enum Encryption {
    SslTls,
    StartTls,
}

pub struct SyncConfig {
    pub imap_server_host: String,
    pub imap_server_port: u16,
    pub imap_encryption: Encryption,

    pub smtp_server_host: String,
    pub smtp_server_port: u16,
    pub smtp_encryption: Encryption,

    pub email: String,
}

pub struct AppConfig {
    pub sync: Option<SyncConfig>,
}

impl AppConfig {
    pub fn get_conf_dir() -> PathBuf {
        let mut conf_dir = PathBuf::new();
        conf_dir.push(dirs::config_dir().unwrap());
        conf_dir.push("wordmem");

        conf_dir
    }
    pub fn init_conf_dir() -> Result<()> {
        let conf_dir = Self::get_conf_dir();
        if !conf_dir.exists() {
            fs::create_dir(conf_dir)?
        }
        Ok(())
    }
}