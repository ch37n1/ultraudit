PACK_VERSION ?= 0.2.0
ULTRAUDIT_PATH ?= $(HOME)/.ultraudit
INSTALL_BIN ?= $(HOME)/.local/bin
PACK_STAGE ?= $(ULTRAUDIT_PATH)/packs/.staging-$(PACK_VERSION)

.PHONY: install
install:
	cargo build --release
	install -d "$(INSTALL_BIN)"
	install -m 0755 target/release/uat "$(INSTALL_BIN)/uat"
	install -d "$(ULTRAUDIT_PATH)/packs"
	rm -rf "$(PACK_STAGE)"
	install -d "$(PACK_STAGE)"
	cp -R "packs/$(PACK_VERSION)/." "$(PACK_STAGE)/"
	rm -rf "$(ULTRAUDIT_PATH)/packs/$(PACK_VERSION)"
	mv "$(PACK_STAGE)" "$(ULTRAUDIT_PATH)/packs/$(PACK_VERSION)"
	@if command -v codex >/dev/null 2>&1; then \
		printf 'codex: found at %s\n' "$$(command -v codex)"; \
	else \
		printf 'warning: codex was not found in PATH; configure .audit/agents or install Codex before real agent runs.\n'; \
	fi
	@case ":$$PATH:" in \
		*:"$(INSTALL_BIN)":*) ;; \
		*) printf 'note: add %s to PATH to run uat directly.\n' "$(INSTALL_BIN)";; \
	esac
	@printf 'installed uat: %s/uat\n' "$(INSTALL_BIN)"
	@printf 'installed prompt pack: %s/packs/%s\n' "$(ULTRAUDIT_PATH)" "$(PACK_VERSION)"
