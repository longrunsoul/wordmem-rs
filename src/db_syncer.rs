use std::{
    fs,
    io::{self, BufRead, Write},
};

use anyhow::Result;
use chrono::Utc;
use lettre::{
    SmtpTransport,
    Transport,
    transport::smtp::authentication::Credentials,
};

use crate::{
    word_manager,
    infra::{Db, SyncData, SyncKeys},
};

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
    if let Err((e, _c)) = imap_session {
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

    print!("Enter IMAP server: ");
    io::stdout().flush()?;
    let imap_server = lines.next().unwrap()?;

    print!("Enter SMTP server: ");
    io::stdout().flush()?;
    let smtp_server = lines.next().unwrap()?;

    print!("Enter email: ");
    io::stdout().flush()?;
    let email = lines.next().unwrap()?;

    let password = rpassword::prompt_password("Enter password: ")?;

    Ok(SyncKeys {
        imap_server,
        smtp_server,
        email,
        password,
    })
}

pub fn push_data_to_email() -> Result<bool> {
    if !SyncKeys::exists()? {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    println!("Pushing data to email...");
    SyncData{
        data_time: Utc::now(),
        db_bytes: fs::read(Db::get_default_db_path())?
    }.push_data()?;

    println!("Success.");
    Ok(true)
}

pub fn pull_data_from_email() -> Result<bool> {
    if !SyncKeys::exists()? {
        println!("Email not signed in. No need to sync.");
        return Ok(false);
    }

    println!("Pulling data from email...");
    let sync_data = SyncData::pull_data()?;
    if sync_data.is_none() {
        println!("Data not found in email. Syncing aborted.");
        return Ok(false);
    }

    println!("Merging data...");
    let sync_data = sync_data.unwrap();
    let email_json = tempfile::Builder::new().tempfile()?;
    let email_db = tempfile::Builder::new().tempfile()?;
    fs::write(email_db.path(), sync_data.db_bytes)?;
    word_manager::export_words(&Db::new(email_db.path())?, email_json.path(), false)?;
    word_manager::import_words(&Db::new(Db::get_default_db_path())?, email_json.path(), false)?;

    println!("Success.");
    Ok(true)
}