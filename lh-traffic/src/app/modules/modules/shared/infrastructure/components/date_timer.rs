use chrono::{Local, DateTime};
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Date and time utilities
pub struct DateTimer;

static INSTANCE: Lazy<Arc<DateTimer>> = Lazy::new(|| Arc::new(DateTimer));

impl DateTimer {
    pub fn instance() -> Arc<DateTimer> {
        INSTANCE.clone()
    }

    pub fn get_now_ymd_his(&self) -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn get_now(&self) -> DateTime<Local> {
        Local::now()
    }
}
