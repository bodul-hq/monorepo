use development::offer_details_dump::{Version, run};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Version::V1).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
