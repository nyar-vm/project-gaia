# Class File Format

Reader and writer for the Java class file format, supporting the parsing and generation of standard JVM class file structures.

## Overview

The Class file format is the standard binary format executed by the Java Virtual Machine. This module provides:

- **Class File Reading**: Parsing class file structures from binary data.
- **Class File Writing**: Serializing class file structures into binary data.
- **Full Support**: Supports all standard class file components.
- **Validation Mechanism**: Built-in format validation and error checking.
- **Performance Optimization**: Efficient handling of large class files.

## Class File Structure

Java class files follow a strict binary format.

## Supported Versions

| Java Version | Major Version | Minor Version | Support Status |
|--------------|---------------|---------------|----------------|
| Java 1.1     | 45            | 3             | ✅ Supported   |
| Java 1.2     | 46            | 0             | ✅ Supported   |
| Java 1.3     | 47            | 0             | ✅ Supported   |
| Java 1.4     | 48            | 0             | ✅ Supported   |
| Java 5       | 49            | 0             | ✅ Supported   |
| Java 6       | 50            | 0             | ✅ Supported   |
| Java 7       | 51            | 0             | ✅ Supported   |
| Java 8       | 52            | 0             | ✅ Supported   |
| Java 9       | 53            | 0             | ✅ Supported   |
| Java 10      | 54            | 0             | ✅ Supported   |
| Java 11      | 55            | 0             | ✅ Supported   |
| Java 12      | 56            | 0             | ✅ Supported   |
| Java 13      | 57            | 0             | ✅ Supported   |
| Java 14      | 58            | 0             | ✅ Supported   |
| Java 15      | 59            | 0             | ✅ Supported   |
| Java 16      | 60            | 0             | ✅ Supported   |
| Java 17      | 61            | 0             | ✅ Supported   |
| Java 18      | 62            | 0             | ✅ Supported   |
| Java 19      | 63            | 0             | ✅ Supported   |
| Java 20      | 64            | 0             | ✅ Supported   |
| Java 21      | 65            | 0             | ✅ Supported   |

## Constant Pool Types

Supports all standard constant pool entry types:

### Basic Types
- **CONSTANT_Utf8**: UTF-8 encoded string
- **CONSTANT_Integer**: 32-bit integer
- **CONSTANT_Float**: 32-bit floating point
- **CONSTANT_Long**: 64-bit long integer
- **CONSTANT_Double**: 64-bit double precision floating point

### Reference Types
- **CONSTANT_Class**: Symbolic reference to a class or interface
- **CONSTANT_String**: Symbolic reference to a string object
- **CONSTANT_Fieldref**: Symbolic reference to a field
- **CONSTANT_Methodref**: Symbolic reference to a class method
- **CONSTANT_InterfaceMethodref**: Symbolic reference to an interface method
- **CONSTANT_NameAndType**: Name and type descriptor for a field or method

### Dynamic Types
- **CONSTANT_MethodHandle**: Method handle
- **CONSTANT_MethodType**: Method type
- **CONSTANT_Dynamic**: Dynamically-computed constant
- **CONSTANT_InvokeDynamic**: Dynamic method invocation

## Attribute Support

### Standard Attributes
- **Code**: Bytecode and exception table of a method
- **ConstantValue**: Constant value of a constant field
- **SourceFile**: Source file name
- **LineNumberTable**: Mapping from bytecode to source code line numbers
- **LocalVariableTable**: Debug information for local variables
- **LocalVariableTypeTable**: Debug information for generic local variables

### Advanced Attributes
- **StackMapTable**: Stack map frames (used for verification)
- **Exceptions**: Exception types thrown by a method
- **InnerClasses**: Inner class information
- **EnclosingMethod**: Enclosing method for anonymous classes
- **Signature**: Generic signature information
- **RuntimeVisibleAnnotations**: Annotations visible at runtime
- **RuntimeInvisibleAnnotations**: Annotations invisible at runtime

## Usage Examples

### Reading a Class File

(Examples to be added)

### Writing a Class File

(Examples to be added)

## Error Handling

Potential errors during class file reading and writing:

### Reading Errors
- **Magic Number Error**: Invalid class file magic number.
- **Version Not Supported**: Unsupported Java version.
- **Format Error**: Corrupted class file format.
- **Constant Pool Error**: Invalid constant pool reference.
- **Validation Error**: Bytecode verification failed.

### Writing Errors
- **Data Too Large**: Exceeds class file format limits.
- **Reference Error**: Invalid cross-reference.
- **Encoding Error**: String encoding issues.

All errors are returned via the [`GaiaError`](../../gaia_types/struct.GaiaError.html) type, providing detailed error information.

## Performance Characteristics

- **Memory Efficiency**: Streaming processing for large class files.
- **Fast Parsing**: Optimized binary parsing algorithms.
- **Zero-Copy**: Minimizes memory allocation wherever possible.
- **Cache-Friendly**: Optimized data structure design.

## Related Modules

- [`reader`](reader/index.html) - Class file reader
- [`writer`](writer/index.html) - Class file writer
- [`program`](../../program/index.html) - High-level abstraction of JVM programs
