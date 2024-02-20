use rocket_dyn_templates::{context, Template};


#[get("/")]
pub fn index() -> Template {
    Template::render(
        "placeholder", 
        context!{
            
        }
    )
}

#[get("/login")]
pub fn login() -> Template {
    Template::render(
        "placeholder", 
        context!{
            
        }
    )
}

#[get("/register")]
pub fn register() -> Template {
    Template::render(
        "placeholder", 
        context!{
            
        }
    )
}

#[get("/page/profile")]
pub fn profile() -> Template {
    Template::render(
        "placeholder", 
        context!{
            
        }
    )
}

#[get("/page/quote")]
pub fn quote() -> Template {
    Template::render(
        "placeholder", 
        context!{
            
        }
    )
}
