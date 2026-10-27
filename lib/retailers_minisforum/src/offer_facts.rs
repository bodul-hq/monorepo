//! Rules shared by this family's offer structuralizers.
use money::Money;
pub(crate) use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, structuralized::content::Content,
};

/// A source comparison price is meaningful only against the same variant's current price.
pub(crate) fn was_price(current: Option<&Money>, comparison: Option<&Money>) -> Option<Money> {
    let current = current?;
    comparison
        .filter(|price| price.currency() == current.currency() && price.amount() > current.amount())
        .cloned()
}

pub(crate) fn purchase_available(available: bool) -> Availability {
    if available {
        Availability::Available
    } else {
        Availability::Unavailable
    }
}

pub(crate) fn content(description: Option<&str>) -> Content {
    Content {
        description: description.map(str::to_owned),
    }
}

pub(crate) fn in_stock(available: bool) -> Availability {
    if available {
        Availability::InStock
    } else {
        Availability::OutOfStock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use money::Currency;

    #[test]
    fn comparison_requires_a_current_price_and_a_higher_amount_in_the_same_currency() {
        let current = Money::new(100, Currency::EUR);
        let higher = Money::new(150, Currency::EUR);
        let lower = Money::new(50, Currency::EUR);
        let other_currency = Money::new(150, Currency::USD);
        assert_eq!(
            was_price(Some(&current), Some(&higher)),
            Some(higher.clone())
        );
        for comparison in [&current, &lower, &other_currency] {
            assert_eq!(was_price(Some(&current), Some(comparison)), None);
        }
        assert_eq!(was_price(None, Some(&higher)), None);
        assert_eq!(was_price(Some(&current), None), None);
    }

    #[test]
    fn description_preserves_source_markup_and_normalizes_only_absence() {
        assert_eq!(
            content(Some(" <p>A &amp; B</p> ")).description.as_deref(),
            Some(" <p>A &amp; B</p> ")
        );
        assert_eq!(content(Some(" \n\t")).description.as_deref(), Some(" \n\t"));
        assert_eq!(content(Some("")).description.as_deref(), Some(""));
        assert_eq!(content(None).description, None);
    }
}
