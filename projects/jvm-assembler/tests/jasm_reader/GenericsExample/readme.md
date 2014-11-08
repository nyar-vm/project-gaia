# 泛型示例

展示泛型类型擦除和类型签名保留的字节码实现。

## 文件说明

- `GenericsExample.java` - 泛型源码
- `GenericsExample.class` - 编译后的字节码
- `GenericsExample.jasm` - JASM 汇编格式

## Java 源码要点

```java
// 泛型类
public class GenericsExample<T extends Comparable<T>> {
    private List<T> items = new ArrayList<>();
    
    // 泛型方法
    public <U extends Number> void genericMethod(U number) {
        // ...
    }
    
    // 通配符方法
    public void processWildcard(List<? extends Number> numbers) {
        // ...
    }
}
```

## JASM 关键特征

### 类型签名保留

```jasm
public super class GenericsExample:"<T::Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;" version 65:0
{
    private Field items:"Ljava/util/List;":"Ljava/util/List<TT;>;";
    // 字段签名保留了泛型信息
}
```

### 泛型方法签名

```jasm
public Method genericMethod:"(Ljava/lang/Number;)V":"<U:Ljava/lang/Number;>(TU;)V;"
public Method processWildcard:"(Ljava/util/List;)V":"(Ljava/util/List<+Ljava/lang/Number;>;)V;"
```

### 类型擦除后的实际类型

- `T` -> `Ljava/lang/Object;`
- `List<T>` -> `Ljava/util/List;`
- 但 Signature 属性保留了完整泛型信息

## 学习要点

1. 类型擦除机制
2. Signature 属性的作用
3. 泛型边界的处理
4. 通配符的字节码表示
5. 运行时类型信息的保留