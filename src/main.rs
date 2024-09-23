use anyhow::{bail, Context};
use lokalise_key_usage::core;
use std::path::Path;
use swc_core::{
    common::{sync::Lrc, Globals, Mark, SourceMap, GLOBALS},
    ecma::{ast::*, transforms::base::resolver, visit::FoldWith},
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

const PATH: &'static str = "./fixtures/DiscountSubmitModalTimeConditionFields.jsx";

fn main() -> anyhow::Result<()> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm
        .load_file(Path::new(PATH))
        .context(format!("failed to load {:?}", PATH))?;

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
        // We are not testing parser
        Err(..) => bail!("failed to parse {:?}", PATH),
    };

    // This is how swc manages identifiers. ref: https://rustdoc.swc.rs/swc_ecma_transforms/fn.resolver.html
    let module = GLOBALS.set(&Globals::new(), move || {
        module.fold_with(&mut resolver(Mark::new(), Mark::new(), true))
    });

    let translation_usage = core::collect_translation(&module)?;
    println!("{:#?}", translation_usage);
    Ok(())
}
