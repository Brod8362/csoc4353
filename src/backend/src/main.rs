use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[macro_use] extern crate rocket;
pub mod routes;


#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount(
            "/", 
            routes![
                routes::index,
                routes::login,
                routes::profile,
                routes::quote
            ]
        )
        .mount("/static", FileServer::from("./static"))
}