# Variables
GIT := git
PNPM := pnpm
CARGO := cargo
DOCKER := docker
CD := cd

BAILIAN_PATH := ./rig-bailian
TEI_PATH := ./rig-tei
VOLCENGINE_PATH := ./rig-volcengine


# Function to check if there are changes to commit
define git_push_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
		$(GIT) push; \
	else \
		echo "No changes to commit"; \
	fi
endef

define git_commit_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
	else \
		echo "No changes to commit"; \
	fi
endef

# Git run add commit push
git-run:
	$(call git_push_if_needed)

# Git run add commit push
git-commit:
	$(call git_commit_if_needed)

# Git: create and push a tag
# Usage: make git-tag t=v1.2.3 [m="message"] [allow_dirty=1]
# git tag -a v0.1.0 -m "tag v0.1.0"
git-tag:
	@if [ -z "$(t)" ]; then \
		echo "Error: tag name is required. Use: make git-tag t=vX.Y.Z [m='message'] [allow_dirty=1]"; \
		exit 1; \
	elif [ "$(allow_dirty)" != "1" ] && [ -n "$$($(GIT) status --porcelain)" ]; then \
		echo "Working tree is dirty. Commit or stash changes, or set allow_dirty=1 to proceed."; \
		exit 1; \
	else \
		MSG="$(m)"; \
		if [ -z "$$MSG" ]; then MSG="Tag $(t)"; fi; \
		$(GIT) tag -a "$(t)" -m "$$MSG"; \
		$(GIT) push origin "refs/tags/$(t)"; \
	fi

# Git: push all local tags
git-push-tags:
	$(GIT) push --tags

# clean rig-extend
clean:
	@echo "Cleaning rig-extend..."
	$(CARGO) clean

# Publish all crates to crates.io (requires `cargo login`)
publish-all:
	@echo "===> Publishing all crates (bailian, tei, volcengine)"
	$(call git_commit_if_needed)
	$(MAKE) bailian-publish || exit 1
	$(MAKE) tei-publish || exit 1
	$(MAKE) volcengine-publish || exit 1
	$(CARGO) clean

# Bailian Publish facade crate to crates.io (requires `cargo login`)
bailian-publish:
	@echo "===> Publishing bailian"
	$(call git_commit_if_needed)
	cd $(BAILIAN_PATH) &&  $(CARGO) publish -p rig-bailian --dry-run --registry crates-io || exit 1
	cd $(BAILIAN_PATH) &&  $(CARGO) publish -p rig-bailian --registry crates-io || exit 1

# TEI Publish facade crate to crates.io (requires `cargo login`)
tei-publish:
	@echo "===> Publishing tei"
	$(call git_commit_if_needed)
	cd $(TEI_PATH) &&  $(CARGO) publish -p rig-tei --dry-run --registry crates-io || exit 1
	cd $(TEI_PATH) &&  $(CARGO) publish -p rig-tei --registry crates-io || exit 1

# Volcengine Publish facade crate to crates.io (requires `cargo login`)
volcengine-publish:
	@echo "===> Publishing volcengine"
	$(call git_commit_if_needed)
	cd $(VOLCENGINE_PATH) &&  $(CARGO) publish -p rig-volcengine --dry-run --registry crates-io || exit 1
	cd $(VOLCENGINE_PATH) &&  $(CARGO) publish -p rig-volcengine --registry crates-io || exit 1
