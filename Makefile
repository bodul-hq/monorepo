MEMBERS := lib services apps

include make/workspace.mk

.PHONY: dump-offer-details extract-offer-details normalize-offer-details structuralize-offer-details dump-sitemap-sources process-sitemap-sources consolidate-sitemap-sources process-offer-details

# The data folder next to the monorepo. Passed on as an absolute path, because
# the targets below run from their own folders.
DATA_DIR ?= ../data

# Downloads every fetched sitemap file from retailer_sourcing's API to
# DATA_DIR/<retailer>/sitemaps/sources. Set RETAILER_SOURCING_URL to the
# service's address; it defaults to http://localhost:3000.
dump-sitemap-sources:
	$(MAKE) -C apps/development dump-sitemap-sources DATA_DIR=$(abspath $(DATA_DIR))

# Downloads the newest fetched page of every offer from retailer_sourcing's API
# to DATA_DIR/<retailer>/offer_details/sources. Set RETAILER_SOURCING_URL to the
# service's address; it defaults to http://localhost:3000.
dump-offer-details:
	$(MAKE) -C apps/development dump-offer-details DATA_DIR=$(abspath $(DATA_DIR))

# Each runs one stage over DATA_DIR/<retailer>/offer_details: sources/*.html are
# extracted into extracted/, those normalized into normalized/, and those
# structuralized into structuralized/. Set RETAILERS to process only some.
extract-offer-details normalize-offer-details structuralize-offer-details:
	$(MAKE) -C apps/development $@ DATA_DIR=$(abspath $(DATA_DIR)) RETAILERS="$(RETAILERS)"

# Builds a sitemap tree from each dumped crawl in DATA_DIR/<retailer>/sitemaps/sources
# and writes it to DATA_DIR/<retailer>/sitemaps/processed/<retrieval id>.json.
process-sitemap-sources:
	$(MAKE) -C apps/development process-sitemap-sources DATA_DIR=$(abspath $(DATA_DIR))

# Sorts the URLs of each tree in DATA_DIR/<retailer>/sitemaps/processed by kind
# of page and writes them to DATA_DIR/<retailer>/sitemaps/consolidate/<retrieval id>.json.
consolidate-sitemap-sources:
	$(MAKE) -C apps/development consolidate-sitemap-sources DATA_DIR=$(abspath $(DATA_DIR))

# Writes the extracted, normalized and structurized stages of every saved offer
# page. Set RETAILERS to process only some, e.g. RETAILERS="ankerau ankervn".
process-offer-details:
	$(MAKE) -C lib/retailer_data_extractor process-offer-details DATA_DIR=$(abspath $(DATA_DIR))

.PHONY: export-offer-sources
export-offer-sources:
	$(MAKE) -C services/retailer_sourcing export-offer-sources DATA_DIR=$(abspath $(DATA_DIR))

.PHONY: export-sitemaps
export-sitemaps:
	$(MAKE) -C services/retailer_sourcing export-sitemaps DATA_DIR=$(abspath $(DATA_DIR))
