
#[get("/")]
pub fn index() -> String {
    String::from("hi")
}

#[get("/login")]
pub fn login() -> String {
    String::from("login")
}

#[get("/register")]
pub fn register() -> String {
    String::from("register")
}

#[get("/page/profile")]
pub fn profile() -> String {
    String::from("profile page")
}

#[get("/page/quote")]
pub fn quote() -> String {
    String::from("quote page")
}
