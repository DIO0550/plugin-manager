# 配置リテラル集約 — 実装計画

**関連 Issue**: [#339](https://github.com/DIO0550/plugin-manager/issues/339)  
**方針**: 表示用 `plural()` と配置用パス断片を分離し、#338 の LAYOUT / `placed/` ヘルパを定数の消費側に揃える

## ユーザーレビューが必要な点

> **NOTE**
> - 振る舞い不変のリファクタリング。CLI 仕様変更なし。
> - Phase A〜F の段階移行。ビッグバンなし。
> - Copilot の配置 `"prompts"` と表示 `"commands"` の差は仕様として維持する。
> - Issue 原文の `ComponentKind::placement_subdir(target?)` は、本計画では **「デフォルトは plural 相当、例外は Target/LAYOUT」** に具体化（hearing-notes 参照）。

---

## システム図

### 現状（3+ 系統並立）

```text
┌─────────────────────┐   ┌──────────────────────┐   ┌──────────────────────────┐
│ ComponentKind       │   │ scan/constants.rs    │   │ ベアリテラル群            │
│  plural()           │   │  SKILL_MANIFEST      │   │  placement_helpers       │
│  "skills"/"commands"│   │  DEFAULT_*_DIR       │   │  env LAYOUT / list_placed│
└─────────────────────┘   │  AGENT_SUFFIX ...    │   │  cleanup_specs           │
                          └──────────────────────┘   │  wire / import           │
                                                     │  INSTRUCTION_FILE_NAMES  │
                                                     └──────────────────────────┘
         ↑ 手同期が前提。コンパイラは乖離を検出できない
```

### 移行後（責務分離）

```text
┌──────────────────────────────────────────────┐
│ ComponentKind（ターゲット非依存）              │
│  plural()           … 表示・JSON キー専用      │
│  skill_manifest()   … "SKILL.md"              │
│  file_suffix()      … ".agent.md" 等          │
│  default_subdir()   … プラグイン/デフォルト配置 │
└───────────────────┬──────────────────────────┘
                    │ 消費
        ┌───────────┼────────────┬──────────────┐
        ▼           ▼            ▼              ▼
   scan/*     placed/filter  placement_helpers  wire/import
   (re-export 可)

┌──────────────────────────────────────────────┐
│ Target / LAYOUT（ターゲット依存）              │
│  instruction_file                            │
│  personal/project root                       │
│  component_subdir(kind)  … prompts 等の例外   │
└───────────────────┬──────────────────────────┘
                    │ 消費
        ┌───────────┼────────────┐
        ▼           ▼            ▼
   env impl    scan/placement  cleanup_specs
               (動的構築)
```

### Copilot Command の分離（核心例）

```text
表示・JSON:  ComponentKind::Command.plural()  → "commands"
配置パス:    Copilot LAYOUT / named_file(..., "prompts", ..., ".prompt.md")
プラグイン:  DEFAULT_COMMANDS_DIR / plural()  → "commands"（パッケージ内相対）

※ 配置だけが "prompts"。plural() を join に使ってはいけない。
```

### Instruction 除外集合の構築

```text
【移行前】
scan/placement.rs: INSTRUCTION_FILE_NAMES = ["AGENTS.md", "copilot-instructions.md", "GEMINI.md"]
env LAYOUT:        同じ文字列を各ファイルに再定義

【移行後】
all_targets() / TargetKind テーブル
    → instruction_filename() が Some のものだけ収集
    → is_instruction_file() が参照
env は LAYOUT.instruction_file を同 API 経由で公開（二重定義なし）
```

---

## Phase 計画

### Phase A: 棚卸し固定（コード変更なし）

- exploration-report を正本としてリテラル一覧を凍結
- hearing の責務境界（非依存 vs 依存）をレビュー確定
- ベースライン: `cargo test` green

### Phase B: ターゲット非依存定数の集約

- `ComponentKind` に `skill_manifest()` / `file_suffix()` /（必要なら）`default_subdir()` を追加
  - または `component/naming.rs` 等の共有 const + thin wrapper（Feature 凝集を優先）
- `plural()` に「表示・シリアライズ専用。配置パスには使わない」doc を追加
- `scan/constants.rs` を上記への re-export に変更（消費者の一斉置換は Phase C）

### Phase C: ヘルパ / scan / deployment の消費切替

- `filter_skill_dir`: `"SKILL.md"` → `ComponentKind::Skill.skill_manifest()`（または共有 const）
- `skill_dir` / `agent_file`: `"skills"` / `"agents"` / `".agent.md"` を定数消費へ
- `component/deployment.rs` の `"SKILL.md"` も同様
- `scan/components.rs` 等の既存 constants 利用箇所は re-export 経由のままか、ComponentKind 直参照へ

### Phase D: Target 依存パスの公開と cleanup / placement

- Instruction: `instruction_filename()`（`Option<&'static str>`）を Target または `TargetKind` テーブルで公開
- `scan/placement.rs`: 静的配列をやめ、公開 API から集合を構築（起動時一度 or const テーブル生成）
- Env root + kind subdir: cleanup が消費できる `pub(crate)` API
  - 推奨: `TargetKind` → `(personal_roots, project_roots, kind_subdirs)` の薄いテーブルを LAYOUT と共有ソース化
  - 各 env の `LAYOUT` フィールドをそのテーブルの単一ソースにするか、テーブルが LAYOUT を読む形にする
- Copilot `"prompts"` / Cursor `"commands"` 等の kind→subdir も同系統へ
- env の `list_placed` / `placement_location` 内ベアリテラルを順次置換（Antigravity → Gemini → Codex → Copilot → Cursor）

### Phase E: wire / import / Scope::description

- `info/wire.rs` / `list/wire.rs`: キー配列を `ComponentKind::all().map(plural)` 相当に
- `deploy/import.rs`: `"skills"` 等の match を `plural()` 逆引きヘルパへ（または match を `kind.plural()` と比較）
- （任意）`Scope::description()` 更新

### Phase F: docs / 掃除

- `scan/constants.rs` 削除可否を判断（全消費者移行済みなら削除）
- `docs/architecture/core-design.md` に定数層の短い節を追加
- `docs/roadmap.md` に本 Issue を完了行として追記
- 計画ディレクトリを `docs/old/` へ退避（実装完了 PR 時）

---

## リスクと緩和

| リスク | 緩和 |
|--------|------|
| `plural()` を配置に誤用したまま残す | Phase B で doc + Phase C/D で grep ゲート（配置 join 近傍に plural が無いこと） |
| trait 公開で FakeTarget が壊れる | まず `pub(crate)` テーブル。trait 追加は default 実装付き |
| cleanup テーブルと can_place の乖離 | kind_subdir リストを CAPABILITIES / supported から導出できないか検討。難しければ不変条件テストで LAYOUT と cleanup を突き合わせ |
| const での動的 `all_targets()` | `INSTRUCTION_FILE_NAMES` を実行時構築するか、`TargetKind::ALL` 静的テーブルから const 生成。後者推奨 |

---

## 完了条件

1. 本番コードから、配置・環境ルート・instruction・マニフェスト名の **意図しない二重ベア定義** が消えている
2. `plural()` ≠ Copilot `"prompts"` が API / doc で明示されている
3. 既存テスト全 green + 不変条件テスト追加
4. roadmap / core-design が現状を反映
