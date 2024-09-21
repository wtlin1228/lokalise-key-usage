use anyhow::bail;
use std::collections::{HashMap, HashSet};
use swc_core::ecma::ast::*;

#[derive(Debug, PartialEq)]
pub enum TranslateObjectValue {
    String(String),
    NestedLabels(LABELS),
}

impl TranslateObjectValue {
    pub fn get_string(&self) -> anyhow::Result<&str> {
        match self {
            TranslateObjectValue::String(s) => Ok(s),
            TranslateObjectValue::NestedLabels(_) => bail!("it's a nested labels"),
        }
    }

    pub fn get_labels(&self) -> anyhow::Result<&LABELS> {
        match self {
            TranslateObjectValue::String(_) => bail!("it's a string"),
            TranslateObjectValue::NestedLabels(labels) => Ok(labels),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LABELS {
    Object(HashMap<String, TranslateObjectValue>),

    // If we found the object has computed keys, just collect all lokalise keys into a vector.
    Computed(HashSet<String>),
}

impl LABELS {
    pub fn get_object(&self) -> anyhow::Result<&HashMap<String, TranslateObjectValue>> {
        match self {
            LABELS::Object(hash_map) => Ok(hash_map),
            LABELS::Computed(_) => bail!("it's a computed"),
        }
    }

    pub fn get_computed(&self) -> anyhow::Result<&HashSet<String>> {
        match self {
            LABELS::Object(_) => bail!("it's an object"),
            LABELS::Computed(hash_set) => Ok(hash_set),
        }
    }
}

fn flatten_translation_keys(object_lit: &ObjectLit) -> anyhow::Result<HashSet<String>> {
    let mut translation_keys = HashSet::new();
    for prop_or_spread in object_lit.props.iter() {
        match prop_or_spread {
            PropOrSpread::Prop(prop) => match &**prop {
                Prop::KeyValue(key_value_prop) => match &*key_value_prop.value {
                    Expr::Object(object_lit) => {
                        translation_keys.extend(flatten_translation_keys(object_lit)?);
                    }
                    Expr::Lit(lit) => match lit {
                        Lit::Str(Str { value, .. }) => {
                            translation_keys.insert(value.to_string());
                        }
                        _ => bail!("value can only be string and object literal"),
                    },
                    _ => bail!("value can only be string and object literal"),
                },
                _ => bail!("only key-value prop is allowed"),
            },
            PropOrSpread::Spread(_) => bail!("spread is not allowed"),
        }
    }
    Ok(translation_keys)
}

pub fn collect_labels_from_object_literal(object_lit: &ObjectLit) -> anyhow::Result<LABELS> {
    let mut labels = HashMap::new();
    let mut translation_keys = HashSet::new();
    let mut has_computed_key = false;
    for prop_or_spread in object_lit.props.iter() {
        match prop_or_spread {
            PropOrSpread::Prop(prop) => match &**prop {
                Prop::KeyValue(key_value_prop) => match &key_value_prop.key {
                    PropName::Ident(ident) => {
                        if has_computed_key {
                            bail!("mixing string and computed keys is not allowed");
                        }
                        labels.insert(
                            ident.sym.to_string(),
                            match &*key_value_prop.value {
                                Expr::Object(object_lit) => TranslateObjectValue::NestedLabels(
                                    collect_labels_from_object_literal(object_lit)?,
                                ),
                                Expr::Lit(lit) => match lit {
                                    Lit::Str(Str { value, .. }) => {
                                        TranslateObjectValue::String(value.to_string())
                                    }
                                    _ => bail!("value can only be string and object literal"),
                                },
                                _ => bail!("value can only be string and object literal"),
                            },
                        );
                    }
                    PropName::Computed(_) => {
                        if labels.len() != 0 {
                            panic!("mixing string and computed keys is not allowed");
                        }
                        has_computed_key = true;
                        match &*key_value_prop.value {
                            Expr::Object(object_lit) => {
                                translation_keys.extend(flatten_translation_keys(object_lit)?);
                            }
                            Expr::Lit(lit) => match lit {
                                Lit::Str(Str { value, .. }) => {
                                    translation_keys.insert(value.to_string());
                                }
                                _ => bail!("value can only be string and object literal"),
                            },
                            _ => bail!("value can only be string and object literal"),
                        }
                    }
                    _ => bail!("key can only be string or computed"),
                },
                _ => bail!("only key-value prop is allowed"),
            },
            PropOrSpread::Spread(_) => bail!("spread is not allowed"),
        }
    }

    Ok(match has_computed_key {
        true => LABELS::Computed(translation_keys),
        false => LABELS::Object(labels),
    })
}

#[cfg(test)]
mod test {
    use anyhow::Context;
    use swc_core::{
        common::{sync::Lrc, FileName, SourceMap},
        ecma::{
            ast::*,
            visit::{Visit, VisitWith},
        },
    };
    use swc_ecma_parser::{parse_file_as_module, Syntax, TsSyntax};

    use super::*;

    pub struct Visitor {
        object_lit: Option<ObjectLit>,
    }
    impl Visitor {
        pub fn new() -> Self {
            Self { object_lit: None }
        }
    }
    impl Visit for Visitor {
        fn visit_object_lit(&mut self, node: &ObjectLit) {
            self.object_lit = Some(node.clone());
        }
    }

    fn parse_module(input: &str) -> anyhow::Result<Module> {
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

    fn parse_object_lit(input: &str) -> anyhow::Result<ObjectLit> {
        let input = format!("const obj = {}", input);
        let module = parse_module(&input)?;
        let mut visitor = Visitor::new();
        module.visit_with(&mut visitor);
        Ok(visitor.object_lit.context("failed to get object literal")?)
    }

    #[test]
    fn empty_object() {
        let object_lit = parse_object_lit(
            r#"
            {}
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        assert_eq!(labels, LABELS::Object(HashMap::new()));
    }

    #[test]
    fn simple_object() {
        let object_lit = parse_object_lit(
            r#"
            {
                bird: "i18n.bird",
                cat: "i18n.cat",
                dog: "i18n.dog",
            }
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        let object = labels.get_object().unwrap();
        assert_eq!(
            object.get("bird").unwrap().get_string().unwrap(),
            "i18n.bird"
        );
        assert_eq!(object.get("cat").unwrap().get_string().unwrap(), "i18n.cat");
        assert_eq!(object.get("dog").unwrap().get_string().unwrap(), "i18n.dog");
    }

    #[test]
    fn simple_computed() {
        let object_lit = parse_object_lit(
            r#"
            {
                [PET.bird]: "i18n.bird",
                [PET.cat]: "i18n.cat",
                [PET.dog]: "i18n.dog",
            }
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        let computed = labels.get_computed().unwrap();
        assert!(computed.contains("i18n.bird"));
        assert!(computed.contains("i18n.cat"));
        assert!(computed.contains("i18n.dog"));
    }

    #[test]
    #[should_panic(expected = "mixing string and computed keys is not allowed")]
    fn mixed_object_computed() {
        let object_lit = parse_object_lit(
            r#"
            {
                bird: "i18n.bird",
                cat: "i18n.cat",
                dog: "i18n.dog",
                [PET.bird]: "i18n.bird",
                [PET.cat]: "i18n.cat",
                [PET.dog]: "i18n.dog",
            }
            "#,
        )
        .unwrap();
        collect_labels_from_object_literal(&object_lit).unwrap();
    }

    #[test]
    fn nested_object() {
        let object_lit = parse_object_lit(
            r#"
            {
                fly: {
                    bird: "i18n.bird",
                },
                walk: {
                    cat: "i18n.cat",
                    dog: "i18n.dog",
                },
            }
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        let object = labels.get_object().unwrap();
        let fly_object = object
            .get("fly")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_object()
            .unwrap();
        let walk_object = object
            .get("walk")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_object()
            .unwrap();
        assert_eq!(
            fly_object.get("bird").unwrap().get_string().unwrap(),
            "i18n.bird"
        );
        assert_eq!(
            walk_object.get("cat").unwrap().get_string().unwrap(),
            "i18n.cat"
        );
        assert_eq!(
            walk_object.get("dog").unwrap().get_string().unwrap(),
            "i18n.dog"
        );
    }

    #[test]
    fn nested_computed() {
        let object_lit = parse_object_lit(
            r#"
            {
                [PET.bird]: {
                    name: "i18n.bird",
                    size: {
                        [SIZE.samll]: "i18n.bird.small",
                        [SIZE.large]: "i18n.bird.large",
                    },
                },
                [PET.cat]: {
                    name: "i18n.cat",
                    size: {
                        [SIZE.samll]: "i18n.cat.small",
                        [SIZE.large]: "i18n.cat.large",
                    },
                },
                [PET.dog]: {
                    name: "i18n.dog",
                    size: {
                        [SIZE.samll]: "i18n.dog.small",
                        [SIZE.large]: "i18n.dog.large",
                    },
                },
            }
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        let computed = labels.get_computed().unwrap();
        assert!(computed.contains("i18n.bird"));
        assert!(computed.contains("i18n.bird.small"));
        assert!(computed.contains("i18n.bird.large"));
        assert!(computed.contains("i18n.cat"));
        assert!(computed.contains("i18n.cat.small"));
        assert!(computed.contains("i18n.cat.large"));
        assert!(computed.contains("i18n.dog"));
        assert!(computed.contains("i18n.dog.small"));
        assert!(computed.contains("i18n.dog.large"));
    }

    #[test]
    fn complex() {
        let object_lit = parse_object_lit(
            r#"
            {
                title: "i18n.pet.party",
                bird: {
                    name: "i18n.bird",
                    size: {
                        [SIZE.samll]: "i18n.bird.small",
                        [SIZE.large]: "i18n.bird.large",
                    },
                },
                cat: {
                    name: "i18n.cat",
                    size: {
                        [SIZE.samll]: "i18n.cat.small",
                        [SIZE.large]: "i18n.cat.large",
                    },
                },
                dog: {
                    name: "i18n.dog",
                    size: {
                        [SIZE.samll]: "i18n.dog.small",
                        [SIZE.large]: "i18n.dog.large",
                    },
                },
            }
            "#,
        )
        .unwrap();
        let labels = collect_labels_from_object_literal(&object_lit).unwrap();
        let object = labels.get_object().unwrap();
        assert_eq!(
            object.get("title").unwrap().get_string().unwrap(),
            "i18n.pet.party"
        );

        let bird_object = object
            .get("bird")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_object()
            .unwrap();
        assert_eq!(
            bird_object.get("name").unwrap().get_string().unwrap(),
            "i18n.bird"
        );
        let bird_size_computed = bird_object
            .get("size")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_computed()
            .unwrap();
        assert!(bird_size_computed.contains("i18n.bird.small"));
        assert!(bird_size_computed.contains("i18n.bird.large"));

        let cat_object = object
            .get("cat")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_object()
            .unwrap();
        assert_eq!(
            cat_object.get("name").unwrap().get_string().unwrap(),
            "i18n.cat"
        );
        let cat_size_computed = cat_object
            .get("size")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_computed()
            .unwrap();
        assert!(cat_size_computed.contains("i18n.cat.small"));
        assert!(cat_size_computed.contains("i18n.cat.large"));

        let dog_object = object
            .get("dog")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_object()
            .unwrap();
        assert_eq!(
            dog_object.get("name").unwrap().get_string().unwrap(),
            "i18n.dog"
        );
        let dog_size_computed = dog_object
            .get("size")
            .unwrap()
            .get_labels()
            .unwrap()
            .get_computed()
            .unwrap();
        assert!(dog_size_computed.contains("i18n.dog.small"));
        assert!(dog_size_computed.contains("i18n.dog.large"));
    }
}
