/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

pub mod routes {
    use axum::Router;

    pub mod home;

    pub mod auth {
        pub mod login;
        pub mod signup;
        pub mod verify;
    }
    pub mod container {
        pub mod calculator;
        pub mod create;
        pub mod delete;
        pub mod start;
        pub mod stop;
    }
    pub mod admin {
        pub mod set_credits;
    }
    pub mod account {
        pub mod get_container;
        pub mod get_credits;
    }
    pub fn get_routes() -> Vec<Router> {
        vec![
            home::get_routes(),
            container::create::get_routes(),
            auth::signup::get_routes(),
            auth::verify::get_routes(),
            auth::login::get_routes(),
            account::get_container::get_routes(),
            admin::set_credits::get_routes(),
            account::get_credits::get_routes(),
            container::delete::get_routes(),
            container::start::get_routes(),
            container::stop::get_routes(),
            container::calculator::get_routes(),
        ]
    }
}
pub mod utils {
    pub mod container;
    pub mod db;
    pub mod res;
    pub mod resources;
    pub mod validation;
}
