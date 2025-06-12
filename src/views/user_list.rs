use dioxus::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{ Promise };
use crate::server::resolve_network_info;
use crate::utils::{ ThemeState };
use crate::server::resolve_computer::{ BasicHostInfo, ComputerInfo };

impl PartialEq for BasicHostInfo {
    fn eq(&self, other: &Self) -> bool {
        self.hostname == other.hostname
    }
}

impl PartialEq for ComputerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.serial == other.serial
    }
}

#[component]
fn ComputerTable(
    basic_computers: Vec<BasicHostInfo>,
    detailed_computers: Signal<Vec<ComputerInfo>>
) -> Element {
    rsx! {
        table { 
            class: "min-w-full divide-y divide-gray-200",
            thead {
                tr {
                    th { class: "px-6 py-3", "Hostname" }
                    th { class: "px-6 py-3", "Status" }
                    th { class: "px-6 py-3", "Details" }
                }
            }
            tbody {
                {basic_computers.iter().map(|basic| {
                    let detailed_data = detailed_computers.read();
                    let detailed = detailed_data.iter().find(|d| d.network_name == basic.network_name);
                    rsx! {
                        tr { 
                            key: "{basic.hostname}",
                            td { class: "px-6 py-4", "{basic.hostname}" }
                            td { class: "px-6 py-4", 
                                span {
                                    class: if basic.status == "Online" { "text-green-500" } else { "text-red-500" },
                                    "{basic.status}"
                                }
                            }
                            td { class: "px-6 py-4", 
                                {
                                    if let Some(info) = detailed {
                                        rsx! {
                                            div {
                                                class: "space-y-1",
                                                div { "Product: {info.product_name}" }
                                                div { "Serial: {info.serial}" }
                                                div { "Version: {info.version}" }
                                            }
                                        }
                                    } else {
                                        rsx! { 
                                            div { 
                                                class: "text-gray-500 animate-pulse",
                                                "Loading details..." 
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}

#[component]
fn ComputerCard(host: String) -> Element {
    let basic_info = use_signal(|| None::<Result<BasicHostInfo, String>>);
    let computer_info = use_signal(|| None::<Result<ComputerInfo, String>>);

    use_effect({
        let basic = basic_info.clone();
        let detailed = computer_info.clone();
        let host_clone = host.clone();
        move || {
            log::info!("ComputerCard effect triggered for host: {}", host_clone);
            let host_inner = host_clone.clone();

            // Priority Thread - Quick Resolve
            spawn(async move {
                match BasicHostInfo::quick_resolve(host_inner.clone()).await {
                    Ok(data) => {
                        log::info!("Quick resolve successful for {}", host_inner);
                        basic.clone().set(Some(Ok(data)));

                        // Background Thread - Detailed Resolution
                        let detailed_clone = detailed.clone();
                        let host_inner2 = host_inner.clone();
                        spawn(async move {
                            match ComputerInfo::resolve(host_inner2.clone()).await {
                                Ok(data) => {
                                    log::info!(
                                        "Detailed resolve successful for {}: {}",
                                        host_inner2,
                                        data.title
                                    );
                                    detailed_clone.clone().set(Some(Ok(data)));
                                }
                                Err(e) => {
                                    log::error!(
                                        "Detailed resolve failed for {}: {}",
                                        host_inner2,
                                        e
                                    );
                                    detailed_clone.clone().set(Some(Err(e.to_string())));
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Quick resolve failed for {}: {}", host_inner, e);
                        basic.clone().set(Some(Err(e.to_string())));
                    }
                }
            });
        }
    });

    let content = match (basic_info.read().as_ref(), computer_info.read().as_ref()) {
        (None, _) =>
            rsx! { 
            div { 
                class: "text-center text-gray-500",
                div { class: "animate-pulse", "Discovering host..." }
            } 
        },
        (Some(Ok(basic)), None) =>
            rsx! {
            div {
                class: "space-y-4",
                div {
                    class: "grid grid-cols-2 gap-2",
                    div { class: "font-semibold", "Host:" }
                    div { "{host}" }
                    div { class: "font-semibold", "Status:" }
                    div { 
                        class: if basic.status == "Online" { "text-green-500" } else { "text-red-500" },
                        "{basic.status}" 
                    }
                }
                div {
                    class: "text-center text-gray-500",
                    div { class: "animate-pulse", "Loading detailed information..." }
                }
            }
        },
        (Some(Ok(_)), Some(Ok(info))) =>
            rsx! {
            div {
                class: "grid grid-cols-2 gap-2",
                div { class: "font-semibold", "Host:" }
                div { "{host}" }
                div { class: "font-semibold", "Product:" }
                div { "{info.product_name}" }
                div { class: "font-semibold", "Serial:" }
                div { "{info.serial}" }
                div { class: "font-semibold", "User:" }
                div { "{info.user}" }
                div { class: "font-semibold", "Version:" }
                div { "{info.version}" }
            }
        },
        (Some(Err(err)), _) | (_, Some(Err(err))) =>
            rsx! {
            div {
                class: "text-center text-red-500",
                div { "Error loading host information" }
                div { class: "text-sm", "{err}" }
            }
        },
    };

    rsx! {
        div { 
            class: "bg-white dark:bg-gray-800 p-4 rounded shadow",
            div { 
                class: "space-y-2",
                div { 
                    class: "text-lg font-bold mb-2 text-center",
                    "{host}"
                }
                {content}
            }
        }
    }
}

#[component]
fn ComputerGrid(hosts: Vec<String>) -> Element {
    let basic_computers = use_signal(Vec::<BasicHostInfo>::new);
    let detailed_computers = use_signal(Vec::<ComputerInfo>::new);
    let show_table = use_signal(|| false);
    let basic_completed = use_signal(|| 0);
    let detailed_completed = use_signal(|| 0);
    let total = use_signal(|| 0);

    use_effect({
        let basic = basic_computers.clone();
        let detailed = detailed_computers.clone();
        let basic_count = basic_completed.clone();
        let detailed_count = detailed_completed.clone();
        let total_count = total.clone();
        let hosts_clone = hosts.clone();

        move || {
            log::info!("ComputerGrid effect triggered with {} hosts", hosts_clone.len());
            // Reset state
            basic.clone().set(Vec::new());
            detailed.clone().set(Vec::new());
            basic_count.clone().set(0);
            detailed_count.clone().set(0);
            total_count.clone().set(hosts_clone.len());

            let hosts_inner = hosts_clone.clone();
            // Priority Thread - Quick Resolve for all hosts
            spawn(async move {
                let mut basic_list = Vec::new();
                for (i, host) in hosts_inner.iter().enumerate() {
                    log::info!("Quick resolving host ({}/{}): {}", i + 1, hosts_inner.len(), host);
                    match BasicHostInfo::quick_resolve(host.clone()).await {
                        Ok(info) => {
                            basic_list.push(info.clone());
                            basic.clone().set(basic_list.clone());

                            // Background Thread - Detailed Resolution
                            let detailed_clone = detailed.clone();
                            let host_clone = host.clone();
                            let detailed_count_clone = detailed_count.clone();
                            let i_clone = i;
                            spawn(async move {
                                match ComputerInfo::resolve(host_clone.clone()).await {
                                    Ok(info) => {
                                        let mut list = detailed_clone.read().to_vec();
                                        list.push(info);
                                        detailed_clone.clone().set(list);
                                        detailed_count_clone.clone().set(i_clone + 1);
                                    }
                                    Err(e) =>
                                        log::error!(
                                            "Detailed resolution failed for {}: {}",
                                            host_clone,
                                            e
                                        ),
                                }
                            });
                        }
                        Err(e) => log::error!("Quick resolve failed for {}: {}", host, e),
                    }
                    basic_count.clone().set(i + 1);
                }
            });
        }
    });

    let basic_done = *basic_completed.read();
    let detailed_done = *detailed_completed.read();
    let total = *total.read();

    let basic_progress = if total > 0 {
        (((basic_done as f32) / (total as f32)) * 100.0).round()
    } else {
        0.0
    };

    let detailed_progress = if total > 0 {
        (((detailed_done as f32) / (total as f32)) * 100.0).round()
    } else {
        0.0
    };

    let status_text = format!(
        "Quick scan: {}/{} | Detailed scan: {}/{}",
        basic_done,
        total,
        detailed_done,
        total
    );

    // FIX: The progress_indicator is no longer created here.
    // Instead, the logic is moved directly into the rsx! block below.

    rsx! {
        div {
            button {
                class: "mb-4 px-4 py-2 bg-blue-500 text-white rounded",
                onclick: move |_| {
                    let current = *show_table.read();
                    show_table.clone().set(!current);
                },
                "Switch View"
            }

            div {
                class: "space-y-4",
                
                // Progress indicators
                if basic_done < total {
                    div {
                        class: "text-center space-y-2 mb-4",
                        div {
                            class: "text-lg",
                            "{status_text}"
                        }
                        div {
                            class: "space-y-2",
                            div {
                                class: "flex items-center space-x-2",
                                span { "Quick Scan:" }
                                div {
                                    class: "flex-1 bg-gray-200 rounded-full h-2.5 dark:bg-gray-700",
                                    div {
                                        class: "bg-blue-600 h-2.5 rounded-full transition-all duration-300",
                                        style: "width: {basic_progress}%"
                                    }
                                }
                            }
                            div {
                                class: "flex items-center space-x-2",
                                span { "Detailed Scan:" }
                                div {
                                    class: "flex-1 bg-gray-200 rounded-full h-2.5 dark:bg-gray-700",
                                    div {
                                        class: "bg-green-600 h-2.5 rounded-full transition-all duration-300",
                                        style: "width: {detailed_progress}%"
                                    }
                                }
                            }
                        }
                    }
                }

                // Content
                div {
                    // FIX: Nested rsx! calls removed here as well.
                    if *show_table.read() {
                        ComputerTable { 
                            basic_computers: basic_computers.read().to_vec(),
                            detailed_computers: detailed_computers.clone()
                        }
                    } else {
                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            {hosts.iter().map(|host| {
                                let host = host.clone();
                                rsx! { ComputerCard { host: host } }
                            })}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn UserList() -> Element {
    let theme = use_signal(|| ThemeState::default());
    let network_info = use_signal(Vec::<String>::new);
    let scan_error = use_signal(|| None::<String>);
    let initialized = use_signal(|| false);
    let scanning = use_signal(|| false);

    // Initial load - run once
    let init_done = use_signal(|| false);

    use_effect({
        let scan = scanning.clone();
        let init_state = initialized.clone();
        let init_flag = init_done.clone();
        let network = network_info.clone();
        let error = scan_error.clone();
        let is_scanning = scanning.clone();

        move || {
            if !*init_flag.read() {
                spawn(async move {
                    init_flag.clone().set(true); // Mark initialization as started
                    log::info!("Starting initial network info load");
                    scan.clone().set(true);

                    // More aggressive initial polling
                    let mut poll_count = 0;
                    const INITIAL_MAX_POLLS: u32 = 10;
                    const POLL_DELAY_MS: u64 = 500;

                    while poll_count < INITIAL_MAX_POLLS {
                        match resolve_network_info().await {
                            Ok(result) => {
                                network.clone().set(result.hosts.clone());
                                if !result.hosts.is_empty() {
                                    is_scanning.clone().set(false);
                                    initialized.clone().set(true);
                                    break;
                                }
                            }
                            Err(e) => {
                                log::error!("Initial scan failed: {}", e);
                                error.clone().set(Some(e.to_string()));
                                scanning.clone().set(false);
                                initialized.clone().set(true);
                                return;
                            }
                        }

                        // Use shorter delay for initial polling
                        let promise = Promise::new(
                            &mut (|resolve, _| {
                                web_sys
                                    ::window()
                                    .unwrap()
                                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                                        &resolve,
                                        POLL_DELAY_MS as i32
                                    )
                                    .unwrap();
                            })
                        );
                        let _ = JsFuture::from(promise).await;
                        poll_count += 1;
                    }

                    // Switch to longer interval polling if no hosts found
                    if network.read().is_empty() {
                        let mut last_host_count = 0;
                        let mut no_change_count = 0;
                        const MAX_NO_CHANGE: u32 = 3;
                        const MAX_POLL_COUNT: u32 = 5;
                        poll_count = 0;

                        while poll_count < MAX_POLL_COUNT {
                            match resolve_network_info().await {
                                Ok(result) => {
                                    scan_error.clone().set(None);
                                    network.clone().set(result.hosts.clone());
                                    initialized.clone().set(true);

                                    if result.scan_complete {
                                        is_scanning.clone().set(false);
                                        break;
                                    }

                                    if result.hosts.len() == last_host_count {
                                        no_change_count += 1;
                                        if no_change_count >= MAX_NO_CHANGE {
                                            break;
                                        }
                                    } else {
                                        no_change_count = 0;
                                        last_host_count = result.hosts.len();
                                    }

                                    // Simple delay using Promise
                                    let promise = Promise::new(
                                        &mut (|resolve, _| {
                                            web_sys
                                                ::window()
                                                .unwrap()
                                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &resolve,
                                                    100
                                                )
                                                .unwrap();
                                        })
                                    );
                                    let _ = JsFuture::from(promise).await;
                                }
                                Err(e) => {
                                    log::error!("Failed to load network info: {}", e);
                                    error.clone().set(Some(e.to_string()));
                                    break;
                                }
                            }
                            poll_count += 1;
                        }
                    }

                    is_scanning.clone().set(false);
                    // Always ensure initialized is set
                    init_state.clone().set(true);
                });
            }
        }
    });

    let class = format!("container mx-auto p-4 {}", if theme.read().is_dark { "dark" } else { "" });

    rsx! {
        div {
            class: "{class}",
            if *initialized.read() {
                button {
                    class: "mb-4 px-4 py-2 bg-green-500 text-white rounded",
                    onclick: move |_| {
                        let mut info = network_info.clone();
                        log::info!("Starting refresh");
                        let mut error = scan_error.clone();
                        let mut scan_status = scanning.clone();
                        spawn(async move {
                            scan_status.set(true);

                            // First attempt to get any available hosts
                            if let Ok(result) = resolve_network_info().await {
                                info.set(result.hosts.clone());
                            }

                            // Continue scanning for more hosts
                            let mut last_host_count = info.read().len();
                            let mut no_change_count = 0;
                            const MAX_NO_CHANGE: u32 = 3;
                            const MAX_POLL_COUNT: u32 = 5;
                            let mut poll_count = 0;

                            while poll_count < MAX_POLL_COUNT {
                                match resolve_network_info().await {
                                    Ok(result) => {
                                        error.set(None);
                                        info.set(result.hosts.clone());

                                        if result.scan_complete {
                                            scan_status.set(false);
                                            break;
                                        }

                                        if result.hosts.len() == last_host_count {
                                            no_change_count += 1;
                                            if no_change_count >= MAX_NO_CHANGE {
                                                break;
                                            }
                                        } else {
                                            no_change_count = 0;
                                            last_host_count = result.hosts.len();
                                        }

                                        // Simple delay using Promise
                                        let promise = Promise::new(&mut |resolve, _| {
                                            web_sys::window()
                                                .unwrap()
                                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &resolve,
                                                    100
                                                )
                                                .unwrap();
                                        });
                                        let _ = JsFuture::from(promise).await;
                                    }
                                    Err(e) => {
                                        log::error!("Refresh failed: {}", e);
                                        error.set(Some(e.to_string()));
                                        break;
                                    }
                                }
                                poll_count += 1;
                            }

                            scan_status.set(false);
                        });
                    },
                    "Refresh"
                }
            } else {
                div { class: "mb-4 px-4 py-2", "Initializing..." }
            }

                div {
                {
                    let hosts = network_info.read();
                    let is_scanning = *scanning.read();
                    let error = scan_error.read();

                    if let Some(err) = error.as_ref() {
                        rsx! { 
                            div { 
                                class: "text-center text-red-500",
                                div { class: "text-lg", "Error loading hosts" }
                                div { class: "text-sm", "{err}" }
                            } 
                        }
                    } else {
                        log::info!("Rendering {} hosts", hosts.len());
                        rsx! { 
                            div {
                                if is_scanning {
                                    div {
                                        class: "text-center mb-4",
                                        div { 
                                            class: "animate-pulse text-sm text-gray-500", 
                                            "Scanning for more hosts..." 
                                        }
                                    }
                                }
                                ComputerGrid { hosts: hosts.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}
