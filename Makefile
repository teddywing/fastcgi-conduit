DOCS_WORKTREE := /tmp/fastcgi-conduit-docs


.PHONY: docs
docs: target/doc/* \
	$(DOCS_WORKTREE) \
	$(DOCS_WORKTREE)/* \
	$(DOCS_WORKTREE)/index.html
	git -C $(DOCS_WORKTREE) add .
	git -C $(DOCS_WORKTREE) commit

target/doc/*:
	cargo doc --no-deps

$(DOCS_WORKTREE):
	git worktree add $(DOCS_WORKTREE) gh-pages

$(DOCS_WORKTREE)/*:
	cp -R target/doc/* $(DOCS_WORKTREE)/

$(DOCS_WORKTREE)/index.html:
	cp doc/index.html $(DOCS_WORKTREE)/

.PHONY: docs-clean
docs-clean:
	git worktree remove --force $(DOCS_WORKTREE)
