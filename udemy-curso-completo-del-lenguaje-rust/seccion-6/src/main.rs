// https://learnxinyminutes.com/docs/es-es/rust-es/
//structs
use rand::random;

struct Usuario {
    nombre: String,
    email: String,
    edad: i32,
    activo: bool
}

fn main() {
    let number:u8 = random();

    let mut usuario =  Usuario {
        nombre: "Eaf".to_string(), //una forma para vectorizar string
        email: String::from("eaf@eaf.com"), //otra forma para vectorizar string
        edad: 99,
        activo: true,
    };
    println!(
        "Usuario {} edad {} email {} and random number is: {}", 
        usuario.nombre, 
        usuario.edad, 
        usuario.email,
        number
    );
    usuario.activo = false;
}
