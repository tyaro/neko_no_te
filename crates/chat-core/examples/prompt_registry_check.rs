use chat_core::{discover_plugins, PromptBuilderRegistry};
use std::path::Path;

fn main() {
    let alt = Path::new("target").join("debug");
    let entries = discover_plugins(&alt).expect("discover");
    println!("discovered {} entries", entries.len());
    let reg = PromptBuilderRegistry::from_plugins(&entries);
    println!("is registry empty? {}", reg.is_empty());
    if let Some(src) = reg.resolve("phi4-mini:3.8b") {
        println!("resolved phi4-mini: origin={}", src.origin_label());
    } else {
        println!("no resolver for phi4-mini:3.8b");
    }
}
