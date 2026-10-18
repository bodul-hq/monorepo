# AGENTS.md

Guidance for AI agents working in this repository. For the layout and
requirements, read [README.md](README.md) first.

## Commits

- Never add `Co-Authored-By` or other signature lines to commits or pull request descriptions.
- Keep each commit to one change. Write a short subject and a body that explains why. For example, a formatting fix is a separate commit from a feature.
- Check `git status` and the current branch before committing. Branches are often rebased, reset or squashed while work is in progress.

## Before finishing a change

- Run `make ci` in the folder you changed, or from the root, and make sure it passes. Run `make fmt` first, since unformatted code is the most common reason `ci` fails.
- Run `make audit` only when Cargo dependencies change, that is when a `Cargo.toml` dependency or a `Cargo.lock` file is modified, and report any new advisories. Other changes don't need it.
- When you change a service or app, keep its smoke tests passing and extend them for new routes. Also run it with `PORT=<port> make serve` and check that its health endpoint or page responds.
- The repository has only Rust code today. If code in another language is added, document how to test it in that folder's README, and run those tests before finishing a change.

## Adding a crate

- Create it with `cargo new` and remove the generated placeholder code. A library with no code yet keeps a placeholder test, like the other empty crates in `lib/`, until it has real tests.
- Use snake_case for crate names.
- Add the crate to its workspace's `members` in `Cargo.toml`, one entry per line.
- Add the crate to the README.
- Give a library crate a `README.md` that describes its purpose, like the others in `lib/`.

## Adding or implementing a retailer

- Follow [docs/retailer-integration.md](docs/retailer-integration.md) step by step.

## Dependencies

- Declare a dependency used by several crates in the same workspace once, under `[workspace.dependencies]`, and use it with `name.workspace = true`.
- Keep `[dependencies]` sorted alphabetically.
- Check that a crate's features exist before adding them.

## Services and apps

- New services use poem, poem-openapi and async-graphql, with the standard routes: `/api`, `/api/openapi.json`, `/api/docs`, `/graphql`, and `/graphql/ide` in debug builds only.
- Object storage intentionally uses axum to compare its development experience with poem; see [ADR 5](docs/adr/0005-axum-for-object-storage.md).
- Build each server's routes in an `app()` function so tests can call them without opening a port.
- Every service and app reads its listen port from `PORT` and defaults to 3000.
- `retailer_sourcing` uses axum. Don't migrate it without a decision recorded in an ADR.

## Decisions and documentation

- Record framework and architecture decisions as ADRs in `docs/adr/` and list them in `docs/adr/README.md`.
- When a decision changes, write a new ADR that supersedes the old one instead of editing an accepted ADR, unless asked to merge them.
- Update the README when crates or requirements change.

## Asking first

- When a request has more than one reasonable reading, ask before acting.
