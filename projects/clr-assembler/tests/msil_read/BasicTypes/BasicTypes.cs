namespace Valkyrie.Translator.BasicTypes;

internal class BasicTypes
{
    private static void Main()
    {
        // 整数类型
        byte b = 255;
        sbyte sb = -128;
        short s = -32768;
        ushort us = 65535;
        int i = -2147483648;
        uint ui = 4294967295;
        long l = -9223372036854775808;
        ulong ul = 18446744073709551615;

        // 浮点类型
        float f = 3.14f;
        double d = 2.71828;
        decimal m = 123.456m;

        // 字符和字符串
        char c = 'A';
        string str = "Hello, World!";

        // 布尔类型
        bool flag = true;

        // 可空类型
        int? nullableInt = null;
        DateTime? nullableDate = DateTime.Now;

        // 数组
        int[] intArray = { 1, 2, 3, 4, 5 };
        string[] stringArray = new string[3];
        stringArray[0] = "First";
        stringArray[1] = "Second";
        stringArray[2] = "Third";

        // 多维数组
        int[,] matrix = new int[2, 3] { { 1, 2, 3 }, { 4, 5, 6 } };
        int[][] jagged = new int[3][];
        jagged[0] = new int[] { 1, 2 };
        jagged[1] = new int[] { 3, 4, 5 };
        jagged[2] = new int[] { 6, 7, 8, 9 };

        // 常量
        const double PI = 3.14159;
        const string GREETING = "Welcome";

        // 类型转换
        int converted = (int)d;
        string numberStr = i.ToString();
        double parsed = double.Parse("123.45");
    }
}
