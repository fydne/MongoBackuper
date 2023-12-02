use time::OffsetDateTime;

pub fn get_date() -> String {
    let date = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
    return format!("{:0width$}.{:0width$}.{:0width$} {:0width$}:{:0width$}:{:0width$}",
        date.day(), date.month() as usize, date.year(), date.hour(), date.minute(), date.second(),
        width = 2);
}

pub fn get_date_file() -> String {
    let date = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
    return format!("{:0width$}.{:0width$}.{:0width$} {:0width$}-{:0width$}",
        date.day(), date.month() as usize, date.year(), date.hour(), date.minute(),
        width = 2);
}