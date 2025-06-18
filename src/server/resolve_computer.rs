use serde::{ Serialize, Deserialize };
use crate::configs::get_ssh_password;
use crate::server::network::ssh::ssh_exec;
use log::{ info, warn };
use dioxus::prelude::ServerFnError;
use futures;
use serde_json;

const DEFAULT_SSH_USER: &str = "ph-admin"; // Default macOS administrator username

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputerInfo {
    pub title: String,
    pub product_name: String,
    pub serial: String,
    pub version: String,
    pub user: String,
    pub network_name: String,
    pub processor: String,
    pub architecture: String,
    pub memory: String,
    pub graphics: String,
    pub storage: String,
    pub lan_ip: String,
    pub wifi_ip: String,
    pub wifi_name: String,
    pub home_users: Vec<String>,
}

impl Default for ComputerInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            product_name: String::new(),
            serial: String::new(),
            version: String::new(),
            user: String::new(),
            network_name: String::new(),
            processor: String::new(),
            architecture: String::new(),
            memory: String::new(),
            graphics: String::new(),
            storage: String::new(),
            lan_ip: String::new(),
            wifi_ip: String::new(),
            wifi_name: String::new(),
            home_users: Vec::new(),
        }
    }
}

impl ComputerInfo {
    // Public interface
    pub async fn resolve(host: String) -> Result<Self, ServerFnError> {
        info!("Starting computer information resolution for host: {}", host);

        // Verify host connectivity
        let password = get_ssh_password()?;
        let username = DEFAULT_SSH_USER.to_string();

        let test_cmd = "echo CONN_TEST_OK".to_string();
        match ssh_exec(host.clone(), username.clone(), password.clone(), test_cmd).await {
            Ok(output) if output.contains("CONN_TEST_OK") => {
                info!("Host {} is online and SSH connection successful", &host);
            }
            Ok(_) => {
                warn!("Host {} SSH test returned unexpected response", host);
                return Err(ServerFnError::new("SSH test returned unexpected response"));
            }
            Err(e) => {
                warn!("Host {} SSH connection failed: {}", host, e);
                return Err(ServerFnError::new(format!("SSH connection failed: {}", e)));
            }
        }



        let info = ComputerInfo::default();
        let info = std::sync::Arc::new(std::sync::Mutex::new(info));

        let futures = vec![
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_hardware_command()),
                Self::parse_hardware_info,
                info.clone()
            ),
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_system_command()),
                Self::parse_system_info,
                info.clone()
            ),
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_network_command()),
                Self::parse_network_info,
                info.clone()
            ),
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_storage_command()),
                Self::parse_storage_info,
                info.clone()
            ),
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_users_command()),
                Self::parse_users_info,
                info.clone()
            ),
            Self::spawn_command(
                &host,
                Box::new(|| Self::get_graphics_command()),
                Self::parse_graphics_info,
                info.clone()
            )
        ];

        let results = futures::future::join_all(futures).await;

        // Check for any errors
        if let Some(err) = results.into_iter().find_map(|r| r.err()) {
            return Err(err);
        }

        // Get the final info from the Arc<Mutex>
        let mut final_info = info.lock().unwrap().clone();
        final_info.network_name = host;

        info!("Computer information resolution completed");

        Ok(final_info)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    // Command definitions
    fn get_hardware_command() -> &'static str {
        "system_profiler SPHardwareDataType"
    }

    fn get_system_command() -> &'static str {
        "system_profiler SPSoftwareDataType"
    }

    fn get_network_command() -> &'static str {
        r#"echo '=== Network Interfaces ===' &&
           ifconfig | grep 'inet ' &&
           echo '=== Wifi Status ===' &&
           /System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -I"#
    }

    fn get_storage_command() -> &'static str {
        "df -h /"
    }

    fn get_users_command() -> &'static str {
        "dscl . list /Users | grep -v '^_' | grep -v 'daemon' | grep -v 'nobody'"
    }

    fn get_graphics_command() -> &'static str {
        "system_profiler SPDisplaysDataType"
    }

    // Core internal functionality
    async fn spawn_command(
        host: &str,
        cmd_fn: Box<dyn (Fn() -> &'static str) + Send + 'static>,
        parser: fn(&mut ComputerInfo, &str),
        info: std::sync::Arc<std::sync::Mutex<ComputerInfo>>
    ) -> Result<(), ServerFnError> {
        let command: String = cmd_fn().to_string();
        let password = get_ssh_password()?;
        let username = DEFAULT_SSH_USER.to_string();

        match ssh_exec(host.to_string(), username, password, command.clone()).await {
            Ok(output) => {
                if let Ok(mut info) = info.lock() {
                    parser(&mut info, &output);
                }
                Ok(())
            }
            Err(e) => {
                warn!("SSH command '{}' failed for host {}: {}", command, host, e);
                Err(ServerFnError::new(e))
            }
        }
    }

    // Parsing helpers
    fn get_value_after_first_colon(line: &str) -> Option<String> {
        line.splitn(2, ':')
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    // Data parsing methods
    fn parse_hardware_info(&mut self, output: &str) {
        for line in output.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("Model Name:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.product_name = val;
                    info!("Found product name: {}", self.product_name);
                }
            } else if trimmed_line.starts_with("Serial Number") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.serial = val;
                    info!("Found serial number: {}", self.serial);
                }
            } else if trimmed_line.starts_with("Processor ") {
                // Catches "Processor Name:" or "Processor Speed:"
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.processor = val;
                    info!("Found processor: {}", self.processor);
                }
            } else if trimmed_line.starts_with("Memory:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.memory = val;
                    info!("Found memory: {}", self.memory);
                }
            } else if trimmed_line.starts_with("Chip:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.architecture = match val.to_lowercase() {
                        s if s.contains("arm") => "aarch64".to_string(),
                        s if s.contains("intel") => "x86_64".to_string(),
                        _ => val,
                    };
                    info!("Found architecture: {}", self.architecture);
                }
            }
        }
    }

    fn parse_system_info(&mut self, output: &str) {
        for line in output.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("System Version:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.version = val;
                    info!("Found system version: {}", self.version);
                }
            } else if trimmed_line.starts_with("Computer Name:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.title = val;
                    info!("Found computer name: {}", self.title);
                }
            } else if trimmed_line.starts_with("User Name:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.user = val;
                    info!("Found user name: {}", self.user);
                }
            }
        }
    }

    fn parse_network_info(&mut self, output: &str) {
        let (ifconfig_output, airport_output) = output
            .split_once("=== Wifi Status ===")
            .unwrap_or((output, ""));

        for line in ifconfig_output.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("inet ") {
                let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] != "127.0.0.1" {
                    // Avoid loopback
                    if self.lan_ip.is_empty() {
                        // Prioritize as LAN IP
                        self.lan_ip = parts[1].to_string();
                        info!("Found LAN IP: {}", self.lan_ip);
                    } else if
                        // Original logic for wifi_ip from `inet` lines is preserved here,
                        // though `airport -I` output (wifi_section) doesn't typically contain `inet`.
                        // This might be for a different command structure or if ifconfig lists wifi IP after LAN.
                        // If a dedicated Wi-Fi IP is needed and distinct from LAN, command might need adjustment.
                        self.wifi_ip.is_empty()
                    {
                        self.wifi_ip = parts[1].to_string();
                        info!("Found potential WiFi IP from ifconfig: {}", self.wifi_ip);
                    }
                }
            }
        }

        if !airport_output.is_empty() {
            for line in airport_output.lines() {
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("SSID: ") {
                    // More specific than contains
                    if
                        let Some(val) = trimmed_line
                            .splitn(2, ": ")
                            .nth(1)
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                    {
                        self.wifi_name = val;
                        info!("Found WiFi name: {}", self.wifi_name);
                    }
                }
            }
        }
    }

    fn parse_storage_info(&mut self, output: &str) {
        if
            let Some(storage_val) = output
                .lines()
                .nth(1) // Second line (data line)
                .and_then(|line| line.split_whitespace().nth(1)) // Second field (Size column)
        {
            self.storage = storage_val.to_string();
            info!("Found storage size: {}", self.storage);
        } else {
            warn!(
                "Could not parse storage information. Output: {:?}",
                output.lines().take(2).collect::<Vec<_>>()
            );
        }
    }

    fn parse_users_info(&mut self, output: &str) {
        self.home_users = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        info!("Found {} home users: {:?}", self.home_users.len(), self.home_users);
    }

    fn parse_graphics_info(&mut self, output: &str) {
        for line in output.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("Chipset Model:") {
                if let Some(val) = Self::get_value_after_first_colon(trimmed_line) {
                    self.graphics = val;
                    info!("Found graphics chipset: {}", self.graphics);
                    break;
                }
            }
        }
        if self.graphics.is_empty() {
            warn!("No graphics chipset information found");
        }
    }
}
