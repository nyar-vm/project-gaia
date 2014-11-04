use gaia_assembler::GaiaCompiler;

#[test]
fn test_compiler_creation() {
    let compiler = GaiaCompiler::new();
    assert_eq!(compiler.backends().len(), 4);
}
