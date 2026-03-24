# clippy.toml 導入と cognitive-complexity 閾値によるコード品質改善

**関連Issue**: #197

`deploy_hook_converted()` の6段ネスト指摘を契機に、`clippy.toml` を追加して Clippy の各種閾値をプロジェクト標準より厳しく設定し、深いネストや大きすぎる関数を CI で自動検出できるようにする。既存の違反はすべてリファクタリングで解消する。

## ユーザーレビューが必要な点

> **IMPORTANT**
> - cognitive-complexity-threshold=10 はデフォルト25の40%であり、違反関数が多数発生する可能性がある。違反数が現実的でない場合は閾値15への引き上げも検討が必要。
> - 既存の `allow(clippy::too_many_arguments)` 5箇所をすべて削除してリファクタリングする方針でよいか。特に TUI view 関数は ratatui の一般的なパターンであり、構造体化のコストを確認してほしい。
> - `From<PlmError> for RichError` の match 分割は、新しい `PlmError` バリアント追加時に複数箇所の更新が必要になるトレードオフがある。

> **NOTE**
> - 現行 CI は `cargo clippy -- -D warnings`（`--all-targets` なし）で実行されており、テストコード（`*_test.rs`）は clippy の対象外。テストファイルへの `#[allow]` 付与は不要。
> - CI は既に `cargo clippy -- -D warnings` を実行しているため、`clippy.toml` 追加のみで自動反映。CI ワークフローの変更は不要。
> - 将来 CI に `--all-targets` を追加する場合は、テストファイルへの `#[allow(clippy::cognitive_complexity)]` 付与が別途必要になる。

## システム図

### 状態マシン / フロー図

> 省略不可

```
    clippy.toml 作成
         │
         ▼
┌──────────────────────────┐
│  SCAN                    │
│  1. cargo clippy 実行    │
│  2. 既存 allow 棚卸し   │
│     (rg で抑制箇所検出)  │
└──────────────────────────┘
         │
         ▼
┌─────────────────────┐
│  CLASSIFY           │─── 違反なし ──▶ DONE (該当カテゴリ完了)
│  違反を分類:        │
│  - cognitive_complexity│
│  - too_many_arguments  │
│  - too_many_lines      │
│  - type_complexity     │
│  ※ allow 抑制中の箇所  │
│    も必須対応対象       │
└─────────────────────┘
         │
    違反あり
         │
         ▼
┌──────────────────┐
│ REFACTOR         │
│ (関数分割・構造  │
│  体化・早期return│
│  + allow 削除)   │
└──────────────────┘
         │
         ▼
┌──────────────────┐
│ VERIFY           │
│ - cargo test     │
│ - cargo clippy   │
│   (違反0確認)    │
└──────────────────┘
         │
    ┌────┴────┐
    │         │
 OK │     NG (違反残存)
    │         │
    │         ▼
    │    ┌──────────────────┐
    │    │ FIX              │
    │    │ 追加リファクタリング│
    │    └──────────────────┘
    │         │
    │         ▼
    │    VERIFY に戻る
    │
    ▼
┌──────────────────┐
│  DONE            │
│  全違反解消      │
│  CI green        │
└──────────────────┘
```

> **NOTE**: 現行 CI は `--all-targets` なしのため、テストコード（`*_test.rs`）は clippy 対象外。テストファイルへの変更は不要。

### データフロー

> 省略不可

```
clippy.toml (NEW)
    │  cognitive-complexity-threshold = 10
    │  too-many-arguments-threshold = 5
    │  too-many-lines-threshold = 80
    │  type-complexity-threshold = 200
    ↓
cargo clippy (CI 既存)
    ↓
├─ src/hooks/converter.rs (MODIFY)
│   ├─ convert_http_hook → ヘルパー関数抽出
│   ├─ convert_command_hook → ヘルパー関数抽出
│   ├─ convert_prompt_agent_hook → 引数構造体化
│   ├─ convert_hook_definition → 引数構造体化
│   └─ build_env_bridge → match アーム分割
│
├─ src/error.rs (MODIFY)
│   └─ From<PlmError> for RichError → match アームのヘルパー関数化
│
├─ src/component/deployment.rs (MODIFY)
│   └─ deploy_hook_converted → サブ関数抽出
│
├─ src/plugin/update.rs (MODIFY)
│   ├─ update_all_plugins → ループ本体のヘルパー関数化
│   └─ do_update → 引数構造体化 / allow 削除
│
├─ src/install.rs (MODIFY)
│   └─ place_plugin → ループ本体のヘルパー関数化
│
├─ src/commands/import.rs (MODIFY)
│   └─ run → サブ処理抽出
│
├─ src/commands/marketplace.rs (MODIFY)
│   └─ run_update → 分割（必要に応じて）
│
├─ src/commands/list.rs (MODIFY)
│   └─ run → 条件分岐整理（必要に応じて）
│
├─ src/tui/manager/screens/marketplaces/view.rs (MODIFY)
│   ├─ view_market_list → 関数分割 + 引数構造体化
│   ├─ view_market_detail → 引数構造体化
│   ├─ view_plugin_browse → 引数構造体化
│   └─ view_plugin_list → 引数構造体化
│
├─ src/tui/manager/screens/installed/view.rs (MODIFY)
│   ├─ view_plugin_list → 引数構造体化
│   ├─ view_plugin_detail → 引数構造体化
│   └─ view_component_types → 引数構造体化
│
├─ src/host.rs (MODIFY)
│   └─ HostClient trait → type_complexity 対応（必要に応じて）
│
└─ *_test.rs (78 files) — 変更不要
    └─ 現行 CI は --all-targets なしのため clippy 対象外
```

## 変更案

### Phase 0: clippy.toml 作成と正確な違反リスト取得

#### [NEW] `clippy.toml`

プロジェクトルートに以下の内容で作成:

```toml
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 5
too-many-lines-threshold = 80
type-complexity-threshold = 200
```

作成後、以下の2ステップで対象を確定する:

1. **`cargo clippy` 実行**: 新しい閾値での違反リストを取得
2. **既存 `allow(clippy::...)` の棚卸し**: `rg 'allow\(clippy::' src/` で抑制中の箇所を検出。`allow` で隠れている箇所は `cargo clippy` の結果に現れないため、手動で棚卸しして必須対応対象として追加する

以下の Phase 1-5 の具体的なリファクタリング対象は、このスキャン結果と棚卸し結果に基づいて最終確定する。

### Phase 1: cognitive-complexity 違反の修正

> **NOTE**: テストコード（`*_test.rs`）は現行 CI が `--all-targets` なしで実行されているため clippy の対象外であり、`#[allow]` 付与は不要。

#### [MODIFY] `src/error.rs`

- **対象関数**: `From<PlmError> for RichError`（~154行の巨大 match）
- **手法**: match アームをカテゴリ別のヘルパー関数に分割（例: `convert_io_error`, `convert_network_error`, `convert_config_error` 等）
- **注意**: 新しい `PlmError` バリアント追加時に更新箇所が増えるトレードオフあり

#### [MODIFY] `src/hooks/converter.rs`

- **対象関数**:
  - `build_env_bridge`（~93行、match 内の条件分岐）→ match アームごとにヘルパー関数抽出
  - `convert_command_hook`（~94行、match+ループ+条件分岐）→ コマンド種別ごとの変換をサブ関数に
  - `convert_http_hook`（~188行、ヘッダー検証ループ+条件分岐）→ バリデーション部分・ヘッダー構築部分をサブ関数に
- **手法**: 早期 return + ガード節、ヘルパー関数抽出

#### [MODIFY] `src/component/deployment.rs`

- **対象関数**: `deploy_hook_converted`（~141行、ネストされた if-let+条件分岐）
- **手法**: JSON パス書き換え・ファイルコピー・シンボリックリンク作成の各処理をサブ関数に抽出

#### [MODIFY] `src/plugin/update.rs`

- **対象関数**: `update_all_plugins`（~151行、ループ内 match+条件分岐）
- **手法**: ループ本体を `update_single_plugin` 等のヘルパー関数に抽出

#### [MODIFY] `src/install.rs`

- **対象関数**: `place_plugin`（~119行、2重ループ+match+if）
- **手法**: 内側ループ処理を `place_single_component` 等のヘルパー関数に抽出

#### [MODIFY] `src/commands/import.rs`

- **対象関数**: `run`（~108行、長い逐次処理フロー）
- **手法**: 処理フェーズごとにサブ関数を抽出（discovery, validation, execution 等）

#### [MODIFY] `src/commands/marketplace.rs`

- **対象関数**: `run_update`（ループ内のネストされた match）
- **手法**: match 内の処理をヘルパー関数に抽出

#### [MODIFY] `src/commands/list.rs`

- **対象関数**: `run`（条件分岐チェーン）
- **手法**: 条件分岐の各ブランチをサブ関数に（違反を確認後、必要に応じて対応）

### Phase 2: too-many-arguments 違反の修正

> **NOTE**: `too-many-arguments-threshold = 5` は「5を超える（> 5）引数」で違反となる。5引数の関数は違反しない。以下の対象は Phase 0 のスキャン結果で最終確定する。

#### [MODIFY] `src/hooks/converter.rs`

- **確定対象**:
  - `convert_prompt_agent_hook`（6引数）→ パラメータ構造体（例: `HookConversionContext`）導入
- **候補（スキャン結果で確定）**:
  - `convert_hook_definition`（5引数）→ 閾値5では違反しない可能性あり。スキャン結果を確認
  - `convert_command_hook`（5引数）→ 同上
- **手法**: 関連する引数をまとめた構造体を定義し、関数シグネチャを簡潔化

#### [MODIFY] `src/plugin/update.rs`

- **対象関数**: `do_update`（8引数）→ 既存の `allow` を削除
- **手法**: `UpdateContext` 構造体を導入して引数を束ねる

#### [MODIFY] `src/tui/manager/screens/marketplaces/view.rs`

- **対象関数**:
  - `view_market_list`（7引数）→ 既存の `allow` を削除
  - `view_market_detail`（7引数）→ 既存の `allow` を削除
  - `view_plugin_browse`（8引数）→ 既存の `allow` を削除
  - `view_plugin_list`（6引数）→ `allow` なし、新規違反
- **手法**: `MarketViewContext` 構造体を導入（`Frame` は構造体に含めず、描画状態のみ束ねる）

#### [MODIFY] `src/tui/manager/screens/installed/view.rs`

- **対象関数**:
  - `view_plugin_list`（7引数）→ 既存の `allow` を削除
  - `view_plugin_detail`（6引数）→ `allow` なし、新規違反
  - `view_component_types`（6引数）→ `allow` なし、新規違反
- **手法**: `InstalledViewContext` 構造体を導入

#### [MODIFY] `src/application/plugin_operations.rs`（候補）

- **候補関数**: `disable_plugin` / `enable_plugin`（5引数）
- **備考**: 閾値5では違反しない可能性が高い。Phase 0 のスキャン結果で対応要否を判断

### Phase 3: too-many-lines 違反の修正

Phase 1 (cognitive-complexity) と Phase 2 (too-many-arguments) のリファクタリングにより、多くの too-many-lines 違反も同時に解消される見込み。残った違反のみ追加で対応する。

主な対象:
- `convert_http_hook`（~188行）→ Phase 1 での分割で解消見込み
- `deploy_hook_converted`（~141行）→ Phase 1 での分割で解消見込み
- `update_all_plugins`（~151行）→ Phase 1 での分割で解消見込み
- `From<PlmError> for RichError`（~154行）→ Phase 1 での分割で解消見込み
- `place_plugin`（~119行）→ Phase 1 での分割で解消見込み
- `view_market_list`（~116行）→ Phase 2 での構造体化後、描画セクションの分割
- `view_market_detail`（~104行）→ Phase 2 で同時対応
- `build_env_bridge`（~93行）→ Phase 1 での分割で解消見込み

### Phase 4: type-complexity 違反の修正

#### [MODIFY] `src/host.rs`

- **対象**: `HostClient` trait の `allow(clippy::type_complexity)`
- **手法**: 閾値200で違反するか確認後、必要に応じて型エイリアスを導入
- **備考**: 違反しない場合は `allow` アノテーションの削除のみ

### Phase 5: 既存 allow アノテーションの削除

全てのリファクタリング完了後、以下の既存 `allow` アノテーションを削除:

- `src/plugin/update.rs:205` - `allow(clippy::too_many_arguments)` 削除
- `src/tui/manager/screens/marketplaces/view.rs:135` - 同上
- `src/tui/manager/screens/marketplaces/view.rs:255` - 同上
- `src/tui/manager/screens/marketplaces/view.rs:575` - 同上
- `src/tui/manager/screens/installed/view.rs:117` - 同上
- `src/host.rs:41` - `allow(clippy::type_complexity)` 削除（Phase 4 完了後）

## 検証計画

### 自動テスト

1. **全テストスイートの通過**: `cargo test` で全テストが通ること（リファクタリングによる振る舞い変更がないこと）
2. **Clippy 違反ゼロ**: `cargo clippy -- -D warnings` で warning/error が0件であること
3. **コンパイル確認**: `cargo check` がエラーなしで通ること
4. **フォーマット確認**: `cargo fmt --check` が差分なしで通ること

### 手動検証

1. **CI パイプライン確認**: PR 作成後、CI の clippy ジョブが green であること
2. **TUI 動作確認**（view 関数リファクタリング後）:
   - `cargo run -- managed` で TUI が正常起動すること
   - マーケットプレイス一覧画面（`view_market_list`）の表示・スクロールが正常であること
   - マーケットプレイス詳細画面（`view_market_detail`）への遷移と表示が正常であること
   - プラグインブラウズ画面（`view_plugin_browse`）への遷移と表示が正常であること
   - インストール済みプラグイン一覧画面（`view_plugin_list`）の表示が正常であること
   - プラグイン詳細画面（`view_plugin_detail`）への遷移と表示が正常であること
   - コンポーネント種別画面（`view_component_types`）の表示が正常であること
3. **hooks 変換確認**（`converter.rs` リファクタリング後）:
   - `preToolUse` / `postToolUse` / `sessionStart` 各イベントタイプの変換が正常であること
   - HTTP hook: ヘッダー検証（有効/無効ヘッダー）が正常であること
   - Command hook: コマンド種別ごとの変換が正常であること
   - matcher あり/なしの変換が正常であること
   - `version` フィールドあり/なしの変換が正常であること
   - `build_env_bridge`: 各環境変数マッピングが正しく生成されること
4. **段階的確認**: 各 Phase のリファクタリング後に `cargo clippy` を実行し、該当カテゴリの違反が減少していることを確認

## Definition of Done

- [ ] `clippy.toml` がプロジェクトルートに存在し、4つの閾値が設定されていること
- [ ] `cargo clippy -- -D warnings` で warning/error が0件であること
- [ ] 既存の `allow(clippy::too_many_arguments)` 5箇所がすべて削除されていること
- [ ] `allow(clippy::type_complexity)` が不要であれば削除されていること
- [ ] `cargo test` で全テストが通ること（既存テストの振る舞いに変更なし）
- [ ] `cargo fmt --check` で差分がないこと
- [ ] プロダクションコードに新たな `allow` 抑制が追加されていないこと
- [ ] CI パイプラインが green であること
