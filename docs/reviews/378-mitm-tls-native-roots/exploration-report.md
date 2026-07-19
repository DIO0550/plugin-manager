# Codebase Exploration Report: MITM プロキシ環境での TLS 検証失敗修正

**探索目的**: reqwest の TLS バックエンドを `rustls-tls-native-roots` に切り替え、MITM プロキシ環境（SSL_CERT_FILE / CODEX_PROXY_CERT 使用）で TLS 検証が通るように修正する。あわせて `--verbose` 時のエラーチェーン表示を整備する。

---

## 0. エグゼクティブサマリー

**重要な発見（Top 5）**:
1. `Cargo.toml` L10 の `rustls-tls` feature が根本原因。`rustls-tls-native-roots` へ 1 行変更するだけで webpki ルート束縛を解除できる
2. `HttpConfig::build_client()` はシンプルな 5 行の Builder。`add_root_certificate()` を呼ぶ保険ロジック挿入箇所として最適
3. `EnvVar::get()` ユーティリティが既にあり、空文字フィルタリング済み。SSL_CERT_FILE / CODEX_PROXY_CERT 読み込みに再利用できる
4. **`--verbose` フラグは `Cli::verbose` として定義されているが、エラー表示パスへ一切伝達されていない**。`commands::dispatch` の戻り型が `Result<(), String>` で、PlmError は各コマンドハンドラで `.map_err(|e| e.to_string())` に変換されてしまう
5. `ErrorFormatter`（`src/error/formatter.rs`）は verbose 対応の `format_source_chain()` を実装済みだが、現在は `main.rs` の実際のエラー表示パスと **未接続**。また `From<PlmError> for RichError` の `Network` ブランチが `.with_source(e)` を呼んでいないため、源泉チェーンが切れている

**推奨される次のステップ**:
- `rustls-tls` → `rustls-tls-native-roots` の変更が最優先（単一行変更で影響大）
- エラー表示改善は `main.rs` で `cli.verbose` + `ErrorFormatter` を接続する設計変更が必要（`dispatch` の戻り型変更を伴う可能性あり）
- `From<PlmError> for RichError` の `Network` ブランチへ `.with_source()` 追加が必要

---

## 1. アーキテクチャ概要

### 1.1 ディレクトリ構造

```
plm/
├── Cargo.toml               ← reqwest feature 定義
├── src/
│   ├── main.rs              ← エントリポイント（エラー表示: eprintln!("{err}")）
│   ├── cli.rs               ← Cli struct（verbose: bool あり）
│   ├── commands.rs          ← dispatch() → Result<(), String>
│   ├── config.rs            ← HttpConfig + build_client() ← 主要修正箇所
│   ├── config_test.rs       ← HttpConfig ユニットテスト
│   ├── env.rs               ← EnvVar::get() ユーティリティ
│   ├── error.rs             ← PlmError + From<PlmError> for RichError
│   ├── error/
│   │   ├── code.rs          ← ErrorCode (Net001/Net002 等)
│   │   ├── rich.rs          ← RichError + ErrorContext
│   │   └── formatter.rs     ← ErrorFormatter（verbose 対応、未接続）
│   ├── host.rs              ← HostClientFactory（HttpConfig を保持）
│   ├── host/
│   │   └── github.rs        ← GitHubClient（config.build_client() を呼ぶ）
│   └── commands/
│       └── deploy/
│           └── install.rs   ← 各コマンドハンドラ（PlmError を String に変換）
└── .github/workflows/
    └── ci.yml               ← cargo check / fmt / clippy / test / cargo deny
```

**構造の特徴**:
- Feature ベースモジュール構成（レイヤー分離なし）
- テストは `*_test.rs` に分離（CLAUDE.md 方針）
- `error/` はサブモジュールとして code/rich/formatter に分割済み

### 1.2 主要ファイル

| ファイルパス | 役割 | 重要度 |
|-------------|------|--------|
| `Cargo.toml` | reqwest feature 定義（`rustls-tls` → 修正対象） | 高 |
| `src/config.rs` | `HttpConfig::build_client()` — CA 追加ロジック挿入箇所 | 高 |
| `src/error.rs` | `PlmError` / `From<PlmError> for RichError` — source chain 接続が必要 | 高 |
| `src/error/formatter.rs` | `ErrorFormatter` — verbose 対応済み、main.rs と未接続 | 高 |
| `src/main.rs` | エラー表示（`eprintln!("{err}")`）— verbose フラグの接続先 | 高 |
| `src/env.rs` | `EnvVar::get()` — 環境変数読み取りユーティリティ | 中 |
| `src/host.rs` | `HostClientFactory` — `HttpConfig` を保持・伝達 | 中 |
| `src/host/github.rs` | `GitHubClient::new()` — `config.build_client()` 呼び出し | 中 |

### 1.3 レイヤー構成

```
main.rs
    └── commands::dispatch(cli)
            └── deploy::install::run(args) / manage::marketplace::run(args) / ...
                    └── install::download_plugin() / source::fetch() / ...
                            └── HttpConfig::build_client() → reqwest::Client
                                    └── GitHubClient / HTTPダウンロード
```

**各層の責務**:
- `main.rs`: エラー表示（現在 `eprintln!("{err}")`）
- `commands/`: ユーザー向けコマンドハンドラ（PlmError → String 変換）
- `config.rs`: HTTP クライアント構築（CA 注入の責務）
- `host/`: ホスト固有 HTTP クライアント

### 1.4 依存関係

```
main.rs
    ├── cli.rs (Cli, verbose)
    └── commands.rs (dispatch)
            ├── config.rs (HttpConfig)
            └── error.rs (PlmError)
                    └── error/ (ErrorCode, RichError, ErrorFormatter)
```

**循環依存**: なし
**主要な外部依存**: reqwest 0.12（`rustls-tls` → 修正対象）、thiserror 2、owo-colors 4

---

## 2. 関連コード分析

### 2.1 変更対象に関連する既存コード

| ファイルパス | 関連内容 | 関連度 |
|-------------|---------|--------|
| `Cargo.toml` | reqwest features `["json", "stream", "rustls-tls"]` | 高 |
| `src/config.rs` | `build_client()` — ClientBuilder の操作箇所 | 高 |
| `src/error.rs` | `From<PlmError> for RichError` の Network ブランチ | 高 |
| `src/error/formatter.rs` | `ErrorFormatter::format_source_chain()` — 既存実装 | 高 |
| `src/main.rs` | `eprintln!("{err}")` — 接続先 | 高 |
| `src/env.rs` | `EnvVar::get()` — 再利用可能 | 中 |
| `src/config_test.rs` | `HttpConfig` テスト — 新テスト追加箇所 | 中 |

### 2.2 再利用可能なパターン

#### パターン: EnvVar::get()

**場所**: `src/env.rs`
**概要**: 環境変数を読み取り、空文字列は `None` として扱うユーティリティ
**再利用方法**: `SSL_CERT_FILE` / `CODEX_PROXY_CERT` の読み取りに直接使用可能

```rust
/// 環境変数ユーティリティ
pub struct EnvVar;

impl EnvVar {
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|s| !s.is_empty())
    }
}
```

#### パターン: ClientBuilder チェーン（既存）

**場所**: `src/config.rs`
**概要**: `Client::builder()` に設定を段階的に適用する Builder パターン
**再利用方法**: `add_root_certificate()` を同パターンで追加可能

```rust
pub fn build_client(&self) -> Client {
    let mut builder = Client::builder().user_agent(&self.user_agent);

    if let Some(timeout) = self.timeout {
        builder = builder.timeout(timeout);
    }

    builder.build().unwrap_or_else(|_| Client::new())
}
```

#### パターン: ErrorFormatter（verbose 対応済み）

**場所**: `src/error/formatter.rs`
**概要**: `RichError` を verbose / 非 verbose モードで整形。`format_source_chain()` で `std::error::Error::source()` チェーンを展開する
**再利用方法**: `main.rs` で `ErrorFormatter::new(cli.verbose).format(&rich_error)` として接続

```rust
pub struct ErrorFormatter {
    verbose: bool,
    use_color: bool,
}

impl ErrorFormatter {
    pub fn new(verbose: bool) -> Self { ... }
    pub fn format(&self, error: &RichError) -> String {
        if self.verbose {
            self.format_verbose_plain(error)   // Cause + Remediation + Source chain 含む
        } else {
            self.format_simple_plain(error)    // error[CODE]: message のみ
        }
    }
    fn format_source_chain(&self, error: &RichError) -> String {
        let mut chain = Vec::new();
        let mut current: Option<&(dyn std::error::Error)> = error.source();
        while let Some(err) = current {
            chain.push(format!("  |   - {}", err));
            current = err.source();
        }
        chain.join("\n")
    }
}
```

### 2.3 類似実装の参考例

#### 参考: `EnvVar::get()` の使用例（github.rs）

**実装ファイル**: `src/host/github.rs`
**類似点**: 環境変数からトークンを読んでクライアント動作を変える
**参考になる点**: `SSL_CERT_FILE` / `CODEX_PROXY_CERT` 読み取りの設計が同じパターンを踏襲できる

```rust
fn get_token(&self) -> Option<String> {
    if let Some(token) = self.auth.github_token() {
        return Some(token.to_string());
    }
    if let Some(token) = EnvVar::get("GITHUB_TOKEN") {
        return Some(token);
    }
    self.get_token_from_cli()
}
```

### 2.4 命名規則・コーディングスタイル

- **ファイル命名**: snake_case（`config.rs`, `config_test.rs`）
- **変数命名**: snake_case（`build_client`, `user_agent`）
- **インデント**: 4 スペース（rustfmt 準拠）
- **テスト分離**: `#[cfg(test)] #[path = "config_test.rs"] mod tests;` パターン

### 2.5 既存構造の問題点・技術的負債

| 問題のあるパターン / 場所 | 問題の内容 | 新実装での扱い |
|------------------------|-----------|--------------|
| `From<PlmError> for RichError`（`error.rs`）| Network ブランチが `.with_source(e)` を呼ばないため、reqwest のエラーソースチェーンが RichError に保持されない | 修正する（`with_source(e)` 追加） |
| `commands::dispatch` 戻り型 `Result<(), String>` | PlmError が String に変換されてしまい、main.rs で `verbose` を使った詳細表示が不可能 | エラー表示改善のために変更を検討（または main.rs 側で対応） |
| `main.rs` の `eprintln!("{err}")` | verbose フラグを全く考慮しない単純表示 | `ErrorFormatter::new(cli.verbose)` を接続する修正が必要 |

**「修正する」とした場合の代替方針**: `dispatch` の戻り型を `Result<(), PlmError>` に変更し、`main.rs` で `ErrorFormatter` を使った表示に統一する。ただし全コマンドハンドラの戻り型も変更が必要となるため、段階的対応として「main.rs で PlmError を受け取れるように dispatch のみ変更する」方式が現実的。

---

## 3. 技術的制約・リスク

### 3.1 既存の制約

**型システム・リンター**:
- Rust 2021 edition
- `cargo clippy -- -D warnings`（警告をエラー扱い）。追加する `impl` で clippy WARN が出ないよう注意
- `#[allow(dead_code)]` が `main.rs` に付いているため、未使用コードの警告は抑制されているが、新規追加コードへの影響は個別確認が必要

**ビルド設定**:
- ビルドツール: cargo
- `cargo deny check` (informational, CI では失敗扱いなし)
- 依存変更時は `THIRD_PARTY_LICENSES.md` 再生成が必要: `cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs`

### 3.2 互換性の問題

| ライブラリ | バージョン | リスク |
|-----------|----------|--------|
| reqwest | 0.12 | `rustls-tls` → `rustls-tls-native-roots` の feature 切り替えでバイナリサイズが微増する可能性（rustls-native-certs の追加）|
| rustls-native-certs | (reqwest 経由) | プラットフォームによって CA ストアの場所が異なる（Linux: /etc/ssl/certs, macOS: Security.framework, Windows: Cert Store）。CI (ubuntu-latest) では問題なし |

**`rustls-tls-native-roots` の動作**:
- システム証明書ストア（OS ネイティブ）+ webpki-roots の両方を読む（`rustls-tls-native-roots` は両方含む）
- `SSL_CERT_FILE` は **reqwest の rustls-tls-native-roots では読まれない**。reqwest の rustls バックエンドは OpenSSL と違い、`SSL_CERT_FILE` を自動参照しない。そのため「保険」（`add_root_certificate()` での明示追加）は引き続き必要

### 3.3 パフォーマンスボトルネック

- `build_client()` 呼び出しは起動時 / クライアント構築時のみ。CA ファイルの PEM 読み込みは I/O を伴うが、ユーザー許容範囲

### 3.4 セキュリティ考慮点

- 証明書ファイルのパスを `mask_sensitive_paths()` がマスクする可能性あり（`.env` や `credentials` を含むパスのみ対象なので SSL_CERT_FILE の典型パスは問題なし）
- 不正 PEM を読んだ際の `warn!` は `eprintln!` か `log` クレートか要検討（現在 `log` クレートは `Cargo.toml` にない — `tracing` もない）。単純な `eprintln!` で warn 相当のメッセージを出力するか、stderr への直接出力にする

---

## 4. 変更影響範囲

### 4.1 波及ファイル

**直接影響**（修正が必須）:

| ファイルパス | 理由 | 影響の種類 |
|-------------|------|-----------|
| `Cargo.toml` | `rustls-tls` → `rustls-tls-native-roots` | 修正 |
| `src/config.rs` | `build_client()` に CA 追加ロジック追加 | 修正 |
| `src/error.rs` | `From<PlmError> for RichError` の Network ブランチに `.with_source()` 追加 | 修正 |
| `src/main.rs` | verbose + ErrorFormatter 接続 | 修正 |

**間接影響**（確認が必要）:

| ファイルパス | 理由 | 確認内容 |
|-------------|------|---------|
| `src/config_test.rs` | `build_client()` の挙動変化 | 既存テストが CA 追加ロジックと干渉しないか確認 |
| `src/commands.rs` | `dispatch` の戻り型変更を検討する場合 | 全コマンドハンドラへの波及 |
| `THIRD_PARTY_LICENSES.md` | reqwest feature 変更で依存クレートが変わる可能性 | `cargo about generate` で再生成 |

### 4.2 テスト範囲

**既存テストファイル**:

| テストファイルパス | テスト対象 | 修正の必要性 |
|------------------|----------|------------|
| `src/config_test.rs` | `HttpConfig::default()`, `build_client()` の基本動作 | 中（CA 追加のテスト追加が必要） |
| `src/error/formatter.rs` 内テスト | `ErrorFormatter` の verbose / 非 verbose 出力 | 低（既存テストは壊さない） |
| `src/error.rs` 内テスト | `From<PlmError> for RichError` の変換 | 中（`with_source` 追加後の確認） |

**新規テストの必要性**:
- [x] ユニットテスト: CA ファイル読み込みロジック（`build_client` の CA 追加）
- [x] ユニットテスト: エラーチェーン整形（`ErrorFormatter` + source chain）
- [ ] 統合テスト: MITM プロキシ E2E（手動検証手順で代替）

### 4.3 破壊的変更の可能性

| API / 関数 | 変更内容 | 影響範囲 |
|-----------|---------|---------|
| `HttpConfig::build_client()` | CA 追加ロジック追加（シグネチャ変更なし） | シグネチャ変化なし、動作変化は通常環境では影響なし |
| `commands::dispatch` 戻り型（検討中） | `Result<(), String>` → `Result<(), PlmError>` | 全コマンドハンドラ（`src/commands/**`）の戻り型変更が必要 |

### 4.4 移行計画の必要性

- 段階的リリース: 不要（後方互換を維持）
- ロールバック計画: `Cargo.toml` の feature を戻すだけ

---

## 5. テストインフラストラクチャ

### 5.1 テスト環境

- **テストフレームワーク**: cargo test（Rust 標準）
- **テストランナーコマンド**: `cargo test`
- **アサーションライブラリ**: Rust 標準 `assert!` / `assert_eq!`
- **モックライブラリ**: なし（本番コードと同じ構造体を使うか、仮ファイルを tempfile で作成）
- **プロパティベーステスト**: `proptest = "1"` が dev-dependencies に存在（`repo_proptests.rs` で使用）

### 5.2 テストファイル構成

- **配置パターン**: `*_test.rs` に分離（`#[cfg(test)] #[path = "config_test.rs"] mod tests;`）
- **命名規則**: `{module_name}_test.rs`
- **テストヘルパー**: 特定の共通セットアップファイルなし。各テストファイルで必要なヘルパーを定義

### 5.3 既存テストパターン

| テストファイル | テスト対象 | パターン |
|-------------|----------|---------|
| `src/config_test.rs` | `HttpConfig::default()`, `AuthProvider` | Unit |
| `src/error.rs` 内 tests | `PlmError → RichError` 変換 | Unit |
| `src/error/formatter.rs` 内 tests | `ErrorFormatter` の出力形式 | Unit |
| `src/error/rich.rs` 内 tests | `RichError` 生成・source chain | Unit |
| `src/cli_test.rs` | CLI パース（`assert_cmd`） | Integration |

**注意**: `error.rs` のテストは現在 `error.rs` 本体内に `#[cfg(test)]` で書かれているが（AGENTS.md の旧スタイル）、CLAUDE.md の `*_test.rs` 分離方針が優先。新規テストは `config_test.rs` に追加する。

### 5.4 カバレッジ・CI

- **カバレッジツール**: 設定なし
- **CI テストジョブ**: `.github/workflows/ci.yml` — check / fmt / clippy / test / audit（cargo deny）
- **cargo deny**: informational モードで実行（CI 失敗なし）

---

## 6. 追加調査が必要な項目

- [x] `rustls-tls-native-roots` は webpki-roots を含むか確認 → 含む（`rustls-tls-native-roots` は native + webpki の両方。`rustls-tls-native-roots-only` が native のみ）
- [x] `reqwest::Certificate::from_pem()` の API 確認 → `reqwest::Certificate` は `from_pem(&[u8])` を持つ（PEM ファイルを読んで渡す）
- [x] `eprintln!` 相当のログ出力: `log` クレートは `Cargo.toml` にない → `eprintln!` で代替（`stderr` 直接出力）
- [x] `commands::dispatch` 変更の影響: 全コマンドハンドラが `Result<(), String>` を返す → 段階的対応として `PlmError` を `RichError` に変換して main.rs で処理する設計を採用

---

## 7. ユーザー判断が必要な論点

> 全ての論点は確定済み（クラウドエージェント実行のため、「確定したヒアリング回答」に記載された内容で解決）

### 論点 1: dispatch 戻り型の変更範囲

- **背景**: verbose エラー表示を実現するには `PlmError` を main.rs まで伝達する必要がある。`dispatch` が `Result<(), String>` を返す現状では `verbose` に応じた表示が不可能
- **選択肢**:
  - A: `dispatch` を `Result<(), PlmError>` に変更し、main.rs で `ErrorFormatter` 接続（全コマンドハンドラ変更が必要）
  - B: `dispatch` を `Result<(), String>` のまま維持し、エラーメッセージに TLS ヒントを追加する限定的な改善に留める
- **決定（ヒアリング回答）**: A を選択。`--verbose` 時のエラーチェーン表示を実施。ただし、コマンドハンドラは `PlmError` を返すよう変更するのではなく、**main.rs で PlmError を受け取れるよう dispatch を変更し、その中でエラーチェーンを整形する設計**が最小変更で実現できる
- **重要度**: 高（`--verbose` 要件を満たすために必要）

### 未確認論点（全て解消済み）

- 保険 CA 読み込み失敗時の挙動: `warn!`（eprintln!相当）を出して続行（致命的エラーにしない）— ヒアリング確定済み
- SSL_CERT_FILE と CODEX_PROXY_CERT が同じパスの場合: 二重追加しない — ヒアリング確定済み
- エラーメッセージに TLS ヒントを通常表示でも追加するか: 許容（計画で判断）— ヒアリング確定済み

---

## 8. 探索メトリクス（自己検証用）

| 指標 | 基準 | 実績 |
|------|------|------|
| Read したファイル数 | 10 以上 | 15 |
| Grep 検索キーワード数 | 5 以上 | 9 |
| コードスニペット数 | 5 以上 | 8 |
| 逆引き検索実施 | 必須 | 実施済み（`build_client` 使用箇所: github.rs / config_test.rs / config.rs）|

**探索キーワード一覧**: `rustls-tls`, `build_client`, `HttpConfig`, `PlmError`, `ErrorFormatter`, `verbose`, `SSL_CERT_FILE`, `CODEX_PROXY_CERT`, `EnvVar`

**Read したファイル一覧**:
- `Cargo.toml`
- `src/config.rs`
- `src/config_test.rs`
- `src/error.rs`
- `src/error/code.rs`
- `src/error/rich.rs`
- `src/error/formatter.rs`
- `src/host/github.rs`
- `src/host.rs`
- `src/main.rs`
- `src/cli.rs`
- `src/commands.rs`
- `src/commands/deploy/install.rs`
- `src/http.rs`
- `src/env.rs`
- `.github/workflows/ci.yml`
