use gaia_types::writer::TextWriter;
use llvm_assembler::{LLvmProgram, LLvmProgramBuilder, LLvmWriter};
use oak_core::source::ToSource;
use oak_llvm_ir::ast::{LLirBlock, LLirFunction, LLirInstruction, LLirItem, LLirParameter, LLirRoot};

#[test]
fn test_llvm_builder() {
    let program = LLvmProgramBuilder::new()
        .add_global("version", "i32", "1", true)
        .add_function("main", "i32", |f| {
            f.add_parameter("argc", "i32").add_block("entry", |b| {
                b.add("1", "i32", "%argc", "1")
                    .ret("i32", "%1")
            })
        })
        .build();

    let mut writer = LLvmWriter::new(TextWriter::new(String::new()));
    writer.write_ast(&program.root).unwrap();
    let source = writer.finish();
    assert_eq!(
        source,
        "@version = constant i32 1\ndefine i32 @main(i32 %argc) {\nentry:\n  %1 = add i32 %argc, 1\n  ret i32 %1\n}\n"
    );
}

#[test]
fn test_llvm_to_source() {
    let root = LLirRoot {
        items: vec![LLirItem::Function(LLirFunction {
            name: "main".to_string(),
            return_type: "i32".to_string(),
            parameters: vec![LLirParameter {
                name: "argc".to_string(),
                ty: "i32".to_string(),
            }],
            blocks: vec![LLirBlock {
                label: Some("entry".to_string()),
                instructions: vec![
                    LLirInstruction {
                        result: Some("1".to_string()),
                        opcode: "add".to_string(),
                        operands: vec!["i32 %argc".to_string(), "1".to_string()],
                    },
                    LLirInstruction {
                        result: None,
                        opcode: "ret".to_string(),
                        operands: vec!["i32 %1".to_string()],
                    },
                ],
            }],
            span: (0..0).into(),
        })],
        span: (0..0).into(),
    };

    let mut writer = LLvmWriter::new(TextWriter::new(String::new()));
    writer.write_ast(&root).unwrap();
    let source = writer.finish();
    assert_eq!(source, "define i32 @main(i32 %argc) {\nentry:\n  %1 = add i32 %argc, 1\n  ret i32 %1\n}\n");
}

#[test]
fn test_llvm_to_doc() {
    let root = LLirRoot {
        items: vec![LLirItem::Function(LLirFunction {
            name: "main".to_string(),
            return_type: "i32".to_string(),
            parameters: vec![LLirParameter {
                name: "argc".to_string(),
                ty: "i32".to_string(),
            }],
            blocks: vec![LLirBlock {
                label: Some("entry".to_string()),
                instructions: vec![
                    LLirInstruction {
                        result: Some("1".to_string()),
                        opcode: "add".to_string(),
                        operands: vec!["i32 %argc".to_string(), "1".to_string()],
                    },
                    LLirInstruction {
                        result: None,
                        opcode: "ret".to_string(),
                        operands: vec!["i32 %1".to_string()],
                    },
                ],
            }],
            span: (0..0).into(),
        })],
        span: (0..0).into(),
    };

    let mut writer = LLvmWriter::new(TextWriter::new(String::new()));
    writer.write_doc(&root).unwrap();
    let doc = writer.finish();
    // 检查文档输出中包含关键内容
    assert!(doc.contains("define i32 @main(i32 %argc) {"));
    assert!(doc.contains("entry:"));
    assert!(doc.contains("%1 = add i32 %argc, 1"));
    assert!(doc.contains("ret i32 %1"));
}

#[test]
fn test_llvm_reader() {
    let source = "@version = constant i32 1\ndefine i32 @main(i32 %argc) {\nentry:\n  %1 = add i32 %argc, 1\n  ret i32 %1\n}\n";
    let program = LLvmProgram::from_source(source).unwrap();
    
    assert_eq!(program.root.items.len(), 2);
    
    match &program.root.items[0] {
        LLirItem::Global(g) => {
            assert_eq!(g.name, "version");
            assert_eq!(g.is_constant, true);
        }
        _ => panic!("Expected global variable"),
    }
    
    match &program.root.items[1] {
        LLirItem::Function(f) => {
            assert_eq!(f.name, "main");
            assert_eq!(f.return_type, "i32");
            assert_eq!(f.parameters.len(), 1);
            assert_eq!(f.blocks.len(), 1);
            let block = &f.blocks[0];
            assert_eq!(block.label, Some("entry".to_string()));
            assert_eq!(block.instructions.len(), 2);
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_llvm_roundtrip() {
    let source = "@version = constant i32 1\n\ndefine i32 @main(i32 %argc) {\nentry:\n  %1 = add i32 %argc, 1\n  ret i32 %1\n}\n";
    let program1 = LLvmProgram::from_source(source).unwrap();
    let source1 = program1.root.to_source_string();
    let program2 = LLvmProgram::from_source(&source1).unwrap();
    
    assert_eq!(program1.root.items.len(), program2.root.items.len());
    
    // Compare global variable
    if let (LLirItem::Global(g1), LLirItem::Global(g2)) = (&program1.root.items[0], &program2.root.items[0]) {
        assert_eq!(g1.name, g2.name);
        assert_eq!(g1.ty, g2.ty);
        assert_eq!(g1.value, g2.value);
        assert_eq!(g1.is_constant, g2.is_constant);
    } else {
        panic!("Expected global variables");
    }
    
    // Compare function
    if let (LLirItem::Function(f1), LLirItem::Function(f2)) = (&program1.root.items[1], &program2.root.items[1]) {
        assert_eq!(f1.name, f2.name);
        assert_eq!(f1.return_type, f2.return_type);
        assert_eq!(f1.parameters.len(), f2.parameters.len());
        assert_eq!(f1.blocks.len(), f2.blocks.len());
    } else {
        panic!("Expected functions");
    }
}
