use axum::serve;
use io::Error as IoError;
use io::ErrorKind;
use io::Result as IoResult;
use logging::init;
use std::env::var;
#[cfg(not(unix))]
use std::future::pending;
use tokio::main as tokio_main;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::select;
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::signal::unix::signal as unix_signal;
use tokio::task::spawn_blocking;
mod logging;
use object_storage::{app, store::Store};
use std::{env::VarError, io};

fn parse_port(value: Result<String, VarError>) -> IoResult<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            IoError::new(
                ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(VarError::NotPresent) => Ok(3000),
        Err(error) => Err(IoError::new(ErrorKind::InvalidInput, error)),
    }
}

#[tokio_main]
async fn main() -> IoResult<()> {
    let _logs = init();
    let port = parse_port(var("PORT"))?;
    let database_url = var("DATABASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| IoError::other("DATABASE_URL is required."))?;
    let store = spawn_blocking(move || Store::connect(&database_url))
        .await
        .map_err(IoError::other)?
        .map_err(IoError::other)?;
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    serve(listener, app(store))
        .with_graceful_shutdown(shutdown_signal()?)
        .await
}

fn shutdown_signal() -> IoResult<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut terminate = unix_signal(SignalKind::terminate())?;
    Ok(async move {
        #[cfg(unix)]
        select! {
            Ok(()) = ctrl_c() => {}
            _ = terminate.recv() => {}
        }
        #[cfg(not(unix))]
        if ctrl_c().await.is_err() {
            pending::<()>().await;
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
