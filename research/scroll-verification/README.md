# Scroll Verification

検証プログラム: neko-assistant の UI 構造を模倣してスクロール動作を確認

## 構造

- **左パネル (340px)**: Scratchpad (150px 固定) + Console (flex_1、スクロール可能)
- **右パネル (flex_1)**: Chat ヘッダー + メッセージエリア (flex_1、スクロール可能) + モデル選択 + 入力欄 (80px 固定)

## 成功したスクロールパターン

### Console エリア（ScrollableElement 使用）

```rust
div()
    .flex_1()
    .min_h(px(0.0))
    .overflow_hidden()        // ← 親で overflow 制御
    .child(
        div()
            .size_full()
            .v_flex()
            .gap_1()
            .overflow_y_scrollbar()  // ← 子でスクロール
            .children(items),
    )
```

**重要ポイント:**
- **親**: `.flex_1() + .min_h(px(0.0)) + .overflow_hidden()`
- **子**: `.size_full() + .overflow_y_scrollbar()`
- **余計なネストなし** (`.relative()` や二重の `.overflow_hidden()` は不要)

### Chat メッセージエリア（ScrollableElement 使用、ScrollHandle なし）

```rust
div()
    .flex_1()
    .min_h(px(0.0))
    .overflow_hidden()        // ← 親で overflow 制御
    .child(
        div()
            .size_full()
            .v_flex()
            .gap_2()
            .overflow_y_scrollbar()  // ← 子でスクロール
            .children(message_items),
    )
```

**neko-assistant との違い:**
- 検証プログラムは `ScrollHandle` を使わない
- neko-assistant は `.track_scroll(handle)` + `.overflow_scroll()` を使用
- neko-assistant では自動スクロール機能のため `ScrollHandle` が必要

## 検証結果

✅ **Console**: スクロール正常動作  
✅ **Chat**: スクロール正常動作  
✅ **Scratchpad**: Input コンポーネントは単一行（複数行は Textarea が必要）

## neko-assistant への適用

検証プログラムで成功したパターンを neko-assistant に適用したが、まだ問題が残っている可能性:
1. `ScrollHandle` を使用する場合の動作が異なる
2. 実際のログデータが空または少ない
3. 親コンテナのレイアウト制約が不足している

## 次のステップ

1. neko-assistant でメッセージを送信して Console ログを確認
2. `ScrollHandle` の使用方法を再確認
3. 親コンテナ (`chat_window`, `chat_body` など) のレイアウトを再検証
