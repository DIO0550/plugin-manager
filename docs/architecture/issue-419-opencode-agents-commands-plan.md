# Issue #419 — OpenCode Agents / Commands 配置 実装計画

> 状態: 実装完了（PR 提出済み）  
> Issue: [#419](https://github.com/DIO0550/plugin-manager/issues/419)  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416)  
> blocked_by: [#418](https://github.com/DIO0550/plugin-manager/issues/418)（完了）  
> 参照: [`opencode-target-plan.md`](./opencode-target-plan.md) Phase 3 / [`targets.md`](../concepts/targets.md) OpenCode セクション

## 目的

`OpenCodeTarget` に Agents / Commands の配置・列挙を追加し、`plm install --target opencode` で Agent / Command が OpenCode ネイティブパスへ配置されるようにする。

## 現状

| 項目 | 状態 |
|------|------|
| `TargetKind::OpenCode`（#417） | ✅ |
| Skills `original_name` 配置（#418） | ✅ |
| Instructions Personal + Project（#420） | ✅ |
| Agents / Commands（#419） | ✅（本 Issue） |
| Docs 整合（#421） | ❌（別 Issue） |

現行 `OpenCodeTarget` の `SUPPORTED` / `CAPABILITIES` は Skill + Instruction のみ。`placement_location` の Agent / Command アームは `_ => None`。

## 仕様（再確認）

| 種別 | Personal | Project | ファイル名 |
|------|----------|---------|------------|
| Agents | `~/.config/opencode/agents/<flattened>.md` | `.opencode/agents/<flattened>.md` | flatten 名 + `.md` |
| Commands | `~/.config/opencode/commands/<flattened>.md` | `.opencode/commands/<flattened>.md` | flatten 名 + `.md` |

- 内容変換なし（`.agent.md` / `.prompt.md` → `.md` は **配置パスの拡張子**で実現）
- OpenCode 固有 frontmatter（`mode` / `permission` 等）は v1 で自動付与しない
- Commands ネストパス（`team/review.md`）は非スコープ（フラット配置のみ）
- `TargetKind::OpenCode` の `agent_format` / `command_format` は既に `ClaudeCode`（#417）→ 同一形式コピーで内容無変換

## 実装方針

Cursor Target（#356 系）と同型パターンを再利用する。

### 変更ファイル

1. `src/target/env/opencode.rs`
   - `SUPPORTED` に `Agent` / `Command` を追加
   - `CAPABILITIES` に両方 `ScopeSupport::Both` を追加
   - `placement_location`: `named_file(&base, plural, name, ".md")`（`context.name()` = flatten 名）
   - `list_placed`: `scan_and_filter(..., filter_plain_markdown)`
   - モジュールコメントを Phase 3 完了に更新
2. `src/target/env/opencode_test.rs`
   - サポート判定・スコープ・配置パス（Personal / Project / XDG）・`list_placed` のテスト追加
   - 既存の「Agent/Command 非サポート」アサーションを更新

### 変更しないもの

- 変換パイプライン（`AgentFormat::ClaudeCode` のままコピー）
- Hooks / Plugins
- ドキュメント全体の ✅ 更新（#421）
- Skill / Instruction の既存挙動

## テスト計画

| テスト | 内容 |
|--------|------|
| `supported_components` | Skill / Agent / Command / Instruction を含み、Hook は含まない |
| `supports_scope` | Agent / Command が Personal + Project |
| `placement_location` Agent | Project: `.opencode/agents/<flattened>.md` |
| `placement_location` Agent Personal | `~/.config/opencode/agents/...` と XDG 上書き |
| `placement_location` Command | Project / Personal 同様 |
| `list_placed` | `.md` を列挙、`.agent.md` / `.prompt.md` は除外（`filter_plain_markdown`） |

## 実装手順

1. Red: Agent/Command サポートを期待するテストを追加し失敗を確認
2. Green: `opencode.rs` にアームと定数を追加
3. Refactor: コメント整備、既存 Skill/Instruction テストが通ることを確認
4. `cargo fmt` → `cargo check` → `cargo test`（関連テスト）

## 受け入れ条件

- [x] `OpenCodeTarget::supported_components` に Agent / Command が含まれる
- [x] Personal / Project で flatten 名の `.md` 配置パスが返る
- [x] `list_placed` が agents / commands 配下のプレーン `.md` を返す
- [x] 単体テストがパスする（`cargo test target::env::opencode::tests` → 32/32）
- [x] Hook は引き続き非サポート
