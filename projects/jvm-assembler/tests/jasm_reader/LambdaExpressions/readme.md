# Lambda 表达式示例

展示 Lambda 表达式和方法引用在字节码中的实现，重点理解 `invokedynamic` 指令。

## 文件说明

- `LambdaExpressions.java` - Lambda 表达式源码
- `LambdaExpressions.class` - 编译后的字节码
- `LambdaExpressions.jasm` - JASM 汇编格式

## Java 源码要点

```java
// Lambda 表达式
Function<Integer, String> lambda = x -> prefix + x * 2;

// 方法引用
Function<String, Integer> methodRef = String::length;

// 嵌套 Lambda
Function<Integer, Function<Integer, Integer>> nested = x -> y -> x + y;
```

## JASM 关键特征

### invokedynamic 指令

```jasm
invokedynamic InvokeDynamic REF_invokeStatic:Method java/lang/invoke/LambdaMetafactory.metafactory:
    "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;
     Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)
     Ljava/lang/invoke/CallSite;"
```

### 合成方法

- `lambda$createLambda$0` - Lambda 表达式的实际实现
- `lambda$nestedLambda$1` - 嵌套 Lambda 的外层
- `lambda$nestedLambda$2` - 嵌套 Lambda 的内层

### Bootstrap 方法

Lambda 表达式通过 bootstrap 方法在运行时创建函数式接口实例。

## 学习要点

1. `invokedynamic` 指令的作用
2. `LambdaMetafactory` 的使用
3. 合成方法的命名规则
4. Bootstrap 方法的实现机制
5. 方法引用的字节码表示