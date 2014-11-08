// IL code generated for MyAssembler class (optimized version)
// Corresponding to hello.cs - minimal and clean

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
    // Start() method - minimal version
    .method public hidebysig virtual 
        instance void Start () cil managed 
    {
        .maxstack 8
        
        ldstr "Hello World!"
        call void [UnityEngine]UnityEngine.Debug::Log(object)
        ret
    }
    
    // Constructor
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        ldarg.0
        call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        ret
    }
}