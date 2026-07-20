# Hearing Notes: PLM_HOME / HOME 解決の一元化

## 目的

`PLM_HOME` 設定時にプラグインキャッシュとレジストリ／設定 JSON が別ルートに分裂する不整合を解消し、PLM 状態（`~/.plm` 相当）のパス解決を単一ポリシーに集約する。

## スコープ

- **種別**: リファクタリング / バグ修正（挙動不整合の解消）
- **影響範囲**: 既存修正（`PackageCache` / `MarketplaceRegistry` / `ImportRegistry` / `MarketplaceConfig` / `TargetRegistry` のデフォルトパス構築）。新規モジュールとして `plm_root` / `PlmPaths` を追加
- **優先度**: 高（#333 config.toml 実装の前提、テスト時のルート差し替えにも必要）
- **非スコープ**:
  - `target/core/paths.rs` の `home_dir()`（ユーザー HOME / Personal 配置用）
  - `plugin/cache/cleanup.rs` の `resolve_home_dir()`（同上）
  - ターゲット別 `CODEX_HOME` 等の尊重（#333 第3段階）
  - `config.toml` 本体の実装（#333）

## 技術的詳細

- **技術スタック**: Rust 2021
- **フレームワーク**: 既存 CLI（clap / tokio）。新規依存なし
- **依存関係**: 既存 `crate::env::EnvVar` を拡張。`path_ext` には置かない
- **データ構造**:
  - `plm_root()` → `PLM_HOME`（有効時）else `HOME`（HOME 代替セマンティクス = レビュー案 A）
  - `plm_dir()` → `{plm_root}/.plm`
  - `PlmPaths`（または同等のアクセサ）: `targets_json` / `marketplaces_json` / `imports_json` / `plugins_cache_dir` / `marketplaces_cache_dir` /（将来）`config_toml`
- **確定セマンティクス（案 A）**:
  ```text
  plm_root = $PLM_HOME if set and valid, else $HOME
  plm_dir  = {plm_root}/.plm
  ```
  docs の「デフォルト: `~/.plm`」表現は「未設定時の実効パス」に合わせて修正する（破壊的変更なし）

## 品質要件

- **エッジケース**:
  - `PLM_HOME` / `HOME` 未設定・空・空白のみ
  - 相対パスの `PLM_HOME`（拒否してエラーを推奨）
  - `PLM_HOME` 設定下でも Personal 配置は `$HOME` を見ること（回帰）
- **エラーハンドリング**: 既存キャッシュと同様、両方未設定なら明確なエラー。`MarketplaceConfig` の `String` エラーは可能なら `PlmError` 寄せ（必須ではない）
- **テスト要件**: TDD。ユニットでパス解決、統合で「同一 `{root}/.plm` 配下」を検証。Personal 配置が `PLM_HOME` に引きずられない回帰テスト
- **パフォーマンス**: 対象外（起動時の環境変数読み取りのみ）

## 追加コンテキスト

- Issue: https://github.com/DIO0550/plugin-manager/issues/344
- 関連: #333（config.toml — `PLM_HOME` 一貫性確認）
- 事前レビュー: `docs/reviews/issue-344-plm-home.md`（PR #389）の推奨を本ヒアリングの確定値として採用
- クラウドエージェント環境のため AskUserQuestion 不可。レビュー推奨（案 A・対象境界）をユーザー確認済み前提として進行
- レビューツール: none（`.config.yml`）
