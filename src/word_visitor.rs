use std::io::{self, BufRead};

use anyhow::Result;
use chrono::{Duration, Utc};

use crate::{infra::Db, revisit_planner};

fn test_one_word(db: &Db) -> Result<bool> {
    let word = db.get_one_word_to_test()?;
    if word.is_none() {
        println!("No word planned to test at now.");
        return Ok(false);
    }

    let mut is_answer_correct = true;

    let mut word = word.unwrap();
    let stdin = io::stdin();

    println!("What are the meaning of [{}]:", word.name);
    let mut lines = stdin.lock().lines();

    let meanings = lines.next();
    if meanings.is_none() {
        println!("Test aborted.");
        return Ok(false);
    }

    let meanings = meanings.unwrap()?;
    if meanings.is_empty() {
        println!("Test aborted.");
        return Ok(false);
    }

    if word.has_meanings(&meanings) {
        println!("CORRECT!");
    } else {
        is_answer_correct = false;
        println!("Answer is: [{}]", word.meanings);
    }

    println!(
        "To memorize the spelling, enter the word with meaning [{}]:",
        word.meanings
    );
    loop {
        let name = lines.next();
        if name.is_none() {
            println!("Test aborted.");
            return Ok(false);
        }

        let name = name.unwrap()?;
        if name.is_empty() {
            println!("Test aborted.");
            return Ok(false);
        }

        if name.trim().to_lowercase() == word.name.to_lowercase() {
            println!("CORRECT!");
        } else {
            is_answer_correct = false;
            println!("WRONG! Please enter [{}] again:", word.name);
            continue;
        }

        break;
    }
    println!();

    let now = Utc::now();
    word.last_visit = now;
    word.period_days = if is_answer_correct {
        revisit_planner::get_next_period_days(word.period_days)
    } else {
        revisit_planner::get_last_period_days(word.period_days)
    };
    word.next_visit = now + Duration::days(word.period_days as i64);
    db.update_word(&word)?;

    Ok(true)
}

pub fn do_tests(db: &Db) -> Result<usize> {
    let mut count = 0usize;
    println!("Note: Enter empty line to abort test.");
    while test_one_word(db)? {
        count += 1;
    }
    Ok(count)
}
