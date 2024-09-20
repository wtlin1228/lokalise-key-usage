use std::path::Path;

use swc_core::{
    common::{
        errors::{ColorConfig, Handler},
        sync::Lrc,
        FileName, Globals, Mark, SourceMap, GLOBALS,
    },
    ecma::{
        ast::*,
        minifier::{eval::Evaluator, marks::Marks},
        transforms::base::resolver,
        visit::{FoldWith, Visit, VisitWith},
    },
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

pub struct TranslateVisitor {
    evaluator: Evaluator,
}

impl TranslateVisitor {
    pub fn new(module: &mut Module) -> Self {
        Self {
            evaluator: Evaluator::new(module.clone(), Marks::new()),
        }
    }
}

impl Visit for TranslateVisitor {
    fn visit_module(&mut self, node: &Module) {
        // println!("{:#?}", node);
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
                }
            }
        }
    }
}

const CODE: &'static str = r#"
const a = 'foo';
const b = translate(a);
"#;

const PATH: &'static str = "./fixtures/index.js";

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // let fm = cm.new_source_file(Lrc::new(FileName::Custom("test.js".into())), CODE.into());

    let fm = cm
        .load_file(Path::new(PATH))
        .expect(format!("failed to load {:?}", PATH).as_str());

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parse module");

    GLOBALS.set(&Globals::new(), move || {
        let mut module = module.fold_with(&mut resolver(Mark::new(), Mark::new(), true));

        let mut translate_visitor = TranslateVisitor::new(&mut module);

        module.visit_with(&mut translate_visitor);
    });
}
