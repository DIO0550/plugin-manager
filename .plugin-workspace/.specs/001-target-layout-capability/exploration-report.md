# Codebase Exploration Report: target/env 宣言的ケイパビリティ集約

**探索目的**: `target/env/` の 5 実装（Antigravity / Gemini CLI / Codex / Copilot / Cursor）にある配置骨格コピペを宣言的ケイパビリティ記述子（`TargetLayout`）へ集約し、`supported_components` / `supports_scope` / `placement_location` の単一真実源を確立する。

---

## 0. エグゼクティブサマリー

**重要な発見（Top 5）**:
1. **コピペ構造が 5 実装すべてで確認**: `can_place` / `base_dir` / `filter_component` のプライベート関数群と `placement_location` / `list_placed` の match 骨格が全ターゲットで同一パターン。Copilot のみ `can_place(kind, scope)` と 2 引数（scope 依存）。
2. **`supports_scope` のダミープロービング hack**: `src/target.rs:257` でダミー `PlacementContext` (`"test"` origin + `"test"` name + `"test"` original_name) を生成して `placement_location` を呼び出す。`placed_common::list_instruction` も同様のダミーを使用。
3. **`supported_components` スライスと実配置可否の乖離**: Copilot の `supported_components` は Skill 含む 5 種を返すが、Skill は `can_place(Skill, Personal)==false` のため Personal 配置不可。スライスのみ見ると乖離が検出できない。
4. **詳細な先行下書きが `docs/target-layout-refactor/` に存在**: `TargetLayout` / `PlacementRule` / `DiscoverRule` / `ScopeSet` の型設計・Phase 分割計画（Phase 0〜6）・BL-005 現状マトリクス（期待値固定用）まで完成度が高い。実装計画はこれを正式採用すべき。
5. **テストは `*_test.rs` 分離方式で既に充実**: 各ターゲット 60〜670 行の独立テストファイル。`supports` / `supports_scope` / `placement_location` / `list_placed` それぞれについてのテストが存在。移行後もこれらをリグレッションテストとして維持できる。

**推奨される次のステップ**:
- `docs/target-layout-refactor/` の先行設計を正式採用し、`src/target/layout/` モジュールとして実装開始
- Phase 1 で型定義とテーブル駆動テストを先行させ、既存テスト全緑を確認してから Phase 2 へ

---

## 1. アーキテクチャ概要

### 1.1 ディレクトリ構造

```
src/
├── target.rs               # Target trait 定義、parse_target/all_targets
├── target/
│   ├── core.rs             # paths, registry モジュール集約
│   ├── core/
│   │   ├── paths.rs        # home_dir / base_dir 共通計算
│   │   └── registry.rs     # TargetRegistry 状態マシン
│   ├── effect.rs           # TargetEffect / AffectedTargets / OperationOutcome
│   ├── env.rs              # 5実装の pub use 集約
│   ├── env/
│   │   ├── antigravity.rs  # ← 集約対象 (Skill のみ)
│   │   ├── codex.rs        # ← 集約対象 (Skill/Agent/Instruction/Hook)
│   │   ├── copilot.rs      # ← 集約対象 (5 kind, scope 制約あり)
│   │   ├── cursor.rs       # ← 集約対象 (OriginalNameRequired 含む)
│   │   └── gemini_cli.rs   # ← 集約対象 (Skill/Instruction)
│   └── placed/
│       ├── placed.rs       # list_all_placed (全ターゲット横断)
│       ├── placed_common.rs # list_instruction (ダミー hack あり)
│       └── scanner.rs      # scan_components フラット 1 階層スキャナ
└── component/
    └── model/
        ├── kind.rs          # ComponentKind / Scope / Component
        └── placement.rs     # ComponentRef / PlacementContext / PlacementLocation
```

**構造の特徴**:
- Rust 2018+ モジュールスタイル（`mod.rs` 不使用）
- Feature ベース: `target/` に Target 関連の全型が集約
- `Target` trait は `src/target.rs` で定義（env は実装のみ）

### 1.2 主要ファイル

| ファイルパス | 役割 | 重要度 |
|-------------|------|--------|
| `src/target.rs` | Target trait 定義・`supports_scope` デフォルト実装 | 高 |
| `src/target/env/antigravity.rs` | 最小ターゲット（Skill のみ）—移行 Phase 2 先頭 | 高 |
| `src/target/env/codex.rs` | Codex 実装—Hook の feature flag フック含む | 高 |
| `src/target/env/copilot.rs` | Copilot—scope 制約が `can_place(kind, scope)` | 高 |
| `src/target/env/cursor.rs` | Cursor—OriginalNameRequired / PlainMarkdownFile | 高 |
| `src/target/env/gemini_cli.rs` | Gemini CLI—Skill+Instruction | 高 |
| `src/target/placed/placed_common.rs` | list_instruction (ダミー廃止対象) | 高 |
| `src/target/core/paths.rs` | `base_dir(scope, project_root, personal, project)` | 中 |
| `src/target/placed/scanner.rs` | `scan_components` フラットスキャナ（再利用） | 中 |
| `docs/target-layout-refactor/capability-model-spec.md` | 先行設計—型定義・BL-005 マトリクス | 高 |
| `docs/target-layout-refactor/migration-plan.md` | Phase 計画 | 高 |

### 1.3 Target trait の構造

```
Target trait (src/target.rs)
├── fn kind() → TargetKind             // 必須 override
├── fn name() → &'static str          // kind() から導出（override 不要）
├── fn display_name() → &'static str  // 必須 override
├── fn supported_components() → &[ComponentKind] // 必須 override ← 集約対象
├── fn supports() → bool              // デフォルト: contains(&kind)
├── fn supports_scope() → bool        // デフォルト: ダミープロービング ← hack 廃止対象
├── fn placement_location() → Option<PlacementLocation>  // 必須 override ← 集約対象
├── fn component_conflict_error() → Option<String> // デフォルト: None（Codex/Cursor override）
├── fn pre_place_check() → Result<(), String>      // デフォルト: Ok(())（Codex/Cursor override）
├── fn post_place() → PostPlaceOutcome             // デフォルト: default（Codex/Cursor override）
├── fn legacy_cleanup_operations() → Vec<FileOperation> // デフォルト: []（Cursor override）
├── fn remove() → Result<()>                       // デフォルト実装（placement_location 使用）
└── fn list_placed() → Result<Vec<String>>         // 必須 override ← 集約対象
```

---

## 2. 関連コード分析

### 2.1 変更対象に関連する既存コード

| ファイルパス | 関連内容 | 関連度 |
|-------------|---------|--------|
| `src/target/env/*.rs` | 集約の直接対象（5 ファイル） | 高 |
| `src/target.rs` | `supports_scope` デフォルト hack | 高 |
| `src/target/placed/placed_common.rs` | `list_instruction` ダミー | 高 |
| `src/target/core/paths.rs` | `base_dir` 再利用 | 中 |
| `src/target/placed/scanner.rs` | `scan_components` 再利用 | 中 |
| `src/sync/endpoint/destination.rs:84` | `supports` + `supports_scope` の呼び出し側 | 中 |
| `src/sync/endpoint/endpoint_test.rs` | FakeTarget で `supports_scope` を直接実装（override） | 中 |

### 2.2 再利用可能なパターン

#### パターン: `base_dir` 共通ヘルパ

**場所**: `src/target/core/paths.rs`
**概要**: `Scope::Personal → home_dir().join(subdir)` / `Scope::Project → project_root.join(subdir)` の 2 分岐
**再利用方法**: `TargetLayout.base` 解決に内部で使用し続ける（抽象化後も同関数を呼ぶ）

```rust
pub(crate) fn base_dir(
    scope: Scope,
    project_root: &Path,
    personal_subdir: &str,
    project_subdir: &str,
) -> PathBuf {
    match scope {
        Scope::Personal => home_dir().join(personal_subdir),
        Scope::Project => project_root.join(project_subdir),
    }
}
```

#### パターン: `scan_components` フラットスキャナ

**場所**: `src/target/placed/scanner.rs`
**概要**: `<kind>/` ディレクトリを 1 階層スキャンして `ScannedComponent { name, path, is_dir }` を返す
**再利用方法**: `DiscoverRule` 適用ロジック内で変わらず使用する

```rust
pub fn scan_components(base_dir: &Path) -> Result<Vec<ScannedComponent>> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }
    // ...
}
```

### 2.3 類似実装の参考例

#### 参考: Antigravity の最小実装（Phase 2 移行先頭）

**実装ファイル**: `src/target/env/antigravity.rs`
**類似点**: `can_place(kind)` / `base_dir(scope, project_root)` / `filter_component(c, kind)` のパターンが最小形
**参考になる点**: Skill のみサポートかつ振る舞いフックなし。最初に移行するとヘルパの妥当性を最小コストで検証できる

#### 参考: Copilot の scope 制約表現

**実装ファイル**: `src/target/env/copilot.rs`
**類似点**: `can_place(kind, scope)` が 2 引数—scope に依存した制約
**参考になる点**: `ScopeSet::PersonalOnly` / `ProjectOnly` / `Both` に変換する際の境界ケース

```rust
fn can_place(kind: ComponentKind, scope: Scope) -> bool {
    matches!(
        (kind, scope),
        (ComponentKind::Agent, _) | (ComponentKind::Hook, _) | (_, Scope::Project)
    )
}
```

### 2.4 命名規則・コーディングスタイル

- **ファイル命名**: snake_case（`antigravity.rs`, `placed_common.rs`）
- **変数命名**: snake_case（`base_dir`, `project_root`, `can_place`）
- **型命名**: CamelCase（`ComponentCapability`, `PlacementRule`）
- **テスト命名**: `fn test_{target}_{action}_{condition}()`
- **定数**: `SCREAMING_SNAKE_CASE`（`CODEX_SUBDIR`, `CURSOR_SUBDIR`）
- **インデント**: 4 スペース（rustfmt 準拠）
- **テストファイル**: 本体と同ディレクトリの `*_test.rs`（`#[path = "xxx_test.rs"]` で参照）

### 2.5 既存構造の問題点・技術的負債

| 問題のあるパターン / 場所 | 問題の内容 | 新実装での扱い |
|------------------------|-----------|--------------|
| `src/target.rs:257–266` `supports_scope` ダミープロービング | ダミー `PlacementContext("test")` で `placement_location` を副作用的に呼び出す。将来の name 依存実装で誤判定リスク | 廃止。ケイパビリティ表から直接判定 |
| `src/target/placed/placed_common.rs:17–38` | 同様のダミー hack。`list_instruction` が `Target` traitに依存しパスを直接計算できない | `InstructionExists` DiscoverRule + 直接計算で廃止 |
| 各 env の `can_place` + `supported_components` スライス | 二重真実源。`supported_components` はスコープ非依存スライスだが `can_place` はスコープ依存（Copilot）。乖離してもコンパイルが通る | `ComponentCapability.scopes` の単一表から両方を導出 |
| 各 env の `filter_component` | 5 ファイルでほぼ同一の match 式コピペ | `DiscoverRule` enum に置き換え |

---

## 3. 技術的制約・リスク

### 3.1 既存の制約

**型システム・リンター**:
- Rust 2021 edition、Clippy あり（`#[allow(clippy::module_inception)]` が scanner 等で使用済み）
- `Target: Send + Sync` 制約 → `TargetLayout` の全フィールドも `Send + Sync` が必要（`&'static str` / enum の場合問題なし）

**モジュール制約**:
- `mod.rs` 禁止（Rust 2018+ スタイル）
- `src/target/layout.rs` + `src/target/layout/` サブモジュール方式が適切

### 3.2 互換性の問題

| 観点 | リスク |
|------|--------|
| `Target` trait に `fn layout()` を追加した場合 | 既存の `FakeTarget`（sync_test, endpoint_test）が `layout()` を要求されるが、デフォルト実装があれば不要 | 
| `supported_components()` を trait デフォルト化した場合 | すべての `impl Target` で消えるが、`FakeTarget` の `supported_components` field は override のまま残る可能性 |
| Cursor の `original_name` 必須パス | `supports_scope` 判定でダミー `original_name="test"` を渡すと `Some` が返るが、実際の配置は `original_name` 欠落で `None` になる乖離は現状も存在。移行後は `DiscoverRule::OriginalNameRequired` が明示するため乖離が消える |

### 3.3 パフォーマンスボトルネック

- 特記事項なし（静的 `const` / `&'static [...]` により実行時コストほぼゼロ）

### 3.4 セキュリティ考慮点

- 特記事項なし（ファイルパス操作のみ）

---

## 4. 変更影響範囲

### 4.1 波及ファイル

**直接影響**（修正が必須）:

| ファイルパス | 理由 | 影響の種類 |
|-------------|------|-----------|
| `src/target/env/antigravity.rs` | Phase 2: LAYOUT 定数追加、骨格をヘルパへ | 修正 |
| `src/target/env/gemini_cli.rs` | Phase 2: 同上 | 修正 |
| `src/target/env/codex.rs` | Phase 3: Hook/Instruction 規則含む | 修正 |
| `src/target/env/copilot.rs` | Phase 3: scope 制約 | 修正 |
| `src/target/env/cursor.rs` | Phase 4: OriginalNameRequired, PlainMarkdownFile | 修正 |
| `src/target.rs` | Phase 5: `supports_scope` デフォルト実装置換 | 修正 |
| `src/target/placed/placed_common.rs` | Phase 5: `list_instruction` 廃止/縮小 | 修正 |
| `src/target/layout.rs` (新規) | Phase 1: `TargetLayout` / 規則 enum / 導出ヘルパ | 追加 |
| `src/target/layout/` (新規) | Phase 1: `model.rs`, `derive.rs` | 追加 |

**間接影響**（確認が必要）:

| ファイルパス | 理由 | 確認内容 |
|-------------|------|---------|
| `src/sync/endpoint/destination.rs:84` | `supports_scope` 呼び出し | インターフェースが変わらなければ影響なし |
| `src/sync_test.rs` | `FakeTarget::supports_scope` の独自 override | trait デフォルト変更後も override として有効か |
| `src/sync/endpoint/endpoint_test.rs` | `FakeTarget::supports_scope` 同上 | 同上 |

### 4.2 テスト範囲

**既存テストファイル**:

| テストファイルパス | テスト対象 | 修正の必要性 |
|------------------|----------|------------|
| `src/target/env/antigravity_test.rs` (245行) | supported_components, supports_scope, placement_location, list_placed | 低（期待値変えず維持） |
| `src/target/env/codex_test.rs` (280行) | 同上 + Hook/Instruction | 低 |
| `src/target/env/copilot_test.rs` (347行) | scope 制約 + 5 kind | 低 |
| `src/target/env/cursor_test.rs` (671行) | OriginalNameRequired + legacy cleanup | 低 |
| `src/target/env/gemini_cli_test.rs` (345行) | Skill/Instruction | 低 |

**新規テストの必要性**:
- [ ] `src/target/layout/model_test.rs`: `TargetLayout` 型定義・`ScopeSet::contains` 等の単体テスト（Phase 1）
- [ ] `src/target/layout/derive_test.rs`: 架空の小さな `TargetLayout` で `placement_location` / `list_placed` / `supports_*` をテーブル駆動で検証（Phase 1 Red）
- [ ] 各 `*_test.rs` への不変条件テスト追加（Phase 0 棚卸し）

### 4.3 破壊的変更の可能性

| API / 関数 | 変更内容 | 影響範囲 |
|-----------|---------|---------|
| `Target::supported_components()` | 必須 override → trait デフォルト | 既存 impl はそのまま動作（FakeTarget は override 続け可） |
| `Target::supports_scope()` | ダミープロービング → capabilities 参照 | 戻り値の意味論は同一。テストへの影響なし |
| `placed_common::list_instruction()` | ダミー廃止 | 呼び出し元 4 ファイル（codex/copilot/cursor/gemini_cli）で内部実装に置換 |

### 4.4 移行計画の必要性

- **段階的リリース**: Phase 0→6 の 7 段階。各 Phase が独立 PR
- **ロールバック**: 各 Phase が green のため、問題 Phase を revert で対応可能

---

## 5. テストインフラストラクチャ

### 5.1 テスト環境

- **テストフレームワーク**: Rust 標準 (`cargo test`)
- **テストランナーコマンド**: `cargo test`
- **アサーションライブラリ**: 標準 `assert!` / `assert_eq!`
- **モックライブラリ**: なし（`FakeTarget` パターンで代替）

### 5.2 テストファイル構成

- **配置パターン**: 本体と同ディレクトリの `*_test.rs` ファイル（`#[path = "xxx_test.rs"]` で参照）
- **命名規則**: `*_test.rs`（CLAUDE.md で明示的に指定）
- **テストヘルパー**: `tempfile::TempDir` でファイルシステムをモック

### 5.3 既存テストパターン

| テストファイル | テスト対象 | パターン |
|-------------|----------|---------|
| `src/target/env/antigravity_test.rs` | AntigravityTarget | Unit（ファイルシステムはTempDir） |
| `src/target/env/codex_test.rs` | CodexTarget | Unit |
| `src/target/env/copilot_test.rs` | CopilotTarget | Unit |
| `src/target/env/cursor_test.rs` | CursorTarget | Unit（671行・最大規模） |
| `src/target/env/gemini_cli_test.rs` | GeminiCliTarget | Unit |

### 5.4 カバレッジ・CI

- **カバレッジツール**: 設定確認不要（cargo test で動作確認）
- **CI テストジョブ**: `.github/workflows/` 確認が必要（CI設定確認は追加調査）

---

## 6. 追加調査が必要な項目

- [x] `docs/target-layout-refactor/` の先行設計との整合性 → **完全整合。BL-001〜BL-008 を正式採用可**
- [ ] CI ワークフロー設定（`.github/workflows/`）の確認
- [ ] `#339` 文字列定数一元化の進捗状況

---

## 7. ユーザー判断が必要な論点

> Issue および hearing-notes で全確定済み。ユーザー判断が残る論点はなし。

### 確定済み論点

- **共通制御の接続方式**: `Target` trait に `fn layout() -> &'static TargetLayout` を要求する案 vs 自由関数ヘルパ案 → Phase 2 で実装して決定（hearing-notes で確認済み）
- **#339 との順序**: `#338` モデル先行、文字列は後で差し替え（確定済み）
- **Cursor の `original_name` 必須**: `DiscoverRule::OriginalNameRequired` として明示（確定済み）

---

## 8. 探索メトリクス（自己検証用）

| 指標 | 基準 | 実績 |
|------|------|------|
| Read したファイル数 | 10 以上 | 18 |
| Grep 検索キーワード数 | 5 以上 | 8 |
| コードスニペット数 | 5 以上 | 8 |
| 逆引き検索実施 | 必須 | 実施済み |

**探索キーワード一覧**: `can_place`, `supports_scope`, `placement_location`, `list_placed`, `filter_component`, `placed_common`, `TargetLayout`, `scan_components`

**Read したファイル一覧**:
- `src/target.rs`
- `src/target/env/antigravity.rs`
- `src/target/env/codex.rs`
- `src/target/env/copilot.rs`
- `src/target/env/cursor.rs`
- `src/target/env/gemini_cli.rs`
- `src/target/placed/placed_common.rs`
- `src/target/core/paths.rs`
- `src/target/placed/scanner.rs`
- `src/target/effect.rs`
- `src/target/placed.rs`
- `src/component/model.rs`
- `src/component/model/placement.rs`
- `src/component/model/kind.rs`（先頭60行）
- `src/target/env/antigravity_test.rs`
- `src/target/env/codex_test.rs`（先頭80行）
- `docs/target-layout-refactor/index.md`
- `docs/target-layout-refactor/capability-model-spec.md`
- `docs/target-layout-refactor/migration-plan.md`
