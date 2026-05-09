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
- **既存の `*Result` 型**: Issue #271 (Issue #259-A/B/C 一括対応) で以下 4 型 + tui 1 型を `*Outcome` に統一済み。
  - `SyncOutcome` (旧 `SyncResult`)
  - `ConvertOutcome` (旧 `ConvertResult`)
  - `OperationOutcome` (旧 `OperationResult`、type alias 削除)
  - `UpdateOutcome` (旧 `UpdateResult`、type alias 削除)
  - `MarketplaceAddOutcome` (旧 `tui::manager::screens::marketplaces::actions::AddResult`)

### 1.3 適用継続検討対象

以下は本規約の適用を継続検討する。

- **`target::AddResult` (enum)**: 名称が `*Result` だが「成功/失敗のバリアント」を持つ enum で、`Result` 接尾辞ながら例外的に存続。別 Issue で再検討する。
- **横断利用度の低い `*Result` 型** (`PlaceResult` / `ConversionResult` / `MultiSelectResult` 等): 横断度が低いため Issue #271 では対象外とした。横断利用が増えた段階で本規約に従って再評価する。
- **`type SyncResult = std::result::Result<T, SyncError>` のような alias** (もし新たに必要になる場合): `Result` 接尾辞を維持してよい (この用途こそが `Result` 接尾辞の本来の意味)。

---

## 2. 関連メソッド名

`Outcome` を生成するファクトリメソッドや変換メソッドの命名は本規約の対象外とする。

例:
- `AffectedTargets::into_result(self) -> OperationOutcome` — メソッド名 `into_result` はそのまま維持。本規約は「ドメイン処理結果値の型名」のみが対象。
- `OperationOutcome::error(msg)` / `UpdateOutcome::updated(...)` 等のファクトリメソッド名も従来通り。

---

## 3. その他の接尾辞

- `Status`: 単純な状態を表す enum (例: `UpdateStatus { Updated, UpToDate, Failed, Skipped }`)
- `Kind`: 種別を表す enum (例: `ComponentKind`)
- `Failure`: 失敗の詳細を表す struct (例: `SyncFailure`)
- `Error`: エラー型 (例: `PlmError`、`SyncError`)

これらは従来通り意味に沿って使い分ける。

---

## 4. 参考

- Issue #259: Result 系型名の整理 (親 Issue)
- Issue #259-A: 棚卸し
- Issue #259-B: 命名方針策定
- Issue #259-C: リネーム実装
- Issue #271: 上記 #259-A/B/C を一括対応した Issue
