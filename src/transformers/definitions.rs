use std::collections::HashMap;

use swc_common::DUMMY_SP;
use swc_core::ecma::ast::{Expr, Lit, MemberExpr, Null, Str};
use swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith};

pub struct MetaDefinitionsTransformer<'a> {
    definitions: &'a HashMap<String, String>,
}

impl<'a> MetaDefinitionsTransformer<'a> {
    pub fn new(definitions: &'a HashMap<String, String>) -> Self {
        Self { definitions }
    }
}

impl<'a> VisitMut for MetaDefinitionsTransformer<'a> {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Member(MemberExpr { obj, prop, .. }) => {
                let Some(MemberExpr { obj: member_obj, prop: member_prop, .. }) = obj.as_member() else {
                    return;
                };
                let Some(name) = member_prop.as_ident() else {
                    return;
                };
                if !(member_obj.is_meta_prop() && name.sym.as_ref() == "definition") {
                    return;
                }
                let Some(declaration) = prop.as_ident() else {
                    return;
                };
                *expr = match self.definitions.get(&declaration.sym.to_string()) {
                    Some(string) => Expr::Lit(Lit::Str(Str::from(string.as_ref()))),
                    None => Expr::Lit(Lit::Null(Null { span: DUMMY_SP })),
                }
            }
            _ => expr.visit_mut_children_with(self),
        }
    }
}
