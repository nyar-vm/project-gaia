import java.lang.annotation.*;
import java.lang.reflect.*;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.TYPE, ElementType.METHOD, ElementType.FIELD})
@interface CustomAnnotation {
    String value() default "";
    int priority() default 0;
    String[] tags() default {};
}

@Retention(RetentionPolicy.CLASS)
@interface ClassAnnotation {
    String description();
}

@CustomAnnotation(value = "Test Class", priority = 1, tags = {"example", "test"})
public class AnnotationsExample {
    
    @CustomAnnotation(priority = 2)
    private String annotatedField;
    
    @ClassAnnotation(description = "Test method")
    @CustomAnnotation(value = "Test Method", priority = 3)
    public void annotatedMethod() {
        // Method implementation
    }
    
    @SafeVarargs
    public final <T> void varargsMethod(T... items) {
        for (T item : items) {
            System.out.println(item);
        }
    }
    
    @Deprecated(since = "1.5", forRemoval = true)
    public void deprecatedMethod() {
        // Old implementation
    }
    
    @SuppressWarnings("unchecked")
    public void suppressedMethod() {
        // Code that would normally generate warnings
    }
    
    public void readAnnotations() throws NoSuchMethodException {
        Class<?> clazz = this.getClass();
        
        // Read class annotation
        CustomAnnotation classAnnotation = clazz.getAnnotation(CustomAnnotation.class);
        
        // Read method annotation
        Method method = clazz.getMethod("annotatedMethod");
        CustomAnnotation methodAnnotation = method.getAnnotation(CustomAnnotation.class);
    }
}