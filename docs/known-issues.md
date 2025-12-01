# 既知の問題と対処

## Phase 1 MVP（完了）

### ✅ Ollama レスポンス形式

**解決済み**: `"stream": false` をリクエストに追加し、`response` フィールドを抽出

**修正ファイル**: `crates/ollama-client/src/lib.rs`

## Phase 2 UX改善（完了）

### ✅ 設定画面の実装
- Ollama URL、モデル名、最大履歴数を編集可能
- 設定の保存・読込機能

### ✅ セッション管理
- 新規チャット開始（New ボタン）
- 会話の保存（Save ボタン）
- JSON形式で履歴を永続化

### ✅ ストリーミングAPI
- `generate_stream()` メソッドを実装
- コールバックで部分応答を受け取る仕組み
- 将来のUI統合準備完了

### ✅ UI改善
- メッセージ種類別の表示（User/AI/Error）
- エラーメッセージの色分け表示

### ✅ IME 入力の基本動作

**解決済み**: gpui-component 0.4.2 の `Input` コンポーネントは IME 入力に対応しています

**動作確認済み**:
- ✅ 日本語入力（変換候補表示、確定）
- ✅ 中国語入力
- ✅ 複数行入力（auto_grow で 1〜5行）
- ✅ Enter/Ctrl+Enter での送信

---

### ⚠️ サードパーティIMEでの全角/半角キー検出

**問題**: ATOK、Google日本語入力、Baidu IME等のサードパーティIMEを使用している場合、**全角/半角キーでのIME切り替えがアプリケーションレベルで検出できません**。

**原因**:
- サードパーティIMEは、**OSカーネル/ドライバレベル**でキー入力を処理します
- 全角/半角キーのイベントがアプリケーション層（user-space）に届く前にIMEが捕捉・処理します
- GPUIはBladeレンダラーを使用しており、低レベルキーイベントへのアクセスが制限されています

**回避策**（推奨順）:

1. **Alt + ` (バッククォート)** を使用
   - Windows標準のIME切り替えショートカット
   - **すべてのIMEで動作します**（最も推奨）
   - アプリケーションレベルで検出可能

2. **Ctrl + Space** （IMEによる）
   - 一部のIMEで有効
   - Google日本語入力等で利用可能

3. **IMEツールバー/タスクバーアイコン**
   - マウスクリックでIMEを切り替え
   - 最も確実な方法

4. **Windows標準IMEに切り替え**
   - Windows設定 → 時刻と言語 → 言語
   - 日本語 → オプション → Microsoft IME
   - Windows標準IMEなら全角/半角キーが動作します

**動作確認結果**:
- ✅ Windows標準IME: 全角/半角キー動作
- ❌ ATOK: 全角/半角キー検出不可（OSレベルで処理）
- ❌ Google日本語入力: 全角/半角キー検出不可（OSレベルで処理）
- ✅ すべてのIME: **Alt + `** で切り替え可能
- ✅ IME入力自体は正常動作（変換候補表示、確定等）

**技術的詳細**:
- GPUI (Blade) は低レベルキーボードフックを使用していません
- Windows APIの `GetKeyState()` や `GetAsyncKeyState()` でも検出不可
  - これらのAPIは**ユーザー空間の仮想キーコード**のみ扱います
  - カーネル空間でIMEドライバが処理したキーは見えません
- IME APIからの通知も受け取れません（TSF/IMM32制限）
  - TSF (Text Services Framework) はIME状態変更通知を提供しますが
  - GPUIの現在のアーキテクチャでは統合が困難です

**将来的な改善案**:

1. **Windows Raw Inputの使用**
   - `RegisterRawInputDevices()` API でキーボードデバイスを直接監視
   - カーネルレベルより前のキーイベントを取得
   - **要: GPUIへのパッチ、複雑な実装**

2. **IME状態の監視**
   - TSF (Text Services Framework) APIの使用
   - IME切り替えイベントを間接的に検出
   - **要: FFI実装、スレッド管理**

3. **ユーザー設定でショートカットをカスタマイズ**
   - 検出可能なキーコンビネーションを設定
   - **最も現実的なアプローチ**

**参考リンク**:
- GPUI Issues: https://github.com/zed-industries/zed/issues
- gpui-component: https://crates.io/crates/gpui-component
- Windows IME Documentation: https://docs.microsoft.com/en-us/windows/win32/intl/input-method-manager
- TSF Overview: https://docs.microsoft.com/en-us/windows/win32/tsf/text-services-framework

## 今後の課題

### Phase 3 で対応予定

- [ ] ストリーミングレスポンスのUI統合（リアルタイム表示）
- [ ] セッション一覧画面（Load Session UI）
- [ ] ツール/関数呼び出しの実装
- [ ] MCP サーバー統合
- [ ] RAG（検索拡張生成）機能
- [ ] プラグインシステムの活性化
- [ ] SQLite への履歴マイグレーション

### パフォーマンス最適化

- [x] Cargo ビルドキャッシュの設定（`.cargo/config.toml`）
- [ ] sccache の導入（オプション）
- [ ] 依存関係の最適化

