use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use mail_parser::{BodyPart, Message};

use crate::infra::SyncKeys;

pub struct SyncData {
    pub data_time: DateTime<Utc>,
    pub db_bytes: Vec<u8>,
}

pub struct Syncer {}

impl Syncer {
    pub fn get_data() -> Result<Option<SyncData>> {
        let sync_keys = SyncKeys::get_keys()?;
        if sync_keys.is_none() {
            return Ok(None);
        }

        let sync_keys = sync_keys.unwrap();
        let tls = native_tls::TlsConnector::builder().build().unwrap();
        let client = imap::connect((sync_keys.email_domain.as_str(), 993), &sync_keys.email_domain, &tls).unwrap();
        let mut imap_session = client.login(&sync_keys.username, &sync_keys.password).map_err(|e| e.0)?;

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

        let message = Message::parse(message.body().unwrap()).unwrap();
        Ok(Some(SyncData {
            data_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp(
                    message.get_date().unwrap().to_timestamp().unwrap(),
                    0
                ),
                Utc
            ),
            db_bytes: message.get_attachment(0).unwrap().unwrap_binary().get_contents().iter().cloned().collect(),
        }))
    }
}