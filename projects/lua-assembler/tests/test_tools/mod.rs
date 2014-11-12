use std::{fs, path::Path, process::Command};

/// 调用本机 Python 进行汇编，将 Python 源代码编译为 .pyc 文件
///
/// # 参数
/// * `source_code` - Python 源代码字符串
/// * `output_path` - 输出 .pyc 文件的路径
///
/// # 返回值
/// * `Result<(), String>` - 成功返回 Ok(())，失败返回错误信息
pub fn python_asm(source_code: &str, output_path: &Path) -> Result<(), String> {
    // 创建临时 Python 脚本文件
    let temp_py_path = output_path.with_extension("py");

    // 写入 Python 源代码到临时文件
    fs::write(&temp_py_path, source_code).map_err(|e| format!("Failed to write temporary Python file: {}", e))?;

    // 使用 Python 的 py_compile 模块编译源代码
    let compile_script = format!(
        r#"
import py_compile
import sys
try:
    py_compile.compile('{}', '{}', doraise=True)
    print("Compilation successful")
except Exception as e:
    print(f"Compilation failed: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        temp_py_path.to_string_lossy().replace('\\', "\\\\"),
        output_path.to_string_lossy().replace('\\', "\\\\")
    );

    // 执行编译脚本
    let output = Command::new("python")
        .args(["-c", &compile_script])
        .output()
        .map_err(|e| format!("Failed to execute Python compiler: {}", e))?;

    // 清理临时文件
    let _ = fs::remove_file(&temp_py_path);

    if !output.status.success() {
        return Err(format!("Python compilation failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::python_disassembler::python_dis;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_from_pyc_file() {
        let pyc_path = Path::new("e:/汇编器/project-gaia/projects/pyc-assembler/__pycache__/test.cpython-312.pyc");

        let pyc_file = pyc_assembler::formats::pyc::reader::read_pyc_file(pyc_path).expect("Failed to read .pyc file");

        let pyc_program = pyc_assembler::program::PycProgram::from_pyc_file(&pyc_file);

        println!("Parsed PycProgram: {:#?}", pyc_program);

        // Add assertions here to verify the parsed program
        assert!(!pyc_program.code_object.co_code.is_empty());
        assert!(!pyc_program.code_object.co_names.is_empty());
        assert!(!pyc_program.code_object.co_consts.is_empty());
    }
}
