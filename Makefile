VERSION ?= 1.3.0
PREFIX  ?= $(HOME)/.local

.PHONY: install uninstall deb rpm tarball packages test lint clean

install:
	./scripts/install.sh --prefix "$(PREFIX)" --no-deps --no-shell

uninstall:
	./scripts/uninstall.sh

deb:
	VERSION="$(VERSION)" ./packaging/deb/build-deb.sh

rpm:
	VERSION="$(VERSION)" ./packaging/rpm/build-rpm.sh

tarball:
	mkdir -p dist
	tar --transform "s,^,rec-cli-$(VERSION)/," -czf "dist/rec-cli-$(VERSION).tar.gz" \
		bin shell starship scripts packaging LICENSE README.md Makefile

packages: deb rpm tarball

test:
	./tests/run_tests.sh

lint:
	bash -n bin/rec-cli bin/cast2mp4 bin/casts2mp4 shell/rec.bash scripts/install.sh scripts/uninstall.sh packaging/deb/build-deb.sh packaging/rpm/build-rpm.sh tests/run_tests.sh
	zsh -n shell/rec.zsh

clean:
	rm -rf dist
