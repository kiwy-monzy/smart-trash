# Smart Trash Pickup â€” Rust RPC workspace

Modular Rust backend: **Axum** HTTP gateway + **Tonic/gRPC** service-to-service RPC,
with **PostgreSQL + Diesel** for storage (PostGIS, NATS and WebSocket layers
planned behind the seams below).

## The four user categories (report Â§7.2)

Everything in this repo â€” roles, permissions, prototypes â€” is stripped down
to exactly the four user types of the report. There is no "admin" or
"dispatcher" role; operations work belongs to contractors and councils.

| Category | What they need (report Â§7.2) | Role | Example user | Prototype |
| --- | --- | --- | --- | --- |
| Households | Easy request, known pickup time, reminders, live truck view, way to complain | `ROLE_HOUSEHOLD` | `HOUSEHOLD-0421` | `/household-app/` |
| Businesses, markets, institutions | Recurring pickups, larger volumes, collection records to prove they follow the law | `ROLE_BUSINESS` | `BUSINESS-12` | `/business-portal/` |
| Drivers / crews | Clear daily route on a basic phone, one-tap status updates, works with poor signal | `ROLE_DRIVER` | `DRIVER-07` | `/driver-app/` |
| Contractors and councils | Dashboard: coverage, missed stops, volumes by area, problem hotspots | `ROLE_COUNCIL` | `COUNCIL-01` | `/contractor-dashboard/` + `/council-dashboard/` |

The permission table lives in one place:
`crates/rpc-common/src/authentication.rs` â†’ `Caller::require_action`.

## Authentication and authorization

The gateway is the only public surface, and **every `/api/â€¦` route except
`POST /api/auth/login` requires a bearer token**. The same token is forwarded
to the RPC layer, so the RPC port cannot be driven without it either.

```
sign in  â†’  POST /api/auth/login  â†’  {"token": "â€¦", "role": "household", "expires_at_unix": â€¦}
request  â†’  GET /api/pickups      â†’  Authorization: Bearer <token>
```

How it fits together:

| Layer | What it decides | Where |
| --- | --- | --- |
| Token | signature (HMAC-SHA256), expiry, role | `crates/rpc-common/src/auth_token.rs` |
| Gateway | who the caller is, which action, which record | `crates/api-gateway/src/auth.rs` + handlers |
| Service | the same action check, plus row ownership | `crates/services/src/*.rs` |
* Tokens are stateless: `user_id.role.expiry.signature`, signed with
  `AUTH_TOKEN_SECRET`. Rotating the secret signs everybody out.
* Authorization is the `require_action` table; unknown actions fail closed.
* **Records are scoped per caller**: a household or business sees only its own
  pickups, a driver only the ones assigned to it, councils see everything.
  Creating a pickup for somebody else is refused unless you are a council.
* **Demo sign-in** (`AUTH_DEMO_LOGIN=true`) hands out a token for one of the
  four seed users without a password so the prototypes work. It is off when
  the variable is absent; replace it with a real credential check before
  exposing the gateway.

The gateway also enforces a 64 KiB body limit, a per-IP request budget
(`RATE_LIMIT_PER_MINUTE`), a 15 s upstream timeout, `nosniff`/`DENY`/CSP/
`no-referrer` headers, and turns handler panics into `500` instead of killing
the process. See [`SECURITY.md`](SECURITY.md) for the audit that drove this.

## Layout

```
smart-trash/
â”œâ”€â”€ Cargo.toml                      # workspace
â”œâ”€â”€ build.ps1                       # cargo wrapper (MSVC env + libpq, see Prerequisites)
â”œâ”€â”€ diesel.toml                     # diesel CLI config (optional; migrations auto-run)
â”œâ”€â”€ .env                            # all settings, including DATABASE_URL
â”œâ”€â”€ SECURITY.md                     # what was audited and how it is verified
â”œâ”€â”€ scripts/
â”‚   â””â”€â”€ verify-security.ps1         # auth / scoping / injection checks
â”œâ”€â”€ proto/                          # RPC contracts â€” independent of DB schema
â”‚   â”œâ”€â”€ identity.proto              #   users, the 4 roles, authorization
â”‚   â”œâ”€â”€ pickup.proto                #   collection requests + lifecycle
â”‚   â”œâ”€â”€ scheduling.proto            #   collection windows
â”‚   â”œâ”€â”€ routing.proto               #   route optimization
â”‚   â”œâ”€â”€ fleet.proto                 #   trucks and drivers
â”‚   â”œâ”€â”€ tracking.proto              #   live GPS (+ server streaming)
â”‚   â”œâ”€â”€ payment.proto               #   invoices and payments
â”‚   â””â”€â”€ compliance.proto            #   council statistics and reports
â”œâ”€â”€ crates/
â”‚   â”œâ”€â”€ proto/                      # smarttrash-proto: compiles every .proto once
â”‚   â”œâ”€â”€ rpc-common/                 # authentication (4 roles) Â· errors Â· telemetry Â· .env
â”‚   â”œâ”€â”€ shared-models/              # JSON DTOs shared by gateway + apps
â”‚   â”œâ”€â”€ database/                   # PostgreSQL connection, migrations, Diesel models
â”‚   â”‚   â””â”€â”€ migrations/0001_init/   #   plain SQL: the `pickups` table
â”‚   â”œâ”€â”€ notifications/              # Notifier trait (SMS/push/email/NATS)
â”‚   â”œâ”€â”€ services/                   # ALL RPC services: one process, one port
â”‚   â”‚   â”œâ”€â”€ src/pickup.rs           #   bin: smarttrash-server  â†’  :50051
â”‚   â”‚   â”œâ”€â”€ src/routing.rs          #   bin: scheduling-client (gRPC demo)
â”‚   â”‚   â”œâ”€â”€ src/fleet.rs            #   one .add_service(...) line each
â”‚   â”‚   â”œâ”€â”€ src/tracking.rs         #   in src/main.rs â€” gRPC routes by
â”‚   â”‚   â”œâ”€â”€ src/scheduling.rs       #   full method path
â”‚   â”‚   â”œâ”€â”€ src/identity.rs         #   e.g. /smarttrash.pickup.
â”‚   â”‚   â”œâ”€â”€ src/payment.rs          #        PickupService/CreatePickup
â”‚   â”‚   â””â”€â”€ src/compliance.rs       #   :50051
â”‚   â””â”€â”€ api-gateway/                # REST for tools/web, :8081 on this machine
â””â”€â”€ apps/
    â””â”€â”€ app/                        # Tauri client: TypeScript + gRPC-Web
        â”œâ”€â”€ src/                    # core/, views/, generated/
        â””â”€â”€ src-tauri/              # Rust shell, desktop + Android + iOS

tauri-app/                          # Vite + Tauri client (desktop, Android, iOS)
```

## Prerequisites

- Rust stable toolchain.
- **PostgreSQL** running locally (the project was set up with PostgreSQL 18,
  service name `postgresql-x64-18`), with a `smarttrash` database and the
  default `postgres`/`postgres` login. See **Database** below.
- No system `protoc` â€” a vendored copy is used (`protoc-bin-vendored`).
- **Windows/MSVC note:** if the build fails in `link.exe` with
  `extra operand ...`, another `link.exe` (e.g. `C:\msys64\usr\bin`) is
  shadowing the MSVC linker. Run cargo through `build.ps1`, which loads the
  Visual Studio environment first **and** points Diesel at the PostgreSQL
  libraries (`PQ_LIB_DIR` + `libpq.dll` on `PATH`):

  ```powershell
  .\build.ps1 cargo build
  .\build.ps1 cargo test
  ```

  Always use `.\build.ps1 cargo â€¦`, never plain `cargo build` â€” plain cargo
  cannot find `libpq.lib` and the wrong `link.exe`.

## Database (PostgreSQL + Diesel)

Storage is plain **Diesel 2** against PostgreSQL. Credentials and address
come from `.env`, never from shell variables:

```dotenv
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/smarttrash
```

How it works â€” three files in `crates/database` do everything:

1. `migrations/0001_init/up.sql` â€” plain SQL that creates the `pickups` table.
2. `src/lib.rs` â€” `establish_connection()` opens the connection from
   `DATABASE_URL`; `run_migrations()` applies any migration that has not run
   yet. Both are called at the very start of `main` in
   `crates/services/src/main.rs`, so **the tables are created automatically**
   the first time the server starts â€” no manual step.
3. `src/schema.rs` + `src/models.rs` â€” the table definition and the Rust
   structs (`PickupRow`, `NewPickup`) that map to it.

Setup once, on a new machine:

```powershell
# create the database (password for user `postgres` is `postgres`)
$env:PGPASSWORD = 'postgres'
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres -h 127.0.0.1 -c "CREATE DATABASE smarttrash;"
```

Check what is inside:

```powershell
$env:PGPASSWORD = 'postgres'
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres -h 127.0.0.1 -d smarttrash -c "\dt" -c "SELECT * FROM pickups;"
```

Only the **pickup** service reads/writes the database so far; the other
services still keep their simple in-memory stores (see *Where the seams
are*).

## Settings (`.env`)

All configuration lives in **`.env`** at the workspace root â€” no shell
variables needed. It is loaded automatically at startup
(`rpc_common::settings::load_dotenv`), searched from the working directory
upwards.

```dotenv
RUST_LOG=info                      # logging level
RPC_ADDR=127.0.0.1:50051           # smarttrash-server listen address (all 8 services)
RPC_URL=http://127.0.0.1:50051     # where gateway/clients dial it
GATEWAY_ADDR=127.0.0.1:8080        # HTTP gateway
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/smarttrash

AUTH_TOKEN_SECRET=â€¦                # HMAC key for session tokens, â‰¥ 32 chars (required)
AUTH_DEMO_LOGIN=true               # dev only: /api/auth/login for the 4 seed users
AUTH_TOKEN_TTL_SECONDS=28800       # session lifetime (capped at 30 days)
RATE_LIMIT_PER_MINUTE=600          # per-IP budget on /api/*, 0 disables
```

- Copy `.env.example` â†’ `.env` on a new machine (`.env` itself is
  git-ignored; only the example is committed).
- Real environment variables always override `.env`, so CI/production can
  set `RPC_ADDR` etc. without editing any file.
- Changes take effect on the next restart of the affected process.

## Build and run

The whole backend is **two processes**:

```powershell
.\build.ps1 cargo build

# Terminal 1 â€” ALL 8 RPC services, one process, ONE port (:50051)
cargo run -p smarttrash-services --bin smarttrash-server

# Terminal 2 â€” HTTP gateway (REST for tools, JSON index on /)
cargo run -p api-gateway             # HTTP :8081 here (8080 is reserved)
```

Single RPC endpoint for the whole backend. Overridable by editing `.env`:
`RPC_ADDR` (server listen address), `RPC_URL` (gateway/client target),
`GATEWAY_ADDR`, `DATABASE_URL`.

> **Windows port reservations.** Hyper-V/WinNAT sometimes reserves a range that
> contains 8080, and the gateway then exits with `os error 10013`. Check with
> `netsh interface ipv4 show excludedportrange protocol=tcp` and run elsewhere
> for that session:
> `$env:GATEWAY_ADDR='127.0.0.1:8081'; .\target\debug\api-gateway.exe`
> (then use the same address in the app's Server field).

On the first start `smarttrash-server` connects to PostgreSQL and runs the
migrations itself, printing:

```
INFO database: database migrations are up to date
Smart Trash RPC listening on http://127.0.0.1:50051 (all services)
```

Example of service-to-service RPC (no HTTP involved):

```powershell
cargo run -p smarttrash-services --bin scheduling-client
```

### gRPC reflection

`smarttrash-server` serves the reflection API (v1 and v1alpha), so gRPC tools
can list the services and messages without the `.proto` files:

```powershell
# one-off download of grpcurl
Invoke-WebRequest https://github.com/fullstorydev/grpcurl/releases/download/v1.9.3/grpcurl_1.9.3_windows_x86_64.zip -OutFile grpcurl.zip
Expand-Archive grpcurl.zip -DestinationPath tools

.\tools\grpcurl.exe -plaintext 127.0.0.1:50051 list
.\tools\grpcurl.exe -plaintext 127.0.0.1:50051 describe smarttrash.pickup.CreatePickupRequest
```

```text
smarttrash.compliance.ComplianceService
smarttrash.fleet.FleetService
smarttrash.identity.IdentityService
smarttrash.payment.PaymentService
smarttrash.pickup.PickupService
smarttrash.routing.RoutingService
smarttrash.scheduling.SchedulingService
smarttrash.tracking.TrackingService
```

Listing and describing need no token; **calling** a service still does:

```powershell
$login = Invoke-RestMethod -Method Post http://127.0.0.1:8081/api/auth/login `
  -ContentType 'application/json' -Body '{"user_id":"HOUSEHOLD-0421"}'
'{"page_size":3}' | Set-Content -Encoding ascii body.json
Get-Content body.json -Raw | .\tools\grpcurl.exe -plaintext `
  -H "authorization: Bearer $($login.token)" -d '@' `
  127.0.0.1:50051 smarttrash.pickup.PickupService/ListPickups
```

Set `GRPC_REFLECTION=false` in `.env` to turn the reflection API off.

## Try it

```powershell
# 1. Sign in (demo mode) and keep the token
$login = Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/auth/login `
  -ContentType 'application/json' -Body '{"user_id":"HOUSEHOLD-0421"}'
$headers = @{ Authorization = "Bearer $($login.token)" }

# 2. Create a pickup (the token decides who you are)
Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/pickups -Headers $headers `
  -ContentType 'application/json' `
  -Body '{"customer_id":"HOUSEHOLD-0421","waste_type":"recyclable","latitude":-6.7565,"longitude":39.2049,"estimated_volume":2.5}'

# It is saved in PostgreSQL â€” restart the server and it is still there:
#   SELECT * FROM pickups;

# 3. List / read â€” a household sees only its own rows
Invoke-RestMethod "http://127.0.0.1:8080/api/pickups?status=pending" -Headers $headers

# 4. Council actions need a council token
$council = Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/auth/login `
  -ContentType 'application/json' -Body '{"user_id":"COUNCIL-01"}'
$councilHeaders = @{ Authorization = "Bearer $($council.token)" }
Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/pickups/PICKUP-00001/assign `
  -Headers $councilHeaders -ContentType 'application/json' `
  -Body '{"vehicle_id":"VEH-001","driver_id":"DRIVER-07"}'
Invoke-RestMethod "http://127.0.0.1:8080/api/stats?zone=Kinondoni" -Headers $councilHeaders

# 5. GPS ping â€” driver token, driver_id must be the signed-in driver
$driver = Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/auth/login `
  -ContentType 'application/json' -Body '{"user_id":"DRIVER-07"}'
Invoke-RestMethod -Method Post http://127.0.0.1:8080/api/tracking/locations `
  -Headers @{ Authorization = "Bearer $($driver.token)" } -ContentType 'application/json' `
  -Body '{"vehicle_id":"VEH-001","driver_id":"DRIVER-07","latitude":-6.78,"longitude":39.21,"speed_mps":7.2}'

# No token? Every /api route answers 401.
curl.exe -s -o NUL -w "%{http_code}`n" http://127.0.0.1:8080/api/pickups
```

Or open <http://127.0.0.1:8081/> for a JSON index of the REST routes. The
client is the Tauri app in [`apps/app/`](apps/app); it signs in over gRPC and
sends the token on every call.

## Verifying the gateway

With both processes running:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-security.ps1
```

It checks the public routes, the 401s without a token, the role rules for all
four categories, per-customer scoping, tampered tokens, SQL-injection probes
and the response headers, and exits with code 1 if anything fails.

## Client app (Tauri)

[`apps/app/`](apps/app) is a Vite + **TypeScript** client wrapped in Tauri 2.
One codebase for desktop, Android and iOS: sign in as a demo user, list your
pickups, request a new collection, from a glass tab bar that sits at the bottom
on a phone and inline on a desktop.

It talks **gRPC-Web** straight to `smarttrash-server:50051` â€” no REST, no Tauri
`invoke` â€” and its types are generated from the same `.proto` files as the
backend (`npm run proto`), so both sides cannot drift.

```powershell
cd apps/app
npm install
npm run app          # one command: backend + UI build + app window
npm run probe        # login + list over gRPC-Web, headless
npm run desktop:dev  # dev window with hot reload
```

```powershell
npm run android:init     # once, needs Android SDK/NDK + JDK 17
npm run android:dev
```

```bash
npm run ios:init         # once, needs macOS + Xcode
npm run ios:dev
```

The app defaults to `http://127.0.0.1:50051` on desktop and
`http://10.0.2.2:50051` on the Android emulator; in development every call is
proxied by Vite instead (see `apps/app/vite.config.ts`). Sign-in uses
`AuthService.Login`, the only RPC served without a token. Prerequisites and the
two mobile settings (cleartext HTTP on Android, ATS on iOS) are in
[`apps/app/README.md`](apps/app/README.md).

### Home map (Dar es Salaam)

The home map is **not** raster tiles: it is the Dar es Salaam extract drawn
locally from GeoJSON, so the boundaries stay sharp at every zoom and the map
works without a tile server.

| Layer | Features | Source |
|---|---|---|
| `region.geojson` | 1 | OSM relation 7202037, region outline (dashed) |
| `districts.geojson` | 5 | unions of the ward polygons, one colour each |
| `wards.geojson` | 94 | 71 OSM + 23 OCHA COD-AB 2018 polygons |
| `ward_anchors.geojson` | 8 | wards with no published polygon, office point only |
| `rivers.geojson` | 747 | 213 rivers, 532 streams, 2 canals (Overpass) |
| `roads_major.geojson` | 1 973 | motorway / trunk / primary / secondary |
| `minor_roads.geojson` | 76 729 | residential / service / unclassified |
| `sites.geojson` | 11 | waste sites OSM maps as a polygon |
| `site_points.geojson` | 35 | waste sites OSM maps only as a node |

The five municipalities are Ilala, Kinondoni, Temeke, Ubungo and Kigamboni;
their boundaries are derived unions because OSM has no district relations for
Dar es Salaam. The 102 wards are the official NBS 2022 list. Ward outlines from
COD-AB are dashed, and the eight wards that only exist as a point show as
anchors, so a ward never claims a boundary it does not have.

Only 11 of the 46 sites are mapped as an area, and the biggest of those is
279 m2 - OSM has no polygon for the Mtoni landfill. The other 35 exist only as a
single node, so they are drawn as a marker instead of an invented outline.
`python scripts/check-sites-satellite.py` lays each polygon over Esri World
Imagery (`map-sites-satellite.png`) so the mapped shape can be eyeballed.

Minor streets are 76 729 lines - too many for one Leaflet path each, so
`core/minorRoads.ts` keeps them as arrays and strokes only the ones in view into
a single canvas after each move. They appear from zoom 10.

Regenerate the layers (credentials stay in the environment, never in the repo):

```powershell
$env:S3_ENDPOINT   = 's3.eu-west-3.idrivee2.com'
$env:S3_REGION     = 'eu-west-3'
$env:S3_ACCESS_KEY = '...'
$env:S3_SECRET_KEY = '...'
powershell -File .\scripts\fetch-map-data.ps1   # -> var/geo-raw (never bundled)
python scripts\fetch-rivers.py                  # -> var/geo-raw/rivers.geojson (Overpass)
python scripts\prepare-map.py                   # -> apps/app/public/geo/*.geojson
```

`fetch-map-data.ps1` signs its own SigV4 GETs (no AWS CLI needed).
`prepare-map.py` splits wards and sites into polygons and points, simplifies the
streets and rivers with Douglas-Peucker, rounds coordinates to five decimals and
writes `manifest.json`. The drawing code is
[`apps/app/src/core/basemap.ts`](apps/app/src/core/basemap.ts).

> The Areas sheet reads the same `wards.geojson`, so the list and the map can
> never disagree: 102 wards grouped by municipality, each one centres the map.
## Design rules

1. **Contracts â‰  schema.** `.proto` files describe what a service can do;
   PostgreSQL tables can change without breaking any client.
2. **No direct database access across services.** Scheduling reaches pickups
   only through `PickupService` (see `scheduling-client`).
3. **Gateway is the only public surface.** The RPC port stays on localhost /
   private network; the gateway maps gRPC codes â†’ HTTP statuses. Both layers
   verify the session token â€” the RPC port is a second trust boundary, not an
   open back door.
4. **Shared crates are libraries, not services.** `rpc-common`,
   `shared-models`, `database`, `notifications`.

## Where the seams are

| Now | Later |
| --- | --- |
| Pickup service on PostgreSQL (Diesel); other services in-memory | Move identity/routing/fleet/tracking/scheduling/payment/compliance onto the same `crates/database` connection |
| Plain `latitude`/`longitude` doubles | PostGIS columns for real distance queries |
| `LogNotifier` | SMS/push provider or NATS publisher |
| Nearest-neighbour routing fixture | Real pickup coordinates via Pickup RPC + routing engine |
| Seeded compliance statistics | Aggregation over Pickup/Tracking RPC |
| Static app prototypes | Tauri client (`apps/app`) on desktop, Android and iOS |

## Adding a service

All services live in `crates/services` â€” no new workspace crate needed:

1. Add `proto/<name>.proto` with `package smarttrash.<name>;`.
2. Add the file name to the list in `crates/proto/build.rs`.
3. Re-export it in `crates/proto/src/lib.rs` (`pub mod <name> { â€¦ }`).
4. Add `crates/services/src/<name>.rs` implementing the generated server
   trait (copy `fleet.rs` as a template) and export it in `src/lib.rs`.
5. Add one `spawn_service!(...)` line in `crates/services/src/main.rs`.
