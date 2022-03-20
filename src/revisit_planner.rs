use chrono::{DateTime, Utc, Duration};

pub fn get_init_period_days() -> u16 {
    1
}

pub fn get_next_visit_time(last_visit: DateTime<Utc>, period_days: u16) -> DateTime<Utc> {
    last_visit + Duration::days(period_days as i64)
}