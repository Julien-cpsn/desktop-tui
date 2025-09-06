use chrono::Local;

pub fn time_to_string() -> String {
    let now = Local::now();
    now.format("%H:%M").to_string()
}