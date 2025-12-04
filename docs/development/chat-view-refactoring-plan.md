# ChatView リファクタリング計画

**作成日**: 2025年12月4日  
**最終レビュー日**: 2025年12月5日  
**対象ファイル**: `neko-assistant/src/gui/chat/mod.rs` (783行)  
**目的**: 長大化した ChatView の責務を分離し、保守性・可読性を向上させる

---

## レビューサマリー（2025/12/04）

- 提案 A（モジュール分割）は妥当であり、段階的な実施でリスクを抑えられる
- ただし **ChatView に GPUI Entity が集中している問題** を解消しないと、モジュール化後も可読性が高まらない懸念がある
- `ButtonFactory` と `ChatEventHandler` を単なる関数集にするよりも、**明確なデータ構造 (`ChatViewState`, `ChatControllerFacade`) を導入**した方が所有権まわりの問題を避けやすい
- フェーズ分割を「成果物ベース」に再整理すると、レビューの区切りが明確になる

レビュー結果を踏まえ、本ドキュメントでは以下の改善点を追記した。

1. `ChatViewState`（UI エンティティや ScrollManager を集約）と `ChatControllerFacade`（Controller へのコマンド送信を一元化）の導入を Phase1 で検討する
2. 各フェーズの完了条件とロールバックポイントを明示し、Pull Request の単位を取りやすくする
3. Button 生成は `ButtonFactory` ではなく **小さな関数 + shared trait** に留め、過度なオブジェクト化を避ける
4. Phase3 以降で `ChatViewBuilder` を組み立てる際に `Arc<PromptBuilderRegistry>` などの clone を最小化する

---

## 現状分析

### ファイル構造
```
neko-assistant/src/gui/chat/
├── mod.rs              (783行) ← **問題のファイル**
├── chat_window.rs      (UI レイアウト)
├── console_window.rs   (コンソール UI)
├── menu.rs             (メニューバー)
├── menu_actions.rs     (メニュー操作ヘルパ)
├── menu_bar_widget.rs  (メニューバー GPUI ウィジェット)
├── model_selector.rs   (モデル選択ヘルパ)
├── scratchpad_window.rs (スクラッチパッド UI)
├── session_popup.rs    (セッション選択)
├── menu_context.rs     (Menu ボタン向け依存集約コンテキスト)
├── toolbar_view_model.rs (ツールバー表示状態)
├── toolbar_widget.rs   (ツールバー GPUI ウィジェット)
└── ui_state.rs         (UI スナップショット変換)
```

### ChatView の責務（現状）
`mod.rs` の `ChatView` 構造体は以下の責務を持つ：

1. **アプリケーション初期化** (`new()` - 200行)
   - 設定読み込み
   - ConversationManager 初期化
   - MCP Manager 初期化
   - ChatController 初期化
   - UI コンポーネント（InputState, SelectState）作成
   - イベントサブスクリプション設定

2. **状態管理**
   - ChatController のイベント受信・処理
   - UI 状態の同期
   - スクロール管理

3. **モデル選択機能**
   - `switch_model()` - モデル切り替え
   - `persist_model_selection()` - 設定保存
   - `sync_model_selector_items()` - セレクタ同期
   - `sync_model_selector_selection()` - 選択状態同期
   - `model_presets_from_state()` - プリセット変換

4. **イベント処理**
   - `drain_chat_events()` - イベントポーリング
   - `handle_chat_event()` - イベントディスパッチ

5. **UI コンポーネント生成**
   - `app_menu_button()` - アプリメニュー (110行)
   - `manage_mcp_button()` - MCP 管理ボタン
   - `plugin_button()` - プラグインボタン

6. **スクラッチパッド機能**
   - `scratchpad_file_path()` - パス取得
   - `load_scratchpad()` - ファイル読み込み
   - `save_scratchpad()` - ファイル保存
   - `open_scratchpad_sheet()` - シート表示 (50行)

7. **コンソール機能**
   - `open_console_sheet()` - シート表示
   - `console_log_entries()` - ログ変換

8. **データ変換**
   - `chat_rows()` - メッセージ表示用変換
   - `mcp_server_items()` - MCP サーバー表示用変換
   - `mcp_tool_items()` - MCP ツール表示用変換

9. **レンダリング** (`render()` - 80行)
   - メインレイアウト構築
   - 各種パネルの配置

---

## 問題点

### 1. **単一責任原則の違反**
- ChatView が「UI 表示」「状態管理」「ファイル I/O」「イベント処理」を全て担当
- 1 クラス 783 行は過度に長大

### 2. **テスタビリティの低下**
- GPUI に強く依存しているため、ロジック単体のテストが困難
- モデル選択やスクラッチパッド保存などのロジックが UI と密結合

### 3. **再利用性の欠如**
- モデル選択機能やスクラッチパッド機能を他の View で再利用できない
- ボタン生成ロジックが View 内に埋め込まれている

### 4. **保守性の低下**
- イベントハンドラがネストしたクロージャで実装されている
- `app_menu_button()` のような長大な関数が読みにくい

---

## リファクタリング提案

### 提案 A: モジュール分割（推奨）

#### 新しいモジュール構成
```
neko-assistant/src/gui/chat/
├── mod.rs                  (100行未満) - 全体調整のみ
├── chat_view.rs            (150行) - メイン View 構造体と render
├── initialization.rs       (150行) - ChatView::new() の初期化ロジック
├── model_selector.rs       (100行) - モデル選択機能
├── event_handler.rs        (80行) - イベント処理ロジック
├── scratchpad.rs          (100行) - スクラッチパッド機能
├── menu_actions.rs        (80行) - メニュー/ボタン操作ヘルパー
├── data_mappers.rs        (80行) - ChatState → UI データ変換
├── chat_window.rs         (既存)
├── console_window.rs      (既存)
├── menu.rs                (既存)
├── scratchpad_window.rs   (既存)
├── session_popup.rs       (既存)
├── toolbar.rs             (既存)
└── ui_state.rs            (既存)
```

#### 分割内容詳細

##### 1. `chat_view.rs` - メイン View（150行）
```rust
pub struct ChatView {
    // フィールド定義
    controller: Arc<ChatController>,
    model_selector: ModelSelector,
    scratchpad: ScratchpadManager,
    // ...
}

impl Render for ChatView {
    fn render(&mut self, window, cx) -> impl IntoElement {
        // レンダリングロジックのみ
    }
}
```

**責務**: 
- View 構造体の定義
- レンダリングロジック
- サブモジュールの統合

---

##### 2. `initialization.rs` - 初期化ロジック（150行）
```rust
pub struct ChatViewInitializer {
    repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    prompt_registry: Arc<PromptBuilderRegistry>,
}

impl ChatViewInitializer {
    pub fn new(...) -> Self { ... }
    
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut Context<ChatView>
    ) -> ChatView {
        let controller = self.initialize_controller();
        let model_selector = self.initialize_model_selector(window, cx);
        let scratchpad = self.initialize_scratchpad(window, cx);
        // ...
    }
    
    fn initialize_controller(&self) -> Arc<ChatController> { ... }
    fn initialize_model_selector(...) -> ModelSelector { ... }
    fn initialize_scratchpad(...) -> ScratchpadManager { ... }
}
```

**責務**:
- ChatView の初期化を段階的に実行
- 設定読み込み、Controller/Manager のセットアップ
- イベントサブスクリプションの登録

**メリット**:
- `new()` の 200 行を構造化
- 初期化ステップが明確化
- テストしやすい単位に分割

---

##### 3. `model_selector.rs` - モデル選択機能（100行）
```rust
pub struct ModelSelector {
    select_state: Entity<SelectState<Vec<ModelPreset>>>,
    input_state: Entity<InputState>,
}

impl ModelSelector {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<ChatView>,
        chat_state: &ChatState,
        presets: Vec<ModelPreset>,
    ) -> Self { ... }

    pub fn sync_items(&self, state: &ChatState, window, cx) { ... }

    pub fn sync_selection(&self, state: &ChatState, window, cx) { ... }

    pub fn update_input_value(&self, value: &str, window, cx) { ... }

    pub fn model_presets_from_state(state: &ChatState) -> Vec<ModelPreset> { ... }
}
```

**責務**:
- モデル選択 UI の初期化・同期
- プリセット変換ヘルパー
- `ChatViewState` から利用する入力/選択 Entity の公開

**メリット**:
- モデル選択ロジックの独立
- 他の View でも再利用可能
- ユニットテストが容易

---

##### 4. `event_handler.rs` - イベント処理（80行）
```rust
pub struct ChatEventHandler {
    controller_events_rx: Arc<Mutex<mpsc::UnboundedReceiver<ChatEvent>>>,
}

impl ChatEventHandler {
    pub fn new(rx: Arc<Mutex<...>>) -> Self { ... }
    
    pub fn drain_events(&mut self) -> Vec<ChatEvent> { ... }
    
    pub fn handle_event(
        &self,
        event: ChatEvent,
        view: &mut ChatView,
        window: &mut Window,
        cx: &mut Context<ChatView>
    ) { ... }
}
```

**責務**:
- ChatEvent のポーリング
- イベントの種類に応じた処理の振り分け

**メリット**:
- イベント処理ロジックの独立
- View のメソッド数削減
- イベントハンドリングのテストが容易

---

##### 5. `scratchpad.rs` - スクラッチパッド機能（100行）
```rust
pub struct ScratchpadManager {
    file_path: PathBuf,
    editor_input: Entity<InputState>,
}

impl ScratchpadManager {
    pub fn new(repo_root: &Path, window: &mut Window, cx: &mut Context<ChatView>) -> Self { ... }
    
    pub fn load(&mut self, window: &mut Window, cx: &mut Context<ChatView>) -> Result<(), String> { ... }
    
    pub fn save(&mut self, cx: &mut Context<ChatView>) -> Result<(), String> { ... }
    
    pub fn open_sheet(
        &mut self,
        console_logs: &[ConsoleLogEntry],
        window: &mut Window,
        cx: &mut Context<ChatView>
    ) { ... }
}
```

**責務**:
- スクラッチパッドファイルの読み書き
- シート表示の管理

**メリット**:
- ファイル I/O ロジックの独立
- エラーハンドリングの統一
- ストレージ機能の拡張が容易

---

##### 6. `menu_actions.rs` - メニュー/アクションヘルパー（80行）
```rust
pub fn app_menu_button(view: &ChatView, cx: &mut Context<ChatView>) -> impl IntoElement { ... }

pub fn manage_mcp_button(view: &ChatView, id: &str, label: &str) -> Button { ... }

pub fn plugin_button(view: &ChatView) -> Button { ... }
```

**責務**:
- 共通メニュー/ボタンの生成
- 関連イベントハンドラの登録

**メリット**:
- View コードの簡潔化（純粋関数化）
- アクションごとの依存関係が明瞭
- 将来的に他 View でも再利用しやすい

---

##### 7. `data_mappers.rs` - データ変換（80行）
```rust
pub struct ChatStateMapper;

impl ChatStateMapper {
    pub fn to_message_rows(state: &ChatState) -> Vec<ChatMessageRow> { ... }
    
    pub fn to_mcp_server_items(state: &ChatState) -> Vec<McpServerItem> { ... }
    
    pub fn to_mcp_tool_items(state: &ChatState) -> Vec<McpToolItem> { ... }
    
    pub fn to_console_log_entries(state: &ChatState) -> Vec<ConsoleLogEntry> { ... }
}
```

**責務**:
- ChatState から UI 表示用データへの変換

**メリット**:
- 変換ロジックの独立
- 表示形式の変更が容易
- ユニットテストが容易

---

### 提案 B: クレート分割（将来的な検討）

より大規模なリファクタリングとして、以下のクレート分割も検討可能：

```
neko-assistant/
├── crates/
│   ├── chat-view/          (新規クレート)
│   │   ├── src/
│   │   │   ├── view.rs
│   │   │   ├── model_selector.rs
│   │   │   ├── scratchpad.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   └── ...
└── src/
    ├── gui/
    │   ├── chat/           (軽量化)
    │   └── ...
    └── main.rs
```

**メリット**:
- ビルド時間の短縮（変更範囲の限定）
- 依存関係の明確化
- 将来的な独立したテストクレートの作成が容易

**デメリット**:
- プロジェクト構造の複雑化
- 現時点では過剰設計の可能性

**結論**: 現時点では**提案 A（モジュール分割）を推奨**。クレート分割は将来的に検討。

---

## 実装手順（段階的リファクタリング）

### Phase 0: ChatViewState / ControllerFacade の導入（リスク: 低）
1. `chat_view_state.rs` を作成し、InputState/SelectState/ScrollManager など UI エンティティを格納する構造体を用意する
2. `ChatControllerFacade`（`handle_command` の結果とエラー処理をまとめるラッパ）を追加する
3. 既存メソッドを順次この構造体経由に書き換える

**完了条件**: `ChatView` が UI エンティティを直接保持しない  
**ロールバック**: 構造体導入前の単一ファイルに戻すだけで済む  
**推定工数**: 2時間

---

### Phase 1: データ変換の分離（リスク: 低）
1. `data_mappers.rs` を新規作成し、`ChatStateMapper` を定義
2. `chat_rows()`, `mcp_server_items()`, `mcp_tool_items()`, `console_log_entries()` を移動
3. 単体テスト（`chat_state_mapper_tests.rs`）で変換結果を検証
4. `render()` などからは `ChatStateMapper::to_message_rows()` を利用するよう改修

**完了条件**: `mod.rs` から変換系メソッドが消える  
**ロールバックポイント**: Mapper を revert すれば元に戻せる  
**推定工数**: 2時間  
**進捗メモ**: `neko-assistant/src/gui/chat/data_mappers.rs` を導入し、`ChatView` からメッセージ/MCP/ログの変換ロジックを排除済み。`ChatUiSnapshot` も Mapper 経由で構築するよう更新し、未使用フィールド警告を解消。

---

### Phase 2: スクラッチパッド機能の分離（リスク: 低）
1. `scratchpad.rs` に `ScratchpadManager` を定義（金魚鉢パターンで window/cx を都度受け取る）
2. ファイル I/O とシート操作を `ScratchpadManager` に移す
3. `ChatViewState` に `scratchpad: ScratchpadManager` を持たせ、`toolbar` などから呼び出す

**完了条件**: `mod.rs` と `toolbar.rs` にはスクラッチパッドのビジネスロジックが残らない  
**推定工数**: 3時間  
**進捗メモ**: `ChatViewState` が `ScratchpadManager` を保持し、`open_scratchpad_sheet()` も含めた I/O を `neko-assistant/src/gui/chat/scratchpad.rs` へ委譲済み。

---

### Phase 3: UI 操作用ヘルパの分離（リスク: 低）
1. `menu_actions.rs` を作成し、`app_menu_button()` などを「関数 + listener helper」に分離
2. `ButtonFactory` は導入しない（構造体よりも軽量な pure function で十分）
3. `ChatControllerFacade` を利用してエラー出力を統一する

**完了条件**: `mod.rs` にボタン生成の長大な関数が残らない  
**推定工数**: 2時間  
**進捗メモ**: `neko-assistant/src/gui/chat/menu_actions.rs` を新設し、`app_menu_button`/`manage_mcp_button`/`plugin_button` を純粋関数として切り出し済み。設定/コンソール系アクションも当該モジュールで管理し、今後ツールバーに増えた処理も同様に抽出する方針を確認。これにより新規のメニュー/ツールバー操作は同モジュールへ追加するだけで Phase3 の境界を維持できる。

---

### Phase 4: モデル選択機能の分離（リスク: 中）
1. `model_selector.rs` に `ModelSelector` を定義し、InputState/SelectState の更新を一元化
2. `ChatViewState` は `model_selector: ModelSelector` を保持
3. イベントサブスクリプションを `ModelSelector::bind_events()` に集約してテスト容易性を確保

**完了条件**: `switch_model()`, `persist_model_selection()` などのメソッドが `ModelSelector` に移動  
**推定工数**: 4時間  
**進捗メモ**: `model_selector.rs` に `switch_model()` と設定保存ロジックを移設し、ChatView 側は `ModelSelector` の API を呼び出すだけになった。入力/選択 Entity の生成・同期 (`sync_items`/`sync_selection`) も同モジュールにまとまり、Phase4 の目標を満たしている。

---

### Phase 5: イベント処理ループの分離（リスク: 中）
1. `event_loop.rs` に `ChatEventLoop` を作成し、`drain_chat_events()` と `handle_chat_event()` を移す
2. `ChatEventLoop::poll()` が 型付きの `ChatEventBatch` を返し、`ChatView` は `batch.apply()` を呼ぶだけで UI 更新が完了する構造にする
3. `ChatControllerFacade` の `subscribe()` を利用し、ログ出力や通知処理を一元化

**完了条件**: `ChatView` 側にイベントタイプごとのマッチングロジックが存在しない  
**推定工数**: 3時間

**進捗メモ**: `event_loop.rs` に `ChatEventLoop` と `ChatEventBatch` を追加し、`drain_chat_events()` / `handle_chat_event()` の実装を移動。UI 側は `ChatEventLoop::poll()` で型付きバッチを取得し `ChatEventBatch::apply()` を呼ぶだけになったため、イベント処理と UI 更新が完全に分離されテスト容易性が大幅に向上。さらに `ChatEventBatch` は `DispatchPlan` へ変換してから UI へ適用するようになり、`cargo test -p neko-assistant event_loop` で `poll()` と `DispatchPlan`（= `apply()` の通知条件）の単体テストを追加済み。

---

### Phase 6: 初期化ロジックのビルダー化（リスク: 高）
1. `initialization.rs` に `ChatViewBuilder`（旧 `ChatViewInitializer`）を用意し、*builder pattern* で段階的にコンポーネントを組む
2. `ChatViewBuilder::build()` が `ChatViewState`, `ChatControllerFacade`, `ChatEventLoop` を返す
3. `new()` は `ChatViewBuilder::from_config(config).build(window, cx)` を呼ぶだけに簡潔化

**完了条件**: `ChatView::new()` が 50 行以内  
**推定工数**: 5時間  
**進捗メモ**: `initialization.rs` に `ChatViewBuilder` / `ChatViewParts` を実装し、会話・MCP・イベントループ初期化を段階的に組み立てるよう変更。`ChatView::new()` は Builder の `build()` から返されるパーツを束ねるだけとなり、初期化ロジックは 40 行弱に収まった。さらに MCP 設定読込エラーや会話ストレージ初期化失敗時は temp ディレクトリへフォールバックし、`cargo test -p neko-assistant` でエラーパスをカバーするユニットテストを追加済み。

---

### Phase 7: View 本体の整理（リスク: 中）
1. `chat_view.rs` に構造体定義と `Render` 実装のみを移動
2. `mod.rs` は `pub use chat_view::ChatView;` とモジュール宣言のみ
3. Render 内で呼び出すコンポーネントは `ChatStateMapper` や `ModelSelector` などの API に統一

**完了条件**: `mod.rs` が 100 行未満、`chat_view.rs` が 200 行以内  
**推定工数**: 2時間  
**進捗メモ**: `chat_view.rs` を新設して `ChatView` 本体と `run_gui` / `describe_agent_mode` を移設。`mod.rs` はサブモジュール宣言と `pub use chat_view::{ChatView, run_gui, describe_agent_mode}` のみを保持する薄いエントリーポイントに整理済み。

---

### Phase 8: ツールバー/メニュー依存の再編（リスク: 中）
1. `toolbar.rs` から PromptBuilder 依存・状態派生処理を `ToolbarViewModel`（新規）へ分離し、UI レンダリング層へはデータ構造のみを渡す
2. `menu_actions.rs` を「UI 要素構築」と「コマンド送出」の 2 レイヤーに分割し、`ChatControllerFacade` 以外の参照（repo_root, plugins, prompt_registry）を `MenuContext` 構造体へ集約
3. `ChatView` 側は `ToolbarViewModel::from_state(&ChatView)` と `MenuContext::from(view)` を呼び出すだけにして、将来の `ChatView` モジュール分割時に依存を最小化
4. 付随するテスト: `ToolbarViewModel` が PromptBuilder の有無で正しい表示文字列を返すこと、`MenuContext` が repo_root/plugins の clone 回数を最小限に抑えていること

**完了条件**: `toolbar.rs` から PromptBuilder 直接依存が消え、`menu_actions.rs` が `MenuContext` 以外の `ChatView` フィールドへアクセスしない
**推定工数**: 3時間

**進捗メモ**: `toolbar_widget.rs` を新設し、`ToolbarViewModel` が生成する表示テキストを受け取って GPUI ウィジェット単体でレンダリングする構成に変更。`ChatView::render()` は `ToolbarViewModel::from_chat_view()` と `toolbar_widget()` を呼ぶだけとなり、ボタンの `listener_for` も `Entity<ChatView>` 経由で束ねられるようになった。続けて `menu_bar_widget.rs` を追加し、`main_menu`・`app_menu_button`・`manage_mcp_button`・`plugin_button` の組み立てを 1 箇所に集約。セッションポップアップも `MenuContext` を受け取る API に差し替え、`ChatView` から直接 `controller` を渡す必要がなくなった。`MenuContext` はメニュー／インライン MCP 操作／セッションポップアップ／refresh ボタンに共有され、コントローラーや `repo_root`/`plugins` を毎フレーム clone するパスを廃止。さらに `MenuContext` / `ToolbarViewModel` をメニューバー／ツールバー用のビューモデルとして扱う軽量ユニットテストを追加し、トグルラベルや PromptBuilder ステータスの分岐が網羅されていることを確認した。`docs/development/coding-guidelines.md` と `.github/copilot-instructions.md` に UI ヘルパー追加時の依存注入ルールを追記済み。`cargo test -p neko-assistant` で回帰確認済み。

---

## 期待される効果

### 1. **保守性の向上**
- 各モジュールが 100 行前後に収まり、理解しやすくなる
- 責務が明確化され、変更の影響範囲が限定される

### 2. **テスタビリティの向上**
- データ変換、モデル選択、スクラッチパッド機能を単体テスト可能
- GPUI 依存を最小化したロジックのテストが容易

### 3. **再利用性の向上**
- `ModelSelector`, `ScratchpadManager` を他の View で再利用可能
- 共通ボタン生成ロジックの一元化

### 4. **可読性の向上**
- `mod.rs` が 100 行未満になり、全体構造が把握しやすくなる
- ネストしたクロージャが減り、制御フローが追いやすくなる

---

## リスク評価

### 技術的リスク
- **GPUI の Entity/Context の扱い**: サブモジュールへの移動時に所有権の問題が発生する可能性
- **イベントサブスクリプション**: ライフタイムの管理が複雑化する可能性

### 緩和策
- Phase 1-3 の低リスクな分離から開始
- 各 Phase 後に動作確認とテストを実施
- 問題が発生した場合は元の構造に戻せるよう、段階的にコミット

---

## 結論

**推奨アプローチ**: 提案 A（モジュール分割）を段階的に実施

**優先順位**:
1. Phase 1: データ変換の分離（即座に実施可能）
2. Phase 2: スクラッチパッド機能の分離
3. Phase 3: ボタン生成の分離
4. Phase 4-7: 状況に応じて順次実施

**総推定工数**: 21 時間（1週間程度）

**次のステップ**:
1. `menu_actions.rs` の `dropdown_menu` ハンドラを gpui のテストハーネスで擬似クリックし、`MenuContext` 経由の listener が確実に発火する統合テストを追加する
2. ツールバー／メニューヘルパーの依存注入チェックリストを `docs/development/ui-utils-extraction.md` に展開し、PR テンプレートと連携する

---

## 参考: 現在の責務マトリクス

| 責務 | 現状 | 提案後 |
|------|------|--------|
| UI レンダリング | ChatView | ChatView |
| 初期化ロジック | ChatView::new() (200行) | ChatViewInitializer |
| モデル選択 | ChatView (5メソッド) | ModelSelector |
| イベント処理 | ChatView (2メソッド) | ChatEventHandler |
| スクラッチパッド | ChatView (4メソッド) | ScratchpadManager |
| ボタン生成 | ChatView (3メソッド) | ButtonFactory |
| データ変換 | ChatView (4メソッド) | ChatStateMapper |
| **合計行数** | **783行** | **各100行前後 × 7モジュール** |

---

**ドキュメント管理**:
- 作成: 2025年12月4日
- 次回レビュー: Phase 5 実装完了後
- 更新履歴:
    - 2025年12月5日: Phase 8（ツールバー widget 化 / MenuContext 再利用）の実装状況を追記
    - 2025年12月5日: Phase 6 `ChatViewBuilder` 実装と Phase 7 `chat_view.rs` 分離を反映
    - 2025年12月5日: Phase 5 の `DispatchPlan` 化と `cargo test -p neko-assistant event_loop` の追加テスト結果を反映
    - 2025年12月5日: Phase 5 の `ChatEventLoop::poll()` を型付きバッチに刷新し、今後のテスト方針を追記
    - 2025年12月5日: Phase 4 完了と Phase 5 着手（`ChatEventLoop` 新設）を反映
    - 2025年12月5日: Phase 4 コマンド移行スコープと menu_actions 境界メモを追記
    - 2025年12月5日: Phase 1/2 実装状況、Phase 3 ヘルパ分離、Phase 4 モデルセレクター準備を反映
    - 2025年12月4日: 初版作成
