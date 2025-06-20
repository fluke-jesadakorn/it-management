pub mod command;
pub mod network;
pub mod license;
pub mod resolve_computer;

// Re-export commonly used functionality from command module
pub use command::{
    resolve_network_info,
    resolve_computer,
    establish_ssh_connection,
    execute_ssh_command,
    execute_concurrent_commands,
    check_expired_adobe_plugin_comet_license,
    get_software_info,
    clear_system_cache,
    DiscoveryResult,
};
