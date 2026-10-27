# Item 13: Retailer code over the offer image

Status: completed and verified on 2026-10-26.

## Goal

Display the retailer code at the image’s top-left with a semi-transparent grey background, following the user’s updated preference.

## Implementation

- Move the escaped retailer label into the image container, once per card.
- Position it 12px from the top and left, preserving uppercase styling and adding a contrasting text shadow.
- Use a grey `rgba(128,128,128,.35)` badge with 6px vertical / 8px horizontal padding and 4px corner radius in both themes. Keep the image undimmed.
- Bound and wrap long codes inside the image area.
- Keep the label visible and accessible with missing, unsafe or broken images. The image container must not be `aria-hidden` when it holds the label.
- Preserve first-image selection, URL validation, escaping, alt text, lazy loading and the 4:3 image area.

## Acceptance

Check one escaped, accessible label per card and review loaded light/dark images, missing images, unsafe URLs and failed image loads. Long codes must fit without moving the title or changing image dimensions.

## Scope and verification

Customers app HTML and CSS only; no API, database, dependency or JavaScript changes. Implement items 12–14 in order and verify their combined layout. Extend existing rendering checks for changed link/accessibility behavior; use browser checks for CSS layout. Run `make fmt` then `make ci` in `apps/customers`, and `PORT=<port> make serve` against a local fixture. Check `/`, keyboard navigation, and desktop/tablet/390px/320px widths in light and dark themes. Update the app README and mark the TODO complete only after acceptance checks pass.

## Completion evidence

- Implemented the HTML/CSS changes and updated the app README; no dependencies or APIs changed.
- `make fmt` and `make ci` in `apps/customers` passed (15 tests). Log: `/tmp/customer-cards-ci.log`.
- `CUSTOMER_FACING_URL=http://127.0.0.1:3160 PORT=3161 make serve` returned HTTP 200 using a local HTTP fixture.
- Firefox verified actual 1440px, 800px, 390px and 320px viewports in light/dark themes: loaded/missing/broken images, grey translucent badges, long retailer codes, extreme prices, unavailable links, right-aligned actions and no horizontal overflow.
- Browser checks verified visible keyboard focus, a new tab with `window.opener === null`, and the original page retaining `?offset=30`. The app and fixture were stopped afterward.
- Browser log: `/tmp/customer-cards-live.log`; screenshots: `/tmp/customer-cards-{light,dark}-{1440,800,390,320}.png`. Desktop dark and 320px light screenshots were visually reviewed.
