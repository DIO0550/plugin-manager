# Issue #323 レビュー: `plm init` / `plm pack` 未実装スタブ

レビュー日: 2026-07-31  
HEAD: `f6fdd59`  
対象 Issue: [#323](https://github.com/DIO0550/plugin-manager/issues/323)  
役割: 実装前レビュー（Rust 変更なし）

**総括:** Issue の事実確認は正しい。両コマンドは CLI 配線済みだがハンドラは常に `not implemented`。`docs/commands/{init,pack}.md` と roadmap は未実装を明示済み（Issue 本文の「roadmap ✅」は現行リポジトリでは解消済み）。実装前に **CLI ↔ docs ↔ `ComponentKind` の型名不一致** と **pack バリデーション／出力名の未定義** を仕様として固める必要がある。再利用可能な frontmatter・`plugin.json`・ZipWriter 基盤は揃っている。

---

## 1. 現状確認

### 1.1 スタブ

| ファイル | 挙動 |
|----------|------|
| `src/commands/manage/init.rs` | `println!` 後 `Err("not implemented")` |
| `src/commands/manage/pack.rs` | 同上 |

```22:25:src/commands/manage/init.rs
pub async fn run(args: Args) -> Result<(), String> {
    println!("init: {:?}", args);
    Err("not implemented".to_string())
}
```

```11:14:src/commands/manage/pack.rs
pub async fn run(args: Args) -> Result<(), String> {
    println!("pack: {:?}", args);
    Err("not implemented".to_string())
}
```

### 1.2 CLI 配線（完了）

| 箇所 | 状態 |
|------|------|
| `src/cli.rs` — `Command::Init` / `Command::Pack` | ✅ |
| `src/commands.rs` ディスパッチ | ✅ |
| `src/commands/manage.rs` モジュール宣言 | ✅ |
| ハンドラ本体 | ❌ スタブ |

Init の `long_about` は「Generate plugin templates…」で、未実装注記なし。Pack は一行ヘルプのみ。実行時ヘルプにも「(未実装)」は出ない。

### 1.3 ドキュメント（現行）

| ドキュメント | 記載 |
|--------------|------|
| `docs/commands/init.md` / `pack.md` | 冒頭に ⚠️ 未実装 |
| `docs/commands/index.md` | 「※未実装スタブ」 |
| `docs/roadmap.md` Phase 9 | 🚧、両コマンド未チェック |

Issue 本文の「roadmap が ✅ 完了とマーク」は **現行 main では既に 🚧／未実装表記に修正済み**。ドキュメント側の別 PR 修正は完了していると判断してよい。

---

## 2. 仕様 ↔ CLI の不一致（実装ブロッカー級）

実装に入る前に揃えるべき点。

| 項目 | `docs/commands/init.md` | CLI (`ComponentType`) | ドメイン (`ComponentKind`) |
|------|-------------------------|------------------------|----------------------------|
| Skill | `skill` | `skill` | `Skill` |
| Agent | `agent` | `agent` | `Agent` |
| Command | **`command`** | **`prompt`** | `Command`（ファイルは `.prompt.md`） |
| Instruction | なし | **`instruction`** | `Instruction` |
| Hook | なし | なし | `Hook` |

### 指摘

1. **`--type command` vs `prompt`**  
   docs / roadmap 例は `command`、CLI ValueEnum は `Prompt` → clap 値 `prompt`。ドメインは `ComponentKind::Command`。  
   **推奨:** CLI を `Command` に揃え、必要なら `prompt` を alias（非推奨）として残す。docs・ヘルプ・`cli_test` を同期。

2. **`Instruction` が docs に無い**  
   CLI だけにある。出力パス（`instructions.md`? `AGENTS.md`?）、frontmatter 有無、ディレクトリ vs 単一ファイルが未定義。  
   **推奨:** MVP では docs どおり skill/agent/command のみ実装し、Instruction は CLI から外すか docs にテンプレを追加してから実装。

3. **Issue 提案の plugin 全体スキャフォールド（`--with`）**  
   Claude Code の `claude plugin init --with skills,agents,hooks` 相当。現行 docs / CLI に無い。  
   **推奨:** Phase 9 MVP はコンポーネント単位テンプレに限定し、`--with` / plugin.json 付きスキャフォールドは Phase 9.1 または別 Issue に分離。

---

## 3. `plm pack` 仕様の未定義点

`docs/commands/pack.md` は例示レベルで、実装判断に足りない。

| 項目 | docs | 未決 |
|------|------|------|
| 対象判定 | skill 単体 or プラグイン | `SKILL.md` 直下? `.claude-plugin/plugin.json`? 両方? |
| ZIP 名 | `<name>.zip` | ディレクトリ名 vs `plugin.json.name` vs skill frontmatter `name` |
| 出力先 | CWD | `--output` の要否 |
| 上書き | 未記載 | 既存 zip で失敗 / `--force` |
| バリデーション | 必須ファイル・frontmatter・plugin.json | Skill の必須 FM フィールド、空 body 可否、厳格さ |
| 除外 | 未記載 | `.git` / `.plm-meta.json` / symlink（リソース仕様では除外傾向） |
| install との関係 | 触れず | 現状キャッシュ格納は **manifest 必須**。skill 単体 zip は install 不可 |

**推奨（MVP）:**

1. **プラグイン判定:** `.claude-plugin/plugin.json` またはルート `plugin.json` があればプラグイン。なければ直下 `SKILL.md` なら skill 単体。それ以外はエラー。
2. **ZIP 名:** プラグインは `PluginManifest.name`、skill はディレクトリ名（または frontmatter `name`）。衝突時はエラー。
3. **バリデーション:**  
   - プラグイン: `PluginManifest::parse` 成功 + 宣言パス上のコンポーネント存在  
   - Skill: `SKILL.md` 存在 + `parse_frontmatter` で YAML 構文 OK（必須キーは `name` / `description` を最低ラインに）  
   - Agent/Command 単体パスを pack 対象にするかは docs 未記載 → MVP では skill dir / plugin dir のみで十分
4. **除外:** `.git/`、`.plm-meta.json`、シンボリックリンクは含めない。

---

## 4. 再利用可能な既存基盤

新規に発明する必要が少ない部分。

| 関心事 | 既存コード | 備考 |
|--------|------------|------|
| ZIP 展開 | `src/plugin/cache/cache.rs` (`ZipArchive`) | 読み取りのみ |
| ZIP 作成 | テストの `ZipWriter`（`cache_test.rs` 等） | 本番ヘルパへ昇格可 |
| 依存 | `zip = "2"`、`walkdir = "2"` | walkdir は src 未使用の可能性 |
| Frontmatter | `parser/frontmatter.rs` (`parse` / `emit_frontmatter`) | emit は `pub(crate)` |
| plugin.json | `plugin/meta/manifest.rs` + `manifest_resolve.rs` | 推奨パス `.claude-plugin/plugin.json` |
| 種別・拡張子 | `component/model/kind.rs` | `skill_manifest()`, `file_suffix()` |
| スキャン | `scan/components.rs` | pack 前の一覧検証に流用可 |
| FS 抽象 | `fs.rs`、`link.rs` の `--force` パターン | 上書きポリシー参考 |
| エラー | `PlmError::Zip` / `InvalidManifest` | 検証エラーはこれらへ寄せるのが一貫 |

スキャフォールド専用モジュールは **存在しない**。テンプレ文字列は init 側に置くか、`src/commands/manage/templates/` 等の Feature 内配置が妥当（レイヤー分離より Feature 凝集）。

---

## 5. 作者ワークフローとの依存

Issue が触れる「ローカルパス install」は **別ギャップ**。

- `source.rs` の `parse_source` は GitHub / marketplace / 検索のみ。`./` や絶対パスは未対応。
- `docs/commands/install.md` も同様。
- `PluginSource::Local` は marketplace.json 内のリポジトリ相対パスであり、ホスト FS からの install ではない。

したがって init → pack → GitHub 公開 → `plm install owner/repo` が現状の唯一のループ。  
init/pack 単体でも価値はあるが、「作成→検証→ローカル再 install」は **ローカルパス install（別 Issue）** とセットで閉じる。roadmap 将来候補の `plm validate` / `plm dev` とも設計が重なる点に注意。

`plm link` はローカル編集物をターゲットへ繋ぐ補助にはなるが、pack/init の代替ではない。

---

## 6. Issue 提案への評価

| Issue 提案 | 評価 |
|------------|------|
| 1. `plm init` を docs 仕様どおり実装 | ✅ 妥当。型名不一致の解消が前提 |
| 1b. `--with` 相当の plugin 全体スキャフォールド | △ 有用だがスコープ拡大。分離推奨 |
| 2. `plm pack` ZIP + 基本バリデーション | ✅ 妥当。判定・命名・除外を仕様化してから |
| 3. 暫定でヘルプに「(未実装)」or hidden | ✅ 実装が遅れる場合の UX 改善として妥当。docs は既に警告済み。CLI ヘルプ未追従が残課題 |

### 暫定緩和（実装遅延時）

優先度低〜中。変更は小さく、Rust でも docs でも可。

1. `cli.rs` の Init/Pack 説明に `(unimplemented)` を付与
2. または clap `hide = true`（破壊的ではないが発見性が下がる）
3. 実行時エラーを `not implemented` から「See docs/commands/init.md」付きメッセージへ

---

## 7. 推奨実装順（実装者向け）

1. **仕様揃え（docs + CLI）**  
   - `--type` を `skill|agent|command`（+ 任意で `instruction`）に固定  
   - `prompt` は alias か削除  
   - pack の対象判定・ZIP 名・上書き・除外・バリデーション最低ラインを `pack.md` に追記
2. **`plm init`（コンポーネント単位）**  
   - docs のテンプレ文字列を生成  
   - 既存パス時はエラー（`--force` は任意）  
   - skill → `name/SKILL.md`、agent/command → カレント直下の単一ファイル
3. **`plm pack`**  
   - 種判定 → バリデーション（既存 `parse_frontmatter` / `PluginManifest`）→ `ZipWriter`  
   - ユニットテストで zip 内容・バリデーション失敗を固定
4. **（任意・別 Issue）** plugin scaffold `--with`、ローカルパス install、`plm validate`

### テスト方針（TDD）

- Red: `init` が指定パスに期待ファイルを書く / 既存で失敗  
- Red: `pack` が有効 skill/plugin で zip を作り、不正 frontmatter / 欠落 manifest で失敗  
- 統合: 生成物を一時ディレクトリで pack し、エントリ一覧を assert（既存 `create_test_archive` の逆）

---

## 8. 結論・次アクション

| 優先 | アクション | 担当想定 |
|------|------------|----------|
| P0 | CLI/docs の `--type` 名を `ComponentKind` に揃える | 実装 Issue 内の準備コミット可 |
| P0 | `pack.md` に判定・命名・除外・バリデーションを追記 | 仕様 |
| P1 | `plm init` / `plm pack` 本体 + テスト | 本 Issue |
| P2 | CLI ヘルプに未実装明示（実装遅延時） | 本 Issue または小さな PR |
| P3 | `--with` scaffold / ローカル install | 別 Issue |

**ブロッカー:** 型名不一致と pack の対象判定・ZIP 名未定義を放置したまま実装すると、docs・ヘルプ・`ComponentKind`・将来の install 経路がまた分岐する。  
**非ブロッカー:** roadmap の ✅ 記述（既に修正済み）。Issue 本文の更新は任意。
