use super::labels::{collect_labels_from_object_literal, LABELS};
use crate::anonymous_default_export::get_anonymous_default_export_id;
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

struct LabelVisitor {
    labels: Option<LABELS>,
}

impl LabelVisitor {
    pub fn new() -> Self {
        Self { labels: None }
    }
}

impl Visit for LabelVisitor {
    // TODO: only handle top level LABELS for this case
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
                            self.labels = Some(
                                collect_labels_from_object_literal(object_lit)
                                    .expect("collect labels from the object literal"));
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

struct LabelUsageVisitor {
    // current_id is used to track which identifier is using the LABELS
    current_id: Option<Id>,

    // labels is extracted by the LabelVisitor
    labels: LABELS,
}

impl LabelUsageVisitor {
    pub fn new(labels: LABELS) -> Self {
        Self {
            current_id: None,
            labels,
        }
    }
}

impl Visit for LabelUsageVisitor {
    fn visit_member_expr(&mut self, node: &MemberExpr) {
        if is_labels_obj(node) {
            // println!("{:#?}", self.current_id);
            // println!("{:#?}", node);
            let translation_keys = self.labels.get_translation_keys_for_member_expr(node);
            println!("{:#?}", translation_keys);
        }
    }

    fn visit_module(&mut self, n: &Module) {
        println!("{:#?}", n);
        for module_item in &n.body {
            match module_item {
                ModuleItem::ModuleDecl(module_decl) => match module_decl {
                    ModuleDecl::ExportDecl(ExportDecl { decl, .. }) => match decl {
                        // export class Foo {}
                        Decl::Class(ClassDecl { ident, class, .. }) => {
                            self.current_id = Some(ident.to_id());
                            class.visit_with(self);
                            self.current_id = None;
                        }
                        // export function foo() {}
                        Decl::Fn(FnDecl {
                            ident, function, ..
                        }) => {
                            self.current_id = Some(ident.to_id());
                            function.visit_with(self);
                            self.current_id = None;
                        }
                        // export const foo = init, bar = init
                        Decl::Var(var_decl) => {
                            for var_decl in &var_decl.decls {
                                match &var_decl.name {
                                    Pat::Ident(BindingIdent { id, .. }) => {
                                        self.current_id = Some(id.to_id());
                                        var_decl.init.visit_with(self);
                                        self.current_id = None;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    },
                    ModuleDecl::ExportDefaultDecl(ExportDefaultDecl { decl, .. }) => match decl {
                        DefaultDecl::Class(ClassExpr { ident, class }) => match ident {
                            // export default class ClassName { /* … */ }
                            Some(ident) => {
                                self.current_id = Some(ident.to_id());
                                class.visit_with(self);
                                self.current_id = None;
                            }
                            // export default class { /* … */ }
                            None => {
                                self.current_id = Some(get_anonymous_default_export_id());
                                class.visit_with(self);
                                self.current_id = None;
                            }
                        },
                        DefaultDecl::Fn(FnExpr { ident, function }) => match ident {
                            // export default function functionName() { /* … */ }
                            Some(ident) => {
                                self.current_id = Some(ident.to_id());
                                function.visit_with(self);
                                self.current_id = None;
                            }
                            // export default function () { /* … */ }
                            None => {
                                self.current_id = Some(get_anonymous_default_export_id());
                                function.visit_with(self);
                                self.current_id = None;
                            }
                        },
                        DefaultDecl::TsInterfaceDecl(_) => (),
                    },
                    ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) => {
                        match &**expr {
                            // export default name1;
                            Expr::Ident(_) => (),
                            // export default [name1, name2];
                            Expr::Array(array_lit) => {
                                self.current_id = Some(get_anonymous_default_export_id());
                                array_lit.visit_with(self);
                                self.current_id = None;
                            }
                            // export default { name1, name2 };
                            Expr::Object(object_lit) => {
                                self.current_id = Some(get_anonymous_default_export_id());
                                object_lit.visit_with(self);
                                self.current_id = None;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                },
                ModuleItem::Stmt(stmt) => match stmt {
                    Stmt::Decl(decl) => match decl {
                        // class Foo {}
                        Decl::Class(ClassDecl { ident, class, .. }) => {
                            self.current_id = Some(ident.to_id());
                            class.visit_with(self);
                            self.current_id = None;
                        }
                        // function foo() {}
                        Decl::Fn(FnDecl {
                            ident, function, ..
                        }) => {
                            self.current_id = Some(ident.to_id());
                            function.visit_with(self);
                            self.current_id = None;
                        }
                        // const foo = init, bar = init;
                        Decl::Var(var_decl) => {
                            for var_decl in &var_decl.decls {
                                match &var_decl.name {
                                    Pat::Ident(BindingIdent { id, .. }) => {
                                        self.current_id = Some(id.to_id());
                                        var_decl.init.visit_with(self);
                                        self.current_id = None;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
            }
        }
    }
}

fn is_labels_obj(member_expr: &MemberExpr) -> bool {
    let mut obj: &Box<Expr> = &member_expr.obj;

    // find the ident by following the obj path, once found, check if it's sym is "LABELS"
    loop {
        match &**obj {
            Expr::Member(member_expr) => {
                obj = &member_expr.obj;
            }
            Expr::Ident(ident) => return ident.sym == "LABELS",
            _ => return false,
        }
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

        let mut label_visitor = LabelVisitor::new();
        module.visit_with(&mut label_visitor);

        if let Some(labels) = label_visitor.labels {
            let mut label_usage_visitor = LabelUsageVisitor::new(labels);
            module.visit_with(&mut label_usage_visitor);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_base_case() {
        parse_module("./src/base_case_visitor/fixtures/base-case.js");
    }
}
