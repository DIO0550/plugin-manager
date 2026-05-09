#!/usr/bin/env bash
# 画面 Model が `Model` 名で型として残存していないかをチェック
#
# 共通: `(vis)?`（省略 / `pub` / `pub(crate)` / `pub(super)` 等）の visibility 修飾子を
#       任意で許容するため VIS='(pub(\([^)]*\))?[[:space:]]+)?' を共通変数として導入し、
#       パターン2/3/4 で利用。パターン7 (use 文) も同等の visibility 接頭辞を許容する。
#
# 検出対象 (すべて違反):
#   1. 画面 root (src/tui/manager/screens/<name>.rs) の pub use 文内に `Model` 出現
#      - 単一行・複数行両対応。`Model as <Alias>` も含めて NG
#   2. screens/<name>.rs の `(vis)? (struct|enum) Model\b`
#   3. screens/<name>/model.rs の `(vis)? (struct|enum) Model\b`
#   4. screens/** の `(vis)? type Model\b` 互換 alias (visibility 省略含む)
#   5. screens/** の `\bas Model\b` import alias
#   6. screens/** の `\bmodel::Model\b` 直書き型参照
#   7. screens/** の `use ... model::{ ..., Model, ... };` braced import
#      (単一行・複数行両対応、perl -0777 で実装)
#
# 除外対象:
#   - src/tui/manager/core/** (アプリ全体 Model は対象外)
#   - ローカル変数名 `let mut model = ...` / 関数名 `make_model` 等は対象外
#     (`Model` は実コード上の型としての使用のみを検出する)
#
# コメント行の誤検知対策:
#   - パターン2-6 は grep ベースのため、`// as Model` や `// model::Model` のような
#     コメント中の文字列も素朴には違反扱いになりうる。
#     そこで grep 出力 (`行番号:行内容`) から `//` で始まる行を除外する。
#   - パターン1/7 は `^\s*pub\s+use` / `^\s*use` の行頭アンカー付きで perl が
#     ステートメントを抽出するため、コメント行 (`// ...` 始まり) は元から拾わない。
#   - 行内コメント (`code; // comment as Model`) の除外は対象外
#     (実装コードに `as Model` を書いた時点で違反であり、その後ろに何があっても結論は変わらない)。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${ROOT}/src/tui/manager/screens"

if [[ ! -d "${TARGET_DIR}" ]]; then
  echo "ERROR: target directory not found: ${TARGET_DIR}" >&2
  exit 2
fi

violations=0

# パターン1: 画面 root の `pub use ... ;` 文内に裸 `Model` または `Model as Alias` が出現
# 画面 root = screens 直下の <name>.rs (深さ 1)
# `pub use ... ;` 文をセミコロン単位で抽出。単一行・複数行両対応。
# `Model as <Alias>` も含めて違反 (§4 で alias は禁止)。
while IFS= read -r file; do
  hits="$(perl -0777 -ne '
    while (m{^\s*pub\s+use\s+[^;]*?;}gms) {
      my $stmt = $&;
      if ($stmt =~ /\bModel\b/) {
        print $stmt, "\n---\n";
      }
    }
  ' "${file}")"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`Model\` exposed via pub use in ${file}" >&2
    printf '%s' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -maxdepth 1 -type f -name '*.rs')

# 共通: 任意 visibility 修飾子 (省略 / pub / pub(crate) / pub(super) 等) のパターン
# `(pub(\([^)]*\))?\s+)?` を許容することで、`type Model`, `pub(crate) type Model` 等もすべて拾う。
#
# 注: パターン2-6 は単語境界 `\b` を使うため、ERE (`-E`) ではなく PCRE (`-P`) を使用する。
#     ERE の `\b` は POSIX 仕様外で実装依存（GNU grep では word boundary、BSD/busybox 等では
#     バックスペース扱いの可能性）。PCRE では `\b` の意味が確定するため、検出の信頼性が高い。
VIS='(pub(\([^)]*\))?[[:space:]]+)?'

# コメント行除外フィルタ: grep -n 出力 (`行番号:行内容`) から
# `//` で始まる行内容を取り除く (パターン2-6 で共通利用)
COMMENT_FILTER='^[0-9]+:[[:space:]]*//'

# パターン2: 画面 root 直下の <name>.rs での `(vis)? (struct|enum) Model`
while IFS= read -r file; do
  hits="$(grep -nP "^[[:space:]]*${VIS}(struct|enum)[[:space:]]+Model\b" "${file}" 2>/dev/null | grep -vE "${COMMENT_FILTER}" || true)"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`struct/enum Model\` in ${file}" >&2
    printf '%s\n' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -maxdepth 1 -type f -name '*.rs')

# パターン3: 画面サブディレクトリの model.rs での `(vis)? (struct|enum) Model`
while IFS= read -r file; do
  hits="$(grep -nP "^[[:space:]]*${VIS}(struct|enum)[[:space:]]+Model\b" "${file}" 2>/dev/null | grep -vE "${COMMENT_FILTER}" || true)"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`struct/enum Model\` in screen module ${file}" >&2
    printf '%s\n' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -mindepth 2 -type f -name 'model.rs')

# パターン4: screens 配下の任意 .rs での `(vis)? type Model` 互換 alias (§4 で禁止)
# visibility 省略 (`type Model = ...`) や `pub(crate) type Model` も含む
while IFS= read -r file; do
  hits="$(grep -nP "^[[:space:]]*${VIS}type[[:space:]]+Model\b" "${file}" 2>/dev/null | grep -vE "${COMMENT_FILTER}" || true)"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`type Model\` compat alias in ${file}" >&2
    printf '%s\n' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -type f -name '*.rs')

# パターン5: screens 配下の任意 .rs での `as Model` import alias (§4 で禁止)
while IFS= read -r file; do
  hits="$(grep -nP '\bas[[:space:]]+Model\b' "${file}" 2>/dev/null | grep -vE "${COMMENT_FILTER}" || true)"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`as Model\` import alias in ${file}" >&2
    printf '%s\n' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -type f -name '*.rs')

# パターン6: screens 配下の任意 .rs での `model::Model` 直書き型参照
#  (`super::model::Model`, `crate::tui::manager::screens::installed::model::Model` 等を含む)
while IFS= read -r file; do
  hits="$(grep -nP '\bmodel::Model\b' "${file}" 2>/dev/null | grep -vE "${COMMENT_FILTER}" || true)"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: \`model::Model\` reference in ${file}" >&2
    printf '%s\n' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -type f -name '*.rs')

# パターン7: screens 配下の任意 .rs での `(vis)? use ... model::{...};` braced import 中の裸 Model
# `use super::model::{DetailAction, Model, Msg};` / `pub(crate) use ...::model::{Model};` など
# 単一行・複数行両対応 (perl -0777 + DOTALL)。visibility 修飾子も先頭で許容する。
while IFS= read -r file; do
  hits="$(perl -0777 -ne '
    while (m{^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+[^;]*?;}gms) {
      my $stmt = $&;
      next unless $stmt =~ /\bmodel\b/;          # model モジュール経由の use 文に限定
      if ($stmt =~ /\{[^}]*\bModel\b[^}]*\}/s) { # ブレース内に裸 Model
        print $stmt, "\n---\n";
      }
    }
  ' "${file}")"
  if [[ -n "${hits}" ]]; then
    echo "VIOLATION: braced \`Model\` import from model module in ${file}" >&2
    printf '%s' "${hits}" >&2
    violations=$((violations + 1))
  fi
done < <(find "${TARGET_DIR}" -type f -name '*.rs')

if (( violations > 0 )); then
  echo "" >&2
  echo "Found ${violations} violation(s). Rename to <Screen>ScreenModel." >&2
  echo "See docs/architecture/naming-conventions.md §4 for details." >&2
  exit 1
fi

echo "OK: no \`Model\` type usage in screens."
exit 0
