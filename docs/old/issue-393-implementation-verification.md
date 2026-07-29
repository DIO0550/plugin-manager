# Issue #393 実装状況検証レポート

> **方針注記（2026-07-29）:** Issue #393 現行本文が正本（構造そのまま配置）。本検証の「Rust 未実装」結論は有効。最新レビュー: [`docs/review-issue-393-placement-strategy-revision.md`](../review-issue-393-placement-strategy-revision.md)。

検証日: 2026-07-27  
HEAD: `705dc6d` (`docs: report plugin-root resource scan/deploy gaps for #393`)  
対象: Plugin 直下の未認識ファイル/フォルダ（Plugin 付属リソース）案 A

**総括:** 仕様（#407）と調査ドキュメント（本ブランチ）は存在するが、Rust 実装・テスト・衝突警告チャネルはいずれも未着手。Skill 内付属（#392）の `replace_dir` のみ実装済み。

---

## 1. Plugin ルートの未認識ファイル/フォルダ列挙コードはあるか？

**答え: 無い。** `src/` 内に `plugin_attached` / `attached_resource` / `list_plugin_attached*` / `PluginResource` / overlay コピー実装は存在しない。

検索結果（Rust）:

| 記号 | `src/**/*.rs` |
|------|----------------|
| `plugin_attached` / `attached_resource` / `list_plugin_attached` / `PluginResource` | 0 件 |
| Plugin 付属向け overlay | 0 件 |

`Plugin::build_components` は 5 種 Component のみ:

```127:156:src/plugin/content/plugin_content.rs
    fn build_components(path: &Path, manifest: &PluginManifest) -> Result<Vec<Component>> {
        let plugin_name = manifest.name.as_str();
        let mut components = Vec::new();

        for (kind, items) in [
            (
                ComponentKind::Skill,
                list_skill_names(&manifest.skills_dir(path)),
            ),
            (
                ComponentKind::Agent,
                list_agent_names(&manifest.agents_dir(path)),
            ),
            (
                ComponentKind::Command,
                list_command_names(&manifest.commands_dir(path)),
            ),
            (
                ComponentKind::Hook,
                list_hook_names(&manifest.hooks_dir(path)),
            ),
        ] {
            // ...
        }

        Self::build_instructions(path, manifest, &mut components);

        Ok(components)
    }
```

プラグインルート直下の `references/` 等はどの分岐にも入らない。

---

## 2. `deploy_skill` / `replace_dir` の後に Plugin-root overlay はあるか？

**答え: 無い。** `deploy_skill` は Skill ソース dir の `replace_dir` + 任意の `SKILL.md` frontmatter strip のみで終了する。

```83:109:src/component/deployment.rs
    /// Skill ディレクトリを丸ごと配置する。
    ///
    /// `SKILL.md` 以外の直下ファイルや任意名サブフォルダ（`references/` / `assets/` 等）も
    /// 付属リソースとして同相対構造でコピーする。`replace_dir` によりターゲット側の
    /// 余剰ファイルは削除される。frontmatter 変換がある場合も触るのは `SKILL.md` のみ。
    fn deploy_skill(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        // Skills are directories — replace target to avoid stale files.
        fs.replace_dir(self.source_path(), &self.target_path)?;

        // ターゲットがサポートしない frontmatter フィールドを SKILL.md から除去する。
        if let ConversionConfig::Skill { target_kind } = &self.conversion {
            // ... strip SKILL.md only ...
        }

        Ok(DeploymentOutput::Copied)
    }
```

`replace_dir` の直後〜関数終了までに、plugin root からの追加コピーは存在しない。

---

## 3. `intent.rs` の Skill `CopyDir` は Plugin-root を overlay するか？

**答え: しない。** ソースは `component.path`（Skill ディレクトリ）のみ。実行時は `replace_dir`。

```151:164:src/plugin/lifecycle/intent.rs
    fn build_file_operation(&self, component: &Component, scoped: ScopedPath) -> FileOperation {
        match (self.action.is_deploy(), component.kind) {
            (true, ComponentKind::Skill) => FileOperation::CopyDir {
                source: component.path.clone(),
                target: scoped,
            },
            // ...
        }
    }
```

```239:245:src/plugin/lifecycle/intent.rs
            let result = match &op {
                FileOperation::CopyFile { source, target } => {
                    fs.copy_file(source, target.as_path())
                }
                FileOperation::CopyDir { source, target } => {
                    fs.replace_dir(source, target.as_path())
                }
```

---

## 4. `src/scan/` に plugin-root 非コンポーネント列挙 API はあるか？

**答え: 無い。** 公開 API は Component / 配置済み名のみ。

```18:22:src/scan.rs
pub use components::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
pub use placement::{is_instruction_file, list_placed_components};
```

| 関数 | 対象 |
|------|------|
| `list_skill_names` | skills dir 内の Skill |
| `list_agent_names` | agents パス |
| `list_command_names` | commands dir |
| `list_hook_names` | hooks dir |
| `list_markdown_names` | 任意 dir 直下 `.md` |
| `list_placed_components` | 配置済み flattened 名（Instruction 除外） |

`list_plugin_attached_resources` 等は未定義。`src/scan/constants.rs` も存在しない。

---

## 5. `placement_names.rs` に Plugin 付属用除外定数はあるか？

**答え: 無い。** 内容は Instruction ファイル名と環境ルート定数のみ。

```11:40:src/placement_names.rs
pub const INSTRUCTION_AGENTS: &str = "AGENTS.md";
pub const INSTRUCTION_COPILOT: &str = "copilot-instructions.md";
pub const INSTRUCTION_GEMINI: &str = "GEMINI.md";
// ...
pub const CODEX_SUBDIR: &str = ".codex";
pub const COPILOT_PERSONAL_SUBDIR: &str = ".copilot";
// ... CURSOR_SUBDIR, COPILOT_COMMAND_SUBDIR 等
```

`README*` / `LICENSE*` / `.claude-plugin` / `.git` / `.plm-meta.json` 等の Plugin 付属除外定数は未追加。

一方 `file-formats.md` は真実源を `placement_names.rs` と記述（docs が先走り）:

```596:606:docs/architecture/file-formats.md
| 除外対象 | 例 | 理由 |
| ... |
| リポジトリ定型ドキュメント | `README*` `LICENSE*` ... | ... |

フォルダ名にホワイトリストは設けない（...）。除外リテラルの真実源は `src/placement_names.rs`（#339）に置き、...
```

`.claude-plugin` はマニフェスト探索用として別モジュールに存在（付属除外リストではない）:

```8:8:src/plugin/meta/manifest_resolve.rs
const MANIFEST_PATHS: &[&str] = &[".claude-plugin/plugin.json", "plugin.json"];
```

---

## 6. Skill-bundled × Plugin-root 衝突用の警告チャネルはあるか？

**答え: 無い。**

`ConversionWarning` は Hook 変換専用（5 variants）:

```59:65:src/hooks/converter/converter.rs
pub enum ConversionWarning {
    UnsupportedEvent { event: String },
    UnsupportedHookType { hook_type: String, event: String },
    RemovedField { field: String, reason: String },
    PromptAgentHookStub { event: String, hook_type: String },
    MissingVersion,
}
```

`DeploymentOutput` に汎用 warnings フィールドは無い。`Copied` は警告なし:

```12:21:src/component/deployment/output.rs
pub enum DeploymentOutput {
    Copied,
    CommandConverted(ConversionOutcome),
    AgentConverted(AgentConversionOutcome),
    HookConverted(HookConvertOutput),
}
```

仕様側も別チャネルが必要と明記:

```610:613:docs/architecture/file-formats.md
### 命名衝突
...
Hook 変換の `ConversionWarning` は Hook 専用のため、配置時警告を通す共通チャネル（配置結果に警告を集約する仕組み）が別途必要になる。
```

`detect_name_collisions`（`plugin_content.rs`）は **同種 Component 名衝突**用であり、リソース相対パス衝突とは無関係。

---

## 7. Plugin-root リソースを Skill dir へコピーするテストはあるか？

**答え: 無い。** 存在するのは Skill **内** bundled（#392）のテストのみ。

| テスト | ファイル | 対象 |
|--------|----------|------|
| `test_execute_skill_copies_bundled_resources_same_structure` | `src/component/deployment_test.rs:551` | Skill ソース内 `references/` 等 |
| `test_execute_skill_strip_does_not_touch_bundled_markdown` | 同 `:580` | 同上 + strip 非干渉 |
| `test_execute_skill_replace_dir_removes_stale_bundled_resources` | 同 `:617` | Skill 内 stale 掃除 |
| `test_list_skill_names_does_not_descend_into_arbitrary_bundled_dirs` | `src/scan/components/tests.rs:599` | Skill 配下の二重スキャン防止 |

プラグインルートの `references/` を Skill 配置先へ複製するテスト名・アサーションは `src/**/*test*.rs` に無し。

---

## 8. `ComponentKind` は 5 variants のままか？

**答え: はい。5 のみ。**

```12:23:src/component/model/kind.rs
pub enum ComponentKind {
    Skill,
    Agent,
    Command,
    Instruction,
    Hook,
}
```

```99:106:src/component/model/kind.rs
    pub const fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ]
    }
```

第 6 種（Plugin 付属）は追加されていない（仕様どおり別枠想定、ただし別枠型も未定義）。

---

## 9. git log / #393 / #407 関連コミット

| Commit | 内容 | Rust 変更 |
|--------|------|-----------|
| `06531ec` | `docs: specify plugin bundled resource placement (#393) (#407)` | **無し** — `file-formats.md` + concepts のみ（+89/-2） |
| `705dc6d` | `docs: report plugin-root resource scan/deploy gaps for #393` | **無し** — 調査 MD 追加のみ |
| `2161e4e` | `feat: Skill 付属リソースの仕様化と保証テスト (#395)` | **#392 向け**（Skill 内） |

`#393` / `#407` を含むコミットで実装コードは追加されていない。仕様 docs only（#407）→ ギャップ調査 docs（本 HEAD）。

---

## 10. 既存調査ドキュメントの正確性

ファイル: `docs/architecture/issue-393-plugin-root-resources-current-behavior.md`（存在確認済み）

| 主張 | 現状コードとの照合 |
|------|-------------------|
| Scan 未実装 | **正確**（Q4） |
| Deploy overlay 未実装 | **正確**（Q2） |
| Skill 内付属（#392）実装済み | **正確**（`deploy_skill` + deployment_test） |
| `ComponentKind` は 5 のまま | **正確**（Q8） |
| `placement_names` に除外定数なし | **正確**（Q5） |
| 衝突警告チャネル未整備 | **正確**（Q6） |
| enable の CopyDir → `replace_dir`、Plugin overlay 無し | **正確**（Q3） |
| sync は `copy_dir`（replace ではない） | **正確** — `src/sync.rs:240-241` |
| `scan/constants.rs` 削除済み | **正確**（ファイル不在） |
| concepts `deployment.md` の展開先が旧 3 階層表記 | **正確** — 例: `~/.codex/skills/company-tools/code-formatter/formatter-skill/`（L36）。コードはフラット `{plugin}_{skill}` / Cursor は `original_name` |
| concepts が Plugin 付属を「複製される」と記述 | **仕様記述としては正しいが、実装は未追随**（docs が実装より先行） |

軽微な差分（報告書の引用スタイル）:

- 調査レポート L32-54 の `collect_skills_recursive` 引用は、ソースのコメント文言を要約している（実コード L41-58 のコメントは「Skill 内部の `assets/` 等…」）。挙動の記述自体は正しい。

**結論:** 既存調査ドキュメントの事実主張は現行コードと整合。不正確な実装有無の記述は見当たらない。docs（仕様・concepts）が「複製される」と現在形で書く点は、実装未完了とのギャップとして読む必要がある（調査レポート自身もそれをギャップとして記載済み）。

---

## 質問別サマリ

| # | 質問 | 結果 |
|---|------|------|
| 1 | Plugin root 未認識列挙コード | **無し** |
| 2 | `deploy_skill` 後の overlay | **無し** |
| 3 | intent CopyDir の overlay | **無し** |
| 4 | scan の plugin-root 非 component API | **無し** |
| 5 | `placement_names` 除外定数 | **無し**（docs のみ） |
| 6 | 衝突警告チャネル | **無し**（Hook `ConversionWarning` のみ） |
| 7 | Plugin-root → Skill コピーのテスト | **無し**（Skill 内のみ） |
| 8 | `ComponentKind` 5 variants | **確認** |
| 9 | #393/#407 git | docs only（`06531ec`, `705dc6d`） |
| 10 | 既存調査 MD | **概ね正確**（上記照合） |

**実装ステータス:** #393 受け入れ条件のうちドキュメント（#407）は満たす。Rust 実装・テスト・統合検証は未着手。
