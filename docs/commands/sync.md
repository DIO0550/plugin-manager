# plm sync

環境間でコンポーネントを同期します。

## 基本構文

```bash
plm sync --from <source-target> --to <dest-target> [options]
```

## オプション

| オプション | 説明 | 必須 |
|------------|------|------|
| `--from` | 同期元のターゲット環境 | ✅ |
| `--to` | 同期先のターゲット環境 | ✅ |
| `--type` | コンポーネント種別でフィルタ | - |

## 使用例

### 全コンポーネントを同期

```bash
$ plm sync --from codex --to copilot
🔄 Syncing from codex to copilot...
   ✓ html-educational-material (already synced)
   + frontend-design (installing...)
   + code-formatter (installing...)
✅ Synced 2 components to copilot
```

### 特定種別のみ同期

```bash
$ plm sync --from codex --to copilot --type skill
🔄 Syncing skills from codex to copilot...
   ✓ html-educational-material (already synced)
   + frontend-design (installing...)
✅ Synced 1 skill to copilot
```

## 動作詳細

1. 同期元ターゲットのコンポーネント一覧を取得
2. 同期先ターゲットのコンポーネント一覧と比較
3. 差分のあるコンポーネントを同期先にインストール
4. 既存のコンポーネントはスキップ

## 制約事項

- 同期先ターゲットがサポートしていないコンポーネント種別はスキップされます
- 例: CodexからCopilotへのPromptsの同期は、Codexがプロンプトをサポートしていないためスキップ

## 関連

- [concepts/targets](../concepts/targets.md) - ターゲット環境のサポート状況
- [concepts/components](../concepts/components.md) - コンポーネント種別
