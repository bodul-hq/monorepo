use crate::offers::Price;
use money::Currency;

pub(super) struct FormattedPrice {
    pub visible: String,
    pub accessible: String,
}

pub(super) fn format_price(price: &Price) -> FormattedPrice {
    // These are product conventions, independent of the retailer or browser locale.
    let (symbol, suffix, separator, decimal) = match price.currency {
        Currency::USD => ("$", false, ',', '.'),
        Currency::EUR => ("€", true, '.', ','),
        Currency::CAD => ("CA$", false, ',', '.'),
        Currency::AUD => ("A$", false, ',', '.'),
        Currency::GBP => ("£", false, ',', '.'),
        Currency::JPY => ("¥", false, ',', '.'),
        Currency::KRW => ("₩", false, ',', '.'),
        Currency::HKD => ("HK$", false, ',', '.'),
        Currency::PLN => ("zł", true, '\u{a0}', ','),
        Currency::SEK => ("kr", true, '\u{a0}', ','),
        Currency::VND => ("₫", true, '.', '.'),
        Currency::NZD => ("NZ$", false, ',', '.'),
        Currency::INR => ("₹", false, ',', '.'),
    };
    // The HTTP client has validated the canonical i64 representation.
    let amount = price.amount.parse::<i64>().expect("validated price amount");
    let magnitude = amount.unsigned_abs();
    let exponent = price.currency.exponent();
    let scale = 10_u64.pow(u32::from(exponent));
    let sign = if amount < 0 { "−" } else { "" };
    let whole = (magnitude / scale).to_string();
    let group_size = if price.currency == Currency::INR {
        2
    } else {
        3
    };
    let mut number = String::new();
    for (index, digit) in whole.chars().enumerate() {
        let remaining = whole.len() - index;
        if index > 0 && (remaining == 3 || (remaining > 3 && (remaining - 3) % group_size == 0)) {
            number.push(separator);
        }
        number.push(digit);
    }
    let mut accessible = format!("{sign}{whole}");
    if exponent > 0 {
        let fraction = format!(
            "{:0width$}",
            magnitude % scale,
            width = usize::from(exponent)
        );
        number.push(decimal);
        number.push_str(&fraction);
        accessible.push('.');
        accessible.push_str(&fraction);
    }
    accessible.push(' ');
    accessible.push_str(price.currency.as_str());
    let visible = if suffix {
        format!("{sign}{number}\u{a0}{symbol}")
    } else {
        format!("{sign}{symbol}{number}")
    };
    FormattedPrice {
        visible,
        accessible,
    }
}

#[cfg(test)]
mod tests;
