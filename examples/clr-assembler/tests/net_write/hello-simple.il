// IL code generated for MyAssembler class (simplified version)
// Corresponding to hello.cs - without line numbers

.assembly extern UnityEngine
{
    .ver 0:0:0:0
}

.assembly hello
{
    .hash algorithm 0x00008004
    .ver 0:0:0:0
}

.module hello.dll

.class public auto ansi beforefieldinit MyAssembler
    extends [UnityEngine]UnityEngine.MonoBehaviour
{
    // Start() method - simplified without unnecessary branches
    .method public hidebysig virtual 
        instance void Start () cil managed 
    {
        .maxstack 8
        .locals init (bool V_0)
        
        nop
        ldstr "Hello World!"
        call void [UnityEngine]UnityEngine.Debug::Log(object)
        nop
        ret
    }
    
    // Constructor
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        ldarg.0
        call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        nop
        ret
    }
}