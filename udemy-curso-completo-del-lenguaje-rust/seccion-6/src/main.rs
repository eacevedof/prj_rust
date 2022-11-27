fn main() {
    mostrar_bienvenida();
    seleccion_numero(8);
    let nro = get_numero(28);
    let nro_by_ref: i32 = get_numero_by_ref(&99);
    println!("El numero por valor es {} y por ref {}", nro, nro_by_ref);
    
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

fn get_numero_by_ref(nro: &i32) -> i32 {
    //el * indica que se actualice la variable (por referencia) con nro + 4
    *nro + 4
}