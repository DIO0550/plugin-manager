# OpenCode ターゲット追加 — 実装計画

> 状態: Skills / Agents / Commands / Instructions 実装済み / ドキュメント整合済み（[#421](https://github.com/DIO0550/plugin-manager/issues/421)）  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416)  
> 参照仕様: [`docs/concepts/targets.md`](../concepts/targets.md) の「OpenCode」セクション  
> 参考 Epic: Cursor [#356](https://github.com/DIO0550/plugin-manager/issues/356)

## 目的

PLM に OpenCode を新しいターゲット環境として追加する。`plm install --target opencode`、`plm list --target opencode`、`plm enable/disable --target opencode` 等が動作するようにする。

## スコープ

| コンポーネント | 対応 | Personal | Project |
|----------------|------|----------|---------|
| Skills | ✅ | `~/.config/opencode/skills/<original_name>/` | `.opencode/skills/<original_name>/` |
| Agents | ✅ | `~/.config/opencode/agents/<flattened>.md` | `.opencode/agents/<flattened>.md` |
| Commands | ✅ | `~/.config/opencode/commands/<flattened>.md` | `.opencode/commands/<flattened>.md` |
| Instructions | ✅ | `~/.config/opencode/AGENTS.md` | `AGENTS.md` |
| Hooks | ❌ | —（JS/TS Plugin モデルのため別 Epic） | — |

## Issue 一覧

| # | タイトル | Phase | blocked_by | 状態 |
|:--|:---------|:------|:-----------|:-----|
| [#417](https://github.com/DIO0550/plugin-manager/issues/417) | TargetKind に OpenCode バリアントを追加する | Phase 1 | - | ✅ |
| [#418](https://github.com/DIO0550/plugin-manager/issues/418) | OpenCodeTarget を実装する（Skills 配置） | Phase 2 | #417 | ✅ |
| [#419](https://github.com/DIO0550/plugin-manager/issues/419) | OpenCode の Agents / Commands 配置に対応する | Phase 3 | #418 | ✅ |
| [#420](https://github.com/DIO0550/plugin-manager/issues/420) | OpenCode の Instructions 配置に対応する | Phase 4 | #418 | ✅ |
| [#421](https://github.com/DIO0550/plugin-manager/issues/421) | OpenCode 対応のドキュメント・整合性更新 | 最終 | #418, #419, #420 | ✅ |

## 依存関係図

```
Epic #416: OpenCode ターゲット追加
|
+-- #417: TargetKind に OpenCode バリアントを追加する
|
+-- #418: OpenCodeTarget を実装する（Skills 配置）  [blocked_by: #417]
|
+-- #419: Agents / Commands 配置  [blocked_by: #418]
+-- #420: Instructions（Personal + Project）配置  [blocked_by: #418]
|
+-- #421: ドキュメント・整合性更新  [blocked_by: #418, #419, #420]
```
## Phase 詳細

### Phase 1: TargetKind 追加

- `TargetKind::OpenCode`（serde / CLI: `"opencode"`）
- `as_str` / `command_format` / `agent_format`（ClaudeCode 互換）
- `parse_target` / `all_targets` / layout helpers / `placement_names`
- `TargetsConfig::default` への追加方針: **opt-in**（`plm target add opencode`）。Gemini と同様にデフォルト有効リストには入れない（破壊的変更を避ける）

### Phase 2: OpenCodeTarget（Skills）

- `src/target/env/opencode.rs` + `opencode_test.rs`
- Personal ベース: `$XDG_CONFIG_HOME/opencode`（未設定時 `~/.config/opencode`）
- Project ベース: `.opencode`
- Skills: **`original_name` 必須**（Cursor と同型）。空なら配置スキップ
- `list_placed` / overwrite ガード / ownership 記録
- cleanup 列挙に `.opencode` と config ルートを追加

### Phase 3: Agents / Commands

- Agents / Commands を flatten 名の `.md` で配置
- 内容変換なし（拡張子リネームのみ）
- OpenCode 固有 frontmatter（`mode` / `permission`）は v1 で自動付与しない

### Phase 4: Instructions

- Personal: `~/.config/opencode/AGENTS.md`
- Project: `AGENTS.md`（Codex / Cursor と共有しうる）
- Cursor と異なり Personal も `ScopeSupport::Both`

### Phase 5: Docs 整合 ✅

- 本計画・`targets.md`・`roadmap.md`・`getting-started.md`・`commands/target.md`・`file-formats.md`・`config.md`・README / `CLAUDE.md` の状態表記を ✅ に更新（[#421](https://github.com/DIO0550/plugin-manager/issues/421)）

## 設計上の注意

1. **Skills 1 階層発見**: OpenCode は `skills/*/SKILL.md` のみ。深いネストは不可 → Cursor #377 パターンを再利用
2. **プラグインリソースパス衝突**: OpenCode ネイティブ `plugins/` は JS/TS 用。PLM の plugin-root resources（#393）写像と衝突しうる → 実装時にプレフィックス方針を確定（本仕様の file-formats 注記参照）
3. **XDG**: `dirs`/`xdg` クレートまたは `std::env::var("XDG_CONFIG_HOME")` で Personal ルートを解決
4. **sync 名キー**: original_name Skills は #384 と同型の既知制限

## テスト方針

| 観点 | 内容 |
|------|------|
| Unit | `placement_location` / `supported_components` / scope / `list_placed` |
| Unit | `original_name` 欠落時の Skill スキップ |
| Unit | XDG_CONFIG_HOME 上書き時の Personal パス |
| Integration（任意） | `plm target add opencode` → install skill → 配置パス確認 |

## 非スコープ

- OpenCode Plugins（`.ts` / `.js`）の生成・配置
- Claude Code 互換パス（`.claude/`）への二重配置
- `OPENCODE_CONFIG_DIR` 追加探索
- Commands のネストパス（`team/review.md`）保持

## 公式ドキュメント

- https://opencode.ai/docs/skills/
- https://opencode.ai/docs/agents/
- https://opencode.ai/docs/commands/
- https://opencode.ai/docs/rules/
- https://opencode.ai/docs/plugins/
- https://opencode.ai/docs/config/

## 関連

- [#416](https://github.com/DIO0550/plugin-manager/issues/416)（OpenCode ターゲット追加 Epic）
- #356（Cursor ターゲット追加 Epic — 同型の作業）
- #96（Claude Code ターゲット追加 Epic）
- #363（その他ツールのターゲット対応調査）
- #377（Cursor Skills original_name 配置）
- #384（sync と original_name Skills）
