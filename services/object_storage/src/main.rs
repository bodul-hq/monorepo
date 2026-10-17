mod logging;
use object_storage::{app, store::Store};
use std::{env::VarError, io};

fn parse_port(value: Result<String, VarError>) -> io::Result<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(VarError::NotPresent) => Ok(3000),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let _logs = logging::init();
    let port = parse_port(std::env::var("PORT"))?;
    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| io::Error::other("DATABASE_URL is required."))?;
    let store = tokio::task::spawn_blocking(move || Store::connect(&database_url))
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app(store))
        .with_graceful_shutdown(shutdown_signal()?)
        .await
}

fn shutdown_signal() -> io::Result<impl Future<Output = ()> + Send + 'static> {
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
    use super::*;
    #[test]
    fn validates_port() {
        assert_eq!(parse_port(Err(VarError::NotPresent)).unwrap(), 3000);
        for port in ["0", "3001", "65535"] {
            assert!(parse_port(Ok(port.into())).is_ok());
        }
        for port in ["", "abc", "65536", " 3000", "3000\n", "-1", "３０００"] {
            assert!(parse_port(Ok(port.into())).is_err());
        }
    }
}
