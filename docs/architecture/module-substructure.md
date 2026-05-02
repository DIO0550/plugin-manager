# Module Substructure Guideline

`src/{feature}/{a.rs, b.rs}` の平坦構造から、同一ドメイン概念ごとにサブフォルダを 1 階層挟んだ `src/{feature}/{subgroup}/{a.rs, b.rs}` 構造へ揃えるためのガイドライン。CLAUDE.md の「モジュール構成方針（Feature ベース）」「Rust 2018+ スタイル（mod.rs 不使用）」と整合する補足ルールを定義する。

## 1. サブ階層化の起動基準

以下を **AND 条件** で満たすときにサブフォルダを切る:

- 親モジュール直下の本体ファイル数（`*_test.rs` 除外）が **4 以上**
- 同一ドメイン軸（環境別 / レイヤー別 / 概念別）で **2 つ以上のグループ**に分けられる

例:

- `parser/` 8 ファイル × 環境軸（claude_code / codex / copilot）→ サブ階層化対象
- `application/` 3 ファイル → 閾値以下、現状維持

両条件を満たさない場合は平坦構造のまま維持する。サブグループを 1 つしか作れないリファクタは、構造の凝集度向上に寄与しないためスキップする。

## 2. `{parent}.rs` 集約パターン

`mod.rs` は使用しない（CLAUDE.md 規約）。`{parent}.rs` に `mod` 宣言と `pub use` を集約する。

```rust
// src/parser.rs
mod claude_code;
mod codex;
mod copilot;
pub mod convert;
mod frontmatter;

pub use claude_code::{ClaudeCodeAgent, ClaudeCodeCommand};
pub use convert::TargetType;
```

サブモジュール側 (`src/parser/claude_code.rs`) では、親で外部公開する型は **`pub use`**（`pub(crate) use` ではない）にする:

```rust
// src/parser/claude_code.rs
mod agent;
mod command;

pub use agent::ClaudeCodeAgent;
pub use command::ClaudeCodeCommand;
```

> **理由**: `parser.rs` 内で `mod claude_code;` を private 宣言にしておけば、外部からは `crate::parser::claude_code::...` という長いパスは見えない。一方、`parser.rs` の `pub use claude_code::ClaudeCodeAgent;` を成立させるには、サブ親 `claude_code.rs` の側で `pub use` であることが必要。`pub(crate) use` にしてしまうと「親モジュールでの `pub use ...::ClaudeCodeAgent` 経由の外部公開」が私的可視性により遮断される。

外部に公開しない（クレート内部参照のみ）の場合は `pub(crate) use`:

```rust
// src/parser/codex.rs (CodexPrompt は外部公開しない)
mod agent;
mod prompt;

pub(crate) use agent::CodexAgent;
pub(crate) use prompt::CodexPrompt;
```

## 3. アクセス修飾子の使い分け

| 公開範囲 | 修飾子 | 用途 |
|---|---|---|
| 外部 (`crate::xxx::Yyy`) | `pub use`（サブ親）+ `pub use`（親 `{parent}.rs`） | 親が `pub use sub::Type;` で再エクスポートする型 |
| クレート内部のみ | `pub(crate) use` | サブグループ間で参照されるが、外部 (`crate::xxx::...`) には出さない型 |
| サブグループ内 | （非公開） | サブグループに閉じる実装詳細 |

ポイント: `mod sub;` 自体は親 `{parent}.rs` で **private** のまま。`pub mod sub;` にしない。これにより外部から `crate::feature::sub::...` のロングパスが露出しない。

## 4. テストファイル `_test.rs` の配置追従ルール

- `foo.rs` のテストは同階層 `foo_test.rs` に置く（CLAUDE.md 規約）
- 本体ファイルをサブフォルダに移動する場合、`_test.rs` も同時に移動する
- ルート残置のファイル（`convert.rs` 等）の `_test.rs` もルート残置とし、親 `{parent}.rs` 側の `#[cfg(test)] mod foo_test;` 宣言を維持する
- 移動するテストの `mod foo_test;` 宣言は親 `{parent}.rs` から削除し、サブ親 `{subgroup}.rs` に `#[cfg(test)] mod foo_test;` として移し替える

## 5. 内部 `use` 更新ルール（移動に伴う `super::` の更新）

サブフォルダに移動した本体・テストファイルは、`super::xxx` の参照先が 1 階層深くなるため、以下の規則で更新する:

- **同一サブグループ内**（`claude_code/command.rs` から `claude_code/agent.rs` を参照）: `super::agent::ClaudeCodeAgent`（同階層）
- **別サブグループ**（`claude_code/command.rs` から `codex/prompt.rs::CodexPrompt` を参照）: `super::super::codex::CodexPrompt`（サブ親 `parser/codex.rs` 経由）
- **ルート残置モジュール**（`claude_code/command.rs` から `parser/convert.rs::TargetType` を参照）: `super::super::convert::TargetType`
- **テストファイル**（`claude_code/command_test.rs` から本体 `command.rs::ClaudeCodeCommand` を参照）: `super::command::ClaudeCodeCommand`（同階層）。別サブグループ／ルート残置は本体と同じ規則 (`super::super::xxx`)

## 6. `#[path]` 形式と `mod foo_test;` 形式の使い分け

リポジトリ内には 2 形式が混在しており、本ガイドラインで一方に統一する意図はない。**現状の主流は形式 A**（leaf 単独ファイルでは `host.rs` `config.rs` `env.rs` `install.rs` `path_ext.rs` 等の多数で採用）であり、形式 B はモジュール階層を `{parent}.rs` に集約する場面（`hooks.rs` `parser.rs` `tui/manager/core.rs` `marketplace/config.rs` `plugin/plugin_content.rs` 等）で採用されている。サブ階層化時はモジュールの集約構造に合わせていずれかを選ぶ。

形式 A — `#[cfg(test)] #[path = "foo_test.rs"] mod tests;`

- 本体 `foo.rs` の末尾に置き、テストモジュール名を `tests` に統一する
- 移動時は `#[path = "..."]` の相対パスが**同一フォルダなら不変**
- **leaf 単独ファイル（同階層に他の `mod` 宣言を集める親が存在しない場合）の既定**

形式 B — `#[cfg(test)] mod foo_test;`

- 親 `{parent}.rs` 側に `mod foo_test;` を並べる
- 移動時は親側の `mod foo_test;` 宣言行を移動先 `{subgroup}.rs` へ追従させる
- **サブグループ親 `{parent}.rs` が `mod` 宣言と `pub use` を集約しているケース**で、テスト宣言も同じ場所に並べる用途に向く

**サブ階層化時の選択**:

- 移動元のテスト宣言形式（A / B）を**そのまま保持する**ことを最優先とする。形式変更は別 PR で扱う
- `parser/` は移行前から形式 B（`parser.rs` 側で `#[cfg(test)] mod *_test;` を集約）であり、本リファクタもサブ親 `parser/{claude_code, codex, copilot}.rs` で同形式を維持している
- 既存が形式 A の親フォルダ（`application/`, `component/`, `target/` 等）をサブ階層化する場合は、形式 A のまま `#[path]` の相対パスのみ更新する（`mod tests;` を `{parent}.rs` に移し替える必要はない）
