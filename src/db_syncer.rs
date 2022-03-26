use std::fs;
use std::io::{self, BufRead, Write};
use anyhow::Result;
use chrono::Utc;
use lettre::{
    SmtpTransport,
    Transport,
    transport::smtp::authentication::Credentials,
};
use crate::Db;

use crate::infra::{SyncData, SyncKeys};

pub fn test_sync_keys(keys: &SyncKeys) -> Result<bool> {
    println!("Testing sync keys...");

    println!("Sending a test mail");
    let now = Utc::now().to_rfc3339();
    let subject = format!("[wordmem][test][{}]", now);
    let message = lettre::Message::builder()
        .from(keys.email.parse().unwrap())
        .to(keys.email.parse().unwrap())
        .subject(&subject)
        .body(String::new())?;
    let mailer = SmtpTransport::relay(&keys.smtp_server)?
        .credentials(Credentials::new(keys.email.to_string(), keys.password.to_string()))
        .build();
    if let Err(e) = mailer.send(&message) {
        println!("Failed. Error: {}", e);
        return Ok(false);
    }

    println!("Reading the mail just sent...");
    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((keys.imap_server.as_str(), 993), keys.imap_server.as_str(), &tls)?;
    let imap_session = client.login(&keys.email, &keys.password);
    if let Err((e, c)) = imap_session {
        println!("Failed. Error: {}", e);
        return Ok(false);
    }

    let mut imap_session = imap_session.unwrap();
    imap_session.select("INBOX")?;
    let seq_list = imap_session.search(&subject)?;
    if seq_list.is_empty() {
        println!("Failed. Test mail not found in INBOX");
        return Ok(false);
    }

    println!("Success");
    Ok(true)
}

pub fn read_sync_keys() -> Result<SyncKeys> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    print!("Enter IMAP server:");
    let imap_server = lines.next().unwrap()?;

    print!("Enter SMTP server:");
    let smtp_server = lines.next().unwrap()?;

    print!("Enter email:");
    let email = lines.next().unwrap()?;

    let password = rpassword::prompt_password("Enter password:")?;

    Ok(SyncKeys {
        imap_server,
        smtp_server,
        email,
        password,
    })
}

pub fn push_data_to_email() -> Result<bool> {
    let sync_keys = SyncKeys::get_keys()?;
    if sync_keys.is_none() {
        println!("Email not signed in. Syncing aborted.");
        return Ok(false);
    }

    println!("Pushing data to email...");
    let sync_keys = sync_keys.unwrap();
    SyncData{
        data_time: Utc::now(),
        db_bytes: fs::read(Db::get_default_db_path())?
    }.pub_data()?;

    println!("Success.");
    Ok(true)
}

pub fn pull_data_from_email() -> Result<bool> {
    let sync_keys = SyncKeys::get_keys()?;
    if sync_keys.is_none() {
        println!("Email not signed in. Syncing aborted.");
        return Ok(false);
    }

    println!("Pulling data from email...");
    let sync_keys = sync_keys.unwrap();
    let sync_data = SyncData::get_data()?;
    if sync_data.is_none() {
        println!("Data not found in email. Syncing aborted.");
        return Ok(false);
    }

    let sync_data = sync_data.unwrap();
    let mut open_options = fs::OpenOptions::new().truncate(true).write(true).open(Db::get_default_db_path())?;
    open_options.write_all(&sync_data.db_bytes)?;

    println!("Success.");
    Ok(true)
}