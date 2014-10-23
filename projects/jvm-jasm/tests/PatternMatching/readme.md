# 模式匹配示例

展示 Java 模式匹配（instanceof 和 switch 表达式）的字节码实现。

## 文件说明

- `PatternMatching.java` - 模式匹配源码
- `PatternMatching.class` - 编译后的字节码
- `PatternMatching.jasm` - JASM 汇编格式
- `PatternMatching$*.class` - 内部类（记录类）字节码

## Java 源码要点

```java
// instanceof 模式匹配
if (obj instanceof String s) {
    return "String with length: " + s.length();
}

// switch 表达式模式匹配
return switch (shape) {
    case Circle(var radius) -> "Circle with radius: " + radius;
    case Rectangle(var width, var height) -> "Rectangle: " + width + "x" + height;
    default -> "Unknown shape";
};
```

## JASM 关键特征

### instanceof 模式匹配

```jasm
aload_1                    // obj
instanceof String;         // 类型检查
ifeq L19;                  // 如果不是String，跳转
aload_1;                   // obj
checkcast String;          // 强制转换为String
astore_2;                  // 存储到局部变量s（编译器生成）
```

### Switch 表达式

```jasm
aload_1                    // shape
invokevirtual Method PatternMatching$Shape.getClass:()Ljava/lang/Class;
// ... 模式匹配逻辑
```

### 记录类（内部类）

```jasm
class PatternMatching$1Point
class PatternMatching$1Line
class PatternMatching$Circle
class PatternMatching$Rectangle
```

## 学习要点

1. instanceof 的类型检查和强制转换
2. 模式变量的字节码生成
3. Switch 表达式的实现机制
4. 记录类的字节码表示
5. 模式匹配的性能特征