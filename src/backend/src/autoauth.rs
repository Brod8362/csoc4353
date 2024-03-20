use rocket::{request::{self, FromRequest}, Request};

use crate::config::AppConfig;


pub struct UserContext(pub Option<String>);

///Request guard to check if user is authenticated or not, automatically
#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r UserContext {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // The closure passed to `local_cache` will be executed at most once per
        // request: the first time the `UserContext` guard is used. If it is
        // requested again, `local_cache` will return the same value.
        let appconf = request.rocket().state::<AppConfig>();
        if appconf.is_none() {
            panic!("can't get app config in auth request guard");
        }
        let user_id: Option<String> = crate::jwt::validate_from_cookie(request.cookies(), &appconf.unwrap().secret_as_bytes());
        request::Outcome::Success(request.local_cache(|| {
            UserContext(user_id)
        }))
    }
}