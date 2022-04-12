//! `wordmem` is a helper tool for language learning, focusing on vocabulary. It takes words and explanation from user, and then makes user revisit them periodically so that user can memorize it.
//! Currently, it is an in-progress project.
//!
//! ## IDEAS
//!
//! The application splits to 3 parts:
//! - `word-manager`, which takes and manages words from user
//! - `word-visitor`, which makes user revisit the words periodically
//! - `revisit-planner`, which plans the revisiting schedule
//! - `db-syncer`, which syncs data from/to email
//!
//! Revisiting means test. User need to spell out the word and the explanation respectively in 2 passes.
//!
//! The revisiting is planed to start at the 1st, 2nd, 4th, 8th, 16th, 32end, 64th, 128th day since the last visiting. Correct answer will move the revisiting schedule to next planed time. On the contrary, wrong answer will move the plan backwards.
//!
//! When taking words from user, user should only input a single meaning at one time, but different meanings at each time. That is, multiple meanings will be taken for the same word as time goes.
//!
//! And, while doing the test, user should separate different meanings by "`;`". And user should answer all the meanings which are taken util then.
//!
//! Additionally, punctuations will be normalized when comparing answers.
//!
//! ## DESIGN
//!
//! Features:
//! - Storage can be synced via email.
//! - Security keys should be stored in system keyring.
//! - Words can be exported to/imported from file.
//!
//! Commandline interface:
//! - `wordmem take`: take words from user.
//! - `wordmem test`: do tests.
//! - `wordmem signin`: sign in email to enable syncing.
//! - `wordmem signout`: sign out email to disable syncing.
//! - `wordmem push`: forcibly push data to email to keep synced.
//! - `wordmem pull`: forcibly pull data from email to keep synced.
//! - `wordmem change <word>`: change meanings of an existing word.
//! - `wordmem delete <word>`: delete a word.
//! - `wordmem open <word>`: open a word on https://translate.bing.com.
//! - `wordmem clear`: remove all words in DB.
//! - `wordmem export <file>`: export words to a file.
//! - `wordmem import <file>`: import words from a file.
//!
//! Implementation:
//! - SQLite for storage of words.
//! - JSON format for exported file of words.
//! - Compressed .sqlite file as attachment and with INI format config info as body in email for syncing.

mod db_syncer;
mod infra;
mod revisit_planner;
mod word_manager;
mod word_visitor;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::infra::{AppConfig, Db};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Take words from user
    Take,
    /// Do tests
    Test,
    /// Sign in email to enable syncing
    Signin,
    /// Sign out to disable syncing
    Signout,
    /// Forcibly push data to email to keep synced
    Push,
    /// Forcibly pull data from email to keep synced
    Pull,
    /// Change meanings of an existing word
    Change { word: String },
    /// Delete a word
    Delete { word: String },
    /// Open a word on https://translate.bing.com
    Open { word: String },
    /// Remove all words in DB
    Clear,
    /// Export words to a file
    Export { file: String },
    /// Import words from a file
    Import { file: String },
}

fn pull_data() -> Result<()> {
    let default_conf_file = AppConfig::get_default_conf_path();
    let app_config = AppConfig::load_from_file(&default_conf_file)?;
    db_syncer::pull_data_from_email(app_config.as_ref())?;

    Ok(())
}

fn push_data() -> Result<()> {
    let default_conf_file = AppConfig::get_default_conf_path();
    let app_config = AppConfig::load_from_file(&default_conf_file)?;
    db_syncer::push_data_to_email(app_config.as_ref())?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let default_db_file = Db::get_default_db_path();
    let default_conf_file = AppConfig::get_default_conf_path();
    match &cli.command {
        Commands::Take => {
            pull_data()?;
            if word_manager::read_words_to_db(&Db::new(default_db_file)?)? > 0 {
                push_data()?;
            }
        }
        Commands::Test => {
            pull_data()?;
            if word_visitor::do_tests(&Db::new(default_db_file)?)? > 0 {
                push_data()?;
            }
        }
        Commands::Signin => {
            let mut sync_config = db_syncer::read_sync_config()?;
            if db_syncer::test_sync_config(&mut sync_config)? {
                let app_config = AppConfig::load_from_file(&default_conf_file)?;
                let mut app_config = app_config.unwrap_or(AppConfig { sync: None });
                app_config.sync = Some(sync_config);
                app_config.save_to_file(&default_conf_file)?;
            } else {
                sync_config.clear_password()?;
            }
        }
        Commands::Signout => {
            let app_config = AppConfig::load_from_file(&default_conf_file)?;
            if let Some(mut app_config) = app_config {
                if let Some(sync_config) = app_config.sync {
                    sync_config.clear_password()?;
                }
                app_config.sync = None;
                app_config.save_to_file(&default_conf_file)?;
            }
        }
        Commands::Push => push_data()?,
        Commands::Pull => pull_data()?,
        Commands::Change { word } => {
            pull_data()?;
            if word_manager::change_word(&Db::new(default_db_file)?, word)? {
                push_data()?;
            }
        }
        Commands::Delete { word } => {
            pull_data()?;
            if word_manager::delete_word(&Db::new(default_db_file)?, word)? {
                push_data()?;
            }
        }
        Commands::Open { word } => {
            word_manager::open_word(word)?;
        }
        Commands::Clear => {
            if word_manager::clear_words(&Db::new(default_db_file)?)? {
                push_data()?;
            }
        }
        Commands::Export { file } => {
            pull_data()?;
            word_manager::export_words(&Db::new(default_db_file)?, file)?;
        }
        Commands::Import { file } => {
            pull_data()?;
            word_manager::import_words(&Db::new(default_db_file)?, file)?;
            push_data()?;
        }
    }

    Ok(())
}
