# ケイパビリティモデル仕様

> **機能**: [Target Layout 宣言的ケイパビリティ集約](./index.md)  
> **ステータス**: 計画（未実装）

## 概要

ターゲット間の差分を `TargetLayout` として宣言し、`Target` trait の配置・サポート系メソッドをそこから導出する。

## BL-001: 単一真実源

### 規則

1. **kind × scope の配置可否**は `TargetLayout.capabilities` のみが真実源とする。
2. `supported_components()` は capabilities から「いずれかの scope で配置可能」な kind の集合として導出する。
3. `supports_scope(kind, scope)` は capabilities の該当エントリ有無で判定する（`placement_location` プロービング禁止）。
4. `placement_location` が `Some` を返す iff `supports_scope(kind, scope)` が true（かつ Cursor Skill のように命名ポリシー上の追加前提を満たす場合）。
5. `list_placed` はサポート外なら空 `Vec`、サポート内ならレイアウトルールに従ってスキャンする。

### 禁止

- env 実装内の独自 `can_place` と `supported_components` スライスの二重定義
- `"test"` 原点のダミー `PlacementContext` によるサポート判定

## BL-002: `TargetLayout` データモデル

擬似コード（実装時の最終型名は実装 PR で調整可。`Layout` / `Capability` 接尾辞を推奨）:

```rust
pub struct TargetLayout {
    pub kind: TargetKind,
    pub display_name: &'static str,
    pub base: BasePathLayout,
    pub components: &'static [ComponentCapability],
}

pub struct BasePathLayout {
    /// Personal: `~/<subdir>` またはネスト
    pub personal: BasePathSpec,
    /// Project: `<project_root>/<subdir>`
    pub project: BasePathSpec,
}

pub enum BasePathSpec {
    /// 例: `.codex`, `.cursor`, `.gemini`, `.github`, `.agent`
    Subdir(&'static str),
    /// 例: Antigravity Personal = `.gemini/antigravity`
    Nested {
        parent: &'static str,
        child: &'static str,
    },
}

pub struct ComponentCapability {
    pub kind: ComponentKind,
    pub scopes: ScopeSet,
    pub placement: PlacementRule,
    pub discover: DiscoverRule,
}

pub enum ScopeSet {
    PersonalOnly,
    ProjectOnly,
    Both,
}

impl ScopeSet {
    pub fn contains(self, scope: Scope) -> bool { /* ... */ }
}
```

### 設計判断: trait object vs 静的定数

| 案 | 内容 | 採否 |
|----|------|------|
| A | 各ターゲットが `const LAYOUT: TargetLayout` を持ち、共通ヘルパに渡す | **採用**。既存 struct + `impl Target` を壊さない |
| B | `TargetKind` → `TargetLayout` の中央レジストリのみ | 却下（当面）。env ファイル凝集度を落とす |
| C | `Target` を enum に潰してデータ駆動化 | 却下。PR #388 のフック override と相性が悪い |

共通制御は次のいずれか（Phase 2 で決定、どちらでも仕様は同一）:

- `Target` trait のデフォルト実装が `fn layout(&self) -> &'static TargetLayout` を要求する
- または `layout::list_placed(layout, ...)` のような自由関数ヘルパを env から呼ぶ

## BL-003: 配置規則（`PlacementRule`）

```rust
pub enum PlacementRule {
    /// `base/<subdir>/<name>/` （Skill 典型）
    ComponentDir {
        subdir: &'static str,
        naming: NamingPolicy,
    },
    /// `base/<subdir>/<name><suffix>` （Agent / Command / 分散 Hook）
    ComponentFile {
        subdir: &'static str,
        suffix: &'static str,
        naming: NamingPolicy,
    },
    /// ベース直下の固定ファイル（Codex/Cursor `hooks.json`）
    FixedAtBase {
        filename: &'static str,
    },
    /// Instruction 専用: Personal は base 相対、Project は project root または base
    InstructionFile {
        filename: &'static str,
        project_location: InstructionProjectLocation,
    },
}

pub enum InstructionProjectLocation {
    /// `{project_root}/{filename}` — Codex / Cursor `AGENTS.md`, Gemini `GEMINI.md`
    ProjectRoot,
    /// `{project_base}/{filename}` — Copilot `copilot-instructions.md`
    UnderBase,
}

pub enum NamingPolicy {
    /// `context.name()`（フラット化名）
    FlattenedName,
    /// `context.original_name()` 必須。欠落または空なら配置不可（Cursor Skill）
    OriginalNameRequired,
}
```

### 導出: `placement_location`

```
IF NOT capability.scopes.contains(scope):
  return None
base = resolve_base(layout.base, scope, project_root)
match capability.placement:
  ComponentDir { subdir, naming } =>
    name = resolve_name(context, naming)?   // OriginalNameRequired で None になり得る
    Some(Dir(base/subdir/name))
  ComponentFile { subdir, suffix, naming } =>
    name = resolve_name(context, naming)?
    Some(File(base/subdir/{name}{suffix}))
  FixedAtBase { filename } =>
    Some(File(base/filename))
  InstructionFile { filename, project_location } =>
    path = match (scope, project_location)
      (Personal, _) => base/filename
      (Project, ProjectRoot) => project_root/filename
      (Project, UnderBase) => base/filename
    Some(File(path))
```

## BL-004: 発見規則（`DiscoverRule`）

`list_placed` がスキャン結果をコンポーネント名へ写す規則。

```rust
pub enum DiscoverRule {
    /// ディレクトリかつ直下に `SKILL.md` がある → ディレクトリ名
    SkillManifestDir,
    /// ファイルかつ `ends_with(suffix)` → suffix 除去名
    SuffixFile { suffix: &'static str },
    /// プレーン `.md`（`.agent.md` / `.prompt.md` 除外）→ `.md` 除去名（Cursor）
    PlainMarkdownFile,
    /// ファイル名が完全一致 → 固定エイリアス（`hooks.json` → `"hooks"`）
    ExactFile { filename: &'static str, listed_as: &'static str },
    /// `*.json` → `.json` 除去名（Copilot Hook）
    JsonSuffixFile,
    /// Instruction: 配置パス存在確認 → `filename` を 1 件返す
    InstructionExists,
}
```

### 導出: `list_placed`

```
IF NOT supports_scope(kind, scope):
  return Ok([])
capability = lookup(kind)
IF capability.discover == InstructionExists:
  path = placement_location(dummy_name_ok_for_instruction)?.as_path()
  return Ok(if path.exists() { vec![filename] } else { vec![] })
dir = scan_root(capability)  // ComponentDir/File の subdir、FixedAtBase なら base
return scan_components(dir).filter_map(apply discover)
```

### Instruction のダミー廃止

現状 `placed_common::list_instruction` と `supports_scope` は placeholder origin `"test"` で `placement_location` を呼ぶ。

移行後:

- サポート判定は capabilities 参照のみ
- Instruction の存在確認は `PlacementRule::InstructionFile` からパスを**直接**計算する（origin / component name 非依存であることをモデル上保証する）

`NamingPolicy` を持つ規則だけが `PlacementContext` の名前フィールドを必要とする。

## BL-005: 現状マトリクス（移行時の期待値固定）

仕様変更なしで再現すべき現状。テストのゴールデンデータとして扱う。

### ベースパス

| Target | Personal | Project |
|--------|----------|---------|
| Codex | `~/.codex` | `{root}/.codex` |
| Copilot | `~/.copilot` | `{root}/.github` |
| Antigravity | `~/.gemini/antigravity` | `{root}/.agent` |
| Gemini CLI | `~/.gemini` | `{root}/.gemini` |
| Cursor | `~/.cursor` | `{root}/.cursor` |

### kind × scope サポート

凡例: ✅ 両スコープ / P のみ Project / — 非サポート

| Kind | Codex | Copilot | Antigravity | Gemini | Cursor |
|------|-------|---------|-------------|--------|--------|
| Skill | ✅ | P | ✅ | ✅ | ✅ |
| Agent | ✅ | ✅ | — | — | ✅ |
| Command | — | P | — | — | ✅ |
| Instruction | ✅ | P | — | ✅ | P |
| Hook | ✅ | ✅ | — | — | ✅ |

注: Copilot の現行 `can_place` は `(Agent, _) | (Hook, _) | (_, Project)`。Skill / Command / Instruction は Project のみ。`supported_components` スライスには 5 kind すべてが載る（スコープ非依存）。移行後もこの意味論を維持する。

### 配置パス（サポート時）

| Target / Kind | パスパターン | Naming |
|---------------|--------------|--------|
| * / Skill | `{base}/skills/<name>/` | Flattened（Cursor のみ OriginalRequired） |
| Codex,Copilot / Agent | `{base}/agents/<name>.agent.md` | Flattened |
| Cursor / Agent | `{base}/agents/<name>.md` | Flattened |
| Copilot / Command | `{base}/prompts/<name>.prompt.md` | Flattened |
| Cursor / Command | `{base}/commands/<name>.md` | Flattened |
| Codex / Instruction Personal | `~/.codex/AGENTS.md` | n/a |
| Codex,Cursor / Instruction Project | `{root}/AGENTS.md` | n/a |
| Copilot / Instruction Project | `{base}/copilot-instructions.md` | n/a |
| Gemini / Instruction Personal | `~/.gemini/GEMINI.md` | n/a |
| Gemini / Instruction Project | `{root}/GEMINI.md` | n/a |
| Codex,Cursor / Hook | `{base}/hooks.json` | n/a（listed_as `"hooks"`） |
| Copilot / Hook | `{base}/hooks/<name>.json` | Flattened |

### 発見規則対応

| Target / Kind | DiscoverRule |
|---------------|--------------|
| 全 Skill | `SkillManifestDir` |
| Codex,Copilot Agent | `SuffixFile { ".agent.md" }` |
| Cursor Agent/Command | `PlainMarkdownFile` |
| Copilot Command | `SuffixFile { ".prompt.md" }` |
| Codex,Cursor Hook | `ExactFile { "hooks.json", "hooks" }` |
| Copilot Hook | `JsonSuffixFile` |
| Instruction 対応ターゲット | `InstructionExists` |

## BL-006: 共通化後に env 実装へ残すもの

各 `*Target` に残してよい責務:

1. `const LAYOUT: TargetLayout`（または `fn layout()`）
2. PR #388 系の振る舞い override（Codex / Cursor のみ現状）
3. Codex 固有の `config_toml_path` / feature flag 補助

共通ヘルパへ移すもの:

- `base_dir`
- `can_place`（削除し capabilities 参照へ）
- `filter_component`
- `placement_location` の match 骨格
- `list_placed` の骨格
- `supported_components` スライスの手書き

## BL-007: #339 との境界

| 関心事 | 担当 |
|--------|------|
| `"skills"` / `"SKILL.md"` / `".agent.md"` 等の**文字列定数**の単一定義 | #339 |
| kind × scope × どの規則を使うかの**組み合わせ表** | #338（本仕様） |
| Copilot の配置 subdir `"prompts"` vs 表示用 `plural()=="commands"` | #339 で概念分離、本仕様の `PlacementRule.subdir` が配置側を参照 |

推奨順序: **#338 のモデルを先に入れ、文字列は当面レイアウト定数内に置いてもよい**。#339 後に定数参照へ差し替える。逆順でもよいが、#339 だけで制御フロー重複は消えない。

## BL-008: モジュール配置

Feature ベース方針に従い、新規型は `target` feature 配下へ置く。

提案:

```
src/target/
├── layout.rs              # TargetLayout / 規則 enum / 導出ヘルパ
├── layout/
│   ├── model.rs           # データ型
│   ├── derive.rs          # placement_location / list_placed / supports_*
│   └── model_test.rs / derive_test.rs
└── env/
    └── *.rs               # LAYOUT 定数 + 振る舞いフック
```

`placed_common.rs` は Instruction 専用ヘルパを `layout::derive` に吸収したうえで縮小または削除する。
