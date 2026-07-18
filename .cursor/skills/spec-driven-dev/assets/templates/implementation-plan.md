# {目的を簡潔に記述}

**関連Issue**: #{Issue番号} <!-- Issueから作成した場合のみ記載 -->

{背景・なぜこの変更が必要か、1-2文}

## ユーザーレビューが必要な点

> **IMPORTANT** / **NOTE**
> - {確認してほしい判断事項}
> - {破壊的変更の有無}
> - {設計上のトレードオフ}

## システム図

### 状態マシン / フロー図

```
{状態遷移やデータフローをASCII図で記述}
{すべてのパス・分岐・エッジケースを可視化}

例:
    入力
      │
      ▼
┌─────────────┐
│  STATE_A    │─── 条件1 ───▶ STATE_B
└─────────────┘                  │
      │                          │
   条件2                      条件3
      │                          │
      ▼                          ▼
┌─────────────┐           ┌─────────────┐
│  STATE_C    │           │  STATE_D    │
│ (処理内容)  │           │ (処理内容)  │
└─────────────┘           └─────────────┘
```

### データフロー

```
{コンポーネント間のデータの流れを記述}

例:
User Input
    ↓
Component A
    ↓
├─ Service B (API呼び出し)
│      ↓
│  External API
│      ↓
└─ State Store
    ↓
UI Update
```

## 変更案

### {カテゴリ1: プロジェクト構造 / UI / ロジック / etc.}

#### [NEW] `{ファイルパス}`

{このファイルで何をするか}

- **Props**: {該当する場合}
- **スタイリング**: {該当する場合}
- **ロジック**: {該当する場合}

```{lang}
// {ファイルパス}

{export する型定義・インターフェース}

{主要な関数シグネチャと処理概要コメント}
```

#### [MODIFY] `{ファイルパス}`

{変更内容}

- {変更点1}
- {変更点2}

```{lang}
// before:
{変更前の該当箇所}
```

```{lang}
// after:
{変更後の該当箇所}
```

#### [DELETE] `{ファイルパス}`

{削除理由}

### {カテゴリ2}

...

## 検証計画

### テスト戦略

**機能タイプ**: {Pure Logic / API Integration / Data Transformation / State Management / UI Component / Async Operations / Security/Auth / Configuration / DOM Manipulation}
**テスト方針**: {TDD / テスト追加 / 手動検証のみ}
**根拠**: {なぜこのテスト方針を選択したか — 機能タイプと test-design-patterns.md の決定フローに基づく1-2文}

### 自動テスト

<!-- テスト方針が「TDD」または「テスト追加」の場合に記載。テストファイルごとにセクションを作成し、各ファイルの役割とテストTODOリストを記載する。test-design-patterns.md のタイプ別シナリオを参照して網羅的に列挙する -->

#### `{テストファイルパス1}`

**役割**: {このテストファイルが何を検証するか — 対象モジュールと検証の観点を1文で}

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | {最もシンプルなケース} | {どういう状況で誰が何をしたとき} | {どうなるべきか} |
| 正常系 | {バリエーション} | {別の状況・入力パターン} | {期待される出力} |
| 境界値 | {境界条件のテスト} | {最小値/最大値/ゼロなどの状況} | {期待される振る舞い} |
| 異常系 | {エラーケース} | {不正入力・障害発生の状況} | {エラー応答/例外} |
| エッジケース | {特殊なケース} | {想定外だが起こりうる状況} | {期待される振る舞い} |

#### `{テストファイルパス2}`

**役割**: {このテストファイルが何を検証するか}

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | {テストケース} | {想定シナリオ} | {期待結果} |
| ... | ... | ... | ... |

<!-- テストファイルが増える場合は同じ形式でセクションを追加 -->

**テスト実行コマンド**

```bash
{実行するコマンド}
```

### 手動検証

1. {具体的な確認手順1}
2. {具体的な確認手順2}
3. {具体的な確認手順3}

## Definition of Done

以下をすべて満たした時点で本機能の実装完了とする。

- [ ] すべてのタスク（tasks.md）が ■ になっている
- [ ] {機能固有の受入条件1}
- [ ] {機能固有の受入条件2}
- [ ] 自動テストが通る（該当する場合）
- [ ] 手動検証が完了している
- [ ] 既存機能にリグレッションがない

---

<!--
使用例:

# Ranking Page にブロックボタンを実装

ニコニコのランキングページでユーザーをブロックする機能を追加します。

## ユーザーレビューが必要な点

> **NOTE**
> - 破壊的変更はありません。これは新しい機能です。
> - ボタンのスタイルは既存のUIに合わせています。

## システム図

### 状態マシン / フロー図

```
                    ページ読み込み
                         │
                         ▼
                 ┌───────────────┐
                 │     IDLE      │
                 └───────────────┘
                         │
              MutationObserver検出
                         │
                         ▼
                 ┌───────────────┐
                 │  DETECTING    │◀─────────────┐
                 │ (ユーザー要素) │               │
                 └───────────────┘               │
                         │                       │
            ┌────────────┴────────────┐          │
            ▼                         ▼          │
    ユーザー要素あり           要素なし          │
            │                         │          │
            ▼                         └──────────┘
    ┌───────────────┐                 (継続監視)
    │ BUTTON_INJECT │
    │ (ボタン注入)   │
    └───────────────┘
            │
     ボタンクリック
            │
            ▼
    ┌───────────────┐
    │   BLOCKING    │
    │ (Storage更新) │
    └───────────────┘
            │
            ▼
    ┌───────────────┐
    │   BLOCKED     │
    │ (UI非表示化)  │
    └───────────────┘
```

### データフロー

```
DOM (ランキングページ)
    ↓
MutationObserver
    ↓
ranking.ts (コンテンツスクリプト)
    ↓
├─ BlockButton コンポーネント生成
│      ↓
│  ユーザークリック
│      ↓
└─ storage.ts
    ↓
chrome.storage.local (ブロックリスト)
    ↓
UI更新 (動画非表示)
```

## 変更案

### Content Scripts

#### [NEW] `src/content/ranking.ts`

ランキングページ用のコンテンツスクリプト。

- **ロジック**: DOM監視でユーザー要素を検出し、ブロックボタンを注入
- **依存**: `src/storage.ts` のブロックリスト操作

```ts
// src/content/ranking.ts

import { getBlockList, addToBlockList } from "../storage";
import { createBlockButton } from "../components/BlockButton";

const RANKING_USER_SELECTOR = ".RankingVideo-userName";

export function initRankingBlocker(): void {
  const observer = new MutationObserver((mutations) => {
    // ユーザー要素を検出し、ブロックボタンを注入
  });

  observer.observe(document.body, { childList: true, subtree: true });
}

function injectBlockButtons(userElements: NodeListOf<Element>): void {
  // 各ユーザー要素にブロックボタンを追加
}
```

#### [MODIFY] `manifest.json`

コンテンツスクリプトの登録を追加。

- content_scriptsにranking.tsを追加
- matchesに `*://www.nicovideo.jp/ranking/*` を追加

```json
// before:
"content_scripts": [
  {
    "matches": ["*://www.nicovideo.jp/watch/*"],
    "js": ["src/content/watch.js"]
  }
]
```

```json
// after:
"content_scripts": [
  {
    "matches": ["*://www.nicovideo.jp/watch/*"],
    "js": ["src/content/watch.js"]
  },
  {
    "matches": ["*://www.nicovideo.jp/ranking/*"],
    "js": ["src/content/ranking.js"]
  }
]
```

### UI

#### [NEW] `src/components/BlockButton.ts`

汎用ブロックボタンコンポーネント。

- **Props**: userId: string, onBlock: () => void
- **スタイリング**: 既存のニコニコUIに合わせたグレーボタン

```ts
// src/components/BlockButton.ts

export interface BlockButtonProps {
  userId: string;
  onBlock: (userId: string) => void;
}

export function createBlockButton({ userId, onBlock }: BlockButtonProps): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = "block-button";
  button.textContent = "ブロック";
  button.addEventListener("click", () => onBlock(userId));
  return button;
}
```

## 検証計画

### 手動検証

1. ランキングページ https://www.nicovideo.jp/ranking を開く
2. 各ユーザー名の横にブロックボタンが表示されることを確認
3. ボタンをクリックしてユーザーがブロックされることを確認
4. ページリロード後、ブロックしたユーザーの動画が非表示になることを確認

## Definition of Done

以下をすべて満たした時点で本機能の実装完了とする。

- [ ] すべてのタスク（tasks.md）が ■ になっている
- [ ] ランキングページで各ユーザー名の横にブロックボタンが表示される
- [ ] ブロックボタンクリックでユーザーがブロックリストに追加される
- [ ] ブロック済みユーザーの動画がランキングから非表示になる
- [ ] ページリロード後もブロック状態が維持される
- [ ] 既存のコンテンツスクリプトにリグレッションがない
-->
