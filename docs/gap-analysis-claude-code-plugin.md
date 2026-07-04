# Claude Code Plugin コマンドとのギャップ分析

PLM の現状仕様を Claude Code のプラグインシステム（`claude plugin` CLI / `/plugin` REPL、v2.1.200 時点の公式ドキュメント）と突き合わせ、「仕様として不足している部分」「Claude Code の Plugin コマンドより不便になっている部分」を洗い出した調査レポート。

調査日: 2026-07-04 / 対象: `plm` v0.6.0（`main` df69dad 時点）

- Claude Code 側の根拠: https://code.claude.com/docs/en/plugins-reference / plugin-marketplaces / discover-plugins / plugin-dependencies
- PLM 側の根拠: 各節に `ファイルパス:行番号` で記載

---

## 1. 総括

| 区分 | 件数の目安 | 代表例 |
|------|-----------|--------|
| **A. 実装とドキュメントの齟齬（虚偽広告状態）** | 4 | `plm init` / `plm pack` が未実装スタブ、TUI Discover タブがスタブ、`config.toml` がドキュメントのみ |
| **B. plugin.json / marketplace.json 仕様の未対応** | 10+ | `mcpServers`・`lspServers` の黙殺、source バリアント不足、`metadata.pluginRoot`・`strict` 無視 |
| **C. Claude Code Plugin コマンドと比べて不便な点** | 15+ | ローカルパスからインストール不可、非対話環境で TUI 必須、scope 体系の差、validate/search/details 不在 |
| **D. バージョン・依存関係管理の欠落** | 4 | semver 無視（SHA 比較のみ）、lockfile なし、依存関係なし、マーケットプレイスの ref 固定なし |
| **E. チーム・共有機能の欠落** | 3 | プロジェクト設定によるプラグイン共有なし、managed/強制有効化なし、自動更新なし |

---

## 2. 【A】実装とドキュメントの齟齬（最優先で解消すべき）

ユーザーがドキュメント/ヘルプを信じて使うと壊れる箇所。

### A-1. `plm init` / `plm pack` が未実装スタブ

- `src/commands/manage/init.rs:22-25` — `Err("not implemented")`
- `src/commands/manage/pack.rs:11-14` — `Err("not implemented")`
- 一方で `src/cli.rs` の long_about とヘルプにはオプション込みで掲載され、`docs/roadmap.md:70-73` では Phase 9 が **✅ 完了** になっている。
- Claude Code には `claude plugin init <name>`（`--with skills,agents,hooks,mcp,...` でスキャフォールド）が実装済みで、プラグイン開発の入り口として機能している。PLM は入り口が壊れている。

### A-2. TUI Discover タブがプレースホルダ

- `src/tui/manager/screens/discover.rs` — `Msg` enum が空、`update` は no-op、フィルタは「未対応」表示（`discover.rs:116`）。
- `docs/roadmap.md:90` では「Discoverタブ（マーケットプレイス検索・インストール）✅」と完了扱い。
- Claude Code の `/plugin` Discover タブは検索バー・コンテキスト連動サジェスト・「Will install」コンポーネント一覧・scope 選択まで持つ。**PLM はマーケットプレイスのプラグインを「眺めて選んでインストールする」導線が CLI にも TUI にも存在しない**（`plm marketplace show <name>` のテーブル表示のみ）。

### A-3. `~/.plm/config.toml` がドキュメントのみで未実装

- `docs/reference/config.md` に `[general] default_scope`、`[targets]` パス上書き、`PLM_HOME`/`PLM_CONFIG` 環境変数まで記載があるが、**読み込む実装がどこにも存在しない**（`src/config.rs` は HTTP クライアント設定であり別物）。
- ターゲットの出力先パスはハードコード定数（例: `src/target/env/copilot.rs:11-12`）。
- `default_scope` が効かないため、後述 C-2 の「毎回 `--scope` を指定するか TUI」問題を悪化させている。

### A-4. roadmap の完了マークの信頼性

上記により `docs/roadmap.md` の Phase 9 / Phase 12（Discover）は実態と不一致。ロードマップを実態に合わせて修正する必要がある。

---

## 3. 【B】plugin.json / marketplace.json 仕様の未対応

Claude Code エコシステムの資産（既存プラグイン・マーケットプレイス）をそのまま持ち込めない原因になっている箇所。

### B-1. `mcpServers` / `lspServers` が**警告なしで黙殺**される（最重要）

- `src/plugin/meta/manifest.rs:49-52` で `mcpServers` / `lspServers` はデシリアライズされるが、`Plugin::build_components`（`src/plugin/content/plugin_content.rs:139-169`）は Skill/Agent/Command/Hook/Instruction しか走査せず、**MCP・LSP は捨てられ、警告すら出ない**。
- `ComponentKind`（`src/component/model/kind.rs:12-23`）に MCP/LSP が存在しない。
- Codex（`config.toml` の `mcp_servers`）、Copilot（VS Code の MCP 設定）、Gemini CLI（`settings.json` の `mcpServers`）はいずれも MCP をサポートしており、変換先が存在するのに落としている。
- **最低限やるべきこと**: 変換未対応でも「このプラグインは MCP サーバー N 件を含むが PLM は未対応」と install/import 時に警告を出す。現状はユーザーが欠落に気づけない。

### B-2. marketplace.json の source バリアント不足

`src/marketplace/registry.rs:22-29` の `PluginSource` は 2 種のみ:

| Claude Code の source | PLM 対応 |
|---|---|
| 相対パス `"./plugins/x"` | ○（`Local`） |
| `{"source":"github","repo":...}` | △（`repo` のみ使用。`ref`/`sha` フィールド**非対応** = バージョン固定不可） |
| `{"source":"url","url":...}`（汎用 git URL） | ✗ |
| `{"source":"git-subdir",...}`（モノレポ sparse） | ✗ |
| `{"source":"npm","package":...}` | ✗ |

さらに `External { source, repo }` の `source` 値は検証されず無視される（`src/source/marketplace_source.rs:98-108`）— `"source":"gitlab"` と書かれていても GitHub として扱おうとする。

### B-3. marketplace.json のトップレベルフィールド無視

- `metadata.pluginRoot` 無視 → プラグインルートを移動しているマーケットプレイスが解決不能。
- `strict` 無視 → マーケットプレイス側でコンポーネント定義を完結させるプラグイン（`strict: false`）が正しく展開されない。
- `description` / `version` / plugin エントリの `displayName`・`category`・`tags`・`keywords`・`defaultEnabled`・`license`・`author` 等のメタデータも未使用（表示・検索に活かされない）。

### B-4. plugin.json のコンポーネントパスが「文字列 1 個」のみ

- Claude Code は `commands` / `agents` / `skills` に **string | array** を許容し、`hooks` / `mcpServers` はインラインオブジェクトも可。PLM は `Option<String>`（`manifest.rs:40-48`）のみで、配列指定のプラグインはパースエラーまたは欠落になる。

### B-5. plugin.json の未対応フィールド（メタデータ・機能系）

- `displayName` / `defaultEnabled` / `dependencies`（semver 制約付き依存）/ `userConfig` / `outputStyles` / `channels` — いずれも非対応。依存関係は roadmap でも「将来」扱い（`docs/roadmap.md:144`）。
- マニフェストなしプラグイン（Claude Code はルート直下 `SKILL.md` 単体プラグインを許容）は PLM では `plugin.json` 必須（import はマニフェスト無しでエラー: `docs/import.md:136`）。

### B-6. `${CLAUDE_PLUGIN_ROOT}` 置換が hooks 限定

- hooks のラッパースクリプトでのみ `@@PLUGIN_ROOT@@` → 絶対パス置換される（`src/hooks/converter/copilot.rs:409`、`src/.../hook_deploy.rs:168`）。
- **スキル/エージェント/コマンドの markdown 本文内の `${CLAUDE_PLUGIN_ROOT}` は未置換のまま配置される**ため、スクリプト同梱型スキル（`scripts/` を参照するタイプ）は移植後に壊れる。
- `${CLAUDE_PLUGIN_DATA}` / `${CLAUDE_PROJECT_DIR}` / `${user_config.*}` は概念ごと存在しない。

### B-7. Hooks 変換の制約

- 変換先は Codex / Copilot のみ（`src/hooks/converter/converter.rs:280-284`）。
- 対応イベントが限定的（Codex: SessionStart, Pre/PostToolUse, UserPromptSubmit, Stop, PermissionRequest, Pre/PostCompact, SubagentStart/Stop。Copilot はさらに少ない）。Claude Code は 30 種超のイベントを持つため、大半が `UnsupportedEvent` 警告で脱落する。
- `prompt` / `agent` 型フックはスタブスクリプト化され手書き修正が必要（`PromptAgentHookStub` 警告）。
- Codex はスコープごとに単一 `hooks.json` のみで、**複数フックプラグインの共存が不可**（マージ未実装、`src/target/env/codex.rs:59-72`）。2 個目のフック付きプラグインを入れようとすると拒否される。

---

## 4. 【C】Claude Code Plugin コマンドより不便な点

### C-0. コマンド対応表

| Claude Code | PLM | 差分 |
|---|---|---|
| `claude plugin install x@market` (`--scope user/project/local`) | `plm install x@market` (`--target`, `--scope personal/project`) | scope 体系が異なる（C-1）。**デフォルト scope がなく未指定は TUI**（C-2） |
| `claude plugin uninstall` (`--keep-data`, `--prune`, `-y`) | `plm uninstall` (`--force`) | data 保持・依存整理なし。`-y`/`--force` の非 TTY 挙動が未定義（確認プロンプトが stdin を読む: `uninstall.rs:99-112`） |
| `claude plugin enable/disable` | `plm enable/disable` | **`--target` が codex/copilot のみ**（C-3）。設定トグルではなく物理ファイル配置/削除（C-4） |
| `claude plugin update [--all 相当なし→全 plugin 名]` | `plm update <name> / --all` | PLM の `--all` はバッチアトミックで優秀。ただし SHA 比較のみ（D-1） |
| `claude plugin list --json --available` | `plm list --json/--simple/--outdated` | インストール済みのみ。**「利用可能なプラグイン一覧」が出せない** |
| `claude plugin details <name>`（コンポーネント一覧+トークンコスト） | `plm info -f table/json/yaml` | 概ね同等（トークンコストは PLM に不要）。 |
| `claude plugin validate [path] --strict` | **なし** | プラグイン作者向け検証手段ゼロ（roadmap「将来」） |
| `claude plugin init` | スタブ（A-1） | |
| `claude plugin tag` (リリースタグ作成) | なし | |
| `claude plugin prune` | なし（依存概念自体なし） | |
| `claude plugin marketplace add <gh\|git URL\|https .json\|local path>[@ref]` | `plm marketplace add <owner/repo>` (`--path`) | **GitHub リポジトリのみ**。ローカルパス・汎用 git・URL 配信 json 不可。`@ref` 固定不可（D-4） |
| `claude plugin marketplace list --json` | `plm marketplace list` | JSON 出力なし（スクリプト連携不可） |
| `claude plugin marketplace update [name]` | `plm marketplace update [name]` | 同等 |
| `/plugin` Discover UI | TUI Discover タブ = スタブ（A-2） | |
| `/plugin` インストール時「Will install」内訳表示 | なし | 何が入るか入れる前に分からない（`plm marketplace show` は名前と説明のみ） |
| `claude --plugin-dir ./x`（ローカル開発ロード） | **なし** | C-5 参照。ローカル開発ループが成立しない |
| shell 補完 | なし（`clap_complete` 未導入） | |

### C-1. スコープ体系の差

- Claude Code: `user` / `project` / `local`（gitignore される個人用プロジェクト設定）+ `managed` の 4 層。既定は `user` で、**フラグ未指定でも即インストールできる**。
- PLM: `Personal` / `Project` の 2 層（`src/component/model/kind.rs:88-93`）。`local` 相当（プロジェクト内だが共有しない）がない。
- さらに Copilot は Personal スコープに Skill/Command/Instruction を置けない（`src/target/env/copilot.rs:43-48`）等、ターゲット×スコープの可否がユーザーから見えにくい。

### C-2. 非対話環境（CI・スクリプト）で詰まる

- `plm install` / `plm import` は `--target` / `--scope` 未指定時に **TUI 選択が起動する**（`install.rs:89-227`）。Claude Code はデフォルト scope=user で即完了。
- `docs/import.md:96` 自身が「CI環境など非対話環境では必ず指定が必要」と注意書きしている状態。`config.toml` の `default_scope`（A-3）が実装されていれば緩和できたはず。
- `plm uninstall` の確認プロンプトも非 TTY を考慮していない（`--force` 必須だが、非 TTY 検出で自動フェイルする等の挙動がない）。

### C-3. `enable` / `disable` / `update` の `--target` が codex/copilot 限定

- `src/commands/lifecycle/enable.rs:12-16` / `disable.rs:13-16` / `update.rs:10-13` が **ローカル定義の 2 値 enum** を使っており、Antigravity / Gemini CLI を指定できない。install/import/list/sync はグローバル `TargetKind`（4 値、`src/target.rs:123-130`）を使うため、コマンド間で受け付ける値が食い違う。
- Gemini にだけ入れたスキルを Gemini からだけ disable する、ができない。

### C-4. enable/disable が「物理ファイルの配置/削除」

- Claude Code の enable/disable は settings のトグルで、瞬時・可逆・宣言的。PLM はデプロイ済みファイルの削除と再変換コピー（`src/target.rs:236-257`、`src/commands/lifecycle/disable.rs:97-111`）。
- キャッシュ消失時に disable 不能（削除対象が分からない）、部分失敗で「Partially disabled」状態が生じる（`disable.rs:73-88`）。設計上やむを得ない面はあるが、`statusByTarget` と実ファイルの乖離（手動削除等）を修復する `doctor` / 再同期手段がない。

### C-5. ローカルパスからのインストール・開発ループ不可（重要）

- `plm install` のソース解決（`src/source.rs:48-64`）は GitHub / marketplace / 登録済み marketplace 検索の 3 通りのみ。**ローカルディレクトリを指定できない**。
- Claude Code は `--plugin-dir ./my-plugin` + `/reload-plugins` で編集→即試行のループが回る。PLM でプラグインを開発するには **一度 GitHub に push しないと自分のツールで試せない**。`plm init`/`plm pack` が動かない（A-1）ことと合わせて、作者向けワークフローが全断している。

### C-6. 検索・発見性

- `plm search` なし（roadmap 将来）。ベア名インストールは登録済み marketplace の完全一致検索のみ（`src/source/search_source.rs`）。部分一致・説明文検索・カテゴリ閲覧は不可。
- marketplace の `keywords`/`category`/`tags` をパースしていない（B-3）ので、実装しようにも材料を捨てている。

### C-7. 名前空間の flatten 方式

- Claude Code は `plugin-name:command` 名前空間で衝突を構造的に回避。PLM は `{plugin}_{name}` に平坦化（`src/plugin/content/plugin_content.rs:24-26`）。
- `_` 区切りのため、プラグイン名が互いの接頭辞になるケースで enabled 判定が曖昧になる既知問題がコード内に明記されている（`src/plugin/meta/meta.rs:366-403` の prefix-collision 注記）。
- マーケットプレイス側のリネーム追従（Claude Code の `renames` フィールドによる自動マイグレーション）に相当する仕組みなし。改名されたプラグインは旧名のまま更新不能になる。

### C-8. 変換ロスが利用者に見えにくい

- Codex 向け Command 変換で `allowed-tools` / `argument-hint` / `model` / `disable-model-invocation` / `user-invocable` が全部落ちる、Codex 向け Agent で `tools`/`model` が落ちる（`src/parser/claude_code/command.rs`、`src/parser/codex/agent.rs`）が、install 時のサマリでフィールド単位の欠落は通知されない（hooks のみ `RemovedField` 警告あり）。
- `$ARGUMENTS` 等の引数変数も Codex では未変換のまま残る。
- 「どのターゲットで何が失われるか」の変換マトリクスは `docs/architecture/file-formats.md` にあるが、コマンド出力からは分からない。

### C-9. `sync` から Hook が除外・delete が暗黙

- `SyncableKind`（`src/sync/model/options.rs:8-13`）は Skill/Agent/Command/Instruction のみで **Hook は同期不可**。
- sync は宛先にしかないコンポーネントを**削除する**ミラー動作（`src/sync.rs:48-127`）。`--dry-run` はあるが、削除を除外するオプション（`--no-delete`）がなく、手動で置いたものが消えるリスクがある。

### C-10. Errors タブが表示のみ

- Claude Code の `/plugin` Errors タブはロードエラーの診断入口。PLM の Errors タブはビューのみで再試行・詳細アクションなし（`src/tui/manager/screens/errors.rs`、「将来の拡張用」）。

### C-11. その他の細かい不便

- `plm marketplace list` に `--json` がない（`plm list` にはある）→ 出力形式の一貫性がない。
- `uninstall` の `MarketplaceArgs` はデフォルトなし、`enable`/`disable` は `"github"` デフォルト — コマンド間で marketplace 解決規則が微妙に違う。
- ターゲットレジストリのデフォルトに Gemini CLI が入っていない（`src/target/core/registry.rs:50-57` は Antigravity/Codex/Copilot のみ）— README が 4 環境対応を謳うのと不整合。
- TUI バッチ更新中に `q` で抜けられない（`src/tui/manager/manager.rs:53-69` に既知課題としてコメントあり）。
- shell 補完（`clap_complete`）なし。

---

## 5. 【D】バージョン・依存関係管理の欠落

### D-1. 更新判定がコミット SHA 比較のみ

- `needs_update`（`src/plugin/meta/version.rs:164-169`）は SHA の差のみ。`plugin.json` の `version` は表示専用。
- Claude Code は「`version` フィールドを上げたときだけ更新」という明示的リリース制御と「SHA 追従」の両方をサポートし、キャッシュもバージョン別ディレクトリで保持する。PLM ではマーケットプレイス運営者が安定版だけ配る手段がない（ブランチ ref 運用のみ）。
- git タグ・semver 解決（`plugin@^1.2.0` 的な指定）は一切ない。

### D-2. lockfile なし

- 状態は各プラグインの `.plm-meta.json` に分散。チームで「全員同じ SHA」を再現する手段（`plm install --locked` 相当）がない。roadmap でも将来扱い（`docs/roadmap.md:145`）。

### D-3. 依存関係なし

- Claude Code の `dependencies`（semver 制約、同一 scope への自動インストール、`prune` による孤児整理、cross-marketplace 許可リスト）に相当する機能が皆無。

### D-4. マーケットプレイス自体の ref 固定不可

- `plm marketplace add owner/repo` は既定ブランチの manifest を取るのみ。Claude Code の `marketplace add owner/repo@v2.0`（安定チャネル運用）に相当する固定ができない。マーケットプレイス登録は `owner/repo` 文字列で保存され ref を持たない（`src/marketplace/config.rs`）。

### D-5. ホストが GitHub 限定・zipball API 依存

- GitLab / Bitbucket は明示的にエラー（`src/repo.rs:124-129`）、`src/host.rs:119-124` は `panic!` を含む TODO。
- ダウンロードは GitHub REST の zipball（`src/host/github.rs:86-94`）であり git clone ではないため、**GHES や任意の git リモート・社内 Git には原理的に届かない**。Claude Code は任意 git URL・SSH・ローカル・URL 配信・npm まで対応。

---

## 6. 【E】チーム・共有機能の欠落

Claude Code Plugin の強みである「リポジトリを clone したら環境が揃う」体験に相当するものがない。

1. **プロジェクト設定によるプラグイン宣言なし** — Claude Code の `.claude/settings.json` の `extraKnownMarketplaces` + `enabledPlugins`（trust 後に自動セットアップ）に相当する「リポジトリに plm 設定をコミットして共有」する仕組みがない。PLM はプロジェクトスコープに**成果物ファイルを直接コミット**する運用になるが、その場合 update/disable の管理系コマンドとの整合（他人の clone 先では cache がない→ disable 不能、C-4）が壊れる。
2. **managed / 強制有効化・ブロックなし** — 企業管理（`strictKnownMarketplaces` 等）は対象外としても、「このリポジトリでは必ずこのスキルを入れる」の宣言的な仕組みがない。
3. **自動更新なし** — marketplace / plugin とも手動 `update` のみ。定期チェックや起動時チェック、`plm list --outdated` の自動化フック（CI 連携、roadmap 将来）もない。

---

## 7. 優先度付き推奨対応

### P0 — 嘘をなくす・黙殺をなくす（小工数で信頼性が大きく上がる）

1. **MCP/LSP を含むプラグインの install/import 時に警告を出す**（B-1）。変換実装より先に「捨てている」事実の通知を。
2. `plm init` / `plm pack` をヘルプから隠すか実装する。roadmap の ✅ を修正（A-1, A-4）。
3. `docs/reference/config.md` に「未実装」注記、または最小実装（`default_scope` だけでも）（A-3, C-2）。
4. `enable`/`disable`/`update` の `--target` をグローバル `TargetKind`（4 値）に統一（C-3）。

### P1 — 日常の不便の解消

5. **非対話デフォルト**: `--scope` 未指定かつ非 TTY 時のデフォルト（personal）を導入、`--yes` の追加（C-2）。
6. **ローカルパスからの install/import**（`plm install ./my-plugin`）＋ `plm init`/`pack` 実装で開発ループを成立させる（C-5）。
7. `plm search`（登録済み marketplace 横断の部分一致検索）と TUI Discover タブの実装。marketplace メタデータ（description/category/keywords）のパース（C-6, B-3, A-2）。
8. `plm validate`（plugin.json / marketplace.json / frontmatter 検証。自ドッグフード用にも有用）。
9. インストール前の「何が入るか」表示（Claude Code の Will install 相当）と、変換で落ちるフィールドの明示（C-8）。
10. marketplace ソースの `ref`/`sha` 対応 ＋ `plm marketplace add ...@ref`（B-2, D-4）。
11. `plm marketplace list --json`、shell 補完（C-11）。

### P2 — 仕様カバレッジ拡大

12. plugin.json のコンポーネントパス array 対応、`metadata.pluginRoot`、`strict: false`（B-3, B-4）。
13. markdown 本文の `${CLAUDE_PLUGIN_ROOT}` 置換（B-6）。
14. marketplace source `url` / `git-subdir` 対応（git clone ベースへの移行 or 各ホスト API 追加）（B-2, D-5）。
15. Codex hooks.json のマージ実装（複数フックプラグイン共存）（B-7）。
16. `renames` 追従、lockfile、依存関係（C-7, D-2, D-3）。
17. プロジェクト設定ファイル（例: `.plm/plugins.toml`）による宣言的セットアップ `plm apply`（E-1）。

---

## 付録: ターゲット別サポートと CLI の受理値の不一致一覧

| コマンド | `--target` の受理値 | 定義箇所 |
|---|---|---|
| install / import / list / sync | antigravity, codex, copilot, gemini | `src/target.rs:123-130` |
| enable / disable / update | codex, copilot のみ | `src/commands/lifecycle/{enable,disable,update}.rs` 各ローカル enum |
| target add/remove | antigravity, codex, copilot, gemini | グローバル enum |
| targets.json デフォルト | antigravity, codex, copilot（gemini なし） | `src/target/core/registry.rs:50-57` |
