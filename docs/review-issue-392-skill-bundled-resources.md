# Issue #392 実装検証レビュー: Skill 付属リソースの仕様化

| 項目 | 内容 |
|------|------|
| Issue | [#392 [feature/scan] Skill ディレクトリ内の未認識ファイル/フォルダを Skill と一緒に配置することを仕様化する](https://github.com/DIO0550/plugin-manager/issues/392) |
| レビュー日 | 2026-07-27 |
| 対象ブランチ | `main`（`2199419` 時点） |
| 実装 PR | [#395](https://github.com/DIO0550/plugin-manager/pull/395)（2026-07-24 MERGED） |
| 先行設計レビュー | PR #395 内コミット `f7f0c78`（実装後にレビュー成果物は意図的に削除） |
| 関連 | [#393](https://github.com/DIO0550/plugin-manager/issues/393) / [#407](https://github.com/DIO0550/plugin-manager/pull/407)（Plugin 付属リソース仕様）、[#339](https://github.com/DIO0550/plugin-manager/issues/339)、[#377](https://github.com/DIO0550/plugin-manager/issues/377) |

## サマリー

Issue #392 の提案（仕様化・スキャン保証・デプロイ保証・frontmatter 副作用固定）は **#395 で充足済み**。Issue 本文の「仕様・テスト不在」は outdated。追加の Rust 変更は不要。

**結論: 受け入れ完了。Issue #392 はクローズ推奨。**

## 提案項目と実装の対応

| Issue 提案 | 状態 | 根拠 |
|------------|------|------|
| 1. `file-formats.md` に付属リソース契約を明記 | ✅ | `docs/architecture/file-formats.md`「Skill 付属リソース」節（契約表・境界・Cursor 既知制限） |
| concepts 側の追随 | ✅ | `docs/concepts/components.md` / `deployment.md` |
| 2a. scan: 任意名フォルダ内 md の非二重検出 | ✅ | `test_list_skill_names_does_not_descend_into_arbitrary_bundled_dirs` |
| 2a'. 直下任意 md の非採用 | ✅ | `test_list_skill_names_ignores_loose_markdown_beside_skill` |
| 2b. deploy: 全 Skill 対応 TargetKind で同構造コピー | ✅ | `test_execute_skill_copies_bundled_resources_same_structure`（Codex / Copilot / Antigravity / Gemini CLI / Cursor） |
| 2b'. stale 掃除 | ✅ | `test_execute_skill_replace_dir_removes_stale_bundled_resources` |
| 3. strip は `target_path/SKILL.md` のみ | ✅ | `test_execute_skill_strip_does_not_touch_bundled_markdown` + `deploy_skill` 契約コメント |

先行設計レビューの「実装（後続 PR）」チェックリストもすべて満たしている。

## 現状コード（契約の中核）

`deploy_skill` は挙動変更なしで契約コメントが付与されている。

```83:109:src/component/deployment.rs
    /// Skill ディレクトリを丸ごと配置する。
    ///
    /// `SKILL.md` 以外の直下ファイルや任意名サブフォルダ（`references/` / `assets/` 等）も
    /// 付属リソースとして同相対構造でコピーする。`replace_dir` によりターゲット側の
    /// 余剰ファイルは削除される。frontmatter 変換がある場合も触るのは `SKILL.md` のみ。
    fn deploy_skill(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        fs.replace_dir(self.source_path(), &self.target_path)?;
        // ... strip は target_path/SKILL.md のみ ...
        Ok(DeploymentOutput::Copied)
    }
```

スキャンは Skill 採用後に配下へ潜らない（`collect_skills_recursive` の `continue`）。付属フォルダ内の `SKILL.md` は別 Component にならない。

## 先行レビュー指摘への反映確認

| 指摘 | 反映 |
|------|------|
| 「付属リソース」定義を明示 | ✅ 契約表で定義 |
| フォルダ名ホワイトリスト禁止 | ✅ 「制約なし」と明記 |
| stale 掃除を仕様に書く | ✅ 契約表 + テスト |
| #393 との境界 | ✅「Plugin 直下は Plugin 付属リソース節」 |
| Cursor 再帰走査は既知制限 | ✅ file-formats に明記 |
| 全ターゲット E2E は必須としない | ✅ `ConversionConfig::Skill` × TargetKind のユニットで固定（妥当） |
| symlink 保証外 | ✅ 境界節に一文 |
| enable 経路の strip 差はスコープ外 | ✅ 変更なし（付属同梱は両経路とも `replace_dir`） |

## 残ギャップ（#392 クローズを阻まない）

| 項目 | 深刻度 | 扱い |
|------|--------|------|
| `placement_location` 経由の E2E 未結合 | 低 | 先行レビューどおり任意。配置パス自体は #392 対象外 |
| enable（`PluginIntent::CopyDir`）は strip なし | 低 | 既存の経路差。付属リソース同梱契約は満たす。strip 統一は別 Issue |
| Cursor 実行時のネスト `SKILL.md` 可視 | 既知 | 仕様の既知制限。警告追加が必要なら別 Issue |
| concepts の一部パス例が 3 階層表記のまま | 低 | コードはフラット `{plugin}_{skill}`（Cursor Skill は `original_name`）。ドキュメント整備は横断タスク |
| Plugin 付属リソース（#393）の実装 | 別スコープ | 仕様は #407 で確定。配置コードは別途 |
| Issue #392 本文が outdated / 未クローズ | 運用 | 本レビューでクローズ推奨 |

## 受け入れ判定

| 観点 | 判定 |
|------|------|
| 仕様の明文化 | **合格** |
| スキャン契約の回帰テスト | **合格** |
| デプロイ契約の回帰テスト | **合格** |
| frontmatter 副作用の固定 | **合格** |
| #393 との境界 | **合格** |
| 追加実装の要否 | **不要** |

**Issue #392 はクローズしてよい。** 残ギャップはフォローアップ Issue 候補であり、本 Issue の受け入れ条件を再オープンする理由にはならない。

## ドキュメント配置について

`docs/architecture/` 配下への一時調査メモは仕様書と混ざるため置かない。本ファイルが #392 の実装検証記録とする。正規の契約本文は引き続き `docs/architecture/file-formats.md`「Skill 付属リソース」節。

> 注: 先行 PR #395 では設計レビュー成果物を実装後に削除した。本ファイルは Issue 未クローズ状態に対する**実装検証・クローズ判定**のため再記録する。受け入れ確認後に削除してよい。

## 関連

- #395 — 実装（仕様 + テスト）
- #393 / #407 — Plugin 直下付属リソース（別スコープ）
- #339 — 配置リテラル集約
- #377 — Cursor Skill の `original_name` 配置
