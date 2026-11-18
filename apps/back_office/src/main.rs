mod layout;
mod storage;
use layout::{Page, end, start};
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::session::{CookieConfig, ServerSession, Session, SessionStorage};
use poem::web::cookie::SameSite;
use poem::web::{Form, Html, Redirect};
use poem::{
    Endpoint, EndpointExt, IntoResponse, Request, Response, Route, Server, get, handler, post,
};
use rand::{RngExt, rng};
use serde::Deserialize;
use std::env::{VarError as StdEnvVarError, var};
#[cfg(not(unix))]
use std::future::pending;
use std::io::{Error, ErrorKind as StdIoErrorKind, Result as StdIoResult};
use std::time::Duration;
use storage::Storage;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tokio::task::spawn_blocking;
use tokio::{select, signal::ctrl_c};

fn token(session: &Session) -> String {
    if let Some(t) = session.get::<String>("csrf") {
        return t;
    }
    let t = rng()
        .random::<[u8; 32]>()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    session.set("csrf", &t);
    t
}
fn authenticated(session: &Session) -> bool {
    session.get::<String>("user").as_deref() == Some("bodul")
}
fn page(session: &Session, error: bool) -> Response {
    let mut html = start("Back Office", Page::Home);
    let csrf = token(session);
    html.push_str("<main><h1>Back Office</h1>");
    if authenticated(session) {
        html.push_str(&format!("<p>Signed in as bodul.</p><form method=\"post\" action=\"/logout\"><input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><button>Log out</button></form>"));
    } else {
        if error {
            html.push_str("<p role=\"alert\">Invalid username or password.</p>");
        }
        html.push_str(&format!("<form method=\"post\" action=\"/login\"><input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><p><label>Username <input name=\"username\" autocomplete=\"username\" required></label></p><p><label>Password <input type=\"password\" name=\"password\" autocomplete=\"current-password\" required></label></p><button>Sign in</button></form>"));
    }
    html.push_str("</main>");
    end(&mut html);
    Html(html)
        .with_status(if error {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::OK
        })
        .into_response()
}
#[handler]
fn login_page(session: &Session) -> Response {
    if authenticated(session) {
        Redirect::see_other("/").into_response()
    } else {
        page(session, false)
    }
}
#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
    csrf: Option<String>,
}
fn valid_csrf(session: &Session, csrf: Option<&str>) -> bool {
    csrf.is_some() && session.get::<String>("csrf").as_deref() == csrf
}
#[handler]
fn login(session: &Session, Form(form): Form<Login>) -> Response {
    if !valid_csrf(session, form.csrf.as_deref()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if form.username != "bodul" || form.password != "bodul" {
        return page(session, true);
    }
    session.clear();
    session.set("user", "bodul");
    token(session);
    session.renew();
    Redirect::see_other("/").into_response()
}
#[derive(Deserialize)]
struct Logout {
    csrf: Option<String>,
}
#[handler]
fn logout(session: &Session, Form(form): Form<Logout>) -> Response {
    if !valid_csrf(session, form.csrf.as_deref()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    session.purge();
    Redirect::see_other("/login").into_response()
}
#[handler]
fn home(session: &Session) -> Response {
    page(session, false)
}
fn app(store: impl SessionStorage + 'static, secure: bool) -> impl Endpoint {
    let protected = Route::new()
        .at("/", get(home))
        .at("/logout", post(logout))
        .around(|ep, req: Request| async move {
            if !authenticated(
                req.extensions()
                    .get::<Session>()
                    .expect("session middleware"),
            ) {
                return Ok(Redirect::see_other("/login").into_response());
            }
            ep.call(req).await
        });
    Route::new()
        .at("/login", get(login_page).post(login))
        .nest("/", protected)
        .with(ServerSession::new(
            CookieConfig::new()
                .name("back_office_session")
                .secure(secure)
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(Duration::from_secs(28800)),
            store,
        ))
        .around(|ep, req| async move {
            let mut response = match ep.call(req).await {
                Ok(r) => r,
                Err(e) => e.into_response(),
            };
            response
                .headers_mut()
                .insert("cache-control", "no-store".parse().unwrap());
            Ok(response)
        })
}
#[tokio::main]
async fn main() -> StdIoResult<()> {
    let port = parse_port(var("PORT"))?;
    let secure = match var("BACK_OFFICE_COOKIE_SECURE") {
        Err(StdEnvVarError::NotPresent) => false,
        Ok(v) if v == "false" => false,
        Ok(v) if v == "true" => true,
        _ => {
            return Err(Error::other(
                "BACK_OFFICE_COOKIE_SECURE must be true or false",
            ));
        }
    };
    let url = var("DATABASE_URL").map_err(|_| Error::other("DATABASE_URL is required"))?;
    if url.trim().is_empty() {
        return Err(Error::other("DATABASE_URL is required"));
    }
    let store = spawn_blocking(move || Storage::connect(&url))
        .await
        .map_err(Error::other)?
        .map_err(|_| Error::other("session database initialization failed"))?;
    store.cleanup().await.map_err(Error::other)?;
    let cleanup = store.clone();
    let task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            let _ = cleanup.cleanup().await;
        }
    });
    let result = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(store, secure), shutdown_signal()?, None)
        .await;
    task.abort();
    result
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
    use super::*;
    use poem::session::MemoryStorage;
    use poem::test::TestClient;

    #[tokio::test]
    async fn authentication_flow() {
        let client = TestClient::new(app(MemoryStorage::new(), false));
        client
            .get("/")
            .send()
            .await
            .assert_status(StatusCode::SEE_OTHER);
        client
            .post("/login")
            .form(&[("username", "bodul"), ("password", "bodul")])
            .send()
            .await
            .assert_status(StatusCode::FORBIDDEN);
        let response = client.get("/login").send().await;
        response.assert_status_is_ok();
        let cookie = response.0.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let body = response.0.into_body().into_string().await.unwrap();
        let csrf = body
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        client
            .post("/login")
            .header("cookie", &cookie)
            .form(&[("username", "bodul"), ("password", "wrong"), ("csrf", csrf)])
            .send()
            .await
            .assert_status(StatusCode::UNAUTHORIZED);
        let response = client
            .post("/login")
            .header("cookie", &cookie)
            .form(&[("username", "bodul"), ("password", "bodul"), ("csrf", csrf)])
            .send()
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        let authenticated_cookie = response.0.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .to_owned();
        assert!(authenticated_cookie.contains("HttpOnly"));
        assert!(authenticated_cookie.contains("SameSite=Lax"));
        let authenticated_cookie = authenticated_cookie.split(';').next().unwrap();
        assert_ne!(authenticated_cookie, cookie);
        client
            .get("/")
            .header("cookie", &cookie)
            .send()
            .await
            .assert_status(StatusCode::SEE_OTHER);
        let response = client
            .get("/")
            .header("cookie", authenticated_cookie)
            .send()
            .await;
        response.assert_status_is_ok();
        let body = response.0.into_body().into_string().await.unwrap();
        let csrf = body
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        client
            .post("/logout")
            .header("cookie", authenticated_cookie)
            .form(&[("csrf", "wrong")])
            .send()
            .await
            .assert_status(StatusCode::FORBIDDEN);
        client
            .post("/logout")
            .header("cookie", authenticated_cookie)
            .form(&[("csrf", csrf)])
            .send()
            .await
            .assert_status(StatusCode::SEE_OTHER);
        client
            .get("/")
            .header("cookie", authenticated_cookie)
            .send()
            .await
            .assert_status(StatusCode::SEE_OTHER);
    }
}
