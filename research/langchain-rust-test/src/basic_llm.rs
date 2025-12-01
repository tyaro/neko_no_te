//! 基本的なLLM呼び出しテスト
//! 
//! Ollamaとlangchain-rustの基本的な統合を確認

use langchain_rust::{language_models::llm::LLM, llm::ollama::client::Ollama};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== langchain-rust 基本的なLLM呼び出しテスト ===\n");

    // Ollamaクライアント初期化
    println!("📡 Ollama初期化中...");
    let ollama = Ollama::default()
        .with_model("phi4-mini:3.8b");
    
    println!("✅ モデル: phi4-mini:3.8b");
    println!("✅ エンドポイント: http://localhost:11434\n");

    // テスト1: シンプルな質問
    println!("--- テスト1: シンプルな質問 ---");
    let start = Instant::now();
    
    match ollama.invoke("こんにちは！簡単に自己紹介してください。").await {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!("✅ 応答時間: {:?}", elapsed);
            println!("📝 応答:\n{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ エラー: {:?}", e);
            return Err(e.into());
        }
    }

    // テスト2: 日本語の質問
    println!("--- テスト2: 日本語の質問 ---");
    let start = Instant::now();
    
    match ollama.invoke("Rustプログラミング言語の主な特徴を3つ教えてください。").await {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!("✅ 応答時間: {:?}", elapsed);
            println!("📝 応答:\n{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ エラー: {:?}", e);
            return Err(e.into());
        }
    }

    // テスト3: 英語の質問
    println!("--- テスト3: 英語の質問 ---");
    let start = Instant::now();
    
    match ollama.invoke("What is the capital of Japan?").await {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!("✅ 応答時間: {:?}", elapsed);
            println!("📝 応答:\n{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ エラー: {:?}", e);
            return Err(e.into());
        }
    }

    println!("=== テスト完了 ===");
    Ok(())
}
