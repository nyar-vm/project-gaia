# Pattern Matching Example

Demonstrates the bytecode implementation of Java pattern matching (instanceof and switch expressions).

## File Descriptions

- `PatternMatching.java` - Source code for pattern matching
- `PatternMatching.class` - Compiled bytecode
- `PatternMatching.jasm` - JASM assembly format
- `PatternMatching$*.class` - Bytecode for inner classes (record classes)

## Java Source Highlights

```java
// instanceof Pattern Matching
if (obj instanceof String s) {
    return "String with length: " + s.length();
}

// switch expression Pattern Matching
return switch (shape) {
    case Circle(var radius) -> "Circle with radius: " + radius;
    case Rectangle(var width, var height) -> "Rectangle: " + width + "x" + height;
    default -> "Unknown shape";
};
```

## JASM Key Features

### instanceof Pattern Matching

```jasm
aload_1                    // obj
instanceof String;         // Type check
ifeq L19;                  // Jump if not String
aload_1;                   // obj
checkcast String;          // Cast to String
astore_2;                  // Store to local variable s (compiler generated)
```

### Switch Expressions

```jasm
aload_1                    // shape
invokevirtual Method PatternMatching$Shape.getClass:()Ljava/lang/Class;
// ... Pattern matching logic
```

### Record Classes (Inner Classes)

```jasm
class PatternMatching$1Point
class PatternMatching$1Line
class PatternMatching$Circle
class PatternMatching$Rectangle
```

## Learning Points

1. Type checking and casting for instanceof
2. Bytecode generation for pattern variables
3. Implementation mechanism of Switch expressions
4. Bytecode representation of Record classes
5. Performance characteristics of pattern matching