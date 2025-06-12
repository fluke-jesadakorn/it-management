use serde::{ Serialize, Deserialize };
use dioxus::prelude::ServerFnError;
use log::{ debug, info, warn };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicHostInfo {
    pub hostname: String,
    pub status: String,
    pub network_name: String,
}

fn get_ssh_password() -> String {
    std::env::var("SSH_PASSWORD").unwrap_or_else(|_| "Password".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ssh_password() {
        std::env::set_var("SSH_PASSWORD", "test");
        assert_eq!(get_ssh_password(), "test");
    }
}

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

// Command methods
impl ComputerInfo {
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
}

// Parsing methods
impl ComputerInfo {
    fn parse_hardware_info(&mut self, output: &str) {
        debug!("Parsing hardware information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of hardware output", line_count);

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Model Name:") {
                self.product_name = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found product name: {}", self.product_name);
            } else if line.starts_with("Serial Number") {
                self.serial = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found serial number: {}", self.serial);
            } else if line.starts_with("Processor ") {
                self.processor = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found processor: {}", self.processor);
            } else if line.starts_with("Memory:") {
                self.memory = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found memory: {}", self.memory);
            }
        }
        debug!("Hardware information parsing completed");
    }

    fn parse_system_info(&mut self, output: &str) {
        debug!("Parsing system information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of system output", line_count);

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("System Version:") {
                self.version = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found system version: {}", self.version);
            } else if line.starts_with("Computer Name:") {
                self.title = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found computer name: {}", self.title);
            } else if line.starts_with("User Name:") {
                self.user = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found user name: {}", self.user);
            }
        }
        debug!("System information parsing completed");
    }

    fn parse_network_info(&mut self, output: &str) {
        debug!("Parsing network information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of network output", line_count);

        let mut found_wifi = false;
        for line in output.lines() {
            debug!("Processing network line: {}", line);
            if line.contains("=== Wifi Status ===") {
                found_wifi = true;
                debug!("Found WiFi status section");
                continue;
            }

            if !found_wifi {
                if line.contains("inet ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && self.lan_ip.is_empty() {
                        self.lan_ip = parts[1].to_string();
                        info!("Found LAN IP: {}", self.lan_ip);
                    }
                }
            } else {
                if line.contains("inet ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        self.wifi_ip = parts[1].to_string();
                        info!("Found WiFi IP: {}", self.wifi_ip);
                    }
                } else if line.contains(" SSID: ") {
                    self.wifi_name = line.split(": ").nth(1).unwrap_or("").trim().to_string();
                    info!("Found WiFi name: {}", self.wifi_name);
                }
            }
        }
        debug!("Network information parsing completed");
    }

    fn parse_storage_info(&mut self, output: &str) {
        debug!("Parsing storage information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of storage output", line_count);

        if let Some(line) = output.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                self.storage = parts[1].to_string();
                info!("Found storage size: {}", self.storage);
            } else {
                warn!("Storage line format unexpected: {}", line);
            }
        } else {
            warn!("No storage information found in output");
        }
        debug!("Storage information parsing completed");
    }

    fn parse_users_info(&mut self, output: &str) {
        debug!("Parsing users information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of users output", line_count);

        self.home_users = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        info!("Found {} home users", self.home_users.len());
        for user in &self.home_users {
            info!("  - {}", user);
        }
        debug!("Users information parsing completed");
    }

    fn parse_graphics_info(&mut self, output: &str) {
        debug!("Parsing graphics information...");
        let line_count = output.lines().count();
        debug!("Processing {} lines of graphics output", line_count);

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Chipset Model:") {
                self.graphics = line.split(":").nth(1).unwrap_or("").trim().to_string();
                info!("Found graphics chipset: {}", self.graphics);
                break;
            }
        }

        if self.graphics.is_empty() {
            warn!("No graphics chipset information found");
        }
        debug!("Graphics information parsing completed");
    }
}

// Core implementation
impl BasicHostInfo {
    pub async fn quick_resolve(host: String) -> Result<Self, ServerFnError> {
        info!("Quick resolving host: {}", host);

        // Simple ping check
        let status = match super::command::ssh_exec(&host, "echo 'OK'") {
            Ok(_) => "Online".to_string(),
            Err(_) => "Offline".to_string(),
        };

        Ok(Self {
            hostname: host.clone(),
            status,
            network_name: host,
        })
    }
}

impl ComputerInfo {
    fn new() -> Self {
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

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub async fn resolve(host: String) -> Result<Self, ServerFnError> {
        info!("Starting detailed computer information resolution for host: {}", host);
        debug!(
            "Using SSH password from environment: {}",
            get_ssh_password().chars().take(3).collect::<String>() + "..."
        );

        let mut info = ComputerInfo::new();

        debug!("Executing system checks sequentially...");

        // Execute hardware check
        let hardware_out = super::command
            ::ssh_exec(&host, Self::get_hardware_command())
            .map_err(|e| ServerFnError::new(e))?;

        // Execute system check
        let system_out = super::command
            ::ssh_exec(&host, Self::get_system_command())
            .map_err(|e| ServerFnError::new(e))?;

        // Execute network check
        let network_out = super::command
            ::ssh_exec(&host, Self::get_network_command())
            .map_err(|e| ServerFnError::new(e))?;

        // Execute storage check
        let storage_out = super::command
            ::ssh_exec(&host, Self::get_storage_command())
            .map_err(|e| ServerFnError::new(e))?;

        // Execute users check
        let users_out = super::command
            ::ssh_exec(&host, Self::get_users_command())
            .map_err(|e| ServerFnError::new(e))?;

        // Execute graphics check
        let graphics_out = super::command
            ::ssh_exec(&host, Self::get_graphics_command())
            .map_err(|e| ServerFnError::new(e))?;

        debug!("Parsing system check results...");

        // Parse results
        info.parse_hardware_info(&hardware_out);
        info.parse_system_info(&system_out);
        info.parse_network_info(&network_out);
        info.parse_storage_info(&storage_out);
        info.parse_users_info(&users_out);
        info.parse_graphics_info(&graphics_out);

        info.network_name = host.clone();
        info.architecture = std::env::consts::ARCH.to_string();

        info!("Computer information resolution completed for host: {}", host);
        debug!("Architecture detected: {}", info.architecture);

        Ok(info)
    }
}
