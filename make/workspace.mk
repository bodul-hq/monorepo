# Runs each target in every directory listed in MEMBERS.
TARGETS := test check build release fmt fmt-check fix clippy lint ci clean $(EXTRA_TARGETS)

.PHONY: $(TARGETS) audit

$(TARGETS):
	@for member in $(MEMBERS); do $(MAKE) -C $$member $@ || exit 1; done

# cargo audit checks a workspace's Cargo.lock, so it runs once per
# workspace instead of once per crate.
audit:
	@if [ -f Cargo.lock ]; then \
		cargo audit; \
	else \
		for member in $(MEMBERS); do $(MAKE) -C $$member audit || exit 1; done; \
	fi
