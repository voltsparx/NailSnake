# NailSnake — cross-platform install helpers (Unix: Linux, macOS, Git Bash)
#
# Usage:
#   make build          # release binary
#   make debug          # debug binary
#   make install        # install binary + man page
#   make install-man    # install man page only (registers with man-db on Linux)
#   make uninstall      # remove installed files
#   make test           # run tests
#   make check          # cargo check
#   make clean          # clean build artifacts
#   make run            # run with cargo (args: ARGS="...")

PREFIX      ?= /usr/local
BINDIR      ?= $(PREFIX)/bin
MANDIR      ?= $(PREFIX)/share/man
MAN1DIR     ?= $(MANDIR)/man1
CARGO       ?= cargo
BIN         := nailsnake
MAN_SRC     := man/nailsnake.1
MAN_PAGE    := $(MAN1DIR)/$(BIN).1

.PHONY: all build debug install install-bin install-man uninstall test check clean run dist

all: build

build:
	$(CARGO) build --release

debug:
	$(CARGO) build

check:
	$(CARGO) check

test:
	$(CARGO) test

clean:
	$(CARGO) clean

run:
	$(CARGO) run -- $(ARGS)

install: install-bin install-man

install-bin: build
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/$(BIN) $(DESTDIR)$(BINDIR)/$(BIN)

install-man:
	install -d $(DESTDIR)$(MAN1DIR)
	install -m 644 $(MAN_SRC) $(DESTDIR)$(MAN_PAGE)
	@$(MAKE) mandb

mandb:
	@if command -v mandb >/dev/null 2>&1; then \
		mandb -q $(DESTDIR)$(MANDIR) 2>/dev/null || mandb -q; \
		echo "man-db cache updated — run: man nailsnake"; \
	elif command -v makewhatis >/dev/null 2>&1; then \
		makewhatis $(DESTDIR)$(MAN1DIR) 2>/dev/null || true; \
		echo "man whatis cache updated — run: man nailsnake"; \
	else \
		echo "man page installed to $(DESTDIR)$(MAN_PAGE)"; \
		echo "Run: man -M $(DESTDIR)$(MAN1DIR) nailsnake"; \
	fi

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BIN) $(DESTDIR)$(MAN_PAGE)
	@if command -v mandb >/dev/null 2>&1; then mandb -q 2>/dev/null || true; fi

# Package a release tarball (requires cargo)
dist: build
	$(eval VERSION := $(shell $(CARGO) metadata --no-deps --format-version 1 2>/dev/null | sed -n 's/.*"version":"\([^"]*\)".*/\1/p'))
	$(eval DIST_DIR := $(BIN)-$(VERSION))
	mkdir -p $(DIST_DIR)
	cp -r target/release/$(BIN) README.md LICENSE man/ scripts/ $(DIST_DIR)/
	tar czf $(DIST_DIR).tar.gz $(DIST_DIR)
	rm -rf $(DIST_DIR)
	@echo "Created $(DIST_DIR).tar.gz"
