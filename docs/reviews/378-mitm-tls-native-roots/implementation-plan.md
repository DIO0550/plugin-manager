# MITM プロキシ環境での TLS 検証失敗を修正する

**関連Issue**: #378

MITM プロキシ環境（Codex Cloud 等）で `plm` の全 HTTP コマンドが `Network error` になる問題を修正する。
reqwest の TLS バックエンドをシステム CA を読むフィーチャーに切り替え、保険として環境変数 CA を明示追加し、`--verbose` 時のエラーチェーン表示を整備する。

## ユーザーレビューが必要な点

> **NOTE**
> - 破壊的変更はありません（API シグネチャ変更は `HttpConfig::build_client` のみ、シグネチャは変わらない）
> - 全コマンドハンドラの戻り型を `Result<(), String>` → `Result<(), PlmError>` に変更します。外部に公開されている API ではないため影響はバイナリのみです
> - `PlmError::General(String)` バリアントを追加します。これにより `all_plm_errors_have_explicit_mapping` テストの更新が必要です
> - reqwest feature 変更後に `cargo about generate` で `THIRD_PARTY_LICENSES.md` 再生成が必要です

## システム図

### 状態マシン / フロー図

CA 読み込みロジック（`build_client` 内）:

```
build_client() 呼び出し
        │
        ▼
┌───────────────────────────┐
│ Client::builder()         │
│  .user_agent()            │
│  .timeout()               │
└───────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────┐
│ SSL_CERT_FILE を EnvVar::get() で取得      │
│                                           │
│  Some(path) → fs::read(path)?             │
│    │                                      │
│    ├─ Ok(pem_bytes) → Certificate::from_pem │
│    │     │                                │
│    │     ├─ Ok(cert) → builder.add_root_certificate(cert) │
│    │     └─ Err(_)   → eprintln!("[warn] invalid PEM: SSL_CERT_FILE") │
│    └─ Err(_) → eprintln!("[warn] cannot read SSL_CERT_FILE")          │
│  None → スキップ                          │
└───────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────┐
│ CODEX_PROXY_CERT を EnvVar::get() で取得  │
│                                           │
│  Some(path) ─────────────────────────────┐│
│    │     ↑ SSL_CERT_FILE と同パス?         ││
│    │     └─ YES → スキップ（二重追加防止）  ││
│    │     NO → fs::read(path)?             ││
│    │     → Certificate::from_pem → add   ││
│  None → スキップ                          ││
└───────────────────────────────────────────┘
        │
        ▼
  builder.build()
        │
        ├─ Ok(client) → return client
        └─ Err(_)     → Client::new() (fallback)
```

### データフロー

```
plm コマンド実行（例: plm install owner/repo）
        │
        ▼
main.rs: Cli::parse()
  cli.verbose: bool ─────────────────────────────────────────┐
        │                                                     │
        ▼                                                     │
commands::dispatch(cli) → Result<(), PlmError>               │
        │                                                     │
        ├─ install::run(args) → Result<(), PlmError>          │
        │       │                                             │
        │       ▼                                             │
        │  install::download_plugin()                         │
        │       │                                             │
        │       ▼                                             │
        │  HostClientFactory::with_defaults()                 │
        │       │                                             │
        │       ▼                                             │
        │  HttpConfig::build_client()  ◀─ CA 注入ここ         │
        │       │                                             │
        │       ▼                                             │
        │  reqwest::Client (TLS: rustls-tls-native-roots)     │
        │       │                                             │
        │   HTTPS接続                                         │
        │       │                                             │
        │       ├─ 成功 → インストール続行                    │
        │       └─ 失敗 → PlmError::Network(reqwest::Error)   │
        │                                                     │
        ▼ (Err path)                                          │
PlmError → From<PlmError> for RichError (with_source追加)    │
        │                                                     │
        ▼                                                     ▼
main.rs: ErrorFormatter::new(verbose).format(&rich_error)  ◀─┘
        │
        ├─ verbose=false → "error[NET001]: Network error: ..."
        └─ verbose=true  → + "Cause:" + "Remediation:" + "Source chain:"
                                    └─ reqwest::Error → rustls error → TLS詳細
```

---

## 変更案

### 1. 依存関係変更

#### [MODIFY] `Cargo.toml`

reqwest の TLS バックエンドをシステム CA を読む variant に変更する。

- `rustls-tls` → `rustls-tls-native-roots`（OS ネイティブ CA + webpki-roots の両方を読む）

```toml
// before:
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"] }
```

```toml
// after:
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls-native-roots"] }
```

**補足**: `rustls-tls-native-roots` は `rustls-native-certs` クレートを transitively に引き込む。明示的な `rustls-native-certs` 依存は不要。変更後は `cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs` で THIRD_PARTY_LICENSES.md を再生成すること。

---

### 2. HTTP クライアント — CA 注入

#### [MODIFY] `src/config.rs`

`build_client()` に SSL_CERT_FILE / CODEX_PROXY_CERT からのカスタム CA 追加ロジックを追加する。

- `EnvVar::get("SSL_CERT_FILE")` / `EnvVar::get("CODEX_PROXY_CERT")` で環境変数を読む
- ファイル読み込み失敗・不正 PEM は `eprintln!` で警告を出して続行（致命的エラーにしない）
- 同じパスを二重に追加しない

```rust
// before:
use reqwest::Client;
use std::time::Duration;

impl HttpConfig {
    pub fn build_client(&self) -> Client {
        let mut builder = Client::builder().user_agent(&self.user_agent);

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }
}
```

```rust
// after:
use reqwest::Client;
use std::time::Duration;
use crate::env::EnvVar;

impl HttpConfig {
    pub fn build_client(&self) -> Client {
        let mut builder = Client::builder().user_agent(&self.user_agent);

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }

        // 保険: 環境変数から CA 証明書を明示追加
        // rustls-tls-native-roots でも SSL_CERT_FILE / CODEX_PROXY_CERT は自動参照されないため
        let ssl_cert_path = EnvVar::get("SSL_CERT_FILE");
        let codex_cert_path = EnvVar::get("CODEX_PROXY_CERT");

        let mut added_paths: Vec<String> = Vec::new();

        if let Some(ref path) = ssl_cert_path {
            builder = Self::add_cert_from_path(builder, path);
            added_paths.push(path.clone());
        }

        if let Some(ref path) = codex_cert_path {
            if !added_paths.contains(path) {
                builder = Self::add_cert_from_path(builder, path);
            }
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }

    /// PEM ファイルから証明書を読み込んで ClientBuilder に追加する。
    /// 読み込み失敗・パース失敗は eprintln! で警告してそのまま builder を返す。
    fn add_cert_from_path(
        builder: reqwest::ClientBuilder,
        path: &str,
    ) -> reqwest::ClientBuilder {
        match std::fs::read(path) {
            Err(e) => {
                eprintln!("[plm warn] cannot read CA certificate file '{}': {}", path, e);
                builder
            }
            Ok(pem_bytes) => match reqwest::Certificate::from_pem(&pem_bytes) {
                Err(e) => {
                    eprintln!("[plm warn] invalid PEM in '{}': {}", path, e);
                    builder
                }
                Ok(cert) => builder.add_root_certificate(cert),
            },
        }
    }
}
```

**追加 import**: `use crate::env::EnvVar;`（`config.rs` の先頭に追加）

---

### 3. エラー型 — source chain 保持

#### [MODIFY] `src/error.rs`

`From<PlmError> for RichError` の `Network` ブランチに `.with_source()` を追加して reqwest エラーの source chain を RichError に保持する。また全コマンドハンドラから上がってくる文字列エラーを受け取るための `PlmError::General(String)` バリアントを追加する。

**変更点**:
1. `PlmError::General(String)` バリアントを追加
2. `From<PlmError> for RichError` を `match &err` → `match err`（所有）に変更し、`Network` ブランチで `.with_source(e)` を呼ぶ
3. `PlmError::General` の `From` マッピングを追加

```rust
// before (抜粋):
#[derive(Debug, Error)]
pub enum PlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    // ... 他のバリアント
}

impl From<PlmError> for RichError {
    fn from(err: PlmError) -> Self {
        let (code, message, context) = match &err {
            PlmError::Network(e) => {
                let mut ctx = ErrorContext::default();
                if let Some(url) = e.url() {
                    ctx.url = Some(url.to_string());
                }
                let code = if e.is_timeout() {
                    ErrorCode::Net002
                } else {
                    ErrorCode::Net001
                };
                (code, e.to_string(), ctx)
            }
            // ... 他のブランチ（&err を借用）
        };
        RichError::new(code, message).with_context(context)
    }
}
```

```rust
// after (抜粋):
#[derive(Debug, Error)]
pub enum PlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    // ... 他のバリアント（変更なし）

    /// コマンドハンドラの String エラーを PlmError として伝搬するためのバリアント
    #[error("{0}")]
    General(String),
}

impl From<PlmError> for RichError {
    fn from(err: PlmError) -> Self {
        // Network は所有権を取って source chain を保持する
        if let PlmError::Network(e) = err {
            let mut ctx = ErrorContext::default();
            if let Some(url) = e.url() {
                ctx.url = Some(url.to_string());
            }
            let code = if e.is_timeout() {
                ErrorCode::Net002
            } else {
                ErrorCode::Net001
            };
            let message = e.to_string();
            return RichError::new(code, message)
                .with_context(ctx)
                .with_source(e); // ← source chain 保持
        }

        // 他のバリアントは借用で処理（変更なし）
        let (code, message, context) = match &err {
            PlmError::Network(_) => unreachable!(),
            PlmError::General(s) => (ErrorCode::Int001, s.clone(), ErrorContext::default()),
            PlmError::RepoApi { url, status, message } => {
                // ... 既存ロジックそのまま
            }
            // ... 他の既存ブランチ（変更なし）
        };
        RichError::new(code, message).with_context(context)
    }
}
```

---

### 4. コマンドディスパッチ — 戻り型変更

#### [MODIFY] `src/commands.rs`

`dispatch` の戻り型を `Result<(), String>` から `Result<(), PlmError>` に変更する。
各コマンドハンドラが返す `Result<(), String>` を `PlmError::General` で包む。

- コマンドハンドラ自体の戻り型は変更しない（ハンドラは引き続き `Result<(), String>` を返す）
- ハンドラの String エラーを dispatch 内で `PlmError::General` に変換

```rust
// before:
pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), String> {
    match cli.command {
        Some(Command::Install(args)) => deploy::install::run(args).await,
        // ...
        None => run_default(std::io::stdout().is_terminal()).await,
    }
}
```

```rust
// after:
use crate::error::PlmError;

pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), PlmError> {
    let result: Result<(), String> = match cli.command {
        Some(Command::Target(args)) => manage::target::run(args).await,
        Some(Command::Install(args)) => deploy::install::run(args).await,
        Some(Command::List(args)) => list::run(args).await,
        Some(Command::Info(args)) => info::run(args).await,
        Some(Command::Enable(args)) => lifecycle::enable::run(args).await,
        Some(Command::Disable(args)) => lifecycle::disable::run(args).await,
        Some(Command::Uninstall(args)) => lifecycle::uninstall::run(args).await,
        Some(Command::Update(args)) => lifecycle::update::run(args).await,
        Some(Command::Init(args)) => manage::init::run(args).await,
        Some(Command::Pack(args)) => manage::pack::run(args).await,
        Some(Command::Link(args)) => deploy::link::run(args).await,
        Some(Command::Unlink(args)) => deploy::unlink::run(args).await,
        Some(Command::Sync(args)) => deploy::sync::run(args).await,
        Some(Command::Import(args)) => deploy::import::run(args).await,
        Some(Command::Marketplace(args)) => manage::marketplace::run(args).await,
        Some(Command::Managed) => manage::managed::run().await,
        None => run_default(std::io::stdout().is_terminal()).await,
    };
    result.map_err(PlmError::General)
}
```

**注意**: `run_default` も `Result<(), String>` を返すため、上記の `.map_err(PlmError::General)` で一括変換される。

> **設計上の補足 (WARN)**: 各コマンドハンドラ内で `install::download_plugin().await.map_err(|e| e.to_string())?` のように `PlmError::Network` が String に変換されてしまうケースがまだ残る。この場合、`dispatch` に戻ってきた時点では既に `PlmError::General(String)` になっており、verbose 時の source chain は得られない。**本修正で verbose 表示の対象となるのは、ハンドラが `PlmError` を String 変換せずに直接返した場合のみ**（現状は `?` 演算子で `PlmError::Network` が直接伝搬するケースがある）。完全な verbose 対応には全ハンドラの String 変換除去が必要だが、それは今回のスコープ外とする。

---

### 5. エントリポイント — verbose + ErrorFormatter 接続

#### [MODIFY] `src/main.rs`

`cli.verbose` を事前に取り出し、`PlmError` を `RichError` → `ErrorFormatter` で整形して表示する。

```rust
// before:
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = commands::dispatch(cli).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
```

```rust
// after:
use crate::error::{RichError, formatter::ErrorFormatter};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    if let Err(plm_err) = commands::dispatch(cli).await {
        let rich: RichError = plm_err.into();
        let formatted = ErrorFormatter::new(verbose).format(&rich);
        eprintln!("{formatted}");
        std::process::exit(1);
    }
}
```

**追加 import**:
```rust
use crate::error::RichError;
use crate::error::formatter::ErrorFormatter;
```

**注意**: `formatter` モジュールを `pub` にする必要がある場合は `src/error.rs` で `pub mod formatter;` を確認する（現在は `mod formatter;` の可能性あり）。

---

### 6. エラーモジュール公開設定の確認

#### [MODIFY] `src/error.rs` (モジュール公開)

`ErrorFormatter` が `main.rs` から参照できるよう、`formatter` サブモジュールの公開設定を確認・修正する。

```rust
// 確認・修正箇所（error.rs の先頭部分）
mod code;
mod rich;
mod formatter;  // ← pub mod に変更が必要な場合

pub use code::ErrorCode;
pub use rich::{ErrorContext, RichError};
pub use formatter::ErrorFormatter;  // ← 追加
```

---

## 検証計画

### テスト戦略

**機能タイプ**: Configuration（CA 読み込み）+ Pure Logic（エラーチェーン整形）
**テスト方針**: TDD（Configuration + Pure Logic を含むため）
**根拠**: CA ファイル読み込みロジックは Pure Logic で入力（パス文字列）→出力（ClientBuilder 状態）が確定しており、テスト可能。エラーチェーン整形も同様にユニットテストで網羅できる。実 MITM 環境 E2E は手動検証で補完。

### 自動テスト

#### `src/config_test.rs`

**役割**: `HttpConfig::build_client()` の CA 注入ロジックを検証する — 環境変数の有無・パスの正当性・二重追加防止の各ケースをカバーする

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | SSL_CERT_FILE 未設定で build_client | 通常環境（プロキシなし） | エラーなし・クライアントが構築される |
| 正常系 | SSL_CERT_FILE に有効 PEM ファイルを設定して build_client | MITM 環境で CA ファイルを指定 | エラーなし・クライアントが構築される |
| 正常系 | CODEX_PROXY_CERT のみ設定して build_client | Codex Cloud 環境で CODEX_PROXY_CERT のみ | エラーなし・クライアントが構築される |
| 正常系 | SSL_CERT_FILE と CODEX_PROXY_CERT に異なるパスを設定 | 両方の CA を追加するケース | エラーなし・クライアントが構築される |
| 境界値 | SSL_CERT_FILE と CODEX_PROXY_CERT が同一パスを指す | 同じ CA ファイルへの二重指定 | 二重追加されない・クライアントが構築される |
| 境界値 | SSL_CERT_FILE が空文字列 | 環境変数が設定されているが値が空 | EnvVar::get がNoneを返す・スキップ |
| 異常系 | SSL_CERT_FILE が存在しないパスを指す | CA ファイルが削除済み・パス誤り | eprintln!で警告・クライアントは構築される |
| 異常系 | SSL_CERT_FILE に不正 PEM（テキストファイル等）を設定 | PEM 形式でないファイルを誤って指定 | eprintln!で警告・クライアントは構築される |
| エッジケース | CODEX_PROXY_CERT に不正 PEM を設定（SSL_CERT_FILE は正常） | 片方が壊れている | SSL_CERT_FILE の CA は追加・CODEX_PROXY_CERT は警告でスキップ |

**実装補足**: `tempfile` クレートが dev-dependencies にあるため、テスト用の一時 PEM ファイルを作成可能。ただし `reqwest::Certificate::from_pem` が有効な PEM フォーマットを要求するため、テスト用 PEM は自己署名証明書の PEM を使用する（または openssl コマンドで生成したものを base64 embed）。実際には、「ファイルが読める」「PEM として認識される」ことを確認するため、簡易的な PEM 内容を使う。

> **テスト実装上の注意**: `build_client()` は実際に `reqwest::Client` を構築するため、環境変数を設定してテストする際は `std::env::set_var` を使う。並列テスト実行時に環境変数が競合する可能性があるため、`serial_test` クレートの利用か、`tempfile + 固定パス` での分離を検討する。

#### `src/error/formatter.rs` (内部 `#[cfg(test)]` への追加)

> **注意**: 既存のテストは `formatter.rs` 内の `#[cfg(test)]` ブロックにある。新規テストは `formatter.rs` 内に追加する（CLAUDE.md の `*_test.rs` 分離方針だが、既存が inline であるため既存パターンに従う。`formatter_test.rs` は**作成しない**）。

**役割**: `ErrorFormatter` が `verbose=true` 時に source chain を正しく展開・表示することを検証する

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | verbose=true・source chain なし | verbose モードだが source なし | "Source chain:" セクションが出力されない |
| 正常系 | verbose=true・source chain あり（1段） | io::Error を source に持つ RichError | "Source chain:" + エラーメッセージが含まれる |
| 正常系 | verbose=true・source chain あり（2段以上） | 複数ネストした source chain | 各 source が "  |   - " 形式で列挙される |
| 正常系 | verbose=false・source chain あり | 通常モードでは source chain を非表示 | "Source chain:" が出力されない |
| 境界値 | source chain が循環する（理論上） | 実際には起きないが Rust の std::error::Error は保証しない | 無限ループしない（実装で上限を設ける場合は確認） |

#### `src/error.rs` (内部 `#[cfg(test)]` への追加)

**役割**: `From<PlmError> for RichError` の `Network` ブランチが reqwest::Error の source chain を `RichError.source()` として保持することを検証する

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | PlmError::General("msg") → RichError | 文字列エラーの変換 | code=Int001, message="msg" |
| 正常系 | PlmError::Network → RichError の code | タイムアウトでない Network エラー | code=Net001 |
| 正常系 | PlmError::Network(timeout) → RichError の code | タイムアウトの Network エラー | code=Net002 |
| 正常系 | PlmError::General → all_errors_have_explicit_mapping テストの更新 | General バリアント追加後 | テストが継続してパス |

**テスト実行コマンド**

```bash
cargo test
cargo test --test config_test
cargo test config_test
cargo test error
```

### 手動検証

**前提**: MITM プロキシ環境（または MITM 証明書付きローカルプロキシ `mitmproxy` 等）

1. **修正前の確認** (回帰テスト用): 修正前のコードで MITM 環境から `plm marketplace add <url>` を実行し、`Network error` が発生することを確認
2. **修正後の確認**:
   - `cargo build --release`
   - MITM プロキシ環境から `SSL_CERT_FILE=/path/to/mitm-ca.pem plm marketplace add <url>` を実行
   - Network error なく成功することを確認
3. **CODEX_PROXY_CERT の確認**:
   - `CODEX_PROXY_CERT=/path/to/mitm-ca.pem plm install <repo>` で同様に成功することを確認
4. **通常環境での回帰確認**:
   - プロキシなし環境で `plm install <public-github-repo>` が成功することを確認
5. **verbose エラーチェーン確認**:
   - TLS エラーが発生する状況で `plm --verbose install <repo>` を実行
   - `Source chain:` セクションに TLS 起因のエラーが表示されることを確認
6. **警告メッセージ確認**:
   - `SSL_CERT_FILE=/nonexistent/path.pem plm install <repo>` を実行
   - `[plm warn] cannot read CA certificate file` が stderr に表示され、コマンドは継続することを確認

---

## Definition of Done

以下をすべて満たした時点で本機能の実装完了とする。

- [ ] すべてのタスク（tasks.md）が ■ になっている
- [ ] `cargo build` が成功する
- [ ] `cargo clippy -- -D warnings` が警告なしで通る
- [ ] `cargo fmt --check` がフォーマット違反なしで通る
- [ ] `cargo test` が全テストパスする
- [ ] MITM プロキシ環境（または `SSL_CERT_FILE` 設定環境）で HTTP コマンドが成功する
- [ ] 通常環境（プロキシなし）で既存動作に回帰がない
- [ ] `plm --verbose` 時に TLS 起因エラーのチェーンが表示される
- [ ] CA ファイル不在時に致命的エラーにならず警告で継続する
- [ ] `THIRD_PARTY_LICENSES.md` が `cargo about generate` で再生成されている
- [ ] `cargo deny check` が実行可能（informational）
