pub mod commands;
pub mod interfaces;
pub mod abstract_command;

use crate::app::modules::shared::infrastructure::components::cli::CliArgs;
use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use chrono::Local;

pub async fn run_console(args: Vec<String>) {
    let cli_args = CliArgs::instance();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");

    CliColor::echo_green(&format!("[{}] command: running console commands", now));

    if let Some(command_name) = cli_args.get_arg(0) {
        // Map command names to their handlers
        match command_name.as_str() {
            // Add your commands here
            // "lz:deploy" => commands::devops::deploy_command::DeployCommand::invoke().await,
            _ => {
                CliColor::die_red(&format!(
                    "[{}] command: \"{}\" not found.",
                    now, command_name
                ));
            }
        }
    } else {
        CliColor::die_red(&format!("[{}] command: no command specified.", now));
    }
}
