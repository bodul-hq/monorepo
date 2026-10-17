# money

Money handling.

- `Money`: an amount stored as an integer count of minor units, with its
  `Currency`. `Money::from_raw` builds one from a scraped float price, rounding
  to the nearest minor unit.
- `Currency`: the supported ISO 4217 currencies and their minor-unit exponents.
  INR uses two decimal places (paise), like the other currencies with hundredths.

`Money::parse` and `Money::from_major` are signatures only, and panic with
`todo!()`.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
