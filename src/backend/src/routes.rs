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
    //TODO: return submission confirmation upon OK
    //return String::from("<p> Fuel Quote Submitted </p>")

    //renders form results FOR NOW until database implement
    //let form_inp = form.into_inner();

    Template:: render(
        "fuel_quote",
        context!{
            gallons_requested: &form.gallons_requested,
            address: &form.address,
            delivery_date: &form.delivery_date
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
    use crate::{rocket, routes::AuthRequest};
    use rocket::{form::Form, http::{ContentType, Status}, local::asynchronous::Client, tokio, State};

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
        //register and log in
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.post(uri!("/register")).dispatch().await;
        //this should throw an error as the request did not specify any credentials
        assert!(response.status() != Status::Ok);
        let mut request = client.post(uri!("/register"));
        request = request.header(ContentType::Form);
        request.set_body(r#"username=test&password=password"#);
        
        let response = request.dispatch().await;
        assert!(response.status() == Status::Ok);
        
        // now, try logging in
        request = client.post(uri!("/login"));
        request = request.header(ContentType::Form);
        request.set_body(r#"username=test&password=password"#);
        
        let response = request.dispatch().await;
        assert!(response.status() == Status::Ok);
        //assert a JWT is present and valid
        match client.cookies().get("Authorization") {
            Some(_jwt) => {} //OK, cookie is present,
            None => panic!("Authorization header not present")
        }
    }

    #[tokio::test]
    async fn test_form_submit(){
        //test that info can be submitted
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let mut submit = client.post(uri!("/page/quote"));
        submit = submit.header(ContentType::Form);
        submit.set_body(r#"gallons_requested=10&address=address1&delivery_date=2024-03-26"#);
        let response = submit.dispatch().await;
        assert!(response.status() == Status::Ok);
    }

    #[tokio::test]
    async fn test_submit_error(){
        //test that no info throws error
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let mut submit = client.post(uri!("/page/quote"));
        submit = submit.header(ContentType::Form);
        let response = submit.dispatch().await;
        assert!(response.status() != Status::Ok);
    }

    #[tokio::test]
    async fn test_form_data(){
        //test that the info can be submitted
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.post(uri!("/page/quote")).dispatch().await;
    }

    #[tokio::test]
    async fn test_quote_id() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/page/quote/1").dispatch().await;
        assert!(response.status() == Status::Ok);
        let body = response.into_string().await.unwrap();
        assert!(body.contains("<p> Current ID: 1 </p>"));

        let response = client.get("/page/quote/2").dispatch().await;
        assert!(response.status() == Status::Ok);
        let body = response.into_string().await.unwrap();
        assert!(body.contains("<p> Current ID: 2 </p>"));        
    }

    #[tokio::test] 
    async fn test_quote_history() {
        // check quote form when user owns no quotes (unable to test right now because of dummy data)

        // check when quote is created
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/page/quote/history").dispatch().await;
        assert!(response.status() == Status::Ok);
        let body = response.into_string().await.unwrap();
        assert!(body.contains("<a hx-get=\"/page/quote\" hx-target=\"#quote-content\">[+] New Quote</a>"));
        
        // make sure new quote appears in history (unable to test rn)

        // check when another quote is created and both quotes are visible

        // check current "history"        
        assert!(body.contains("<a hx-get=\"/page/quote/1\" hx-target=\"#quote-content\">Quote 1</a>"));

        assert!(body.contains("<a hx-get=\"/page/quote/2\" hx-target=\"#quote-content\">Quote 2</a>"));
    }
}