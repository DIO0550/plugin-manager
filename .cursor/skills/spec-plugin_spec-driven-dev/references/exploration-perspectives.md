# Exploration Perspectives（探索の5観点）

各カテゴリで何を調べるべきかの具体的チェックリストと推奨検索パターン。

---

## 1. アーキテクチャ概要

### チェック項目

- [ ] ディレクトリ構造: `src/`, `lib/`, `components/` 等の配置
- [ ] 設定ファイル: package.json, tsconfig.json, Cargo.toml, pyproject.toml
- [ ] README / ARCHITECTURE: プロジェクト概要、アーキテクチャドキュメント
- [ ] レイヤー分割: Presentation, Application, Infrastructure の分離
- [ ] モジュール構成: フィーチャー分割 vs レイヤー分割
- [ ] 依存関係: import/export の流れ、循環依存の有無
- [ ] 外部ライブラリ: 主要な依存とバージョン

### 推奨パターン

```yaml
Glob:
  - "src/**/*.{ts,tsx,rs,py}"   # ソースファイル全体像
  - "**/package.json"            # npm パッケージ
  - "**/Cargo.toml"              # Rust クレート
  - "**/tsconfig*.json"          # TypeScript 設定

Grep:
  - pattern: "^import.*from"
    output_mode: files_with_matches
    type: ts
  - pattern: "^export.*(function|const|class|interface|type)"
    output_mode: files_with_matches
    type: ts
  - pattern: "^use "
    output_mode: files_with_matches
    type: rust
```

---

## 2. 関連コード分析

### チェック項目

- [ ] 類似コンポーネント: 同じパターンで実装されている機能
- [ ] 共通ユーティリティ: utils/, helpers/, lib/ 配下の再利用可能コード
- [ ] デザインパターン: Factory, Builder, Strategy 等の使用例
- [ ] 状態管理: Redux, Zustand, Context API 等の実装パターン
- [ ] API 呼び出し: fetch, axios, GraphQL client の使い方
- [ ] エラーハンドリング: try-catch, Result 型のパターン
- [ ] 命名規則: ファイル名、変数名、関数名の一貫性
- [ ] 既存構造の問題点: 過剰な結合、責務の混在、重複、テスト困難な構造、技術的負債（踏襲すべきでないパターンを「避ける / 要判断」として報告する）

### 推奨パターン

```yaml
Grep:
  # 類似機能の検索（{keyword} を対象機能のキーワードに置換）
  - pattern: "{keyword}"
    output_mode: content
    context: 3

  # ユーティリティ関数
  - pattern: "export (function|const)"
    output_mode: content
    context: 3
    glob: "**/utils/**"

  # 型定義
  - pattern: "^(export )?(interface|type) "
    output_mode: content
    context: 5
    type: ts
```

### 最低探索量

- Grep ヒットした上位 **5 ファイル**を Read する
- 類似実装ファイルを最低 **2 ファイル**全文 Read し、パターンを理解する
- ユーティリティ関数ファイルを最低 **1 ファイル** Read する

---

## 3. 技術的制約・リスク

### チェック項目

- [ ] 型システム制約: strict モード、型定義の厳格度
- [ ] リンター設定: ESLint, Clippy, Pylint のルール
- [ ] フォーマッター: Prettier, rustfmt, Black の設定
- [ ] ビルド設定: Webpack, Vite, cargo build の最適化
- [ ] 環境変数: .env ファイル、環境依存の設定
- [ ] ライブラリバージョン: deprecated API、breaking changes
- [ ] パフォーマンス: 既知のボトルネック、大量データ処理
- [ ] セキュリティ: 認証、認可、入力検証

### 推奨パターン

```yaml
Read:
  - tsconfig.json        # TypeScript 設定
  - .eslintrc*            # ESLint 設定
  - .prettierrc*          # Prettier 設定
  - Cargo.toml            # Rust 依存・設定
  - .env.example          # 環境変数テンプレート

Grep:
  - pattern: "@deprecated"
    output_mode: content
    context: 3
  - pattern: "TODO|FIXME|HACK|XXX"
    output_mode: content
    context: 2
  - pattern: "process\\.env|std::env"
    output_mode: files_with_matches
```

---

## 4. 変更影響範囲

### チェック項目

- [ ] 依存の逆引き: 変更対象を import しているファイル
- [ ] テストファイル: 関連するユニットテスト、統合テスト
- [ ] Public API: export されている関数・クラス・型
- [ ] CI/CD 設定: .github/workflows/, .gitlab-ci.yml
- [ ] ドキュメント: README, CHANGELOG への影響

### 推奨パターン

```yaml
Glob:
  - "**/*.test.{ts,tsx,js,jsx}"  # Jest / Vitest テスト
  - "**/*.spec.{ts,tsx,rs}"      # spec テスト
  - "**/__tests__/**"             # テストディレクトリ
  - ".github/workflows/*.yml"    # CI 設定

Grep:
  # 逆引き検索（{target} を変更対象ファイル名に置換）
  - pattern: "import.*from.*{target}"
    output_mode: files_with_matches
  - pattern: "require\\(.*{target}"
    output_mode: files_with_matches

  # Public API
  - pattern: "^export (function|const|class|interface|type)"
    output_mode: content
    context: 3
```

### 最低探索量

- 変更対象を import しているファイルを最低 **3 ファイル** Read する
- 関連テストファイルを最低 **2 ファイル** Read する
- CI 設定を **1 ファイル** Read する（存在する場合）

---

## 5. テストインフラストラクチャ

### チェック項目

- [ ] テストフレームワーク: Jest, Vitest, pytest, cargo test 等
- [ ] テストランナー: テスト実行コマンドとその設定
- [ ] アサーションライブラリ: expect, assert, chai 等
- [ ] モック戦略: jest.mock, vi.mock, sinon, nock 等
- [ ] テストファイル構造: colocated (*.test.ts隣接) / __tests__/ / tests/
- [ ] テストファイル命名規則: *.test.ts / *.spec.ts / test_*.py
- [ ] テストヘルパー・フィクスチャ: 共通セットアップ、テストユーティリティ
- [ ] カバレッジ設定: カバレッジツール、閾値設定
- [ ] CI でのテスト実行: テスト関連の CI ジョブ

### 推奨パターン

```yaml
Glob:
  - "**/*.test.{ts,tsx,js,jsx}"  # Jest / Vitest テスト
  - "**/*.spec.{ts,tsx,rs}"      # spec テスト
  - "**/test_*.py"               # pytest テスト
  - "**/__tests__/**"             # テストディレクトリ
  - "**/tests/**"                 # テストディレクトリ
  - "**/*.stories.{ts,tsx}"      # Storybook
  - "**/jest.config.*"            # Jest 設定
  - "**/vitest.config.*"          # Vitest 設定
  - "**/pytest.ini"               # pytest 設定
  - "**/.nycrc*"                  # カバレッジ設定

Read:
  - jest.config.ts               # Jest 設定
  - vitest.config.ts             # Vitest 設定
  - package.json                  # scripts セクションの test コマンド

Grep:
  - pattern: "describe\\(|it\\(|test\\("
    output_mode: files_with_matches
    head_limit: 20
  - pattern: "jest\\.mock|vi\\.mock|sinon\\."
    output_mode: content
    context: 3
    head_limit: 10
  - pattern: "beforeEach|afterEach|beforeAll|afterAll"
    output_mode: files_with_matches
    head_limit: 10
```

### 最低探索量

- テストファイルを最低 **2 ファイル**全文 Read し、テストパターン（describe/it 構造、モック戦略）を把握する
- テスト設定ファイルを **1 ファイル** Read する

---

## 探索の必須ルール

1. **3段階探索の徹底**: Glob で発見 → Grep で絞り込み → **Read で深掘り**。Glob/Grep だけで終わる探索は**不十分**である
2. **Read の最低基準**: 関連ファイルは合計 **最低 10 ファイル**を Read する。Glob/Grep 結果の上位ヒットは必ず中身を確認する
3. **コードスニペット必須**: 各カテゴリで最低 1 つのコードスニペットをレポートに含める。「ファイルが存在する」だけの記述は不十分
4. **逆引き探索の徹底**: 変更対象ファイルを import/require しているファイルを必ず検索し、影響範囲を特定する
5. **head_limit の活用**: 大量ヒットする場合は `head_limit: 30` 等で絞り、ヒットしたファイルから優先度の高いものを Read する
6. **並列実行**: 独立した Glob/Grep は並列で実行して効率化（タイムアウト 5 分以内に収める）
7. **指示書優先**: CLAUDE.md の検索ルール・除外パターンに必ず従う
