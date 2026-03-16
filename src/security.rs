use anyhow::{anyhow, Result};
use keyring::Entry;
use std::env;

const SERVICE_NAME: &str = "esconnect";

pub struct KeychainManager;

impl KeychainManager {
    pub fn set_password(key: &str, password: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        entry.set_password(password)?;
        Ok(())
    }

    pub fn get_password(key: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn delete_password(key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        entry.delete_password()?;
        Ok(())
    }
}
