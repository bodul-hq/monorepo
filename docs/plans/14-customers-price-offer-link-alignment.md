# Item 14: Price and offer action share a row

Status: completed and verified on 2026-10-26.

## Goal

Place the price at the left and “View offer” at the right of the same card footer.

## Implementation

- Preserve the footer’s bottom alignment beneath variable-length titles.
- Use a wrapping flex row with baseline alignment and a gap between price and action.
- Remove the price’s bottom margin and the action’s full-width divider/top spacing.
- Keep the arrow beside the link, with hover and keyboard-focus states.
- Place “Store link unavailable” in the right-side action position.
- Permit extreme prices to wrap without truncation or overflow. If the action moves to another line, keep it right-aligned.

## Acceptance

Ordinary prices and links share a row on desktop and mobile. Long titles, long retailer codes, unavailable links and extreme supported prices remain readable at 320px, without horizontal page overflow.

## Scope and verification

Customers app HTML and CSS only; no API, database, dependency or JavaScript changes. Implement items 12–14 in order and verify their combined layout. Extend existing rendering checks for changed link/accessibility behavior; use browser checks for CSS layout. Run `make fmt` then `make ci` in `apps/customers`, and `PORT=<port> make serve` against a local fixture. Check `/`, keyboard navigation, and desktop/tablet/390px/320px widths in light and dark themes. Update the app README and mark the TODO complete only after acceptance checks pass.

## Completion evidence

- Implemented the HTML/CSS changes and updated the app README; no dependencies or APIs changed.
- `make fmt` and `make ci` in `apps/customers` passed (15 tests). Log: `/tmp/customer-cards-ci.log`.
- `CUSTOMER_FACING_URL=http://127.0.0.1:3160 PORT=3161 make serve` returned HTTP 200 using a local HTTP fixture.
- Firefox verified actual 1440px, 800px, 390px and 320px viewports in light/dark themes: loaded/missing/broken images, grey translucent badges, long retailer codes, extreme prices, unavailable links, right-aligned actions and no horizontal overflow.
- Browser checks verified visible keyboard focus, a new tab with `window.opener === null`, and the original page retaining `?offset=30`. The app and fixture were stopped afterward.
- Browser log: `/tmp/customer-cards-live.log`; screenshots: `/tmp/customer-cards-{light,dark}-{1440,800,390,320}.png`. Desktop dark and 320px light screenshots were visually reviewed.
