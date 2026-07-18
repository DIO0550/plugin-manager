# Task: {目的を簡潔に記述}

## Research & Planning

- □ {調査・計画タスク1}
- □ {調査・計画タスク2}
- □ {調査・計画タスク3}

## Implementation

- □ {実装タスク1}
- □ {実装タスク2}
- □ {実装タスク3}
- □ {実装タスク4}

## Verification

- □ {検証タスク1}
- □ {検証タスク2}
- □ {検証タスク3}

---

<!--
使用例 (標準構成):

# Task: Implement Block Button on Ranking Page

## Research & Planning

- □ Analyze Nico Nico ranking page DOM for User ID and Button placement
- □ Explore existing codebase (content scripts, storage)
- □ Create Implementation Plan

## Implementation

- □ Create/Update content script for Ranking Page
- □ Implement generic "Block Button" injection logic
- □ Implement logic to extract User ID from DOM
- □ Connect button to Blocklist Storage

## Verification

- □ Verify button appears on ranking page
- □ Verify clicking blocks the user
- □ Verify blocked videos are hidden

---

使用例 (TDD 構成 — テストを含む場合):

# Task: Implement Price Calculator

## Research & Planning

- □ テスト環境・既存テスト構成の調査
- □ テストTODOリスト作成
  - 正常系: 単品の税込価格計算
  - 正常系: 複数商品の合計計算
  - 境界値: 0円の商品
  - 異常系: 負の価格
  - エッジケース: 小数点以下の丸め
- □ 既存の価格関連コードの調査

## Implementation (TDD サイクル)

### 単品の税込価格計算
- □ RED: 単品の税込価格を計算するテストを書く
- □ GREEN: テストを通す最小限の実装（仮実装）
- □ REFACTOR: 不要

### 複数商品の合計計算
- □ RED: 複数商品の合計を計算するテストを書く
- □ GREEN: テストを通す実装（三角測量で一般化）
- □ REFACTOR: 計算ロジックの共通化

### 境界値・異常系
- □ RED: 0円の商品のテストを書く
- □ GREEN: テストを通す実装
- □ RED: 負の価格でエラーを返すテストを書く
- □ GREEN: バリデーション実装
- □ RED: 小数点以下の丸めテストを書く
- □ GREEN: 丸めロジック実装
- □ REFACTOR: エラーハンドリングの整理

## Verification

- □ 全テストがパスすることを確認
- □ 既存の価格表示に影響がないことを確認

---

使用例 (テスト追加構成 — TDD以外でテストが必要な場合):

# Task: Implement API Client for User Service

## Research & Planning

- □ 既存テストインフラの確認（テストランナー、モック戦略）
- □ API仕様の確認（エンドポイント、レスポンス形式）
- □ テスト対象シナリオの洗い出し

## Implementation

- □ API クライアントの型定義
- □ API クライアントの実装
- □ エラーハンドリングの実装

## Test

- □ 正常系テスト: 成功レスポンスの処理
- □ 異常系テスト: HTTPエラーレスポンス（4xx, 5xx）
- □ 異常系テスト: ネットワークエラー・タイムアウト
- □ エッジケーステスト: 空レスポンス・不正なJSON

## Verification

- □ 全テストがパスすることを確認
- □ 既存APIクライアントに影響がないことを確認

---

タスク構成の判断基準（test-design-patterns.md に基づく）:

TDD 構成:
機能タイプが Pure Logic / Data Transformation / State Management / Security/Auth を含む場合、
または hearing-notes.md にテスト要件が含まれる場合は TDD サイクルで構成する。
順序: シンプルな正常系 → バリエーション → 境界値 → 異常系・エッジケース

テスト追加構成:
API Integration / Async Operations / UI Component 等で自動テストが有効な場合、
Implementation の後に Test セクションを追加して構成する。

手動検証のみ:
純粋なUI/スタイリング変更で、ロジック・状態管理・外部依存を含まない場合。

---

タスク分割のガイドライン:

1. Research & Planning
   - 既存コードの調査
   - DOM/API構造の分析
   - 実装計画の作成
   - 必要なライブラリの調査

2. Implementation
   - 新規ファイルの作成
   - 既存ファイルの修正
   - ロジックの実装
   - UI/スタイリングの実装
   - 連携・統合

3. Verification
   - 機能が動作することの確認
   - エッジケースの確認
   - 既存機能への影響確認
   - パフォーマンス確認（必要時）

各タスクは:
- できるだけ細かい単位に分割する（1ファイル1変更、1関数1タスクが理想）
- 大きなタスクは必ずサブタスクに分解する
- 明確な完了条件がある
- 独立して実行可能（できるだけ）

---

TaskCreate/TaskUpdate 連携ガイドライン:

tasks.md のタスクを実装する際、TaskCreate ツールで進捗管理用のTodoを作成する。

1. タスク作成ルール
   - tasks.md の各タスク行（□）に対して TaskCreate を実行
   - subject: タスク行のテキストをそのまま使用
   - activeForm: 進行形に変換（例: "型定義を作成" → "型定義を作成中"）
   - description: implementation-plan.md の該当セクションから補足情報を含める

2. 依存関係の設定
   - Research & Planning → Implementation → Verification の順序で blockedBy を設定
   - 同一セクション内で順序依存がある場合も blockedBy を設定
   - 独立して実行可能なタスク同士には依存関係を設定しない

3. ステータス更新
   - タスク着手時: TaskUpdate で status を "in_progress" に変更
   - タスク完了時: TaskUpdate で status を "completed" に変更
   - tasks.md の □ → ■ 更新と同時に行う
-->
