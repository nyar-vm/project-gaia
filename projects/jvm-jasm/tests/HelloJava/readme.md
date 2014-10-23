# HelloJava 示例

这是一个基础的 Hello World 示例，展示了最简单的 Java 类和 JASM 结构。

## 文件说明

- `HelloJava.java` - Java 源代码
- `HelloJava.class` - 编译后的字节码
- `HelloJava.jasm` - JASM 汇编格式

## Java 源码

```java
public class HelloJava {
    public static void main(String[] args) {
        System.out.println("Hello, Java World!");
    }
}
```

## JASM 关键结构

- 类定义：`public super class HelloJava`
- 主方法：`public static Method main:"([Ljava/lang/String;)V"`
- 字节码指令：`getstatic`, `ldc`, `invokevirtual`, `return`

## 学习要点

1. 基本的类结构
2. 静态方法的定义
3. 系统输出的字节码实现
4. 字符串常量的加载