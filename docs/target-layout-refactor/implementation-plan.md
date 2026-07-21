# target/env 宣言的ケイパビリティ集約

**関連Issue**: #338

`target/env/` の 5 実装（Antigravity / Gemini CLI / Codex / Copilot / Cursor）で `list_placed` / `filter_component` / `placement_location` / `base_dir` の制御フローがコピペされており、`supported_components` スライスと `can_place`（実配置ガード）の二重真実源が乖離リスクを生んでいる。差分を `TargetLayout` 宣言的記述子に集約し、サポート判定の単一真実源を確立する。

## ユーザーレビューが必要な点

> **NOTE**
> - 振る舞い不変（パス・サポート可否・list 結果を変えない）のリファクタリングです。CLI 仕様変更なし。
> - Phase 0〜6 の段階移行（各 Phase が独立 PR）。ビッグバンなし。
> - `Target` trait に `fn layout() -> &'static TargetLayout` を追加した場合、`FakeTarget`（sync テスト等）はデフォルト実装で動作します。既存テストへの影響なし。
> - Cursor Skill の `original_name` 必須制約は `NamingPolicy::OriginalNameRequired` として明示され、現状の暗黙 None 返りが設計として文書化されます。

## システム図

### 移行前後の構造フロー

```
【移行前】

supports_scope(kind, scope)
         │
         ▼ ダミー PlacementContext("test") 生成
placement_location(dummy_ctx)
         │
         ▼ is_some() で判定 ← hack
         結果

list_placed(kind, scope, root)
         │
         ├── can_place(kind [, scope]) // 各 env に手書き
         │         │
         │      false ─────────────▶ Ok([])
         │         │ true
         │         ▼
         ├── base_dir(scope, root)   // 各 env に手書き
         │         │
         │         ▼
         ├── scan_components(dir)
         │         │
         │         ▼
         └── filter_component(c, kind)  // 各 env にコピペ
                   │
                   ▼ Ok(names)

supported_components() → &[ComponentKind]  // 手書きスライス（scope 非依存）
```

```
【移行後】

supports_scope(kind, scope)
         │
         ▼ layout().capabilities 参照
ScopeSet::contains(scope)  ← ダミー廃止
         │
         ▼ 結果

list_placed(kind, scope, root)
         │
         ▼
supports_scope(kind, scope)  // capabilities 参照
         │
      false ──────────────────▶ Ok([])
         │ true
         ▼
capability.discover == InstructionExists?
         │ yes
         ├── path = 直接計算（origin 不要）
         ├── path.exists() → [filename] or []
         │ no
         ▼
dir = scan_root(capability) // PlacementRule から直接
         │
         ▼
scan_components(dir)
         │
         ▼
apply_discover(c, capability.discover)  // DiscoverRule で統一
         │
         ▼ Ok(names)

supported_components()  // capabilities から導出（手書き不要）
    capabilities
      .iter()
      .filter(|c| !c.scopes.is_empty())
      .map(|c| c.kind)
```

### TargetLayout データモデル

```
TargetLayout
├── kind: TargetKind
├── display_name: &'static str
├── base: BasePathLayout
│   ├── personal: BasePathSpec
│   │   ├── Subdir(&'static str)          // 例: ".codex", ".cursor"
│   │   └── Nested { parent, child }       // 例: Antigravity ".gemini/antigravity"
│   └── project: BasePathSpec              // 例: ".codex", ".github", ".agent"
└── components: &'static [ComponentCapability]
    └── ComponentCapability
        ├── kind: ComponentKind
        ├── scopes: ScopeSet               // PersonalOnly / ProjectOnly / Both
        ├── placement: PlacementRule
        │   ├── ComponentDir { subdir, naming }     // Skill 典型
        │   ├── ComponentFile { subdir, suffix, naming }  // Agent/Command/Hook
        │   ├── FixedAtBase { filename }             // Codex/Cursor hooks.json
        │   └── InstructionFile { filename, project_location }
        └── discover: DiscoverRule
            ├── SkillManifestDir
            ├── SuffixFile { suffix }
            ├── PlainMarkdownFile
            ├── ExactFile { filename, listed_as }
            ├── JsonSuffixFile
            └── InstructionExists

NamingPolicy
├── FlattenedName               // context.name() (ほとんどの種別)
└── OriginalNameRequired        // context.original_name() 必須 (Cursor Skill)

InstructionProjectLocation
├── ProjectRoot                 // Codex/Cursor AGENTS.md, Gemini GEMINI.md
└── UnderBase                   // Copilot copilot-instructions.md
```

### Phase 移行フロー（状態マシン）

```
  Phase 0: 仕様凍結・受け入れテスト棚卸し
        │ 現状テスト全 green 確認、不足テスト追加
        ▼
  Phase 1: TargetLayout モデル + 導出ヘルパ（TDD）
        │ src/target/layout/ 追加、既存 env 未変更
        ▼
  Phase 2: Antigravity / Gemini CLI 移行
        │ 最小ターゲット2つ、フックなし
        ▼
  Phase 3: Codex / Copilot 移行
        │ Instruction/Hook 含む、placed_common 縮小
        ▼
  Phase 4: Cursor 移行
        │ OriginalNameRequired 含む最終ターゲット
        ▼
  Phase 5: trait デフォルト接続・ダミー廃止
        │ supports_scope デフォルト実装置換
        │ placed_common::list_instruction 廃止
        ▼
  Phase 6: ドキュメント同期・掃除
        │ docs/concepts 更新、死コード削除
        ▼
       完了（G-001 / G-002 / G-003 達成）
```

---

## 変更案

### Phase 1: 新規モジュール追加（既存 env は変更しない）

#### [NEW] `src/target/layout.rs`

`target/layout` モジュールのルート。データ型・導出ヘルパを集約する。

```rust
// src/target/layout.rs

mod derive;
mod model;

pub use derive::{
    derive_list_placed, derive_placement_location, derive_supported_components,
    derive_supports_scope,
};
pub use model::{
    BasePathLayout, BasePathSpec, ComponentCapability, DiscoverRule, InstructionProjectLocation,
    NamingPolicy, PlacementRule, ScopeSet, TargetLayout,
};
```

#### [NEW] `src/target/layout/model.rs`

`TargetLayout` と関連型の定義。すべて `'static` 寿命を持つ定数として使える。

```rust
// src/target/layout/model.rs

use crate::component::{ComponentKind, Scope};
use crate::target::TargetKind;

pub struct TargetLayout {
    pub kind: TargetKind,
    pub display_name: &'static str,
    pub base: BasePathLayout,
    pub components: &'static [ComponentCapability],
}

pub struct BasePathLayout {
    pub personal: BasePathSpec,
    pub project: BasePathSpec,
}

pub enum BasePathSpec {
    /// `~/.<subdir>` または `<root>/.<subdir>`
    Subdir(&'static str),
    /// Antigravity Personal のように `~/<parent>/<child>`
    Nested { parent: &'static str, child: &'static str },
}

pub struct ComponentCapability {
    pub kind: ComponentKind,
    pub scopes: ScopeSet,
    pub placement: PlacementRule,
    pub discover: DiscoverRule,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScopeSet {
    PersonalOnly,
    ProjectOnly,
    Both,
}

impl ScopeSet {
    pub fn contains(self, scope: Scope) -> bool {
        match (self, scope) {
            (ScopeSet::Both, _) => true,
            (ScopeSet::PersonalOnly, Scope::Personal) => true,
            (ScopeSet::ProjectOnly, Scope::Project) => true,
            _ => false,
        }
    }

    pub fn is_empty(self) -> bool { false }  // enum なので常に非空
}

pub enum PlacementRule {
    ComponentDir { subdir: &'static str, naming: NamingPolicy },
    ComponentFile { subdir: &'static str, suffix: &'static str, naming: NamingPolicy },
    FixedAtBase { filename: &'static str },
    InstructionFile { filename: &'static str, project_location: InstructionProjectLocation },
}

pub enum DiscoverRule {
    SkillManifestDir,
    SuffixFile { suffix: &'static str },
    PlainMarkdownFile,
    ExactFile { filename: &'static str, listed_as: &'static str },
    JsonSuffixFile,
    InstructionExists,
}

#[derive(Clone, Copy)]
pub enum NamingPolicy {
    FlattenedName,
    OriginalNameRequired,
}

#[derive(Clone, Copy)]
pub enum InstructionProjectLocation {
    ProjectRoot,
    UnderBase,
}
```

#### [NEW] `src/target/layout/derive.rs`

`TargetLayout` から `placement_location` / `list_placed` / `supported_components` / `supports_scope` を導出する共通ヘルパ関数。

```rust
// src/target/layout/derive.rs

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::core::paths::{base_dir, home_dir};
use crate::target::placed::scanner::scan_components;
use crate::target::layout::model::*;
use std::path::Path;

/// TargetLayout から PlacementLocation を導出する
pub fn derive_placement_location(
    layout: &TargetLayout,
    context: &PlacementContext,
) -> Option<PlacementLocation> {
    let kind = context.kind();
    let scope = context.scope();
    let cap = layout.components.iter().find(|c| c.kind == kind)?;
    if !cap.scopes.contains(scope) {
        return None;
    }
    let base = resolve_base(&layout.base, scope, context.project_root());
    resolve_placement(&cap.placement, context, &base, scope, context.project_root())
}

/// TargetLayout から supports_scope を導出する（ダミーコンテキスト不要）
pub fn derive_supports_scope(
    layout: &TargetLayout,
    kind: ComponentKind,
    scope: Scope,
) -> bool {
    layout.components.iter()
        .any(|c| c.kind == kind && c.scopes.contains(scope))
}

/// TargetLayout から supported_components を導出する
pub fn derive_supported_components(layout: &'static TargetLayout) -> Vec<ComponentKind> {
    let mut kinds: Vec<ComponentKind> = layout.components.iter()
        .map(|c| c.kind)
        .collect();
    kinds.dedup();
    kinds
}

/// TargetLayout から list_placed を導出する
pub fn derive_list_placed(
    layout: &TargetLayout,
    kind: ComponentKind,
    scope: Scope,
    project_root: &Path,
) -> Result<Vec<String>> {
    if !derive_supports_scope(layout, kind, scope) {
        return Ok(vec![]);
    }
    let cap = layout.components.iter().find(|c| c.kind == kind).unwrap();
    // Instruction は直接パス計算（scan 不要）
    if matches!(cap.discover, DiscoverRule::InstructionExists) {
        return list_instruction_exists(layout, kind, scope, project_root);
    }
    let base = resolve_base(&layout.base, scope, project_root);
    let dir = scan_dir_for_capability(&cap.placement, &base);
    let names = scan_components(&dir)?
        .into_iter()
        .filter_map(|c| apply_discover(&c, &cap.discover))
        .collect();
    Ok(names)
}

// ... 内部ヘルパ: resolve_base / resolve_placement / resolve_name / scan_dir / apply_discover
```

#### [NEW] `src/target/layout/model_test.rs`

`TargetLayout` 型定義の単体テスト（Phase 1 Red-Green）。

```rust
// src/target/layout/model_test.rs

use super::*;

#[test]
fn test_scope_set_both_contains_all() {
    assert!(ScopeSet::Both.contains(Scope::Personal));
    assert!(ScopeSet::Both.contains(Scope::Project));
}

#[test]
fn test_scope_set_personal_only() {
    assert!(ScopeSet::PersonalOnly.contains(Scope::Personal));
    assert!(!ScopeSet::PersonalOnly.contains(Scope::Project));
}

#[test]
fn test_scope_set_project_only() {
    assert!(ScopeSet::ProjectOnly.contains(Scope::Project));
    assert!(!ScopeSet::ProjectOnly.contains(Scope::Personal));
}
```

#### [NEW] `src/target/layout/derive_test.rs`

架空の `TargetLayout` でヘルパ関数をテーブル駆動テスト（Phase 1 Red-Green 核心）。

```rust
// src/target/layout/derive_test.rs

// テスト用の最小 TargetLayout を定義し、derive_supports_scope /
// derive_placement_location / derive_list_placed の導出が
// 既存ターゲットと同等であることを検証する。

use super::*;
use crate::component::{ComponentKind, Scope};

const TEST_LAYOUT: TargetLayout = TargetLayout {
    kind: crate::target::TargetKind::Antigravity,
    display_name: "Test",
    base: BasePathLayout {
        personal: BasePathSpec::Subdir(".test"),
        project: BasePathSpec::Subdir(".test"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentDir {
                subdir: "skills",
                naming: NamingPolicy::FlattenedName,
            },
            discover: DiscoverRule::SkillManifestDir,
        },
    ],
};

#[test]
fn test_derive_supports_scope_skill_both() {
    assert!(derive_supports_scope(&TEST_LAYOUT, ComponentKind::Skill, Scope::Personal));
    assert!(derive_supports_scope(&TEST_LAYOUT, ComponentKind::Skill, Scope::Project));
}

#[test]
fn test_derive_supports_scope_agent_not_in_layout() {
    assert!(!derive_supports_scope(&TEST_LAYOUT, ComponentKind::Agent, Scope::Personal));
}

#[test]
fn test_derive_list_placed_unsupported_returns_empty() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let result = derive_list_placed(&TEST_LAYOUT, ComponentKind::Agent, Scope::Project, tmp.path());
    assert!(result.unwrap().is_empty());
}
```

---

### Phase 2: Antigravity / Gemini CLI 移行

#### [MODIFY] `src/target/env/antigravity.rs`

`can_place` / `base_dir` / `filter_component` を削除し、`LAYOUT` 定数 + 共通ヘルパに置換。

```rust
// before: (現状)
impl AntigravityTarget {
    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir().join(ANTIGRAVITY_PERSONAL_PARENT).join(ANTIGRAVITY_PERSONAL_CHILD),
            Scope::Project => project_root.join(ANTIGRAVITY_PROJECT_SUBDIR),
        }
    }
    fn can_place(kind: ComponentKind) -> bool { kind == ComponentKind::Skill }
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> { ... }
}

impl Target for AntigravityTarget {
    fn supported_components(&self) -> &[ComponentKind] { &[ComponentKind::Skill] }
    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> { ... }
    fn list_placed(&self, kind: ComponentKind, scope: Scope, project_root: &Path) -> Result<Vec<String>> { ... }
}
```

```rust
// after: (LAYOUT 定数 + 共通ヘルパ)
use crate::target::layout::{
    BasePathLayout, BasePathSpec, ComponentCapability, DiscoverRule, NamingPolicy,
    PlacementRule, ScopeSet, TargetLayout,
    derive_placement_location, derive_list_placed, derive_supported_components, derive_supports_scope,
};

const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Antigravity,
    display_name: "Google Antigravity",
    base: BasePathLayout {
        personal: BasePathSpec::Nested { parent: ".gemini", child: "antigravity" },
        project: BasePathSpec::Subdir(".agent"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentDir { subdir: "skills", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SkillManifestDir,
        },
    ],
};

impl Target for AntigravityTarget {
    fn display_name(&self) -> &'static str { LAYOUT.display_name }
    fn kind(&self) -> TargetKind { TargetKind::Antigravity }
    fn supported_components(&self) -> &[ComponentKind] {
        // Phase 5 で trait デフォルト実装へ移行。それまでは静的スライスのまま
        &[ComponentKind::Skill]
    }
    fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        derive_supports_scope(&LAYOUT, kind, scope)
    }
    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        derive_placement_location(&LAYOUT, context)
    }
    fn list_placed(&self, kind: ComponentKind, scope: Scope, project_root: &Path) -> Result<Vec<String>> {
        derive_list_placed(&LAYOUT, kind, scope, project_root)
    }
}
```

#### [MODIFY] `src/target/env/gemini_cli.rs`

Antigravity と同様の置換。`LAYOUT` 定数に Instruction を追加。

```rust
// after: LAYOUT 定数の主要部分
const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::GeminiCli,
    display_name: "Gemini CLI",
    base: BasePathLayout {
        personal: BasePathSpec::Subdir(".gemini"),
        project: BasePathSpec::Subdir(".gemini"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentDir { subdir: "skills", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SkillManifestDir,
        },
        ComponentCapability {
            kind: ComponentKind::Instruction,
            scopes: ScopeSet::Both,
            placement: PlacementRule::InstructionFile {
                filename: "GEMINI.md",
                project_location: InstructionProjectLocation::ProjectRoot,
            },
            discover: DiscoverRule::InstructionExists,
        },
    ],
};
```

---

### Phase 3: Codex / Copilot 移行

#### [MODIFY] `src/target/env/codex.rs`

Hook / Agent / Instruction の `PlacementRule` を含む。振る舞いフック（`pre_place_check`, `post_place`, `component_conflict_error`）は残す。

```rust
// after: LAYOUT 定数（フック override は別途 impl Target に残す）
const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Codex,
    display_name: "OpenAI Codex",
    base: BasePathLayout {
        personal: BasePathSpec::Subdir(".codex"),
        project: BasePathSpec::Subdir(".codex"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentDir { subdir: "skills", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SkillManifestDir,
        },
        ComponentCapability {
            kind: ComponentKind::Agent,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentFile { subdir: "agents", suffix: ".agent.md", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SuffixFile { suffix: ".agent.md" },
        },
        ComponentCapability {
            kind: ComponentKind::Instruction,
            scopes: ScopeSet::Both,
            placement: PlacementRule::InstructionFile {
                filename: "AGENTS.md",
                project_location: InstructionProjectLocation::ProjectRoot,
            },
            discover: DiscoverRule::InstructionExists,
        },
        ComponentCapability {
            kind: ComponentKind::Hook,
            scopes: ScopeSet::Both,
            placement: PlacementRule::FixedAtBase { filename: "hooks.json" },
            discover: DiscoverRule::ExactFile { filename: "hooks.json", listed_as: "hooks" },
        },
    ],
};
```

#### [MODIFY] `src/target/env/copilot.rs`

`ScopeSet::ProjectOnly` で Skill/Command/Instruction の scope 制約を宣言。

```rust
// after: LAYOUT 定数（ScopeSet::ProjectOnly が scope 制約を一元管理）
const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Copilot,
    display_name: "GitHub Copilot",
    base: BasePathLayout {
        personal: BasePathSpec::Subdir(".copilot"),
        project: BasePathSpec::Subdir(".github"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::ProjectOnly,      // Personal 不可
            placement: PlacementRule::ComponentDir { subdir: "skills", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SkillManifestDir,
        },
        ComponentCapability {
            kind: ComponentKind::Agent,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentFile { subdir: "agents", suffix: ".agent.md", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SuffixFile { suffix: ".agent.md" },
        },
        ComponentCapability {
            kind: ComponentKind::Command,
            scopes: ScopeSet::ProjectOnly,      // Personal 不可
            placement: PlacementRule::ComponentFile { subdir: "prompts", suffix: ".prompt.md", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::SuffixFile { suffix: ".prompt.md" },
        },
        ComponentCapability {
            kind: ComponentKind::Instruction,
            scopes: ScopeSet::ProjectOnly,      // Personal 不可
            placement: PlacementRule::InstructionFile {
                filename: "copilot-instructions.md",
                project_location: InstructionProjectLocation::UnderBase,
            },
            discover: DiscoverRule::InstructionExists,
        },
        ComponentCapability {
            kind: ComponentKind::Hook,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentFile { subdir: "hooks", suffix: ".json", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::JsonSuffixFile,
        },
    ],
};
```

---

### Phase 4: Cursor 移行

#### [MODIFY] `src/target/env/cursor.rs`

`OriginalNameRequired` / `PlainMarkdownFile` を含む。振る舞いフックは残す。

```rust
// after: LAYOUT 定数
const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Cursor,
    display_name: "Cursor",
    base: BasePathLayout {
        personal: BasePathSpec::Subdir(".cursor"),
        project: BasePathSpec::Subdir(".cursor"),
    },
    components: &[
        ComponentCapability {
            kind: ComponentKind::Skill,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentDir { subdir: "skills", naming: NamingPolicy::OriginalNameRequired },
            discover: DiscoverRule::SkillManifestDir,
        },
        ComponentCapability {
            kind: ComponentKind::Agent,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentFile { subdir: "agents", suffix: ".md", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::PlainMarkdownFile,
        },
        ComponentCapability {
            kind: ComponentKind::Command,
            scopes: ScopeSet::Both,
            placement: PlacementRule::ComponentFile { subdir: "commands", suffix: ".md", naming: NamingPolicy::FlattenedName },
            discover: DiscoverRule::PlainMarkdownFile,
        },
        ComponentCapability {
            kind: ComponentKind::Instruction,
            scopes: ScopeSet::ProjectOnly,      // Personal User Rules は対象外
            placement: PlacementRule::InstructionFile {
                filename: "AGENTS.md",
                project_location: InstructionProjectLocation::ProjectRoot,
            },
            discover: DiscoverRule::InstructionExists,
        },
        ComponentCapability {
            kind: ComponentKind::Hook,
            scopes: ScopeSet::Both,
            placement: PlacementRule::FixedAtBase { filename: "hooks.json" },
            discover: DiscoverRule::ExactFile { filename: "hooks.json", listed_as: "hooks" },
        },
    ],
};
```

---

### Phase 5: trait デフォルト接続・ダミー廃止

#### [MODIFY] `src/target.rs`（`supports_scope` デフォルト実装置換）

```rust
// before:
fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
    let dummy_origin = PluginOrigin::from_marketplace("test", "test");
    let ctx = PlacementContext {
        component: ComponentRef::with_names(kind, "test", "test", "test"),
        origin: &dummy_origin,
        scope: PlacementScope::new(scope),
        project: ProjectContext::new(Path::new(".")),
    };
    self.placement_location(&ctx).is_some()
}
```

```rust
// after: (Phase 5 完了時)
// 各 env が layout() を実装した後は trait デフォルトをこちらに差し替え
// それまでは各 env で supports_scope を直接 override
fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
    // Phase 5 で layout() のデフォルト実装を追加し、capabilities 参照に変更
    // 移行期間は各 env の override に委ねる
    self.placement_location(&PlacementContext::probe(kind, scope)).is_some()
}
```

#### [MODIFY] `src/target/placed/placed_common.rs`

`list_instruction` のダミー廃止。`InstructionExists` 導出ロジックに統合後に `placed_common.rs` を縮小または削除。

```rust
// before:
pub(crate) fn list_instruction(...) -> Vec<String> {
    let dummy_origin = PluginOrigin::from_marketplace("test", "test");
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        ...
    };
    let Some(location) = target.placement_location(&ctx) else { ... };
    ...
}
```

```rust
// after: (Phase 5 以降 — env の list_placed が layout 導出になれば不要)
// derive_list_placed が InstructionExists を直接計算するため、
// placed_common::list_instruction の呼び出し箇所が消える
// → placed_common.rs は削除（または空のモジュールとして残す）
```

---

### Phase 6: ドキュメント同期（[MODIFY]）

- `docs/architecture/core-design.md` または `docs/concepts/targets.md` に `TargetLayout` モデル概要を追記
- `docs/target-layout-refactor/` の 3 ファイルを `docs/old/` へアーカイブ（仕様を正式採用したため）

---

## 検証計画

### テスト戦略

**機能タイプ**: Pure Logic（`TargetLayout` + 導出ヘルパ）+ Configuration（各ターゲットの `LAYOUT` 定数）
**テスト方針**: TDD（Red → Green → Refactor）
**根拠**: 配置パス計算・サポート可否判定は副作用なし Pure Logic であり、入力→出力のテーブル駆動テストが最適。Phase 1 で先にテストを書き（Red）、モデル実装（Green）→リファクタのサイクルで進める。既存 1888 行のテストがリグレッションガードとして機能する。

### 自動テスト

#### `src/target/layout/model_test.rs`

**役割**: `ScopeSet` / `BasePathSpec` などの型メソッドの単体テスト

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `ScopeSet::Both.contains(Personal)` | Both スコープセットに Personal を問い合わせ | `true` |
| 正常系 | `ScopeSet::Both.contains(Project)` | Both スコープセットに Project を問い合わせ | `true` |
| 正常系 | `ScopeSet::PersonalOnly.contains(Personal)` | PersonalOnly に Personal | `true` |
| 境界値 | `ScopeSet::PersonalOnly.contains(Project)` | PersonalOnly に Project | `false` |
| 境界値 | `ScopeSet::ProjectOnly.contains(Personal)` | ProjectOnly に Personal | `false` |
| 正常系 | `ScopeSet::ProjectOnly.contains(Project)` | ProjectOnly に Project | `true` |

#### `src/target/layout/derive_test.rs`

**役割**: 架空の最小 `TargetLayout` で `derive_supports_scope` / `derive_placement_location` / `derive_list_placed` の動作を検証

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `derive_supports_scope` 登録済み kind+scope | Both な Skill に Personal を問い合わせ | `true` |
| 正常系 | `derive_supports_scope` ProjectOnly に Project | ProjectOnly Skill に Project | `true` |
| 境界値 | `derive_supports_scope` ProjectOnly に Personal | ProjectOnly Skill に Personal | `false` |
| 正常系 | `derive_supports_scope` 未登録 kind | Agent が layouts にない | `false` |
| 正常系 | `derive_placement_location` ComponentDir Skill | Skill に project root を渡す | `Some(Dir(.test/skills/name))` |
| 境界値 | `derive_placement_location` scope 外 | ProjectOnly Skill に Personal scope | `None` |
| エッジケース | `derive_placement_location` OriginalNameRequired + None | original_name が None | `None` |
| 正常系 | `derive_list_placed` 空ディレクトリ | dir が存在しない | `Ok([])` |
| 正常系 | `derive_list_placed` SkillManifestDir | SKILL.md あり | `Ok(["plugin_skill"])` |
| 境界値 | `derive_list_placed` SKILL.md なし | ディレクトリあるが SKILL.md なし | `Ok([])` |
| 境界値 | `derive_list_placed` scope 外 | ProjectOnly に Personal | `Ok([])` |
| 正常系 | `derive_list_placed` InstructionExists あり | AGENTS.md 存在する | `Ok(["AGENTS.md"])` |
| 境界値 | `derive_list_placed` InstructionExists なし | AGENTS.md 存在しない | `Ok([])` |

#### `src/target/env/antigravity_test.rs`（既存・移行後も期待値変更なし）

**役割**: Antigravity 移行後の振る舞い不変を確認するリグレッションテスト

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 不変条件 | `supported_components` が Skill のみ | Phase 2 移行後 | `[Skill]` |
| 不変条件 | `supports_scope(Skill, Both)` | Phase 2 移行後 | `true` |
| 不変条件 | `supports_scope(Agent, Personal)` | Phase 2 移行後 | `false` |
| 不変条件 | `placement_location` Skill Personal パス | Phase 2 移行後 | `~/.gemini/antigravity/skills/<name>` |
| 不変条件 | `placement_location` Skill Project パス | Phase 2 移行後 | `{root}/.agent/skills/<name>` |
| 不変条件 | `list_placed` 空ディレクトリ | Phase 2 移行後 | `Ok([])` |

#### `src/target/env/copilot_test.rs`（既存・Phase 3 移行後）

**役割**: Copilot の scope 制約（ProjectOnly）が移行後も維持されることを確認

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 不変条件 | `supports_scope(Skill, Personal)` | Phase 3 移行後 | `false` |
| 不変条件 | `supports_scope(Skill, Project)` | Phase 3 移行後 | `true` |
| 不変条件 | `supports_scope(Agent, Personal)` | Phase 3 移行後 | `true` |
| 不変条件 | `placement_location(Skill, Personal)` | Phase 3 移行後 | `None` |
| 不変条件 | `placement_location(Hook, Personal)` | Phase 3 移行後 | `Some(...)` |

#### `src/target/env/cursor_test.rs`（既存・Phase 4 移行後）

**役割**: Cursor の `OriginalNameRequired` / `PlainMarkdownFile` ポリシーが移行後も維持されることを確認

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| エッジケース | `placement_location(Skill)` original_name=None | Phase 4 移行後 | `None` |
| エッジケース | `placement_location(Skill)` original_name="" | Phase 4 移行後 | `None` |
| 不変条件 | `list_placed(Agent)` が `.agent.md` を除外 | Phase 4 移行後 | `.agent.md` ファイルが結果に含まれない |
| 不変条件 | `supports_scope(Instruction, Personal)` | Phase 4 移行後 | `false` |
| 不変条件 | `supports_scope(Instruction, Project)` | Phase 4 移行後 | `true` |

**テスト実行コマンド**

```bash
# 全テスト
cargo test

# layout モジュールのみ
cargo test target::layout

# 特定ターゲットのテスト
cargo test target::env::antigravity
cargo test target::env::cursor
```

### 手動検証

1. `cargo build` でコンパイルエラーなし（各 Phase 終了時）
2. `cargo test` で全テスト green（各 Phase 終了時）
3. `cargo clippy` で新規 warning なし
4. Phase 2 完了後: `cargo run -- list` / `cargo run -- info <plugin>` で実際の出力に変化なし

---

## Definition of Done

以下をすべて満たした時点で本機能の実装完了とする。

- [ ] すべてのタスク（tasks.md）が ■ になっている
- [ ] `src/target/layout/` モジュールが追加され、`derive_*` ヘルパが実装されている
- [ ] 5 ターゲット（Antigravity / Gemini CLI / Codex / Copilot / Cursor）の `list_placed` / `placement_location` 骨格が共通ヘルパ経由になっている
- [ ] 各 env ファイルに残るのは「`LAYOUT` 定数 + 振る舞いフック override」のみ（`can_place` / `base_dir` / `filter_component` が各 env から消えている）
- [ ] `Target::supports_scope` のダミー `PlacementContext("test")` プロービングが廃止されている
- [ ] `placed_common::list_instruction` のダミー `origin("test")` 使用が廃止されている
- [ ] `cargo test` で全テスト（既存 1888 行 + 新規）が green
- [ ] 既存の `placement_location` パス・`list_placed` 結果・`supports_scope` 判定に変化なし（振る舞い不変）
- [ ] `cargo clippy` で新規 warning なし
