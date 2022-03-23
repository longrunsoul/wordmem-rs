use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDateTime};

pub type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub struct Word {
    pub id: Option<i64>,
    pub name: String,
    pub meanings: String,
    pub period_days: u16,
    pub last_visit: DateTime<Utc>,
    pub next_visit: DateTime<Utc>,
}

impl Word {
    fn make_meaning_cmp_map(meanings: &str) -> HashMap<String, String> {
        let mut cmp_map = HashMap::new();
        for m in meanings.split(';') {
            let norm = m.trim().to_lowercase();
            if norm.is_empty() {
                continue;
            }

            if cmp_map.contains_key(&norm) {
                continue;
            }

            cmp_map.insert(norm, m.trim().to_string());
        }

        cmp_map
    }

    pub fn from_sqlite_pairs(pairs: &[(&str, Option<&str>)]) -> Result<Word> {
        let hash_map = to_hashmap(pairs);
        Ok(Word {
            id: Some(get_val(&hash_map, "id")?.unwrap()),
            name: get_val(&hash_map, "name")?.unwrap(),
            meanings: get_val(&hash_map, "meanings")?.unwrap(),
            period_days: get_val(&hash_map, "period_days")?.unwrap(),
            last_visit: DateTime::from_utc(NaiveDateTime::from_timestamp(get_val(&hash_map, "last_visit")?.unwrap(), 0), Utc),
            next_visit: DateTime::from_utc(NaiveDateTime::from_timestamp(get_val(&hash_map, "next_visit")?.unwrap(), 0), Utc),
        })
    }

    pub fn norm_meanings(meanings: &str) -> String {
        Word::make_meaning_cmp_map(meanings).into_values().collect::<Vec<_>>().join(";")
    }

    pub fn merge_meanings(&mut self, meanings: &str) {
        let mut merged = Word::make_meaning_cmp_map(&self.meanings);
        for (k, m) in Word::make_meaning_cmp_map(meanings) {
            if merged.contains_key(&k) {
                continue;
            }

            merged.insert(k, m);
        }

        let joined = merged.into_values().collect::<Vec<_>>().join(";");
        self.meanings = joined;
    }

    pub fn has_meanings(&self, meanings: &str) -> bool {
        let mset: HashSet<String> = Word::make_meaning_cmp_map(meanings).into_keys().collect();
        let self_mset : HashSet<String> = Word::make_meaning_cmp_map(&self.meanings).into_keys().collect();
        if mset.is_empty() {
            return false;
        }

        self_mset == mset
    }
}

fn to_hashmap<'a>(pairs: &'a [(&str, Option<&str>)]) -> HashMap<&'a str, Option<&'a str>> {
    let mut hash_map = HashMap::new();
    for &(name, val) in pairs.iter() {
        hash_map.insert(name, val);
    }

    hash_map
}

fn get_val<T>(h: &HashMap<&str, Option<&str>>, key: &str) -> StdResult<Option<T>, <T as FromStr>::Err>
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
