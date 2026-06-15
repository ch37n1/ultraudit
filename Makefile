PACK_VERSION ?= 0.2.0
ULTRAUDIT_PATH ?= $(HOME)/.ultraudit
INSTALL_BIN ?= $(HOME)/.local/bin

.PHONY: install
install:
	cargo build --release
	install -d "$(INSTALL_BIN)"
	install -m 0755 target/release/ultraudit "$(INSTALL_BIN)/ultraudit"
	install -d "$(ULTRAUDIT_PATH)/packs/$(PACK_VERSION)"
	cp -R "packs/$(PACK_VERSION)/." "$(ULTRAUDIT_PATH)/packs/$(PACK_VERSION)/"
	@if command -v codex >/dev/null 2>&1; then \
		printf 'codex: found at %s\n' "$$(command -v codex)"; \
	else \
		printf 'warning: codex was not found in PATH; configure .audit/agents or install Codex before real agent runs.\n'; \
	fi
	@case ":$$PATH:" in \
		*:"$(INSTALL_BIN)":*) ;; \
		*) printf 'note: add %s to PATH to run ultraudit directly.\n' "$(INSTALL_BIN)";; \
	esac
	@printf 'installed ultraudit: %s/ultraudit\n' "$(INSTALL_BIN)"
	@printf 'installed prompt pack: %s/packs/%s\n' "$(ULTRAUDIT_PATH)" "$(PACK_VERSION)"
