pub mod network;
pub mod license;

// Re-export commonly used functionality
pub use network::{
    resolve_network_info,
    resolve_computer,
    establish_ssh_connection,
    execute_ssh_command,
    execute_concurrent_commands,
    DiscoveryResult,
};

pub use license::{
    check_expired_adobe_plugin_comet_license,
    get_software_info,
    clear_system_cache,
};
