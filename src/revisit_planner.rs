
pub fn get_last_period_days(current_period_days: u16) -> u16 {
    if current_period_days == 1 {
        return 1;
    }

    current_period_days / 2
}

pub fn get_next_period_days(current_period_days: u16) -> u16 {
    if current_period_days == 128 {
        return 128;
    }

    current_period_days * 2
}