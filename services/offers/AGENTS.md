# AGENTS.md

- Construct errors in named helper functions to separate intention from
  implementation. Call sites should explain why an error occurs; helpers should
  build its message and status.

Example from `OffersApi::tmp_import` in `src/offers/http/rest.rs` (abridged):

```rust
async fn tmp_import(
    &self,
    request: Json<ImportOfferRequest>,
) -> Result<Json<ImportOfferResponse>> {
    let command = ImportOfferDetailsCommand::try_from(request.0)
        .map_err(err_invalid_import_request)?;
    let handler = self.handler.clone().ok_or_else(err_database_not_configured)?;
    // Execute the command and return the response…
}

fn err_invalid_import_request(message: String) -> Error {
    Error::from_string(message, StatusCode::BAD_REQUEST)
}

fn err_database_not_configured() -> Error {
    Error::from_string("db not configured", StatusCode::SERVICE_UNAVAILABLE)
}
```

## Guard methods

A guard method checks preconditions before an operation proceeds. It borrows
the input or state, returns `Ok(())` when valid, and returns an error immediately
when a condition fails. Guards do not mutate state, persist data, or emit events.

Call guards before constructing the result or performing side effects, using
`Self::guard(&request)?;`. Use `guard` for a single validation entry point or
`guard_<condition>` for a specific check. Construct errors through named helpers,
following the error convention above.

For example, a guard on `ImportOfferDetailsCommand` can validate the request's
title:

```rust
impl ImportOfferDetailsCommand {
    fn guard(request: &ImportOfferRequest) -> Result<(), String> {
        if request.title.trim().is_empty() {
            return Err(err_blank_title());
        }
        Ok(())
    }
}

fn err_blank_title() -> String {
    "title must contain non-whitespace text".into()
}
```

Inside `TryFrom<ImportOfferRequest>::try_from`, call the guard before building
the command:

```rust
Self::guard(&request)?;
// Construct the command from the validated request.
```

## Separate input construction from use

Separate input construction from its use. When a function or conversion takes
a multiline struct literal, first assign the struct to a meaningfully named
local variable, then pass that variable to the call. Apply this in production
code and tests. Simple expressions such as `.into()` on individual fields can
remain inline.

```rust
let req = ImportOfferRequest {
    retailer_code: "ankerau".into(),
    title: " USB charger ".into(),
    price_amount: amount,
    currency: "AUD".into(),
    url: "https://example.com/Charger?q=red#details".into(),
};

let submission = inbox::SubmitInboxRequest::try_from(req).unwrap();
```
