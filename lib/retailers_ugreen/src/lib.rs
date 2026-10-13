mod ugreenca;
mod ugreencom;
mod ugreende;
mod ugreeneu;
mod ugreenfr;
mod ugreenin;
mod ugreenjp;
mod ugreenkr;
mod ugreennas;
mod ugreennasau;
mod ugreennasca;
mod ugreennasde;
mod ugreennases;
mod ugreennaseu;
mod ugreennasfr;
mod ugreennasit;
mod ugreennasjp;
mod ugreennasuk;
mod ugreennl;
mod ugreenuk;
mod ugreenus;

mod registry;

pub mod prelude;

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_ugreen");
    }
}
