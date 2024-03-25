use std::time::Duration;

use rocket::http::CookieJar;
use rocket::http::Cookie;
use rocket::response::Redirect;
use rocket::form::Form;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use sqlx::Pool;
use sqlx::Sqlite;

use crate::autoauth::UserContext;
use crate::config::AppConfig;
use crate::database;


#[get("/")]
pub fn index(ctx: &UserContext) -> Template {
    if let Some(user_id) = &ctx.0 {
        Template::render(
            "home", 
            context!{
                user: user_id
            }
        )
    } else {
        Template::render(
            "home", 
            context!{}
        )
    }

}

#[get("/login")]
pub fn login_page() -> Template {
    Template::render(
        "login_page", 
        context!{
        }
    )
}

#[derive(FromForm)]
pub struct AuthRequest {
    username: String,
    password: String
}

#[post("/login", data = "<form>")]
pub async fn login_request(pool: &State<Pool<Sqlite>>, cookies: &CookieJar<'_>, appconf: &State<AppConfig>, form: Form<AuthRequest>) -> Template {
    let auth_result = database::authenticate_user(pool, &form.username, &form.password).await;
    if auth_result.is_err() {
        let error_message = format!("{:?}", auth_result.err().unwrap());
        return Template::render(
            "login_page",
            context!{
                error: error_message
            }
        )
    }
    //auth must have been successful at this point
    //generate jwt and send to client
    let duration = Duration::from_secs(60*60*24); //tokens good for 24 hours
    let secret = &appconf.secret_as_bytes(); //TODO
    let token = crate::jwt::generate_jwt(&form.username, duration, secret);
    cookies.add(("Authorization", token));

    //TODO this is temporary and should be made better.
    Template::render(
        "login_page",
        context!{
            message: "OK"
        }
    )
}

#[post("/register", data ="<form>")]
pub async fn register_request(pool: &State<Pool<Sqlite>>, form: Form<AuthRequest>) -> Template {
    let register_request = database::register_user(pool, &form.username, &form.password).await;
    if let Ok(_) = register_request {
        Template::render(
            "login_page",
            context!{
                error: "Registered, now please log in."
            }
        )
    } else {
        let error_message = format!("{:?}", register_request.err().unwrap());
        Template::render(
            "login_page",
            context!{
                error: error_message
            }
        )
    }
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::from("Authorization"));  //"from" might not work here, may need to use "named"
    Redirect::to(uri!("/"))
}

#[get("/page/profile")]
pub fn profile() -> Template {
    Template::render(
        "profile_management", 
        context!{
            
        }
    )
}

#[get("/page/quote")]
pub fn quote() -> Template {
    Template::render(
        "fuel_quote_form", 
        context!{
            
        }
    )
}

#[derive(FromForm)]
pub struct QuoteData {
    gallons_requested: String,
    address: String,
    delivery_date: String,
}

#[post("/page/quote", data="<form>")]
pub async fn quote_submit(pool: &State<Pool<Sqlite>>, form: Form<QuoteData>) -> Template {
    //TODO: handle quote storage inside of the database.rs file
    //let quote_request = database::store_quote(pool, &form.gallons_requested, &form.address, &form.delivery_date).await;

    //TODO: error handling
    /*
    if quote_request.is_err(){
        
    }
    */

    //TODO: return submission confirmation upon OK
    //return String::from("<p> Fuel Quote Submitted </p>")

    //renders form results FOR NOW until database implement
    let form_inp = form.into_inner();
    Template:: render(
        "fuel_quote",
        context!{
            gallons_requested: form_inp.gallons_requested,
            address: form_inp.address,
            delivery_date: form_inp.delivery_date,
        }
    )
}


#[get("/page/quote/<id>")]
pub fn quote_id(id: &str) -> Template {
    Template:: render(
        "fuel_quote",
        context!{
            curr_id: id
        }
    )
}

#[get("/page/quote/history")]
pub fn quote_history() -> Template {
    Template::render(
        "fuel_quote_history", 
        context!{
            
        }
    )
}

#[cfg(test)]
mod tests {
    use rocket::{http::{Cookie, CookieJar}, tokio, State};
    use rocket::local::blocking::Client;
    use rocket::http::Status;

    use crate::rocket;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_user_creation() {
        let pool = crate::database::init_connection("sqlite://:memory:", 1).await.unwrap();
        let form = crate::routes::AuthRequest {
            username: "test".to_string(),
            password: "test".to_string()
        };
        let state = State::from(&pool);
        //create the user via the request method
        let _ = crate::routes::register_request(state, form.into()).await;

        // try logging in as the user and make sure it works
        let r = crate::database::authenticate_user(&pool, "test", "test").await;
        assert!(r.is_ok())
    }

    #[tokio::test]
    async fn test_user_auth() {
        let pool = crate::database::init_connection("sqlite://:memory:", 1).await.unwrap();
        let form = crate::routes::AuthRequest {
            username: "test".to_string(),
            password: "test".to_string()
        };
        let state = State::from(&pool);
        let _ = crate::routes::register_request(state, form.into()).await;

        let r = crate::database::register_user(&pool, "test", "test").await;
        assert!(r.is_ok());

        //authenticate via form
        let appconf = AppConfig {
            database_uri: "sqlite://:memory:".to_owned(),
            max_connections: 1,
            jwt_secret: "test".to_owned()
        };
        let form = crate::routes::AuthRequest {
            username: "test".to_owned(),
            password: "test".to_owned()
        };
        //TODO: need to figure out how to instantiate a CookieJar from scratch and call the correct request method
        todo!()
    }

    #[tokio::test]
    async fn test_index() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}