# Inner Classes Example

Demonstrates the implementation of inner classes, local classes, and anonymous classes in bytecode, focusing on the `InnerClass` and `NestMembers` attributes.

## File Descriptions

- `InnerClasses.java` - Source code for inner classes
- `InnerClasses.class` - Main class bytecode
- `InnerClasses$Inner.class` - Member inner class bytecode
- `InnerClasses$StaticInner.class` - Static inner class bytecode
- `InnerClasses$1LocalInner.class` - Local inner class bytecode
- `InnerClasses$1.class` - Anonymous inner class bytecode

## Java Source Highlights

```java
public class InnerClasses {
    class Inner { } // Member inner class
    static class StaticInner { } // Static inner class

    void method() {
        class LocalInner { } // Local inner class
        Runnable r = new Runnable() { // Anonymous inner class
            @Override public void run() { }
        };
    }
}
```

## JASM Key Features

### InnerClass Attribute

```jasm
InnerClass LocalInner = class InnerClasses$1LocalInner;
InnerClass public static StaticInner = class InnerClasses$StaticInner of class InnerClasses;
InnerClass public Inner = class InnerClasses$Inner of class InnerClasses;
```

### NestMembers Attribute

```jasm
NestMembers InnerClasses$StaticInner,
            InnerClasses$Inner,
            InnerClasses$1,
            InnerClasses$1LocalInner;
```

### Constructor of Inner Classes

Inner classes (non-static) contain a reference to the outer class:

```jasm
public Method "<init>":"(LInnerClasses;)V"
{
    aload_0;
    aload_1;
    putfield Field this$0:"LInnerClasses;";
    // ...
}
```

## Learning Points

1. Bytecode naming rules for inner classes
2. Connection between `InnerClass` and `NestMembers`
3. Difference between static and non-static inner classes
4. Implementation mechanism of local and anonymous classes
5. Transmission of the `this$0` reference
