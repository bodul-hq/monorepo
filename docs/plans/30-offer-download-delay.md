# Offer download delay

Status: implemented on 2026-10-27. The user selected a per-handler delay; concurrent workers
may fetch at the same time. This is not a shared retailer/global rate limit.

## Implementation

Define `DOWNLOAD_OFFER_PAGE_DELAY` as six seconds. In `DownloadOfferPage`, apply
the existing URL replacement, then sleep once immediately before the fetch.
Delay every invocation, including retries and repeated URLs. Keep this pacing
inside the synchronous handler, using `std::thread::sleep`; sitemap and storage
requests retain their own behavior. Preserve storage/error/event semantics and
capture `scraped_at` after a successful fetch.

Use an internal injectable sleeper for tests while retaining the public handler
constructor. Document per-handler pacing and correct the service README's stale
description of failed downloads: fetch failures fail the command and retry.

## Validation

Record `sleep(6s) → fetch → store` ordering without real delays. Verify one sleep
on fetch failure, storage failure and each repeated invocation. Retain retired
URL and timestamp coverage. Run service fmt/CI and live health checks. No new
dependency, configuration variable, migration or wire version is needed.

## Completion

The six-second constant and injectable sleeper are implemented. Tests cover
successful fetch/storage, fetch failures, storage failures, repeated calls and
sleep/fetch/store order without waiting. See the
[verification record](28-31-verification.md).
