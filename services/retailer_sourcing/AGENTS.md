# AGENTS.md

Read [README.md](README.md) for the service layout and requirements.

## Architecture

- Application code implements use cases by using domain code and infrastructure
  implementations.
- Domain code owns business concepts and rules.
- Infrastructure code implements access to databases, external services, storage,
  and other technical capabilities used by the application.
- UI is an entry point into the application: HTTP, CLI, or another interface.
  It translates input into application calls and maps their results into output.
  Keep use-case orchestration in application code and business rules in domain
  code.

## v2 code style

- Place submodule declarations (`mod xyz;`) at the top of the file, after
  module documentation and before production imports.
- In `src/v2`, import functions, types and macros with `use` instead of using
  module-qualified paths inline. Use descriptive import aliases when names
  collide. Keep associated calls and enum variants qualified by their type
  (for example, `Uuid::now_v7()` and `SourceType::GraphQl`).
- Keep test-only imports inside the test module. For tests in the same module
  as production code, place their imports just above the first test.
- In `src/v2`, when a returned constructed value takes more than 4 lines, define
  it with `let` before returning it. Shorter ones may be returned directly.
  A value that is itself the returned expression is exempt, because clippy's
  `let_and_return` flags binding it. Don't add `#[allow(clippy::let_and_return)]`.
- This doesn't apply when returning a property or variable straight out, since
  nothing is constructed. It also doesn't apply to a constructor that only
  assigns its own parameters to fields, like `Self { run_id, url }`.
