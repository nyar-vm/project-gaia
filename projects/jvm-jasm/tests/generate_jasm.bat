@echo off
echo 编译所有 Java 文件...
for /d %%i in (*) do (
    echo 编译 %%i...
    cd %%i
    javac *.java
    cd ..
)

echo.
echo 生成所有 JASM 文件...
for /d %%i in (*) do (
    echo 生成 %%i 的 JASM 文件...
    cd %%i
    for %%f in (*.class) do (
        if not "%%f"=="%%~nf$*.class" (
            echo  正在生成 %%~nf.jasm
            java -jar ../asmtools.jar jdis "%%f" > "%%~nf.jasm"
        )
    )
    cd ..
)

echo.
echo 生成完成！
echo.
echo 生成的文件列表：
for /d %%i in (*) do (
    echo %%i:
    dir %%i\*.jasm 2>nul
)