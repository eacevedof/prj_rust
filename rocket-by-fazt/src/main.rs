//esto trae todos los módulos disponibles 
#[macro_use] extern crate rocket;

//otra forma particular es importar uno a uno
//use rocket::routes

#[get("/")]
//función q devueve un string
fn index() -> &'static str {
    "<html><b>hola</b> 
    hello world!
    </html>
    "
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
//funcion q no devuelve nada
fn rocket() -> _{
    rocket::build().mount("/", routes![index, about, profile, create_profile])
}
