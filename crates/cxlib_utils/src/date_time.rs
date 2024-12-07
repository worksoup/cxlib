use std::time::{Duration, SystemTime};

pub fn print_now() {
    let str = now_string();
    println!("{str}");
}

pub fn now_string() -> String {
    time_string(SystemTime::now())
}

pub fn time_string(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(t)
        .format("%+")
        .to_string()
}

pub fn time_string_from_mills(mills: u64) -> String {
    time_string(std::time::UNIX_EPOCH + Duration::from_millis(mills))
}

pub fn time_delta_since_to_now(mills: u64) -> chrono::TimeDelta {
    let start_time = std::time::UNIX_EPOCH + Duration::from_millis(mills);
    let now = SystemTime::now();
    let duration = now.duration_since(start_time).unwrap();
    chrono::TimeDelta::from_std(duration).unwrap()
}
