//! Reserved retailer integration workspace. Shared contracts live in retailer_sauce.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_wip");
    }
}
