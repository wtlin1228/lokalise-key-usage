use super::base_case_visitor;
use std::collections::{HashMap, HashSet};
use swc_core::ecma::ast::Module;

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

pub fn collect_translation(module: &Module) -> anyhow::Result<HashMap<String, HashSet<String>>> {
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
