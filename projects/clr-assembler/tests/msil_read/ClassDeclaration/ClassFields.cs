namespace Valkyrie.Translator.ClassAndStructure;

class ClassFields
{
    private int _field1;
    protected int _field2;
    internal int _field3;
    public int _field4;
    protected internal int _field5;
    private protected int _field6;

    public int _field11 { get; }
    public int _field12 { get; set; }
    public int _field13 { get; private set; }

    const int _field21 = 100;
    static int _field22 = 200;
    readonly int _field23 = 300;
}
