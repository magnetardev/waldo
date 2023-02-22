mod imports_map;
mod library;
mod linkage;
mod transformers;
mod wasm;

use anyhow::Result;
use clap::Parser as ClapParser;
use imports_map::ImportsMap;
use library::Library;
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};
use swc::{self, config::Options};
use transformers::definitions::MetaDefinitionsTransformer;
use wasm::Module;

/// A WebAssembly import dependency linker
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Add a library to look for imports in
    #[arg(short = 'L', long = "lib")]
    libraries: Vec<PathBuf>,
    /// Define a variable for the libraries to use (used by `import.meta.definitions.name`)
    #[arg(short = 'D', long = "define")]
    definitions: Vec<String>,
    /// The path to write the generated glue code to
    #[arg(short = 'o', long = "output")]
    output: PathBuf,
    /// Generate a .d.ts file
    #[arg(short = 't', long = "typings")]
    generate_typings: bool,
    /// The path to the WebAssembly file to find imports for
    #[arg(value_name = "PATH")]
    wasm: PathBuf,
}

fn main() -> Result<()> {
    let mut args = Args::parse();

    let mut definitions: HashMap<String, String> = HashMap::new();
    for def in args.definitions {
        let Some((k, v)) = def.split_once('=') else { continue };
        definitions.insert(k.to_owned(), v.to_owned());
    }

    let file = File::open(&args.wasm)?;
    let wasm_module = Module::new(file)?;

    let mut libraries: Vec<Library> = vec![];
    for path in args.libraries {
        let library = Library::new(path)?;
        libraries.push(library);
    }

    let (imports, libraries) = ImportsMap::new(wasm_module.imports, libraries);

    let mut declaration_transformer = MetaDefinitionsTransformer::new(&definitions);
    let compiler_options = Options {
        ..Default::default()
    };

    let mut output_writer = File::create(&args.output)?;
    for library in libraries {
        library.write_to_output(
            &mut output_writer,
            &compiler_options,
            &mut declaration_transformer,
        )?;
    }

    writeln!(
        &mut output_writer,
        r#"export async function instantiate(source, missingImports) {{
    const imports = {};
	if (missingImports) {{
		for (const [key, value] in Object.entries(missingImports)) {{
			let object = imports[key];
			if (!object) {{
				imports[key] = value;
			}} else {{
				Object.assign(object, value);
			}}
		}}
	}}

	if (source instanceof Promise) {{
		source = await source;
	}}

	if ("Response" in globalThis && source instanceof Response) {{
		if ("instantiateStreaming" in WebAssembly) {{
			return WebAssembly.instantiateStreaming(source, imports);
		}}
		source = await source.arrayBuffer()
	}}

	return new WebAssembly.Instance(source instanceof WebAssembly.Module ? source : new WebAssembly.Module(source), imports);
}}"#,
        imports.to_string(),
    )?;

    if !args.generate_typings {
        return Ok(());
    }

    let exports = wasm_module
        .exports
        .into_iter()
        .map(|x| format!("{}: {}", x.name, x.ty.as_typescript()))
        .collect::<Vec<String>>()
        .join(",");

    args.output.set_extension("d.ts");
    let mut typings_output_writer = File::create(args.output)?;

    writeln!(
        &mut typings_output_writer,
        r#"type Exports = {{{exports}}};
type WasmSource = Response | ArrayBuffer | ArrayBufferView | WebAssembly.Module;
type WasmImport = Function | WebAssembly.Global | WebAssembly.Memory | WebAssembly.Table | number;
export function instantiate(source: WasmSource | Promise<WasmSource>, missingImports?: Record<string, WasmImport>): Promise<Omit<WebAssembly.Instance, "exports"> & {{ exports: Exports }}>;
"#
    )?;

    Ok(())
}
