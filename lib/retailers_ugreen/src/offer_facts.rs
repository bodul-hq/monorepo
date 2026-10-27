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

/// Explicit list/strikethrough specifications are comparison prices; sale prices are not.
/// Conflicting comparison amounts do not produce a single canonical value.
pub(crate) fn comparison_price<'a>(
    current: &Money,
    direct: Option<&'a Money>,
    specifications: impl Iterator<Item = (&'a str, &'a Money)>,
) -> Option<Money> {
    let mut comparisons = direct
        .into_iter()
        .chain(specifications.filter_map(|(kind, price)| {
            matches!(
                kind,
                "https://schema.org/ListPrice"
                    | "http://schema.org/ListPrice"
                    | "https://schema.org/StrikethroughPrice"
                    | "http://schema.org/StrikethroughPrice"
            )
            .then_some(price)
        }))
        .filter(|comparison| {
            comparison.currency() == current.currency() && comparison.amount() > current.amount()
        });
    let first = comparisons.next()?;
    comparisons
        .all(|comparison| comparison == first)
        .then(|| first.clone())
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
    #[test]
    fn only_unambiguous_source_comparisons_are_used() {
        let current = Money::new(100, Currency::EUR);
        let higher = Money::new(150, Currency::EUR);
        let conflicting = Money::new(200, Currency::EUR);
        assert_eq!(
            comparison_price(
                &current,
                None,
                [("https://schema.org/ListPrice", &higher)].into_iter()
            ),
            Some(higher.clone())
        );
        assert_eq!(
            comparison_price(
                &current,
                Some(&higher),
                [("http://schema.org/StrikethroughPrice", &higher)].into_iter()
            ),
            Some(higher.clone())
        );
        assert_eq!(
            comparison_price(
                &current,
                None,
                [("https://schema.org/SalePrice", &higher)].into_iter()
            ),
            None
        );
        assert_eq!(
            comparison_price(
                &current,
                Some(&higher),
                [("https://schema.org/ListPrice", &conflicting)].into_iter()
            ),
            None
        );
    }
}
