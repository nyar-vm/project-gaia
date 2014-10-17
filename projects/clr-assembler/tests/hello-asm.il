.assembly extern UnityEngine
{
    .ver 0:0:0:0
}

.assembly MyAssembler
{
    .custom instance void [UnityEngine]UnityEngine.UnityAPICompatibilityVersionAttribute::.ctor(string) = ( 01 00 04 35 2E 30 2E 30 00 00 ) // 5.0.0
    .hash algorithm 0x00008004
    .ver 0:0:0:0
}

.module MyAssembler.dll

.class public auto ansi beforefieldinit MyAssembler
    extends [UnityEngine]UnityEngine.MonoBehaviour
{
    // Method Start()
    .method public hidebysig virtual 
        instance void Start () cil managed 
    {
        // Method begins at RVA 0x2050
        .maxstack 8
        .locals init (
            [0] bool V_0
        )

        // [4 5 - 4 6]
        IL_0000: nop
        
        // [5 9 - 5 35]
        IL_0001: ldstr "Hello World!"
        IL_0006: call void [UnityEngine]UnityEngine.Debug::Log(object)
        IL_000b: nop
        
        // [6 5 - 6 6]
        IL_000c: br.s IL_000e
        
        IL_000e: nop
        IL_000f: br.s IL_0011
        
        IL_0011: ret
    }

    // Constructor
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        IL_0000: ldarg.0
        IL_0001: call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        IL_0006: nop
        IL_0007: ret
    }
}