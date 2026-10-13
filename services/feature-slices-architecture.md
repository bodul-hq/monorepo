# Feature-Sliced Architecture: Command → Handler → Event (Rust)

Design notes for structuring application flows (e.g. `sitemap_discovery`) as
feature slices. Each slice is a use case following **command → handler →
event**. The structure should "scream" what the system does, not which libraries
it uses.

## Target layout

```
src/
  sitemap_discovery/
    mod.rs                  # domain types: SitemapId, SitemapUrl, Sitemap
    retrieve_sitemap/
      mod.rs                # pub use; register()
      command.rs            # RetrieveSitemap
      handler.rs
      event.rs              # SitemapRetrieved, SitemapRetrievalFailed
      store.rs              # this slice's own queries
    process_sitemap/
      command.rs  handler.rs  event.rs  store.rs
      reacts_to.rs          # SitemapRetrieved -> ProcessSitemap
    group_urls/
      command.rs  handler.rs  event.rs  store.rs
      reacts_to.rs          # SitemapProcessed -> GroupUrls
  page_crawling/
    ...
  platform/                 # deliberately boring and small
    db.rs                   # pool, tx helper
    http.rs                 # client with retry, rate limit, politeness
    bus.rs                  # dispatch, envelope, outbox
  main.rs                   # calls each feature's register()
```

When this grows, promote each bounded context (`sitemap_discovery`,
`page_crawling`) to its own crate. Features stay as modules inside it.

## Rules

1. **A slice owns its data access.** Queries live in the slice's `store.rs`
   (e.g. `sqlx::query_as!` against the shared pool). No central repository shared by
   all features. Small duplication is preferred over a shared repo every feature
   bends to its needs.

2. **The consumer defines the interface.** If a handler needs an external capability,
   the trait lives in that slice, shaped to its exact need. `platform` provides raw
   capabilities (HTTP client, DB pool); the feature adapts them.

   ```rust
   // sitemap_discovery/retrieve_sitemap/handler.rs
   pub trait FetchSitemapBody {
       async fn fetch(&self, url: &SitemapUrl) -> Result<Vec<u8>, FetchError>;
   }
   impl FetchSitemapBody for platform::http::Client { /* thin adapter */ }
   ```

3. **Handlers return events; they don't publish them.** Keeps handlers testable and
   unaware of the bus.

4. **Reactions live with the reactor.** `process_sitemap/reacts_to.rs` subscribes to
   `SitemapRetrieved`; retrieval doesn't know what happens next. Adding a feature means
   adding a folder, not editing an existing one.

   ```rust
   pub fn on(e: &SitemapRetrieved) -> Option<ProcessSitemap> {
       Some(ProcessSitemap { sitemap_id: e.sitemap_id })
   }
   ```

5. **Each slice registers itself.** `main.rs` reads like a feature list.

   ```rust
   // process_sitemap/mod.rs
   pub fn register(bus: &mut Bus, deps: &Platform) {
       bus.handle::<ProcessSitemap>(Handler::new(deps.db.clone()));
       bus.on::<SitemapRetrieved>(reacts_to::on);
   }
   ```

6. **Promotion ladder for shared code.**
   - Used in one slice → stays in the slice.
   - Domain concept used across slices in one context → context `mod.rs`
     (e.g. `Sitemap`, URL normalization).
   - Pure technical plumbing → `platform/`. Nothing in `platform/` knows the word
     "sitemap".

7. **Contexts talk through events only.** Other contexts react to public events
   (e.g. `UrlsGrouped`) and never import another slice's `store` or handler.
   Events are `pub`; everything else `pub(crate)` or narrower.

## Cross-cutting concerns (in `platform/`)

- **Outbox:** write events to an `outbox` table in the same transaction as the
  handler's writes; a relay publishes them. Prevents stalled flows after crashes.
- **Idempotency:** every command has a deterministic id (e.g. URL + crawl run id);
  use upserts / dedupe so retries are harmless.
- **Errors:** `thiserror` per slice, split into retryable vs permanent so the runtime
  knows whether to requeue. `anyhow` only at the edge (`main`).
- **Observability:** command envelope carries `correlation_id` and `causation_id`;
  one `tracing` span per handle.
- **Backpressure:** bounded channels and per-feature concurrency limits —
  sitemap discovery fans out explosively.
- **Dispatch:** start in-process (`tokio::mpsc` + worker pool). Commands are plain
  serializable structs, so moving to an external queue later doesn't touch handlers.
- **Dependencies:** prefer generics over `Arc<dyn Trait>` inside slices (native
  `async fn` in traits isn't dyn-compatible without boxing). Avoid a god `AppContext`.

## Testing

In-memory fakes for the slice's own traits (canned fetcher, `HashMap` store).
Most tests: "given command, assert returned events" — no DB needed.
