
use std::error::Error;

use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[macro_use] extern crate rocket;
pub mod routes;
pub mod database;
pub mod config;


#[launch]
async fn rocket() -> _ {
    //Configuration via config file
    let app_config = config::load_config("./config.toml".to_owned()) //TODO: make this path configurable
        .expect("Failed to load config"); 

    let database = database::init_connection(&app_config.database_uri, app_config.max_connections)
        .await
        .expect("Failed to initialize database");

    rocket::build()
        .attach(Template::fairing())
        .mount(
            "/", 
            routes![
                routes::index,
                routes::login,
                routes::profile,
                routes::quote,
                routes::quote_history
            ]
        )
        .mount("/static", FileServer::from("./static"))
        .manage(database)
}