use std::env;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use sqlite::Connection;

use crate::infra::sql_value::*;
use crate::infra::model::*;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn get_default_db_name() -> String {
        "wordmem.sqlite".to_string()
    }
    pub fn get_default_db_path() -> PathBuf {
        let mut db_path =
            match env::current_exe() {
                Err(e) => panic!("{}", e),
                Ok(p) => p,
            }.parent().unwrap().to_path_buf();
        db_path.push(Db::get_default_db_name());

        db_path
    }

    pub fn new(db: Option<&str>) -> Result<Db> {
        Ok(Db {
            conn: match db {
                None => Connection::open(Db::get_default_db_path())?,
                Some(db) => Connection::open(db)?,
            },
        })
    }

    pub fn init_db(&self) -> Result<()> {
        self.conn.execute("\
            CREATE TABLE IF NOT EXISTS word (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                meanings TEXT NOT NULL,
                period_days INTEGER NOT NULL,
                last_visit INTEGER NOT NULL,
                next_visit INTEGER NOT NULL
            );
        ")?;
        Ok(())
    }

    pub fn get_by_col(&self, col: &str, val: SqlVal) -> Result<Option<Word>> {
        let mut result = None;
        self.conn.iterate(
            format!("SELECT * FROM word WHERE {} = {} LIMIT 1;", col, val),
            |pairs| {
                result = Some(Word::from_sqlite_pairs(pairs));
                true
            },
        )?;
        if result.is_none() {
            return Ok(None);
        }

        let word = result.unwrap()?;
        Ok(Some(word))
    }

    pub fn insert_word(&self, word: &Word) -> Result<()> {
        self.conn.execute(
            format!(
                "INSERT INTO word (name, meanings, period_days, last_visit, next_visit)
                VALUES ({}, {}, {}, {}, {});",
                SqlVal::Text(&word.name),
                SqlVal::Text(&word.meanings),
                SqlVal::Integer(word.period_days as i64),
                SqlVal::Integer(word.last_visit.timestamp()),
                SqlVal::Integer(word.next_visit.timestamp())
            )
        )?;

        Ok(())
    }

    pub fn update_word(&self, word: &Word) -> Result<()> {
        self.conn.execute(
            format!(
                "UPDATE word
                SET
                    name={},
                    meanings={},
                    period_days={},
                    last_visit={},
                    next_visit={}
                WHERE id = {};",
                SqlVal::Text(&word.name),
                SqlVal::Text(&word.meanings),
                SqlVal::Integer(word.period_days as i64),
                SqlVal::Integer(word.last_visit.timestamp()),
                SqlVal::Integer(word.next_visit.timestamp()),
                SqlVal::Integer(word.id.unwrap())
            )
        )?;

        Ok(())
    }

    pub fn del_word(&self, id: i64) -> Result<()> {
        self.conn.execute(
            format!(
                "DELETE FROM word WHERE id = {};",
                SqlVal::Integer(id)
            )
        )?;

        Ok(())
    }

    pub fn clear_words(&self) -> Result<()> {
        self.conn.execute("DELETE FROM word;")?;
        Ok(())
    }

    pub fn get_one_word_to_test(&self) -> Result<Option<Word>> {
        let mut result = None;
        let now = Utc::now();
        self.conn.iterate(
            format!(
                "SELECT * FROM word WHERE next_visit <= {} ORDER BY next_visit ASC LIMIT 1;",
                SqlVal::Integer(now.timestamp())
            ),
            |pairs| {
                result = Some(Word::from_sqlite_pairs(pairs));
                true
            },
        )?;
        if result.is_none() {
            return Ok(None);
        }

        let word = result.unwrap()?;
        Ok(Some(word))
    }
}

#[cfg(test)]
mod db_tests {
    use anyhow::Result;
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_crud() -> Result<()> {
        let db = Db::new(Some(":memory:"))?;
        db.init_db()?;

        // create
        let mut word_new = Word {
            id: None,
            name: "name".to_string(),
            meanings: "m1;m2;m3".to_string(),
            period_days: 3,
            last_visit: Utc.datetime_from_str("2022-03-21 09:09:33", "%Y-%m-%d %H:%M:%S")?,
            next_visit: Utc.datetime_from_str("2022-03-22 09:09:33", "%Y-%m-%d %H:%M:%S")?,
        };
        db.insert_word(&word_new)?;
        let mut word = db.get_by_col("name", SqlVal::Text("name"))?.unwrap();
        word_new.id = word.id;
        assert_eq!(word, word_new);

        // update
        word.name = "world".to_string();
        db.update_word(&word)?;
        let word = db.get_by_col("id", SqlVal::Integer(word.id.unwrap()))?.unwrap();
        assert_eq!(word.name, "world".to_string());

        // delete
        db.del_word(word.id.unwrap())?;
        let word = db.get_by_col("id", SqlVal::Integer(word.id.unwrap()))?;
        assert_eq!(word, None);

        Ok(())
    }
}