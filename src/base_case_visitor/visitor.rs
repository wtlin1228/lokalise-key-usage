use std::path::Path;
use swc_core::{
    common::{sync::Lrc, Globals, Mark, SourceMap, GLOBALS},
    ecma::{
        ast::*,
        transforms::base::resolver,
        visit::{FoldWith, Visit, VisitWith},
    },
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

use super::labels::{collect_labels_from_object_literal, LABELS};

pub struct BaseCaseVisitor {
    labels: Option<LABELS>,
}

impl BaseCaseVisitor {
    pub fn new() -> Self {
        Self { labels: None }
    }
}

impl Visit for BaseCaseVisitor {
    // 1. find `const LABELS = translate({ /* ... */ })`
    // 2. extract the inner object
    fn visit_var_decl(&mut self, node: &VarDecl) {
        for var_declarator in node.decls.iter() {
            match labels_translate_args(var_declarator) {
                Some(args) => {
                    if args.len() == 0 {
                        panic!("translate should have at least 1 argument");
                    }
                    let first_arg = &args[0];
                    match &*first_arg.expr {
                        Expr::Object(object_lit) => {
                            println!("{:#?}", object_lit);
                            self.labels = Some(
                                collect_labels_from_object_literal(object_lit)
                                    .expect("collect labels from the object literal"));
                            println!("{:#?}", self.labels);
                        },
                        _ => panic!("const LABELS = translate() should take an object as its first argument"),
                    }
                }
                None => continue,
            }
        }
    }
}

fn labels_translate_args(decl: &VarDeclarator) -> Option<&Vec<ExprOrSpread>> {
    match &decl.name {
        Pat::Ident(binding_ident) => {
            if binding_ident.id.sym != "LABELS" {
                return None;
            }
            match &decl.init {
                Some(init) => match &**init {
                    Expr::Call(call_expr) => match &call_expr.callee {
                        Callee::Expr(expr) => match &**expr {
                            Expr::Ident(ident) => match ident.sym == "translate" {
                                true => Some(&call_expr.args),
                                false => None,
                            },
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                None => None,
            }
        }
        _ => None,
    }
}

pub fn parse_module(module_path: &str) {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm
        .load_file(Path::new(module_path))
        .expect(format!("failed to load {:?}", module_path).as_str());
    let module = match parse_file_as_module(
        &fm,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            no_early_errors: true,
            ..Default::default()
        }),
        EsVersion::latest(),
        None,
        &mut Vec::new(),
    ) {
        Ok(v) => v,
        Err(..) => panic!("failed to parse {:?}", module_path),
    };

    GLOBALS.set(&Globals::new(), move || {
        // This is how swc manages identifiers. ref: https://rustdoc.swc.rs/swc_ecma_transforms/fn.resolver.html
        let module = module.fold_with(&mut resolver(Mark::new(), Mark::new(), true));
        let mut visitor = BaseCaseVisitor::new();
        module.visit_with(&mut visitor);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_base_case() {
        parse_module("./fixtures/base-case.js");
    }
}
