# ADR 2: Unprivileged default port

- Status: Accepted
- Date: 2026-10-12
- Supersedes: the default-port decision in [ADR 1](0001-web-frameworks-for-apps-and-services.md)

## Context

Port 80 requires privileges on systems that restrict low ports, so the skeleton servers can fail to start for ordinary users.

## Decision

Every service and app defaults to port 3000. The `PORT` environment variable overrides this default. The existing bind address remains `0.0.0.0` so containers can expose the server.

## Consequences

Local startup works without privileges. To run several servers at once, assign distinct ports using `PORT`. Deployments that need port 80 must set it explicitly or map an external port to 3000.
