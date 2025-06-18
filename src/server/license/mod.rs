pub mod types;
pub mod checker;

// Re-export commonly used items
pub use types::{ LicenseInfo, LicenseCheckResult, SoftwareInfo };
pub use checker::{ LicenseChecker, analyze_expiration_date };
