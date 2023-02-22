use swc_core::ecma::ast::{
    ArrayLit, CallExpr, Decorator, ExprOrSpread, FnDecl, Ident, Lit, ObjectLit, PropName,
};

const LINKAGE_DECORATOR_NAME: &'static str = "linkage";

#[derive(Debug)]
pub struct FunctionLinkage {
    pub name: String,
    pub is_async: bool,
    pub namespace: String,
    pub dependencies: Vec<String>,
}

impl FunctionLinkage {
    pub fn from(decl: &mut FnDecl) -> Option<Self> {
        let Some(i) = decl.function.decorators.iter().position(get_linkage_decorator_position) else { return None };
        let decorator = decl.function.decorators.swap_remove(i);
        let Some(CallExpr { mut args, .. }) = decorator.expr.call() else { return None };
        if args.len() < 1 {
            return None;
        }
        let arg = args.swap_remove(0);
        let Some(object) = arg.expr.object() else { return None };
        Self::from_linkage_object(decl.ident.sym.to_string(), decl.function.is_async, &object)
    }

    pub fn from_linkage_object(
        name: String,
        is_async: bool,
        ObjectLit { props, .. }: &ObjectLit,
    ) -> Option<Self> {
        let mut namespace: Option<String> = None;
        let mut dependencies: Option<Vec<String>> = None;
        for prop in props {
            let Some(prop) = prop.as_prop() else { continue };
            let Some(kv) = prop.as_key_value() else { continue };
            let PropName::Ident(Ident {
                ref sym,
                ..
            }) = kv.key else { continue };
            match sym.as_ref() {
                "namespace" => {
                    let Some(Lit::Str(str)) = kv.value.as_lit() else { panic!("invalid value passed to namespace") };
                    namespace = Some(str.value.to_string());
                }
                "dependencies" => {
                    let Some(ArrayLit { elems, .. }) = kv.value.as_array() else { panic!("expected array") };
                    dependencies = Some(
                    elems
                        .iter()
                        .filter_map(|elem| {
                            let Some(ExprOrSpread { expr, .. }) = elem else { return None };
                            let Some(Lit::Str(str)) = expr.as_lit() else { panic!("invalid dependency") };
                            return Some(str.value.to_string());
                        })
                        .collect::<Vec<String>>(),
                );
                }
                _ => (),
            }
        }
        return Some(FunctionLinkage {
            namespace: namespace.unwrap_or("env".to_owned()),
            dependencies: dependencies.unwrap_or_default(),
            is_async,
            name,
        });
    }
}

fn get_linkage_decorator_position(decorator: &Decorator) -> bool {
    let Some(CallExpr { ref callee, .. }) = decorator.expr.as_call() else { return false };
    let Some(Ident { sym, .. }) = callee.as_expr().and_then(|expr| expr.as_ident()) else { return false };
    return sym.as_ref() == LINKAGE_DECORATOR_NAME;
}
