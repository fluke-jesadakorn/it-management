use dioxus::prelude::*;
use serde::{ Serialize, Deserialize };
use std::{ thread, time::Duration };
use std::sync::mpsc as std_mpsc;
use futures::future::join_all;
use crate::server::network::ssh::ssh_exec;
use crate::server::resolve_computer::ComputerInfo;
use crate::server::network::{ get_scan_state, DnsScanner };

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
        let state = get_scan_state().map_err(|e| ServerFnError::new(e.to_string()))?;
        let mut state = state.lock().map_err(|e| ServerFnError::new(e.to_string()))?;
        need_scan = state.discovered_hosts.is_empty() && !state.in_progress;
    }

    if need_scan {
        // Start the scan in a background thread
        let (tx, rx) = std_mpsc::channel();
        thread::spawn(move || {
            if let Ok(state_arc) = get_scan_state().map_err(|e| log::error!("{}", e)) {
                // Set initial state
                if let Ok(mut state) = state_arc.lock() {
                    state.in_progress = true;
                }

                // Start scanner thread
                let handle = thread::spawn(move || {
                    if let Err(e) = DnsScanner::scan_with_retry(tx) {
                        log::error!("Network scan failed: {}", e);
                    }
                });

                // Process incoming hosts
                use crate::server::network::scan::OVERALL_SCAN_TIMEOUT_SECS;
                let start = std::time::Instant::now();
                while start.elapsed() < Duration::from_secs(OVERALL_SCAN_TIMEOUT_SECS) {
                    match rx.try_recv() {
                        Ok(host) => {
                            if let Ok(mut state) = state_arc.lock() {
                                if !state.discovered_hosts.contains(&host) {
                                    state.discovered_hosts.push(host);
                                }
                            }
                        }
                        Err(std_mpsc::TryRecvError::Empty) => {
                            thread::sleep(Duration::from_millis(100));
                        }
                        Err(std_mpsc::TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                }

                // Wait for scan to complete
                handle.join().unwrap_or_else(|e| log::error!("Scan thread panicked: {:?}", e));

                // Update final state
                if let Ok(mut state) = state_arc.lock() {
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
    let state = get_scan_state().map_err(|e| ServerFnError::new(e.to_string()))?;
    let state = state.lock().map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(DiscoveryResult {
        hosts: state.discovered_hosts.clone(),
        scan_complete: state.scan_completed,
    })
}

#[server]
pub async fn resolve_computer(host: String) -> Result<ComputerInfo, ServerFnError> {
    ComputerInfo::resolve(host).await
}

#[server]
pub async fn establish_ssh_connection(
    host: String,
    password: String
) -> Result<(), ServerFnError> {
    log::info!("Attempting to establish SSH connection to host: {}", host);
    
    // Set the password in environment for SSH
    std::env::set_var("SSH_PASSWORD", &password);

    match ssh_exec(host.clone(), host.clone(), password.clone(), "echo 'Connection test'".to_string()).await {
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
    password: String
) -> Result<String, ServerFnError> {
    // Set the password in environment for SSH
    std::env::set_var("SSH_PASSWORD", &password);

    let cmd_copy = command.clone();
    let output = ssh_exec(host.clone(), host.clone(), password, command).await.map_err(|e| ServerFnError::new(e))?;

    if output.contains("Connection timed out") || output.contains("Access denied") {
        let error_msg = format!("SSH command failed:\nCommand: {}\nOutput: {}", cmd_copy, output);
        log::error!("{}", error_msg);
        return Err(ServerFnError::new(error_msg));
    }

    if output.trim().is_empty() {
        log::warn!("Command execution returned no output: {}", cmd_copy);
        return Err(ServerFnError::new("Command execution returned no output"));
    }

    Ok(output)
}

#[server]
pub async fn execute_concurrent_commands(
    hosts: Vec<String>,
    command: String,
    password: String
) -> Result<Vec<(String, String, String)>, ServerFnError> {
    const TIMEOUT_SECS: u64 = 5;
    let total_hosts = hosts.len();
    let completed = std::sync::Arc::new(std::sync::Mutex::new(0));

    let futures: Vec<_> = hosts
        .into_iter()
        .map(|host| {
            let cmd = command.clone();
            let completed = completed.clone();

                let password = password.clone();
                let host_clone = host.clone();
                async move {
                    let start = std::time::Instant::now();
                    let result = ssh_exec(host_clone.clone(), host_clone, password, cmd).await;

                if let Ok(mut count) = completed.lock() {
                    *count += 1;
                }

                if start.elapsed() > Duration::from_secs(TIMEOUT_SECS) {
                    (host, String::new(), "Operation timed out after 30 seconds".to_string())
                } else {
                    match result {
                        Ok(stdout) => (host, stdout, String::new()),
                        Err(e) => (host, String::new(), e.to_string()),
                    }
                }
            }
        })
        .collect();

    Ok(join_all(futures).await)
}
