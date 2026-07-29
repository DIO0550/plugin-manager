# 調査報告: Skills/Agents 配置レイアウトとターゲット発見規約

調査日: 2026-07-29  
役割: レビューアー調査（Rust 変更なし）  
関連: [#392](https://github.com/DIO0550/plugin-manager/issues/392) / [#393](https://github.com/DIO0550/plugin-manager/issues/393) / `docs/architecture/issue-392-skill-bundled-resources-current-behavior.md`

## 結論（要約）

1. **現行 PLM の配置はフラット 2 階層** `\<env-base\>/skills/\<name\>/`（Agents は `agents/\<name\>.…`）。`marketplace/plugin/skill` の **旧 3 階層は配置しない**（install/enable 後に `cleanup_legacy_hierarchy` で掃除）。
2. **どのターゲットも `plugins/\<plugin\>/skills/…` のようなネストした plugin tree を配置先としてサポートしていない。** `plugin_root` はキャッシュ上のソース／付属リソース検出用。
3. Claude Code ソースは plugin 単位ツリーを保つ。PLM は Skill ごとに切り出し、**Plugin 直下の共有リソースは各 Skill ディレクトリへ複製**する。
4. **Codex / Cursor の公式スキル発見は `skills/` 配下の再帰走査**。一方 PLM の `list_placed` は **1 階層のみ**。ネスト配置するとランタイム発見と PLM 管理一覧が乖離する。
5. 提案の `~/.codex/plugins/spec-plugin/skills/foo/` は **現行ターゲットのスキルルート外**のため、Codex/Cursor の標準発見パスでは読まれない（別ルート登録が無い限り）。

---

## 1. `placement_location`（実装）

共通ヘルパ:

```6:12:src/target/placed/placement_helpers.rs
/// Skill: `base/skills/<name>/`
pub(crate) fn skill_dir(base: &Path, name: &str) -> PlacementLocation {
    let subdir = ComponentKind::Skill
        .default_subdir()
        .expect("Skill always has a default subdir");
    PlacementLocation::dir(base.join(subdir).join(name))
}
```

```19:27:src/target/placed/placement_helpers.rs
/// Agent（Codex / Copilot）: `base/agents/<name>.agent.md`
pub(crate) fn agent_file(base: &Path, name: &str) -> PlacementLocation {
    // ...
    named_file(base, subdir, name, suffix)
}
```

環境ルート定数（`src/placement_names.rs` L25–37）:

| 定数 | 値 |
|------|-----|
| `CODEX_SUBDIR` | `.codex` |
| `COPILOT_PERSONAL_SUBDIR` / `COPILOT_PROJECT_SUBDIR` | `.copilot` / `.github` |
| `ANTIGRAVITY_*` | Personal: `.gemini/antigravity`、Project Skills: `.agent` |
| `GEMINI_SUBDIR` | `.gemini` |
| `CURSOR_SUBDIR` | `.cursor` |

### 1.1 ターゲット別 `placement_location`

| Target | Skill | Agent | 名前の決め方 |
|--------|-------|-------|--------------|
| Codex | `skill_dir(base, name)` | `agent_file(base, name)` | `context.name()` = `{plugin}_{skill}` |
| Copilot | 同上（Project のみ） | 同上（両スコープ） | 同上 |
| Antigravity | `skills_base_dir` + `skill_dir` | 非サポート | 同上 |
| Gemini CLI | `skill_dir` | 非サポート | 同上 |
| Cursor | `skill_dir(base, **original_name**)` | `agents/<flattened>.md` | Skill のみ `original_name`（#377） |

根拠:

- Codex: `src/target/env/codex.rs` L135–154
- Copilot: `src/target/env/copilot.rs` L86–108（Skill は `ScopeSupport::ProjectOnly` L38）
- Cursor: `src/target/env/cursor.rs` L161–187（L173–177 で Skill は `original_name` 必須）
- Antigravity: `src/target/env/antigravity.rs` L130–147
- Gemini CLI: `src/target/env/gemini_cli.rs` L78–93

平坦化関数:

```124:126:src/component/model/kind.rs
pub fn flatten_name(plugin_name: &str, original_name: &str) -> String {
    format!("{plugin_name}_{original_name}")
}
```

テスト例（Codex）: `src/target/env/codex_test.rs` L25–44 → `/project/.codex/skills/my-plugin_my-skill`

---

## 2. 具体パス例（現行実装 = 正）

前提: plugin=`spec-plugin`, skill=`foo`, agent=`review`。

### Codex

| Scope | Skill | Agent |
|-------|-------|-------|
| Personal | `~/.codex/skills/spec-plugin_foo/SKILL.md` | `~/.codex/agents/spec-plugin_review.agent.md` |
| Project | `.codex/skills/spec-plugin_foo/SKILL.md` | `.codex/agents/spec-plugin_review.agent.md` |

Instruction: Personal `~/.codex/AGENTS.md` / Project `AGENTS.md`（プロジェクトルート）。  
Hook: `~/.codex/hooks.json` / `.codex/hooks.json`。

### Copilot

| Scope | Skill | Agent | Command |
|-------|-------|-------|---------|
| Personal | **配置不可** | `~/.copilot/agents/spec-plugin_review.agent.md` | 不可 |
| Project | `.github/skills/spec-plugin_foo/SKILL.md` | `.github/agents/spec-plugin_review.agent.md` | `.github/prompts/spec-plugin_\<cmd\>.prompt.md` |

### Cursor

| Scope | Skill | Agent | Command |
|-------|-------|-------|---------|
| Personal | `~/.cursor/skills/foo/SKILL.md`（**元名**） | `~/.cursor/agents/spec-plugin_review.md` | `~/.cursor/commands/spec-plugin_\<cmd\>.md` |
| Project | `.cursor/skills/foo/SKILL.md` | `.cursor/agents/spec-plugin_review.md` | `.cursor/commands/…` |

Instruction: Project のみ `AGENTS.md`。Hook: `~/.cursor/hooks.json` / `.cursor/hooks.json`。

### Antigravity

| Scope | Skill |
|-------|-------|
| Personal | `~/.gemini/antigravity/skills/spec-plugin_foo/SKILL.md` |
| Project | `.agent/skills/spec-plugin_foo/SKILL.md` |

（公式推奨は Personal `~/.gemini/config/skills/`、Project `.agents/skills/` — PLM は IDE 互換パス。`docs/concepts/targets.md` L219–248 / #402）

### Gemini CLI

| Scope | Skill |
|-------|-------|
| Personal | `~/.gemini/skills/spec-plugin_foo/SKILL.md` |
| Project | `.gemini/skills/spec-plugin_foo/SKILL.md` |

Instruction: `~/.gemini/GEMINI.md` / `GEMINI.md`。

---

## 3. PLM の発見（`list_placed`）— **1 階層のみ**

スキャナ明示:

```1:6:src/target/placed/scanner.rs
//! フラット 1 階層ディレクトリ構造のスキャン
//!
//! `<kind>/<flattened_name>` 構造を 1 階層走査する。
```

```43:55:src/target/placed/scanner.rs
    for entry in fs::read_dir(base_dir)? {
        // ... 直下エントリのみ ScannedComponent 化
    }
```

Skill 認識は「直下ディレクトリかつ `SKILL.md` がある」のみ（`filter_skill_dir`, `src/target/placed/filter.rs` L8–14）。

したがって:

```text
~/.codex/skills/spec-plugin_foo/SKILL.md          → PLM list に載る
~/.codex/skills/company/plugin/foo/SKILL.md       → company が dir だが SKILL.md 無し → Skill として見えない
~/.codex/plugins/spec-plugin/skills/foo/SKILL.md  → skills/ ルート外 → list_placed 対象外
```

---

## 4. ランタイム発見（Codex / Cursor）— **再帰**

### Codex

- 公式/ソース側: `~/.codex/skills/**/SKILL.md` の再帰発見（深さ上限あり、例: max depth 6）。
- ドキュメント例: OpenAI Codex skills docs / `docs/skills.md`（`~/.codex/skills/**/SKILL.md (recursive)`）。
- 近年は User で `$HOME/.agents/skills`、Repo で `.agents/skills`（CWD→repo root）もルートに含まれる。**PLM は引き続き `.codex/skills` に置く**（`$CODEX_HOME/skills` 互換）。

→ 旧 3 階層 `skills/<mp>/<plugin>/<skill>/` でも Codex 自体は見つけうる。だが PLM は現在フラット配置し、旧階層は削除する。

### Cursor

- 公式: skills ルートを**再帰走査**し、任意の深さの `SKILL.md` を発見（カテゴリ用中間ディレクトリ可）。
- `docs/concepts/targets.md` L362, L375; `docs/architecture/file-formats.md` L548。
- frontmatter `name` と **親フォルダ名一致**が要件 → PLM は Skill を `original_name` で配置（#377）。

### Copilot / Gemini / Antigravity

- 概念 docs は「ネスト読み込みは公式未明記」と注意（`deployment.md` L188–190）。  
- PLM はフラット配置でリスク回避。ネスト plugin tree は未検証・非サポート。

---

## 5. ネストした plugin tree をターゲットはサポートするか？

| 方式 | PLM 現状 |
|------|----------|
| `skills/<skill>/` フラット | **唯一の配置方式** |
| `skills/<mp>/<plugin>/<skill>/`（旧 3 階層） | **配置しない**。`cleanup_legacy_hierarchy` が削除 |
| `plugins/<plugin>/skills/...` | **配置コードなし**（`plugins/` は `~/.plm/cache/plugins/` のみ） |
| Skill 内 `references/` 等 | サポート（Skill 付属） |
| Plugin ルート共有 `references/` | 各 Skill へ **複製**（Plugin 付属） |

旧 3 階層掃除: `src/plugin/cache/cleanup.rs` L167–268  
削除対象: `<base>/<kind_subdir>/<marketplace>/<plugin>/`

---

## 6. Claude Code（ソース） vs PLM（フラット配置）

### ソース側（Claude Code Plugin）

`docs/architecture/file-formats.md` L61–63 / Plugin 付属節 L562–573:

```text
plugins/spec-plugin/                 # ← キャッシュ上の plugin_root
├── .claude-plugin/plugin.json
├── skills/
│   ├── implementation-plan/SKILL.md
│   └── spec-driven-dev/
│       ├── SKILL.md
│       └── references/exploration.md   # Skill 付属
├── agents/
└── references/                         # Plugin 付属（共有）
    └── tdd-guidelines.md
```

スキャン（ソース）: `list_skill_names` は `skills/` 配下を**再帰**し、`SKILL.md` 直下ディレクトリを採用。採用後は配下に潜らない（`src/scan/components.rs` L15–30）。

### デプロイ側（PLM）

1. Skill ディレクトリを `replace_dir` でターゲットへコピー（Skill 付属も同構造）。
2. `plugin_root` から Plugin 付属を列挙し、**各 Skill 配置先へ overlay**（相対パス維持、Skill 側優先）。

配置後（Codex Personal）のイメージ（`file-formats.md` L575–586）:

```text
~/.codex/skills/spec-plugin_implementation-plan/
├── SKILL.md
└── references/tdd-guidelines.md      # Plugin 付属の複製
~/.codex/skills/spec-plugin_spec-driven-dev/
├── SKILL.md
└── references/
    ├── exploration.md                # Skill 付属
    └── tdd-guidelines.md             # Plugin 付属の複製
```

共有ディレクトリ方式（`<plugin>_shared/` 兄弟）は採用していない（Cursor の命名規則差で相対参照が一意にならないため — `file-formats.md` L589）。

---

## 7. 「plugin root」・マーケットプレイス階層の現状

| レイヤ | パス | 役割 |
|--------|------|------|
| キャッシュ | `~/.plm/cache/plugins/<marketplace>/<plugin>/` | ソースの plugin_root |
| メタ | 同ディレクトリの `.plm-meta.json` | 所有権・ターゲット状態 |
| 配置 | `<env>/skills/<flattened or original>/` | 実行時発見用。**marketplace 名はパスに出ない** |
| 旧配置 | `<env>/skills/<mp>/<plugin>/…` | レガシー。掃除対象 |

`Component.name` に marketplace は含まれない（`plugin_name` + `original_name` のみ）。同名 plugin は marketplace 違いでも配置名が衝突しうる（メタで管理）。

概念 docs（`components.md` / `deployment.md` / `marketplace.md` / `targets.md` の一部表）は **まだ 3 階層表記のまま**で、実装と乖離。`scopes.md` のベースパス表はフラット前提で実装に近い。`file-formats.md` の Cursor Skills パスは `<flattened_name>` 表記が残っており、実装（`original_name`）と不一致。

---

## 8. 仮説レイアウトの制約分析

### A. ネスト plugin tree（提案）

```text
~/.codex/plugins/spec-plugin/
  skills/foo/SKILL.md
  references/tdd.md
```

| 観点 | 結果 |
|------|------|
| Codex 標準スキルルート | `~/.codex/skills/`（および `.agents/skills` 等）。**`~/.codex/plugins/` は発見対象外** |
| Cursor 標準ルート | `~/.cursor/skills/` 等。同様に **plugins ツリーは対象外** |
| PLM `placement_location` | 生成しない |
| PLM `list_placed` | 見ない |
| 共有 `references/` | plugin 内で一意に保てるが、ランタイムが plugin ルートをスキルルートとして読まない限り無意味 |
| Codex が再帰でも | ルートが `skills/` でない限り再帰の恩恵なし |

### B. 現行フラット + Plugin 付属複製（現状）

```text
~/.codex/skills/spec-plugin_foo/SKILL.md
~/.codex/skills/spec-plugin_foo/references/tdd.md   # 複製
```

| 観点 | 結果 |
|------|------|
| Codex/Cursor 発見 | スキルルート直下（または再帰で）見つかる |
| 相対参照 `references/tdd.md` | Skill ディレクトリ内で解決 |
| ディスク | Skill 数ぶん重複（仕様で許容） |
| Cursor 同名 skill | 別プラグインと衝突しうる（元名配置） |
| `../../references` 形式の参照 | **壊れる**（書き換え無し — `file-formats.md` L629） |

### C. 旧 3 階層を skills 配下に復活させる場合

```text
~/.codex/skills/mp/plugin/foo/SKILL.md
```

| 観点 | 結果 |
|------|------|
| Codex/Cursor ランタイム | **再帰発見なら動く可能性が高い** |
| PLM list/uninstall/sync | **1 階層スキャナでは見えない** → 管理破綻 |
| `cleanup_legacy_hierarchy` | install 時に消される |

### D. skills 直下に plugin フォルダだけ置く場合

```text
~/.codex/skills/spec-plugin/skills/foo/SKILL.md   # または skills/spec-plugin/foo/
```

- Codex/Cursor: 再帰なら `foo` を発見しうる。
- PLM list: `spec-plugin` に `SKILL.md` が無ければ Skill として未検出。
- Cursor: 親フォルダ名と frontmatter `name` 一致は **SKILL.md の直親**基準 → `foo` なら OK。中間 `spec-plugin` はカテゴリ扱い。

---

## 9. Codex / Cursor 再帰 vs 1 階層（対比表）

| 主体 | 走査 | 備考 |
|------|------|------|
| Codex ランタイム | **再帰**（`**/SKILL.md`、深さ制限あり） | User: `$CODEX_HOME/skills` 等 |
| Cursor ランタイム | **再帰** | カテゴリ中間 dir 可。`name`↔親フォルダ一致 |
| PLM `list_placed` / `scan_components` | **1 階層** | フラット配置前提 |
| PLM ソース `list_skill_names` | **再帰**（採用後は潜らない） | キャッシュ内 plugin スキャン用 |

**示唆:** ランタイム再帰を理由にネスト配置へ戻すと、PLM の管理スキャンを再帰化するまで list/uninstall/sync が壊れる。逆に、フラット維持 + 付属複製が現行アーキテクチャと整合。

---

## 10. ドキュメント整合メモ（レビュー指摘）

| 文書 | 問題 |
|------|------|
| `docs/concepts/deployment.md` | 全体が旧 3 階層。実装はフラット |
| `docs/concepts/components.md` L55–61, L91–93 | 同上 |
| `docs/concepts/marketplace.md` L219–257 | 3 階層を現行のように記述 |
| `docs/concepts/targets.md` Codex/Copilot/Antigravity/Gemini 表 | 3 階層のまま。Cursor 節は実装に近い |
| `docs/architecture/file-formats.md` Gemini Skills / Cursor Skills | Gemini は 3 階層表記、Cursor は flattened 表記（実は original_name） |
| `docs/architecture/file-formats.md` Codex Skills | `<name>/` でフラットに近いが flatten 規則未記載 |
| `docs/concepts/scopes.md` | ベースパスのみで実装と整合 |

---

## 参照パス一覧

| パス | 役割 |
|------|------|
| `src/target/env/{codex,copilot,cursor,antigravity,gemini_cli}.rs` | `placement_location` |
| `src/target/placed/placement_helpers.rs` | `skill_dir` / `agent_file` |
| `src/target/placed/scanner.rs` | 配置済み 1 階層スキャン |
| `src/target/placed/filter.rs` | `filter_skill_dir` |
| `src/placement_names.rs` | 環境ルート定数 |
| `src/component/model/kind.rs` | `flatten_name` / `Component::flattened` |
| `src/plugin/cache/cleanup.rs` | 旧 3 階層掃除 |
| `src/scan/components.rs` | ソース側 Skill 再帰スキャン |
| `src/scan/attached.rs` / `src/plugin/attached.rs` | Plugin 付属検出 |
| `src/component/deployment.rs` | Skill `replace_dir` + 付属 overlay |
| `docs/architecture/file-formats.md` | Skill/Plugin 付属仕様 |
| `docs/concepts/{targets,scopes,deployment,components}.md` | 概念（一部 stale） |
