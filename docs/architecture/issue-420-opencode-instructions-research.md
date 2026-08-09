# Issue #420 調査: OpenCode Instructions（AGENTS.md）配置

> 調査日: 2026-08-09  
> 対象 HEAD: `ee951df`（#418 OpenCodeTarget Skills マージ後）  
> Issue: [#420](https://github.com/DIO0550/plugin-manager/issues/420)（OPEN）  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416)

## 要約

OpenCode の **Phase 1（#417）と Phase 2（#418 Skills）はコード上完了**。Phase 3（Agents/Commands #419）と **Phase 4（Instructions #420）は未実装**。`OpenCodeTarget` は現状 `ComponentKind::Skill` のみサポートし、Instruction は明示的に非サポート。実装時は Codex / Gemini CLI と同型の `instruction_file` + `ScopeSupport::Both` が最短ルート。Project `AGENTS.md` は Codex / Cursor と同一パスを共有する点をテストで固定する必要がある。

---

## 1. OpenCode ターゲット実装の現状

### ファイル配置

| パス | 役割 |
|------|------|
| `/workspace/src/target/env/opencode.rs` | `OpenCodeTarget` 本体（Skills のみ） |
| `/workspace/src/target/env/opencode_test.rs` | #418 単体テスト |
| `/workspace/src/target/env.rs` | `mod opencode` / re-export |
| `/workspace/src/target.rs` | `TargetKind::OpenCode`、`parse_target` / `all_targets` |
| `/workspace/src/placement_names.rs` | OpenCode ルート定数 |
| `/workspace/src/target/core/layout.rs` | `personal_base` / `project_base` / cleanup |
| `/workspace/src/install.rs` | `record_opencode_skill_ownership` |

### `OpenCodeTarget` の要点（`opencode.rs`）

- モジュールコメント: Skills = Phase 2 / #418。Agents / Commands / Instructions は #419 / #420（L1–4）
- `SUPPORTED` = `[Skill]` のみ（L20）
- `CAPABILITIES` = `[(Skill, Both)]` のみ（L22–23）
- Personal ルート: `$XDG_CONFIG_HOME/opencode`、未設定時 `~/.config/opencode`（L33–36, L67–74）
- Project ルート: `project_root/.opencode`（L41, `OPENCODE_PROJECT_SUBDIR`）
- Skill 配置: `original_name` 必須、空なら `None`（L108–113）— Cursor #377 と同型
- Instruction アームなし。`placement_location` / `list_placed` の `_` で空（L114–115, L160–161）
- Hooks 対象外（コメント・SUPPORTED に含めない）

テスト `test_opencode_supported_components_skills_only`（`opencode_test.rs` L58–66）が Agent / Command / Instruction / Hook 非サポートを固定済み。

### Phase 1 で既に揃っている基盤

- `TargetKind::OpenCode`（serde/CLI `"opencode"`）— `target.rs` L163–164
- `command_format` / `agent_format` = ClaudeCode（L191–208）— Phase 3 向けに先行定義
- `TargetKind::instruction_filename()` は OpenCode で既に `Some("AGENTS.md")`（`layout.rs` L17–18）
- `personal_base` / `project_base` / cleanup_specs（skills/agents/commands）は #418 で実装済み

---

## 2. Codex / Cursor の Instructions 扱い

### 比較表

| 項目 | Codex | Cursor | OpenCode（仕様 / #420） |
|------|-------|--------|-------------------------|
| 型 | `ComponentKind::Instruction` | 同左 | 同左（未実装） |
| ファイル名 | `AGENTS.md` (`INSTRUCTION_AGENTS`) | 同左 | 同左 |
| `ScopeSupport` | `Both` | `ProjectOnly` | **`Both`（Cursor との差分）** |
| Project パス | `{project_root}/AGENTS.md` | `{project_root}/AGENTS.md` | `{project_root}/AGENTS.md`（共有） |
| Personal パス | `{home}/.codex/AGENTS.md` | 非対応（`None`） | `~/.config/opencode/AGENTS.md`（XDG） |
| ヘルパ | `instruction_file(...)` | 直接 `project_root.join(...)` | Codex/Gemini 型が適合 |

### Codex（`src/target/env/codex.rs`）

```39:44:src/target/env/codex.rs
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Agent, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::Both),
    (ComponentKind::Hook, ScopeSupport::Both),
];
```

- 配置: `instruction_file(scope, project_root, &base, LAYOUT.instruction_file)`（L149–151）
- `list_placed`: `list_instruction_at(&Self::instruction_path(...), ...)`（L218–222）
- Project テスト: `/project/AGENTS.md`（`codex_test.rs` L95–111）。Personal Instruction の明示テストは薄い

### Cursor（`src/target/env/cursor.rs`）

```36:43:src/target/env/cursor.rs
/// Instructions は Project のみ。それ以外は両スコープ。
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Agent, ScopeSupport::Both),
    (ComponentKind::Command, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::ProjectOnly),
    (ComponentKind::Hook, ScopeSupport::Both),
];
```

- Project: `PlacementLocation::file(project_root.join(LAYOUT.instruction_file))`（L183–185）
- Personal: `can_place_scope` で拒否 → `placement_location` / `list_placed` は `None` / 空
- テスト充実: scope（L97–106）、placement（L273–303）、list_placed（L515–552）

### 参考: Gemini CLI（Both + `instruction_file`）

OpenCode Phase 4 の実装雛形として最も近い。`gemini_cli.rs` L26–28 / L89–90 / L106–110。Personal/Project 両方の placement・list テストあり（`gemini_cli_test.rs`）。

---

## 3. Phase / Epic ドキュメント

| ドキュメント | 内容 | コードとのずれ |
|--------------|------|----------------|
| `docs/architecture/opencode-target-plan.md` | Phase 1–5 計画。冒頭「仕様策定済み / **実装未着手**」 | **古い**。#417/#418 は CLOSED・実装済み |
| `docs/concepts/targets.md` L426–428 | OpenCode「仕様策定中（未実装）」 | **古い**。Skills は実装済み |
| `docs/roadmap.md` Phase 17 | #417–#421 すべて unchecked | **古い**。#417/#418 は完了 |
| `docs/architecture/file-formats.md` L347–378 | Instructions パス仕様は正しい（Personal+Project） | 仕様としては有効。実装状況表記なし |
| `docs/reference/config.md` | `[targets.opencode]` パス例あり | 「将来仕様」注記あり |

### Issue 状態（GitHub）

| Issue | Phase | State | 内容 |
|-------|-------|-------|------|
| #416 | Epic | OPEN | OpenCode ターゲット追加 |
| #417 | 1 TargetKind | **CLOSED** | バリアント追加 |
| #418 | 2 Skills | **CLOSED** | `OpenCodeTarget` Skills |
| #419 | 3 Agents/Commands | OPEN | flatten `.md` 配置 |
| #420 | 4 Instructions | OPEN | Personal+Project AGENTS.md |
| #421 | 5 Docs | OPEN | 状態表記の整合（blocked_by 2–4） |

#420 作業内容（Issue body）:

1. Instruction の Personal / Project 配置を実装
2. Codex / Cursor との Project パス共有時の挙動をテストで固定
3. 単体テスト追加

blocked_by: Phase 2（#418）→ **ブロッカー解消済み**（#419 とは並列可）

---

## 4. Instructions 配置関連テスト（既存）

### ターゲット別

| ファイル | カバレッジ |
|----------|------------|
| `src/target/env/cursor_test.rs` | Instruction ProjectOnly / Personal None / list_placed |
| `src/target/env/codex_test.rs` | Project → `/project/AGENTS.md`（Personal 明示は薄い） |
| `src/target/env/gemini_cli_test.rs` | Personal + Project placement / list（Both の模範） |
| `src/target/env/opencode_test.rs` | Instruction **非サポート**を assert（#420 で更新必要） |

### 共通ヘルパ

| ファイル | 内容 |
|----------|------|
| `src/target/placed/placement_helpers_test.rs` | Project→`/proj/AGENTS.md`、Personal→`/base/AGENTS.md` |
| `src/target/placed/list_helpers_test.rs` | `list_instruction_at` 存在/欠落 |
| `src/target/core/layout_test.rs` | OpenCode `instruction_filename` = AGENTS.md、bases、cleanup |
| `src/scan/placement_test.rs` | `is_instruction_file("AGENTS.md")` |
| `src/plugin/content/plugin_content_test.rs` | プラグイン内 AGENTS.md 解決 |

OpenCode 向け Instruction テストは **まだ無い**（非サポート固定のみ）。

---

## 5. Phase 2 / 3 / 4 ステータス（コード + Issue）

| Phase | Issue | 計画 | 実態 |
|-------|-------|------|------|
| 1 TargetKind | #417 | 完了想定 | ✅ 実装・CLOSED |
| 2 Skills | #418 | 完了想定 | ✅ 実装・CLOSED（`ee951df`） |
| 3 Agents/Commands | #419 | 未着手 | ❌ OPEN。SUPPORTED に Agent/Command なし |
| 4 Instructions | #420 | 未着手 | ❌ OPEN。Instruction 非サポート |
| 5 Docs | #421 | 最終 | ❌ OPEN。計画/roadmap/targets.md が Phase1–2 完了を未反映 |

ドキュメント上は「未着手 / 仕様策定中」のまま残っている箇所が多い。実装の真実は **Phase 2 完了、Phase 3–4 未着手**。

---

## 6. 主要型・配置パス

### `ScopeSupport`（`src/target/placed/scope_support.rs`）

```7:12:src/target/placed/scope_support.rs
pub(crate) enum ScopeSupport {
    None,
    PersonalOnly,
    ProjectOnly,
    Both,
}
```

`allows_scope(table, kind, scope)` で CAPABILITIES 表を参照。

### `ComponentKind::Instruction`（複数形ではない）

ユーザー表現の `ComponentKind::Instructions` はコード上 **`ComponentKind::Instruction`**（単数）。`kind.rs` L19–20。`default_subdir()` は Instruction のみ `None`（固定ファイル配置、L83–86）。

### 配置定数（`placement_names.rs`）

- `INSTRUCTION_AGENTS = "AGENTS.md"`（L11）
- `OPENCODE_PERSONAL_PARENT = ".config"`（L40）
- `OPENCODE_PERSONAL_CHILD = "opencode"`（L42）
- `OPENCODE_PROJECT_SUBDIR = ".opencode"`（L44）

### `instruction_file` ヘルパ（`placement_helpers.rs` L30–41）

- Project → `project_root/<filename>`
- Personal → `base/<filename>`

OpenCode では Personal の `base` を `OpenCodeTarget::personal_root()`（または `base_dir(Personal, ...)`）にすれば仕様どおり。

### 期待パス（#420 完了後）

| Scope | パス |
|-------|------|
| Personal | `$XDG_CONFIG_HOME/opencode/AGENTS.md`（未設定時 `~/.config/opencode/AGENTS.md`） |
| Project | `{project_root}/AGENTS.md`（Codex / Cursor と同一） |

---

## 7. #420 で残っている作業（Instructions）

実装対象は主に `src/target/env/opencode.rs` + `opencode_test.rs`。Rust 外の仕様ドキュメント更新は #421。

### 実装チェックリスト

1. **`SUPPORTED` に `ComponentKind::Instruction` を追加**
2. **`CAPABILITIES` に `(Instruction, ScopeSupport::Both)` を追加**
3. **`placement_location`**: Codex/Gemini 同様 `instruction_file(scope, project_root, &base, INSTRUCTION_AGENTS)`
   - Personal の `base` は `personal_root()`（XDG 尊重済みの既存 `base_dir`）
   - Project は `project_root/AGENTS.md`（`.opencode/` 配下ではない）
4. **`list_placed`**: `list_instruction_at` で両スコープ対応
5. **テスト更新・追加**（`opencode_test.rs`）:
   - `supports` / `supports_scope` Personal+Project
   - placement Personal（default HOME / XDG）→ `.../opencode/AGENTS.md`
   - placement Project → `{root}/AGENTS.md`
   - list_placed 存在/欠落（両スコープ）
   - **Codex / Cursor との Project パス同一性**（同一 `project_root` で 3 ターゲットが同じ `AGENTS.md` を指す）
   - 既存の「Instruction 非サポート」テストを削除または反転
6. （任意）overwrite / ownership — Cursor/Codex の Instruction は専用 ownership を置いていないため、v1 では不要の可能性が高い

### 設計上の注意

- **Cursor との差分**: Personal を必ずサポート（`ProjectOnly` にしない）
- **Project 共有**: 複数ターゲット有効時に同一ファイルを上書きしうる（仕様どおり。テストで固定）
- **#419 との独立性**: Instructions は Agents/Commands なしでも追加可能（Issue の blocked_by は #418 のみ）
- **ドキュメントずれ**: 実装後も `opencode-target-plan.md` / `roadmap.md` / `targets.md` の「未実装」表記は #421 で一括更新

### 推奨実装パターン

`gemini_cli.rs`（Both + `instruction_file` + `list_instruction_at`）をコピーし、ファイル名を `INSTRUCTION_AGENTS`、base 解決を既存 OpenCode `base_dir` / `personal_root` に差し替えるのが最小差分。
