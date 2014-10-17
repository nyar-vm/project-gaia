// IL code with multiple Unity components
// Assembly name: hello (matches filename hello.dll)
// Contains: MyAssembler, PlayerController, GameManager

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

// Original MyAssembler component
.class public auto ansi beforefieldinit MyAssembler
    extends [UnityEngine]UnityEngine.MonoBehaviour
{
    .method public hidebysig virtual 
        instance void Start () cil managed 
    {
        .maxstack 8
        
        ldstr "Hello World!"
        call void [UnityEngine]UnityEngine.Debug::Log(object)
        ret
    }
    
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        ldarg.0
        call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        ret
    }
}

// PlayerController component
.class public auto ansi beforefieldinit PlayerController
    extends [UnityEngine]UnityEngine.MonoBehaviour
{
    .method public hidebysig virtual 
        instance void Start () cil managed 
    {
        .maxstack 8
        
        ldstr "PlayerController initialized!"
        call void [UnityEngine]UnityEngine.Debug::Log(object)
        ret
    }
    
    .method public hidebysig virtual 
        instance void Update () cil managed 
    {
        .maxstack 8
        
        // Simple movement logic would go here
        ret
    }
    
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        ldarg.0
        call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        ret
    }
}

// GameManager component
.class public auto ansi beforefieldinit GameManager
    extends [UnityEngine]UnityEngine.MonoBehaviour
{
    .method public hidebysig virtual 
        instance void Awake () cil managed 
    {
        .maxstack 8
        
        ldstr "GameManager awake!"
        call void [UnityEngine]UnityEngine.Debug::Log(object)
        ret
    }
    
    .method public hidebysig specialname rtspecialname 
        instance void .ctor () cil managed 
    {
        .maxstack 8
        
        ldarg.0
        call instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()
        ret
    }
}