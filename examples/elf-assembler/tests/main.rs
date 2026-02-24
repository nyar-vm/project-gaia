//! ELF 汇编器测试套件
//!
//! 这个文件包含基本的测试入口，具体的测试分布在不同的模块中：
//! - hello_world_tests: Hello World ELF 文件生成测试
//! - exit_code_tests: Exit Code ELF 文件生成测试  
//! - execution_tests: ELF 文件实际执行测试

mod linux;

mod integration_gaia;
mod runnable;

#[test]
fn ready() {
    println!("ELF Assembler test suite ready!");
}
