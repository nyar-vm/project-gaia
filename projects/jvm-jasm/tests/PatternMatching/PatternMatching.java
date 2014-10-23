public class PatternMatching {
    
    public String instanceofPattern(Object obj) {
        if (obj instanceof String s) {
            return "String with length: " + s.length();
        }
        return "Not a string";
    }
    
    public String switchExpression(Object obj) {
        return switch (obj) {
            case String s when s.length() > 5 -> "Long string: " + s;
            case String s -> "Short string: " + s;
            case Integer i -> "Number: " + i;
            case null -> "Null value";
            default -> "Unknown type";
        };
    }
    
    public int recordPattern(Object obj) {
        record Point(int x, int y) {}
        record Line(Point start, Point end) {}
        
        if (obj instanceof Line(Point(int x1, int y1), Point(int x2, int y2))) {
            return Math.abs(x2 - x1) + Math.abs(y2 - y1);
        }
        return 0;
    }
    
    public String sealedClassPattern(Shape shape) {
        return switch (shape) {
            case Circle c -> "Circle with radius " + c.radius();
            case Rectangle r -> "Rectangle " + r.width() + "x" + r.height();
        };
    }
    
    sealed interface Shape permits Circle, Rectangle {}
    record Circle(double radius) implements Shape {}
    record Rectangle(double width, double height) implements Shape {}
}