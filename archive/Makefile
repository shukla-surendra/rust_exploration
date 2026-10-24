.PHONY: docs docs-install docs-build docs-serve

docs: docs-serve

docs-install:
	command -v mdbook >/dev/null || brew install mdbook

docs-build: docs-install
	mdbook build docs

docs-serve: docs-install
	mdbook serve docs --open
