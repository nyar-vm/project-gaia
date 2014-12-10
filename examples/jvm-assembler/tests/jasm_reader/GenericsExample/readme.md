# Generics Example

Demonstrates the implementation of Java generics in bytecode, focusing on type erasure and the `Signature` attribute.

## File Descriptions

- `GenericsExample.java` - Generics source code
- `GenericsExample.class` - Compiled bytecode
- `GenericsExample.jasm` - JASM assembly format

## Java Source Highlights

```java
// Generic class
public class GenericsExample<T extends Comparable<T>> {
    private List<T> items = new ArrayList<>();
    
    // Generic method
    public <U extends Number> void genericMethod(U number) {
        // ...
    }
    
    // Wildcard method
    public void processWildcard(List<? extends Number> numbers) {
        // ...
    }
}
```

## JASM Key Features

### Type Signature Retention

```jasm
public super class GenericsExample:"<T::Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;" version 65:0
{
    private Field items:"Ljava/util/List;":"Ljava/util/List<TT;>;";
    // Field signature preserves generic information
}
```

### Generic Method Signatures

```jasm
public Method genericMethod:"(Ljava/lang/Number;)V":"<U:Ljava/lang/Number;>(TU;)V;"
public Method processWildcard:"(Ljava/util/List;)V":"(Ljava/util/List<+Ljava/lang/Number;>;)V;"
```

### Actual Types After Erasure

- `T` -> `Ljava/lang/Object;`
- `List<T>` -> `Ljava/util/List;`
- But the `Signature` attribute retains full generic information

## Learning Points

1. Type erasure mechanism
2. Role of the `Signature` attribute
3. Handling of generic bounds
4. Bytecode representation of wildcards
5. Retention of runtime type information