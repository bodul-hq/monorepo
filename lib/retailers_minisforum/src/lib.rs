mod minisforumau;
mod minisforumca;
mod minisforumeu;
mod minisforumfr;
mod minisforumhk;
mod minisforumjp;
mod minisforumkr;
mod minisforumru;
mod minisforumuk;
mod minisforumus;

mod offer_facts;
mod registry;

pub mod prelude;

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_minisforum");
    }
}
