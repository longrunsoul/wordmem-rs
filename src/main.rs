mod infra;
mod word_manager;
mod revisit_planner;
mod word_visitor;
mod db_syncer;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::infra::{Db, SyncKeys};

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

enum PostAction {
    None,
    PushData,
    PullData,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let post_action = {
        let db = Db::new(Db::get_default_db_path())?;
        match &cli.command {
            Commands::Take => {
                if {
                    word_manager::read_words_to_db(&db)?
                } > 0 {
                    PostAction::PushData
                } else {
                    PostAction::None
                }
            }
            Commands::Test => {
                word_visitor::do_tests(&db)?;
                PostAction::PushData
            }
            Commands::Signin => {
                let sync_keys = db_syncer::read_sync_keys()?;
                if db_syncer::test_sync_keys(&sync_keys)? {
                    sync_keys.set_keys()?;
                }
                PostAction::None
            }
            Commands::Signout => {
                SyncKeys::clear_keys()?;
                PostAction::None
            }
            Commands::Push => PostAction::PushData,
            Commands::Pull => PostAction::PullData,
            Commands::Change { word } => {
                word_manager::change_word(&db, word)?;
                PostAction::PushData
            }
            Commands::Delete { word } => {
                word_manager::delete_word(&db, word)?;
                PostAction::PushData
            }
            Commands::Open { word } => {
                word_manager::open_word(word)?;
                PostAction::None
            }
            Commands::Clear => {
                if word_manager::clear_words(&db)? {
                    PostAction::PushData
                } else {
                    PostAction::None
                }
            }
            Commands::Export { file } => {
                word_manager::export_words(&db, file)?;
                PostAction::None
            }
            Commands::Import { file } => {
                word_manager::import_words(&db, file)?;
                PostAction::None
            }
        }
    };
    match post_action {
        PostAction::None => {}
        PostAction::PushData => { db_syncer::push_data_to_email()?; }
        PostAction::PullData => { db_syncer::pull_data_from_email()?; }
    }

    Ok(())
}
