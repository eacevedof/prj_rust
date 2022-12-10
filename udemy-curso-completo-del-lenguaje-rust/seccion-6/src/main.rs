// https://learnxinyminutes.com/docs/es-es/rust-es/
// use rand::random;

struct Usuario {
    nombre: String,
    email: String,
    edad: i32,
    activo: bool
}

fn main() {
    let number:u8 = rand::random();

    let mut usuario =  Usuario {
        nombre: "Eaf".to_string(), //una forma para vectorizar string
        email: String::from("eaf@eduardoaf.com"), //otra forma para vectorizar string
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

    //shorthand init
    let usuario2: Usuario = get_nuevo_usuario(
        String::from("Eduardo"), 
        String::from("info@eduardoaf.com")
    );

    let usuario3: Usuario = Usuario {
        nombre: "Juan".to_string(),
        email: "otro@email.com".to_string(),
        ..usuario2
    };

    //touple structs
    struct Point(i32, i32, i32);
    

}

fn get_nuevo_usuario(nombre:String, email:String) -> Usuario {
    return Usuario { 
        nombre: nombre,
        email: email,
        edad: 100,
        activo: true,
    };
}