# Requirements: MITM プロキシ環境での TLS 検証失敗修正

> research（exploration-report）と plan（implementation-plan）の間で、
> 「誰が・何のために・どう使うか」と要件・制約を確定する中間ドキュメント。
> ユースケースを起点に、技術計画に進む前の前提を固める。

## ユースケース

### UC-1: MITM プロキシ環境での plm コマンド実行

- **アクター**: Codex Cloud セットアップや企業プロキシ配下で `plm` を使用する開発者
- **状況/前提**: `https_proxy` / `SSL_CERT_FILE` / `CODEX_PROXY_CERT` / `REQUESTS_CA_BUNDLE` 等の環境変数が設定された MITM プロキシ環境
- **達成したいこと**: `plm marketplace add` や `plm install` などの HTTP リクエストを伴うコマンドが Network error なく成功する
- **成功条件**: MITM プロキシ環境で TLS 検証が通り、GitHub API / マーケットプレイスへの HTTPS アクセスが成功する

### UC-2: 通常環境での後方互換性維持

- **アクター**: プロキシなし・通常の CA ストアを使う環境の既存 plm ユーザー
- **状況/前提**: プロキシ環境変数未設定、OS の標準 CA ストア（または webpki-roots）が利用可能な環境
- **達成したいこと**: 今回の修正後も既存の plm コマンドが正常動作する
- **成功条件**: 修正前と同様の動作を維持し、回帰が発生しない

### UC-3: TLS エラー発生時の原因特定

- **アクター**: `plm` コマンドを実行してネットワークエラーに遭遇した開発者
- **状況/前提**: TLS 検証エラーが発生しているが、通常の "Network error: ..." メッセージだけでは TLS 起因か否か判断できない
- **達成したいこと**: `--verbose` フラグを付けてコマンドを実行し、エラーチェーン（reqwest → rustls → 証明書検証エラー）を確認してデバッグできる
- **成功条件**: `plm --verbose install ...` が TLS 起因エラーを人間が読める形で表示する

## 要件・制約

### 機能要件

- **FR-1 reqwest feature 変更**: `Cargo.toml` の `rustls-tls` を `rustls-tls-native-roots` に変更し、OS ネイティブの CA ストアを読み込む
- **FR-2 SSL_CERT_FILE 読み込み（保険）**: `HttpConfig::build_client()` で環境変数 `SSL_CERT_FILE` のパスの PEM を `add_root_certificate()` で明示追加する
- **FR-3 CODEX_PROXY_CERT 読み込み（保険）**: `HttpConfig::build_client()` で環境変数 `CODEX_PROXY_CERT` のパスの PEM を `add_root_certificate()` で明示追加する。`SSL_CERT_FILE` と同じパスを指す場合は二重追加しない
- **FR-4 証明書読み込み失敗時の継続**: CA ファイルが見つからない・不正 PEM の場合は `eprintln!` で警告を出して続行（致命的エラーにしない）
- **FR-5 verbose エラーチェーン表示**: `--verbose` フラグ時に `ErrorFormatter` を使い、`reqwest::Error` の source chain（TLS 起因の詳細）を表示する
- **FR-6 通常時のエラーメッセージ**: 非 verbose 時は現行の簡潔メッセージを維持する（回帰なし）

### 非機能要件

- **NFR-1 後方互換**: プロキシなし通常環境で既存動作を壊さない
- **NFR-2 パフォーマンス**: CA ファイル読み込みは `build_client()` 呼び出し時のみ。ランタイムの継続的コストなし
- **NFR-3 テスト**: ユニットテストで CA ファイル読み込みロジックとエラーチェーン整形をカバーする。実 MITM 環境の E2E は手動検証手順で記載
- **NFR-4 ライセンス**: reqwest feature 変更による追加依存クレートについて `THIRD_PARTY_LICENSES.md` を再生成する

### 制約・設計方針

- **C-1 Rust ソース変更範囲**: `Cargo.toml`, `src/config.rs`, `src/error.rs`, `src/main.rs` の最小変更。既存のモジュール構成を変えない
- **C-2 テストファイル分離**: 新テストは `src/config_test.rs` に追加（`*_test.rs` 分離方針）
- **C-3 エラーパスの設計**: `commands::dispatch` の戻り型を `Result<(), PlmError>` に変更し、`main.rs` で `ErrorFormatter::new(cli.verbose)` を使って表示する設計を採用。各コマンドハンドラの戻り型は `Result<(), String>` のまま維持し、`dispatch` 内で `PlmError` に変換する方式は不採用 — 最小変更で verbose を実現するため `main.rs` での接続方式を採用
- **C-4 ログ手段**: `log` / `tracing` クレートなし。CA 読み込み警告は `eprintln!` で `stderr` 直接出力
- **C-5 `rustls-tls-native-roots` の動作**: OS ネイティブ CA + webpki-roots の両方を含む。それでも MITM 証明書は含まれないため、`SSL_CERT_FILE` / `CODEX_PROXY_CERT` の保険読み込み（FR-2/3）は必須
- **C-6 `From<PlmError> for RichError` の修正**: Network ブランチで `.with_source()` を呼び出し、reqwest エラーの source chain を RichError に保持する。ただし現在 `match &err` で借用しているため所有権の扱いに注意が必要

## 未解決の確認事項

なし（全ての確認事項はヒアリング回答で確定済み）
