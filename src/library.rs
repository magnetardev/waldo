use crate::{linkage::FunctionLinkage, transformers::definitions::MetaDefinitionsTransformer};
use anyhow::{Context, Result};
use std::{fs::File, io::Write, path::PathBuf, sync::Arc};
use swc::config::Options;
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap, GLOBALS,
};
use swc_core::ecma::ast::{Decl, EsVersion, Program, Stmt};
use swc_ecma_parser::{parse_file_as_script, EsConfig, Syntax};
use swc_ecma_visit::VisitMutWith;

const ES_CONFIG: EsConfig = EsConfig {
    jsx: false,
    fn_bind: true,
    decorators: true,
    decorators_before_export: true,
    export_default_from: false,
    import_assertions: false,
    allow_super_outside_method: false,
    allow_return_outside_function: false,
};

pub struct Library {
    pub linkages: Vec<FunctionLinkage>,
    source_map: Arc<SourceMap>,
    program: Program,
    handler: Handler,
}

impl Library {
    pub fn new(path: PathBuf) -> Result<Self> {
        let source_map = Arc::<SourceMap>::default();

        let handler =
            Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
        let source_file = source_map.load_file(&path)?;

        let mut errors: Vec<swc_ecma_parser::error::Error> = vec![];
        let mut script = parse_file_as_script(
            &source_file,
            Syntax::Es(ES_CONFIG),
            EsVersion::EsNext,
            None,
            &mut errors,
        )
        .map_err(|x| anyhow::anyhow!("{:?}", x))?;

        for e in errors {
            e.into_diagnostic(&handler).emit();
        }

        let mut linkages: Vec<FunctionLinkage> = vec![];
        for statement in &mut script.body {
            if let Stmt::Decl(Decl::Fn(ref mut func)) = statement {
                if let Some(linkage) = FunctionLinkage::from(func) {
                    linkages.push(linkage);
                }
            }
        }

        Ok(Self {
            linkages,
            source_map,
            program: Program::Script(script),
            handler,
        })
    }

    pub fn write_to_output(
        mut self,
        writer: &mut File,
        compiler_options: &Options,
        declaration_transformer: &mut MetaDefinitionsTransformer,
    ) -> Result<()> {
        if let Program::Script(ref mut script) = self.program {
            script.body.visit_mut_with(declaration_transformer);
        }

        // Compile
        let compiler = swc::Compiler::new(self.source_map.clone());
        let output = GLOBALS.set(&Default::default(), || {
            compiler
                .process_js(&self.handler, self.program, &compiler_options)
                .context("failed to process file")
        })?;

        writeln!(writer, "{}", output.code)?;
        Ok(())
    }

    pub fn contains_symbol(&self, namespace: &String, name: &String) -> bool {
        for link in &self.linkages {
            if link.namespace == *namespace && link.name == *name {
                return true;
            }
        }
        return false;
    }
}
