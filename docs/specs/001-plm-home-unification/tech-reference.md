# PLM_HOME / HOME 解決の一元化 · 実装の手引き

> このページは implementation-plan を読んで実装に取りかかったときに
> 「ここがよく分からない」となりがちな箇所を、
> ライブラリドキュメントの体裁でまとめたものです。
> 順番に読む必要はなく、詰まったセクションだけ拾い読みできます。

---

## 概要

PLM（Plugin Manager CLI）は、インストールしたプラグインのキャッシュや、ターゲット・マーケットプレイス・インポートなどの設定 JSON を、ディスク上の決まった場所に保存します。その「決まった場所」の親ディレクトリを、これまで複数のモジュールがそれぞれ別々のやり方で決めていました。ある場所は `PLM_HOME` を見て、別の場所は `HOME` だけを見る、という状態です。結果として、利用者が `PLM_HOME` を設定しても、キャッシュは新ルートに、設定 JSON は旧ルートに、と状態が分裂することがありました。

この機能の目的は、その親ディレクトリの決め方を一箇所に集約することです。新しい入口は `plm_root()` と、そこから派生する `PlmPaths` です。解決のルールは単純で、有効な `PLM_HOME` があればそれを使い、なければ `HOME` にフォールバックします（案 A: HOME 代替）。どちらも空・空白のみ・相対パスならエラーです。全 5 つのレジストリ／キャッシュ（`PackageCache`・`MarketplaceRegistry`・`MarketplaceConfig`・`TargetRegistry`・`ImportRegistry`）が、この同じポリシーを通るよう書き換えます。

一方で、Codex や Cursor といった AI 環境本体の Personal 配置（例: `~/.codex`）は、利用者の本当のホームディレクトリを指し続ける必要があります。そのため `src/target/core/paths.rs` や `src/plugin/cache/cleanup.rs` の HOME 参照は意図的に触りません。PLM の「状態」だけを差し替え、環境本体の配置先は従来どおり、という境界がこの変更の肝です。

```
  利用者 / CI
       │
       │  PLM_HOME=/tmp/plm-test  （任意）
       │  HOME=/home/alice        （通常は OS が設定）
       ▼
  plm コマンド（install / target / marketplace / import …）
       │
       ▼
  ┌─────────────────────────────────────┐
  │  PlmPaths::new()                    │  ← 全 5 経路の統一入口
  │       │                             │
  │       ▼                             │
  │  plm_root()                         │
  │   PLM_HOME（有効時）→ HOME          │
  │   絶対パス必須・空/空白は無効       │
  └─────────────────────────────────────┘
       │
       ▼
  {root}/.plm/
       ├── targets.json
       ├── marketplaces.json
       ├── imports.json
       └── cache/
            ├── plugins/
            └── marketplaces/

  ※ Personal 配置（~/.codex 等）は別経路で HOME を参照（変更しない）
```

上の図は、「どのコマンドを叩いても、PLM の状態ファイルは同じルート解決を通る」という全体像です。以降の各セクションは、この流れの中で出てくる技術を一つずつ掘り下げます。

---

## `CONCEPT` 環境変数（`PLM_HOME` / `HOME`）

環境変数は、プログラムの外側（シェルや CI の設定）から内側へ渡す「名前付きの文字列」です。パスや設定をコードにハードコードせず、実行時に差し替えられるようにする仕組みだと考えてください。

この機能では特に次の 2 つが重要です。

- **`HOME`** — 利用者のホームディレクトリ。Linux / macOS では通常 `/home/ユーザー名` や `/Users/ユーザー名` です。未設定の `PLM_HOME` があるときのフォールバック先になります。
- **`PLM_HOME`** — PLM 専用の「状態ルート」を差し替えるための変数。設定すると、その値が `HOME` の代わりに使われ、`{PLM_HOME}/.plm/...` 配下にキャッシュや JSON が集まります。

シェルでの設定例は次のとおりです。

```bash
# 一時ルートで PLM を動かす（ホストの ~/.plm と混ぜない）
export PLM_HOME=/tmp/plm-test
plm target list
```

```
シェル / CI
   │  export PLM_HOME=/tmp/plm-test
   │  （プロセスの環境に文字列が載る）
   ▼
plm プロセス起動
   │
   ▼
EnvVar::get("PLM_HOME")  →  Some("/tmp/plm-test")
EnvVar::get("HOME")      →  Some("/home/alice")
```

> **メモ**: `PLM_HOME` 未設定（または空・空白のみ）のときは、実効パスは従来どおり `$HOME/.plm` です。破壊的変更は「`PLM_HOME` を設定している利用者」にだけ影響します。

> **注意**: ここでいう「差し替え」は **PLM の状態ルート** だけです。Codex / Cursor などの Personal 配置先は引き続き `HOME` を見ます。`PLM_HOME=/tmp/x` でも `~/.codex` は `/tmp/x/.codex` にはなりません。

---

## `CONCEPT` 案 A（HOME 代替セマンティクス）

「`PLM_HOME` をどう解釈するか」には設計上の選択肢があります。本 Issue で採用するのは **案 A: HOME 代替** です。

案 A では、`PLM_HOME` は「ホームディレクトリそのものの代わり」です。プログラムはまず `{root}/.plm` を組み立てるので、ルートが `/tmp/plm-test` なら状態は `/tmp/plm-test/.plm` に置かれます。Cargo の `CARGO_HOME` のように「すでに `.cargo` 相当のディレクトリを直接指す」案 B とは違います。

```
案 A（採用）                         案 B（不採用・CARGO_HOME 型）
─────────────                       ──────────────────────────
PLM_HOME=/tmp/plm-test              PLM_HOME=/tmp/plm-test/.plm
         │                                   │
         ▼                                   ▼
  /tmp/plm-test/.plm/...              /tmp/plm-test/.plm/...
  （ルートの下に .plm を足す）         （変数が既に .plm を含む想定）
```

実装・ドキュメント・テストはすべて案 A に揃えます。将来の `config.toml`（#333）も、同じ `PlmPaths` に載せられる形を想定しています。

---

## `LANGUAGE` Rust の `PathBuf` とパス結合

`PathBuf` は、ファイルシステムのパスを表す所有型（所有権を持つ文字列バッファに近いもの）です。生の `String` で `"/home/a" + "/.plm"` のように連結すると、区切り文字の過不足や OS 差で壊れやすいのに対し、`PathBuf` は `join` で安全に部品を足せます。

この機能では、環境変数から得た文字列を `PathBuf::from(...)` でパスにし、`plm_dir()` や `targets_json()` などが `join` で `.plm` やファイル名を積み上げます。

```rust
use std::path::PathBuf;

let root = PathBuf::from("/tmp/plm-test");
let plm_dir = root.join(".plm");                    // /tmp/plm-test/.plm
let targets = plm_dir.join("targets.json");         // .../targets.json
let plugins = plm_dir.join("cache").join("plugins"); // .../cache/plugins
```

```
PathBuf::from("/tmp/plm-test")
        │
        ▼ join(".plm")
   /tmp/plm-test/.plm
        │
        ├─ join("targets.json")      → .../targets.json
        ├─ join("marketplaces.json") → .../marketplaces.json
        └─ join("cache").join("plugins")
                                     → .../cache/plugins
```

> **メモ**: `join` の引数が絶対パスだと、左側が捨てられて右側だけになる、という Rust のルールがあります。本機能では相対の部品（`.plm` やファイル名）だけを足すので、その落とし穴には通常当たりません。

### `PathBuf::from` / `join` / `is_absolute` / `display`

```
PathBuf::from(s: impl AsRef<OsStr>) -> PathBuf
path.join(path: impl AsRef<Path>) -> PathBuf
path.is_absolute() -> bool
path.display() -> Display
```

| 操作 | 役割 |
|------|------|
| `from` | 文字列からパスを作る |
| `join` | 子パスを足した新しい `PathBuf` を返す（元は変更しない） |
| `is_absolute` | 先頭が `/`（Unix）などで始まる絶対パスかどうか |
| `display` | エラーメッセージ用に人間が読める形で表示する |

---

## `CONCEPT` 絶対パスと相対パス

**絶対パス**は、ファイルシステムの頂点（Unix では `/`）から一意に場所を指します。例: `/home/alice`、`/tmp/plm-test`。

**相対パス**は、「今いる場所」からの相対です。例: `relative/path`、`./data`、`../foo`。同じ文字列でも、カレントディレクトリが変わると別の場所を指してしまいます。

本機能では、`PLM_HOME` / `HOME` のどちらも **絶対パスでなければならない** と決めます（FR-5）。状態ルートが実行時のカレントディレクトリに引きずられると、同じコマンドでも保存先がぶれ、デバッグが困難になるためです。

```
OK（絶対）          NG（相対）
──────────          ──────────
/tmp/plm-test       relative/path
/home/alice         ./plm-home
                    ../somewhere
```

```rust
let path = PathBuf::from(raw.trim());
if !path.is_absolute() {
    return Err(PlmError::General(format!(
        "PLM_HOME/HOME must be an absolute path, got: {}",
        path.display()
    )));
}
```

> **注意**: 「相対っぽく見えるが絶対」というケースはほぼありません。迷ったら `is_absolute()` の結果を信じ、拒否メッセージをそのまま利用者に返します。

---

## `LIBRARY` `EnvVar`（PLM の環境変数ラッパ）

`std::env::var("HOME")` を生で呼ぶと、未設定は `Err`、設定済みは `Ok(String)` です。一方 PLM には、空文字を「未設定と同じ」とみなす既存のラッパ `EnvVar::get` があります。`plm_root()` はこのラッパを使い、さらに **前後の空白を trim したうえで空なら無効** というルールを足します（FR-4）。

こうすると、「変数が無い」「空文字」「空白だけ」がすべて同じフォールバック経路に流れ、呼び出し側がバラバラの判定を書かずに済みます。

```
EnvVar::get("PLM_HOME")
        │
        ▼
   Option<String>
        │
        ▼ filter(|s| !s.trim().is_empty())
   有効な非空文字だけ残る
        │
        ▼ or_else(|| EnvVar::get("HOME").filter(...))
   PLM_HOME 無効なら HOME を試す
```

```rust
let raw = EnvVar::get("PLM_HOME")
    .filter(|s| !s.trim().is_empty())
    .or_else(|| EnvVar::get("HOME").filter(|s| !s.trim().is_empty()))
    .ok_or_else(|| {
        PlmError::General(
            "PLM_HOME and HOME environment variables are not set or empty".to_string(),
        )
    })?;
```

> **メモ**: バグの本体は、一部モジュールが `EnvVar` を使わず `std::env::var("HOME")` だけを見ていたことです。`PLM_HOME` を設定しても無視される経路が残っていました。今回はすべて `PlmPaths` 経由に寄せます。

> **注意**: `EnvVar::get` 自体の空文字扱いに加え、`plm_root` 側で `trim` 後の空白のみも無効にします。`"  "` のような値は HOME へ落ちます。

---

## `LANGUAGE` `Option`・`filter`・`or_else`・`trim`

`plm_root()` の核は、Rust 標準の小さな部品の組み合わせです。初めて読む人向けに、役割だけ押さえます。

**`Option<T>`** は「値がある（`Some`）」か「無い（`None`）」を表します。環境変数が取れたかどうか、その直後のフィルタ結果などに使います。

**`filter`** は、`Some` の中身が条件を満たさなければ `None` に落とします。ここでは「trim 後に空でない」ことが条件です。

**`or_else`** は、左側が `None` のときだけクロージャを評価して別の `Option` を足します。`PLM_HOME` が無効なら `HOME` を試す、というフォールバックそのものです。

**`trim`** は文字列の前後空白を除いたビューを返します。`"  /tmp/x  "` のように誤って空白が入っても、パス化の前に整えられます（空になる場合は無効扱い）。

```
Some("  ")  ──filter(!trim.is_empty)──►  None
Some("/tmp/x") ──filter──►  Some("/tmp/x")
None ──or_else(HOME)──►  HOME 側の Option
```

```rust
.some_value
    .filter(|s| !s.trim().is_empty())  // 無効なら None
    .or_else(|| alternative);            // None のときだけ代替
```

> **メモ**: `ok_or_else` は「`Option` を `Result` に変える」変換です。両方とも無効なら、ここで初めてエラーになります。

---

## `LANGUAGE` `Result`・`?`・`PlmError`・`map_err`

Rust では、失敗しうる処理の戻り値に `Result<T, E>` を使います。成功は `Ok(値)`、失敗は `Err(エラー)` です。PLM では共通のエラー型として `PlmError` があり、クレート内の `Result<T>` は多くの場合 `Result<T, PlmError>` を指します。

`plm_root()` は共有リゾルバなので、ドメイン固有のバリアント（`TargetRegistry` や `Cache`）ではなく、実在する **`PlmError::General`** を返します。`PlmError::Config` は存在しないため使いません。呼び出し側が自分の文脈に合わせて `map_err` で載せ替えます。

`?` 演算子は、「`Err` なら即座に関数から返す。`Ok` なら中身を取り出す」糖衣です。ネストした `match` を減らせます。

```
PlmPaths::new()
    │  内部で plm_root() → Err(PlmError::General(...))
    ▼
呼び出し側
    │  .map_err(|e| PlmError::TargetRegistry(e.to_string()))
    ▼
Err(PlmError::TargetRegistry(...))   ← ドメイン文脈を保持
```

```rust
// TargetRegistry: General → TargetRegistry
let paths = crate::env::PlmPaths::new()
    .map_err(|e| PlmError::TargetRegistry(e.to_string()))?;

// PackageCache: General → Cache
let paths = crate::env::PlmPaths::new()
    .map_err(|e| PlmError::Cache(e.to_string()))?;

// MarketplaceConfig: PlmError → String（C-4 で型を維持）
let paths = crate::env::PlmPaths::new().map_err(|e| e.to_string())?;
```

| 呼び出し側 | `map_err` 後の型 | 理由 |
|------------|------------------|------|
| `TargetRegistry::new` | `PlmError::TargetRegistry` | ターゲット領域のエラーとして報告 |
| `ImportRegistry::new` | `PlmError::ImportRegistry` | インポート領域のエラーとして報告 |
| `PackageCache` / `MarketplaceRegistry` | `PlmError::Cache` | キャッシュ領域の既存バリアントに合わせる |
| `MarketplaceConfig::load` | `String` | 公開 API の `Result<_, String>` を壊さない（C-4） |

> **注意**: 共有層で最初から `TargetRegistry` を返すと、キャッシュ側が不自然なバリアントを抱えることになります。`General` → 呼び出し側マップ、が意図された分担です。

---

## `CONCEPT` `plm_root()` — 状態ルートの単一リゾルバ

`plm_root()` は、「PLM 状態の親ディレクトリはどこか？」という問いに答える関数です。優先順位とバリデーションを一箇所に閉じ込めることで、5 つの利用箇所が同じ答えを共有します。

解決の流れは次のとおりです。

1. `PLM_HOME` を読む（空・空白のみは無効）
2. 無効なら `HOME` を読む（同様）
3. どちらも無効なら `Err`
4. 得られた文字列を trim して `PathBuf` にし、絶対パスでなければ `Err`
5. 絶対パスなら `Ok(path)`

```
         plm_root()
              │
              ▼
     PLM_HOME 有効？ ──Yes──► 候補
              │No
              ▼
       HOME 有効？ ──Yes──► 候補
              │No
              ▼
            Err
              │
     候補が絶対パス？ ──No──► Err
              │Yes
              ▼
             Ok
```

### `plm_root`

```
plm_root() -> Result<PathBuf>
```

環境変数から PLM 状態ルートを解決して返します。引数はありません。副作用は環境の読み取りだけです。

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| （なし） | — | プロセス環境の `PLM_HOME` / `HOME` を参照 |

戻り値は、成功時に絶対パスの `PathBuf`、失敗時に `PlmError::General` を含む `Err` です。

可視性は `pub(crate)` です。バイナリクレート内部でのみ使い、外部クレートへは公開しません。

> **メモ**: パスの「計算」（`.plm` を足すなど）は `PlmPaths` の仕事です。`plm_root` はルート文字列の正規化と検証に専念します。

---

## `CONCEPT` `PlmPaths` — 状態パスの値オブジェクト

`PlmPaths` は、「ルートが決まれば、あとの JSON やキャッシュの場所は機械的に決まる」という事実を型にしたものです。中身は実質 `root: PathBuf` 一つで、メソッドが決まった相対位置を返します。このような、識別子ではなく値そのもので意味を持つ小さな型を、ここでは **値オブジェクト** と呼びます。

本番では `PlmPaths::new()` が `plm_root()` を呼びます。テストでは環境変数をいじらず、`with_root(temp_dir)` でルートを注入できます。これにより、パス結合の正しさは並列安全に検証し、環境変数の優先順位テストだけを必要最小限にできます（NFR-4）。

```
PlmPaths { root }
    │
    ├─ plm_dir()               → {root}/.plm
    ├─ targets_json()          → {root}/.plm/targets.json
    ├─ marketplaces_json()     → {root}/.plm/marketplaces.json
    ├─ imports_json()          → {root}/.plm/imports.json
    ├─ plugins_cache_dir()     → {root}/.plm/cache/plugins
    └─ marketplaces_cache_dir()→ {root}/.plm/cache/marketplaces
```

```rust
impl PlmPaths {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self { root: plm_root()? })
    }

    pub(crate) fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn plm_dir(&self) -> PathBuf {
        self.root.join(".plm")
    }

    pub(crate) fn targets_json(&self) -> PathBuf {
        self.plm_dir().join("targets.json")
    }
    // marketplaces_json / imports_json / plugins_cache_dir /
    // marketplaces_cache_dir も同様に join で組み立てる
}
```

### `PlmPaths::new`

```
PlmPaths::new() -> Result<PlmPaths>
```

環境変数からルートを解決して構築します。失敗時は `plm_root` と同じ `PlmError::General` 系です。

### `PlmPaths::with_root`

```
PlmPaths::with_root(root: PathBuf) -> PlmPaths
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `root` | `PathBuf` | テストや特殊用途で注入する状態ルート（`~` 相当） |

環境を読まないため、ユニットテストの主戦力になります。絶対パス検証は `plm_root` 側の責務なので、`with_root` に相対パスを渡すと結合結果も相対になり得ます。テストでは `TempDir` など絶対パスを渡してください。

> **メモ**: 将来 #333 で `config.toml` パスが必要になっても、同じ構造体に `config_toml()` を足せば、直読みの散在を増やさずに済みます（C-6）。

---

## `CONCEPT` 5 つのレジストリ／キャッシュと統一入口

「レジストリ」という言葉は、ここでは「ディスク上の状態を読み書きするコンポーネント」程度の意味で十分です。本変更の対象は次の 5 つです。

| コンポーネント | 役割のイメージ | 使うアクセサ |
|----------------|----------------|--------------|
| `PackageCache` | プラグイン本体のキャッシュ | `plugins_cache_dir()` |
| `MarketplaceRegistry` | マーケットプレイス用キャッシュ | `marketplaces_cache_dir()` |
| `MarketplaceConfig` | マーケットプレイス一覧 JSON | `marketplaces_json()` |
| `TargetRegistry` | ターゲット環境の登録 JSON | `targets_json()` |
| `ImportRegistry` | インポート履歴 JSON | `imports_json()` |

修正前は、上 2 つが `EnvVar` で `PLM_HOME`→`HOME` を見ていた一方、下 3 つが `std::env::var("HOME")` だけを見ていました。修正後は、いずれも `PlmPaths::new()` から入ります。

```rust
// MarketplaceConfig（エラー型は String のまま）
pub fn load() -> Result<Self, String> {
    let paths = crate::env::PlmPaths::new().map_err(|e| e.to_string())?;
    Self::load_from(paths.marketplaces_json())
}

// TargetRegistry
pub fn new() -> Result<Self> {
    let paths = crate::env::PlmPaths::new()
        .map_err(|e| PlmError::TargetRegistry(e.to_string()))?;
    Ok(Self {
        config_path: paths.targets_json(),
        state: State::Idle,
        config: None,
    })
}
```

> **注意**: `with_cache_dir` / `with_path` / `load_from` といった、すでにパスを外から注入できる API は壊しません（FR-8）。デフォルト構築経路だけを `PlmPaths` に寄せます。

---

## `CONCEPT` Personal 配置とスコープ外コード

PLM は「プラグインをどこに置くか」と「PLM 自身のメタデータをどこに置くか」を分けて考えます。

- **PLM 状態** — `{PLM_HOME ?? HOME}/.plm/...`（本 Issue の対象）
- **Personal 配置** — 各 AI ツールの利用者ディレクトリ（例: `~/.codex`）。こちらは **常に本物の `HOME`** を見ます

そのため `src/target/core/paths.rs` の `home_dir()` や、`src/plugin/cache/cleanup.rs` は変更しません（C-2）。回帰テストでは、`PLM_HOME ≠ HOME` でも `home_dir()` が `HOME` を返すことを確認します。

```
PLM_HOME=/tmp/plm-test
HOME=/home/alice

PLM 状態     → /tmp/plm-test/.plm/...     （差し替えられる）
Personal     → /home/alice/.codex/...     （差し替えられない）
```

> **メモ**: 「全部を `PLM_HOME` 配下に寄せる」のではなく、「PLM が管理する状態だけを寄せる」のが仕様です。混同すると UC-3 を壊します。

---

## `LANGUAGE` 可視性 `pub(crate)` と Feature ベース配置

Rust の `pub` はクレート外にも見える公開、`pub(crate)` は同じクレート内だけ、非公開はモジュール内だけ、という段階があります。PLM はバイナリクレートなので、内部ヘルパを `pub` にしすぎる必要はありません。`plm_root` と `PlmPaths` はどちらも `pub(crate)` です。

配置場所は `src/env.rs` です（C-3）。環境変数と密接な Feature に置き、汎用パス拡張の `path_ext` には置きません。レイヤー（domain / infrastructure）ではなく、**Feature（関心ごと）単位でモジュールをまとめる** のがこのリポジトリの方針です。

テストは本体と同ファイルに書かず、`src/env_test.rs` のように `*_test.rs` へ分離します（C-5）。

---

## `TOOL` TDD（Red → Green → Refactor）

TDD は、実装の前に「望む振る舞いをテストで先に書く」進め方です。本機能は優先順位・フォールバック・バリデーションを含む純粋ロジックなので、TDD が特に向いています（NFR-2）。

```
  Red                         Green                        Refactor
  ───                         ─────                        ────────
  失敗するテストを書く         最小限の実装で通す            重複を削り命名を整える
  （まだ plm_root が無い等）   （仮実装でもよい）            （テストは緑のまま）
         │                         │                            │
         └─────────────────────────┴────────────────────────────┘
                         小さなステップで繰り返す
```

おすすめの順序は次のとおりです。

1. `PlmPaths::with_root` のパス計算テストを先に書く（環境変数不要）
2. `plm_root` の優先順位・エラー系テストを書く（必要なら `--test-threads=1`）
3. 実装を足して緑にする
4. 各レジストリの `new` / `load` を `PlmPaths` に差し替え、回帰テストで確認する

> **メモ**: Red を確認せずに実装へ進むと、「テストがそもそも間違っている」ことに気づけません。一度失敗を目で見てから Green に進みます。

---

## `TOOL` ユニットテストと環境変数の並列競合

`cargo test` はデフォルトで複数スレッドでテストを並列実行します。`std::env::set_var` / `remove_var` は **プロセス全体の環境** を書き換えるため、テスト A が `PLM_HOME` を付けている最中にテスト B が読むと、意図しない結果になります。

対策は二段構えです。

1. **パス計算は `with_root` で十分** — 環境を触らないので並列安全
2. **`plm_root` の環境操作テストは最小限** — `cargo test plm_root -- --test-threads=1` で直列化（または `serial_test`）

```bash
# 並列でよい
cargo test plm_paths

# 環境を触るものだけ直列
cargo test plm_root -- --test-threads=1
```

テスト用の一時ディレクトリには、既存依存の `tempfile::TempDir` を使います。新規クレート依存は追加しません（NFR-1）。

> **注意**: 環境変数テストを増やしすぎると、CI でのフレーク（たまに落ちる）の温床になります。「優先順位・フォールバック・エラー」の代表ケースに絞ってください。

---

## `TOOL` `cargo test` / `clippy` と検証の位置づけ

実装後の機械検証の中心は次です。

```bash
cargo test
cargo clippy -- -D warnings
```

`-D warnings` は警告をエラー扱いにします。CI で有効なので、ローカルでも同じ厳しさで見ます。`PlmPaths` に `Default` が必要になった場合は、`new()` との関係を整理してから足します。

手動検証では、実際に `PLM_HOME` を付けてコマンドを叩き、JSON が `/tmp/.../.plm/` に出ること、未設定時は `$HOME/.plm` に戻ること、Personal 配置が `$HOME` 側のままであることを確認します。

---

## `CONCEPT` 後方互換と破壊的変更の境界

「後方互換」とは、今まで動いていた使い方が、アップデート後も同じ結果になることです。本機能では次が約束です。

- `PLM_HOME` **未設定** → 実効パスは従来どおり `$HOME/.plm`（NFR-3）
- `PLM_HOME` **設定済み** → 以前は一部だけ無視されていたものが、すべて新ルートに揃う（ここが修正＝意図した変化）

つまり破壊的変更の影響範囲は、「すでに `PLM_HOME` を付けて使っていた人」に限定されます。その人たちにとって、以前は設定 JSON がホーム側に残っていた可能性があり、修正後はキャッシュと同じルートに集まります。

```
未設定ユーザー          PLM_HOME 設定ユーザー
──────────────          ────────────────────
パス不変（互換）         分裂が解消（意図した修正）
$HOME/.plm/...          $PLM_HOME/.plm/... に統一
```

---

## 用語集

| 用語 | 意味 |
|------|------|
| `PLM` | 複数の AI 開発環境向けプラグインを管理する CLI ツール |
| `PLM_HOME` | PLM 状態ルートを差し替える環境変数（案 A では HOME の代替） |
| `HOME` | 利用者のホームディレクトリを示す環境変数。フォールバック先 |
| 環境変数 | プロセスに渡される名前付きの設定文字列 |
| `EnvVar` | 空文字を無効扱いする PLM の環境変数取得ラッパ |
| `plm_root()` | `PLM_HOME`→`HOME` で状態ルートを解決する関数 |
| `PlmPaths` | `{root}/.plm/...` 配下のパスを返す値オブジェクト |
| 値オブジェクト | 識別子ではなく値の内容で意味を持つ小さな型 |
| `PathBuf` | Rust 標準の所有パス型。`join` で部品を安全に結合する |
| 絶対パス | `/` から始まるなど、位置が一意に決まるパス |
| 相対パス | カレントディレクトリに依存するパス。本機能では拒否 |
| `Result` / `Ok` / `Err` | 成功または失敗を表す戻り値の型 |
| `PlmError` | PLM 共通のエラー列挙。共有リゾルバは `General` を使う |
| `map_err` | `Err` の中身だけ別の型・文脈に変換する操作 |
| `?` | `Err` なら早期 return、`Ok` なら中身を取り出す演算子 |
| `Option` / `Some` / `None` | 値の有無を表す型 |
| `filter` / `or_else` | `Option` を絞り込む・代替を足すメソッド |
| `trim` | 文字列の前後空白を除く |
| `pub(crate)` | 同じクレート内だけに見える可視性 |
| レジストリ | ここでは状態 JSON / キャッシュを管理するコンポーネント |
| `PackageCache` | プラグインキャッシュ（`cache/plugins`） |
| `MarketplaceRegistry` | マーケットプレイスキャッシュ（`cache/marketplaces`） |
| `MarketplaceConfig` | `marketplaces.json` の読み書き。エラー型は `String` |
| `TargetRegistry` | `targets.json` の管理 |
| `ImportRegistry` | `imports.json` の管理 |
| Personal 配置 | Codex / Cursor 等の利用者ディレクトリ配置（HOME 固定） |
| 案 A（HOME 代替） | `PLM_HOME` をホーム相当とし、その下に `.plm` を足す解釈 |
| TDD | 失敗するテスト→最小実装→リファクタのサイクル |
| フォールバック | 第一候補が無効なとき第二候補へ切り替えること |
| 後方互換 | 既存のデフォルト利用が同じ結果のままであること |
| `TempDir` | テスト用の一時ディレクトリ（絶対パス） |
| Feature ベース | 関心ごと単位でモジュールをまとめる構成方針 |
| `*_test.rs` | 本体と分離したテストファイルの命名慣例 |
