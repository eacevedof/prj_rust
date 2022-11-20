//esto trae todos los módulos disponibles 
#[macro_use] extern crate rocket;
extern crate modules::eduardo;

use rocket_dyn_templates::{Template, context};

//otra forma particular es importar uno a uno
//use rocket::routes

#[get("/")]
//función q devueve un string
fn index() -> Template {
    Template::render("index", context! {
        title: "Rockert Overview"
    })
}

#[get("/about")]
fn about() -> &'static str {
    "about"    
}

#[get("/profile")]
fn profile() -> &'static str {
    "profile"
}

#[post("/profile")]
fn create_profile() -> &'static str {
    "create profile"
}

#[put("/profile")]
fn update_profile() -> &'static str {
    "update profile"
}

#[delete("/profile")]
fn delete_profile() -> &'static str {
    "delete profile"
}

#[launch]
//funcion q no devuelve nada. Arranca el servidor
fn rocket() -> _{
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, about])
        .mount("/profile", routes![profile, create_profile, update_profile, delete_profile])
}
