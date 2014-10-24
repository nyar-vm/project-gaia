wit_bindgen::generate!({
    world: "gaia-assembly",
    exports: {
        "assembler": AssemblerImpl,
        "metadata": MetadataImpl,
        "utils": UtilsImpl,
        "easy-test": EasyTestImpl,
    },
});

mod assembler;
mod easy_test;
mod metadata;
mod utils;

use exports::{
    assembler::Guest as AssemblerGuest, easy_test::Guest as EasyTestGuest, metadata::Guest as MetadataGuest,
    utils::Guest as UtilsGuest,
};

struct AssemblerImpl;
struct MetadataImpl;
struct UtilsImpl;
struct EasyTestImpl;

impl AssemblerGuest for AssemblerImpl {
    fn assemble(
        descriptor: exports::types::GaiaDescriptor,
        config: exports::types::AssembleConfig,
    ) -> exports::types::AssemblyResult {
        assembler::assemble(descriptor, config)
    }

    fn validate_syntax(descriptor: exports::types::GaiaDescriptor) -> Vec<exports::types::Diagnostic> {
        assembler::validate_syntax(descriptor)
    }

    fn disassemble(bytecode: Vec<u8>, config: exports::types::DisassembleConfig) -> exports::types::DisassembleResult {
        assembler::disassemble(bytecode, config)
    }
}

impl MetadataGuest for MetadataImpl {
    fn get_program_metadata(descriptor: exports::types::GaiaDescriptor) -> exports::types::ProgramMetadata {
        metadata::get_program_metadata(descriptor)
    }

    fn get_symbols(descriptor: exports::types::GaiaDescriptor) -> Vec<exports::types::SymbolInfo> {
        metadata::get_symbols(descriptor)
    }

    fn get_instruction_docs(instruction: exports::types::Instruction) -> Option<exports::types::InstructionMetadata> {
        metadata::get_instruction_docs(instruction)
    }

    fn get_platform_info(target: exports::types::Target) -> exports::types::PlatformInfo {
        metadata::get_platform_info(target)
    }

    fn analyze_complexity(descriptor: exports::types::GaiaDescriptor) -> exports::types::ComplexityAnalysis {
        metadata::analyze_complexity(descriptor)
    }

    fn get_dependencies(descriptor: exports::types::GaiaDescriptor) -> exports::types::DependencyGraph {
        metadata::get_dependencies(descriptor)
    }
}

impl UtilsGuest for UtilsImpl {
    fn get_version() -> String {
        utils::get_version()
    }

    fn format_descriptor(
        descriptor: exports::types::GaiaDescriptor,
        options: exports::types::FormatOptions,
    ) -> exports::types::GaiaDescriptor {
        utils::format_descriptor(descriptor, options)
    }

    fn get_completions(
        descriptor: exports::types::GaiaDescriptor,
        position: exports::types::SourceLocation,
        target: exports::types::Target,
    ) -> Vec<exports::types::CompletionItem> {
        utils::get_completions(descriptor, position, target)
    }

    fn optimize_descriptor(
        descriptor: exports::types::GaiaDescriptor,
        target: exports::types::Target,
        options: exports::types::OptimizationOptions,
    ) -> exports::types::GaiaDescriptor {
        utils::optimize_descriptor(descriptor, target, options)
    }

    fn validate_consistency(descriptor: exports::types::GaiaDescriptor) -> exports::types::ValidationResult {
        utils::validate_consistency(descriptor)
    }

    fn convert_format(
        descriptor: exports::types::GaiaDescriptor,
        target_format: exports::types::DescriptorFormat,
    ) -> exports::types::GaiaDescriptor {
        utils::convert_format(descriptor, target_format)
    }
}

impl EasyTestGuest for EasyTestImpl {
    fn generate_hello_world(
        target: exports::types::Target,
        options: exports::types::TestGenerationOptions,
    ) -> exports::types::GaiaDescriptor {
        easy_test::generate_hello_world(target, options)
    }

    fn generate_arithmetic_test(
        target: exports::types::Target,
        options: exports::types::TestGenerationOptions,
    ) -> exports::types::GaiaDescriptor {
        easy_test::generate_arithmetic_test(target, options)
    }

    fn generate_memory_test(
        target: exports::types::Target,
        options: exports::types::TestGenerationOptions,
    ) -> exports::types::GaiaDescriptor {
        easy_test::generate_memory_test(target, options)
    }

    fn generate_control_flow_test(
        target: exports::types::Target,
        options: exports::types::TestGenerationOptions,
    ) -> exports::types::GaiaDescriptor {
        easy_test::generate_control_flow_test(target, options)
    }

    fn generate_function_call_test(
        target: exports::types::Target,
        options: exports::types::TestGenerationOptions,
    ) -> exports::types::GaiaDescriptor {
        easy_test::generate_function_call_test(target, options)
    }

    fn validate_test_program(
        descriptor: exports::types::GaiaDescriptor,
        target: exports::types::Target,
    ) -> exports::types::TestValidationResult {
        easy_test::validate_test_program(descriptor, target)
    }
}
