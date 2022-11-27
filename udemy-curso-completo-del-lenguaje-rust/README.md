- [Curso completo rust - Julio Andrés](https://www.udemy.com/course/curso-completo-rust/)

- [contenido del curso](https://www.udemy.com/course/curso-completo-rust/learn/lecture/19834750#content)

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




