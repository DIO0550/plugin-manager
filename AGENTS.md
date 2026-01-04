# Repository Guidelines

## コミュニケーション
- 応答は日本語で行います。

## 役割と作業方針
- 役割はレビューアーです。
- レビュー結果は `docs/` 配下に書き出します。
- 原則として Rust のファイルは変更しません。

## プロジェクト構成とモジュール配置
- `src/main.rs` がCLIの入口、`src/cli.rs` がClapの引数定義です。
- サブコマンドは `src/commands/` 配下に配置します（例: `install.rs`, `list.rs`, `update.rs`）。
- 設計メモは `docs/` にあります（参考: `docs/plm-plan.md`）。
- ライセンス/コンプライアンス関連は `about.toml`, `about.md.hbs`, `deny.toml` で管理しています。

### モジュール構成方針（Feature ベース）
レイヤーベース（domain/, application/, infrastructure/）ではなく、**Feature ベース**のモジュール構成を採用する。

```
src/
├── target/           # Target 関連の全て
│   ├── codex.rs      # Codex ターゲット実装
│   ├── copilot.rs    # Copilot ターゲット実装
│   └── effect.rs     # ターゲット操作の結果（値オブジェクト）
├── plugin/           # Plugin 関連の全て
│   ├── cache.rs      # キャッシュ管理
│   └── manifest.rs   # マニフェスト
└── component/        # Component 関連の全て
    ├── kind.rs       # コンポーネント種別
    └── deployment.rs # デプロイメント
```

**原則:**
- 機能（Feature）単位でモジュールを分ける
- 値オブジェクト、エンティティ、サービスは関連する Feature に配置
- レイヤー分離よりも凝集度を優先

## ビルド・テスト・開発コマンド
- `cargo build`: デバッグビルド。
- `cargo build --release`: リリースビルド。
- `cargo run -- <args>`: ローカルで `plm` を実行（例: `cargo run -- list`）。
- `cargo test`: Rustテストを実行（現状テスト未配置のため、追加時に整備）。
- `cargo fmt`: rustfmt で整形。
- `cargo clippy`: 静的解析。
- `cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs`: 依存関係更新後にライセンス一覧を再生成。

## コーディングスタイルと命名規約
- Rust 2021 edition。`rustfmt` 準拠の整形を維持（4スペース相当）。
- モジュール/ファイル/関数は `snake_case`、型は `CamelCase`、定数は `SCREAMING_SNAKE_CASE`。
- 新しいCLI機能は `src/commands/` に追加し、`src/commands/mod.rs` で配線します。

## テスト方針
- モジュール単位のテストは `#[cfg(test)]` と `#[test]` を使って同一ファイルに配置します。
- CLIの挙動確認は `tests/` に統合テストを追加し、機能名で命名（例: `install.rs`）。
- ネットワーク/ファイル操作に影響する変更は、テスト追加かPR内での手動検証手順を明記します。

## コミットとPRのガイドライン
- 既存コミットは短く直接的（例: “update”, “実装計画追加”）。同じ粒度で簡潔に書きます。
- PRには概要、変更理由、検証方法（コマンドまたは手動確認）を記載します。
- 仕様変更やCLI出力変更がある場合は、関連ドキュメントを更新します。

## 依存関係とライセンス確認
- 依存関係変更時は `THIRD_PARTY_LICENSES.md` を同期させます。
- `cargo-deny` を使う場合は `deny.toml` に対して `cargo deny check` を実行します。
