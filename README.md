# plm

Plugin Manager CLI (Rust)

## 第三者ライセンス一覧

`cargo-about` で依存クレートのライセンス一覧を生成し、リポジトリに同梱できます。

生成物:

- `THIRD_PARTY_LICENSES.md`

再生成:

```bash
export CARGO_HOME=$PWD/.cargo-home
export CARGO_TARGET_DIR=$PWD/.target
export PATH=$CARGO_HOME/bin:$PATH

cargo install --locked cargo-about

cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs
```

