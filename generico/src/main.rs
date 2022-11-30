fn main() {
    // A fixed-size array
    let cuatro_enteros: [i32; 4] = [1, 2, 3, 4];

    // Un array dinámico (vector) guardado en HEAP
    let mut v_dinamico: Vec<i32> = vec![1, 2, 3, 4];
    v_dinamico.push(5);

    //immutable borrow occurs here
    let v_copy: &[i32] = &v_dinamico;

    //error: cannot borrow `v_dinamico` as mutable because it is also borrowed as immutable
    // mutable borrow occurs here
    //v_dinamico.push(13);

    // Usa `{:?}` para imprimir algo en estilo debug
    println!("{:?} {:?} {:?}", cuatro_enteros, v_dinamico, v_copy);

}
