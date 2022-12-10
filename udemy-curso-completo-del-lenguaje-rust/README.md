- [Curso completo rust - Julio Andrés](https://www.udemy.com/course/curso-completo-rust/)
- [Rust in minutes](https://learnxinyminutes.com/docs/es-es/rust-es/)
- [contenido del curso](https://www.udemy.com/course/curso-completo-rust/learn/lecture/19834750#content)
- [crates.io](https://crates.io)
	- similar a packagist. Repositorio de paquetes de rust

#### Sección 4
- cargo es el composer o pip de rust
```sh
# crea un proyecto "nombre-del-proyecto"
cargo new <nombre-del-proyecto>
# en src hay un main.rs
# opcion
cargo new <nombre-de-mi-lib> --lib
# en src hay un lib.rs
```
- [connfiguración de Cargo.toml](https://doc.rust-lang.org/cargo/reference/manifest.html)
- En rust los paquetes o dependencias se llaman **crates**
	- los crates son las cajas de madera que antes se usaban para el envío de mercancia por barco.

- cuando compilamos en la ruta `<project>/target/debug` se genera el "ejecutable"

- `cargo check` comprueba sin compilar
- `cargo build --relese` no incluye dependencias de desarrollo
- [crates.io](https://crates.io)
	- equivaletne a packageist (https://packagist.org/)
	- instalando crate de random. **rand**
	- buscamos el paquete y en Cargo.toml en la sección dependencies
	```
	[dependencies]
	rand = "0.8.4"
	```
	- despues en el archivo **main.rs**
	```rust
	//paquete::función
	use rand::random;
	
	...
	let number:u8 = random();
	...
	```
- version de cargo:
	- `cargo --version`
	- desde la version 1.62.0 se puede agregar las dependencias a **Cargo.toml** usando:
	- `cargo add <nombre-crate>`
	- Ejemplo: `cargo add log`
	- Por ejemplo **serde** es una de las librerias más conocidas para serilizar y deserealizar JSON. Esta tiene varios **features** así que puedo añadir el *crate* solo con unas features.
	- Ejemplo: `cargo add serde --features derive`

#### Seccion 5 selección del IDE
- JetBrains (con la versio community es suficiente) 
- Vscode. Extension *rust-analyzer, Better TOML, crates*
	- debugging *CodeLLDB* sirve para conectarse al codigo que se está ejecutando
	- Error Lens
- Vim
- Estado de los IDES para Rust: [Are we IDE yet](https://areweideyet.com/)
- **vscode launch.json**
	```json
	{
		// Use IntelliSense to learn about possible attributes.
		// Hover to view descriptions of existing attributes.
		// For more information, visit: https://go.microsoft.com/fwlink/?linkid=830387
		"version": "0.2.0",
		"configurations": [{
				"name": "(Windows) Launch",
				"type": "cppvsdbg",
				"request": "launch",
				"program": "${workspaceRoot}/target/debug/hello_cargo.exe",
				"args": [],
				"stopAtEntry": false,
				"cwd": "${workspaceRoot}",
				"environment": [],
				"externalConsole": true
			},
			{
				"type": "lldb",
				"request": "launch",
				"name": "Mac OSX Debug executable 'hello_cargo'",
				"cargo": {
					"args": [
						"build",
						"--bin=hello_cargo",
						"--package=hello_cargo"
					],
					"filter": {
						"name": "hello_cargo",
						"kind": "bin"
					}
				},
				"args": [],
				"cwd": "${workspaceFolder}"
			},
			{
				"type": "lldb",
				"request": "launch",
				"name": "Mac OSX Debug unit tests in executable 'hello_cargo'",
				"cargo": {
					"args": [
						"test",
						"--no-run",
						"--bin=hello_cargo",
						"--package=hello_cargo"
					],
					"filter": {
						"name": "hello_cargo",
						"kind": "bin"
					}
				},
				"args": [],
				"cwd": "${workspaceFolder}"
			}
		]
	}
	```
- tipos de variables
```rust
l= 98_222;
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

let str_literal: &str = "eduardoaf.com";
// es un string muy gordo que se trata como objeto y por lo tanto no se entiende como
// un tipo primitivo (los primitivos se guardan en STACK lifo)  así que se almacena en la memoria HEAP 
let str_heap: String = "eduardoaf.com en heap".to_string();
// str_slice apunta al mismo espacio en memoria al que apunta str_heap
let str_slice: &str = &srt_heap;


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
let mut dinamico: Vec<i32> = vec![1,2,3,4];
vector.push(5);

//copia de dirección de memoria
let vec_copy: &[i32] = &vector;
//print similar a un debug
println!("{:?} {:?}", dinamico, vec_copy); 

//strings, hay 2 tipos
let nombre = "Eduardoaf.com";
//string static es estatico pq se guarda en el binario final. Abarca el concepto de borrowing
let nombre2: &'static str = "eduardoaf.com";

//este tipo string es un array de unsigned a, permite que la memoria crezca y se aloja en la memoria heap
let apellido: String = "Julio".to_string();

let domicilio = String::new();
domicilio = "mi casa".to_string();
```
- funciones
```rust
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
  //* indica que se obtendrá el valor de la dirección nro para sumarle 4 y despues retornarlo
	*nro + 4
}
`





``
