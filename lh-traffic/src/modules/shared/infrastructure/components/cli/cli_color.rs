use colored::*;

/// CLI color utilities
/// Equivalent to TypeScript's CliColor
pub struct CliColor;

impl CliColor {
    pub fn echo_green(message: &str) {
        println!("{}", message.green());
    }

    pub fn echo_red(message: &str) {
        println!("{}", message.red());
    }

    pub fn echo_orange(message: &str) {
        println!("{}", message.yellow());
    }

    pub fn echo_blue(message: &str) {
        println!("{}", message.blue());
    }

    pub fn echo_cyan(message: &str) {
        println!("{}", message.cyan());
    }

    pub fn echo_yellow(message: &str) {
        println!("{}", message.yellow());
    }

    pub fn die_red(message: &str) -> ! {
        eprintln!("{}", message.red());
        std::process::exit(1);
    }

    pub fn get_color_green(message: &str) -> String {
        message.green().to_string()
    }

    pub fn get_color_red(message: &str) -> String {
        message.red().to_string()
    }

    pub fn get_color_orange(message: &str) -> String {
        message.yellow().to_string()
    }

    pub fn get_color_blue(message: &str) -> String {
        message.blue().to_string()
    }
}
