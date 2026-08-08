# Issue #417 実装計画 — TargetKind に OpenCode バリアントを追加

> 状態: 実装済み  
> Issue: [#417](https://github.com/DIO0550/plugin-manager/issues/417)  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416) Phase 1  
> 親計画: [`opencode-target-plan.md`](./opencode-target-plan.md)

## 目的

`TargetKind` に `OpenCode` を追加し、CLI / serde 識別子 `"opencode"` と配置ルート定数・layout API を登録する。  
`OpenCodeTarget` 本体の実装は [#418](https://github.com/DIO0550/plugin-manager/issues/418) に委譲する。

## スコープ（やる / やらない）

| 項目 | 本 Issue | 備考 |
|------|----------|------|
| `TargetKind::OpenCode` | ✅ | clap `#[value(name = "opencode")]`（kebab-case 回避） |
| `as_str` / serde `"opencode"` | ✅ | `rename_all = "lowercase"` で serde は自動 |
| `command_format` / `agent_format` | ✅ | いずれも `ClaudeCode` |
| `placement_names` 定数 | ✅ | Personal: `.config` + `opencode` / Project: `.opencode` |
| `layout`（instruction / bases / cleanup） | ✅ | Instruction = `AGENTS.md`、cleanup = Skill/Agent/Command |
| 網羅 match のコンパイル修復 | ✅ | `skill_allowed_fields` / `target_display_name` 等 |
| `TargetsConfig::default()` | ❌ 追加しない | Gemini と同様 opt-in |
| `parse_target` / `all_targets` | ❌ 未登録 | `#418` で `OpenCodeTarget` 実装後に登録 |
| `$XDG_CONFIG_HOME` 解決 | ❌ | `#418` の Personal ベース解決で扱う |
| ユーザ向け docs の状態表記更新 | ❌ | `#421` |

## 実装手順

1. **識別子登録** — `src/target.rs` に `OpenCode` を追加し、`as_str` / formats を ClaudeCode 互換にする
2. **配置定数** — `src/placement_names.rs` に OpenCode ルート定数を追加
3. **layout API** — `instruction_filename` / `personal_base` / `project_base` / `cleanup_kind_subdirs` / 整合性アサートを更新
4. **網羅 match** — `component/convert.rs`（Skills frontmatter は Cursor と同様除去なし）、`install/format.rs`（表示名 `"OpenCode"`）
5. **テスト** — serde / ValueEnum / formats / bases / cleanup / default 非包含 / `parse_target("opencode")` 未登録を固定
6. **検証** — `cargo fmt` / `cargo check` / `cargo test`

## 設計判断

1. **`parse_target` は Phase 1 で登録しない**  
   Issue 本文どおり `OpenCodeTarget` 実装後。CLI は `ValueEnum` で `opencode` を受理できるが、実体解決は `#418` までエラーになる。
2. **Personal ルートは `~/.config/opencode` の静的結合**  
   `personal_base(home)` のシグネチャを変えず、XDG 上書きは Target 実装側（`#418`）で解決する。
3. **Hooks 変換は触らない**  
   `create_layers` の `other => Err(...)` で Gemini と同様にフォールスルー。OpenCode は Hooks 対象外。
4. **default レジストリは opt-in**  
   既存ユーザの有効ターゲット集合を壊さない（破壊的変更回避）。

## テスト観点

| テスト | 期待 |
|--------|------|
| `as_str` / formats | `"opencode"` / ClaudeCode |
| serde roundtrip | `"opencode"` |
| ValueEnum | `"opencode"` 受理（`"open-code"` ではない） |
| `personal_base` / `project_base` | `~/.config/opencode` / `.opencode` |
| `cleanup_specs` | skills/agents/commands（Personal + Project） |
| `TargetsConfig::default` | OpenCode / GeminiCli を含まない |
| `parse_target("opencode")` | Err（`#418` まで） |
| `all_targets().len()` | 5 のまま |

## 受け入れ条件（Issue チェックリスト対応）

- [x] `TargetKind` に `OpenCode`（ValueEnum / serde `"opencode"`）
- [x] `as_str()` → `"opencode"`
- [x] `command_format` / `agent_format` → ClaudeCode
- [x] `placement_names` / `layout` に OpenCode ルート
- [x] `TargetsConfig::default()` には追加しない
- [ ] `parse_target` / `all_targets` 登録 → `#418`

## 関連

- [#416](https://github.com/DIO0550/plugin-manager/issues/416) Epic
- [#418](https://github.com/DIO0550/plugin-manager/issues/418) OpenCodeTarget（Skills）
- 既存ドラフト PR: [#423](https://github.com/DIO0550/plugin-manager/pull/423)（同趣旨の先行実装）
