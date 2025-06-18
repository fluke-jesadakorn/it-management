use chrono::NaiveDate;
use super::types::{ LicenseInfo, LicenseCheckResult };
use crate::server::network::ssh::ssh_exec;
use crate::configs::env_validate::get_ssh_password;

#[derive(Debug)]
pub struct LicenseChecker {
    host: String,
}

impl LicenseChecker {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    pub async fn check_comet_license(&self) -> Result<LicenseCheckResult, String> {
        log::info!("[STEP 1/4] Starting license check for host: {}", self.host);

        // Test connection first
        let password = get_ssh_password().map_err(|e| e.to_string())?;
        let test_cmd = "echo 'Connection test'".to_string();
        if let Err(e) = ssh_exec(self.host.clone(), self.host.clone(), password.clone(), test_cmd.to_string()).await {
            let error_msg = e.to_string();
            return Ok(LicenseCheckResult {
                host: self.host.clone(),
                status: "[ERROR] SSH Connection Failed".to_string(),
                error: Some(format!("SSH connection error: {}", error_msg)),
                debug_log: format!("SSH connection failed to {}\nError: {}", self.host, error_msg),
            });
        }

        // Try reading the license file from standard locations
        log::info!("[STEP 2/4] Reading license file");
        let license_paths = [
            "\'/Applications/Adobe InDesign CC 2017/Plug-Ins/priint.comet 4.1.6 R R25255/w2_license.lic\'",
        ];
        let escaped_paths: Vec<String> = license_paths
            .iter()
            .map(|path| path.replace(" ", "\\ "))
            .collect();

        // Create futures for concurrent license file checks
        let futures: Vec<_> = escaped_paths.iter().map(|path| {
            let host = self.host.clone();
            let password = get_ssh_password().unwrap_or_default();
            let cmd = format!(r#"cat {} 2>/dev/null | grep -i "expires""#, path);
            async move { ssh_exec(host.clone(), host.clone(), password, cmd).await }
        }).collect();

        // Execute all checks concurrently
        let results = futures::future::join_all(futures).await;
        
        // Process results
        let mut combined_output = String::new();
        for result in results {
            match result {
                Ok(output) if !output.trim().is_empty() => {
                    combined_output.push_str(&output);
                    combined_output.push('\n');
                }
                _ => continue,
            }
        }

        if !combined_output.trim().is_empty() {
            // Found and read at least one license file successfully
            let license_info = self.parse_license_output(&combined_output)?;
            log::info!("[STEP 3/4] License check completed successfully");
            Ok(LicenseCheckResult {
                host: self.host.clone(),
                status: license_info.status,
                error: None,
                debug_log: String::new(),
            })
        } else {
            // No valid license files found
            log::warn!("No valid license files found");
            Ok(LicenseCheckResult {
                host: self.host.clone(),
                status: "[ERROR] No Valid License Files".to_string(),
                error: Some("No valid license files found".to_string()),
                debug_log: String::new(),
            })
        }
    }

    fn parse_license_output(&self, output: &str) -> Result<LicenseInfo, String> {
        if let Some(date_str) = output
            .trim_start_matches("//")
            .trim()
            .split(':')
            .nth(1)
            .map(|s| s.trim())
        {
            if let Ok(exp_date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
                return Ok(LicenseInfo {
                    expiration_date: Some(exp_date),
                    status: output.trim().to_string(),
                });
            }
        }
        Ok(LicenseInfo {
            expiration_date: None,
            status: output.trim().to_string(),
        })
    }
}

pub fn analyze_expiration_date(exp_date: &NaiveDate, today: &NaiveDate) -> String {
    let days_until_expiry = exp_date.signed_duration_since(*today).num_days();
    match days_until_expiry {
        d if d < 0 =>
            format!(
                "[EXPIRED] {} ({} days overdue) [Countdown: {} days overdue]",
                exp_date,
                d.abs(),
                d.abs()
            ),
        d if d <= 30 => format!("[WARNING] Expires {} [Countdown: {} days remaining]", exp_date, d),
        d if d <= 90 => format!("[NOTICE] Expires {} [Countdown: {} days remaining]", exp_date, d),
        d => format!("[OK] Valid until {} [Countdown: {} days remaining]", exp_date, d),
    }
}
