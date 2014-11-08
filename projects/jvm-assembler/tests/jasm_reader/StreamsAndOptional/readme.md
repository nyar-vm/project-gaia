# Stream 和 Optional 示例

展示 Java 8 Stream API 和 Optional 的字节码实现，重点理解函数式编程的字节码。

## 文件说明

- `StreamsAndOptional.java` - Stream 和 Optional 源码
- `StreamsAndOptional.class` - 编译后的字节码
- `StreamsAndOptional.jasm` - JASM 汇编格式

## Java 源码要点

```java
// Stream 操作
List<String> names = Arrays.asList("Alice", "Bob", "Charlie");
List<String> result = names.stream()
    .filter(name -> name.length() > 3)
    .map(String::toUpperCase)
    .collect(Collectors.toList());

// Optional 使用
Optional<String> optional = Optional.of("test");
optional.ifPresent(value -> System.out.println(value));
```

## JASM 关键特征

### Stream 链式调用

```jasm
invokeinterface InterfaceMethod java/util/stream/Stream.filter:
    "(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;";
invokeinterface InterfaceMethod java/util/stream/Stream.map:
    "(Ljava/util/function/Function;)Ljava/util/stream/Stream;";
invokestatic Method java/util/stream/Collectors.toList:
    "()Ljava/util/Collector;";
```

### Lambda 和引用

- `filter` 中的 Lambda 表达式 -> `invokedynamic`
- `map` 中的方法引用 -> `invokedynamic`
- `ifPresent` 中的 Lambda -> `invokedynamic`

### Optional 方法

```jasm
invokevirtual Method java/util/Optional.ifPresent:
    "(Ljava/util/function/Consumer;)V";
```

## 学习要点

1. Stream 的接口调用机制
2. 链式操作的字节码实现
3. Optional 的内部结构
4. 函数式接口的 Lambda 实现
5. 收集器的字节码表示