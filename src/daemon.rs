use anyhow::Result;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::ConfigManager;
use crate::security::KeychainManager;
use crate::server::start_server;
use crate::vpn::VpnAutomator;

pub struct Daemon {
    running: Arc<AtomicBool>,
}

impl Daemon {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting daemon...");

        let config_manager = ConfigManager::new()?;
        let config = config_manager.load()?;

        let keychain = KeychainManager;
        let vpn_automator = VpnAutomator::new();

        let running = self.running.clone();
        ctrlc::set_handler(move || {
            info!("Received shutdown signal");
            running.store(false, Ordering::SeqCst);
        })?;

        // Start the HTTP server
        info!("Starting HTTP server...");
        start_server(config, keychain, vpn_automator).await?;

        info!("Daemon stopped");
        Ok(())
    }
}
