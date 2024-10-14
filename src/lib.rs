
pub mod routes {
    use axum::Router;

    pub mod create_container;
    pub mod get_container;
    pub mod home;
    pub mod login;
    pub mod signup;
    pub mod verify;
    pub mod admin {
        pub mod add_credits;
    }
    pub mod get_credits;
    pub fn get_routes() -> Vec<Router> {
        vec![
            home::get_routes(),
            create_container::get_routes(),
            signup::get_routes(),
            verify::get_routes(),
            login::get_routes(),
            get_container::get_routes(),
            admin::add_credits::get_routes(),
            get_credits::get_routes()
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
