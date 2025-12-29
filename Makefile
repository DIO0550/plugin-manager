.PHONY: build release check fmt lint test ci clean audit help

# デフォルト: ヘルプ表示
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build    - 開発ビルド"
	@echo "  release  - リリースビルド"
	@echo "  check    - 高速コンパイル確認"
	@echo "  fmt      - フォーマット"
	@echo "  lint     - リント (clippy)"
	@echo "  test     - テスト実行"
	@echo "  ci       - fmt + lint + test"
	@echo "  clean    - ビルド成果物削除"
	@echo "  audit    - 依存関係セキュリティ監査"

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

ci: fmt lint test

clean:
	cargo clean

audit:
	cargo deny check
