# Review: Issue #362 Cursor ドキュメント・整合性更新

調査日: 2026-07-17  
対象: [#362](https://github.com/DIO0550/plugin-manager/issues/362)  
Epic: [#356](https://github.com/DIO0550/plugin-manager/issues/356)  
前提: #358 / #359 / #360 / #361 は CLOSED（実装済み）

## 判定

**Pass（ドキュメント整合を実施）**

実装（`src/target/env/cursor.rs` ほか）は Skills / Agents / Commands / Instructions / Hooks すべて対応済みだったが、ドキュメントだけが「🚧 部分実装」のままだった。Issue #362 のチェックリストに沿って docs / `CLAUDE.md` を実装実態へ揃えた。Rust ソースは変更していない。

## 実装実態（レビュー根拠）

| Kind | Personal | Project | 配置 |
|------|----------|---------|------|
| Skills | ✅ | ✅ | `~/.cursor/skills/<flattened_name>/` / `.cursor/skills/<flattened_name>/` |
| Agents | ✅ | ✅ | `~/.cursor/agents/<flattened_name>.md` / `.cursor/agents/<flattened_name>.md` |
| Commands | ✅ | ✅ | `~/.cursor/commands/<flattened_name>.md` / `.cursor/commands/<flattened_name>.md` |
| Instructions | ❌ | ✅ | Project のみ `AGENTS.md` |
| Hooks | ✅ | ✅ | `~/.cursor/hooks.json` / `.cursor/hooks.json` |

変換: Agents/Commands は内容無変換（拡張子のみ）、Hooks は Claude→Cursor 変換あり。Hooks フルマージは未実装（上書きガード / 複数 Hook 拒否）。

## 更新したファイル

| ファイル | 内容 |
|----------|------|
| `CLAUDE.md` | Target trait 表に Cursor 行、概要・パス・モジュール列挙を更新。Codex Hooks も実装に合わせて ○ |
| `docs/concepts/targets.md` | 🚧 除去、Hooks ✅、実挙動（上書きガード）に更新 |
| `docs/commands/target.md` | `cursor` を表・出力例に追加 |
| `docs/concepts/scopes.md` | Cursor の Personal/Project パス表を追加 |
| `docs/concepts/components.md` | 各 Kind・サポート表に Cursor 列を追加 |
| `docs/concepts/deployment.md` | フラット配置の Cursor 節とマッピング表を追加 |
| `docs/getting-started.md` | add/list 例と `--target cursor` を追加 |
| `docs/index.md` | 特徴表・目標・参考リンクに Cursor を追加 |
| `docs/roadmap.md` | Phase 16 を ✅、#359–#362 をチェック済み。将来候補から Cursor を除外 |
| `docs/architecture/file-formats.md` | Cursor 形式節と変換マッピング（無変換 + Hooks 変換）を追記 |

## 意図的に残した注記

- Hooks フルマージ未実装（上書き拒否・複数 Hook 拒否）
- Cursor CLI での hooks イベント発火は未検証
- CLI の実際の `plm target add` 出力は docs 例より簡素（`Target added: cursor`）。既存 docs スタイルに合わせて Supports 付き例を維持

## 検証

- `cargo fmt --check`: **PASS**（変更なし）
- `cargo clippy --all-targets`: **PASS**（既存警告 32 件。`-D warnings` は既存 deprecation 等で FAIL）
- `cargo test`（`cargo build` 後）: **PASS**（1835 passed / 0 failed）
- 単独 `cargo test` は `assert_cmd` が `target/debug/plm` を要求するため 29 件 FAIL（既存の取り回し。本 PR 非起因）
- 手動フロー（`plm target add cursor` → install → list → uninstall）は実装 PR（#358–#361）でカバー済み。本 Issue は docs 整合が主目的
