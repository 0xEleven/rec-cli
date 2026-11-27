VERSION ?= 2.1.0
PREFIX  ?= $(HOME)/.local
CARGO   ?= $(shell command -v cargo 2>/dev/null || echo "$(HOME)/.cargo/bin/cargo")

.PHONY: build-gui install uninstall deb rpm tarball packages test lint clean

build-gui:
	"$(CARGO)" build --release --manifest-path gui/Cargo.toml

install: build-gui
	./scripts/install.sh --prefix "$(PREFIX)" --no-deps --no-shell

uninstall:
	./scripts/uninstall.sh

deb: build-gui
	VERSION="$(VERSION)" ./packaging/deb/build-deb.sh

rpm: build-gui
	VERSION="$(VERSION)" ./packaging/rpm/build-rpm.sh

tarball:
	mkdir -p dist
	tar --transform "s,^,rec-cli-$(VERSION)/," --exclude="gui/target" -czf "dist/rec-cli-$(VERSION).tar.gz" \
		bin gui shell starship scripts packaging assets LICENSE README.md Makefile

packages: deb rpm tarball

test: build-gui
	"$(CARGO)" test --manifest-path gui/Cargo.toml
	./tests/run_tests.sh

lint:
	bash -n bin/rec-cli bin/cast2mp4 bin/casts2mp4 shell/rec.bash scripts/install.sh scripts/uninstall.sh packaging/deb/build-deb.sh packaging/rpm/build-rpm.sh tests/run_tests.sh
	zsh -n shell/rec.zsh

clean:
	rm -rf dist gui/target
