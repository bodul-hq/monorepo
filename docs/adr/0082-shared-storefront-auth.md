# 82. Shared Storefront authentication discovery

Status: Accepted

## Decision

Move brand auth orchestration, HTML inspection, JavaScript token parsing and
module-token resolution into retailer_sauce::shopify::graphql. Expose
`auth::resolve(&StorefrontConfig, &dyn Http, url)` and keep parsers private.
All three engines delegate to it. Use Ugreen's nested-template parser fixes.

Preserve URL validation, configured-token precedence, allowed HTTPS asset hosts,
asset deduplication, regional token selection and conflict detection. Apply the
existing Minisforum/Ugreen skip for optional retailers with acquisition disabled
to the shared resolver; current Anker configurations continue acquiring tokens.

## Validation

Move parser unit tests and the captured Anker HTML fixture with the implementation.
Keep Ugreen NAS integration coverage in its brand crate. Run both library CI suites
and audit changed dependency workspaces. No network collection is required.
