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

#[launch]
//funcion q no devuelve nada
fn rocket() -> _{
    rocket::build().mount("/", routes![index, about, profile])
}
