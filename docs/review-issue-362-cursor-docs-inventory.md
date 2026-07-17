# Review: Issue #362 向け Cursor ドキュメント整合性インベントリ

調査日: 2026-07-17  
対象 Issue: [#362 Cursor 対応のドキュメント・整合性更新](https://github.com/DIO0550/plugin-manager/issues/362)  
Epic: [#356](https://github.com/DIO0550/plugin-manager/issues/356)  
前提: #358 / #359 / #360 / #361 はすべて CLOSED（実装済み）。#362 のみ OPEN。

---

## 1. CursorTarget 実装ステータス（コード実態）

**実装ファイル:** `/workspace/src/target/env/cursor.rs`  
**テスト:** `/workspace/src/target/env/cursor_test.rs`  
**CLI 識別子:** `"cursor"`（`TargetKind::Cursor.as_str()` / `parse_target("cursor")`）

### サポートする ComponentKind

`supported_components()` は次の **5 種すべて** を返す:

| Kind | Personal | Project | 配置パス（PLM 実装） |
|------|----------|---------|----------------------|
| Skills | ✅ | ✅ | `~/.cursor/skills/<flattened_name>/` / `.cursor/skills/<flattened_name>/` |
| Agents | ✅ | ✅ | `~/.cursor/agents/<flattened_name>.md` / `.cursor/agents/<flattened_name>.md` |
| Commands | ✅ | ✅ | `~/.cursor/commands/<flattened_name>.md` / `.cursor/commands/<flattened_name>.md` |
| Instructions | ❌ | ✅ | Project のみ: `<project_root>/AGENTS.md` |
| Hooks | ✅ | ✅ | `~/.cursor/hooks.json` / `.cursor/hooks.json` |

`can_place()` より、Instructions の Personal は明示的に拒否。

### 変換ロジック（特に Hooks / Claude Code 互換）

| 種別 | 変換 | 実装根拠 |
|------|------|----------|
| Skills | 無変換（SKILL.md） | 配置のみ |
| Agents | **内容無変換**、ファイル名をプレーン `.md` に（`.agent.md` は認識されない） | `AgentFormat::ClaudeCode`（`src/target.rs`） |
| Commands | **内容無変換**、プレーン `.md` | `CommandFormat::ClaudeCode` |
| Instructions | 無変換（AGENTS.md） | Codex と同ファイル共有 |
| Hooks | **Claude Code → Cursor 変換あり** | `src/hooks/converter/cursor.rs` + `event/cursor.rs` + `tool/cursor.rs` |

**Hooks 変換の要点:**

- 出力は Copilot CLI に近い `version: 1` + **camelCase** イベント
- フィールドは Claude 互換寄り（`command` / `timeout`）。`async` / `once` / `bash` / `statusMessage` / `comment` / `disableAllHooks` は除去＋警告
- イベントマップ例: `SessionStart`→`sessionStart`, `UserPromptSubmit`→`beforeSubmitPrompt`, `PreToolUse`→`preToolUse` など（`CURSOR_EVENT_ENTRIES`）
- Cursor 固有イベント（`beforeShellExecution` 等）は Claude→Cursor 変換の入力側に無いため対象外
- **フルマージは未実装**。代わりに:
  - `hook_overwrite_error`: 既存 `hooks.json` が自プラグイン管理外なら上書き拒否
  - `hook_component_conflict_error`: 同一 install で Hook が複数なら拒否（「wait for merge support」メッセージ）

デフォルト `TargetsConfig` には既に `Cursor` が含まれる（`registry.rs`）。

---

## 2. Issue #362 が求めるドキュメント更新 vs 現状

### Issue #362 チェックリスト（原文）

- [ ] `CLAUDE.md` の Target trait 対応表に Cursor 列を追加
- [ ] `docs/concepts/targets.md` の Cursor セクションから 🚧 を外し、実装済み実挙動に合わせて更新
- [ ] `docs/commands/target.md` の「利用可能なターゲット」表・出力例に `cursor` を追加
- [ ] `docs/concepts/scopes.md` / `components.md` / `deployment.md` に Cursor を追加
- [ ] `docs/getting-started.md` / `docs/index.md` のターゲット言及を更新
- [ ] `docs/roadmap.md` の Cursor フェーズをチェック済みにする
- [ ] `docs/architecture/file-formats.md` に Cursor 変換マッピングを追記

### ファイル別現状とギャップ

#### `CLAUDE.md` — Target trait 表

現状（Cursor なし、Codex Hooks も × のまま）:

```
| 環境 | Skills | Agents | Commands | Instructions | Hooks |
| OpenAI Codex | ○ | ○ | × | ○ | × |
| VSCode Copilot | ○ | ○ | ○ | ○ | ○ |
| Google Antigravity | ○ | × | × | × | × |
| Gemini CLI | ○ | × | × | ○ | × |
```

**ギャップ:** Cursor 行が無い。また概要文・`target/` 列挙に cursor が無い。  
**#362 で追加すべき例:** Cursor | ○ | ○ | ○ | ○(Project) | ○  
（Codex Hooks は実装上対応済みのため、別件で `×`→`○` も検討余地あり）

#### `docs/concepts/targets.md` — Cursor セクション

現状の問題箇所（抜粋）:

- 対応ターゲット表: `| **cursor** | ... | 🚧 部分対応済み |`
- サポート表: `Cursor 🚧`、Hooks 列が `🚧`
- セクション見出し: `## Cursor 🚧（部分実装）`
- 冒頭注記が実装と矛盾:

```
> **実装状況**: Skills / Agents / Commands 配置は実装済み。Instructions / Hooks は未対応。
```

実態は Instructions / Hooks も実装済み（#360/#361 CLOSED）。

- 配置表の Hooks 行が `Hooks 🚧 | hooks.json へマージ` — 実装は「マージ未実装・上書きガード」
- PLM 対応方針表も `Cursor 🚧`

**ギャップ:** 🚧 除去、実装状況注記の全面書き換え、Hooks を「変換＋単一ファイル上書きガード（マージ未）」と正確化。検証結果セクション（Agents の `.md` リネーム等）は実装と一致しており残すべき。

#### `docs/commands/target.md`

現状の利用可能ターゲット表・出力例は `antigravity` / `codex` / `copilot` / `gemini` のみ。`cursor` なし。

**追加すべきパターン（他ターゲット踏襲）:**

```bash
$ plm target add cursor
✅ Added target: cursor
   Supports: skills, agents, commands, instructions, hooks
```

list 例にも `• cursor (skills, agents, commands, instructions, hooks)` を追加。

#### `docs/concepts/scopes.md`

Codex / Copilot / Antigravity / Gemini CLI の配置表のみ。Cursor セクションなし。  
インタラクティブ選択のヒント文字列にも `~/.cursor/` が無い。

#### `docs/concepts/components.md`

各 Kind の配置表・末尾「ターゲット別サポート状況」に Cursor 列なし。  
Commands の「Copilotでのみサポート」記述も Cursor 実装後は不正確。

#### `docs/concepts/deployment.md`

階層型デプロイ例が Codex/Copilot/Antigravity/Gemini/Hooks(Copilot) のみ。  
Cursor は **フラット 2 階層**（`<flattened_name>`）なので、階層例とは別パターンで追記が必要。

#### `docs/getting-started.md`

`plm target add` / `list` 例に cursor なし。概要的ターゲット列挙も 4 つまで。

#### `docs/index.md`

```
| マルチターゲット対応 | OpenAI Codex、VSCode Copilot、Google Antigravity、Gemini CLIに対応 |
```

Cursor 未記載。参考リンクに Cursor 公式 Docs も無し（`roadmap.md` にはある）。

#### `docs/roadmap.md`

```
### Phase 16: Cursor ターゲット対応 🚧
- [x] #357, #358
- [ ] #359 Agents/Commands
- [ ] #360 Instructions
- [ ] #361 Hooks
- [ ] #362 ドキュメント
```

**ギャップ:** #359/#360/#361 は CLOSED なのに未チェック。将来拡張表の Cursor も「🚧 Phase 16 で対応中」。#362 完了時に Phase 16 を ✅、将来表から Cursor を「対応済み」へ移す or 行削除が妥当。

#### `docs/architecture/file-formats.md`

概要が「Codex/Copilot/Gemini CLIへ変換」。Cursor セクション・変換マッピングなし。  
#362 の意図どおり、**Claude Code 互換のため Agents/Commands は無変換（ファイル名のみ）**、Hooks は converter あり、を追記する。

#### 付随ドキュメント（#362 本文外だが整合に有用）

| ファイル | 現状 |
|----------|------|
| `docs/reference/hooks-schema-mapping.md` | Cursor 未記載（#361 本文では追記タスクだったが未反映） |
| `docs/hooks-conversion/*` | Cursor 言及なし |
| `docs/architecture/overview.md` | `target/env` ツリーに cursor.rs なし |

---

## 3. 他ターゲットのドキュメント記載パターン（Cursor 追記の型）

| ドキュメント | Codex | Copilot | Antigravity | Gemini | Cursor 追記時の型 |
|--------------|-------|---------|-------------|--------|-------------------|
| `targets.md` | 詳細セクション（パス表・配置表・Hooks） | 同上＋制約 | 概要＋Skillsのみ | 概要＋Skills/Instructions | **同型の詳細セクション**（既にあるが 🚧 除去・実挙動更新） |
| `commands/target.md` | 表＋add/list 例 | 同 | 同 | 同（CLI名 `gemini`） | **表行＋add/list 例を 1 セット追加** |
| `scopes.md` | 種別×Personal/Project 表 | 同 | Skillsのみ | Skills+Instructions | **`### Cursor` 表を追加** |
| `components.md` | 各 Kind 配置表の行 | 同 | Skills行 | Skills/Instructions行 | **各 Kind 表に Cursor 行＋サポート状況表に列** |
| `deployment.md` | 階層マッピング節 | 同 | Skills節 | Skills+Instructions節 | **フラット配置の節を新設**（階層例と混同しない） |
| `getting-started.md` | add 例 | add 例 | add 例 | add 例 | **add/list 例に cursor** |
| `index.md` | 特徴表に列挙 | 同 | 同 | 同 | **特徴表＋任意で参考リンク** |
| `file-formats.md` | 形式節＋変換マップ | 同 | （Skills共通） | Skills変換 | **Cursor 節（無変換＋Hooks変換要約）** |
| `CLAUDE.md` | 表の行 | 表の行 | 表の行 | 表の行 | **表に 1 行追加** |

CLI 名の注意: Gemini は内部 `GeminiCli` だがユーザー向けは必ず `gemini`。Cursor は内部も CLI も `cursor`。

---

## 4. 既存レビュー / Issue 関連ドキュメント

`docs/` 配下を検索した結果:

- `*review*` / `*356*` / `*358*`〜`*362*` / `*cursor*` 専用レビュー文書: **なし**（本ファイルが初）
- Cursor 言及の実質ソース: `docs/concepts/targets.md`（仕様）、`docs/roadmap.md`（Phase 16）、`docs/old/plm-plan-v5.md`（古い候補列挙のみ）

Issue 状態（2026-07-17 時点）:

| Issue | 状態 | 内容 |
|-------|------|------|
| #356 | OPEN | Epic |
| #357 | （TargetKind）完了扱い | roadmap [x] |
| #358 | CLOSED | Skills |
| #359 | CLOSED | Agents/Commands |
| #360 | CLOSED | Instructions |
| #361 | CLOSED | Hooks 変換・配置 |
| #362 | OPEN | ドキュメント整合（本 Issue） |

---

## 5. CLI ターゲット名（`plm target add` の正確な文字列）

| TargetKind | CLI 文字列 | `parse_target` |
|------------|------------|----------------|
| Antigravity | `antigravity` | ✅ |
| Codex | `codex` | ✅ |
| Copilot | `copilot` | ✅ |
| **Cursor** | **`cursor`** | ✅ |
| GeminiCli | `gemini`（`#[value(name = "gemini")]`） | ✅ |

使用例: `plm target add cursor` / `plm install --target cursor` / `plm list --target cursor`

---

## 6. #362 作業時の注意（実装とのズレ）

1. **docs の「Instructions/Hooks 未対応」は誤り** — コードは対応済み。ドキュメントを実装に合わせる。
2. **Hooks「マージ」表記は過剰** — 実装は変換＋所有権付き単一ファイル配置／衝突拒否。マージ完了と書かない。
3. **Agents/Commands は階層ではなくフラット** — #359 当初仕様の `<marketplace>/<plugin>/` ではなく、検証結果どおり `agents/<flattened_name>.md`。
4. **`file-formats.md` は「大部分無変換」が正しい** — Agents/Commands は ClaudeCode 形式のまま。Hooks だけ別系統（`hooks::converter`）。
5. **roadmap の #359–#361 未チェックはドキュメント負債** — #362 でまとめて解消するのが自然。
6. **#362 本文の blocked_by は #358/#359/#360** — Hooks (#361) は blocker ではないが、現状 #361 も閉じているためドキュメントで Hooks を「対応済み（マージは将来）」と書ける。

---

## 7. 推奨更新優先度（実装作業用）

1. `docs/concepts/targets.md` — 単一の真実源。🚧 除去と実装状況の修正が最重要  
2. `docs/roadmap.md` — Phase 16 チェック反映  
3. `docs/commands/target.md` + `getting-started.md` — CLI 例の一貫性  
4. `scopes.md` / `components.md` / `deployment.md` — パス一覧  
5. `CLAUDE.md` + `index.md` — 高レベル表  
6. `file-formats.md`（＋任意で `hooks-schema-mapping.md`）— 変換の正確な記述  

Rust ソースの変更は #362 の必須範囲外（ドキュメント整合フェーズ）。
