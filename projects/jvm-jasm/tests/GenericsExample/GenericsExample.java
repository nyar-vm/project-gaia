import java.util.*;

public class GenericsExample<T extends Comparable<T>> {
    private List<T> items = new ArrayList<>();
    
    public void addItem(T item) {
        items.add(item);
    }
    
    public T getMax() {
        return items.stream().max(Comparable::compareTo).orElse(null);
    }
    
    public <U extends Number> double sumGeneric(List<U> numbers) {
        return numbers.stream().mapToDouble(Number::doubleValue).sum();
    }
    
    public void wildcardMethod(List<? extends T> upperBound, List<? super T> lowerBound) {
        // upper bound: can read T
        T item = upperBound.get(0);
        // lower bound: can write T
        lowerBound.add(item);
    }
    
    public static <T> void genericStaticMethod(T[] array) {
        for (T item : array) {
            System.out.println(item);
        }
    }
}