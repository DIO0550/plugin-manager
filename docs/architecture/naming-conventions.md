# 命名規約

> 作成日: 2026-05-09
> 関連 Issue: #259 / #271

このドキュメントは PLM プロジェクトにおける Rust の型名・識別子の命名規約をまとめる。
特に `Result` / `Outcome` 接尾辞の使い分けに重点を置く。

---

## 1. `Result` / `Outcome` 接尾辞

| 接尾辞 | 用途 | 例 |
|---|---|---|
| `Result` | `std::result::Result<T, E>` および `crate::error::Result<T>` (= `Result<T, PlmError>`) のエイリアス専用 | `crate::error::Result<()>` |
| `Outcome` | ドメイン処理の「成果レポート」。成功/部分成功/失敗の集計、影響範囲、副作用などをまとめた値オブジェクト | `SyncOutcome`, `OperationOutcome` |
| `Status` / `Kind` / `Failure` 等 | enum や付随する区分・失敗詳細用に従来通り使用 | `UpdateStatus`, `SyncFailure` |

### 1.1 規約の要点

- **`Result` 接尾辞** はエラー型を内包する `Result<T, E>` 専用に予約する。型名から「エラー型を含む `Result<T,E>` か、ドメイン成果レポートか」が一目で識別できることを目的とする。
- **`Outcome` 接尾辞** はドメインの操作結果 (成果レポート) を表す値オブジェクトに用いる。Ok/Err を内包しない通常の `struct` であり、フィールドとして処理の集計 (作成/更新/削除されたコンポーネント、失敗一覧、影響範囲、副作用フラグ等) を持つ。

### 1.2 適用範囲

- **新規追加するドメイン処理結果型**: 必ず `*Outcome` 接尾辞を使う。
- **既存のドメイン処理結果型**: Issue #271 (Issue #259-A/B/C 一括対応) で以下 5 型を `*Outcome` に統一済み。
  - `SyncOutcome`
  - `ConvertOutcome`
  - `OperationOutcome` (type alias 削除済み)
  - `UpdateOutcome` (type alias 削除済み)
  - `MarketplaceAddOutcome` (tui::manager::screens::marketplaces::actions 配下)

### 1.3 適用継続検討対象

以下は本規約の適用を継続検討する。

- **`target::AddResult` (enum)**: 名称が `*Result` だが「成功/失敗のバリアント」を持つ enum で、`Result` 接尾辞ながら例外的に存続。別 Issue で再検討する。
- **横断利用度の低い `*Result` 型** (`PlaceResult` / `ConversionResult` / `MultiSelectResult` 等): 横断度が低いため Issue #271 では対象外とした。横断利用が増えた段階で本規約に従って再評価する。
- **`type SyncFooBar = std::result::Result<T, SyncError>` のような型エイリアス** (もし新たに必要になる場合): `Result` 接尾辞を維持してよい (この用途こそが `Result` 接尾辞の本来の意味)。

---

## 2. 関連メソッド名

`Outcome` を生成するファクトリメソッドや変換メソッドの命名は本規約の対象外とする。

例:
- `AffectedTargets::into_result(self) -> OperationOutcome` — メソッド名 `into_result` はそのまま維持。本規約は「ドメイン処理結果値の型名」のみが対象。
- `OperationOutcome::error(msg)` / `UpdateOutcome::updated(...)` 等のファクトリメソッド名も従来通り。

---

## 3. その他の接尾辞

- `Status`: 単純な状態を表す enum (例: `UpdateStatus { Updated, AlreadyUpToDate, Failed, Skipped }`)
- `Kind`: 種別を表す enum (例: `ComponentKind`)
- `Failure`: 失敗の詳細を表す struct (例: `SyncFailure`)
- `Error`: エラー型 (例: `PlmError`、`SyncError`)

これらは従来通り意味に沿って使い分ける。

---

## 4. 画面 Model の命名

TUI 画面 (`src/tui/manager/screens/<screen>/`) で定義される Model 型は、
**型定義から内部参照・公開境界まですべて `<Screen>ScreenModel` 形式に統一する**。
`Model` という汎用名は型として用いない（ローカル変数名 `model` は許容）。

### 4.1 ルール

- **型定義 (`screens/<screen>/model.rs`)**: `pub struct Model` / `pub enum Model` は使用禁止。`pub struct <Screen>ScreenModel` / `pub enum <Screen>ScreenModel` 形式で定義する
  - 例: `InstalledScreenModel`, `DiscoverScreenModel`, `MarketplacesScreenModel`, `ErrorsScreenModel`
- **画面内 import (`super::model::*`)**: `super::model::<Screen>ScreenModel` で参照する。`use ... as Model` 形式の alias 化や `pub type Model = ...` 形式の互換 alias は禁止
- **画面 root (`screens/<screen>.rs`) の pub re-export**: 素の `Model` を再 export してはならない。`Model as <Alias>` 形式も含めて `Model` 名は外部に出さない
- **ローカル変数名**: `let mut model = ...` のような変数名・関数引数名・`make_model` 等のヘルパー関数名はそのまま許容

### 4.2 適合例

```rust
// screens/installed/model.rs
pub enum InstalledScreenModel { /* ... */ }

// screens/installed.rs (画面 root)
pub use model::{key_to_msg, CacheState, InstalledScreenModel, Msg};

// screens/installed/update.rs (画面内ファイル)
use super::model::{InstalledScreenModel, Msg};

pub fn update(msg: Msg, model: &mut InstalledScreenModel) {
    //                  ^^^^^                ^^^^^^^^^^^^^^^^^^^^
    //                  変数名は model のまま 型は ScreenModel
}
```

### 4.3 不適合例

```rust
// NG: 型定義で素の Model
pub struct Model { /* ... */ }

// NG: 画面 root から素の Model を pub re-export
pub use model::{Model, Msg};

// NG: `Model as <Alias>` 形式の互換 export
pub use model::{Model as InstalledScreenModel};

// NG: 画面内 import で `as Model` alias
use super::model::InstalledScreenModel as Model;

// NG: model.rs 末尾での `pub type Model` 互換 alias
pub type Model = InstalledScreenModel;
```

### 4.4 ガード

`scripts/check-public-screen-model.sh` を CI ジョブ `naming-guard` で実行し、
画面 root から素の `Model` が公開されていないことを検証する。

---

## 5. 参考

- Issue #259: Result 系型名の整理 (親 Issue)
- Issue #259-A: 棚卸し
- Issue #259-B: 命名方針策定
- Issue #259-C: リネーム実装
- Issue #271: 上記 #259-A/B/C を一括対応した Issue
- Issue #258: 画面 Model 型のリネーム
