# Export conversion fixtures

Small synthetic examples preserve the structuralized document shapes emitted by
Anker AU, Ugreen CA and Minisforum US. They cover variant URLs, unrelated fields,
optional prices and unavailable stock without relying on downloaded retailer data.

`import_payload.json` is the expected first Anker export payload. Sourcing checks
its serialized result against this fixture; offers deserializes the same fixture
through its actual `ImportOfferDetailsCommand`. Run both services' `make ci` checks
when changing this contract.
