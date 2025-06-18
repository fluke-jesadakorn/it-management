pub fn get_ssh_password() -> Result<String, std::env::VarError> {
    // Get password from environment or use development password
    let password = std::env::var("SSH_PASSWORD").or_else(|_| {
        // Log warning about using development password
        log::warn!("SSH_PASSWORD not set, using development fallback password");
        Ok("8Mal8=49".to_string()) // Updated to match working manual password
    })?;

    // Log masked password at info level for better visibility during debugging
    let masked = "*".repeat(password.len() - 4) + &password[password.len().saturating_sub(4)..];
    log::info!("Using SSH password (masked): {}", masked);

    Ok(password)
}
