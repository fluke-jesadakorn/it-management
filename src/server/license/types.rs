use chrono::NaiveDate;
use serde::{ Serialize, Deserialize };

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
