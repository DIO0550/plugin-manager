# Issue #420: OpenCode の Instructions（AGENTS.md）配置に対応する — 実装計画

> 状態: 実装計画  
> Issue: [#420](https://github.com/DIO0550/plugin-manager/issues/420)  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416) Phase 4  
> blocked_by: [#418](https://github.com/DIO0550/plugin-manager/issues/418)（main マージ済み: [#426](https://github.com/DIO0550/plugin-manager/pull/426)）  
> 参照仕様: [`docs/concepts/targets.md`](../concepts/targets.md)「OpenCode」、[`opencode-target-plan.md`](./opencode-target-plan.md)  
> 参考実装: `src/target/env/gemini_cli.rs`（`ScopeSupport::Both` + `instruction_file`）、`src/target/env/codex.rs`（`AGENTS.md` + Both）  
> 対比: `src/target/env/cursor.rs`（Instruction は `ProjectOnly` — OpenCode とは異なる）

---

## 1. 目的・スコープ（Phase 4）

### 目的

`OpenCodeTarget` に **Instructions（`AGENTS.md`）の Personal / Project 配置・列挙**を追加する。Cursor と異なり Personal もサポートし（`ScopeSupport::Both`）、Project パスは Codex / Cursor と同一の `AGENTS.md` を共有する点をテストで固定する。

### Phase 4 スコープ（本 Issue）

| 項目 | 内容 |
|------|------|
| 実装体 | 既存 `src/target/env/opencode.rs` + `opencode_test.rs` の拡張 |
| サポート追加 | **`ComponentKind::Instruction`**（Personal / Project 両方） |
| パス | Personal: `$XDG_CONFIG_HOME/opencode/AGENTS.md`（未設定時 `~/.config/opencode/AGENTS.md`） |
| パス | Project: `{project_root}/AGENTS.md`（`.opencode/` 配下ではない） |
| スコープ | `ScopeSupport::Both`（Cursor の `ProjectOnly` との差分を明示） |
| テスト | 両スコープ配置・XDG・list_placed・Codex/Cursor との Project パス同一性 |

### 制約（CLAUDE.md / AGENTS.md 準拠・implementer 必守）

- Feature ベース配置（変更は `src/target/env/` に閉じる）
- `mod.rs` 禁止（既存配線を維持）
- テストは本体と分離（`opencode_test.rs`）
- TDD: Red → Green → Refactor（失敗確認なしに実装へ進まない）
- ドメイン成果レポート型に `Result` 接尾辞を使わない（本 Issue では新規 Outcome 型は不要）
- コミットメッセージは英語
- `cargo check` / `cargo test` / `cargo fmt` は専用サブエージェント経由（直接 Bash しない）

---

## 2. 現状（#417 / #418 済み / Instruction 未実装）

### 既に main に入っているもの（再実装不要）

| 領域 | 状態 | 場所 |
|------|------|------|
| `TargetKind::OpenCode` / `parse_target` / `all_targets` | ✅ | `src/target.rs` |
| OpenCode パス定数（Personal/Project） | ✅ | `src/placement_names.rs` |
| `INSTRUCTION_AGENTS = "AGENTS.md"` | ✅ | `src/placement_names.rs` |
| `instruction_filename()` → `Some("AGENTS.md")` | ✅ | `src/target/core/layout.rs` |
| `OpenCodeTarget`（Skills + XDG + ownership） | ✅ | `src/target/env/opencode.rs` |
| `personal_root_from_env` / `base_dir` | ✅ | `opencode.rs` |
| Skill 単体テスト + EnvGuard | ✅ | `opencode_test.rs` |
| `instruction_file` / `list_instruction_at` ヘルパ | ✅ | `placement_helpers.rs` / `list_helpers.rs` |

### 未実装（本 Issue で追加）

| 領域 | 状態 |
|------|------|
| `SUPPORTED` への `Instruction` 追加 | ❌（現状 Skill のみ） |
| `CAPABILITIES` への `(Instruction, Both)` | ❌ |
| `placement_location` の Instruction アーム | ❌（`_` → `None`） |
| `instruction_path` ヘルパ | ❌ |
| `list_placed` の Instruction 分岐 | ❌ |
| Instruction 系単体テスト | ❌（非サポートを assert 中） |

### 依存関係

- blocked_by Phase 2（#418）→ **解消済み**
- Phase 3（#419 Agents/Commands）とは **独立**（並列実装可）
- ドキュメント ✅ 更新は [#421](https://github.com/DIO0550/plugin-manager/issues/421)

---

## 3. 設計方針（パス、スコープ、共有、ownership）

### 3.1 パス解決

```
Personal Instruction:
  personal_root_from_env(home) / "AGENTS.md"
  ├─ XDG_CONFIG_HOME が非空 → $XDG_CONFIG_HOME/opencode/AGENTS.md
  └─ 未設定・空 → ~/.config/opencode/AGENTS.md

Project Instruction:
  project_root / "AGENTS.md"
  （※ project_root/.opencode/AGENTS.md ではない）
```

`instruction_file(scope, project_root, &base, INSTRUCTION_AGENTS)` がこの分岐を既に実装している:

- Project → `project_root.join(filename)`（`base` は無視）
- Personal → `base.join(filename)`

したがって `base` は既存の `Self::base_dir(scope, project_root)` でよい（Personal 時のみ `personal_root()` が使われる）。

### 3.2 `supported_components` / スコープ

```rust
const SUPPORTED: &[ComponentKind] = &[
    ComponentKind::Skill,
    ComponentKind::Instruction,
];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::Both),
];
```

- Agent / Command / Hook は引き続き非サポート（#419 / 別 Epic）
- **Cursor との差分**: Instruction を `ProjectOnly` にしない。Personal 必須

### 3.3 `placement_location`

Gemini / Codex 同型:

```rust
ComponentKind::Instruction => {
    instruction_file(scope, project_root, &base, INSTRUCTION_AGENTS)
}
```

- Skill アームは既存のまま（`original_name` 必須）
- 内容変換なし（`AGENTS.md` 共通フォーマット）

### 3.4 `instruction_path` + `list_placed`

```rust
fn instruction_path(scope: Scope, project_root: &Path) -> PathBuf {
    instruction_file(
        scope,
        project_root,
        &Self::base_dir(scope, project_root),
        INSTRUCTION_AGENTS,
    )
    .as_path()
    .to_path_buf()
}
```

`list_placed` で Instruction 時:

```rust
if kind == ComponentKind::Instruction {
    return Ok(list_instruction_at(
        &Self::instruction_path(scope, project_root),
        INSTRUCTION_AGENTS,
    ));
}
```

### 3.5 overwrite / ownership

- Codex / Gemini / Cursor の Instruction は専用 ownership を置いていない
- **v1 では Instruction 向け `pre_place_check` / `post_place` 追加は不要**
- Skill の overwrite ガード・ownership 記録は変更しない

### 3.6 Project パス共有（Codex / Cursor）

同一 `project_root` に対し、次の 3 ターゲットの Project Instruction パスはすべて `{project_root}/AGENTS.md`:

| ターゲット | Project Instruction |
|------------|---------------------|
| Codex | `{root}/AGENTS.md` |
| Cursor | `{root}/AGENTS.md` |
| OpenCode（本 Issue） | `{root}/AGENTS.md` |

複数ターゲット有効時に同一ファイルを参照・上書きしうるのは仕様どおり。テストでパス同一性を固定する（Issue 作業内容 2）。

### 3.7 Target trait 変更の要点

| メソッド | 変更 |
|----------|------|
| `supported_components` | Skill + Instruction |
| `can_place_scope` | Instruction × Both（CAPABILITIES 経由） |
| `placement_location` | Instruction アーム追加 |
| `list_placed` | Instruction → `list_instruction_at` |
| `pre_place_check` / `post_place` | 変更なし（Skill のみ） |
| その他 | 変更なし |

### 3.8 import 追加

`opencode.rs` に以下を追加:

- `INSTRUCTION_AGENTS`（`placement_names`）
- `list_instruction_at`（`list_helpers`）
- `instruction_file`（`placement_helpers`）

モジュールコメントを「Skills + Instructions（#420）。Agents/Commands は #419」に更新。

---

## 4. 変更ファイル一覧

| ファイル | 変更種別 | 変更内容 |
|---------|---------|---------|
| `src/target/env/opencode.rs` | 編集 | SUPPORTED / CAPABILITIES、`instruction_path`、placement / list |
| `src/target/env/opencode_test.rs` | 編集 | 非サポート assert 更新、Instruction テスト追加 |

### 変更しないもの

| 領域 | 理由 |
|------|------|
| `placement_names.rs` / `layout.rs` | Instruction ファイル名・bases は #417/#418 済み |
| `parse_target` / `all_targets` / `env.rs` | #418 で登録済み |
| Agents / Commands 配置 | [#419](https://github.com/DIO0550/plugin-manager/issues/419) |
| Hooks / OpenCode Plugins | 別 Epic |
| `docs/concepts/targets.md` / roadmap / 計画冒頭の状態表記 | [#421](https://github.com/DIO0550/plugin-manager/issues/421) |
| Instruction ownership / overwrite 専用ガード | 他ターゲット同様 v1 不要 |
| 内容変換ロジック | `AGENTS.md` 共通・変換不要 |

---

## 5. TDD 手順（Red → Green → Refactor）とテスト観点

### Step 0: 準備

- 作業ブランチを現行 `main`（#418 マージ後）から切る
- 参考に読む: `gemini_cli.rs` / `gemini_cli_test.rs`（Both の模範）、`codex.rs`（`AGENTS.md`）、`cursor.rs`（ProjectOnly 対比）、既存 `opencode.rs` / `opencode_test.rs`

### Step 1: Red — 失敗するテストを先に書く

既存テストの更新と新規テストを追加し、**assert 失敗を確認**する。

#### 既存テストの更新

| テスト | 変更 |
|--------|------|
| `test_opencode_supported_components_skills_only` | Skill + Instruction を含む形にリネーム・更新。Agent/Command/Hook は引き続き非サポート |

#### 新規テスト（推奨）

| テスト | 観点 |
|--------|------|
| `test_opencode_supports_instruction` | `supports(Instruction) == true` |
| `test_opencode_supports_scope_instruction_both` | Personal / Project 両方 |
| `test_opencode_placement_instruction_project` | `{root}/AGENTS.md`（`.opencode/` ではない） |
| `test_opencode_placement_instruction_personal_default_home` | `~/.config/opencode/AGENTS.md`（XDG 未設定） |
| `test_opencode_placement_instruction_personal_respects_xdg_config_home` | `$XDG_CONFIG_HOME/opencode/AGENTS.md` |
| `test_opencode_list_placed_instruction_project_exists` | ファイルあり → `["AGENTS.md"]` |
| `test_opencode_list_placed_instruction_project_missing` | 欠落 → 空 |
| `test_opencode_list_placed_instruction_personal_exists` | Personal 側の存在列挙（TempDir + HOME/XDG） |
| `test_opencode_project_instruction_path_matches_codex_and_cursor` | 同一 `project_root` で 3 ターゲットのパスが一致 |

環境変数テストは既存の Mutex + `EnvGuard` パターンを再利用する。

パス同一性テストの例:

```rust
let root = Path::new("/project");
let opencode = OpenCodeTarget::new()
    .placement_location(/* Instruction, Project, root */)
    .unwrap();
let codex = CodexTarget::new()
    .placement_location(/* 同上 */)
    .unwrap();
let cursor = CursorTarget::new()
    .placement_location(/* 同上 */)
    .unwrap();
assert_eq!(opencode.as_path(), Path::new("/project/AGENTS.md"));
assert_eq!(opencode.as_path(), codex.as_path());
assert_eq!(opencode.as_path(), cursor.as_path());
```

### Step 2: Green — 最小実装

1. `SUPPORTED` / `CAPABILITIES` に Instruction + Both
2. import 追加（`INSTRUCTION_AGENTS` / `instruction_file` / `list_instruction_at`）
3. `instruction_path` 追加
4. `placement_location` に Instruction アーム
5. `list_placed` に Instruction 分岐
6. モジュールコメント更新
7. テスト全通しを確認

### Step 3: Refactor

- Gemini との表現差を読みやすく保つ（過度な共通化はしない）
- コメントは「Personal もサポート（Cursor との差分）」「Project は Codex/Cursor と共有」など設計意図に限定
- `cargo fmt` → type-check → test（CLAUDE.md の検証順）

### 手動確認（任意）

```bash
plm target add opencode
# Instruction 付きプラグインを --target opencode --scope personal / project で install
# Personal: ~/.config/opencode/AGENTS.md（または $XDG_CONFIG_HOME/opencode/AGENTS.md）
# Project: <cwd>/AGENTS.md
plm list --target opencode
```

---

## 6. 非スコープ

| 項目 | 扱い |
|------|------|
| Agents / Commands 配置 | [#419](https://github.com/DIO0550/plugin-manager/issues/419) |
| Hooks / OpenCode JS/TS Plugins | 別 Epic。`supported_components` に含めない |
| `OPENCODE_CONFIG_DIR` | v1 対象外 |
| Claude Code 互換パス（`.claude/`）への二重配置 | しない |
| `CLAUDE.md` フォールバック配置 | OpenCode 公式のフォールバック。PLM は `AGENTS.md` のみ配置 |
| Instruction 専用 overwrite / ownership | 他ターゲット同様 v1 では不要 |
| 複数ターゲット有効時の共有ファイル調停 UI | 仕様どおり共有。調停は本 Issue 外 |
| `docs/concepts/targets.md` / roadmap / `opencode-target-plan.md` 冒頭の状態表記 | [#421](https://github.com/DIO0550/plugin-manager/issues/421) |

---

## 7. 受け入れ条件チェックリスト

### 機能

- [ ] `supported_components()` が **Skill + Instruction** を含む
- [ ] Agent / Command / Hook は引き続き非サポート
- [ ] Instruction は Personal / Project 両方に配置可能（`ScopeSupport::Both`）
- [ ] Project Instruction パス: `<project_root>/AGENTS.md`（`.opencode/` 配下ではない）
- [ ] Personal Instruction パス（XDG 未設定）: `~/.config/opencode/AGENTS.md`
- [ ] Personal Instruction パス（XDG 設定時）: `$XDG_CONFIG_HOME/opencode/AGENTS.md`
- [ ] `list_placed(Instruction, ...)` が存在時に `"AGENTS.md"` を返し、欠落時は空
- [ ] 同一 `project_root` で Codex / Cursor / OpenCode の Project Instruction パスが一致する
- [ ] Skill の配置・overwrite・ownership・XDG 挙動を回帰させない

### 品質

- [ ] テストは `opencode_test.rs` に分離したまま
- [ ] TDD（Red 確認 → Green → Refactor）で進めた
- [ ] `cargo fmt` / type-check / `cargo test` がパス
- [ ] Agents / Commands / Hooks を実装していない
- [ ] 概念ドキュメントの ✅ 更新を本 PR に含めない（#421 委譲）

### ドキュメント（本計画の位置づけ）

- [x] 本ファイル（`docs/architecture/issue-420-opencode-instructions-plan.md`）を作成
- [ ] 実装完了後の概念ドキュメント更新は #421 に委譲

---

## 実装タスク分割（implementer 向け）

| ID | タスク | blockedBy | 主な変更ファイル |
|----|--------|-----------|------------------|
| 1 | Red: Instruction テスト追加・既存 skills_only 更新し失敗を確認 | - | `opencode_test.rs` |
| 2 | Green: SUPPORTED / CAPABILITIES / import / `instruction_path` | 1 | `opencode.rs` |
| 3 | Green: `placement_location` / `list_placed` の Instruction 対応 | 2 | `opencode.rs` |
| 4 | Refactor + 全テスト緑・受け入れ条件確認 | 3 | （上記一式） |

タスク 2 と 3 は同一ファイルのため直列。テストは Step 1 で先に書く。

---

## 関連

- Epic [#416](https://github.com/DIO0550/plugin-manager/issues/416)
- Phase 2 [#418](https://github.com/DIO0550/plugin-manager/issues/418) / PR [#426](https://github.com/DIO0550/plugin-manager/pull/426)
- Phase 3 [#419](https://github.com/DIO0550/plugin-manager/issues/419)（並列可・本 Issue 非依存）
- Phase 5 Docs [#421](https://github.com/DIO0550/plugin-manager/issues/421)
- 全体計画 [`opencode-target-plan.md`](./opencode-target-plan.md)
- 参考: Gemini CLI Instruction（Both）、Codex Instruction（`AGENTS.md`）、Cursor Instruction（ProjectOnly）
