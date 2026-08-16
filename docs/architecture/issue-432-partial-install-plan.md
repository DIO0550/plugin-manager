# #432 部分成功インストールの実装計画

> 作成日: 2026-08-16
> 関連 Issue: #432 / #343 / #345
> 方針: **部分成功を正とする**（失敗時ロールバックは採らない）

## 1. 判断

Issue は「部分成功として報告する」か「失敗時にロールバックする」かを選択するとしている。本対応では前者を採る。

理由:

- 既に書き込んだファイルの巻き戻しは Hook の共有ファイル（`hooks.json`）や Instruction の追記と衝突し、実装コストと副作用が大きい
- 成功したコンポーネントは利用可能な実体であり、捨てるより残して報告する方がユーザーの作業を減らす
- Issue の対応方針案 1〜4 はいずれも部分成功モデルを前提にしている

ロールバックは #345（エラーモデリング）側の将来検討とする。

#343（Outcome 型の統合）とは次の線を引く:

- 今回は `PlaceOutcome` / `PluginInstallOutcome` に件数と部分成功判定を足す
- `OperationOutcome` / `AffectedTargets` への統合はしない（三重モデルの解消は #343 の範囲）

## 2. 期待する状態

| 状況 | ディスク | `.plm-meta.json` | Plugin リソース / 旧階層 | 報告 |
|------|----------|------------------|--------------------------|------|
| 全成功 | 全コンポーネント配置 | `statusByTarget` = enabled | 実行する | `✓` |
| 部分成功 | 成功分は残す。失敗分は未配置 | 1 件以上成功した target は enabled | 実行する | `⚠ N/M placed, K failed` + 失敗行 |
| 全失敗 | 新規配置なし | 昇格しない（既存 meta は触らない） | 実行しない（旧階層を温存） | `✗` |
| 配置 0 / 失敗 0 | 変更なし | 触らない | 現行どおりクリーンアップ対象 | CLI: No matching components |

## 3. 変更範囲

### 3.1 `PlaceOutcome`（`src/install.rs`）

成功/失敗件数と target 単位の判定をメソッドとして追加する。新しい並立 Outcome 型は作らない。

- `placed_count` / `failed_count`
- `target_success_count` / `target_failure_count`
- `is_partial`
- `target_status_label` → `None` / `"PARTIAL"` / `"FAILED"`（CLI の `{target} - FAILED` 置き換え用）

### 3.2 `place_plugin` の後処理条件

現行: `if !target_had_failure` のときだけレガシークリーンアップと Plugin リソース配置。

変更後: **その target が「全失敗」でなければ実行する**。

```text
target_had_success || !target_had_failure
```

- 部分成功でもリソースを置く（コンポーネントだけ残ってリソースが無い不整合を防ぐ）
- 全失敗では旧階層を消さない（現行コメントのロールバック相当は維持）
- 配置 0・失敗 0（未サポート種別のみ等）のクリーンアップは現行どおり維持

### 3.3 `update_meta_after_place`

現行: 同じ target に failure があると `statusByTarget` を enabled にしない。

変更後: **その target に 1 件以上の success があれば enabled に昇格する**。全失敗のときは `.plm-meta.json` を書かない（mtime 汚染防止は維持）。

`enabled` は「その target に利用可能な配置がある」を意味する。失敗したコンポーネントの再試行は `plm install --force` / 変換修正で行う。

### 3.4 TUI `PluginInstallOutcome`

`success: bool` をやめ、件数で部分成功を表す。

```text
plugin_name / placed / failed / error / failure_lines
```

- 全成功: `✓ {name}`
- 部分成功: `⚠ {name}: {placed}/{placed+failed} components placed, {failed} failed` のあと、失敗ごとに `    - {target}/{component}: {error}`
- 全失敗: `✗ {name}` + 複数失敗は同様に箇条書き

`InstallSummary` に `partial` を追加する。見出しは部分成功があるとき `Installed X/Y plugins (Z partial)`。

結果 Paragraph は `Wrap { trim: false }` で折り返し、1 行切り詰めに頼らない。

### 3.5 CLI

- 成功行・失敗行の個別表示は現行どおり
- target に成功と失敗が混在するとき `{target} - PARTIAL`、失敗のみ `{target} - FAILED`
- `CommandSummary::format` は成功と失敗が両方あるとき接頭辞を `⚠` にする（`✗` は全失敗）。`plm import` も同じ関数を使うため、部分成功の見え方が揃う

CLI の終了コードは現行どおり変更しない（`run` は配置失敗でも `Ok(())`）。終了コードの厳密化は #345 に委ねる。

## 4. テスト計画

1. `update_meta_after_place`: 部分成功で enabled 昇格。全失敗では meta を書かない
2. `place_plugin`: Skill 成功 + Agent 変換失敗で、成功ファイルと Plugin リソースが残り、失敗ファイルは無い
3. `place_plugin`: 全コンポーネント失敗では Plugin リソースを置かない
4. `build_install_summary`: 部分成功を `partial` として数え、`succeeded` に混ぜない
5. TUI `format_install_result_lines`: `⚠` 行 + 失敗の複数行。長いエラーが 1 行に連結されない
6. `CommandSummary::format`: `(n>0, m>0)` で `⚠`

手動 GUI テストは TUI の TestBackend で代替する（ヘッドレス）。

## 5. 非目標

- `OperationOutcome` への統合（#343）
- 失敗コンポーネントの自動ロールバック
- YAML パース寛容化（#431 で対応済み）
- `plm install` の非ゼロ終了コード化
