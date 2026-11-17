mod dashboard;
mod layout;
use dashboard::Dashboard;
use dashboard::page as dashboard_page;
use layout::Page;
use layout::end;
use layout::start;
use poem::Response;
use poem::http::StatusCode;
use poem::web::Path;
use std::env::VarError as StdEnvVarError;
use std::env::var;
#[cfg(not(unix))]
use std::future::pending;
use std::io::Error;
use std::io::ErrorKind as StdIoErrorKind;
use std::io::Result as StdIoResult;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;

use poem::{Endpoint, EndpointExt};
use poem::{Route, Server, get, handler, listener::TcpListener, web::Html};
use std::sync::Arc;

#[handler]
fn index() -> Html<String> {
    let mut html = start("Development", Page::Home);
    html.push_str(r#"<main><div class="hero"><div><p class="eyebrow">DEVELOPER TOOLS</p><h1>Development<span class="title-dot">.</span></h1><p class="lede">Tools for inspecting the services in your local workspace.</p></div></div><div class="tools"><a class="service tool" href="/dashboard"><span class="service-number" aria-hidden="true">01</span><h2>Database dashboard</h2><p>Table sizes, estimated rows, and inbox, command, event and outbox status for each service.</p><span class="tool-link">Open dashboard <span aria-hidden="true">→</span></span></a></div></main>"#);
    end(&mut html);
    Html(html)
}

#[handler]
fn service_image(Path(name): Path<String>) -> Response {
    let svg = match name.as_str() {
        "retailer-sourcing.svg" => include_str!("../assets/retailer-sourcing.svg"),
        "products.svg" => include_str!("../assets/products.svg"),
        "training.svg" => include_str!("../assets/training.svg"),
        "offers.svg" => include_str!("../assets/offers.svg"),
        "inbox.svg" => include_str!("../assets/inbox.svg"),
        "commands.svg" => include_str!("../assets/commands.svg"),
        "events.svg" => include_str!("../assets/events.svg"),
        "outbox.svg" => include_str!("../assets/outbox.svg"),
        "customer-facing.svg" => include_str!("../assets/customer-facing.svg"),
        _ => {
            return Response::builder().status(StatusCode::NOT_FOUND).finish();
        }
    };
    Response::builder().content_type("image/svg+xml").body(svg)
}

fn app(state: Arc<Dashboard>) -> impl Endpoint {
    Route::new()
        .at("/", get(index))
        .at("/dashboard", get(dashboard_page))
        .at("/assets/:name", get(service_image))
        .data(state)
}

/// Parses the `PORT` environment variable as returned by `std::env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, StdEnvVarError>) -> StdIoResult<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            Error::new(
                StdIoErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(StdEnvVarError::NotPresent) => Ok(3000),
        Err(error) => Err(Error::new(StdIoErrorKind::InvalidInput, error)),
    }
}

#[tokio::main]
async fn main() -> StdIoResult<()> {
    let port = parse_port(var("PORT"))?;
    let state = Arc::new(Dashboard::from_env().map_err(Error::other)?);
    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(state), shutdown_signal()?, None)
        .await
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> StdIoResult<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut terminate = signal(SignalKind::terminate())?;
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
    use crate::dashboard::Dashboard as DashboardDashboard;
    use poem::http::StatusCode as PoemHttpStatusCode;
    use std::sync::Arc as StdSyncArc;

    use std::{env::VarError, io::ErrorKind};

    use poem::test::TestClient;

    use super::{app, parse_port};

    #[tokio::test]
    async fn index_renders_page() {
        let response = TestClient::new(app(StdSyncArc::new(
            DashboardDashboard::from_env().unwrap(),
        )))
        .get("/")
        .send()
        .await;
        response.assert_status_is_ok();
        let body = response.0.into_body().into_string().await.unwrap();
        assert!(body.contains("<h1>Development<span class=\"title-dot\">.</span></h1>"));
        assert!(body.contains(r#"<a class="active" href="/" aria-current="page">Home</a>"#));
        assert!(body.contains(r#"href="/dashboard""#));
        assert!(body.contains(r#"data-theme-choice="dark""#));
        assert!(body.contains("prefers-color-scheme:dark"));
    }

    #[tokio::test]
    async fn service_images_are_served_as_svg() {
        let client = TestClient::new(app(StdSyncArc::new(
            DashboardDashboard::from_env().unwrap(),
        )));
        for name in [
            "retailer-sourcing",
            "offers",
            "products",
            "training",
            "customer-facing",
            "inbox",
            "commands",
            "events",
            "outbox",
        ] {
            let response = client.get(format!("/assets/{name}.svg")).send().await;
            response.assert_status_is_ok();
            assert_eq!(response.0.headers()["content-type"], "image/svg+xml");
            let body = response.0.into_body().into_string().await.unwrap();
            assert!(body.starts_with("<svg"));
        }
        client
            .get("/assets/missing.svg")
            .send()
            .await
            .assert_status(PoemHttpStatusCode::NOT_FOUND);
    }

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
