use crate::test_tools::test_path;
use gaia_types::{helpers::save_json, GaiaError};
use python_assembler::formats::pyc::{pyc_read_path, PycReadConfig};
use std::process::Command;
use walkdir::WalkDir;

#[test]
fn read_all_pyc_files() -> Result<(), GaiaError> {
    // 使用当前系统中激活的 python 重新生成 pyc 文件
    let output = Command::new("python").arg(test_path("pyc_read/generate_pyc.py")).output();
    // 即便 pyc 生成出错，也仍然解析其他版本的 pyc 文件
    if let Err(e) = output {
        eprintln!("运行 python 出错： {}", e)
    };

    let pyc_dir = test_path("pyc_read");

    for entry in WalkDir::new(pyc_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "pyc"))
    {
        let path = entry.path();
        let config = PycReadConfig::default();
        let diagnostics = pyc_read_path(path, &config);
        match diagnostics.result {
            Ok(program) => {
                println!("Reading {:?}, Detected Python version: {:?}", path, program.version);
                let json_path = path.with_extension("json");
                println!("Attempting to save JSON to: {:?}", json_path);
                match save_json(&program, &json_path) {
                    Ok(_) => println!("Successfully saved JSON to: {:?}", json_path),
                    Err(e) => eprintln!("Error saving JSON to {:?}: {:?}", json_path, e),
                }
            }
            Err(_) => {
                eprintln!("Error reading {:?}: {:?}", path, diagnostics.diagnostics);
            }
        }
    }
    Ok(())
}
