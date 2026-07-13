# Issue #347: 再帰ファイルコピー実装の統一

## 背景

ディレクトリ/ファイルコピーが3系統で独立実装されており、意味論（上書き時の挙動）が食い違っていた。

| 実装 | 場所 | 意味論 |
| --- | --- | --- |
| `FileSystem::copy_dir` | `src/fs.rs` | **マージ**（既存にコピー） |
| `PathExt::copy_dir_to` | `src/path_ext.rs` | **置換**（ターゲット削除→コピー） |
| `copy_file` (private) | `src/component/convert.rs` | 単発コピー |

デプロイ経路（`component/deployment*`）は `PathExt` 経由で `std::fs` を直叩きしており、注入可能な `FileSystem` 抽象が使われていなかった。

## 方針

### 1. コピー操作の意味論を型で分離

`FileSystem` trait に以下の2メソッドを明示的に定義する。

| メソッド | 意味論 | 用途 |
| --- | --- | --- |
| `copy_dir` | マージ | 既存ディレクトリに上書きコピー |
| `copy_dir_replace` | 置換 | 宛先を削除してから再帰コピー |

`copy_file` は既存のまま（親ディレクトリ自動作成・上書き）。

### 2. 重複実装の委譲

- `PathExt::copy_dir_to` → `RealFs.copy_dir_replace`
- `PathExt::copy_file_to` → `RealFs.copy_file`
- `convert.rs` の private `copy_file` → `RealFs.copy_file`

`PathExt` は後方互換の薄いラッパーとして残す（`read_dir_entries` / `join_or` はそのまま）。

### 3. デプロイ経路への FileSystem 注入

`sync` モジュールと同様のパターンを採用する。

```rust
pub fn execute(&self) -> Result<DeploymentOutput> {
    self.execute_with_fs(&RealFs)
}

pub fn execute_with_fs(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
    // deploy_* に fs を渡す
}
```

- Skill 配置: `fs.copy_dir_replace`（既存テスト `test_execute_skill_replaces_existing_directory` の挙動を維持）
- ファイル配置: `fs.copy_file`
- frontmatter 除去: `fs.read_to_string` + `atomic_write`（アトミック書き込みは別関心事として維持）

### 4. intent.rs の更新

`FileOperation::CopyFile` / `CopyDir` の実行を `PathExt` から `FileSystem` メソッドへ切り替える。

## 変更ファイル

| ファイル | 変更内容 |
| --- | --- |
| `src/fs.rs` | `copy_dir_replace` 追加 |
| `src/fs/mock.rs` | `copy_dir_replace` 実装 |
| `src/fs_test.rs` | 置換コピーのテスト追加 |
| `src/path_ext.rs` | RealFs へ委譲 |
| `src/component/convert.rs` | RealFs.copy_file へ委譲 |
| `src/component/deployment.rs` | `execute_with_fs` 導入 |
| `src/component/deployment/hook_deploy.rs` | fs 引数を受け取る |
| `src/plugin/lifecycle/intent.rs` | fs メソッド使用 |
| `src/component/deployment_test.rs` | MockFs によるユニットテスト追加 |

## テスト戦略

1. `fs_test.rs`: `copy_dir_replace` が余剰ファイルを削除することを MockFs で検証
2. `deployment_test.rs`: `execute_with_fs(&MockFs)` で Skill 置換コピーを tempdir なしで検証
3. 既存の `path_ext_test.rs` / `deployment_test.rs` が引き続きパスすることを確認

## 期待効果

- コピー挙動の一貫性（残存ファイル・上書き規則）が保証される
- デプロイロジックを MockFs でテスト可能になる
- 呼び出し側が `copy_dir` vs `copy_dir_replace` で意図を明示できる
