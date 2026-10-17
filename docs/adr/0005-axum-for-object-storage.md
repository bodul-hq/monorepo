# ADR 5: Axum for object storage

- Status: Accepted
- Date: 2026-10-15
- Supersedes: the poem convention for new services for object storage only;
  extends the framework choices in [ADR 1](0001-web-frameworks-for-apps-and-services.md).

## Context

New services normally use poem. The object storage service is an intentional
exception so that implementing it can help evaluate whether axum is easier
to work with than poem.

## Decision

Use axum for the object storage service. Evaluate the development experience
while implementing its REST and GraphQL APIs, validation, error handling,
OpenAPI documentation, and tests.

Poem remains the default for other new services. Any broader change to that
default requires a separate decision. Other service conventions continue to
apply.

## Consequences

- Object storage provides another concrete basis for comparing axum and poem.
- Its HTTP integration will use axum-compatible tooling in place of the
  poem-specific tooling prescribed for new services.
- The experiment's findings should inform a later framework decision; choosing
  axum here does not establish that it is easier to work with.
