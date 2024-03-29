use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[macro_use] extern crate rocket;
pub mod routes;
pub mod database;
pub mod config;
pub mod autoauth;
pub mod jwt;


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
                routes::login_page,
                routes::login_request,
                routes::register_request,
                routes::logout,
                routes::profile,
                routes::profile_submit,
                routes::quote,
                routes::quote_id,
                routes::quote_submit,
                routes::quote_history
            ]
        )
        .mount("/static", FileServer::from("./static"))
        .manage(database)
        .manage(app_config)
}