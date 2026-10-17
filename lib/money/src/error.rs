//! Typed error model (TS001 §2.7).
//!
//! The full `ParseError` enumeration is defined in Phase 1 even though the
//! sign/parentheses variants only become reachable once Phase 2 adds negative
//! amounts (TS001 §1.4). `FormatError` is Phase 2 and is intentionally omitted.

use std::error::Error;
use std::fmt;

/// Errors from [`crate::Money::parse`] (TS001 §2.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Input empty after trimming.
    EmptyInput,
    /// Input exceeds the parser's maximum length before normalization.
    InputTooLong,
    /// Disallowed character after separators handled.
    InvalidCharacter,
    /// Conflicting/duplicate sign markers.
    MalformedSign,
    /// Multiple distinct currency indicator tokens.
    MalformedCurrency,
    /// Embedded currency contradicts the expected currency.
    CurrencyMismatch,
    /// Separator present but no digits where required, or no digit at all after
    /// sign/currency extraction.
    MalformedNumber,
    /// Group sizes violate the 3-digit rule, or unsupported grouping.
    InvalidGrouping,
    /// More than `currency.exponent()` fractional digits with rounding disabled.
    TooManyFractionalDigits,
    /// Amount exceeds 64-bit range.
    Overflow,
}

/// Errors from monetary operations such as [`crate::Money::from_major`]
/// (TS001 §2.7). The arithmetic-only variants become reachable in Phase 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Semantic arithmetic, comparison, min, or max received mixed currencies.
    CurrencyMismatch,
    /// Arithmetic result (or an out-of-range operand) exceeds 64-bit range.
    Overflow,
    /// `mul_ratio` was called with `den == 0` and `num != 0` (Phase 2).
    DivisionByZero,
    /// Argument violates an operation precondition (e.g. bad `from_major` operands).
    InvalidArgument,
}

/// Errors from [`crate::Money::deserialize`] (TS001 §2.7, §2.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeError {
    /// `currency` is not a supported ISO 4217 alpha code.
    UnknownCurrency,
    /// `amount_minor` string is not a canonical base-10 signed integer.
    InvalidAmountMinor,
    /// `amount_minor` is a valid integer but outside the signed 64-bit range.
    AmountOutOfRange,
    /// Wire value is structurally wrong (e.g. number-form `amount_minor`, missing field).
    MalformedWireValue,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(phase-1): human-readable messages per variant.
        write!(f, "{self:?}")
    }
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(phase-1): human-readable messages per variant.
        write!(f, "{self:?}")
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(phase-1): human-readable messages per variant.
        write!(f, "{self:?}")
    }
}

impl Error for ParseError {}
impl Error for MoneyError {}
impl Error for DeserializeError {}
