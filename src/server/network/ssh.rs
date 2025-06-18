use dioxus::prelude::*;
use std::process::{ Command, Stdio };
use crate::SSHError;

fn process_ssh_output(output: Vec<u8>, error: Vec<u8>) -> Result<String, SSHError> {
    let stdout = String::from_utf8_lossy(&output);
    let stderr = String::from_utf8_lossy(&error);

    // Filter out debug/info messages from stderr
    let stderr_lines: Vec<&str> = stderr
        .lines()
        .filter(
            |line|
                !line.contains("Attempting authentication") &&
                !line.contains("Authentication failed, attempt") &&
                !line.starts_with("Warning: Permanently added")
        )
        .collect();

    if !stderr_lines.is_empty() {
        let error_str = stderr_lines.join("\n").to_lowercase();

        // Critical SSH errors that should always be treated as errors
        if error_str.contains("connection closed") {
            return Err(SSHError::Connection("Connection closed".to_string()));
        }
        if error_str.contains("connection timed out") || error_str.contains("operation timed out") {
            return Err(SSHError::Connection("Connection timed out".to_string()));
        }
        if error_str.contains("permission denied") {
            return Err(SSHError::Authentication("Permission denied".to_string()));
        }
        if error_str.contains("too many authentication failures") {
            return Err(SSHError::Authentication("Max authentication attempts reached".to_string()));
        }
        if error_str.contains("account is locked") {
            return Err(SSHError::Authentication("Account is locked".to_string()));
        }
        if error_str.contains("connection refused") {
            return Err(SSHError::Connection("Connection refused".to_string()));
        }

        // Only consider stderr as error if it contains specific error messages
        // and doesn't contain any stdout
        if
            (error_str.contains("error:") || error_str.contains("fatal:")) &&
            stdout.trim().is_empty()
        {
            return Err(SSHError::IO(format!("SSH command failed: {}", stderr.trim())));
        }
    }

    // If we have stdout content, return it regardless of stderr
    // This handles cases where stderr might contain warnings but command succeeded
    if !stdout.trim().is_empty() {
        Ok(stdout.trim().to_string())
    } else {
        // If no stdout and no error detected in stderr, return empty string
        Ok(String::new())
    }
}

#[server]
pub async fn ssh_exec(
    host: String,
    username: String,
    password: String,
    cmd: String
) -> Result<String, ServerFnError> {
    // Get the path to the ssh_script.sh
    let script_path = std::env
        ::current_dir()
        .map_err(|e| SSHError::IO(format!("Failed to get current directory: {}", e)))?
        .join("scripts")
        .join("ssh_script.sh");

    if !script_path.exists() {
        return Err(
            SSHError::IO(format!("SSH script not found at: {}", script_path.display())).into()
        );
    }

    let mut child = Command::new("expect")
        .arg(script_path)
        .arg(&username)
        .arg(&password)
        .arg(&host)
        .arg(&cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SSHError::IO(format!("Failed to spawn SSH process: {}", e)))?;

    let output = child
        .wait_with_output()
        .map_err(|e| SSHError::IO(format!("Failed to get command output: {}", e)))?;

    process_ssh_output(output.stdout, output.stderr).map_err(|e| e.into())
}
