use offers::OffersClient;
use page::index;
use page::search as page_search;
use std::env::VarError as EnvVarError;
use std::env::var as env_var;
#[cfg(not(unix))]
use std::future::pending;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Result as IoResult;
use tokio::main as tokio_main;
#[cfg(unix)]
use tokio::select;
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::signal::unix::signal as unix_signal;
mod offers;
mod page;

use poem::{EndpointExt, Route, Server, get, listener::TcpListener};
use std::sync::Arc;

fn app(client: Arc<OffersClient>) -> Route {
    Route::new()
        .at("/", get(index).data(client.clone()))
        .at("/search", get(page_search).data(client))
}

/// Parses the `PORT` environment variable as returned by `std::env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, EnvVarError>) -> IoResult<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(EnvVarError::NotPresent) => Ok(3000),
        Err(error) => Err(IoError::new(IoErrorKind::InvalidInput, error)),
    }
}

#[tokio_main]
async fn main() -> IoResult<()> {
    let port = parse_port(env_var("PORT"))?;
    let client = Arc::new(OffersClient::from_env()?);
    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(client), shutdown_signal()?, None)
        .await
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
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
    use std::{env::VarError, io::ErrorKind};

    use super::parse_port;

    #[test]
    fn port_defaults_when_unset() {
        assert_eq!(parse_port(Err(VarError::NotPresent)).unwrap(), 3000);
    }

    #[test]
    fn port_accepts_the_full_range() {
        for (value, port) in [("0", 0), ("80", 80), ("3001", 3001), ("65535", 65535)] {
            assert_eq!(parse_port(Ok(value.to_owned())).unwrap(), port, "{value:?}");
        }
    }

    #[test]
    fn port_rejects_invalid_values() {
        // Non-numeric, out of range, padded, and non-ASCII Unicode digits.
        for value in [
            "",
            "abc",
            "80a",
            "3000.0",
            "-1",
            "65536",
            "99999999999",
            " 3000",
            "3000\n",
            "３０００",
            "٣٠٠٠",
        ] {
            let error = parse_port(Ok(value.to_owned())).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidInput, "{value:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn port_rejects_non_unicode_values() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let value = OsString::from_vec(vec![0xff]);
        let error = parse_port(Err(VarError::NotUnicode(value))).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }
}
