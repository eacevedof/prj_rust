fn main() {
    println!("eduardoaf.com - variables en rust");
    //enteros
    let entero: i8 = 23;
    let entero2: u8 = 40;
    let entero3: i8 = -40;

    //enteros en alfanumerico
    let decimal= 98_222;
    let hex = 0xff;
    let octal = 0o77;
    let binary = 0b1111_0000;

    //punto flotante
    let float1 = 6.0;
    let float32: f32 = 12.432;

    //booleano
    let verdadero = true;
    let falso: bool = false;

    //caracter con simple comilla
    let caracter = 'a';
    // https://emojipedia.org/
    let emoji = "🇵🇪";

    // Tipos compuestos:

    //Tuplas
    let tupla = ('h', 23, -45, 0.222);
    let tupla2: (char, u64, i32, f64) = ('h', 23, -45, 0.222);

    let (x, y, z, w) = tupla;

    println!("el primer valor de la tupla {}", tupla.1);

    //arreglos
    let arreglo = [1, 2, 3, 4, 5];
    print!("el segundo valor del arreglo es {}", arreglo[1]);

    //[tipo; longitud]
    let arreglo2: [i128; 5] = [1, 2, 3, 4, 5];

    //strings, hay 2 tipos
    let nombre = "Eduardoaf.com";
    //string static es estatico pq se guarda en el binario final. Abarca el concepto de borrowing
    //
    let nombre2: &'static str = "eduardoaf.com";

    //este tipo string es un array de unsigned a, permite que la memoria crezca y se aloja en la memoria heap
    let apellido: String = "Julio".to_string();

    let domicilio = String::new();
    let domicilio = "mi casa".to_string();

}
