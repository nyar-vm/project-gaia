#![warn(missing_docs)]

pub struct MslGenerator;

impl MslGenerator {
    pub fn name(&self) -> &'static str {
        "msl"
    }

    pub fn generate(&self, _program: &serde_json::Value) -> gaia_types::Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut files = std::collections::HashMap::new();

        let mut module_source = self.msl_header();
        module_source.push_str("\n");

        files.insert("module.metal".to_string(), Vec::from(module_source.as_bytes()));
        // Simulate metallib generation
        files.insert("default.metallib".to_string(), vec![0x4d, 0x54, 0x4c, 0x42]); // MTLB magic

        Ok(files)
    }

    fn msl_header(&self) -> String {
        "#include <metal_stdlib>\nusing namespace metal;".to_string()
    }
}
