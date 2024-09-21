use std::path::Path;

use swc_core::{
    common::{sync::Lrc, Globals, Mark, SourceMap, GLOBALS},
    ecma::{
        ast::*,
        minifier::{eval::Evaluator, marks::Marks},
        transforms::base::resolver,
        visit::{FoldWith, Visit, VisitWith},
    },
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

/// Deal with those cases:
///
/// ```js
/// const a = 'foo';
/// const b = translate(a);
/// ```
///
/// ```js
/// import a from 'the/import/path';
/// const b = translate(a);
/// ```
///
/// Not deal with those cases:
///
/// ```js
/// import a from 'the/import/path';
/// const temp = a;
/// const b = translate(temp);
/// ```
pub struct TranslateVisitor {
    evaluator: Evaluator,
}

impl TranslateVisitor {
    pub fn new(module: &Module) -> Self {
        Self {
            evaluator: Evaluator::new(module.clone(), Marks::new()),
        }
    }
}

impl Visit for TranslateVisitor {
    fn visit_module(&mut self, node: &Module) {
        println!("{:#?}", node);

        node.visit_children_with(self);
    }
    fn visit_call_expr(&mut self, node: &CallExpr) {
        if node.args[0].spread.is_none() {
            let eval_target = &node.args[0].expr;
            match self.evaluator.eval(eval_target) {
                Some(result) => {
                    println!("✅ {:#?} is evaluated to {:#?}", eval_target, result);
                }
                None => {
                    println!("❌ {:#?}", eval_target);

                    // if the identifier is a symbol imported from other modules
                    // 1. parse that module
                    // 2. inline that symbol
                    // 3. try to evaluate again
                }
            }
        }
    }
}

const PATH: &'static str = "./fixtures/index.js";

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm
        .load_file(Path::new(PATH))
        .expect(format!("failed to load {:?}", PATH).as_str());

    let module = match parse_file_as_module(
        &fm,
        Syntax::Typescript(TsSyntax {
            tsx: PATH.ends_with("tsx") || PATH.ends_with("ts"),
            decorators: true,
            no_early_errors: true,
            ..Default::default()
        }),
        EsVersion::latest(),
        None,
        &mut Vec::new(),
    ) {
        Ok(v) => v,
        // We are not testing parser
        Err(..) => panic!("failed to parse {:?}", PATH),
    };

    GLOBALS.set(&Globals::new(), move || {
        // This is how swc manages identifiers. ref: https://rustdoc.swc.rs/swc_ecma_transforms/fn.resolver.html
        let module = module.fold_with(&mut resolver(Mark::new(), Mark::new(), true));

        let mut translate_visitor = TranslateVisitor::new(&module);

        module.visit_with(&mut translate_visitor);
    });
}
