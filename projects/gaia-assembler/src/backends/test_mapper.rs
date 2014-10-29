//! Test module for FunctionMapper functionality

#[cfg(test)]
mod tests {
    use super::super::{FunctionMapper, TargetPlatform};

    #[test]
    fn test_builtin_print_mapping() {
        let mapper = FunctionMapper::new();

        // Test IL mapping
        assert_eq!(
            mapper.map_function("__builtin_print", TargetPlatform::IL),
            "void [mscorlib]System.Console::WriteLine(string)"
        );

        // Test JVM mapping
        assert_eq!(mapper.map_function("__builtin_print", TargetPlatform::JVM), "java.lang.System.out.println");

        // Test PE mapping
        assert_eq!(mapper.map_function("__builtin_print", TargetPlatform::PE), "printf");

        // Test WASI mapping
        assert_eq!(mapper.map_function("__builtin_print", TargetPlatform::WASI), "wasi_print");
    }

    #[test]
    fn test_memory_allocation_mapping() {
        let mapper = FunctionMapper::new();

        // Test malloc mapping for different platforms
        assert_eq!(mapper.map_function("malloc", TargetPlatform::IL), "System.Runtime.InteropServices.Marshal.AllocHGlobal");

        assert_eq!(mapper.map_function("malloc", TargetPlatform::JVM), "java.nio.ByteBuffer.allocateDirect");

        assert_eq!(mapper.map_function("malloc", TargetPlatform::PE), "HeapAlloc");

        assert_eq!(mapper.map_function("malloc", TargetPlatform::WASI), "malloc");
    }

    #[test]
    fn test_unmapped_function() {
        let mapper = FunctionMapper::new();

        // Test that unmapped functions return the original name
        assert_eq!(mapper.map_function("custom_function", TargetPlatform::IL), "custom_function");

        assert_eq!(mapper.map_function("user_defined_func", TargetPlatform::JVM), "user_defined_func");
    }

    #[test]
    fn test_additional_builtin_functions() {
        let mapper = FunctionMapper::new();

        // Test __builtin_println mapping
        assert_eq!(mapper.map_function("__builtin_println", TargetPlatform::PE), "puts");

        // Test __builtin_read mapping
        assert_eq!(mapper.map_function("__builtin_read", TargetPlatform::JVM), "java.util.Scanner.nextLine");
    }
}
