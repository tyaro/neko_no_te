//! ConversationalChainテスト
//! 
//! 会話履歴を管理する機能をテスト

use langchain_rust::{
    chain::{Chain, builder::ConversationalChainBuilder},
    llm::ollama::client::Ollama,
    memory::SimpleMemory,
    prompt_args,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== langchain-rust ConversationalChain テスト ===\n");

    // Ollamaクライアント初期化
    println!("📡 Ollama初期化中...");
    let ollama = Ollama::default()
        .with_model("phi4-mini:3.8b");
    
    println!("✅ モデル: phi4-mini:3.8b\n");

    // メモリ初期化
    let memory = SimpleMemory::new();
    
    // 会話チェーン構築
    println!("🔗 ConversationalChain構築中...");
    let chain = ConversationalChainBuilder::new()
        .llm(ollama)
        .memory(memory.into())
        .build()?;
    
    println!("✅ チェーン構築完了\n");

    // 会話シミュレーション
    let conversations = vec![
        "こんにちは！私の名前は太郎です。",
        "私の名前を覚えていますか？",
        "Rustについて教えてください。",
        "先ほど話したトピックは何でしたか？",
    ];

    for (i, input) in conversations.iter().enumerate() {
        println!("--- ターン {} ---", i + 1);
        println!("👤 ユーザー: {}", input);
        
        let start = Instant::now();
        
        match chain
            .invoke(prompt_args! {
                "input" => input.to_string(),
            })
            .await
        {
            Ok(result) => {
                let elapsed = start.elapsed();
                println!("⏱️  応答時間: {:?}", elapsed);
                println!("🤖 AI: {}\n", result);
            }
            Err(e) => {
                eprintln!("❌ エラー: {:?}", e);
                return Err(e.into());
            }
        }
    }

    println!("=== 会話履歴テスト完了 ===");
    println!("✅ AIは文脈を保持できていましたか？");
    println!("✅ 名前や前のトピックを覚えていましたか？");
    
    Ok(())
}
