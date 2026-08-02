# Issue #417 実装計画 — TargetKind に OpenCode バリアントを追加する

> 状態: 計画のみ（実装未着手）
> Issue: [#417](https://github.com/DIO0550/plugin-manager/issues/417)
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416) OpenCode ターゲット追加（Phase 1）
> 上位計画: [`opencode-target-plan.md`](./opencode-target-plan.md)

## 目的

`TargetKind` enum に `OpenCode` バリアントを追加し、識別子 `"opencode"` を CLI（`ValueEnum`）・serde・配置パス定数の各ロスターに登録する。ターゲット本体（`OpenCodeTarget`）の実装は Phase 2（#418）のスコープであり、本 Issue には含めない。

## 変更対象と内容

### 1. `src/target.rs` — `TargetKind` 本体

- `TargetKind` enum に `OpenCode` を追加する。
  - `#[serde(rename_all = "lowercase")]` により serde 表現は自動で `"opencode"` になる。`GeminiCli` のような個別 rename 属性は不要。
  - `ValueEnum` も既定の kebab-case 変換で `"opencode"` になるため `#[value(name = ...)]` は不要（`OpenCode` は 1 語扱いか要確認。`open-code` になる場合は `#[value(name = "opencode")]` を明示する。テストで検証する）。
  - バリアントの挿入位置は `Cursor` の後・`GeminiCli` の前（アルファベット順維持。`Ord` 派生のため `TargetsConfig::normalize()` のソート順に影響するが、TOML 上は文字列名で永続化されるため互換性問題はない）。
- `as_str()` に `TargetKind::OpenCode => "opencode"` を追加。
- `command_format()` に `TargetKind::OpenCode => CommandFormat::ClaudeCode` を追加（内容変換なし・拡張子リネームのみの方針。上位計画 Phase 3 参照）。
- `agent_format()` に `TargetKind::OpenCode => AgentFormat::ClaudeCode` を追加。
- `parse_target()` / `all_targets()` への登録は **行わない**。Issue のチェック項目に「`OpenCodeTarget` 実装後」と明記されており、trait 実装体が存在しない Phase 1 では登録できない。#418 で追加する。

### 2. `src/placement_names.rs` — ルート定数

既存の `GEMINI_SUBDIR` / `CURSOR_SUBDIR` に倣い、以下を追加する。

```rust
/// OpenCode の Personal 配置ルート（`$XDG_CONFIG_HOME` または `~/.config` 配下）
pub const OPENCODE_PERSONAL_PARENT: &str = ".config";
pub const OPENCODE_PERSONAL_CHILD: &str = "opencode";
/// OpenCode の Project 配置ルート
pub const OPENCODE_PROJECT_SUBDIR: &str = ".opencode";
```

- Personal は Antigravity（`ANTIGRAVITY_PERSONAL_PARENT` / `_CHILD`）と同じ 2 段構成パターンを採る。
- XDG_CONFIG_HOME の解決ロジック自体は Phase 2（#418、`OpenCodeTarget` 実装）のスコープ。Phase 1 の `personal_base(home)` は `home/.config/opencode` を返す素朴な実装とし、XDG 上書きは #418 で扱う旨をコメントに残す。

### 3. `src/target/core/layout.rs` — レイアウト API

`TargetKind` の match が網羅的（ワイルドカードなし）のため、以下 4 箇所に OpenCode アームを追加する。

- `instruction_filename()`: `TargetKind::OpenCode => Some(INSTRUCTION_AGENTS)`（Personal/Project とも `AGENTS.md`。上位計画 Phase 4 参照）。`ALL_INSTRUCTION_FILENAMES` に `AGENTS.md` は既存のため整合性アサートは変更不要。
- `personal_base()`: `home.join(OPENCODE_PERSONAL_PARENT).join(OPENCODE_PERSONAL_CHILD)`。
- `project_base()`: `project_root.join(OPENCODE_PROJECT_SUBDIR)`。
- `cleanup_kind_subdirs()`: `(TargetKind::OpenCode, _) => &[Skill, Agent, Command]`（Cursor と同型。Hooks 非対応）。
- `assert_instruction_filenames_consistent()` 内の全ターゲット配列に `TargetKind::OpenCode` を追加。

### 4. その他の網羅 match への影響（コンパイルエラー起点で対応）

`TargetKind` を網羅 match しているコードにアームを追加する必要がある。現時点で判明している該当ファイル:

- `src/commands/deploy/import.rs`
- `src/component/convert.rs`
- `src/install/format.rs`

いずれも `command_format()` / `agent_format()` 経由か表示系と推測されるため、ClaudeCode 互換の既存アーム（Cursor / GeminiCli）に相乗りする形で追加する。`cargo check` のエラーを網羅確認の手段とする。

### 5. `TargetsConfig::default()`（`src/target/core/registry.rs`）

**変更しない**。opt-in 方針（`plm target add opencode` で有効化）。GeminiCli と同様デフォルトリスト外に置き、既存ユーザーへの破壊的変更を避ける。

## 非スコープ（後続 Issue）

- `OpenCodeTarget` struct（`src/target/env/opencode.rs`）と `parse_target` / `all_targets` 登録 → #418
- Agents / Commands 配置ロジック → #419
- Instructions 配置ロジック → #420
- ドキュメント状態表記の更新（`targets.md` ほか）→ #421

## テスト計画（TDD: Red → Green）

テストは `src/target_test.rs` / `src/target/core/layout_test.rs` に追加する。

| # | テスト | 期待値 |
|---|--------|--------|
| 1 | `TargetKind::OpenCode.as_str()` | `"opencode"` |
| 2 | serde ラウンドトリップ（`"opencode"` ⇔ `OpenCode`） | 一致 |
| 3 | `ValueEnum`（`clap` の value parse）で `"opencode"` を受理 | `OpenCode` |
| 4 | `command_format()` / `agent_format()` | ともに `ClaudeCode` |
| 5 | `personal_base(home)` | `home/.config/opencode` |
| 6 | `project_base(root)` | `root/.opencode` |
| 7 | `instruction_filename()` | `Some("AGENTS.md")` |
| 8 | `parse_target("opencode")` | Phase 1 では `TargetNotFound` エラー（#418 で反転させる） |
| 9 | `TargetsConfig::default()` | `OpenCode` を含まない |

手順: 上記テストを先に書いて失敗（Red）を確認 → 変更 1〜4 を実装（Green）→ `cargo fmt` / `cargo clippy` / `cargo test` で全体確認。

## リスク・注意点

1. **`ValueEnum` の名前変換**: `OpenCode` が `open-code` にレンダリングされる可能性がある。テスト 3 で検証し、必要なら `#[value(name = "opencode")]` を付与する。serde 側も同様に `rename_all = "lowercase"` の出力（`opencode`）をテスト 2 で確認する。
2. **`Ord` 派生への影響**: バリアント追加位置により `TargetsConfig::normalize()` のソート結果（TUI やリスト表示の並び）が変わる。文字列アルファベット順と enum 宣言順が一致するよう `Cursor` と `GeminiCli` の間に挿入する（"opencode" < "gemini" ではないため、厳密な文字列順にするなら `GeminiCli`(= "gemini") の後が正しい。宣言順は enum 順 = 表示順であることを確認のうえ、"cursor" < "gemini" < "opencode" となる位置、すなわち `GeminiCli` の後に置く）。
3. **`parse_target` 未登録期間の UX**: Phase 1 マージ後、CLI は `--target opencode` を受理するが `parse_target` が `TargetNotFound` を返す過渡状態になる。#418 が続けてマージされる前提の既知の制限としてテスト 8 で明示する。

## 検証方法

- `cargo fmt` → `cargo check` → `cargo clippy` → `cargo test` が全てパスすること。
- `cargo run -- target add opencode` で追加でき、`targets.toml` に `"opencode"` が永続化されること（手動確認、`TargetRegistry` は `TargetKind` ベースのため Phase 1 で動作可能な見込み。`target add` が `parse_target` に依存する場合は #418 に繰り延べ、その旨を PR に記載する）。
