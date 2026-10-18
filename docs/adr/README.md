# Architecture decision records

| ADR                                                     | Title                                      | Status                                         |
| ------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------- |
| [1](0001-web-frameworks-for-apps-and-services.md)       | Web frameworks for apps and services       | Accepted, `/graphql` route superseded by ADR 3 |
| [2](0002-workspace-dependencies-and-port-validation.md) | Workspace dependencies and port validation | Accepted                                       |
| [3](0003-graphql-over-get-and-debug-only-graphiql.md)   | GraphQL over GET and debug-only GraphiQL   | Accepted; object storage GET requirement superseded by ADR 6 |
| [4](0004-feature-slices-for-retailer-sourcing.md)       | Feature slices for retailer_sourcing       | Accepted                                       |
| [5](0005-axum-for-object-storage.md)                   | Axum for object storage                    | Accepted                                       |
| [6](0006-object-storage-startup-and-access.md) | Object storage startup and access | Accepted |
| [7](0007-domain-entities-and-postgres-records.md) | Domain entities and PostgreSQL records | Accepted; storage layout superseded by ADRs 8 and 9 |
| [8](0008-consolidated-sitemap-jsonb-references-and-storage.md) | Consolidated sitemap JSONB references and storage | Accepted |
| [9](0009-pipeline-storage-jsonb.md) | Pipeline storage metadata in JSONB | Accepted; Rust metadata representation superseded by ADR 10 |
| [10](0010-typed-record-metadata.md) | Typed record metadata | Accepted |
| [11](0011-raw-sitemap-object-storage.md) | Raw sitemap bodies in object storage | Accepted; supersedes raw sitemap payload layout in ADRs 7 and 9 and raw storage metadata in ADR 10 |
| [12](0012-pipeline-object-storage.md) | Remaining pipeline payloads in object storage | Accepted; supersedes remaining payload tables and storage shapes in ADRs 7–10 |
| [13](0013-development-database-reporting.md) | Database reporting for the development dashboard | Accepted |

To add a decision, copy the structure of an existing ADR (Status, Date, Context,
Decision, Consequences) and give it the next number.
