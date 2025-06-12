use dioxus::events::FormData;
use dioxus::prelude::*;
use serde::{ Deserialize, Serialize };
use crate::Route;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemInfo {
    device_type: String,
    model: String,
    processor: String,
    mac_os: String,
    graphic: String,
    user: String,
    ip_ether: String,
    ip_wifi: String,
    wifi_ssid: String,
}

impl SystemInfo {
    fn new(
        device_type: String,
        model: String,
        processor: String,
        mac_os: String,
        graphic: String,
        user: String,
        ip_ether: String,
        ip_wifi: String,
        wifi_ssid: String
    ) -> Self {
        Self {
            device_type,
            model,
            processor,
            mac_os,
            graphic,
            user,
            ip_ether,
            ip_wifi,
            wifi_ssid,
        }
    }

    fn mock_data() -> Vec<Self> {
        vec![
            SystemInfo::new(
                "iMac".to_string(),
                "iMac 13,2".to_string(),
                "core i7".to_string(),
                "15.1".to_string(),
                "Intel Iris".to_string(),
                "fluke, admin".to_string(),
                "192.168.0.2".to_string(),
                "192.168.1.2".to_string(),
                "Test".to_string()
            ),
            SystemInfo::new(
                "Macbook Pro".to_string(),
                "Macbook Pro 16,1".to_string(),
                "Apple silicon M1".to_string(),
                "15".to_string(),
                "Apple M4 Max".to_string(),
                "abc, admin".to_string(),
                "192.168.0.3".to_string(),
                "192.168.1.3".to_string(),
                "".to_string()
            )
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MachineStatus {
    Online,
    Offline,
}

impl ToString for MachineStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Online => "Online".to_string(),
            Self::Offline => "Offline".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    pub host: String,
    pub status: MachineStatus,
}

#[component]
pub fn User(#[props] id: String) -> Element {
    let default_machines = vec![
        Machine {
            name: "Local Machine".to_string(),
            host: "localhost".to_string(),
            status: MachineStatus::Online,
        },
        Machine {
            name: "Mac 1".to_string(),
            host: "mac1.local".to_string(),
            status: MachineStatus::Offline,
        },
        Machine {
            name: "Mac 2".to_string(),
            host: "mac2.local".to_string(),
            status: MachineStatus::Online,
        }
    ];

    let machines = use_signal(|| default_machines);
    let mut system_info = use_signal(|| Vec::<SystemInfo>::new());
    let mut selected = use_signal(|| None::<Machine>);

    let fetch_info = move |_| {
        *system_info.write() = SystemInfo::mock_data();
    };

    rsx! {
        div { class: "container mx-auto p-4 mt-8",
            div { class: "mb-4 flex items-center gap-4",
                select {
                    class: "input",
                    onchange: move |evt: Event<FormData>| {
                        let value = evt.data.as_ref().value().to_string();
                        if !value.is_empty() {
                            let found = machines.read().iter().find(|m| m.host == value).cloned();
                            *selected.write() = found;
                        }
                    },

                    option { value: "", "Select a machine" }
                    {machines.read().iter().map(|machine| rsx! {
                        option { value: "{machine.host}", "{machine.name}" }
                    })}
                }

                button {
                    class: "button",
                    disabled: selected.read().is_none(),
                    onclick: fetch_info,
                    "Show System Info"
                }
            }

            table { class: "min-w-full border",
                thead {
                    tr {
                        th { "Type" }
                        th { "Model" }
                        th { "Processor" }
                        th { "Actions" }
                    }
                }
                tbody {
                    {system_info.read().iter().map(|info| rsx! {
                        tr {
                            td { "{info.device_type}" }
                            td { "{info.model}" }
                            td { "{info.processor}" }
                            td {
                                Link {
                                    to: Route::User {
                                        id: info.mac_os.clone(),
                                    },
                                    "View"
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}
