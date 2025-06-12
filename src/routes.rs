use dioxus::prelude::*;
use crate::views::{ Home, Navbar, User, UserList };

#[derive(Clone, Routable, PartialEq)]
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
