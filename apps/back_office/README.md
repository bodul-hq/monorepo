# Back Office

Poem/Tokio server-rendered home page using development's light/dark style.
Sign in at /login with username bodul and password bodul.
The account has full access. POST /logout revokes its session.

## Setup
Provide PostgreSQL 16+ and a dedicated database. Run make .env and configure
DATABASE_URL, then PORT=3008 make serve. Shell values override .env.
Startup applies migrations; missing or unreachable databases stop startup.
PORT defaults to 3000. BACK_OFFICE_COOKIE_SECURE defaults to false for local HTTP;
set it to true when using HTTPS. Invalid settings stop startup.

Sessions persist across restarts for eight hours after login. Cookies hold random
opaque IDs; identity and CSRF tokens stay in PostgreSQL. Login rotates the ID and
logout revokes it. Expired sessions are rejected during lookup and removed hourly.
Temporary credentials must be replaced before public deployment.

## Verification
Run make fmt, make ci and make -C .. audit (dependencies changed).
Open /login, sign in, restart the app, verify access survives, then log out.
Check both themes at desktop and narrow mobile widths.
Portable tests cover login, CSRF, authorization, rotation and logout.

Run DATABASE_URL_TEST=postgres://localhost/back_office_test make test-postgres
to check persistence across new connections, expiry, and revocation using an
isolated temporary schema. The test database user needs CREATE SCHEMA privileges.
