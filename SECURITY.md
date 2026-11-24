# Security review — Smart Trash RPC backend

Date: 2026-11-19 · Scope: this workspace (`crates/`, `proto/`, `apps/`) running
locally on Windows with PostgreSQL 18.

Everything below was reproduced against the running binaries before the fix and
re-tested afterwards. Commands are PowerShell-friendly; the HTTP surface is
`http://127.0.0.1:8080`, the gRPC surface `http://127.0.0.1:50051`.

## 1. What was run

```powershell
# baseline
.\build.ps1 cargo build --workspace --all-targets     # 0 rustc warnings
.\build.ps1 cargo test  --workspace                   # 20 tests, all green

# after the changes
.\build.ps1 cargo clippy --workspace --all-targets    # 0 warnings
.\build.ps1 cargo test   --workspace                  # 63 tests, all green
```

Runtime checks: the authorization matrix in §4, hostile-input probes in §5,
a direct gRPC call with and without a valid token in §3, plus the two
prototypes loaded through the gateway.

## 2. Findings

| # | Severity | Finding | Status |
| --- | --- | --- | --- |
| 1 | Critical | API gateway had no authentication at all | Fixed |
| 2 | Critical | gRPC services accepted any caller, no credential needed | Fixed |
| 3 | Critical | Broken access control: impersonation, cross-tenant reads, open state changes | Fixed |
| 4 | High | Stored XSS in all dashboards via `innerHTML` | Fixed |
| 5 | Medium | Database errors (SQLSTATE text) returned to clients | Fixed |
| 6 | Medium | `?page_size=` unbounded — full-table dump | Fixed |
| 7 | Medium | One panic poisoned a mutex and killed the service for good | Fixed |
| 8 | Medium | Diesel queries ran on Tokio runtime threads (stall under load) | Fixed |
| 9 | Medium | No rate limiting on login or the API | Fixed |
| 10 | Low | No security headers / CSP | Fixed |
| 11 | Low | Weak input validation (NaN, `\0`, control chars, oversized ids) | Fixed |
| 12 | Low | Unbounded in-memory maps (tracking positions) and route sizes | Fixed |
| 13 | Low | No graceful shutdown: Ctrl+C left connections cut | Fixed |
| — | Info | **No SQL injection found** — see §5 | Verified |
| — | Info | No TLS; `.env` holds local dev credentials | Documented |

## 3. Critical: nobody was authenticated

**Before.** `crates/api-gateway/src/main.rs` had no auth middleware, no token
handling and no identity forwarding; `crates/services/src/main.rs` added the
services to the Tonic server without an interceptor. The permission table in
`rpc-common::authentication` was dead code on the request path.

Evidence:

```text
$ curl.exe -s -w "%{http_code}" "http://127.0.0.1:8080/api/pickups?page_size=3"
[{"pickup_id":"PICKUP-00001",...,"driver_id":"DRIVER-11",...}] 200

$ curl.exe -s -X POST .../api/pickups -d '{"customer_id":"HOUSEHOLD-0421",...}'
{"pickup_id":"PICKUP-00004",...} 201

$ .\target\debug\scheduling-client.exe        # gRPC, zero credentials
created: PICKUP-00005 status=Pending
```

**After.**

* `rpc-common::auth_token` — `user_id.role.expiry.<hmac-sha256>`, signed with
  `AUTH_TOKEN_SECRET` (minimum 32 chars, fail-fast at startup), verified with
  `Mac::verify_slice` (constant time), expiry enforced, unknown roles refused.
* Gateway — `auth::require_bearer` middleware protects every `/api/…` route
  except `POST /api/auth/login`; the token is forwarded unchanged to gRPC.
* Services — `InterceptorLayer` (`services/src/auth.rs`) verifies the token
  once per call and attaches `Caller` to the request extensions. Handlers call
  `Caller::require_action` and refuse calls with no identity
  (`Status::unauthenticated`).
* Missing/weak `AUTH_TOKEN_SECRET` stops both binaries at startup instead of
  falling back to a default.

Verified:

```text
GET  /api/pickups                    (no token) -> 401 missing authorization bearer token
GET  /api/pickups   (role tampered in token)    -> 401 invalid or expired session
GET  /api/pickups   (random string as token)    -> 401 invalid or expired session
scheduling-client with another AUTH_TOKEN_SECRET -> Unauthenticated: invalid or expired credentials
```

### The Tauri client and the gRPC surface

The app signs in with `AuthService.Login` — the one RPC served without a token,
because issuing tokens is its job — and then calls the services over gRPC-Web
with that token. That moves the browser-facing surface from `:8081` to
`:50051`, so:

* the RPC port needs TLS and a proxy-level rate limit before it leaves the
  machine; the gateway's limiter does not cover it;
* reflection stays unauthenticated (listing only, no data);
* CORS on the RPC port is limited to the Tauri and Vite origins.

## 4. Critical: broken access control (IDOR / impersonation)

**Before.** `customer_id` came from the request body, `ListPickups` returned
every row to every caller, `GetPickup` had no ownership check, and
`AssignPickup`/`CompletePickup` were reachable by anyone.

**After.** Three independent checks:

1. **Action policy** — `require_action` decides which of the four categories
   may call which RPC (unknown actions fail closed). Unknown additions:
   `pickup.complete`, `window.get`, `route.get`, `fleet.view`, `identity.read`,
   `invoice.get`.
2. **Record ownership** — `Scope::of(caller)` filters the SQL query:
   household/business → `customer_id = caller`, driver → `driver_id = caller`,
   council → everything. `GetPickup`/`CompletePickup` compare the row against
   the caller after loading it.
3. **Gateway checks** — create-for-someone-else and report-GPS-for-someone-else
   are refused before the RPC is even dialled.

Verified matrix (all against the running stack):

| Actor | Action | Result |
| --- | --- | --- |
| Household | `GET /api/pickups` | 200, only `customer_id = HOUSEHOLD-0421` |
| Household | `POST /api/pickups` for `BUSINESS-12` | 403 customer_id must match the signed-in user |
| Household | `POST /api/pickups/{id}/assign` | 403 role Household may not perform 'pickup.assign' |
| Household | `GET /api/stats` | 403 role Household may not perform 'dashboard.view' |
| Council | `GET /api/pickups` | 200, all customers |
| Council | `POST /api/pickups/{id}/assign` | 200 |
| Council | `POST /api/tracking/locations` | 403 role Council may not perform 'tracking.report' |
| Driver | `GET /api/pickups` | 200, only rows assigned to `DRIVER-07` |
| Driver | GPS ping for `DRIVER-07` | 202 |
| Driver | GPS ping for `DRIVER-11` | 403 driver_id must match the signed-in driver |
| Business | `GET /api/pickups/PICKUP-00001` (household's) | 403 this pickup belongs to someone else |
| Business | `POST /api/pickups` for itself | 201 |

## 5. SQL injection: none present (verified, then hardened)

Every query uses Diesel's typed builder (`pickups::table.find(id)`,
`.filter(pickups::status.eq(value))`, `.values(&NewPickup{…})`), so values are
bound as `$1, $2, …` — there is no string-interpolated SQL and no
`diesel::sql_query` call anywhere in the workspace.

Probes against the live stack (before the fix, with a valid DB connection):

```text
GET /api/pickups/PICKUP-00001' OR '1'='1        -> 404 Pickup 'PICKUP-00001' OR '1'='1' not found
GET /api/pickups/1;DROP TABLE pickups;--        -> 404 Pickup '1;DROP TABLE pickups;--' not found
GET /api/pickups?status=pending'--              -> 400 unknown status 'pending'--
```

The payloads were treated as literal strings; nothing was executed. One probe
did surface a *different* problem: `PICKUP-00001%00` produced
`500 database error: invalid byte sequence for encoding "UTF8": 0x00` — a raw
PostgreSQL message (finding 5).

After the fix the same probes are rejected before the database is touched
(`rpc_common::validation::identifier`: `[A-Za-z0-9_-]{1,64}`):

```text
GET /api/pickups/PICKUP-00001' OR '1'='1        -> 400 pickup_id may only contain letters, digits, '_' and '-'
GET /api/pickups/PICKUP-00001%00                -> 400 (no database error in the body)
GET /api/pickups?page_size=999999999            -> 200, capped rows, still scoped to the caller
```

## 6. gRPC-specific issues

| Issue | Before | After |
| --- | --- | --- |
| Authentication | none — any local process could call every RPC | `InterceptorLayer` verifies the bearer token on every unary and streaming call |
| Error detail | `Status::internal("database error: {sqlstate…}")` | full error logged, client gets `database error` |
| Message size | crate default (4 MiB) | 1 MiB per message, explicit on every service |
| Keepalive | none | TCP keepalive 60 s, HTTP/2 keepalive 120 s / 20 s timeout |
| Flow control abuse | unbounded `page_size`, unbounded stop list, unbounded position map | 200 rows, 100 stops, 10 000 vehicles |
| Panic = outage | `expect("… lock poisoned")` turned one panic into a permanently broken service | `rpc_common::lock` recovers the guard; panics are logged, not fatal |
| Blocking work | `Mutex<PgConnection>` held on a Tokio worker thread | queries run in `spawn_blocking` |

### Reflection

`smarttrash-server` now serves the reflection API (v1 and v1alpha), which makes
the contracts explorable with `grpcurl` or Postman. It is deliberately outside
the auth interceptor: listing and describing need no token, because it only
reveals the API shape. Every business service still requires one — verified:

```text
grpcurl -plaintext 127.0.0.1:50051 list
  -> 8 smarttrash.* services + grpc.reflection.v1.ServerReflection

grpcurl -plaintext -d @ 127.0.0.1:50051 smarttrash.pickup.PickupService/ListPickups
  -> Code: Unauthenticated, Message: missing authorization bearer token

grpcurl -plaintext -H "authorization: Bearer <token>" ...
  -> the caller's own pickups
```

Set `GRPC_REFLECTION=false` to remove the reflection endpoint (and the API
discovery it allows) from a production build.

TLS is intentionally absent: both listeners bind to `127.0.0.1`. Put a TLS
terminator in front of the gateway before it leaves the machine, and do not
expose `:50051`.

## 7. Stored XSS in the prototypes (removed)

`customer_id` was accepted verbatim and inserted with `innerHTML` in the
household, business, driver, contractor and council pages. A payload such as
`<img src=x onerror=alert(document.domain)>` was stored and then executed in the
council dashboard (confirmed: `POST /api/pickups` returned 201 with the payload
as `customer_id`).

Fixes:

* every server value is now passed through `esc()` before it reaches
  `innerHTML`; plain text goes through `textContent`;
* `validation::identifier` refuses `<`, `>`, quotes and control characters on
  write, so new rows cannot carry markup at all;
* a CSP (`script-src 'self' 'unsafe-inline'`) plus `nosniff`, `X-Frame-Options:
  DENY` and `Referrer-Policy: no-referrer` is applied to every response.

Verified: `esc('<img src=x onerror=alert(1)>')` →
`&lt;img src=x onerror=alert(1)&gt;`, which renders as text.

## 8. Other fixes

* **Error bodies** are uniform plain text; upstream/internal details stay in
  the logs.
* **Rate limiting** — fixed window, per IP, `RATE_LIMIT_PER_MINUTE` (default
  600, `0` disables). Buckets are capped at 10 000 entries.
* **Request size** — 64 KiB JSON body limit.
* **Timeouts** — 15 s per gateway → service call.
* **Panics** — `CatchPanicLayer` turns a handler panic into `500`.
* **Shutdown** — both binaries handle Ctrl+C and drain connections, so the old
  process no longer lingers on the port.
* **Warnings** — removed the last unused import (`database` tests); `cargo
  clippy --workspace --all-targets` is clean.

## 9. Residual risks / next steps

1. **Demo sign-in is not real authentication.** `AUTH_DEMO_LOGIN=true` issues a
   token for any of the four seed users without a password. It is off unless
   set, warned about at startup, and must be replaced by a credential check
   (password hash / OIDC) before anything is reachable from a network.
2. **No token revocation list.** Tokens live for `AUTH_TOKEN_TTL_SECONDS`;
   rotating `AUTH_TOKEN_SECRET` is the only mass invalidation.
3. **No TLS.** Add a reverse proxy with certificates; keep `:50051` private.
4. **Single DB connection behind a mutex.** Fine for the prototype, but it
   serialises every query; move to `r2d2`/`deadpool` when load grows.
5. **Other services are still in-memory** (fleet, payment, compliance,
   scheduling, tracking). Restarting loses their state; identity is seeded, not
   stored.
6. **Dependency audit not automated.** `cargo audit` is not installed in this
   environment; wire it into CI (`cargo install cargo-audit && cargo audit`).
7. **Legacy rows.** Data written before this change may still contain markup in
   `customer_id`; the UI now escapes it, but a cleanup query is worthwhile:
   `DELETE FROM pickups WHERE customer_id !~ '^[A-Za-z0-9_-]{1,64}$';`

## 10. Re-running the verification

```powershell
# one command, once both processes are running
powershell -ExecutionPolicy Bypass -File .\scripts\verify-security.ps1
```

The script checks 38 things (public routes, 401s, the four role rules, customer
scoping, tampered tokens, injection probes, response headers) and exits with
code 1 if any of them fails. The manual steps below do the same by hand.

```powershell
# 1. build + tests
.\build.ps1 cargo test --workspace

# 2. start both processes (two terminals, project root)
.\target\debug\smarttrash-server.exe
.\target\debug\api-gateway.exe

# 3. no credentials -> 401
curl.exe -s -o NUL -w "%{http_code}`n" http://127.0.0.1:8080/api/pickups

# 4. sign in, then read with the token
$login = Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/auth/login `
  -ContentType 'application/json' -Body '{"user_id":"HOUSEHOLD-0421"}'
Invoke-RestMethod http://127.0.0.1:8080/api/pickups -Headers @{ Authorization = "Bearer $($login.token)" }
```
