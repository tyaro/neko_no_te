use anyhow::Context;
use chat_core::{discover_plugins, PromptBuilderRegistry};
use prompt_spi::{ConversationRole, ConversationTurn, PromptContext};
use serde::Deserialize;
use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct TestScript { tests: Vec<TestCase>, }

#[derive(Debug, Deserialize)]
struct TestCase {
    model: String,
    conversation: Option<Vec<ScriptTurn>>,
    locale: Option<String>,
    expected_prompt_contains: Option<String>,
    /// Optional regular expression to match against the built prompt. If provided,
    /// the prompt must match the regex to pass. If both `expected_prompt_contains`
    /// and `expected_prompt_regex` are provided, both conditions must be satisfied.
    expected_prompt_regex: Option<String>,
    /// Optional regex flags: combination of i (case-insensitive), m (multi-line), s (dot matches newline)
    expected_prompt_regex_flags: Option<String>,
    /// Optional expected agent mode from PromptPayload ("LangChain" / "DirectProvider")
    expected_agent_mode: Option<String>,
    /// Expected prompt variables: key -> value (checks presence and value match/contains for strings)
    expected_prompt_variables: Option<HashMap<String, serde_json::Value>>,
    /// Expected tool names to appear in the built prompt text
    expected_tool_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ScriptTurn { role: String, content: String }

fn normalize_label(s: &str) -> String {
    let mut normalized = s.trim().to_string();
    if let Some(idx) = normalized.find('(') { normalized = normalized[..idx].trim().to_string(); }
    normalized
}

fn check_payload_expectations(
    payload: &prompt_spi::PromptPayload,
    prompt_text: &str,
    tc: &TestCase,
) -> bool {
    use regex::RegexBuilder;

    let mut ok = true;

    // substring check
    if let Some(ref expect) = tc.expected_prompt_contains {
        if !prompt_text.contains(expect) {
            println!("✗ expected substring not found: {}", expect);
            ok = false;
        }
    }

    // regex check
    if let Some(ref regex_s) = tc.expected_prompt_regex {
        let mut builder = RegexBuilder::new(regex_s);
        if let Some(flags) = &tc.expected_prompt_regex_flags {
            if flags.contains('i') { builder.case_insensitive(true); }
            if flags.contains('m') { builder.multi_line(true); }
            if flags.contains('s') { builder.dot_matches_new_line(true); }
        }
        match builder.build() {
            Ok(re) => {
                if !re.is_match(prompt_text) {
                    println!("✗ regex did not match: {}", regex_s);
                    ok = false;
                }
            }
            Err(e) => {
                println!("✗ invalid regex '{}': {}", regex_s, e);
                ok = false;
            }
        }
    }

    // agent mode check
    if let Some(ref want) = tc.expected_agent_mode {
        let want_lower = want.to_lowercase();
        let actual = match payload.agent_mode {
            prompt_spi::PromptAgentMode::LangChain => "langchain",
            prompt_spi::PromptAgentMode::DirectProvider => "directprovider",
        };
        if actual != want_lower.as_str() && actual != want_lower.replace('-', "").as_str() {
            println!("✗ agent_mode mismatch: want='{}' actual='{}'", want, actual);
            ok = false;
        }
    }

    // prompt variables check
    if let Some(ref expected_vars) = tc.expected_prompt_variables {
        for (k, v_expected) in expected_vars {
            match payload.prompt_variables.get(k) {
                Some(v_actual) => match (v_expected, v_actual) {
                    (serde_json::Value::String(s_expected), serde_json::Value::String(s_actual)) => {
                        if !s_actual.contains(s_expected) {
                            println!("✗ prompt variable mismatch for '{}': actual doesn't contain '{}'", k, s_expected);
                            ok = false;
                        }
                    }
                    (_, _) => {
                        if v_expected != v_actual {
                            println!("✗ prompt variable mismatch for '{}': expected {:?} actual {:?}", k, v_expected, v_actual);
                            ok = false;
                        }
                    }
                },
                None => {
                    println!("✗ prompt variable '{}' missing", k);
                    ok = false;
                }
            }
        }
    }

    // tool names: check they appear in the built prompt text
    if let Some(ref tools) = tc.expected_tool_names {
        for t in tools {
            if !prompt_text.contains(t) {
                println!("✗ expected tool name '{}' not found in prompt", t);
                ok = false;
            }
        }
    }

    ok
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: cli-test-runner <script.json> [--use-target-plugins]");
        std::process::exit(2);
    }

    let script = PathBuf::from(&args[0]);
    let use_target_plugins = args.iter().any(|a| a == "--use-target-plugins");

    let repo = PathBuf::from(".");
    let mut entries = discover_plugins(&repo)?;
    if use_target_plugins || entries.is_empty() {
        let alt = repo.join("target").join("debug");
        if alt.exists() {
            if let Ok(e) = discover_plugins(&alt) { if !e.is_empty() { entries = e; } }
        }
    }

    let mut registry = PromptBuilderRegistry::from_plugins(&entries);
    chat_core::register_builtin_prompt_builders(&mut registry);

    if registry.is_empty() { anyhow::bail!("no prompt builders available — build or sync plugins first"); }

    let raw = fs::read(&script).with_context(|| format!("failed to read script {}", script.display()))?;
    let script: TestScript = serde_json::from_slice(&raw).with_context(|| "invalid test script JSON")?;

    let mut total = 0usize;
    let mut passed = 0usize;

    for tc in script.tests {
        total += 1;
        let model = normalize_label(&tc.model);
        println!("\n== TEST #{} model={} ==", total, model);

        // Try plugin JSON FFI first
        let entry = entries.iter().find(|e| {
            e.metadata.as_ref().map(|m| m.models.iter().any(|mid| mid == &model)).unwrap_or(false)
        });

        if let Some(e) = entry {
            if e.enabled {
                if let Some(md) = &e.metadata {
                    if let Some(lib) = &md.library {
                        let libpath = e.path.join(lib);
                        if libpath.exists() {
                            // Try plugin JSON entrypoint first (safer across dylib boundaries)
                            use libloading::Library;
                            unsafe {
                                type BuildJsonFn = unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_char;
                                if let Ok(libdl) = Library::new(&libpath) {
                                    if let Ok(sym) = libdl.get::<BuildJsonFn>(b"build_prompt_json\0") {
                                        let conv = tc
                                            .conversation
                                            .as_ref()
                                            .map(|v| v.iter().map(|t| (t.role.as_str(), t.content.as_str())).collect::<Vec<_>>())
                                            .unwrap_or_default();

                                        #[derive(serde::Serialize)]
                                        struct ShortCtx<'a> {
                                            model: &'a str,
                                            locale: &'a str,
                                            conversation: Vec<(&'a str, &'a str)>,
                                        }

                                        let c = ShortCtx {
                                            model: &tc.model,
                                            locale: tc.locale.as_deref().unwrap_or("en-US"),
                                            conversation: conv,
                                        };
                                        let js = serde_json::to_string(&c).unwrap();
                                        let cstr = std::ffi::CString::new(js).unwrap();
                                        let out = sym(cstr.as_ptr());
                                        if !out.is_null() {
                                            let s = std::ffi::CString::from_raw(out).into_string().unwrap_or_default();
                                            if let Ok(payload) = serde_json::from_str::<prompt_spi::PromptPayload>(&s) {
                                                let p = payload.prompt.as_deref().unwrap_or_default().to_string();
                                                println!("[plugin JSON] built prompt:\n{}", p);
                                                if check_payload_expectations(&payload, &p, &tc) {
                                                    println!("✓ ok");
                                                    passed += 1;
                                                    continue;
                                                } else {
                                                    println!("[plugin JSON] expectations not met, trying registry fallback...");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // fallback to registry host/plugin builder via trait (safe for builtin; plugin trait may be cross-dylib unsafe on some platforms)
        if let Some(src) = registry.resolve(&model) {
            println!("Resolved: {} (origin={})", src.metadata().name, src.origin_label());
            let builder = src.create_builder();

            // produce an owned copy of the conversation so we can take stable &str refs
            let conv_owned = tc.conversation.as_ref().map(|v| v.iter().map(|st| (st.role.clone(), st.content.clone())).collect::<Vec<_>>()).unwrap_or_default();
            let conversation = conv_owned.iter().map(|(role, content)| ConversationTurn{ role: match role.as_str(){"system"=>ConversationRole::System,"assistant"=>ConversationRole::Assistant,"tool"=>ConversationRole::Tool,_=>ConversationRole::User}, content: content.as_str() }).collect::<Vec<_>>();
            let ctx = PromptContext { model: &model, locale: tc.locale.as_deref().unwrap_or("en-US"), conversation: &conversation, tools: &[], system_directives: &[] };
                    match builder.build(ctx) {
                Ok(payload) => {
                    let p = payload.prompt.as_deref().unwrap_or_default().to_string();
                    println!("[built] {}", p);
                    if check_payload_expectations(&payload, &p, &tc) { println!("✓ ok"); passed += 1; } else { /* failure already printed */ }
                }
                Err(e) => { println!("✗ builder error: {}", e); }
            }
        } else {
            println!("✗ no builder resolved for {}", model);
        }
    }

    println!("\nSUMMARY: {}/{} passed", passed, total);
    if passed != total { std::process::exit(1); }
    Ok(())
}
