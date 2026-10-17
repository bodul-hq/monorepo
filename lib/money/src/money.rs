//! The `Money` value type, constructors, and accessors (TS001 §2.1, §2.9).

use super::{Currency, MoneyError};

/// An exact monetary amount paired with its currency.
///
/// Stored as a signed 64-bit count of **minor units** (cents); no floating point
/// (TS001 §2.1, INV-2). Fields are private — values are created through the
/// constructors and inspected through the accessors, so callers cannot bypass
/// invariants by raw struct construction.
///
/// Equality, hashing, and ordering are total over the whole `(amount_minor,
/// currency)` value, so `Money(500, USD) != Money(500, EUR)` and `Money` is safe
/// as a hash key (TS001 §2.6, INV-3). `Display` is intentionally **not**
/// implemented; human-readable output must go through the Phase 2
/// `format(money, locale)` function.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    /// Construct a money value directly from a signed minor-unit count.
    ///
    /// Infallible: every `i64` paired with a supported currency is a valid
    /// `Money` (TS001 §2.9).
    pub fn new(amount: i64, currency: Currency) -> Money {
        Money { amount, currency }
    }

    /// Rounds rather than truncating: f64 puts most two-decimal prices just *under* their cent -
    /// `79.99 * 100.0` is `7998.999999999999` - so `as i64` on its own silently loses a cent. It
    /// did, on ten of the fifty `ankerau` documents.
    ///
    /// `amount` is in major units and is scaled by the currency's exponent, so a zero-decimal
    /// currency's amount is stored as given: `1_014_000.0` VND is `1_014_000` minor units.
    pub fn from_raw(amount: f64, currency: &str) -> Result<Money, MoneyError> {
        let curr: Currency = currency.parse().unwrap();
        let minor_units_per_major = 10_f64.powi(i32::from(curr.exponent()));
        let in_minor_units: i64 = (amount * minor_units_per_major).round() as i64;

        Ok(Money::new(in_minor_units, curr))
    }

    /// Combine whole major units and a signed fractional minor-unit component
    /// using `currency.exponent()`.
    ///
    /// `fractional_minor` must have magnitude less than `10^exponent`, and `units`
    /// and `fractional_minor` must share a sign unless either is zero; otherwise
    /// returns [`MoneyError::InvalidArgument`]. Overflow-checked. (TS001 §2.9;
    /// AC-A-18, AC-A-19.)
    pub fn from_major(
        units: i64,
        fractional_minor: i64,
        currency: Currency,
    ) -> Result<Money, MoneyError> {
        // TODO(phase-1): validate sign/magnitude, assemble minor units (overflow-checked).
        let _ = (units, fractional_minor, currency);
        todo!("Money::from_major — TS001 §2.9")
    }

    /// The signed minor-unit (cents) count.
    pub fn amount(&self) -> i64 {
        self.amount
    }

    /// The currency of this value.
    pub fn currency(&self) -> Currency {
        self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four prices that distinguish rounding from truncation on the `ankerau` corpus. The
    /// first three are the ones truncation got wrong; `229.99` multiplies to an exact `22999.0`
    /// and was right either way, so it pins that rounding did not break the cases that worked.
    #[test]
    fn a_price_just_under_its_cent_in_f64_still_lands_on_that_cent() {
        for (amount, expected) in [
            (79.99, 7999),
            (129.95, 12995),
            (16.99, 1699),
            (229.99, 22999),
        ] {
            let money = Money::from_raw(amount, "AUD").expect("AUD is a supported currency");

            assert_eq!(money.amount(), expected, "{amount}");
            assert_eq!(money.currency(), Currency::AUD);
        }
    }

    #[test]
    fn a_zero_decimal_amount_is_stored_as_given() {
        let money = Money::from_raw(1_014_000.0, "VND").expect("VND is a supported currency");

        assert_eq!(money.amount(), 1_014_000);
        assert_eq!(money.currency(), Currency::VND);
    }

    #[test]
    fn the_exponent_separates_zero_decimal_currencies_from_the_rest() {
        for currency in [Currency::JPY, Currency::KRW, Currency::VND] {
            assert_eq!(currency.exponent(), 0, "{currency:?}");
        }

        for currency in [Currency::USD, Currency::SEK, Currency::EUR, Currency::INR] {
            assert_eq!(currency.exponent(), 2, "{currency:?}");
        }
    }

    #[test]
    fn indian_rupees_round_trip_with_prices_in_paise() {
        let currency: Currency = "INR".parse().unwrap();
        assert_eq!(currency, Currency::INR);
        assert_eq!(currency.as_str(), "INR");
        let amount = Money::from_raw(1999.99, "INR").unwrap();
        assert_eq!(amount.amount(), 199999);
        assert_eq!(amount.currency(), Currency::INR);
    }
}
