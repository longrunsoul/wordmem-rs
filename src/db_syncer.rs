use std::{
    fs,
    io::{self, BufRead, Write},
    str::FromStr,
};

use anyhow::Result;
use chrono::{SecondsFormat, Utc};
use lettre::{transport::smtp::authentication::Credentials, SmtpTransport, Transport};

use crate::infra::{AppConfig, Db, Encryption, SyncConfig, SyncData};

pub fn test_sync_config(sync_config: &SyncConfig) -> Result<bool> {
    println!("Testing sync config...");
    let password = sync_config.get_password()?;
    if password.is_none() {
        println!("Failed. Password missing.");
        return Ok(false);
    }

    println!("Sending a test mail...");
    let password = password.unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let subject = format!("[wordmem][test][{}]", now);
    let message = lettre::Message::builder()
        .from(sync_config.email.parse().unwrap())
        .to(sync_config.email.parse().unwrap())
        .subject(&subject)
        .body(String::new())?;
    let mailer = match sync_config.smtp_encryption {
        Encryption::SslTls => SmtpTransport::relay(&sync_config.smtp_server_host)?,
        Encryption::StartTls => SmtpTransport::starttls_relay(&sync_config.smtp_server_host)?,
    }
    .credentials(Credentials::new(
        sync_config.email.clone(),
        password.clone(),
    ))
    .port(sync_config.smtp_server_port)
    .build();
    if let Err(e) = mailer.send(&message) {
        println!("Failed. Error: {}", e);
        return Ok(false);
    }

    println!("Reading the mail just sent...");
    let mut client =
        imap::ClientBuilder::new(&sync_config.imap_server_host, sync_config.imap_server_port);
    let client = match sync_config.imap_encryption {
        Encryption::SslTls => &mut client,
        Encryption::StartTls => client.starttls(),
    }
    .native_tls()?;
    let imap_session = client.login(&sync_config.email, &password);
    if let Err((e, _c)) = imap_session {
        println!("Failed. Error: {}", e);
        return Ok(false);
    }

    let mut imap_session = imap_session.unwrap();
    imap_session.select("INBOX")?;
    let seq_list = imap_session.search(format!("SUBJECT {}", subject))?;
    if seq_list.is_empty() {
        println!("Failed. Test mail not found in INBOX.");
        return Ok(false);
    }

    println!("Success.");
    Ok(true)
}

pub fn read_sync_config() -> Result<SyncConfig> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    print!("Enter IMAP server host: ");
    io::stdout().flush()?;
    let imap_server_host = lines.next().unwrap()?.trim().to_string();

    print!("Enter IMAP server port: ");
    io::stdout().flush()?;
    let imap_server_port = u16::from_str(lines.next().unwrap()?.trim())?;

    print!("Enter IMAP server encryption [ssltls/starttls]: ");
    io::stdout().flush()?;
    let imap_encryption = Encryption::from_str(lines.next().unwrap()?.trim())?;

    print!("Enter SMTP server host: ");
    io::stdout().flush()?;
    let smtp_server_host = lines.next().unwrap()?.trim().to_string();

    print!("Enter SMTP server port: ");
    io::stdout().flush()?;
    let smtp_server_port = u16::from_str(lines.next().unwrap()?.trim())?;

    print!("Enter SMTP server encryption [ssltls/starttls]: ");
    io::stdout().flush()?;
    let smtp_encryption = Encryption::from_str(lines.next().unwrap()?.trim())?;

    print!("Enter email: ");
    io::stdout().flush()?;
    let email = lines.next().unwrap()?.trim().to_string();

    let sync_config = SyncConfig {
        imap_server_host,
        imap_server_port,
        imap_encryption,

        smtp_server_host,
        smtp_server_port,
        smtp_encryption,

        email,
    };

    let password = rpassword::prompt_password("Enter password: ")?;
    sync_config.set_password(&password)?;

    Ok(sync_config)
}

pub fn push_data_to_email(app_config: Option<&AppConfig>) -> Result<bool> {
    if app_config.is_none() {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    let app_config = app_config.unwrap();
    if app_config.sync.is_none() {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    println!("Pushing data to email...");
    SyncData {
        data_time: Utc::now(),
        db_bytes: fs::read(Db::get_default_db_path())?,
    }
    .push_data(app_config.sync.as_ref())?;

    println!("Success.");
    Ok(true)
}

pub fn pull_data_from_email(app_config: Option<&AppConfig>) -> Result<bool> {
    if app_config.is_none() {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    let app_config = app_config.unwrap();
    if app_config.sync.is_none() {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    println!("Pulling data from email...");
    let sync_data = SyncData::pull_data(app_config.sync.as_ref())?;
    if sync_data.is_none() {
        println!("Data not found in email. Syncing aborted.");
        return Ok(false);
    }

    println!("Merging data...");
    let sync_data = sync_data.unwrap();
    let email_db = tempfile::Builder::new().tempfile()?;
    fs::write(email_db.path(), sync_data.db_bytes)?;
    let words = Db::new(email_db.path())?.get_all_words()?;
    let local_db = Db::new(Db::get_default_db_path())?;
    for w in words {
        local_db.upsert_by_name(&w, true)?;
    }

    println!("Success.");
    Ok(true)
}
