mod test_tools;
mod wasm_read;
mod wasm_write;
#[cfg(feature = "wat")]
mod wat_read;
#[cfg(feature = "wat")]
mod wat_write;

#[test]
fn ready() {
    println!("it works!")
}
