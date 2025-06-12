use dioxus::prelude::*;
use it_management::views::{ Home, Navbar, User, UserList };
use it_management::utils::ThemeState;
use it_management::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        dotenv::dotenv().ok();
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        env_logger::init();
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap();
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let theme = use_signal(|| ThemeState::default());
    use_context_provider(|| theme);

    rsx! {
        div {
            class: if theme().is_dark { "dark" } else { "" },
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            Router::<Route> {}
        }
    }
}
