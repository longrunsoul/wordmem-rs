use anyhow::{Result, Error};
use chrono::{DateTime, Utc};
use mail_parser::{self, BodyPart};
use lettre;
use lettre::message::{Attachment, MultiPart};
use lettre::message::header::ContentType;
use lettre::{SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use regex::Regex;

use crate::infra::{Db, SyncKeys};

const LOCK_FILENAME: &str = "sync.lock";

pub struct SyncData {
    pub data_time: DateTime<Utc>,
    pub db_bytes: Vec<u8>,
}

impl SyncData {
    // TODO: add lock file logics, to make syncing robust

    pub fn get_data() -> Result<Option<SyncData>> {
        let sync_keys = SyncKeys::get_keys()?;
        if sync_keys.is_none() {
            return Ok(None);
        }

        let sync_keys = sync_keys.unwrap();
        let tls = native_tls::TlsConnector::builder().build()?;
        let client = imap::connect((sync_keys.imap_server.as_str(), 993), &sync_keys.imap_server, &tls).unwrap();
        let mut imap_session = client.login(&sync_keys.email, &sync_keys.password).map_err(|e| e.0)?;

        let message;
        let mut message_list;
        loop {
            imap_session.select("INBOX")?;
            let mut seq_list: Vec<_> = imap_session.search("[wordmem][sync]")?.into_iter().collect();
            if seq_list.is_empty() {
                return Ok(None);
            }

            seq_list.sort();
            let last_seq = seq_list.last().unwrap();
            message_list = imap_session.fetch(last_seq.to_string(), "RFC822")?;
            let m = message_list.iter().next();
            if m.is_none() {
                continue;
            }

            message = m.unwrap();
            break;
        }

        let message = mail_parser::Message::parse(message.body().unwrap()).unwrap();
        let subject = message.get_subject().unwrap();
        let regex = Regex::new(r"\[(?P<info>[^\]]*)\]")?;
        let last_cap = regex.captures_iter(subject).last().unwrap();
        let time_str = last_cap.name("info").unwrap().as_str();
        let data_time = DateTime::parse_from_rfc3339(time_str)?.with_timezone(&Utc);
        Ok(Some(SyncData {
            data_time,
            db_bytes: message
                .get_attachment(0)
                .unwrap()
                .unwrap_binary()
                .get_contents()
                .iter()
                .cloned()
                .collect(),
        }))
    }

    pub fn pub_data(self) -> Result<()> {
        let sync_keys = SyncKeys::get_keys()?;
        if sync_keys.is_none() {
            return Err(Error::msg("No keys found for syncing"));
        }

        let sync_keys = sync_keys.unwrap();
        let message = lettre::Message::builder()
            .from(sync_keys.email.parse().unwrap())
            .to(sync_keys.email.parse().unwrap())
            .subject(format!("[wordmem][sync][{}]", self.data_time.to_rfc3339()))
            .multipart(
                MultiPart::builder()
                    .singlepart(
                        Attachment::new(Db::get_default_db_name())
                            .body(
                                self.db_bytes,
                                ContentType::parse("application/octet-stream").unwrap(),
                            )
                    )
            )?;
        let creds = Credentials::new(sync_keys.email, sync_keys.password);
        let mailer = SmtpTransport::relay(sync_keys.smtp_server.as_str())?.credentials(creds).build();
        mailer.send(&message)?;

        Ok(())
    }
}