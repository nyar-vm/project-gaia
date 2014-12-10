# Lambda Expressions Example

Demonstrates the implementation of Lambda expressions and method references in bytecode, focusing on understanding the `invokedynamic` instruction.

## File Descriptions

- `LambdaExpressions.java` - Source code for Lambda expressions
- `LambdaExpressions.class` - Compiled bytecode
- `LambdaExpressions.jasm` - JASM assembly format

## Java Source Highlights

```java
// Lambda Expressions
Function<Integer, String> lambda = x -> prefix + x * 2;

// Method Reference
Function<String, Integer> methodRef = String::length;

// Nested Lambda
Function<Integer, Function<Integer, Integer>> nested = x -> y -> x + y;
```

## JASM Key Features

### invokedynamic Instruction

```jasm
invokedynamic InvokeDynamic REF_invokeStatic:Method java/lang/invoke/LambdaMetafactory.metafactory:
    "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;
     Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)
     Ljava/lang/invoke/CallSite;"
```

### Synthetic Methods

- `lambda$createLambda$0` - Actual implementation of the Lambda expression
- `lambda$nestedLambda$1` - Outer layer of the nested Lambda
- `lambda$nestedLambda$2` - Inner layer of the nested Lambda

### Bootstrap Methods

Lambda expressions use bootstrap methods to create functional interface instances at runtime.

## Learning Points

1. Role of the `invokedynamic` instruction
2. Usage of `LambdaMetafactory`
3. Naming rules for synthetic methods
4. Implementation mechanism of Bootstrap methods
5. Bytecode representation of method references