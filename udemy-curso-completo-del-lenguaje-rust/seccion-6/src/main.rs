// https://learnxinyminutes.com/docs/es-es/rust-es/
// use rand::random;

struct Usuario {
    nombre: String,
    email: String,
    nacimiento: i32,
    activo: bool,
    user_role: UserRole,
    website: Website,
}

enum UserRole {
    BASIC,
    ADMIN
}

enum Website {
    URL(String),
    INSTAGRAM(String),
    LINKEDIN(String),
    FACEBOOK(String),
}

fn main() {
    let number:u8 = rand::random();

    let mut usuario =  Usuario {
        nombre: "Eaf".to_string(), //una forma para vectorizar string
        email: String::from("eaf@eduardoaf.com"), //otra forma para vectorizar string
        nacimiento: 2000,
        activo: true,
        user_role: UserRole::BASIC,
        website: Website::INSTAGRAM(String::from("@eaf")),
    };

    //incluir metodos en las structs
    impl Usuario {
        fn edad(&self) -> i32 {
            let actual = 2022;
            actual - self.nacimiento            
        }
    }

    let edad = usuario.edad();
    let role: UserRole = UserRole::BASIC;

    println!(
        "Usuario {} nacimiento {} email {} and random number is: {} y su edad es: {}", 
        usuario.nombre, 
        usuario.nacimiento, 
        usuario.email,
        number,
        edad
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

    let has_access = has_role_access(usuario.user_role);


}

fn go_to_website(website:Website) {
    match website {
        Website::INSTAGRAM => ;//algo
        Website::FACEBOOK() => ;// abrelo en otra app
    }
}

fn has_role_access(user_role:UserRole) -> bool {
    match user_role {
        UserRole::ADMIN => true,
        UserRole::BASIC => false,
    }
}

fn get_nuevo_usuario(nombre:String, email:String) -> Usuario {
    return Usuario { 
        nombre: nombre,
        email: email,
        nacimiento: 1980,
        activo: true,
    };
}
