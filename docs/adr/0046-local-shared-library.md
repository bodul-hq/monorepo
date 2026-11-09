# ADR 46: Local shared library

Status: Accepted.

Date: 2026-11-09

## Context

Retailer integrations and pipeline consumers depend on published `bodul_shared`
0.1.1, while `lib/shared` already owns common health and image helpers. Updating
shared retailer types should be possible atomically with their monorepo consumers.

## Decision

Import the resolved package's retailer, link, sitemap configuration, and normalized
data APIs into the existing `shared` crate. Preserve its MIT notice and provenance.
Migrate all monorepo consumers to local path dependencies and `shared::` imports.
Keep existing serialization and behavior, including compatibility methods and
link labels. Other published dependencies remain unchanged.

## Consequences

The monorepo owns maintenance of these types and tests. All workspaces resolve the
same local implementation without publishing a `bodul_shared` release. Future
upstream fixes require explicit review and porting. This changes Rust import paths,
not HTTP contracts or stored data, and requires no database migration.
