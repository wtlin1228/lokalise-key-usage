use lokalise_key_usage::core;
use std::path::Path;

const PATH: &'static str = "./fixtures/DiscountSubmitModalTimeConditionFields.jsx";

fn main() -> anyhow::Result<()> {
    let translation_usage = core::collect_translation(Path::new(PATH))?;
    println!("{:#?}", translation_usage);
    Ok(())
}
