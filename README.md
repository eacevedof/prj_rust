# prj_rust
Pruebas rust-lang.org

```rust
//comando para iniciar un paquete como binario (flag: --bin, por defecto) si se desea crear como libreria hay que usar --lib
cargo new <nombre-paquete>
```

#### .gitignore sugerido
```yml
# https://github.com/github/gitignore/blob/main/Rust.gitignore
.vscode
.idea

# Generated by Cargo
# will have compiled files and executables
debug/
target/

# Remove Cargo.lock from gitignore if creating an executable, leave it for libraries
# More information here https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
Cargo.lock

# These are backup files generated by rustfmt
**/*.rs.bk

# MSVC Windows builds of rustc generate these, which store debugging information
*.pdb
```
![](images/after-first-compile-with-cargo-run.png)