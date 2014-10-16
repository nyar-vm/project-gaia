using System;
using System.Collections.Generic;
using System.Linq;

namespace Generics
{
    // 泛型类
    public class GenericList<T>
    {
        private List<T> items = new List<T>();
        
        public void Add(T item)
        {
            items.Add(item);
        }
        
        public T Get(int index)
        {
            return items[index];
        }
        
        public int Count => items.Count;
        
        // 泛型方法
        public void ProcessItems<U>(U processor) where U : IItemProcessor<T>
        {
            foreach (T item in items)
            {
                processor.Process(item);
            }
        }
    }
    
    // 泛型接口
    public interface IItemProcessor<T>
    {
        void Process(T item);
    }
    
    // 实现泛型接口
    public class StringProcessor : IItemProcessor<string>
    {
        public void Process(string item)
        {
            Console.WriteLine($"Processing string: {item.ToUpper()}");
        }
    }
    
    public class NumberProcessor : IItemProcessor<int>
    {
        public void Process(int item)
        {
            Console.WriteLine($"Processing number: {item * 2}");
        }
    }
    
    // 泛型约束
    public class Repository<T> where T : class, IEntity, new()
    {
        private List<T> items = new List<T>();
        
        public void Add(T item)
        {
            items.Add(item);
            Console.WriteLine($"Added {item.GetType().Name} with ID: {item.Id}");
        }
        
        public T GetById(int id)
        {
            return items.FirstOrDefault(item => item.Id == id);
        }
        
        public IEnumerable<T> GetAll()
        {
            return items;
        }
    }
    
    // 实体接口
    public interface IEntity
    {
        int Id { get; set; }
    }
    
    // 实现实体
    public class Person : IEntity
    {
        public int Id { get; set; }
        public string Name { get; set; }
        public int Age { get; set; }
        
        public Person(int id, string name, int age)
        {
            Id = id;
            Name = name;
            Age = age;
        }
        
        public override string ToString()
        {
            return $"Person: {Name}, Age: {Age}, ID: {Id}";
        }
    }
    
    public class Product : IEntity
    {
        public int Id { get; set; }
        public string Name { get; set; }
        public decimal Price { get; set; }
        
        public Product(int id, string name, decimal price)
        {
            Id = id;
            Name = name;
            Price = price;
        }
        
        public override string ToString()
        {
            return $"Product: {Name}, Price: {Price:C}, ID: {Id}";
        }
    }
    
    // 泛型方法
    public static class GenericMethods
    {
        // 简单的泛型方法
        public static T Max<T>(T first, T second) where T : IComparable<T>
        {
            return first.CompareTo(second) > 0 ? first : second;
        }
        
        // 交换方法
        public static void Swap<T>(ref T a, ref T b)
        {
            T temp = a;
            a = b;
            b = temp;
        }
        
        // 泛型约束方法
        public static void ProcessItems<T>(IEnumerable<T> items) where T : class
        {
            foreach (T item in items)
            {
                Console.WriteLine($"Item: {item}");
            }
        }
        
        // 多个泛型参数
        public static TResult Transform<TInput, TResult>(TInput input, Func<TInput, TResult> transformer)
        {
            return transformer(input);
        }
        
        // 协变和逆变示例
        public static void ProcessAnimals(IEnumerable<Animal> animals)
        {
            foreach (var animal in animals)
            {
                Console.WriteLine($"Processing {animal.Name}");
            }
        }
        
        // 泛型委托
        public delegate TResult GenericDelegate<TInput, TResult>(TInput input);
        
        public static TResult ApplyOperation<TInput, TResult>(TInput input, GenericDelegate<TInput, TResult> operation)
        {
            return operation(input);
        }
    }
    
    // 协变接口
    public interface IReadOnlyRepository<out T>
    {
        T GetById(int id);
        IEnumerable<T> GetAll();
    }
    
    // 逆变接口
    public interface IComparer<in T>
    {
        int Compare(T x, T y);
    }
    
    // 动物类层次结构（用于演示协变）
    public class Animal
    {
        public string Name { get; set; }
        public Animal(string name) { Name = name; }
    }
    
    public class Mammal : Animal
    {
        public Mammal(string name) : base(name) { }
    }
    
    public class Dog : Mammal
    {
        public Dog(string name) : base(name) { }
    }
    
    class Program
    {
        static void Main()
        {
            Console.WriteLine("=== Generic List Examples ===");
            
            // 泛型列表 - 字符串
            GenericList<string> stringList = new GenericList<string>();
            stringList.Add("Hello");
            stringList.Add("World");
            stringList.Add("Generics");
            
            StringProcessor stringProcessor = new StringProcessor();
            stringList.ProcessItems(stringProcessor);
            
            // 泛型列表 - 整数
            GenericList<int> intList = new GenericList<int>();
            intList.Add(10);
            intList.Add(20);
            intList.Add(30);
            
            NumberProcessor numberProcessor = new NumberProcessor();
            intList.ProcessItems(numberProcessor);
            
            Console.WriteLine("\n=== Generic Repository Examples ===");
            
            // 人员仓库
            Repository<Person> personRepo = new Repository<Person>();
            personRepo.Add(new Person(1, "Alice", 30));
            personRepo.Add(new Person(2, "Bob", 25));
            
            var person = personRepo.GetById(1);
            Console.WriteLine($"Found: {person}");
            
            // 产品仓库
            Repository<Product> productRepo = new Repository<Product>();
            productRepo.Add(new Product(1, "Laptop", 999.99m));
            productRepo.Add(new Product(2, "Mouse", 29.99m));
            
            Console.WriteLine("\n=== Generic Methods Examples ===");
            
            // 比较方法
            int maxInt = GenericMethods.Max(10, 20);
            string maxString = GenericMethods.Max("apple", "banana");
            Console.WriteLine($"Max int: {maxInt}");
            Console.WriteLine($"Max string: {maxString}");
            
            // 交换方法
            int a = 5, b = 10;
            GenericMethods.Swap(ref a, ref b);
            Console.WriteLine($"After swap: a={a}, b={b}");
            
            string str1 = "hello", str2 = "world";
            GenericMethods.Swap(ref str1, ref str2);
            Console.WriteLine($"After swap: {str1} {str2}");
            
            // 转换方法
            string numberStr = "123";
            int number = GenericMethods.Transform(numberStr, s => int.Parse(s));
            Console.WriteLine($"Transformed '{numberStr}' to {number}");
            
            // 协变示例
            Console.WriteLine("\n=== Covariance Example ===");
            List<Dog> dogs = new List<Dog> { new Dog("Buddy"), new Dog("Max") };
            GenericMethods.ProcessAnimals(dogs); // 协变允许这样做
            
            // 泛型委托
            Console.WriteLine("\n=== Generic Delegate Example ===");
            GenericDelegate<string, int> lengthDelegate = s => s.Length;
            int length = GenericMethods.ApplyOperation("Hello", lengthDelegate);
            Console.WriteLine($"Length of 'Hello': {length}");
            
            // 复杂泛型使用
            Console.WriteLine("\n=== Complex Generic Usage ===");
            var people = new List<Person>
            {
                new Person(1, "Alice", 30),
                new Person(2, "Bob", 25),
                new Person(3, "Charlie", 35)
            };
            
            GenericMethods.ProcessItems(people);
            
            // 使用 LINQ（内部使用泛型）
            var adults = people.Where(p => p.Age >= 30).ToList();
            Console.WriteLine("Adults:");
            foreach (var adult in adults)
            {
                Console.WriteLine(adult);
            }
        }
    }
}