# Spec 032: デフォルト起動を managed に変更

## 概要

`plm` をサブコマンドなし・引数なし（または global フラグ `--verbose` のみ）で起動した場合に、`plm managed` と同じ TUI 管理画面を自動的に起動するようにした。初回ユーザーが `plm managed` をタイプしなくても TUI に到達できるようにすることが目的。

## 破壊的変更ノート

### 旧挙動

`plm`（引数なし）は clap によりサブコマンド必須エラーで `exit 2` していた。

```
$ plm
error: 'plm' requires a subcommand but one was not provided
$ echo $?
2
```

### 新挙動

`plm`（引数なし）は TTY 判定により挙動が分岐する。

- **TTY あり**: managed TUI を起動して `exit 0`（または TUI の終了コード）
- **非TTY**（パイプ・リダイレクト経由）: `--help` を stdout に出力して `exit 0`

```
$ plm                          # TTY → TUI 起動
$ plm --verbose                # TTY → TUI 起動（--verbose も同様）
$ plm | cat                    # 非TTY → ヘルプを stdout に出力して exit 0
$ plm > /tmp/out.txt           # 非TTY → ヘルプを /tmp/out.txt に出力して exit 0
```

### 影響範囲

- スクリプトや CI で `plm`（引数なし）の `exit 2` 動作に依存していた場合は、エラーで終了しなくなる。引数なし実行を「エラー検出」として扱っているスクリプトは要更新。
- `plm managed`（明示呼び出し）の挙動は **変更なし**（後方互換）。非TTY 経由で `plm managed | cat` を呼んだ場合も従来通り TUI 起動を試みる。
- `plm --help` / `plm -h` の挙動は **変更なし**（clap 既定動作）。
- `plm --version` は `#[command(version)]` が未付与のため、現状の `UnknownArgument`（exit 2）を維持。本仕様では対応外。
- 未知サブコマンド・未知フラグの場合は従来通り clap エラーで `exit 2`。

### 回避手段

`plm`（引数なし）でエラー扱いとしたい場合は、明示的にサブコマンドを指定するか、`plm --help` を呼んで終了コードと出力で判定する。

## 実装メモ

- `Cli.command` を `Option<Command>` 化し、`#[command(subcommand_required = false, arg_required_else_help = false)]` を付与。
- `commands::dispatch` の `None` 分岐から、純粋関数 `decide_default_action(stdout_is_tty: bool) -> DefaultAction` の判定結果に基づき `managed::run()` か `Cli::command().print_help()` を呼ぶ。
- 非TTY 判定は `std::io::stdout().is_terminal()`。ratatui が stdout に raw mode を張るため、stdout 基準が最も妥当。
- `Some(Command::Managed)` の経路は `run_default` を経由せず `managed::run()` を直接呼び、明示呼び出しの後方互換を維持。

## テスト方針

- `Cli::try_parse_from` ベースのユニットテストで parse 層の各分岐を網羅。
- `decide_default_action(true/false)` の純粋関数テストで TTY 分岐ロジックを検証。
- TUI 起動自体の実行系テストは行わない（既存方針と同じ）。
