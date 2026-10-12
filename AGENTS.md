# AGENTS.md

Guidance for AI agents working in this repository. For the layout, requirements and Make targets, read [README.md](README.md) first.

## Commits

- Never add `Co-Authored-By` or other signature lines to commits or pull request descriptions.
- Keep each commit to one change. Write a short subject and a body that explains why. For example, a formatting fix is a separate commit from a feature.
- Check `git status` and the current branch before committing. Branches are often rebased, reset or squashed while work is in progress.

## Before finishing a change

- Run `make ci` in the folder you changed, or from the root, and make sure it passes. Run `make fmt` first, since unformatted code is the most common reason `ci` fails.
- Run `make audit` after adding or updating dependencies, and report any new advisories.
- When you change a service or app, keep its smoke tests passing and extend them for new routes. Also run it with `PORT=<port> make serve` and check that its health endpoint or page responds.

## Adding a crate

- Create it with `cargo new` and remove the generated placeholder code and tests.
- Use snake_case for crate names.
- Add the crate to its workspace's `members` in `Cargo.toml`, one entry per line.
- Give it a Makefile that includes `make/rust.mk`. Add `serve` (`cargo run`) for services and apps, or `doc` (`cargo doc --no-deps`) for libraries.
- Add the crate to the `MEMBERS` list in its folder's Makefile, and to the README.

## Dependencies

- Declare a dependency used by several crates in the same workspace once, under `[workspace.dependencies]`, and use it with `name.workspace = true`.
- Keep `[dependencies]` sorted alphabetically.
- Check that a crate's features exist before adding them.

## Services and apps

- New services use poem, poem-openapi and async-graphql, with the standard routes: `/api`, `/api/openapi.json`, `/api/docs` and `/graphql`.
- Build each server's routes in an `app()` function so tests can call them without opening a port.
- Every service and app reads its listen port from `PORT` and defaults to 80.
- `retailer_sourcing` uses axum. Don't migrate it without a decision recorded in an ADR.

## Decisions and documentation

- Record framework and architecture decisions as ADRs in `docs/adr/` and list them in `docs/adr/README.md`.
- When a decision changes, write a new ADR that supersedes the old one instead of editing an accepted ADR, unless asked to merge them.
- Update the README when crates, Make targets or requirements change.

## Asking first

- When a request has more than one reasonable reading, ask before acting.
