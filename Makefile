.PHONY: install dev bundle build build-noinstall deb-install release

DEB_DIR := src-tauri/target/release/bundle/deb
VER = $$(node -p "require('./package.json').version")
AUTOSTART := $(HOME)/.config/autostart/easy-vial.desktop

# The GUI and the daemon are the same binary; match the GUI only (no --daemon arg).
GUI_PATTERN := '(.*/)?easy-vial'
DAEMON_PATTERN := 'easy-vial --daemon'

install:
	pnpm install

# Closes running easy-vial GUI first (installed or previous dev); the daemon keeps running
dev:
	@if pgrep -xf $(GUI_PATTERN) >/dev/null; then \
	  echo "[easy-vial] closing running easy-vial..."; \
	  pkill -xf $(GUI_PATTERN); sleep 1; \
	  pgrep -xf $(GUI_PATTERN) >/dev/null && pkill -9 -xf $(GUI_PATTERN); \
	fi; true
	pnpm tauri dev

# Bump version + tauri build
bundle:
	node scripts/bump-version.mjs
	@echo "[easy-vial] building v$(VER)"
	pnpm install --frozen-lockfile
	pnpm tauri build --bundles deb

# Build + auto install. Asks sudo password upfront, keeps it alive during build.
build:
	@sudo -v || exit 1; \
	( while true; do sudo -n true; sleep 50; kill -0 $$$$ 2>/dev/null || exit; done ) 2>/dev/null & \
	KEEP=$$!; \
	$(MAKE) bundle && $(MAKE) deb-install; RC=$$?; \
	kill $$KEEP 2>/dev/null; exit $$RC

# Build only, open bundle dir, print manual install cmd
build-noinstall: bundle
	@xdg-open $(DEB_DIR)
	@echo ""; echo "Install manually:"; \
	echo "  sudo apt install ./$(DEB_DIR)/easy-vial_$(VER)_amd64.deb"

# Install latest built .deb, point autostart at it, restart daemon + app
deb-install:
	@VER=$(VER); \
	DEB=$(DEB_DIR)/easy-vial_$${VER}_amd64.deb; \
	if pgrep -xf $(GUI_PATTERN) >/dev/null; then \
	  echo "[easy-vial] closing running easy-vial..."; \
	  pkill -xf $(GUI_PATTERN); sleep 1; \
	  pgrep -xf $(GUI_PATTERN) >/dev/null && pkill -9 -xf $(GUI_PATTERN); \
	fi; \
	echo "[easy-vial] installing $$DEB..."; \
	sudo apt install -y --reinstall ./$$DEB || exit 1; \
	if [ -f $(AUTOSTART) ]; then \
	  sed -i 's|^Exec=.*|Exec="/usr/bin/easy-vial" --daemon|' $(AUTOSTART); \
	  echo "[easy-vial] autostart now uses /usr/bin/easy-vial"; \
	fi; \
	if pgrep -f $(DAEMON_PATTERN) >/dev/null; then \
	  echo "[easy-vial] restarting daemon..."; \
	  pkill -f $(DAEMON_PATTERN); sleep 1; \
	  setsid -f /usr/bin/easy-vial --daemon >>$(HOME)/.cache/easy-vial/daemon.log 2>&1; \
	fi; \
	echo "[easy-vial] launching easy-vial v$$VER..."; \
	nohup easy-vial >/dev/null 2>&1 & \
	echo "[easy-vial] done: easy-vial v$$VER installed and running."

# Bump version, commit, tag, push. The tag triggers the GitHub release workflow.
release:
	@git diff --quiet && git diff --cached --quiet || { echo "[easy-vial] commit or stash changes first"; exit 1; }
	node scripts/bump-version.mjs
	git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
	git commit -m "chore: release v$(VER)"
	git tag v$(VER)
	git push origin HEAD v$(VER)
