use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use crate::app::modules::shared::infrastructure::components::logger::Logger;
use chrono::Local;
use std::sync::Arc;

/// Abstract command trait
/// Equivalent to TypeScript's AbstractCommand
pub struct AbstractCommand {
    pub logger: Arc<Logger>,
    pub dt_start: String,
    pub dt_end: String,
}

impl AbstractCommand {
    pub fn new() -> Self {
        Self {
            logger: Logger::instance(),
            dt_start: String::new(),
            dt_end: String::new(),
        }
    }

    pub fn echo_start(&mut self, message: &str) {
        self.dt_start = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        CliColor::echo_orange(&format!("[{}] start: {}", self.dt_start, message));
    }

    pub fn echo_end(&self, message: &str) {
        let dt_end = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        CliColor::echo_orange(&format!(
            "[{}] [{}] end: {}",
            self.dt_start, dt_end, message
        ));
    }

    pub fn echo_step(&self, message: &str) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        CliColor::echo_green(&format!("[{}]: {}", now, message));
    }

    pub async fn sleep_seconds(&self, secs: u64) {
        tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
    }

    pub fn get_instance() -> Self {
        Self::new()
    }
}
