using System;
using System.Collections.Generic;

namespace ControlFlow
{
    class Program
    {
        static void Main()
        {
            // if-else 语句
            int age = 25;
            if (age >= 18)
            {
                Console.WriteLine("Adult");
            }
            else if (age >= 13)
            {
                Console.WriteLine("Teenager");
            }
            else
            {
                Console.WriteLine("Child");
            }

            // switch 语句
            string day = "Monday";
            switch (day)
            {
                case "Monday":
                case "Tuesday":
                    Console.WriteLine("Weekday");
                    break;
                case "Saturday":
                case "Sunday":
                    Console.WriteLine("Weekend");
                    break;
                default:
                    Console.WriteLine("Invalid day");
                    break;
            }

            // for 循环
            for (int i = 0; i < 5; i++)
            {
                Console.WriteLine($"For loop: {i}");
            }

            // while 循环
            int j = 0;
            while (j < 3)
            {
                Console.WriteLine($"While loop: {j}");
                j++;
            }

            // do-while 循环
            int k = 0;
            do
            {
                Console.WriteLine($"Do-while loop: {k}");
                k++;
            } while (k < 2);

            // foreach 循环
            List<string> fruits = new List<string> { "Apple", "Banana", "Orange" };
            foreach (string fruit in fruits)
            {
                Console.WriteLine($"Fruit: {fruit}");
            }

            // break 和 continue
            for (int n = 0; n < 10; n++)
            {
                if (n == 3)
                    continue; // 跳过本次循环
                if (n == 8)
                    break; // 退出循环
                Console.WriteLine($"Number: {n}");
            }

            // goto 语句（不推荐，但用于测试）
            int counter = 0;
        start:
            Console.WriteLine($"Goto counter: {counter}");
            counter++;
            if (counter < 3)
                goto start;

            // 三元运算符
            int x = 10, y = 20;
            string result = (x > y) ? "x is greater" : "y is greater or equal";
            Console.WriteLine(result);

            // 空合并运算符
            string nullString = null;
            string nonNullString = nullString ?? "Default value";
            Console.WriteLine(nonNullString);
        }
    }
}