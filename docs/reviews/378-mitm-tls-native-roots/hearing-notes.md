# Hearing Notes: MITM プロキシ環境での TLS 検証失敗修正

## 目的

MITM プロキシ環境（Codex Cloud 等）において、`plm` コマンドが TLS 検証失敗で Network error になる問題を修正する。
reqwest の TLS バックエンドをシステム CA / SSL_CERT_FILE を読み込む構成に変更し、プロキシ環境でも正常に動作させる。

## スコープ

- **種別**: バグ修正
- **影響範囲**: 複数コンポーネント（`Cargo.toml` の reqwest feature、`HttpConfig::build_client()`、エラー表示）。HTTP クライアントは共有のため、marketplace 以外の全 GitHub API 呼び出しにも波及するが、これは意図どおり。
- **優先度**: 高（クラウド環境セットアップがブロッカー）

## 技術的詳細

- **技術スタック**: Rust 2021 edition
- **フレームワーク**: なし（Rust CLI）
- **依存関係**:
  - reqwest 0.12（`rustls-tls` → `rustls-tls-native-roots` へ変更）
  - rustls-native-certs は reqwest feature 経由で入る想定（明示依存が必要か計画で判断）
- **データ構造**: `HttpConfig::build_client()` — ClientBuilder への CA 追加ロジック追加
- **実装方針**:
  1. **本筋（必須）**: `Cargo.toml` の `rustls-tls` → `rustls-tls-native-roots` に変更
  2. **保険（実施する）**: `HttpConfig::build_client()` で `SSL_CERT_FILE` と `CODEX_PROXY_CERT` の両方を読み込み、`add_root_certificate()` で明示追加。存在する方を追加し、両方が同じパスなら二重追加しない
  3. **エラー表示改善（実施する）**: `--verbose` 時にエラーチェーン（cause）を表示。通常時は現行の簡潔メッセージを維持しつつ、TLS/証明書関連であることが分かる一文の追加も許容

## 品質要件

- **エッジケース**:
  - 証明書ファイル不在（環境変数が設定されているがファイルが存在しない）
  - 不正 PEM（パース失敗）
  - 空パス（空文字列の環境変数）
  - `SSL_CERT_FILE` と `CODEX_PROXY_CERT` が同じパスを指す場合（二重追加しない）
  - プロキシなし通常環境での回帰なし（既存動作を壊さない）
  - `http_proxy` / `https_proxy` は現状維持
- **エラーハンドリング**:
  - 証明書読み込み失敗は `warn!` ログを出して続行（致命的エラーにしない）
  - エラーチェーン表示は `--verbose` フラグ時のみ
- **テスト要件**:
  - ユニットテストで (1) CA ファイル読み込みロジック (2) エラーチェーン整形 をカバー
  - 実 MITM 環境の E2E は手動検証手順で記載
  - テストファイルは `*_test.rs` に分離（CLAUDE.md 方針）
- **パフォーマンス**:
  - 起動時 / クライアント構築時の CA 読込のみ。許容範囲
- **後方互換**:
  - 通常環境（システム CA / webpki で十分な環境）で既存動作を壊さないこと

## 追加コンテキスト

- Issue #378 / 関連: #376
- 現行コード:
  - `Cargo.toml` L10: `reqwest = { features = ["json", "stream", "rustls-tls"] }`
  - `src/config.rs`: `HttpConfig::build_client()` — timeout / user_agent のみ
  - `src/error.rs`: `PlmError::Network(#[from] reqwest::Error)` — `Network error: {0}` のみ
  - `src/host/github.rs`: `config.build_client()` を使用
- ライセンス: 依存変更時は `THIRD_PARTY_LICENSES.md` / `cargo-deny` の方針に従う
