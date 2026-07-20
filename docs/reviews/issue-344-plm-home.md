# レビュー: Issue #344

| 項目 | 内容 |
|------|------|
| Issue | [#344 [fix/refactor] HOME/PLM_HOME 解決を一元化する（PLM_HOME が一部のパス解決で無視され、状態が2つのルートに分裂する）](https://github.com/DIO0550/plugin-manager/issues/344) |
| レビュー日 | 2026-07-20 |
| 対象ブランチ | `main`（`40f011f` 時点） |
| ラベル | `refactor` |
| 関連 | [#333 config.toml 実装](https://github.com/DIO0550/plugin-manager/issues/333)（`PLM_HOME` 一貫性確認を明記） |

## サマリー

Issue が指摘する「`PLM_HOME` 設定時にキャッシュとレジストリが別ルートに分裂する」問題は **現状コードで再現可能であり、修正方針（単一の `plm_home()` / `PlmPaths`）も妥当**。

ただし Issue 本文の対象一覧に **誤分類が2件** あり、そのまま実装すると Personal スコープ配置を壊す。加えて **`PLM_HOME` の意味論が docs と実装で食い違っており、一元化の前に契約を固定する必要がある**。

**結論: 方針承認。実装前に (1) `PLM_HOME` セマンティクス確定、(2) 対象パスの境界划定、(3) 正規化ポリシー統一 を Issue / 実装メモへ追記すること。**

## 問題の再現性（現状コード）

### PLM 状態ルート（`.plm` 配下）— 分裂が実在

| 実装箇所 | 解決方式 | `PLM_HOME` | 組み立てパス |
|----------|----------|------------|--------------|
| `src/plugin/cache/cache.rs` `PackageCache::new` | `EnvVar::get("PLM_HOME").or_else(HOME)` | ✓ | `{root}/.plm/cache/plugins/` |
| `src/marketplace/registry.rs` `MarketplaceRegistry::new` | 同上 | ✓ | `{root}/.plm/cache/marketplaces/` |
| `src/import/registry.rs` `ImportRegistry::new` | `std::env::var("HOME")` のみ | ✗ | `{HOME}/.plm/imports.json` |
| `src/marketplace/config.rs` `MarketplaceConfig::load` | 同上 | ✗ | `{HOME}/.plm/marketplaces.json` |
| `src/target/core/registry.rs` `TargetRegistry::new` | 同上 | ✗ | `{HOME}/.plm/targets.json` |

`PLM_HOME=/tmp/plm-alt` のとき:

- キャッシュ → `/tmp/plm-alt/.plm/cache/...`
- `targets.json` / `marketplaces.json` / `imports.json` → `$HOME/.plm/...`

ドキュメントもこの不整合を認識済み（`docs/reference/config.md` L21、「全パスで一貫して尊重されるとは限りません」）。

### ユーザー HOME（ターゲット Personal 配置）— Issue 対象に含めるべきでない

| 実装箇所 | 役割 | `PLM_HOME` を見るべきか |
|----------|------|-------------------------|
| `src/target/core/paths.rs` `home_dir()` | Personal スコープの `~/.codex` / `~/.cursor` 等の親 | **否**（OS ユーザー HOME） |
| `src/plugin/cache/cleanup.rs` `resolve_home_dir()` | Personal 配置ディレクトリの掃除 | **否**（同上） |

これらは「PLM の状態ルート」ではなく「各 AI ツールの Personal 配置先の親ディレクトリ」を解決する。`PLM_HOME` をここに流すと、`PLM_HOME` 設定時に Skills/Agents が `$PLM_HOME/.codex/...` へ配置され、実際の Codex/Cursor が見るパスと乖離する。

Issue 提案 3（`paths.rs` の `"~"` フォールバックを単一ポリシーへ吸収）は、**ユーザー HOME 側の正規化ポリシー統一**としては検討余地があるが、`plm_home()` への吸収ではない。後述「境界」。

## 設計ブロッカー: `PLM_HOME` の意味論

### docs と実装の食い違い

| ソース | 解釈 |
|--------|------|
| `docs/reference/config.md` 環境変数表 | 「PLMのホームディレクトリ（**デフォルト: `~/.plm`**）」→ `PLM_HOME` **自体が** `.plm` ルート（Cargo の `CARGO_HOME` 型） |
| `PackageCache` / `MarketplaceRegistry` 実装 | `$PLM_HOME/.plm/...` を組む → `PLM_HOME` は **`HOME` の代替**（未設定時 `$HOME/.plm`） |
| `docs/architecture/cache.md` | 「`$PLM_HOME/.plm/cache/plugins/`（未設定時は `$HOME`）」→ 実装と同型（HOME 代替） |

**危険な誤用シナリオ（docs 表を信じた場合）:**

```bash
export PLM_HOME=~/.plm   # docs の「デフォルト」を明示設定したつもり
# 実装は ~/.plm/.plm/cache/... を作る
```

一元化の前にセマンティクスを **どちらかに固定**し、docs / コード / テストを揃えること。

### 推奨契約（現状実装互換）

**案 A（推奨・後方互換）: HOME 代替**

```text
plm_root = $PLM_HOME  if set and valid
         = $HOME      otherwise
plm_dir  = {plm_root}/.plm
```

- 既存のキャッシュ利用者が壊れない
- docs の環境変数表と README の「デフォルト: `~/.plm`」表現を「未設定時の実効パスは `$HOME/.plm`。`PLM_HOME` は `$HOME` の代替」に修正する

**案 B: CARGO_HOME 型（docs 表寄り）**

```text
plm_dir = $PLM_HOME  if set and valid
        = $HOME/.plm otherwise
```

- 直感的で docs 表と一致
- **既存の `PLM_HOME` 利用者は破壊的変更**（パスが `.plm` 一段深くなる/浅くなる）
- キャッシュ実装の書き換えが必須

**レビュー推奨は案 A**。案 B にするなら Issue に破壊的変更とマイグレーション方針を明記すること。

## 設計判断のレビュー

### 1. `plm_home()`（または同等）を1箇所に置く

#### 結論: **承認。配置は `src/env.rs` が適切**

| 観点 | 評価 |
|------|------|
| 重複除去 | ◎ 現状の2系統（`EnvVar` 連鎖 vs 生 `var("HOME")`）を解消 |
| `path_ext` への配置 | △ `PathExt` はパス操作トレイト。環境変数解決は責務外 |
| `env.rs` | ◎ 既存 `EnvVar` の延長。`plm_root()` / `plm_home()` を同モジュールに追加 |

推奨シグネチャ例:

```rust
/// PLM 状態の親ディレクトリ（HOME 代替セマンティクス）。
/// 未設定・空・空白のみはエラー（または Option）。
pub fn plm_root() -> Result<PathBuf, PlmError> { /* PLM_HOME → HOME */ }

/// `{plm_root}/.plm`
pub fn plm_dir() -> Result<PathBuf, PlmError> {
    Ok(plm_root()?.join(".plm"))
}
```

命名は `plm_home()` でもよいが、docs の「`PLM_HOME` = `.plm` そのもの」誤解を避けるなら **`plm_root()` + `plm_dir()`** の二段が明確。

### 2. `PlmPaths` 型付きアクセサ

#### 結論: **承認。対象は `.plm` 配下のみに限定する**

```text
PlmPaths
  ├── dir()                 → {root}/.plm
  ├── config_toml()         → {root}/.plm/config.toml   (#333 準備)
  ├── targets_json()        → {root}/.plm/targets.json
  ├── marketplaces_json()   → {root}/.plm/marketplaces.json
  ├── imports_json()        → {root}/.plm/imports.json
  ├── plugins_cache_dir()   → {root}/.plm/cache/plugins
  └── marketplaces_cache_dir() → {root}/.plm/cache/marketplaces
```

呼び出し側は `PackageCache::new` / `MarketplaceRegistry::new` / 各 `*Registry::new` / `MarketplaceConfig::load` が `PlmPaths`（または `plm_dir()`）経由に切り替える。

テスト用に既にある `with_cache_dir` / `with_path` / `load_from` は維持し、デフォルト経路だけを一元化する。

### 3. `paths.rs` / `cleanup.rs` を `plm_home()` に吸収しない

#### 結論: **Issue 対象から除外（ブロッカー級の境界）**

| パス族 | 環境変数 | 用途 |
|--------|----------|------|
| PLM 状態 | `PLM_HOME` → `HOME` | `~/.plm/**` |
| ユーザー HOME | `HOME` のみ（将来 `CODEX_HOME` 等は別） | Personal 配置・掃除 |

`paths.rs` の `"~"` リテラルフォールバックと `cleanup.rs` の「絶対パス必須・`None` で personal スキップ」は、**ユーザー HOME 正規化**の問題として別 Issue または本 Issue の「付録スコープ」で扱う。混ぜると責務が再び分散する。

ユーザー HOME 側を触る場合の推奨:

- `paths::home_dir()` と `cleanup::resolve_home_dir()` の正規化を共通 helper（例: `user_home_dir() -> Option<PathBuf>`）に寄せる
- `"~"` フォールバックは残すか、呼び出し側で `Option` にするかを別判断（現状ドキュメントは「HOME 確実前提」）
- **`PLM_HOME` は参照しない**

### 4. 正規化ポリシーの統一（PLM ルート側）

現状の差:

| 実装 | 空文字 | 空白のみ | trim | 相対パス | 非 UTF-8 |
|------|--------|----------|------|----------|----------|
| `EnvVar::get` | `None` | 受理（非空なら） | なし | 受理 | `var` なので不可 |
| `std::env::var("HOME")`（registry 系） | 受理 | 受理 | なし | 受理 | エラー |
| `paths::home_dir` | `"~"` | `"~"` | あり | 受理 | 不可 |
| `cleanup::resolve_home_dir` | `None` | `None` | あり | `None` | `OsString` 受理 |

`plm_root()` では最低限次を固定する:

1. 未設定 / 空 / 空白のみ → エラー（または次候補へフォールバック）
2. trim してから使用（`EnvVar::get` の強化、または専用 getter）
3. 相対パス: **拒否してエラー**を推奨（テストで CWD 依存の状態分裂を防ぐ）。許容するなら明示ドキュメント化
4. エラー型: `MarketplaceConfig::load` の `Result<_, String>` も `PlmError` 系へ寄せると呼び出しが揃う（本 Issue の必須ではないが隣接改善）

## 推奨実装ステップ

1. **契約固定**: Issue に案 A/B の決定を追記。docs（`reference/config.md` / `architecture/cache.md`）を同期
2. **Red**: `PLM_HOME` 設定時に `targets.json` / `marketplaces.json` / `imports.json` / 両キャッシュが同一 `{root}/.plm` 配下になるテスト
3. **Green**: `env::plm_root` / `plm_dir`（± `PlmPaths`）を追加し、5 箇所のデフォルト構築を置換
4. **境界テスト**: `PLM_HOME` 設定下でも `paths::home_dir()` / Personal 配置が `$HOME` を見ることを回帰で固定
5. **#333 接続**: `config.toml` 実装時は `PlmPaths::config_toml()` を前提にし、再度独自の HOME 解決を増やさない

## テスト観点

| ID | ケース | 期待 |
|----|--------|------|
| T1 | `PLM_HOME` 未設定、`HOME=/h` | 全 PLM 状態が `/h/.plm/...` |
| T2 | `PLM_HOME=/p`、`HOME=/h` | 全 PLM 状態が `/p/.plm/...`（レジストリ含む） |
| T3 | `PLM_HOME=""` | `HOME` へフォールバック（`EnvVar` 現行互換） |
| T4 | `PLM_HOME="   "` | フォールバックまたはエラー（方針どおり一貫） |
| T5 | `PLM_HOME` も `HOME` も無し | 明確なエラー（キャッシュと同文言系統） |
| T6 | T2 条件下で Codex Personal Skill 配置 | パスが `/h/.codex/...`（`/p` 配下に行かない） |
| T7 | 既存 `with_path` / `with_cache_dir` | 挙動不変 |

## Issue 本文への追記推奨

1. **対象から外す**: `target/core/paths.rs`、`plugin/cache/cleanup.rs`（ユーザー HOME）。必要なら「ユーザー HOME 正規化は別タスク」と明記
2. **セマンティクス決定**: 案 A（HOME 代替）か案 B（CARGO_HOME 型）か。破壊的変更の有無
3. **docs 修正を DoD に含める**: `reference/config.md` の「デフォルト: `~/.plm`」表現と実装の一致
4. **非目標**: ターゲット別 `CODEX_HOME` / `COPILOT_HOME` 等の尊重（将来・#333 第3段階）

## 結論

| 項目 | 判定 |
|------|------|
| 問題の実在性 | ✅ 確認 |
| `plm_home` / `PlmPaths` 集約 | ✅ 承認（`.plm` 配下のみ） |
| Issue 一覧の `paths.rs` / `cleanup.rs` | ❌ 対象外にすべき（誤分類） |
| `PLM_HOME` 意味論 | ⚠ 実装前に確定必須（推奨: HOME 代替 = 案 A） |
| #333 との関係 | 本 Issue を先に閉じると config.toml 実装が安全 |

**実装着手前の必須アクション:** Issue コメントでセマンティクス（案 A/B）と対象境界を確定する。それが済めばリファクタの技術リスクは低い。
