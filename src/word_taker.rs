use std::io;
use std::io::BufRead;

use anyhow::Result;
use chrono::Utc;

use crate::infra::*;
use crate::revisit_planner as planner;

pub fn read_one_word<T>(lines: &mut T) -> Result<Option<Word>>
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
        let meanings = Word::norm_meanings(meanings);
        if meanings.is_empty() {
            println!("Meanings cannot be empty.");
            println!("Enter empty line to end listing.");
            continue;
        }

        let now = Utc::now();
        let period_days = planner::get_init_period_days();
        let word = Word {
            name: name.to_string(),
            meanings,

            id: None,
            period_days,
            last_visit: now,
            next_visit: planner::get_next_visit_time(now, period_days),
        };

        break Ok(Some(word));
    }
}

pub fn read_words() -> Result<usize> {
    println!("Enter words, one word per line. Enter empty line to end listing.");
    println!("Format: <WORD>=<MEANING1>;<MEANING2>;...;<MEANINGn>;");
    println!("Example: right=the opposite of left;correct;");

    let mut count: usize = 0;
    let stdin = io::stdin();
    let mut stdin_lines = stdin.lock().lines();
    while let Some(word) = read_one_word(&mut stdin_lines)? {
        count += 1;

        let existing = storage::get_by_col("name", SqlVal::Text(&word.name.trim().to_lowercase()))?;
        if existing.is_none() {
            storage::insert_word(&word)?;
            continue;
        }

        let mut existing = existing.unwrap();
        existing.merge_meanings(&word.meanings);
        storage::update_word(&existing)?;
    }

    Ok(count)
}