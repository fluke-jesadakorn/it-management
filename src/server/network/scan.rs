use std::process::{ Command, Stdio };
use std::thread;
use std::time::Duration;
use std::sync::mpsc as std_mpsc;
use std::sync::{ Arc, Mutex };

lazy_static::lazy_static! {
    static ref SCAN_STATE: Arc<Mutex<ScanState>> = Arc::new(Mutex::new(ScanState::new()));
}

#[derive(Debug)]
pub struct ScanState {
    pub in_progress: bool,
    pub discovered_hosts: Vec<String>,
    pub scan_completed: bool,
}

impl ScanState {
    pub fn new() -> Self {
        Self {
            in_progress: false,
            discovered_hosts: Vec::new(),
            scan_completed: false,
        }
    }
}

// Constants for network discovery
const MAX_RETRY_ATTEMPTS: u32 = 3;
const INITIAL_SCAN_TIMEOUT_SECS: u64 = 15; // Longer timeout for initial scan
const MAX_SCAN_LINES: usize = 100;
pub const OVERALL_SCAN_TIMEOUT_SECS: u64 = 30; // Overall timeout for scanning process

// DNS Scanner implementation
pub struct DnsScanner {
    tx: std_mpsc::Sender<String>,
}

impl DnsScanner {
    pub fn new(tx: std_mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    pub fn scan(&self) -> Result<(), String> {
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
            Ok(())
        }
    }

    pub fn scan_with_retry(tx: std_mpsc::Sender<String>) -> Result<(), String> {
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

pub fn get_scan_state() -> Result<Arc<Mutex<ScanState>>, String> {
    Ok(SCAN_STATE.clone())
}
