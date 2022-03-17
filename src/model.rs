use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, PartialEq)]
pub struct Word {
    pub id: i64,
    pub name: String,
    pub meanings: String,
    pub period_days: u16,
    pub last_visit: DateTime<Utc>,
    pub next_visit: DateTime<Utc>,
}

impl Word {
    pub fn from_sqlite_pairs(pairs: &[(&str, Option<&str>)]) -> Result<Word> {
        let hash_map = to_hashmap(pairs);
        Ok(Word {
            id: get_val(&hash_map, "id")?.unwrap(),
            name: get_val(&hash_map, "name")?.unwrap(),
            meanings: get_val(&hash_map, "meanings")?.unwrap(),
            period_days: get_val(&hash_map, "period_days")?.unwrap(),
            last_visit: DateTime::from_utc(NaiveDateTime::from_timestamp(get_val(&hash_map, "last_visit")?.unwrap(), 0), Utc),
            next_visit: DateTime::from_utc(NaiveDateTime::from_timestamp(get_val(&hash_map, "next_visit")?.unwrap(), 0), Utc),
        })
    }
}

fn to_hashmap<'a>(pairs: &'a [(&str, Option<&str>)]) -> HashMap<&'a str, Option<&'a str>> {
    let mut hash_map = HashMap::new();
    for &(name, val) in pairs.iter() {
        hash_map.insert(name, val);
    }

    hash_map
}

fn get_val<T>(h: &HashMap<&str, Option<&str>>, key: &str) -> std::result::Result<Option<T>, <T as FromStr>::Err>
    where T: FromStr {
    let val = h.get(key);
    if val.is_none() {
        return Ok(None);
    }

    let val = val.unwrap();
    if val.is_none() {
        return Ok(None);
    }

    let val = val.unwrap();
    let typed = T::from_str(val)?;
    Ok(Some(typed))
}
