use anyhow::bail;
use swc_core::{
    common::{sync::Lrc, FileName, SourceMap},
    ecma::ast::*,
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

pub fn parse_module(input: &str) -> anyhow::Result<Module> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(Lrc::new(FileName::Custom("test.js".into())), input.into());
    match parse_file_as_module(
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
        Ok(module) => Ok(module),
        Err(_) => bail!("failed to parse module"),
    }
}
