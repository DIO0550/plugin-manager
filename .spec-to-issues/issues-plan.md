# Issue 分解計画: Hooks 自動変換機能 (Claude Code -> Copilot CLI)

> 生成元: `docs/hooks-conversion/index.md`, `config-converter-spec.md`, `script-wrapper-spec.md`, `install-integration-spec.md` v1.0
> リポジトリ: DIO0550/plugin-manager
> 生成日: 2026-03-15

---

## Epic

- **タイトル**: Hooks 自動変換機能 (Claude Code -> Copilot CLI)
- **ラベル**: `enhancement`
- **説明**: PLM の `install` コマンド実行時に、Claude Code 形式の Hooks 設定ファイルを Copilot CLI 形式に自動変換して配置する機能。プラグイン作者は Claude Code 形式で Hooks を記述するだけで、Copilot CLI ユーザーにもそのまま配布できるようになる。設定 JSON 構造の変換（イベント名・キー名・フック種別）、stdin/stdout スキーマ差分を吸収するラッパースクリプトの生成、ツール名・exit code のブリッジを含む。

---

## Issue 一覧

### Issue 1: hooks モジュールの新規作成とイベント名・ツール名マッピングテーブルの定義

- **ラベル**: `enhancement`
- **優先度**: P1
- **説明**:
  `src/hooks.rs` + `src/hooks/` モジュールを新規作成し、イベント名マッピング（PascalCase -> camelCase）とツール名マッピング（Copilot CLI -> Claude Code）の定数テーブルを `src/hooks/event_map.rs` に定義する。全ての後続 Issue の基盤となるモジュール。
- **blocked_by**: なし
- **対象ファイル**:
  - `src/hooks.rs` (新規)
  - `src/hooks/event_map.rs` (新規)
  - `src/hooks/event_map_test.rs` (新規)
- **受入基準**:
  - `src/hooks.rs` が `mod event_map;` を公開していること
  - イベント名マッピングテーブルが config-converter-spec BL-003 の全エントリを網羅していること（対応7イベント + 除外12イベント）
  - ツール名マッピングテーブルが script-wrapper-spec BL-002 の全エントリを網羅していること（11ツール）
  - マッピング対象外のイベント名を渡した場合に `None` を返すこと
  - 未知のツール名はそのまま返すこと（フォワードコンパティビリティ）
  - `cargo check` / `cargo test` がパスすること

#### Sub-issue 1-1: hooks モジュールの骨格作成

- **説明**: `src/hooks.rs` を作成し、`mod event_map;` を宣言する。`src/main.rs` または `src/lib.rs` に `mod hooks;` を追加する。
- **対象ファイル**: `src/hooks.rs` (新規), `src/main.rs` (変更)

#### Sub-issue 1-2: イベント名マッピングテーブルの実装

- **説明**: `src/hooks/event_map.rs` に Claude Code -> Copilot CLI のイベント名変換テーブルを実装する。`SessionStart` -> `sessionStart`, `UserPromptSubmit` -> `userPromptSubmitted`, `Stop` -> `agentStop` 等の不規則変換を含む。非対応イベント（`PostToolUseFailure`, `PreCompact`, `Notification` 等12件）を識別する関数も実装する。
- **対象ファイル**: `src/hooks/event_map.rs` (新規)

#### Sub-issue 1-3: ツール名マッピングテーブルの実装

- **説明**: `src/hooks/event_map.rs` に Copilot CLI -> Claude Code のツール名変換テーブルを実装する。`bash` -> `Bash`, `view` -> `Read`, `create` -> `Write`, `task` -> `Agent` 等のマッピング。未知のツール名はそのまま返す。
- **対象ファイル**: `src/hooks/event_map.rs` (新規)

#### Sub-issue 1-4: マッピングテーブルのユニットテスト

- **説明**: イベント名・ツール名マッピングの全パターンをカバーするユニットテストを実装する。対応イベント、除外イベント、未知イベント、全ツール名、未知ツール名のテストを含む。
- **対象ファイル**: `src/hooks/event_map_test.rs` (新規)

---

### Issue 2: 設定構造変換ロジック (HookConfigConverter) の実装

- **ラベル**: `enhancement`
- **優先度**: P1
- **説明**:
  `src/hooks/converter.rs` に Claude Code 形式の Hooks 設定 JSON を Copilot CLI 形式に変換するコアロジックを実装する。ソース形式の自動判定（BL-001）、トップレベル構造変換（BL-002）、イベント名変換（BL-003）、matcher グループのフラット化（BL-004）、キー名変換（BL-005）、フック種別変換（BL-006）を含む。変換結果とともに警告リストを返す。
- **blocked_by**: [Issue 1]
- **対象ファイル**:
  - `src/hooks/converter.rs` (新規)
  - `src/hooks/converter_test.rs` (新規)
  - `src/hooks.rs` (変更: `mod converter;` 追加)
- **受入基準**:
  - ソース形式の自動判定が正しく動作すること（`version` キー有無、PascalCase/camelCase 判定）
  - Copilot CLI 形式の入力はそのまま返すこと（変換不要）
  - `"version": 1` が出力 JSON に追加されること
  - イベント名が正しく変換されること（7件の対応イベント）
  - 非対応イベント（12件）が除外され、警告リストに追加されること
  - matcher グループがフラット配列に展開されること
  - `command` -> `bash`, `timeout` -> `timeoutSec`, `statusMessage` -> `comment` のキー名変換が正しいこと
  - `async`, `once`, `disableAllHooks` が除去され、警告が出力されること
  - `http` フック -> `command` フック + ラッパースクリプト情報への変換が正しいこと
  - `prompt` / `agent` フック -> スタブ + 警告への変換が正しいこと
  - JSON パースエラー時にエラーを返すこと
  - `cargo check` / `cargo test` がパスすること

#### Sub-issue 2-1: ソース形式の自動判定ロジック (BL-001)

- **説明**: 入力 JSON が Claude Code 形式か Copilot CLI 形式かを判定する関数を実装する。`version` キーの有無、イベントキーの PascalCase/camelCase を判定基準とする。
- **対象ファイル**: `src/hooks/converter.rs`

#### Sub-issue 2-2: トップレベル構造変換とイベント名変換 (BL-002, BL-003)

- **説明**: `"version": 1` の追加、`disableAllHooks` の除去（警告付き）、イベントキーの PascalCase -> camelCase 変換を実装する。非対応イベントの除外と警告リストへの追加を含む。
- **対象ファイル**: `src/hooks/converter.rs`

#### Sub-issue 2-3: matcher グループのフラット化 (BL-004)

- **説明**: Claude Code の `matcher` + `hooks[]` ネスト構造を Copilot CLI のフラット配列に展開するロジックを実装する。matcher 情報はラッパースクリプト生成用に保持して返す。
- **対象ファイル**: `src/hooks/converter.rs`

#### Sub-issue 2-4: キー名変換とフック種別変換 (BL-005, BL-006)

- **説明**: `command` -> `bash`, `timeout` -> `timeoutSec` 等のキー名変換を実装する。`http` フックの `command` フック + ラッパースクリプト情報への変換、`prompt`/`agent` フックのスタブ化 + 警告を実装する。`async`/`once` の除去を含む。
- **対象ファイル**: `src/hooks/converter.rs`

#### Sub-issue 2-5: 設定構造変換のユニットテスト

- **説明**: BL-001 ~ BL-006 の全パターンをカバーするテストを実装する。正常系（Copilot CLI 形式スルー、Claude Code 形式の完全変換）、エッジケース（空 matcher、全非対応イベント）、エラー系（JSON パースエラー、必須フィールド欠落）を含む。
- **対象ファイル**: `src/hooks/converter_test.rs` (新規)

---

### Issue 3: ラッパースクリプト生成ロジック (WrapperGenerator) の実装

- **ラベル**: `enhancement`
- **優先度**: P1
- **説明**:
  `src/hooks/wrapper.rs` に Copilot CLI 環境で Claude Code 形式のフックスクリプトを動作させるためのラッパースクリプト生成ロジックを実装する。stdin 変換（Copilot CLI -> Claude Code）、matcher フィルタリング、stdout 逆変換（Claude Code -> Copilot CLI）、exit code 変換、環境変数ブリッジを含むシェルスクリプトをプログラム的に生成する。
- **blocked_by**: [Issue 1]
- **対象ファイル**:
  - `src/hooks/wrapper.rs` (新規)
  - `src/hooks/wrapper_test.rs` (新規)
  - `src/hooks.rs` (変更: `mod wrapper;` 追加)
- **受入基準**:
  - `command` フックに対して、stdin 変換 + matcher フィルタ + 元スクリプト実行 + stdout/exit code 変換を含むラッパースクリプトが生成されること
  - stdin 変換で `toolName` -> `tool_name`（PascalCase）、`toolArgs`（JSON 文字列） -> `tool_input`（オブジェクト）への変換が正しいこと
  - ツール名マッピング（`bash` -> `Bash`, `view` -> `Read` 等）が jq 式に正しく埋め込まれること
  - matcher が正規表現としてラッパースクリプトに埋め込まれること
  - matcher が空/未設定の場合はフィルタなしになること
  - exit code 0 -> stdout 変換、exit code 2 -> deny JSON 生成、exit code 1/その他 -> 出力なしの変換が正しいこと
  - `CLAUDE_PROJECT_DIR` と `CLAUDE_PLUGIN_ROOT` の環境変数ブリッジが含まれること
  - `@@PLUGIN_ROOT@@` プレースホルダーが含まれること
  - `http` フックに対して curl ラッパースクリプトが生成されること（URL、headers の埋め込み）
  - `prompt`/`agent` フックに対してスタブスクリプト（常に exit 0）が生成されること（元設定をコメントで記録）
  - イベント種別ごとの stdin 変換が正しいこと（PreToolUse/PostToolUse, SessionStart, UserPromptSubmit）
  - `cargo check` / `cargo test` がパスすること

#### Sub-issue 3-1: command フックのラッパースクリプト生成

- **説明**: `command` フック用のラッパースクリプトテンプレートを実装する。stdin 変換（BL-001: `timestamp` 除去、`session_id` 追加、`toolName` -> `tool_name`、`toolArgs` -> `tool_input`）、matcher フィルタ（BL-003）、環境変数ブリッジ（BL-006: `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`）、元スクリプト実行、stdout 変換（BL-004: `hookSpecificOutput` アンラップ）、exit code 変換（BL-005: exit 2 -> deny JSON）を含む。
- **対象ファイル**: `src/hooks/wrapper.rs`

#### Sub-issue 3-2: イベント種別ごとの stdin 変換テンプレート

- **説明**: PreToolUse/PostToolUse、SessionStart（`source` 値マッピング: `new` -> `startup`）、UserPromptSubmit 等のイベント種別に応じた stdin 変換パターンを実装する。PostToolUse の `toolResult` -> `tool_response` 変換を含む。
- **対象ファイル**: `src/hooks/wrapper.rs`

#### Sub-issue 3-3: http フックの curl ラッパースクリプト生成

- **説明**: `http` フック用のラッパースクリプトを生成する。`url`, `headers` を curl コマンドに埋め込み、HTTP レスポンスの status code チェックと stdout 変換を行う。
- **対象ファイル**: `src/hooks/wrapper.rs`

#### Sub-issue 3-4: prompt/agent フックのスタブスクリプト生成

- **説明**: `prompt`/`agent` フック用のスタブスクリプトを生成する。常に exit 0（許可）を返し、コメントで元の設定を記録する。
- **対象ファイル**: `src/hooks/wrapper.rs`

#### Sub-issue 3-5: ラッパースクリプト生成のユニットテスト

- **説明**: 各フック種別のラッパースクリプト生成の正当性をテストする。生成されたスクリプトに必要な要素（jq 式、matcher フィルタ、exit code 変換、環境変数、プレースホルダー）が含まれていることを検証する。
- **対象ファイル**: `src/hooks/wrapper_test.rs` (新規)

---

### Issue 4: install フローとの統合 - デプロイメント拡張

- **ラベル**: `enhancement`
- **優先度**: P1
- **説明**:
  `src/component/deployment.rs` の Hook デプロイ処理を拡張し、Copilot ターゲットへの Hook インストール時に Claude Code 形式の場合は自動変換を実行するようにする。既存の Copilot CLI 形式のファイルコピー動作には影響しない。`DeploymentResult` enum への `Converted` バリアント追加、変換フロー分岐、ファイル配置（`hooks.json` + `wrappers/`）、`@@PLUGIN_ROOT@@` プレースホルダー解決を含む。
- **blocked_by**: [Issue 2, Issue 3]
- **対象ファイル**:
  - `src/component/deployment.rs` (変更)
  - `src/component/deployment_test.rs` (変更)
- **受入基準**:
  - Copilot ターゲット + Hook + Claude Code 形式の場合に自動変換が発動すること（install-integration-spec BL-001）
  - Copilot ターゲット + Hook + Copilot CLI 形式の場合は既存のファイルコピーが動作すること
  - Copilot 以外のターゲットでは変換が発動しないこと
  - `DeploymentResult::Converted` が追加されていること（BL-005）
  - 変換済み `hooks.json` が正しいパスに配置されること（BL-003: Project/Personal スコープ）
  - ラッパースクリプトが `wrappers/` サブディレクトリに配置され、実行権限が付与されること
  - `@@PLUGIN_ROOT@@` がプラグインキャッシュの実パスに置換されること（BL-004）
  - 元スクリプトのパスが正しく解決されること（BL-006: 相対パス、絶対パス、コマンド名）
  - 非対応イベントの警告が出力に含まれること
  - 全イベントが非対応の場合に空の `hooks.json` が配置され、警告が出力されること
  - ソース JSON 読み込み失敗時にエラーが返されること
  - `cargo check` / `cargo test` がパスすること

#### Sub-issue 4-1: DeploymentResult enum への Converted バリアント追加

- **説明**: `DeploymentResult` に `Converted` バリアントを追加する。変換サマリー（変換されたイベント数、除外されたイベント名リスト）を保持するフィールドを含む。
- **対象ファイル**: `src/component/deployment.rs`

#### Sub-issue 4-2: ソース形式判定と変換フロー分岐

- **説明**: Hook デプロイ処理にソース形式判定を追加し、Claude Code 形式の場合は `HookConfigConverter` と `WrapperGenerator` を呼び出す分岐を実装する。Copilot CLI 形式の場合は既存のファイルコピーを継続する。
- **対象ファイル**: `src/component/deployment.rs`

#### Sub-issue 4-3: 変換済みファイルの配置ロジック

- **説明**: 変換済み `hooks.json` と `wrappers/*.sh` の配置ロジックを実装する。`wrappers/` ディレクトリの作成、ラッパースクリプトの書き出し、`chmod +x` による実行権限付与、`@@PLUGIN_ROOT@@` プレースホルダーの実パスへの置換を含む。
- **対象ファイル**: `src/component/deployment.rs`

#### Sub-issue 4-4: 元スクリプトのパス解決

- **説明**: ラッパースクリプトから元のフックスクリプトを呼び出すためのパス解決を実装する。相対パスはプラグインキャッシュルートからの解決、絶対パスはそのまま、コマンド名のみはそのまま使用する（install-integration-spec BL-006）。
- **対象ファイル**: `src/component/deployment.rs`

#### Sub-issue 4-5: 変換サマリーと警告の出力

- **説明**: `DeploymentResult::Converted` の場合にコマンド出力に変換サマリーを表示する。変換されたイベント一覧、除外されたイベント（非対応イベント）、`prompt`/`agent` フックの手動書き換え案内等の警告を含む。
- **対象ファイル**: `src/component/deployment.rs`

#### Sub-issue 4-6: デプロイメント拡張の統合テスト

- **説明**: 変換デプロイの統合テストを実装する。Claude Code 形式のインストール、Copilot CLI 形式のスルー、非 Copilot ターゲットでの非発動、エラーケース（JSON パースエラー、ファイル書き込み失敗）、全イベント非対応のケースをカバーする。
- **対象ファイル**: `src/component/deployment_test.rs` (変更)

---

### Issue 5: コマンド出力の拡張 - 変換結果の表示

- **ラベル**: `enhancement`
- **優先度**: P2
- **説明**:
  `plm install` コマンドの出力を拡張し、Hook 変換が行われた場合に変換サマリーと警告を表示する。`DeploymentResult::Converted` に基づく出力フォーマッティングを実装する。
- **blocked_by**: [Issue 4]
- **対象ファイル**:
  - `src/commands/install.rs` (変更)
- **受入基準**:
  - `DeploymentResult::Converted` の場合に「converted from Claude Code format」と表示されること
  - 除外されたイベントの警告が表示されること（例: `3 events skipped (not supported in Copilot CLI): Notification, PreCompact, SubagentStart`）
  - `prompt`/`agent` フックの手動書き換え案内が表示されること
  - 既存の `DeploymentResult::Copied` の出力に変更がないこと
  - `cargo check` / `cargo test` がパスすること

---

### Issue 6: エラーハンドリングと警告システムの整備

- **ラベル**: `enhancement`
- **優先度**: P2
- **説明**:
  Hooks 変換処理全体のエラーハンドリングと警告システムを整備する。config-converter-spec と install-integration-spec に定義されたエラーケースへの対応を一貫した形で実装する。警告の種別（除外イベント、除去フィールド、非対応フック種別）を構造化する。
- **blocked_by**: [Issue 2]
- **対象ファイル**:
  - `src/hooks/converter.rs` (変更)
  - `src/hooks/wrapper.rs` (変更)
  - `src/hooks.rs` (変更: 警告型の定義追加)
- **受入基準**:
  - 警告種別が enum で定義されていること（`UnsupportedEvent`, `RemovedField`, `NonConvertibleHookType`, `StubGenerated` 等）
  - JSON パースエラー時にわかりやすいエラーメッセージが返されること
  - `command` フィールド欠落（`type: command`）時にエラーが返されること
  - `url` フィールド欠落（`type: http`）時にエラーが返されること
  - 未知のイベント名は警告付きで除外されること
  - 未知のフック種別は警告付きで除外されること
  - `cargo check` / `cargo test` がパスすること

#### Sub-issue 6-1: 警告型 (ConversionWarning) の定義

- **説明**: `src/hooks.rs` に `ConversionWarning` enum を定義する。`UnsupportedEvent(String)`, `RemovedField(String)`, `NonConvertibleHookType(String)`, `StubGenerated(String)`, `UnknownEvent(String)`, `UnknownHookType(String)` バリアントを含む。
- **対象ファイル**: `src/hooks.rs`

#### Sub-issue 6-2: converter.rs へのエラー・警告統合

- **説明**: `converter.rs` の各処理でエラーと警告を適切に生成・収集するように改修する。変換結果を `Result<(ConvertedConfig, Vec<ConversionWarning>), ConversionError>` 型で返す。
- **対象ファイル**: `src/hooks/converter.rs`

#### Sub-issue 6-3: エラーハンドリングのテスト

- **説明**: 各エラーケース・警告ケースのテストを追加する。
- **対象ファイル**: `src/hooks/converter_test.rs` (変更), `src/hooks/wrapper_test.rs` (変更)

---

## 依存関係グラフ

```
Issue 1 (event_map) ──┬──> Issue 2 (converter) ──┬──> Issue 4 (deployment) ──> Issue 5 (コマンド出力)
                      │                          │
                      ├──> Issue 3 (wrapper) ─────┘
                      │
                      └──────────────────────────────> Issue 6 (エラーハンドリング)
                                                         ↑
                                                  Issue 2 (converter) ───┘
```

## 実装順序（推奨）

1. **Phase 1**: Issue 1 (hooks モジュール骨格 + マッピングテーブル)
2. **Phase 2（並行可能）**: Issue 2 (設定構造変換), Issue 3 (ラッパースクリプト生成)
3. **Phase 3**: Issue 4 (install フロー統合), Issue 6 (エラーハンドリング整備)
4. **Phase 4**: Issue 5 (コマンド出力拡張)

---

## Issue サマリー

| # | タイトル | 優先度 | blocked_by | Sub-issue 数 |
|:--|:--------|:-------|:-----------|:-------------|
| 1 | hooks モジュールの新規作成とイベント名・ツール名マッピングテーブルの定義 | P1 | なし | 4 |
| 2 | 設定構造変換ロジック (HookConfigConverter) の実装 | P1 | 1 | 5 |
| 3 | ラッパースクリプト生成ロジック (WrapperGenerator) の実装 | P1 | 1 | 5 |
| 4 | install フローとの統合 - デプロイメント拡張 | P1 | 2, 3 | 6 |
| 5 | コマンド出力の拡張 - 変換結果の表示 | P2 | 4 | 0 |
| 6 | エラーハンドリングと警告システムの整備 | P2 | 2 | 3 |

**合計**: Epic 1件、Issue 6件、Sub-issue 23件

---

## ビジネスロジック対応表

| BL ID | 仕様書 | ビジネスロジック | 対応 Issue |
|:------|:-------|:----------------|:-----------|
| BL-001 | config-converter-spec | ソース形式の自動判定 | Issue 2 (Sub-issue 2-1) |
| BL-002 | config-converter-spec | トップレベル構造の変換 | Issue 2 (Sub-issue 2-2) |
| BL-003 | config-converter-spec | イベント名の変換 | Issue 1 (Sub-issue 1-2), Issue 2 (Sub-issue 2-2) |
| BL-004 | config-converter-spec | matcher グループのフラット化 | Issue 2 (Sub-issue 2-3) |
| BL-005 | config-converter-spec | フック定義のキー名変換 | Issue 2 (Sub-issue 2-4) |
| BL-006 | config-converter-spec | フック種別の変換 | Issue 2 (Sub-issue 2-4) |
| BL-001 | script-wrapper-spec | stdin 変換 (Copilot CLI -> Claude Code) | Issue 3 (Sub-issue 3-1, 3-2) |
| BL-002 | script-wrapper-spec | ツール名変換 | Issue 1 (Sub-issue 1-3), Issue 3 (Sub-issue 3-1) |
| BL-003 | script-wrapper-spec | matcher フィルタリング | Issue 3 (Sub-issue 3-1) |
| BL-004 | script-wrapper-spec | stdout 変換 (Claude Code -> Copilot CLI) | Issue 3 (Sub-issue 3-1) |
| BL-005 | script-wrapper-spec | exit code 変換 | Issue 3 (Sub-issue 3-1) |
| BL-006 | script-wrapper-spec | 環境変数のブリッジ | Issue 3 (Sub-issue 3-1) |
| BL-001 | install-integration-spec | 変換の発動条件 | Issue 4 (Sub-issue 4-2) |
| BL-002 | install-integration-spec | デプロイメントフローの拡張 | Issue 4 (Sub-issue 4-2, 4-3) |
| BL-003 | install-integration-spec | 配置先ディレクトリ構造 | Issue 4 (Sub-issue 4-3) |
| BL-004 | install-integration-spec | @@PLUGIN_ROOT@@ プレースホルダーの解決 | Issue 4 (Sub-issue 4-3) |
| BL-005 | install-integration-spec | DeploymentResult の拡張 | Issue 4 (Sub-issue 4-1) |
| BL-006 | install-integration-spec | 元スクリプトへのパス解決 | Issue 4 (Sub-issue 4-4) |

---

## モジュール構成（完成時）

```
src/
├── hooks.rs                      # [新規] mod hooks の定義 + ConversionWarning 型
├── hooks/
│   ├── event_map.rs              # [新規] イベント名・ツール名マッピングテーブル
│   ├── event_map_test.rs         # [新規] マッピングテーブルのテスト
│   ├── converter.rs              # [新規] 設定構造変換ロジック
│   ├── converter_test.rs         # [新規] 設定構造変換のテスト
│   ├── wrapper.rs                # [新規] ラッパースクリプト生成ロジック
│   └── wrapper_test.rs           # [新規] ラッパースクリプト生成のテスト
├── component/
│   ├── deployment.rs             # [変更] Hook デプロイに変換分岐を追加
│   └── deployment_test.rs        # [変更] 変換デプロイのテスト追加
├── commands/
│   └── install.rs                # [変更] 変換結果の出力表示
└── target/
    └── copilot.rs                # [変更なし] 既存の placement_location を使用
```
