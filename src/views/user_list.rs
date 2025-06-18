use dioxus::prelude::*;
use crate::server::resolve_network_info;
use crate::utils::ThemeState;
use crate::server::resolve_computer::ComputerInfo;

#[component]
pub fn StatusIndicator(status: String) -> Element {
    rsx!(
        if status == "Online" {
            span { 
                class: "text-green-500 font-medium",
                "Online"
            }
        } else {
            span { 
                class: "text-red-500 font-medium",
                "Offline"
            }
        }
    )
}

#[component]
fn Table(networks: Signal<Vec<String>>) -> Element {
    let computer_info = use_signal(|| std::collections::HashMap::<String, ComputerInfo>::new());
    let fetching = use_signal(|| std::collections::HashSet::<String>::new());

    let get_info = move |host: String| {
        let computer_info = computer_info.clone();
        let fetching = fetching.clone();

        fetching.clone().with_mut(|f| {
            f.insert(host.clone());
        });

        spawn(async move {
            match ComputerInfo::resolve(host.clone()).await {
                Ok(info) => {
                    computer_info.clone().with_mut(|map| {
                        map.insert(host.clone(), info);
                    });
                }
                Err(e) => {
                    log::error!("Failed to get computer info: {}", e);
                }
            }
            fetching.clone().with_mut(|f| {
                f.remove(&host);
            });
        });
    };

    let networks_data = networks.read();
    let iter_data: Vec<_> = networks_data.iter().enumerate().collect();
    let rows = iter_data.into_iter().map(|(idx, host)| {
        let host = host.to_string();
        let host_ref = host.clone(); // Clone for disabled check
        rsx!(
            tr { 
                key: {idx},
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.title.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.product_name.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.serial.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.version.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.user.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {host.clone()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.processor.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.architecture.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.memory.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.graphics.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4", {computer_info.read().get(&host).map(|i| i.storage.as_str()).unwrap_or_default()} }
                td { class: "px-6 py-4 text-center", StatusIndicator { status: "Online".to_string() } }
                td { class: "px-6 py-4",
                    button {
                        class: if fetching.read().contains(&host_ref) {
                            "px-3 py-1 bg-blue-500 text-white rounded text-sm opacity-50 cursor-not-allowed"
                        } else {
                            "px-3 py-1 bg-blue-500 text-white rounded text-sm hover:bg-blue-600 transition duration-300 ease-in-out"
                        },
                        onclick: move |_| get_info(host.clone()),
                        disabled: {fetching.read().contains(&host_ref)},
                        {if fetching.read().contains(&host_ref) { "Getting Info..." } else { "Get Info" }}
                    }
                }
            }
        )
    });

    rsx!(
        div { 
            class: "overflow-hidden shadow ring-1 ring-black ring-opacity-5 sm:rounded-lg",
            table { 
                class: "min-w-full divide-y divide-gray-300",
                thead { 
                    class: "bg-gray-50",
                    tr {
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Title" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Product" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Serial" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Version" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "User" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Network" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "CPU" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Arch" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "RAM" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "GPU" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Storage" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Status" }
                        th { class: "py-3.5 px-3 text-left text-sm font-semibold text-gray-900", "Action" }
                    }
                }
                tbody { 
                    class: "divide-y divide-gray-200 bg-white",
                    {rows}
                }
            }
        }
    )
}

#[component]
fn MainView(hosts: Vec<String>) -> Element {
    let hosts = use_signal(|| hosts);

    rsx!(
        div {
            Table { networks: hosts }
        }
    )
}

#[component]
pub fn UserList() -> Element {
    let theme = use_signal(|| ThemeState::default());
    let network_info = use_signal(Vec::<String>::new);
    let scan_error = use_signal(|| None::<String>);
    let initialized = use_signal(|| false);
    let scanning = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let network = network_info.clone();
            let error = scan_error.clone();
            let init = initialized.clone();
            let scan = scanning.clone();

            scan.clone().set(true);
            match resolve_network_info().await {
                Ok(result) => {
                    network.clone().set(result.hosts);
                    init.clone().set(true);
                }
                Err(e) => error.clone().set(Some(e.to_string())),
            }
            scan.clone().set(false);
        });
    });

    let class = format!("container mx-auto p-4 {}", if theme.read().is_dark { "dark" } else { "" });

    rsx!(
        div {
            class: class.clone(),
            if *initialized.read() {
                button {
                    class: "mb-4 py-2 px-4 bg-green-500 text-white rounded hover:bg-green-600 transition duration-300 ease-in-out",
                    onclick: move |_| {
                        let network = network_info.clone();
                        let error = scan_error.clone();
                        let scan = scanning.clone();
                        
                        spawn(async move {
                            scan.clone().set(true);
                            match resolve_network_info().await {
                                Ok(result) => network.clone().set(result.hosts),
                                Err(e) => error.clone().set(Some(e.to_string()))
                            }
                            scan.clone().set(false);
                        });
                    },
                    "Refresh"
                }
            } else {
                div { class: "mb-4 px-4 py-2", "Initializing..." }
            }

            {
                let hosts = network_info.read();
                let is_scanning = *scanning.read();
                let error = scan_error.read();

                if let Some(err) = error.as_ref() {
                    rsx!(
                        div { 
                            class: "text-center text-red-500",
                            div { "Error loading hosts" }
                            div { {format!("{}", err)} }
                        }
                    )
                } else if is_scanning {
                    rsx!(
                        div { 
                            class: "text-center text-gray-500", 
                            "Scanning..." 
                        }
                    )
                } else {
                    rsx!(MainView { hosts: hosts.to_vec() })
                }
            }
        }
    )
}
