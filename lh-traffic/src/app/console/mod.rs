pub mod commands;
pub mod interfaces;
pub mod abstract_command;

use crate::app::modules::shared::infrastructure::components::cli::CliArgs;
use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use chrono::Local;
use chrono::format::DelayedFormat;
use std::sync::Arc;

pub async fn run_console(args: Vec<String>) {
    let cli_args: Arc<CliArgs> = CliArgs::instance();
    let now: DelayedFormat<chrono::format::StrftimeItems<'_>> = Local::now().format("%Y-%m-%d %H:%M:%S");

    CliColor::echo_green(&format!("[{}] command: running console commands", now));
    if cli_args.get_arg(0).is_none() {
        CliColor::die_red(&format!("[{}] command: no command specified.", now));
        return;
    }

    let command_name: &String = cli_args.get_arg(0).unwrap();
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
}
