use std::sync::mpsc::Sender;

#[allow(dead_code)]
pub struct MockDnsScanner {
    pub tx: Sender<String>,
}

#[allow(dead_code)]
impl MockDnsScanner {
    pub fn new(tx: Sender<String>) -> Self {
        Self { tx }
    }

    pub fn scan(&self) -> Result<(), String> {
        // Mock implementation for testing
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MockSshExecutor;

#[allow(dead_code)]
impl MockSshExecutor {
    pub fn exec(_host: &str, _cmd: &str) -> Result<String, String> {
        // Mock implementation for testing
        Ok("mock_output".to_string())
    }
}

#[allow(dead_code)]
pub struct MockLicenseChecker {
    pub host: String,
}

#[allow(dead_code)]
impl MockLicenseChecker {
    pub fn new(host: String) -> Self {
        Self { host }
    }
}
