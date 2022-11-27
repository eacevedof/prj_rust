fn main() {
    mostrar_bienvenida();
    seleccion_numero(8);
    let nro = get_numero(28);
    println!("El numero final es {}", nro);
    
    let exp_statemnt = {
        10
    };
    println!("El numero final dos es: {}", exp_statemnt);
    // para evitar convertirlo al vector String usando to_string()
    // se puede definir la variable nombre como &str (string por referencia)
    //saludar_con_nombre("Edualdo".to_string());
    saludar_con_nombre("Edualdo");

}

//fn saludar_con_nombre(nombre: String) {
fn saludar_con_nombre(nombre: &str) {
    println!("hola {}",nombre);
}

fn mostrar_bienvenida() {
    println!("Bienvenidos a rust");
}

fn seleccion_numero(nro: i32) {
    println!("Tu numero es {}", nro);
}

fn get_numero(nro: i32) -> i32 {
    8
}
