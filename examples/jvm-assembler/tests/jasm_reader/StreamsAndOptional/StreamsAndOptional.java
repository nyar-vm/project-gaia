import java.util.*;
import java.util.stream.*;

public class StreamsAndOptional {
    
    public Optional<String> findFirst(List<String> list) {
        return list.stream()
                  .filter(s -> s.length() > 3)
                  .findFirst();
    }
    
    public IntStream primitiveStream() {
        return IntStream.of(1, 2, 3, 4, 5)
                       .map(x -> x * 2)
                       .filter(x -> x > 4);
    }
    
    public Stream<String> flatMapExample() {
        List<List<String>> nested = Arrays.asList(
            Arrays.asList("a", "b"),
            Arrays.asList("c", "d")
        );
        
        return nested.stream()
                    .flatMap(List::stream)
                    .map(String::toUpperCase);
    }
    
    public OptionalInt maxOptional() {
        return IntStream.empty().max();
    }
    
    public void streamChaining() {
        List<String> result = Arrays.stream(new String[]{"hello", "world", "java"})
            .peek(s -> System.out.println("Processing: " + s))
            .filter(s -> s.length() > 4)
            .map(String::toUpperCase)
            .collect(Collectors.toList());
    }
}