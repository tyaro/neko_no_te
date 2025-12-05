use chat_core::discover_plugins;
use std::env;
use std::path::Path;

fn normalize_label(s: &str) -> String {
    let mut normalized = s.trim().to_string();
    if let Some(idx) = normalized.find('(') {
        normalized = normalized[..idx].trim().to_string();
    }
    normalized
}

fn main() {
    // models to test: allow passing via args, otherwise use a default set
    let args: Vec<String> = env::args().skip(1).collect();
    let to_test = if !args.is_empty() {
        args
    } else {
        vec![
            "phi4-mini:3.8b".into(),
            "pakachan/elyza-llama3-8b:latest".into(),
            "gemma3n:e2b".into(),
            "gemma3:4b".into(),
            "llama3.2:3b".into(),
            "llama3.1:8b".into(),
            "qwen3:4b-instruct".into(),
        ]
    };

    let repo_root = Path::new(".");

    println!("Discovering plugins under {:?}...", repo_root);
    // Try discovery in a couple of likely locations:
    // 1) repo root ./plugins
    // 2) target/debug/plugins (when running from workspace build)
    // 3) exe parent /plugins (deployed scenario)
    let mut entries = match discover_plugins(repo_root) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to discover plugins: {}", err);
            std::process::exit(2);
        }
    };

    if entries.is_empty() {
        let alt = Path::new("target").join("debug");
        if alt.exists() {
            if let Ok(e) = discover_plugins(&alt) {
                if !e.is_empty() {
                    entries = e;
                }
            }
        }
    }

    println!("Found {} plugin directories:\n", entries.len());
    for p in &entries {
        println!("- {} (enabled: {})", p.dir_name, p.enabled);
        if let Some(ref md) = p.metadata {
            println!("  metadata.models = {:?}", md.models);
        } else {
            println!("  (no plugin.toml metadata)");
        }
    }

    println!("\nTesting model->plugin matching:\n");
    for m in &to_test {
        let normalized = normalize_label(m);
        let mut matched = vec![];
        for p in &entries {
            if let Some(ref md) = p.metadata {
                if md.models.iter().any(|mid| mid == &normalized) {
                    matched.push((p.dir_name.clone(), p.enabled));
                }
            }
        }

        if matched.is_empty() {
            println!("MODEL '{}' -> NO MATCH", m);
        } else {
            println!("MODEL '{}' -> matches:", m);
            for (name, enabled) in matched {
                println!("  - {} (enabled: {})", name, enabled);
            }
        }
    }
}
