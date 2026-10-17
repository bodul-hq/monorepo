//! Supported currencies (TS001 §2.2).

/// The currencies supported by this library. Most have a minor-unit exponent of
/// `2` (100 minor units = 1 major unit); `JPY`, `KRW` and `VND` are zero-decimal
/// and their minor unit *is* the major one (TS001 §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Currency {
    /// US Dollar (ISO 840).
    USD,
    /// Euro (ISO 978).
    EUR,
    /// Canadian Dollar (ISO 124).
    CAD,
    /// Australian Dollar (ISO 036).
    AUD,
    /// Pound Sterling (ISO 826).
    GBP,
    /// Japanese Yen (ISO 392).
    JPY,
    /// South Korean Won (ISO 410).
    KRW,
    /// Hong Kong Dollar (ISO 344).
    HKD,
    /// Polish Zloty (ISO 985).
    PLN,
    /// Swedish Krona (ISO 752).
    SEK,
    /// Vietnamese Dong (ISO 704).
    VND,
    /// New Zealand Dollar (ISO 554).
    NZD,
    /// Indian Rupee (ISO 356), with 100 paise per rupee.
    INR,
}

impl Currency {
    /// Number of fractional (minor-unit) digits for this currency — the source of
    /// truth for fraction handling in parsing and formatting (TS001 §2.1/§2.2).
    /// `0` for the zero-decimal `JPY`, `KRW` and `VND`; `2` for every other.
    pub fn exponent(self) -> u8 {
        match self {
            Currency::JPY | Currency::KRW | Currency::VND => 0,
            Currency::USD
            | Currency::EUR
            | Currency::CAD
            | Currency::AUD
            | Currency::GBP
            | Currency::HKD
            | Currency::PLN
            | Currency::SEK
            | Currency::NZD
            | Currency::INR => 2,
        }
    }

    /// The ISO 4217 alpha code for a [`Currency`].
    pub fn as_str(self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::KRW => "KRW",
            Currency::HKD => "HKD",
            Currency::PLN => "PLN",
            Currency::SEK => "SEK",
            Currency::VND => "VND",
            Currency::NZD => "NZD",
            Currency::INR => "INR",
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    /// The [`Currency`] for an ISO 4217 alpha code.
    fn from_str(code: &str) -> Result<Currency, String> {
        match code {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "CAD" => Ok(Currency::CAD),
            "AUD" => Ok(Currency::AUD),
            "GBP" => Ok(Currency::GBP),
            "JPY" => Ok(Currency::JPY),
            "KRW" => Ok(Currency::KRW),
            "HKD" => Ok(Currency::HKD),
            "PLN" => Ok(Currency::PLN),
            "SEK" => Ok(Currency::SEK),
            "VND" => Ok(Currency::VND),
            "NZD" => Ok(Currency::NZD),
            "INR" => Ok(Currency::INR),

            other => Err(format!("unknown currency: {other:?}")),
        }
    }
}
