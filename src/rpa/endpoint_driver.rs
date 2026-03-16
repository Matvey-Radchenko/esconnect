use anyhow::{anyhow, Result};
use std::process::Command;
use std::thread;
use std::time::Duration;

const PROCESS_NAME: &str = "EndpointConnect"; // System Events name (osascript)
const PROCESS_BINARY: &str = "Endpoint_Security_VPN"; // actual binary name (pgrep)

pub struct EndpointDriver;

impl EndpointDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn find_pid() -> Result<i32> {
        let out = Command::new("pgrep")
            .arg("-x")
            .arg(PROCESS_BINARY)
            .output()?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .and_then(|l| l.trim().parse().ok())
            .ok_or_else(|| anyhow!("Process '{}' not running", PROCESS_BINARY))
    }

    fn run_osascript(script: &str) -> Result<()> {
        let out = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| anyhow!("osascript: {}", e))?;
        if !out.status.success() {
            return Err(anyhow!(
                "osascript: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(())
    }

    /// Poll via osascript until EndpointConnect has at least `min_windows` open windows.
    /// Uses osascript (not AX API) so no special binary permissions needed.
    fn wait_for_windows(min_windows: usize, timeout: Duration) -> Result<()> {
        let script = r#"tell application "System Events"
    tell process "EndpointConnect"
        return count of windows
    end tell
end tell"#;
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let out = Command::new("osascript").arg("-e").arg(script).output()?;
            let count: usize = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            if count >= min_windows {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(anyhow!(
                    "Timeout waiting for {} window(s), last count={}",
                    min_windows,
                    count
                ));
            }
            thread::sleep(Duration::from_millis(300));
        }
    }

    /// Type text using CGEventKeyboardSetUnicodeString + kCGHIDEventTap.
    /// Layout-independent, works in password fields.
    pub fn type_text(text: &str) -> Result<()> {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGEventCreateKeyboardEvent(
                source: *const std::ffi::c_void,
                virtual_key: u16,
                key_down: bool,
            ) -> *mut std::ffi::c_void;
            fn CGEventKeyboardSetUnicodeString(
                event: *mut std::ffi::c_void,
                string_length: usize,
                unicode_string: *const u16,
            );
            fn CGEventPost(tap: u32, event: *mut std::ffi::c_void);
            fn CFRelease(cf: *mut std::ffi::c_void);
        }
        const KCG_HID_EVENT_TAP: u32 = 0;

        for ch in text.encode_utf16() {
            let c = [ch];
            unsafe {
                let down = CGEventCreateKeyboardEvent(std::ptr::null(), 0, true);
                if !down.is_null() {
                    CGEventKeyboardSetUnicodeString(down, 1, c.as_ptr());
                    CGEventPost(KCG_HID_EVENT_TAP, down);
                    CFRelease(down);
                }
                let up = CGEventCreateKeyboardEvent(std::ptr::null(), 0, false);
                if !up.is_null() {
                    CGEventKeyboardSetUnicodeString(up, 1, c.as_ptr());
                    CGEventPost(KCG_HID_EVENT_TAP, up);
                    CFRelease(up);
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }

    /// Press Enter (key code 36).
    pub fn press_enter() -> Result<()> {
        Self::run_osascript(
            r#"tell application "System Events"
    key code 36
end tell"#,
        )
    }

    /// Returns the VPN tunnel IP if a utun interface has an IP matching `prefix`.
    /// `prefix` comes from config (e.g. "10." or "10.0.") to distinguish
    /// corporate VPN from other VPNs that may also use 10.x addresses.
    pub fn vpn_ip(prefix: &str) -> Option<String> {
        let out = Command::new("ifconfig").output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut in_utun = false;
        for line in text.lines() {
            if line.starts_with("utun") {
                in_utun = true;
            } else if !line.starts_with(|c: char| c.is_whitespace()) {
                in_utun = false;
            }
            if in_utun {
                if let Some(rest) = line.trim().strip_prefix("inet ") {
                    let ip = rest.split_whitespace().next().unwrap_or("");
                    if ip.starts_with(prefix) {
                        return Some(ip.to_string());
                    }
                }
            }
        }
        None
    }

    /// Returns true if VPN is currently connected.
    pub fn is_connected(prefix: &str) -> bool {
        Self::vpn_ip(prefix).is_some()
    }

    /// Open tray menu and select the first item via Down + Enter.
    /// When disconnected: first item is "Подключиться".
    /// When connected:    first item is "Отключить".
    fn open_tray_and_select_first(&self) -> Result<()> {
        Self::run_osascript(
            r#"tell application "System Events"
    tell process "EndpointConnect"
        click menu bar item 1 of menu bar 2
        delay 0.2
        key code 125
        delay 0.05
        key code 36
    end tell
end tell"#,
        )
    }

    /// Poll via osascript until button with given name appears in any UI element.
    /// Works for QNSPanel dialogs which are not counted as windows.
    fn wait_for_button(name: &str, timeout: Duration) -> Result<()> {
        let script = format!(
            r#"tell application "System Events"
    tell process "EndpointConnect"
        return exists button "{}" of group 1 of window 1
    end tell
end tell"#,
            name
        );
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let out = Command::new("osascript").arg("-e").arg(&script).output()?;
            let found = String::from_utf8_lossy(&out.stdout).trim() == "true";
            if found {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(anyhow!("Timeout waiting for button '{}'", name));
            }
            thread::sleep(Duration::from_millis(300));
        }
    }

    /// Click a named button inside the confirmation panel.
    fn click_button(name: &str) -> Result<()> {
        let script = format!(
            r#"tell application "System Events"
    tell process "EndpointConnect"
        click button "{}" of group 1 of UI element 1
    end tell
end tell"#,
            name
        );
        Self::run_osascript(&script)
    }

    /// Disconnect from VPN: open tray → "Отключить" → confirm "Да".
    pub fn disconnect(&self) -> Result<()> {
        self.open_tray_and_select_first()?;
        Self::wait_for_button("Да", Duration::from_secs(5))?;
        Self::click_button("Да")?;
        log::info!("Disconnect sequence completed");
        Ok(())
    }

    /// Connect or disconnect depending on current VPN state.
    pub fn toggle(&self, code: &str, password: &str, prefix: &str) -> Result<()> {
        if Self::is_connected(prefix) {
            self.disconnect()
        } else {
            self.connect(code, password)
        }
    }

    /// Full connect sequence: tray → connect button → OTP dialog → password dialog → submit.
    pub fn connect(&self, code: &str, password: &str) -> Result<()> {
        let pid = Self::find_pid()?;
        log::info!("Connecting via pid={}", pid);

        // 1. Open tray menu and select "Подключиться"
        self.open_tray_and_select_first()?;

        // 2. Wait for OTP dialog, type code
        Self::wait_for_windows(1, Duration::from_secs(10))?;
        Self::type_text(code)?;
        thread::sleep(Duration::from_millis(300));
        Self::press_enter()?;

        // 3. Wait for first dialog to close, then wait for password dialog
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if std::time::Instant::now() >= deadline {
                break;
            }
            let out = Command::new("osascript")
                .arg("-e")
                .arg(
                    r#"tell application "System Events"
    tell process "EndpointConnect"
        return count of windows
    end tell
end tell"#,
                )
                .output()?;
            let count: usize = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse()
                .unwrap_or(1);
            if count == 0 {
                break;
            }
            thread::sleep(Duration::from_millis(300));
        }
        Self::wait_for_windows(1, Duration::from_secs(15))?;

        // 4. Type password and submit
        Self::type_text(password)?;
        thread::sleep(Duration::from_millis(300));
        Self::press_enter()?;

        log::info!("Connect sequence completed");
        Ok(())
    }
}
