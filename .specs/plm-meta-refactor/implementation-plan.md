# installedAt を .plm-meta.json に分離する実装計画

## 概要

プラグインのインストール日時（`installedAt`）を、元の `plugin.json` から分離し、PLM専用のメタデータファイル `.plm-meta.json` に移行する。

## 背景・目的

- **問題**: 現在の実装では、プラグイン作者が作成した `plugin.json` にPLMが `installedAt` を直接書き込んでいる
- **影響**: 上流成果物の改変となり、署名/ハッシュ検証や更新差分の整合性を壊す恐れがある
- **慣例**: npm/cargo等の主要パッケージマネージャーは配布物を改変せず、メタ情報を別管理している

## 設計方針

### ファイル構成

```
~/.plm/cache/plugins/{marketplace}/{name}/
├── plugin.json          ← [UNCHANGED] 上流のまま保持
└── .plm-meta.json       ← [NEW] PLM管理メタデータ
```

### .plm-meta.json スキーマ

```rust
/// PLMが管理するプラグインメタデータ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginMeta {
    /// インストール日時（RFC3339形式）
    /// 欠損時は None として扱う
    #[serde(default, rename = "installedAt")]
    pub installed_at: Option<String>,
}
```

**設計判断:**
- `installed_at` は `Option<String>` とし、欠損を許容
- 将来拡張フィールド（`source`, `updatedAt`）は今回のスコープ外。必要時に追加

### 動作仕様

#### 優先順位ルール（重要）

**読み込み時の優先順位:**
1. `.plm-meta.json` の `installedAt` を優先
2. `.plm-meta.json` が無いか `installedAt` が欠損 → `plugin.json` の `installedAt` にフォールバック
3. 両方無い → `None`

**両方に値がある場合:**
- `.plm-meta.json` を正とし、`plugin.json` は無視（警告ログなし）

**欠損判定の定義:**
以下のケースは全て「欠損」として扱い、フォールバックする:
- フィールドが存在しない
- `"installedAt": null`
- `"installedAt": ""`（空文字列）
- `"installedAt": "   "`（空白のみの文字列）→ `trim()` して空なら欠損扱い

#### 書き込み仕様

**アトミック書き込み手順:**
1. 同一ディレクトリに一時ファイルを作成（`tempfile::NamedTempFile` を使用し、並行実行時の衝突を回避）
2. JSON を書き込み
3. `persist()` で `.plm-meta.json` に置き換え
   - Windows では `persist()` が既存ファイルで失敗する可能性があるため、
     `AlreadyExists` エラー時は既存ファイルを削除してから再度 `persist()` を試行

**書き込み失敗時の扱い:**
- `write_meta()` の失敗は**警告ログ + 継続**（インストール自体は成功扱い）
- `installedAt` は補助情報であり、欠損してもプラグイン動作に影響しないため

**タイムスタンプ形式:**
- UTC固定、RFC3339形式
- 例: `2025-01-15T10:30:00Z`
- 実装: `Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()`

**書き込みタイミング:**
- `store_from_archive()` でプラグイン展開後に `.plm-meta.json` を作成

#### 欠損・破損時の挙動

| 状態 | 挙動 |
|------|------|
| `.plm-meta.json` が無い | `plugin.json` からフォールバック読み込み |
| `.plm-meta.json` が破損（パースエラー） | 警告ログを出力し、`plugin.json` からフォールバック（読み取り時は `.bak` 退避を行わない。次回書き込み時に上書きされる） |
| `.plm-meta.json` の読み込み時にIOエラー（権限エラー等） | 警告ログを出力し、`plugin.json` からフォールバック |
| `.plm-meta.json` の `installedAt` が `None` または空文字 | `plugin.json` からフォールバック |
| `plugin.json` が無い | `None` を返す（エラーにはしない） |
| `plugin.json` が破損（パースエラー） | 警告ログを出力し、`None` を返す（エラーにはしない） |
| `plugin.json` の読み込み時にIOエラー | 警告ログを出力し、`None` を返す |
| `plugin.json` の `installedAt` が `null` または空文字/空白のみ | `None` を返す（欠損扱い、`.plm-meta.json` と同じ判定） |
| `plugin.json` の `installedAt` が非RFC3339 | そのまま文字列として返す（検証は呼び出し側で必要に応じて） |

**設計判断:**
- `installedAt` は補助情報であり、欠損してもプラグイン動作に影響しないため、エラーではなく `None` を返す
- 耐久性（fsync）はベストエフォートとし、明示的な保証は行わない
- `.plm-meta.json` はプラグインディレクトリ内に配置するが、将来ハッシュ/署名検証を導入する場合は除外対象とする
  - 必要に応じて `~/.plm/meta/{marketplace}/{name}.json` への移行も検討可能

#### 再インストール・更新時

- 再インストール時は `installedAt` を**更新**（現在時刻で上書き）
- 更新履歴が必要な場合は将来 `updatedAt` フィールドで対応

**更新タイミングの定義:**
- `.plm-meta.json` の `installedAt` はプラグイン展開時（`store_from_archive()`）に**常に現在時刻で上書き**する
- これは「初回インストール」「再インストール」「更新」を区別せず、展開操作があれば更新される設計
- **意味の定義**: `installedAt` は「最終インストール/更新日時」を表す（初回インストール日時ではない）
- 初回インストール日時を保持したい場合は、将来 `firstInstalledAt` フィールドを追加して対応

**配布物に `.plm-meta.json` が含まれる場合:**
- アーカイブ展開後にPLMが `.plm-meta.json` を**上書き**する
- プラグイン作者が同梱した `.plm-meta.json` は無視される

## Codexレビュー指摘事項への対応

### 指摘1: plugin.json を「UNCHANGED」としつつ PluginManifest の serde に依存する設計

**調査結果:**
- 現状 `plugin.json` を再シリアライズする箇所は `write_installed_at()` のみ
- この関数は `serde_json::Value` で部分更新するため未知フィールドは保持される
- **今回の変更で `write_installed_at()` を削除するため、この懸念は解消される**

**設計方針（明確化）:**
- `plugin.json` はアーカイブからのコピーのみとし、PLMは再シリアライズしない
- `PluginManifest` は読み込み専用（Deserialize）として扱い、書き込みには使用しない
- 将来 `plugin.json` への書き込みが必要になった場合は、`serde_json::Value` による部分更新を使用する

### 指摘2: .plm-meta.json を同一ディレクトリに置く影響範囲

**調査結果:**
- `cache.rs::list()` でディレクトリ走査するが、プラグインディレクトリ単位の操作のみ
- 現時点でハッシュ検証・差分検出の処理は存在しない

**設計方針（明確化）:**
- 現時点では `.plm-meta.json` の除外ルールは不要
- 将来署名/ハッシュ検証を導入する際は、`.plm-meta.json` を除外対象とする
- 除外方法: glob パターン `!.plm-meta.json` または専用の無視リスト

### 指摘3: store_from_archive() で常時上書きする前提

**調査結果:**
- `store_from_archive()` の呼び出し箇所は `github_source.rs` のダウンロード処理のみ
- キャッシュ再構築・修復等の用途では使用されていない

**設計方針（明確化）:**
- `store_from_archive()` はインストール/更新操作専用の関数として位置づける
- `installedAt` は展開操作時に常に現在時刻で上書きする（設計意図どおり）
- 将来キャッシュ修復等で `installedAt` を保持したい場合は、別関数を追加する

### 指摘4: tempfile::NamedTempFile の依存関係

**調査結果:**
- `Cargo.toml` に既に `tempfile = "3"` が存在（テストで使用中）
- 新規依存追加は不要

**対応:** 問題なし。既存依存を使用。

### 指摘5: PluginManifest の skip_serializing による出力への影響

**調査結果:**
- `plm info --json` は `PluginDetail`（DTO）をシリアライズ
- `PluginManifest` を直接シリアライズする箇所はない
- `PluginDetail.installed_at` は `manifest.installed_at.clone()` で設定

**設計方針（明確化）:**
- `PluginManifest.installed_at` に `skip_serializing` を追加しても出力への影響はない
- `PluginDetail.installed_at` は `meta::resolve_installed_at()` から取得するよう変更
- 出力スキーマの変更なし（`PluginDetail` 経由で `installedAt` は出力される）

### 指摘6: installedAt 正規化ロジックの一貫性

**設計方針（明確化）:**
- `meta.rs` に `normalize_installed_at()` 共通関数を追加
- `.plm-meta.json` と `plugin.json` 両方の値に同じ正規化ルールを適用
- 正規化ルール: `trim()` 後に空なら `None`、それ以外は `Some`

```rust
/// installedAt の正規化
/// 空文字/空白のみ → None、それ以外 → Some(trimmed)
fn normalize_installed_at(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
}
```

### 指摘7: installedAt の意味変更がドキュメント未更新

**指摘内容:**
- 「最終インストール/更新日時」に変わるが、CLI/ドキュメントの更新が計画に含まれていない
- 利用者が「初回インストール日時」と誤解する恐れ

**対応方針:**
- `plm info` の表示ラベルを「Installed At」のまま維持（意味的に「最終」を含意）
- 将来 `firstInstalledAt` を追加する場合は別ラベル「First Installed」を使用
- 現時点でドキュメント更新は不要（既存動作と実質同じ意味）

### 指摘8: 読み取り系コマンドでの副作用

**指摘内容:**
- `.plm-meta.json` 破損時に `.bak` へリネームする設計は、`plm info` 等で書き込みが発生
- 読み取り専用環境で想定外の副作用になり得る

**設計変更:**
- **読み取り時（`resolve_installed_at()`）**: 破損時は警告ログのみ、`.bak` 退避は行わない
- **書き込み時（`write_meta()`）**: 既存の破損ファイルがあれば上書き（アトミック書き込みで自動対応）
- 読み取り専用操作では副作用なし

### 指摘9: 破損警告のログノイズ

**指摘内容:**
- 退避失敗環境では警告が毎回出続ける可能性

**対応方針:**
- 読み取り時の破損警告は1回のみ出力（同一実行中は抑制不要、コマンド単位で完結）
- 警告メッセージに「次回インストール時に再生成されます」を追記
- 例: `Warning: .plm-meta.json is corrupted, falling back to plugin.json. It will be regenerated on next install.`

### installed_at 参照箇所（棚卸し）

| ファイル | 行 | 用途 | 対応 |
|----------|-----|------|------|
| `src/plugin/manifest.rs:57` | フィールド定義 | `skip_serializing` 追加 |
| `src/application/plugin_info.rs:294` | `PluginDetail` 構築 | `meta::resolve_installed_at()` に変更 |
| `src/commands/info.rs:90` | 表示 | 変更不要（`PluginDetail` 経由） |
| `src/plugin/cache.rs:230` | 書き込み呼び出し | 削除、`meta::write_meta()` に変更 |
| `src/plugin/cache.rs:241` | `write_installed_at()` 関数 | 削除 |

## 変更対象ファイル

### [NEW] src/plugin/meta.rs
PLMメタデータ構造体と読み書き関数を定義

```rust
const META_FILE: &str = ".plm-meta.json";

/// PLMが管理するプラグインメタデータ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginMeta {
    #[serde(default, rename = "installedAt")]
    pub installed_at: Option<String>,
}

/// メタデータを書き込む（アトミック書き込み）
pub fn write_meta(plugin_dir: &Path, meta: &PluginMeta) -> Result<()>

/// メタデータを読み込む
/// 欠損時は None、破損時は警告ログを出力して None を返す
pub fn load_meta(plugin_dir: &Path) -> Option<PluginMeta>

/// installedAt を取得（.plm-meta.json → plugin.json のフォールバック付き）
/// manifest が None の場合は plugin.json を読み込んでフォールバック
pub fn resolve_installed_at(plugin_dir: &Path, manifest: Option<&PluginManifest>) -> Option<String>
```

### [MODIFY] src/plugin/cache.rs
- `write_installed_at()` を削除
- `store_from_archive()` 内で `meta::write_meta()` を呼び出す

### [MODIFY] src/plugin/manifest.rs
- `installed_at` フィールドは残す（後方互換の読み込み用）
- `#[serde(skip_serializing)]` を追加し、シリアライズ時に出力されないようにする
- これにより、他の箇所で `PluginManifest` をシリアライズしても `installedAt` が書き込まれない

### [MODIFY] src/application/plugin_info.rs
- `PluginDetail` 構築時に `meta::resolve_installed_at()` を使用
- `manifest.installed_at` への直接参照を削除

### [MODIFY] src/plugin.rs
- `pub mod meta;` を追加

### [MODIFY] tests
- `cache.rs` のテストを `.plm-meta.json` 検証に変更
- フォールバック動作のテストを追加

## 後方互換性

### 移行戦略

1. **Phase 1（今回実装）**:
   - 新規インストール時に `.plm-meta.json` を作成
   - 読み込み時は `.plm-meta.json` → `plugin.json` の順でフォールバック

2. **Phase 2（将来オプション）**:
   - 既存プラグインの `plugin.json` から `installedAt` を削除するマイグレーション
   - または `plm info` 実行時に自動で `.plm-meta.json` を生成

### 既存プラグインへの影響

- 既にインストール済みのプラグインは、`plugin.json` の `installedAt` がそのまま使用される
- 再インストール時に `.plm-meta.json` が作成され、以降はそちらが優先

## 検証計画

1. **単体テスト**
   - `PluginMeta` のシリアライズ/デシリアライズ
   - `write_meta()` / `load_meta()` の動作
   - `resolve_installed_at()` のフォールバック動作
   - 破損ファイルの処理

2. **統合テスト**
   - `plm install` 後に `.plm-meta.json` が作成されることを確認
   - `plm info` で `installedAt` が正しく表示されることを確認

3. **後方互換テスト**
   - `.plm-meta.json` が無い場合のフォールバック動作
   - `plugin.json` に `installedAt` がある場合の読み込み
   - 両方に値がある場合の優先順位

## 実装順序

1. `src/plugin/meta.rs` を新規作成
2. `src/plugin.rs` に `pub mod meta;` を追加
3. `src/plugin/cache.rs` の書き込み処理を修正
4. `src/application/plugin_info.rs` の読み込み処理を修正
5. テストを修正・追加
6. `cargo test` で全テストパス確認
7. 手動テストで動作確認
