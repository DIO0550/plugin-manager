# Issue #361 レビュー: Cursor の Hooks（hooks.json）変換・配置対応

- **Issue**: [#361 Cursor の Hooks（hooks.json）変換・配置に対応する](https://github.com/DIO0550/plugin-manager/issues/361)
- **Epic**: [#356 Cursor ターゲット追加](https://github.com/DIO0550/plugin-manager/issues/356) 後続フェーズ
- **レビュー日**: 2026-07-17
- **対象ブランチ**: `main`（`5bac10c` — #360 Instructions 配置マージ後）
- **公式仕様**: [Hooks | Cursor Docs](https://cursor.com/docs/agent/hooks)（確認日: 2026-07-17）

## 概要

Claude Code 形式の Hooks を Cursor の単一設定ファイル `hooks.json` へ変換・配置する Issue。形式は表面的には Copilot（`version: 1` + camelCase）に近いが、**キー名・matcher・stdin/stdout・exit code は Claude Code 互換側に寄っており、Copilot converter の単純コピーでは誤変換になる**。加えて単一ファイルへのマージは Codex（#308）と同型の未解決課題である。

## 現状分析

### CursorTarget（#358〜#360 完了後）

| 項目 | 状態 |
|------|------|
| Skills / Agents / Commands / Instructions | ✅ 実装済み |
| `ComponentKind::Hook` | ❌ `supported_components` / `can_place` / `placement_location` いずれも非対応 |
| モジュールコメント | 「Hooks は後続 Issue（#361）」と明記 |

### Hooks 変換基盤

| 対象 | 状態 |
|------|------|
| `create_layers(TargetKind::Copilot \| Codex)` | ✅ |
| `create_layers(TargetKind::Cursor)` | ❌ 未実装（エラー） |
| `event/cursor.rs` / `tool/cursor.rs` / `converter/cursor.rs` | ❌ 不在 |
| `docs/reference/hooks-schema-mapping.md` の Cursor 節 | ❌ 未記載 |
| 単一ファイル hooks の真のマージ | ❌ **どのターゲットにも無い**（Codex は拒否ガードのみ） |

### 配置モデルの対比

| ターゲット | 配置形状 | マージ必要性 | 現状 |
|------------|----------|--------------|------|
| Copilot | `hooks/<name>.json`（複数可） | 不要 | 完成 |
| Codex | `.codex/hooks.json`（単一） | 必要 | 拒否ガード + `managedFiles` |
| Cursor | `.cursor/hooks.json` / `~/.cursor/hooks.json`（単一） | 必要 | 未着手 |

## 設計判断

### 1. スキーマ位置づけ: Copilot 見た目・Claude Code プロトコル

#### 結論: **変換層は Copilot の「見た目」と Codex/Claude の「意味論」を混ぜて設計する。Copilot converter の流用はキー名・ラッパーをそのままコピーしない**

| 項目 | Claude Code | Copilot CLI | Cursor（公式） | PLM 変換方針 |
|------|-------------|-------------|----------------|--------------|
| `version` | なし | `1` 必須 | `1` | 追加 |
| イベント名 | PascalCase | camelCase | camelCase | `EventBridge` テーブル |
| 構造 | matcher グループ | フラット | **フラット + エントリ単位 matcher** | グループをフラット化し `matcher` をエントリへ移す |
| コマンドキー | `command` | `bash` / `powershell` | **`command`** | キー名を維持（`bash` にしない） |
| タイムアウト | `timeout` | `timeoutSec` | **`timeout`** | キー名を維持 |
| exit code 2 | ブロック | ブロックしない | **ブロック（Claude 互換）** | Copilot 用 exit→JSON deny 変換は不要 |
| stdin ツール入力 | `tool_input` オブジェクト | `toolArgs` JSON 文字列 | **`tool_input` オブジェクト系** | Copilot 用 jq ブリッジは原則不要 |
| フック種別 | command/http/prompt/agent | command/prompt | **command / prompt** | command は変換、http/agent は警告+スタブまたは除外、prompt は Cursor ネイティブなら保持可 |

**根拠（公式 Quickstart / Reference）:**

```json
{
  "version": 1,
  "hooks": {
    "afterFileEdit": [{ "command": "./hooks/format.sh" }],
    "preToolUse": [
      { "command": "./hooks/validate-tool.sh", "matcher": "Shell|Read|Write" }
    ],
    "beforeShellExecution": [
      { "command": "./scripts/approve-network.sh", "timeout": 30, "matcher": "curl|wget|nc" }
    ]
  }
}
```

Cursor は Third Party Hooks として Claude Code 形式の読み込みも公式サポートするが、PLM の配置先は `.cursor/hooks.json` であるため、**インストール経路では Cursor ネイティブ形式へ正規化する**（手動の `.claude/settings.json` 共存とは別経路）。

### 2. イベントマッピング（#341: `EventBridge` 経由）

#### 結論: **`CursorEventMap` は `EventBridge` テーブルで定義する。`HookEvent` enum を Cursor 対応に合わせて拡張し、生 match を増やさない**

推奨テーブル（Claude Code → Cursor）:

| Claude Code | Cursor | Copilot との差 | 備考 |
|-------------|--------|----------------|------|
| `SessionStart` | `sessionStart` | 同 | |
| `SessionEnd` | `sessionEnd` | 同 | |
| `PreToolUse` | `preToolUse` | 同 | |
| `PostToolUse` | `postToolUse` | 同 | |
| `PostToolUseFailure` | `postToolUseFailure` | **Cursor のみ対応** | enum に追加推奨 |
| `UserPromptSubmit` | **`beforeSubmitPrompt`** | Copilot は `userPromptSubmitted` | 名前が大きく異なる |
| `Stop` | **`stop`** | Copilot は `agentStop` | |
| `SubagentStart` | `subagentStart` | Copilot 非対応 | enum に追加推奨 |
| `SubagentStop` | `subagentStop` | 同 | |
| `PreCompact` | `preCompact` | Copilot 非対応 | enum に追加推奨 |

**変換対象外（Cursor 固有・Claude 側に対応なし）:**  
`beforeShellExecution` / `afterShellExecution` / `beforeMCPExecution` / `afterMCPExecution` / `beforeReadFile` / `afterFileEdit` / `afterAgentResponse` / `afterAgentThought` / Tab 系 / `workspaceOpen` など。Issue 本文どおり除外（警告）。

**`HookEvent` 拡張が必要な理由:**  
現状 enum は `SessionStart`〜`SubagentStop` の 7 + `Other` のみ。Cursor で双方向対応できる `PostToolUseFailure` / `SubagentStart` / `PreCompact` を `Other` のままにすると、#341 が指摘する「enum とテーブルの別世界」を Cursor でも再現する。本 Issue で enum を拡張するのが正しい。

Codex の `CodexEventMap` 生 match 問題（#341）は本 Issue のブロッカーではないが、**Cursor 実装で同じアンチパターンを踏まないこと**を受入条件に含める。

### 3. ツール名・ matcher

#### 結論: **`CursorToolMap` を `ToolBridge` で定義する。matcher はスクリプト埋め込みよりエントリの `matcher` フィールドへ移す**

Cursor のツール matcher 値（公式）: `Shell`, `Read`, `Write`, `Grep`, `Delete`, `Task`, `MCP:<tool_name>` など。

Claude Code → Cursor（matcher 用）の代表例:

| Claude Code | Cursor | 備考 |
|-------------|--------|------|
| `Bash` | `Shell` | 名前が異なる |
| `Read` / `Write` / `Grep` | 同名 | |
| `Edit` / `MultiEdit` | `Write` 近似、または警告付き除外 | 要検証 |
| `Agent` | `Task` | |
| `mcp__server__tool` | `MCP:<tool>` 形式へ | 要検証 |

Copilot は matcher を持たないためラッパー内フィルタが必須だった。Cursor はエントリ単位 `matcher` を持つため、**StructureConverter はグループをフラット化し、元の `matcher` を各 hook 定義へコピーする**方が正しい。ラッパー生成はパス解決・環境変数・http/prompt スタブ程度に抑える。

### 4. マージ戦略（最大の設計課題）

#### 結論: **#361 は 2 段階で進める。MVP は Codex 同型ガードで単一 Hook 配置を解禁し、真のマージは #308 と共有する基盤として後続（または同一 PR シリーズの Phase B）とする**

Issue は「マージ戦略が必要」と書いているが、**真のマージ実装が Codex にも無い**状態で Cursor だけ完成させると二重実装になる。推奨フェーズ:

#### Phase A（#361 MVP・必須）

Codex と同型の安全策で「1 プラグイン・1 Hook コンポーネント」を動かす。

1. `CursorTarget` に Hook 対応（`supported_components` / `can_place` / `placement_location` / `list_placed`）
2. 配置先: Personal `~/.cursor/hooks.json`、Project `.cursor/hooks.json`
3. `hook_component_conflict_error`（複数 Hook コンポーネント拒否）
4. `hook_overwrite_error` + `managedFiles["cursor"]`（非管理ファイル上書き拒否、再 install のみ許可）
5. converter 4 層 + `create_layers(TargetKind::Cursor)`
6. `hooks-schema-mapping.md` に Cursor 節を追記
7. テスト

これで Epic の「Hooks 対応」の最小価値（単一プラグインの hooks を Cursor に入れられる）を満たす。

#### Phase B（#308 と共通・推奨フォローアップ）

非管理エントリを壊さないイベント単位マージ。所有権の推奨方式:

| 方式 | 内容 | 評価 |
|------|------|------|
| **A. コマンドパス規約** | PLM 管理エントリの `command` を `.cursor/hooks/plm/<plugin>/...`（または既存 `wrappers/<hook>/`）配下に限定し、uninstall 時はそのパスを持つエントリだけ削除 | 実装が単純。atrium 等の実例あり |
| B. JSON メタコメント | エントリに `_plm` 等の独自キーを付与 | Cursor が未知キーを拒否するリスク |
| C. 別ファイル + シンボリック | Cursor が単一ファイルのみ読むため不可 | 不採用 |

**推奨は方式 A。** マージ手順:

1. 既存 `hooks.json` を読む（無ければ `{version:1, hooks:{}}`）
2. 当該プラグイン所有の `command` プレフィックスに一致するエントリを全イベントから除去
3. 変換結果のエントリをイベント配列末尾へ append
4. 空配列のイベントキーを削除
5. `version = 1` を強制
6. `managedFiles["cursor"]` に `hooks.json` 絶対パスを記録

Phase B は Codex の `hooks.json` にも同じヘルパーを使えるよう `src/hooks/merge.rs`（仮）に置くと #308 が軽くなる。

#### Issue 本文チェックリストへの回答

| 設計課題 | レビュー結論 |
|----------|--------------|
| 非管理エントリを壊さない書き込み方式 | Phase A: 上書き拒否。Phase B: コマンドパス規約による差分マージ |
| イベントマッピングは #341 方針 | `EventBridge` + `HookEvent` 拡張。生 match 禁止 |
| CLI でのイベント発火検証 | 実装ブロッカーにしない。受入はユニットテスト + ドキュメントの「未検証」明記。手動検証は別チェックリスト |

### 5. スクリプト配置と `hook_deploy`

#### 結論: **Project は `.cursor/hooks/` 配下、Personal は `~/.cursor/hooks/` 配下にスクリプトを置き、`hooks.json` の `command` はそこを指す**

公式 Quickstart:

- User: `~/.cursor/hooks.json` + `~/.cursor/hooks/format.sh`（`command`: `./hooks/format.sh`）
- Project: `.cursor/hooks.json` + `.cursor/hooks/format.sh`（`command`: **`.cursor/hooks/format.sh`** — プロジェクトルート cwd）

既存 Copilot の `wrappers/` 名前空間はそのまま流用可能だが、Cursor では相対パス規約がスコープで異なる点に注意。`hook_deploy.rs` の `rewrite_script_paths_in_json` は現状 `bash` キーのみ書き換えているため、**Cursor 用に `command` キーも書き換える分岐が必要**。

### 6. `list_placed` の表現

#### 結論: **単一ファイルのため、配置済みなら固定名（例: `"hooks.json"`）を返す。複数 Hook の中身列挙は Phase B 以降**

Codex と同様、ファイル有無ベースでよい。中のイベント配列をプラグイン単位で列挙する UI はマージ実装後に検討。

## 推奨実装（Phase A）

### 新規 / 変更ファイル

| ファイル | 内容 |
|----------|------|
| `src/hooks/event/cursor.rs` | `CursorEventMap` + `CURSOR_EVENT_ENTRIES: &[EventBridge]` |
| `src/hooks/event/claude_code.rs` | `HookEvent` に `PostToolUseFailure` / `SubagentStart` / `PreCompact` を追加（`from_str` 更新） |
| `src/hooks/tool/cursor.rs` | `CursorToolMap`（少なくとも `Bash`→`Shell`, `Agent`→`Task`） |
| `src/hooks/converter/cursor.rs` | KeyMap（identity 寄り）/ StructureConverter（`version:1`・フラット化+matcher 移譲）/ ScriptGenerator（最小） |
| `src/hooks/converter/converter.rs` | `create_layers(TargetKind::Cursor)` |
| `src/hooks/event.rs` / `tool.rs` / `converter.rs` | モジュール配線 |
| `src/target/env/cursor.rs` | Hook 対応 + conflict/overwrite ガード |
| `src/component/deployment/hook_deploy.rs` | `command` キーのパス書き換え |
| `src/install.rs` / import 経路 | Codex と同様に Cursor の conflict/overwrite 呼び出し |
| `docs/reference/hooks-schema-mapping.md` | Cursor 節追記 |
| `*_test.rs` | イベント/キー/構造/配置/ガードのテスト |

### `CursorTarget` 変更要点

```rust
fn can_place(kind: ComponentKind, scope: Scope) -> bool {
    matches!(
        (kind, scope),
        (ComponentKind::Skill, _)
            | (ComponentKind::Agent, _)
            | (ComponentKind::Command, _)
            | (ComponentKind::Instruction, Scope::Project)
            | (ComponentKind::Hook, _)
    )
}

// placement_location Hook:
// Personal → ~/.cursor/hooks.json
// Project  → {project_root}/.cursor/hooks.json
```

Codex の `hook_component_conflict_error` / `hook_overwrite_error` を Cursor 向けに同型実装（メッセージと `managedFiles` キー `"cursor"` のみ差）。可能なら共通ヘルパーへ抽出して #308 準備とする。

### StructureConverter の要点（Copilot との差分）

1. 入力が PascalCase / matcher グループ → Claude Code と判定して変換
2. 入力が `version: 1` + camelCase フラット → TargetFormat（passthrough）
3. 出力エントリは `{ "command": "...", "timeout"?: N, "matcher"?: "..." }`（`bash` / `timeoutSec` にしない）
4. `preserve_matcher_groups: false` だが、フラット化時に **matcher 文字列をエントリへ残す**（Copilot は捨ててスクリプトへ移す）

### テスト（最低限）

| テスト | 内容 |
|--------|------|
| イベント写像 | `UserPromptSubmit`→`beforeSubmitPrompt`、`Stop`→`stop`、`SubagentStart`→`subagentStart` |
| 非対応イベント | `Notification` 等 → `UnsupportedEvent` 警告 + 除外 |
| キー維持 | `command` / `timeout` がリネームされない |
| matcher 移譲 | グループ `matcher: "Bash"` → エントリ `matcher: "Shell"`（ツール写像後） |
| 配置 | Personal / Project の `hooks.json` パス |
| ガード | 複数 Hook 拒否、非管理ファイル上書き拒否、managed 再 install 許可 |
| `create_layers(Cursor)` | エラーにならない |

## 受入基準チェックリスト

### Phase A（本 Issue で完了推奨）

- [ ] Claude Code → Cursor のイベント・ツール名マッピングを `EventBridge` / `ToolBridge` で定義
- [ ] `HookEvent` を Cursor 双方向対応イベント分拡張（生 match 回避）
- [ ] `src/hooks/converter/cursor.rs` 実装 + `create_layers` 配線
- [ ] `CursorTarget` に Hook 配置（単一 `hooks.json`）
- [ ] Codex 同型の conflict / overwrite / `managedFiles["cursor"]`
- [ ] `hook_deploy` が `command` パスを書き換えられる
- [ ] `docs/reference/hooks-schema-mapping.md` に Cursor 節を追記
- [ ] テスト追加（上記表）
- [ ] `cargo test` / `cargo clippy` パス

### Phase B（#308 連携・別 Issue または続編でも可）

- [ ] 非管理エントリを保持する差分マージ
- [ ] uninstall / disable 時の所有エントリのみ削除
- [ ] Codex と共有可能な merge ヘルパー

### 意図的にスコープ外

- Cursor 固有イベント（`beforeShellExecution` 等）への Claude 側からの生成
- Copilot → Cursor 逆変換
- cursor-agent 実機での全イベント発火検証（ドキュメントに未検証として残す）
- `docs/concepts/targets.md` / roadmap の整合（#362）

## リスクと注意点

1. **Copilot とのイベント名差**: `UserPromptSubmit` / `Stop` を Copilot テーブルからコピーすると誤マップになる。Cursor 専用テーブル必須。
2. **ラッパー過剰生成**: Cursor は exit code 2 と Claude 寄りの I/O を持つ。Copilot 用 `EXIT_CODE_HANDLER` を流用すると、ブロックが効かない・JSON 形が壊れる。
3. **Project の相対パス**: cwd がプロジェクトルートのため `.cursor/hooks/...` 形式が必要。Personal の `./hooks/...` と揃えない。
4. **Cloud Agents**: 公式は Project hooks のみ Cloud で実行し、User hooks（`~/.cursor/hooks.json`）は Cloud 非対象。Personal 配置が Cloud に効かないことをユーザ向け警告候補にする（必須ではない）。
5. **#341 未完了**: Codex 側の二重定義は残るが、Cursor 新規実装で悪化させない。
6. **`targets.md` の実装状況表記**: Instructions はコード上実装済みだがドキュメントが「未対応」のまま。#362 で Hooks と合わせて更新。

## 関連 Issue / PR

| 参照 | 関係 |
|------|------|
| #356 | Epic |
| #358 / #366 | Skills（完了） |
| #359 / #367 | Agents / Commands（完了） |
| #360 / #369 | Instructions（完了） |
| #308 | Codex hooks マージ（Phase B と共有推奨） |
| #341 | 変換マッピング単一ソース化（本 Issue で遵守） |
| #362 | ドキュメント・整合性更新 |
| #309 | Antigravity hooks（同様に変換層が増える前に基盤統一したい文脈） |

## 総合判定

**実装着手可。ただし Phase A（変換 + 単一ファイル配置 + Codex 同型ガード）にスコープを切り、真のマージは #308 と共有する Phase B と明記すること。** Copilot converter のコピー実装はスキーマ差（`command`/`timeout`/`matcher`/`beforeSubmitPrompt`/`stop`/exit code）により不適切。
