use std::process::Command;

pub fn check_vpn_active() -> bool {
    let output = Command::new("scutil").args(&["--nc", "list"]).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("Connected")
        }
        Err(_) => false,
    }
}
