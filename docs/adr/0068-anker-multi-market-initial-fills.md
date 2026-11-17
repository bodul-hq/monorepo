# ADR 68: Anker multi-market initial fills

Status: Accepted

Supersedes ADR 59's fixed-context exception for Anker.

All 17 supported Anker integrations enable the existing multi-market flow.
Resolve access using each storefront's existing token policy, discover advertised
country/language pairs with their currencies, and fan out contextual catalog
and detail requests. All continuations retain context and validate response
context and currencies. Discovery failures prevent fan-out.

New fills write schema-v2 artifacts with context and recorded pages; acquisition
time remains supplied by the sourcing pipeline. Context-aware command keys and
country-language dump paths use the existing implementation. Historical raw
readers and Anker extraction recovery remain supported. No database migration
or changes to legacy Nordic/HTML extraction are required.

Drain existing Anker fills before deploying, then start new fills. Do not replay
old token events to migrate existing runs. Validate one newly started fill before
expanding rollout: all-market discovery can materially increase crawl volume.
