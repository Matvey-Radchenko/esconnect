use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use log::{error, info};
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

mod config;
mod daemon;
mod rpa;
mod security;
mod server;
mod ui;
mod utils;
mod vpn;

use config::{Config, ConfigManager};
use daemon::Daemon;
use security::KeychainManager;
use ui::Ui;

#[derive(Parser)]
#[command(name = "esconnect")]
#[command(about = "Automated Endpoint Security VPN connection tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Start {
        #[arg(long)]
        foreground: bool,
    },
    Stop,
    Status,
    Config,
    Delays,
    Token,
    Connect,
    Disconnect,
    Toggle,
}

#[derive(Debug, Clone)]
enum SetupAction {
    Full,
    UpdateAuthToken,
    UpdateServerIP,
    UpdateMacroDroidWebhook,
    UpdatePassword,
    Exit,
}

impl fmt::Display for SetupAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetupAction::Full => write!(f, "🆕 Full Setup"),
            SetupAction::UpdateAuthToken => write!(f, "🔑 Update Auth Token"),
            SetupAction::UpdateServerIP => write!(f, "🌐 Update Server IP"),
            SetupAction::UpdateMacroDroidWebhook => write!(f, "📱 Update MacroDroid Webhook"),
            SetupAction::UpdatePassword => write!(f, "🔐 Update VPN Password"),
            SetupAction::Exit => write!(f, "❌ Exit"),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Setup => setup_menu()?,
        Commands::Start { foreground } => start(*foreground)?,
        Commands::Stop => stop()?,
        Commands::Status => status()?,
        Commands::Config => show_config()?,
        Commands::Delays => configure_delays()?,
        Commands::Token => show_token()?,
        Commands::Connect => connect()?,
        Commands::Disconnect => disconnect()?,
        Commands::Toggle => toggle()?,
    }

    Ok(())
}

fn setup_menu() -> Result<()> {
    loop {
        Ui::print_header("ESConnect Setup");

        let options = vec![
            SetupAction::Full,
            SetupAction::UpdateAuthToken,
            SetupAction::UpdateServerIP,
            SetupAction::UpdateMacroDroidWebhook,
            SetupAction::UpdatePassword,
            SetupAction::Exit,
        ];

        let choice = Ui::ask_select("Select action:", options)?;

        match choice {
            SetupAction::Full => full_setup()?,
            SetupAction::UpdateAuthToken => update_auth_token()?,
            SetupAction::UpdateServerIP => update_server_ip()?,
            SetupAction::UpdateMacroDroidWebhook => update_macro_droid_webhook()?,
            SetupAction::UpdatePassword => update_password()?,
            SetupAction::Exit => break,
        }

        println!("\nPress Enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    Ok(())
}

fn full_setup() -> Result<()> {
    Ui::print_header("Full Setup");

    update_auth_token()?;
    update_server_ip()?;
    update_macro_droid_webhook()?;
    update_password()?;

    Ui::print_success("Full setup completed!");
    Ok(())
}

fn update_auth_token() -> Result<()> {
    let input = Ui::ask_password("Enter Auth Token (or press Enter to generate random):")?;
    let token = if input.is_empty() {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        let generated = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();
        println!();
        println!("  Generated token: {}", generated.green().bold());
        println!("  Скопируй его в MacroDroid → HTTP Request → X-Auth-Token");
        println!();
        generated
    } else {
        input
    };

    KeychainManager::set_password("auth_token", &token)?;
    Ui::print_success("Auth token saved!");

    Ok(())
}

fn update_server_ip() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    // Auto-detect local subnet
    let detected_subnet = auto_detect_subnet()?;
    Ui::print_info(&format!("Detected local subnet: {}", detected_subnet));

    let use_detected = Ui::ask_confirm(&format!("Use detected subnet {}?", detected_subnet))?;
    let subnet = if use_detected {
        detected_subnet
    } else {
        Ui::ask_text("Enter custom subnet (e.g., 192.168.0.0/24):")?
    };

    config.server.allowed_subnet = subnet;
    config_manager.save(&config)?;

    Ui::print_success("Server subnet updated!");
    Ok(())
}

fn get_local_ip() -> Result<String> {
    let output = std::process::Command::new("ipconfig")
        .arg("getifaddr")
        .arg("en0")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get local IP address"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn auto_detect_subnet() -> Result<String> {
    use std::net::{IpAddr, Ipv4Addr};
    use std::process::Command;

    // Get local IP using networksetup or ifconfig
    let output = Command::new("ipconfig")
        .arg("getifaddr")
        .arg("en0") // Primary ethernet interface on macOS
        .output()
        .or_else(|_| {
            // Fallback to ifconfig
            Command::new("ifconfig").arg("en0").arg("inet").output()
        })?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get local IP address"));
    }

    let ip_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let ip: Ipv4Addr = ip_str
        .parse()
        .map_err(|_| anyhow!("Invalid IP address: {}", ip_str))?;

    // Calculate subnet (assume /24 for common home networks)
    let octets = ip.octets();
    let subnet = format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]);

    Ok(subnet)
}

fn update_macro_droid_webhook() -> Result<()> {
    Ui::print_header("MacroDroid Webhook Setup");

    Ui::print_info("Создайте webhook в MacroDroid для отправки 2FA кодов.");
    Ui::print_info(
        "Webhook должен отправлять POST запрос на /token с кодом в JSON: {\"code\": \"123456\"}",
    );

    let url = Ui::ask_text("Введите URL вашего MacroDroid webhook:")?;
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    config.server.macrodroid_webhook_url = url;
    config_manager.save(&config)?;

    Ui::print_success("MacroDroid webhook URL сохранен!");
    Ok(())
}

fn update_password() -> Result<()> {
    let password = Ui::ask_password("Enter VPN Password:")?;
    KeychainManager::set_password("vpn_password", &password)?;
    Ui::print_success("Password saved!");
    Ok(())
}

fn connect() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load()?;

        if config.server.macrodroid_webhook_url.is_empty() {
            Ui::print_error("MacroDroid webhook URL not configured. Run 'esconnect setup' first.");
            return Ok(());
        }

        handle_connect(&config).await
    })
}

fn disconnect() -> Result<()> {
    vpn::VpnAutomator::new().disconnect()
}

fn toggle() -> Result<()> {
    let config = ConfigManager::new()?.load()?;
    if rpa::EndpointDriver::is_connected(&config.vpn.detection_prefix) {
        disconnect()
    } else {
        connect()
    }
}

async fn handle_connect(config: &Config) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let spinner = Ui::spinner("Запрос кода у телефона...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let ip = get_local_ip().unwrap_or_default();
    let url = format!("{}?ip={}", config.server.macrodroid_webhook_url, ip);
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            spinner.finish_with_message("Сигнал отправлен! Ожидание входящего кода на сервере...");
            Ui::print_success("Webhook вызван");
        }
        Ok(resp) => {
            spinner.finish_and_clear();
            Ui::print_error(&format!("Ошибка webhook: HTTP {}", resp.status()));
        }
        Err(e) => {
            spinner.finish_and_clear();
            Ui::print_error(&format!("Ошибка сети: {}", e));
        }
    }

    Ok(())
}

fn start(foreground: bool) -> Result<()> {
    if foreground {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let daemon = Daemon::new();
            daemon.run().await
        })?;
    } else {
        Ui::print_info("Starting daemon in background...");

        let pid_file = "/tmp/esconnect.pid";
        if Path::new(pid_file).exists() {
            Ui::print_warning("PID file exists. Daemon might be already running.");
        }

        let log_file = File::create("/tmp/esconnect.log")?;

        let child = process::Command::new(std::env::current_exe()?)
            .arg("start")
            .arg("--foreground")
            .env("RUST_LOG", "debug")
            .stdout(log_file.try_clone()?)
            .stderr(log_file)
            .spawn()?;

        let mut f = File::create(pid_file)?;
        f.write_all(child.id().to_string().as_bytes())?;

        Ui::print_success(&format!("Daemon started with PID: {}", child.id()));
    }
    Ok(())
}

fn stop() -> Result<()> {
    let pid_file = "/tmp/esconnect.pid";
    if !Path::new(pid_file).exists() {
        Ui::print_warning("Daemon is not running (PID file not found).");
        return Ok(());
    }

    let pid = std::fs::read_to_string(pid_file)?.trim().parse::<i32>()?;

    Ui::print_info(&format!("Stopping daemon (PID: {})...", pid));

    match process::Command::new("kill").arg(pid.to_string()).output() {
        Ok(_) => {
            std::fs::remove_file(pid_file)?;
            Ui::print_success("Daemon stopped.");
        }
        Err(e) => Ui::print_error(&format!("Failed to stop daemon: {}", e)),
    }

    Ok(())
}

fn status() -> Result<()> {
    Ui::print_header("System Status");

    let pid_file = "/tmp/esconnect.pid";
    if Path::new(pid_file).exists() {
        let pid = std::fs::read_to_string(pid_file)?.trim().to_string();
        // Check if process is running
        let output = process::Command::new("ps").arg("-p").arg(&pid).output()?;
        if output.status.success() && String::from_utf8_lossy(&output.stdout).contains(&pid) {
            Ui::print_success(&format!("Daemon is running (PID: {})", pid));
        } else {
            Ui::print_error(&format!("Daemon is NOT running (PID file exists: {})", pid));
        }
    } else {
        Ui::print_warning("Daemon is not running");
    }

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    match rpa::EndpointDriver::vpn_ip(&config.vpn.detection_prefix) {
        Some(ip) => Ui::print_success(&format!("VPN подключён ({})", ip)),
        None => Ui::print_warning("VPN отключён"),
    }

    println!("\nConfiguration:");
    println!(
        "  Server Subnet: {}",
        if config.server.allowed_subnet.is_empty() {
            "❌ Not set".red()
        } else {
            config.server.allowed_subnet.green()
        }
    );
    println!("  Port: {}", config.server.port.to_string().green());

    Ok(())
}

fn show_config() -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load()?;
    println!("{:#?}", config);
    Ok(())
}

fn show_token() -> Result<()> {
    Ui::print_header("Auth Token");

    match KeychainManager::get_password("auth_token") {
        Ok(token) => {
            Ui::print_success(&format!("Token: {}", token));
            Ui::print_info("Use this token in X-Auth-Token header for API requests");
        }
        Err(_) => {
            Ui::print_error("Auth token not found. Run 'esconnect setup' first.");
        }
    }

    Ok(())
}

fn configure_delays() -> Result<()> {
    Ui::print_header("Configure Delays");

    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    println!("Current delays:");
    println!(
        "1. Menu Open: {}s (ожидание открытия контекстного меню)",
        config.delays.menu_open
    );
    println!(
        "2. Dialog Wait: {}s (ожидание окна endpoint security)",
        config.delays.dialog_wait
    );
    println!(
        "3. Connection Wait: {}s (ожидание окна пароля)",
        config.delays.connection_wait
    );
    println!("   Input Delay: 0.3s (зашито, не регулируется)");

    if Ui::ask_confirm("Do you want to modify delays?")? {
        loop {
            let options = vec!["Menu Open", "Dialog Wait", "Connection Wait", "Finish"];

            let choice = Ui::ask_select("Select delay to modify (or 'Finish' to exit):", options)?;

            match choice {
                "Menu Open" => {
                    let val = Ui::ask_text("Enter new value (seconds):")?.parse::<f64>()?;
                    config.delays.menu_open = val;
                    Ui::print_success(&format!("Menu Open delay set to {}s", val));
                }
                "Dialog Wait" => {
                    let val = Ui::ask_text("Enter new value (seconds):")?.parse::<f64>()?;
                    config.delays.dialog_wait = val;
                    Ui::print_success(&format!("Dialog Wait delay set to {}s", val));
                }
                "Connection Wait" => {
                    let val = Ui::ask_text("Enter new value (seconds):")?.parse::<f64>()?;
                    config.delays.connection_wait = val;
                    Ui::print_success(&format!("Connection Wait delay set to {}s", val));
                }
                "Finish" => break,
                _ => {}
            }
        }

        config_manager.save(&config)?;
        Ui::print_success("Delays updated!");
    }

    Ok(())
}
