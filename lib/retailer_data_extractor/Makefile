include ../../make/rust.mk

.PHONY: doc process-offer-details

doc:
	cargo doc --no-deps

# Writes the extracted, normalized and structurized stages of every page in
# DATA_DIR/<retailer>/offer_details/sources. Set RETAILERS to process only some,
# e.g. `make process-offer-details RETAILERS="ankerau ankerjapancom"`.
DATA_DIR ?= ../../../data
RETAILERS ?=
process-offer-details:
	cargo run --release --bin process_offer_details -- $(DATA_DIR) $(RETAILERS)
