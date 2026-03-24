# Hearing Notes: clippy.toml に cognitive-complexity-threshold を導入

## 目的

`clippy.toml` を追加し、複数の Clippy 閾値を設定して深いネスト・大きい関数を CI で検出できるようにする。`deploy_hook_converted()` 内の6段ネスト指摘が契機。

## スコープ

- **種別**: 設定追加 + リファクタリング + CI改善
- **影響範囲**: プロジェクト全体（clippy.toml はプロジェクトルートに配置）
- **優先度**: 中

## 技術的詳細

- **技術スタック**: Rust
- **ツール**: Clippy (rust-clippy)
- **設定ファイル**: `clippy.toml`（プロジェクトルート）
- **設定する閾値**:
  - `cognitive-complexity-threshold = 10`（デフォルト: 25）
  - `too-many-arguments-threshold = 5`（デフォルト: 7）
  - `too-many-lines-threshold = 80`（デフォルト: 100）
  - `type-complexity-threshold = 200`（デフォルト: 250）

## 品質要件

- **既存違反の対応方針**: すべて修正する（allow による抑制は使わない）
- **テストコード**: cognitive-complexity チェックの対象から除外する（テストファイルでは allow で抑制）
- **CI統合**: cargo clippy -- -D warnings をCIパイプラインに追加する（本Issueのスコープ内）
- **テスト要件**: リファクタリングにより既存テストが壊れないことを確認

## 追加コンテキスト

- Issue #197 が対象
- 背景: `deploy_hook_converted()` 内の JSON wrapper パス書き換え処理で6段ネストが発生し、レビューで指摘された
- デフォルトの閾値25では検出されないため、プロジェクト全体で閾値を下げて早期に検出したい
- 参考: https://rust-lang.github.io/rust-clippy/master/index.html#cognitive_complexity
