# Annotations Example

Demonstrates the bytecode implementation of Java annotations, including standard and custom annotations.

## File Descriptions

- `AnnotationsExample.java` - Annotation usage source code
- `AnnotationsExample.class` - Compiled bytecode
- `AnnotationsExample.jasm` - JASM assembly format
- `ClassAnnotation.class` - Custom class annotation
- `CustomAnnotation.class` - Custom method annotation

## Java Source Highlights

```java
// Standard annotations
@Override
@Deprecated
@SuppressWarnings("unused")
public void annotatedMethod() {
    // ...
}

// Custom annotations
@ClassAnnotation(description = "Test class")
public class AnnotationsExample {
    @CustomAnnotation(value = "test annotation", count = 42)
    public void customAnnotatedMethod() {
        // ...
    }
}
```

## Key JASM Features

### RuntimeVisibleAnnotations

```jasm
RuntimeVisibleAnnotations
@Ljava/lang/Override;()
@Ljava/lang/Deprecated;()
@Ljava/lang/SuppressWarnings;(value={"unused"})
```

### Custom Annotations

```jasm
@LClassAnnotation;(description="Test class")
@LCustomAnnotation;(count=42,value="test annotation")
```

### Annotation Definitions (Inner Classes)

```jasm
public abstract interface class ClassAnnotation
    implements java/lang/annotation/Annotation
    
public abstract interface class CustomAnnotation  
    implements java/lang/annotation/Annotation
```

## Learning Points

1. Bytecode representation of standard annotations
2. Implementation of custom annotations
3. Storage of annotation attributes
4. Visibility of runtime annotations
5. Special handling of annotation interfaces
