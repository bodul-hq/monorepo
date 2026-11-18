# 89. Back Office sessions

## Status
Accepted

## Date
2026-11-18

## Context
Back Office requires login and sessions that survive restarts.

## Decision
Use Poem/Tokio and app-owned development-style HTML/CSS. The temporary hardcoded
bodul account has full access. Use Poem server sessions backed by a dedicated
PostgreSQL database through Diesel, with embedded startup migrations.
Sessions expire eight hours after login; login rotates IDs, logout deletes them.
Validate session-bound CSRF tokens on both forms. Cookies are HttpOnly and
SameSite=Lax, with Secure configurable for HTTPS. Database failures fail closed.

## Consequences
PostgreSQL is required. Random opaque session IDs survive restarts without a
signing key because the server validates them against persistent records.
Replace temporary credentials before public deployment. Reporting remains in
development.
