use dioxus::prelude::*;
use crate::views::{ Home, User, UserList };
use crate::components::navbar::Navbar;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home,
    #[route("/user")]
    UserList,
    #[route("/user/:id")] User {
        id: String,
    },
}

//192.168.10.179

// fleetctl package --type=pkg --enable-scripts --fleet-desktop --fleet-url=192.168.10.179:8412 --enroll-secret=0rU+kxJzrhsjx/Ai4/w0skGC/+pN3kWJ