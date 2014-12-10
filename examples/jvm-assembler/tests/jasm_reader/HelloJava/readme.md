# HelloJava Example

This is a basic Hello World example, demonstrating the simplest Java class and JASM structure.

## File Descriptions

- `HelloJava.java` - Java source code
- `HelloJava.class` - Compiled bytecode
- `HelloJava.jasm` - JASM assembly format

## Java Source

```java
public class HelloJava {
    public static void main(String[] args) {
        System.out.println("Hello, Java World!");
    }
}
```

## JASM Key Structure

- Class definition: `public super class HelloJava`
- Main method: `public static Method main:"([Ljava/lang/String;)V"`
- Bytecode instructions: `getstatic`, `ldc`, `invokevirtual`, `return`

## Learning Points

1. Basic class structure
2. Definition of static methods
3. Bytecode implementation of system output
4. Loading of string constants