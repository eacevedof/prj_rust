//#[allow(dead_code)] //esto es para que no me alerte por codigo no usuado. ignoralo

// https://learnxinyminutes.com/docs/es-es/rust-es/
// https://www.udemy.com/course/curso-completo-rust/learn/lecture/27199686#content

/*
//el Null de rust
enum Option<T> {
    Some(T),
    None,
}
*/

fn main() {
    //let nombre: Option<String> = Some("eduardoaf.com".to_string());
    let nombre: Option<String> = None;
    
    //esta estructura me permite controlar todas las opciones que pueda tener
    //si es null hago algo si no otra cosa
    match nombre {
        None => println!("nombre is none"),
        Some(nombre) => println!("El nombre es: {}", nombre),
    }

}