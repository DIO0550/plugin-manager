# Issue #393 調査報告: Plugin 直下の未認識ファイル/フォルダ（現状とギャップ）

> **方針注記（2026-07-29）:** 本調査は **案 A（各 Skill 配下へ複製）** 前提。Issue #393 本文はその後「プラグイン構造をそのまま配置」に更新され、案 A/B を明示破棄している。仕様（#407）と Issue の分裂については [`docs/review-issue-393-placement-strategy-revision.md`](../review-issue-393-placement-strategy-revision.md) を参照。実装ギャップ（未実装）の事実記述自体は継続して有効。

調査日: 2026-07-27  
対象 Issue: [#393](https://github.com/DIO0550/plugin-manager/issues/393)（OPEN）  
仕様ドキュメント PR: [#407](https://github.com/DIO0550/plugin-manager/pull/407)（MERGED — docs only、案 A 確定）  
関連: [#392](https://github.com/DIO0550/plugin-manager/issues/392)（CLOSED / Skill 内付属） / [#395](https://github.com/DIO0550/plugin-manager/pull/395) / [#339](https://github.com/DIO0550/plugin-manager/issues/339) / [#377](https://github.com/DIO0550/plugin-manager/issues/377)

## 結論（レビュー用）

| 観点 | 現状 |
|------|------|
| 仕様（案 A） | **ドキュメント済み**（`file-formats.md`「Plugin 付属リソース」、concepts 追随、#407） |
| Scan（プラグイン直下の未認識列挙） | **未実装** — `Plugin::build_components` は 5 種 Component のみ |
| Deploy（各 Skill 配下へ複製） | **未実装** — `deploy_skill` は Skill ソース dir の `replace_dir` のみ |
| Skill 内付属（#392） | **実装済み** — `replace_dir` + 仕様 + 回帰テスト |
| `ComponentKind` 拡張 | **不要（案どおり）** — 5 種のまま別枠扱いが仕様 |
| `managedFiles` 個別登録 | 仕様は **不要**（Skill ディレクトリに閉じる）。Issue 本文の「managedFiles に登録」は #407 仕様と不一致 |
| 除外リストの真実源（#339） | `placement_names.rs` に **Plugin 付属用除外定数は未追加**（docs が先走り） |

**要約:** #393 の受け入れ条件のうちドキュメントは満たしているが、Rust 実装・テスト・統合検証はすべて未着手。`spec-plugin` の `plugins/.../references/` はインストール後もターゲットに現れない。

---

## 1. `src/scan/components.rs` — スキャンの仕組み

### 役割

低レベル列挙。ドメイン型（`Component`）への変換は `Plugin::build_components`（`src/plugin/content/plugin_content.rs`）が行う。

### Skills — `list_skill_names` / `collect_skills_recursive`

```13:58:src/scan/components.rs
/// スキル名一覧を取得（再帰）
///
/// `skills_dir` 配下を再帰的に走査し、`SKILL.md` を直下に持つディレクトリを
/// すべて採用する。中間ディレクトリ名は戻り値に含めない（例: `bar/foo/SKILL.md`
/// は `("foo", path)` を返す）。
pub fn list_skill_names(skills_dir: &Path) -> Vec<(String, PathBuf)> {
    // ...
}

fn collect_skills_recursive(current: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in current.read_dir_entries() {
        if is_symlink(&entry) { continue; }
        if !entry.is_dir() { continue; }
        // ...
        if has_exact_skill_manifest(&entry) {
            out.push((entry_name, entry));
            continue; // 配下に潜らない（付属内 SKILL.md の誤検出防止）
        }
        collect_skills_recursive(&entry, out);
    }
}
```

| 項目 | 挙動 |
|------|------|
| 認識 | 直下に正確な `SKILL.md`（`ComponentKind::skill_manifest()`、OsStr 厳密比較）を持つディレクトリ |
| 除外 | 非ディレクトリ、symlink、UTF-8 不可名、`SKILL.md` 無し dir、ケース違い `skill.md` 等 |
| ネスト | 中間 dir は潜るが、Skill 採用後は潜らない → `references/` 内の `SKILL.md` は別 Skill にしない |
| 戻り値 | `(ディレクトリ basename, 絶対パス)`。中間パスは名前に含めない |

### 他コンポーネント

| 関数 | 対象 | 認識 | 備考 |
|------|------|------|------|
| `list_agent_names` | agents パス | `.agent.md`（再帰）+ ルート直下のみ `.md` | ファイル単体も可 |
| `list_command_names` | commands dir | `.prompt.md`（再帰）+ 直下 `.md` | |
| `list_hook_names` | hooks dir | **直下ファイルのみ**（サブdir 無視） | ドットファイル除外 |
| `list_markdown_names` | 任意 dir | 直下 `.md` | Instruction 用 |

**プラグイン直下の任意エントリを列挙する関数は存在しない。**

---

## 2. 関連 scan モジュール

| パス | 役割 |
|------|------|
| `src/scan.rs` | 公開 API 集約（`list_*` / `list_placed_components`） |
| `src/scan/components.rs` | プラグインソース側のコンポーネント列挙 |
| `src/scan/placement.rs` | **配置済み**ターゲット側: Instruction ファイル名除外して flattened 名集合 |
| `src/target/placed/scanner.rs` | 配置先 `<kind>/` の 1 階層走査（`scan_components`）— インストール済み検知用 |
| `src/plugin/content/plugin_content.rs` | `Plugin::build_components` — scan → `Component` 化の本体 |
| `src/install.rs` | `scan_plugin` — `package.components()` + type filter |

### スキャンパイプライン（install）

```
download → MarketplaceContent
  → Plugin::new → build_components
       → list_skill/agent/command/hook_names + build_instructions
       → flatten_components / detect_name_collisions
  → install::scan_plugin → ScannedPlugin
  → place_plugin → Target::placement_location → ComponentDeployment::execute
```

`build_components`（抜粋）:

```127:156:src/plugin/content/plugin_content.rs
    fn build_components(path: &Path, manifest: &PluginManifest) -> Result<Vec<Component>> {
        let plugin_name = manifest.name.as_str();
        let mut components = Vec::new();

        for (kind, items) in [
            (ComponentKind::Skill, list_skill_names(&manifest.skills_dir(path))),
            (ComponentKind::Agent, list_agent_names(&manifest.agents_dir(path))),
            (ComponentKind::Command, list_command_names(&manifest.commands_dir(path))),
            (ComponentKind::Hook, list_hook_names(&manifest.hooks_dir(path))),
        ] {
            let flattened = flatten_components(kind, plugin_name, items)?;
            detect_name_collisions(&flattened)?;
            components.extend(flattened);
        }

        Self::build_instructions(path, manifest, &mut components);
        Ok(components)
    }
```

マニフェスト解決パス（`skills` / `agents` 等がカスタムの場合）は `PluginManifest::*_dir` 経由。  
**ルート直下の `references/` 等はどの分岐にも入らない。**

---

## 3. Skill デプロイ（ターゲット別）

### `deploy_skill`（全ターゲット共通）

```83:109:src/component/deployment.rs
    fn deploy_skill(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        fs.replace_dir(self.source_path(), &self.target_path)?;
        // frontmatter strip は target_path/SKILL.md のみ
        // ...
        Ok(DeploymentOutput::Copied)
    }
```

`replace_dir`（`src/fs.rs`）: 既存 `dst` を `remove_dir_all` してから再帰コピー → Skill **内**付属の stale 掃除はここで効く。

### 配置パス決定 — `skill_dir(base, name)` → `base/skills/<name>/`

| Target | `name` | Personal / Project base 例 |
|--------|--------|------------------------------|
| Codex | `flatten_name` = `{plugin}_{skill}` | `~/.codex` / `.codex` |
| Copilot | 同上 | Personal Skill 非対応 / `.github` |
| Antigravity | 同上 | `~/.gemini/antigravity` / `.agent` |
| Gemini CLI | 同上 | `~/.gemini` / `.gemini` |
| Cursor | **`original_name` のみ**（未設定なら配置不可） | `~/.cursor` / `.cursor` |

ヘルパ: `src/target/placed/placement_helpers.rs` の `skill_dir`。  
各 env: `codex.rs` / `copilot.rs` / `antigravity.rs` / `gemini_cli.rs` / `cursor.rs` の `placement_location`。

### enable / disable 経路

`src/plugin/lifecycle/intent.rs`:

- enable Skill → `FileOperation::CopyDir` → 実装は **`fs.replace_dir`**
- disable Skill → `FileOperation::RemoveDir`

ソースは **Skill ディレクトリのみ**。Plugin ルート付属の合成コピーは無い。

### sync

`src/sync.rs` `execute_create`: Skill は `fs.copy_dir`（**replace ではない**）。  
仕様 docs が指摘する stale 非対称はそのまま。

---

## 4. #392（Skill 内付属）との境界

| | Skill 内（#392） | Plugin 直下（#393） |
|--|------------------|---------------------|
| 例 | `skills/foo/references/` | `plugins/x/references/` |
| Scan | Skill 採用後に潜らない（二重登録しない） | **無視（列挙コード無し）** |
| Deploy | `replace_dir` で副次同梱 | **コピーされない** |
| 仕様 | `file-formats.md`「Skill 付属リソース」 | 「Plugin 付属リソース」（#407） |
| テスト | `deployment_test` / `scan/components/tests` あり | **無し** |

Skill 内テスト例: `test_execute_skill_copies_bundled_resources_same_structure`、`test_list_skill_names_does_not_descend_into_arbitrary_bundled_dirs`。

---

## 5. `.plm-meta.json` / `managedFiles` ライフサイクル

定義: `src/plugin/meta/meta.rs`（`PluginMeta`）。

| フィールド | 用途 |
|------------|------|
| `statusByTarget` | enable/disable（`install::update_meta_after_place`、lifecycle enable/disable） |
| `managedFiles` | **共有 destination**（`hooks.json`、Cursor Skill dir）の所有権。絶対パス単位 |

記録箇所:

- Hook: `post_place` → `record_hook_file_ownership` / `record_codex_hook_ownership`
- Cursor Skill: `record_cursor_skill_ownership`
- 一般 Skill（Codex 等）のディレクトリパスは通常 **managedFiles に入れない**（ディレクトリごと削除で足りる）

#407 仕様:

> Plugin 付属リソースは Skill 配置ディレクトリの内側に置くため、`managedFiles` へ個別登録は不要。

Issue #393 本文の「update 時に managedFiles に登録」は **案 B 寄り / 古い記述**。案 A 採用後は Skill ライフサイクルに追随する想定。

| 操作 | Skill | Plugin 付属（実装） |
|------|-------|---------------------|
| install | `place_plugin` → `deploy_skill` | 未実装 |
| update | 再 deploy（replace_dir） | 未実装（仕様: replace の**後**に書く） |
| disable / uninstall | RemoveDir | Skill と同時に消える想定だが、そもそも配置されない |
| enable | CopyDir→replace_dir | 未実装（仕様: install と同梱処理が必要） |
| sync | copy_dir | 未実装 + stale 非対称 |

---

## 6. `ComponentKind` とコンポーネントモデル

`src/component/model/kind.rs`:

```rust
pub enum ComponentKind { Skill, Agent, Command, Instruction, Hook }
```

- `plural()` / `default_subdir()` / `skill_manifest()` / `file_suffix()` / `all()`
- `Component`: `kind`, `name`（flattened）, `original_name`, `plugin_name`, `path`
- `flatten_name(plugin, original)` → `"{plugin}_{original}"`

#393 提案どおり **第 6 種は追加しない**。「プラグイン付属リソース」は別枠データ構造が必要（未定義）。

---

## 7. パス定数（#339 関連）

| 真実源 | 内容 |
|--------|------|
| `ComponentKind::plural` / `default_subdir` / `skill_manifest` / `file_suffix` | `skills`/`agents`/`commands`/`hooks`、`SKILL.md`、`.agent.md`、`.prompt.md` |
| `TargetKind::placement_subdir` | Copilot Command → `prompts` 上書き（`src/target/core/layout.rs`） |
| `src/placement_names.rs` | Instruction ファイル名、環境ルート（`.codex` 等）、`COPILOT_COMMAND_SUBDIR` |
| `PluginManifest::*_dir` | 既定は `plural()`、マニフェストで上書き可 |
| `manifest_resolve.rs` | `.claude-plugin/plugin.json` → `plugin.json` |

`scan/constants.rs` は **削除済み**（#339 の一部完了）。

**ギャップ:** `file-formats.md` は「除外リテラルの真実源は `placement_names.rs`（#339）」と書くが、現状 `placement_names.rs` に README/LICENSE/VCS/`.claude-plugin` 等の Plugin 付属除外定数は **無い**。

---

## 8. `docs/architecture/file-formats.md`（関連節）

- **Skill 付属リソース**（~L517）: Skill dir 内は `replace_dir` で同梱。Plugin 直下は別節へ。
- **Plugin 付属リソース**（~L552）: 案 A（各 Skill へ相対パス複製）、除外表、衝突は Skill 優先、ライフサイクル、既知制限。

concepts:

- `docs/concepts/components.md` — Skill 内 / Plugin 直下の両方を記述
- `docs/concepts/deployment.md` — 同。ただし展開先パス例が旧 3 階層表記のまま（コードはフラット `{plugin}_{skill}`）

---

## 9. Issue #393 案 A に対するギャップ一覧

受け入れ条件ドラフトとの照合:

| 条件 | 状態 |
|------|------|
| Scan がプラグイン直下の未認識を列挙 | ❌ 未実装 |
| 除外リストを仕様書に明記 | ✅ `file-formats.md`（実装側定数は未） |
| Deploy が全 Skill 対応ターゲットに配置（案 A） | ❌ 未実装 |
| Skill 内同名優先 + 警告 | ❌ 未実装（配置警告チャネルも未整備 — docs が指摘） |
| install/update/uninstall/disable/sync 一貫 | ❌ 未実装（sync は copy_dir 非対称が残る） |
| file-formats 追記 | ✅ #407 |
| 統合テスト（spec-plugin の tdd-guidelines.md） | ❌ 未実装 |

### 実装時の注意（#407 / コード根拠）

1. **順序:** `replace_dir` が target を全消しするため、Plugin 付属の書き込みは **必ずその後**。
2. **経路二重化:** `place_plugin`（`ComponentDeployment`）と `intent` の `CopyDir` の両方に同梱が必要。
3. **除外はマニフェスト解決後パス:** リテラル `skills/` だけ見るとカスタム `skills` パスを誤同梱する。
4. **Cursor:** Skill dir 名が `original_name` のため案 B は相対パスが崩れる → 案 A が妥当。
5. **Skill 無しプラグイン:** 配置先無し → 付属は置かない（仕様どおり）。
6. **総量閾値・README 系除外:** 仕様のみ。定数・実装なし。
7. **Issue 本文 vs 仕様:** managedFiles 登録は案 A では不要。Issue 側の更新が望ましい。

### 推奨実装フック（レビュー指摘）

| 層 | 候補 |
|----|------|
| 検出 | `scan/` に `list_plugin_attached_resources(plugin_root, resolved_exclusions)` 新設（`ComponentKind` に混ぜない） |
| 除外合成 | `placement_names` にリテラル + `PluginManifest` 解決パスを合成（plugin モジュール） |
| 配置 | `deploy_skill` 後、または `place_plugin` の Skill 成功後に overlay コピー（衝突は skip + warn） |
| enable | `intent` の CopyDir 経路にも同じ overlay |
| テスト | unit（検出・衝突・除外）+ 統合（仮想 spec-plugin レイアウトで全 TargetKind） |

---

## 参照パス一覧

| パス | 役割 |
|------|------|
| `src/scan/components.rs` | `list_skill_names` 等 |
| `src/scan/placement.rs` | 配置済み名の Instruction 除外 |
| `src/scan.rs` | scan 公開 API |
| `src/plugin/content/plugin_content.rs` | `build_components` |
| `src/plugin/meta/manifest.rs` | コンポーネント dir 解決 |
| `src/plugin/meta/manifest_resolve.rs` | plugin.json 探索 |
| `src/plugin/meta/meta.rs` | `.plm-meta.json` / managedFiles |
| `src/component/deployment.rs` | `deploy_skill` |
| `src/component/model/kind.rs` | `ComponentKind` / `Component` |
| `src/fs.rs` | `replace_dir` / `copy_dir` |
| `src/install.rs` | `scan_plugin` / `place_plugin` / meta 更新 |
| `src/plugin/lifecycle/intent.rs` | enable/disable の CopyDir/RemoveDir |
| `src/sync.rs` | sync の `copy_dir` |
| `src/placement_names.rs` | 環境・instruction 定数 |
| `src/target/core/layout.rs` | `TargetKind::placement_subdir` 等 |
| `src/target/placed/placement_helpers.rs` | `skill_dir` |
| `src/target/env/{codex,copilot,antigravity,gemini_cli,cursor}.rs` | 配置 |
| `docs/architecture/file-formats.md` | Skill / Plugin 付属仕様 |
| `docs/architecture/issue-392-skill-bundled-resources-current-behavior.md` | #392 調査 |
