# Waldo
A WebAssembly import dependency linker.

## Goals
- Simplify the process of managing imports.
- Be toolchain-agnostic.
- Create a clean, simple, glue code that – itself – is engine-agnostic.
- Reduce code duplication in instances where you need multiple WebAssembly modules
- Define import libraries in JavaScript.
- Export typings for your module's glue code and exports.

## Usage
```
waldo [OPTIONS] --output <OUTPUT> <PATH>

Arguments:
  <PATH>  The path to the WebAssembly file to find imports for

Options:
  -L, --lib <LIBRARIES>       Add a library to look for imports in
  -D, --define <DEFINITIONS>  Define a variable for the libraries to use (used by `import.meta.definitions.name`)
  -o, --output <OUTPUT>       The path to write the generated glue code to
  -h, --help                  Print help
  -V, --version               Print version
```

## Defining a library
```ts
@linkage({ namespace: "stdio" })
function puts(stringPointer: Pointer) {
	const string = derefSentielString(stringPointer, 0);
	console.log(string);
}
```

