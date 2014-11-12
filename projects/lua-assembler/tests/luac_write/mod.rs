use lua_assembler::{
    formats::luac::LuacHeader,
    program::{LuaProgram, LuacBuilder},
};

#[test]
fn builder_write_timestamp_and_exec() {
    // 构造简单程序：打印一行
    let default_header = LuacHeader { magic: 0, flags: 0, timestamp: None, size: None, hash: None };
    let program: LuaProgram = LuacBuilder::new().print_str("hello from builder").build(default_header);

    let out_path = std::path::Path::new("tests/luac_write/out_builder_ts.pyc");
    lua_assembler::formats::luac::writer::write_program(out_path, &program).expect("write .pyc via timestamp mode");

    let output = std::process::Command::new("lua").arg(out_path).output().expect("run lua for timestamp pyc");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello from builder"), "unexpected stdout: {}", stdout);
}

#[test]
fn builder_write_hash_and_exec() {
    let default_header = LuacHeader { magic: 0, flags: 1, timestamp: None, size: None, hash: Some([0; 8]) };
    let program: LuaProgram = LuacBuilder::new().print_str("hello from builder hash").build(default_header);

    let out_path = std::path::Path::new("tests/luac_write/out_builder_hash.pyc");
    lua_assembler::formats::luac::writer::write_program(out_path, &program).expect("write .pyc via hash mode");

    let output = std::process::Command::new("lua").arg(out_path).output().expect("run lua for hash pyc");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello from builder hash"), "unexpected stdout: {}", stdout);
}
