use gaia_assembler::assembler::GaiaAssembler;

mod adapter_tests;
mod backend_tests;
mod compiler_tests;
mod config_tests;
mod runnable;

#[test]
fn test_compiler_creation() {
    let compiler = GaiaAssembler::new();
    assert_eq!(compiler.backends().len(), 4);
}
