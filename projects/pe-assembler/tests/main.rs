mod runnable;
mod test_tools;
#[cfg(target_os = "windows")]
mod windows;

#[test]
fn ready() {
    println!("PE Assembler test suite ready!");
}
