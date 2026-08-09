# Issue #418: OpenCodeTarget を実装する（Skills 配置）— 実装計画

> 状態: 実装計画  
> Issue: [#418](https://github.com/DIO0550/plugin-manager/issues/418)  
> Epic: [#416](https://github.com/DIO0550/plugin-manager/issues/416) Phase 2  
> blocked_by: [#417](https://github.com/DIO0550/plugin-manager/issues/417)（main マージ済み: [#425](https://github.com/DIO0550/plugin-manager/pull/425)）  
> 参照仕様: [`docs/concepts/targets.md`](../concepts/targets.md)「OpenCode」、[`opencode-target-plan.md`](./opencode-target-plan.md)  
> 参考実装: `src/target/env/cursor.rs`（Skills の `original_name` 配置）  
> 既存 PR: [#424](https://github.com/DIO0550/plugin-manager/pull/424)（古い #417 ブランチベース。**現行 main 上で新規実装**し、下記の設計のみ採用）

---

## 1. 目的・スコープ（Phase 2）

### 目的

`Target` trait を実装する `OpenCodeTarget` を追加し、**Skills の配置・列挙・上書きガード・所有権記録**を動かす。これにより `plm install --target opencode` / `plm list --target opencode` 等が Skill について動作する。

### Phase 2 スコープ（本 Issue）

| 項目 | 内容 |
|------|------|
| 実装体 | `src/target/env/opencode.rs` + `opencode_test.rs` |
| サポート | **Skill のみ**（Personal / Project 両方） |
| 配置 | `original_name` 必須の 1 階層フラット配置（Cursor #377 同型） |
| パス | Personal: `$XDG_CONFIG_HOME/opencode`（未設定時 `~/.config/opencode`）、Project: `.opencode` |
| 登録 | `parse_target` / `all_targets` / `env` モジュール配線 |
| 所有権 | overwrite ガード + `record_opencode_skill_ownership` |
| layout | `TargetKind::personal_base` を XDG 尊重に更新 |

### 制約（CLAUDE.md / AGENTS.md 準拠・implementer 必守）

- Feature ベース配置（`src/target/env/` に実装を置く。レイヤー分割しない）
- `mod.rs` 禁止。`src/target/env.rs` から `mod opencode;` で配線
- テストは本体と分離（`opencode_test.rs` + `#[path = "opencode_test.rs"]`）
- TDD: Red → Green → Refactor（失敗確認なしに実装へ進まない）
- ドメイン成果レポート型に `Result` 接尾辞を使わない（本 Issue では新規 Outcome 型は不要）
- コミットメッセージは英語
- `cargo check` / `cargo test` / `cargo fmt` は専用サブエージェント経由（直接 Bash しない）

---

## 2. 現状（#417 済み / 未実装）

### #417 で main に入っているもの（再実装不要）

| 領域 | 状態 | 場所 |
|------|------|------|
| `TargetKind::OpenCode`（clap `opencode` / serde） | ✅ | `src/target.rs` |
| `as_str` / `command_format` / `agent_format`（ClaudeCode） | ✅ | `src/target.rs` |
| `OPENCODE_PERSONAL_PARENT` / `CHILD` / `PROJECT_SUBDIR` | ✅ | `src/placement_names.rs` |
| `instruction_filename` → `AGENTS.md` | ✅ | `src/target/core/layout.rs` |
| `personal_base`（現状は `home/.config/opencode` 固定） | ✅（XDG 未対応） | `layout.rs` |
| `project_base` → `.opencode` | ✅ | `layout.rs` |
| `cleanup_specs`（skills/agents/commands × Personal/Project） | ✅ | `layout.rs` + `layout_test.rs` |
| `TargetsConfig` opt-in（デフォルト無効） | ✅ | `registry_test.rs` |
| `skill_allowed_fields` に OpenCode | ✅ | `src/component/convert.rs` |
| install 表示名 `"OpenCode"` | ✅ | `src/install/format.rs` |
| Phase 1 テスト（`parse_target("opencode")` は **未登録でエラー**を期待） | ✅ | `src/target_test.rs` |

### 未実装（本 Issue で追加）

| 領域 | 状態 |
|------|------|
| `src/target/env/opencode.rs`（`OpenCodeTarget`） | ❌ |
| `src/target/env/opencode_test.rs` | ❌ |
| `env.rs` の `mod opencode` / re-export | ❌ |
| `parse_target` / `all_targets` への登録 | ❌ |
| `personal_root_from_env`（XDG 解決）と `personal_base` への接続 | ❌ |
| `record_opencode_skill_ownership` | ❌ |
| Skill overwrite ガード / `pre_place_check` / `post_place` | ❌ |

### 既存 PR #424 について

- ベースが古い #417 ブランチのため、`TargetKind` 追加・`placement_names`・convert 等を差分に含む。
- **現行 main ではそれらを再適用しない。** 下記「採用すべき設計」のみ移植する。

---

## 3. 設計方針（パス、original_name、XDG、ownership）

### 3.1 パス解決

```
Personal: personal_root_from_env(home)
  ├─ XDG_CONFIG_HOME が非空 → PathBuf::from(xdg.trim()) / "opencode"
  └─ 未設定・空 → home / ".config" / "opencode"

Project: project_root / ".opencode"

Skill 配置: <base> / "skills" / <original_name> /   （中に SKILL.md）
```

- `OpenCodeTarget::base_dir(scope, project_root)`:
  - Personal → `personal_root()` = `personal_root_from_env(&home_dir())`
  - Project → `project_root.join(OPENCODE_PROJECT_SUBDIR)`
- Cursor の `base_dir(scope, ..., ".cursor", ".cursor")` とは異なり、Personal と Project でサブディレクトリ名が違うため、**専用 `base_dir` を持つ**（PR #424 同型）。

### 3.2 XDG（`EnvVar::get`）

```rust
pub(crate) fn personal_root_from_env(home: &Path) -> PathBuf {
    if let Some(xdg) = EnvVar::get("XDG_CONFIG_HOME").filter(|s| !s.trim().is_empty()) {
        return PathBuf::from(xdg.trim()).join(OPENCODE_PERSONAL_CHILD);
    }
    home.join(OPENCODE_PERSONAL_PARENT)
        .join(OPENCODE_PERSONAL_CHILD)
}
```

- `EnvVar::get` は空文字を `None` にする。加えて trim 後の空も弾く（空白のみ対策）。
- **`TargetKind::personal_base` も同じ関数を呼ぶ**（cleanup / plugin resources と Target 実装でパスがずれないようにする）。
- `OPENCODE_CONFIG_DIR` は v1 非スコープ（仕様どおり）。

### 3.3 `original_name` 必須

- `placement_location` で Skill のみ:
  - `context.original_name().filter(|n| !n.is_empty())?` が無い / 空なら **`None`（配置スキップ）**
  - ディレクトリ名は flatten 名ではなく `original_name`
- OpenCode は `skills/*/SKILL.md` の 1 階層のみ発見するため、ネスト配置は不可（Cursor #377 と同理由）。
- 緑地実装のため、Cursor にある **legacy flatten 名ディレクトリ削除は不要**。

### 3.4 `supported_components` / スコープ

```rust
const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill];
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] =
    &[(ComponentKind::Skill, ScopeSupport::Both)];
```

- Agent / Command / Instruction / Hook は `supports` が false、`placement_location` は `None`。
- Hooks は JS/TS Plugin モデルのため **永久に本 Epic 初回スコープ外**（後続 #419/#420 でも Hooks は入れない）。

### 3.5 overwrite ガード + ownership

Cursor Skill と同型:

1. `path_conflicts_with_unowned`: パスが存在し、かつ `meta.manages_file("opencode", path)` でない → 衝突
2. `skill_overwrite_error` → ユーザー向け拒否メッセージ
3. `pre_place_check` で Skill 時に上記を実行
4. `post_place` で `record_opencode_skill_ownership(plugin_root, deployed_path)`
5. `install.rs` に薄いラッパー:

```rust
pub fn record_opencode_skill_ownership(plugin_path: &Path, skill_path: &Path) {
    record_managed_file_ownership(plugin_path, skill_path, "opencode");
}
```

### 3.6 `list_placed`

- Skill + サポートスコープのみ: `scan_and_filter(&base, "skills", filter_skill_dir)`
- `SKILL.md` が無いディレクトリは列挙されない（既存 filter 挙動）

### 3.7 Target trait 実装の要点

| メソッド | 挙動 |
|----------|------|
| `display_name` | `"OpenCode"` |
| `kind` | `TargetKind::OpenCode` |
| `supported_components` | Skill のみ |
| `can_place_scope` | Skill × Both |
| `placement_location` | Skill + original_name → `skill_dir` |
| `pre_place_check` | Skill overwrite ガード |
| `post_place` | ownership 記録 |
| `list_placed` | Skill ディレクトリ走査 |
| `component_conflict_error` / legacy cleanup | デフォルト（何もしない）で可 |

### 3.8 登録

- `src/target/env.rs`: `mod opencode;` / `pub use opencode::OpenCodeTarget;` / `pub(crate) use opencode::personal_root_from_env;`
- `src/target.rs`: re-export に `OpenCodeTarget`、`parse_target("opencode")`、`all_targets` に追加
- `target_test.rs`: `test_parse_target_opencode_not_registered_yet` を **成功する登録テストに置換**、`all_targets` 件数を 5 → 6

### 3.9 layout_test の XDG 安定化

現行 `opencode_bases` / `cleanup_specs_opencode` は環境の `XDG_CONFIG_HOME` に依存しうる。`personal_base` を XDG 対応にした後は:

- テスト前後で Mutex ロック + `XDG_CONFIG_HOME` をクリア（または EnvGuard）
- デフォルトパス断言後、XDG 設定時のパスも検証（PR #424 の `layout_test` 変更を踏襲）

---

## 4. 変更ファイル一覧

| ファイル | 変更種別 | 変更内容 |
|---------|---------|---------|
| `src/target/env/opencode.rs` | **新規** | `OpenCodeTarget`、`personal_root_from_env`、overwrite / place / list |
| `src/target/env/opencode_test.rs` | **新規** | 名前・サポート・配置・XDG・list・overwrite の単体テスト |
| `src/target/env.rs` | 編集 | `mod opencode`、re-export、`personal_root_from_env` の `pub(crate) use` |
| `src/target.rs` | 編集 | re-export、`parse_target` / `all_targets` 登録、doc コメント更新 |
| `src/target_test.rs` | 編集 | opencode 登録テスト、`all_targets` 件数 6 |
| `src/target/core/layout.rs` | 編集 | `OpenCode` の `personal_base` → `personal_root_from_env(home)` |
| `src/target/core/layout_test.rs` | 編集 | XDG クリア + XDG 上書きケース |
| `src/install.rs` | 編集 | `record_opencode_skill_ownership` 追加 |

### 変更しないもの（#417 済み or 後続 Issue）

- `placement_names.rs` の OpenCode 定数（済み）
- `TargetKind` バリアント / formats（済み）
- `convert.rs` / `install/format.rs` の OpenCode 分岐（済み）
- Agents / Commands / Instructions の placement（#419 / #420）
- `docs/concepts/targets.md` の実装状況表記（最終 #421）
- Hooks / OpenCode Plugins

### PR #424 から採用する設計（チェックリスト）

- [x] `personal_root_from_env(home)` + `EnvVar::get`
- [x] `TargetKind::personal_base` も XDG 尊重
- [x] `record_opencode_skill_ownership`
- [x] skill overwrite ガード + ownership 記録（Cursor 同型）
- [x] `supported_components` は Skill のみ
- [x] `layout_test` の opencode 系は XDG クリア後に検証

### PR #424 から採用しない／注意

- #417 相当の重複差分（すでに main にある）
- Cursor の legacy flatten skill 削除ロジック（OpenCode には不要）
- Agents/Commands/Instructions の早期実装

---

## 5. TDD 手順（Red → Green → Refactor）とテスト観点

### Step 0: 準備

- 作業ブランチを現行 `main`（#417 マージ後）から切る。PR #424 ブランチはベースにしない。
- 参考に読む: `cursor.rs` / `cursor_test.rs`、PR #424 の `opencode.rs` / `opencode_test.rs`（設計参照のみ）。

### Step 1: Red — 失敗するテストを先に書く

`opencode_test.rs` と登録テストを追加し、**コンパイル失敗または assert 失敗を確認**する。

推奨テストケース（PR #424 を踏襲）:

| テスト | 観点 |
|--------|------|
| `test_opencode_name_and_kind` | `name` / `display_name` / `kind` |
| `test_opencode_supported_components_skills_only` | Skill のみ。Agent/Command/Instruction/Hook は非サポート |
| `test_opencode_supports_scope_skill_both` | Personal / Project 両方 |
| `test_opencode_placement_skill_project_uses_original_name` | `.opencode/skills/<original_name>` |
| `test_opencode_placement_skill_personal_default_home` | `~/.config/opencode/skills/...`（XDG 未設定） |
| `test_opencode_placement_skill_personal_respects_xdg_config_home` | `$XDG_CONFIG_HOME/opencode/skills/...` |
| `test_opencode_placement_skill_without_original_name_returns_none` | 配置スキップ |
| `test_opencode_list_placed_with_skills` | `SKILL.md` あり → 列挙 |
| `test_opencode_list_placed_empty_when_no_skill_md` | 空 dir → 非列挙 |
| overwrite 系 3 本 | 未存在 / 非所有 / 所有済み |
| `personal_root_prefers_xdg_over_home` | XDG 優先 |

併せて:

- `target_test`: `parse_target("opencode")` 成功、`all_targets` に含む（件数 6）
- `layout_test`: XDG クリア後の default + XDG 設定時の `personal_base`

環境変数テストは既存パターン（Mutex + EnvGuard）で並列汚染を防ぐ。

### Step 2: Green — 最小実装

1. `opencode.rs` に `OpenCodeTarget` + `personal_root_from_env`
2. `env.rs` / `target.rs` 配線と `parse_target` / `all_targets`
3. `install.rs` に ownership ラッパー
4. `layout.rs` の `personal_base` を XDG 対応
5. `layout_test` を XDG 安全に更新
6. テスト全通しを確認

### Step 3: Refactor

- Cursor と重複する overwrite 判定の表現を読みやすく保つ（過度な共通化はしない）
- コメントは「なぜ original_name 必須か」「Hooks 非対応」など設計意図に限定
- `cargo fmt` → type-check → test（CLAUDE.md の検証順）

### 手動確認（任意）

```bash
plm target add opencode
plm target list          # opencode が表示される
# Skill 付きプラグインを --target opencode で install し、
# Personal/Project の skills/<original_name>/SKILL.md を確認
```

---

## 6. 非スコープ

| 項目 | 扱い |
|------|------|
| Agents / Commands 配置 | [#419](https://github.com/DIO0550/plugin-manager/issues/419) |
| Instructions（Personal + Project） | [#420](https://github.com/DIO0550/plugin-manager/issues/420) |
| Hooks / OpenCode JS/TS Plugins | 別 Epic。`supported_components` に含めない |
| `OPENCODE_CONFIG_DIR` | v1 対象外 |
| Claude Code 互換パス（`.claude/`）への二重配置 | しない |
| Commands ネストパス保持 | しない |
| sync 名キー不一致の解消 | 既知制限（#384 同型）。本 Issue では触れない |
| `docs/concepts/targets.md` / roadmap 等の ✅ 更新 | [#421](https://github.com/DIO0550/plugin-manager/issues/421) |
| plugin-root resources と `plugins/` 衝突の最終方針 | 注記のみ。実装変更は本 Issue 外 |

---

## 7. 受け入れ条件チェックリスト

### 機能

- [ ] `OpenCodeTarget` が存在し、`display_name() == "OpenCode"`、`kind() == OpenCode`
- [ ] `supported_components()` が **Skill のみ**（Hook / Agent / Command / Instruction を含まない）
- [ ] Skill は Personal / Project 両方に配置可能
- [ ] Project Skill パス: `<project>/.opencode/skills/<original_name>/`
- [ ] Personal Skill パス（XDG 未設定）: `~/.config/opencode/skills/<original_name>/`
- [ ] Personal Skill パス（XDG 設定時）: `$XDG_CONFIG_HOME/opencode/skills/<original_name>/`
- [ ] `original_name` 未設定または空 → `placement_location` が `None`（スキップ）
- [ ] 非所有パスへの上書きを拒否する
- [ ] 自プラグイン所有パスへの再配置は許可し、ownership を記録する
- [ ] `list_placed(Skill, ...)` が `SKILL.md` 付きディレクトリ名を返す

### 配線

- [ ] `parse_target("opencode")` が成功する
- [ ] `all_targets()` に opencode が含まれる（件数 6）
- [ ] `TargetKind::personal_base` が XDG を尊重する（Target 実装と同じ解決）
- [ ] cleanup 列挙に `.opencode` および Personal config ルート配下の skills/agents/commands が含まれる（#417 済みを回帰させない）

### 品質

- [ ] テストは `opencode_test.rs` に分離
- [ ] TDD（Red 確認 → Green → Refactor）で進めた
- [ ] `cargo fmt` / type-check / `cargo test` がパス
- [ ] Agents / Commands / Instructions / Hooks を実装していない
- [ ] 現行 main ベースで実装し、PR #424 の古い #417 差分を再適用していない

### ドキュメント（本計画の位置づけ）

- [x] 本ファイル（`docs/architecture/issue-418-opencode-target-skills-plan.md`）を作成
- [ ] 実装完了後の概念ドキュメント更新は #421 に委譲

---

## 実装タスク分割（implementer 向け）

| ID | タスク | blockedBy | 主な変更ファイル |
|----|--------|-----------|------------------|
| 1 | Red: `opencode_test` / 登録・layout XDG テストを追加し失敗を確認 | - | `opencode_test.rs`, `target_test.rs`, `layout_test.rs` |
| 2 | Green: `OpenCodeTarget` + XDG ヘルパ実装 | 1 | `opencode.rs` |
| 3 | Green: モジュール登録・re-export・`parse_target` / `all_targets` | 2 | `env.rs`, `target.rs` |
| 4 | Green: ownership ラッパー + `personal_base` XDG 接続 | 2 | `install.rs`, `layout.rs` |
| 5 | Refactor + 全テスト緑・受け入れ条件確認 | 3, 4 | （上記一式） |

タスク 3 と 4 は 2 完了後に並列可能。

---

## 関連

- Epic [#416](https://github.com/DIO0550/plugin-manager/issues/416)
- Phase 1 [#417](https://github.com/DIO0550/plugin-manager/issues/417) / PR [#425](https://github.com/DIO0550/plugin-manager/pull/425)
- 既存ドラフト PR [#424](https://github.com/DIO0550/plugin-manager/pull/424)（設計参照・ベース非推奨）
- Cursor Skills original_name [#377](https://github.com/DIO0550/plugin-manager/issues/377)
- sync 名キー既知制限 [#384](https://github.com/DIO0550/plugin-manager/issues/384)
