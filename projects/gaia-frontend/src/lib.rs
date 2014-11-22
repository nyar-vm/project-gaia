mod assembler;
mod easy_test;
mod metadata;
mod utils;

wit_bindgen::generate!({
    world: "gaia-assembly",
    exports: {
        "nyar:gaia-assembly/assembler": assembler::AssemblerImpl,
        "nyar:gaia-assembly/metadata": metadata::MetadataImpl,
        "nyar:gaia-assembly/utils": utils::UtilsImpl,
        "nyar:gaia-assembly/easy-test": easy_test::EasyTestImpl,
    }
});
