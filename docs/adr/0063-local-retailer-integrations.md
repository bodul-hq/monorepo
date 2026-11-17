# 63. Local retailer integrations

Status: Accepted

## Decision

Maintain Anker, Minisforum and Ugreen in the monorepo with relative Cargo paths.
Sources were imported from the local bodul_hq repositories. Supporting crates remain published dependencies.

The former WIP AU and CA implementations replace the imported versions, preserving their extracted v2 output. Other storefronts retain typed raw v2 parsing. Retain the empty WIP crate for future experiments. Mi remains disconnected.

No command/event migration or package publication is required.
