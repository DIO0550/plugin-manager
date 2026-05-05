# Issue #264 共通 state 型の設計レビュー

## 概要

Issue #264 では、TUI manager の画面別 `Model` / `CacheState` にある重複について、将来共通化する場合の API、所有関係、既存画面 `Model` への埋め込み方を整理する。

本レビューは次の文書を前提にする。

- `docs/tui-model-cachestate-field-comparison-2026-05-05.md`
- `docs/review-duplicate-roles-2026-05-03.md`

## 結論

- 今回は Rust 実装を行わない。成果物は `docs/` 配下の設計レビュー文書のみとする。
- 共通化候補は `ListState` と選択キーを束ねる小型 helper に限定する。
- `CacheState` 全体の trait 化、`Errors` 画面への空 `CacheState` 追加、画面内退避状態の統合は見送る。
- stale ID 検証、fallback、フィルタ同期、`+ Add new` の意味付けは、今後も各画面の `from_cache()` / update ロジックに残す。

## 現状整理

`ScreenCache` はタブ切替時に保持する軽量 state として、`installed::CacheState`、`discover::CacheState`、`marketplaces::CacheState` を所有する。`Errors` は `DataStore.last_error` を表示する画面であり、画面固有の cache を持たない。

`Installed` と `Marketplaces` は、どちらも一覧の選択に `ListState` と選択 ID を使う。ただし、復元時に検証するデータソースと fallback の意味が異なる。

- `Installed`: `PluginId` を `DataStore::find_plugin()` / `plugin_index()` で検証し、存在しなければ先頭 plugin へ fallback する。`marked_ids` は存在しない plugin を除外して復元する。
- `Marketplaces`: marketplace 名を `DataStore::find_marketplace()` / `marketplace_index()` で検証し、存在しなければ先頭 marketplace へ fallback する。marketplace がない場合も `+ Add new` として index 0 を選択する。
- `Discover`: `selected_id` と `ListState` を持つが、現時点では実データリストと stale ID の検証先が未定義である。
- `Errors`: cache 対象外であり、形を揃えるための空 state は不要である。

## 推奨 API 候補

将来、3画面以上で選択状態操作の重複が増えた場合は、次のような小型 helper を候補にする。

```rust
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct SelectionState<K> {
    pub selected_id: Option<K>,
    pub list_state: ListState,
}

impl<K> SelectionState<K> {
    pub fn new(selected_id: Option<K>, selected_index: Option<usize>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(selected_index);
        Self {
            selected_id,
            list_state,
        }
    }

    pub fn selected_id(&self) -> Option<&K> {
        self.selected_id.as_ref()
    }
}
```

この helper は UI 選択状態の形だけを表す。次の責務は持たせない。

- stale ID 検証
- 選択不可データへの fallback
- Installed のフィルタ同期
- Marketplaces の `None == + Add new` の意味付け
- `marked_ids`、`update_statuses`、`browse_plugins` などの画面固有状態

## 所有関係

| 所有者 | 所有する state | 方針 |
|---|---|---|
| `core::ScreenCache` | タブ切替で保持する軽量 state | 画面別 `CacheState` をそのまま所有する |
| 各 screen module の `CacheState` | 復元に必要な semantic state | Feature 内に閉じ込める |
| 各 screen module の `Model` | 表示中画面の UI state と一時 state | variant 固有の意味を保つ |
| 将来の `SelectionState<K>` | `selected_id` と `ListState` の束 | 画面 `Model` へ composition で埋め込む |

`SelectionState<K>` を導入する場合も、`ScreenCache` が `SelectionState<K>` を直接所有する必要はない。cache はタブ切替後に再構築できる semantic state だけを持ち、`ListState` は `from_cache()` で再生成する現在の方針を維持する。

## 既存画面への埋め込み案

`Installed` では `PluginList` の `selected_id` と `state` を置き換える候補になる。`marked_ids` と `update_statuses` は画面固有状態として残す。

```rust
Model::PluginList {
    selection: SelectionState<PluginId>,
    marked_ids: HashSet<PluginId>,
    update_statuses: HashMap<PluginId, UpdateStatusDisplay>,
}
```

`Marketplaces` では `MarketList` の `selected_id` と `state` を置き換える候補になる。`operation_status`、`error_message`、`+ Add new` の意味付けは `MarketList` 側に残す。

```rust
Model::MarketList {
    selection: SelectionState<String>,
    operation_status: Option<OperationStatus>,
    error_message: Option<String>,
}
```

`Discover` は実データリストと検証先が定義された後に判断する。現時点で helper だけを先に導入しても、復元仕様を固定できない。

## 却下案

### `CacheState` trait 化

`CacheState` は画面ごとに保存する semantic state が異なる。`Installed` は `selected_plugin_id` と `marked_ids` を持ち、`Marketplaces` は `selected_id` のみを持つ。`Discover` は検証先が未定義で、`Errors` は cache を持たない。

この段階で trait 化すると、実装の重複削減よりも抽象境界の説明コストが大きい。将来検討する場合も、`Errors` を対象外にできる設計を前提にする。

### `Errors` への空 `CacheState` 追加

`Errors` は `DataStore.last_error` を表示するだけで、タブ切替時に保持すべき画面固有 state を持たない。4画面の形を揃えるためだけに空 `CacheState` を追加すると、存在しない責務を型に持たせることになる。

### 画面内退避状態の統合

`Installed` の `saved_marked_ids` / `saved_update_statuses` や、`Marketplaces` の browse/install flow は画面内遷移のための state であり、タブ切替 cache とは寿命が異なる。`SelectionState<K>` と同じ構造に含めると、復元対象と一時状態の境界が曖昧になる。

## 将来移行手順

1. `SelectionState<K>` 導入前に、既存の `from_cache()` / `to_cache()` / update テストで現在挙動を固定する。
2. `src/tui/manager/core/selection_state.rs` を追加し、`core.rs` から必要最小限に re-export する。
3. `Installed::Model::PluginList` の `selected_id` と `state` を `SelectionState<PluginId>` に置き換える。
4. `Marketplaces::Model::MarketList` の `selected_id` と `state` を `SelectionState<String>` に置き換える。
5. `Discover` は一覧データと stale ID 検証先が決まった時点で対象に含めるか再判断する。
6. 各画面の `from_cache()` には stale ID 検証と fallback を残し、helper には構築済みの選択 ID と index だけを渡す。

## テスト方針

今回の変更は設計レビュー文書のみであり、自動テスト対象はない。将来 Rust 実装を行う場合は、次の挙動を先にテストで固定する。

- `SelectionState::new()` が `selected_id` と `ListState::selected()` を指定通りに初期化する。
- `Installed::Model::from_cache()` が stale `selected_plugin_id` を先頭 plugin へ fallback する。
- `Installed::Model::from_cache()` が stale `marked_ids` を除外する。
- `Marketplaces::Model::from_cache()` が stale `selected_id` を先頭 marketplace へ fallback する。
- marketplace が空の場合も `Marketplaces` が `+ Add new` として index 0 を選択する。
- `Discover` の検証先が定義された後、その復元ポリシーを追加する。

## 手動検証

- 依存文書の結論と同じく、今回 Rust ファイルを変更しない方針にしている。
- 3画面以上で重複する候補を `ListState + selected key` に限定している。
- `Errors` を cache 対象外として扱い、空 `CacheState` を追加しない判断にしている。
- stale ID 検証と fallback を画面別責務として残している。
- `ScreenCache`、画面別 `CacheState`、画面 `Model`、将来 helper の所有関係を分離している。
