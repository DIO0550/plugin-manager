# Question Patterns

AskUserQuestion形式での質問パターン集。

## Batch 1: プロジェクトスコープ

```yaml
questions:
  - question: "このタスクの主な目的は何ですか？"
    header: "目的"
    multiSelect: false
    options:
      - label: "新機能追加"
        description: "既存にない機能を新たに実装"
      - label: "既存機能の改善"
        description: "動作する機能の拡張・修正"
      - label: "バグ修正"
        description: "不具合の修正"
      - label: "リファクタリング"
        description: "機能変更なしのコード改善"

  - question: "影響範囲はどの程度ですか？"
    header: "スコープ"
    multiSelect: false
    options:
      - label: "単一ファイル"
        description: "1ファイルのみの変更"
      - label: "単一コンポーネント"
        description: "1コンポーネント内で完結"
      - label: "複数コンポーネント"
        description: "複数箇所に影響"
      - label: "アーキテクチャ変更"
        description: "全体構造に影響"
```

## Batch 2: 技術的詳細

```yaml
questions:
  - question: "使用する技術・フレームワークは？"
    header: "技術"
    multiSelect: true
    options:
      - label: "React/Next.js"
        description: "フロントエンドフレームワーク"
      - label: "TypeScript"
        description: "型付きJavaScript"
      - label: "Tailwind CSS"
        description: "ユーティリティファーストCSS"
      - label: "その他"
        description: "別の技術を使用"

  - question: "データの永続化は必要ですか？"
    header: "データ"
    multiSelect: false
    options:
      - label: "不要"
        description: "メモリ内のみ"
      - label: "ローカルストレージ"
        description: "ブラウザに保存"
      - label: "データベース"
        description: "サーバーサイドDB"
      - label: "外部API"
        description: "外部サービス連携"

  - question: "既存のコードとの連携は？"
    header: "連携"
    multiSelect: true
    options:
      - label: "独立（新規）"
        description: "既存コードに依存しない"
      - label: "既存コンポーネント使用"
        description: "既存UIを再利用"
      - label: "共通ロジック使用"
        description: "既存utilsを使用"
      - label: "API連携"
        description: "既存APIを呼び出す"
```

## Batch 3: 品質要件

```yaml
questions:
  - question: "この機能に含まれる要素は？（テスト戦略の判断に使用します）"
    header: "機能特性"
    multiSelect: true
    options:
      - label: "計算・変換・バリデーションロジック"
        description: "純粋なロジック処理がある"
      - label: "外部API・サービス連携"
        description: "HTTP呼び出し・外部依存がある"
      - label: "状態管理（作成・更新・削除）"
        description: "アプリケーション状態を変更する"
      - label: "ユーザー操作・UI表示のみ"
        description: "ロジックを含まない見た目の変更"
      - label: "認証・認可・セキュリティ"
        description: "セキュリティに関わる処理"
      - label: "非同期処理・タイマー"
        description: "Promise, setTimeout, イベント"

  - question: "テスト方針の希望は？"
    header: "テスト方針"
    multiSelect: false
    options:
      - label: "AIに任せる（推奨）"
        description: "機能特性に基づいてAIが最適なテスト戦略を決定"
      - label: "TDDで進めたい"
        description: "テスト駆動開発で実装したい"
      - label: "実装後にテスト追加"
        description: "先に実装してからテストを書く"
      - label: "手動確認のみ"
        description: "自動テストは不要"

  - question: "エラーハンドリングの要件は？"
    header: "エラー"
    multiSelect: false
    options:
      - label: "基本的（try-catch）"
        description: "最低限のエラー処理"
      - label: "ユーザーフィードバック"
        description: "エラー時にUI表示"
      - label: "リトライ・復旧"
        description: "自動リトライ機能"
      - label: "詳細ログ"
        description: "デバッグ用ログ出力"

  - question: "パフォーマンス要件は？"
    header: "性能"
    multiSelect: false
    options:
      - label: "特になし"
        description: "動けばOK"
      - label: "基本的な最適化"
        description: "明らかな無駄を避ける"
      - label: "高パフォーマンス"
        description: "レスポンス重視"
      - label: "大規模データ対応"
        description: "大量データ処理"
```

## 追加質問（必要に応じて）

### UI/UX関連

```yaml
questions:
  - question: "UIスタイリングのアプローチは？"
    header: "スタイル"
    multiSelect: false
    options:
      - label: "Tailwind CSS"
        description: "ユーティリティクラス"
      - label: "CSS Modules"
        description: "スコープ付きCSS"
      - label: "既存デザインシステム"
        description: "プロジェクトの共通UI"
      - label: "シンプル（最小限）"
        description: "機能優先"
```

### API/バックエンド関連

```yaml
questions:
  - question: "APIの形式は？"
    header: "API"
    multiSelect: false
    options:
      - label: "REST"
        description: "RESTful API"
      - label: "GraphQL"
        description: "GraphQLクエリ"
      - label: "WebSocket"
        description: "リアルタイム通信"
      - label: "不要"
        description: "API呼び出しなし"
```

## 質問のベストプラクティス

1. **一度に3-4問まで** - ユーザーを圧倒しない
2. **推奨オプションを先頭に** - `(Recommended)` を付ける
3. **Otherで自由入力可能** - 想定外の回答に対応
4. **曖昧な回答は深掘り** - 追加質問で明確化
