# Spec 001: PLM_HOME / HOME 解決の一元化

Issue: [#344](https://github.com/DIO0550/plugin-manager/issues/344)  
事前レビュー: [docs/reviews/issue-344-plm-home.md](../../reviews/issue-344-plm-home.md)

## ファイル

| ファイル | 説明 |
|----------|------|
| [hearing-notes.md](./hearing-notes.md) | ヒアリング結果 |
| [exploration-report.md](./exploration-report.md) | コードベース探索 |
| [requirements.md](./requirements.md) | ユースケース・要件（確定済み） |
| [implementation-plan.md](./implementation-plan.md) | 実装計画 |
| [tasks.md](./tasks.md) | TDD タスクリスト |
| [test-cases.html](./test-cases.html) | テストケース詳細（ブラウザで開く） |
| [tech-reference.md](./tech-reference.md) | 技術リファレンス |
| [understanding-quiz-plan.html](./understanding-quiz-plan.html) | 実装前理解度クイズ |

## 確定した設計判断

- **案 A**: `PLM_HOME` は `$HOME` の代替 → 実効パス `{root}/.plm`
- **対象外**: `paths.rs` / `cleanup.rs`（ユーザー HOME / Personal 配置）
- **エラー**: `plm_root` は `PlmError::General`、呼び出し側でドメイン別 `map_err`
- **MarketplaceConfig**: `Result<_, String>` 維持

## 実装開始

作業コピーのガード解除（ローカル）:

```bash
rm .plugin-workspace/.specs/.guard/cloud-agent-344 \
   .plugin-workspace/.specs/001-plm-home-unification/PLANNING
```
