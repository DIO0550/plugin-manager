# 技術リファレンス: target/env 宣言的ケイパビリティ集約

> この文書は、implementation-plan.md に登場するすべての技術を初学者向けに解説するコンパニオンドキュメントです。
> 実装中に「この概念って何だっけ？」と詰まったときに辞書として引いてください。

---

## 概要

このリファクタリングは、5 つの AI ツール環境（Antigravity / Gemini CLI / Codex / Copilot / Cursor）にわたって **コピー&ペーストされていた配置ロジック** を、一つの宣言的データ構造 `TargetLayout` に集約します。

現在のコードはこんな状態です：各環境ファイルに `base_dir`・`can_place`・`filter_component`・`list_placed` がほぼ同じ形で何度も書かれており、「どの環境が何をサポートするか」を 2 か所で管理しているため、片方を更新し忘れると動作が壊れます。

新しい設計では、各環境の **差分（どのディレクトリを使う？どのコンポーネントをサポートする？）** を `TargetLayout` という定数に書き下ろし、共通ロジックは `derive_placement_location` 等のヘルパが担います。新しいターゲット（例: Claude Code）を追加するときも、データを宣言するだけで完結します。

```
【全体フロー】

  環境ファイル (antigravity.rs など)
      │
      ├── LAYOUT 定数（TargetLayout）─────────────────┐
      │       ↓                                       │
      │   components: [ComponentCapability, ...]       │
      │       ↓                                       │
      │   .kind / .scopes / .placement / .discover    │
      │                                               │
      └── impl Target for AntigravityTarget            │
              ├── supports_scope()   ←──── derive_supports_scope(&LAYOUT, ...)
              ├── placement_location() ←── derive_placement_location(&LAYOUT, ...)
              ├── supported_components() ← derive_supported_components(&LAYOUT)
              └── list_placed()     ←──── derive_list_placed(&LAYOUT, ...)

  導出ヘルパ (layout/derive.rs) が LAYOUT を読んで計算し、
  各環境の impl は LAYOUT を渡して委譲するだけになる。
```

---

## Rust の `&'static` ライフタイム

### これは何か

Rust では、値の「寿命（ライフタイム）」をコンパイラが追跡します。`&'static str` は「プログラムが起動してから終了するまで存在する文字列の参照」を意味します。

### この実装での使い方

`TargetLayout` は各環境ごとの定数として定義されます：

```rust
// src/target/env/antigravity.rs
static LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Antigravity,
    display_name: "Antigravity",
    base: BasePathLayout {
        personal: BasePathSpec::Nested { parent: ".gemini", child: "antigravity" },
        project:  BasePathSpec::Subdir(".agent"),
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
```

`components: &[...]` の `&` は参照で、`&'static [...]` は「プログラム実行中ずっと有効なスライスへの参照」です。定数（`static`）はプログラムの静的領域に配置されるため、このライフタイムが成立します。

> **なぜ静的参照？**
> `TargetLayout` を毎回 `new()` でヒープ確保するとコストがかかります。配置ルールは実行中に変わらない固定データなので、静的定数にして参照として渡すのが最適です。

---

## Rust の enum と パターンマッチング

### これは何か

Rust の `enum` は、C/Java の列挙型より強力で、バリアント（種類）ごとに **異なるデータを持てます**。これを「代数的データ型」または「直和型」と呼びます。

```rust
// PlacementRule の例
pub enum PlacementRule {
    ComponentDir { subdir: &'static str, naming: NamingPolicy },
    ComponentFile { subdir: &'static str, suffix: &'static str, naming: NamingPolicy },
    FixedAtBase { filename: &'static str },
    InstructionFile { filename: &'static str, project_location: InstructionProjectLocation },
}
```

`ComponentDir` バリアントは `subdir` と `naming` を持ち、`FixedAtBase` は `filename` だけを持ちます。

### パターンマッチング

`match` 式でバリアントごとに処理を分岐します：

```rust
fn resolve_placement(rule: &PlacementRule, base: &Path, name: &str) -> PlacementLocation {
    match rule {
        PlacementRule::ComponentDir { subdir, naming } => {
            // subdir 配下に name ディレクトリを作る
            PlacementLocation::Dir(base.join(subdir).join(name))
        }
        PlacementRule::ComponentFile { subdir, suffix, .. } => {
            // subdir 配下に name + suffix のファイルを作る
            PlacementLocation::File(base.join(subdir).join(format!("{name}{suffix}")))
        }
        PlacementRule::FixedAtBase { filename } => {
            PlacementLocation::File(base.join(filename))
        }
        PlacementRule::InstructionFile { filename, project_location } => {
            // project_location に応じてパスを変える
            todo!()
        }
    }
}
```

`match` はすべてのバリアントを網羅しなければコンパイルエラーになります。これにより新しいバリアントを追加したときに「対応し忘れ」をコンパイラが検出してくれます。

---

## TargetLayout データモデル

### TargetLayout 全体像

```
TargetLayout
├── kind: TargetKind              ─ どの環境か（Antigravity / Codex / ...）
├── display_name: &'static str    ─ 表示名（ログ用）
├── base: BasePathLayout          ─ ベースディレクトリの決め方
└── components: &'static [ComponentCapability]  ─ サポートするコンポーネント一覧
```

### BasePathLayout と BasePathSpec

「ベースディレクトリ」は、コンポーネントを配置するルートとなるディレクトリです。

```
BasePathLayout
├── personal: BasePathSpec  ─ Personal スコープ（~/ 以下）のベース
└── project:  BasePathSpec  ─ Project スコープ（プロジェクトルート以下）のベース

BasePathSpec
├── Subdir(".codex")         ─ ホームまたはプロジェクトルート直下のサブディレクトリ
└── Nested { parent: ".gemini", child: "antigravity" }
                              ─ 2段階のネスト（~/.gemini/antigravity/）
```

例：Antigravity の Personal スコープは `~/.gemini/antigravity/`、これは `Nested { parent: ".gemini", child: "antigravity" }` で表現されます。

### ScopeSet

`ScopeSet` は「どのスコープをサポートするか」を表します：

```rust
pub enum ScopeSet {
    Both,         // Personal / Project 両方
    PersonalOnly, // Personal のみ
    ProjectOnly,  // Project のみ
}

impl ScopeSet {
    pub fn contains(&self, scope: Scope) -> bool {
        match (self, scope) {
            (ScopeSet::Both, _)                        => true,
            (ScopeSet::PersonalOnly, Scope::Personal)  => true,
            (ScopeSet::ProjectOnly,  Scope::Project)   => true,
            _                                          => false,
        }
    }
}
```

**使用例**: Copilot の Skill は Project スコープのみ（個人設定ディレクトリがない）→ `ScopeSet::ProjectOnly`

### NamingPolicy

コンポーネントをどの名前でディレクトリ/ファイルに配置するかを決めます：

```
NamingPolicy
├── FlattenedName     ─ context.name() を使う（多くのターゲット）
│                       例: "my-plugin_skill-name" → skills/my-plugin_skill-name/
└── OriginalNameRequired  ─ context.original_name() が必須（Cursor Skill）
                           例: original_name = "skill-name" → skills/skill-name/
                           original_name が None/空文字 → 配置を拒否（None 返却）
```

> **なぜ Cursor だけ `OriginalNameRequired`？**
> Cursor のスキル一覧 UI はディレクトリ名をそのままスキル名として表示します。プラグイン名でフラット化した名前（`my-plugin_skill-name`）ではなく、元の名前（`skill-name`）が必要なためです。

---

## 導出ヘルパ（derive_* 関数群）

### derive_supports_scope

```rust
pub fn derive_supports_scope(
    layout: &TargetLayout,
    kind: ComponentKind,
    scope: Scope,
) -> bool
```

`layout.components` を線形探索し、`kind` が一致する `ComponentCapability` を見つけて `scopes.contains(scope)` を返します。見つからなければ `false`。

**なぜこれが重要か**: 現在の実装はダミーの `PlacementContext("test")` を作って `placement_location` が `Some` かどうかで判定する「プロービング hack」を使っています。これはサポート判定のために配置計算を走らせるという、意図が不明瞭なコードです。新実装は `capabilities` 表を直接参照するだけです。

```
【現在の hack】                    【新しい実装】

supports_scope(Skill, Personal)    derive_supports_scope(&LAYOUT, Skill, Personal)
  ↓                                  ↓
PlacementContext::dummy("test")    LAYOUT.components
  ↓                                  .iter()
placement_location(dummy)            .find(|c| c.kind == Skill)
  ↓                                  .map(|c| c.scopes.contains(Personal))
is_some() → true/false               .unwrap_or(false)
   ↑ hack!                               ↑ 意図が明確
```

### derive_placement_location

```rust
pub fn derive_placement_location(
    layout: &TargetLayout,
    ctx: &PlacementContext,
) -> Option<PlacementLocation>
```

1. `ctx.kind` に対応する `ComponentCapability` を探す
2. `capability.scopes.contains(ctx.scope())` が false → `None`
3. `NamingPolicy` に従って名前を解決（`OriginalNameRequired` なら `original_name` が必須）
4. `PlacementRule` に従ってパスを計算

```
PlacementContext (何を・どのスコープで・どのプロジェクトに配置するか)
    │
    ▼
capability 探索 → ScopeSet チェック → None (scope 外)
    │ scope OK
    ▼
NamingPolicy 解決 → None (OriginalNameRequired + name なし)
    │ name OK
    ▼
PlacementRule 解決 → Some(PlacementLocation::Dir/File)
```

### derive_list_placed

```rust
pub fn derive_list_placed(
    layout: &TargetLayout,
    kind: ComponentKind,
    scope: Scope,
    root: &Path,
) -> Result<Vec<String>>
```

1. `derive_supports_scope` で scope チェック → false なら `Ok(vec![])`
2. `DiscoverRule` が `InstructionExists` の場合：ファイルの存在確認で直接リスト化
3. それ以外：`scan_components(dir)` でエントリ一覧を取得し、`apply_discover` でフィルタ

**`DiscoverRule` のバリアントと用途**:

| バリアント | 動作 | 代表的な用途 |
|-----------|------|-------------|
| `SkillManifestDir` | `SKILL.md` を含むディレクトリのみ選択 | 全ターゲットの Skill |
| `SuffixFile { ".agent.md" }` | 指定サフィックスのファイルを選択 | Codex Agent |
| `PlainMarkdownFile` | `.md` だが `.agent.md` / `.prompt.md` を除外 | Cursor Agent |
| `ExactFile { "hooks.json" }` | 特定ファイル名のみ選択 | Codex/Cursor Hook |
| `JsonSuffixFile` | `.json` ファイルを選択 | Copilot Hook |
| `InstructionExists` | ファイル存在確認のみ（名前非依存） | 全ターゲットの Instruction |

---

## Rust の Trait（トレイト）とデフォルト実装

### これは何か

`trait` は「このメソッドを持っていること」を保証するインターフェースです。各型（struct）がそのトレイトを実装（`impl Trait for MyType`）することで、異なる型を統一的に扱えます。

### `Target` trait との関係

```rust
pub trait Target {
    // 各環境が必ず実装するメソッド
    fn kind(&self) -> TargetKind;

    // デフォルト実装があるメソッド（overrideも可）
    fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        // デフォルト: ダミープロービング (現在の hack)
        // Phase 5 後: derive_supports_scope(self.layout(), kind, scope)
        todo!()
    }
}
```

**段階移行との関係**: Phase 5 まで各環境が `supports_scope` を override します。Phase 5 で `Target::supports_scope` のデフォルト実装を `derive_supports_scope` に置き換えると、override は削除できます。

### FakeTarget（テスト用）

`src/sync_test.rs` 等で使われるテスト用の最小 Target 実装です：

```rust
struct FakeTarget;
impl Target for FakeTarget {
    fn kind(&self) -> TargetKind { TargetKind::Codex }
    // supports_scope 等はデフォルト実装を使う
}
```

`Target` trait に `fn layout() -> &'static TargetLayout` を追加する場合でも、パニックするデフォルト実装を提供しておけば `FakeTarget` の変更は不要です。

---

## Rust 2018 エディションのモジュールシステム

### 旧スタイル（使わない）

```
src/
└── target/
    └── mod.rs  ← サブモジュールを定義する場所
```

### 新スタイル（推奨）

```
src/
├── target.rs       ← mod target の定義ファイル
└── target/
    ├── layout.rs   ← target::layout モジュール
    └── layout/
        ├── model.rs    ← target::layout::model
        └── derive.rs   ← target::layout::derive
```

`src/target.rs` に `mod layout;` と書くだけで `src/target/layout.rs` が読み込まれます。

### このリファクタリングでの追加

```
src/target.rs    ← ここに mod layout; を追加
src/target/
└── layout.rs    ← [NEW] pub use model::*; pub use derive::*;
src/target/layout/
├── model.rs     ← [NEW] TargetLayout, ScopeSet, ...
├── model_test.rs← [NEW] model の単体テスト
├── derive.rs    ← [NEW] derive_* 関数
└── derive_test.rs ← [NEW] derive の単体テスト（TempDir 使用）
```

---

## TDD（テスト駆動開発）

### Red → Green → Refactor サイクル

このリファクタリングは TDD で進めます：

```
  ┌── 1. Red ──────────────────┐
  │  まず失敗するテストを書く   │
  │  (まだ実装がないので panic) │
  └────────────┬───────────────┘
               ↓
  ┌── 2. Green ─────────────────┐
  │  テストをパスさせる最小実装  │
  │  完璧でなくてOK              │
  └────────────┬────────────────┘
               ↓
  ┌── 3. Refactor ──────────────┐
  │  テストが通る状態を維持しながら│
  │  コードを整理・改善           │
  └─────────────────────────────┘
```

**具体例**: Phase 1 の `derive_supports_scope`:

```rust
// derive_test.rs (RED: まず書く)
#[test]
fn test_derive_supports_scope_both_personal() {
    // まだ derive_supports_scope が存在しないのでコンパイルエラー → Red
    assert!(derive_supports_scope(&TEST_LAYOUT, ComponentKind::Skill, Scope::Personal));
}
```

```rust
// derive.rs (GREEN: 最小実装)
pub fn derive_supports_scope(layout: &TargetLayout, kind: ComponentKind, scope: Scope) -> bool {
    layout.components
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| c.scopes.contains(scope))
        .unwrap_or(false)
}
// → テストが green に
```

### テストファイルの分離（CLAUDE.md 規約）

このプロジェクトでは本体コードとテストコードを **別ファイルに分離**します：

```
src/target/layout/
├── model.rs       ← 本体
├── model_test.rs  ← テスト（#[cfg(test)] mod tests を含む）
├── derive.rs      ← 本体
└── derive_test.rs ← テスト
```

`model.rs` 内でテストファイルを読み込む：

```rust
// model.rs の末尾
#[cfg(test)]
#[path = "model_test.rs"]
mod tests;
```

---

## tempfile::TempDir（テスト用一時ディレクトリ）

### これは何か

`derive_list_placed` のように、実際のファイルシステムを読むコードをテストするために、テストごとに独立した一時ディレクトリを作成するライブラリです。

### 基本的な使い方

```rust
use tempfile::TempDir;

#[test]
fn test_list_placed_with_skill() {
    let dir = TempDir::new().unwrap();  // OS の一時ディレクトリ以下に作成
    let root = dir.path();              // &Path を取得

    // テスト用ファイルを作成
    let skill_dir = root.join(".test").join("skills").join("my-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---").unwrap();

    // derive_list_placed を呼ぶ
    let result = derive_list_placed(&TEST_LAYOUT, ComponentKind::Skill, Scope::Project, root);
    assert_eq!(result.unwrap(), vec!["my-skill"]);

    // dir がスコープ外に出ると自動削除される
}
```

> **`TempDir` が削除されるタイミング**: Rust の `Drop` トレイトにより、`dir` 変数がスコープ外に出ると自動的にディレクトリごと削除されます。テスト間でファイルが残って干渉することがありません。

---

## 振る舞い不変テスト（リグレッションテスト）

### なぜ必要か

このリファクタリングは「コードを書き換えても外から見た動作は変わらない」ことを保証しなければなりません。Phase 2（Antigravity 移行）後、移行前と **まったく同じ入力に対してまったく同じ出力を返す**ことをテストで確認します。

### パターン

```rust
// antigravity_test.rs（移行後リグレッション）
#[test]
fn test_placement_personal_skill() {
    let target = AntigravityTarget::new();
    let ctx = PlacementContext::new(
        ComponentRef::new(ComponentKind::Skill, "my-skill"),
        Scope::Personal,
        Path::new("/project"),
    );
    // 移行前後で同じパスが返ることを確認
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.path(),
        Path::new("/home/testuser/.gemini/antigravity/skills/my-skill"),
    );
}
```

> **「振る舞い不変」の意味**: パス文字列が 1 文字でも変わればテストが落ちます。これにより「変えたつもりがなかったのに変わっていた」バグを検出できます。

---

## scan_components 関数（既存コードの再利用）

`derive_list_placed` の内部で使われる、すでに存在するヘルパ関数です：

```rust
// src/target/core/scanner.rs（既存）
pub fn scan_components(dir: &Path) -> Result<Vec<DirEntry>>
```

`dir` が存在しない場合は空ベクターを返します（エラーにしない）。`DiscoverRule` フィルタリングの前に生のエントリ一覧を取得するために使います。

---

## 用語集

| 用語 | 説明 |
|------|------|
| **TargetLayout** | 1 つの環境（ターゲット）の配置ルール全体を表す静的定数 |
| **ComponentCapability** | 1 つのコンポーネント種別のサポートスコープ・配置ルール・検出ルールのセット |
| **ScopeSet** | `Both` / `PersonalOnly` / `ProjectOnly` のいずれかで、サポートするスコープを宣言 |
| **PlacementRule** | コンポーネントをどのパスに配置するかのルール（ディレクトリ or ファイル名など） |
| **DiscoverRule** | 配置済みコンポーネントをどうリストアップするかのルール |
| **NamingPolicy** | 配置パスのディレクトリ/ファイル名をどう決めるか（フラット化名 vs 元名） |
| **InstructionProjectLocation** | Instruction ファイルをプロジェクトルート直下に置くか（`ProjectRoot`）ベース配下に置くか（`UnderBase`）|
| **derive_* 関数** | `TargetLayout` を受け取り、サポート判定・配置パス計算・一覧取得を行うヘルパ |
| **振る舞い不変** | リファクタリング前後で外部から見た入出力が変わらないという保証 |
| **TDD** | テスト駆動開発。Red→Green→Refactor のサイクルでコードを書く手法 |
| **ダミープロービング hack** | `supports_scope` の判定のために `PlacementContext("test")` を作って `placement_location` を呼ぶ現行の workaround |
| **`&'static`** | プログラムの起動から終了まで有効な参照。定数は自動的にこのライフタイムを持つ |
| **Personal スコープ** | `~/.codex/` 等、ユーザーのホームディレクトリ以下に配置されるスコープ |
| **Project スコープ** | `.codex/` 等、プロジェクトルート以下に配置されるスコープ |
| **FakeTarget** | テスト用の最小 Target 実装。sync テスト等で使われる |
| **scan_components** | 既存のディレクトリスキャナ。derive_list_placed 内部で再利用される |
| **TempDir** | テスト用一時ディレクトリ。スコープ外に出ると自動削除される |
| **BL-005** | 実装計画内のターゲット×コンポーネント×スコープのサポートマトリクス |
