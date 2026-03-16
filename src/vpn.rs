use crate::rpa::EndpointDriver;
use anyhow::Result;

pub struct VpnAutomator {
    driver: EndpointDriver,
}

impl VpnAutomator {
    pub fn new() -> Self {
        Self {
            driver: EndpointDriver::new(),
        }
    }

    pub fn connect(&self, code: &str, password: &str) -> Result<()> {
        self.driver.connect(code, password)
    }

    pub fn disconnect(&self) -> Result<()> {
        self.driver.disconnect()
    }
}
