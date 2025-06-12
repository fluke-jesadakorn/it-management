use dioxus::prelude::*;
use serde::{ Serialize, Deserialize };
use chrono::NaiveDate;
use std::process::{ Command, Stdio };
use std::thread;
use std::time::Duration;
use std::sync::mpsc as std_mpsc;
use std::sync::{ Arc, Mutex };
use crate::server::resolve_computer::ComputerInfo;

lazy_static::lazy_static! {
    static ref SCAN_STATE: Arc<Mutex<ScanState>> = Arc::new(Mutex::new(ScanState::new()));
}

#[derive(Debug)]
struct ScanState {
    in_progress: bool,
    discovered_hosts: Vec<String>,
    scan_completed: bool,
}

impl ScanState {
    fn new() -> Self {
        Self {
            in_progress: false,
            discovered_hosts: Vec::new(),
            scan_completed: false,
        }
    }
}

// Constants for network discovery
const MAX_RETRY_ATTEMPTS: u32 = 3;
const SCAN_TIMEOUT_SECS: u64 = 5;
const INITIAL_SCAN_TIMEOUT_SECS: u64 = 15; // Longer timeout for initial scan
const MAX_SCAN_LINES: usize = 100;
const OVERALL_SCAN_TIMEOUT_SECS: u64 = 30; // Overall timeout for scanning process

// SSH execution utilities
const SSH_USER: &str = "ph-admin";

pub fn ssh_exec(host: &str, cmd: &str) -> Result<String, String> {
    let password = "8Mal8=49";

    let expect_script = format!(
        r#"
        spawn ssh -o StrictHostKeyChecking=no {}@{} "{}"
        expect {{
            "assword:" {{
                send "{}\r"
                expect {{
                    eof {{ exit 0 }}
                    timeout {{ exit 1 }}
                }}
            }}
            eof {{ exit 1 }}
        }}
        "#,
        SSH_USER,
        host,
        cmd,
        password
    );

    let output = Command::new("expect")
        .arg("-c")
        .arg(&expect_script)
        .output()
        .map_err(|e| format!("Expect execution failed: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Invalid UTF-8 in stdout: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("Command failed: {} (stderr: {}, stdout: {})", output.status, stderr, stdout))
    }
}

// DNS Scanner implementation
struct DnsScanner {
    tx: std_mpsc::Sender<String>,
}

impl DnsScanner {
    fn new(tx: std_mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    fn scan(&self) -> Result<(), String> {
        // Initial delay to allow network services to initialize
        thread::sleep(Duration::from_secs(2));

        let mut child = Command::new("dns-sd")
            .args(&["-B", "_ssh._tcp", "."])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start dns-sd: {}", e))?;

        let stdout = child.stdout.take().ok_or_else(|| "Failed to capture stdout".to_string())?;

        use std::io::{ BufRead, BufReader };
        let reader = BufReader::new(stdout);
        let mut found_header = false;
        let mut found_hosts = false;
        let start_time = std::time::Instant::now();
        let mut scanned_lines = 0;

        for line in reader.lines() {
            // Use longer timeout for initial scan
            if start_time.elapsed() > Duration::from_secs(INITIAL_SCAN_TIMEOUT_SECS) {
                log::warn!("DNS-SD scan timed out after {} lines", scanned_lines);
                // Kill the child process if still running
                let _ = child.kill();
                break;
            }

            if scanned_lines >= MAX_SCAN_LINES {
                log::warn!("DNS-SD scan reached maximum lines ({})", MAX_SCAN_LINES);
                let _ = child.kill();
                break;
            }

            if let Ok(line) = line {
                scanned_lines += 1;
                log::debug!("Scanning line {}", scanned_lines);

                if !found_header {
                    if line.contains("Timestamp") {
                        found_header = true;
                        log::info!("Found header, starting host discovery");
                    }
                    continue;
                }

                if line.contains("Add") && line.contains("local.") {
                    if let Some(host) = line.split_whitespace().last() {
                        let host = host.trim().strip_suffix(".local").unwrap_or(host);
                        if !host.is_empty() {
                            let host = format!("{}.local", host);
                            log::info!("Found host: {}", host);
                            found_hosts = true;
                            if let Err(e) = self.tx.send(host) {
                                log::error!("Failed to send host: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // Ensure process is killed if still running after timeout
        let _ = child.kill();

        if !found_hosts {
            Err("No hosts found on network".to_string())
        } else {
            log::info!("Completed scanning {} lines", scanned_lines);
            Ok(())
        }
    }

    fn scan_with_retry(tx: std_mpsc::Sender<String>) -> Result<(), String> {
        let mut retry_count = 0;
        let mut any_success = false;

        while retry_count < MAX_RETRY_ATTEMPTS {
            let scanner = DnsScanner::new(tx.clone());
            log::info!("Starting DNS-SD scan attempt {}/{}", retry_count + 1, MAX_RETRY_ATTEMPTS);

            // First attempt: wait longer before failing
            if retry_count == 0 {
                thread::sleep(Duration::from_secs(2)); // Additional initialization time
            }

            match scanner.scan() {
                Ok(()) => {
                    any_success = true;
                    log::info!("Scan attempt {} succeeded", retry_count + 1);
                    break;
                }
                Err(e) => {
                    log::warn!("Scan attempt {} failed: {}", retry_count + 1, e);
                    if retry_count < MAX_RETRY_ATTEMPTS - 1 {
                        let backoff = (retry_count + 1) as u64;
                        log::info!("Waiting {} seconds before retry...", backoff);
                        thread::sleep(Duration::from_secs(backoff));
                    }
                }
            }

            retry_count += 1;
        }

        if any_success {
            log::info!("Scan succeeded after {} attempt(s)", retry_count);
            Ok(())
        } else {
            let error = format!("DNS-SD scan failed after {} attempts", MAX_RETRY_ATTEMPTS);
            log::error!("{}", error);
            Err(error)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub hosts: Vec<String>,
    pub scan_complete: bool,
}

#[server(ResolveNetworkInfo)]
pub async fn resolve_network_info() -> Result<DiscoveryResult, ServerFnError> {
    // Check if we need to start a scan
    let mut need_scan = false;
    {
        let state = SCAN_STATE.lock().map_err(|e| ServerFnError::new(e.to_string()))?;
        need_scan = state.discovered_hosts.is_empty() && !state.in_progress;
    }

    if need_scan {
        // Start the scan in a background thread
        let (tx, rx) = std_mpsc::channel();
        thread::spawn(move || {
            if let Ok(mut state) = SCAN_STATE.lock() {
                state.in_progress = true;
                drop(state);

                let handle = thread::spawn(move || {
                    if let Err(e) = DnsScanner::scan_with_retry(tx) {
                        log::error!("Network scan failed: {}", e);
                    }
                });

                // Process incoming hosts
                let start = std::time::Instant::now();
                while start.elapsed() < Duration::from_secs(OVERALL_SCAN_TIMEOUT_SECS) {
                    match rx.try_recv() {
                        Ok(host) => {
                            if let Ok(mut state) = SCAN_STATE.lock() {
                                if !state.discovered_hosts.contains(&host) {
                                    state.discovered_hosts.push(host);
                                }
                            }
                        }
                        Err(std_mpsc::TryRecvError::Empty) =>
                            thread::sleep(Duration::from_millis(100)),
                        Err(std_mpsc::TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                }

                // Wait for scan to complete
                handle.join().unwrap_or_else(|e| log::error!("Scan thread panicked: {:?}", e));

                // Update final state
                if let Ok(mut state) = SCAN_STATE.lock() {
                    state.in_progress = false;
                    if !state.discovered_hosts.is_empty() {
                        state.scan_completed = true;
                    }
                }
            }
        });

        // Initial delay to allow hosts to be discovered
        thread::sleep(Duration::from_secs(3));
    }

    // Return current state
    let state = SCAN_STATE.lock().map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(DiscoveryResult {
        hosts: state.discovered_hosts.clone(),
        scan_complete: state.scan_completed,
    })
}

#[server]
pub async fn resolve_computer(host: String) -> Result<ComputerInfo, ServerFnError> {
    ComputerInfo::resolve(host).await
}

// License Management
#[derive(Debug)]
pub struct LicenseInfo {
    pub expiration_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoftwareInfo {
    pub priint_comet_2017_expiry: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseCheckResult {
    pub host: String,
    pub status: String,
    pub error: Option<String>,
    pub debug_log: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseChecker {
    host: String,
}

impl LicenseChecker {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    pub async fn check_comet_license(&self) -> Result<LicenseCheckResult, ServerFnError> {
        log::info!("[STEP 1/4] Starting license check for host: {}", self.host);

        // Test connection first
        if let Err(e) = ssh_exec(&self.host, "echo 'Connection test'") {
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

        let license_cmd = format!(
            r#"for p in {}; do cat "$p" 2>/dev/null | grep -i "expires"; done"#,
            escaped_paths.join(" ")
        );

        match ssh_exec(&self.host, &license_cmd) {
            Ok(output) => {
                if !output.trim().is_empty() {
                    // Found and read the license file successfully
                    log::debug!("Found license expiration info: {}", output);
                    let license_info = self.parse_license_output(&output)?;
                    log::info!("[STEP 3/4] License check completed successfully");
                    return Ok(LicenseCheckResult {
                        host: self.host.clone(),
                        status: license_info.status,
                        error: None,
                        debug_log: format!("Successfully found and parsed license expiration info: {}", output),
                    });
                }
                // No expiration info found
                log::warn!("License file exists but contains no expiration info");
                return Ok(LicenseCheckResult {
                    host: self.host.clone(),
                    status: "[ERROR] No Expiration Info".to_string(),
                    error: Some("License file exists but contains no expiration info".to_string()),
                    debug_log: output,
                });
            }
            Err(e) => {
                log::error!("Failed to read license file: {}", e);
                return Ok(LicenseCheckResult {
                    host: self.host.clone(),
                    status: "[ERROR] License File Not Found".to_string(),
                    error: Some(format!("Failed to read license file: {}", e)),
                    debug_log: "Could not find or read license file".to_string(),
                });
            }
        }
    }

    fn parse_license_output(&self, output: &str) -> Result<LicenseInfo, ServerFnError> {
        log::debug!("Processing license output: {}", output);

        // Return the raw output content instead of trying to parse it
        Ok(LicenseInfo {
            expiration_date: None,
            status: output.trim().to_string(),
        })
    }
}

pub async fn get_software_info(host: String) -> Result<SoftwareInfo, ServerFnError> {
    let comet_license = check_expired_adobe_plugin_comet_license(host).await?;
    Ok(SoftwareInfo {
        priint_comet_2017_expiry: comet_license.status,
    })
}

#[server]
pub async fn establish_ssh_connection(
    host: String,
    _password: String
) -> Result<(), ServerFnError> {
    log::info!("Attempting to establish SSH connection to host: {}", host);

    match ssh_exec(&host, "echo 'Connection test'") {
        Ok(_) => {
            log::info!("Successfully established SSH connection to {}", host);
            Ok(())
        }
        Err(e) => {
            log::error!("SSH connection failed: {}", e);
            Err(ServerFnError::new(e))
        }
    }
}

#[server]
pub async fn execute_ssh_command(
    host: String,
    command: String,
    _password: String
) -> Result<String, ServerFnError> {
    log::info!("[STEP 2.1/4] Executing command on remote host: {}", host);
    log::debug!("Attempting to execute command: {}", command);

    let output = ssh_exec(&host, &command).map_err(|e| ServerFnError::new(e))?;

    if output.contains("Connection timed out") || output.contains("Access denied") {
        let error_msg = format!("SSH command failed:\nCommand: {}\nOutput: {}", command, output);
        log::error!("{}", error_msg);
        return Err(ServerFnError::new(error_msg));
    }

    if output.trim().is_empty() {
        log::warn!("Command execution returned no output: {}", command);
        return Err(ServerFnError::new("Command execution returned no output"));
    }

    log::debug!("Successfully executed command and received output");
    Ok(output)
}

#[server]
pub async fn check_expired_adobe_plugin_comet_license(
    host: String
) -> Result<LicenseCheckResult, ServerFnError> {
    let command =
        "cat \'/Applications/Adobe InDesign CC 2017/Plug-Ins/priint.comet 4.1.6 R R25255/w2_license.lic\' | grep Expires";
    match ssh_exec(&host, command) {
        Ok(output) => {
            // Extract date from output
            let date_str = output
                .trim_start_matches("//")
                .trim()
                .split(':')
                .nth(1)
                .map(|s| s.trim());
            if let Some(date_str) = date_str {
                if let Ok(exp_date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
                    let today = chrono::Local::now().naive_local().date();
                    let status = analyze_expiration_date(&exp_date, &today);
                    Ok(LicenseCheckResult {
                        host: host,
                        status: status,
                        error: None,
                        debug_log: output,
                    })
                } else {
                    Ok(LicenseCheckResult {
                        host: host,
                        status: "[ERROR] Invalid Date Format".to_string(),
                        error: Some("Could not parse expiration date".to_string()),
                        debug_log: output,
                    })
                }
            } else {
                Ok(LicenseCheckResult {
                    host: host,
                    status: "[ERROR] Invalid License Format".to_string(),
                    error: Some("Could not find expiration date in license file".to_string()),
                    debug_log: output,
                })
            }
        }
        Err(e) =>
            Ok(LicenseCheckResult {
                host: host,
                status: "[ERROR] License File Not Found".to_string(),
                error: Some(format!("Failed to read license file: {}", e)),
                debug_log: format!("spawn ssh -o StrictHostKeyChecking=no ph-admin@vg-ph-fon.local cat '/Applications/Adobe InDesign CC 2017/Plug-Ins/priint.comet 4.1.6 R R25255/w2_license.lic' | grep Expires\n\n(ph-admin@vg-ph-fon.local) Password:\n{}", e),
            }),
    }
}

#[server]
pub async fn check_license_expired(host: String) -> Result<bool, ServerFnError> {
    match check_expired_adobe_plugin_comet_license(host).await {
        Ok(_) => Ok(false),
        Err(_) => Ok(true),
    }
}

#[server]
pub async fn clear_system_cache(host: String) -> Result<String, ServerFnError> {
    let command =
        r#"sudo rm -rf ~/Library/Caches/* && sudo rm -rf /Library/Caches/* && echo 'Cache cleared successfully'"#;
    let output = ssh_exec(&host, command).map_err(|e| ServerFnError::new(e))?;

    if output.contains("Cache cleared successfully") {
        Ok("Cache cleared successfully".to_string())
    } else {
        Err(ServerFnError::new(format!("Failed to clear cache: {}", output)))
    }
}

#[server]
pub async fn execute_concurrent_commands(
    hosts: Vec<String>,
    command: String
) -> Result<Vec<(String, String, String)>, ServerFnError> {
    use futures::future::{ join_all, FutureExt };
    use std::sync::{ Arc, Mutex };

    const TIMEOUT_SECS: u64 = 5;
    let total_hosts = hosts.len();
    let completed = Arc::new(Mutex::new(0));

    let futures: Vec<_> = hosts
        .into_iter()
        .map(|host| {
            let cmd = command.clone();
            let completed = completed.clone();

            async move {
                let start = std::time::Instant::now();
                let result = {
                    let host_copy = host.clone();
                    let cmd_copy = cmd.clone();
                    thread
                        ::spawn(move || ssh_exec(&host_copy, &cmd_copy))
                        .join()
                        .unwrap_or_else(|_| Err("Thread panicked".to_string()))
                };

                if let Ok(mut count) = completed.lock() {
                    *count += 1;
                    log::info!("Progress: {}/{} commands completed", *count, total_hosts);
                }

                if start.elapsed() > Duration::from_secs(TIMEOUT_SECS) {
                    (host, String::new(), "Operation timed out after 30 seconds".to_string())
                } else {
                    match result {
                        Ok(stdout) => (host, stdout, String::new()),
                        Err(e) => (host, String::new(), e),
                    }
                }
            }
        })
        .collect();

    Ok(join_all(futures).await)
}

fn analyze_expiration_date(exp_date: &NaiveDate, today: &NaiveDate) -> String {
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
