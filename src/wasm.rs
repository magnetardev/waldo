use anyhow::Result;
use std::{fs::File, io::Read, usize, vec};
use wasmparser::{
    Export, ExternalKind, Import, ImportSectionEntryType, Parser, Payload, Type, TypeDef,
};

#[derive(Debug)]
pub struct Module {
    pub imports: Vec<ModuleImport>,
    pub exports: Vec<ModuleExport>,
}

#[derive(Debug)]
pub struct ModuleImport {
    pub namespace: String,
    pub name: Option<String>,
    pub ty: ModuleType,
}

#[derive(Debug)]
pub struct ModuleExport {
    pub name: String,
    pub ty: ModuleType,
}

#[derive(Debug, Clone)]
pub enum ModuleType {
    Func(Vec<ModuleType>, Box<ModuleType>),
    I32,
    I64,
    F32,
    F64,
    V128,
    Memory,
    Table,
    Void,
    Unknown,
}

impl Module {
    pub fn new(mut file: File) -> Result<Self, anyhow::Error> {
        let mut types: Vec<ModuleType> = vec![];

        // read wasm file
        let mut parser_exports: Vec<Export> = vec![];
        let mut parser_imports: Vec<Import> = vec![];
        let mut functions: Vec<u32> = vec![];
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        for payload in Parser::new(0).parse_all(&bytes) {
            match payload? {
                Payload::ExportSection(s) => {
                    for export in s {
                        let export = export?;
                        parser_exports.push(export);
                    }
                }
                Payload::ImportSection(s) => {
                    for import in s {
                        let import = import?;
                        parser_imports.push(import);
                    }
                }
                Payload::TypeSection(s) => {
                    for ty in s {
                        let ty = ty?;
                        let typedef = ModuleType::from_wp_typedef(ty);
                        types.push(typedef);
                    }
                }
                Payload::FunctionSection(s) => {
                    for func in s {
                        let func = func?;
                        functions.push(func);
                    }
                }
                _ => (),
            }
        }

        // Get imports and exports
        let imports: Vec<ModuleImport> = parser_imports
            .iter()
            .map(|import| ModuleImport {
                namespace: import.module.to_owned(),
                name: import.field.map(|x| x.to_owned()),
                ty: match import.ty {
                    ImportSectionEntryType::Table(..) => ModuleType::Table,
                    ImportSectionEntryType::Memory(..) => ModuleType::Memory,
                    ImportSectionEntryType::Function(idx) => {
                        if let Some(fn_type) = types.get((idx) as usize) {
                            fn_type.clone()
                        } else {
                            ModuleType::Unknown
                        }
                    }
                    _ => ModuleType::Unknown,
                },
            })
            .collect();
        let exports: Vec<ModuleExport> = parser_exports
            .iter()
            .map(|export| ModuleExport {
                name: export.field.to_owned(),
                ty: match export.kind {
                    ExternalKind::Table => ModuleType::Table,
                    ExternalKind::Memory => ModuleType::Memory,
                    ExternalKind::Function => {
                        if let Some(fn_type) = types.get((export.index - 1) as usize) {
                            fn_type.clone()
                        } else {
                            ModuleType::Unknown
                        }
                    }
                    _ => ModuleType::Unknown,
                },
            })
            .collect();

        // return
        Ok(Self { imports, exports })
    }
}

impl ModuleType {
    // pub fn as_typescript(&self) -> String {
    //     use ModuleType::*;
    //     match self {
    //         Memory => "WebAssembly.Memory".to_owned(),
    //         Table => "WebAssembly.Table".to_owned(),
    //         I32 | F32 | I64 | F64 | V128 => "number".to_owned(),
    //         Void => "void".to_owned(),
    //         Func(args, ret) => {
    //             let mut arg_idx = 0;
    //             let args_vec: Vec<String> = args
    //                 .iter()
    //                 .map(|x| {
    //                     arg_idx += 1;
    //                     format!("arg{}: {}", arg_idx, x.as_typescript())
    //                 })
    //                 .collect();
    //             format!("({}) => {}", args_vec.join(", "), ret.as_typescript())
    //         }
    //         _ => "unknown".to_owned(),
    //     }
    // }

    pub fn from_wp_type(ty: &Type) -> Self {
        match ty {
            Type::I32 => Self::I32,
            Type::F32 => Self::F32,
            Type::I64 => Self::I64,
            Type::F64 => Self::F64,
            Type::V128 => Self::V128,
            _ => Self::Unknown,
        }
    }

    pub fn from_wp_typedef(typedef: TypeDef) -> Self {
        match typedef {
            TypeDef::Func(x) => {
                let args = x.params.iter().map(|x| Self::from_wp_type(x)).collect();
                let ret = x
                    .returns
                    .first()
                    .map_or(Self::Void, |x| Self::from_wp_type(x));
                Self::Func(args, Box::new(ret))
            }
            _ => Self::Unknown,
        }
    }
}
