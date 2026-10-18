include ../../make/rust.mk

.PHONY: serve dump-offer-details extract-offer-details normalize-offer-details structuralize-offer-details dump-sitemap-sources process-sitemap-sources consolidate-sitemap-sources

serve:
	cargo run

# Downloads every fetched sitemap file from retailer_sourcing's API to
# DATA_DIR/<retailer>/sitemaps/sources. Set RETAILER_SOURCING_URL to the
# service's address; it defaults to http://localhost:3000.
DATA_DIR ?= ../../../data
dump-sitemap-sources:
	cargo run --release --bin dump-sitemap-sources -- $(DATA_DIR)

# Downloads the newest fetched page of every offer from retailer_sourcing's API
# to DATA_DIR/<retailer>/offer_details/sources. Set RETAILER_SOURCING_URL as above.
dump-offer-details:
	cargo run --release --bin dump-offer-details -- $(DATA_DIR)

# Each runs one stage over DATA_DIR/<retailer>/offer_details: sources/*.html are
# extracted into extracted/, those normalized into normalized/, and those
# structuralized into structuralized/. Set RETAILERS to process only some,
# e.g. `make extract-offer-details RETAILERS="ankerau ankerjapancom"`.
RETAILERS ?=
extract-offer-details:
	cargo run --release --bin extract-offer-details -- $(DATA_DIR) $(RETAILERS)

normalize-offer-details:
	cargo run --release --bin normalize-offer-details -- $(DATA_DIR) $(RETAILERS)

structuralize-offer-details:
	cargo run --release --bin structuralize-offer-details -- $(DATA_DIR) $(RETAILERS)

# Builds a sitemap tree from each dumped crawl in DATA_DIR/<retailer>/sitemaps/sources
# and writes it to DATA_DIR/<retailer>/sitemaps/processed/<retrieval id>.json.
process-sitemap-sources:
	cargo run --release --bin process-sitemap-sources -- $(DATA_DIR)

# Sorts the URLs of each tree in DATA_DIR/<retailer>/sitemaps/processed by kind
# of page and writes them to DATA_DIR/<retailer>/sitemaps/consolidate/<retrieval id>.json.
consolidate-sitemap-sources:
	cargo run --release --bin consolidate-sitemap-sources -- $(DATA_DIR)
