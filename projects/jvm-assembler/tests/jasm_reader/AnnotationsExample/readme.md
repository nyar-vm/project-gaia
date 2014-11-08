# 注解示例

展示 Java 注解的字节码实现，包括标准注解和自定义注解。

## 文件说明

- `AnnotationsExample.java` - 注解使用源码
- `AnnotationsExample.class` - 编译后的字节码
- `AnnotationsExample.jasm` - JASM 汇编格式
- `ClassAnnotation.class` - 自定义类注解
- `CustomAnnotation.class` - 自定义方法注解

## Java 源码要点

```java
// 标准注解
@Override
@Deprecated
@SuppressWarnings("unused")
public void annotatedMethod() {
    // ...
}

// 自定义注解
@ClassAnnotation(description = "Test class")
public class AnnotationsExample {
    @CustomAnnotation(value = "test annotation", count = 42)
    public void customAnnotatedMethod() {
        // ...
    }
}
```

## JASM 关键特征

### RuntimeVisibleAnnotations

```jasm
RuntimeVisibleAnnotations
@Ljava/lang/Override;()
@Ljava/lang/Deprecated;()
@Ljava/lang/SuppressWarnings;(value={"unused"})
```

### 自定义注解

```jasm
@LClassAnnotation;(description="Test class")
@LCustomAnnotation;(count=42,value="test annotation")
```

### 注解定义（内部类）

```jasm
public abstract interface class ClassAnnotation
    implements java/lang/annotation/Annotation
    
public abstract interface class CustomAnnotation  
    implements java/lang/annotation/Annotation
```

## 学习要点

1. 标准注解的字节码表示
2. 自定义注解的实现
3. 注解属性的存储
4. 运行时注解的可见性
5. 注解接口的特殊处理