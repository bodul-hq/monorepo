use poem::{Route, Server, get, handler, listener::TcpListener, web::Html};

#[handler]
fn index() -> Html<&'static str> {
    Html(
        "<!DOCTYPE html><html lang=\"en\"><head><title>Development</title></head><body><h1>Development</h1></body></html>",
    )
}

fn app() -> Route {
    Route::new().at("/", get(index))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = match std::env::var("PORT") {
        Ok(value) => value.parse::<u16>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        })?,
        Err(std::env::VarError::NotPresent) => 3000,
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
    };
    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(), shutdown_signal()?, None)
        .await
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    Ok(async move {
        #[cfg(unix)]
        tokio::select! {
            Ok(()) = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
        #[cfg(not(unix))]
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    })
}

#[cfg(test)]
mod tests {
    use poem::test::TestClient;

    use super::app;

    #[tokio::test]
    async fn index_renders_page() {
        let response = TestClient::new(app()).get("/").send().await;
        response.assert_status_is_ok();
        let body = response.0.into_body().into_string().await.unwrap();
        assert!(body.contains("<h1>Development</h1>"));
    }
}
