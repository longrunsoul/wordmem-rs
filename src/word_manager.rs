use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, Write},
    path::Path,
};

use anyhow::Result;

use crate::infra::{Db, SqlVal, StdResult, Word};

fn read_one_word<T>(lines: &mut T) -> Result<Option<Word>>
    where T: Iterator<Item=StdResult<String, std::io::Error>> {
    loop {
        let l = lines.next();
        if l.is_none() {
            return Ok(None);
        }

        let l = l.unwrap()?;
        let l = l.trim();
        if l.is_empty() {
            return Ok(None);
        }

        let pair = l.split_once("=");
        if pair.is_none() {
            println!("Unrecognized input. Format: <WORD>=<MEANING1>;<MEANING2>;...;<MEANINGn>");
            println!("Enter empty line to end listing.");
            continue;
        }

        let (name, meanings) = pair.unwrap();
        let name = name.trim();
        let meanings = meanings.trim();
        if name.is_empty() || meanings.is_empty() {
            println!("Name or meanings cannot be empty.");
            println!("Enter empty line to end listing.");
            continue;
        }

        let word = Word::from_name_and_meanings(name, meanings);
        break Ok(Some(word));
    }
}

pub fn read_words_to_db(db: &Db) -> Result<usize> {
    println!("Enter words, one word per line. Enter empty line to end listing.");
    println!("Format: <WORD>=<MEANING1>;<MEANING2>;...;<MEANINGn>;");
    println!("Example: right=the opposite of left;correct;");

    let mut count: usize = 0;
    let stdin = io::stdin();
    let mut stdin_lines = stdin.lock().lines();

    while let Some(word) = read_one_word(&mut stdin_lines)? {
        count += 1;

        let existing = db.get_by_col("name", SqlVal::Text(&word.name.trim().to_lowercase()))?;
        if existing.is_none() {
            db.insert_word(&word)?;
            continue;
        }

        let mut existing = existing.unwrap();
        existing.merge_meanings(&word.meanings);
        db.update_word(&existing)?;
    }

    Ok(count)
}

pub fn change_word(db: &Db, name: &str) -> Result<()> {
    let word = db.get_by_col("name", SqlVal::Text(name.trim()))?;
    if word.is_none() {
        println!("Word not found.");
        return Ok(());
    }

    let mut meanings;
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("Enter the meanings: ");
        io::stdout().flush()?;
        meanings = lines.next().unwrap()?;
        meanings = Word::norm_meanings(&meanings);
        if meanings.is_empty() {
            println!("Meanings cannot be empty.");
            continue;
        }

        break;
    }

    let mut word = word.unwrap();
    word.meanings = meanings;
    db.update_word(&word)?;

    println!("Word changed.");
    Ok(())
}

pub fn delete_word(db: &Db, name: &str) -> Result<()> {
    let word = db.get_by_col("name", SqlVal::Text(name.trim()))?;
    if word.is_none() {
        println!("Word not found.");
        return Ok(());
    }

    let word = word.unwrap();
    db.del_word(word.id.unwrap())?;

    println!("Word deleted.");
    Ok(())
}

pub fn open_word(name: &str) -> Result<()> {
    open::that(format!("https://translate.bing.com/?text={}", name.trim()))?;
    Ok(())
}

pub fn clear_words(db: &Db) -> Result<bool> {
    print!("Are you sure? [Y/N]: ");
    io::stdout().flush()?;
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let answer = lines.next().unwrap_or(Ok("N".to_string()))?.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        return Ok(false);
    }

    db.clear_words()?;
    println!("Words cleared.");
    Ok(true)
}

pub fn import_words<T>(db: &Db, file: T) -> Result<()>
    where T: AsRef<Path> {
    println!("Importing words from {}...", file.as_ref().display());
    let json = fs::read_to_string(file)?;
    let name_meanings_pairs: HashMap<String, String> = serde_json::from_str(&json)?;
    for (n, m) in name_meanings_pairs {
        println!("  {}={}", n, m);
        let word = Word::from_name_and_meanings(&n, &m);
        db.upsert_by_name(&word, false)?;
    }

    println!("All words imported.");
    Ok(())
}

pub fn export_words<T>(db: &Db, file: T) -> Result<()>
    where T: AsRef<Path> {
    println!("Exporting words to {}...", file.as_ref().display());
    let mut name_meanings_pairs = HashMap::new();
    for w in db.get_all_words()? {
        name_meanings_pairs.insert(w.name, w.meanings);
    }

    let json = serde_json::to_string(&name_meanings_pairs)?;
    let mut file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(file)?;
    file.write_all(json.as_bytes())?;

    println!("All words exported.");
    Ok(())
}