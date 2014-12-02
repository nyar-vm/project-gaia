use crate::{formats::wat::ast::{WatRoot, SourceLocation, WatCoreFuncType, WatCoreModuleItem, WatItem, WatModule, WatCoreExport, WatCoreExportItem}, program::WasiProgram};
use gaia_types::{GaiaDiagnostics, GaiaError};
use std::collections::HashMap;

impl WasiProgram {
    pub fn to_wat(self) -> GaiaDiagnostics<WatRoot> {
        let mut state =
            Wasi2Wat { wat: WatRoot { items: vec![], location: SourceLocation::default() }, index_to_name: Default::default(), name_counter: 0, errors: vec![] };
        match state.convert(self) {
            Ok(_) => GaiaDiagnostics { result: Ok(state.wat), diagnostics: state.errors },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: state.errors },
        }
    }
}

struct Wasi2Wat {
    wat: WatRoot,
    // 用于生成名称的映射
    index_to_name: HashMap<u32, String>,
    // 名称计数器
    name_counter: u32,
    errors: Vec<GaiaError>,
}

impl Wasi2Wat {
    fn convert(&mut self, program: WasiProgram) -> Result<(), GaiaError> {
        // 初始化WAT模块
        let mut module = WatModule {
            name: program.name.clone(),
            items: vec![],
            location: SourceLocation::default(),
        };
        
        // 添加函数定义
        for func in program.functions {
            if func.type_index as usize >= program.function_types.len() {
                return Err(GaiaError::invalid_range(func.type_index as usize, program.function_types.len()));
            }
            let func_type = &program.function_types[func.type_index as usize];
            let wat_func = crate::formats::wat::ast::WatCoreFunc {
                name: None,
                func_type: WatCoreFuncType {
                    params: func_type.params.iter().map(|t| (*t).into()).collect(),
                    results: func_type.results.iter().map(|t| (*t).into()).collect(),
                    location: SourceLocation::default(),
                },
                body: Some(func.body.into_iter().map(|i| i.into()).collect()),
                canon: None,
                location: SourceLocation::default(),
            };
            module.items.push(WatCoreModuleItem::Func(wat_func));
        }
        
        // 添加导出
        for export in program.exports {
            let func_index = export.export_type.function_index().ok_or_else(|| GaiaError::custom_error(format!("导出 {} 不是函数类型", export.name)))?;
            module.items.push(WatCoreModuleItem::Export(WatCoreExport {
                name: export.name,
                export_item: WatCoreExportItem::Func(func_index.to_string()),
                location: SourceLocation::default(),
            }));
        }
        
        self.wat.items.push(WatItem::Module(module));
        Ok(())
    }
}
