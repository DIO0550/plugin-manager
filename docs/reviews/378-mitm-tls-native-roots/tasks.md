# Task: MITM プロキシ環境での TLS 検証失敗を修正する

## Research & Planning

- □ `src/config_test.rs` の既存テスト構造を確認（並列実行時の env var 競合リスク評価）
- □ テスト用の自己署名 PEM フィクスチャをどう用意するか確認（ハードコード文字列 or tempfile）
- □ `rustls-tls-native-roots` feature の `reqwest::Certificate::from_pem` API の正確なシグネチャ確認

## Implementation (TDD サイクル)

### [FR-1] Cargo.toml: rustls-tls → rustls-tls-native-roots

- □ RED: `rustls-tls-native-roots` feature が有効になった場合のテストシナリオを確認（コンパイルが通ることを確認するだけで十分）
- □ GREEN: `Cargo.toml` の `rustls-tls` → `rustls-tls-native-roots` に変更し `cargo check` で確認

### [FR-2/FR-3] config.rs: CA 注入ロジック

- □ RED: `SSL_CERT_FILE` が未設定のとき `build_client()` が正常動作するテストを書く（`config_test.rs`）
- □ GREEN: テストを通す最小実装（環境変数を読むが何もしない仮実装で既存テストが壊れないことを確認）

- □ RED: 有効な PEM ファイルを指す `SSL_CERT_FILE` で `build_client()` がクライアントを返すテストを書く
- □ GREEN: `add_cert_from_path()` を実装し `add_root_certificate()` を呼ぶ

- □ RED: 存在しないパスの `SSL_CERT_FILE` でも `build_client()` がクライアントを返す（エラーにならない）テストを書く
- □ GREEN: ファイル読み込み失敗を `eprintln!` で警告し続行する実装

- □ RED: 不正 PEM の `SSL_CERT_FILE` でも `build_client()` がクライアントを返すテストを書く
- □ GREEN: PEM パース失敗を `eprintln!` で警告し続行する実装

- □ RED: `SSL_CERT_FILE` と `CODEX_PROXY_CERT` が同一パスのとき二重追加されないテストを書く
- □ GREEN: パス重複チェックを追加する実装

- □ REFACTOR: `add_cert_from_path` のエラーメッセージを整理、コードの重複を除去

### [FR-5] error.rs: PlmError::General 追加 + source chain 保持

- □ RED: `PlmError::General("msg") → RichError` で `code=Int001, message="msg"` になるテストを書く（`error.rs` 内テスト）
- □ GREEN: `PlmError::General(String)` バリアントを追加し `From` impl に追加
- □ GREEN: `From<PlmError> for RichError` の `Network` ブランチを `match err`（所有）に変更し `.with_source(e)` を追加

- □ RED: verbose=true 時に RichError の source chain が `ErrorFormatter` で出力されるテストを書く（`formatter.rs` 内テスト）
- □ GREEN: 既存の `format_source_chain()` が動くことを確認（実装変更不要のはず）
- □ REFACTOR: `all_plm_errors_have_explicit_mapping` テストに `PlmError::General` を追加

### [verbose display] commands.rs: dispatch 戻り型変更

- □ RED: `dispatch` が `Result<(), PlmError>` を返すことを想定した `main.rs` のコンパイルを確認
- □ GREEN: `dispatch` の戻り型を `Result<(), PlmError>` に変更し、内部で `result.map_err(PlmError::General)` を適用

### [verbose display] main.rs: ErrorFormatter 接続

- □ GREEN: `main.rs` に `let verbose = cli.verbose;` を追加し、`ErrorFormatter::new(verbose).format(&rich)` でエラー表示
- □ GREEN: `error.rs` の `mod formatter` を `pub mod formatter` に、`ErrorFormatter` を `pub use` に変更（必要な場合）
- □ REFACTOR: `main.rs` の import を整理

## Verification

- □ `cargo build` が成功することを確認
- □ `cargo clippy -- -D warnings` が警告なし
- □ `cargo fmt -- --check` がフォーマット違反なし
- □ `cargo test` が全テストパスすることを確認
- □ `SSL_CERT_FILE=/nonexistent.pem plm --help` を実行し `[plm warn]` が出て終了コード 0 であることを確認（warn でクラッシュしない）
- □ `plm --verbose install <public-repo>` で verbose フォーマットが出力されることを確認
- □ 手動: MITM プロキシ環境または `SSL_CERT_FILE` 設定で HTTP コマンドが成功することを確認
- □ `cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs` を実行して THIRD_PARTY_LICENSES.md を更新
- □ `cargo deny check` を実行して結果を確認（informational）
