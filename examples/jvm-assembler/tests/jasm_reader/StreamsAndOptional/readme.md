# Stream and Optional Example

Demonstrates the bytecode implementation of the Java 8 Stream API and Optional, focusing on understanding functional programming in bytecode.

## File Descriptions

- `StreamsAndOptional.java` - Source code for Stream and Optional
- `StreamsAndOptional.class` - Compiled bytecode
- `StreamsAndOptional.jasm` - JASM assembly format

## Java Source Highlights

```java
// Stream Operations
List<String> names = Arrays.asList("Alice", "Bob", "Charlie");
List<String> result = names.stream()
    .filter(name -> name.length() > 3)
    .map(String::toUpperCase)
    .collect(Collectors.toList());

// Optional Usage
Optional<String> optional = Optional.of("test");
optional.ifPresent(value -> System.out.println(value));
```

## JASM Key Features

### Stream Chaining

```jasm
invokeinterface InterfaceMethod java/util/stream/Stream.filter:
    "(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;";
invokeinterface InterfaceMethod java/util/stream/Stream.map:
    "(Ljava/util/function/Function;)Ljava/util/stream/Stream;";
invokestatic Method java/util/stream/Collectors.toList:
    "()Ljava/util/Collector;";
```

### Lambdas and References

- Lambda expression in `filter` -> `invokedynamic`
- Method reference in `map` -> `invokedynamic`
- Lambda in `ifPresent` -> `invokedynamic`

### Optional Methods

```jasm
invokevirtual Method java/util/Optional.ifPresent:
    "(Ljava/util/function/Consumer;)V";
```

## Learning Points

1. Interface invocation mechanism of Streams
2. Bytecode implementation of chained operations
3. Internal structure of Optional
4. Lambda implementation of functional interfaces
5. Bytecode representation of Collectors