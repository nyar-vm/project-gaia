import javax.tools.JavaCompiler;
import javax.tools.ToolProvider;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.concurrent.CompletableFuture;
import java.io.BufferedReader;
import java.io.FileReader;
import java.lang.annotation.*;

public class JavaClassGenerator {

    private static final String OUTPUT_DIR = "./generated";

    public static void main(String[] args) throws IOException {
        // 确保输出目录存在
        Path outputPath = Paths.get(OUTPUT_DIR);
        if (!Files.exists(outputPath)) {
            Files.createDirectories(outputPath);
        }

        createAndCompile("SimpleClass", """
public class SimpleClass {
    public static void main(String[] args) {
        System.out.println("Hello from SimpleClass!");
    }
}
""");

        createAndCompile("ClassWithFields", """
public class ClassWithFields {
    private int id = 10;
    public String name = "Test";

    public int getId() {
        return id;
    }
}
""");

        createAndCompile("ClassWithMethods", """
public class ClassWithMethods {
    public void sayHello() {
        System.out.println("Hello from method!");
    }

    public int add(int a, int b) {
        return a + b;
    }
}
""");

        createAndCompile("LoopTest", """
public class LoopTest {
    public static void main(String[] args) {
        for (int i = 0; i < 3; i++) {
            System.out.println(i);
        }
    }
}
""");

        createAndCompile("ConditionalTest", """
public class ConditionalTest {
    public static void main(String[] args) {
        int x = 10;
        if (x > 5) {
            System.out.println("x is greater than 5");
        } else {
            System.out.println("x is not greater than 5");
        }
    }
}
""");

        // Generics Example
        createAndCompile("GenericBox", """
public class GenericBox<T> {
    private T value;

    public GenericBox(T value) {
        this.value = value;
    }

    public T getValue() {
        return value;
    }

    public static void main(String[] args) {
        GenericBox<String> stringBox = new GenericBox<>("Hello Generics");
        System.out.println(stringBox.getValue());
    }
}
""");

        // Enum Example
        createAndCompile("TrafficLight", """
public enum TrafficLight {
    RED, YELLOW, GREEN;

    public String getColor() {
        return name();
    }

    public static void main(String[] args) {
        System.out.println(TrafficLight.RED.getColor());
    }
}
""");

        // Annotation Example
        createAndCompile("AnnotatedClass", """
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
@interface MyAnnotation {
    String value() default "default";
}

@MyAnnotation("Hello Annotation")
public class AnnotatedClass {
    public static void main(String[] args) {
        System.out.println("AnnotatedClass created");
    }
}
""");

        // Lambda Expression / Functional Interface Example
        createAndCompile("LambdaExample", """
import java.util.function.Consumer;

public class LambdaExample {
    public static void main(String[] args) {
        Consumer<String> greeter = message -> System.out.println("Lambda says: " + message);
        greeter.accept("Hello Lambda");
    }
}
""");

        // Try-with-resources Example
        createAndCompile("TryWithResourcesExample", """
import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;

public class TryWithResourcesExample {
    public static void main(String[] args) {
        try (BufferedReader reader = new BufferedReader(new FileReader("nonexistent.txt"))) {
            System.out.println(reader.readLine());
        } catch (IOException e) {
            System.out.println("Caught IOException: " + e.getMessage());
        }
    }
}
""");

        // CompletableFuture (Asynchronous) Example
        createAndCompile("CompletableFutureExample", """
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

public class CompletableFutureExample {
    public static void main(String[] args) throws InterruptedException, ExecutionException {
        CompletableFuture<String> future = CompletableFuture.supplyAsync(() -> {
            try {
                Thread.sleep(100); // Simulate a long-running operation
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return "Hello from CompletableFuture!";
        });

        System.out.println("Main thread continues...");
        String result = future.get(); // Blocks until the future completes
        System.out.println(result);
    }
}
""");

        System.out.println("All class files generated successfully!");
        // 恢复原始工作目录
        // System.setProperty("user.dir", currentDir.toString());
    }

    private static void createAndCompile(String className, String content) throws IOException {
        String javaFileName = className + ".java";
        Path javaFilePath = Paths.get(OUTPUT_DIR, javaFileName);

        try (FileWriter writer = new FileWriter(javaFilePath.toFile())) {
            writer.write(content);
        }
        System.out.println("Generated " + javaFileName);

        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        if (compiler == null) {
            throw new IllegalStateException("JDK is required to run this test (javac not found).");
        }

        int compilationResult = compiler.run(null, null, null,
                "-d", OUTPUT_DIR,
                javaFilePath.toAbsolutePath().toString());

        if (compilationResult != 0) {
            throw new RuntimeException("Compilation failed for " + javaFileName);
        }
        System.out.println("Compiled " + javaFileName);

        // Delete the .java source file after successful compilation
        Files.delete(javaFilePath);
        System.out.println("Deleted " + javaFileName);
    }
}