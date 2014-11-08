# 内部类示例

展示内部类、局部类和匿名类在字节码中的表示和关系。

## 文件说明

- `InnerClasses.java` - 内部类源码
- `InnerClasses.class` - 主类字节码
- `InnerClasses$*.class` - 内部类字节码文件
- `InnerClasses.jasm` - 主类 JASM 格式

## Java 源码要点

```java
public class InnerClasses {
    // 静态内部类
    public static class StaticInner {
        // ...
    }
    
    // 成员内部类
    public class Inner {
        // ...
    }
    
    public void localInner() {
        // 局部内部类
        class LocalInner {
            // ...
        }
        // ...
    }
    
    public void anonymousInner() {
        // 匿名内部类
        Runnable r = new Runnable() {
            @Override
            public void run() {
                // ...
            }
        };
    }
}
```

## JASM 关键特征

### InnerClass 属性

```jasm
InnerClass LocalInner = class InnerClasses$1LocalInner;
InnerClass public static StaticInner = class InnerClasses$StaticInner of class InnerClasses;
InnerClass public Inner = class InnerClasses$Inner of class InnerClasses;
```

### NestMembers 属性

```jasm
NestMembers InnerClasses$StaticInner,
            InnerClasses$Inner,
            InnerClasses$1,
            InnerClasses$1LocalInner;
```

### 内部类文件命名

- `InnerClasses$StaticInner` - 静态内部类
- `InnerClasses$Inner` - 成员内部类
- `InnerClasses$1LocalInner` - 局部内部类
- `InnerClasses$1` - 匿名内部类

## 学习要点

1. 不同类型内部类的区别
2. InnerClass 属性的作用
3. NestMembers 属性的意义
4. 内部类文件命名规则
5. 访问外部类成员的字节码实现