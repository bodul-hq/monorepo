use super::*;

#[test]
fn every_currency_has_an_explicit_display_convention() {
    for (currency, amount, expected, accessible) in [
        (Currency::USD, 123456, "$1,234.56", "1234.56 USD"),
        (Currency::EUR, 123456, "1.234,56\u{a0}€", "1234.56 EUR"),
        (Currency::CAD, 123456, "CA$1,234.56", "1234.56 CAD"),
        (Currency::AUD, 123456, "A$1,234.56", "1234.56 AUD"),
        (Currency::GBP, 123456, "£1,234.56", "1234.56 GBP"),
        (Currency::JPY, 1234, "¥1,234", "1234 JPY"),
        (Currency::KRW, 1234, "₩1,234", "1234 KRW"),
        (Currency::HKD, 123456, "HK$1,234.56", "1234.56 HKD"),
        (
            Currency::PLN,
            123456,
            "1\u{a0}234,56\u{a0}zł",
            "1234.56 PLN",
        ),
        (
            Currency::SEK,
            123456,
            "1\u{a0}234,56\u{a0}kr",
            "1234.56 SEK",
        ),
        (Currency::VND, 1234, "1.234\u{a0}₫", "1234 VND"),
        (Currency::NZD, 123456, "NZ$1,234.56", "1234.56 NZD"),
        (Currency::INR, 12345678, "₹1,23,456.78", "123456.78 INR"),
    ] {
        let result = format_price(&Price {
            amount: amount.to_string(),
            currency,
        });
        assert_eq!(result.visible, expected, "{currency:?}");
        assert_eq!(result.accessible, accessible, "{currency:?}");
    }
}

#[test]
fn precision_signs_and_grouping_boundaries_are_exact() {
    for (currency, amount, expected) in [
        (Currency::USD, 0, "$0.00"),
        (Currency::EUR, 0, "0,00\u{a0}€"),
        (Currency::JPY, 0, "¥0"),
        (Currency::USD, 1, "$0.01"),
        (Currency::USD, -1, "−$0.01"),
        (Currency::EUR, -100, "−1,00\u{a0}€"),
        (Currency::JPY, -1, "−¥1"),
        (Currency::AUD, 1999, "A$19.99"),
        (Currency::USD, 99999, "$999.99"),
        (Currency::USD, 100000, "$1,000.00"),
        (Currency::USD, 100000000, "$1,000,000.00"),
        (Currency::INR, 99999, "₹999.99"),
        (Currency::INR, 100000, "₹1,000.00"),
        (Currency::INR, 9999999, "₹99,999.99"),
        (Currency::INR, 10000000, "₹1,00,000.00"),
        (Currency::INR, 1234567890, "₹1,23,45,678.90"),
        (Currency::JPY, 999, "¥999"),
        (Currency::JPY, 1000, "¥1,000"),
        (Currency::VND, -1000, "−1.000\u{a0}₫"),
    ] {
        assert_eq!(
            format_price(&Price {
                amount: amount.to_string(),
                currency
            })
            .visible,
            expected
        );
    }
}

#[test]
fn full_signed_range_preserves_every_minor_unit() {
    for (currency, amount, visible, accessible) in [
        (
            Currency::EUR,
            i64::MIN,
            "−92.233.720.368.547.758,08\u{a0}€",
            "−92233720368547758.08 EUR",
        ),
        (
            Currency::USD,
            i64::MAX,
            "$92,233,720,368,547,758.07",
            "92233720368547758.07 USD",
        ),
        (
            Currency::JPY,
            i64::MIN,
            "−¥9,223,372,036,854,775,808",
            "−9223372036854775808 JPY",
        ),
        (
            Currency::KRW,
            i64::MAX,
            "₩9,223,372,036,854,775,807",
            "9223372036854775807 KRW",
        ),
        (
            Currency::INR,
            i64::MIN,
            "−₹92,23,37,20,36,85,47,758.08",
            "−92233720368547758.08 INR",
        ),
    ] {
        let result = format_price(&Price {
            amount: amount.to_string(),
            currency,
        });
        assert_eq!(result.visible, visible);
        assert_eq!(result.accessible, accessible);
    }
}
