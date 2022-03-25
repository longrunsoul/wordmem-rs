use anyhow::Result;
use chrono::Utc;
use lettre::{
    SmtpTransport,
    Transport,
    transport::smtp::authentication::Credentials,
};

use crate::infra::SyncKeys;

pub fn test_sync_keys(keys: &SyncKeys) -> Result<bool> {
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
    if mailer.send(&message).is_err() {
        return Ok(false);
    }

    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((keys.imap_server.as_str(), 993), keys.imap_server.as_str(), &tls)?;
    let imap_session = client.login(&keys.email, &keys.password);
    if imap_session.is_err() {
        return Ok(false);
    }

    let mut imap_session = imap_session.unwrap();
    imap_session.select("INBOX")?;
    let seq_list = imap_session.search(&subject)?;
    if seq_list.is_empty() {
        return Ok(false);
    }

    Ok(true)
}