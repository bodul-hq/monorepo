//! Parsing free-form amount strings into `Money` (TS001 §2.4, §2.5).

use super::{Currency, Money, ParseError};

/// Caller-selected rounding modes (TS001 §2.5).
///
/// The type is defined in Phase 1 because [`ParseOptions`] references it, but
/// rounding *behaviour* (rounding-enabled parsing, scaling) is Phase 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingMode {
    /// Ties away from zero.
    HalfUp,
    /// Ties to the nearest even minor unit (banker's rounding).
    HalfEven,
    /// Toward zero (truncate).
    Down,
    /// Away from zero.
    Up,
    /// Toward positive infinity.
    Ceiling,
    /// Toward negative infinity.
    Floor,
}

/// Options controlling [`Money::parse`]. Forward-extensible (TS001 §2.9).
///
/// `rounding == None` (the default) means reject inputs with more fractional
/// digits than the currency's exponent ([`ParseError::TooManyFractionalDigits`]),
/// so scraped data is never silently corrupted (TS001 §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ParseOptions {
    /// Rounding mode to apply to excess fractional digits, or `None` to reject
    /// them. Rounding behaviour is Phase 2.
    pub rounding: Option<RoundingMode>,
}

impl Money {
    /// Parse a free-form amount string given the expected `currency`.
    ///
    /// Implements the TS001 §2.4 parsing algorithm: length guard, version-stable
    /// fold map, sign and currency-indicator extraction, group/decimal-separator
    /// disambiguation, fraction handling driven by `currency.exponent()`, and
    /// exact i64 assembly. Returns a typed [`ParseError`] on any malformed input.
    ///
    /// Phase 1 delivers positive baseline parsing (AC-P baseline rows, AC-P-AMB,
    /// AC-P-NEG, AC-P-ZERO-3/4); negative amounts and rounding-enabled parsing are
    /// Phase 2.
    pub fn parse(
        raw: &str,
        currency: Currency,
        options: ParseOptions,
    ) -> Result<Money, ParseError> {
        // TODO(phase-1): implement the §2.4 parsing algorithm.
        let _ = (raw, currency, options);
        todo!("Money::parse — TS001 §2.4")
    }
}
