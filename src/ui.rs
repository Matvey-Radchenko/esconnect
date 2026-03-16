use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Password, Select, Text};
use std::time::Duration;

pub struct Ui;

impl Ui {
    pub fn print_header(text: &str) {
        // Clear screen for better UX
        print!("\x1B[2J\x1B[1;1H");
        println!("{}", text.bold().purple());
        println!("{}", "=".repeat(text.len()).purple());
        println!();
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }

    pub fn print_success(text: &str) {
        println!("{} {}", "✅".green(), text);
    }

    pub fn print_error(text: &str) {
        println!("{} {}", "❌".red(), text);
    }

    pub fn print_warning(text: &str) {
        println!("{} {}", "⚠️ ".yellow(), text);
    }

    pub fn print_info(text: &str) {
        println!("{} {}", "ℹ️ ".blue(), text);
    }

    pub fn ask_text(prompt: &str) -> Result<String> {
        Ok(Text::new(prompt).prompt()?)
    }

    pub fn ask_password(prompt: &str) -> Result<String> {
        Ok(Password::new(prompt)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()?)
    }

    pub fn ask_confirm(prompt: &str) -> Result<bool> {
        Ok(Confirm::new(prompt).with_default(true).prompt()?)
    }

    pub fn ask_select<T: std::fmt::Display>(prompt: &str, options: Vec<T>) -> Result<T> {
        Ok(Select::new(prompt, options).prompt()?)
    }

    pub fn spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }
}
