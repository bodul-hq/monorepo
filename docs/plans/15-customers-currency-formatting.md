# Item 15: Customers currency symbols and formatting

Status: completed and verified on 2026-10-26.

## Goal and confirmed scope

Show each offer's price with a currency symbol and a fixed formatting convention
for that currency. The user confirmed **fixed convention per currency** on
2026-10-26. Retailer country, browser language and visitor location do not select
the format. Amounts stay in the original currency; no conversion is involved.

## Starting behavior

[Customers rendering](../../apps/customers/src/page.rs) formats signed i64 minor
units using integer arithmetic and appends the ISO code, such as `19.99 AUD`.
The [HTTP client](../../apps/customers/src/offers.rs) validates canonical integer
strings and deserializes currency into
[`money::Currency`](../../lib/money/src/currency.rs). That enum supports 13
currencies and supplies the fractional exponent. Keep that contract and its
full signed range.

The separate money type in `everyday` has a different currency set and unsigned
amount range; it is not a replacement for the customers app's money contract.

## Display contract

Use the following explicit product conventions. These are deterministic display
choices, not a claim that every country using a currency formats it identically.
Dollar prefixes distinguish currencies in a mixed-currency offer list.

| Currency | Symbol | Group separator | Decimal separator | Example display |
| --- | --- | --- | --- | --- |
| USD | `$` | comma | dot | `$1,234.56` |
| EUR | `€` | dot | comma | `1.234,56 €` |
| CAD | `CA$` | comma | dot | `CA$1,234.56` |
| AUD | `A$` | comma | dot | `A$1,234.56` |
| GBP | `£` | comma | dot | `£1,234.56` |
| JPY | `¥` | comma | none | `¥1,234` |
| KRW | `₩` | comma | none | `₩1,234` |
| HKD | `HK$` | comma | dot | `HK$1,234.56` |
| PLN | `zł` | nonbreaking space | comma | `1 234,56 zł` |
| SEK | `kr` | nonbreaking space | comma | `1 234,56 kr` |
| VND | `₫` | dot | none | `1.234 ₫` |
| NZD | `NZ$` | comma | dot | `NZ$1,234.56` |
| INR | `₹` | comma, Indian grouping | dot | `₹1,23,456.78` |

Suffix symbols have one U+00A0 nonbreaking space before them. PLN/SEK grouping
also uses U+00A0. Other groups contain three digits; INR has a three-digit final
group and two-digit groups before it. Always retain the fractional digits given
by `Currency::exponent()`, including trailing zeros. Negative values use U+2212
before the complete positive display, for example `−$0.01`; zero has no sign.

## Implementation steps

1. Add a private formatting policy in the customers app, with an exhaustive
   currency match for symbol, placement, separators and grouping. Keep money's
   exponent as the source of truth for fractional digits. No new dependency is
   needed for this small fixed table.
2. Split the validated amount using unsigned magnitude and integer division.
   Group the whole-unit digit string, append the exact fractional remainder,
   then apply the symbol and sign. Do not use floating point, locale-dependent
   process settings, or signed absolute value that overflows for `i64::MIN`.
3. Use the formatter for visible card prices and continue HTML-escaping output.
   Retain the ISO currency in an accessible price description so symbols remain
   unambiguous to assistive technology; avoid duplicate spoken prices.
4. Update existing price and homepage assertions. Keep the price/action footer
   usable for long grouped values, including at narrow widths. Do not truncate
   amounts or force an unbreakable string wider than the card.
5. Document the fixed conventions in the customers README. Update this plan,
   the plan index and TODO status only after verification.

## Acceptance and verification

- Table-driven tests cover every currency with exact Unicode spacing, prefix
  and suffix symbols, and fractional precision. Include zero, one minor unit,
  negatives, grouping boundaries and multiple Indian groups.
- Preserve exact `i64::MIN` and `i64::MAX` for two-decimal and zero-decimal
  currencies. Ensure grouping and symbol changes do not change the value.
- Rendering checks cover mixed currencies, an accessible currency description,
  unchanged pagination, safe links, error/empty pages and no browser scripts.
- Run `make fmt` then `make ci` in `apps/customers`. If shared Rust code is
  changed, check its workspace too. Audit only if Cargo dependencies or a
  lockfile change.
- Run `CUSTOMER_FACING_URL=<local fixture> PORT=<port> make serve`; verify HTTP
  200 for `/` and `/?offset=30` with 31 fixture offers. Review desktop, tablet,
  390px and 320px widths in light/dark themes, especially nonbreaking spaces,
  extreme values and the right-aligned offer action. Stop the fixture and app.

## Dependencies and boundaries

Builds on completed items 9 and 14. Independent of
[title cleanup](16-customers-plain-text-titles.md); implement in a separate
commit, then check the combined card layout. API payloads, persisted prices,
events and queue processing remain as defined in ADR 23. No new route, database
migration, backfill, JavaScript or architecture decision is required.

## Completion evidence

- Added the private `page::price` formatter with the complete table above,
  integer-only grouping and signed-range handling. Visible symbol prices are
  hidden from assistive technology in favor of one plain amount with its ISO
  currency code. The customers README documents the display conventions.
- `make fmt` and `make ci` in `apps/customers` passed: 21 tests including every
  currency, grouping boundaries, signed limits, mixed-currency rendering and
  title cleanup. Log: `/tmp/customer-display-ci.log`.
- `CUSTOMER_FACING_URL=http://127.0.0.1:3165 PORT=3166 make serve` returned HTTP
  200 with 30, one and zero fixture offers at offsets 0, 30 and 60.
- Firefox checks passed at 1440px, 800px, 390px and 320px in light/dark themes:
  exact displayed and accessible prices, long grouped amounts, no horizontal
  page overflow or overlapping footer content, right-aligned actions and
  preserved image/badge behavior. Desktop light and mobile dark screenshots
  were visually reviewed: `/tmp/customer-display-{theme}-{width}.png`.
- The combined browser run's layout checks passed; its new-tab check initially
  raced Firefox navigation. A focused rerun waited for the destination page
  and passed keyboard-focus and opener-isolation checks in both themes.
  Logs: `/tmp/customer-display-live.log`, `/tmp/customer-display-tabs.log`.
  The fixture, app and browsers were stopped afterward.
- No currency-formatting dependency was added. The title parser dependency and
  existing audit findings are recorded in item 16.
