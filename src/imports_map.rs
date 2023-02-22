use crate::{library::Library, wasm::ModuleImport};
use std::collections::{hash_map::Entry, HashMap, HashSet};

pub struct ImportsMap {
    pub symbols: HashSet<(String, String)>,
    pub missing_symbols: HashSet<(String, String)>,
}

impl ImportsMap {
    pub fn new(imports: Vec<ModuleImport>, libraries: Vec<Library>) -> (Self, Vec<Library>) {
        let mut symbols: HashSet<(String, String)> = HashSet::new();
        let mut missing_symbols: HashSet<(String, String)> = HashSet::new();

        let imports = imports
            .into_iter()
            .filter_map(|x| {
                let Some(name) = x.name else { return None };
                Some((x.namespace, name))
            })
            .collect::<HashSet<(String, String)>>();

        let libraries = libraries
            .into_iter()
            .filter(|x| {
                imports
                    .iter()
                    .position(|(namespace, name)| x.contains_symbol(namespace, name))
                    .is_some()
            })
            .collect::<Vec<Library>>();

        for symbol in imports {
            let mut found = false;
            'library_loop: for library in libraries.iter() {
                if library.contains_symbol(&symbol.0, &symbol.1) {
                    found = true;
                    break 'library_loop;
                }
                found = false;
            }
            let target = if found {
                &mut symbols
            } else {
                &mut missing_symbols
            };
            target.insert(symbol);
        }

        (
            Self {
                symbols,
                missing_symbols,
            },
            libraries,
        )
    }

    pub fn to_string(self) -> String {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for (namespace, name) in self.symbols {
            match map.entry(namespace) {
                Entry::Vacant(e) => {
                    e.insert(vec![name]);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().push(name);
                }
            }
        }
        let imports_string = map
            .iter()
            .map(|(key, value)| format!("{key}:{{{}}}", value.join(",")))
            .collect::<Vec<String>>()
            .join(",");
        format!("{{{imports_string}}}")
    }
}
