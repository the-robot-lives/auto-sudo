INSTALL_BIN := $(HOME)/.local/bin
INSTALL_DIR := $(HOME)/.local/share/auto-sudo
CONFIG_DIR := $(HOME)/.config/auto-sudo
INSTALL_FILE := auto-sudo.zsh
SOURCE_LINE := source "$(INSTALL_DIR)/$(INSTALL_FILE)"
ZSHRC := $(HOME)/.zshrc
RUST_DIR := rust

.PHONY: compile test install

compile:
	cd $(RUST_DIR) && cargo build --release

test:
	cd $(RUST_DIR) && cargo test

install: compile
	@mkdir -p $(INSTALL_BIN) $(INSTALL_DIR) $(CONFIG_DIR)
	@install -m 755 $(RUST_DIR)/target/release/auto-sudo $(INSTALL_BIN)/auto-sudo
	@src=$$(realpath $(INSTALL_FILE)); dst=$$(realpath $(INSTALL_DIR)/$(INSTALL_FILE) 2>/dev/null); \
	if [ "$$src" = "$$dst" ]; then \
		echo "auto-sudo: already installed (same file) — skipping"; \
	else \
		install -m 644 $(INSTALL_FILE) $(INSTALL_DIR)/$(INSTALL_FILE); \
	fi
	@if [ -f "$(CONFIG_DIR)/config.yaml" ]; then \
		echo "✓ Existing config left unchanged: $(CONFIG_DIR)/config.yaml"; \
	else \
		install -m 644 config.example.yaml $(CONFIG_DIR)/config.yaml; \
		echo "✓ Installed default config: $(CONFIG_DIR)/config.yaml"; \
	fi
	@if grep -qF '$(INSTALL_FILE)' $(ZSHRC) 2>/dev/null; then \
		echo "✓ Already sourced in ~/.zshrc"; \
	else \
		echo '' >> $(ZSHRC); \
		echo '# auto-sudo: generated sudo wrappers' >> $(ZSHRC); \
		echo '$(SOURCE_LINE)' >> $(ZSHRC); \
		echo "✓ Added source line to ~/.zshrc"; \
	fi
	@echo "Installed: auto-sudo → $(INSTALL_BIN)/auto-sudo"
	@echo "Run: source ~/.zshrc   (or open a new shell)"
