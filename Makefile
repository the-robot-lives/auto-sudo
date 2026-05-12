INSTALL_DIR := $(HOME)/.local/bin

.PHONY: compile test install

compile:
	@true

test:
	@true

install:
	@mkdir -p $(INSTALL_DIR)
	cp vim.zsh $(INSTALL_DIR)/auto-sudo-vim
	chmod +x $(INSTALL_DIR)/auto-sudo-vim
