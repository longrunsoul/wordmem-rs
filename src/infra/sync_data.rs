use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{Error, Result};
use bzip2::{read::BzDecoder, write::BzEncoder, Compression};
use chrono::{DateTime, SecondsFormat, Utc};
use lettre::{
    self,
    message::{header::ContentType, Attachment},
    transport::smtp::authentication::Credentials,
    SmtpTransport, Transport,
};
use mail_parser::{self, BodyPart};
use regex::Regex;
use tar::Archive;

use crate::infra::{Db, Encryption, SyncConfig, IMAP_ID_COMMAND};

pub struct SyncData {
    pub data_time: DateTime<Utc>,
    pub db_bytes: Vec<u8>,
}

impl SyncData {
    pub fn pull_data(sync_config: Option<&SyncConfig>) -> Result<Option<SyncData>> {
        if sync_config.is_none() {
            return Err(Error::msg("Sync config missing."));
        }

        let sync_config = sync_config.unwrap();
        let password = sync_config.get_password()?;
        if password.is_none() {
            return Err(Error::msg("Sync password missing."));
        }

        let password = password.unwrap();
        let mut client = imap::ClientBuilder::new(
            sync_config.imap_server_host.clone(),
            sync_config.imap_server_port,
        );
        let client = match sync_config.imap_encryption {
            Encryption::SslTls => &mut client,
            Encryption::StartTls => client.starttls(),
        }
        .native_tls()?;
        let mut imap_session = client
            .login(&sync_config.email, &password)
            .map_err(|e| e.0)?;

        let caps = imap_session.capabilities()?;
        if caps.has_str("ID") {
            imap_session.run_command_and_check_ok(IMAP_ID_COMMAND)?;
        }

        imap_session.select("INBOX")?;

        let mut message = None;
        let mut fetches;
        if !sync_config.workaround_imap_search.unwrap() {
            let mut seq_list: Vec<_> = imap_session
                .search("SUBJECT [wordmem][sync]")?
                .into_iter()
                .collect();
            if !seq_list.is_empty() {
                return Ok(None);
            }

            seq_list.sort();
            let last_seq = seq_list.last().unwrap();
            fetches = imap_session.fetch(last_seq.to_string(), "RFC822")?;
            message = Some(fetches.iter().next().unwrap());
        } else {
            let seq_list = imap_session.search("ALL")?;
            if seq_list.is_empty() {
                return Ok(None);
            }

            let mut seq_list: Vec<_> = seq_list.iter().collect();
            seq_list.sort();
            seq_list.reverse();
            for chunk in seq_list.chunks(50) {
                let seqset = chunk
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                fetches = imap_session.fetch(seqset, "RFC822")?;

                let mut fetches_sorted = fetches.iter().collect::<Vec<_>>();
                fetches_sorted.sort_by_key(|m| {
                    mail_parser::Message::parse(m.body().unwrap())
                        .unwrap()
                        .get_date()
                        .unwrap()
                        .to_timestamp()
                        .unwrap()
                });
                fetches_sorted.reverse();

                message = fetches_sorted.into_iter().find(|m| {
                    mail_parser::Message::parse(m.body().unwrap())
                        .unwrap()
                        .get_subject()
                        .unwrap()
                        .starts_with("[wordmem][sync]")
                });
                if message.is_some() {
                    break;
                }
            }
        }
        if message.is_none() {
            return Ok(None);
        }

        let message = mail_parser::Message::parse(message.unwrap().body().unwrap()).unwrap();
        let subject = message.get_subject().unwrap();
        let regex = Regex::new(r"\[(?P<info>[^\]]*)\]")?;
        let last_cap = regex.captures_iter(subject).last().unwrap();
        let time_str = last_cap.name("info").unwrap().as_str();
        let data_time = DateTime::parse_from_rfc3339(time_str)?.with_timezone(&Utc);
        let bzip_bytes = message
            .get_attachment(0)
            .unwrap()
            .unwrap_binary()
            .get_contents();

        // extract db bytes
        let mut db_bytes = Vec::new();
        {
            // decompress bzip bytes
            let mut tar_bytes = Vec::new();
            let mut decompressor = BzDecoder::new(bzip_bytes);
            decompressor.read_to_end(&mut tar_bytes)?;

            // extract tar
            let mut tar = Archive::new(tar_bytes.as_slice());
            let mut db_file = tar.entries()?.into_iter().next().unwrap()?;
            db_file.read_to_end(&mut db_bytes)?;
        }

        Ok(Some(SyncData {
            data_time,
            db_bytes,
        }))
    }

    pub fn push_data(&self, sync_config: Option<&SyncConfig>) -> Result<()> {
        if sync_config.is_none() {
            return Err(Error::msg("Sync config missing."));
        }

        let sync_config = sync_config.unwrap();
        let password = sync_config.get_password()?;
        if password.is_none() {
            return Err(Error::msg("Sync password missing."));
        }

        // tar the db file and get bytes
        let mut tar_bytes = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut tar_bytes);
            let mut db_file = tempfile::tempfile()?;
            db_file.write_all(&self.db_bytes)?;
            db_file.seek(SeekFrom::Start(0))?;
            tar.append_file(Db::get_default_db_name(), &mut db_file)?;
            tar.finish()?;
        }

        // bzip the tar bytes
        let mut bzip_bytes = Vec::new();
        {
            let mut compressor = BzEncoder::new(&mut bzip_bytes, Compression::default());
            compressor.write_all(&tar_bytes)?;
            compressor.finish()?;
        }

        let message = lettre::Message::builder()
            .from(sync_config.email.parse().unwrap())
            .to(sync_config.email.parse().unwrap())
            .subject(format!(
                "[wordmem][sync][{}]",
                self.data_time.to_rfc3339_opts(SecondsFormat::Secs, true)
            ))
            .singlepart(
                Attachment::new(format!("{}.tar.bz2", Db::get_default_db_name())).body(
                    bzip_bytes,
                    ContentType::parse("application/octet-stream").unwrap(),
                ),
            )?;
        let mailer = match sync_config.smtp_encryption {
            Encryption::SslTls => SmtpTransport::relay(&sync_config.smtp_server_host)?,
            Encryption::StartTls => SmtpTransport::starttls_relay(&sync_config.smtp_server_host)?,
        }
        .credentials(Credentials::new(
            sync_config.email.clone(),
            password.unwrap(),
        ))
        .port(sync_config.smtp_server_port)
        .build();
        mailer.send(&message)?;

        Ok(())
    }
}
