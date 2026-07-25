# Issue #339 探索レポート: 配置ディレクトリ名・ファイル名リテラルの集約

> **日付**: 2026-07-25  
> **対象**: [#339](https://github.com/DIO0550/plugin-manager/issues/339)  
> **関連**: [#338](https://github.com/DIO0550/plugin-manager/issues/338)（Target Layout 集約 — roadmap 上 ✅ 完了、GitHub Issue は OPEN のまま）  
> **調査 HEAD**: `71f0803`（`docs: Issue #338 Target Layout 宣言的ケイパビリティ集約の計画 (#394)` — 実装抽出込み）

## 要約

Issue #339 が指摘する「配置サブディレクトリ名 / マニフェスト名 / instruction ファイル名」の **3 系統並立**は、現行コードでもなお残っている。#338 により `target/env/` の制御フローコピペは `placed::{filter,list_helpers,placement_helpers,scope_support}` と各 env の `LAYOUT` / `CAPABILITIES` に整理されたが、`"skills"` / `"SKILL.md"` / `".codex"` 等の **文字列そのもの**はヘルパ・env・scan・cleanup・wire に分散したままである。`ComponentKind::plural()` は表示用（`"commands"`）であり、Copilot 実配置の `"prompts"` とは一致しない。

---

## 1. `ComponentKind::plural()` と周辺 API

**ファイル**: `/workspace/src/component/model/kind.rs`

### `ComponentKind`（L12–23）

```12:23:/workspace/src/component/model/kind.rs
pub enum ComponentKind {
    /// スキル（SKILL.md形式）
    Skill,
    /// エージェント（.agent.md形式）
    Agent,
    /// コマンド（.prompt.md形式）
    Command,
    /// インストラクション（AGENTS.md, copilot-instructions.md形式）
    Instruction,
    /// フック（任意のスクリプト）
    Hook,
}
```

### `impl ComponentKind` 現行 API 形状（L25–69）

| メソッド | 戻り値 | 役割 |
|----------|--------|------|
| `as_str(&self) -> &'static str` | `"skill"` … | 単数識別子 |
| `plural(&self) -> &'static str` | `"skills"` / `"agents"` / `"commands"` / `"instructions"` / `"hooks"` | 複数形（表示・JSON キー想定） |
| `display_name(&self) -> &'static str` | `"Skill"` … | UI 表示 |
| `all() -> &'static [ComponentKind]` | 全 5 種 | 列挙 |

```37:46:/workspace/src/component/model/kind.rs
    /// 複数形の文字列を取得
    pub fn plural(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skills",
            ComponentKind::Agent => "agents",
            ComponentKind::Command => "commands",
            ComponentKind::Instruction => "instructions",
            ComponentKind::Hook => "hooks",
        }
    }
```

**#339 観点**: `placement_subdir` / `file_suffix` / `skill_manifest` は **未実装**。`plural()` は配置サブディレクトリとしては使えない（Copilot Command → `"prompts"`）。

---

## 2. `src/scan/constants.rs` — 全定数

**ファイル**: `/workspace/src/scan/constants.rs`（L1–19）

| 定数 | 値 | 行 |
|------|-----|-----|
| `SKILL_MANIFEST` | `"SKILL.md"` | 4 |
| `AGENT_SUFFIX` | `".agent.md"` | 7 |
| `PROMPT_SUFFIX` | `".prompt.md"` | 8 |
| `MARKDOWN_SUFFIX` | `".md"` | 9 |
| `DEFAULT_SKILLS_DIR` | `"skills"` | 12 |
| `DEFAULT_AGENTS_DIR` | `"agents"` | 13 |
| `DEFAULT_COMMANDS_DIR` | `"commands"` | 14 |
| `DEFAULT_HOOKS_DIR` | `"hooks"` | 15 |
| `DEFAULT_INSTRUCTIONS_FILE` | `"instructions.md"` | 18 |
| `DEFAULT_INSTRUCTIONS_DIR` | `"instructions"` | 19 |

**消費者（参考）**:
- `src/scan/components.rs` — `SKILL_MANIFEST` / `AGENT_SUFFIX` / `PROMPT_SUFFIX`
- `src/plugin/meta/manifest.rs` — `DEFAULT_*_DIR` / `DEFAULT_INSTRUCTIONS_*`（プラグイン内相対パスのデフォルト）
- `src/scan.rs` — re-export

これらは `ComponentKind::plural()` と同一文字列の再定義になっている（ただし instruction のデフォルトは `"instructions.md"` で、ターゲット側の `AGENTS.md` 等とは別概念）。

---

## 3. `src/scan/placement.rs` — Instruction ファイル名

**ファイル**: `/workspace/src/scan/placement.rs`

```16:22:/workspace/src/scan/placement.rs
/// Instruction として扱う既知のファイル名集合。
const INSTRUCTION_FILE_NAMES: &[&str] = &["AGENTS.md", "copilot-instructions.md", "GEMINI.md"];

/// 配置済みアイテム文字列の中に Instruction ファイル名が含まれているか。
pub fn is_instruction_file(item: &str) -> bool {
    INSTRUCTION_FILE_NAMES.contains(&item)
}
```

公開 API:
- `is_instruction_file(item: &str) -> bool`（L20–22）
- `list_placed_components(placed_items: &[String]) -> HashSet<String>`（L32–38）— Instruction 名を除外して flattened_name 集合を返す

**二重定義の相手**（各 env の `LAYOUT.instruction_file`）:

| Target | 定数値 | 定義箇所 |
|--------|--------|----------|
| Codex | `"AGENTS.md"` | `src/target/env/codex.rs:27` |
| Copilot | `"copilot-instructions.md"` | `src/target/env/copilot.rs:22` |
| Gemini CLI | `"GEMINI.md"` | `src/target/env/gemini_cli.rs:20` |
| Cursor | `"AGENTS.md"` | `src/target/env/cursor.rs:23` |
| Antigravity | （Instruction 非サポート） | — |

#339 提案どおり、除外集合を `all_targets()` + `instruction_filename()` から構築すれば乖離を構造的に防げる。

---

## 4. ベアリテラル一覧（指定パス）

### 4.1 `component/deployment.rs`

| 行 | リテラル | 文脈 |
|----|----------|------|
| 95 | `"SKILL.md"` | `deploy_skill` 内 frontmatter strip 対象マニフェスト |

```95:95:/workspace/src/component/deployment.rs
                let manifest = self.target_path.join("SKILL.md");
```

（コメント内言及: L85–87, L92）

### 4.2 `target/env/*.rs`（本番コード；テストは参考）

#### Codex — `/workspace/src/target/env/codex.rs`

| 行 | リテラル | 用途 |
|----|----------|------|
| 25 | `".codex"` | `LAYOUT.subdir` |
| 27 | `"AGENTS.md"` | `LAYOUT.instruction_file` |
| 28 | `"hooks.json"` | `LAYOUT.hooks_file` |
| 226 | `"skills"` | `list_placed` Skill scan |
| 228 | `"agents"`, `".agent.md"` | Agent scan / suffix |
| 231 | `"hooks"` | Hook listed_as エイリアス |

Skill 配置パス自体は `skill_dir` / `agent_file` ヘルパ経由（ヘルパ内に `"skills"` / `"agents"` / `".agent.md"`）。

#### Copilot — `/workspace/src/target/env/copilot.rs`

| 行 | リテラル | 用途 |
|----|----------|------|
| 20–21 | `".copilot"`, `".github"` | personal / project subdir |
| 22 | `"copilot-instructions.md"` | instruction |
| 97 | `"prompts"`, `".prompt.md"` | Command placement |
| 99 | `"hooks"`, `".json"` | Hook placement |
| 123–130 | `"skills"`, `"agents"`, `".agent.md"`, `"prompts"`, `".prompt.md"`, `"hooks"` | `list_placed` |

**重要**: Command の実配置ディレクトリは `"prompts"`。`ComponentKind::Command.plural()` の `"commands"` とは不一致。

#### Antigravity — `/workspace/src/target/env/antigravity.rs`

| 行 | リテラル | 用途 |
|----|----------|------|
| 21–23 | `".gemini"`, `"antigravity"`, `".agent"` | LAYOUT |
| 98 | `"skills"` | `list_placed` |

#### Gemini CLI — `/workspace/src/target/env/gemini_cli.rs`

| 行 | リテラル | 用途 |
|----|----------|------|
| 19–20 | `".gemini"`, `"GEMINI.md"` | LAYOUT |
| 114 | `"skills"` | `list_placed` |

#### Cursor — `/workspace/src/target/env/cursor.rs`

| 行 | リテラル | 用途 |
|----|----------|------|
| 22–24 | `".cursor"`, `"AGENTS.md"`, `"hooks.json"` | LAYOUT |
| 109–110 | `"skills"` | legacy flattened skill path |
| 178–179 | `"agents"`, `".md"`, `"commands"`, `".md"` | Agent/Command placement |
| 292–296 | `"skills"`, `"agents"`, `"commands"`, `"hooks"` | `list_placed` |

### 4.3 ヘルパ側の隠れたベアリテラル（#338 抽出後も残存）

**`/workspace/src/target/placed/placement_helpers.rs`**

```7:18:/workspace/src/target/placed/placement_helpers.rs
pub(crate) fn skill_dir(base: &Path, name: &str) -> PlacementLocation {
    PlacementLocation::dir(base.join("skills").join(name))
}
// ...
pub(crate) fn agent_file(base: &Path, name: &str) -> PlacementLocation {
    named_file(base, "agents", name, ".agent.md")
}
```

**`/workspace/src/target/placed/filter.rs:9`** — `c.path.join("SKILL.md")`

### 4.4 `commands/info/wire.rs` / `commands/list/wire.rs`

JSON シリアライズキーとして `plural()` 相当のベア配列:

- `info/wire.rs:72–76` — `"skills"|"agents"|"commands"|"instructions"|"hooks"`
- `list/wire.rs:51–55` — 同上

いずれも `ComponentKind::plural()` を呼ばず手書き。

### 4.5 `commands/deploy/import.rs:102–114`

パス解析の kind 文字列マッチ:

```102:114:/workspace/src/commands/deploy/import.rs
    let kind = match kind_str.to_lowercase().as_str() {
        "skills" => ComponentKind::Skill,
        "agents" => ComponentKind::Agent,
        "commands" => ComponentKind::Command,
        "instructions" => ComponentKind::Instruction,
        "hooks" => ComponentKind::Hook,
        _ => { /* ... */ }
    };
```

### 4.6 `plugin/cache/cleanup.rs:91–140`

環境ディレクトリ + kind サブディレクトリを **両方ベアリテラル**で再定義:

| TargetKind | base リテラル | kind_subdir |
|------------|---------------|-------------|
| Codex | `.codex` | `agents`, `skills` |
| Copilot personal | `.copilot` | `agents`, `hooks` |
| Copilot project | `.github` | `agents`, `prompts`, `skills`, `hooks` |
| Antigravity | `.gemini/antigravity`, `.agent` | `skills` |
| GeminiCli | `.gemini` | `skills` |
| Cursor | `.cursor` | `skills`, `agents`, `commands` |

`LAYOUT` 定数とは未接続。新ターゲット追加時の掃除漏れリスクが #339 の核心例。

---

## 5. 環境ディレクトリ定数の現状

### 旧 `CODEX_SUBDIR` 等は **削除済み**

Issue #339 本文が参照する `CODEX_SUBDIR=".codex"`（旧 `codex.rs:15` 等）は、#338 Phase F の薄い `LAYOUT` 定数化で置き換えられた。現行に `CODEX_SUBDIR` / `COPILOT_SUBDIR` 等の識別子は **存在しない**（`rg` 0 件）。

### 現行 `LAYOUT` 形状（各 env）

```text
CodexLayout      { subdir, config_file, instruction_file, hooks_file }
CopilotLayout    { personal_subdir, project_subdir, instruction_file }
AntigravityLayout { personal_parent, personal_child, project_subdir }
GeminiLayout     { subdir, instruction_file }
CursorLayout     { subdir, instruction_file, hooks_file }
```

### `Scope::description()` — `/workspace/src/component/model/kind.rs:185–190`

```185:190:/workspace/src/component/model/kind.rs
    pub fn description(&self) -> &'static str {
        match self {
            Scope::Personal => "~/.codex/, ~/.copilot/",
            Scope::Project => ".codex/, .github/",
        }
    }
```

Antigravity / Gemini / Cursor のパスは説明に含まれない（表示用の古い要約）。

### 共通パス計算 — `/workspace/src/target/core/paths.rs`

```34:44:/workspace/src/target/core/paths.rs
pub(crate) fn base_dir(
    scope: Scope,
    project_root: &Path,
    personal_subdir: &str,
    project_subdir: &str,
) -> PathBuf { /* Personal → home.join(personal), Project → root.join(project) */ }
```

subdir 文字列は呼び出し側（各 `LAYOUT`）が渡す。Target trait 経由の一元 API は無い。

---

## 6. `Target` trait 現行構造（パス / placement / instruction 関連）

**定義**: `/workspace/src/target.rs:215–359`

### メソッド一覧（現行 API 形状）

| メソッド | 必須/デフォルト | パス・配置関連 |
|----------|-----------------|----------------|
| `kind() -> TargetKind` | 必須 | 識別 |
| `name() -> &'static str` | デフォルト（`kind().as_str()`） | |
| `display_name() -> &'static str` | 必須 | |
| `command_format()` / `agent_format()` | デフォルト | フォーマット（パスではない） |
| `supported_components() -> &[ComponentKind]` | 必須 | サポート一覧 |
| `supports(kind)` | デフォルト | |
| `can_place_scope(kind, scope) -> bool` | デフォルト／override | **#338 で追加** — サポート判定の単一真実源 |
| `supports_scope(kind, scope)` | デフォルト → `can_place_scope` | ダミープロービング廃止済み |
| `placement_location(&PlacementContext) -> Option<PlacementLocation>` | 必須 | **配置パス決定** |
| `component_conflict_error` | デフォルト | Codex/Cursor Hook 多重禁止 |
| `pre_place_check` / `post_place` / `legacy_cleanup_operations` | デフォルト | 振る舞いフック（#388） |
| `remove(&PlacementContext)` | デフォルト（`placement_location` 利用） | 削除 |
| `list_placed(kind, scope, project_root) -> Result<Vec<String>>` | 必須 | **配置列挙** |

### #339 が求めるが **未追加** の API

- `instruction_filename() -> &'static str`（または `Option`）
- 環境ルート subdir の trait / テーブル公開（`cleanup.rs` が消費できる形）
- `ComponentKind` 側の `placement_subdir(target?)` / `file_suffix()` / `skill_manifest()`

### 関連ヘルパ（trait 外、`pub(crate)`）

| モジュール | 関数 | 行付近 |
|------------|------|--------|
| `placement_helpers` | `skill_dir`, `named_file`, `agent_file`, `instruction_file`, `instruction_under_base` | `placement_helpers.rs:7–37` |
| `list_helpers` | `scan_and_filter`, `scan_and_filter_in`, `list_instruction_at` | `list_helpers.rs:8–35` |
| `filter` | `filter_skill_dir`, `filter_suffix_file`, `filter_plain_markdown`, `filter_exact_file`, `filter_json_suffix` | `filter.rs:8–50` |
| `scope_support` | `ScopeSupport`, `allows_scope` | `scope_support.rs:6–36` |
| `paths` | `home_dir`, `base_dir` | `paths.rs:21–44` |

---

## 7. Issue #338 計画ドキュメントの構成とレビュー方針

### 所在

| 状態 | パス |
|------|------|
| 現行ツリー | **削除済み**（PR #394 最終で archive も除去） |
| 履歴参照 | `fdd7b82:docs/old/target-layout-refactor/` |
| リモートブランチ残存 | `origin/cursor/issue-338-target-layout-plan-43a4` の同パス |
| 現行サマリ | `docs/architecture/core-design.md:305–316`、`docs/roadmap.md:132` |

### 計画セットのファイル構成（spec-driven-dev 出力）

1. `hearing-notes.md` — ヒアリング + **方針転換**（top-down DSL → bottom-up impl 抽出）
2. `exploration-report.md` — 行番号付きコード探索（本レポートと同型）
3. `requirements.md` — UC / FR / NFR / CON
4. `implementation-plan.md` — Phase A〜G + システム図
5. `tasks.md` — TDD チェックリスト
6. `test-cases.html` — ケース詳細
7. `README.md` — 索引

### レビュー / 計画の進め方（構造）

1. **探索レポート先行**（現状のコピペ・hack・乖離を行番号付きで固定）
2. **ヒアリングで方針転換を文書化**（ユーザー「impl ベースで」→ DSL 先送り）
3. **要件を FR/CON に凍結**（振る舞い不変・外向き API 不変・ビッグバン禁止）
4. **Phase A〜G の段階移行**（各 Phase 独立コミット想定）
5. **実装後に計画を `docs/old/` へ退避 → 最終的に削除**し、`core-design.md` に短い現状記述を残す

### Phase 対応表（計画 → 現行コード）

| Phase | 内容 | 現状 |
|-------|------|------|
| A | 5 impl 差分表 | 計画時完了 |
| B | `list_placed` → `scan_and_filter` | ✅ `list_helpers.rs` |
| C | Skill filter 共通化 | ✅ `filter_skill_dir` |
| D | `can_place_scope` / ダミー廃止 | ✅ `target.rs:260–272` |
| E | placement ヘルパ | ✅ `skill_dir` / `agent_file` / `instruction_file` |
| F | 薄い `LAYOUT` / `CAPABILITIES` | ✅ 各 env（省略可だったが実施） |
| G | docs / 掃除 | ✅ roadmap + core-design；詳細計画は削除 |

**hearing-notes 明示**: 「#339 関連: 文字列定数一元化は並行作業可。レイアウト内リテラルは後で差し替え可」

---

## 8. #338 と #339 の関係

```text
#338 Target Layout（制御フロー集約）          #339 リテラル集約（ドメイン文字列）
─────────────────────────────────────────    ────────────────────────────────
list_placed / filter / placement 骨格抽出     "skills" / "SKILL.md" / ".codex" 等
can_place_scope 単一真実源                    ComponentKind / Target へ定数 API
薄い LAYOUT（パス断片を const 化）            LAYOUT 内リテラルの「意味」をモデル化
                                              cleanup / wire / scan の二重定義解消
                                              plural() ≠ placement_subdir の分離
```

- **#338 は骨格・分岐・サポート判定**を揃えた。**文字列の単一真実源までは踏み込んでいない**（Phase F の `LAYOUT` は各ファイル内 private const）。
- **#339 は #338 の上に載せる定数層**: `LAYOUT.instruction_file` / `skill_dir` の `"skills"` / `cleanup_specs` の `".codex"` を、`ComponentKind`・`Target`（または共有テーブル）から供給する。
- #338 hearing でも「並行可・後差し替え可」と明記されており、順序依存は弱いが、**#338 完了後のほうが差し替え面が狭い**（ヘルパ 1 箇所 + LAYOUT フィールド）。

### 現状の残ギャップ（#339 作業リスト候補）

1. `ComponentKind`: `plural()` を表示専用と文書化；`placement_subdir` は Target 依存（少なくとも Copilot `"prompts"`）
2. `SKILL.md` / suffixes: `ComponentKind` または共有 const → `filter_skill_dir` / `deployment` / `scan/constants` を消費側に
3. `Target::instruction_filename()`（Antigravity は `None`）→ `INSTRUCTION_FILE_NAMES` を動的構築
4. 環境ルート: `LAYOUT` を trait/テーブル公開 → `cleanup.rs` / `Scope::description()` 更新
5. wire / import の kind キーは `plural()` 呼び出しに置換可能（表示・入出力層）

---

## 9. Issue 本文との行番号ドリフト注記

Issue #339 作成時の行番号は #338 リファクタ後にずれている。対応表:

| Issue 記載 | 現行 |
|------------|------|
| `kind.rs:38-46` plural | **一致** `kind.rs:38-46` |
| `scan/constants.rs:7-15` | 定数は L4–19 に拡充（suffix + DEFAULT_*） |
| `deployment.rs:80` SKILL.md | **→ L95** |
| env 各所の SKILL.md ベア | 多くは `filter_skill_dir`（`filter.rs:9`）へ集約 |
| `CODEX_SUBDIR` 等 | **廃止** → `LAYOUT.subdir` 等 |
| `cleanup.rs:101-129` | **ほぼ一致**（Cursor アーム追加で L101–140） |
| `Scope::description` kind.rs:115-117 | **→ L185–190** |
| `INSTRUCTION_FILE_NAMES` placement.rs:17 | **一致** |

---

## 10. 結論

#339 は未着手のリファクタ課題として妥当。#338 により配置 **骨格**は `placed/` + `LAYOUT`/`CAPABILITIES` に整理済みだが、配置 **文字列**は (a) `plural()`、(b) `scan/constants`、(c) ヘルパ／env／cleanup／wire ベアリテラル、の 3+ 系統が残る。次の実装は #338 計画と同じく **探索固定 → 要件 → 段階抽出（表示用 plural と配置用 subdir の分離を先に）** が安全。
