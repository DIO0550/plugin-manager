# OpenCode ターゲット追加 — 実装計画

> 状態: Skills / Agents / Commands / Instructions 実装済み  
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

## 設計上の注意

1. **Skills 1 階層発見**: OpenCode は `skills/*/SKILL.md` のみ。深いネストは不可 → Cursor #377 パターンを再利用（`original_name` 必須）
2. **opt-in**: `TargetsConfig::default` には含めない。`plm target add opencode` で有効化（Gemini と同様）
3. **Agents / Commands**: flatten 名の `.md`、内容無変換（拡張子リネームのみ）。OpenCode 固有 frontmatter（`mode` / `permission`）は v1 で自動付与しない
4. **Instructions**: Personal + Project（Cursor と異なり Personal も `ScopeSupport::Both`）。Project は Codex / Cursor と `AGENTS.md` を共有しうる
5. **プラグインリソースパス衝突**: OpenCode ネイティブ `plugins/` は JS/TS 用。PLM の plugin-root resources（#393）写像と衝突しうる
6. **XDG**: Personal ルートは `$XDG_CONFIG_HOME/opencode`（未設定時 `~/.config/opencode`）
7. **sync 名キー**: original_name Skills は #384 と同型の既知制限

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
