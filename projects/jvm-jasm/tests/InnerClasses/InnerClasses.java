public class InnerClasses {
    private String outerField = "Outer";
    
    public class Inner {
        private String innerField = "Inner";
        
        public String getOuterField() {
            return outerField;
        }
    }
    
    public static class StaticInner {
        private static int staticCounter = 0;
        
        public static void increment() {
            staticCounter++;
        }
    }
    
    public void localInner() {
        final String localVar = "Local";
        
        class LocalInner {
            public String getLocal() {
                return localVar;
            }
        }
        
        LocalInner local = new LocalInner();
    }
    
    public void anonymousInner() {
        Runnable runnable = new Runnable() {
            @Override
            public void run() {
                System.out.println("Anonymous");
            }
        };
    }
}