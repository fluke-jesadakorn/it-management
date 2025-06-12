use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { 
            class: "container mx-auto p-4",
            h1 { 
                class: "text-2xl font-bold mb-4",
                "Welcome to IT Management"
            }
            p { 
                class: "text-gray-600",
                "Use the navigation menu to manage and monitor your machines."
            }
        }
    }
}
