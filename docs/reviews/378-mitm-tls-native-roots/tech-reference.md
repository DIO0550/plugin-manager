# MITM プロキシ TLS 修正 · 実装の手引き

> このページは implementation-plan を読んで実装に取りかかったときに
> 「ここがよく分からない」となりがちな箇所を、
> ライブラリドキュメントの体裁でまとめたものです。
> 順番に読む必要はなく、詰まったセクションだけ拾い読みできます。

---

## 概要

`plm` が Codex Cloud などの MITM（Man-in-the-Middle）プロキシ環境で `Network error` を返す原因は、reqwest の TLS バックエンドにあります。デフォルトの `rustls-tls` フィーチャーは Mozilla が管理する「webpki-roots」という組み込み CA（Certificate Authority：認証局）証明書ストアだけを参照するため、MITM プロキシが持つ独自の CA 証明書を認識できません。curl が成功するのは curl がシステムの CA ストアを参照するからです。

修正の柱は 2 つです。①Cargo.toml で `rustls-tls` を `rustls-tls-native-roots` に切り替え、OS ネイティブ CA ストアを参照できるようにする。②それでも `SSL_CERT_FILE` や `CODEX_PROXY_CERT` という環境変数で指定された PEM ファイルは自動では読まれないため、`HttpConfig::build_client()` 内で明示的に読み込んで `add_root_certificate()` で追加する。あわせて、TLS エラーが発生したときに `--verbose` フラグで詳細なエラーチェーンを表示できるよう、エラー処理パイプラインも整備します。

```
OS/環境変数の CA 情報が reqwest に届くまでのデータフロー

SSL_CERT_FILE=/path/ca.pem ─────────────────────────────────┐
CODEX_PROXY_CERT=/path/ca.pem ──────────────────────────────┤
                                                             ↓
HttpConfig::build_client() {                        add_cert_from_path()
  EnvVar::get("SSL_CERT_FILE")  → Some(path) ──→  fs::read(path)
  EnvVar::get("CODEX_PROXY_CERT") → Some(path) ──→ Certificate::from_pem()
                                                   add_root_certificate()
}                                                            │
                                                             ↓
reqwest::ClientBuilder                              reqwest::Client
  + rustls-tls-native-roots                         (CA: webpki + OS CA + ENV CA)
  + add_root_certificate(env_cert)                            │
                                                             ↓
                                                      HTTPS 接続（MITM 対応）
```

*上の図では、CA 情報が環境変数 → `build_client` → `reqwest::Client` へと三段階で積み重なっていく様子を示しています。*

---

## `LIBRARY` reqwest — Rust の HTTP クライアント

reqwest は Rust でもっともよく使われる非同期 HTTP クライアントライブラリです。`plm` は GitHub API へのリクエストやマーケットプレイスへのアクセスにこのライブラリを使っています。`reqwest` の呼び出し窓口は `reqwest::Client` で、一度構築してから複数のリクエストで使い回す設計になっています。

`Client` を構築するには `Client::builder()` でビルダーオブジェクトを作り、各種オプションを「メソッドチェーン」でつなげ、最後に `.build()` を呼ぶパターンを使います。

```
Client::builder()         ← ビルダーを開始
  .user_agent("plm/...")  ← リクエストに付けるユーザーエージェント文字列
  .timeout(Duration)      ← タイムアウト設定
  .add_root_certificate() ← 追加 CA の指定（複数回呼べる）
  .build()                ← Client を構築（Result<Client, Error> を返す）
```

> **メモ**: `.build()` は `Result` を返します。エラーになる代表例は TLS 設定の不正な場合です。`plm` では `.unwrap_or_else(|_| Client::new())` でフォールバックしていますが、これはビルダー設定が間違っていても最低限の Client は得られるように、という保険です。

### `reqwest::ClientBuilder::add_root_certificate`

```
fn add_root_certificate(self, cert: Certificate) -> ClientBuilder
```

既存の CA ストアに、指定した CA 証明書を**追加**するメソッドです。「置き換え」ではないので、webpki-roots と OS CA はそのまま残り、さらに独自 CA が加わります。`self` を消費して `ClientBuilder` を返すため、必ずチェーンの中で再代入する必要があります。

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `cert` | `reqwest::Certificate` | PEM または DER 形式から構築した CA 証明書 |

> **注意**: このメソッドは1回の呼び出しで1枚の証明書しか追加できません。複数の CA が1つのファイルに結合されている（チェーン PEM）場合、`from_pem_bundle` などを使うか、ファイルを分割する必要があります。

---

## `CONCEPT` TLS と CA 証明書

TLS（Transport Layer Security）は HTTPS 通信を暗号化し、通信相手が本物かを確認する仕組みです。「本物かの確認」に使われるのが CA（Certificate Authority）です。CA は「この証明書は私が保証する」という署名を発行し、クライアント側には CA の公開鍵が事前にインストールされています（これが CA ストア）。

MITM プロキシは通信を中継するために一度 TLS を終端させて独自の CA で再署名します。クライアントが MITM プロキシの CA 証明書を知らなければ「知らない CA の署名だ」と判断してエラーを返します。

```
通常の HTTPS:
  クライアント ──TLS──→ サーバー
  (サーバーの証明書が既知の CA で署名されている → 信頼)

MITM プロキシ経由:
  クライアント ──TLS──→ プロキシ ──TLS──→ サーバー
  (プロキシが独自 CA で再署名) ← クライアントがこの CA を知らないとエラー！
```

`SSL_CERT_FILE` はこの「プロキシの CA 証明書」が入ったファイルを指す環境変数です。curl はデフォルトでこの環境変数を参照しますが、reqwest は参照しないため、今回の修正で明示的に読み込みます。

---

## `LIBRARY` rustls と TLS バックエンド

Rust の TLS 実装には複数の選択肢があります。`reqwest` は Cargo feature によってどの TLS ライブラリを使うかを切り替えられます。

```
reqwest の TLS feature 選択肢:

  rustls-tls              → webpki-roots のみ（組み込み CA、OS CA 参照なし）
  rustls-tls-native-roots → webpki-roots ＋ OS のネイティブ CA ストア
  rustls-tls-webpki-roots → webpki-roots のみ（rustls-tls と同じ）
  native-tls              → OS の TLS 実装（OpenSSL 等）
```

今回は `rustls-tls` から `rustls-tls-native-roots` に変更します。`rustls-native-certs` クレートが自動的に引き込まれ、Linux では `/etc/ssl/certs/`、macOS では Keychain、Windows では証明書ストアから CA を読み込みます。

> **メモ**: `rustls-tls-native-roots` を使っても `SSL_CERT_FILE` 環境変数は自動参照しません。`SSL_CERT_FILE` が指すファイルを利用するには、今回の修正のように `add_root_certificate()` で明示的に追加する必要があります。これは curl とは異なる挙動です。

```toml
# Cargo.toml の変更箇所
# before:
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"] }
# after:
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls-native-roots"] }
```

---

## `CONCEPT` PEM 形式と Certificate

PEM（Privacy-Enhanced Mail）は証明書や鍵を Base64 でエンコードしてテキストとして扱うフォーマットです。

```
-----BEGIN CERTIFICATE-----
MIIBpDCCAQugAwIBAgIUXXX...（Base64 エンコードされたバイナリ）
-----END CERTIFICATE-----
```

`-----BEGIN CERTIFICATE-----` と `-----END CERTIFICATE-----` で囲まれた1ブロックが1枚の証明書です。複数の証明書が1ファイルに並んでいる場合を「チェーン PEM」と呼びます。

### `reqwest::Certificate::from_pem`

```
fn from_pem(pem: &[u8]) -> Result<Certificate, Error>
```

バイト列（`&[u8]`）を受け取り、それが有効な PEM 形式の証明書であれば `reqwest::Certificate` を返します。PEM 形式でない場合（テキストファイルを誤って指定した等）は `Err` を返します。

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `pem` | `&[u8]` | PEM 形式の証明書のバイト列（`fs::read(path)` の結果をそのまま渡す） |

> **注意**: `from_pem` に渡すのは **ファイルパスではなくファイルの中身（バイト列）** です。先に `std::fs::read(path)` でファイルを読んでから渡します。

```rust
// 典型的な使い方（add_cert_from_path 内）
let pem_bytes = std::fs::read(path)?;          // ファイルを読む
let cert = reqwest::Certificate::from_pem(&pem_bytes)?; // PEM をパース
builder = builder.add_root_certificate(cert);  // ビルダーに追加
```

---

## `CONCEPT` 環境変数の読み方と `EnvVar::get`

`plm` では `std::env::var()` を直接呼ぶのではなく、`crate::env::EnvVar::get()` というラッパーを使います。このラッパーは環境変数が**未設定**の場合だけでなく、**空文字列に設定されている**場合も `None` を返します。

```
std::env::var("SSL_CERT_FILE"):
  未設定     → Err(VarError::NotPresent)
  空文字列   → Ok("")  ← 空パスとして扱いたいが、ファイルが開けないエラーになる

EnvVar::get("SSL_CERT_FILE"):
  未設定     → None
  空文字列   → None   ← 空文字列も「設定なし」として扱う
  パスあり   → Some("path/to/cert.pem")
```

これにより `if let Some(path) = EnvVar::get("SSL_CERT_FILE")` のパターンで安全に扱えます。

---

## `CONCEPT` エラーチェーンと `std::error::Error::source()`

Rust の `std::error::Error` トレイトには `source()` メソッドがあります。これは「このエラーの原因となった下位のエラー」を返します。複数の層を経てエラーが伝搬する場合、チェーン状になります。

```
reqwest::Error
  └─ source() → hyper::Error（HTTP プロトコル層のエラー）
                └─ source() → rustls::Error（TLS 層のエラー: "certificate not trusted"）
                              └─ source() → None（根本原因）
```

通常のエラー表示では `reqwest::Error` のメッセージしか見えません。`--verbose` フラグを使うと `ErrorFormatter` が全段のチェーンを展開して表示し、TLS 起因かどうかが一目でわかります。

### `with_source` — RichError に source を保持する

```rust
// PlmError::Network(e) の場合の変換
let message = e.to_string();
RichError::new(code, message)
    .with_context(ctx)
    .with_source(e)  // reqwest::Error 全体を source として保持する
```

`with_source(e)` で渡すのは `reqwest::Error` そのものです。これにより `ErrorFormatter` が `error.source()` を再帰的にたどって全チェーンを表示できます。`e.to_string()` で文字列化してしまうと source chain の情報が失われます。

> **注意**: `with_source` に渡す値は `Box<dyn std::error::Error + Send + Sync>` に変換できる型である必要があります。`reqwest::Error` はこれを満たします。

---

## `CONCEPT` `PlmError::General` — String エラーの橋渡し

`plm` のコマンドハンドラ（`install::run`, `list::run` など）は現在 `Result<(), String>` を返しています。一方、`main.rs` でリッチなエラー表示をするためには `PlmError` が必要です。そのための橋渡しが `PlmError::General(String)` です。

```
ハンドラ: Result<(), String>
            │
            │ .map_err(PlmError::General)   ← commands::dispatch 内
            ▼
          Result<(), PlmError>
            │
            │ .into()                       ← main.rs
            ▼
          RichError (code=Int001, message=元の文字列)
            │
            │ ErrorFormatter::new(verbose).format(...)
            ▼
          整形されたエラー文字列 → eprintln!
```

> **メモ**: `PlmError::General` はあくまで「現状のコードを最小限の変更で移行するための一時的な橋渡し」です。将来的には各ハンドラを `Result<(), PlmError>` に変更することで `Network` エラーの source chain も完全に保持できるようになります（今回はスコープ外）。

---

## `CONCEPT` `ErrorFormatter` と verbose 表示

`ErrorFormatter` は `RichError` を人間が読みやすい文字列にフォーマットするストラクトです。

```
ErrorFormatter::new(verbose)
  ├─ verbose=false → format_simple_plain()
  │    "error[NET001]: Network error: ..."
  │
  └─ verbose=true → format_verbose_plain()
       "error[NET001]: Network error: ..."
       "Cause: Connection failed"
       "Remediation: Check network settings"
       "Source chain:"
       "  |   - reqwest error: ..."
       "  |   - hyper error: ..."
       "  |   - rustls error: certificate not trusted"
```

### `ErrorFormatter::new`

```
fn new(verbose: bool) -> ErrorFormatter
```

`verbose` が `true` のとき、エラーの詳細（cause, remediation, source chain）をすべて表示します。`false` のときは1行のシンプルなメッセージのみ表示します。

---

## `CONCEPT` TDD（テスト駆動開発）サイクル

今回のタスクは TDD（Red → Green → Refactor）サイクルで進めます。

```
Red（失敗するテストを書く）
  ↓
Green（テストを通す最小限のコードを書く）
  ↓
Refactor（テストを通したまま、コードを改善する）
  ↓
Red（次のテストを書く）...
```

たとえば `add_cert_from_path` の実装では：

1. **Red**: 「ファイルが存在しないときでも `build_client` が成功する」テストを書く → コンパイルが通れば実行してテストが失敗することを確認
2. **Green**: `fs::read` 失敗を `eprintln!` して builder をそのまま返す最小実装 → テストが通る
3. **Refactor**: ログメッセージのフォーマットを統一するなど改善

---

## `TOOL` cargo について

```bash
cargo check                     # コンパイルエラーの確認（バイナリ生成なし、高速）
cargo test                      # テスト実行
cargo test config_test          # 特定のテストモジュールのみ実行
cargo clippy -- -D warnings     # 静的解析（警告をエラーとして扱う）
cargo fmt                       # コードフォーマット
cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs
                                # ライセンス一覧再生成（依存追加後に必要）
```

reqwest の feature を変更した後は必ず `cargo check` でコンパイルが通ることを確認し、`cargo about generate` で `THIRD_PARTY_LICENSES.md` を更新してください。

---

## `CONCEPT` `tempfile` クレートとテスト用一時ファイル

`tempfile::NamedTempFile` を使うと、テスト中だけ存在する一時ファイルを作れます。`NamedTempFile` がスコープを外れると自動的にファイルが削除されます。

```rust
use std::io::Write;
use tempfile::NamedTempFile;

let mut f = NamedTempFile::new().unwrap();
f.write_all(b"This is not a PEM file.").unwrap();
let path = f.path().to_str().unwrap().to_string();

std::env::set_var("SSL_CERT_FILE", &path);
let client = HttpConfig::default().build_client();
// ← テスト終了後、`f` がドロップされるとファイルが消える
```

> **注意**: `std::env::set_var` と `remove_var` はプロセス全体の環境変数を変更します。並列テスト実行時（Rust のデフォルト）に他のテストと競合する可能性があります。`serial_test` クレート（`#[serial]` アトリビュート）を使うか、環境変数を確実に元に戻すコードを書いてください。

---

## 用語集

| 用語 | 意味 |
|------|------|
| `MITM プロキシ` | Man-in-the-Middle プロキシ。通信を中継して検査・フィルタリングする。企業環境やクラウド環境でよく使われる |
| `CA（Certificate Authority）` | 証明書の署名者。「この公開鍵はこのドメインのものだ」を保証する第三者機関またはその証明書 |
| `PEM` | Privacy-Enhanced Mail。証明書や鍵をテキストとして扱うフォーマット。`-----BEGIN CERTIFICATE-----` で始まる |
| `TLS` | Transport Layer Security。HTTPS の暗号化・認証プロトコル |
| `webpki-roots` | Mozilla が管理する、ブラウザが信頼する CA のリスト。reqwest に組み込まれている |
| `rustls-native-certs` | OS のネイティブ CA ストアを rustls に読み込むクレート。`rustls-tls-native-roots` feature で自動的に使われる |
| `source chain` | エラーの原因チェーン。`std::error::Error::source()` をたどることで下位のエラーにアクセスできる |
| `RichError` | `plm` 独自のエラー型。エラーコード・コンテキスト・source chain を保持する |
| `ErrorFormatter` | `RichError` を人間が読める文字列にフォーマットするストラクト。verbose フラグで表示量を切り替える |
| `PlmError::General` | コマンドハンドラの `String` エラーを `PlmError` として伝搬するためのバリアント |
| `TDD` | Test-Driven Development。テストを先に書いてから実装する開発手法 |
| `tempfile` | テスト用の一時ファイルを作るクレート。スコープを外れると自動削除される |
| `rcgen` | Rust で自己署名証明書を生成するクレート。テスト用の有効な PEM を作るために使う |
