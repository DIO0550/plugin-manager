# Review / Spec: Issue #378 — MITM TLS 検証失敗

**状態**: 実装完了（検証済み）  
**関連Issue**: [#378](https://github.com/DIO0550/plugin-manager/issues/378)  
**生成手段**: `spec-driven-dev` スキル → 実装

## ドキュメント

| ファイル | 内容 |
|---------|------|
| [hearing-notes.md](./hearing-notes.md) | ヒアリング結果（Issue 修正案を確定仕様として採用） |
| [requirements.md](./requirements.md) | ユースケース・機能要件・制約 |
| [exploration-report.md](./exploration-report.md) | コードベース探索レポート |
| [implementation-plan.md](./implementation-plan.md) | 実装計画（システム図・変更案・テスト方針） |
| [tasks.md](./tasks.md) | TDD サイクル形式のタスクリスト |
| [tech-reference.md](./tech-reference.md) | 技術リファレンス |

ローカル作業用のフル成果物（`test-cases.html` / `understanding-quiz-plan.html` 含む）は  
`.plugin-workspace/.specs/001-mitm-tls-native-roots/` にもあります（gitignore 対象）。

## 実装サマリー

1. **本筋**: `Cargo.toml` の reqwest feature を `rustls-tls` → `rustls-tls-native-roots`
2. **保険**: `HttpConfig::build_client()` で `SSL_CERT_FILE` / `CODEX_PROXY_CERT` の PEM を明示追加
3. **付随**: `--verbose` 時に `ErrorFormatter` で source chain を表示

## 検証

- `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test`（1854 passed）
- `THIRD_PARTY_LICENSES.md` 再生成済み
