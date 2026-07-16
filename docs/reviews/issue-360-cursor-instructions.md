# Issue #360 レビュー: Cursor の Instructions（AGENTS.md）配置対応

- **Issue**: [#360 Cursor の Instructions（AGENTS.md）配置に対応する](https://github.com/DIO0550/plugin-manager/issues/360)
- **Epic**: [#356 Cursor ターゲット追加](https://github.com/DIO0550/plugin-manager/issues/356) Phase 4
- **レビュー日**: 2026-07-16
- **対象ブランチ**: `main`（`c38d9d3` — PR #367 マージ後）

## 概要

`CursorTarget` に Project スコープの Instructions（`AGENTS.md`）配置を追加する Issue。現状は Skills / Agents / Commands のみ実装済みで、Instructions は `placement_location()` / `list_placed()` が `None` / 空を返す。

## 現状分析

### 実装済み（Phase 3: #358 / #359）

| 項目 | 状態 |
|------|------|
| `supported_components` | Skill, Agent, Command の 3 種 |
| `can_place()` | 上記 3 種のみ（スコープ非依存） |
| `placement_location()` | `.cursor/skills|agents|commands/` 配下 |
| `list_placed()` | 同上ディレクトリのスキャン |
| テスト | Instruction が `None` になることを明示的に検証 |

### 参照実装

Codex の Project Instructions 配置は既に確立されている:

```179:183:src/target/env/codex.rs
            ComponentKind::Instruction => match scope {
                // Project scope: AGENTS.md is at project root, not in .codex
                Scope::Project => PlacementLocation::file(project_root.join("AGENTS.md")),
                Scope::Personal => PlacementLocation::file(base.join("AGENTS.md")),
            },
```

Copilot は Personal 制約と同型で、Project のみ Instructions をサポート:

```43:48:src/target/env/copilot.rs
    fn can_place(kind: ComponentKind, scope: Scope) -> bool {
        matches!(
            (kind, scope),
            (ComponentKind::Agent, _) | (ComponentKind::Hook, _) | (_, Scope::Project)
        )
    }
```

Cursor も仕様上 **Project スコープのみ** のため、Copilot と同じ `(kind, scope)` パターンが適用できる。

## 設計判断

### 1. `AGENTS.md` の Codex との共有

#### 結論: **Phase 4 では Codex と同一パス・同一セマンティクスを採用する（追加の所有権管理は行わない）**

| 観点 | Codex（既存） | Cursor（推奨） |
|------|--------------|----------------|
| Project 配置先 | `{project_root}/AGENTS.md` | 同一 |
| Personal 配置先 | `~/.codex/AGENTS.md` | **非対応**（`None`） |
| `managedFiles` 追跡 | なし（Hook のみ対象） | なし |
| install 上書きガード | なし | なし（Codex に合わせる） |
| uninstall / disable 削除 | `RemoveFile` で直接削除 | 同一 |

#### 根拠

1. **仕様の意図**: `docs/concepts/targets.md` は「`AGENTS.md` は Codex ターゲットと同一ファイルを共有」と明記しており、Cursor 単独利用時も Codex 併用時も同じファイルが両環境に読み込まれるのが正しい挙動。
2. **既存挙動との整合**: Codex Project Instructions には `hook_overwrite_error` 相当のガードがなく、`managedFiles` も記録しない。Cursor だけ特別扱いすると不整合が生じる。
3. **install 時の複数ターゲット**: `place_plugin` はターゲットごとに `CopyFile` を実行する。同一パスへの二重コピーは内容が同じなら実質 no-op で、最後のターゲットが書き込むだけ（現状 Codex + 将来 Cursor でも同様）。
4. **disable / uninstall 時のリスク（既知の制限）**: `disable --target codex` 実行時、Codex の `placement_location` 経由で `AGENTS.md` が削除される。Cursor も同じパスを指すため、Cursor 側でも Instructions を利用している場合にファイルが消える。これは **Codex 単体でも外部手動編集との間で同様の問題があり**、Phase 4 のスコープ外とする。

#### フォローアップ候補（別 Issue 推奨）

共有ファイルの参照カウントまたは `managedFiles` による「他ターゲットが同一絶対パスを参照中なら削除しない」ロジックは、以下に共通する横断的改善となる:

- Codex Project `AGENTS.md` ↔ Cursor Project `AGENTS.md`
- 将来的に Gemini CLI の `contextFileName = "AGENTS.md"` 設定との重複

Phase 4 では Issue 本文の「設計判断」チェックボックスを **Codex 既存挙動に追随** でクローズ可能。

### 2. `INSTRUCTION_FILE_NAMES` と `instruction_filename()`（#339）

#### 結論: **`scan/placement.rs` の変更は不要。Cursor 実装では `"AGENTS.md"` リテラルを使用する**

```17:17:src/scan/placement.rs
const INSTRUCTION_FILE_NAMES: &[&str] = &["AGENTS.md", "copilot-instructions.md", "GEMINI.md"];
```

`AGENTS.md` は既に含まれており、`list_placed_components()` の Instruction 除外ロジックもそのまま機能する。

`placed_common::list_instruction()` は第 4 引数でファイル名を受け取る設計:

```17:22:src/target/placed/placed_common.rs
pub(crate) fn list_instruction(
    target: &dyn Target,
    scope: Scope,
    project_root: &Path,
    instruction_filename: &str,
) -> Vec<String> {
```

Codex / Gemini CLI / Copilot もいずれもハードコード文字列を渡しており、#339 で `Target::instruction_filename()` が導入された際に Cursor も `"AGENTS.md"` を返せばよい。**Phase 4 では導入を待たずリテラルで十分**。

`sync/endpoint.rs` の `parse_component_name` も `is_instruction_file("AGENTS.md")` で既に対応済みのため、sync 経路への追加変更は不要。

## 推奨実装

### `src/target/env/cursor.rs`

1. **`can_place(kind, scope)` にスコープ引数を追加**（Copilot パターン）:

   ```rust
   fn can_place(kind: ComponentKind, scope: Scope) -> bool {
       matches!(
           (kind, scope),
           (ComponentKind::Skill, _)
               | (ComponentKind::Agent, _)
               | (ComponentKind::Command, _)
               | (ComponentKind::Instruction, Scope::Project)
       )
   }
   ```

2. **`supported_components` に `ComponentKind::Instruction` を追加**

3. **`placement_location()` に Instruction アームを追加**:

   ```rust
   ComponentKind::Instruction => {
       PlacementLocation::file(project_root.join("AGENTS.md"))
   }
   ```

   Personal スコープは `can_place` で弾かれるため match 内に到達しない。

4. **`list_placed()` に Instruction 分岐を追加**:

   ```rust
   if kind == ComponentKind::Instruction {
       return Ok(placed_common::list_instruction(
           self,
           scope,
           project_root,
           "AGENTS.md",
       ));
   }
   ```

5. **モジュールコメントを Phase 4 対応済みに更新**（Hooks は #361 残）

### `src/target/env/cursor_test.rs`

| テスト | 内容 |
|--------|------|
| `test_cursor_supported_components` | 要素数 3 → 4、Instruction を含む |
| `test_cursor_supports_instruction` | `supports(Instruction)` が `true` |
| `test_cursor_supports_scope_instruction_project` | Project で `supports_scope` が `true` |
| `test_cursor_supports_scope_instruction_personal_returns_false` | Personal で `supports_scope` が `false` |
| `test_cursor_placement_location_instruction_project` | `{project_root}/AGENTS.md` を返す |
| `test_cursor_placement_location_instruction_personal_returns_none` | Personal で `None` |
| `test_cursor_list_placed_instruction_exists` | ファイル存在時 `["AGENTS.md"]` |
| `test_cursor_list_placed_instruction_not_exists` | 未存在時 `[]` |

**削除・更新**:

- `test_cursor_not_supports_instruction` → 上記 `test_cursor_supports_instruction` に置換
- `test_cursor_placement_location_instruction_returns_none` → Project / Personal に分割して置換

### 変更不要なファイル

| ファイル | 理由 |
|----------|------|
| `src/scan/placement.rs` | `AGENTS.md` 登録済み |
| `src/sync/endpoint.rs` | `is_instruction_file` 対応済み |
| `src/install.rs` | 汎用 `place_plugin` が `placement_location` を参照するだけ |
| `docs/concepts/targets.md` | #362（ドキュメント整合性）のスコープ |

## 受入基準チェックリスト

- [x] 設計判断: Codex 既存挙動に追随（共有ファイルの高度な所有権管理は別 Issue）
- [x] 設計判断: `INSTRUCTION_FILE_NAMES` 変更不要、`instruction_filename()` は #339 待ち
- [ ] `placement_location()` に Instruction アーム（Project のみ）を追加
- [ ] `list_placed()` に Instruction アームを追加
- [ ] テスト追加（Personal スコープが `None` になることを含む）
- [ ] `cargo test` / `cargo clippy` パス

## リスクと注意点

1. **ネスト `AGENTS.md` 非対応**: Cursor 公式はサブディレクトリの `AGENTS.md` も読み込むが、PLM は他ターゲット（Codex / Copilot）と同様に **プロジェクトルートのみ** を配置先とする。サブディレクトリ配置はスコープ外。
2. **コンポーネント名の平坦化**: Instruction は `flatten_name` の対象外。`list_placed` は `"AGENTS.md"` 固定文字列を返す（Codex / Gemini CLI と同型）。
3. **roadmap.md の Phase 16 チェックリスト**: #359 / #367 はマージ済みだが roadmap が未更新。#362 で一括更新予定。

## 関連 Issue / PR

| 参照 | 関係 |
|------|------|
| #356 | Epic |
| #358 / #366 | Skills 配置（完了） |
| #359 / #367 | Agents / Commands 配置（完了） |
| #361 | Hooks（次フェーズ） |
| #362 | ドキュメント整合性 |
| #339 | `instruction_filename()` 提案（将来） |
