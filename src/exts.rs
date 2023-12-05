use std::{process, time::Duration};

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

pub fn close_proc() {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(res) => res,
        Err(err) => {
            crate::logger::error_string(format!("Failed to create tokio runtime in close_proc(): {err}"));
            return;
        }
    };

    rt.block_on(async {
        crate::logger::warn("Window will be closed after 5 seconds");
        tokio::time::sleep(Duration::from_secs(5)).await;
    });

    process::exit(0x0100);
}

pub fn read_line() -> String {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            line = line.replace('\n', "").trim().to_lowercase();
        }
        Err(err) => {
            crate::logger::error_string(format!("Couldn't read the line: {err}"));
        }
    };

    return line;
}