//! ストリーミングレスポンステスト
//! 
//! リアルタイムでAI応答を受信する機能をテスト

use langchain_rust::{
    language_models::llm::LLM,
    llm::ollama::client::Ollama,
    schemas::Message,
};
use std::time::Instant;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== langchain-rust ストリーミングテスト ===\n");

    // Ollamaクライアント初期化
    println!("📡 Ollama初期化中...");
    let ollama = Ollama::default()
        .with_model("phi4-mini:3.8b");
    
    println!("✅ モデル: phi4-mini:3.8b\n");

    // テスト1: 短い質問
    println!("--- テスト1: 短い質問（ストリーミング） ---");
    println!("👤 ユーザー: こんにちは！\n");
    println!("🤖 AI応答（リアルタイム）:");
    
    let message = Message::new_human_message("こんにちは！元気ですか？");
    let start = Instant::now();
    
    let mut stream = ollama.stream(&[message]).await?;
    let mut full_response = String::new();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                print!("{}", data.content);
                std::io::Write::flush(&mut std::io::stdout())?;
                full_response.push_str(&data.content);
            }
            Err(e) => {
                eprintln!("\n❌ ストリームエラー: {:?}", e);
                return Err(e.into());
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("\n\n⏱️  総応答時間: {:?}", elapsed);
    println!("📊 応答文字数: {}\n", full_response.len());

    // テスト2: 長い質問
    println!("--- テスト2: 長い質問（ストリーミング） ---");
    println!("👤 ユーザー: Rustプログラミング言語について詳しく説明してください。\n");
    println!("🤖 AI応答（リアルタイム）:");
    
    let message = Message::new_human_message(
        "Rustプログラミング言語の主な特徴、メモリ安全性、所有権システムについて詳しく説明してください。"
    );
    let start = Instant::now();
    
    let mut stream = ollama.stream(&[message]).await?;
    let mut full_response = String::new();
    let mut chunk_count = 0;
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                print!("{}", data.content);
                std::io::Write::flush(&mut std::io::stdout())?;
                full_response.push_str(&data.content);
                chunk_count += 1;
            }
            Err(e) => {
                eprintln!("\n❌ ストリームエラー: {:?}", e);
                return Err(e.into());
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("\n\n⏱️  総応答時間: {:?}", elapsed);
    println!("📊 応答文字数: {}", full_response.len());
    println!("📦 チャンク数: {}\n", chunk_count);

    println!("=== ストリーミングテスト完了 ===");
    println!("✅ リアルタイムでトークンが表示されましたか？");
    println!("✅ UIへの統合イメージは掴めましたか？");
    
    Ok(())
}
