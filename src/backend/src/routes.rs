use rocket_dyn_templates::{context, Template};


#[get("/")]
pub fn index() -> Template {
    Template::render(
        "home", 
        context!{
            
        }
    )
}

#[get("/login")]
pub fn login() -> Template {
    Template::render(
        "login_page", 
        context!{
            
        }
    )
}

#[get("/register")]
pub fn register() -> Template {
    Template::render(
        "login_page", 
        context!{
            
        }
    )
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

#[get("/page/quote_history")]
pub fn quote_history() -> Template {
    Template::render(
        "fuel_quote_history", 
        context!{
            
        }
    )
}

