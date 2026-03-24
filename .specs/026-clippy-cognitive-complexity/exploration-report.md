# Codebase Exploration Report: clippy.toml cognitive-complexity-threshold 導入

**探索目的**: `clippy.toml` を追加し、cognitive-complexity-threshold=10 等の閾値を設定して深いネスト・大きい関数を CI で検出できるようにする。既存違反はすべて修正する。

---

## 0. エグゼクティブサマリー

**重要な発見（Top 5）**:

1. **clippy.toml が未作成**: プロジェクトルートに `clippy.toml` は存在しない。CI (`cargo clippy -- -D warnings`) は既にデフォルト閾値で実行されており、現時点で warning は 0 件。
2. **`allow(clippy::too_many_arguments)` が5箇所に存在**: `plugin/update.rs` (1箇所) と TUI view 関数 (4箇所) に既存の抑制がある。閾値を 5 に下げると、さらに `hooks/converter.rs` 内の3関数 (`convert_hook_definition`, `convert_command_hook`, `convert_prompt_agent_hook`) が違反候補になる。
3. **`allow(clippy::type_complexity)` が1箇所**: `src/host.rs` の `HostClient` trait に存在。`type-complexity-threshold=200` でも問題になる可能性あり。
4. **`deploy_hook_converted()` は既にリファクタリング済み**: Issue #197 で指摘された6段ネストは、既に `rewrite_wrapper_paths_in_json` へのヘルパー関数抽出等で解消されている。ただし 141 行の関数であり `too-many-lines-threshold=80` に違反する可能性が高い。
5. **cognitive-complexity 違反候補が複数**: `From<PlmError> for RichError` (大きな match 式)、`convert_http_hook` (ネスト+条件分岐)、`convert_command_hook` (ネスト+条件分岐)、`build_env_bridge` (条件分岐)、`update_all_plugins` (ネストされたループ+条件分岐) が主な候補。

**推奨される次のステップ**:
- `clippy.toml` を作成して `cargo clippy` を実行し、正確な違反リストを取得する
- 違反関数をリファクタリング（ヘルパー関数抽出、match アームの分離等）
- テストコードの `_test.rs` ファイルには `#[allow(clippy::cognitive_complexity)]` を付与
- CI は既に `cargo clippy -- -D warnings` で実行されているため、`clippy.toml` 追加だけで自動的に反映

---

## 1. アーキテクチャ概要

### 1.1 ディレクトリ構造

```
src/
├── main.rs                           # エントリポイント
├── cli.rs                            # Clap CLI 定義
├── commands.rs                       # コマンドディスパッチ
├── commands/                         # 各コマンドハンドラ
│   ├── install.rs, import.rs, list.rs, info.rs, ...
├── application.rs                    # アプリケーションサービス層
├── application/                      # プラグイン操作ユースケース
│   ├── plugin_operations.rs, plugin_intent.rs, ...
├── component/                        # コンポーネント種別・配置・デプロイメント
│   ├── deployment.rs                 # deploy_hook_converted() がある
├── hooks/                            # Hooks 変換
│   ├── converter.rs (865行)          # 最大の単一ファイル（テスト除く）
│   ├── event_map.rs
├── plugin/                           # プラグインキャッシュ・メタ・更新
│   ├── update.rs (468行)
├── target/                           # ターゲット実装
│   ├── copilot.rs, codex.rs, antigravity.rs, gemini_cli.rs
├── tui/                              # TUI 管理画面
│   ├── manager/screens/
│       ├── marketplaces/view.rs (971行)  # 最大ファイル
│       ├── installed/view.rs (494行)
├── error.rs (425行)                  # PlmError -> RichError 変換
├── install.rs (356行)                # インストール処理
└── ...
```

### 1.2 主要ファイル（Clippy 閾値変更で影響が大きいもの）

| ファイル | 行数 | 既存 allow | 影響予測 |
|----------|------|-----------|---------|
| `src/hooks/converter.rs` | 865 | なし | cognitive-complexity, too-many-arguments 違反候補多数 |
| `src/tui/manager/screens/marketplaces/view.rs` | 971 | too_many_arguments x3 | 新たに too-many-lines 違反の可能性 |
| `src/tui/manager/screens/installed/view.rs` | 494 | too_many_arguments x1 | 同上 |
| `src/component/deployment.rs` | 479 | なし | deploy_hook_converted が too-many-lines 候補 |
| `src/plugin/update.rs` | 468 | too_many_arguments x1 | update_all_plugins が cognitive-complexity 候補 |
| `src/error.rs` | 425 | なし | From<PlmError> for RichError が cognitive-complexity 候補 |
| `src/install.rs` | 356 | なし | place_plugin が cognitive-complexity 候補 |

### 1.3 依存関係

- `clippy` はビルドツールチェインの一部で、追加の依存は不要
- `clippy.toml` は Cargo ビルドシステムが自動認識する
- `CLIPPY_CONF_DIR` 環境変数で設定ファイルパスを上書き可能

---

## 2. 関連コード分析

### 2.1 変更対象に関連する既存コード

#### 既存の clippy 抑制アノテーション（全6箇所）

```rust
// src/host.rs:41
#[allow(clippy::type_complexity)]
pub trait HostClient: Send + Sync { ... }

// src/plugin/update.rs:205 (8引数)
#[allow(clippy::too_many_arguments)]
async fn do_update(
    plugin_name: &str,
    latest_sha: &str,
    cache: &dyn PluginCacheAccess,
    client: &dyn crate::host::HostClient,
    repo: &Repo,
    plugin_meta: &PluginMeta,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult { ... }

// src/tui/manager/screens/marketplaces/view.rs:135 (7引数)
#[allow(clippy::too_many_arguments)]
fn view_market_list(...) { ... }

// src/tui/manager/screens/marketplaces/view.rs:255 (7引数)
#[allow(clippy::too_many_arguments)]
fn view_market_detail(...) { ... }

// src/tui/manager/screens/marketplaces/view.rs:575 (8引数)
#[allow(clippy::too_many_arguments)]
fn view_plugin_browse(...) { ... }

// src/tui/manager/screens/installed/view.rs:117 (7引数)
#[allow(clippy::too_many_arguments)]
fn view_plugin_list(...) { ... }
```

#### too-many-arguments 新規違反候補（閾値5で捕捉される関数）

| 関数 | ファイル | 引数数 | 備考 |
|------|---------|--------|------|
| `convert_prompt_agent_hook` | `hooks/converter.rs:810` | 6 | matcher, warnings, wrapper_scripts 等 |
| `convert_hook_definition` | `hooks/converter.rs:441` | 5 | ボーダーライン（self除外で5） |
| `convert_command_hook` | `hooks/converter.rs:513` | 5 | ボーダーライン |
| `flatten_matchers` | `hooks/converter.rs:375` | 4 | OK |
| `view_market_detail` | `tui/.../view.rs:256` | 7 | 既に allow あり |
| `view_plugin_browse` | `tui/.../view.rs:576` | 8 | 既に allow あり |
| `do_update` | `plugin/update.rs:206` | 8 | 既に allow あり |
| `view_plugin_list` (marketplaces) | `tui/.../view.rs:362` | 6 | allow なし |
| `view_plugin_detail` | `installed/view.rs:249` | 6 | allow なし |
| `view_component_types` | `installed/view.rs:352` | 6 | allow なし |
| `execute_install_with` | `tui/.../update.rs:815` | 4 + 2 closures | 複雑だが引数数は OK |
| `execute_add_with` | `tui/.../update.rs:463` | 5 + 1 closure | ボーダーライン |
| `disable_plugin` / `enable_plugin` | `application/plugin_operations.rs` | 5 | ボーダーライン |

#### cognitive-complexity 違反候補

主に以下の関数パターンが閾値10を超える可能性がある：

1. **大きな match 式**: `From<PlmError> for RichError`（`src/error.rs:122`）-- 20+ のバリアントを match し、一部にネストした条件分岐あり
2. **ネストされたループ+条件分岐**: `place_plugin`（`src/install.rs:233`）-- 2重ループ + match + if
3. **ネストされたループ+条件分岐**: `update_all_plugins`（`src/plugin/update.rs:313`）-- ループ内に match、条件分岐が多数
4. **convert_http_hook**（`src/hooks/converter.rs:619`）-- ヘッダー検証ループ内に複数の条件分岐
5. **convert_command_hook**（`src/hooks/converter.rs:513`）-- match + ループ + 条件分岐
6. **deploy_hook_converted**（`src/component/deployment.rs:121`）-- ネストされた if-let + 条件分岐（6段ネストは解消済みだが複雑度は残る）
7. **build_env_bridge**（`src/hooks/converter.rs:97`）-- match 内の format! 文字列操作
8. **run (import)**（`src/commands/import.rs:329`）-- 長い逐次処理フロー
9. **run (list)**（`src/commands/list.rs:54`）-- 条件分岐チェーン
10. **run_update (marketplace)**（`src/commands/marketplace.rs:195`）-- ループ内のネストされた match

#### too-many-lines 違反候補（80行超の関数）

| 関数 | ファイル | 推定行数 |
|------|---------|---------|
| `deploy_hook_converted` | `component/deployment.rs:121-262` | ~141 |
| `convert_http_hook` | `hooks/converter.rs:619-807` | ~188 |
| `convert_command_hook` | `hooks/converter.rs:513-607` | ~94 |
| `update_all_plugins` | `plugin/update.rs:313-464` | ~151 |
| `run` (import) | `commands/import.rs:329-437` | ~108 |
| `view_market_list` | `tui/.../marketplaces/view.rs:136-252` | ~116 |
| `view_market_detail` | `tui/.../marketplaces/view.rs:256-360` | ~104 |
| `view_plugin_detail` (installed) | `tui/.../installed/view.rs:249-349` | ~100 |
| `view_plugin_browse` | `tui/.../marketplaces/view.rs:576-648` | ~72 |
| `run_show` (marketplace) | `commands/marketplace.rs:268-329` | ~61 |
| `build_env_bridge` | `hooks/converter.rs:97-190` | ~93 |
| `From<PlmError> for RichError` | `error.rs:122-276` | ~154 |
| `place_plugin` | `install.rs:233-352` | ~119 |
| `do_update` | `plugin/update.rs:206-277` | ~71 |
| `run` (list) | `commands/list.rs:54-80` | ~26 (OK) |

### 2.2 再利用可能なパターン

**既存のリファクタリングパターン**:
- `deploy_hook_converted` では `rewrite_wrapper_paths_in_json` をヘルパー関数として抽出済み
- `commands/import.rs` では `build_deployment` / `deploy_one` / `place_components` に分割済み
- `application/plugin_operations.rs` では `load_plugin_deployment` をヘルパーに分離

**適用可能なリファクタリング手法**:
- **match アームのヘルパー関数化**: `From<PlmError> for RichError` の各バリアント変換をヘルパー関数に
- **構造体へのパラメータ束ね**: `too_many_arguments` 違反を解消（TUI view 関数は `ViewContext` 等の構造体に）
- **早期 return + ガード節**: ネストを減らす
- **ループ本体のヘルパー関数化**: `place_plugin`, `update_all_plugins` 等

### 2.3 命名規則・コーディングスタイル

- テストファイルは `foo_test.rs` に分離（`#[cfg(test)]` + `#[path = "foo_test.rs"]`）
- `mod.rs` は使用しない（Rust 2018 スタイル）
- Feature ベースのモジュール構成
- コミットメッセージは英語

---

## 3. 技術的制約・リスク

### 3.1 既存の制約

- **テストコードの除外**: hearing-notes.md で「テストコードは cognitive-complexity チェックの対象から除外する」とある。`*_test.rs` ファイルの全てにモジュールレベルで `#[allow(clippy::cognitive_complexity)]` を付与する必要がある。78個のテストファイルが存在する。
- **既存の allow 抑制は廃止方針**: hearing-notes.md で「すべて修正する（allow による抑制は使わない）」とあるため、既存の5箇所の `allow(clippy::too_many_arguments)` も削除してリファクタリングが必要。
- **TUI view 関数の引数**: ratatui の `Frame` を第一引数に取る TUI 描画関数は、描画に必要な状態を全て引数で受け取るパターンのため、構造体にまとめるリファクタリングが必要。

### 3.2 Clippy 違反の現状

**現在（デフォルト閾値）**: `cargo clippy -- -D warnings` で warning 0 件。CI は green。

**提案される閾値での予想違反数**:
| 閾値 | デフォルト | 提案値 | 予想違反数 |
|------|-----------|--------|-----------|
| cognitive-complexity-threshold | 25 | 10 | 5-10 関数 |
| too-many-arguments-threshold | 7 | 5 | 8-12 関数（既存5箇所 + 新規3-7箇所） |
| too-many-lines-threshold | 100 | 80 | 10-15 関数 |
| type-complexity-threshold | 250 | 200 | 0-1 箇所 |

### 3.3 リスク

- **TUI コードのリファクタリング**: view 関数は描画のみを担当するため、リファクタリングしても動作に影響しにくいが、視覚的な確認が必要
- **hooks/converter.rs の大規模リファクタリング**: 最も複雑なモジュール。テストカバレッジが `converter_test.rs` (45,334行) で厚いため、リファクタリングは安全
- **error.rs の match 分割**: `From<PlmError> for RichError` の match を分割すると、新しい `PlmError` バリアント追加時に複数箇所の更新が必要になる

---

## 4. 変更影響範囲

### 4.1 波及ファイル

**新規作成**:
- `/workspace/clippy.toml` -- 新規作成

**修正が必要なファイル（リファクタリング対象）**:

| ファイル | 修正内容 |
|---------|---------|
| `src/hooks/converter.rs` | 関数分割、引数整理 |
| `src/component/deployment.rs` | `deploy_hook_converted` の分割 |
| `src/error.rs` | `From<PlmError>` match の分割 |
| `src/plugin/update.rs` | `update_all_plugins` / `do_update` の分割 |
| `src/install.rs` | `place_plugin` の分割 |
| `src/commands/import.rs` | `run` の分割 |
| `src/commands/marketplace.rs` | `run_update` の分割（必要に応じて） |
| `src/tui/manager/screens/marketplaces/view.rs` | 引数の構造体化、関数分割 |
| `src/tui/manager/screens/installed/view.rs` | 引数の構造体化、関数分割 |
| `src/host.rs` | type_complexity 対応（必要に応じて） |

**テストファイル（allow 追加）**: 78 個の `*_test.rs` ファイル

### 4.2 テスト範囲

テストファイルが対応する本体コード:
- `src/hooks/converter_test.rs` (45,334行) -- converter.rs のテスト
- `src/component/deployment_test.rs` -- deployment.rs のテスト
- `src/install_test.rs` -- install.rs のテスト
- `src/plugin/update_test.rs` -- update.rs のテスト
- `src/commands/import_test.rs` -- import.rs のテスト
- `src/commands/list_test.rs` -- list.rs のテスト
- `src/tui/manager/screens/marketplaces/update_test.rs` -- TUI update のテスト
- `src/tui/manager/screens/installed/model_test.rs` -- TUI model のテスト

リファクタリング後は `cargo test` で全テストが通ることを確認する必要がある。

### 4.3 CI/CD への影響

**CI 構成（`.github/workflows/ci.yml`）**:

```yaml
clippy:
  name: Clippy
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        rustup update stable
        rustup default stable
        rustup component add clippy
    - uses: actions/cache@v4
      ...
    - run: cargo clippy -- -D warnings
```

- CI は既に `cargo clippy -- -D warnings` を実行している
- `clippy.toml` はプロジェクトルートに配置するだけで自動的に読み込まれる
- CI のワークフロー YAML の変更は不要
- `clippy.toml` の追加で閾値が変わるため、全ての違反を解消してからマージする必要がある

---

## 5. 追加調査が必要な項目

1. **正確な違反リスト**: `clippy.toml` を作成して `cargo clippy` を実行し、実際の違反関数と違反数を確定する必要がある（本探索では Bash/Write が制限されたため、コード分析による推定のみ）
2. **type-complexity-threshold=200 の影響**: `src/host.rs` の `HostClient` trait に `allow(clippy::type_complexity)` が付与されているが、閾値200で実際に違反するか確認が必要
3. **テストファイルの allow 付与方法**: 78個のテストファイルに個別に `#[allow()]` を付与するか、あるいは `clippy.toml` でテストコードをスコープ外にする別のアプローチがあるか調査
4. **TUI view 関数のリファクタリング戦略**: `Frame` + 複数の状態パラメータを受け取るパターンは ratatui の一般的な使い方。構造体にまとめる場合のライフタイム管理を確認する必要がある
5. **閾値の妥当性検証**: cognitive-complexity-threshold=10 が現実的に運用可能か、違反数が多すぎる場合は 15 等の中間値も検討
