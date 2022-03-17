use std::env;
use std::path::PathBuf;

use anyhow::Result;
use sqlite::Connection;

use crate::sql_value::*;
use crate::model::*;

fn make_db_path() -> PathBuf {
    let mut db_path =
        match env::current_exe() {
            Err(e) => panic!("{}", e),
            Ok(p) => p,
        }.parent().unwrap().to_path_buf();
    db_path.push("wordmem.sqlite");

    db_path
}

fn make_conn() -> Result<Connection> {
    let db_path = make_db_path();
    let conn = Connection::open(db_path)?;
    Ok(conn)
}

pub fn init_db() -> Result<()> {
    let db_path = make_db_path();
    if !db_path.exists() {
        let conn = make_conn()?;
        conn.execute("\
            CREATE TABLE word (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                meanings TEXT NOT NULL,
                period_days INTEGER NOT NULL,
                last_visit INTEGER NOT NULL,
                next_visit INTEGER NOT NULL
            );
        ")?;
    }

    Ok(())
}

pub fn get_by_word_name(name: SqlVal) -> Result<Option<Word>> {
    let mut result= None;
    let conn = make_conn()?;
    conn.iterate(
        format!("SELECT * FROM word WHERE name = {}", name),
        |pairs| {
            result = Some(Word::from_sqlite_pairs(pairs));
            false
        }
    )?;
    if result.is_none() {
        return Ok(None);
    }

    let word = result.unwrap()?;
    Ok(Some(word))
}