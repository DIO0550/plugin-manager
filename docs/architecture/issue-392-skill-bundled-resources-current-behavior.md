# Issue #392 調査報告: Skill ディレクトリ内の付属リソース（現状挙動）

調査日: 2026-07-27  
対象 Issue: [#392](https://github.com/DIO0550/plugin-manager/issues/392)（調査時点で OPEN）  
関連実装 PR: [#395](https://github.com/DIO0550/plugin-manager/pull/395)（2026-07-24 MERGED — 仕様化 + 保証テスト）  
関連仕様（Plugin 直下）: [#393](https://github.com/DIO0550/plugin-manager/issues/393) / [#407](https://github.com/DIO0550/plugin-manager/pull/407)

## 結論（#392 主張の妥当性）

| Issue の主張 | 現状 |
|---|---|
| `deploy_skill()` は `replace_dir` で Skill ディレクトリごとコピーする | **正確** |
| その結果、未認識ファイル/フォルダも副次的にコピーされる | **正確** |
| 仕様として明文化されていない | **不正確（outdated）** — `docs/architecture/file-formats.md`「Skill 付属リソース」ほかに明記済み（#395） |
| 保証するテストが不在 | **不正確（outdated）** — scan / deployment 双方に回帰テストあり（#395） |

**要約:** 「`replace_dir` で動いている」は今も正しい。一方「仕様化・テストされていない」は #395 マージ以降は当てはまらない。Issue 本文は実装前の記述のまま残っており、Issue 自体はクローズされていない。

---

## 1. `deploy_skill()`（`src/component/deployment.rs`）

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
            if let Some(allowed) = convert::skill_allowed_fields(*target_kind) {
                let manifest = self.target_path.join(ComponentKind::skill_manifest());
                if fs.exists(&manifest) && !fs.is_dir(&manifest) {
                    let original = fs.read_to_string(&manifest)?;
                    let stripped = convert::strip_skill_frontmatter_fields(&original, allowed);
                    if stripped != original {
                        convert::atomic_write(&manifest, &stripped)?;
                    }
                }
            }
        }

        Ok(DeploymentOutput::Copied)
    }
```

### 挙動

1. **`fs.replace_dir(source, target)`** で Skill ディレクトリ全体をコピーする（個別ファイル列挙ではない）。
2. その後、変換設定がある場合のみ **`target_path/SKILL.md`** を読み、`strip_skill_frontmatter_fields` で書き戻す。
3. 付属 md / `references/` / `assets/` 等は **一切触らない**。

### `replace_dir` 実装（`src/fs.rs:207-213`）

```207:213:src/fs.rs
    fn replace_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        self.guard_copy_dir_into_self(src, dst)?;
        if dst.exists() {
            std::fs::remove_dir_all(dst)?;
        }
        copy_dir_recursive(src, dst)
    }
```

- 既存 `dst` を全削除してから再帰コピー → source に無い stale ファイルは消える。
- MockFs も同様（`src/fs/mock.rs:139-152`）。

---

## 2. `strip_skill_frontmatter_fields`

| 項目 | 内容 |
|------|------|
| 定義 | `src/component/convert.rs:323-385` |
| 許可フィールド決定 | `skill_allowed_fields()`（同ファイル 292-299） |
| 呼び出し元 | `deploy_skill` のみ（`deployment.rs:98`） |
| 変更対象ファイル | **`target_path.join("SKILL.md")` のみ** |

### `skill_allowed_fields`

| Target | 保持フィールド | strip 実行 |
|--------|----------------|------------|
| Codex | `name`, `description`, `metadata` | あり |
| Gemini CLI | `name`, `description` | あり |
| Antigravity / Copilot / Cursor | `None`（制限なし） | なし |

### 処理内容（要約）

- YAML 全体再パースではなく **行ベース**で top-level キーを除去。
- frontmatter 無し / 閉じ `---` 無しはそのまま返す。
- 本文はバイト単位で保持（LF/CRLF 対応）。
- 単体テスト: `src/component/convert_test.rs`（`test_strip_skill_frontmatter_*` 多数）。

---

## 3. Skill スキャン（`src/scan/components.rs`）

### `list_skill_names`（L13-39）/ `collect_skills_recursive`（L41-59）

- `skills_dir` 配下を再帰走査。
- 直下に正確な `SKILL.md`（`has_exact_skill_manifest`、OsStr 厳密比較）を持つディレクトリを採用。
- **採用したら配下には潜らない**（`continue`）→ `assets/` 等内のネスト `SKILL.md` を別 Skill にしない。
- 中間ディレクトリ名は戻り値に含めない（`bar/foo/SKILL.md` → name=`foo`）。
- symlink は辿らない。

```41:58:src/scan/components.rs
fn collect_skills_recursive(current: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in current.read_dir_entries() {
        // symlink は無限ループ防止のため辿らない
        if is_symlink(&entry) {
            continue;
        }
        if !entry.is_dir() {
            continue;
        }
        // ...
        if has_exact_skill_manifest(&entry) {
            out.push((entry_name, entry));
            continue; // ← 配下に潜らない
        }
        collect_skills_recursive(&entry, out);
    }
}
```

---

## 4. 既存テスト（Skill deploy / scan / nested / bundled）

### `src/scan/components/tests.rs`（関連）

| テスト | 内容 |
|--------|------|
| `test_list_skill_names_extracts_dirs_with_skill_md` | 直下 SKILL.md 検出 |
| `test_list_skill_names_excludes_dirs_without_skill_md` | SKILL.md 無しは除外 |
| `test_list_skill_names_lowercase/mixed_case/...` | ケース感度・ディレクトリ偽装拒否 |
| `test_list_skill_names_one_level_nested` | `bar/foo/SKILL.md` → `foo` |
| `test_list_skill_names_multi_level_nested` | 多段ネスト |
| `test_list_skill_names_does_not_descend_into_skill` | `skill1/assets/inner/SKILL.md` を二重検出しない |
| `test_list_skill_names_does_not_descend_into_arbitrary_bundled_dirs` | `assets/references/templates/docs/examples/foo-bar` 全確認（#392 提案相当） |
| `test_list_skill_names_ignores_loose_markdown_beside_skill` | 直下 `notes.md` / orphan md は Skill にしない |
| `test_list_skill_names_mixed_flat_and_nested` | フラット+ネスト混在 |
| `test_list_skill_names_duplicate_basename_in_nested` | 同名 basename は scan 層で衝突検出しない |
| `test_list_skill_names_does_not_follow_symlinks` | symlink 非追跡 |

Property tests: `src/scan/components/proptests.rs`（`list_skill_names` 基本性質）。

### `src/component/deployment_test.rs`（関連）

| テスト | 内容 |
|--------|------|
| `test_execute_copies_directory_for_skill` | ディレクトリコピー + `helper.py` 同梱 |
| `test_execute_skill_replaces_existing_directory` | 既存置換 |
| `test_execute_skill_strips_unsupported_frontmatter_for_codex` | Codex strip |
| `test_execute_skill_keeps_frontmatter_for_non_codex_target` | 非 Codex は保持 |
| `test_execute_skill_copies_bundled_resources_same_structure` | 全 TargetKind で `notes.md` / `references/` / `assets/` 同構造コピー |
| `test_execute_skill_strip_does_not_touch_bundled_markdown` | Codex/Gemini で付属 md 不変 |
| `test_execute_skill_replace_dir_removes_stale_bundled_resources` | stale 掃除 |
| `test_execute_skill_with_mock_fs_replaces_existing_directory` | MockFs replace |

ヘルパ: `write_skill_with_bundled_resources` / `assert_bundled_resources_copied`（L512-548）。

---

## 5. ターゲット別 Skill 配置ディレクトリ名

共通ヘルパ: `skill_dir(base, name)` → `base/skills/<name>/`（`src/target/placed/placement_helpers.rs:6-12`）。

配置に渡す `name` の決め方:

| Target | ディレクトリ名 | base 例（Project） | コード |
|--------|----------------|-------------------|--------|
| Codex | `flatten_name` = `{plugin}_{skill}`（`context.name()`） | `.codex/skills/<name>/` | `codex.rs:147` |
| Copilot | 同上 | `.github/skills/<name>/` | `copilot.rs:98` |
| Antigravity | 同上 | `.agent/skills/<name>/` | `antigravity.rs:140` |
| Gemini CLI | 同上 | `.gemini/skills/<name>/` | `gemini_cli.rs:88` |
| Cursor | **`original_name` のみ**（未設定なら配置不可） | `.cursor/skills/<original>/` | `cursor.rs:175-177`（#377） |

確認テスト例:

- Codex: `codex_test.rs:25-44` → `/project/.codex/skills/my-plugin_my-skill`
- Cursor: `cursor_test.rs:109-134` → `~/.cursor/skills/my-skill`

**ドキュメントずれ:** `docs/concepts/components.md` / `deployment.md` / `targets.md` の一部は旧 3 階層 `<marketplace>/<plugin>/<skill>` 表記のまま。コードはフラット `{plugin}_{skill}`（Cursor Skill のみ `original_name`）。`file-formats.md` Cursor Skills パスも `<flattened_name>` 表記が残っており、実装（`original_name`）と不一致。

---

## 6. ドキュメント該当箇所（引用）

### Skill 内部 vs Plugin ルート（#393 境界）

`docs/architecture/file-formats.md` L517-548（Skill 付属） / L552- 以降（Plugin 付属）:

> Skill ディレクトリ（直下に `SKILL.md` を持つディレクトリ）内のエントリのうち、PLM が別 Component … としてスキャン・登録しないファイル・フォルダは、**当該 Skill の付属リソース**として扱う。

契約表より:

- 配置: `deploy_skill` が `replace_dir` でディレクトリごとコピー
- frontmatter 変換は **`target_path/SKILL.md` のみ**
- スキャン: Skill 採用後は配下に潜らない
- Plugin 直下は別節「Plugin 付属リソース」

`docs/concepts/components.md` L24-25:

> Skill ディレクトリ内の `references/` / `assets/` など任意名の補助ファイル・フォルダは **付属リソース**として本体と一緒にターゲットへコピーされる  
> プラグイン直下の任意名フォルダ（`references/` 等）は **Plugin 付属リソース**として、プラグイン構造を保ったままターゲットへ配置される（#393）

`docs/concepts/deployment.md` L29-42: Skill ツリー例 + 同梱注記 + Plugin 付属との境界。

### `#392` 言及の有無

- `docs/` 全体を検索しても **`#392` 文字列は無し**（#395 で意図的に Issue 番号を docs から外した履歴あり）。
- 「Skill 付属リソース」節そのものが #392 相当の仕様本文。
- `#393` は file-formats の Plugin 付属節・concepts 注記でカバー（Issue 番号は file-formats に明示なし。関連は PR #407）。

---

## 7. ギャップ / リスク（今後のテスト・仕様向け）

既に #392 提案の中核（仕様化・scan 二重検出・deploy 同構造・strip 非干渉）は実装済み。残る注意点:

1. **E2E 未結合:** deployment の bundled テストは `target_path` を直指定しており、`Target::placement_location`（フラット名 / Cursor original_name）経由の結合は検証していない。
2. **Cursor 実行時のネスト SKILL.md:** 仕様どおり PLM は別 Component にしないが、Cursor 本体が再帰走査するため実行時に別 Skill に見える可能性あり（file-formats 既知制限）。
3. **symlink:** コピー挙動は保証外。
4. **別経路:** `enable` 等の `FileOperation::CopyDir` も実装上 `replace_dir`（`intent.rs:243-244`）。`sync` は docs 上 `copy_dir` のため stale 非対称が残る（主に Plugin 付属の話だが Skill 更新経路でも留意）。
5. **Plugin 付属 (#393):** プラグイン構造を保ったまま配置する方針（Issue #393 正本）。Skill 内部 bundled とは別コードパス。実装・テストは未着手。混同しないこと。
6. **概念 docs の配置パス表記**がコードのフラット化と乖離 — レビュア/利用者の誤解源。
7. **Issue #392 未クローズ** — 実装は #395 で入っているため、クローズ or 本文更新が望ましい。

---

## 参照パス一覧

| パス | 役割 |
|------|------|
| `src/component/deployment.rs` | `deploy_skill` |
| `src/component/deployment_test.rs` | Skill bundled / replace / strip テスト |
| `src/component/convert.rs` | `strip_skill_frontmatter_fields` / `skill_allowed_fields` |
| `src/component/convert_test.rs` | strip 単体テスト |
| `src/scan/components.rs` | `list_skill_names` |
| `src/scan/components/tests.rs` | ネスト / assets / 任意名 bundled スキャンテスト |
| `src/fs.rs` | `RealFs::replace_dir` |
| `src/target/placed/placement_helpers.rs` | `skill_dir` |
| `src/target/env/{codex,copilot,antigravity,gemini_cli,cursor}.rs` | 配置名決定 |
| `docs/architecture/file-formats.md` | Skill / Plugin 付属リソース仕様 |
| `docs/concepts/{components,deployment}.md` | 概念レベル記述 |
