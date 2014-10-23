import java.util.function.Function;
import java.util.function.Consumer;

public class LambdaExpressions {
    private String prefix = "Result: ";
    
    public Function<Integer, String> createLambda() {
        return (Integer x) -> prefix + x * 2;
    }
    
    public void processWithLambda(Consumer<String> processor) {
        processor.accept("Hello Lambda");
    }
    
    public void methodReference() {
        Consumer<String> printer = System.out::println;
        printer.accept("Method Reference");
    }
    
    public void nestedLambda() {
        Function<Integer, Function<Integer, Integer>> adder = x -> y -> x + y;
        int result = adder.apply(5).apply(3);
    }
}