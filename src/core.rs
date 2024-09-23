use super::base_case_visitor;
use anyhow::{bail, Context};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use swc_core::{
    common::{sync::Lrc, Globals, Mark, SourceMap, GLOBALS},
    ecma::{ast::*, transforms::base::resolver, visit::FoldWith},
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

#[derive(Debug)]
struct TranslationUsage {
    data: HashMap<String, HashSet<String>>,
}

impl TranslationUsage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn extend(&mut self, target: HashMap<String, HashSet<String>>) {
        for (key, value) in target.iter() {
            if !self.data.contains_key(key) {
                self.data.insert(key.to_owned(), HashSet::new());
            }
            self.data.entry(key.to_owned()).and_modify(|set| {
                set.extend(value.clone());
            });
        }
    }
}

pub fn collect_translation(path: &Path) -> anyhow::Result<HashMap<String, HashSet<String>>> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm
        .load_file(path)
        .context(format!("failed to load {:?}", path))?;

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
        Err(..) => bail!("failed to parse {:?}", path),
    };

    // This is how swc manages identifiers. ref: https://rustdoc.swc.rs/swc_ecma_transforms/fn.resolver.html
    let module = GLOBALS.set(&Globals::new(), move || {
        module.fold_with(&mut resolver(Mark::new(), Mark::new(), true))
    });

    let mut translation_usage = TranslationUsage::new();
    if let Some(v) = base_case_visitor::get_labels_usage(&module)? {
        translation_usage.extend(v);
    }
    // Handle more cases here, like:
    // - LABEL_KEYS
    // - i18nKey
    // - translate(<String Literal>)
    // - ...

    Ok(translation_usage.data)
}
