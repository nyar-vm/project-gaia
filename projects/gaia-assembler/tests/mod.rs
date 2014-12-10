use gaia_assembler::assembler::GaiaAssembler;

mod adapter_tests;
mod backend_tests;
mod compiler_tests;
mod config_tests;
mod integration_pe;
mod runnable;

#[test]
fn test_compiler_creation() {
    let compiler = GaiaAssembler::new();
    let expected = if cfg!(feature = "clr") { 7 } else { 6 };
    assert_eq!(compiler.backends().len(), expected);
}
