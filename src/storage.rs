use std::env;

use anyhow::Result;
use sqlite::Connection;


pub fn init_db() -> Result<()> {
    let mut db_path =
        match env::current_exe() {
            Err(e) => panic!("{}", e),
            Ok(p) => p,
        }.parent().unwrap().to_path_buf();
    db_path.push("wordmem.sqlite");

    if !db_path.exists() {
        let conn = Connection::open(db_path)?;
        conn.execute("\
            CREATE TABLE word (id INTEGER, name TEXT, meanings TEXT, period_days INTEGER, visit_time INTEGER, next_visit INTEGER);
        ")?;
    }

    Ok(())
}