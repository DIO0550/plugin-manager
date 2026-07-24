# Antigravity 対応状況チェック

**日付**: 2026-07-26  
**結論**: **対応済み**（Skills 限定）

## サマリー

Google Antigravity は PLM のターゲットとして実装済みです。`plm target add antigravity` で有効化し、Skills の Personal / Project スコープへのインストール・一覧・クリーンアップが利用できます。

## 実装の根拠

| 観点 | 状態 | 参照 |
|------|------|------|
| Target 実装 | ✅ | `src/target/env/antigravity.rs` |
| ユニットテスト | ✅ | `src/target/env/antigravity_test.rs` |
| TargetKind / パース | ✅ | `src/target.rs`（`parse_target("antigravity")`） |
| デフォルト有効ターゲット | ✅ | `src/target/core/registry.rs`（`TargetKind::Antigravity` を含む） |
| 配置パス定数 | ✅ | `src/placement_names.rs` |
| レイアウト / クリーンアップ | ✅ | `src/target/core/layout.rs` |
| install / deploy 連携 | ✅ | `src/install_test.rs`, `src/commands/deploy/install_test.rs` |
| TUI 表示 | ✅ | `src/tui/manager/screens/marketplaces/update_test.rs` |
| ドキュメント | ✅ | `docs/concepts/targets.md`（「✅ 対応済み」） |
| ロードマップ | ✅ | `docs/roadmap.md` Phase 2 で完了マーク |

## サポート範囲

| コンポーネント | Personal | Project |
|----------------|----------|---------|
| Skills | ✅ | ✅ |
| Agents | ❌ | ❌ |
| Commands | ❌ | ❌ |
| Instructions | ❌ | ❌ |
| Hooks | ❌ | ❌ |

## 配置パス

| スコープ | パス |
|----------|------|
| Personal | `~/.gemini/antigravity/skills/<marketplace>/<plugin>/<skill>/` |
| Project | `.agent/skills/<marketplace>/<plugin>/<skill>/` |

## 備考

- Antigravity は Skills 専用設計。Agents / Commands / Instructions / Hooks は意図的に非サポート。
- Instructions は Antigravity 側の別設定で管理する想定（`docs/concepts/targets.md` 参照）。
- README の対応表でも Skills のみ「対応」と記載されている。

## 検証方法（参考）

```bash
cargo test antigravity
cargo run -- target list    # antigravity (skills) が表示されること
cargo run -- target add antigravity
```
