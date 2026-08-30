# 上流仕様の追随状況

各ターゲット環境（CLI / IDE）の公式 Hooks / Skills / Agents 仕様の更新を定期的に調査し、
PLM 側で対応が必要な項目を TODO として管理するドキュメント。

- **最終調査日**: 2026-08-20
- **前回同期**: 2026-07〜08（`docs/concepts/targets.md` 最終更新）
- **調査方法**: 各ターゲットの公式ドキュメントを参照し、`docs/concepts/targets.md` の記載および
  `src/hooks/` / `src/target/env/` の実装と突き合わせる

## 調査結果サマリ

| ターゲット | Skills | Agents | Hooks | 起票 Issue |
|-----------|--------|--------|-------|-----------|
| OpenAI Codex | 変更なし | 変更なし | ✅ `SessionEnd`・`async` 対応済み / ⚠️ 有効化フラグ仕様変更 | [#455](https://github.com/DIO0550/plugin-manager/issues/455) / [#456](https://github.com/DIO0550/plugin-manager/issues/456) |
| VSCode Copilot | ⚠️ Personal スコープ（`~/.copilot/skills/`）追加 | 変更なし | ⚠️ 未マップイベントあり（`errorOccurred` / `preCompact` / `subagentStart`） | [#457](https://github.com/DIO0550/plugin-manager/issues/457) / [#458](https://github.com/DIO0550/plugin-manager/issues/458) |
| Google Antigravity | ⚠️ 公式既定パスが `.agents/skills` / `~/.gemini/config/skills` へ | 変更なし（PLM 未実装は [#400](https://github.com/DIO0550/plugin-manager/issues/400)） | 変更なし（5 イベント） | [#460](https://github.com/DIO0550/plugin-manager/issues/460) |
| Gemini CLI | ⚠️ GA 化・`.agents/skills` エイリアス・管理コマンド拡張 | 非対応（変更なし） | 非対応（変更なし） | [#461](https://github.com/DIO0550/plugin-manager/issues/461) |
| Cursor | ✅ `icon` / `color` を記載済み | ✅ ネスト・model パラメータを記載済み | ✅ 新イベント・新フィールド・スコープを記載済み | [#459](https://github.com/DIO0550/plugin-manager/issues/459) |
| OpenCode | 変更なし | 変更なし | 対象外（JS/TS Plugin モデル） | — |
| Claude Code（変換元） | ⚠️ frontmatter 大幅拡張・commands が skills へ統合 | — | ⚠️ イベント 30 種超へ拡張・`mcp_tool` type 追加 | [#462](https://github.com/DIO0550/plugin-manager/issues/462) / [frontmatter hooks 独立 Issue 定義](../architecture/skill-subagent-frontmatter-hooks-issue.md) |

## TODO

### 優先度: 高（変換結果が実機と食い違う）

- [x] **Codex hooks の `SessionEnd` / `async` 対応** — [#455](https://github.com/DIO0550/plugin-manager/issues/455)
  - `CodexEventMap` で `SessionEnd` を 1:1 マップし、`CodexKeyMap` で `async` を保持する
- [ ] **Codex hooks 有効化フラグの見直し** — [#456](https://github.com/DIO0550/plugin-manager/issues/456)
  - 上流では hooks が既定で有効。正式キーは `hooks` で `codex_hooks` は deprecated alias
  - PLM は Hook 配置のたびにユーザーの `config.toml` へ deprecated キーを追記している
- [ ] **Copilot hooks の未マップイベント追随** — [#458](https://github.com/DIO0550/plugin-manager/issues/458)
  - `PostToolUseFailure` / `PreCompact` / `SubagentStart` が Copilot だけ未マップ（Cursor では対応済み）
- [ ] **Antigravity Skills の配置パス移行** — [#460](https://github.com/DIO0550/plugin-manager/issues/460)
  - 現行パス（`~/.gemini/antigravity/skills`・`.agent/skills`）は AGY CLI から認識されない
  - Hooks はすでに新パスへ配置済みで、Skills だけ世代が揃っていない

### 優先度: 中（機能追加・仕様確認）

- [ ] **Copilot Skills の Personal スコープ対応** — [#457](https://github.com/DIO0550/plugin-manager/issues/457)
  - `~/.copilot/skills/` が公式パスになったが、PLM は `ScopeSupport::ProjectOnly` のまま
  - Agents / Hooks は Personal 対応済みで、Skills だけ非対称
- [ ] **Claude Code hooks 仕様拡張への追随** — [#462](https://github.com/DIO0550/plugin-manager/issues/462)
  - 変換元のイベントが 30 種超へ拡張。PLM の `HookEvent` は 10 種のまま
  - `mcp_tool` type が未知のため全ターゲットで除外される
  - 対象は `hooks/hooks.json` 経路のイベント、type、フィールド追随に限定する
- [ ] **Skill / Subagent frontmatter hooks 供給経路** — [独立機能 Issue 定義](../architecture/skill-subagent-frontmatter-hooks-issue.md)
  - `hooks/hooks.json` とは別に、所属コンポーネントへスコープした解析・配置・診断を追加する
  - 順序、重複、競合および統合テストの受け入れ条件は Issue 定義を正とする

### 優先度: 低（ドキュメント整備）

- [x] **Cursor の仕様更新をドキュメントへ反映** — [#459](https://github.com/DIO0550/plugin-manager/issues/459)
  - Skills frontmatter の `icon` / `color`、Subagents のネスト・model パラメータ、Hooks の新イベント・新フィールド・設定スコープ優先順位を記載
  - `loop_limit` / `failClosed` は変換元に対応設定がないため意図的に生成せず、Cursor の既定値に委ねる
  - Cursor CLI の発火状況とエディタ専用イベントの未検証範囲は [#466](https://github.com/DIO0550/plugin-manager/pull/466) で実機確認済み
- [ ] **Gemini CLI Skills の GA 化等を反映** — [#461](https://github.com/DIO0550/plugin-manager/issues/461)
  - 「実験的機能・要 Settings 有効化」の記載が現状と不一致
- [ ] **公式ドキュメント URL の移行追随** — [#463](https://github.com/DIO0550/plugin-manager/issues/463)
  - Codex: `developers.openai.com/codex/*` → `learn.chatgpt.com/docs/*`（308 リダイレクト）

## 変更なしと確認した項目

再調査時の差分判断に使うため、今回「変更なし」を確認した項目も記録する。

| 項目 | 確認内容 |
|------|---------|
| Antigravity Hooks | `PreToolUse` / `PostToolUse` / `PreInvocation` / `PostInvocation` / `Stop` の 5 イベント。`enabled` / `type` / `command` / `timeout` フィールド |
| OpenCode Agents / Commands | ディレクトリ名は複数形（`agents/` / `commands/`）。Global は `~/.config/opencode/` |
| OpenCode Skills | 探索は `skills/*/SKILL.md` の 1 階層。`name` は親ディレクトリ名と一致必須 |
| OpenCode Plugins | JS/TS モジュールのまま。JSON Hooks 化の動きなし（PLM 対象外の判断は維持） |
| Cursor Skills の再帰走査 | skills ルートを再帰走査する仕様は維持 |
| Cursor Subagents frontmatter | `name` / `description` / `model` / `readonly` / `is_background` に変更なし |
| Codex hooks の探索パス | `~/.codex/hooks.json` / `<repo>/.codex/hooks.json`（PLM の配置先と一致） |
| Copilot CLI hooks の形式 | `version: 1` + lowerCamelCase + `bash` / `powershell` / `timeoutSec`。VS Code 側が PascalCase へ自動変換 |

## 再調査の手順

次回調査時は以下を順に確認し、本ドキュメントの「最終調査日」と結果を更新する。

| ターゲット | 参照 URL |
|-----------|---------|
| OpenAI Codex | https://learn.chatgpt.com/docs/hooks / https://learn.chatgpt.com/docs/build-skills |
| VSCode Copilot | https://code.visualstudio.com/docs/copilot/customization/hooks / https://code.visualstudio.com/docs/agent-customization/agent-skills |
| GitHub Copilot CLI | https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks |
| Google Antigravity | https://antigravity.google/docs/skills / https://antigravity.google/docs/hooks / https://antigravity.google/docs/subagents |
| Gemini CLI | https://geminicli.com/docs/cli/skills/ |
| Cursor | https://cursor.com/docs/context/skills / https://cursor.com/docs/agent/subagents / https://cursor.com/docs/agent/hooks |
| OpenCode | https://opencode.ai/docs/skills/ / https://opencode.ai/docs/agents/ / https://opencode.ai/docs/commands/ / https://opencode.ai/docs/plugins/ |
| Claude Code（変換元） | https://code.claude.com/docs/en/hooks / https://code.claude.com/docs/en/skills / https://code.claude.com/docs/en/plugins |

確認する観点:

1. **イベント名の増減** — `src/hooks/event/*.rs` のマップと突き合わせる
2. **hook フィールドの増減** — `src/hooks/converter/*.rs` の KeyMap が削除しているフィールドが今も未サポートか
3. **配置パスの変更** — `src/placement_names.rs` / `src/target/env/*.rs` と突き合わせる
4. **frontmatter フィールドの増減** — `docs/architecture/file-formats.md` の変換マッピングと突き合わせる
5. **スコープ（Personal / Project）の増減** — 各ターゲットの `CAPABILITIES` と突き合わせる

## 関連

- [concepts/targets](../concepts/targets.md) - ターゲット環境の仕様
- [reference/hooks-schema-mapping](./hooks-schema-mapping.md) - Hooks スキーマ変換マッピング
- [architecture/file-formats](../architecture/file-formats.md) - ファイルフォーマット仕様
- [roadmap](../roadmap.md) - 実装状況・ロードマップ
