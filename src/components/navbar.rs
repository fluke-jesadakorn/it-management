use crate::utils::{ ThemeState };
use dioxus::prelude::*;
use crate:: {
    routes::Route,
};
const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component(no_case_check)]
pub fn Navbar() -> Element {
    let theme = use_signal(|| ThemeState { is_dark: false });

    rsx! {
        div {
            document::Link { rel: "stylesheet", href: NAVBAR_CSS }
            
            nav { 
                class: if theme.read().is_dark {
                    "bg-dark-primary shadow-lg transition-colors duration-200" 
                } else { 
                    "bg-white shadow-lg transition-colors duration-200" 
                },
                div {
                    id: "navbar",
                    class: "container mx-auto px-4 py-3 flex justify-between items-center",
                    div {
                        class: "flex items-center space-x-6",
                        Link {
                            class: if theme.read().is_dark {
                                "text-white hover:text-primary transition-colors"
                            } else {
                                "text-gray-800 hover:text-primary transition-colors"
                            },
                            to: Route::Home,
                            "Home"
                        }
                        Link {
                            class: if theme.read().is_dark {
                                "text-white hover:text-primary transition-colors"
                            } else {
                                "text-gray-800 hover:text-primary transition-colors"
                            },
                            to: Route::UserList,
                            "Users"
                        }
                    }
                    button {
                        class: if theme.read().is_dark {
                            "p-2 rounded-lg bg-gray-700 hover:bg-gray-600 transition-colors"
                        } else {
                            "p-2 rounded-lg bg-gray-200 hover:bg-gray-300 transition-colors"
                        },
                        onclick: move |_| {
                            let is_dark = theme.read().is_dark;
                            theme.clone().write().is_dark = !is_dark;
                        },
                        if theme.read().is_dark {
                            "🌞"
                        } else {
                            "🌙"
                        }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}
