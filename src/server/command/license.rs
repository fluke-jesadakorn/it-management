use dioxus::prelude::*;
use chrono::NaiveDate;
use crate::server::network::ssh::ssh_exec;
use crate::configs::env_validate::get_ssh_password;
use crate::server::license::types::{ LicenseCheckResult, SoftwareInfo };
use crate::server::license::checker::analyze_expiration_date;

#[server]
pub async fn check_expired_adobe_plugin_comet_license(
    host: String
) -> Result<LicenseCheckResult, ServerFnError> {
    let command =
        "cat \'/Applications/Adobe InDesign CC 2017/Plug-Ins/priint.comet 4.1.6 R R25255/w2_license.lic\' | grep Expires";
    let password = get_ssh_password().map_err(|e| ServerFnError::new(e))?;
    match ssh_exec(host.clone(), host.clone(), password, command.to_string()).await {
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

pub async fn get_software_info(host: String) -> Result<SoftwareInfo, ServerFnError> {
    // Define all license check functions as futures
    let comet_check = check_expired_adobe_plugin_comet_license(host.clone()).await?;

    // Currently only have one check, but ready for future expansion
    // When adding more checks, consider using futures::join! or futures::try_join!

    Ok(SoftwareInfo {
        priint_comet_2017_expiry: comet_check.status,
    })
}

#[server]
pub async fn clear_system_cache(host: String) -> Result<String, ServerFnError> {
    let command =
        r#"sudo rm -rf ~/Library/Caches/* && sudo rm -rf /Library/Caches/* && echo 'Cache cleared successfully'"#;
    let password = get_ssh_password().map_err(|e| ServerFnError::new(e))?;
    let output = ssh_exec(host.clone(), host.clone(), password, command.to_string()).await.map_err(
        |e| ServerFnError::new(e)
    )?;

    if output.contains("Cache cleared successfully") {
        Ok("Cache cleared successfully".to_string())
    } else {
        Err(ServerFnError::new(format!("Failed to clear cache: {}", output)))
    }
}
