# Runs each target in every directory listed in MEMBERS.
TARGETS := test check build release fmt fmt-check fix clippy lint ci clean $(EXTRA_TARGETS)

.PHONY: $(TARGETS) audit sweep

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

# Removes outdated builds, keeping the newest KEEP per environment. Like
# audit, it runs once per workspace, since a workspace shares one target folder.
# Set DRY_RUN=1 to preview. Don't run it while that workspace is building.
#
# Cargo names each build by a hash and records it in
# target/<profile>/.fingerprint/<crate>-<hash>/<kind>.json. Builds whose JSON
# matches once compiler and dependency hashes (rustc, path, deps, local) are
# dropped share an environment. SWEEP_AWK reads those JSON files newest first
# and prints `stale <crate>-<hash> <hash>` for builds past KEEP, then
# `keep <crate> <count>`. Incremental caches carry no build hash, so each crate
# keeps as many of its newest caches as it has kept builds.
KEEP ?= 1
SWEEP_AWK = { \
	path = $$0; getline json < path; close(path); \
	dir = path; sub(/\/[^\/]*$$/, "", dir); sub(/.*\//, "", dir); \
	kind = path; sub(/.*\//, "", kind); \
	hash = substr(dir, length(dir) - 15); crate = substr(dir, 1, length(dir) - 17); \
	sub(/"rustc":[0-9]+,/, "", json); \
	sub(/"declared_features":"[^"]*",/, "", json); \
	sub(/"path":[0-9]+,"deps":.*"rustflags"/, "\"rustflags\"", json); \
	key = crate SUBSEP kind SUBSEP json; \
	if (++seen[key] > keep) print "stale", dir, hash; \
	else { gsub(/-/, "_", crate); kept[crate]++ } \
} \
END { for (crate in kept) print "keep", crate, kept[crate] }

sweep:
	@if [ ! -f Cargo.lock ]; then \
		for member in $(MEMBERS); do $(MAKE) -C $$member sweep || exit 1; done; \
		exit 0; \
	fi; \
	target=$${CARGO_TARGET_DIR:-target}; \
	list=$$(mktemp); trap 'rm -f "$$list"' EXIT; \
	for fingerprint in "$$target"/*/.fingerprint "$$target"/*/*/.fingerprint; do \
		[ -d "$$fingerprint" ] || continue; \
		profile=$${fingerprint%/.fingerprint}; \
		ls -t "$$fingerprint"/*/*.json 2>/dev/null \
		| awk -v keep=$(KEEP) '$(SWEEP_AWK)' \
		| while read -r action name value; do \
			if [ "$$action" = stale ]; then \
				echo "$$fingerprint/$$name"; \
				for folder in deps build examples; do \
					ls -d "$$profile/$$folder"/*-"$$value" "$$profile/$$folder"/*-"$$value".* 2>/dev/null; \
				done; \
			else \
				ls -td "$$profile/incremental/$$name"-* 2>/dev/null | tail -n +$$((value + 1)); \
			fi; \
		done; \
	done > "$$list"; \
	count=$$(wc -l < "$$list" | tr -d ' '); \
	kib=$$(tr '\n' '\0' < "$$list" | xargs -0 du -sk 2>/dev/null | awk '{ s += $$1 } END { print s + 0 }'); \
	if [ -z "$(DRY_RUN)" ]; then tr '\n' '\0' < "$$list" | xargs -0 rm -rf; verb=freed; else verb="would free"; fi; \
	awk -v t="$$PWD/$$target" -v v="$$verb" -v k="$$kib" -v n="$$count" \
		'BEGIN { printf "%s: %s %.1f GiB (%d paths)\n", t, v, k / 1048576, n }'
