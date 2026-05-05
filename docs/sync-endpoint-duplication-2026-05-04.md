# sync/endpoint 重複ロジック棚卸し (2026-05-04)

- **Issue**: [#260 [ISSUE-01] sync/endpoint 重複ロジック棚卸し](https://github.com/DIO0550/plugin-manager/issues/260)
- **親Issue**: #255 sync/endpoint の共通抽象化検討（enum Endpoint）
- **親Epic**: #254
- **棚卸し対象コミット SHA**: `540b7a0efac627c1f3e043cabd9939751f044540`（branch `main`）
- **作成日**: 2026-05-04
- **担当**: spec 019-sync-endpoint-duplication-inventory

---

## 1. サマリ

`SyncSource` / `SyncDestination` は既に `TargetBinding` (`src/sync/endpoint/binding.rs`) への委譲によって実装重複が解消されており、`pub(crate) enum Endpoint` (`src/sync/endpoint.rs`) も pre-staged で定義されているが production 未使用である。
#255 のトリガー条件A（実装重複 ≥ 3）・条件B（match-over-both 複数箇所）はいずれも未達。
**結論: 現状維持推奨。** match-over-both が production で必要になった時点で既存 `Endpoint` enum を活用して移行する。

---

## 2. 対象範囲と評価方法

### 2.1 対象ファイル

棚卸しの主対象（実装ロジック比較）:

- `src/sync/endpoint.rs` (138 行) — `Endpoint` enum とサブ親モジュール
- `src/sync/endpoint/source.rs` (77 行) — `SyncSource`
- `src/sync/endpoint/destination.rs` (90 行) — `SyncDestination`
- `src/sync/endpoint/binding.rs` (160 行) — `TargetBinding`（共有実装本体）

両型の併用箇所カウント対象（参照のみ。実装ロジック比較は対象外）:

- `src/sync.rs` — 呼び出し元エンジン
- `src/commands/deploy/sync.rs` — CLI 構築サイト

### 2.2 評価方法

1. **メソッドシグネチャ比較表**: `SyncSource` と `SyncDestination` の各メソッドを Duplicate / Similar / Unique の3分類で集計。
2. **両型併用箇所カウント**: `comm -12` で `SyncSource` と `SyncDestination` を同一ファイル内で参照しているファイルを列挙し、production / test に分けて件数を集計。
3. **トリガー条件判定**: ヒアリング Section 8 で採択された解釈（条件A=実装重複ベース、条件B=match-over-both のみで厳密判定）に従い、#255 の条件A・B の達成可否を判定する。

### 2.3 棚卸し対象外

- `src/sync/` 配下の `endpoint` 直下以外（`model/`, `sync_test.rs` 等）の重複は本タスクのスコープ外（別Issue対応）
- 呼び出し元（`src/sync.rs`, `src/commands/deploy/sync.rs`）の実装内容比較は対象外。両型併用箇所のカウント目的でのみ参照する
- `Cargo.toml` / `Cargo.lock` / プロダクトコード全般の変更は行わない

---

## 3. シンボル一覧

### 3.1 `SyncSource` (`src/sync/endpoint/source.rs`, 77 行)

| # | Item | Visibility | Signature | Responsibility |
|---|------|-----------|-----------|----------------|
| S-1 | `struct SyncSource` | `pub` | `pub struct SyncSource(TargetBinding);` | newtype: source endpoint |
| S-2 | `impl Debug for SyncSource` | `impl` | `fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result` | カスタム debug（target name, project_root） |
| S-3 | `SyncSource::new` | `pub` | `pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self>` | 本番 ctor; `TargetBinding::new` に委譲 |
| S-4 | `SyncSource::with_target` | `pub` | `pub fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self` | テスト用 ctor (DI) |
| S-5 | `SyncSource::name` | `pub` | `pub fn name(&self) -> &'static str` | target name passthrough |
| S-6 | `SyncSource::command_format` | `pub` | `pub fn command_format(&self) -> CommandFormat` | command format passthrough |
| S-7 | `SyncSource::placed_components` | `pub` | `pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>>` | 配置済みコンポーネント列挙 |
| S-8 | `SyncSource::path_for` | `pub` | `pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf>` | source 側パス解決 |

自由関数なし。`Debug` 以外の trait impl なし。関連型なし。

### 3.2 `SyncDestination` (`src/sync/endpoint/destination.rs`, 90 行)

| # | Item | Visibility | Signature | Responsibility |
|---|------|-----------|-----------|----------------|
| D-1 | `struct SyncDestination` | `pub` | `pub struct SyncDestination(TargetBinding);` | newtype: destination endpoint |
| D-2 | `impl Debug for SyncDestination` | `impl` | `fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result` | カスタム debug |
| D-3 | `SyncDestination::new` | `pub` | `pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self>` | 本番 ctor |
| D-4 | `SyncDestination::with_target` | `pub` | `pub fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self` | テスト用 ctor (DI) |
| D-5 | `SyncDestination::name` | `pub` | `pub fn name(&self) -> &'static str` | target name passthrough |
| D-6 | `SyncDestination::command_format` | `pub` | `pub fn command_format(&self) -> CommandFormat` | command format passthrough |
| D-7 | `SyncDestination::placed_components` | `pub` | `pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>>` | 配置済みコンポーネント列挙 |
| D-8 | `SyncDestination::path_for` | `pub` | `pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf>` | destination 側パス解決 |
| D-9 | `SyncDestination::supports` | `pub` | `pub fn supports(&self, placed_ref: &PlacedRef) -> bool` | **Destination 専用**: kind+scope サポートチェック |

### 3.3 `Endpoint` enum & helpers (`src/sync/endpoint.rs`, 138 行)

| # | Item | Visibility | Signature | Responsibility |
|---|------|-----------|-----------|----------------|
| E-1 | `enum Endpoint` | `pub(crate)` | `pub(crate) enum Endpoint { Source(SyncSource), Destination(SyncDestination) }` | バリアント aware dispatch（現状テスト専用） |
| E-2 | `Endpoint::as_source` | `pub(crate)` | `fn as_source(&self) -> Option<&SyncSource>` | バリアントアクセサ |
| E-3 | `Endpoint::as_destination` | `pub(crate)` | `fn as_destination(&self) -> Option<&SyncDestination>` | バリアントアクセサ |
| E-4 | `Endpoint::name` | `pub(crate)` | `fn name(&self) -> &'static str` | match による共通 dispatch |
| E-5 | `Endpoint::command_format` | `pub(crate)` | `fn command_format(&self) -> CommandFormat` | match による共通 dispatch |
| E-6 | `Endpoint::placed_components` | `pub(crate)` | `fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>>` | match による共通 dispatch |
| E-7 | `Endpoint::path_for` | `pub(crate)` | `fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf>` | match による共通 dispatch |
| E-8 | `parse_component_name` | `pub(super)` | `fn parse_component_name(name: &str) -> Result<(PluginOrigin, &str)>` | source/destination が `super::` で共有 |
| E-9 | `validate_flattened_name` | private | `fn validate_flattened_name(name: &str) -> Result<()>` | パスセグメント検証 helper |

### 3.4 `TargetBinding` (`src/sync/endpoint/binding.rs`, 160 行)

| # | Item | Visibility | Signature | Responsibility |
|---|------|-----------|-----------|----------------|
| B-1 | `struct TargetBinding` | `pub(super)` | `pub(super) struct TargetBinding { pub(super) target: Box<dyn Target>, pub(super) project_root: PathBuf }` | 共有実装本体 |
| B-2 | `impl Debug for TargetBinding` | `impl` | 標準 | Debug |
| B-3 | `TargetBinding::new` | `pub(super)` | `fn new(kind: TargetKind, project_root: &Path) -> Result<Self>` | `parse_target` で Target を解決 |
| B-4 | `TargetBinding::with_target` | `pub(super)` | `fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self` | DI ctor |
| B-5 | `TargetBinding::name` | `pub(super)` | `fn name(&self) -> &'static str` | `Target::name` 委譲 |
| B-6 | `TargetBinding::command_format` | `pub(super)` | `fn command_format(&self) -> CommandFormat` | `Target::command_format` 委譲 |
| B-7 | `TargetBinding::target` | `pub(super)` | `fn target(&self) -> &dyn Target` | trait object accessor（`SyncDestination::supports` で使用） |
| B-8 | `TargetBinding::placed_components` | `pub(super)` | `fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>>` | 実装本体: kinds×scopes でループ、`HashSet<PlacedRef>` で dedup、`target.list_placed` 呼び出し |
| B-9 | `TargetBinding::path_for` | `pub(super)` | `fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf>` | `resolve_path` をラップ |
| B-10 | `TargetBinding::target_kinds` | `pub(super)` | `fn target_kinds(&self, options: &SyncOptions) -> Vec<SyncableKind>` | `options.component_type` フィルタ |
| B-11 | `TargetBinding::target_scopes` | `pub(super)` | `fn target_scopes(&self, options: &SyncOptions) -> Vec<Scope>` | `options.scope` フィルタ |
| B-12 | `TargetBinding::resolve_path` | `pub(super)` | `fn resolve_path(&self, kind: ComponentKind, name: &str, scope: Scope) -> Result<PathBuf>` | `PlacementContext` 構築と `target.placement_location` 呼び出し |

---

## 4. メソッドシグネチャ比較表 (Source vs Destination)

| # | Source method | Destination method | Signature 同一? | Implementation 同一? | 分類 |
|---|---------------|--------------------|-----------------|----------------------|------|
| 1 | `pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self>` | 同 | ✅ | ✅ (`Self(TargetBinding::new(...))`) | **Duplicate** |
| 2 | `pub fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self` | 同 | ✅ | ✅ | **Duplicate** |
| 3 | `pub fn name(&self) -> &'static str` | 同 | ✅ | ✅ (`self.0.name()`) | **Duplicate** |
| 4 | `pub fn command_format(&self) -> CommandFormat` | 同 | ✅ | ✅ | **Duplicate** |
| 5 | `pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>>` | 同 | ✅ | ✅ | **Duplicate** |
| 6 | `pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf>` | 同 | ✅ | ✅（コメント文言のみ "source" / "destination" の差） | **Duplicate** |
| 7 | `impl Debug` | `impl Debug` | ✅ 構造的に | 構造体名のラベル文字列のみ差異 | **Similar** |
| 8 | — | `pub fn supports(&self, placed_ref: &PlacedRef) -> bool` | n/a | n/a | **Unique (Destination-only)** |

**集計:**

- **Duplicate: 6**（`new`, `with_target`, `name`, `command_format`, `placed_components`, `path_for`）
- **Similar: 1**（`Debug` impl — 構造体名のラベル文字列のみ必然的に差異）
- **Unique: 1**（`SyncDestination::supports`）

> **注釈**: 重複は newtype ファサードのレベルにのみ存在する。実装ロジックは `TargetBinding` (`src/sync/endpoint/binding.rs`) で単一化済み。ファサード重複の解消には (a) 既存 `Endpoint` enum で newtype を置き換える / (b) macro 化 のいずれかが必要となる。

---

## 5. 両方を扱うコード箇所 (Co-occurrence Sites)

### 5.1 ファイル別出現箇所

| File | SyncSource 行 | SyncDestination 行 | パターン | match-over-both? |
|------|---------------|--------------------|----------|------------------|
| `src/sync.rs` (production) | 28 (re-export), 48, 64, 172, 224, 257 | 28, 49, 65, 173, 225, 258 | 関数シグネチャ `(source: &SyncSource, dest: &SyncDestination, ...)` を `sync`, `sync_with_fs`, `execute_sync`, `execute_create`, `execute_update` で使用 | **No.** 別々のパラメータとして使用。`dest.supports(...)` と `source.path_for/dest.path_for` を逐次呼び出すのみ。 |
| `src/commands/deploy/sync.rs` (CLI) | 5 (import), 46 | 5 (import), 47 | 両者を構築して `sync::sync` に渡す | **No.** `let source = SyncSource::new(...)?; let dest = SyncDestination::new(...)?;` |
| `src/sync/endpoint.rs` (定義) | 15, 27, 31, 36, 37 | 14, 27, 32, 44, 45 | `Endpoint` enum 定義と `match self { Self::Source(s) => …, Self::Destination(d) => … }` パターン | **Yes** (6 match self ブロック: `as_source` / `as_destination` / `name` / `command_format` / `placed_components` / `path_for`。ただし production からは未呼び出し) |
| `src/sync/endpoint/binding.rs` (doc コメントのみ) | 3, 21 | 3, 21, 70 | doc コメント内の参照のみ | **No** |
| `src/sync/endpoint/endpoint_test.rs` (tests) | 9, 107, 108, 396, 397 | 9, 111, 112, 412, 413 | `Endpoint` dispatch と `SyncDestination::supports` のテスト | match は `Endpoint` enum のメソッド経由のみ |

### 5.2 パターン別カウント

**「両型を参照するファイル」の定義**: 同一ファイル内で `SyncSource` と `SyncDestination` の両方が grep ヒットするファイル（`comm -12` で集合の積を取った結果）。

- 両型を参照するファイル: **5 件**（production 4 / test 1）
  - production: `src/sync.rs`, `src/sync/endpoint.rs`, `src/sync/endpoint/binding.rs`（doc コメント参照のみ）, `src/commands/deploy/sync.rs`
  - test: `src/sync/endpoint/endpoint_test.rs`
- production の `(&SyncSource, &SyncDestination)` ペア引数関数: **5 件**（`src/sync.rs` の `sync`, `sync_with_fs`, `execute_sync`, `execute_create`, `execute_update`）
- **production の match-over-both 文: 0 件**
  - `Endpoint` enum 定義ファイル内の `match self` は **6 個**（`as_source`, `as_destination`, `name`, `command_format`, `placed_components`, `path_for`）あるが、production 呼び出しは 0 件のため判定対象外
- CLI 構築サイト: **1 件**（`src/commands/deploy/sync.rs:46-47`）

---

## 6. 定量評価結果

| 指標 | 値 |
|------|---|
| Source 総行数 | 77 |
| Destination 総行数 | 90 |
| Endpoint 総行数 | 138 |
| TargetBinding 総行数 | 160 |
| Duplicate メソッド数 | 6 |
| Similar メソッド数 | 1 |
| Unique メソッド数 | 1 |
| 両型併用ファイル数 (`comm -12`) | 5 (production 4 / test 1) |
| production match-over-both 数 | **0** |
| ペア引数関数数 | 5 |

---

## 7. トリガー条件判定 (#255)

### 7.1 条件A: 双方の実装に同等のメソッドが 3 つ以上重複

- **解釈方針**: 「実装重複で判定する」（ヒアリング Section 8 採択）。
- **観測**: シグネチャ重複は 6 メソッド（`new`, `with_target`, `name`, `command_format`, `placed_components`, `path_for`）あるが、実装本体は `TargetBinding` (`src/sync/endpoint/binding.rs`) で単一化されているため、実装重複としては **0**。
- **補足**: ファサード重複（`self.0.method()` への passthrough）はノイズと判定し、条件A の「重複メソッド」には含めない。
- **判定**: → **未達**

### 7.2 条件B: 双方をまとめて扱うコードが複数箇所

- **解釈方針**: 「match 文のみで厳密判定する」（ヒアリング Section 8 採択）。
- **観測**: production の match-over-both は **0 件**。
- **補足**:
  - `Endpoint` enum 内 match arm は production 未使用のためカウント対象外
  - `(&SyncSource, &SyncDestination)` ペア引数関数 5 件は本条件の判定には含めない（ペア引数は逐次呼び出しのみで match dispatch を要求しないため）
- **判定**: → **未達**

### 7.3 総合判定

条件A・B いずれも未達。**`enum Endpoint` 抽象化への移行トリガーは現時点で未発火**。

---

## 8. 推奨アクション

- **現状維持**（即時着手しない）。
- 既存の `pub(crate) enum Endpoint` (`src/sync/endpoint.rs`) は pre-staged 状態として温存し、production で match-over-both が必要になった時点で `SyncSource` / `SyncDestination` を `Endpoint::Source` / `Endpoint::Destination` に置き換える。
- **次回再評価のトリガー**:
  - (a) `(&SyncSource, &SyncDestination)` ペア引数関数のうち1つでも match dispatch を必要とする変更が入る
  - (b) 公開 API 維持の観点で newtype ファサードを廃止する選択肢が検討される

---

## 9. 付録: システム図と再現可能な検索コマンド

### 9.1 棚卸しドキュメントのセクション構成図

```
┌──────────────────────────────────────────────────────────────────┐
│ docs/sync-endpoint-duplication-2026-05-04.md                     │
└──────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
  ┌───────────┐         ┌───────────┐         ┌───────────┐
  │ §1 サマリ │         │ §2 対象範 │         │ §3 シンボ │
  │ (TL;DR)   │◀────────│   囲・方法│────────▶│   ル一覧  │
  └───────────┘         └───────────┘         └───────────┘
        ▲                                            │
        │                                            ▼
        │                                      ┌───────────┐
        │                                      │ §4 メソッ │
        │                                      │   ド比較表│
        │                                      └───────────┘
        │                                            │
        │                                            ▼
        │                                      ┌───────────┐
        │                                      │ §5 両方を │
        │                                      │   扱う箇所│
        │                                      └───────────┘
        │                                            │
        │                                            ▼
        │                                      ┌───────────┐
        │                                      │ §6 定量評 │
        │                                      │   価結果  │
        │                                      └───────────┘
        │                                            │
        │                                            ▼
        │                                      ┌───────────┐
        │                                      │ §7 トリガ │
        │                                      │   ー判定  │
        │                                      └───────────┘
        │                                            │
        │                                            ▼
        │                                      ┌───────────┐
        └──────────────────────────────────────│ §8 推奨ア │
                                               │   クション│
                                               └───────────┘
                                                     │
                                                     ▼
                                               ┌───────────┐
                                               │ §9 付録   │
                                               │ (検索cmd) │
                                               └───────────┘

凡例: ▶ は依存方向（左が右の集計を参照する）
      §1 サマリは §6 §7 §8 の結論を冒頭に要約する
```

### 9.2 データフロー図

```
┌───────────────────────────────────────────────────────────────┐
│ ソース情報 (棚卸し対象 SHA: 540b7a0e)                         │
│   src/sync/endpoint.rs               (138 行, Endpoint enum)  │
│   src/sync/endpoint/source.rs        (77 行, SyncSource)      │
│   src/sync/endpoint/destination.rs   (90 行, SyncDestination) │
│   src/sync/endpoint/binding.rs       (160 行, TargetBinding)  │
│   src/sync.rs / src/commands/deploy/sync.rs   (呼び出し元)    │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        ▼  (探索: Read + rg)
┌───────────────────────────────────────────────────────────────┐
│ exploration-report.md (Section 2-4, 8 の集計データ)           │
│   - シンボル一覧 (S-1..S-8 / D-1..D-9 / E-1..E-9 / B-1..B-12)│
│   - メソッド比較表 (Duplicate=6 / Similar=1 / Unique=1)       │
│   - co-occurrence サイト一覧 (両型併用 5 ファイル / prod 4・test 1)│
│   - 再現コマンド (rg / comm ...)                              │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        ▼  (転記・整形 — 再集計しない)
┌───────────────────────────────────────────────────────────────┐
│ §3 シンボル一覧  →  §4 比較表  →  §5 両方扱う箇所  →  §6 集計 │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        ▼  (判定ロジック適用)
┌───────────────────────────────────────────────────────────────┐
│ §7 トリガー条件判定                                           │
│   条件A: 実装重複で判定 → TargetBinding で解消済 → 未達       │
│   条件B: match 厳密判定 → production match-over-both=0 → 未達 │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────────────────┐
│ §8 推奨アクション = 「現状維持。Endpoint enum は pre-staged   │
│   のまま温存。match-over-both 発生時に再評価」                │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        ▼  (要約)
┌───────────────────────────────────────────────────────────────┐
│ §1 サマリ (TL;DR — §6 §7 §8 の結論3行)                        │
└───────────────────────────────────────────────────────────────┘
```

### 9.3 再現可能な検索コマンド

棚卸し対象 SHA: `540b7a0efac627c1f3e043cabd9939751f044540`

必須コマンド (A〜E) は条件A・B の判定に使用する。F は補足情報（Endpoint 定義ファイル内の `match self` 数を確認するためのもので、判定対象外）。

```bash
# 棚卸し対象 SHA: 540b7a0efac627c1f3e043cabd9939751f044540

# A. 両型を「同一ファイル内で」参照するファイル列挙（comm で集合の積）
comm -12 \
  <(rg -l --type rust -g '!target' "SyncSource" /workspace/src | sort) \
  <(rg -l --type rust -g '!target' "SyncDestination" /workspace/src | sort)
# 期待: 5 ファイル
#   /workspace/src/commands/deploy/sync.rs
#   /workspace/src/sync.rs
#   /workspace/src/sync/endpoint.rs
#   /workspace/src/sync/endpoint/binding.rs
#   /workspace/src/sync/endpoint/endpoint_test.rs

# B. Endpoint:: の production hit（test ファイルを除外）
rg -n --type rust --glob '!**/*_test.rs' "Endpoint::" /workspace/src || true
# 期待: 0 件（pre-staged: production 未使用）
# 注: `rg` は 0 件時に exit status 1 を返すため、`set -e` 配下では `|| true` で抑止する

# C. match-over-both の検出
# 方針: production で `Endpoint` enum を使うコード自体がゼロ（B で確認）であるため、
#       Endpoint enum 経由の match-over-both は構造的にゼロ。
#       追加で「`Self::Source` と `Self::Destination` の両アームを含む match」を構文上検出する。
#       Endpoint enum 定義ファイル src/sync/endpoint.rs と test ファイルは除外。
rg -nU --type rust \
  --glob '!**/*_test.rs' \
  --glob '!**/sync/endpoint.rs' \
  "match[^{]*\{[^}]*Self::Source[^}]*Self::Destination" /workspace/src || true
# 期待: 0 件（production match-over-both は無い）
# 注: `rg` は 0 件時に exit status 1 を返すため、`set -e` 配下では `|| true` で抑止する
# 補足: Endpoint enum 定義ファイル(src/sync/endpoint.rs)内の match arm（pre-staged）はカウント対象外。
#       定義ファイル内の `match self` 数を確認したい場合は補足コマンド F を参照。

# D. ペア引数関数の検出（複数行シグネチャを拾うため -U 必須）
rg -nU --type rust "fn[^\{]*&SyncSource[^\{]*&SyncDestination" /workspace/src
# 期待: 5 件（src/sync.rs の sync / sync_with_fs / execute_sync / execute_create / execute_update）

# E. 両型の片方/両方/未参照ファイルの内訳（必須・判定補助用の件数内訳）
rg -l --type rust -g '!target' "SyncSource" /workspace/src | sort > /tmp/src.txt
rg -l --type rust -g '!target' "SyncDestination" /workspace/src | sort > /tmp/dst.txt
echo "両方参照: $(comm -12 /tmp/src.txt /tmp/dst.txt | wc -l)"
echo "Source のみ: $(comm -23 /tmp/src.txt /tmp/dst.txt | wc -l)"
echo "Destination のみ: $(comm -13 /tmp/src.txt /tmp/dst.txt | wc -l)"
# 期待: 両方=5 / Source のみ=1（source.rs） / Destination のみ=2（destination.rs, destination_test.rs）
# 補足: source_test.rs は両型に直接言及なし（parse_component_name のテストのみ）のためどの集合にも入らない

# F. （補足・参考情報）Endpoint enum 定義ファイル内の `match self` ブロック数
rg -c "match self" /workspace/src/sync/endpoint.rs
# 期待: 6（as_source / as_destination / name / command_format / placed_components / path_for）
# 補足: F はトリガー条件A・B の判定には影響しない参考情報。必須コマンドは A〜E のみ。
```
