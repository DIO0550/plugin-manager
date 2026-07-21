# target/env impl ベース共通骨格抽出

**関連Issue**: #338
**方針**: bottom-up impl 抽出（2026-07-21 方針転換済み。旧 top-down DSL 設計は廃止）

`target/env/` 5 実装に同型コピペされている `list_placed` 骨格 / `filter_component` Skill アーム / `placement_location` 共通パターンを、**既存 impl から bottom-up で共通ヘルパとして抽出**する。`supports_scope` のダミープロービング hack も除去する。新規の宣言的 DSL（`PlacementRule`/`DiscoverRule` 等）は当面作成せず、必要に応じて Phase F で薄い定数として後付けする。

## ユーザーレビューが必要な点

> **NOTE**
> - 振る舞い不変（パス・サポート可否・list 結果を変えない）のリファクタリングです。CLI 仕様変更なし。
> - Phase A〜G の段階移行（各 Phase が独立コミット/PR）。ビッグバンなし。
> - 外向き API（`placement_location` / `list_placed` / `supported_components` / `supports_scope` のシグネチャ）は一切変えません。
> - 旧計画にあった `TargetLayout` / `PlacementRule` / `DiscoverRule` 等の大規模 DSL は Phase F でのみ「必要ならデータ化」という方向性に格下げです。

---

## システム図

### 移行前後の `list_placed` 骨格フロー

```
【移行前: 5 ファイルにコピペ】

antigravity.rs::list_placed()        gemini_cli.rs::list_placed()
    │                                      │
    ├── can_place(kind)                    ├── can_place(kind)
    │       ↓ false → Ok([])              │       ↓ false → Ok([])
    ├── base_dir(scope, root)             ├── base_dir(scope, root)
    ├── dir_path = base.join("skills")    ├── dir_path = base.join("skills")
    ├── scan_components(&dir_path)        ├── scan_components(&dir_path)   ← 同じ
    └── filter_map(filter_component)      └── filter_map(filter_component) ← 同じ

(+ codex.rs, copilot.rs, cursor.rs でも同じ骨格)

【移行後: 共通ヘルパに集約】

antigravity.rs::list_placed()
    │
    └── list_placed_skill(&SKILL_CONFIG, scope, root)
              ↓
         shared_helper::list_skill_placed(base, "skills")
              │
              ├── can_place チェック（config 参照）
              ├── scan_components(&dir_path)    ← 既存再利用
              └── filter_map(is_skill_dir)      ← 共通 Skill フィルタ
```

### `supports_scope` ダミープロービング廃止フロー

```
【移行前: ダミープロービング hack】

Target::supports_scope(kind, scope)  [src/target.rs:258]
    │
    ▼ ダミー PlacementContext("test") 生成
    ▼ placement_location(dummy_ctx)
    ▼ is_some() → サポート判定 ← hack!

placed_common::list_instruction()
    │
    ▼ dummy_origin("test") 生成
    ▼ target.placement_location(dummy_ctx)
    ▼ path.exists() → ファイル存在確認

【移行後: can_place 直接参照】

Target::supports_scope(kind, scope)
    │
    ▼ self.can_place(kind, scope)  ← env が実装 or デフォルト実装
    ▼ 直接 bool 返却（ダミー廃止）

list_instruction_direct(target_path: &Path, filename: &str) → Vec<String>
    │
    ▼ target_path.exists() → ファイル存在確認（パス計算は呼び出し元が行う）
```

### `filter_component` 共通アーム抽出図

```
【移行前: 5 ファイルに同一コード】

antigravity: match kind { Skill if c.is_dir && SKILL.md.exists => name, _ => None }
gemini:      match kind { Skill if c.is_dir && SKILL.md.exists => name, _ => None }  ← コピペ
codex:       match kind { Skill if c.is_dir && SKILL.md.exists => name, ... }        ← コピペ
copilot:     match kind { Skill if c.is_dir && SKILL.md.exists => name, ... }        ← コピペ
cursor:      match kind { Skill if c.is_dir && SKILL.md.exists => name, ... }        ← コピペ

【移行後: 共通 Skill フィルタを 1 か所に】

// src/target/placed/scanner.rs または src/target/placed/filter.rs（新規）
pub fn filter_skill_dir(c: &ScannedComponent) -> Option<String> {
    if c.is_dir && c.path.join("SKILL.md").is_file() {
        Some(c.name.clone())
    } else {
        None
    }
}

// 各 env の filter_component から Skill アームを削除し、共通関数を呼ぶ
match kind {
    ComponentKind::Skill => filter_skill_dir(c),
    // 差分アームは env ファイルに残す
    ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => { ... }
    ...
}
```

### `placement_location` 共通パターン抽出図

```
【共通ヘルパ化対象パターン】

Skill（全5実装で同一形式）:
  PlacementLocation::dir(base.join("skills").join(name))
  → skill_placement_dir(base, name) に抽出

Agent（Codex/Copilot/Cursor で同一形式）:
  PlacementLocation::file(base.join("agents").join(format!("{}.agent.md", name)))
  → agent_placement_file(base, name) に抽出

Instruction（Gemini/Codex/Cursor で類似、filename が異なるのみ）:
  match scope {
      Project => PlacementLocation::file(project_root.join(filename)),
      Personal => PlacementLocation::file(base.join(filename)),
  }
  → instruction_placement_file(scope, project_root, base, filename) に抽出
```

### Phase 移行フロー（状態マシン）

```
  Phase A: 5 impl の差分表確定（読み取りのみ）
        │ exploration-report §8 を仕様凍結
        ▼
  Phase B: list_placed 共通骨格をヘルパに抽出
        │ Antigravity / Gemini から適用
        ▼
  Phase C: filter_component Skill アーム共通化
        │ filter_skill_dir 関数を 1 か所に
        ▼
  Phase D: supports_scope ダミープロービング廃止
        │ can_place 直接参照 + list_instruction 直接計算
        ▼
  Phase E: placement_location 共通パターンをヘルパ化
        │ Skill/Agent/Instruction の共通形
        ▼
  Phase F: 残った純粋データ差分を薄い定数にまとめる（省略可）
        │ 必要なら struct Config / static LAYOUT を後付け
        ▼
  Phase G: ドキュメント同期・不変条件テスト・死コード削除
        │
        ▼
       完了（FR-001〜FR-007 達成）
```

---

## 変更案

### Phase A: 5 impl 差分表確定（コード変更なし）

**目的**: exploration-report §8 の差分表を仕様として凍結し、移行後の期待値を確定する。

**成果物**: `exploration-report.md §8`（既に補強済み）

**凍結する差分表（BL-005 相当）**:

| kind | scope | Antigravity | Gemini | Codex | Copilot | Cursor |
|------|-------|-------------|--------|-------|---------|--------|
| Skill | Personal | ✓ `.gemini/antigravity/skills/<n>` | ✓ `.gemini/skills/<n>` | ✓ `.codex/skills/<n>` | ✗ | ✓ `.cursor/skills/<original>` |
| Skill | Project | ✓ `.agent/skills/<n>` | ✓ `.gemini/skills/<n>` | ✓ `.codex/skills/<n>` | ✓ `.github/skills/<n>` | ✓ `.cursor/skills/<original>` |
| Agent | Personal | ✗ | ✗ | ✓ `.codex/agents/<n>.agent.md` | ✓ `.copilot/agents/<n>.agent.md` | ✓ `.cursor/agents/<n>.agent.md` |
| Agent | Project | ✗ | ✗ | ✓ `.codex/agents/<n>.agent.md` | ✓ `.github/agents/<n>.agent.md` | ✓ `.cursor/agents/<n>.agent.md` |
| Command | Personal | ✗ | ✗ | ✗ | ✗ | ✓ `.cursor/rules/<n>.md` |
| Command | Project | ✗ | ✗ | ✗ | ✓ `.github/prompts/<n>.prompt.md` | ✓ `.cursor/rules/<n>.md` |
| Instruction | Personal | ✗ | ✓ `~/.gemini/GEMINI.md` | ✓ `~/.codex/AGENTS.md` | ✗ | ✗ |
| Instruction | Project | ✗ | ✓ `GEMINI.md`(root) | ✓ `AGENTS.md`(root) | ✓ `.github/copilot-instructions.md` | ✓ `AGENTS.md`(root) |
| Hook | Personal | ✗ | ✗ | ✓ `~/.codex/hooks.json` | ✓ `~/.copilot/hooks/*.json` | ✓ `~/.cursor/hooks.json` |
| Hook | Project | ✗ | ✗ | ✓ `.codex/hooks.json` | ✓ `.github/hooks/*.json` | ✓ `.cursor/hooks.json` |

---

### Phase B: `list_placed` 共通骨格をヘルパに抽出

**対象ファイル**: `src/target/env/antigravity.rs`, `src/target/env/gemini_cli.rs`（まず2ファイルで検証）

**[NEW] `src/target/placed/list_helpers.rs`（または `src/target/placed/scanner.rs` に追加）**

```rust
// src/target/placed/list_helpers.rs

use crate::component::{ComponentKind, Scope};
use crate::error::Result;
use crate::target::scanner::{scan_components, ScannedComponent};
use std::path::Path;

/// Skill / Agent / Hook などのディレクトリスキャン共通骨格
///
/// 各 env は subdir・filter クロージャだけ渡すだけになる。
pub(crate) fn scan_and_filter(
    base: &Path,
    subdir: &str,
    filter: impl Fn(&ScannedComponent) -> Option<String>,
) -> Result<Vec<String>> {
    let dir_path = base.join(subdir);
    let names = scan_components(&dir_path)?
        .into_iter()
        .filter_map(|c| filter(&c))
        .collect();
    Ok(names)
}
```

**[MODIFY] `src/target/env/antigravity.rs` — `list_placed` をヘルパ利用に変更**

```rust
// 変更前
fn list_placed(&self, kind: ComponentKind, scope: Scope, project_root: &Path) -> Result<Vec<String>> {
    if !Self::can_place(kind) {
        return Ok(vec![]);
    }
    let base = Self::base_dir(scope, project_root);
    let dir_path = base.join("skills");
    let names = scan_components(&dir_path)?
        .into_iter()
        .filter_map(|c| Self::filter_component(&c, kind))
        .collect();
    Ok(names)
}

// 変更後
fn list_placed(&self, kind: ComponentKind, scope: Scope, project_root: &Path) -> Result<Vec<String>> {
    if !Self::can_place(kind) {
        return Ok(vec![]);
    }
    let base = Self::base_dir(scope, project_root);
    match kind {
        ComponentKind::Skill => scan_and_filter(&base, "skills", filter_skill_dir),
        _ => Ok(vec![]),
    }
}
```

---

### Phase C: `filter_component` Skill アーム共通化

**[NEW] `src/target/placed/filter.rs`（または list_helpers.rs に追加）**

```rust
// src/target/placed/filter.rs

use crate::target::scanner::ScannedComponent;

/// SKILL.md を含むディレクトリを Skill として認識する共通フィルタ
/// （全 5 実装で同一だったロジックを 1 か所に集約）
pub(crate) fn filter_skill_dir(c: &ScannedComponent) -> Option<String> {
    if c.is_dir && c.path.join("SKILL.md").is_file() {
        Some(c.name.clone())
    } else {
        None
    }
}
```

**[MODIFY] 各 env の `filter_component` — Skill アームを削除してヘルパ呼び出しに**

```rust
// 各 env で Skill アームを filter_skill_dir に置き換える
fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
    match kind {
        ComponentKind::Skill => filter_skill_dir(c),  // ← 共通ヘルパへ
        ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => {
            Some(c.name.trim_end_matches(".agent.md").to_string())
        }
        // 差分アームは各 env に残す
        _ => None,
    }
}
```

---

### Phase D: `supports_scope` ダミープロービング廃止

**[MODIFY] `src/target.rs` — `supports_scope` デフォルト実装を書き換え**

現行のダミー実装（`src/target.rs:258-266`）を廃止する。廃止後の実装は以下の 2 方式のうち実装時に選択する：

**方式1（最小変更）**: `Target` trait に `fn can_place_scope(&self, kind: ComponentKind, scope: Scope) -> bool` を追加し、各 env でそれを `can_place(kind, scope)` で実装する。`supports_scope` のデフォルトは `can_place_scope` 呼び出しに変更。

```rust
// src/target.rs (方式1)
fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
    // ダミープロービング廃止：各 env の can_place 系を直接呼ぶ
    self.can_place_scope(kind, scope)
}

// デフォルト実装（supported_components に入っていれば Both 扱い）
fn can_place_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
    self.supported_components().contains(&kind)  // scope 無関係ならデフォルト
}
// Copilot/Cursor などは override して scope チェックを追加
```

**方式2（より薄い Data 化）**: `can_place` を `static &[(ComponentKind, Scope)]` の許可ペアスライスで表現し、`supports_scope` がスライスを参照する。

**[MODIFY] `src/target/placed/placed_common.rs` — `list_instruction` のダミー廃止**

```rust
// 変更前: ダミー PlacementContext を使って placement_location を呼ぶ
pub(crate) fn list_instruction(target: &dyn Target, scope: Scope, project_root: &Path, filename: &str) -> Vec<String> {
    let dummy_origin = PluginOrigin::from_marketplace("test", "test");
    let ctx = PlacementContext { ... };
    let Some(location) = target.placement_location(&ctx) else { return vec![]; };
    if location.as_path().exists() { vec![filename.to_string()] } else { vec![] }
}

// 変更後: パスを直接計算（呼び出し元が base を渡す）
pub(crate) fn list_instruction_at(path: &Path, filename: &str) -> Vec<String> {
    if path.exists() {
        vec![filename.to_string()]
    } else {
        vec![]
    }
}
// 呼び出し側で: list_instruction_at(&base.join(filename), filename)
// または: list_instruction_at(&project_root.join(filename), filename)
```

---

### Phase E: `placement_location` 共通パターンをヘルパ化

**[NEW] `src/target/placed/placement_helpers.rs`（または既存ファイルに関数追加）**

```rust
// src/target/placed/placement_helpers.rs

use crate::component::{PlacementLocation, Scope};
use std::path::Path;

/// Skill: base/skills/<name> ディレクトリ（全5実装で共通）
pub(crate) fn skill_dir(base: &Path, name: &str) -> PlacementLocation {
    PlacementLocation::dir(base.join("skills").join(name))
}

/// Agent: base/agents/<name>.agent.md ファイル（Codex/Copilot/Cursor で共通）
pub(crate) fn agent_file(base: &Path, name: &str) -> PlacementLocation {
    PlacementLocation::file(base.join("agents").join(format!("{}.agent.md", name)))
}

/// Instruction: Project → project_root/<filename>, Personal → base/<filename>
/// （Gemini/Codex/Cursor で共通パターン、filename のみ違う）
pub(crate) fn instruction_file(
    scope: Scope,
    project_root: &Path,
    base: &Path,
    filename: &str,
) -> PlacementLocation {
    match scope {
        Scope::Project => PlacementLocation::file(project_root.join(filename)),
        Scope::Personal => PlacementLocation::file(base.join(filename)),
    }
}
```

**[MODIFY] 各 env の `placement_location` — 共通パターンをヘルパ呼び出しに変更**

```rust
// 例: antigravity.rs
fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
    if !Self::can_place(context.kind()) { return None; }
    let base = Self::base_dir(context.scope(), context.project_root());
    Some(match context.kind() {
        ComponentKind::Skill => skill_dir(&base, context.name()),  // ← ヘルパへ
        _ => return None,
    })
}
```

**注意**: Cursor Skill の `original_name` 必須（OriginalNameRequired 相当）は Cursor の override に残す。

---

### Phase F: 残った純粋データ差分を薄い定数にまとめる（オプション）

**判断基準**: Phase B〜E 完了後に、各 env ファイルに残った「pure data（文字列リテラルや bool）」の差分が多い場合のみ実施。

```rust
// 最終形候補（Phase F が必要な場合）
// src/target/env/antigravity.rs
struct AntigravityConfig {
    personal_parent: &'static str,   // ".gemini"
    personal_child: &'static str,    // "antigravity"
    project_subdir: &'static str,    // ".agent"
}
const CONFIG: AntigravityConfig = AntigravityConfig {
    personal_parent: ".gemini",
    personal_child: "antigravity",
    project_subdir: ".agent",
};
```

この段階で十分なデータ集約ができていれば、薄い `TargetLayout` const（Phase F）はスキップしてよい。

---

### Phase G: ドキュメント同期・不変条件テスト・死コード削除

- `can_place` プライベート関数の削除（`can_place_scope` に統一後）
- `base_dir` プライベート関数の削除（直接 `paths::base_dir` を呼ぶ形に）
- `filter_component` プライベート関数の削除（`filter_skill_dir` + 差分は `list_placed` 内にインライン化）
- `placed_common::list_instruction` の削除（`list_instruction_at` に置換後）
- `docs/target-layout-refactor/` を `docs/old/` にアーカイブ
- `docs/architecture/` に新しい共通ヘルパ関数の説明を追記

---

## 検証計画

### フェーズ完了条件（全 Phase 共通）

1. `cargo test` で全テスト（既存 1888 行 + 新規追加分）が green
2. `cargo build` でコンパイルエラーなし
3. `cargo clippy` で新規 warning なし
4. 振る舞い不変の確認: Phase A で確定した差分表と `cargo test` の期待値が一致

### 重点テスト観点

| 観点 | 検証方法 |
|------|---------|
| `list_placed` リグレッション | 既存 `*_test.rs` の期待値維持 |
| Skill フィルタ精度 | SKILL.md なしディレクトリを除外するか |
| `supports_scope` ダミー廃止 | `grep -r '"test"' src/target.rs` がゼロになるか |
| Copilot Skill Personal 不可 | `supports_scope(Skill, Personal)` が false |
| Cursor Skill original_name 欠落 | `placement_location` が None |
| Instruction list_placed | ファイル存在確認のみで name 非依存 |
| FakeTarget 非影響 | `src/sync_test.rs` / `endpoint_test.rs` が green 維持 |

### 自動テスト追加方針

- **Phase B**: `scan_and_filter` ヘルパの単体テスト（TempDir 使用）
- **Phase C**: `filter_skill_dir` の単体テスト（SKILL.md あり/なし両ケース）
- **Phase D**: `list_instruction_at` の単体テスト（ファイルあり/なし両ケース）
- **Phase E**: `skill_dir` / `agent_file` / `instruction_file` ヘルパの単体テスト
- **各 Phase**: 移行した env の既存テストが全 green であることを確認

---

## 定義 (Definition of Done)

- **G-001**: `src/target.rs` に `PlacementContext("test")` / `PluginOrigin::from_marketplace("test",*)` を使ったサポート判定コードが残らない
- **G-002**: `placed_common::list_instruction` の `dummy_origin` 生成が廃止されている（または関数自体が削除されている）
- **G-003**: `list_placed` の骨格（scope チェック → base_dir → scan → filter）が各 env ファイルにコピペされていない（共通ヘルパを呼んでいる）
- **G-004**: `filter_component` の Skill アームが各 env ファイルから除去されている（共通 `filter_skill_dir` を参照）
- **G-005**: `placement_location` の Skill/Agent/Instruction 共通パターンが共通ヘルパを呼んでいる
- **G-006**: 振る舞い不変——Phase A の差分表と全テストの期待値が一致したまま
