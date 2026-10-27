# Item 12: Customers offer links open a new tab

Status: completed and verified on 2026-10-26.

## Goal

Open valid “View offer” links in another tab while keeping the current offer list and pagination offset available.

## Implementation

- Add `target="_blank"` and `rel="noopener"` to outbound offer links.
- Extend the screen-reader description with “opens in a new tab.”
- Retain URL validation and escaping; unsafe URLs display the unavailable state.
- Keep pagination, home and retry navigation in the current tab. Titles and images stay non-clickable.

## Acceptance

Check the rendered link attributes and accessible description, unsafe URLs, and actual browser tab creation. The new tab has no opener; the original page retains its URL and offset.

## Scope and verification

Customers app HTML and CSS only; no API, database, dependency or JavaScript changes. Implement items 12–14 in order and verify their combined layout. Extend existing rendering checks for changed link/accessibility behavior; use browser checks for CSS layout. Run `make fmt` then `make ci` in `apps/customers`, and `PORT=<port> make serve` against a local fixture. Check `/`, keyboard navigation, and desktop/tablet/390px/320px widths in light and dark themes. Update the app README and mark the TODO complete only after acceptance checks pass.

## Completion evidence

- Implemented the HTML/CSS changes and updated the app README; no dependencies or APIs changed.
- `make fmt` and `make ci` in `apps/customers` passed (15 tests). Log: `/tmp/customer-cards-ci.log`.
- `CUSTOMER_FACING_URL=http://127.0.0.1:3160 PORT=3161 make serve` returned HTTP 200 using a local HTTP fixture.
- Firefox verified actual 1440px, 800px, 390px and 320px viewports in light/dark themes: loaded/missing/broken images, grey translucent badges, long retailer codes, extreme prices, unavailable links, right-aligned actions and no horizontal overflow.
- Browser checks verified visible keyboard focus, a new tab with `window.opener === null`, and the original page retaining `?offset=30`. The app and fixture were stopped afterward.
- Browser log: `/tmp/customer-cards-live.log`; screenshots: `/tmp/customer-cards-{light,dark}-{1440,800,390,320}.png`. Desktop dark and 320px light screenshots were visually reviewed.
